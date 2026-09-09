use chrono::Utc;
use serde::{Deserialize, Serialize};

use crate::db::Db;
use crate::error::AppResult;

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct ServerRow {
    pub id: String,
    pub name: String,
    pub host: String,
    pub port: i64,
    pub favorite: bool,
    pub created_at: String,
}

pub async fn list(db: &Db) -> AppResult<Vec<ServerRow>> {
    let rows = sqlx::query_as::<_, ServerRow>(
        "SELECT * FROM servers ORDER BY favorite DESC, created_at DESC",
    )
    .fetch_all(db.pool())
    .await?;
    Ok(rows)
}

pub async fn list_favorites(db: &Db) -> AppResult<Vec<ServerRow>> {
    let rows = sqlx::query_as::<_, ServerRow>(
        "SELECT * FROM servers WHERE favorite = 1 ORDER BY created_at DESC",
    )
    .fetch_all(db.pool())
    .await?;
    Ok(rows)
}

pub async fn insert(db: &Db, s: &ServerRow) -> AppResult<()> {
    let now = Utc::now().to_rfc3339();
    sqlx::query(
        "INSERT INTO servers (id, name, host, port, favorite, created_at)
		 VALUES (?, ?, ?, ?, ?, ?)
		 ON CONFLICT(id) DO UPDATE SET
			name = excluded.name,
			host = excluded.host,
			port = excluded.port,
			favorite = excluded.favorite",
    )
    .bind(&s.id)
    .bind(&s.name)
    .bind(&s.host)
    .bind(s.port)
    .bind(s.favorite)
    .bind(now)
    .execute(db.pool())
    .await?;
    Ok(())
}

pub async fn delete(db: &Db, id: &str) -> AppResult<()> {
    sqlx::query("DELETE FROM servers WHERE id = ?")
        .bind(id)
        .execute(db.pool())
        .await?;
    Ok(())
}

pub async fn set_favorite(db: &Db, id: &str, favorite: bool) -> AppResult<()> {
    sqlx::query("UPDATE servers SET favorite = ? WHERE id = ?")
        .bind(favorite)
        .bind(id)
        .execute(db.pool())
        .await?;
    Ok(())
}
