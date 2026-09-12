use serde::{Deserialize, Serialize};
use tauri::{Emitter, State};
use uuid::Uuid;

use crate::db::models::ProfileRow;
use crate::error::AppResult;
use crate::state::AppState;

fn dir_size_recursive(p: &std::path::Path) -> u64 {
    let mut total = 0u64;
    if let Ok(entries) = std::fs::read_dir(p) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                total += dir_size_recursive(&path);
            } else if let Ok(meta) = path.metadata() {
                total += meta.len();
            }
        }
    }
    total
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HealthCheckResult {
    pub client_jar: bool,
    pub natives: bool,
    pub mods_ok: bool,
    pub issues: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FileTreeEntry {
    pub name: String,
    pub path: String,
    pub is_dir: bool,
    pub size: u64,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ScreenshotEntry {
    pub name: String,
    pub path: String,
    pub modified: String,
    pub data_url: Option<String>,
}

#[tauri::command]
pub async fn instances_list(_state: State<'_, AppState>) -> AppResult<Vec<ProfileRow>> {
    let db = crate::db::shared_db().await?;
    crate::db::schema::profiles::list(&db).await
}

#[tauri::command]
pub async fn instances_duplicate(_state: State<'_, AppState>, id: String) -> AppResult<ProfileRow> {
    let db = crate::db::shared_db().await?;
    let existing = sqlx::query_as::<_, ProfileRow>("SELECT * FROM profiles WHERE id = ?")
        .bind(&id)
        .fetch_optional(db.pool())
        .await?
        .ok_or_else(|| crate::error::AppError::NotFound(format!("profile {id} not found")))?;

    let now = chrono::Utc::now();
    let row = ProfileRow {
        id: Uuid::new_v4().to_string(),
        name: format!("{} (copy)", existing.name),
        icon: existing.icon,
        mc_version: existing.mc_version,
        loader: existing.loader,
        loader_version: existing.loader_version,
        java_path: existing.java_path,
        jvm_args: existing.jvm_args,
        resolution_w: existing.resolution_w,
        resolution_h: existing.resolution_h,
        fullscreen: existing.fullscreen,
        game_dir: existing.game_dir,
        created_at: now,
        updated_at: now,
        favorite: false,
        notes: None,
        last_played: None,
        launch_count: 0,
        mod_count: existing.mod_count,
        disk_usage: existing.disk_usage,
        ram_mb: existing.ram_mb,
        instance_group: existing.instance_group,
        auto_optimize: existing.auto_optimize,
        use_vulkan: existing.use_vulkan,
    };
    crate::db::schema::profiles::upsert(&db, &row).await?;
    Ok(row)
}

#[tauri::command]
pub async fn instances_open_folder(_state: State<'_, AppState>, id: String) -> AppResult<()> {
    let db = crate::db::shared_db().await?;
    let row = sqlx::query_as::<_, ProfileRow>("SELECT * FROM profiles WHERE id = ?")
        .bind(&id)
        .fetch_optional(db.pool())
        .await?
        .ok_or_else(|| crate::error::AppError::NotFound(format!("profile {id} not found")))?;

    open::that(&row.game_dir)?;
    Ok(())
}

#[tauri::command]
pub async fn instances_screenshots(
    _state: State<'_, AppState>,
    id: String,
) -> AppResult<Vec<ScreenshotEntry>> {
    let db = crate::db::shared_db().await?;
    let row = sqlx::query_as::<_, ProfileRow>("SELECT * FROM profiles WHERE id = ?")
        .bind(&id)
        .fetch_optional(db.pool())
        .await?
        .ok_or_else(|| crate::error::AppError::NotFound(format!("profile {id} not found")))?;

    let dir = std::path::Path::new(&row.game_dir).join("screenshots");
    let mut entries = Vec::new();

    if dir.is_dir() {
        for entry in std::fs::read_dir(&dir)? {
            let entry = entry?;
            let path = entry.path();

            let is_image = path
                .extension()
                .and_then(|e| e.to_str())
                .map(|e| matches!(e.to_lowercase().as_str(), "png" | "jpg" | "jpeg"))
                .unwrap_or(false);

            if !is_image {
                continue;
            }

            let metadata = entry.metadata()?;
            let modified = metadata
                .modified()
                .ok()
                .and_then(|t| {
                    let dt: chrono::DateTime<chrono::Utc> = t.into();
                    Some(dt.to_rfc3339())
                })
                .unwrap_or_default();

            let data_url = None;

            entries.push(ScreenshotEntry {
                name: path
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("")
                    .to_string(),
                path: path.to_string_lossy().to_string(),
                modified,
                data_url,
            });
        }
    }

    Ok(entries)
}

/// Delete a screenshot file by its absolute path.
/// Only files inside a `screenshots/` sub-directory are accepted to prevent
/// path-traversal attacks.
#[tauri::command]
pub async fn screenshot_delete(path: String) -> AppResult<()> {
    let p = std::path::Path::new(&path);
    let canonical = match p.canonicalize() {
        Ok(c) => c,
        Err(_) => {
            return Err(crate::error::AppError::Internal(
                "Path does not exist".to_string(),
            ));
        }
    };
    let path_str = canonical.to_string_lossy();
    let dominated_by_screenshots = path_str
        .split(std::path::MAIN_SEPARATOR)
        .any(|component| component == "screenshots");
    if !dominated_by_screenshots {
        return Err(crate::error::AppError::Internal(
            "Path must be inside a screenshots directory".to_string(),
        ));
    }
    tokio::fs::remove_file(&canonical).await?;
    Ok(())
}

/// Open the screenshots folder in the system file manager.
#[tauri::command]
pub async fn screenshots_open_folder(app: tauri::AppHandle, profile_id: String) -> AppResult<()> {
    use tauri_plugin_opener::OpenerExt;
    let db = crate::db::shared_db().await?;
    let row =
        sqlx::query_as::<_, crate::db::models::ProfileRow>("SELECT * FROM profiles WHERE id = ?")
            .bind(&profile_id)
            .fetch_optional(db.pool())
            .await?
            .ok_or_else(|| {
                crate::error::AppError::NotFound(format!("profile {profile_id} not found"))
            })?;

    let dir = std::path::Path::new(&row.game_dir).join("screenshots");
    if !dir.exists() {
        tokio::fs::create_dir_all(&dir).await?;
    }
    app.opener()
        .open_path(dir.to_string_lossy().as_ref(), None::<&str>)
        .map_err(|e| crate::error::AppError::Internal(e.to_string()))?;
    Ok(())
}

#[derive(Debug, Deserialize)]
struct CfManifest {
    #[serde(default)]
    minecraft: CfMinecraft,
    #[serde(default)]
    files: Vec<CfFile>,
}

#[derive(Debug, Default, Deserialize)]
struct CfMinecraft {
    #[serde(default)]
    version: String,
    #[serde(default, alias = "modLoaders")]
    mod_loaders: Vec<CfModLoader>,
}

#[derive(Debug, Deserialize)]
struct CfModLoader {
    #[serde(default)]
    id: String,
}

#[derive(Debug, Deserialize)]
struct CfFile {
    #[serde(alias = "projectID", alias = "projectId")]
    project_id: u64,
    #[serde(alias = "fileID", alias = "fileId")]
    file_id: u64,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
#[allow(dead_code)]
struct MrpackManifest {
    #[serde(default)]
    format_version: u32,
    #[serde(default)]
    game: serde_json::Value,
    #[serde(default)]
    version_id: Option<String>,
    #[serde(default)]
    name: Option<String>,
    #[serde(default)]
    summary: Option<String>,
    #[serde(default)]
    files: Vec<MrpackFile>,
    #[serde(default)]
    mods: Vec<MrpackMod>,
    #[serde(default)]
    dependencies: std::collections::HashMap<String, serde_json::Value>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
#[allow(dead_code)]
struct MrpackFile {
    path: String,
    #[serde(default)]
    hashes: std::collections::HashMap<String, String>,
    #[serde(default)]
    env: Option<MrpackEnv>,
    #[serde(default)]
    downloads: Vec<String>,
    #[serde(default)]
    file_size: Option<u64>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
#[allow(dead_code)]
struct MrpackEnv {
    #[serde(default)]
    client: Option<String>,
    #[serde(default)]
    server: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
#[allow(dead_code)]
struct MrpackMod {
    project_id: Option<String>,
    version_id: Option<String>,
    file_name: Option<String>,
    path: Option<String>,
    #[serde(default)]
    env: std::collections::HashMap<String, serde_json::Value>,
}

#[tauri::command]
#[allow(non_snake_case)]
pub async fn instance_import_modpack(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
    file_path: String,
    profile_name: String,
    mc_version: String,
    loader: String,
    icon: Option<String>,
    ram_mb: Option<i64>,
) -> AppResult<ProfileRow> {
    tracing::info!(file_path = %file_path, profile_name = %profile_name, mc_version = %mc_version, loader = %loader, "instance_import_modpack called");
    let file = std::fs::File::open(&file_path).map_err(|e| {
        tracing::error!(file_path = %file_path, error = %e, "failed to open modpack zip");
        e
    })?;
    let file_size = file.metadata().map(|m| m.len()).unwrap_or(0);
    tracing::info!(size = file_size, "modpack zip opened");
    let mut archive = zip::ZipArchive::new(std::io::BufReader::new(file))
        .map_err(|e| {
            tracing::error!(error = %e, "failed to parse zip archive");
            crate::error::AppError::InvalidState(format!("invalid zip: {e}"))
        })?;
    tracing::info!(entries = archive.len(), "zip archive opened");

    let manifest_entry = archive.by_name("manifest.json").map_err(|e| {
        tracing::error!(error = %e, "manifest.json not found in zip");
        crate::error::AppError::NotFound(format!("manifest.json not found in zip: {e}"))
    })?;

    let manifest: CfManifest = serde_json::from_reader(manifest_entry)
        .map_err(|e| {
            tracing::error!(error = %e, "failed to parse manifest.json");
            crate::error::AppError::InvalidState(format!("invalid manifest: {e}"))
        })?;
    tracing::info!(files = manifest.files.len(), "manifest parsed");

    let manifest_mc_version = manifest.minecraft.version.clone();
    let manifest_loader_type = manifest.minecraft.mod_loaders.first().map(|ml| {
        let parts: Vec<&str> = ml.id.split('-').collect();
        parts[0].to_string()
    });
    let loader_version = manifest
        .minecraft
        .mod_loaders
        .first()
        .map(|ml| {
            let parts: Vec<&str> = ml.id.split('-').collect();
            if parts.len() > 1 {
                parts[1..].join("-")
            } else {
                ml.id.clone()
            }
        })
        .unwrap_or_default();
    tracing::info!(manifest_mc_version = %manifest_mc_version, manifest_loader_type = ?manifest_loader_type, loader_version = %loader_version, "manifest loader info extracted");

    let mc_version = if !manifest_mc_version.is_empty() {
        tracing::info!(original = %mc_version, manifest = %manifest_mc_version, "overriding mc_version with manifest version");
        manifest_mc_version
    } else {
        mc_version
    };
    let loader = if let Some(ref lt) = manifest_loader_type {
        tracing::info!(original_loader = %loader, detected = %lt, "using manifest loader type for modpack");
        lt.to_lowercase()
    } else if !loader.is_empty() {
        loader.to_lowercase()
    } else {
        "fabric".to_string()
    };

    let profile_id = Uuid::new_v4().to_string();
    let base_dir = directories::ProjectDirs::from("io", "github", "Luxmc").ok_or_else(|| {
        crate::error::AppError::InvalidState("could not determine data dir".into())
    })?;
    let instance_dir = base_dir.data_dir().join("instances").join(&profile_name);
    let mods_dir = instance_dir.join("mods");
    tokio::fs::create_dir_all(&mods_dir).await?;

    let storage_mods_dir = base_dir.data_dir().join("mods").join(&profile_id);
    let _ = tokio::fs::create_dir_all(&storage_mods_dir).await;
    tracing::info!("directories created, extracting overrides");

    // Extract overrides directory if present
    for i in 0..archive.len() {
        if let Ok(mut file) = archive.by_index(i) {
            let name = file.name().to_string();
            if let Some(rel_path) = name.strip_prefix("overrides/") {
                if !rel_path.is_empty() {
                    let outpath = instance_dir.join(rel_path);
                    if let Ok(canonical_parent) = outpath.parent().unwrap_or(&instance_dir).canonicalize() {
                        if !canonical_parent.starts_with(&instance_dir) {
                            tracing::warn!(path = %rel_path, "skipping override entry: path traversal detected");
                            continue;
                        }
                    }
                    if file.is_dir() {
                        let _ = std::fs::create_dir_all(&outpath);
                    } else {
                        if let Some(p) = outpath.parent() {
                            let _ = std::fs::create_dir_all(p);
                        }
                        if let Ok(mut outfile) = std::fs::File::create(&outpath) {
                            let _ = std::io::copy(&mut file, &mut outfile);
                        }
                    }
                }
            }
        }
    }

    let project_ids: Vec<u64> = manifest.files.iter().map(|f| f.project_id).collect();
    let mod_names = crate::core::mods::curseforge::get_mod_names_batch(&state.http, &project_ids).await;
    tracing::info!(resolved = mod_names.len(), total = project_ids.len(), "resolved mod names from CurseForge");

    let mut mods_ok = 0u32;
    let mut mods_fail = 0u32;
    let total_files = manifest.files.len() as u32;

    let _ = app.emit("modpack-progress", serde_json::json!({
        "phase": "downloading",
        "current": 0,
        "total": total_files,
        "status": format!("Preparando download de {} mods...", total_files)
    }));

    for (idx, cf_file) in manifest.files.iter().enumerate() {
        let project_id_str = cf_file.project_id.to_string();
        let file_id_str = cf_file.file_id.to_string();
        let progress_pct = ((idx as f64 / total_files as f64) * 100.0) as u32;

        let _ = app.emit("modpack-progress", serde_json::json!({
            "phase": "downloading",
            "current": idx + 1,
            "total": total_files,
            "percent": progress_pct,
            "status": format!("Baixando mod {}/{}...", idx + 1, total_files)
        }));

        let download_url = match crate::core::mods::curseforge::get_download_url(
            &state.http,
            &project_id_str,
            &file_id_str,
        ).await {
            Ok(url) => url,
            Err(e) => {
                tracing::warn!(project_id = %project_id_str, error = %e, "skipping mod: could not get download url");
                mods_fail += 1;
                continue;
            }
        };

        let resp = match state.http.get(&download_url).send().await {
            Ok(r) => match r.error_for_status() {
                Ok(r) => r,
                Err(e) => { mods_fail += 1; tracing::warn!(project_id = %project_id_str, error = %e, "skip: HTTP error"); continue; }
            },
            Err(e) => { mods_fail += 1; tracing::warn!(project_id = %project_id_str, error = %e, "skip: request failed"); continue; }
        };

        use futures_util::StreamExt;
        let display_name = mod_names.get(&cf_file.project_id).cloned().unwrap_or_default();
        let filename = if display_name.is_empty() {
            format!("{}_{}.jar", project_id_str, file_id_str)
        } else {
            let safe_name = display_name.chars().map(|c| if c.is_alphanumeric() || c == ' ' || c == '-' || c == '_' { c } else { '_' }).collect::<String>();
            format!("{}.jar", safe_name)
        };
        let file_path = mods_dir.join(&filename);
        let storage_path = storage_mods_dir.join(&filename);

        let mut file = match tokio::fs::File::create(&file_path).await {
            Ok(f) => f,
            Err(e) => { mods_fail += 1; tracing::warn!(error = %e, "skip: cannot create file"); continue; }
        };
        let mut storage_file = tokio::fs::File::create(&storage_path).await.ok();

        let mut stream = resp.bytes_stream();
        let mut total_bytes: u64 = 0;
        while let Some(chunk) = stream.next().await {
            match chunk {
                Ok(bytes) => {
                    total_bytes += bytes.len() as u64;
                    let _ = tokio::io::AsyncWriteExt::write_all(&mut file, &bytes).await;
                    if let Some(ref mut sf) = storage_file {
                        let _ = tokio::io::AsyncWriteExt::write_all(sf, &bytes).await;
                    }
                }
                Err(e) => {
                    tracing::warn!(project_id = %project_id_str, error = %e, "stream error mid-download");
                    break;
                }
            }
        }

        if let Ok(db) = crate::db::shared_db().await {
            let mod_row = crate::db::schema::mods::ModRow {
                profile_id: profile_id.clone(),
                project_id: project_id_str,
                version_id: file_id_str,
                file_name: filename,
                sha1: String::new(),
                source: "curseforge".into(),
                installed_at: String::new(),
            };
            let _ = crate::db::schema::mods::upsert(&db, &mod_row).await;
        }
        mods_ok += 1;
        tracing::debug!(idx = idx + 1, total = total_files, bytes = total_bytes, "mod downloaded");
    }

    let _ = app.emit("modpack-progress", serde_json::json!({
        "phase": "complete",
        "current": total_files,
        "total": total_files,
        "percent": 100,
        "status": format!("{} mods instalados, {} falharam", mods_ok, mods_fail)
    }));
    tracing::info!(mods_ok, mods_fail, total = total_files, "curseforge modpack mod download summary");

    let now = chrono::Utc::now();
    let profile_row = ProfileRow {
        id: profile_id,
        name: profile_name,
        icon: icon.unwrap_or_else(|| "default".into()),
        mc_version,
        loader,
        loader_version: Some(loader_version),
        java_path: None,
        jvm_args: None,
        resolution_w: None,
        resolution_h: None,
        fullscreen: false,
        game_dir: instance_dir.to_string_lossy().to_string(),
        created_at: now,
        updated_at: now,
        favorite: false,
        notes: None,
        last_played: None,
        launch_count: 0,
        mod_count: manifest.files.len() as i64,
        disk_usage: 0,
        ram_mb: Some(ram_mb.unwrap_or(4096)),
        instance_group: None,
        auto_optimize: true,
        use_vulkan: false,
    };

    let db = crate::db::shared_db().await?;
    crate::db::schema::profiles::upsert(&db, &profile_row).await?;
    Ok(profile_row)
}

#[tauri::command]
#[allow(non_snake_case)]
pub async fn instance_health_check(
    _state: State<'_, AppState>,
    profileId: String,
) -> AppResult<HealthCheckResult> {
    let db = crate::db::shared_db().await?;
    let row = sqlx::query_as::<_, ProfileRow>("SELECT * FROM profiles WHERE id = ?")
        .bind(&profileId)
        .fetch_optional(db.pool())
        .await?
        .ok_or_else(|| {
            crate::error::AppError::NotFound(format!("profile {profileId} not found"))
        })?;

    let base_dir = directories::ProjectDirs::from("io", "github", "Luxmc").ok_or_else(|| {
        crate::error::AppError::InvalidState("could not determine data dir".into())
    })?;
    let data_dir = base_dir.data_dir();

    let client_jar = data_dir
        .join("versions")
        .join(&row.mc_version)
        .join(format!("{}.jar", row.mc_version));
    let client_jar_ok = client_jar.exists();

    let natives_dir = data_dir
        .join("versions")
        .join(&row.mc_version)
        .join("natives");
    let natives_ok = natives_dir.exists() || {
        let libs_dir = data_dir.join("libraries");
        if let Ok(entries) = std::fs::read_dir(&libs_dir) {
            let has_natives = entries.filter_map(|e| e.ok()).any(|e| {
                let name = e.file_name().to_string_lossy().to_lowercase();
                name.contains("natives") && name.contains(&row.mc_version)
            });
            has_natives
        } else {
            false
        }
    };

    let mods_dir = data_dir.join("mods").join(&profileId);
    let mods_ok = if mods_dir.exists() {
        let mod_count = std::fs::read_dir(&mods_dir)
            .map(|entries| {
                entries
                    .filter_map(|e| e.ok())
                    .filter(|e| {
                        e.path()
                            .extension()
                            .and_then(|ext| ext.to_str())
                            .map(|ext| ext == "jar")
                            .unwrap_or(false)
                    })
                    .count()
            })
            .unwrap_or(0);
        let db_mod_count = crate::db::schema::mods::list_by_profile(&db, &profileId)
            .await
            .map(|m| m.len())
            .unwrap_or(0);
        mod_count == db_mod_count
    } else {
        true
    };

    let mut issues = Vec::new();
    if !client_jar_ok {
        issues.push(format!("Client JAR missing for {}", row.mc_version));
    }
    if !natives_ok {
        issues.push(format!("Natives not extracted for {}", row.mc_version));
    }
    if !mods_ok {
        issues.push("Mods directory out of sync with database".into());
    }

    Ok(HealthCheckResult {
        client_jar: client_jar_ok,
        natives: natives_ok,
        mods_ok,
        issues,
    })
}

#[tauri::command]
#[allow(non_snake_case)]
pub async fn instance_file_tree(
    _state: State<'_, AppState>,
    profileId: String,
    subPath: Option<String>,
) -> AppResult<Vec<FileTreeEntry>> {
    let db = crate::db::shared_db().await?;
    let row = sqlx::query_as::<_, ProfileRow>("SELECT * FROM profiles WHERE id = ?")
        .bind(&profileId)
        .fetch_optional(db.pool())
        .await?
        .ok_or_else(|| {
            crate::error::AppError::NotFound(format!("profile {profileId} not found"))
        })?;

    let base = if let Some(ref sub) = subPath {
        std::path::PathBuf::from(&row.game_dir).join(sub)
    } else {
        std::path::PathBuf::from(&row.game_dir)
    };

    let mut entries = Vec::new();

    if base.is_dir() {
        for entry in std::fs::read_dir(&base)? {
            let entry = entry?;
            let path = entry.path();
            let metadata = entry.metadata()?;
            let is_dir = metadata.is_dir();
            let size = if is_dir { 0 } else { metadata.len() };

            entries.push(FileTreeEntry {
                name: path
                    .file_name()
                    .map(|n| n.to_string_lossy().to_string())
                    .unwrap_or_default(),
                path: path.to_string_lossy().to_string(),
                is_dir,
                size,
            });
        }
    }

    entries.sort_by(|a, b| {
        if a.is_dir == b.is_dir {
            a.name.to_lowercase().cmp(&b.name.to_lowercase())
        } else if a.is_dir {
            std::cmp::Ordering::Less
        } else {
            std::cmp::Ordering::Greater
        }
    });

    Ok(entries)
}

#[tauri::command]
#[allow(non_snake_case)]
pub async fn instance_import_mrpack(
    state: State<'_, AppState>,
    file_path: String,
    profile_name: String,
    icon: Option<String>,
    ram_mb: Option<i64>,
) -> AppResult<ProfileRow> {
    tracing::info!(file_path = %file_path, profile_name = %profile_name, "instance_import_mrpack called");
    let data = tokio::fs::read(&file_path).await?;
    let cursor = std::io::Cursor::new(data);
    let mut archive = zip::ZipArchive::new(cursor)
        .map_err(|e| crate::error::AppError::InvalidState(format!("invalid zip: {e}")))?;

    let manifest_entry = archive.by_name("modrinth.index.json").map_err(|e| {
        crate::error::AppError::NotFound(format!("modrinth.index.json not found: {e}"))
    })?;

    let manifest: MrpackManifest = serde_json::from_reader(manifest_entry)
        .map_err(|e| crate::error::AppError::InvalidState(format!("invalid mrpack manifest: {e}")))?;

	let mc_version = manifest
		.dependencies
		.get("minecraft")
		.and_then(|v| v.as_str())
		.map(|s| s.to_string())
		.or_else(|| {
			if let Some(s) = manifest.game.as_str() {
				if s != "minecraft" {
					return Some(s.to_string());
				}
			}
			manifest.game.get("version").and_then(|v| v.as_str()).map(|s| s.to_string())
		})
		.unwrap_or_else(|| "1.21.1".to_string());
	let profile_id = Uuid::new_v4().to_string();

	let base_dir = directories::ProjectDirs::from("io", "github", "Luxmc").ok_or_else(|| {
		crate::error::AppError::InvalidState("could not determine data dir".into())
	})?;
	let instance_dir = base_dir.data_dir().join("instances").join(&profile_name);
	let mods_dir = instance_dir.join("mods");
	tokio::fs::create_dir_all(&mods_dir).await?;

	let storage_mods_dir = base_dir.data_dir().join("mods").join(&profile_id);
	let _ = tokio::fs::create_dir_all(&storage_mods_dir).await;

	for i in 0..archive.len() {
		if let Ok(mut file) = archive.by_index(i) {
			let name = file.name().to_string();
			let rel_path = if let Some(rel) = name.strip_prefix("overrides/") {
				Some(rel)
			} else if let Some(rel) = name.strip_prefix("client-overrides/") {
				Some(rel)
			} else {
				None
			};

			if let Some(rel_path) = rel_path {
				if !rel_path.is_empty() {
					let outpath = instance_dir.join(rel_path);
                    if let Ok(canonical_out) = outpath.canonicalize().or_else(|_| {
                        std::path::Path::new(rel_path).parent()
                            .map(|p| instance_dir.join(p))
                            .ok_or_else(|| std::io::Error::new(std::io::ErrorKind::Other, "no parent"))
                            .and_then(|p| std::fs::canonicalize(p))
                    }) {
						if !canonical_out.starts_with(&instance_dir) {
							tracing::warn!(path = %rel_path, "skipping zip entry: path traversal detected");
							continue;
						}
					}
					if file.is_dir() {
						let _ = std::fs::create_dir_all(&outpath);
					} else {
						if let Some(p) = outpath.parent() {
							let _ = std::fs::create_dir_all(p);
						}
						if let Ok(mut outfile) = std::fs::File::create(&outpath) {
							let _ = std::io::copy(&mut file, &mut outfile);
						}
					}
				}
			}
		}
	}

	let client = crate::core::mods::ModrinthClient::new(state.http.clone());
	let db = crate::db::shared_db().await?;
	let mut installed_mods_count = 0;

	for mrpack_file in &manifest.files {
		if let Some(env) = &mrpack_file.env {
			if env.client.as_deref() == Some("unsupported") {
				continue;
			}
		}

		if mrpack_file.downloads.is_empty() {
			continue;
		}

		let outpath = instance_dir.join(&mrpack_file.path);
		if let Some(parent) = outpath.parent() {
			let _ = tokio::fs::create_dir_all(parent).await;
		}

		for url in &mrpack_file.downloads {
			if let Ok(resp) = state.http.get(url).send().await {
				if resp.status().is_success() {
					if let Ok(bytes) = resp.bytes().await {
						let _ = tokio::fs::write(&outpath, &bytes).await;

						if mrpack_file.path.starts_with("mods/") || mrpack_file.path.ends_with(".jar") {
							installed_mods_count += 1;
							let filename = std::path::Path::new(&mrpack_file.path)
								.file_name()
								.and_then(|n| n.to_str())
								.unwrap_or("mod.jar");
							let _ = tokio::fs::write(storage_mods_dir.join(filename), &bytes).await;

							let sha1 = mrpack_file
								.hashes
								.get("sha1")
								.cloned()
								.unwrap_or_default();

							let mod_row = crate::db::schema::mods::ModRow {
								profile_id: profile_id.clone(),
								project_id: filename.to_string(),
								version_id: String::new(),
								file_name: filename.to_string(),
								sha1,
								source: "modrinth".into(),
								installed_at: String::new(),
							};
							let _ = crate::db::schema::mods::upsert(&db, &mod_row).await;
						}
						break;
					}
				}
			}
		}
	}

	if manifest.files.is_empty() {
		for mrpack_mod in &manifest.mods {
			if let (Some(project_id), Some(version_id)) =
				(&mrpack_mod.project_id, &mrpack_mod.version_id)
			{
				let version_detail = client
					.get_version_detail(project_id, version_id, &mc_version)
					.await;
				if let Ok(version) = version_detail {
					if let Some(file) = version.files.first() {
						if let Ok(resp) = state.http.get(&file.url).send().await {
							if resp.status().is_success() {
								if let Ok(bytes) = resp.bytes().await {
									installed_mods_count += 1;
									let filename = mrpack_mod.file_name.as_deref().unwrap_or(&file.filename);
									let file_path = mods_dir.join(filename);
									let _ = tokio::fs::write(&file_path, &bytes).await;
									let _ = tokio::fs::write(storage_mods_dir.join(filename), &bytes).await;

									let mod_row = crate::db::schema::mods::ModRow {
										profile_id: profile_id.clone(),
										project_id: project_id.clone(),
										version_id: version_id.clone(),
										file_name: filename.to_string(),
										sha1: file.sha1.clone(),
										source: "modrinth".into(),
										installed_at: String::new(),
									};
									let _ = crate::db::schema::mods::upsert(&db, &mod_row).await;
								}
							}
						}
					}
				}
			}
		}
	}

	let (loader, loader_version) = if let Some(v) = manifest
		.dependencies
		.get("fabric-loader")
		.and_then(|v| v.as_str())
	{
		("fabric", Some(v.to_string()))
	} else if let Some(v) = manifest
		.dependencies
		.get("neoforge")
		.and_then(|v| v.as_str())
	{
		("neoforge", Some(v.to_string()))
	} else if let Some(v) = manifest
		.dependencies
		.get("forge")
		.and_then(|v| v.as_str())
	{
		("forge", Some(v.to_string()))
	} else if let Some(v) = manifest
		.dependencies
		.get("quilt-loader")
		.and_then(|v| v.as_str())
	{
		("quilt", Some(v.to_string()))
	} else {
		("vanilla", None)
	};

	let now = chrono::Utc::now();
	let profile_row = ProfileRow {
		id: profile_id,
		name: profile_name,
		icon: icon.unwrap_or_else(|| "default".into()),
		mc_version,
		loader: loader.into(),
		loader_version,
		java_path: None,
		jvm_args: None,
		resolution_w: None,
		resolution_h: None,
		fullscreen: false,
		game_dir: instance_dir.to_string_lossy().to_string(),
		created_at: now,
		updated_at: now,
		favorite: false,
		notes: None,
		last_played: None,
		launch_count: 0,
		mod_count: installed_mods_count as i64,
		disk_usage: 0,
		ram_mb: Some(ram_mb.unwrap_or(4096)),
		instance_group: None,
		auto_optimize: true,
		use_vulkan: false,
	};

	crate::db::schema::profiles::upsert(&db, &profile_row).await?;
    Ok(profile_row)
}

#[tauri::command]
#[allow(non_snake_case)]
pub async fn instance_export(
    _state: State<'_, AppState>,
    profileId: String,
    destPath: String,
) -> AppResult<String> {
    let db = crate::db::shared_db().await?;
    let row = sqlx::query_as::<_, ProfileRow>("SELECT * FROM profiles WHERE id = ?")
        .bind(&profileId)
        .fetch_optional(db.pool())
        .await?
        .ok_or_else(|| {
            crate::error::AppError::NotFound(format!("profile {profileId} not found"))
        })?;

    let mods_db = crate::db::schema::mods::list_by_profile(&db, &profileId).await?;

    let manifest = serde_json::json!({
        "name": row.name,
        "mcVersion": row.mc_version,
        "loader": row.loader,
        "loaderVersion": row.loader_version,
        "mods": mods_db.iter().map(|m| serde_json::json!({
            "projectId": m.project_id,
            "versionId": m.version_id,
            "fileName": m.file_name,
            "source": m.source,
        })).collect::<Vec<_>>(),
        "exportedAt": chrono::Utc::now().to_rfc3339(),
        "launcher": "Luxmc",
    });

    let file = std::fs::File::create(&destPath).map_err(|e| {
        crate::error::AppError::InvalidState(format!("cannot create export file: {e}"))
    })?;
    let writer = std::io::BufWriter::new(file);
    serde_json::to_writer_pretty(writer, &manifest)
        .map_err(|e| crate::error::AppError::InvalidState(format!("serialize error: {e}")))?;

    Ok(destPath)
}

#[tauri::command]
#[allow(non_snake_case)]
pub async fn instance_set_notes(
    _state: State<'_, AppState>,
    profileId: String,
    notes: Option<String>,
) -> AppResult<()> {
    let db = crate::db::shared_db().await?;
    let row = sqlx::query_as::<_, ProfileRow>("SELECT * FROM profiles WHERE id = ?")
        .bind(&profileId)
        .fetch_optional(db.pool())
        .await?
        .ok_or_else(|| {
            crate::error::AppError::NotFound(format!("profile {profileId} not found"))
        })?;

    let now = chrono::Utc::now();
    let updated = ProfileRow {
        notes,
        updated_at: now,
        ..row
    };
    crate::db::schema::profiles::upsert(&db, &updated).await?;
    Ok(())
}

#[tauri::command]
#[allow(non_snake_case)]
pub async fn instance_set_favorite(
    _state: State<'_, AppState>,
    profileId: String,
    favorite: bool,
) -> AppResult<()> {
    let db = crate::db::shared_db().await?;
    let row = sqlx::query_as::<_, ProfileRow>("SELECT * FROM profiles WHERE id = ?")
        .bind(&profileId)
        .fetch_optional(db.pool())
        .await?
        .ok_or_else(|| {
            crate::error::AppError::NotFound(format!("profile {profileId} not found"))
        })?;

    let now = chrono::Utc::now();
    let updated = ProfileRow {
        favorite,
        updated_at: now,
        ..row
    };
    crate::db::schema::profiles::upsert(&db, &updated).await?;
    Ok(())
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WorldDetail {
    pub name: String,
    pub folder_name: String,
    pub icon_base64: Option<String>,
    pub last_played: Option<i64>,
    pub game_mode: Option<String>,
    pub size_bytes: u64,
}

#[tauri::command]
#[allow(non_snake_case)]
pub async fn instance_worlds_list(
    _state: State<'_, AppState>,
    profileId: String,
) -> AppResult<Vec<WorldDetail>> {
    let db = crate::db::shared_db().await?;
    let row = sqlx::query_as::<_, ProfileRow>("SELECT * FROM profiles WHERE id = ?")
        .bind(&profileId)
        .fetch_optional(db.pool())
        .await?
        .ok_or_else(|| {
            crate::error::AppError::NotFound(format!("profile {profileId} not found"))
        })?;

    let saves_dir = std::path::PathBuf::from(&row.game_dir).join("saves");
    let mut list = Vec::new();

    if saves_dir.is_dir() {
        if let Ok(entries) = std::fs::read_dir(&saves_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    let folder_name = path
                        .file_name()
                        .map(|n| n.to_string_lossy().to_string())
                        .unwrap_or_default();
                    let world_name = folder_name.clone();

                    let icon_path = path.join("icon.png");
                    let icon_base64 = if icon_path.exists() {
                        if let Ok(bytes) = std::fs::read(&icon_path) {
                            use base64::Engine;
                            Some(format!(
                                "data:image/png;base64,{}",
                                base64::engine::general_purpose::STANDARD.encode(&bytes)
                            ))
                        } else {
                            None
                        }
                    } else {
                        None
                    };

                    let total_size = dir_size_recursive(&path);

                    let last_played = entry
                        .metadata()
                        .ok()
                        .and_then(|m| m.modified().ok())
                        .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
                        .map(|d| d.as_secs() as i64);

                    list.push(WorldDetail {
                        name: world_name,
                        folder_name,
                        icon_base64,
                        last_played,
                        game_mode: Some("Sobrevivência".to_string()),
                        size_bytes: total_size,
                    });
                }
            }
        }
    }

    Ok(list)
}

#[tauri::command]
#[allow(non_snake_case)]
pub async fn instance_world_delete(
    _state: State<'_, AppState>,
    profileId: String,
    folderName: String,
) -> AppResult<()> {
    if folderName.is_empty()
        || folderName.contains("..")
        || folderName.contains('/')
        || folderName.contains('\\')
    {
        return Err(crate::error::AppError::InvalidInput(
            "Invalid folder name".into(),
        ));
    }

    let db = crate::db::shared_db().await?;
    let row = sqlx::query_as::<_, ProfileRow>("SELECT * FROM profiles WHERE id = ?")
        .bind(&profileId)
        .fetch_optional(db.pool())
        .await?
        .ok_or_else(|| {
            crate::error::AppError::NotFound(format!("profile {profileId} not found"))
        })?;

    let world_path = std::path::PathBuf::from(&row.game_dir)
        .join("saves")
        .join(folderName);
    let canonical_world = world_path.canonicalize().map_err(|_| {
        crate::error::AppError::NotFound("World folder not found".to_string())
    })?;
    let game_dir = std::path::PathBuf::from(&row.game_dir);
    let canonical_game_dir = game_dir.canonicalize().unwrap_or(game_dir);
    if !canonical_world.starts_with(&canonical_game_dir) {
        return Err(crate::error::AppError::InvalidInput("Invalid world path".into()));
    }
    if canonical_world.is_dir() {
        std::fs::remove_dir_all(&canonical_world)?;
    }

    Ok(())
}

#[tauri::command]
#[allow(non_snake_case)]
pub async fn instance_mod_toggle(
    _state: State<'_, AppState>,
    profileId: String,
    fileName: String,
    enabled: bool,
) -> AppResult<String> {
    if fileName.is_empty() || fileName.contains("..") || fileName.contains('/') || fileName.contains('\\') {
        return Err(crate::error::AppError::InvalidInput("Invalid file name".into()));
    }

    let db = crate::db::shared_db().await?;
    let row = sqlx::query_as::<_, ProfileRow>("SELECT * FROM profiles WHERE id = ?")
        .bind(&profileId)
        .fetch_optional(db.pool())
        .await?
        .ok_or_else(|| crate::error::AppError::NotFound(format!("profile {profileId} not found")))?;

    let mods_dir = std::path::PathBuf::from(&row.game_dir).join("mods");
    let current_path = mods_dir.join(&fileName);

    let new_name = if enabled {
        if fileName.ends_with(".jar.disabled") {
            fileName.trim_end_matches(".disabled").to_string()
        } else {
            fileName.clone()
        }
    } else {
        if fileName.ends_with(".jar") {
            format!("{}.disabled", fileName)
        } else {
            fileName.clone()
        }
    };

    let target_path = mods_dir.join(&new_name);
    if current_path.exists() && current_path != target_path {
        std::fs::rename(&current_path, &target_path)?;
    }

    Ok(new_name)
}

#[tauri::command]
#[allow(non_snake_case)]
pub async fn instance_mod_delete(
    _state: State<'_, AppState>,
    profileId: String,
    fileName: String,
) -> AppResult<()> {
    if fileName.is_empty() || fileName.contains("..") || fileName.contains('/') || fileName.contains('\\') {
        return Err(crate::error::AppError::InvalidInput("Invalid file name".into()));
    }

    let db = crate::db::shared_db().await?;
    let row = sqlx::query_as::<_, ProfileRow>("SELECT * FROM profiles WHERE id = ?")
        .bind(&profileId)
        .fetch_optional(db.pool())
        .await?
        .ok_or_else(|| crate::error::AppError::NotFound(format!("profile {profileId} not found")))?;

    let mod_path = std::path::PathBuf::from(&row.game_dir).join("mods").join(&fileName);
    let canonical_mod = mod_path.canonicalize().map_err(|_| {
        crate::error::AppError::NotFound("Mod file not found".to_string())
    })?;
    let mods_dir = std::path::PathBuf::from(&row.game_dir).join("mods");
    let canonical_mods_dir = mods_dir.canonicalize().unwrap_or(mods_dir);
    if !canonical_mod.starts_with(&canonical_mods_dir) {
        return Err(crate::error::AppError::InvalidInput("Invalid mod path".into()));
    }
    if canonical_mod.exists() {
        std::fs::remove_file(&canonical_mod)?;
    }

    // Also remove from data_dir/mods/{profileId} if present
    if let Some(base_dir) = directories::ProjectDirs::from("io", "github", "Luxmc") {
        let alt_path = base_dir.data_dir().join("mods").join(&profileId).join(&fileName);
        if alt_path.exists() {
            let _ = std::fs::remove_file(alt_path);
        }
    }

    Ok(())
}

#[tauri::command]
#[allow(non_snake_case)]
pub async fn instance_mod_add(
    _state: State<'_, AppState>,
    profileId: String,
    sourcePath: String,
) -> AppResult<String> {
    let src = std::path::PathBuf::from(&sourcePath);
    if !src.is_file() {
        return Err(crate::error::AppError::NotFound("Source file does not exist".into()));
    }

    let file_name = src
        .file_name()
        .and_then(|n| n.to_str())
        .ok_or_else(|| crate::error::AppError::InvalidInput("Invalid source filename".into()))?
        .to_string();

    let db = crate::db::shared_db().await?;
    let row = sqlx::query_as::<_, ProfileRow>("SELECT * FROM profiles WHERE id = ?")
        .bind(&profileId)
        .fetch_optional(db.pool())
        .await?
        .ok_or_else(|| crate::error::AppError::NotFound(format!("profile {profileId} not found")))?;

    let mods_dir = std::path::PathBuf::from(&row.game_dir).join("mods");
    if !mods_dir.exists() {
        std::fs::create_dir_all(&mods_dir)?;
    }

    let dest = mods_dir.join(&file_name);
    std::fs::copy(&src, &dest)?;

    Ok(file_name)
}

#[tauri::command]
#[allow(non_snake_case)]
pub async fn instance_mods_open_folder(
    _state: State<'_, AppState>,
    profileId: String,
) -> AppResult<()> {
    let db = crate::db::shared_db().await?;
    let row = sqlx::query_as::<_, ProfileRow>("SELECT * FROM profiles WHERE id = ?")
        .bind(&profileId)
        .fetch_optional(db.pool())
        .await?
        .ok_or_else(|| crate::error::AppError::NotFound(format!("profile {profileId} not found")))?;

    let mods_dir = std::path::PathBuf::from(&row.game_dir).join("mods");
    if !mods_dir.exists() {
        std::fs::create_dir_all(&mods_dir)?;
    }

    open::that(&mods_dir)?;
    Ok(())
}

#[tauri::command]
#[allow(non_snake_case)]
pub async fn instance_pack_add(
    _state: State<'_, AppState>,
    profileId: String,
    packType: String,
    sourcePath: String,
) -> AppResult<String> {
    let src = std::path::PathBuf::from(&sourcePath);
    if !src.is_file() {
        return Err(crate::error::AppError::NotFound("Source file does not exist".into()));
    }

    let file_name = src
        .file_name()
        .and_then(|n| n.to_str())
        .ok_or_else(|| crate::error::AppError::InvalidInput("Invalid source filename".into()))?
        .to_string();

    let db = crate::db::shared_db().await?;
    let row = sqlx::query_as::<_, ProfileRow>("SELECT * FROM profiles WHERE id = ?")
        .bind(&profileId)
        .fetch_optional(db.pool())
        .await?
        .ok_or_else(|| crate::error::AppError::NotFound(format!("profile {profileId} not found")))?;

    let folder_name = match packType.as_str() {
        "shaders" | "shaderpacks" => "shaderpacks",
        "datapacks" => "datapacks",
        _ => "resourcepacks",
    };

    let target_dir = std::path::PathBuf::from(&row.game_dir).join(folder_name);
    if !target_dir.exists() {
        std::fs::create_dir_all(&target_dir)?;
    }

    let dest = target_dir.join(&file_name);
    std::fs::copy(&src, &dest)?;

    Ok(file_name)
}

#[tauri::command]
#[allow(non_snake_case)]
pub async fn instance_pack_delete(
    _state: State<'_, AppState>,
    profileId: String,
    packType: String,
    fileName: String,
) -> AppResult<()> {
    let db = crate::db::shared_db().await?;
    let row = sqlx::query_as::<_, ProfileRow>("SELECT * FROM profiles WHERE id = ?")
        .bind(&profileId)
        .fetch_optional(db.pool())
        .await?
        .ok_or_else(|| crate::error::AppError::NotFound(format!("profile {profileId} not found")))?;

    let folder_name = match packType.as_str() {
        "shaders" | "shaderpacks" => "shaderpacks",
        "datapacks" => "datapacks",
        _ => "resourcepacks",
    };

    let target_path = std::path::PathBuf::from(&row.game_dir).join(folder_name).join(&fileName);
    if target_path.exists() {
        std::fs::remove_file(&target_path)?;
    }

    Ok(())
}

#[tauri::command]
#[allow(non_snake_case)]
pub async fn instance_pack_open_folder(
    _state: State<'_, AppState>,
    profileId: String,
    packType: String,
) -> AppResult<()> {
    let db = crate::db::shared_db().await?;
    let row = sqlx::query_as::<_, ProfileRow>("SELECT * FROM profiles WHERE id = ?")
        .bind(&profileId)
        .fetch_optional(db.pool())
        .await?
        .ok_or_else(|| crate::error::AppError::NotFound(format!("profile {profileId} not found")))?;

    let folder_name = match packType.as_str() {
        "shaders" | "shaderpacks" => "shaderpacks",
        "datapacks" => "datapacks",
        _ => "resourcepacks",
    };

    let target_dir = std::path::PathBuf::from(&row.game_dir).join(folder_name);
    if !target_dir.exists() {
        std::fs::create_dir_all(&target_dir)?;
    }

    open::that(&target_dir)?;
    Ok(())
}
