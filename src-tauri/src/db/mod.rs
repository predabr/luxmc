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
