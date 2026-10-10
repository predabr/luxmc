use crate::error::{AppError, AppResult};
use serde::Serialize;
use std::path::{Path, PathBuf};

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ContentEntry { name: String, title: String, kind: String, icon: Option<String> }

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct InstanceContent { pub directory: String, pub entries: Vec<ContentEntry>, pub isolated: bool, pub world_accounts: Vec<String>, pub world_account: Option<String> }

pub(super) fn workspace(id: &str) -> AppResult<PathBuf> {
    if uuid::Uuid::parse_str(id).is_err() { return Err(AppError::InvalidInput("Instância Bedrock inválida".into())); }
    Ok(super::bedrock_catalog::root()?.join("instances").join(id))
}

fn content_folder(kind: &str) -> AppResult<&'static str> {
    match kind { "resources" => Ok("resource_packs"), "behaviors" => Ok("behavior_packs"), "worlds" => Ok("minecraftWorlds"), _ => Err(AppError::InvalidInput("Tipo de conteúdo Bedrock inválido".into())) }
}

fn initialize(id: &str) -> AppResult<PathBuf> {
    super::bedrock::managed_content(id)?;
    let root = workspace(id)?;
    std::fs::create_dir_all(&root)?;
    if !root.canonicalize()?.starts_with(super::bedrock_catalog::root()?.canonicalize()?) { return Err(AppError::InvalidInput("Pasta Bedrock fora dos dados do Luxmc".into())); }
    for category in ["resource_packs", "behavior_packs", "minecraftWorlds"] { std::fs::create_dir_all(root.join("shared").join(category))?; }
    Ok(root)
}

fn content_list(root: &Path) -> AppResult<Vec<ContentEntry>> {
    let mut result = Vec::new();
    let mut bases = vec![root.join("shared")];
    if let Ok(entries) = std::fs::read_dir(root) { bases.extend(entries.flatten().filter(|entry| entry.file_type().is_ok_and(|kind| kind.is_dir() && !kind.is_symlink()) && valid_slot(&entry.file_name().to_string_lossy())).map(|entry| entry.path())); }
    for (kind, folder) in [("resources", "resource_packs"), ("behaviors", "behavior_packs"), ("worlds", "minecraftWorlds")] {
        for base in &bases {
            let Ok(entries) = std::fs::read_dir(base.join(folder)) else { continue; };
            for entry in entries.flatten().filter(|entry| entry.file_type().is_ok_and(|kind| kind.is_dir() && !kind.is_symlink())) {
                let path = entry.path();
                let name = entry.file_name().to_string_lossy().into_owned();
                let title = if kind == "worlds" { std::fs::read_to_string(path.join("levelname.txt")).ok().map(|name| name.chars().take(160).collect::<String>()) }
                    else { std::fs::File::open(path.join("manifest.json")).ok().and_then(|file| serde_json::from_reader::<_, serde_json::Value>(std::io::Read::take(file, 1024 * 1024)).ok()).and_then(|manifest| manifest.pointer("/header/name").and_then(|value| value.as_str()).map(str::to_owned)) };
                let icon_file = path.join(if kind == "worlds" { "world_icon.jpeg" } else { "pack_icon.png" });
                let icon = std::fs::read(icon_file).ok().filter(|bytes| bytes.len() <= 256000).map(|bytes| { use base64::Engine; format!("data:image/{};base64,{}", if kind == "worlds" {"jpeg"} else {"png"}, base64::engine::general_purpose::STANDARD.encode(bytes)) });
                result.push(ContentEntry { name: format!("{}/{}", base.file_name().unwrap().to_string_lossy(), name), title: title.unwrap_or_else(|| name.clone()), kind: kind.into(), icon });
            }
        }
    }
    result.sort_by(|left, right| left.title.to_lowercase().cmp(&right.title.to_lowercase()));
    Ok(result)
}

#[tauri::command]
pub async fn bedrock_instance_content(id: String) -> AppResult<InstanceContent> {
    tokio::task::spawn_blocking(move || { let root = initialize(&id)?; let (gdk, preview) = super::bedrock::managed_content(&id)?; let accounts = if gdk { world_accounts(preview)? } else { Vec::new() }; let account = chosen_account(&root, &accounts); if let Some(account) = &account { std::fs::write(root.join("world-account"), account)?; } Ok(InstanceContent { directory: root.to_string_lossy().into_owned(), entries: content_list(&root)?, isolated: root.join(".linked").is_file(), world_accounts: accounts, world_account: account }) }).await.map_err(|error| AppError::Internal(error.to_string()))?
}

fn extract_content(source: &Path, root: &Path, kind: &str) -> AppResult<()> {
    let folder = content_folder(kind)?;
    let file = std::fs::File::open(source)?;
    if file.metadata()?.len() > 1024 * 1024 * 1024 { return Err(AppError::InvalidInput("Importe um pacote de até 1 GB".into())); }
    let mut archive = zip::ZipArchive::new(file)?;
    if archive.len() > 50000 { return Err(AppError::InvalidInput("O pacote contém arquivos demais".into())); }
    let stage = root.join("imports").join(uuid::Uuid::new_v4().to_string());
    std::fs::create_dir_all(&stage)?;
    let result = (|| -> AppResult<()> {
        let mut total = 0u64;
        for index in 0..archive.len() {
            let mut entry = archive.by_index(index)?;
            total = total.saturating_add(entry.size());
            if total > 2 * 1024 * 1024 * 1024 || entry.unix_mode().is_some_and(|mode| mode & 0o170000 == 0o120000) { return Err(AppError::InvalidInput("O pacote contém links ou excede 2 GB descompactado".into())); }
            let relative = entry.enclosed_name().ok_or_else(|| AppError::InvalidInput("Caminho inseguro no pacote".into()))?.to_owned();
            if relative.to_string_lossy().contains(':') { return Err(AppError::InvalidInput("Caminho inválido no pacote".into())); }
            let target = stage.join(relative);
            if entry.is_dir() { std::fs::create_dir_all(target)?; }
            else {
                std::fs::create_dir_all(target.parent().unwrap())?;
                let expected = entry.size();
                let written = std::io::copy(&mut std::io::Read::take(&mut entry, expected.saturating_add(1)), &mut std::fs::File::create(target)?)?;
                if written != expected { return Err(AppError::InvalidInput("O tamanho real de um arquivo não corresponde ao pacote".into())); }
            }
        }
        if kind == "worlds" { if !stage.join("level.dat").is_file() { return Err(AppError::InvalidInput("O arquivo não contém um mundo Bedrock válido".into())); } }
        else {
            let manifest: serde_json::Value = serde_json::from_reader(std::fs::File::open(stage.join("manifest.json"))?)?;
            let expected = if kind == "resources" { "resources" } else { "data" };
            if !manifest.get("modules").and_then(|modules| modules.as_array()).is_some_and(|modules| modules.iter().any(|module| module.get("type").and_then(|value| value.as_str()) == Some(expected))) { return Err(AppError::InvalidInput("Selecione o tipo correspondente ao manifest.json do pacote".into())); }
        }
        let slot = if kind == "worlds" { std::fs::read_to_string(root.join("world-account")).ok().filter(|id| valid_account(id)).map(|id| format!("account_{id}")).unwrap_or_else(|| "shared".into()) } else { "shared".into() };
        let destination = root.join(slot).join(folder).join(uuid::Uuid::new_v4().to_string());
        std::fs::create_dir_all(destination.parent().unwrap())?;
        std::fs::rename(&stage, destination)?;
        Ok(())
    })();
    if stage.exists() { let _ = std::fs::remove_dir_all(&stage); }
    result
}

#[tauri::command]
pub async fn bedrock_import_content(id: String, kind: String, path: String) -> AppResult<()> {
    ensure_closed().await?;
    let _guard = super::bedrock_catalog::INSTALL_LOCK.try_lock().map_err(|_| AppError::InvalidState("Aguarde a instalação Bedrock terminar".into()))?;
    tokio::task::spawn_blocking(move || { let root = initialize(&id)?; extract_content(Path::new(&path), &root, &kind) }).await.map_err(|error| AppError::Internal(error.to_string()))?
}

#[tauri::command]
pub async fn bedrock_delete_content(id: String, kind: String, name: String) -> AppResult<()> {
    ensure_closed().await?;
    let _guard = super::bedrock_catalog::INSTALL_LOCK.try_lock().map_err(|_| AppError::InvalidState("Aguarde a instalação Bedrock terminar".into()))?;
    let root = initialize(&id)?;
    let (slot, name) = name.split_once('/').ok_or_else(|| AppError::InvalidInput("Conteúdo inválido".into()))?;
    if (slot != "shared" && !valid_slot(slot)) || name.is_empty() || name.contains(['/', '\\', ':']) || name == "." || name == ".." { return Err(AppError::InvalidInput("Conteúdo inválido".into())); }
    let target = root.join(slot).join(content_folder(&kind)?).join(name).canonicalize()?;
    if !target.starts_with(root.canonicalize()?) || target == root.canonicalize()? { return Err(AppError::InvalidInput("Conteúdo fora da instância".into())); }
    tokio::fs::remove_dir_all(target).await?;
    Ok(())
}

#[tauri::command]
pub async fn bedrock_open_folder(id: String) -> AppResult<()> {
    let root = initialize(&id)?;
    crate::core::process::std_command(if cfg!(windows) { "explorer.exe" } else { "xdg-open" }).arg(root).spawn()?;
    Ok(())
}

pub(super) async fn ensure_closed() -> AppResult<()> {
    #[cfg(windows)] {
        let output = tokio::process::Command::new(shell()?).args(["-NoProfile", "-NonInteractive", "-Command", "if(Get-Process -Name Minecraft.Windows,Minecraft.WindowsBeta -ErrorAction SilentlyContinue){exit 1}"]).creation_flags(0x08000000).kill_on_drop(true).output().await?;
        if !output.status.success() { return Err(AppError::InvalidState("Feche o Minecraft Bedrock antes de alterar ou trocar a instância".into())); }
    }
    Ok(())
}

#[cfg(windows)]
fn shell() -> AppResult<PathBuf> { Ok(PathBuf::from(std::env::var_os("SystemRoot").ok_or_else(|| AppError::InvalidState("Windows indisponível".into()))?).join("System32/WindowsPowerShell/v1.0/powershell.exe")) }

fn valid_account(id: &str) -> bool { !id.is_empty() && id.len() <= 32 && id.bytes().all(|byte| byte.is_ascii_digit()) }
fn valid_slot(slot: &str) -> bool { slot.strip_prefix("account_").is_some_and(valid_account) }
fn gdk_users(preview: bool) -> AppResult<PathBuf> { Ok(PathBuf::from(std::env::var_os("APPDATA").ok_or_else(|| AppError::InvalidState("AppData indisponível".into()))?).join(if preview { "Minecraft Bedrock Preview/Users" } else { "Minecraft Bedrock/Users" })) }
fn world_accounts(preview: bool) -> AppResult<Vec<String>> {
    let mut accounts = Vec::new();
    if let Ok(entries) = std::fs::read_dir(gdk_users(preview)?) { for entry in entries.flatten() { let id = entry.file_name().to_string_lossy().into_owned(); if valid_account(&id) && entry.file_type()?.is_dir() { accounts.push(id); } } }
    accounts.sort(); Ok(accounts)
}
fn chosen_account(root: &Path, accounts: &[String]) -> Option<String> { std::fs::read_to_string(root.join("world-account")).ok().filter(|id| accounts.contains(id)).or_else(|| (accounts.len() == 1).then(|| accounts[0].clone())) }

#[tauri::command]
pub async fn bedrock_select_world_account(id: String, account: String) -> AppResult<()> {
    ensure_closed().await?;
    let root = initialize(&id)?;
    let (gdk, preview) = super::bedrock::managed_content(&id)?;
    if !gdk || !world_accounts(preview)?.contains(&account) { return Err(AppError::InvalidInput("Conta Bedrock indisponível".into())); }
    std::fs::write(root.join("world-account"), account)?;
    if root.join(".linked").exists() { std::fs::remove_file(root.join(".linked"))?; }
    Ok(())
}

pub(super) async fn prepare(id: &str, gdk: bool, preview: bool) -> AppResult<Option<String>> {
    ensure_closed().await?;
    let root = initialize(id)?;
    #[cfg(windows)] {
        let sources = if gdk {
            let users = gdk_users(preview)?;
            std::fs::create_dir_all(users.join("Shared/games"))?;
            let mut paths = vec![(users.join("Shared/games/com.mojang"), root.join("shared"))];
            let accounts = world_accounts(preview)?;
            if let Some(account) = chosen_account(&root, &accounts) {
                std::fs::write(root.join("world-account"), &account)?;
                let target = root.join(format!("account_{account}"));
                if let Ok(entries) = std::fs::read_dir(root.join("shared/minecraftWorlds")) { for entry in entries.flatten() { if entry.file_type()?.is_dir() && !entry.file_type()?.is_symlink() { let destination = target.join("minecraftWorlds").join(entry.file_name()); if destination.exists() { return Err(AppError::InvalidState("Há um mundo com o mesmo nome na pasta da conta; importação preservada".into())); } std::fs::create_dir_all(destination.parent().unwrap())?; std::fs::rename(entry.path(), destination)?; } } }
                paths.push((users.join(&account).join("games/com.mojang"), target));
            }
            else if accounts.len() > 1 { return Err(AppError::InvalidState("Há várias contas Bedrock neste Windows. Escolha a conta dos mundos na página da instância antes de jogar".into())); }
            paths
        } else {
            let local = PathBuf::from(std::env::var_os("LOCALAPPDATA").ok_or_else(|| AppError::InvalidState("AppData indisponível".into()))?);
            vec![(local.join(if preview { "Packages/Microsoft.MinecraftWindowsBeta_8wekyb3d8bbwe/LocalState/games/com.mojang" } else { "Packages/Microsoft.MinecraftUWP_8wekyb3d8bbwe/LocalState/games/com.mojang" }), root.join("shared"))]
        };
        let owned = super::bedrock_catalog::root()?.join("instances");
        for (source, target) in &sources { link_data(source, target, &owned).await?; }
        let complete = !gdk || sources.len() > 1;
        if complete { std::fs::write(root.join(".linked"), b"verified")?; }
        return Ok((!complete).then(|| "Recursos isolados. Após entrar na conta do Bedrock pela primeira vez, feche o jogo e abra esta instância novamente para isolar também os mundos dessa conta.".into()));
    }
    #[cfg(not(windows))] { let _ = (gdk, preview, root); Ok(None) }
}

#[cfg(windows)]
async fn link_data(source: &Path, target: &Path, owned: &Path) -> AppResult<()> {
    std::fs::create_dir_all(target)?;
    if !target.canonicalize()?.starts_with(owned.canonicalize()?) { return Err(AppError::InvalidInput("Destino Bedrock fora das instâncias".into())); }
    std::fs::create_dir_all(source.parent().unwrap())?;
    let mut previous = None;
    let mut adopted = false;
    let mut backup = None;
    if std::fs::symlink_metadata(source).is_ok() {
        if std::fs::read_link(source).is_ok() {
            let existing = source.canonicalize()?;
            if existing == target.canonicalize()? { return Ok(()); }
            if !existing.starts_with(owned.canonicalize()?) { return Err(AppError::InvalidState("A pasta Bedrock pertence a outro provedor. Seus arquivos foram preservados".into())); }
            previous = Some(existing);
            std::fs::remove_dir(source)?;
        } else {
            let empty = !walkdir::WalkDir::new(target).follow_links(false).into_iter().filter_map(Result::ok).any(|entry| entry.file_type().is_file() || entry.file_type().is_symlink());
            if empty {
                std::fs::remove_dir_all(target)?;
                std::fs::rename(source, target)?;
                adopted = true;
            } else {
            let original = owned.parent().unwrap().join("world-backups").join(format!("migration-{}", uuid::Uuid::new_v4()));
            std::fs::create_dir_all(original.parent().unwrap())?;
            std::fs::rename(source, &original)?;
            backup = Some(original.clone());
            let record = target.parent().unwrap().join("original-data.json");
            let mut records = std::fs::read(&record).ok().and_then(|bytes| serde_json::from_slice::<Vec<serde_json::Value>>(&bytes).ok()).unwrap_or_default();
            records.push(serde_json::json!({"source": source, "backup": original}));
            std::fs::write(record, serde_json::to_vec(&records)?)?;
            }
        }
    }
    if let Err(error) = create_link(source, target).await {
        if std::fs::read_link(source).is_ok() { let _ = std::fs::remove_dir(source); }
        if adopted { let _ = std::fs::rename(target, source); }
        else if let Some(backup) = backup { let _ = std::fs::rename(backup, source); }
        else if let Some(previous) = previous { let _ = create_link(source, &previous).await; }
        return Err(error);
    }
    let record = target.parent().unwrap().join("links.json");
    let mut sources = std::fs::read(&record).ok().and_then(|bytes| serde_json::from_slice::<Vec<PathBuf>>(&bytes).ok()).unwrap_or_default();
    if !sources.iter().any(|path| path == source) { sources.push(source.to_owned()); }
    std::fs::write(record, serde_json::to_vec(&sources)?)?;
    Ok(())
}

#[cfg(windows)]
async fn create_link(source: &Path, target: &Path) -> AppResult<()> {
    let output = tokio::time::timeout(std::time::Duration::from_secs(20), tokio::process::Command::new(shell()?).args(["-NoProfile", "-NonInteractive", "-Command", "$ErrorActionPreference='Stop'; New-Item -ItemType Junction -Path $env:LUXMC_BEDROCK_SOURCE -Target $env:LUXMC_BEDROCK_TARGET | Out-Null"]).env("LUXMC_BEDROCK_SOURCE", source).env("LUXMC_BEDROCK_TARGET", target).creation_flags(0x08000000).kill_on_drop(true).output()).await.map_err(|_| AppError::InvalidState("O Windows demorou para preparar a pasta Bedrock".into()))??;
    if !output.status.success() || source.canonicalize()? != target.canonicalize()? { return Err(AppError::InvalidState(format!("Não foi possível isolar a pasta Bedrock: {}", String::from_utf8_lossy(&output.stderr)))); }
    Ok(())
}

#[cfg(windows)]
fn valid_runtime_source(source: &Path) -> AppResult<bool> {
    let local = PathBuf::from(std::env::var_os("LOCALAPPDATA").ok_or_else(|| AppError::InvalidState("AppData indisponível".into()))?);
    for family in ["Microsoft.MinecraftUWP", "Microsoft.MinecraftWindowsBeta"] { if source == local.join("Packages").join(format!("{family}_8wekyb3d8bbwe/LocalState/games/com.mojang")) { return Ok(true); } }
    for preview in [false, true] { let users = gdk_users(preview)?; if let Ok(relative) = source.strip_prefix(users) { let components = relative.components().map(|component| component.as_os_str().to_string_lossy().into_owned()).collect::<Vec<_>>(); if components.len() == 3 && (components[0] == "Shared" || valid_account(&components[0])) && components[1] == "games" && components[2] == "com.mojang" { return Ok(true); } } }
    Ok(false)
}

pub(super) async fn detach(id: &str) -> AppResult<()> {
    let root = workspace(id)?;
    #[cfg(windows)] {
        if !root.exists() { return Ok(()); }
        let canonical = root.canonicalize()?;
        if !canonical.starts_with(super::bedrock_catalog::root()?.canonicalize()?) { return Err(AppError::InvalidInput("Pasta Bedrock inválida".into())); }
        let record = root.join("links.json");
        if let Ok(bytes) = std::fs::read(record) {
            let sources: Vec<PathBuf> = serde_json::from_slice(&bytes)?;
            for source in sources { if valid_runtime_source(&source)? && std::fs::read_link(&source).is_ok() && source.canonicalize().is_ok_and(|path| path.starts_with(&canonical)) { std::fs::remove_dir(source)?; } }
        }
    }
    #[cfg(not(windows))] let _ = root;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    #[test]
    fn rejects_false_expanded_sizes_without_importing_contents() {
        let fixture = tempfile::tempdir().unwrap();
        let archive_path = fixture.path().join("invalid.mcpack");
        let mut archive = zip::ZipWriter::new(std::fs::File::create(&archive_path).unwrap());
        archive.start_file("manifest.json", zip::write::FileOptions::default().compression_method(zip::CompressionMethod::Deflated)).unwrap();
        archive.write_all(br#"{"header":{"name":"Untrusted size"},"modules":[{"type":"resources"}]}"#).unwrap();
        archive.finish().unwrap();
        let mut bytes = std::fs::read(&archive_path).unwrap();
        let central = bytes.windows(4).position(|signature| signature == b"PK\x01\x02").unwrap();
        bytes[central + 24..central + 28].copy_from_slice(&0u32.to_le_bytes());
        std::fs::write(&archive_path, bytes).unwrap();
        let root = fixture.path().join("instance");
        assert!(extract_content(&archive_path, &root, "resources").is_err());
        assert!(!root.join("shared/resource_packs").exists());
        assert_eq!(std::fs::read_dir(root.join("imports")).unwrap().count(), 0);
    }
    #[test]
    fn rejects_archive_traversal_and_imports_valid_resource_pack() {
        let fixture = tempfile::tempdir().unwrap();
        let archive_path = fixture.path().join("sample.mcpack");
        let mut zip = zip::ZipWriter::new(std::fs::File::create(&archive_path).unwrap());
        zip.start_file("manifest.json", zip::write::FileOptions::default()).unwrap();
        zip.write_all(br#"{"header":{"name":"Meu pacote"},"modules":[{"type":"resources"}]}"#).unwrap();
        zip.finish().unwrap();
        let root = fixture.path().join("instance");
        extract_content(&archive_path, &root, "resources").unwrap();
        assert_eq!(content_list(&root).unwrap()[0].title, "Meu pacote");
        assert!(extract_content(&archive_path, &root, "worlds").is_err());
        let mut zip = zip::ZipWriter::new(std::fs::File::create(&archive_path).unwrap());
        zip.start_file("../outside.txt", zip::write::FileOptions::default()).unwrap();
        zip.write_all(b"invalid").unwrap(); zip.finish().unwrap();
        assert!(extract_content(&archive_path, &root, "resources").is_err());
        assert!(!fixture.path().join("outside.txt").exists());
        assert!(workspace("../bad").is_err());
    }
    #[cfg(windows)]
    #[tokio::test]
    async fn switching_junctions_keeps_worlds_in_their_own_instance() {
        let fixture = tempfile::tempdir().unwrap();
        let owned = fixture.path().join("instances");
        let one = owned.join("one/shared"); let two = owned.join("two/shared");
        let source = fixture.path().join("game/com.mojang");
        std::fs::create_dir_all(&source).unwrap();
        std::fs::write(source.join("world.txt"), "original world").unwrap();
        link_data(&source, &one, &owned).await.unwrap();
        assert_eq!(std::fs::read_to_string(one.join("world.txt")).unwrap(), "original world");
        link_data(&source, &two, &owned).await.unwrap();
        assert!(one.join("world.txt").exists()); assert!(!two.join("world.txt").exists());
        std::fs::write(source.join("new-world.txt"), "new world").unwrap();
        link_data(&source, &one, &owned).await.unwrap();
        assert!(!source.join("new-world.txt").exists()); assert!(two.join("new-world.txt").exists());
        std::fs::remove_dir(&source).unwrap();
    }
}
