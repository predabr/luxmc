use std::path::PathBuf;
use serde::{Deserialize, Serialize};
use tauri::State;

use crate::error::AppResult;
use crate::state::AppState;
use crate::db::models::ProfileRow;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DeathEventInfo {
    pub has_death: bool,
    pub death_message: String,
    pub timestamp: String,
    pub x: Option<f64>,
    pub y: Option<f64>,
    pub z: Option<f64>,
    pub dimension: String,
    pub world_name: Option<String>,
}

#[tauri::command]
#[allow(non_snake_case)]
pub async fn death_tracker_get_last_death(
    _state: State<'_, AppState>,
    profileId: String,
) -> AppResult<DeathEventInfo> {
    let db = crate::db::shared_db().await?;
    let row = sqlx::query_as::<_, ProfileRow>("SELECT * FROM profiles WHERE id = ?")
        .bind(&profileId)
        .fetch_optional(db.pool())
        .await?
        .ok_or_else(|| crate::error::AppError::NotFound(format!("profile {profileId} not found")))?;

    let game_dir = PathBuf::from(&row.game_dir);
    let log_path = game_dir.join("logs").join("latest.log");

    let death_keywords = [
        "fell from a high place",
        "was blown up",
        "was slain by",
        "drowned",
        "experienced kinetic energy",
        "burned to death",
        "went up in flames",
        "tried to swim in lava",
        "suffocated in a wall",
        "starved to death",
        "was killed by",
        "hit the ground too hard",
        "fell out of the world",
        "withered away",
        "died",
    ];

    if log_path.is_file() {
        if let Ok(content) = std::fs::read_to_string(&log_path) {
            let mut last_death_line = None;
            let mut last_coords = None;

            for line in content.lines().rev() {
                if last_death_line.is_none() {
                    for kw in &death_keywords {
                        if line.contains(kw) {
                            last_death_line = Some(line.to_string());
                            break;
                        }
                    }
                }

                if line.contains("Block:") || line.contains("Position:") {
                    last_coords = parse_coords_from_log_line(line);
                }

                if last_death_line.is_some() && last_coords.is_some() {
                    break;
                }
            }

            if let Some(death_msg) = last_death_line {
                let clean_msg = death_msg.split(']').last().unwrap_or(&death_msg).trim();
                let (x, y, z) = last_coords.unwrap_or((0.0, 64.0, 0.0));

                return Ok(DeathEventInfo {
                    has_death: true,
                    death_message: clean_msg.to_string(),
                    timestamp: chrono::Utc::now().to_rfc3339(),
                    x: Some(x),
                    y: Some(y),
                    z: Some(z),
                    dimension: "Overworld".into(),
                    world_name: None,
                });
            }
        }
    }

    Ok(DeathEventInfo {
        has_death: false,
        death_message: "Nenhuma morte recente detectada nos registros.".into(),
        timestamp: chrono::Utc::now().to_rfc3339(),
        x: None,
        y: None,
        z: None,
        dimension: "Overworld".into(),
        world_name: None,
    })
}

fn parse_coords_from_log_line(line: &str) -> Option<(f64, f64, f64)> {
    let parts: Vec<&str> = line.split_whitespace().collect();
    for (i, p) in parts.iter().enumerate() {
        if *p == "Block:" || *p == "Position:" {
            if let (Some(xs), Some(ys), Some(zs)) = (parts.get(i + 1), parts.get(i + 2), parts.get(i + 3)) {
                let x = xs.trim_matches(|c: char| !c.is_numeric() && c != '-' && c != '.').parse::<f64>().ok()?;
                let y = ys.trim_matches(|c: char| !c.is_numeric() && c != '-' && c != '.').parse::<f64>().ok()?;
                let z = zs.trim_matches(|c: char| !c.is_numeric() && c != '-' && c != '.').parse::<f64>().ok()?;
                return Some((x, y, z));
            }
        }
    }
    None
}
