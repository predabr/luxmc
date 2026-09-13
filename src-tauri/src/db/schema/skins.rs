use chrono::Utc;
use serde::{Deserialize, Serialize};

use crate::db::Db;
use crate::error::AppResult;

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct SavedSkinRow {
    pub id: String,
    pub name: String,
    pub skin_url: String,
    pub avatar_url: String,
    pub model_type: String,
    pub is_custom: bool,
    pub created_at: String,
}

pub async fn list(db: &Db) -> AppResult<Vec<SavedSkinRow>> {
    let rows = sqlx::query_as::<_, SavedSkinRow>(
        "SELECT * FROM skins ORDER BY created_at DESC",
    )
    .fetch_all(db.pool())
    .await?;
    Ok(rows)
}

pub async fn get(db: &Db, id: &str) -> AppResult<Option<SavedSkinRow>> {
    let row = sqlx::query_as::<_, SavedSkinRow>(
        "SELECT * FROM skins WHERE id = ?",
    )
    .bind(id)
    .fetch_optional(db.pool())
    .await?;
    Ok(row)
}

pub async fn upsert(db: &Db, s: &SavedSkinRow) -> AppResult<()> {
    let now = Utc::now().to_rfc3339();
    let created = if s.created_at.is_empty() {
        now
    } else {
        s.created_at.clone()
    };

    sqlx::query(
        "INSERT INTO skins (id, name, skin_url, avatar_url, model_type, is_custom, created_at)
         VALUES (?, ?, ?, ?, ?, ?, ?)
         ON CONFLICT(id) DO UPDATE SET
            name = excluded.name,
            skin_url = excluded.skin_url,
            avatar_url = excluded.avatar_url,
            model_type = excluded.model_type,
            is_custom = excluded.is_custom",
    )
    .bind(&s.id)
    .bind(&s.name)
    .bind(&s.skin_url)
    .bind(&s.avatar_url)
    .bind(&s.model_type)
    .bind(s.is_custom)
    .bind(created)
    .execute(db.pool())
    .await?;
    Ok(())
}

pub async fn delete(db: &Db, id: &str) -> AppResult<()> {
    sqlx::query("DELETE FROM skins WHERE id = ?")
        .bind(id)
        .execute(db.pool())
        .await?;
    Ok(())
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct SavedCapeRow {
    pub id: String,
    pub name: String,
    pub cape_url: String,
    pub created_at: String,
}

pub async fn list_capes(db: &Db) -> AppResult<Vec<SavedCapeRow>> {
    let rows = sqlx::query_as::<_, SavedCapeRow>(
        "SELECT * FROM capes ORDER BY created_at DESC",
    )
    .fetch_all(db.pool())
    .await?;
    Ok(rows)
}

pub async fn upsert_cape(db: &Db, c: &SavedCapeRow) -> AppResult<()> {
    let now = Utc::now().to_rfc3339();
    let created = if c.created_at.is_empty() {
        now
    } else {
        c.created_at.clone()
    };

    sqlx::query(
        "INSERT INTO capes (id, name, cape_url, created_at)
         VALUES (?, ?, ?, ?)
         ON CONFLICT(id) DO UPDATE SET
            name = excluded.name,
            cape_url = excluded.cape_url",
    )
    .bind(&c.id)
    .bind(&c.name)
    .bind(&c.cape_url)
    .bind(created)
    .execute(db.pool())
    .await?;
    Ok(())
}

pub async fn delete_cape(db: &Db, id: &str) -> AppResult<()> {
    sqlx::query("DELETE FROM capes WHERE id = ?")
        .bind(id)
        .execute(db.pool())
        .await?;
    Ok(())
}
