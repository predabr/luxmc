use futures_util::StreamExt;
use std::path::{Path, PathBuf};
use tauri::Emitter;
use tokio::io::AsyncWriteExt;
use url::Url;

use crate::error::{AppError, AppResult};

const MAX_UPDATE_BYTES: u64 = 2 * 1024 * 1024 * 1024;

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
        lower.ends_with(".appimage") || lower.ends_with(".pkg.tar.zst") || lower.ends_with(".deb")
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
    let temp_file_path = temp_dir.join(format!("luxmc-update-{}-{file_name}", uuid::Uuid::new_v4()));

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

        // Se estiver rodando como AppImage, substitui o AppImage original e reinicia
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
                    std::fs::rename(&staged, &appimage_target)?;
                    Ok(())
                })();
                if let Err(error) = staged_result {
                    let _ = std::fs::remove_file(&staged);
                    return Err(error);
                }
                let _ = std::fs::remove_file(&temp_file_path);
                std::process::Command::new(&appimage_target)
                    .spawn()
                    .map_err(|e| {
                        AppError::Internal(format!("Falha ao iniciar AppImage atualizado: {e}"))
                    })?;
                std::process::exit(0);
            }
        }

        let environment = app_update_environment().mode;
        let file_name = temp_file_path.to_string_lossy().to_ascii_lowercase();
        let (program, arguments, terminal_command) = match environment.as_str() {
            "pacman" if file_name.ends_with(".pkg.tar.zst") => (
                "pacman",
                vec!["-U".to_string(), temp_file_path.to_string_lossy().into_owned()],
                format!("sudo pacman -U -- {}", quote_for_shell(&temp_file_path)),
            ),
            "debian" if file_name.ends_with(".deb") => (
                "dpkg",
                vec!["-i".to_string(), temp_file_path.to_string_lossy().into_owned()],
                format!("sudo dpkg -i {}", quote_for_shell(&temp_file_path)),
            ),
            "pacman" | "debian" | "system" => return Err(AppError::InvalidInput("A versão disponível não corresponde ao formato instalado. Abra o GitHub para baixar o pacote correto.".into())),
            _ => return Err(AppError::InvalidInput("Atualização automática disponível apenas para AppImage, pacman, dpkg e Windows.".into())),
        };
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
        if temp_file_path.to_string_lossy().to_ascii_lowercase().ends_with(".exe") {
            let mut cmd = crate::core::process::std_command(&temp_file_path);
            cmd.spawn()
                .map_err(|e| AppError::Internal(format!("Falha ao iniciar instalador: {e}")))?;
            std::process::exit(0);
        }
        if temp_file_path.to_string_lossy().to_ascii_lowercase().ends_with(".msi") {
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

    #[test]
    fn accepts_official_release_asset_url() {
        let url = Url::parse("https://github.com/predabr/luxmc/releases/download/v1/Luxmc.AppImage").unwrap();
        assert!(is_official_release_url(&url));
    }

    #[test]
    fn rejects_untrusted_update_urls() {
        let url = Url::parse("https://example.com/predabr/luxmc/releases/download/v1/Luxmc.AppImage").unwrap();
        assert!(!is_official_release_url(&url));
        assert!(!is_allowed_redirect_url(&url));
    }
}
