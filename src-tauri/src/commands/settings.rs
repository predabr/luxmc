use serde_json::Value;
use sqlx::Row;

use crate::db;
use crate::error::{AppError, AppResult};

const APP_SETTINGS_UPSERT: &str = "INSERT INTO app_settings (key, value) VALUES ('app', ?) ON CONFLICT(key) DO UPDATE SET value = CASE WHEN json_valid(app_settings.value) THEN json_patch(app_settings.value, excluded.value) ELSE excluded.value END";

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
    if !value.is_object() { return Err(AppError::InvalidInput("As configurações devem ser um objeto".into())); }
    let conn = db::shared_db().await?;
    let serialized = serde_json::to_string(&value)?;
    sqlx::query(APP_SETTINGS_UPSERT)
    .bind(serialized)
    .execute(conn.pool())
    .await
    .map_err(AppError::from)?;
    if let Some(enabled) = value.get("closeToTray").and_then(Value::as_bool) { crate::set_close_to_tray(enabled); }
    Ok(())
}

#[tauri::command]
pub async fn settings_set_concurrent_downloads(value: u32) -> AppResult<()> {
    let limit = value.clamp(1, 64);
    crate::core::downloader::set_max_concurrent_downloads(limit as usize);
    let conn = db::shared_db().await?;
    sqlx::query(
        "INSERT INTO app_settings (key, value) VALUES ('concurrent_downloads', ?) \
		 ON CONFLICT(key) DO UPDATE SET value = excluded.value",
    )
    .bind(limit.to_string())
    .execute(conn.pool())
    .await
    .map_err(AppError::from)?;
    Ok(())
}

pub async fn apply_stored_download_limit(conn: &crate::db::Db) -> AppResult<()> {
    use sqlx::Row;
    if let Ok(Some(row)) = sqlx::query("SELECT value FROM app_settings WHERE key = 'concurrent_downloads'")
        .fetch_optional(conn.pool())
        .await
    {
        let raw: String = row.try_get("value").unwrap_or_default();
        if let Ok(limit) = raw.parse::<usize>() {
            crate::core::downloader::set_max_concurrent_downloads(limit);
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    #[tokio::test]
    async fn background_preferences_preserve_active_account_and_native_settings() {
        let pool = sqlx::SqlitePool::connect("sqlite::memory:").await.unwrap();
        sqlx::query("CREATE TABLE app_settings (key TEXT PRIMARY KEY, value TEXT NOT NULL)").execute(&pool).await.unwrap();
        sqlx::query(super::APP_SETTINGS_UPSERT).bind(r#"{"activeAccountId":"account-123","clientId":"oauth-client","closeToTray":true}"#).execute(&pool).await.unwrap();
        sqlx::query(super::APP_SETTINGS_UPSERT).bind(r#"{"closeToTray":false,"language":"en"}"#).execute(&pool).await.unwrap();
        let stored: String = sqlx::query_scalar("SELECT value FROM app_settings WHERE key = 'app'").fetch_one(&pool).await.unwrap();
        let value: serde_json::Value = serde_json::from_str(&stored).unwrap();
        assert_eq!(value["activeAccountId"], "account-123");
        assert_eq!(value["clientId"], "oauth-client");
        assert_eq!(value["closeToTray"], false);
        assert_eq!(value["language"], "en");
    }
}
