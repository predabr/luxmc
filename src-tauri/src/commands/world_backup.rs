use crate::error::{AppError, AppResult};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WorldBackupEntry {
    pub file_name: String,
    pub file_path: String,
    pub size_bytes: u64,
    pub created_at: String,
    pub world_name: String,
}

#[tauri::command]
#[allow(non_snake_case)]
pub async fn instance_backup_world(
    profileId: String,
    worldFolder: String,
) -> AppResult<WorldBackupEntry> {
    let db = crate::db::shared_db().await?;
    let row = sqlx::query_as::<_, crate::db::models::ProfileRow>("SELECT * FROM profiles WHERE id = ?")
        .bind(&profileId)
        .fetch_optional(db.pool())
        .await?
        .ok_or_else(|| AppError::NotFound(format!("profile {profileId} not found")))?;

    let saves_dir = PathBuf::from(&row.game_dir).join("saves");
    let world_dir = crate::core::instance_paths::resolve_within(&saves_dir, &worldFolder)?;
    if !world_dir.is_dir() {
        return Err(AppError::NotFound("Pasta do mundo não encontrada".into()));
    }

    let backups_dir = PathBuf::from(&row.game_dir).join("backups_luxmc");
    tokio::fs::create_dir_all(&backups_dir).await?;

    let now_str = chrono::Local::now().format("%Y%m%d_%H%M%S").to_string();
    let zip_name = format!("{}_{}.zip", worldFolder, now_str);
    let zip_dest = backups_dir.join(&zip_name);

    let zip_dest_result = zip_dest.clone();
    let size_bytes = tokio::task::spawn_blocking(move || -> AppResult<u64> {
        let file = std::fs::File::create(&zip_dest)
            .map_err(|e| AppError::Internal(format!("Falha ao criar arquivo de backup: {e}")))?;
        let mut zip = zip::ZipWriter::new(file);

        let options = zip::write::FileOptions::default()
            .compression_method(zip::CompressionMethod::Deflated);

        add_folder_to_zip(&mut zip, &world_dir, "", options)?;
        zip.finish().map_err(|e| AppError::Internal(e.to_string()))?;

        Ok(std::fs::metadata(&zip_dest).map(|m| m.len()).unwrap_or(0))
    })
    .await
    .map_err(|error| AppError::Internal(error.to_string()))??;

    if let Ok(mut entries) = tokio::fs::read_dir(&backups_dir).await {
        let mut world_backups = Vec::new();
        while let Ok(Some(entry)) = entries.next_entry().await {
            let name = entry.file_name().to_string_lossy().to_string();
            if name.starts_with(&worldFolder) && name.ends_with(".zip") {
                world_backups.push(entry.path());
            }
        }
        if world_backups.len() > 5 {
            world_backups.sort();
            for old in world_backups.iter().take(world_backups.len() - 5) {
                let _ = tokio::fs::remove_file(old).await;
            }
        }
    }

    Ok(WorldBackupEntry {
        file_name: zip_name,
        file_path: zip_dest_result.to_string_lossy().to_string(),
        size_bytes,
        created_at: chrono::Local::now().format("%d/%m/%Y %H:%M").to_string(),
        world_name: worldFolder,
    })
}

#[tauri::command]
#[allow(non_snake_case)]
pub async fn instance_list_world_backups(
    profileId: String,
) -> AppResult<Vec<WorldBackupEntry>> {
    let db = crate::db::shared_db().await?;
    let row = sqlx::query_as::<_, crate::db::models::ProfileRow>("SELECT * FROM profiles WHERE id = ?")
        .bind(&profileId)
        .fetch_optional(db.pool())
        .await?
        .ok_or_else(|| AppError::NotFound(format!("profile {profileId} not found")))?;

    let backups_dir = PathBuf::from(&row.game_dir).join("backups_luxmc");
    if !backups_dir.exists() {
        return Ok(Vec::new());
    }

    let mut result = Vec::new();
    if let Ok(mut entries) = tokio::fs::read_dir(&backups_dir).await {
        while let Ok(Some(entry)) = entries.next_entry().await {
            let path = entry.path();
            if path.extension().map_or(false, |e| e == "zip") {
                let name = entry.file_name().to_string_lossy().to_string();
                let meta = entry.metadata().await.ok();
                let size_bytes = meta.as_ref().map(|m| m.len()).unwrap_or(0);
                let created_at = meta
                    .and_then(|m| m.created().or_else(|_| m.modified()).ok())
                    .map(|st| chrono::DateTime::<chrono::Local>::from(st).format("%d/%m/%Y %H:%M").to_string())
                    .unwrap_or_else(|| "Recente".into());

                let world_name = name.split('_').next().unwrap_or(&name).to_string();
                result.push(WorldBackupEntry {
                    file_name: name,
                    file_path: path.to_string_lossy().to_string(),
                    size_bytes,
                    created_at,
                    world_name,
                });
            }
        }
    }

    result.sort_by(|a, b| b.file_name.cmp(&a.file_name));
    Ok(result)
}

fn add_folder_to_zip(
    zip: &mut zip::ZipWriter<std::fs::File>,
    dir: &std::path::Path,
    prefix: &str,
    options: zip::write::FileOptions,
) -> AppResult<()> {
    use std::io::Write;

    let listing = match std::fs::read_dir(dir) {
        Ok(entries) => entries,
        Err(e) => {
            tracing::warn!(dir = %dir.display(), error = %e, "unreadable directory during backup");
            return Ok(());
        }
    };
    {
        let entries = listing;
        for entry in entries.flatten() {
            let path = entry.path();
            let name = entry.file_name().to_string_lossy().to_string();
            let zip_path = if prefix.is_empty() {
                name.clone()
            } else {
                format!("{}/{}", prefix, name)
            };

            if path.is_dir() {
                add_folder_to_zip(zip, &path, &zip_path, options)?;
            } else if path.is_file() {
                match std::fs::read(&path) {
                    Ok(bytes) => {
                        zip.start_file(&zip_path, options)?;
                        zip.write_all(&bytes)?;
                    }
                    Err(e) => {
                        tracing::warn!(path = %path.display(), error = %e, "skipped unreadable file during backup");
                    }
                }
            }
        }
    }
    Ok(())
}
