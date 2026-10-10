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
}

#[tauri::command]
#[allow(non_snake_case)]
pub async fn instance_shield_scan(
    profileId: String,
) -> AppResult<ShieldScanResult> {
    let db = crate::db::shared_db().await?;
    let row = sqlx::query_as::<_, ProfileRow>("SELECT * FROM profiles WHERE id = ?")
        .bind(&profileId)
        .fetch_optional(db.pool())
        .await?
        .ok_or_else(|| AppError::NotFound(format!("profile {profileId} not found")))?;

    let mods_dir = PathBuf::from(&row.game_dir).join("mods");
    tokio::task::spawn_blocking(move || scan_mods_directory(&mods_dir))
        .await
        .map_err(|error| AppError::Internal(error.to_string()))
}
