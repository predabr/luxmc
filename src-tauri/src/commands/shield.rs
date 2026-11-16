use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use crate::db::models::ProfileRow;
use crate::error::{AppError, AppResult};

mod scanner;
pub use scanner::scan_mods_directory;
pub(crate) use scanner::enforce_scan;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ShieldThreat {
    pub file_name: String,
    pub file_path: String,
    pub severity: String,
    #[serde(default)]
    pub code: String,
    pub threat_type: String,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ShieldScanResult {
    pub total_scanned: usize,
    pub is_clean: bool,
    pub threats: Vec<ShieldThreat>,
    pub scan_time_ms: u64,
    #[serde(default)]
    pub purged: bool,
    #[serde(default)]
    pub instance_name: Option<String>,
}

pub fn has_critical_threat(result: &ShieldScanResult) -> bool {
    result.threats.iter().any(|threat| threat.severity == "critical")
}

pub(crate) async fn purge_for_launch(profile: &ProfileRow, result: &ShieldScanResult) -> AppResult<bool> {
    if !has_critical_threat(result) { return Ok(false); }
    if crate::core::launcher::get_active_game_pid() != 0 { return Ok(false); }
    let db = crate::db::shared_db().await?;
    let game_dir = PathBuf::from(&profile.game_dir);
    let managed = directories::ProjectDirs::from("io", "github", "Luxmc")
        .is_some_and(|dirs| game_dir.starts_with(dirs.data_dir().join("instances")));
    if managed {
        tokio::fs::remove_dir_all(&game_dir).await?;
    } else if game_dir.join("mods").exists() {
        tokio::fs::remove_dir_all(game_dir.join("mods")).await?;
    }
    crate::db::schema::profiles::delete(&db, &profile.id).await?;
    Ok(true)
}

pub(crate) fn mark_result_purged(result: &mut ShieldScanResult, profile: &ProfileRow) {
    result.purged = true;
    result.instance_name = Some(profile.name.clone());
}

pub async fn instance_shield_scan_core(app: Option<tauri::AppHandle>, profileId: String) -> AppResult<ShieldScanResult> {
    let db = crate::db::shared_db().await?;
    let row = sqlx::query_as::<_, ProfileRow>("SELECT * FROM profiles WHERE id = ?")
        .bind(&profileId)
        .fetch_optional(db.pool())
        .await?
        .ok_or_else(|| AppError::NotFound(format!("profile {profileId} not found")))?;

    let mods_dir = PathBuf::from(&row.game_dir).join("mods");
    let mut result = tokio::task::spawn_blocking(move || scan_mods_directory(&mods_dir))
        .await
        .map_err(|error| AppError::Internal(error.to_string()))?;

    if purge_for_launch(&row, &result).await? {
        mark_result_purged(&mut result, &row);
        if let Some(app) = app {
            use tauri::Emitter;
            app.emit("launcher-shield-warning", &result).ok();
        }
    }
    Ok(result)
}

#[tauri::command]
#[allow(non_snake_case)]
pub async fn instance_shield_scan(
    app: tauri::AppHandle,
    profileId: String,
) -> AppResult<ShieldScanResult> {
    instance_shield_scan_core(Some(app), profileId).await
}
