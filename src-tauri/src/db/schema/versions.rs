use chrono::Utc;
use serde::{Deserialize, Serialize};

use crate::db::Db;
use crate::error::AppResult;

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct VersionRow {
    pub id: String,
    #[sqlx(rename = "type")]
    pub version_type: String,
    pub url: String,
    pub time: String,
    pub release_time: String,
    pub fetched_at: String,
}

pub async fn list(db: &Db) -> AppResult<Vec<VersionRow>> {
    let rows = sqlx::query_as::<_, VersionRow>(
		"SELECT id, type, url, time, release_time, fetched_at FROM mc_versions ORDER BY release_time DESC",
	)
	.fetch_all(db.pool())
	.await?;
    Ok(rows)
}

pub async fn get(db: &Db, id: &str) -> AppResult<Option<VersionRow>> {
    let row = sqlx::query_as::<_, VersionRow>(
        "SELECT id, type, url, time, release_time, fetched_at FROM mc_versions WHERE id = ?",
    )
    .bind(id)
    .fetch_optional(db.pool())
    .await?;
    Ok(row)
}

pub async fn upsert(db: &Db, v: &VersionRow) -> AppResult<()> {
    let now = Utc::now().to_rfc3339();
    sqlx::query(
        "INSERT INTO mc_versions (id, type, url, time, release_time, fetched_at)
		 VALUES (?, ?, ?, ?, ?, ?)
		 ON CONFLICT(id) DO UPDATE SET
			type = excluded.type,
			url = excluded.url,
			time = excluded.time,
			release_time = excluded.release_time,
			fetched_at = excluded.fetched_at",
    )
    .bind(&v.id)
    .bind(&v.version_type)
    .bind(&v.url)
    .bind(&v.time)
    .bind(&v.release_time)
    .bind(now)
    .execute(db.pool())
    .await?;
    Ok(())
}

pub async fn upsert_many(db: &Db, versions: &[VersionRow]) -> AppResult<()> {
    let mut tx = db.pool().begin().await?;
    let now = Utc::now().to_rfc3339();
    for v in versions {
        sqlx::query(
            "INSERT INTO mc_versions (id, type, url, time, release_time, fetched_at)
			 VALUES (?, ?, ?, ?, ?, ?)
			 ON CONFLICT(id) DO UPDATE SET
				type = excluded.type,
				url = excluded.url,
				time = excluded.time,
				release_time = excluded.release_time,
				fetched_at = excluded.fetched_at",
        )
        .bind(&v.id)
        .bind(&v.version_type)
        .bind(&v.url)
        .bind(&v.time)
        .bind(&v.release_time)
        .bind(&now)
        .execute(&mut *tx)
        .await?;
    }
    tx.commit().await?;
    Ok(())
}
