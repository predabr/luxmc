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

pub async fn launch_game_core(
    state: &AppState,
    app: Option<tauri::AppHandle>,
    request: LaunchRequest,
) -> AppResult<LaunchResponse> {
    if request.enable_vulkan == Some(false) {
        std::env::set_var("LUXMC_DISABLE_VULKAN", "1");
    } else {
        std::env::remove_var("LUXMC_DISABLE_VULKAN");
    }

    let emit_log = |msg: &str| {
        if let Some(ref a) = app {
            a.emit("launcher-log", msg).ok();
        }
        tracing::info!(target: "launch", "{}", msg);
    };

    emit_log(&format!("Starting launch for version {}", request.version_id));

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
                emit_log(msg);
                AppError::NotFound(msg.into())
            })?;

    emit_log(&format!(
        "Account: {}, Version: {}, Profile: {}",
        account.username, profile.mc_version, profile.name
    ));

    let base_dir = directories::ProjectDirs::from("io", "github", "Luxmc")
        .ok_or_else(|| AppError::InvalidState("could not determine data dir".into()))?;
    let data_dir = base_dir.data_dir().to_path_buf();

    emit_log("Verificando integridade dos arquivos do modpack...");
    if let Err(e) = crate::commands::instances::heal_modpack(state, &profile).await {
        emit_log(&format!("Aviso na verificação do modpack: {}. Continuando lançamento...", e));
    }

    let raw_req = request.version_id.trim().trim_matches('\'').trim_matches('"');
    let clean_req_ver = raw_req.split('-').next().unwrap_or(raw_req);
    let profile_mc_ver = profile.mc_version.trim().trim_matches('\'').trim_matches('"');

    let version_row = match crate::db::schema::versions::get(&db, raw_req).await? {
        Some(v) => v,
        None => {
            if let Ok(Some(v_clean)) = crate::db::schema::versions::get(&db, clean_req_ver).await {
                v_clean
            } else if let Ok(Some(v_prof)) = crate::db::schema::versions::get(&db, profile_mc_ver).await {
                v_prof
            } else {
                emit_log(&format!("Versão {} não encontrada localmente. Buscando manifesto oficial...", raw_req));
                let manifest = minecraft::fetch_version_manifest(&state.http).await?;
                if let Some(target) = manifest.versions.iter().find(|v| {
                    v.id == raw_req || v.id == clean_req_ver || v.id == profile_mc_ver
                }) {
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
                        "Version {} (ou versão base {}) não encontrada no manifesto oficial. Atualize a lista de versões.",
                        raw_req, profile_mc_ver
                    );
                    emit_log(&msg);
                    return Err(AppError::NotFound(msg));
                }
            }
        }
    };

    let local_ver_json = data_dir.join("versions").join(raw_req).join(format!("{}.json", raw_req));
    let local_clean_ver_json = data_dir.join("versions").join(clean_req_ver).join(format!("{}.json", clean_req_ver));
    let local_prof_ver_json = data_dir.join("versions").join(profile_mc_ver).join(format!("{}.json", profile_mc_ver));

    emit_log(&format!("Fetching version detail for {}...", raw_req));
    let detail = match minecraft::fetch_version_detail(&state.http, &version_row.url).await {
        Ok(d) => {
            let version_dir = data_dir.join("versions").join(&d.id);
            if tokio::fs::create_dir_all(&version_dir).await.is_ok() {
                let json_path = version_dir.join(format!("{}.json", d.id));
                if let Ok(serialized) = serde_json::to_string(&d) {
                    let _ = tokio::fs::write(&json_path, serialized).await;
                }
            }
            d
        }
        Err(err) => {
            if local_ver_json.exists() {
                emit_log("Modo offline ativo: carregando especificações locais da versão...");
                let content = tokio::fs::read_to_string(&local_ver_json).await
                    .map_err(AppError::Io)?;
                serde_json::from_str::<minecraft::VersionDetail>(&content)
                    .map_err(AppError::Serde)?
            } else if local_clean_ver_json.exists() {
                emit_log("Modo offline ativo: carregando especificações locais da versão base...");
                let content = tokio::fs::read_to_string(&local_clean_ver_json).await
                    .map_err(AppError::Io)?;
                serde_json::from_str::<minecraft::VersionDetail>(&content)
                    .map_err(AppError::Serde)?
            } else if local_prof_ver_json.exists() {
                emit_log("Modo offline ativo: carregando especificações da versão da instância...");
                let content = tokio::fs::read_to_string(&local_prof_ver_json).await
                    .map_err(AppError::Io)?;
                serde_json::from_str::<minecraft::VersionDetail>(&content)
                    .map_err(AppError::Serde)?
            } else {
                return Err(err);
            }
        }
    };

    emit_log(&format!(
        "Version detail fetched. Main class: {}",
        detail.main_class.as_deref().unwrap_or("unknown")
    ));

    emit_log("Verifying download...");
    let mut downloader = DownloadManager::new(state.http.clone(), data_dir.clone());
    if let Some(ref a) = app {
        downloader = downloader.with_app(a.clone());
    }
    let mut java = JavaRuntimeManager::new(state.http.clone(), data_dir.clone());
    if let Some(ref a) = app {
        java = java.with_app(a.clone());
    }

    if let Err(e) = downloader.download_version(&detail).await {
        let client_jar = data_dir.join("versions").join(&detail.id).join(format!("{}.jar", detail.id));
        if client_jar.exists() {
            emit_log(&format!("Aviso de rede no download (modo offline): {}. Usando arquivos locais existentes.", e));
        } else {
            return Err(e);
        }
    }

    emit_log("Download verified. Validating version...");
    if let Err(e) = downloader.validate_version(&detail).await {
        let client_jar = data_dir.join("versions").join(&detail.id).join(format!("{}.jar", detail.id));
        if client_jar.exists() {
            emit_log(&format!("Aviso de validação (modo offline): {}. Prosseguindo com arquivos locais.", e));
        } else {
            return Err(e);
        }
    }
    emit_log("Version validated.");

    let mut launcher = GameLauncher::new(downloader, java);
    if let Some(ref a) = app {
        launcher = launcher.with_app(a.clone());
    }

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
            if let Some(ref a) = app {
                a.emit("launcher-shield-warning", &shield_result).ok();
            }
            emit_log(&format!(
                "⚠️ Luxmc Shield: detectada(s) {} possível(is) ameaça(s) nos mods da instância!",
                shield_result.threats.len()
            ));

            if shield_result.threats.iter().any(|t| t.severity == "critical") {
                return Err(AppError::InvalidState(
                    "Luxmc Shield bloqueou o lançamento: detectado mod com malware crítico.".into(),
                ));
            }
        }
    }

    emit_log(&format!(
        "Spawning Java process for {} ({}, {})...",
        account.username, user_type, final_uuid
    ));
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

    emit_log(&format!("Game launched with PID {}", pid));
    Ok(LaunchResponse { pid })
}

#[tauri::command]
pub async fn launch_game(
    state: State<'_, AppState>,
    app: tauri::AppHandle,
    request: LaunchRequest,
) -> AppResult<LaunchResponse> {
    launch_game_core(state.inner(), Some(app), request).await
}

pub async fn launch_game_daemon(
    state: &AppState,
    request: LaunchRequest,
) -> AppResult<LaunchResponse> {
    launch_game_core(state, None, request).await
}
