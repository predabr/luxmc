use serde::{Deserialize, Serialize};
use tauri::State;

use crate::db;
use crate::error::AppResult;
use crate::state::AppState;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LaunchLogSummary {
	pub id: i64,
	pub profile_id: Option<String>,
	pub version_id: String,
	pub started_at: String,
	pub ended_at: Option<String>,
	pub exit_code: Option<i64>,
	pub summary: Option<String>,
	pub error_classification: Option<String>,
	line_count: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LaunchLogLineDto {
	pub id: i64,
	pub log_id: i64,
	pub stream: String,
	pub level: String,
	pub message: String,
	pub ts: String,
}

#[tauri::command]
pub async fn launch_logs_list(
	_state: State<'_, AppState>,
	limit: Option<i64>,
) -> AppResult<Vec<LaunchLogSummary>> {
	let conn = db::shared_db().await?;
	let limit = limit.unwrap_or(100);
	let rows = db::schema::launch_logs::list(&conn, limit).await?;
	let mut out = Vec::new();
	for r in rows {
		let lines = db::schema::launch_logs::lines(&conn, r.id).await?;
		out.push(LaunchLogSummary {
			id: r.id,
			profile_id: r.profile_id,
			version_id: r.version_id,
			started_at: r.started_at.to_rfc3339(),
			ended_at: r.ended_at.map(|d| d.to_rfc3339()),
			exit_code: r.exit_code,
			summary: r.summary,
			error_classification: r.error_classification,
			line_count: lines.len() as i64,
		});
	}
	Ok(out)
}

#[tauri::command]
pub async fn launch_logs_get(
	_state: State<'_, AppState>,
	id: i64,
) -> AppResult<Vec<LaunchLogLineDto>> {
	let conn = db::shared_db().await?;
	let rows = db::schema::launch_logs::lines(&conn, id).await?;
	Ok(rows
		.into_iter()
		.map(|r| LaunchLogLineDto {
			id: r.id,
			log_id: r.log_id,
			stream: r.stream,
			level: r.level,
			message: r.message,
			ts: r.ts,
		})
		.collect())
}

#[tauri::command]
pub async fn launch_logs_search(
	_state: State<'_, AppState>,
	query: String,
	min_level: String,
	limit: Option<i64>,
) -> AppResult<Vec<LaunchLogLineDto>> {
	let conn = db::shared_db().await?;
	let limit = limit.unwrap_or(200);
	let rows = db::schema::launch_logs::search(&conn, &query, &min_level, limit).await?;
	Ok(rows
		.into_iter()
		.map(|r| LaunchLogLineDto {
			id: r.id,
			log_id: r.log_id,
			stream: r.stream,
			level: r.level,
			message: r.message,
			ts: r.ts,
		})
		.collect())
}

#[tauri::command]
pub async fn launch_logs_clear(_state: State<'_, AppState>) -> AppResult<()> {
	let conn = db::shared_db().await?;
	sqlx::query("DELETE FROM launch_log_lines")
		.execute(conn.pool())
		.await?;
	sqlx::query("DELETE FROM launch_logs")
		.execute(conn.pool())
		.await?;
	Ok(())
}
