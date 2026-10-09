use crate::error::{AppError, AppResult};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

#[derive(Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BedrockProvider {
    pub executable: String,
    pub data_directory: String,
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BedrockInstallation {
    pub id: String,
    pub profile_id: String,
    pub profile_name: String,
    pub name: String,
    pub version: String,
    pub directory: String,
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BedrockInstance {
    pub id: String,
    pub name: String,
    pub installation_id: String,
    pub profile_id: String,
}

#[derive(Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BedrockState {
    pub provider: Option<BedrockProvider>,
    pub installations: Vec<BedrockInstallation>,
    pub instances: Vec<BedrockInstance>,
    pub warning: Option<String>,
    pub supported: bool,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BedrockOpenResult { pub provider_opened: bool, pub notice: Option<String> }

#[derive(Default, Serialize, Deserialize)]
struct SavedState {
    provider: Option<BedrockProvider>,
    instances: Vec<BedrockInstance>,
    #[serde(default)]
    managed: Vec<ManagedInstallation>,
}

#[derive(Clone, Serialize, Deserialize)]
struct ManagedInstallation {
    installation: BedrockInstallation,
    package: String,
    package_version: String,
    channel: String,
    app_id: String,
}

pub(super) async fn register_managed(version: super::bedrock_catalog::BedrockVersion, path: PathBuf, app_id: String, name: String) -> AppResult<()> {
    let name = name.trim();
    if name.chars().count()>80 {return Err(AppError::InvalidInput("Use um nome de até 80 caracteres".into()));}
    let name = if name.is_empty() {format!("Bedrock {}",version.version)}else{name.into()};
    let _guard=STATE_LOCK.lock().await;
    let mut state=load()?;
    state.managed.retain(|row|row.installation.id!=version.id);
    state.managed.push(ManagedInstallation {installation:BedrockInstallation {id:version.id.clone(),profile_id:"managed".into(),profile_name:"Luxmc".into(),name:name.clone(),version:version.version,directory:path.parent().unwrap().to_string_lossy().into_owned()},package:path.to_string_lossy().into_owned(),package_version:version.package_version,channel:version.channel,app_id});
    if !state.instances.iter().any(|row|row.installation_id==version.id&&row.profile_id=="managed") {
        state.instances.push(BedrockInstance{id:uuid::Uuid::new_v4().to_string(),name,installation_id:version.id,profile_id:"managed".into()});
    }
    save(&state)
}

static STATE_LOCK: tokio::sync::Mutex<()> = tokio::sync::Mutex::const_new(());

fn state_file() -> AppResult<PathBuf> {
    let dirs = directories::ProjectDirs::from("io", "github", "Luxmc")
        .ok_or_else(|| AppError::InvalidState("Diretório de dados indisponível".into()))?;
    Ok(dirs.data_dir().join("bedrock-provider.json"))
}

fn read_json(path: &Path) -> AppResult<serde_json::Value> {
    let file = std::fs::File::open(path)?;
    if file.metadata()?.len() > 4 * 1024 * 1024 {
        return Err(AppError::InvalidInput("A configuração do provedor excede 4 MB".into()));
    }
    Ok(serde_json::from_reader(file)?)
}

fn load() -> AppResult<SavedState> {
    let path = state_file()?;
    if !path.exists() { return Ok(SavedState::default()); }
    Ok(serde_json::from_value(read_json(&path)?)?)
}

fn save(state: &SavedState) -> AppResult<()> {
    let path = state_file()?;
    std::fs::create_dir_all(path.parent().unwrap())?;
    let temporary = path.with_extension(format!("{}.tmp", uuid::Uuid::new_v4()));
    std::fs::write(&temporary, serde_json::to_vec(state)?)?;
    std::fs::rename(temporary, path)?;
    Ok(())
}

fn safe_directory(root: &Path, profile: &str, installation: &str) -> AppResult<PathBuf> {
    for part in [profile, installation] {
        if part.is_empty() || part.contains(['\\', '/', ':', '\0']) || part == "." || part == ".." {
            return Err(AppError::InvalidInput("Diretório inválido no BedrockLauncher".into()));
        }
    }
    let path = root.join("installations").join(profile).join(installation);
    if path.exists() && !path.canonicalize()?.starts_with(root.canonicalize()?) {
        return Err(AppError::InvalidInput("A instalação aponta para fora do provedor".into()));
    }
    Ok(path)
}

fn parse_installations(root: &Path, value: &serde_json::Value) -> AppResult<Vec<BedrockInstallation>> {
    let profiles = value.get("profiles").and_then(|value| value.as_object())
        .ok_or_else(|| AppError::InvalidInput("Selecione a pasta que contém user_profile.json do BedrockLauncher".into()))?;
    let mut result = Vec::new();
    for (profile_id, profile) in profiles {
        let Some(profile_path) = profile.get("ProfilePath").and_then(|v| v.as_str()) else { continue; };
        let Some(items) = profile.get("Installations").and_then(|v| v.as_array()) else { continue; };
        for item in items {
            let text = |key| item.get(key).and_then(|v| v.as_str()).unwrap_or("");
            let id = text("InstallationUUID");
            let name = text("DisplayName");
            if id.is_empty() || name.is_empty() { continue; }
            let directory = safe_directory(root, profile_path, if text("DirectoryName").is_empty() { name } else { text("DirectoryName") })?;
            result.push(BedrockInstallation {
                id: id.into(), profile_id: profile_id.clone(),
                profile_name: profile.get("Name").and_then(|v| v.as_str()).unwrap_or(profile_id).into(),
                name: name.into(), version: match item.get("VersioningMode").and_then(|v| v.as_i64()) {
                    Some(0) => "Preview mais recente".into(),
                    Some(1) => "Beta mais recente".into(),
                    Some(2) => "Versão mais recente".into(),
                    _ => text("VersionUUID").into(),
                }, directory: directory.to_string_lossy().into_owned(),
            });
        }
    }
    result.sort_by(|a, b| a.name.cmp(&b.name));
    Ok(result)
}

fn catalog(provider: &BedrockProvider) -> AppResult<Vec<BedrockInstallation>> {
    let root = Path::new(&provider.data_directory);
    parse_installations(root, &read_json(&root.join("user_profile.json"))?)
}

fn native_installations(value: serde_json::Value) -> Vec<BedrockInstallation> {
    let items = match value { serde_json::Value::Array(items) => items, serde_json::Value::Object(_) => vec![value], _ => Vec::new() };
    items.into_iter().filter_map(|item| {
        let id = item.get("AppID")?.as_str()?;
        let family = id.split_once('!')?.0;
        if !["Microsoft.MinecraftUWP_8wekyb3d8bbwe", "Microsoft.MinecraftWindowsBeta_8wekyb3d8bbwe"].contains(&family)
            || !id.bytes().all(|byte| byte.is_ascii_alphanumeric() || b"._!-".contains(&byte)) { return None; }
        Some(BedrockInstallation { id: id.into(), profile_id: "windows".into(), profile_name: "Windows".into(),
            name: item.get("Name").and_then(|v| v.as_str()).unwrap_or("Minecraft Bedrock").into(),
            version: "Versão instalada no Windows".into(), directory: String::new() })
    }).collect()
}

fn windows_catalog() -> AppResult<Vec<BedrockInstallation>> {
    #[cfg(windows)] {
        use std::os::windows::process::CommandExt;
        let powershell = PathBuf::from(std::env::var_os("SystemRoot").ok_or_else(|| AppError::InvalidState("Windows indisponível".into()))?).join("System32/WindowsPowerShell/v1.0/powershell.exe");
        let output = std::process::Command::new(powershell).args(["-NoProfile", "-NonInteractive", "-Command", "[Console]::OutputEncoding = [System.Text.Encoding]::UTF8; Get-StartApps | Where-Object { $_.AppID -like 'Microsoft.MinecraftUWP_8wekyb3d8bbwe!*' -or $_.AppID -like 'Microsoft.MinecraftWindowsBeta_8wekyb3d8bbwe!*' } | Select-Object Name,AppID | ConvertTo-Json -Compress"]).creation_flags(0x08000000).output()?;
        if !output.status.success() { return Err(AppError::InvalidState("Não foi possível consultar o Bedrock instalado no Windows. Use Atualizar para tentar novamente.".into())); }
        if output.stdout.iter().all(u8::is_ascii_whitespace) { return Ok(Vec::new()); }
        Ok(native_installations(serde_json::from_slice(&output.stdout)?))
    }
    #[cfg(not(windows))] { Ok(Vec::new()) }
}

#[tauri::command]
pub async fn bedrock_state() -> AppResult<BedrockState> {
    let _guard = STATE_LOCK.lock().await;
    tokio::task::spawn_blocking(|| {
        let saved = load()?;
        let (mut installations, mut warning) = match &saved.provider {
            Some(provider) => match catalog(provider) {
                Ok(items) => (items, None), Err(error) => (Vec::new(), Some(error.to_string())),
            }, None => (Vec::new(), None),
        };
        match windows_catalog() { Ok(mut native) => { native.append(&mut installations); installations = native; }, Err(error) => warning = Some(error.to_string()) }
        installations.extend(saved.managed.iter().map(|row|row.installation.clone()));
        Ok(BedrockState { provider: saved.provider, instances: saved.instances, installations, warning, supported: cfg!(target_os = "windows") })
    }).await.map_err(|error| AppError::Internal(error.to_string()))?
}

#[tauri::command]
pub async fn bedrock_connect(executable: String, data_directory: String) -> AppResult<()> {
    if !cfg!(target_os = "windows") { return Err(AppError::InvalidState("O BedrockLauncher requer Windows".into())); }
    let _guard = STATE_LOCK.lock().await;
    tokio::task::spawn_blocking(move || {
        let path = Path::new(&executable).canonicalize()?;
        if !path.is_file() || !path.file_name().is_some_and(|name| name.to_string_lossy().eq_ignore_ascii_case("BedrockLauncher.exe")) {
            return Err(AppError::InvalidInput("Selecione BedrockLauncher.exe".into()));
        }
        let provider = BedrockProvider { executable: path.to_string_lossy().into_owned(), data_directory: Path::new(&data_directory).canonicalize()?.to_string_lossy().into_owned() };
        catalog(&provider)?;
        let mut state = load()?;
        if state.provider.as_ref().is_some_and(|old| old.data_directory != provider.data_directory) && !state.instances.is_empty() {
            return Err(AppError::InvalidState("Remova os vínculos existentes antes de trocar a pasta do provedor".into()));
        }
        state.provider = Some(provider);
        save(&state)
    }).await.map_err(|error| AppError::Internal(error.to_string()))?
}

#[tauri::command]
pub async fn bedrock_add(name: String, profile_id: String, installation_id: String) -> AppResult<()> {
    let name = name.trim().to_string();
    if name.is_empty() || name.chars().count() > 80 { return Err(AppError::InvalidInput("Use um nome de 1 a 80 caracteres".into())); }
    let _guard = STATE_LOCK.lock().await;
    tokio::task::spawn_blocking(move || {
        let mut state = load()?;
        if profile_id=="managed" {
            if !state.managed.iter().any(|row|row.installation.id==installation_id){return Err(AppError::NotFound("Versão Bedrock não encontrada".into()));}
            if state.instances.iter().any(|row|row.installation_id==installation_id&&row.profile_id==profile_id){return Err(AppError::InvalidInput("Essa versão já está na biblioteca".into()));}
            state.instances.push(BedrockInstance{id:uuid::Uuid::new_v4().to_string(),name,profile_id,installation_id});return save(&state);
        }
        if profile_id == "windows" {
            let selected = windows_catalog()?.into_iter().find(|item| item.id == installation_id).ok_or_else(|| AppError::NotFound("Instale o Minecraft Bedrock no Windows e atualize a lista.".into()))?;
            if state.instances.iter().any(|item| item.installation_id == selected.id && item.profile_id == "windows") { return Err(AppError::InvalidInput("Esse Bedrock já está na biblioteca".into())); }
            state.instances.push(BedrockInstance { id: uuid::Uuid::new_v4().to_string(), name, profile_id, installation_id });
            return save(&state);
        }
        let provider = state.provider.as_ref().ok_or_else(|| AppError::InvalidState("Conecte o BedrockLauncher primeiro".into()))?;
        let installations = catalog(provider)?;
        let selected = installations.iter().find(|item| item.id == installation_id && item.profile_id == profile_id)
            .ok_or_else(|| AppError::NotFound("A instalação não existe mais no provedor".into()))?;
        if state.instances.iter().any(|item| item.installation_id == installation_id && item.profile_id == profile_id) {
            return Err(AppError::InvalidInput("Essa instalação já está na biblioteca".into()));
        }
        if installations.iter().any(|item| (item.id != selected.id || item.profile_id != selected.profile_id) && item.directory.eq_ignore_ascii_case(&selected.directory)) {
            return Err(AppError::InvalidInput("Duas instalações do provedor usam a mesma pasta. Separe-as no BedrockLauncher antes de importar".into()));
        }
        state.instances.push(BedrockInstance { id: uuid::Uuid::new_v4().to_string(), name, profile_id, installation_id });
        save(&state)
    }).await.map_err(|error| AppError::Internal(error.to_string()))?
}

#[tauri::command]
pub async fn bedrock_remove(id: String) -> AppResult<()> {
    let _installation = super::bedrock_catalog::INSTALL_LOCK.try_lock().map_err(|_| AppError::InvalidState("Aguarde a instalação Bedrock terminar antes de excluir.".into()))?;
    let _guard = STATE_LOCK.lock().await;
    let mut state = load()?;
    let instance = state.instances.iter().find(|instance| instance.id == id).cloned()
        .ok_or_else(|| AppError::NotFound("Instância Bedrock não encontrada".into()))?;
    if instance.profile_id == "managed" {
        let managed = state.managed.iter().find(|item| item.installation.id == instance.installation_id).cloned()
            .ok_or_else(|| AppError::NotFound("Arquivos da versão Bedrock não encontrados".into()))?;
        let root = state_file()?.parent().unwrap().join("bedrock").join("packages");
        let directory = PathBuf::from(&managed.installation.directory);
        if directory.exists() {
            let canonical_root = tokio::fs::canonicalize(&root).await?;
            let canonical = tokio::fs::canonicalize(&directory).await?;
            if canonical == canonical_root || !canonical.starts_with(&canonical_root) {
                return Err(AppError::InvalidInput("A pasta da versão está fora dos pacotes do Luxmc. Seus documentos foram preservados.".into()));
            }
        }
        remove_windows_package(&managed.app_id, Some(&managed.package_version)).await?;
        if directory.exists() {
            let canonical_root = tokio::fs::canonicalize(&root).await?;
            let canonical = tokio::fs::canonicalize(&directory).await?;
            if canonical == canonical_root || !canonical.starts_with(&canonical_root) { return Err(AppError::InvalidInput("Pasta Bedrock inválida".into())); }
            tokio::fs::remove_dir_all(canonical).await?;
        }
        state.managed.retain(|item| item.installation.id != instance.installation_id);
        state.instances.retain(|item| item.profile_id != "managed" || item.installation_id != instance.installation_id);
    } else if instance.profile_id == "windows" {
        remove_windows_package(&instance.installation_id, None).await?;
        state.instances.retain(|item| item.id != id);
    } else {
        return Err(AppError::InvalidState("Esta versão pertence a um provedor externo. Exclua a instalação nesse provedor; o Luxmc não apagou arquivos externos nem seus documentos.".into()));
    }
    save(&state)
}

async fn remove_windows_package(app_id: &str, version: Option<&str>) -> AppResult<()> {
    let family = app_id.split('_').next().unwrap_or_default();
    if !matches!(family, "Microsoft.MinecraftUWP" | "Microsoft.MinecraftWindowsBeta") || version.is_some_and(|value| value.is_empty() || value.len() > 32 || !value.bytes().all(|byte| byte.is_ascii_digit() || byte == b'.')) {
        return Err(AppError::InvalidInput("Identidade do pacote Bedrock inválida".into()));
    }
    let shell = PathBuf::from(std::env::var_os("SystemRoot").ok_or_else(|| AppError::InvalidState("Windows indisponível".into()))?).join("System32/WindowsPowerShell/v1.0/powershell.exe");
    let mut command = tokio::process::Command::new(shell);
    command.env("LUXMC_REMOVE_FAMILY", family).env("LUXMC_REMOVE_VERSION", version.unwrap_or_default())
        .args(["-NoProfile", "-NonInteractive", "-Command", "$ErrorActionPreference='Stop'; if(Get-Process -Name Minecraft.Windows,Minecraft.WindowsBeta,gamingservicesui -ErrorAction SilentlyContinue | Where-Object {$_.MainWindowTitle -like 'Minecraft*'}){throw 'Feche o Minecraft Bedrock antes de desinstalar.'}; $packages=@(Get-AppxPackage -Name $env:LUXMC_REMOVE_FAMILY | Where-Object {!$env:LUXMC_REMOVE_VERSION -or $_.Version.ToString() -eq $env:LUXMC_REMOVE_VERSION}); foreach($package in $packages){Remove-AppxPackage -Package $package.PackageFullName -ErrorAction Stop}; if(Get-AppxPackage -Name $env:LUXMC_REMOVE_FAMILY | Where-Object {!$env:LUXMC_REMOVE_VERSION -or $_.Version.ToString() -eq $env:LUXMC_REMOVE_VERSION}){throw 'O pacote ainda está instalado no Windows.'}"])
        .kill_on_drop(true);
    #[cfg(windows)] command.creation_flags(0x08000000);
    let output = tokio::time::timeout(std::time::Duration::from_secs(180), command.output()).await
        .map_err(|_| AppError::InvalidState("O Windows demorou para desinstalar. Atualize a lista antes de tentar novamente.".into()))??;
    if !output.status.success() { return Err(AppError::InvalidState(format!("O Windows não concluiu a desinstalação: {}", String::from_utf8_lossy(&output.stderr).chars().take(1000).collect::<String>()))); }
    Ok(())
}

#[tauri::command]
pub async fn bedrock_rename(id: String, name: String) -> AppResult<()> {
    let name = name.trim();
    if name.is_empty() || name.chars().count() > 80 || name.chars().any(char::is_control) { return Err(AppError::InvalidInput("Use um nome de até 80 caracteres".into())); }
    let _guard = STATE_LOCK.lock().await;
    let mut state = load()?;
    let instance = state.instances.iter_mut().find(|instance| instance.id == id)
        .ok_or_else(|| AppError::NotFound("Instância Bedrock não encontrada".into()))?;
    instance.name = name.into();
    save(&state)
}

#[tauri::command]
pub async fn bedrock_open(instance_id: Option<String>) -> AppResult<BedrockOpenResult> {
    if !cfg!(target_os = "windows") { return Err(AppError::InvalidState("O BedrockLauncher requer Windows".into())); }
    let state = { let _guard = STATE_LOCK.lock().await; load()? };
    if let Some(id)=&instance_id {
        if let Some(instance)=state.instances.iter().find(|row|&row.id==id&&row.profile_id=="managed") {
            let _guard=super::bedrock_catalog::INSTALL_LOCK.try_lock().map_err(|_|AppError::InvalidState("Aguarde a instalação Bedrock em andamento".into()))?;
            let installation=state.managed.iter().find(|row|row.installation.id==instance.installation_id).ok_or_else(||AppError::NotFound("Pacote Bedrock não encontrado".into()))?;
            if installation.package.ends_with(".msixvc") && native_preparation_active().await? {return Ok(BedrockOpenResult {provider_opened:false,notice:Some("O Windows já está preparando o Bedrock. Aguarde essa janela terminar antes de clicar em Jogar novamente.".into())});}
            let id=super::bedrock_catalog::deploy(PathBuf::from(&installation.package),installation.package_version.clone(),installation.channel.clone()).await?;
            let explorer=PathBuf::from(std::env::var_os("SystemRoot").ok_or_else(||AppError::InvalidState("Windows indisponível".into()))?).join("explorer.exe");
            tokio::process::Command::new(explorer).arg(format!("shell:AppsFolder\\{id}")).spawn()?;
            return Ok(BedrockOpenResult {provider_opened:false,notice:installation.package.ends_with(".msixvc").then(||"O Gaming Services pode preparar arquivos na primeira abertura. Aguarde a janela do Windows; se for solicitado, entre na conta que possui Minecraft.".into())});
        }
    }
    if let Some(id) = &instance_id {
        if let Some(instance) = state.instances.iter().find(|item| &item.id == id && item.profile_id == "windows") {
            let installation_id = instance.installation_id.clone();
            let exists = tokio::task::spawn_blocking(move || windows_catalog().map(|items| items.into_iter().any(|item| item.id == installation_id))).await.map_err(|error| AppError::Internal(error.to_string()))??;
            if !exists { return Err(AppError::NotFound("O Minecraft Bedrock não está instalado neste usuário do Windows.".into())); }
            let explorer = PathBuf::from(std::env::var_os("SystemRoot").ok_or_else(|| AppError::InvalidState("Windows indisponível".into()))?).join("explorer.exe");
            tokio::process::Command::new(explorer).arg(format!("shell:AppsFolder\\{}", instance.installation_id)).spawn()?;
            return Ok(BedrockOpenResult { provider_opened: false, notice: None });
        }
    }
    let provider = state.provider.ok_or_else(|| AppError::InvalidState("Conecte o BedrockLauncher primeiro".into()))?;
    let mut command = tokio::process::Command::new(&provider.executable);
    command.current_dir(Path::new(&provider.executable).parent().unwrap());
    let launching = instance_id.is_some();
    if let Some(id) = instance_id {
        let instance = state.instances.iter().find(|item| item.id == id).ok_or_else(|| AppError::NotFound("Instância Bedrock não encontrada".into()))?;
        let items = catalog(&provider)?;
        let item = items.iter().find(|item| item.profile_id == instance.profile_id && item.id == instance.installation_id)
            .ok_or_else(|| AppError::NotFound("A instalação foi removida do BedrockLauncher".into()))?;
        command.arg("--launch").arg(&item.profile_id).arg(&item.name);
    }
    let mut child = command.spawn()?;
    if let Ok(result) = tokio::time::timeout(std::time::Duration::from_secs(3), child.wait()).await {
        let status = result?;
        if launching {
            tokio::process::Command::new(&provider.executable).current_dir(Path::new(&provider.executable).parent().unwrap()).spawn()?;
            return Ok(BedrockOpenResult { provider_opened: true, notice: Some("O provedor não confirmou o comando de abertura. Selecione sua instalação no BedrockLauncher e clique em Jogar.".into()) });
        }
        if !status.success() { return Err(AppError::InvalidState("O BedrockLauncher encerrou com erro. Confira a instalação do provedor".into())); }
    }
    Ok(BedrockOpenResult { provider_opened: !launching, notice: None })
}

async fn native_preparation_active() -> AppResult<bool> {
    let shell=PathBuf::from(std::env::var_os("SystemRoot").ok_or_else(||AppError::InvalidState("Windows indisponível".into()))?).join("System32/WindowsPowerShell/v1.0/powershell.exe");
    let mut query=tokio::process::Command::new(shell);
    query.args(["-NoProfile","-NonInteractive","-Command","if(Get-Process -Name gamingservicesui -ErrorAction SilentlyContinue | Where-Object { $_.MainWindowTitle -like 'Minecraft*' }){Write-Output 'preparing'}"]).kill_on_drop(true);
    #[cfg(windows)] query.creation_flags(0x08000000);
    let output=tokio::time::timeout(std::time::Duration::from_secs(20),query.output()).await.map_err(|_|AppError::InvalidState("O Windows não respondeu à consulta de abertura".into()))??;
    Ok(output.status.success() && String::from_utf8_lossy(&output.stdout).trim()=="preparing")
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn native_catalog_only_accepts_minecraft_package_identities() {
        let result = native_installations(serde_json::json!([
            {"AppID":"Microsoft.MinecraftUWP_8wekyb3d8bbwe!App","Name":"Minecraft"},
            {"AppID":"unrelated!App","Name":"Minecraft"},
            {"AppID":"Microsoft.MinecraftUWP_8wekyb3d8bbwe!App;calc","Name":"Minecraft"}
        ]));
        assert_eq!(result.len(), 1); assert_eq!(result[0].profile_id, "windows");
    }
    #[test]
    fn imports_only_installation_metadata_and_preserves_profile_key() {
        let value = serde_json::json!({"profiles":{"profile-id":{"Name":"Player","MicrosoftAccountId":"private","ProfilePath":"Player","Installations":[{"InstallationUUID":"one","DisplayName":"Bedrock 1.20","DirectoryName":"one","VersionUUID":"exact-version-id"}]}}});
        let items = parse_installations(Path::new("."), &value).unwrap();
        assert_eq!(items.len(), 1);
        assert_eq!(items[0].profile_id, "profile-id");
        assert_eq!(items[0].version, "exact-version-id");
        assert!(!serde_json::to_string(&items).unwrap().contains("private"));
    }
    #[test]
    fn rejects_provider_paths_that_escape_or_are_absolute() {
        for part in ["..", "../other", "C:\\game", "/game", "a\\b", ""] {
            assert!(safe_directory(Path::new("."), part, "instance").is_err());
            assert!(safe_directory(Path::new("."), "profile", part).is_err());
        }
    }
}
