use super::*;
use std::collections::HashSet;

impl LocalServerManager {
    pub async fn import_installed_files(&self, id: &str, source: PathBuf) -> AppResult<serde_json::Value> {
        let _guard = self.change()?;
        if self.current(id).is_ok() { return Err(unavailable("Pare e salve o servidor antes de importar uma instância.")); }
        if self.contents(id).await?.iter().any(|item| item.staged) { return Err(unavailable("Aplique ou cancele as pendências antes de importar a instância.")); }
        let source = tokio::fs::canonicalize(source).await?;
        let directory = self.directory(id).await?;
        if source.starts_with(&directory) || directory.starts_with(&source) { return Err(invalid("Origem da instância inválida.")); }
        crate::core::mods::pack_download::atomic_write(&directory.join(".luxmc-instance-pending"), b"Import and apply the instance contents before starting.\n").await?;
        let prepared = directory.join(format!(".luxmc-instance-copy-{}", uuid::Uuid::new_v4()));
        tokio::fs::create_dir(&prepared).await?;
        let output = prepared.clone();
        let result = tokio::task::spawn_blocking(move || copy(&source, &output)).await.map_err(|error| AppError::Internal(error.to_string()))?;
        let result = match result {
            Ok((count, excluded)) => {
                let staging = directory.join(".luxmc-staging");
                if staging.exists() { tokio::fs::remove_dir(&staging).await?; }
                tokio::fs::rename(&prepared, &staging).await?;
                let report = serde_json::json!({"prepared":count,"excluded":excluded,"localCopy":true});
                crate::core::mods::pack_download::atomic_write(&directory.join(".luxmc-instance-report.json"), &serde_json::to_vec(&report)?).await?;
                Ok(report)
            }
            Err(error) => Err(error),
        };
        if prepared.exists() { let root = directory; let _ = tokio::task::spawn_blocking(move || backups::remove_owned_tree(&root, &prepared)).await; }
        result
    }
}

fn copy(source: &Path, prepared: &Path) -> AppResult<(usize, Vec<String>)> {
    let mut client_only = HashSet::new();
    let index = source.join("modrinth.index.json");
    if index.is_file() && backups::normal_file(&index)?.len() <= 8 * 1024 * 1024 {
        let index: serde_json::Value = serde_json::from_slice(&std::fs::read(index)?)?;
        if let Some(files) = index["files"].as_array() {
            for file in files { if file.pointer("/env/server").and_then(|value| value.as_str()) == Some("unsupported") { if let Some(path) = file["path"].as_str() { client_only.insert(path.to_string()); } } }
        }
    }
    let mut stack = Vec::new();
    for prefix in content::PACK_FOLDERS {
        if prefix == &"world/datapacks/" { continue; }
        let folder = source.join(prefix.trim_end_matches('/'));
        if folder.exists() { backups::normal_file(&folder)?; stack.push(folder); }
    }
    let mut count = 0; let mut total = 0u64; let mut excluded = Vec::new();
    while let Some(directory) = stack.pop() {
        for entry in std::fs::read_dir(directory)? {
            let path = entry?.path(); let metadata = backups::normal_file(&path)?;
            if metadata.is_dir() { stack.push(path); if stack.len() > 4096 { return Err(invalid("Pastas demais na instância.")); } continue; }
            if !metadata.is_file() { return Err(invalid("A instância contém arquivo especial.")); }
            let relative = path.strip_prefix(source).map_err(|_| invalid("Caminho de instância inválido."))?.to_string_lossy().replace('\\', "/");
            content::pack_path(&relative)?;
            if client_only.contains(&relative) { excluded.push(relative); continue; }
            if relative.starts_with("mods/") {
                if !relative.ends_with(".jar") { continue; }
                match content::server_archive(&path,"mod") {
                    Ok(false) => { excluded.push(relative); continue; }
                    Ok(true) => {},
                    Err(error) => return Err(invalid(format!("{}: {error}",path.file_name().unwrap_or_default().to_string_lossy()))),
                }
            }
            total = total.checked_add(metadata.len()).ok_or_else(|| invalid("Instância muito grande."))?;
            if metadata.len() > 512 * 1024 * 1024 || total > 4 * 1024 * 1024 * 1024 || count >= 4096 { return Err(invalid("A cópia excede 4 GiB ou 4096 arquivos. Use um pacote de servidor do autor.")); }
            if std::fs::File::open(&path)?.metadata()?.len() != metadata.len() { return Err(unavailable("Um arquivo da instância mudou durante a cópia. Tente novamente.")); }
            let target = prepared.join(uuid::Uuid::new_v4().to_string()); std::fs::create_dir(&target)?;
            let copied = std::fs::copy(&path, target.join("payload"))?;
            if copied != metadata.len() { return Err(unavailable("Um arquivo da instância mudou durante a cópia. Tente novamente.")); }
            std::fs::write(target.join("pending.json"), serde_json::to_vec(&content::Pending { kind:"modpack".into(), name:relative, remove:false })?)?;
            count += 1;
        }
    }
    Ok((count, excluded))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    #[test]
    fn client_metadata_with_raw_control_characters_does_not_block_server_import() {
        let root = tempfile::tempdir().unwrap(); let source = root.path().join("client"); let target = root.path().join("pending");
        std::fs::create_dir_all(source.join("mods")).unwrap(); std::fs::create_dir(&target).unwrap();
        let mut zip = zip::ZipWriter::new(std::fs::File::create(source.join("mods/visual.jar")).unwrap());
        zip.start_file("fabric.mod.json", zip::write::FileOptions::default()).unwrap();
        zip.write_all(b"{\"id\":\"visual\",\"description\":\"line one\nline two\",\"environment\":\"client\"}").unwrap(); zip.finish().unwrap();
        let (count, excluded) = copy(&source, &target).unwrap();
        assert_eq!(count, 0); assert_eq!(excluded, vec!["mods/visual.jar"]);
    }
    #[test]
    #[ignore]
    fn inspect_installed_content_without_starting_minecraft() {
        let source = PathBuf::from(std::env::var_os("LUXMC_CONTENT_INSPECTION_SOURCE").expect("Choose an installed instance directory"));
        let temporary = tempfile::tempdir().unwrap();
        let (count, excluded) = copy(&source, temporary.path()).unwrap();
        println!("{}", serde_json::json!({"prepared":count,"excluded":excluded,"sourceUnmodified":true,"minecraftStarted":false}));
    }
    #[test]
    fn installed_bootstrap_with_client_manifest_is_excluded_without_executing_it() {
        let root = tempfile::tempdir().unwrap();
        let source = root.path().join("client"); let target = root.path().join("pending");
        std::fs::create_dir_all(source.join("mods")).unwrap(); std::fs::create_dir(&target).unwrap();
        let mut nested = zip::ZipWriter::new(std::io::Cursor::new(Vec::new()));
        nested.start_file("META-INF/MANIFEST.MF", zip::write::FileOptions::default()).unwrap();
        nested.write_all(b"Manifest-Version: 1.0\r\nModSide: CLIENT\r\n").unwrap();
        let nested = nested.finish().unwrap().into_inner();
        let mut outer = zip::ZipWriter::new(std::fs::File::create(source.join("mods/bootstrap.jar")).unwrap());
        outer.start_file("essential-loader.properties", zip::write::FileOptions::default()).unwrap();
        outer.write_all(b"pinnedFile=/embedded.jar\npublisherSlug=essential\n").unwrap();
        outer.start_file("embedded.jar", zip::write::FileOptions::default()).unwrap(); outer.write_all(&nested).unwrap(); outer.finish().unwrap();
        let (count, excluded) = copy(&source, &target).unwrap();
        assert_eq!(count, 0); assert_eq!(excluded, vec!["mods/bootstrap.jar"]);
        assert!(source.join("mods/bootstrap.jar").exists());
    }
    #[test]
    fn local_copy_splits_content_and_excludes_client_mods_and_game_options() {
        let root = tempfile::tempdir().unwrap(); let source = root.path().join("client"); let target = root.path().join("pending");
        std::fs::create_dir_all(source.join("mods")).unwrap(); std::fs::create_dir_all(source.join("config")).unwrap(); std::fs::create_dir(&target).unwrap();
        for (name, environment) in [("client.jar","client"),("server.jar","*")] {
            let mut zip = zip::ZipWriter::new(std::fs::File::create(source.join("mods").join(name)).unwrap());
            zip.start_file("fabric.mod.json", zip::write::FileOptions::default()).unwrap();
            write!(zip,"{{\"id\":\"fixture\",\"environment\":\"{environment}\"}}").unwrap(); zip.finish().unwrap();
        }
        std::fs::write(source.join("config/example.json"), b"{}").unwrap(); std::fs::write(source.join("options.txt"), b"client options").unwrap();
        let (count, excluded) = copy(&source,&target).unwrap(); assert_eq!(count,2); assert_eq!(excluded,vec!["mods/client.jar"]);
    }
}
