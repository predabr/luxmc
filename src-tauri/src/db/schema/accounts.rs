use crate::db::models::AccountRow;
use crate::db::Db;
use crate::error::AppResult;
use chrono::Utc;

pub async fn list(db: &Db) -> AppResult<Vec<AccountRow>> {
    let rows = sqlx::query_as::<_, AccountRow>("SELECT * FROM accounts ORDER BY created_at ASC")
        .fetch_all(db.pool())
        .await
        .map_err(crate::error::AppError::from)?;
    Ok(rows)
}

pub async fn upsert(db: &Db, account: &AccountRow) -> AppResult<()> {
    let now = Utc::now();
    sqlx::query(
		"INSERT INTO accounts (id, username, uuid, refresh_token, access_token, expires_at, created_at, updated_at)
		 VALUES (?, ?, ?, ?, ?, ?, ?, ?)
		 ON CONFLICT(uuid) DO UPDATE SET
			username = excluded.username,
			refresh_token = excluded.refresh_token,
			access_token = excluded.access_token,
			expires_at = excluded.expires_at,
			updated_at = excluded.updated_at",
	)
	.bind(&account.id)
	.bind(&account.username)
	.bind(&account.uuid)
	.bind(&account.refresh_token)
	.bind(&account.access_token)
	.bind(&account.expires_at)
	.bind(&account.created_at)
	.bind(now)
	.execute(db.pool())
	.await
	.map_err(crate::error::AppError::from)?;
    Ok(())
}

pub async fn get_by_uuid(db: &Db, uuid: &str) -> AppResult<Option<AccountRow>> {
    let row = sqlx::query_as::<_, AccountRow>("SELECT * FROM accounts WHERE uuid = ?")
        .bind(uuid)
        .fetch_optional(db.pool())
        .await
        .map_err(crate::error::AppError::from)?;
    Ok(row)
}

pub async fn delete(db: &Db, uuid: &str) -> AppResult<()> {
    sqlx::query("DELETE FROM accounts WHERE uuid = ?")
        .bind(uuid)
        .execute(db.pool())
        .await
        .map_err(crate::error::AppError::from)?;
    Ok(())
}
