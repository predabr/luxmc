use super::*;
use crate::core::mods::pack_download;
use futures_util::StreamExt;
use std::collections::{BTreeMap, HashSet};
use std::io::Read;

struct Pack {
    index: serde_json::Value,
    overrides: BTreeMap<String, PathBuf>,
    excluded:HashSet<String>,
}

fn bundle_resource_folders(prepared: &mut BTreeMap<String, PathBuf>, transaction: &Path) -> AppResult<()> {
    use std::io::Write;
    let folders = prepared.keys().filter_map(|name| name.strip_prefix("resourcepacks/").and_then(|name| name.split_once('/')).filter(|(_, tail)| *tail == "pack.mcmeta").map(|(folder, _)| folder.to_string())).collect::<Vec<_>>();
    for folder in folders {
        let prefix = format!("resourcepacks/{folder}/");
        let name = format!("resourcepacks/{folder}.zip");
        if prepared.contains_key(&name) { return Err(invalid("O modpack contém uma pasta e um ZIP de texturas com o mesmo nome.")); }
        let output = transaction.join(format!(".luxmc-texture-{}.zip", uuid::Uuid::new_v4()));
        let mut zip = zip::ZipWriter::new(std::fs::File::create(&output)?);
        let entries = prepared.keys().filter(|name| name.starts_with(&prefix)).cloned().collect::<Vec<_>>();
        for entry in entries {
            let source = prepared.remove(&entry).ok_or_else(|| invalid("Arquivo de textura não encontrado."))?;
            zip.start_file(&entry[prefix.len()..], zip::write::FileOptions::default().compression_method(zip::CompressionMethod::Deflated))?;
            std::io::copy(&mut std::fs::File::open(source)?, &mut zip)?;
        }
        zip.flush()?;
        zip.finish()?;
        prepared.insert(name, output);
    }
    let datapacks = prepared.keys().filter(|name| name.starts_with("datapacks/")).cloned().collect::<Vec<_>>();
    for name in datapacks {
        let target = format!("world/{name}");
        if prepared.contains_key(&target) { return Err(invalid("O modpack contém datapacks duplicados.")); }
        let source = prepared.remove(&name).ok_or_else(|| invalid("Datapack não encontrado."))?;
        prepared.insert(target, source);
    }
    Ok(())
}

fn read_server_pack(path: &Path, prepared: &Path) -> AppResult<(usize, Vec<String>)> {
    let mut archive = zip::ZipArchive::new(std::fs::File::open(path)?)?;
    if archive.len() > 40_000 {
        return Err(invalid("O pacote de servidor tem arquivos demais."));
    }
    let mut names = HashSet::new();
    let mut total = 0u64;
    let mut mods = 0; let mut excluded = Vec::new();
    for index in 0..archive.len() {
        let file = archive.by_index(index)?;
        if file.enclosed_name().is_none()
            || file
                .unix_mode()
                .is_some_and(|mode| mode & 0o170000 == 0o120000)
        {
            return Err(invalid("Pacote de servidor contém caminho inseguro."));
        }
        if file.is_dir() {
            continue;
        }
        let raw = file.name().replace('\\', "/");
        let name = if content::PACK_FOLDERS
            .iter()
            .any(|prefix| raw.starts_with(prefix))
        {
            raw
        } else if let Some((_, suffix)) = raw.split_once('/') {
            if content::PACK_FOLDERS
                .iter()
                .any(|prefix| suffix.starts_with(prefix))
            {
                suffix.into()
            } else {
                continue;
            }
        } else {
            continue;
        };
        if !content::server_pack_entry(&name) {excluded.push(name);continue;}
        content::pack_path(&name)?;
        total = total
            .checked_add(file.size())
            .ok_or_else(|| invalid("Pacote muito grande."))?;
        if total > 4 * 1024 * 1024 * 1024 || file.size() > 512 * 1024 * 1024 || names.len() >= 4096
        {
            return Err(invalid(
                "O pacote de servidor excede o limite de arquivos ou espaço.",
            ));
        }
        if !names.insert(name.to_ascii_lowercase()) {
            return Err(invalid("Pacote de servidor contém arquivos duplicados."));
        }
        let entry = prepared.join(uuid::Uuid::new_v4().to_string());
        std::fs::create_dir(&entry)?;
        let payload = entry.join("payload");
        let mut output = std::fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&payload)?;
        let expected_size = file.size();
        let copied = std::io::copy(&mut file.take(expected_size.saturating_add(1)), &mut output)?;
        if copied != expected_size {
            return Err(invalid(
                "O tamanho descompactado não corresponde ao arquivo declarado.",
            ));
        }
        output.sync_all()?;
        drop(output);
        if name.starts_with("mods/") {
            if !content::server_archive(&payload,"mod")? {
                excluded.push(name); std::fs::remove_file(&payload)?; std::fs::remove_dir(&entry)?; continue;
            }
            mods += 1;
        }
        std::fs::write(
            entry.join("pending.json"),
            serde_json::to_vec(&content::Pending {
                kind: "modpack".into(),
                name,
                remove: false,
            })?,
        )?;
    }
    if mods == 0 {
        return Err(invalid("Este pacote não inclui os mods de servidor e pode depender de scripts externos. Nada será executado automaticamente; use o servidor preparado pelo autor e o compartilhamento de servidor externo."));
    }
    let entries = std::fs::read_dir(prepared)?.collect::<Result<Vec<_>,_>>()?;
    let mut files = BTreeMap::new();
    for entry in &entries {
        let record: content::Pending = serde_json::from_slice(&std::fs::read(entry.path().join("pending.json"))?)?;
        files.insert(record.name, entry.path().join("payload"));
    }
    bundle_resource_folders(&mut files, prepared)?;
    let prepared_count = files.len();
    for (name, source) in files {
        let entry = prepared.join(uuid::Uuid::new_v4().to_string());
        std::fs::create_dir(&entry)?;
        std::fs::rename(source, entry.join("payload"))?;
        std::fs::write(entry.join("pending.json"),serde_json::to_vec(&content::Pending{kind:"modpack".into(),name,remove:false})?)?;
    }
    for entry in entries {
        if entry.path().join("payload").exists() { std::fs::remove_file(entry.path().join("payload"))?; }
        std::fs::remove_file(entry.path().join("pending.json"))?;
        std::fs::remove_dir(entry.path())?;
    }
    Ok((prepared_count,excluded))
}

fn read_pack(path: &Path, extracted: &Path) -> AppResult<Pack> {
    std::fs::create_dir_all(extracted)?;
    let metadata = backups::normal_file(path)?;
    if !metadata.is_file() || metadata.len() > 512 * 1024 * 1024 {
        return Err(invalid("Escolha um modpack de até 512 MiB."));
    }
    let mut archive = zip::ZipArchive::new(std::fs::File::open(path)?)?;
    if archive.len() > 40_000 {
        return Err(invalid("O modpack tem arquivos demais."));
    }
    let mut bytes = Vec::new();
    archive
        .by_name("modrinth.index.json")?
        .take(8 * 1024 * 1024 + 1)
        .read_to_end(&mut bytes)?;
    if bytes.len() > 8 * 1024 * 1024 {
        return Err(invalid("Índice do modpack muito grande."));
    }
    let index: serde_json::Value = serde_json::from_slice(&bytes)?;
    if index["game"] != "minecraft" || index["formatVersion"] != 1 {
        return Err(invalid("Formato de modpack não suportado."));
    }
    let mut overrides = BTreeMap::new(); let mut excluded=HashSet::new();
    let mut size = 0u64;
    for prefix in ["overrides/", "server-overrides/"] {
        for position in 0..archive.len() {
            let file = archive.by_index(position)?;
            if file.enclosed_name().is_none()
                || file
                    .unix_mode()
                    .is_some_and(|mode| mode & 0o170000 == 0o120000)
            {
                return Err(invalid("O modpack contém um caminho inseguro."));
            }
            let Some(name) = file.name().strip_prefix(prefix).map(str::to_owned) else {
                continue;
            };
            if file.is_dir() {
                continue;
            }
            if !content::PACK_FOLDERS
                .iter()
                .any(|prefix| name.starts_with(prefix))
            {
                continue;
            }
            if !content::server_pack_entry(&name) {excluded.insert(name);continue;}
            content::pack_path(&name)?;
            size = size
                .checked_add(file.size())
                .ok_or_else(|| invalid("Modpack muito grande."))?;
            if size > 1024 * 1024 * 1024
                || overrides.len() >= 4096
                || file.size() > 128 * 1024 * 1024
            {
                return Err(invalid(
                    "As substituições do modpack excedem o limite de espaço ou arquivos.",
                ));
            }
            let payload = extracted.join(uuid::Uuid::new_v4().to_string());
            let mut output = std::fs::OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(&payload)?;
            let expected_size = file.size();
            let copied =
                std::io::copy(&mut file.take(expected_size.saturating_add(1)), &mut output)?;
            if copied != expected_size {
                return Err(invalid(
                    "O tamanho descompactado não corresponde ao arquivo declarado.",
                ));
            }
            drop(output);
            if let Some(previous) = overrides.insert(name, payload) {
                std::fs::remove_file(previous)?;
            }
        }
    }
    Ok(Pack { index, overrides, excluded })
}

impl LocalServerManager {
    pub async fn import_server_pack(
        &self,
        id: &str,
        path: PathBuf,
    ) -> AppResult<serde_json::Value> {
        let _guard = self.change()?;
        if self.contents(id).await?.iter().any(|item| item.staged) {
            return Err(unavailable(
                "Aplique ou cancele as pendências antes de importar outro pacote.",
            ));
        }
        let metadata = backups::normal_file(&path)?;
        if metadata.len() > 1024 * 1024 * 1024 {
            return Err(invalid("Pacote de servidor maior que 1 GiB."));
        }
        let directory = self.directory(id).await?;
        let prepared = directory.join(format!(".luxmc-server-pack-{}", uuid::Uuid::new_v4()));
        tokio::fs::create_dir(&prepared).await?;
        let extract = prepared.clone();
        let result: AppResult<serde_json::Value> = async {
            let (count, excluded) = tokio::task::spawn_blocking(move || read_server_pack(&path, &extract))
                .await
                .map_err(|error| AppError::Internal(error.to_string()))??;
            let staging = directory.join(".luxmc-staging");
            if staging.exists() {
                tokio::fs::remove_dir(&staging).await?;
            }
            tokio::fs::rename(&prepared, staging).await?;
            Ok(serde_json::json!({"prepared":count,"excluded":excluded.len(),"excludedFiles":excluded}))
        }
        .await;
        if prepared.exists() {
            let temporary = prepared.clone();
            let _ = tokio::task::spawn_blocking(move || {
                backups::remove_owned_tree(&directory, &temporary)
            })
            .await;
        }
        result
    }

    pub async fn import_modpack(&self, id: &str, path: PathBuf) -> AppResult<serde_json::Value> {
        let _guard = self.change()?;
        let directory = self.directory(id).await?;
        let transaction = directory.join(format!(".luxmc-pack-{}", uuid::Uuid::new_v4()));
        tokio::fs::create_dir(&transaction).await?;
        let overrides_root = transaction.join("overrides");
        let result = async {
        let pack = tokio::task::spawn_blocking(move || read_pack(&path, &overrides_root)).await.map_err(|error| AppError::Internal(error.to_string()))??;
        let config = self.config(id).await?; let loader = self.loader(id).await?;
        if pack.index.pointer("/dependencies/minecraft").and_then(|value| value.as_str()) != Some(config.mc_version.as_str()) { return Err(invalid("O modpack usa outra versão do Minecraft. Crie um servidor separado com a versão correta.")); }
        let dependencies = pack.index["dependencies"].as_object().ok_or_else(|| invalid("Modpack sem versões fixadas."))?;
        for (name, version) in dependencies {
            if name == "minecraft" { continue; }
            let kind = match name.as_str() { "fabric-loader" => "fabric", "quilt-loader" => "quilt", "forge" => "forge", "neoforge" => "neoforge", _ => return Err(invalid("O modpack exige um loader de servidor ainda não suportado.")) };
            if kind != loader.kind || version.as_str() != Some(loader.version.as_str()) { return Err(invalid(format!("Configure primeiro o servidor com {kind} {} na aba Configurações. O mundo e os arquivos atuais foram preservados.", version.as_str().unwrap_or_default()))); }
        }
        let files = pack.index["files"].as_array().ok_or_else(|| invalid("Modpack sem lista de arquivos."))?;
        if files.len() > 2048 { return Err(invalid("O modpack excede 2048 downloads.")); }
        let staging = directory.join(".luxmc-staging");
        if self.contents(id).await?.iter().any(|item| item.staged) { return Err(unavailable("Aplique ou revise as alterações pendentes antes de importar um modpack.")); }
        let mut prepared = BTreeMap::<String, PathBuf>::new(); let mut excluded = pack.excluded.clone(); let mut total = 0u64; let mut names = HashSet::new(); let mut jobs = Vec::new(); let mut projects = BTreeMap::<String,String>::new();
        for file in files {
            let name = file["path"].as_str().ok_or_else(|| invalid("Arquivo do modpack sem caminho."))?;
            pack_download::relative_path(name)?;
            if !content::PACK_FOLDERS.iter().any(|prefix| name.starts_with(prefix)) || !content::server_pack_entry(name) || (name.starts_with("mods/") && matches!(file.pointer("/env/server").and_then(|value| value.as_str()), Some("unsupported" | "optional"))) { excluded.insert(name.to_owned()); continue; }
            content::pack_path(name)?;
            if !names.insert(name.to_ascii_lowercase()) { return Err(invalid("Modpack com caminho duplicado.")); }
            let size = file["fileSize"].as_u64().ok_or_else(|| invalid("Arquivo do modpack sem tamanho."))?;
            total = total.checked_add(size).ok_or_else(|| invalid("Modpack muito grande."))?;
            if size > 512 * 1024 * 1024 || total > 4 * 1024 * 1024 * 1024 { return Err(invalid("O modpack excede 4 GiB de downloads.")); }
            let sha1 = file.pointer("/hashes/sha1").and_then(|value| value.as_str()).map(str::to_owned); let sha512 = file.pointer("/hashes/sha512").and_then(|value| value.as_str()).map(str::to_owned);
            if sha1.is_none() && sha512.is_none() { return Err(invalid("Arquivo do modpack sem hash de integridade.")); }
            let urls = file["downloads"].as_array().ok_or_else(|| invalid("Arquivo sem download permitido."))?.iter().take(8).map(|url| url.as_str().ok_or_else(|| invalid("URL de download inválida.")).and_then(|value| pack_download::download_url(value).map(|_| value.to_string()))).collect::<AppResult<Vec<_>>>()?;
            if urls.is_empty() { return Err(invalid("Arquivo sem download permitido.")); }
            if let Some(project) = urls.iter().filter_map(|url| url::Url::parse(url).ok()).filter(|url| url.host_str()==Some("cdn.modrinth.com")).find_map(|url| {let parts=url.path_segments()?.collect::<Vec<_>>();(parts.len()>=4 && parts[0]=="data").then(||parts[1].to_string())}) {projects.insert(name.to_string(),project);}
            jobs.push((name.to_string(), urls, size, sha1, sha512));
        }
        let download_client = pack_download::client()?;
        let results = futures_util::stream::iter(jobs).map(|(name, urls, size, sha1, sha512)| {
            let client = &download_client; let root = &transaction;
            async move {
                let path = pack_download::destination(root, &name)?; let mut completed = false;
                let cache=pack_download::content_cache_root();
                if size<=8*1024*1024 {if let Some(cache)=&cache {if let Some(bytes)=pack_download::cached_content(cache,Some(size),sha1.as_deref(),sha512.as_deref()).await {pack_download::atomic_write(&path,&bytes).await?;completed=true;}}}
                for url in urls {
                    if completed {break;}
                    for attempt in 0..2 {
                        if crate::core::mods::resumable::download(client, &url, &path, Some(&self.shutting_down)).await.is_ok() && pack_download::verify_existing(&path, Some(size), sha1.as_deref(), sha512.as_deref(), true).await { completed = true; break; }
                        if path.exists() { let _ = tokio::fs::remove_file(&path).await; }
                        pack_download::backoff(attempt, Some(&self.shutting_down)).await?;
                    }
                    if completed { break; }
                }
                if !completed { return Err(invalid(format!("Falha no download ou integridade de {name}; os arquivos ativos foram preservados."))); }
                if size<=8*1024*1024 {if let Some(cache)=&cache {if let Ok(bytes)=tokio::fs::read(&path).await {pack_download::cache_verified_content(cache,&bytes,sha1.as_deref(),sha512.as_deref()).await;}}}
                let compatible = if name.starts_with("mods/") { let check = path.clone(); tokio::task::spawn_blocking(move || content::server_archive(&check, "mod")).await.map_err(|error| AppError::Internal(error.to_string()))?.map_err(|error|invalid(format!("{name}: {error}")))? } else { true };
                Ok((name, path, compatible))
            }
        }).buffer_unordered(crate::core::downloader::max_concurrent_downloads().clamp(1,12)).collect::<Vec<AppResult<(String, PathBuf, bool)>>>().await;
        for result in results { let (name, path, compatible) = result?; if compatible { prepared.insert(name, path); } else { excluded.insert(name); tokio::fs::remove_file(path).await?; } }
        for (name, source) in pack.overrides {
            if excluded.contains(&name) { continue; }
            let path = pack_download::destination(&transaction, &name)?; tokio::fs::create_dir_all(path.parent().ok_or_else(|| invalid("Caminho inválido."))?).await?; tokio::fs::rename(&source, &path).await?;
            if name.starts_with("mods/") { let check = path.clone(); if !tokio::task::spawn_blocking(move || content::server_archive(&check, "mod")).await.map_err(|error| AppError::Internal(error.to_string()))?.map_err(|error|invalid(format!("{name}: {error}")))? { excluded.insert(name); tokio::fs::remove_file(path).await?; continue; } }
            prepared.insert(name, path);
        }
        let root = transaction.clone();
        prepared = tokio::task::spawn_blocking(move || { bundle_resource_folders(&mut prepared, &root)?; Ok::<_,AppError>(prepared) }).await.map_err(|error| AppError::Internal(error.to_string()))??;
        if prepared.is_empty() {
            if self.contents(id).await?.iter().any(|item| !item.staged && item.kind == "mod") { return Err(invalid("Este pacote não exige mods de servidor, mas há mods instalados aqui. Use um servidor separado ou revise os conteúdos atuais antes de continuar.")); }
            return Ok(serde_json::json!({"prepared":0,"excluded":excluded.len(),"excludedFiles":excluded.iter().cloned().collect::<Vec<_>>(),"noRequiredServerFiles":true}));
        }
        let batch = directory.join(format!(".luxmc-ready-{}", uuid::Uuid::new_v4())); tokio::fs::create_dir(&batch).await?;
        let count = prepared.len();
        for (name, source) in prepared {
            let path = batch.join(uuid::Uuid::new_v4().to_string()); tokio::fs::create_dir(&path).await?;
            tokio::fs::rename(source, path.join("payload")).await?;
            pack_download::atomic_write(&path.join("pending.json"), &serde_json::to_vec(&content::Pending { kind: "modpack".into(), name, remove: false })?).await?;
        }
        if staging.exists() { if tokio::fs::read_dir(&staging).await?.next_entry().await?.is_some() { return Err(unavailable("Outra importação criou pendências; lote preparado preservado.")); } tokio::fs::remove_dir(&staging).await?; }
        tokio::fs::rename(batch, staging).await?;
        pack_download::atomic_write(&directory.join(".luxmc-content-projects.json"),&serde_json::to_vec(&projects)?).await?;
        Ok(serde_json::json!({"prepared":count,"excluded":excluded.len(),"excludedFiles":excluded.iter().cloned().collect::<Vec<_>>()}))
        }.await;
        if transaction.exists() {
            let root = directory.clone();
            let temporary = transaction.clone();
            let _ =
                tokio::task::spawn_blocking(move || backups::remove_owned_tree(&root, &temporary))
                    .await;
        }
        result
    }
}
