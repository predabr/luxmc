use chrono::Utc;
use serde::{Deserialize, Serialize};
use tauri::State;
use uuid::Uuid;

use crate::db::models::ProfileRow;
use crate::error::AppResult;
use crate::state::AppState;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProfileCreate {
    pub name: String,
    pub icon: Option<String>,
    pub mc_version: String,
    pub loader: String,
    pub loader_version: Option<String>,
    pub java_path: Option<String>,
    pub jvm_args: Option<String>,
    pub resolution_w: Option<i64>,
    pub resolution_h: Option<i64>,
    pub fullscreen: Option<bool>,
    pub game_dir: Option<String>,
    #[serde(default)]
    pub favorite: Option<bool>,
    #[serde(default)]
    pub notes: Option<String>,
    #[serde(default)]
    pub ram_mb: Option<i64>,
    #[serde(default)]
    pub instance_group: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProfileUpdate {
    pub id: String,
    pub name: Option<String>,
    pub icon: Option<String>,
    pub mc_version: Option<String>,
    pub loader: Option<String>,
    pub loader_version: Option<Option<String>>,
    pub java_path: Option<Option<String>>,
    pub jvm_args: Option<Option<String>>,
    pub resolution_w: Option<Option<i64>>,
    pub resolution_h: Option<Option<i64>>,
    pub fullscreen: Option<bool>,
    pub game_dir: Option<String>,
    pub favorite: Option<bool>,
    pub notes: Option<Option<String>>,
    pub last_played: Option<String>,
    pub launch_count: Option<i64>,
    pub mod_count: Option<i64>,
    pub disk_usage: Option<i64>,
    pub ram_mb: Option<Option<i64>>,
    pub instance_group: Option<Option<String>>,
}

#[tauri::command]
pub async fn profiles_list(_state: State<'_, AppState>) -> AppResult<Vec<ProfileRow>> {
    let db = crate::db::shared_db().await?;
    crate::db::schema::profiles::list(&db).await
}

#[tauri::command]
pub async fn profiles_get(_state: State<'_, AppState>, id: String) -> AppResult<ProfileRow> {
    let db = crate::db::shared_db().await?;
    let row = sqlx::query_as::<_, ProfileRow>("SELECT * FROM profiles WHERE id = ?")
        .bind(&id)
        .fetch_optional(db.pool())
        .await?
        .ok_or_else(|| crate::error::AppError::NotFound(format!("profile {id} not found")))?;
    Ok(row)
}

#[tauri::command]
pub async fn profiles_create(
    _state: State<'_, AppState>,
    input: ProfileCreate,
) -> AppResult<ProfileRow> {
    let db = crate::db::shared_db().await?;
    let now = Utc::now();
    let profile_name = input.name.clone();
    let game_dir = input.game_dir.unwrap_or_else(|| {
        let base = directories::ProjectDirs::from("io", "github", "Luxmc")
            .map(|d| d.data_dir().to_string_lossy().to_string())
            .unwrap_or_else(|| {
                let home = std::env::var("HOME").unwrap_or_else(|_| "/tmp".into());
                format!("{}/.local/share/luxmc", home)
            });
        format!("{}/instances/{}", base, profile_name)
    });
    let row = ProfileRow {
        id: Uuid::new_v4().to_string(),
        name: input.name,
        icon: input.icon.unwrap_or_else(|| "grass_block".into()),
        mc_version: input.mc_version,
        loader: input.loader,
        loader_version: input.loader_version,
        java_path: input.java_path,
        jvm_args: input.jvm_args,
        resolution_w: input.resolution_w,
        resolution_h: input.resolution_h,
        fullscreen: input.fullscreen.unwrap_or(false),
        game_dir,
        created_at: now,
        updated_at: now,
        favorite: input.favorite.unwrap_or(false),
        notes: input.notes,
        last_played: None,
        launch_count: 0,
        mod_count: 0,
        disk_usage: 0,
        ram_mb: input.ram_mb,
        instance_group: input.instance_group,
    };
    crate::db::schema::profiles::upsert(&db, &row).await?;
    Ok(row)
}

#[tauri::command]
pub async fn profiles_update(
    _state: State<'_, AppState>,
    input: ProfileUpdate,
) -> AppResult<ProfileRow> {
    let db = crate::db::shared_db().await?;
    let existing = sqlx::query_as::<_, ProfileRow>("SELECT * FROM profiles WHERE id = ?")
        .bind(&input.id)
        .fetch_optional(db.pool())
        .await?
        .ok_or_else(|| {
            crate::error::AppError::NotFound(format!("profile {} not found", input.id))
        })?;

    let now = Utc::now();
    let row = ProfileRow {
        id: existing.id,
        name: input.name.unwrap_or(existing.name),
        icon: input.icon.unwrap_or(existing.icon),
        mc_version: input.mc_version.unwrap_or(existing.mc_version),
        loader: input.loader.unwrap_or(existing.loader),
        loader_version: input.loader_version.unwrap_or(existing.loader_version),
        java_path: input.java_path.unwrap_or(existing.java_path),
        jvm_args: input.jvm_args.unwrap_or(existing.jvm_args),
        resolution_w: input.resolution_w.unwrap_or(existing.resolution_w),
        resolution_h: input.resolution_h.unwrap_or(existing.resolution_h),
        fullscreen: input.fullscreen.unwrap_or(existing.fullscreen),
        game_dir: input.game_dir.unwrap_or(existing.game_dir),
        created_at: existing.created_at,
        updated_at: now,
        favorite: input.favorite.unwrap_or(existing.favorite),
        notes: input.notes.unwrap_or(existing.notes),
        last_played: input
            .last_played
            .and_then(|s| s.parse().ok())
            .map(|dt| dt)
            .or(existing.last_played),
        launch_count: input.launch_count.unwrap_or(existing.launch_count),
        mod_count: input.mod_count.unwrap_or(existing.mod_count),
        disk_usage: input.disk_usage.unwrap_or(existing.disk_usage),
        ram_mb: input.ram_mb.unwrap_or(existing.ram_mb),
        instance_group: input.instance_group.unwrap_or(existing.instance_group),
    };
    crate::db::schema::profiles::upsert(&db, &row).await?;
    Ok(row)
}

#[tauri::command]
pub async fn profiles_delete(_state: State<'_, AppState>, id: String) -> AppResult<()> {
    let db = crate::db::shared_db().await?;
    crate::db::schema::profiles::delete(&db, &id).await
}
