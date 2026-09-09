#![allow(non_snake_case)]

use tauri::State;

use crate::db;
use crate::error::AppResult;
use crate::state::AppState;

#[tauri::command]
pub async fn launch_log_open(
    _state: State<'_, AppState>,
    profileId: Option<String>,
    versionId: String,
) -> AppResult<i64> {
    let conn = db::shared_db().await?;
    let id = db::schema::launch_logs::create(&conn, profileId.as_deref(), &versionId).await?;
    Ok(id)
}

#[tauri::command]
pub async fn launch_log_append(
    _state: State<'_, AppState>,
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
    _state: State<'_, AppState>,
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
