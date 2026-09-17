#![allow(non_snake_case)]

use crate::db;
use crate::error::AppResult;

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
    db::schema::launch_logs::add_line(&conn, logId, &stream, &level, &message, &now).await?;
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
