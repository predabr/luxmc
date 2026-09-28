use futures_util::StreamExt;
use std::path::{Path, PathBuf};
use tauri::Emitter;
use tokio::io::AsyncWriteExt;
use url::Url;

use crate::error::{AppError, AppResult};

const MAX_UPDATE_BYTES: u64 = 2 * 1024 * 1024 * 1024;

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
        && url.path().starts_with("/predabr/luxmc/releases/")
}

fn is_allowed_redirect_url(url: &Url) -> bool {
    url.scheme() == "https"
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
    download_url: String,
) -> AppResult<UpdateOutcome> {
    let source_url = Url::parse(&download_url)
        .map_err(|_| AppError::InvalidInput("URL de atualização inválida".into()))?;
    if !is_official_release_url(&source_url) {
        return Err(AppError::InvalidInput(
            "A atualização deve ser baixada da versão oficial no GitHub".into(),
        ));
    }
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
        .redirect(reqwest::redirect::Policy::custom(|attempt| {
            if attempt.previous().len() >= 5 || !is_allowed_redirect_url(attempt.url()) {
                attempt.stop()
            } else {
                attempt.follow()
            }
        }))
        .build()
        .map_err(|e| AppError::Internal(format!("Falha ao preparar atualização: {e}")))?;
    let resp = client
        .get(source_url)
        .header("User-Agent", "Luxmc-Launcher-Updater")
        .send()
        .await
        .map_err(|e| AppError::Internal(format!("Erro ao baixar atualização: {e}")))?;

    if !resp.status().is_success() {
        return Err(AppError::Internal(format!(
            "Falha no download da atualização: HTTP {}",
            resp.status()
        )));
    }

    if !is_allowed_redirect_url(resp.url()) {
        return Err(AppError::InvalidInput(
            "O download foi redirecionado para uma origem não confiável".into(),
        ));
    }
    let file_name = update_file_name(resp.url())?;
    let total_bytes = resp.content_length().unwrap_or(0);
    if total_bytes > MAX_UPDATE_BYTES {
        return Err(AppError::InvalidInput(
            "A atualização excede o tamanho máximo permitido".into(),
        ));
    }
    let temp_dir = std::env::temp_dir();
    let temp_file_path =
        temp_dir.join(format!("luxmc-update-{}-{file_name}", uuid::Uuid::new_v4()));

    let mut out_file = tokio::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&temp_file_path)
        .await
        .map_err(AppError::Io)?;

    let mut stream = resp.bytes_stream();
    let mut downloaded: u64 = 0;

    while let Some(chunk) = stream.next().await {
        let chunk = match chunk {
            Ok(chunk) => chunk,
            Err(error) => {
                let _ = tokio::fs::remove_file(&temp_file_path).await;
                return Err(AppError::Internal(format!("Erro de transmissão: {error}")));
            }
        };
        downloaded += chunk.len() as u64;
        if downloaded > MAX_UPDATE_BYTES {
            let _ = tokio::fs::remove_file(&temp_file_path).await;
            return Err(AppError::InvalidInput(
                "A atualização excede o tamanho máximo permitido".into(),
            ));
        }
        if let Err(error) = out_file.write_all(&chunk).await {
            let _ = tokio::fs::remove_file(&temp_file_path).await;
            return Err(AppError::Io(error));
        }

        let percent = if total_bytes > 0 {
            ((downloaded as f64 / total_bytes as f64) * 100.0) as u32
        } else {
            50
        };

        let mb_down = (downloaded as f64 / 1_048_576.0).round();
        let mb_tot = (total_bytes as f64 / 1_048_576.0).round();

        let _ = app.emit(
            "update-progress",
            UpdateProgress {
                percent: percent.min(99),
                transferred: downloaded,
                total: total_bytes,
                status: format!("Baixando atualização: {} MB / {} MB", mb_down, mb_tot),
            },
        );
    }

    out_file.flush().await.map_err(AppError::Io)?;
    drop(out_file);

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
            Ok(_) => {
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
    }

    #[cfg(target_os = "windows")]
    {
        if temp_file_path
            .to_string_lossy()
            .to_ascii_lowercase()
            .ends_with(".exe")
        {
            let mut cmd = crate::core::process::std_command(&temp_file_path);
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

#[cfg(test)]
mod tests {
    use super::*;

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
