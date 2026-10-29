use super::*;
use std::io::{Read, Write};

const LIMIT_BYTES: u64 = 12 * 1024 * 1024 * 1024;
const LIMIT_FILES: usize = 100_000;

#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Backup {
    pub id: String,
    pub created_at: String,
    pub bytes: u64,
    pub files: usize,
}

pub(super) fn normal_file(path: &Path) -> AppResult<std::fs::Metadata> {
    let metadata = std::fs::symlink_metadata(path)?;
    #[cfg(windows)]
    {
        use std::os::windows::fs::MetadataExt;
        if metadata.file_attributes() & 0x400 != 0 {
            return Err(invalid(
                "Uma pasta contém um link ou ponto de montagem; os dados foram preservados.",
            ));
        }
    }
    if metadata.file_type().is_symlink() {
        return Err(invalid("Links não são aceitos nesta operação de servidor."));
    }
    Ok(metadata)
}

pub(super) fn tree_files(root: &Path, limit: u64) -> AppResult<Vec<(PathBuf, u64)>> {
    scan(root, limit, false)
}

fn scan(root: &Path, limit: u64, filtered: bool) -> AppResult<Vec<(PathBuf, u64)>> {
    normal_file(root)?;
    let mut pending = vec![root.to_path_buf()];
    let mut files = Vec::new();
    let mut total = 0u64;
    let mut directories = 0usize;
    while let Some(directory) = pending.pop() {
        directories += 1;
        if directories > LIMIT_FILES {
            return Err(invalid("A pasta contém diretórios demais."));
        }
        for entry in std::fs::read_dir(directory)? {
            let entry = entry?;
            let path = entry.path();
            if filtered
                && !included(
                    path.strip_prefix(root)
                        .map_err(|_| invalid("Caminho inválido."))?,
                )
            {
                continue;
            }
            let metadata = normal_file(&path)?;
            if metadata.is_dir() {
                pending.push(path);
            } else if metadata.is_file() {
                total = total
                    .checked_add(metadata.len())
                    .ok_or_else(|| invalid("Pasta muito grande."))?;
                if total > limit || files.len() >= LIMIT_FILES {
                    return Err(invalid("A operação excede o limite de arquivos ou espaço. Exporte os dados pela pasta do servidor."));
                }
                files.push((
                    path.strip_prefix(root)
                        .map_err(|_| invalid("Caminho inválido."))?
                        .to_path_buf(),
                    metadata.len(),
                ));
            } else {
                return Err(invalid("Arquivo especial não permitido."));
            }
        }
    }
    Ok(files)
}

fn included(path: &Path) -> bool {
    let name = path
        .components()
        .next()
        .map(|value| value.as_os_str().to_string_lossy().into_owned())
        .unwrap_or_default();
    !name.starts_with(".luxmc-")
        && ![
            "libraries",
            "logs",
            "crash-reports",
            "versions",
            "server.jar",
            "luxmc-backup.json",
            "luxmc-loader-installer.jar",
            "luxmc-loader-ready.json",
        ]
        .contains(&name.as_str())
        && !name.ends_with(".log")
        && !name.ends_with(".lck")
}

pub(super) fn remove_owned_tree(directory: &Path, target: &Path) -> AppResult<()> {
    let root = std::fs::canonicalize(directory)?;
    let resolved = std::fs::canonicalize(target)?;
    if resolved.parent() != Some(root.as_path())
        || !resolved
            .file_name()
            .is_some_and(|value| value.to_string_lossy().starts_with(".luxmc-"))
    {
        return Err(invalid(
            "A limpeza não pertence à pasta temporária deste servidor.",
        ));
    }
    tree_files(&resolved, LIMIT_BYTES)?;
    std::fs::remove_dir_all(resolved)?;
    Ok(())
}

fn export(directory: &Path, target: &Path, record: &Backup) -> AppResult<Backup> {
    let files = scan(directory, LIMIT_BYTES, true)?;
    let required = files
        .iter()
        .map(|(_, size)| *size)
        .sum::<u64>()
        .saturating_add(256 * 1024 * 1024);
    let disks = sysinfo::Disks::new_with_refreshed_list();
    if disks
        .list()
        .iter()
        .filter(|disk| target.starts_with(disk.mount_point()))
        .max_by_key(|disk| disk.mount_point().components().count())
        .is_some_and(|disk| disk.available_space() < required)
    {
        return Err(unavailable("Não há espaço livre suficiente para uma cópia consistente com reserva de 256 MiB. Exporte ou remova backups antigos e tente novamente."));
    }
    let mut record = record.clone();
    let file = std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(target)?;
    let mut archive = zip::ZipWriter::new(file);
    let options = zip::write::FileOptions::default()
        .compression_method(zip::CompressionMethod::Deflated)
        .compression_level(Some(1))
        .unix_permissions(0o600);
    for (relative, size) in files.into_iter().filter(|(path, _)| included(path)) {
        let name = relative
            .to_str()
            .ok_or_else(|| invalid("Um arquivo tem nome inválido."))?
            .replace('\\', "/");
        archive.start_file(name, options)?;
        let mut source = std::fs::File::open(directory.join(&relative))?;
        std::io::copy(&mut source, &mut archive)?;
        record.bytes += size;
        record.files += 1;
    }
    archive.start_file("luxmc-backup.json", options)?;
    archive.write_all(&serde_json::to_vec(&record)?)?;
    archive.finish()?.sync_all()?;
    Ok(record)
}

impl LocalServerManager {
    async fn backup_root(&self, id: &str) -> AppResult<PathBuf> {
        self.directory(id).await?;
        let root = self.root().await?.join("backups");
        tokio::fs::create_dir_all(&root).await?;
        normal_file(&root)?;
        let path = root.join(id);
        tokio::fs::create_dir_all(&path).await?;
        normal_file(&path)?;
        Ok(tokio::fs::canonicalize(path).await?)
    }

    pub async fn backups(&self, id: &str) -> AppResult<Vec<Backup>> {
        let root = self.backup_root(id).await?;
        tokio::task::spawn_blocking(move || -> AppResult<Vec<Backup>> {
            let mut result = Vec::new();
            for entry in std::fs::read_dir(root)? {
                let entry = entry?;
                let path = entry.path();
                normal_file(&path)?;
                if path.extension().and_then(|value| value.to_str()) != Some("zip") {
                    continue;
                }
                if result.len() >= 20 {
                    return Err(invalid(
                        "Há mais de 20 backups. Abra a pasta de dados para conferir.",
                    ));
                }
                let mut archive = zip::ZipArchive::new(std::fs::File::open(path)?)?;
                let mut bytes = Vec::new();
                archive
                    .by_name("luxmc-backup.json")?
                    .take(65537)
                    .read_to_end(&mut bytes)?;
                if bytes.len() > 65536 {
                    return Err(invalid("Metadados de backup inválidos."));
                }
                let record: Backup = serde_json::from_slice(&bytes)?;
                if entry.file_name().to_string_lossy() != format!("{}.zip", record.id)
                    || uuid::Uuid::parse_str(&record.id).is_err()
                {
                    return Err(invalid("Identificador de backup inválido."));
                }
                result.push(record);
            }
            result.sort_by(|a, b| b.created_at.cmp(&a.created_at));
            Ok(result)
        })
        .await
        .map_err(|error| AppError::Internal(error.to_string()))?
    }

    pub(super) async fn backup_unlocked(&self, id: &str) -> AppResult<Backup> {
        if self.current(id).is_ok() {
            return Err(unavailable(
                "Pare e salve o servidor antes de criar uma cópia consistente.",
            ));
        }
        let records = self.backups(id).await?;
        if records.len() >= 10 {
            return Err(unavailable("O limite de dez backups foi atingido. Exporte ou remova uma cópia antiga antes de continuar; nenhuma cópia foi apagada automaticamente."));
        }
        let directory = self.directory(id).await?;
        let root = self.backup_root(id).await?;
        let record = Backup {
            id: uuid::Uuid::new_v4().to_string(),
            created_at: chrono::Utc::now().to_rfc3339(),
            bytes: 0,
            files: 0,
        };
        let temporary = root.join(format!("{}.partial", record.id));
        let destination = root.join(format!("{}.zip", record.id));
        let partial = temporary.clone();
        let result = tokio::task::spawn_blocking(move || export(&directory, &partial, &record))
            .await
            .map_err(|error| AppError::Internal(error.to_string()))?;
        match result {
            Ok(record) => {
                tokio::fs::rename(temporary, destination).await?;
                Ok(record)
            }
            Err(error) => {
                let _ = tokio::fs::remove_file(temporary).await;
                Err(error)
            }
        }
    }

    pub async fn backup(&self, id: &str) -> AppResult<Backup> {
        let _guard = self.change()?;
        self.backup_unlocked(id).await
    }

    pub async fn export_backup(
        &self,
        id: &str,
        backup_id: &str,
        destination: PathBuf,
    ) -> AppResult<()> {
        let _guard = self.change()?;
        if !self
            .backups(id)
            .await?
            .iter()
            .any(|record| record.id == backup_id)
        {
            return Err(invalid("Backup não encontrado."));
        }
        if destination
            .extension()
            .and_then(|value| value.to_str())
            .is_none_or(|value| !value.eq_ignore_ascii_case("zip"))
        {
            return Err(invalid("Exporte o backup como arquivo .zip."));
        }
        let parent = tokio::fs::canonicalize(
            destination
                .parent()
                .ok_or_else(|| invalid("Pasta de exportação inválida."))?,
        )
        .await?;
        if parent.starts_with(self.root().await?) {
            return Err(invalid(
                "Escolha uma pasta fora dos dados gerenciados do servidor para exportar.",
            ));
        }
        let destination = parent.join(
            destination
                .file_name()
                .ok_or_else(|| invalid("Nome de exportação inválido."))?,
        );
        if destination.exists() {
            normal_file(&destination)?;
        }
        let source = self.backup_root(id).await?.join(format!("{backup_id}.zip"));
        normal_file(&source)?;
        let temporary = parent.join(format!(".luxmc-export-{}.part", uuid::Uuid::new_v4()));
        let result: AppResult<()> = async {
            let mut input = tokio::fs::File::open(source).await?;
            let mut output = tokio::fs::OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(&temporary)
                .await?;
            tokio::io::copy(&mut input, &mut output).await?;
            output.sync_all().await?;
            drop(output);
            tokio::fs::rename(&temporary, destination).await?;
            Ok(())
        }
        .await;
        if result.is_err() {
            let _ = tokio::fs::remove_file(temporary).await;
        }
        result
    }

    pub async fn remove_backup(&self, id: &str, backup_id: &str) -> AppResult<()> {
        let _guard = self.change()?;
        let records = self.backups(id).await?;
        if records.len() <= 1 {
            return Err(unavailable(
                "A última cópia de segurança é preservada. Crie outra antes de removê-la.",
            ));
        }
        if !records.iter().any(|record| record.id == backup_id) {
            return Err(invalid("Backup não encontrado."));
        }
        let path = self.backup_root(id).await?.join(format!("{backup_id}.zip"));
        normal_file(&path)?;
        tokio::fs::remove_file(path).await?;
        Ok(())
    }

    pub async fn restore_backup(&self, id: &str, backup_id: &str) -> AppResult<()> {
        let _guard = self.change()?;
        if self.current(id).is_ok() {
            return Err(unavailable(
                "Pare e salve o servidor antes de restaurar um backup.",
            ));
        }
        if !self
            .backups(id)
            .await?
            .iter()
            .any(|record| record.id == backup_id)
        {
            return Err(invalid("Backup não encontrado."));
        }
        let source = self.backup_root(id).await?.join(format!("{backup_id}.zip"));
        normal_file(&source)?;
        let directory = self.directory(id).await?;
        let staged = directory.join(".luxmc-restore");
        if staged.exists() {
            return Err(unavailable("Existe uma restauração anterior preservada. Confira .luxmc-restore na pasta do servidor."));
        }
        let extract = staged.clone();
        let expected = id.to_string();
        tokio::task::spawn_blocking(move || -> AppResult<()> {
            let mut zip = zip::ZipArchive::new(std::fs::File::open(source)?)?;
            if zip.len() > LIMIT_FILES {
                return Err(invalid("Backup com arquivos demais."));
            }
            let mut total = 0u64;
            for index in 0..zip.len() {
                let file = zip.by_index(index)?;
                total = total
                    .checked_add(file.size())
                    .ok_or_else(|| invalid("Backup muito grande."))?;
                if total > LIMIT_BYTES
                    || file.enclosed_name().is_none()
                    || file
                        .unix_mode()
                        .is_some_and(|mode| mode & 0o170000 == 0o120000)
                {
                    return Err(invalid("Backup contém caminho ou arquivo inválido."));
                }
            }
            std::fs::create_dir(&extract)?;
            for index in 0..zip.len() {
                let file = zip.by_index(index)?;
                let relative = file
                    .enclosed_name()
                    .ok_or_else(|| invalid("Caminho inválido."))?
                    .to_path_buf();
                if relative == Path::new("luxmc-backup.json") || !included(&relative) {
                    continue;
                }
                let path = extract.join(relative);
                if file.is_dir() {
                    std::fs::create_dir_all(path)?;
                } else {
                    std::fs::create_dir_all(
                        path.parent().ok_or_else(|| invalid("Caminho inválido."))?,
                    )?;
                    let mut output = std::fs::OpenOptions::new()
                        .write(true)
                        .create_new(true)
                        .open(path)?;
                    let expected_size = file.size();
                    let copied = std::io::copy(
                        &mut file.take(expected_size.saturating_add(1)),
                        &mut output,
                    )?;
                    if copied != expected_size {
                        return Err(invalid(
                            "O tamanho descompactado não corresponde ao arquivo declarado.",
                        ));
                    }
                }
            }
            let config: LocalServerConfig =
                serde_json::from_slice(&std::fs::read(extract.join("luxmc-server.json"))?)?;
            if config.id != expected {
                return Err(invalid("Este backup pertence a outro servidor."));
            }
            validate_config(&config)?;
            Ok(())
        })
        .await
        .map_err(|error| AppError::Internal(error.to_string()))??;
        self.backup_unlocked(id).await?;
        let rollback = directory.join(format!(".luxmc-before-restore-{}", uuid::Uuid::new_v4()));
        tokio::fs::create_dir(&rollback).await?;
        let mut moved = Vec::new();
        let mut installed = Vec::new();
        let result: AppResult<()> = async {
            let mut entries = tokio::fs::read_dir(&directory).await?;
            while let Some(entry) = entries.next_entry().await? {
                let name = entry.file_name();
                if name.to_string_lossy().starts_with(".luxmc-") || !included(Path::new(&name)) {
                    continue;
                }
                normal_file(&entry.path())?;
                tokio::fs::rename(entry.path(), rollback.join(&name)).await?;
                moved.push(name);
            }
            let mut entries = tokio::fs::read_dir(&staged).await?;
            while let Some(entry) = entries.next_entry().await? {
                let name = entry.file_name();
                tokio::fs::rename(entry.path(), directory.join(&name)).await?;
                installed.push(name);
            }
            Ok(())
        }
        .await;
        if let Err(error) = result {
            for name in installed.iter().rev() {
                let _ = tokio::fs::rename(directory.join(name), staged.join(name)).await;
            }
            for name in moved.iter().rev() {
                let _ = tokio::fs::rename(rollback.join(name), directory.join(name)).await;
            }
            return Err(error);
        }
        tokio::fs::remove_dir(&staged).await?;
        let marker = directory.join("luxmc-loader-ready.json");
        if marker.exists() {
            normal_file(&marker)?;
            tokio::fs::remove_file(marker).await?;
        }
        tokio::task::spawn_blocking(move || remove_owned_tree(&directory, &rollback))
            .await
            .map_err(|error| AppError::Internal(error.to_string()))??;
        Ok(())
    }
}
