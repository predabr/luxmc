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
    pub skin_url: Option<String>,
    pub skin_variant: Option<String>,
    pub cape_url: Option<String>,
    pub server_ip: Option<String>,
    pub server_port: Option<u16>,
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

    let mut account: AccountRow =
        sqlx::query_as::<_, AccountRow>("SELECT * FROM accounts WHERE uuid = ? OR id = ? OR username = ?")
            .bind(&request.account_id)
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
                    skin_url: None,
                    skin_variant: None,
                    cape_url: None,
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

    let instance_dir = std::path::PathBuf::from(&profile.game_dir);
    let manifest_path = instance_dir.join("manifest.json");
    if manifest_path.exists() {
        if let Ok(manifest_content) = tokio::fs::read_to_string(&manifest_path).await {
            if let Ok(manifest) = serde_json::from_str::<serde_json::Value>(&manifest_content) {
                if let Some(files) = manifest.get("files").and_then(|f| f.as_array()) {
                    let total_manifest_files = files.len();
                    let mods_dir = instance_dir.join("mods");
                    let mut existing_jar_count = 0;
                    if let Ok(mut rd) = tokio::fs::read_dir(&mods_dir).await {
                        while let Ok(Some(entry)) = rd.next_entry().await {
                            let name = entry.file_name().to_string_lossy().to_string();
                            if name.ends_with(".jar") || name.ends_with(".jar.disabled") {
                                if let Ok(meta) = entry.metadata().await {
                                    if meta.len() > 100 {
                                        existing_jar_count += 1;
                                    }
                                }
                            }
                        }
                    }
                    if total_manifest_files > 0 && existing_jar_count < total_manifest_files {
                        let missing = total_manifest_files.saturating_sub(existing_jar_count);
                        app.emit("launcher-log", format!(
                            "Modpack possui mods pendentes ({}/{} instalados). Reparando {} mods ausentes...",
                            existing_jar_count, total_manifest_files, missing
                        )).ok();
                        let _ = crate::commands::instances::instance_repair_modpack(
                            app.clone(),
                            state.clone(),
                            request.profile_id.clone(),
                        ).await;
                    }
                }
            }
        }
    }

    let clean_req_ver = request.version_id.split('-').next().unwrap_or(&request.version_id);
    let version_row = match crate::db::schema::versions::get(&db, &request.version_id).await? {
        Some(v) => v,
        None => {
            if let Ok(Some(v_clean)) = crate::db::schema::versions::get(&db, clean_req_ver).await {
                v_clean
            } else {
                app.emit("launcher-log", format!("Versão {} não encontrada localmente. Buscando manifesto oficial...", request.version_id)).ok();
                let manifest = minecraft::fetch_version_manifest(&state.http).await?;
                if let Some(target) = manifest.versions.iter().find(|v| v.id == request.version_id || v.id == clean_req_ver) {
                    let now = chrono::Utc::now().to_rfc3339();
                    let vrow = crate::db::schema::versions::VersionRow {
                        id: target.id.clone(),
                        version_type: target.version_type.clone(),
                        url: target.url.clone(),
                        time: target.release_time.clone(),
                        release_time: target.release_time.clone(),
                        fetched_at: now,
                    };
                    crate::db::schema::versions::upsert(&db, &vrow).await?;
                    vrow
                } else {
                    let msg = format!(
                        "Version {} not found in official manifest. Try refreshing the version list.",
                        request.version_id
                    );
                    app.emit("launcher-log", &msg).ok();
                    return Err(AppError::NotFound(msg));
                }
            }
        }
    };

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

    if !account.refresh_token.is_empty() {
        let client_id = crate::commands::auth::get_configured_client_id().await;
        if let Ok(refreshed) = state.auth.refresh_account_with_client_id(&account.refresh_token, Some(&client_id)).await {
            account.access_token = Some(refreshed.access_token.clone());
            account.refresh_token = refreshed.refresh_token.clone();
            account.expires_at = Some(chrono::DateTime::from_timestamp(refreshed.expires_at, 0).unwrap_or_default());
            if refreshed.skin_url.is_some() {
                account.skin_url = refreshed.skin_url;
            }
            if refreshed.skin_variant.is_some() {
                account.skin_variant = refreshed.skin_variant;
            }
            account.updated_at = chrono::Utc::now();
            let _ = crate::db::schema::accounts::upsert(&db, &account).await;
        }
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
        (compute_offline_uuid(&account.username), "-".to_string(), "mojang")
    };

    let has_explicit_custom_skin = request
        .skin_url
        .as_deref()
        .map(|s| {
            let trimmed = s.trim();
            !trimmed.is_empty()
                && (trimmed.starts_with("data:image/")
                    || (!trimmed.starts_with("http://") && !trimmed.starts_with("https://")))
        })
        .unwrap_or(false);

    let effective_skin_url = if is_real_msa && !has_explicit_custom_skin {
        None
    } else {
        request
            .skin_url
            .filter(|s| !s.trim().is_empty())
            .or_else(|| account.skin_url.filter(|s| !s.trim().is_empty()))
            .or_else(|| (!is_real_msa).then(|| format!("https://minotar.net/skin/{}", account.username)))
    };

    let effective_skin_variant = request
        .skin_variant
        .filter(|s| !s.trim().is_empty())
        .or_else(|| account.skin_variant.filter(|s| !s.trim().is_empty()))
        .unwrap_or_else(|| "classic".to_string());

    let effective_cape_url = request
        .cape_url
        .filter(|c| !c.trim().is_empty())
        .or_else(|| account.cape_url.filter(|c| !c.trim().is_empty()));

    let mods_dir = game_dir.join("mods");
    if mods_dir.is_dir() {
        let shield_result = crate::commands::shield::scan_mods_directory(&mods_dir);
        if !shield_result.is_clean {
            app.emit("launcher-shield-warning", &shield_result).ok();
            app.emit(
                "launcher-log",
                format!(
                    "⚠️ Luxmc Shield: detectada(s) {} possível(is) ameaça(s) nos mods da instância!",
                    shield_result.threats.len()
                ),
            )
            .ok();

            if shield_result.threats.iter().any(|t| t.severity == "critical") {
                return Err(AppError::InvalidState(
                    "Luxmc Shield bloqueou o lançamento: detectado mod com malware crítico.".into(),
                ));
            }
        }
    }

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
            effective_skin_url.as_deref(),
            Some(&effective_skin_variant),
            effective_cape_url.as_deref(),
            request.server_ip.as_deref(),
            request.server_port,
        )
        .await?;

    app.emit("launcher-log", format!("Game launched with PID {}", pid))
        .ok();
    Ok(LaunchResponse { pid })
}
