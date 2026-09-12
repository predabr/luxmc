use serde::{Deserialize, Serialize};
use tauri::Emitter;
use tauri::State;

use crate::core::downloader::DownloadManager;
use crate::core::java::JavaRuntimeManager;
use crate::core::launcher::GameLauncher;
use crate::core::minecraft;
use crate::db::models::AccountRow;
use crate::error::{AppError, AppResult};
use crate::state::AppState;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LaunchRequest {
    pub version_id: String,
    pub account_id: String,
    pub profile_id: String,
    pub enable_vulkan: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LaunchResponse {
    pub pid: u32,
}

pub fn compute_offline_uuid(username: &str) -> String {
    let digest = md5::compute(format!("OfflinePlayer:{}", username).as_bytes());
    let mut bytes = digest.0;
    bytes[6] = (bytes[6] & 0x0f) | 0x30;
    bytes[8] = (bytes[8] & 0x3f) | 0x80;
    uuid::Uuid::from_bytes(bytes).simple().to_string()
}

#[tauri::command]
pub async fn launch_game(
    state: State<'_, AppState>,
    app: tauri::AppHandle,
    request: LaunchRequest,
) -> AppResult<LaunchResponse> {
    if request.enable_vulkan == Some(false) {
        std::env::set_var("LUXMC_DISABLE_VULKAN", "1");
    } else {
        std::env::remove_var("LUXMC_DISABLE_VULKAN");
    }
    app.emit(
        "launcher-log",
        format!("Starting launch for version {}", request.version_id),
    )
    .ok();

    let db = crate::db::shared_db().await?;

    let account: AccountRow =
        sqlx::query_as::<_, AccountRow>("SELECT * FROM accounts WHERE uuid = ? OR id = ?")
            .bind(&request.account_id)
            .bind(&request.account_id)
            .fetch_optional(db.pool())
            .await?
            .unwrap_or_else(|| {
                let fallback_username = if request.account_id.contains("offline") {
                    request
                        .account_id
                        .replace("offline-", "")
                        .replace("offline_", "")
                } else {
                    "Player".to_string()
                };
                AccountRow {
                    id: request.account_id.clone(),
                    username: fallback_username,
                    uuid: request.account_id.clone(),
                    refresh_token: String::new(),
                    access_token: Some(String::new()),
                    expires_at: None,
                    created_at: chrono::Utc::now(),
                    updated_at: chrono::Utc::now(),
                }
            });

    let profile =
        sqlx::query_as::<_, crate::db::models::ProfileRow>("SELECT * FROM profiles WHERE id = ?")
            .bind(&request.profile_id)
            .fetch_optional(db.pool())
            .await?
            .ok_or_else(|| {
                let msg = "No profile found. Please create a profile first.";
                app.emit("launcher-log", msg).ok();
                AppError::NotFound(msg.into())
            })?;

    app.emit(
        "launcher-log",
        format!(
            "Account: {}, Version: {}, Profile: {}",
            account.username, profile.mc_version, profile.name
        ),
    )
    .ok();

    let version_row = crate::db::schema::versions::get(&db, &request.version_id)
        .await?
        .ok_or_else(|| {
            let msg = format!(
                "Version {} not found. Try refreshing the version list.",
                request.version_id
            );
            app.emit("launcher-log", &msg).ok();
            AppError::NotFound(msg)
        })?;

    app.emit(
        "launcher-log",
        format!("Fetching version detail for {}...", request.version_id),
    )
    .ok();
    let detail = minecraft::fetch_version_detail(&state.http, &version_row.url).await?;
    app.emit(
        "launcher-log",
        format!(
            "Version detail fetched. Main class: {}",
            detail.main_class.as_deref().unwrap_or("unknown")
        ),
    )
    .ok();

    let base_dir = directories::ProjectDirs::from("io", "github", "Luxmc")
        .ok_or_else(|| AppError::InvalidState("could not determine data dir".into()))?;
    let data_dir = base_dir.data_dir().to_path_buf();

    app.emit("launcher-log", "Checking environment...").ok();

    let has_unzip = std::process::Command::new("which")
        .arg("unzip")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false);
    if !has_unzip {
        app.emit(
            "launcher-log",
            "WARNING: 'unzip' not found. Natives extraction may fail.",
        )
        .ok();
    }

    app.emit("launcher-log", "Verifying download...").ok();
    let downloader =
        DownloadManager::new(state.http.clone(), data_dir.clone()).with_app(app.clone());
    let java = JavaRuntimeManager::new(state.http.clone(), data_dir.clone()).with_app(app.clone());

    downloader.download_version(&detail).await?;
    app.emit("launcher-log", "Download verified. Validating version...")
        .ok();
    downloader.validate_version(&detail).await?;
    app.emit("launcher-log", "Version validated.").ok();

    let launcher = GameLauncher::new(downloader, java).with_app(app.clone());

    let game_dir = std::path::PathBuf::from(&profile.game_dir);
    tokio::fs::create_dir_all(&game_dir).await?;

    #[cfg(target_os = "linux")]
    {
        let user = std::env::var("USER").unwrap_or_else(|_| "root".to_string());
        let _ = std::process::Command::new("pkill")
            .args(["-f", "-u", &user, "net.minecraft.client.main.Main"])
            .output();
        tokio::time::sleep(std::time::Duration::from_millis(500)).await;
    }

    let token_str = account.access_token.as_deref().unwrap_or("");
    let is_real_msa = !token_str.is_empty()
        && !token_str.starts_with("offline")
        && !token_str.starts_with("token_")
        && !token_str.starts_with("dev-")
        && token_str.len() > 100;

    let (final_uuid, final_token, user_type) = if is_real_msa {
        (account.uuid.clone(), token_str.to_string(), "msa")
    } else {
        let mojang_uuid = match state
            .http
            .get(format!(
                "https://api.mojang.com/users/profiles/minecraft/{}",
                account.username
            ))
            .timeout(std::time::Duration::from_millis(1500))
            .send()
            .await
        {
            Ok(resp) if resp.status().is_success() => {
                if let Ok(json) = resp.json::<serde_json::Value>().await {
                    json.get("id")
                        .and_then(|i| i.as_str())
                        .map(|s| s.to_string())
                } else {
                    None
                }
            }
            _ => None,
        };

        let resolved_uuid = mojang_uuid.unwrap_or_else(|| compute_offline_uuid(&account.username));
        (resolved_uuid, "-".to_string(), "mojang")
    };

    app.emit(
        "launcher-log",
        format!(
            "Spawning Java process for {} ({}, {})...",
            account.username, user_type, final_uuid
        ),
    )
    .ok();
    let pid = launcher
        .launch(
            &detail,
            &account.username,
            &final_uuid,
            &final_token,
            user_type,
            &game_dir,
            &profile,
        )
        .await?;

    app.emit("launcher-log", format!("Game launched with PID {}", pid))
        .ok();
    Ok(LaunchResponse { pid })
}
