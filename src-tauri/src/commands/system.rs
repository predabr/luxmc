use serde::Serialize;

use crate::db::models::{AccountRow, ProfileRow};

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
        name: env!("CARGO_PKG_NAME"),
        version: env!("CARGO_PKG_VERSION"),
        identifier: "io.github.luxmc.Luxmc",
    }
}

#[tauri::command]
pub async fn app_init() -> Result<AppInitState, crate::error::AppError> {
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

    let account = if let Some(id) = active_account_id {
        if let Ok(Some(acc)) = crate::db::schema::accounts::get_by_id(&db, id).await {
            Some(acc)
        } else {
            crate::db::schema::accounts::list(&db).await?.into_iter().next()
        }
    } else {
        crate::db::schema::accounts::list(&db).await?.into_iter().next()
    };

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
    let mut sys = sysinfo::System::new_all();
    sys.refresh_memory();

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

    let total_ram_mb = sys.total_memory() / 1024 / 1024;
    let total_ram_mb = if total_ram_mb > 0 { total_ram_mb } else { 8192 };

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
