use std::path::Path;
#[cfg(target_os = "linux")]
use std::path::PathBuf;
use tauri::Emitter;
use url::Url;

use crate::error::{AppError, AppResult};

pub(super) const MAX_UPDATE_BYTES: u64 = 2 * 1024 * 1024 * 1024;

fn verify_release_signature(manifest: &[u8], encoded: &[u8], public_key: &[u8]) -> AppResult<()> {
    use base64::Engine;
    let signature = std::str::from_utf8(encoded).ok()
        .filter(|value| value.len() <= 128)
        .and_then(|value| base64::engine::general_purpose::STANDARD.decode(value.trim()).ok())
        .filter(|value| value.len() == 64)
        .ok_or_else(|| AppError::InvalidInput("Assinatura da atualização inválida.".into()))?;
    ring::signature::UnparsedPublicKey::new(&ring::signature::ED25519, public_key)
        .verify(manifest, &signature)
        .map_err(|_| AppError::InvalidInput("A assinatura oficial da atualização não confere. Nenhum instalador será executado.".into()))
}

#[cfg(target_os = "linux")]
fn replace_appimage(staged: &Path, target: &Path) -> AppResult<Option<PathBuf>> {
    use std::os::unix::fs::PermissionsExt;

    std::fs::set_permissions(staged, std::fs::Permissions::from_mode(0o755))?;
    let parent = target
        .parent()
        .ok_or_else(|| AppError::InvalidState("Caminho do AppImage inválido".into()))?;
    let old = parent.join(format!(
        ".{}.{}.old",
        target
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("luxmc"),
        uuid::Uuid::new_v4()
    ));

    let had_target = target.exists();
    if had_target {
        std::fs::rename(target, &old)?;
    }
    if let Err(error) = std::fs::rename(staged, target) {
        if old.exists() {
            let _ = std::fs::rename(&old, target);
        }
        return Err(error.into());
    }
    std::fs::set_permissions(target, std::fs::Permissions::from_mode(0o755))?;
    Ok(had_target.then_some(old))
}

fn is_official_release_url(url: &Url) -> bool {
    url.scheme() == "https"
        && url.host_str() == Some("github.com")
        && url.username().is_empty()
        && url.password().is_none()
        && url.port().is_none()
        && (url.path().starts_with("/predabr/luxmc/releases/download/")
            || url.path().starts_with("/predabr/luxmc/releases/latest/download/"))
}

fn is_allowed_redirect_url(url: &Url) -> bool {
    url.scheme() == "https"
        && url.username().is_empty()
        && url.password().is_none()
        && url.port().is_none()
        && matches!(
            url.host_str(),
            Some(
                "github.com"
                    | "objects.githubusercontent.com"
                    | "github-releases.githubusercontent.com"
                    | "release-assets.githubusercontent.com"
            )
        )
}

fn accepted_update_extension(file_name: &str) -> bool {
    let lower = file_name.to_ascii_lowercase();
    #[cfg(target_os = "linux")]
    {
        lower.ends_with(".appimage")
            || lower.ends_with(".pkg.tar.zst")
            || lower.ends_with(".deb")
            || lower.ends_with(".rpm")
    }
    #[cfg(target_os = "windows")]
    {
        lower.ends_with(".exe") || lower.ends_with(".msi")
    }
    #[cfg(target_os = "macos")]
    {
        lower.ends_with(".dmg")
    }
}

fn update_file_name(url: &Url) -> AppResult<String> {
    let file_name = url
        .path_segments()
        .and_then(|mut segments| segments.next_back())
        .filter(|name| !name.is_empty())
        .ok_or_else(|| AppError::InvalidInput("Arquivo de atualização inválido".into()))?;
    if !accepted_update_extension(file_name) {
        return Err(AppError::InvalidInput(
            "O arquivo de atualização não é compatível com esta plataforma".into(),
        ));
    }
    Ok(file_name.to_string())
}

#[derive(Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateProgress {
    pub percent: u32,
    pub transferred: u64,
    pub total: u64,
    pub status: String,
}

#[derive(Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateEnvironment {
    pub mode: String,
}

#[derive(Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateOutcome {
    pub action: String,
    pub terminal_command: Option<String>,
}

#[tauri::command]
pub fn app_update_environment() -> UpdateEnvironment {
    #[cfg(target_os = "linux")]
    {
        if std::env::var_os("APPIMAGE").is_some() {
            return UpdateEnvironment {
                mode: "appimage".into(),
            };
        }
        if std::env::current_exe()
            .ok()
            .is_some_and(|path| path.starts_with("/usr/bin"))
        {
            if Path::new("/usr/bin/pacman").is_file() {
                return UpdateEnvironment {
                    mode: "pacman".into(),
                };
            }
            if Path::new("/usr/bin/dpkg").is_file() {
                return UpdateEnvironment {
                    mode: "debian".into(),
                };
            }
            if Path::new("/usr/bin/dnf").is_file()
                || Path::new("/usr/bin/zypper").is_file()
                || Path::new("/usr/bin/rpm").is_file()
            {
                return UpdateEnvironment { mode: "rpm".into() };
            }
            return UpdateEnvironment {
                mode: "system".into(),
            };
        }
        return UpdateEnvironment {
            mode: "manual".into(),
        };
    }
    #[cfg(target_os = "windows")]
    {
        UpdateEnvironment {
            mode: "windows".into(),
        }
    }
    #[cfg(target_os = "macos")]
    {
        UpdateEnvironment {
            mode: "macos".into(),
        }
    }
}

fn quote_for_shell(path: &Path) -> String {
    format!("'{}'", path.to_string_lossy().replace('\'', "'\\\"'\\\"'"))
}

#[cfg(target_os = "linux")]
fn linux_package_installer(
    environment: &str,
    package: &Path,
) -> AppResult<(&'static str, Vec<String>, String)> {
    let package_path = package.to_string_lossy().into_owned();
    let quoted = quote_for_shell(package);
    match environment {
        "pacman" if package_path.to_ascii_lowercase().ends_with(".pkg.tar.zst") => Ok((
            "pacman",
            vec!["-U".into(), "--needed".into(), package_path],
            format!("sudo pacman -U --needed -- {quoted}"),
        )),
        "debian" if package_path.to_ascii_lowercase().ends_with(".deb") => Ok((
            "apt-get",
            vec!["install".into(), "-y".into(), package_path],
            format!("sudo apt-get install -y {quoted}"),
        )),
        "rpm" if package_path.to_ascii_lowercase().ends_with(".rpm") => {
            if Path::new("/usr/bin/dnf").is_file() {
                Ok((
                    "dnf",
                    vec!["install".into(), "-y".into(), package_path],
                    format!("sudo dnf install -y {quoted}"),
                ))
            } else if Path::new("/usr/bin/zypper").is_file() {
                Ok((
                    "zypper",
                    vec!["--non-interactive".into(), "install".into(), package_path],
                    format!("sudo zypper --non-interactive install {quoted}"),
                ))
            } else {
                Ok((
                    "rpm",
                    vec!["-Uvh".into(), "--replacepkgs".into(), package_path],
                    format!("sudo rpm -Uvh --replacepkgs {quoted}"),
                ))
            }
        }
        "pacman" | "debian" | "rpm" | "system" => Err(AppError::InvalidInput(
            "A versão disponível não corresponde ao formato instalado. Abra o GitHub para baixar o pacote correto.".into(),
        )),
        _ => Err(AppError::InvalidInput(
            "Atualização automática disponível apenas para AppImage, pacman, APT, RPM e Windows.".into(),
        )),
    }
}

#[tauri::command]
pub async fn app_perform_update(
    app: tauri::AppHandle,
    state: tauri::State<'_, crate::state::AppState>,
    download_url: String,
) -> AppResult<UpdateOutcome> {
    let _launch_guard = state.launch_lock.try_lock().map_err(|_| {
        AppError::InvalidState("Aguarde o lançamento ou a atualização em andamento.".into())
    })?;
    ensure_update_idle(crate::core::launcher::get_active_game_pid())?;
    let source_url = Url::parse(&download_url)
        .map_err(|_| AppError::InvalidInput("URL de atualização inválida".into()))?;
    if !is_official_release_url(&source_url) {
        return Err(AppError::InvalidInput(
            "A atualização deve ser baixada da versão oficial no GitHub".into(),
        ));
    }
    let file_name = update_file_name(&source_url)?;
    tracing::info!(url = %source_url, "Iniciando download da atualização");

    let _ = app.emit(
        "update-progress",
        UpdateProgress {
            percent: 0,
            transferred: 0,
            total: 0,
            status: "Conectando ao servidor...".to_string(),
        },
    );

    let client = reqwest::Client::builder()
        .connect_timeout(std::time::Duration::from_secs(15))
        .timeout(std::time::Duration::from_secs(900))
        .redirect(reqwest::redirect::Policy::custom(|attempt| {
            if attempt.previous().len() >= 5 || !is_allowed_redirect_url(attempt.url()) {
                attempt.stop()
            } else {
                attempt.follow()
            }
        }))
        .build()
        .map_err(|e| AppError::Internal(format!("Falha ao preparar atualização: {e}")))?;
    use sha2::Digest;
    let base = directories::ProjectDirs::from("io","github","Luxmc").ok_or_else(|| AppError::InvalidState("Cache de atualização indisponível".into()))?;
    let update_dir = base.cache_dir().join("updates");
    tokio::fs::create_dir_all(&update_dir).await?;
    let temp_file_path = update_dir.join(format!("{:x}-{file_name}",sha2::Sha256::digest(source_url.as_str().as_bytes())));
    let (downloaded, total_bytes) = super::updater_download::download(&client, &source_url, &temp_file_path, is_allowed_redirect_url, |downloaded, total| {
        let _ = app.emit("update-progress", UpdateProgress {
            percent: if total > 0 { ((downloaded as f64 / total as f64) * 100.0).min(99.0) as u32 } else { 0 },
            transferred: downloaded, total,
            status: "Baixando e verificando atualização...".into(),
        });
    }).await?;
    let sums_url = source_url.join("SHA256SUMS").map_err(|e| AppError::InvalidInput(e.to_string()))?;
    let response = client.get(sums_url).send().await?.error_for_status()?;
    if !is_allowed_redirect_url(response.url()) || response.content_length().is_some_and(|n| n > 256 * 1024) { return Err(AppError::InvalidInput("Manifesto de integridade inválido".into())); }
    use futures_util::StreamExt;
    let mut chunks = response.bytes_stream(); let mut manifest = Vec::new();
    while let Some(chunk) = chunks.next().await { let chunk=chunk?; if manifest.len()+chunk.len()>256*1024 {return Err(AppError::InvalidInput("Manifesto acima do limite".into()));} manifest.extend_from_slice(&chunk); }
    let signature_url = source_url.join("SHA256SUMS.sig").map_err(|e| AppError::InvalidInput(e.to_string()))?;
    let signature_response = client.get(signature_url).send().await?.error_for_status()?;
    if !is_allowed_redirect_url(signature_response.url()) || signature_response.content_length().is_some_and(|length| length > 128) {
        return Err(AppError::InvalidInput("Origem ou tamanho da assinatura inválido.".into()));
    }
    let mut signature_stream = signature_response.bytes_stream();
    let mut signature = Vec::new();
    while let Some(chunk) = signature_stream.next().await {
        let chunk = chunk?;
        if signature.len() + chunk.len() > 128 { return Err(AppError::InvalidInput("Assinatura acima do limite.".into())); }
        signature.extend_from_slice(&chunk);
    }
    verify_release_signature(&manifest, &signature, include_bytes!("../../../packaging/security/release-public-key.bin"))?;
    let manifest = String::from_utf8(manifest).map_err(|e| AppError::InvalidInput(e.to_string()))?;
    let expected = checksum_entry(&manifest,&file_name)?;
    let verification_path = temp_file_path.clone();
    let actual = tokio::task::spawn_blocking(move || -> AppResult<String> { use std::io::Read; let mut file=std::fs::File::open(verification_path)?; let mut digest=sha2::Sha256::new(); let mut buffer=[0u8;65536]; loop {let n=file.read(&mut buffer)?;if n==0{break;} digest.update(&buffer[..n]);} Ok(format!("{:x}",digest.finalize())) }).await.map_err(|e| AppError::Internal(e.to_string()))??;
    if !actual.eq_ignore_ascii_case(&expected) { tokio::fs::remove_file(&temp_file_path).await?; return Err(AppError::InvalidInput("A conferência SHA-256 da atualização falhou. O instalador foi descartado; tente novamente.".into())); }
    ensure_update_idle(crate::core::launcher::get_active_game_pid())?;

    let _ = app.emit(
        "update-progress",
        UpdateProgress {
            percent: 100,
            transferred: downloaded,
            total: total_bytes,
            status: "Instalando nova versão e reiniciando o Luxmc...".to_string(),
        },
    );

    tokio::time::sleep(std::time::Duration::from_millis(500)).await;

    #[cfg(target_os = "linux")]
    {
        let outcome = tokio::task::spawn_blocking(move || -> AppResult<UpdateOutcome> {
            use std::os::unix::fs::PermissionsExt;
            let _ = std::fs::set_permissions(&temp_file_path, std::fs::Permissions::from_mode(0o755));

            if let Ok(appimage_path) = std::env::var("APPIMAGE") {
                let appimage_target = PathBuf::from(&appimage_path);
                if temp_file_path.to_string_lossy().ends_with(".AppImage") {
                    let parent = appimage_target
                        .parent()
                        .ok_or_else(|| AppError::InvalidState("Caminho do AppImage inválido".into()))?;
                    let staged = parent.join(format!(
                        ".{}.{}.update",
                        appimage_target
                            .file_name()
                            .and_then(|name| name.to_str())
                            .unwrap_or("luxmc"),
                        uuid::Uuid::new_v4()
                    ));
                    let staged_result = (|| -> AppResult<()> {
                        let mut source = std::fs::File::open(&temp_file_path)?;
                        let mut target = std::fs::OpenOptions::new()
                            .write(true)
                            .create_new(true)
                            .open(&staged)?;
                        std::io::copy(&mut source, &mut target)?;
                        target.sync_all()?;
                        std::fs::set_permissions(&staged, std::fs::Permissions::from_mode(0o755))?;
                        let old = replace_appimage(&staged, &appimage_target)?;
                        let launched = std::process::Command::new(&appimage_target).spawn();
                        match launched {
                            Ok(_) => {
                                if let Some(old) = old {
                                    let _ = std::fs::remove_file(old);
                                }
                            }
                            Err(error) => {
                                if let Some(old) = old {
                                    let _ = std::fs::remove_file(&appimage_target);
                                    let _ = std::fs::rename(old, &appimage_target);
                                }
                                return Err(AppError::Internal(format!(
                                    "Falha ao iniciar AppImage atualizado: {error}"
                                )));
                            }
                        }
                        Ok(())
                    })();
                    if let Err(error) = staged_result {
                        let _ = std::fs::remove_file(&staged);
                        return Err(error);
                    }
                    let _ = std::fs::remove_file(&temp_file_path);
                    std::process::exit(0);
                }
            }

            let environment = app_update_environment().mode;
            let (program, arguments, terminal_command) =
                linux_package_installer(&environment, &temp_file_path)?;
            match std::process::Command::new("pkexec")
                .arg(program)
                .args(&arguments)
                .spawn()
            {
                Ok(mut child) => {
                    std::thread::spawn(move || {
                        let _ = child.wait();
                    });
                    return Ok(UpdateOutcome {
                        action: "system-installer".into(),
                        terminal_command: Some(terminal_command),
                    })
                }
                Err(_) => {
                    return Ok(UpdateOutcome {
                        action: "terminal".into(),
                        terminal_command: Some(terminal_command),
                    })
                }
            }
        })
        .await
        .map_err(|error| AppError::Internal(error.to_string()))??;
        return Ok(outcome);
    }

    #[cfg(target_os = "windows")]
    {
        if temp_file_path
            .to_string_lossy()
            .to_ascii_lowercase()
            .ends_with(".exe")
        {
            let mut cmd = crate::core::process::std_command(&temp_file_path);
            cmd.args(["/P", "/UPDATE", "/R"]);
            cmd.spawn()
                .map_err(|e| AppError::Internal(format!("Falha ao iniciar instalador: {e}")))?;
            std::process::exit(0);
        }
        if temp_file_path
            .to_string_lossy()
            .to_ascii_lowercase()
            .ends_with(".msi")
        {
            crate::core::process::std_command("msiexec.exe")
                .args(["/i", &temp_file_path.to_string_lossy()])
                .spawn()
                .map_err(|e| AppError::Internal(format!("Falha ao iniciar instalador: {e}")))?;
            std::process::exit(0);
        }
    }

    #[cfg(target_os = "macos")]
    {
        let _ = open::that(&temp_file_path);
        std::process::exit(0);
    }

    #[cfg(not(target_os = "linux"))]
    Ok(UpdateOutcome {
        action: "downloaded".into(),
        terminal_command: None,
    })
}

fn ensure_update_idle(game_pid: u32) -> AppResult<()> {
    if game_pid != 0 {
        return Err(AppError::InvalidState("Feche o Minecraft antes de atualizar o Luxmc. Seu jogo continuará aberto.".into()));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn update_requires_minecraft_to_be_closed() {
        assert!(ensure_update_idle(0).is_ok());
        assert!(ensure_update_idle(42).is_err());
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn replaces_existing_appimage_without_overwriting_its_inode() {
        use std::os::unix::fs::PermissionsExt;

        let dir = std::env::temp_dir().join(format!("luxmc-updater-test-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&dir).unwrap();
        let target = dir.join("Luxmc.AppImage");
        let staged = dir.join("Luxmc.AppImage.update");
        std::fs::write(&target, b"old").unwrap();
        std::fs::write(&staged, b"new").unwrap();

        let old = replace_appimage(&staged, &target).unwrap().unwrap();

        assert_eq!(std::fs::read(&target).unwrap(), b"new");
        assert_eq!(std::fs::read(&old).unwrap(), b"old");
        assert_eq!(
            std::fs::metadata(&target).unwrap().permissions().mode() & 0o777,
            0o755
        );
        assert!(!staged.exists());
        assert_eq!(std::fs::read_dir(&dir).unwrap().count(), 2);
        std::fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn accepts_official_release_asset_url() {
        let url =
            Url::parse("https://github.com/predabr/luxmc/releases/download/v1/Luxmc.AppImage")
                .unwrap();
        assert!(is_official_release_url(&url));
    }

    #[test]
    fn rejects_untrusted_update_urls() {
        let url =
            Url::parse("https://example.com/predabr/luxmc/releases/download/v1/Luxmc.AppImage")
                .unwrap();
        assert!(!is_official_release_url(&url));
        assert!(!is_allowed_redirect_url(&url));
    }

    #[test]
    fn rejects_credentials_ports_and_release_pages() {
        for value in [
            "https://attacker@github.com/predabr/luxmc/releases/download/v3.6.0/a.exe",
            "https://github.com:8443/predabr/luxmc/releases/download/v3.6.0/a.exe",
            "https://github.com/predabr/luxmc/releases/tag/v3.6.0",
            "https://github.com/predabr/luxmc/releases/download/../../other/a.exe",
        ] { assert!(!is_official_release_url(&Url::parse(value).unwrap())); }
    }

    #[test]
    fn signature_rejects_tampering_and_unknown_signers() {
        use base64::Engine;
        use ring::signature::KeyPair;
        let random = ring::rand::SystemRandom::new();
        let document = ring::signature::Ed25519KeyPair::generate_pkcs8(&random).unwrap();
        let key = ring::signature::Ed25519KeyPair::from_pkcs8(document.as_ref()).unwrap();
        let content = b"official hash manifest";
        let encoded = base64::engine::general_purpose::STANDARD.encode(key.sign(content).as_ref());
        assert!(verify_release_signature(content, encoded.as_bytes(), key.public_key().as_ref()).is_ok());
        assert!(verify_release_signature(b"changed manifest", encoded.as_bytes(), key.public_key().as_ref()).is_err());
        assert!(verify_release_signature(content, encoded.as_bytes(), include_bytes!("../../../packaging/security/release-public-key.bin")).is_err());
        assert!(verify_release_signature(content, b"invalid", key.public_key().as_ref()).is_err());
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn accepts_and_prepares_rpm_updates() {
        let url = Url::parse(
            "https://github.com/predabr/luxmc/releases/download/v2.0.1/Luxmc-2.0.1-1.x86_64.rpm",
        )
        .unwrap();
        assert_eq!(update_file_name(&url).unwrap(), "Luxmc-2.0.1-1.x86_64.rpm");

        let (program, arguments, terminal) =
            linux_package_installer("rpm", Path::new("/tmp/Luxmc update.rpm")).unwrap();
        assert!(matches!(program, "dnf" | "zypper" | "rpm"));
        assert!(arguments.iter().any(|argument| argument.ends_with(".rpm")));
        assert!(terminal.contains("'/tmp/Luxmc update.rpm'"));
    }
}

fn checksum_entry(manifest: &str, name: &str) -> AppResult<String> {
    for line in manifest.lines() {
        if let Some((hash,file)) = line.split_once(char::is_whitespace) {
            if file.trim().trim_start_matches('*') == name && hash.len()==64 && hash.chars().all(|c| c.is_ascii_hexdigit()) { return Ok(hash.into()); }
        }
    }
    Err(AppError::InvalidInput("A atualização não possui uma entrada SHA-256 válida no manifesto oficial".into()))
}

#[cfg(test)]
mod checksum_tests {
    #[test]
    fn matches_exact_filename_and_requires_sha256() {
        let hash="a".repeat(64);
        assert_eq!(super::checksum_entry(&format!("{hash}  Lux.MC.Launcher.exe"),"Lux.MC.Launcher.exe").unwrap(),hash);
        assert!(super::checksum_entry("bad  Lux.MC.Launcher.exe","Lux.MC.Launcher.exe").is_err());
        assert!(super::checksum_entry(&format!("{hash}  other.exe"),"Lux.MC.Launcher.exe").is_err());
    }
}
