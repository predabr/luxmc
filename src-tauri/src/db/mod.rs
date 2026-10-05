use std::path::PathBuf;
use std::sync::Arc;

use directories::ProjectDirs;
use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};
use sqlx::SqlitePool;
use tokio::sync::OnceCell;

use crate::error::AppError;
use crate::error::AppResult;

pub mod models;
pub mod schema;

#[derive(Clone)]
pub struct Db {
    pool: SqlitePool,
}

impl Db {
    pub async fn init() -> AppResult<Self> {
        let path = db_path()?;
        if let Some(parent) = path.parent() {
            tokio::fs::create_dir_all(parent).await?;
        }

        let opts = SqliteConnectOptions::new()
            .filename(&path)
            .create_if_missing(true)
            .journal_mode(sqlx::sqlite::SqliteJournalMode::Wal)
            .foreign_keys(true);

        let pool = SqlitePoolOptions::new()
            .max_connections(8)
            .connect_with(opts)
            .await
            .map_err(|e| AppError::Internal(format!("sqlite connect: {e}")))?;

        let mut migrator = sqlx::migrate!("./migrations");
        migrator.set_ignore_missing(true);
        if let Err(e) = migrator.run(&pool).await {
            let err_str = e.to_string();
            if err_str.contains("was previously applied but is missing in the resolved migrations") {
                tracing::warn!(error = %e, "Ignoring a migration recorded by a newer Luxmc build");
            } else if err_str.contains("was previously applied but has been modified") {
                tracing::warn!("SQLX migration checksum mismatch detected, self-repairing _sqlx_migrations...");
                for m in migrator.iter() {
                    let _ = sqlx::query("UPDATE _sqlx_migrations SET checksum = ? WHERE version = ?")
                        .bind(&*m.checksum)
                        .bind(m.version)
                        .execute(&pool)
                        .await;
                }
                migrator
                    .run(&pool)
                    .await
                    .map_err(|e2| AppError::Internal(format!("migrate retry: {e2}")))?;
            } else {
                return Err(AppError::Internal(format!("migrate: {e}")));
            }
        }

        normalize_account_expirations(&pool).await?;

        let _ = sqlx::query("ALTER TABLE accounts ADD COLUMN skin_url TEXT").execute(&pool).await;
        let _ = sqlx::query("ALTER TABLE accounts ADD COLUMN skin_variant TEXT").execute(&pool).await;
        let _ = sqlx::query("ALTER TABLE accounts ADD COLUMN cape_url TEXT").execute(&pool).await;

        let _ = sqlx::query(
            "CREATE TABLE IF NOT EXISTS skins (
                id TEXT PRIMARY KEY NOT NULL,
                name TEXT NOT NULL,
                skin_url TEXT NOT NULL,
                avatar_url TEXT NOT NULL,
                model_type TEXT NOT NULL,
                is_custom INTEGER NOT NULL DEFAULT 1,
                created_at TEXT NOT NULL
            )"
        ).execute(&pool).await;

        let _ = sqlx::query(
            "CREATE TABLE IF NOT EXISTS play_time_stats (
                id TEXT PRIMARY KEY NOT NULL,
                data TEXT NOT NULL,
                updated_at TEXT NOT NULL
            )"
        ).execute(&pool).await;

        let _ = sqlx::query(
            "CREATE TABLE IF NOT EXISTS capes (
                id TEXT PRIMARY KEY NOT NULL,
                name TEXT NOT NULL,
                cape_url TEXT NOT NULL,
                created_at TEXT NOT NULL
            )"
        ).execute(&pool).await;

        let _ = sqlx::query(
            "CREATE TABLE IF NOT EXISTS share_codes (
                code TEXT PRIMARY KEY NOT NULL,
                data TEXT NOT NULL,
                created_at TEXT NOT NULL
            )"
        ).execute(&pool).await;

        let _ = sqlx::query(
            "CREATE TABLE IF NOT EXISTS mod_icons (
                key TEXT PRIMARY KEY NOT NULL,
                icon_url TEXT NOT NULL
            )"
        ).execute(&pool).await;

        #[cfg(target_os = "linux")]
        {
            let mut transaction = pool.begin().await?;
            let first_enable = sqlx::query("INSERT OR IGNORE INTO app_settings (key, value) VALUES ('linux_gamemode_default_v1', 'enabled')")
                .execute(&mut *transaction).await?.rows_affected() > 0;
            if first_enable {
                sqlx::query("UPDATE profiles SET use_gamemode = 1 WHERE use_gamemode = 0")
                    .execute(&mut *transaction).await?;
            }
            transaction.commit().await?;
        }

        Ok(Self { pool })
    }

    pub fn pool(&self) -> &SqlitePool {
        &self.pool
    }
}

async fn normalize_account_expirations(pool: &SqlitePool) -> AppResult<()> {
    let rows = sqlx::query_as::<_, (String, String)>(
        "SELECT id, CAST(expires_at AS TEXT) FROM accounts WHERE expires_at IS NOT NULL",
    ).fetch_all(pool).await?;
    for (id, value) in rows {
        let Ok(epoch) = value.parse::<i64>() else { continue };
        let date = if epoch.unsigned_abs() > 100_000_000_000 {
            chrono::DateTime::from_timestamp_millis(epoch)
        } else {
            chrono::DateTime::from_timestamp(epoch, 0)
        }.ok_or_else(|| AppError::InvalidState("Data de expiração da conta fora do intervalo permitido".into()))?;
        sqlx::query("UPDATE accounts SET expires_at = ? WHERE id = ?")
            .bind(date.to_rfc3339()).bind(id).execute(pool).await?;
    }
    Ok(())
}

#[cfg(test)]
mod expiry_tests {
    use super::*;

    #[tokio::test]
    async fn normalizes_epoch_seconds_and_milliseconds_without_changing_dates() {
        let pool = SqlitePool::connect("sqlite::memory:").await.unwrap();
        sqlx::query("CREATE TABLE accounts (id TEXT, expires_at TEXT)").execute(&pool).await.unwrap();
        for (id, expiry) in [("seconds", Some("1791129970")), ("milliseconds", Some("1791129970000")), ("iso", Some("2026-10-04T00:00:00Z")), ("empty", None)] {
            sqlx::query("INSERT INTO accounts VALUES (?, ?)").bind(id).bind(expiry).execute(&pool).await.unwrap();
        }
        normalize_account_expirations(&pool).await.unwrap();
        normalize_account_expirations(&pool).await.unwrap();
        let rows = sqlx::query_as::<_, (String, Option<chrono::DateTime<chrono::Utc>>)>("SELECT id, expires_at FROM accounts ORDER BY id").fetch_all(&pool).await.unwrap();
        assert!(rows[0].1.is_none());
        assert_eq!(rows[1].1.unwrap().to_rfc3339(), "2026-10-04T00:00:00+00:00");
        assert_eq!(rows[2].1, rows[3].1);
        assert_eq!(rows[2].1.unwrap().timestamp_millis(), 1791129970000);
    }
}

pub fn db_path() -> AppResult<PathBuf> {
    let dirs = ProjectDirs::from("io", "github", "Luxmc")
        .ok_or_else(|| AppError::InvalidState("could not determine data dir".into()))?;
    Ok(dirs.data_dir().join("luxmc.db"))
}

static DB: OnceCell<Arc<Db>> = OnceCell::const_new();

pub async fn shared_db() -> AppResult<Arc<Db>> {
    DB.get_or_try_init(|| async { Db::init().await.map(Arc::new).map_err(AppError::from) })
        .await
        .cloned()
}
