use std::path::Path;
use sqlx::SqlitePool;
use crate::error::{AppError, AppResult};

pub(super) async fn verify(pool: &SqlitePool) -> AppResult<()> {
    let results = sqlx::query_scalar::<_, String>("PRAGMA quick_check").fetch_all(pool).await;
    match results {
        Ok(rows) if rows.len() == 1 && rows[0] == "ok" => Ok(()),
        Ok(_) => Err(AppError::InvalidState("O banco local precisa de recuperação. Seus arquivos de instâncias e mundos estão preservados. Os backups ficam na pasta database-backups dos dados do Luxmc.".into())),
        Err(error) => Err(AppError::Internal(format!("Não foi possível verificar o banco local; nenhum dado foi apagado: {error}"))),
    }
}

pub(super) async fn save(pool: &SqlitePool, database: &Path) -> AppResult<()> {
    let parent = database.parent().ok_or_else(|| AppError::InvalidState("Database directory unavailable".into()))?;
    let folder = parent.join("database-backups");
    tokio::fs::create_dir_all(&folder).await?;
    let target = folder.join(format!("luxmc-{}.db", chrono::Utc::now().format("%Y-%m-%d")));
    if target.exists() { return Ok(()); }
    let temporary = folder.join(format!("{}.tmp", uuid::Uuid::new_v4()));
    sqlx::query("VACUUM INTO ?").bind(temporary.to_string_lossy().as_ref()).execute(pool).await?;
    let backup = SqlitePool::connect_with(sqlx::sqlite::SqliteConnectOptions::new().filename(&temporary).read_only(true)).await?;
    let result = verify(&backup).await;
    backup.close().await;
    result?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        tokio::fs::set_permissions(&temporary, std::fs::Permissions::from_mode(0o600)).await?;
    }
    tokio::fs::rename(temporary, target).await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn snapshot_preserves_committed_records_and_does_not_replace_todays_backup() {
        let root = std::env::temp_dir().join(format!("luxmc-db-{}", uuid::Uuid::new_v4()));
        tokio::fs::create_dir_all(&root).await.unwrap();
        let path = root.join("luxmc.db");
        let pool = SqlitePool::connect_with(sqlx::sqlite::SqliteConnectOptions::new().filename(&path).create_if_missing(true).journal_mode(sqlx::sqlite::SqliteJournalMode::Wal)).await.unwrap();
        sqlx::query("CREATE TABLE profiles(id TEXT PRIMARY KEY, name TEXT)").execute(&pool).await.unwrap();
        sqlx::query("INSERT INTO profiles VALUES ('one', 'My world')").execute(&pool).await.unwrap();
        save(&pool, &path).await.unwrap();
        sqlx::query("DELETE FROM profiles").execute(&pool).await.unwrap();
        save(&pool, &path).await.unwrap();
        let target = root.join("database-backups").join(format!("luxmc-{}.db", chrono::Utc::now().format("%Y-%m-%d")));
        let backup = SqlitePool::connect_with(sqlx::sqlite::SqliteConnectOptions::new().filename(target).read_only(true)).await.unwrap();
        assert_eq!(sqlx::query_scalar::<_, String>("SELECT name FROM profiles WHERE id = 'one'").fetch_one(&backup).await.unwrap(), "My world");
        backup.close().await;
        pool.close().await;
        tokio::fs::remove_dir_all(root).await.unwrap();
    }
}
