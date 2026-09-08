use chrono::Utc;
use serde::{Deserialize, Serialize};

use crate::db::Db;
use crate::error::AppResult;

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct ModRow {
	pub profile_id: String,
	pub project_id: String,
	pub version_id: String,
	pub file_name: String,
	pub sha1: String,
	pub source: String,
	pub installed_at: String,
}

pub async fn list_by_profile(db: &Db, profile_id: &str) -> AppResult<Vec<ModRow>> {
	let rows = sqlx::query_as::<_, ModRow>(
		"SELECT * FROM mods WHERE profile_id = ? ORDER BY installed_at DESC",
	)
	.bind(profile_id)
	.fetch_all(db.pool())
	.await?;
	Ok(rows)
}

pub async fn upsert(db: &Db, m: &ModRow) -> AppResult<()> {
	let now = Utc::now().to_rfc3339();
	sqlx::query(
		"INSERT INTO mods (profile_id, project_id, version_id, file_name, sha1, source, installed_at)
		 VALUES (?, ?, ?, ?, ?, ?, ?)
		 ON CONFLICT(profile_id, project_id) DO UPDATE SET
			version_id = excluded.version_id,
			file_name = excluded.file_name,
			sha1 = excluded.sha1,
			source = excluded.source,
			installed_at = excluded.installed_at",
	)
	.bind(&m.profile_id)
	.bind(&m.project_id)
	.bind(&m.version_id)
	.bind(&m.file_name)
	.bind(&m.sha1)
	.bind(&m.source)
	.bind(now)
	.execute(db.pool())
	.await?;
	Ok(())
}

pub async fn delete(db: &Db, profile_id: &str, project_id: &str) -> AppResult<()> {
	sqlx::query("DELETE FROM mods WHERE profile_id = ? AND project_id = ?")
		.bind(profile_id)
		.bind(project_id)
		.execute(db.pool())
		.await?;
	Ok(())
}
