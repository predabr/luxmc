#![allow(non_snake_case)]

use crate::db;
use crate::error::AppResult;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct LaunchLogInput {
    pub stream: String,
    pub level: String,
    pub message: String,
    pub ts: String,
}

#[tauri::command]
pub async fn launch_log_append_batch(logId: i64, entries: Vec<LaunchLogInput>) -> AppResult<()> {
    let conn = db::shared_db().await?;
    append_batch(conn.pool(), logId, entries).await
}

async fn append_batch(pool: &sqlx::SqlitePool, log_id: i64, entries: Vec<LaunchLogInput>) -> AppResult<()> {
    if entries.len() > 128 { return Err(crate::error::AppError::InvalidInput("O lote de logs excede 128 registros.".into())); }
    let mut transaction = pool.begin().await?;
    for entry in entries {
        sqlx::query("INSERT INTO launch_log_lines (log_id, stream, level, message, ts) VALUES (?, ?, ?, ?, ?)")
            .bind(log_id).bind(entry.stream).bind(entry.level)
            .bind(crate::core::process::sanitize_diagnostic(&entry.message)).bind(entry.ts)
            .execute(&mut *transaction).await?;
    }
    transaction.commit().await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn diagnostic_batches_preserve_order_redact_credentials_and_rollback_on_failure() {
        let pool = sqlx::sqlite::SqlitePoolOptions::new().max_connections(1).connect("sqlite::memory:").await.unwrap();
        sqlx::query("CREATE TABLE launch_log_lines (id INTEGER PRIMARY KEY, log_id INTEGER, stream TEXT, level TEXT, message TEXT, ts TEXT CHECK (ts <> 'invalid'))").execute(&pool).await.unwrap();
        let line = |message: &str, ts: &str| LaunchLogInput { stream: "stdout".into(), level: "INFO".into(), message: message.into(), ts: ts.into() };
        append_batch(&pool, 7, vec![line("first", "one"), line("--accessToken private-token", "two")]).await.unwrap();
        let rows: Vec<(i64, String, String)> = sqlx::query_as("SELECT log_id, message, ts FROM launch_log_lines ORDER BY id").fetch_all(&pool).await.unwrap();
        assert_eq!(rows, vec![(7, "first".into(), "one".into()), (7, "--[redacted]".into(), "two".into())]);
        assert!(append_batch(&pool, 8, vec![line("partial", "three"), line("failed", "invalid")]).await.is_err());
        let count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM launch_log_lines").fetch_one(&pool).await.unwrap();
        assert_eq!(count.0, 2);
        assert!(append_batch(&pool, 8, vec![line("overflow", "four"); 129]).await.is_err());
        pool.close().await;
    }
}

#[tauri::command]
pub async fn launch_log_open(
    profileId: Option<String>,
    versionId: String,
) -> AppResult<i64> {
    let conn = db::shared_db().await?;
    let id = db::schema::launch_logs::create(&conn, profileId.as_deref(), &versionId).await?;
    Ok(id)
}

#[tauri::command]
pub async fn launch_log_append(
    logId: i64,
    stream: String,
    level: String,
    message: String,
) -> AppResult<()> {
    let conn = db::shared_db().await?;
    let now = chrono::Utc::now().to_rfc3339();
    db::schema::launch_logs::add_line(&conn, logId, &stream, &level, &crate::core::process::sanitize_diagnostic(&message), &now).await?;
    Ok(())
}

#[tauri::command]
#[allow(non_snake_case)]
pub async fn launch_log_close(
    logId: i64,
    exitCode: Option<i64>,
    summary: Option<String>,
    errorClassification: Option<String>,
) -> AppResult<()> {
    let conn = db::shared_db().await?;
    db::schema::launch_logs::finish(
        &conn,
        logId,
        exitCode,
        summary.as_deref(),
        errorClassification.as_deref(),
    )
    .await?;
    Ok(())
}
