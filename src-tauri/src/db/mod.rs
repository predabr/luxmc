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
            std::fs::create_dir_all(parent)?;
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

        let migrator = sqlx::migrate!("./migrations");
        if let Err(e) = migrator.run(&pool).await {
            let err_str = e.to_string();
            if err_str.contains("was previously applied but has been modified") {
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

        Ok(Self { pool })
    }

    pub fn pool(&self) -> &SqlitePool {
        &self.pool
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
