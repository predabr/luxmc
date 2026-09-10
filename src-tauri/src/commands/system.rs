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

    let accounts = crate::db::schema::accounts::list(&db).await?;
    let account = accounts.into_iter().next();

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
}

#[tauri::command]
pub fn get_system_specs() -> SystemSpecs {
    let mut os_distro = "Linux Generic".to_string();
    if let Ok(content) = std::fs::read_to_string("/etc/os-release") {
        for line in content.lines() {
            if line.starts_with("PRETTY_NAME=") {
                os_distro = line
                    .trim_start_matches("PRETTY_NAME=")
                    .trim_matches('"')
                    .to_string();
                break;
            } else if line.starts_with("NAME=") && os_distro == "Linux Generic" {
                os_distro = line
                    .trim_start_matches("NAME=")
                    .trim_matches('"')
                    .to_string();
            }
        }
    }

    let kernel_version = std::process::Command::new("uname")
        .arg("-r")
        .output()
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
        .unwrap_or_else(|_| "Linux Kernel".to_string());

    let mut total_ram_mb = 8192;
    if let Ok(mem_info) = std::fs::read_to_string("/proc/meminfo") {
        for line in mem_info.lines() {
            if line.starts_with("MemTotal:") {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 2 {
                    if let Ok(kb) = parts[1].parse::<u64>() {
                        total_ram_mb = kb / 1024;
                    }
                }
                break;
            }
        }
    }

    SystemSpecs {
        os_distro,
        kernel_version,
        arch: std::env::consts::ARCH.to_string(),
        total_ram_mb,
        launcher_version: "1.0.0-BETA".to_string(),
    }
}
