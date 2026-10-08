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
    let _launch_guard = state
        .launch_lock
        .try_lock()
        .map_err(|_| AppError::InvalidState("Já existe um lançamento em andamento.".into()))?;

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

    let preparation_started = std::time::Instant::now();
    let mut phases=Vec::<serde_json::Value>::new();
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

    let mut profile =
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
    crate::core::instance_paths::isolate(&mut profile, &data_dir).await?;
    let backup_started=std::time::Instant::now();
    crate::commands::world_backup::automatic(&profile.id).await?;
    phases.push(serde_json::json!({"name":"backup","seconds":backup_started.elapsed().as_secs_f64()}));
    let verification_started=std::time::Instant::now();
    let fingerprint_profile = profile.clone();
    let verified = tokio::task::spawn_blocking(move || crate::core::launcher::launch_state::is_valid(&fingerprint_profile))
        .await.map_err(|error| AppError::Internal(error.to_string()))?;
    let fast_launch = !crate::commands::instance_lab::isolation_testing(&profile.id)? && verified;

    if crate::commands::instance_lab::isolation_testing(&profile.id)? {
        emit_log("Diagnóstico de mods ativo: teste iniciado com a seleção temporária de JARs.");
    } else if fast_launch {
        emit_log("Fast Launch: integridade local preservada; verificação profunda ignorada.");
    } else {
        emit_log("Verificando integridade dos arquivos do modpack...");
        let integrity_started = std::time::Instant::now();
        crate::commands::instances::heal_modpack(state, &profile).await?;
        crate::commands::instances::repair_instance_duplicates(&profile).await?;
        let audit_root = std::path::PathBuf::from(&profile.game_dir);
        let audit = tokio::task::spawn_blocking(move || crate::core::mods::validation::audit(&audit_root,false)).await.map_err(|e| AppError::Internal(e.to_string()))??;
        if !audit.errors.is_empty() { return Err(AppError::InvalidInput(audit.errors.join("\n"))); }
        for warning in audit.warnings { emit_log(&warning); }
        crate::core::launcher::launch_state::store(&profile)?;
        if profile.force_full_verification {
            profile.force_full_verification = false;
            crate::db::schema::profiles::upsert(&db, &profile).await?;
        }
        emit_log(&format!("Integridade concluída em {:.1}s. Preparando Java e loader...", integrity_started.elapsed().as_secs_f32()));
    }

    phases.push(serde_json::json!({"name":"integrity","seconds":verification_started.elapsed().as_secs_f64()}));
    let metadata_started=std::time::Instant::now();
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
    let detail_request = if fast_launch && (local_ver_json.exists() || local_clean_ver_json.exists() || local_prof_ver_json.exists()) {
        Err(AppError::NotFound("Fast Launch usa o manifesto local".into()))
    } else {
        minecraft::fetch_version_detail(&state.http, &version_row.url).await
    };
    let detail = match detail_request {
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
    phases.push(serde_json::json!({"name":"metadata","seconds":metadata_started.elapsed().as_secs_f64()}));
    let downloads_started=std::time::Instant::now();
    let mut downloader = DownloadManager::new(state.http.clone(), data_dir.clone());
    if let Some(ref a) = app {
        downloader = downloader.with_app(a.clone());
    }
    let mut java = JavaRuntimeManager::new(state.http.clone(), data_dir.clone());
    if let Some(ref a) = app {
        java = java.with_app(a.clone());
    }

    let assets_ready = detail.asset_index.as_ref().map_or(true, |index| downloader.assets_dir().join(format!(".complete_{}", index.id)).exists());
    let version_ready = fast_launch && assets_ready && downloader.validate_version(&detail).await.is_ok();
    if version_ready {
        emit_log("Fast Launch: Minecraft e bibliotecas já estão instalados; usando arquivos locais.");
    } else {
        downloader.download_version(&detail).await?;
        downloader.validate_version(&detail).await?;
    }
    emit_log("Download verified. Validating version...");
    emit_log("Version validated.");

    let mut launcher = GameLauncher::new(downloader, java);
    phases.push(serde_json::json!({"name":"minecraftFiles","seconds":downloads_started.elapsed().as_secs_f64()}));
    let runtime_started=std::time::Instant::now();
    if let Some(ref a) = app {
        launcher = launcher.with_app(a.clone());
    }

    let game_dir = std::path::PathBuf::from(&profile.game_dir);
    tokio::fs::create_dir_all(&game_dir).await?;


    if !account.refresh_token.is_empty() && account.expires_at.map_or(true, |expiry| expiry <= chrono::Utc::now() + chrono::Duration::minutes(10)) {
        let client_id = crate::commands::auth::get_configured_client_id().await;
        match state.auth.refresh_account_with_client_id(&account.refresh_token, Some(&client_id)).await {
            Ok(refreshed) => {
                account.access_token = Some(refreshed.access_token.clone());
                account.refresh_token = refreshed.refresh_token.clone();
                account.expires_at = Some(chrono::DateTime::from_timestamp(refreshed.expires_at, 0).unwrap_or_default());
                if account.skin_url.is_none() && refreshed.skin_url.is_some() {
                    account.skin_url = refreshed.skin_url;
                }
                if account.skin_variant.is_none() && refreshed.skin_variant.is_some() {
                    account.skin_variant = refreshed.skin_variant;
                }
                account.updated_at = chrono::Utc::now();
                if let Err(error) = crate::db::schema::accounts::upsert(&db, &account).await {
                    tracing::error!(
                        target: "auth",
                        "Falha ao persistir o refresh_token rotacionado para {}: {error}. A sessão pode exigir novo login na próxima execução.",
                        account.username
                    );
                }
            }
            Err(error) => {
                emit_log(&format!("Aviso: falha ao renovar a sessão Microsoft: {}", error));
                let expired = account
                    .expires_at
                    .map(|exp| exp <= chrono::Utc::now() - chrono::Duration::minutes(5))
                    .unwrap_or(true);
                if expired {
                    return Err(AppError::InvalidState(
                        "Sessão Microsoft expirada e não foi possível renová-la. Faça login novamente antes de jogar.".into(),
                    ));
                }
            }
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

    let effective_skin_url = request
        .skin_url
        .filter(|s| !s.trim().is_empty())
        .or_else(|| account.skin_url.filter(|s| !s.trim().is_empty()))
        .or_else(|| (!is_real_msa).then(|| format!("https://minotar.net/skin/{}", account.username)));

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
        let shield_result = tokio::task::spawn_blocking(move || crate::commands::shield::scan_mods_directory(&mods_dir)).await
            .map_err(|error| AppError::Internal(error.to_string()))?;
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

    if let Some(hook) = profile.pre_launch_hook.as_deref() {
        emit_log(&format!("Executando hook pré-lançamento: {}", hook.trim()));
        let hook_env = crate::core::hooks::HookEnv {
            profile_id: &profile.id,
            profile_name: &profile.name,
            version_id: &detail.id,
            game_dir: &game_dir,
            exit_code: None,
        };
        crate::core::hooks::run("pré-lançamento", hook, &hook_env).await?;
        emit_log("Hook pré-lançamento concluído.");
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

    emit_log(&format!("Game launched with PID {}. Launcher preparation: {:.2}s", pid, preparation_started.elapsed().as_secs_f64()));
    phases.push(serde_json::json!({"name":"javaLoaderAndProcess","seconds":runtime_started.elapsed().as_secs_f64()}));
    let _ = crate::commands::experience::record_performance(&request.profile_id,serde_json::json!({"kind":"preparation","seconds":preparation_started.elapsed().as_secs_f64(),"pid":pid,"phases":phases})).await;
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

async fn is_process_alive(pid: u32) -> bool {
    let spid = sysinfo::Pid::from_u32(pid);
    let mut sys = sysinfo::System::new();
    sys.refresh_processes(sysinfo::ProcessesToUpdate::Some(&[spid]), true);
    sys.process(spid).is_some()
}

async fn wait_for_process_exit(pid: u32, timeout_ms: u64) -> bool {
    let deadline = std::time::Instant::now() + std::time::Duration::from_millis(timeout_ms);
    loop {
        if !is_process_alive(pid).await {
            return true;
        }
        if std::time::Instant::now() >= deadline {
            return false;
        }
        tokio::time::sleep(std::time::Duration::from_millis(250)).await;
    }
}

#[tauri::command]
pub async fn stop_game(pid: Option<u32>) -> AppResult<bool> {
    let target_pid = match pid {
        Some(p) if p > 0 => p,
        _ => crate::core::launcher::get_active_game_pid(),
    };

    if target_pid == 0 {
        return Ok(false);
    }

    tracing::info!(target: "launch", "Terminating Minecraft process with PID {}", target_pid);

    if let Some(game_dir) = crate::core::launcher::get_active_game_dir() {
        let crash_dir = game_dir.join("local").join("crash_assistant");
        if crash_dir.is_dir() {
            let normal_stop = crash_dir.join(format!("normal_stop_pid{}.tmp", target_pid));
            let prevent_window = crash_dir.join(format!("prevent_crash_assistant_window_pid{}.tmp", target_pid));
            let _ = tokio::fs::write(&normal_stop, b"").await;
            let _ = tokio::fs::write(&prevent_window, b"").await;
        }
    }

    #[cfg(unix)]
    let exited = {
        let _ = crate::core::process::tokio_command("pkill")
            .args(["-TERM", "-P", &target_pid.to_string()])
            .output()
            .await;
        let _ = crate::core::process::tokio_command("kill")
            .args(["-TERM", &target_pid.to_string()])
            .output()
            .await;
        wait_for_process_exit(target_pid, 6_000).await
    };

    #[cfg(windows)]
    let exited = {
        let _ = crate::core::process::tokio_command("taskkill")
            .args(["/PID", &target_pid.to_string(), "/T"])
            .output()
            .await;
        wait_for_process_exit(target_pid, 6_000).await
    };

    if !exited {
        tracing::info!(target: "launch", "PID {} ignored graceful shutdown, forcing termination", target_pid);
        #[cfg(unix)]
        {
            let _ = crate::core::process::tokio_command("pkill")
                .args(["-9", "-P", &target_pid.to_string()])
                .output()
                .await;

            let _ = crate::core::process::tokio_command("kill")
                .args(["-9", "--", &format!("-{}", target_pid)])
                .output()
                .await;

            let _ = crate::core::process::tokio_command("kill")
                .args(["-9", &target_pid.to_string()])
                .output()
                .await;
        }

        #[cfg(windows)]
        {
            let _ = crate::core::process::tokio_command("taskkill")
                .args(["/PID", &target_pid.to_string(), "/F", "/T"])
                .output()
                .await;
        }

    } else {
        #[cfg(unix)]
        {
            let _ = crate::core::process::tokio_command("pkill")
                .args(["-KILL", "-P", &target_pid.to_string()])
                .output()
                .await;
        }
        #[cfg(windows)]
        {
            let _ = crate::core::process::tokio_command("taskkill")
                .args(["/PID", &target_pid.to_string(), "/T", "/F"])
                .output()
                .await;
        }
        tracing::info!(target: "launch", "PID {} stopped gracefully", target_pid);
    }

    crate::core::launcher::clear_active_game_pid();
    crate::core::launcher::clear_active_game_dir();
    Ok(true)
}
