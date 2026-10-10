use crate::error::{AppError, AppResult};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use tauri::Emitter;

#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BedrockVersion {
    pub id: String,
    pub version: String,
    pub channel: String,
    pub package_type: String,
    pub package_version: String,
    pub update_id: Option<String>,
    pub urls: Vec<String>,
}

pub(super) static INSTALL_LOCK: tokio::sync::Mutex<()> = tokio::sync::Mutex::const_new(());
static CANCEL: AtomicBool = AtomicBool::new(false);

pub(super) fn root() -> AppResult<PathBuf> {
    Ok(directories::ProjectDirs::from("io", "github", "Luxmc").ok_or_else(|| AppError::InvalidState("Dados do Bedrock indisponíveis".into()))?.data_dir().join("bedrock"))
}

fn http() -> AppResult<reqwest::Client> {
    reqwest::Client::builder().connect_timeout(std::time::Duration::from_secs(20))
        .tls_built_in_native_certs(true).user_agent("Luxmc/3.6.0")
        .redirect(reqwest::redirect::Policy::custom(|attempt| {
            if attempt.previous().len() > 5 || !microsoft_package_url(attempt.url()) { attempt.stop() } else { attempt.follow() }
        })).build().map_err(|error| AppError::Internal(error.to_string()))
}

fn microsoft_url(url: &url::Url) -> bool {
    url.scheme() == "https" && url.username().is_empty() && url.password().is_none() && url.port().is_none()
        && url.host_str().is_some_and(|host| host.ends_with(".delivery.mp.microsoft.com") || ["assets1.xboxlive.com", "assets2.xboxlive.com", "d1.xboxlive.com", "d2.xboxlive.com"].contains(&host))
}

fn microsoft_package_url(url: &url::Url) -> bool {
    if microsoft_url(url) {return true;}
    if url.scheme()!="http" {return false;}
    let mut secured=url.clone();
    secured.set_scheme("https").is_ok() && microsoft_url(&secured)
}

fn safe_url(value: &str) -> Option<String> {
    let mut parsed = url::Url::parse(value).ok()?;
    if parsed.scheme() == "http" { parsed.set_scheme("https").ok()?; }
    microsoft_url(&parsed).then(|| parsed.to_string())
}

fn package_version(version: &str) -> Option<String> {
    let parts = version.split('.').map(str::parse::<u32>).collect::<Result<Vec<_>,_>>().ok()?;
    if parts.len() != 4 || parts.iter().any(|value| *value > 65535) { return None; }
    let patch = parts[2].checked_mul(100)?.checked_add(parts[3])?;
    (patch <= 65535).then(|| format!("{}.{}.{}.0", parts[0], parts[1], patch))
}

fn parse_catalog(legacy: serde_json::Value, gdk: serde_json::Value) -> Vec<BedrockVersion> {
    let mut versions = Vec::new();
    if let Some(rows) = legacy.as_array() {
        for row in rows {
            let Some(values) = row.as_array() else { continue };
            if values.len() < 4 || values[3].as_str() != Some("x64") { continue; }
            let Some(version) = values[0].as_str() else { continue };
            let Some(package_version) = package_version(version) else { continue };
            let Some(update) = values[1].as_str().filter(|value| uuid::Uuid::parse_str(value).is_ok()) else { continue };
            let channel = match values[2].as_str() { Some("0") => "release", Some("1") => "beta", Some("2") => "preview", _ => continue };
            versions.push(BedrockVersion { id: format!("uwp-{update}"), version: version.into(), channel: channel.into(), package_type: "UWP".into(), package_version, update_id: Some(update.into()), urls: Vec::new() });
        }
    }
    for channel in ["release", "preview"] {
        if let Some(entries) = gdk.get(channel).and_then(|value| value.as_object()) {
            for (version, candidates) in entries {
                let Some(package_version) = package_version(version) else { continue };
                let family = if channel == "preview" { "microsoft.minecraftwindowsbeta" } else { "microsoft.minecraftuwp" };
                let expected = format!("{family}_{package_version}_x64__8wekyb3d8bbwe.msixvc");
                let urls = candidates.as_array().into_iter().flatten().filter_map(|value| value.as_str()).filter_map(safe_url)
                    .filter(|value| url::Url::parse(value).ok().and_then(|url| url.path_segments().and_then(|mut parts| parts.next_back()).map(str::to_ascii_lowercase)).is_some_and(|name| name == expected)).collect::<Vec<_>>();
                if urls.is_empty() { continue; }
                versions.push(BedrockVersion { id: format!("gdk-{channel}-{version}"), version: version.clone(), channel: channel.into(), package_type: "GDK".into(), package_version, update_id: None, urls });
            }
        }
    }
    let mut identities=std::collections::HashSet::new();
    versions.retain(|version|identities.insert(version.id.clone()));
    versions.sort_by_cached_key(|value| std::cmp::Reverse(value.version.split('.').filter_map(|part| part.parse::<u32>().ok()).collect::<Vec<_>>()));
    versions
}

async fn bounded_json(client: &reqwest::Client, address: &str) -> AppResult<serde_json::Value> {
    use futures_util::StreamExt;
    let response = client.get(address).send().await?.error_for_status()?;
    let mut stream = response.bytes_stream();
    let mut bytes = Vec::new();
    while let Some(chunk) = tokio::time::timeout(std::time::Duration::from_secs(20), stream.next()).await.map_err(|_| AppError::InvalidState("O catálogo Bedrock não respondeu".into()))? {
        let chunk = chunk?;
        if bytes.len() + chunk.len() > 4 * 1024 * 1024 { return Err(AppError::InvalidInput("Catálogo Bedrock acima do limite".into())); }
        bytes.extend_from_slice(&chunk);
    }
    Ok(serde_json::from_slice(&bytes)?)
}

#[tauri::command]
pub async fn bedrock_versions() -> AppResult<Vec<BedrockVersion>> {
    let cache = root()?.join("catalog.json");
    let client = reqwest::Client::builder().timeout(std::time::Duration::from_secs(25)).user_agent("Luxmc/3.6.0").build()?;
    let result = tokio::try_join!(bounded_json(&client,"https://www.raythnetwork.co.uk/versions.php?type=json"), bounded_json(&client,"https://www.raythnetwork.co.uk/gdk.urls.min.json"));
    match result {
        Ok((legacy,gdk)) => {
            let versions = parse_catalog(legacy,gdk);
            if versions.is_empty() { return Err(AppError::InvalidState("O catálogo retornou uma lista vazia".into())); }
            super::super::core::mods::pack_download::atomic_write(&cache, &serde_json::to_vec(&versions)?).await?;
            Ok(versions)
        },
        Err(error) => match tokio::fs::read(cache).await { Ok(bytes) => Ok(serde_json::from_slice(&bytes)?), Err(_) => Err(error) }
    }
}

async fn uwp_urls(client: &reqwest::Client, update: &str) -> AppResult<Vec<String>> {
    let update = uuid::Uuid::parse_str(update).map_err(|_| AppError::InvalidInput("Versão Bedrock inválida".into()))?;
    let namespace = "http://www.microsoft.com/SoftwareDistribution/Server/ClientWebService";
    let address = "https://fe3.delivery.mp.microsoft.com/ClientWebService/client.asmx/secured";
    let xml = format!(r#"<s:Envelope xmlns:s="http://www.w3.org/2003/05/soap-envelope" xmlns:a="http://www.w3.org/2005/08/addressing" xmlns:u="http://docs.oasis-open.org/wss/2004/01/oasis-200401-wss-wssecurity-utility-1.0.xsd"><s:Header><a:Action s:mustUnderstand="1">{namespace}/GetExtendedUpdateInfo2</a:Action><a:MessageID>urn:uuid:{message}</a:MessageID><a:To s:mustUnderstand="1">{address}</a:To><Security s:mustUnderstand="1" xmlns="http://docs.oasis-open.org/wss/2004/01/oasis-200401-wss-wssecurity-secext-1.0.xsd"><u:Timestamp><u:Created>{created}</u:Created><u:Expires>{expires}</u:Expires></u:Timestamp><WindowsUpdateTicketsToken xmlns="http://schemas.microsoft.com/msus/2014/10/WindowsUpdateAuthorization" u:id="ClientMSA"><TicketType Name="AAD" Version="1.0" Policy="MBI_SSL" /></WindowsUpdateTicketsToken></Security></s:Header><s:Body><GetExtendedUpdateInfo2 xmlns="{namespace}"><updateIDs><UpdateIdentity><UpdateID>{update}</UpdateID><RevisionNumber>1</RevisionNumber></UpdateIdentity></updateIDs><infoTypes><XmlUpdateFragmentType>FileUrl</XmlUpdateFragmentType></infoTypes><deviceAttributes>OSVersion=10.0.22631.0&amp;OSArchitecture=AMD64&amp;FlightRing=Retail&amp;DeviceFamily=Windows.Desktop</deviceAttributes></GetExtendedUpdateInfo2></s:Body></s:Envelope>"#, message=uuid::Uuid::new_v4(), created=chrono::Utc::now().to_rfc3339(), expires=(chrono::Utc::now()+chrono::Duration::hours(1)).to_rfc3339());
    let response = client.post(address).header("Content-Type","application/soap+xml; charset=utf-8").body(xml).send().await?.error_for_status()?;
    if !microsoft_url(response.url()) { return Err(AppError::InvalidInput("Servidor Bedrock não confiável".into())); }
    let bytes = response.bytes().await?;
    if bytes.len() > 1024*1024 { return Err(AppError::InvalidInput("Metadados Bedrock acima do limite".into())); }
    let body = std::str::from_utf8(&bytes).map_err(|_| AppError::InvalidInput("Metadados inválidos".into()))?;
    parse_store_urls(body)
}

fn parse_store_urls(body: &str) -> AppResult<Vec<String>> {
    use quick_xml::events::Event;
    let mut reader = quick_xml::Reader::from_str(body);
    let mut urls = Vec::new();
    loop {
        match reader.read_event().map_err(|error| AppError::InvalidInput(error.to_string()))? {
            Event::Start(element) if element.local_name().as_ref()==b"Url" => {
                if let Ok(text)=reader.read_text(element.name()) { if let Ok(decoded)=text.decode() { if let Ok(value) = quick_xml::escape::unescape(&decoded) { if let Some(url) = safe_url(&value) { urls.push(url); } } } }
            },
            Event::Eof => break,
            _ => {}
        }
    }
    if urls.is_empty() { return Err(AppError::NotFound("A Microsoft não disponibilizou o pacote dessa versão. Betas podem exigir inscrição no programa Insider.".into())); }
    Ok(urls)
}

pub(super) async fn deploy(path: PathBuf, version: String, channel: String) -> AppResult<String> {
    #[cfg(windows)] {
        let canonical = path.canonicalize()?;
        if !canonical.starts_with(root()?.canonicalize()?) || !canonical.is_file() { return Err(AppError::InvalidInput("Pacote fora da pasta Bedrock".into())); }
        if canonical.extension().is_some_and(|value|value=="appx") {
            validate_appx(&canonical,&version,&channel)?;
        }
        let family = if channel == "preview" || channel == "beta" { "Microsoft.MinecraftWindowsBeta" } else { "Microsoft.MinecraftUWP" };
        let shell = PathBuf::from(std::env::var_os("SystemRoot").ok_or_else(|| AppError::InvalidState("Windows indisponível".into()))?).join("System32/WindowsPowerShell/v1.0/powershell.exe");
        let mut query=tokio::process::Command::new(&shell);
        query.args(["-NoProfile","-NonInteractive","-Command","$package=Get-AppxPackage -Name $env:LUXMC_BEDROCK_FAMILY; if($package){ Write-Output $package.Version.ToString() }"]).env("LUXMC_BEDROCK_FAMILY",family).creation_flags(0x08000000).kill_on_drop(true);
        let installed=tokio::time::timeout(std::time::Duration::from_secs(30),query.output()).await.map_err(|_|AppError::InvalidState("O Windows não respondeu à consulta de versão".into()))??;
        if !installed.status.success(){return Err(AppError::InvalidState("Não foi possível conferir a versão instalada".into()));}
        if String::from_utf8_lossy(&installed.stdout).trim()!=version {
            ensure_store_framework(&shell).await?;
            if canonical.extension().is_some_and(|value|value=="msixvc") {ensure_gaming_services(&shell).await?;}
            tokio::task::spawn_blocking(backup_worlds).await.map_err(|error|AppError::Internal(error.to_string()))??;
        }
        let mut command = tokio::process::Command::new(shell);
        command.args(["-NoProfile","-NonInteractive","-Command", "$ErrorActionPreference='Stop'; [Console]::OutputEncoding=[System.Text.Encoding]::UTF8; $package=Get-AppxPackage -Name $env:LUXMC_BEDROCK_FAMILY; if (!$package -or $package.Version.ToString() -ne $env:LUXMC_BEDROCK_VERSION) { if (Get-Process -Name Minecraft.Windows,Minecraft.WindowsBeta -ErrorAction SilentlyContinue) { throw 'Feche o Bedrock antes de trocar a versão.' }; Add-AppxPackage -Path $env:LUXMC_BEDROCK_PACKAGE -ForceUpdateFromAnyVersion; $package=Get-AppxPackage -Name $env:LUXMC_BEDROCK_FAMILY }; if (!$package -or $package.Version.ToString() -ne $env:LUXMC_BEDROCK_VERSION) { throw 'O Windows não confirmou a versão solicitada.' }; $app=Get-StartApps | Where-Object { $_.AppID.StartsWith($package.PackageFamilyName+'!') } | Select-Object -First 1; if (!$app) { throw 'A instalação não criou uma entrada de abertura.' }; Write-Output $app.AppID"])
            .env("LUXMC_BEDROCK_PACKAGE",deployment_path(&canonical)).env("LUXMC_BEDROCK_FAMILY",family).env("LUXMC_BEDROCK_VERSION",version).creation_flags(0x08000000).kill_on_drop(true);
        let output = tokio::time::timeout(std::time::Duration::from_secs(600), command.output()).await.map_err(|_| AppError::InvalidState("A instalação Bedrock demorou além do limite. Confira Aplicativos no Windows antes de tentar novamente.".into()))??;
        if !output.status.success() { return Err(AppError::InvalidState(format!("O Windows não concluiu a instalação Bedrock. {}",String::from_utf8_lossy(&output.stderr).chars().take(1200).collect::<String>()))); }
        let id = String::from_utf8_lossy(&output.stdout).trim().to_string();
        if !id.starts_with(&format!("{family}_8wekyb3d8bbwe!")) || !id.bytes().all(|byte| byte.is_ascii_alphanumeric() || b"._!-".contains(&byte)) { return Err(AppError::InvalidState("Identidade instalada inesperada".into())); }
        Ok(id)
    }
    #[cfg(not(windows))] { let _=(path,version,channel); Err(AppError::InvalidState("A edição Bedrock para PC requer Windows".into())) }
}

fn deployment_path(path: &std::path::Path) -> String {
    let value=path.to_string_lossy();
    if let Some(network)=value.strip_prefix(r"\\?\UNC\") {format!(r"\\{network}")}
    else {value.strip_prefix(r"\\?\").unwrap_or(&value).to_string()}
}

#[cfg(windows)]
async fn ensure_store_framework(shell: &std::path::Path) -> AppResult<()> {
    use sha2::{Digest,Sha256};
    use futures_util::StreamExt;
    use std::io::Read;
    let mut query=tokio::process::Command::new(shell);
    query.args(["-NoProfile","-NonInteractive","-Command","$package=Get-AppxPackage -Name Microsoft.Services.Store.Engagement | Where-Object { $_.Architecture -eq 'X64' -and [version]$_.Version -ge [version]'10.0.23012.0' }; if($package){Write-Output 'installed'}"]).creation_flags(0x08000000).kill_on_drop(true);
    let installed=tokio::time::timeout(std::time::Duration::from_secs(30),query.output()).await.map_err(|_|AppError::InvalidState("O Windows não respondeu à consulta de componentes".into()))??;
    if installed.status.success() && String::from_utf8_lossy(&installed.stdout).trim()=="installed" {return Ok(());}
    let client=reqwest::Client::builder().https_only(true).timeout(std::time::Duration::from_secs(60)).build()?;
    let response=client.get("https://api.nuget.org/v3-flatcontainer/microsoft.services.store.engagement/10.2307.3001/microsoft.services.store.engagement.10.2307.3001.nupkg").send().await?.error_for_status()?;
    let mut stream=response.bytes_stream();let mut bytes=Vec::new();
    while let Some(chunk)=stream.next().await {let chunk=chunk?;if bytes.len()+chunk.len()>2*1024*1024{return Err(AppError::InvalidInput("Componente Microsoft acima do limite".into()));}bytes.extend_from_slice(&chunk);}
    if format!("{:x}",Sha256::digest(&bytes))!="223d86691a9dfa9948a5b94a8631e82a6b4e5a147013b4e3603744172fab2d81" {return Err(AppError::InvalidInput("A integridade do componente Microsoft não foi confirmada".into()));}
    let package={
        let mut archive=zip::ZipArchive::new(std::io::Cursor::new(bytes))?;
        let mut entry=archive.by_name("SDK/Windows%20Kits/10/ExtensionSDKs/Microsoft.Services.Store.Engagement/10.0/Appx/x64/Microsoft.Services.Store.Engagement.x64.10.0.appx")?;
        if entry.size()>1024*1024{return Err(AppError::InvalidInput("Componente Microsoft inválido".into()));}
        let mut package=Vec::new();entry.read_to_end(&mut package)?;package
    };
    let path=root()?.join("dependencies/StoreEngagement.appx");
    crate::core::mods::pack_download::atomic_write(&path,&package).await?;
    let mut command=tokio::process::Command::new(shell);
    command.args(["-NoProfile","-NonInteractive","-Command","$ErrorActionPreference='Stop'; Add-AppxPackage -Path $env:LUXMC_BEDROCK_DEPENDENCY"]).env("LUXMC_BEDROCK_DEPENDENCY",deployment_path(&path.canonicalize()?)).creation_flags(0x08000000).kill_on_drop(true);
    let output=tokio::time::timeout(std::time::Duration::from_secs(120),command.output()).await.map_err(|_|AppError::InvalidState("O Windows demorou para instalar o componente Microsoft".into()))??;
    if !output.status.success(){return Err(AppError::InvalidState(format!("O Windows recusou o componente Store Engagement. {}",String::from_utf8_lossy(&output.stderr).chars().take(800).collect::<String>())));}
    Ok(())
}

#[cfg(windows)]
async fn ensure_gaming_services(shell: &std::path::Path) -> AppResult<()> {
    let mut query=tokio::process::Command::new(shell);
    query.args(["-NoProfile","-NonInteractive","-Command","if(Get-AppxPackage -Name Microsoft.GamingServices){Write-Output 'installed'}else{$installer=Get-AppxPackage -Name Microsoft.DesktopAppInstaller; if($installer){Write-Output (Join-Path $installer.InstallLocation 'winget.exe')}}"]).creation_flags(0x08000000).kill_on_drop(true);
    let output=tokio::time::timeout(std::time::Duration::from_secs(30),query.output()).await.map_err(|_|AppError::InvalidState("O Windows não respondeu à consulta do Gaming Services".into()))??;
    let value=String::from_utf8_lossy(&output.stdout).trim().to_string();
    if output.status.success() && value=="installed" {return Ok(());}
    let path=PathBuf::from(&value);
    let programs=PathBuf::from(std::env::var_os("ProgramFiles").ok_or_else(||AppError::InvalidState("Pasta de programas do Windows indisponível".into()))?).join("WindowsApps");
    if !output.status.success() || !path.is_file() || path.file_name().is_none_or(|name|name!="winget.exe") || !path.canonicalize()?.starts_with(programs.canonicalize()?) {return Err(AppError::InvalidState("Esta versão requer Gaming Services da Microsoft. Instale o componente pela Microsoft Store e tente novamente: https://apps.microsoft.com/detail/9mwpm2cqnlhn".into()));}
    let mut command=tokio::process::Command::new(path);
    command.args(["install","--id","9MWPM2CQNLHN","--source","msstore","--accept-source-agreements","--accept-package-agreements","--silent","--disable-interactivity"]).creation_flags(0x08004000).kill_on_drop(true);
    let installed=tokio::time::timeout(std::time::Duration::from_secs(600),command.output()).await.map_err(|_|AppError::InvalidState("O Gaming Services ainda está sendo preparado pela Microsoft Store. Aguarde sua instalação antes de tentar novamente.".into()))??;
    if !installed.status.success(){return Err(AppError::InvalidState("A Microsoft Store não concluiu a instalação do Gaming Services. Abra a Store, entre na sua conta e instale o componente: https://apps.microsoft.com/detail/9mwpm2cqnlhn".into()));}
    Ok(())
}

fn validate_appx(path: &std::path::Path, version: &str, channel: &str) -> AppResult<()> {
    use std::io::Read;
    use quick_xml::events::Event;
    let mut archive=zip::ZipArchive::new(std::fs::File::open(path)?).map_err(|error|AppError::InvalidInput(error.to_string()))?;
    if archive.by_name("AppxSignature.p7x").is_err() {return Err(AppError::InvalidInput("O pacote Bedrock não contém a assinatura Microsoft.".into()));}
    let mut manifest=archive.by_name("AppxManifest.xml").map_err(|_|AppError::InvalidInput("Manifesto Bedrock ausente".into()))?;
    if manifest.size()>1024*1024{return Err(AppError::InvalidInput("Manifesto Bedrock acima do limite".into()));}
    let mut xml=String::new();manifest.read_to_string(&mut xml)?;
    let mut reader=quick_xml::Reader::from_str(&xml);
    loop {
        match reader.read_event().map_err(|error|AppError::InvalidInput(error.to_string()))? {
            Event::Start(element)|Event::Empty(element) if element.local_name().as_ref()==b"Identity" => {
                let attributes=element.attributes().filter_map(Result::ok).map(|attribute|(attribute.key.as_ref().to_vec(),String::from_utf8_lossy(&attribute.value).into_owned())).collect::<std::collections::HashMap<_,_>>();
                let name=if channel=="release"{"Microsoft.MinecraftUWP"}else{"Microsoft.MinecraftWindowsBeta"};
                if attributes.get(b"Name".as_slice()).map(String::as_str)!=Some(name)||attributes.get(b"Version".as_slice()).map(String::as_str)!=Some(version)||attributes.get(b"ProcessorArchitecture".as_slice()).map(String::as_str)!=Some("x64") {return Err(AppError::InvalidInput("O pacote baixado não corresponde à versão e edição escolhidas.".into()));}
                return Ok(());
            },
            Event::Eof=>return Err(AppError::InvalidInput("Identidade do pacote não encontrada".into())),
            _=>{}
        }
    }
}

pub(super) fn backup_worlds() -> AppResult<()> {
    use sha2::{Digest, Sha256};
    let Some(local) = std::env::var_os("LOCALAPPDATA") else { return Ok(()) };
    let Some(roaming) = std::env::var_os("APPDATA") else { return Ok(()) };
    let sources = [PathBuf::from(&local).join("Packages/Microsoft.MinecraftUWP_8wekyb3d8bbwe/LocalState/games/com.mojang"), PathBuf::from(local).join("Packages/Microsoft.MinecraftWindowsBeta_8wekyb3d8bbwe/LocalState/games/com.mojang"), PathBuf::from(&roaming).join("Minecraft Bedrock/Users"), PathBuf::from(roaming).join("Minecraft Bedrock Preview/Users")];
    let base = root()?;
    let mut files = Vec::new();
    let mut fingerprint = Sha256::new();
    for (index, source) in sources.iter().enumerate().filter(|(_,path)| path.exists()) {
        if base.join("instances").canonicalize().is_ok_and(|owned| source.canonicalize().is_ok_and(|path| path.starts_with(owned))) { continue; }
        for entry in walkdir::WalkDir::new(source).follow_links(false).sort_by_file_name() {
            let entry = entry.map_err(|error| AppError::Internal(error.to_string()))?;
            if !entry.file_type().is_file() { continue; }
            let relative = PathBuf::from(index.to_string()).join(entry.path().strip_prefix(source).map_err(|error| AppError::Internal(error.to_string()))?);
            let metadata = entry.metadata().map_err(|error| AppError::Internal(error.to_string()))?;
            fingerprint.update(relative.to_string_lossy().as_bytes());
            fingerprint.update(metadata.len().to_le_bytes());
            fingerprint.update(metadata.modified()?.duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_nanos().to_le_bytes());
            files.push((entry.path().to_owned(), relative));
        }
    }
    let digest = format!("{:x}", fingerprint.finalize());
    let marker = base.join("world-backups/last-source-fingerprint");
    if files.is_empty() || std::fs::read_to_string(&marker).is_ok_and(|value| value == digest) { return Ok(()); }
    let destination = base.join("world-backups").join(uuid::Uuid::new_v4().to_string());
    for (source, relative) in files { let target = destination.join(relative); std::fs::create_dir_all(target.parent().unwrap())?; std::fs::copy(source, target)?; }
    std::fs::write(marker, digest)?;
    Ok(())
}

#[tauri::command]
pub fn bedrock_cancel_install() { CANCEL.store(true, Ordering::Relaxed); }

#[tauri::command]
pub async fn bedrock_install(app: tauri::AppHandle, version_id: String, name: String) -> AppResult<()> {
    bedrock_install_core(Some(app),version_id,name).await
}

pub(crate) async fn bedrock_install_core(app: Option<tauri::AppHandle>, version_id: String, name: String) -> AppResult<()> {
    if !cfg!(windows) { return Err(AppError::InvalidState("A instalação Bedrock requer Windows".into())); }
    if name.trim().chars().count()>80 || name.chars().any(char::is_control){return Err(AppError::InvalidInput("Use um nome de até 80 caracteres, sem controles".into()));}
    let _guard = INSTALL_LOCK.try_lock().map_err(|_| AppError::InvalidState("Uma instalação Bedrock já está em andamento".into()))?;
    CANCEL.store(false,Ordering::Relaxed);
    let version = bedrock_versions().await?.into_iter().find(|value| value.id == version_id).ok_or_else(|| AppError::NotFound("Versão Bedrock não encontrada".into()))?;
    if version.id.contains("..") || !version.id.bytes().all(|byte|byte.is_ascii_alphanumeric()||b".-".contains(&byte)) || package_version(&version.version).as_ref()!=Some(&version.package_version) {return Err(AppError::InvalidInput("Identificação da versão inválida".into()));}
    let client = http()?;
    let urls = match &version.update_id { Some(update) => uwp_urls(&client,update).await?, None => version.urls.clone() };
    let mut attempts=Vec::new();
    for source in urls {
        let parsed=url::Url::parse(&source).map_err(|_|AppError::InvalidInput("URL do pacote inválida".into()))?;
        if !microsoft_url(&parsed){return Err(AppError::InvalidInput("Origem do pacote não confiável".into()));}
        attempts.push(source);
        let mut legacy=parsed;legacy.set_scheme("http").map_err(|_|AppError::InvalidInput("Transporte do pacote inválido".into()))?;
        attempts.push(legacy.to_string());
    }
    let directory = root()?.join("packages").join(&version.id);
    tokio::fs::create_dir_all(&directory).await?;
    let path = directory.join(if version.package_type == "GDK" { "Minecraft.msixvc" } else { "Minecraft.appx" });
    let progress = |phase: &str, downloaded: u64, total: u64| { if let Some(app)=&app {let _=app.emit("bedrock-install-progress",serde_json::json!({"phase":phase,"downloaded":downloaded,"total":total,"percent":if total>0{(downloaded as f64/total as f64*90.0).min(90.0)}else{match phase {"backing-up"=>92.0,"registering"=>96.0,_=>0.0}}}));} };
    if !path.is_file() {
        use futures_util::StreamExt;
        use tokio::io::AsyncWriteExt;
        let temporary = path.with_extension("part");
        let mut last = None;
        let mut completed = false;
        for source in attempts {
            let result: AppResult<()> = async {
                let response = client.get(&source).send().await?.error_for_status()?;
                if !microsoft_package_url(response.url()) { return Err(AppError::InvalidInput("Origem do pacote não confiável".into())); }
                let total = response.content_length().unwrap_or(0);
                if total>4*1024*1024*1024 {return Err(AppError::InvalidInput("Pacote Bedrock maior que 4 GB".into()));}
                let mut output=tokio::fs::File::create(&temporary).await?;
                let mut stream=response.bytes_stream(); let mut count=0u64; let mut emitted=std::time::Instant::now();
                while let Some(chunk)=tokio::time::timeout(std::time::Duration::from_secs(30),stream.next()).await.map_err(|_|AppError::InvalidState("Download Bedrock sem resposta".into()))? {
                    if CANCEL.load(Ordering::Relaxed){return Err(AppError::InvalidState("Instalação Bedrock cancelada".into()));}
                    let chunk=chunk?;count+=chunk.len() as u64;
                    if count>4*1024*1024*1024{return Err(AppError::InvalidInput("Pacote acima do limite".into()));}
                    output.write_all(&chunk).await?;
                    if emitted.elapsed()>std::time::Duration::from_millis(150){progress("downloading",count,total);emitted=std::time::Instant::now();}
                }
                if count==0 || total>0 && count!=total{return Err(AppError::InvalidState("Pacote Bedrock incompleto".into()));}
                output.sync_all().await?;drop(output);
                if version.package_type=="UWP" {validate_appx(&temporary,&version.package_version,&version.channel)?;}
                tokio::fs::rename(&temporary,&path).await?;Ok(())
            }.await;
            match result {Ok(())=>{completed=true;break;},Err(error)=>{last=Some(error);if CANCEL.load(Ordering::Relaxed){break;}}}
        }
        if !completed{return Err(last.unwrap_or_else(||AppError::InvalidState("Nenhum download disponível".into())));}
    }
    if CANCEL.load(Ordering::Relaxed){return Err(AppError::InvalidState("Instalação Bedrock cancelada".into()));}
    progress("backing-up",0,0);
    progress("registering",0,0);
    let app_id=deploy(path.clone(),version.package_version.clone(),version.channel.clone()).await?;
    super::bedrock::register_managed(version,path,app_id,name).await?;
    if let Some(app)=&app {let _=app.emit("bedrock-install-progress",serde_json::json!({"phase":"completed","percent":100,"downloaded":0,"total":0}));}
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn catalog_validates_exact_microsoft_identity_and_version() {
        let legacy=serde_json::json!([["1.20.81.01","48f0d81e-0cc1-4e97-ab85-c9d3d24e848f","0","x64"],["1.20.81.01","../evil","0","x64"]]);
        let gdk=serde_json::json!({"release":{"1.26.52.3":["http://assets1.xboxlive.com/test/Microsoft.MinecraftUWP_1.26.5203.0_x64__8wekyb3d8bbwe.msixvc","https://evil.example/Microsoft.MinecraftUWP_1.26.5203.0_x64__8wekyb3d8bbwe.msixvc","https://assets1.xboxlive.com/test/Microsoft.MinecraftUWP_1.26.5103.0_x64__8wekyb3d8bbwe.msixvc"]}});
        let versions=parse_catalog(legacy,gdk);assert_eq!(versions.len(),2);assert_eq!(versions[0].urls.len(),1);assert!(versions[0].urls[0].starts_with("https://"));assert_eq!(versions[1].package_version,"1.20.8101.0");
        assert!(safe_url("https://attacker@assets1.xboxlive.com/a").is_none());assert!(package_version("../evil").is_none());
    }
    #[test]
    fn store_urls_preserve_all_signed_query_parameters() {
        let parsed=parse_store_urls("<FileLocations><Url>http://tlu.dl.delivery.mp.microsoft.com/a?P1=one&amp;P2=two&amp;P3=three</Url><Url>https://evil.example/a</Url></FileLocations>").unwrap();
        assert_eq!(parsed,vec!["https://tlu.dl.delivery.mp.microsoft.com/a?P1=one&P2=two&P3=three"]);
    }
    #[test]
    fn deployment_uses_a_windows_path_without_a_device_prefix() {
        assert_eq!(deployment_path(std::path::Path::new(r"\\?\C:\Users\player\Minecraft.appx")),r"C:\Users\player\Minecraft.appx");
        assert_eq!(deployment_path(std::path::Path::new(r"\\?\UNC\server\data\Minecraft.appx")),r"\\server\data\Minecraft.appx");
    }
}
