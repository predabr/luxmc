use serde_json::Value;
use sqlx::Row;

use crate::db;
use crate::error::{AppError, AppResult};

#[tauri::command]
pub async fn settings_get() -> AppResult<Value> {
    let conn = db::shared_db().await?;
    let row = sqlx::query("SELECT value FROM app_settings WHERE key = 'app'")
        .fetch_optional(conn.pool())
        .await?;
    match row {
        Some(r) => {
            let raw: String = r.try_get("value")?;
            let parsed: Value = serde_json::from_str(&raw)?;
            Ok(parsed)
        }
        None => Ok(Value::Null),
    }
}

#[tauri::command]
pub async fn settings_set(value: Value) -> AppResult<()> {
    let conn = db::shared_db().await?;
    let serialized = serde_json::to_string(&value)?;
    sqlx::query(
        "INSERT INTO app_settings (key, value) VALUES ('app', ?) \
		 ON CONFLICT(key) DO UPDATE SET value = excluded.value",
    )
    .bind(serialized)
    .execute(conn.pool())
    .await
    .map_err(AppError::from)?;
    Ok(())
}
