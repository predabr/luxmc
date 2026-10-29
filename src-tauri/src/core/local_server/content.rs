use super::*;
use std::io::Read;

#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ContentFile {
    pub name: String,
    pub kind: String,
    pub bytes: u64,
    pub staged: bool,
    pub remove: bool,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ApplyResult {
    pub applied: usize,
    pub preserved: Vec<String>,
}

#[derive(Serialize, Deserialize)]
pub(super) struct Pending {
    pub kind: String,
    pub name: String,
    pub remove: bool,
}

pub(super) const PACK_FOLDERS: &[&str] = &[
    "mods/",
    "config/",
    "defaultconfigs/",
    "kubejs/",
    "scripts/",
    "resources/",
    "resourcepacks/",
    "global_packs/",
    "openloader/",
    "structures/",
    "datapacks/",
    "world/datapacks/",
    "world/",
];

pub(super) fn server_pack_entry(name:&str)->bool {
    !name.starts_with("mods/") || (name.to_ascii_lowercase().ends_with(".jar") && !name.split('/').skip(1).any(|part|part.starts_with('.')))
}

fn folder(kind: &str) -> AppResult<&'static str> {
    match kind {
        "mod" => Ok("mods"),
        "datapack" => Ok("world/datapacks"),
        "resourcepack" => Ok("resourcepacks"),
        "modpack" => Ok(""),
        _ => Err(invalid(
            "Use a área de mods, datapacks ou pacotes de recursos.",
        )),
    }
}

fn filename(name: &str, kind: &str) -> AppResult<()> {
    if kind == "modpack" {
        return pack_path(name).map(|_| ());
    }
    let expected = if kind == "mod" { ".jar" } else { ".zip" };
    if name.trim_end_matches(".disabled").len() > 180
        || name.starts_with('.')
        || name.contains(['/', '\\', ':'])
        || name.chars().any(char::is_control)
        || !name.trim_end_matches(".disabled").to_ascii_lowercase().ends_with(expected)
    {
        return Err(invalid("Nome ou extensão do conteúdo inválido."));
    }
    Ok(())
}

pub(super) fn pack_path(name: &str) -> AppResult<PathBuf> {
    let relative = crate::core::mods::pack_download::relative_path(name)?;
    if !PACK_FOLDERS.iter().any(|prefix| name.starts_with(prefix))
        || relative.components().count() < 2
        || relative
            .extension()
            .and_then(|value| value.to_str())
            .is_some_and(|value| {
                ["exe", "dll", "so", "bat", "cmd", "sh", "ps1", "vbs"]
                    .contains(&value.to_ascii_lowercase().as_str())
            })
    {
        return Err(invalid(
            "O pacote contém um caminho não permitido para conteúdo de servidor.",
        ));
    }
    Ok(relative)
}

fn client_manifest<R: Read + std::io::Seek>(zip: &mut zip::ZipArchive<R>) -> AppResult<bool> {
    let Ok(file) = zip.by_name("META-INF/MANIFEST.MF") else { return Ok(false); };
    let mut text = String::new();
    file.take(262145).read_to_string(&mut text)?;
    if text.len() > 262144 { return Err(invalid("Manifesto do mod muito grande.")); }
    Ok(text.lines().filter_map(|line| line.split_once(':')).any(|(key, value)| key.trim().eq_ignore_ascii_case("ModSide") && value.trim().eq_ignore_ascii_case("CLIENT")))
}

fn metadata_json(bytes: &[u8]) -> AppResult<serde_json::Value> {
    if let Ok(value) = serde_json::from_slice(bytes) { return Ok(value); }
    let mut repaired = Vec::with_capacity(bytes.len());
    let mut quoted = false; let mut escaped = false;
    for &byte in bytes {
        if quoted && byte < 32 {
            repaired.extend_from_slice(format!("\\u{byte:04x}").as_bytes());
            escaped = false;
            continue;
        }
        repaired.push(byte);
        if escaped { escaped = false; }
        else if quoted && byte == b'\\' { escaped = true; }
        else if byte == b'"' { quoted = !quoted; }
    }
    Ok(serde_json::from_slice(&repaired)?)
}

fn client_bootstrap<R: Read + std::io::Seek>(zip: &mut zip::ZipArchive<R>) -> AppResult<bool> {
    let mut properties = String::new();
    match zip.by_name("essential-loader.properties") {
        Ok(file) => { file.take(262145).read_to_string(&mut properties)?; }
        Err(_) => return Ok(false),
    }
    if properties.len() > 262144 { return Err(invalid("Metadados do carregador muito grandes.")); }
    let Some(pinned) = properties.lines().filter_map(|line| line.split_once('=')).find_map(|(key, value)| (key.trim() == "pinnedFile").then_some(value.trim().trim_start_matches('/'))) else { return Ok(false); };
    if !pinned.ends_with(".jar") { return Ok(false); }
    let Ok(file) = zip.by_name(pinned) else { return Ok(false); };
    if file.size() > 64 * 1024 * 1024 { return Err(invalid("Mod incorporado muito grande.")); }
    let mut bytes = Vec::new();
    file.take(64 * 1024 * 1024 + 1).read_to_end(&mut bytes)?;
    if bytes.len() > 64 * 1024 * 1024 { return Err(invalid("Mod incorporado muito grande.")); }
    client_manifest(&mut zip::ZipArchive::new(std::io::Cursor::new(bytes))?)
}

pub(super) fn validate_archive(path: &Path, kind: &str) -> AppResult<()> {
    if !server_archive(path,kind)? { return Err(invalid("Este mod é exclusivo do cliente e não pode ser colocado no servidor.")); }
    Ok(())
}

pub(super) fn server_archive(path: &Path, kind: &str) -> AppResult<bool> {
    backups::normal_file(path)?;
    let mut zip = zip::ZipArchive::new(std::fs::File::open(path)?)?;
    if zip.len() > if kind == "mod" { 200_000 } else { 40_000 } {
        return Err(invalid("O pacote contém arquivos demais."));
    }
    let mut total = 0u64;
    for index in 0..zip.len() {
        let entry = zip.by_index(index)?;
        total = total
            .checked_add(entry.size())
            .ok_or_else(|| invalid("Pacote muito grande."))?;
        if total > 2 * 1024 * 1024 * 1024
            || entry.enclosed_name().is_none()
            || entry
                .unix_mode()
                .is_some_and(|mode| mode & 0o170000 == 0o120000)
        {
            return Err(invalid("Pacote com caminho ou tamanho inválido."));
        }
    }
    if kind != "mod" && zip.by_name("pack.mcmeta").is_err() {
        return Err(invalid("O pacote precisa conter pack.mcmeta na raiz. Modpacks e mundos usam importação própria."));
    }
    if kind == "mod" {
        if client_manifest(&mut zip)? || client_bootstrap(&mut zip)? {
            return Ok(false);
        }
        if let Ok(file) = zip.by_name("fabric.mod.json") {
            let mut bytes = Vec::new();
            file.take(262145).read_to_end(&mut bytes)?;
            if bytes.len() > 262144 {
                return Err(invalid("Metadados do mod muito grandes."));
            }
            let metadata = metadata_json(&bytes)?;
            if metadata.get("environment").and_then(|value| value.as_str()) == Some("client") {
                return Ok(false);
            }
        }
        if let Ok(file) = zip.by_name("quilt.mod.json") {
            let mut bytes = Vec::new();
            file.take(262145).read_to_end(&mut bytes)?;
            if bytes.len() > 262144 {
                return Err(invalid("Metadados Quilt muito grandes."));
            }
            let metadata = metadata_json(&bytes)?;
            if metadata
                .pointer("/minecraft/environment")
                .and_then(|value| value.as_str())
                == Some("client")
            {
                return Ok(false);
            }
        }
        if let Ok(file) = zip.by_name("META-INF/mods.toml") {
            let mut text = String::new();
            file.take(2 * 1024 * 1024 + 1).read_to_string(&mut text)?;
            if text.len() > 2 * 1024 * 1024 {
                return Err(invalid("Metadados Forge muito grandes."));
            }
            let metadata: toml::Value = text
                .parse()
                .map_err(|_| invalid("Metadados Forge inválidos."))?;
            if metadata
                .get("clientSideOnly")
                .and_then(|value| value.as_bool())
                == Some(true)
            {
                return Ok(false);
            }
        }
        let recognized = [
            "fabric.mod.json",
            "quilt.mod.json",
            "META-INF/mods.toml",
            "META-INF/neoforge.mods.toml",
            "mcmod.info",
        ]
        .iter()
        .any(|name| zip.by_name(name).is_ok());
        if !recognized {
            return Err(invalid(
                "O arquivo não contém metadados de um mod reconhecido. Não foi executado.",
            ));
        }
    }
    Ok(true)
}

fn validate_mod_set(files: &[(PathBuf, String)], loader: &str) -> AppResult<()> {
    let mut ids = std::collections::BTreeMap::new();
    for (path, name) in files {
        validate_archive(path, "mod")?;
        let metadata = crate::core::mods::validation::mod_metadata(path)
            .or_else(|| legacy_metadata(path))
            .ok_or_else(|| invalid(format!("Não foi possível ler os metadados de {name}.")))?;
        if loader == "vanilla"
            || !metadata
                .loaders
                .iter()
                .any(|kind| kind == loader || (loader == "quilt" && kind == "fabric"))
        {
            return Err(invalid(format!(
                "O mod {name} não declara compatibilidade com o loader {loader} deste servidor."
            )));
        }
        for id in metadata.ids {
            if let Some(previous) = ids.insert(id.clone(), name.clone()) {
                if previous != *name {
                    return Err(invalid(format!("O mod {id} aparece em {previous} e {name}. Agende a remoção da versão antiga antes de aplicar a nova.")));
                }
            }
        }
    }
    Ok(())
}

fn legacy_metadata(path: &Path) -> Option<crate::core::mods::validation::ModMetadata> {
    let mut zip = zip::ZipArchive::new(std::fs::File::open(path).ok()?).ok()?;
    let value = zip.by_name("mcmod.info").ok().and_then(|entry| {
        serde_json::from_reader::<_, serde_json::Value>(entry.take(2 * 1024 * 1024)).ok()
    })?;
    let entries = value
        .as_array()
        .or_else(|| value.get("modList").and_then(|value| value.as_array()))?;
    let mut metadata = crate::core::mods::validation::ModMetadata::default();
    for entry in entries {
        if let Some(id) = entry
            .get("modid")
            .and_then(|value| value.as_str())
            .filter(|value| !value.is_empty())
        {
            metadata.ids.insert(id.into());
        }
    }
    metadata.loaders.insert("forge".into());
    (!metadata.ids.is_empty()).then_some(metadata)
}

impl LocalServerManager {
    pub(super) async fn validate_installed_mods(&self, id: &str) -> AppResult<()> {
        let directory = self.directory(id).await?;
        let loader = self.loader(id).await?;
        let files = self
            .contents(id)
            .await?
            .into_iter()
            .filter(|item| {
                !item.staged
                    && item.kind == "mod"
                    && item.name.to_ascii_lowercase().ends_with(".jar")
            })
            .map(|item| (directory.join("mods").join(&item.name), item.name))
            .collect::<Vec<_>>();
        tokio::task::spawn_blocking(move || validate_mod_set(&files, &loader.kind))
            .await
            .map_err(|error| AppError::Internal(error.to_string()))?
    }

    pub async fn clear_pending(&self, id: &str) -> AppResult<()> {
        let _guard = self.change()?;
        let directory = self.directory(id).await?;
        let staging = directory.join(".luxmc-staging");
        if !staging.exists() {
            return Ok(());
        }
        let mut entries = tokio::fs::read_dir(&staging).await?;
        while let Some(entry) = entries.next_entry().await? {
            if entry.path().join("previous").exists() {
                return Err(unavailable("Uma recuperação pendente contém arquivos anteriores. Restaure o backup antes de descartar."));
            }
        }
        tokio::task::spawn_blocking(move || backups::remove_owned_tree(&directory, &staging))
            .await
            .map_err(|error| AppError::Internal(error.to_string()))??;
        Ok(())
    }

    pub async fn contents(&self, id: &str) -> AppResult<Vec<ContentFile>> {
        let directory = self.directory(id).await?;
        tokio::task::spawn_blocking(move || -> AppResult<Vec<ContentFile>> {
            let mut result = Vec::new();
            for kind in ["mod", "datapack", "resourcepack"] {
                let path = directory.join(folder(kind)?);
                if !path.exists() {
                    continue;
                }
                backups::normal_file(&path)?;
                for entry in std::fs::read_dir(path)? {
                    let entry = entry?;
                    let metadata = backups::normal_file(&entry.path())?;
                    if metadata.is_file() {
                        result.push(ContentFile {
                            name: entry.file_name().to_string_lossy().into_owned(),
                            kind: kind.into(),
                            bytes: metadata.len(),
                            staged: false,
                            remove: false,
                        });
                    }
                    if result.len() > 4096 {
                        return Err(invalid(
                            "Há mais de 4096 conteúdos. Use a pasta do servidor para conferir.",
                        ));
                    }
                }
            }
            let staging = directory.join(".luxmc-staging");
            if staging.exists() {
                backups::normal_file(&staging)?;
                for entry in std::fs::read_dir(staging)? {
                    let entry = entry?;
                    backups::normal_file(&entry.path())?;
                    let record: Pending =
                        serde_json::from_slice(&std::fs::read(entry.path().join("pending.json"))?)?;
                    folder(&record.kind)?;
                    filename(&record.name, &record.kind)?;
                    let bytes = if record.remove {
                        0
                    } else {
                        backups::normal_file(&entry.path().join("payload"))?.len()
                    };
                    result.push(ContentFile {
                        name: record.name,
                        kind: record.kind,
                        bytes,
                        staged: true,
                        remove: record.remove,
                    });
                    if result.len() > 4096 {
                        return Err(invalid("Conteúdos pendentes demais."));
                    }
                }
            }
            result.sort_by(|a, b| {
                a.kind
                    .cmp(&b.kind)
                    .then(a.name.to_lowercase().cmp(&b.name.to_lowercase()))
            });
            Ok(result)
        })
        .await
        .map_err(|error| AppError::Internal(error.to_string()))?
    }

    pub async fn stage_content(&self, id: &str, kind: &str, path: &Path) -> AppResult<()> {
        let _guard = self.change()?;
        let directory = self.directory(id).await?;
        folder(kind)?;
        if kind == "mod" && self.loader(id).await?.kind == "vanilla" {
            return Err(invalid(
                "Configure um loader de servidor compatível antes de adicionar mods.",
            ));
        }
        let name = path
            .file_name()
            .and_then(|value| value.to_str())
            .ok_or_else(|| invalid("Nome de arquivo inválido."))?
            .to_string();
        filename(&name, kind)?;
        if self
            .contents(id)
            .await?
            .iter()
            .any(|file| file.staged && file.name.eq_ignore_ascii_case(&name) && file.kind == kind)
        {
            return Err(invalid(
                "Já existe uma alteração pendente para este arquivo.",
            ));
        }
        let metadata = backups::normal_file(path)?;
        if !metadata.is_file() || metadata.len() > 512 * 1024 * 1024 {
            return Err(invalid("Escolha um pacote de até 512 MiB."));
        }
        let source = path.to_path_buf();
        let format = kind.to_string();
        tokio::task::spawn_blocking(move || validate_archive(&source, &format))
            .await
            .map_err(|error| AppError::Internal(error.to_string()))??;
        let staging = directory.join(".luxmc-staging");
        tokio::fs::create_dir_all(&staging).await?;
        backups::normal_file(&staging)?;
        let pending = staging.join(uuid::Uuid::new_v4().to_string());
        tokio::fs::create_dir(&pending).await?;
        tokio::fs::copy(path, pending.join("payload")).await?;
        crate::core::mods::pack_download::atomic_write(
            &pending.join("pending.json"),
            &serde_json::to_vec(&Pending {
                kind: kind.into(),
                name,
                remove: false,
            })?,
        )
        .await?;
        Ok(())
    }

    pub async fn stage_remove(&self, id: &str, kind: &str, name: &str) -> AppResult<()> {
        let _guard = self.change()?;
        let directory = self.directory(id).await?;
        folder(kind)?;
        filename(name, kind)?;
        let current = self.contents(id).await?;
        if current
            .iter()
            .any(|file| file.staged && file.kind == kind && file.name.eq_ignore_ascii_case(name))
        {
            return Err(unavailable(
                "Já existe uma alteração pendente para este conteúdo.",
            ));
        }
        if !current
            .iter()
            .any(|file| !file.staged && file.kind == kind && file.name == name)
        {
            return Err(invalid("Conteúdo não encontrado."));
        }
        let staging = directory.join(".luxmc-staging");
        tokio::fs::create_dir_all(&staging).await?;
        backups::normal_file(&staging)?;
        let pending = staging.join(uuid::Uuid::new_v4().to_string());
        tokio::fs::create_dir(&pending).await?;
        crate::core::mods::pack_download::atomic_write(
            &pending.join("pending.json"),
            &serde_json::to_vec(&Pending {
                kind: kind.into(),
                name: name.into(),
                remove: true,
            })?,
        )
        .await?;
        Ok(())
    }

    pub async fn apply_content(&self, id: &str) -> AppResult<ApplyResult> {
        let _guard = self.change()?;
        if self.current(id).is_ok() {
            return Err(unavailable("Pare e salve o servidor para aplicar conteúdos. Os downloads pendentes foram preservados."));
        }
        let directory = self.directory(id).await?;
        let staging = directory.join(".luxmc-staging");
        if !staging.exists() {
            return Ok(ApplyResult {
                applied: 0,
                preserved: Vec::new(),
            });
        }
        backups::normal_file(&staging)?;
        let baseline_path = directory.join("luxmc-pack-baseline.json");
        let mut baseline: std::collections::BTreeMap<String, String> = if baseline_path.exists() {
            let metadata = backups::normal_file(&baseline_path)?;
            if metadata.len() > 2 * 1024 * 1024 {
                return Err(invalid("Registro de arquivos do modpack muito grande."));
            }
            serde_json::from_slice(&tokio::fs::read(&baseline_path).await?)?
        } else {
            Default::default()
        };
        let mut incoming = std::collections::BTreeSet::new();
        let mut preserved = Vec::new();
        let mut skipped = Vec::new();
        let mut plan = Vec::new();
        let mut targets = std::collections::HashSet::new();
        let mut entries = tokio::fs::read_dir(&staging).await?;
        while let Some(entry) = entries.next_entry().await? {
            let path = entry.path();
            backups::normal_file(&path)?;
            let record: Pending =
                serde_json::from_slice(&self.read(&path, "pending.json").await?.ok_or_else(
                    || invalid("Alteração pendente incompleta; confira a pasta do servidor."),
                )?)?;
            let destination = directory.join(folder(&record.kind)?);
            filename(&record.name, &record.kind)?;
            tokio::fs::create_dir_all(&destination).await?;
            if !tokio::fs::canonicalize(&destination)
                .await?
                .starts_with(&directory)
            {
                return Err(invalid("Pasta de conteúdo fora do servidor."));
            }
            backups::normal_file(&destination)?;
            let target = if record.kind == "modpack" {
                crate::core::mods::pack_download::destination(&directory, &record.name)?
            } else {
                crate::core::mods::pack_download::destination(&destination, &record.name)?
            };
            tokio::fs::create_dir_all(
                target
                    .parent()
                    .ok_or_else(|| invalid("Caminho inválido."))?,
            )
            .await?;
            if !targets.insert(target.to_string_lossy().to_ascii_lowercase()) {
                return Err(invalid("Duas alterações pendentes usam o mesmo destino. Revise as pendências antes de aplicar."));
            }
            if target.exists() {
                backups::normal_file(&target)?;
            }
            if path.join("previous").exists() {
                return Err(unavailable(
                    "Uma aplicação anterior está preservada; confira a pasta antes de continuar.",
                ));
            }
            if !record.remove {
                let source = path.join("payload");
                backups::normal_file(&source)?;
                let source_copy = source.clone();
                let kind = record.kind.clone();
                let name = record.name.clone();
                let validate = tokio::task::spawn_blocking(move || {
                    if kind == "modpack" {
                        if name.starts_with("mods/") {
                            validate_archive(&source_copy, "mod")
                        } else {
                            Ok(())
                        }
                    } else {
                        validate_archive(&source_copy, &kind)
                    }
                })
                .await
                .map_err(|error| AppError::Internal(error.to_string()))?;
                validate?;
                if record.kind == "modpack" {
                    use sha2::Digest;
                    incoming.insert(record.name.clone());
                    let hash = format!(
                        "{:x}",
                        sha2::Sha256::digest(tokio::fs::read(&source).await?)
                    );
                    if target.exists() && !record.name.starts_with("mods/") {
                        if backups::normal_file(&target)?.len() > 128 * 1024 * 1024 {
                            return Err(invalid("Uma configuração local excede 128 MiB; confira-a manualmente antes de atualizar."));
                        }
                        let current = format!(
                            "{:x}",
                            sha2::Sha256::digest(tokio::fs::read(&target).await?)
                        );
                        if current != hash && baseline.get(&record.name) != Some(&current) {
                            preserved.push(record.name.clone());
                            skipped.push(path);
                            continue;
                        }
                    }
                    baseline.insert(record.name.clone(), hash);
                }
            }
            plan.push((path, record, target, false, false));
        }
        if !incoming.is_empty() {
            let obsolete = baseline
                .iter()
                .filter(|(name, _)| name.starts_with("mods/") && !incoming.contains(*name))
                .map(|(name, hash)| (name.clone(), hash.clone()))
                .collect::<Vec<_>>();
            for (name, expected) in obsolete {
                content::pack_path(&name)?;
                let target = crate::core::mods::pack_download::destination(&directory, &name)?;
                if !target.exists() {
                    baseline.remove(&name);
                    continue;
                }
                backups::normal_file(&target)?;
                use sha2::Digest;
                if format!(
                    "{:x}",
                    sha2::Sha256::digest(tokio::fs::read(&target).await?)
                ) != expected
                {
                    preserved.push(name);
                    continue;
                }
                if !targets.insert(target.to_string_lossy().to_ascii_lowercase()) {
                    continue;
                }
                let path = staging.join(uuid::Uuid::new_v4().to_string());
                tokio::fs::create_dir(&path).await?;
                let record = Pending {
                    kind: "modpack".into(),
                    name: name.clone(),
                    remove: true,
                };
                crate::core::mods::pack_download::atomic_write(
                    &path.join("pending.json"),
                    &serde_json::to_vec(&record)?,
                )
                .await?;
                baseline.remove(&name);
                plan.push((path, record, target, false, false));
            }
        }
        let loader = self.loader(id).await?;
        let mut mod_files = self
            .contents(id)
            .await?
            .into_iter()
            .filter(|item| {
                !item.staged
                    && item.kind == "mod"
                    && item.name.to_ascii_lowercase().ends_with(".jar")
            })
            .map(|item| (directory.join("mods").join(&item.name), item.name))
            .filter(|(path, _)| !plan.iter().any(|(_, _, target, _, _)| target == path))
            .collect::<Vec<_>>();
        for (path, record, target, _, _) in &plan {
            if !record.remove && target.parent() == Some(directory.join("mods").as_path()) {
                mod_files.push((
                    path.join("payload"),
                    target
                        .file_name()
                        .unwrap_or_default()
                        .to_string_lossy()
                        .into_owned(),
                ));
            }
        }
        tokio::task::spawn_blocking(move || validate_mod_set(&mod_files, &loader.kind))
            .await
            .map_err(|error| AppError::Internal(error.to_string()))??;
        let applied: AppResult<()> = async {
            for (path, record, target, moved, installed) in &mut plan {
                if target.exists() {
                    tokio::fs::rename(&target, path.join("previous")).await?;
                    *moved = true;
                }
                if !record.remove {
                    tokio::fs::rename(path.join("payload"), &target).await?;
                    *installed = true;
                }
            }
            if !incoming.is_empty() {
                crate::core::mods::pack_download::atomic_write(
                    &baseline_path,
                    &serde_json::to_vec_pretty(&baseline)?,
                )
                .await?;
            }
            Ok(())
        }
        .await;
        if let Err(error) = applied {
            let mut recovered = true;
            for (path, _, target, moved, installed) in plan.iter().rev() {
                if *installed
                    && tokio::fs::rename(target, path.join("payload"))
                        .await
                        .is_err()
                {
                    recovered = false;
                }
                if *moved
                    && tokio::fs::rename(path.join("previous"), target)
                        .await
                        .is_err()
                {
                    recovered = false;
                }
            }
            return Err(unavailable(format!(
                "Não foi possível aplicar o lote: {error}. {}",
                if recovered {
                    "Os arquivos anteriores foram restaurados e as pendências preservadas."
                } else {
                    "A recuperação automática ficou incompleta. Preserve .luxmc-staging e restaure o backup antes de iniciar."
                }
            )));
        }
        let applied_count = plan.len();
        for path in skipped {
            tokio::fs::remove_file(path.join("payload")).await?;
            tokio::fs::remove_file(path.join("pending.json")).await?;
            tokio::fs::remove_dir(path).await?;
        }
        for (path, _, _, _, _) in plan {
            if path.join("previous").exists() {
                tokio::fs::remove_file(path.join("previous")).await?;
            }
            tokio::fs::remove_file(path.join("pending.json")).await?;
            tokio::fs::remove_dir(path).await?;
        }
        tokio::fs::remove_dir(staging).await?;
        let marker = directory.join(".luxmc-instance-pending");
        if marker.exists() { tokio::fs::remove_file(marker).await?; }
        Ok(ApplyResult {
            applied: applied_count,
            preserved,
        })
    }
}
