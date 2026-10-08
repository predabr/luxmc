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
pub async fn instance_backup_world(state: tauri::State<'_,crate::state::AppState>, profileId: String, worldFolder: String) -> AppResult<WorldBackupEntry> {
    let _guard=state.launch_lock.try_lock().map_err(|_|AppError::InvalidState("Aguarde a preparação do Minecraft terminar".into()))?;
    instance_backup_world_core(profileId,worldFolder).await
}

#[allow(non_snake_case)]
pub async fn instance_backup_world_core(profileId: String, worldFolder: String) -> AppResult<WorldBackupEntry> {
    let db = crate::db::shared_db().await?;
    let row = sqlx::query_as::<_, crate::db::models::ProfileRow>("SELECT * FROM profiles WHERE id = ?")
        .bind(&profileId)
        .fetch_optional(db.pool())
        .await?
        .ok_or_else(|| AppError::NotFound(format!("profile {profileId} not found")))?;

    let saves_dir = PathBuf::from(&row.game_dir).join("saves");
    if crate::core::launcher::get_active_game_pid() != 0 { return Err(AppError::InvalidState("Feche o Minecraft antes de criar um backup consistente".into())); }
    let world_dir = crate::core::instance_paths::resolve_within(&saves_dir, &worldFolder)?;
    if !world_dir.is_dir() {
        return Err(AppError::NotFound("Pasta do mundo não encontrada".into()));
    }

    let backups_dir = PathBuf::from(&row.game_dir).join("backups_luxmc");
    tokio::fs::create_dir_all(&backups_dir).await?;

    let now_str = chrono::Local::now().format("%Y%m%d_%H%M%S%.9f").to_string();
    let zip_name = format!("{}_{}.zip", worldFolder, now_str);
    let zip_dest = backups_dir.join(&zip_name);

    let zip_dest_result = zip_dest.clone();
    let size_bytes = tokio::task::spawn_blocking(move || -> AppResult<u64> {
        let temporary = zip_dest.with_extension("zip.partial");
        let file = std::fs::File::create(&temporary)
            .map_err(|e| AppError::Internal(format!("Falha ao criar arquivo de backup: {e}")))?;
        let mut zip = zip::ZipWriter::new(file);

        let options = zip::write::FileOptions::default()
            .compression_method(zip::CompressionMethod::Deflated);

        add_folder_to_zip(&mut zip, &world_dir, "", options)?;
        zip.finish().map_err(|e| AppError::Internal(e.to_string()))?;
        std::fs::rename(&temporary,&zip_dest)?;

        Ok(std::fs::metadata(&zip_dest).map(|m| m.len()).unwrap_or(0))
    })
    .await
    .map_err(|error| AppError::Internal(error.to_string()))??;

    if let Ok(mut entries) = tokio::fs::read_dir(&backups_dir).await {
        let mut world_backups = Vec::new();
        while let Ok(Some(entry)) = entries.next_entry().await {
            let name = entry.file_name().to_string_lossy().to_string();
            if name.rsplitn(3,'_').last() == Some(worldFolder.as_str()) && name.ends_with(".zip") {
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

                let world_name = name.rsplitn(3,'_').last().unwrap_or(&name).to_string();
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

    let timestamp=|name: &str| { let parts: Vec<_>=name.trim_end_matches(".zip").rsplitn(3,'_').collect(); format!("{}_{}",parts.get(1).unwrap_or(&""),parts.first().unwrap_or(&"")) };
    result.sort_by(|a,b| timestamp(&b.file_name).cmp(&timestamp(&a.file_name)));
    Ok(result)
}

fn add_folder_to_zip(
    zip: &mut zip::ZipWriter<std::fs::File>,
    dir: &std::path::Path,
    prefix: &str,
    options: zip::write::FileOptions,
) -> AppResult<()> {

    let listing = match std::fs::read_dir(dir) {
        Ok(entries) => entries,
        Err(e) => return Err(e.into()),
    };
    {
        let entries = listing;
        for entry in entries {
            let entry = entry?;
            let path = entry.path();
            if entry.file_type()?.is_symlink() { return Err(AppError::InvalidInput("O mundo contém um link simbólico; backup interrompido".into())); }
            let name = entry.file_name().to_string_lossy().to_string();
            let zip_path = if prefix.is_empty() {
                name.clone()
            } else {
                format!("{}/{}", prefix, name)
            };

            if path.is_dir() {
                add_folder_to_zip(zip, &path, &zip_path, options)?;
            } else if path.is_file() {
                zip.start_file(&zip_path, options)?;
                std::io::copy(&mut std::fs::File::open(&path)?,zip)?;
            }
        }
    }
    Ok(())
}

pub async fn automatic(profile_id: &str) -> AppResult<()> {
    let settings = crate::commands::settings::settings_get().await?;
    if settings["autoBackup"] != true { return Ok(()); }
    let db = crate::db::shared_db().await?;
    let game_dir: String = sqlx::query_scalar("SELECT game_dir FROM profiles WHERE id = ?").bind(profile_id).fetch_one(db.pool()).await?;
    let saves = PathBuf::from(&game_dir).join("saves");
    if !saves.is_dir() { return Ok(()); }
    let existing = instance_list_world_backups(profile_id.into()).await?;
    let mut worlds = tokio::fs::read_dir(&saves).await?;
    while let Some(world) = worlds.next_entry().await? {
        if !world.file_type().await?.is_dir() || !world.path().join("level.dat").is_file() { continue; }
        let name = world.file_name().to_string_lossy().into_owned();
        if name.starts_with('.') {continue;}
        let recent = existing.iter().filter(|backup| backup.world_name == name).any(|backup| std::fs::metadata(&backup.file_path).and_then(|m| m.modified()).ok().and_then(|t| t.elapsed().ok()).is_some_and(|elapsed| elapsed.as_secs() < 86400));
        if !recent { instance_backup_world_core(profile_id.into(),name).await?; }
    }
    let limit = settings["backupLimitGb"].as_u64().unwrap_or(5).clamp(1,100) * 1024 * 1024 * 1024;
    let entries = instance_list_world_backups(profile_id.into()).await?;
    let mut size: u64 = entries.iter().map(|entry| entry.size_bytes).sum();
    let mut newest = std::collections::HashSet::new();
    let protected: std::collections::HashSet<String> = entries.iter().filter(|entry| newest.insert(entry.world_name.clone())).map(|entry| entry.file_name.clone()).collect();
    for entry in entries.iter().rev() {
        if size <= limit { break; }
        if protected.contains(&entry.file_name) { continue; }
        tokio::fs::remove_file(&entry.file_path).await?;
        size = size.saturating_sub(entry.size_bytes);
    }
    Ok(())
}

#[tauri::command]
pub async fn world_restore(state: tauri::State<'_,crate::state::AppState>,profile_id: String, file_name: String) -> AppResult<()> {
    let _guard=state.launch_lock.try_lock().map_err(|_|AppError::InvalidState("Aguarde a preparação do Minecraft terminar".into()))?;
    world_restore_core(profile_id,file_name).await
}

pub async fn world_restore_core(profile_id: String, file_name: String) -> AppResult<()> {
    if crate::core::launcher::get_active_game_pid() != 0 { return Err(AppError::InvalidState("Feche o Minecraft antes de restaurar".into())); }
    let entries = instance_list_world_backups(profile_id.clone()).await?;
    let entry = entries.into_iter().find(|entry| entry.file_name == file_name).ok_or_else(|| AppError::NotFound("Backup não encontrado".into()))?;
    let db = crate::db::shared_db().await?;
    let game_dir: String = sqlx::query_scalar("SELECT game_dir FROM profiles WHERE id = ?").bind(&profile_id).fetch_one(db.pool()).await?;
    let saves = PathBuf::from(&game_dir).join("saves");
    let world = crate::core::instance_paths::resolve_within(&saves,&entry.world_name)?;
    let staging = saves.join(format!(".restore-{}",uuid::Uuid::new_v4()));
    tokio::fs::create_dir_all(&staging).await?;
    let staging_copy = staging.clone();
    let extracted = tokio::task::spawn_blocking(move || -> AppResult<()> {
        extract_backup(std::path::Path::new(&entry.file_path),&staging_copy)?;
        Ok(())
    }).await.map_err(|e| AppError::Internal(e.to_string()))?;
    if let Err(error) = extracted { let _ = tokio::fs::remove_dir_all(&staging).await; return Err(error); }
    if crate::core::launcher::get_active_game_pid() != 0 { return Err(AppError::InvalidState("O Minecraft foi aberto durante a restauração; feche-o e tente novamente".into())); }
    let previous = saves.join(format!(".before-restore-{}",uuid::Uuid::new_v4()));
    let had_world = world.exists();
    if had_world { tokio::fs::rename(&world,&previous).await?; }
    if let Err(error) = tokio::fs::rename(&staging,&world).await { if had_world { let _ = tokio::fs::rename(&previous,&world).await; } return Err(error.into()); }
    Ok(())
}

fn extract_backup(source: &std::path::Path, staging: &std::path::Path) -> AppResult<()> {
        let mut archive = zip::ZipArchive::new(std::fs::File::open(source)?)?;
        let mut total = 0u64;
        for index in 0..archive.len() {
            let mut file = archive.by_index(index)?;
            if file.unix_mode().is_some_and(|mode| mode & 0o170000 == 0o120000) { return Err(AppError::InvalidInput("Link simbólico no backup".into())); }
            total = total.saturating_add(file.size());
            if total > 32 * 1024 * 1024 * 1024 { return Err(AppError::InvalidInput("Backup descompactado acima de 32 GiB".into())); }
            let target = crate::core::mods::pack_download::destination(staging,file.name())?;
            if file.is_dir() { std::fs::create_dir_all(target)?; }
            else { if let Some(parent) = target.parent() { std::fs::create_dir_all(parent)?; } std::io::copy(&mut file,&mut std::fs::File::create(target)?)?; }
        }
        if !staging.join("level.dat").is_file() { return Err(AppError::InvalidInput("O backup não contém level.dat".into())); }
    Ok(())
}

#[cfg(test)]
mod tests {
    #[test]
    fn roundtrip_keeps_nested_world_files_and_rejects_traversal() {
        use std::io::Write;
        let root=std::env::temp_dir().join(format!("luxmc-world-{}",uuid::Uuid::new_v4()));
        let world=root.join("world");let staging=root.join("restored");
        std::fs::create_dir_all(world.join("region")).unwrap();std::fs::create_dir_all(&staging).unwrap();
        std::fs::write(world.join("level.dat"),b"world metadata").unwrap();std::fs::write(world.join("region/r.0.0.mca"),b"world data").unwrap();
        let archive=root.join("backup.zip");let mut writer=zip::ZipWriter::new(std::fs::File::create(&archive).unwrap());
        super::add_folder_to_zip(&mut writer,&world,"",zip::write::FileOptions::default()).unwrap();writer.finish().unwrap();
        super::extract_backup(&archive,&staging).unwrap();assert_eq!(std::fs::read(staging.join("region/r.0.0.mca")).unwrap(),b"world data");assert_eq!(std::fs::read(world.join("level.dat")).unwrap(),b"world metadata");
        let attack=root.join("attack.zip");let mut writer=zip::ZipWriter::new(std::fs::File::create(&attack).unwrap());writer.start_file("../escape.txt",zip::write::FileOptions::default()).unwrap();writer.write_all(b"escape").unwrap();writer.finish().unwrap();
        assert!(super::extract_backup(&attack,&staging).is_err());assert!(!root.join("escape.txt").exists());std::fs::remove_dir_all(root).unwrap();
    }
}
