use serde::Serialize;
use tauri::State;

use crate::db::models::{AccountRow, ProfileRow};
use crate::state::AppState;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AppInfo {
    pub name: &'static str,
    pub version: &'static str,
    pub identifier: &'static str,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AppInitState {
    pub dev_mode: bool,
    pub account: Option<AccountRow>,
    pub profiles: Vec<ProfileRow>,
    pub active_profile_id: Option<String>,
    pub stress_test: bool,
}

#[tauri::command]
pub async fn ping() -> Result<String, crate::error::AppError> {
    Ok("pong".into())
}

#[tauri::command]
pub fn app_info() -> AppInfo {
    AppInfo {
        name: "Luxmc",
        version: env!("CARGO_PKG_VERSION"),
        identifier: "io.github.luxmc",
    }
}

async fn fetch_mojang_textures(uuid: &str) -> Option<(String, Option<String>, Option<String>)> {
    let clean_uuid = uuid.replace('-', "");
    if clean_uuid.len() != 32 {
        return None;
    }
    let url = format!("https://sessionserver.mojang.com/session/minecraft/profile/{}", clean_uuid);
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(4))
        .build()
        .ok()?;
    let resp = client.get(&url).send().await.ok()?;
    if !resp.status().is_success() {
        return None;
    }
    let body: serde_json::Value = resp.json().await.ok()?;
    let props = body.get("properties")?.as_array()?;
    let textures_prop = props.iter().find(|p| p.get("name").and_then(|n| n.as_str()) == Some("textures"))?;
    let b64 = textures_prop.get("value")?.as_str()?;
    use base64::Engine;
    let decoded = base64::engine::general_purpose::STANDARD.decode(b64).ok()?;
    let parsed: serde_json::Value = serde_json::from_slice(&decoded).ok()?;
    let textures = parsed.get("textures")?;
    let skin = textures.get("SKIN");
    let skin_url = skin.and_then(|s| s.get("url")).and_then(|u| u.as_str()).map(|s| {
        if s.starts_with("http://") {
            s.replacen("http://", "https://", 1)
        } else {
            s.to_string()
        }
    });
    let skin_variant = skin.and_then(|s| s.get("metadata")).and_then(|m| m.get("model")).and_then(|v| v.as_str()).map(|s| s.to_string());
    let cape_url = textures.get("CAPE").and_then(|c| c.get("url")).and_then(|u| u.as_str()).map(|s| {
        if s.starts_with("http://") {
            s.replacen("http://", "https://", 1)
        } else {
            s.to_string()
        }
    });
    skin_url.map(|s_url| (s_url, skin_variant, cape_url))
}

#[tauri::command]
pub async fn app_init(
    state: State<'_, AppState>,
) -> Result<AppInitState, crate::error::AppError> {
    app_init_core(&state).await
}

pub async fn app_init_core(
    state: &AppState,
) -> Result<AppInitState, crate::error::AppError> {
    let dev_mode = std::env::var("LUXMC_DEV_MODE").unwrap_or_default() == "1";
    let stress_test = std::env::var("LUXMC_STRESS_TEST").unwrap_or_default() == "1";
    let db = crate::db::shared_db().await?;

    let profiles = crate::db::schema::profiles::list(&db).await?;

    let settings: serde_json::Value = {
        use sqlx::Row;
        let row = sqlx::query("SELECT value FROM app_settings WHERE key = 'app'")
            .fetch_optional(db.pool())
            .await?;
        match row {
            Some(r) => {
                let raw: String = r.try_get("value")?;
                serde_json::from_str(&raw).unwrap_or(serde_json::Value::Null)
            }
            None => serde_json::Value::Null,
        }
    };

    let active_account_id = settings
        .get("activeAccountId")
        .and_then(|v| v.as_str());

    let mut account = if let Some(id) = active_account_id {
        if let Ok(Some(acc)) = crate::db::schema::accounts::get_by_id(&db, id).await {
            Some(acc)
        } else {
            crate::db::schema::accounts::list(&db).await?.into_iter().next()
        }
    } else {
        crate::db::schema::accounts::list(&db).await?.into_iter().next()
    };

    // Auto-refresh token if expired or close to expiry for Microsoft accounts
    if let Some(ref acc) = account {
        if !acc.refresh_token.is_empty() {
            let now = chrono::Utc::now();
            let is_expired = match acc.expires_at {
                Some(exp) => exp <= now + chrono::Duration::minutes(30),
                None => true,
            };
            if is_expired {
                let client_id = crate::commands::auth::get_configured_client_id().await;
                if let Ok(refreshed) = state.auth.refresh_account_with_client_id(&acc.refresh_token, Some(&client_id)).await {
                    let is_custom_cape = acc.cape_url.as_deref().map(|c| {
                        let trimmed = c.trim();
                        !trimmed.is_empty()
                            && !trimmed.starts_with("https://textures.minecraft.net/")
                            && !trimmed.starts_with("http://textures.minecraft.net/")
                    }).unwrap_or(false);
                    let effective_cape = if is_custom_cape {
                        acc.cape_url.clone()
                    } else {
                        refreshed.cape_url.or_else(|| acc.cape_url.clone())
                    };

                    let updated_row = AccountRow {
                        id: refreshed.id.clone(),
                        username: refreshed.username.clone(),
                        uuid: refreshed.uuid.clone(),
                        refresh_token: refreshed.refresh_token.clone(),
                        access_token: Some(refreshed.access_token.clone()),
                        expires_at: Some(chrono::DateTime::from_timestamp(refreshed.expires_at, 0).unwrap_or_default()),
                        created_at: acc.created_at,
                        updated_at: chrono::Utc::now(),
                        skin_url: refreshed.skin_url.or_else(|| acc.skin_url.clone()),
                        skin_variant: refreshed.skin_variant.or_else(|| acc.skin_variant.clone()),
                        cape_url: effective_cape,
                    };
                    let _ = crate::db::schema::accounts::upsert(&db, &updated_row).await;
                    account = Some(updated_row);
                }
            }
        }
    }

    if let Some(ref mut acc) = account {
        let is_custom_cape = acc.cape_url.as_deref().map(|c| {
            let trimmed = c.trim();
            !trimmed.is_empty()
                && !trimmed.starts_with("https://textures.minecraft.net/")
                && !trimmed.starts_with("http://textures.minecraft.net/")
        }).unwrap_or(false);

        if (acc.skin_url.is_none() || (acc.cape_url.is_none() && !is_custom_cape)) && !acc.uuid.is_empty() {
            if let Some((skin_url, skin_variant, cape_url)) = fetch_mojang_textures(&acc.uuid).await {
                if acc.skin_url.is_none() {
                    acc.skin_url = Some(skin_url);
                }
                if acc.skin_variant.is_none() && skin_variant.is_some() {
                    acc.skin_variant = skin_variant;
                }
                if !is_custom_cape && acc.cape_url.is_none() && cape_url.is_some() {
                    acc.cape_url = cape_url;
                }
            }
        }
        if acc.skin_url.is_none() || acc.skin_url.as_deref().unwrap_or("").trim().is_empty() {
            acc.skin_url = Some(format!("https://minotar.net/skin/{}", acc.username));
        }
        acc.updated_at = chrono::Utc::now();
        let _ = crate::db::schema::accounts::upsert(&db, acc).await;
    }

    let active_profile_id = settings
        .get("activeProfileId")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());

    Ok(AppInitState {
        dev_mode,
        account,
        profiles,
        active_profile_id,
        stress_test,
    })
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SystemSpecs {
    pub os_distro: String,
    pub kernel_version: String,
    pub arch: String,
    pub total_ram_mb: u64,
    pub launcher_version: String,
    pub gpu_vendor: String,
    pub gpu_renderer: String,
    pub gpu_supports_zink: bool,
}

#[tauri::command]
pub fn get_system_specs() -> SystemSpecs {
    let raw_name = sysinfo::System::name().unwrap_or_else(|| {
        if cfg!(target_os = "windows") {
            "Windows".to_string()
        } else if cfg!(target_os = "macos") {
            "macOS".to_string()
        } else {
            "Linux Generic".to_string()
        }
    });

    let os_version = sysinfo::System::os_version().unwrap_or_default();
    let os_distro = if !os_version.is_empty() && !raw_name.contains(&os_version) {
        format!("{} {}", raw_name, os_version)
    } else {
        raw_name
    };

    let kernel_version = sysinfo::System::kernel_version().unwrap_or_else(|| {
        std::env::consts::OS.to_string()
    });

    let total_ram_mb = crate::core::optimizer::get_total_memory_mb() as u64;

    let gpu = crate::core::optimizer::detect_gpu();

    SystemSpecs {
        os_distro,
        kernel_version,
        arch: std::env::consts::ARCH.to_string(),
        total_ram_mb,
        launcher_version: env!("CARGO_PKG_VERSION").to_string(),
        gpu_vendor: gpu.vendor,
        gpu_renderer: gpu.renderer,
        gpu_supports_zink: gpu.supports_zink,
    }
}

static LAST_OVERLAY_TOGGLE: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);

pub fn trigger_overlay_toggle(app: &tauri::AppHandle) {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64;
    let mut prev = LAST_OVERLAY_TOGGLE.load(std::sync::atomic::Ordering::SeqCst);
    loop {
        if now.saturating_sub(prev) < 500 {
            return;
        }
        match LAST_OVERLAY_TOGGLE.compare_exchange_weak(
            prev,
            now,
            std::sync::atomic::Ordering::SeqCst,
            std::sync::atomic::Ordering::SeqCst,
        ) {
            Ok(_) => break,
            Err(actual) => prev = actual,
        }
    }

    use tauri::Emitter;
    let _ = app.emit("luxmc-toggle-overlay", ());
}

#[tauri::command]
pub fn client_overlay_close(app: tauri::AppHandle) -> Result<(), String> {
    use tauri::Manager;
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.set_always_on_top(false);
    }
    Ok(())
}

