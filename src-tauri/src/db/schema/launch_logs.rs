use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::Row;

use crate::db::Db;
use crate::error::AppResult;

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct LaunchLogRow {
	pub id: i64,
	pub profile_id: Option<String>,
	pub version_id: String,
	pub started_at: DateTime<Utc>,
	pub ended_at: Option<DateTime<Utc>>,
	pub exit_code: Option<i64>,
	pub summary: Option<String>,
	pub error_classification: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct LaunchLogLine {
	pub id: i64,
	pub log_id: i64,
	pub stream: String,
	pub level: String,
	pub message: String,
	pub ts: String,
}

pub async fn create(
	db: &Db,
	profile_id: Option<&str>,
	version_id: &str,
) -> AppResult<i64> {
	let now = Utc::now();
	let row = sqlx::query(
		"INSERT INTO launch_logs (profile_id, version_id, started_at) VALUES (?, ?, ?) RETURNING id",
	)
	.bind(profile_id)
	.bind(version_id)
	.bind(now)
	.fetch_one(db.pool())
	.await?;
	Ok(row.try_get::<i64, _>("id")?)
}

pub async fn finish(
	db: &Db,
	id: i64,
	exit_code: Option<i64>,
	summary: Option<&str>,
	error_classification: Option<&str>,
) -> AppResult<()> {
	sqlx::query(
		"UPDATE launch_logs SET ended_at = ?, exit_code = ?, summary = ?, error_classification = ? WHERE id = ?",
	)
	.bind(Utc::now())
	.bind(exit_code)
	.bind(summary)
	.bind(error_classification)
	.bind(id)
	.execute(db.pool())
	.await?;
	Ok(())
}

pub async fn add_line(
	db: &Db,
	log_id: i64,
	stream: &str,
	level: &str,
	message: &str,
	ts: &str,
) -> AppResult<()> {
	sqlx::query(
		"INSERT INTO launch_log_lines (log_id, stream, level, message, ts) VALUES (?, ?, ?, ?, ?)",
	)
	.bind(log_id)
	.bind(stream)
	.bind(level)
	.bind(message)
	.bind(ts)
	.execute(db.pool())
	.await?;
	Ok(())
}

pub async fn list(db: &Db, limit: i64) -> AppResult<Vec<LaunchLogRow>> {
	let rows = sqlx::query_as::<_, LaunchLogRow>(
		"SELECT id, profile_id, version_id, started_at, ended_at, exit_code, summary, error_classification \
		 FROM launch_logs ORDER BY started_at DESC LIMIT ?",
	)
	.bind(limit)
	.fetch_all(db.pool())
	.await?;
	Ok(rows)
}

pub async fn lines(db: &Db, log_id: i64) -> AppResult<Vec<LaunchLogLine>> {
	let rows = sqlx::query_as::<_, LaunchLogLine>(
		"SELECT id, log_id, stream, level, message, ts FROM launch_log_lines \
		 WHERE log_id = ? ORDER BY id ASC",
	)
	.bind(log_id)
	.fetch_all(db.pool())
	.await?;
	Ok(rows)
}

pub async fn search(
	db: &Db,
	query: &str,
	min_level: &str,
	limit: i64,
) -> AppResult<Vec<LaunchLogLine>> {
	let levels: Vec<&str> = match min_level {
		"ERROR" => vec!["ERROR"],
		"WARN" => vec!["WARN", "ERROR"],
		"INFO" => vec!["INFO", "WARN", "ERROR"],
		_ => vec!["INFO", "WARN", "ERROR"],
	};
	let mut result = Vec::new();
	for level in levels {
		let rows = sqlx::query_as::<_, LaunchLogLine>(
			"SELECT id, log_id, stream, level, message, ts FROM launch_log_lines \
			 WHERE level = ? AND message LIKE ? ORDER BY id DESC LIMIT ?",
		)
		.bind(level)
		.bind(format!("%{}%", query))
		.bind(limit)
		.fetch_all(db.pool())
		.await?;
		result.extend(rows);
	}
	result.sort_by(|a, b| b.id.cmp(&a.id));
	result.truncate(limit as usize);
	Ok(result)
}
