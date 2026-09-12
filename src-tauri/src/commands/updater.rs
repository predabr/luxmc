use std::path::PathBuf;
use tauri::{Emitter, State};
use futures_util::StreamExt;
use tokio::io::AsyncWriteExt;

use crate::error::{AppError, AppResult};
use crate::state::AppState;

#[derive(Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateProgress {
    pub percent: u32,
    pub transferred: u64,
    pub total: u64,
    pub status: String,
}

#[tauri::command]
pub async fn app_perform_update(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
    download_url: String,
) -> AppResult<()> {
    tracing::info!(url = %download_url, "Iniciando download da atualização");

    let _ = app.emit("update-progress", UpdateProgress {
        percent: 0,
        transferred: 0,
        total: 0,
        status: "Conectando ao servidor...".to_string(),
    });

    let resp = state
        .http
        .get(&download_url)
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

    let total_bytes = resp.content_length().unwrap_or(0);
    let temp_dir = std::env::temp_dir();
    let file_name = download_url
        .split('/')
        .last()
        .unwrap_or("luxmc_update.bin")
        .split('?')
        .next()
        .unwrap_or("luxmc_update.bin");

    let temp_file_path = temp_dir.join(format!("luxmc_new_{}", file_name));

    let mut out_file = tokio::fs::File::create(&temp_file_path)
        .await
        .map_err(AppError::Io)?;

    let mut stream = resp.bytes_stream();
    let mut downloaded: u64 = 0;

    while let Some(chunk) = stream.next().await {
        let chunk = chunk.map_err(|e| AppError::Internal(format!("Erro de transmissão: {e}")))?;
        downloaded += chunk.len() as u64;
        out_file.write_all(&chunk).await.map_err(AppError::Io)?;

        let percent = if total_bytes > 0 {
            ((downloaded as f64 / total_bytes as f64) * 100.0) as u32
        } else {
            50
        };

        let mb_down = (downloaded as f64 / 1_048_576.0).round();
        let mb_tot = (total_bytes as f64 / 1_048_576.0).round();

        let _ = app.emit("update-progress", UpdateProgress {
            percent: percent.min(99),
            transferred: downloaded,
            total: total_bytes,
            status: format!("Baixando atualização: {} MB / {} MB", mb_down, mb_tot),
        });
    }

    out_file.flush().await.map_err(AppError::Io)?;
    drop(out_file);

    let _ = app.emit("update-progress", UpdateProgress {
        percent: 100,
        transferred: downloaded,
        total: total_bytes,
        status: "Instalando nova versão e reiniciando o Luxmc...".to_string(),
    });

    tokio::time::sleep(std::time::Duration::from_millis(500)).await;

    #[cfg(target_os = "linux")]
    {
        use std::os::unix::fs::PermissionsExt;
        let _ = std::fs::set_permissions(&temp_file_path, std::fs::Permissions::from_mode(0o755));

        // Se estiver rodando como AppImage, substitui o AppImage original e reinicia
        if let Ok(appimage_path) = std::env::var("APPIMAGE") {
            let appimage_target = PathBuf::from(&appimage_path);
            if temp_file_path.to_string_lossy().ends_with(".AppImage") {
                if let Err(e) = std::fs::rename(&temp_file_path, &appimage_target) {
                    tracing::warn!(error = %e, "Falha no rename direto do AppImage, usando fallback");
                    let old_target = appimage_target.with_extension("bak_update");
                    let _ = std::fs::rename(&appimage_target, &old_target);
                    if std::fs::copy(&temp_file_path, &appimage_target).is_ok() {
                        let _ = std::fs::remove_file(&old_target);
                        let _ = std::fs::remove_file(&temp_file_path);
                    } else {
                        let _ = std::fs::rename(&old_target, &appimage_target);
                    }
                }
                let _ = std::fs::set_permissions(&appimage_target, std::fs::Permissions::from_mode(0o755));
                std::process::Command::new(&appimage_target)
                    .spawn()
                    .map_err(|e| AppError::Internal(format!("Falha ao iniciar AppImage atualizado: {e}")))?;
                std::process::exit(0);
            }
        }

        // Se for um AppImage baixado em execução de desenvolvimento/binário:
        if temp_file_path.to_string_lossy().ends_with(".AppImage") {
            std::process::Command::new(&temp_file_path)
                .spawn()
                .map_err(|e| AppError::Internal(format!("Falha ao iniciar AppImage: {e}")))?;
            std::process::exit(0);
        }

        // Caso seja binário padrão no Linux
        if let Ok(current_exe) = std::env::current_exe() {
            if let Err(e) = std::fs::rename(&temp_file_path, &current_exe) {
                tracing::warn!(error = %e, "Falha no rename direto do executavel, usando fallback");
                let old_target = current_exe.with_extension("bak_update");
                let _ = std::fs::rename(&current_exe, &old_target);
                if std::fs::copy(&temp_file_path, &current_exe).is_ok() {
                    let _ = std::fs::remove_file(&old_target);
                    let _ = std::fs::remove_file(&temp_file_path);
                } else {
                    let _ = std::fs::rename(&old_target, &current_exe);
                }
            }
            let _ = std::fs::set_permissions(&current_exe, std::fs::Permissions::from_mode(0o755));
            std::process::Command::new(&current_exe)
                .spawn()
                .map_err(|e| AppError::Internal(format!("Falha ao reiniciar nova versão: {e}")))?;
            std::process::exit(0);
        }
    }

    #[cfg(target_os = "windows")]
    {
        if temp_file_path.to_string_lossy().ends_with(".exe") {
            let mut cmd = std::process::Command::new(&temp_file_path);
            cmd.spawn()
                .map_err(|e| AppError::Internal(format!("Falha ao iniciar instalador: {e}")))?;
            std::process::exit(0);
        }
    }

    #[cfg(target_os = "macos")]
    {
        let _ = open::that(&temp_file_path);
        std::process::exit(0);
    }

    Ok(())
}
