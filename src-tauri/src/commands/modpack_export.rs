use crate::error::{AppError, AppResult};
use directories::UserDirs;
use serde::{Deserialize, Serialize};
use std::io::Write;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExportResult {
    pub success: bool,
    pub file_path: String,
    pub file_size: u64,
    pub total_mods: usize,
}

#[tauri::command]
#[allow(non_snake_case)]
pub async fn instance_export_modpack(
    profileId: String,
    exportFormat: String,
    customName: Option<String>,
) -> AppResult<ExportResult> {
    let db = crate::db::shared_db().await?;
    let row = sqlx::query_as::<_, crate::db::models::ProfileRow>(
        "SELECT * FROM profiles WHERE id = ?",
    )
    .bind(&profileId)
    .fetch_optional(db.pool())
    .await?
    .ok_or_else(|| AppError::NotFound(format!("profile {profileId} not found")))?;

    let game_dir = PathBuf::from(&row.game_dir);
    if !game_dir.exists() {
        return Err(AppError::NotFound("Pasta da instância não existe".into()));
    }

    let default_name = customName
        .unwrap_or_else(|| row.name.clone())
        .replace(|c: char| !c.is_alphanumeric() && c != '_' && c != '-', "_");

    let ext = if exportFormat.to_lowercase() == "mrpack" {
        "mrpack"
    } else {
        "zip"
    };

    let downloads_dir = UserDirs::new()
        .and_then(|u| u.download_dir().map(|p| p.to_path_buf()))
        .unwrap_or_else(|| game_dir.clone());

    let out_file_path = downloads_dir.join(format!("{}.{}", default_name, ext));
    let file = std::fs::File::create(&out_file_path)
        .map_err(|e| AppError::Internal(format!("Falha ao criar arquivo de exportação: {e}")))?;
    let mut zip = zip::ZipWriter::new(file);

    let options = zip::write::FileOptions::default()
        .compression_method(zip::CompressionMethod::Deflated)
        .unix_permissions(0o755);

    let folders_to_include = [
        "config",
        "defaultconfigs",
        "kubejs",
        "patchouli_books",
        "openloader",
        "resourcepacks",
        "shaderpacks",
    ];

    let mut total_mods = 0usize;

    let prefix = "overrides/";

    for folder_name in folders_to_include {
        let src = game_dir.join(folder_name);
        if src.is_dir() {
            add_dir_to_zip(&mut zip, &src, &format!("{}{}/", prefix, folder_name), options)?;
        }
    }

    let mods_dir = game_dir.join("mods");
    if mods_dir.is_dir() {
        if let Ok(entries) = std::fs::read_dir(&mods_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.extension().map_or(false, |e| e == "jar") {
                    let fname = entry.file_name().to_string_lossy().to_string();
                    let zip_path = format!("{}mods/{}", prefix, fname);
                    if let Ok(bytes) = std::fs::read(&path) {
                        zip.start_file(zip_path, options).map_err(|e| AppError::Internal(e.to_string()))?;
                        zip.write_all(&bytes).map_err(|e| AppError::Internal(e.to_string()))?;
                        total_mods += 1;
                    }
                }
            }
        }
    }

    if ext == "mrpack" {
        let index = serde_json::json!({
            "formatVersion": 1,
            "game": "minecraft",
            "versionId": default_name,
            "name": row.name,
            "summary": "Modpack exportado via Luxmc Launcher",
            "files": [],
            "dependencies": {
                "minecraft": row.mc_version,
                row.loader.to_lowercase(): row.loader_version.clone().unwrap_or_else(|| "latest".into())
            }
        });
        zip.start_file("modrinth.index.json", options).map_err(|e| AppError::Internal(e.to_string()))?;
        let json_bytes = serde_json::to_vec_pretty(&index).unwrap_or_default();
        zip.write_all(&json_bytes).map_err(|e| AppError::Internal(e.to_string()))?;
    } else {
        let manifest = serde_json::json!({
            "minecraft": {
                "version": row.mc_version,
                "modLoaders": [
                    {
                        "id": format!("{}-{}", row.loader, row.loader_version.clone().unwrap_or_else(|| "latest".into())),
                        "primary": true
                    }
                ]
            },
            "manifestType": "minecraftModpack",
            "manifestVersion": 1,
            "name": row.name,
            "version": "1.0.0",
            "author": "Luxmc Player",
            "files": [],
            "overrides": "overrides"
        });
        zip.start_file("manifest.json", options).map_err(|e| AppError::Internal(e.to_string()))?;
        let json_bytes = serde_json::to_vec_pretty(&manifest).unwrap_or_default();
        zip.write_all(&json_bytes).map_err(|e| AppError::Internal(e.to_string()))?;
    }

    zip.finish().map_err(|e| AppError::Internal(format!("Falha ao finalizar ZIP: {e}")))?;

    let file_size = std::fs::metadata(&out_file_path)
        .map(|m| m.len())
        .unwrap_or(0);

    Ok(ExportResult {
        success: true,
        file_path: out_file_path.to_string_lossy().to_string(),
        file_size,
        total_mods,
    })
}

fn add_dir_to_zip(
    zip: &mut zip::ZipWriter<std::fs::File>,
    dir: &Path,
    prefix: &str,
    options: zip::write::FileOptions,
) -> AppResult<()> {
    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            let name = entry.file_name().to_string_lossy().to_string();
            let zip_path = format!("{}{}", prefix, name);
            if path.is_dir() {
                add_dir_to_zip(zip, &path, &format!("{}/", zip_path), options)?;
            } else if path.is_file() {
                if let Ok(bytes) = std::fs::read(&path) {
                    let _ = zip.start_file(zip_path, options);
                    let _ = zip.write_all(&bytes);
                }
            }
        }
    }
    Ok(())
}
