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

#[cfg(target_os = "linux")]
fn scrub_appimage_env(cmd: &mut std::process::Command) {
    cmd.env_remove("LD_LIBRARY_PATH");
    cmd.env_remove("LD_PRELOAD");
    cmd.env_remove("APPDIR");
    cmd.env_remove("APPIMAGE");
    cmd.env_remove("OWD");
    cmd.env_remove("PYTHONPATH");
    cmd.env_remove("GIO_MODULE_DIR");
    cmd.env_remove("GSETTINGS_SCHEMA_DIR");
    cmd.env_remove("GTK_PATH");
    cmd.env_remove("GTK_EXE_PREFIX");
    cmd.env_remove("QT_PLUGIN_PATH");

    if let Ok(orig_ld) = std::env::var("LD_LIBRARY_PATH_ORIG") {
        cmd.env("LD_LIBRARY_PATH", orig_ld);
    }
    if let Ok(orig_xdg) = std::env::var("XDG_DATA_DIRS_ORIG") {
        cmd.env("XDG_DATA_DIRS", orig_xdg);
    }
}

pub fn open_folder_safe(path: &std::path::Path) -> AppResult<()> {
    if !path.exists() {
        let _ = std::fs::create_dir_all(path);
    }

    #[cfg(target_os = "linux")]
    {
        let mut gio_cmd = std::process::Command::new("gio");
        gio_cmd.args(["open", &path.to_string_lossy()]);
        scrub_appimage_env(&mut gio_cmd);
        if let Ok(mut child) = gio_cmd.spawn() {
            tokio::spawn(async move {
                let _ = child.wait();
            });
            return Ok(());
        }

        let mut xdg_cmd = std::process::Command::new("xdg-open");
        xdg_cmd.arg(path);
        scrub_appimage_env(&mut xdg_cmd);
        if let Ok(mut child) = xdg_cmd.spawn() {
            tokio::spawn(async move {
                let _ = child.wait();
            });
            return Ok(());
        }

        for fm in &["nautilus", "dolphin", "thunar", "nemo", "pcmanfm"] {
            let mut fm_cmd = std::process::Command::new(fm);
            fm_cmd.arg(path);
            scrub_appimage_env(&mut fm_cmd);
            if let Ok(mut child) = fm_cmd.spawn() {
                tokio::spawn(async move {
                    let _ = child.wait();
                });
                return Ok(());
            }
        }
    }

    open::that_detached(path).map_err(crate::error::AppError::Io)?;
    Ok(())
}

pub fn open_url_safe(url: &str) -> AppResult<()> {
    #[cfg(target_os = "linux")]
    {
        let mut gio_cmd = std::process::Command::new("gio");
        gio_cmd.args(["open", url]);
        scrub_appimage_env(&mut gio_cmd);
        if let Ok(mut child) = gio_cmd.spawn() {
            tokio::spawn(async move {
                let _ = child.wait();
            });
            return Ok(());
        }

        let mut xdg_cmd = std::process::Command::new("xdg-open");
        xdg_cmd.arg(url);
        scrub_appimage_env(&mut xdg_cmd);
        if let Ok(mut child) = xdg_cmd.spawn() {
            tokio::spawn(async move {
                let _ = child.wait();
            });
            return Ok(());
        }
    }

    open::that_detached(url).map_err(crate::error::AppError::Io)?;
    Ok(())
}

#[tauri::command]
pub async fn instances_open_folder(_state: State<'_, AppState>, id: String) -> AppResult<()> {
    let db = crate::db::shared_db().await?;
    let row = sqlx::query_as::<_, ProfileRow>("SELECT * FROM profiles WHERE id = ?")
        .bind(&id)
        .fetch_optional(db.pool())
        .await?
        .ok_or_else(|| crate::error::AppError::NotFound(format!("profile {id} not found")))?;

    let path = std::path::PathBuf::from(&row.game_dir);
    open_folder_safe(&path)?;
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
    if !dir.exists() {
        return Ok(Vec::new());
    }

    let mut entries = Vec::new();

    if dir.is_dir() {
        if let Ok(read_dir) = std::fs::read_dir(&dir) {
            for entry in read_dir.flatten() {
                let path = entry.path();

                let is_image = path
                    .extension()
                    .and_then(|e| e.to_str())
                    .map(|e| matches!(e.to_lowercase().as_str(), "png" | "jpg" | "jpeg"))
                    .unwrap_or(false);

                if !is_image {
                    continue;
                }

                let modified_time = entry
                    .metadata()
                    .ok()
                    .and_then(|m| m.modified().ok());

                let modified_rfc3339 = modified_time
                    .map(|t| {
                        let dt: chrono::DateTime<chrono::Utc> = t.into();
                        dt.to_rfc3339()
                    })
                    .unwrap_or_default();

                entries.push((
                    modified_time,
                    ScreenshotEntry {
                        name: path
                            .file_name()
                            .and_then(|n| n.to_str())
                            .unwrap_or("")
                            .to_string(),
                        path: path.to_string_lossy().to_string(),
                        modified: modified_rfc3339,
                        data_url: None,
                    },
                ));
            }
        }
    }

    // Sort newest first
    entries.sort_by(|a, b| b.0.cmp(&a.0));

    // Provide lightweight data_url fallback only for the first 6 screenshots if <= 1.5MB
    use base64::Engine;
    let result: Vec<ScreenshotEntry> = entries
        .into_iter()
        .enumerate()
        .map(|(idx, (_, mut entry))| {
            if idx < 6 {
                if let Ok(bytes) = std::fs::read(&entry.path) {
                    if bytes.len() <= 1500 * 1024 {
                        let ext = std::path::Path::new(&entry.path)
                            .extension()
                            .and_then(|e| e.to_str())
                            .unwrap_or("png")
                            .to_lowercase();
                        let mime = if ext == "jpg" || ext == "jpeg" {
                            "image/jpeg"
                        } else {
                            "image/png"
                        };
                        entry.data_url = Some(format!(
                            "data:{};base64,{}",
                            mime,
                            base64::prelude::BASE64_STANDARD.encode(&bytes)
                        ));
                    }
                }
            }
            entry
        })
        .collect();

    Ok(result)
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
pub async fn screenshots_open_folder(_app: tauri::AppHandle, profile_id: String) -> AppResult<()> {
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
    open_folder_safe(&dir)?;
    Ok(())
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct CfManifest {
    #[serde(default)]
    minecraft: CfMinecraft,
    #[serde(default)]
    files: Vec<CfFile>,
    #[serde(default)]
    overrides: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
struct CfMinecraft {
    #[serde(default)]
    version: String,
    #[serde(default, alias = "modLoaders")]
    mod_loaders: Vec<CfModLoader>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct CfModLoader {
    #[serde(default)]
    id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
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

fn smart_modpack_ram(user_ram: Option<i64>) -> i64 {
    if let Some(r) = user_ram.filter(|r| *r > 0) {
        return r;
    }
    let mut sys = sysinfo::System::new_all();
    sys.refresh_memory();
    let total = sys.total_memory() / 1024 / 1024;
    if total >= 24576 {
        10240
    } else if total >= 15360 {
        8192
    } else if total >= 11264 {
        6144
    } else {
        std::cmp::min(std::cmp::max(total * 6 / 10, 4096), 6144) as i64
    }
}

#[tauri::command]
pub async fn instance_cancel_import(state: State<'_, AppState>) -> AppResult<()> {
    state.import_cancel.store(true, std::sync::atomic::Ordering::SeqCst);
    Ok(())
}

async fn download_cf_mod_file(
    http: &reqwest::Client,
    cf_file: &CfFile,
    file_info: Option<&crate::core::mods::curseforge::CurseForgeFileInfo>,
    display_name: Option<&str>,
    mods_dir: &std::path::Path,
    storage_mods_dir: &std::path::Path,
    profile_id: &str,
) -> bool {
    let project_id_str = cf_file.project_id.to_string();
    let file_id_str = cf_file.file_id.to_string();

    let mut urls_to_try: Vec<String> = Vec::new();

    if let Some(info) = file_info {
        if let Some(ref dl) = info.download_url {
            if !dl.is_empty() && !urls_to_try.contains(dl) {
                urls_to_try.push(dl.clone());
            }
        }
    }

    if urls_to_try.is_empty() {
        if let Ok(url) = crate::core::mods::curseforge::get_download_url(http, &project_id_str, &file_id_str).await {
            if !url.is_empty() && !urls_to_try.contains(&url) {
                urls_to_try.push(url);
            }
        }
    }

    let p1 = cf_file.file_id / 1000;
    let p2 = cf_file.file_id % 1000;

    if let Some(info) = file_info {
        let edge = format!("https://edge.forgecdn.net/files/{}/{}/{}", p1, p2, urlencoding::encode(&info.file_name));
        if !urls_to_try.contains(&edge) { urls_to_try.push(edge); }
        let media = format!("https://mediafilez.forgecdn.net/files/{}/{}/{}", p1, p2, urlencoding::encode(&info.file_name));
        if !urls_to_try.contains(&media) { urls_to_try.push(media); }
    }

    if let Some(name) = display_name {
        let guess = format!("https://edge.forgecdn.net/files/{}/{}/{}.jar", p1, p2, urlencoding::encode(name));
        if !urls_to_try.contains(&guess) { urls_to_try.push(guess); }
    }

    if urls_to_try.is_empty() {
        if let Ok(details) = crate::core::mods::curseforge::get_mod_file_details(http, &project_id_str, &file_id_str).await {
            if !details.url.is_empty() && !urls_to_try.contains(&details.url) {
                urls_to_try.push(details.url);
            }
        }
    }

    if urls_to_try.is_empty() {
        return false;
    }

    let filename = if let Some(info) = file_info {
        info.file_name.clone()
    } else if let Some(name) = display_name {
        let safe = name.chars().map(|c| if c.is_alphanumeric() || c == ' ' || c == '-' || c == '_' { c } else { '_' }).collect::<String>();
        format!("{}.jar", safe)
    } else {
        format!("{}_{}.jar", project_id_str, file_id_str)
    };

    let file_path = mods_dir.join(&filename);
    let storage_path = storage_mods_dir.join(&filename);

    if file_path.exists() {
        if let Ok(f) = std::fs::File::open(&file_path) {
            if zip::ZipArchive::new(std::io::BufReader::new(f)).is_ok() {
                return true;
            }
        }
    }

    for try_url in &urls_to_try {
        for _attempt in 0..2 {
            let resp = match http.get(try_url).timeout(std::time::Duration::from_secs(30)).send().await {
                Ok(r) if r.status().is_success() => r,
                _ => continue,
            };

            let bytes = match resp.bytes().await {
                Ok(b) if b.len() > 100 => b,
                _ => continue,
            };

            let is_valid = zip::ZipArchive::new(std::io::Cursor::new(&bytes)).is_ok();
            if !is_valid {
                continue;
            }

            if tokio::fs::write(&file_path, &bytes).await.is_err() {
                continue;
            }
            let _ = tokio::fs::write(&storage_path, &bytes).await;

            let mut final_filename = filename.clone();
            if final_filename.starts_with(&project_id_str) {
                if let Some(real_name) = crate::commands::mods::extract_mod_name_from_jar(&file_path) {
                    let safe = real_name.replace(['/', '\\', ':', '*', '?', '"', '<', '>', '|'], "_");
                    let new_filename = format!("{}.jar", safe);
                    let new_path = mods_dir.join(&new_filename);
                    let new_storage_path = storage_mods_dir.join(&new_filename);
                    if new_path != file_path {
                        let _ = tokio::fs::rename(&file_path, &new_path).await;
                        let _ = tokio::fs::rename(&storage_path, &new_storage_path).await;
                        final_filename = new_filename;
                    }
                }
            }

            if let Ok(db) = crate::db::shared_db().await {
                let mod_row = crate::db::schema::mods::ModRow {
                    profile_id: profile_id.to_string(),
                    project_id: project_id_str.clone(),
                    version_id: file_id_str.clone(),
                    file_name: final_filename,
                    sha1: String::new(),
                    source: "curseforge".into(),
                    installed_at: String::new(),
                };
                let _ = crate::db::schema::mods::upsert(&db, &mod_row).await;
            }

            return true;
        }
    }

    false
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
    state.import_cancel.store(false, std::sync::atomic::Ordering::SeqCst);
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

    let mut manifest_idx = None;
    let mut root_prefix = String::new();

    for i in 0..archive.len() {
        if let Ok(file) = archive.by_index(i) {
            let name = file.name().replace('\\', "/");
            let clean = name.trim_start_matches('/');
            if clean == "manifest.json" {
                manifest_idx = Some(i);
                root_prefix = String::new();
                break;
            } else if clean.ends_with("/manifest.json") && clean.matches('/').count() == 1 {
                if let Some(prefix) = clean.strip_suffix("manifest.json") {
                    manifest_idx = Some(i);
                    root_prefix = prefix.to_string();
                }
            }
        }
    }

    let manifest_idx = manifest_idx.ok_or_else(|| {
        tracing::error!("manifest.json not found in zip");
        crate::error::AppError::NotFound("manifest.json not found in zip".into())
    })?;

    let manifest: CfManifest = {
        let entry = archive.by_index(manifest_idx).map_err(|e| {
            tracing::error!(error = %e, "failed to read manifest entry");
            crate::error::AppError::InvalidState(format!("failed to read manifest entry: {e}"))
        })?;
        serde_json::from_reader(entry).map_err(|e| {
            tracing::error!(error = %e, "failed to parse manifest.json");
            crate::error::AppError::InvalidState(format!("invalid manifest: {e}"))
        })?
    };
    tracing::info!(files = manifest.files.len(), prefix = %root_prefix, "manifest parsed");

    let manifest_mc_version = manifest
        .minecraft
        .version
        .trim()
        .trim_matches('\'')
        .trim_matches('"')
        .to_string();
    let manifest_loader_type = manifest.minecraft.mod_loaders.first().map(|ml| {
        let parts: Vec<&str> = ml.id.split('-').collect();
        parts[0].trim().to_lowercase()
    });
    let loader_version = manifest
        .minecraft
        .mod_loaders
        .first()
        .map(|ml| {
            let parts: Vec<&str> = ml.id.split('-').collect();
            if parts.len() > 1 {
                parts[1..].join("-").trim().to_string()
            } else {
                ml.id.trim().to_string()
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
    let _ = tokio::fs::create_dir_all(&instance_dir).await;
    let _ = tokio::fs::write(instance_dir.join("manifest.json"), serde_json::to_vec_pretty(&manifest).unwrap_or_default()).await;
    let mods_dir = instance_dir.join("mods");
    tokio::fs::create_dir_all(&mods_dir).await?;

    let storage_mods_dir = base_dir.data_dir().join("mods").join(&profile_id);
    let _ = tokio::fs::create_dir_all(&storage_mods_dir).await;
    tracing::info!("directories created, extracting overrides");

    // Extract overrides directory if present
    let overrides_folder = manifest
        .overrides
        .as_deref()
        .unwrap_or("overrides")
        .trim_matches('/')
        .trim_matches('\\');

    let full_overrides_prefix = format!("{}{}/", root_prefix, overrides_folder).to_lowercase();
    let fallback_overrides_prefix = format!("{}overrides/", root_prefix).to_lowercase();
    let fallback_client_prefix = format!("{}client-overrides/", root_prefix).to_lowercase();
    let fallback_client_dir = format!("{}client/", root_prefix).to_lowercase();

    for i in 0..archive.len() {
        if let Ok(mut file) = archive.by_index(i) {
            let raw_name = file.name().replace('\\', "/");
            let clean_name = raw_name.trim_start_matches('/');
            let lower_name = clean_name.to_lowercase();

            let rel_str = if lower_name.starts_with(&full_overrides_prefix) {
                clean_name.get(full_overrides_prefix.len()..)
            } else if lower_name.starts_with(&fallback_overrides_prefix) {
                clean_name.get(fallback_overrides_prefix.len()..)
            } else if lower_name.starts_with(&fallback_client_prefix) {
                clean_name.get(fallback_client_prefix.len()..)
            } else if lower_name.starts_with(&fallback_client_dir) {
                clean_name.get(fallback_client_dir.len()..)
            } else {
                None
            };

            if let Some(rel) = rel_str {
                let rel_path = std::path::Path::new(rel);
                if rel.is_empty()
                    || rel_path.is_absolute()
                    || rel.contains("..")
                    || rel.contains(':')
                {
                    continue;
                }
                let outpath = instance_dir.join(rel_path);
                if file.is_dir() || clean_name.ends_with('/') {
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

    let project_ids: Vec<u64> = manifest.files.iter().map(|f| f.project_id).collect();
    let file_ids: Vec<u64> = manifest.files.iter().map(|f| f.file_id).collect();
    let mod_names = crate::core::mods::curseforge::get_mod_names_batch(&state.http, &project_ids).await;
    let file_infos = crate::core::mods::curseforge::get_files_batch(&state.http, &file_ids).await;
    tracing::info!(resolved_names = mod_names.len(), resolved_files = file_infos.len(), total = project_ids.len(), "resolved CurseForge batch metadata");

    let total_files = manifest.files.len() as u32;

    let _ = app.emit("modpack-progress", serde_json::json!({
        "phase": "downloading",
        "current": 0,
        "total": total_files,
        "percent": 0,
        "status": format!("Preparando download concorrente de {} mods...", total_files)
    }));

    use futures_util::StreamExt;
    let completed_count = std::sync::Arc::new(std::sync::atomic::AtomicU32::new(0));
    let mods_ok = std::sync::Arc::new(std::sync::atomic::AtomicU32::new(0));
    let mods_fail = std::sync::Arc::new(std::sync::atomic::AtomicU32::new(0));

    let files_stream = futures_util::stream::iter(manifest.files.clone()).map(|cf_file| {
        let http = state.http.clone();
        let cancel = state.import_cancel.clone();
        let file_info = file_infos.get(&cf_file.file_id).cloned();
        let display_name = mod_names.get(&cf_file.project_id).cloned();
        let mods_dir = mods_dir.clone();
        let storage_mods_dir = storage_mods_dir.clone();
        let profile_id = profile_id.clone();
        let app = app.clone();
        let completed = completed_count.clone();
        let ok_counter = mods_ok.clone();
        let fail_counter = mods_fail.clone();

        async move {
            if cancel.load(std::sync::atomic::Ordering::SeqCst) {
                return;
            }

            let ok = download_cf_mod_file(
                &http,
                &cf_file,
                file_info.as_ref(),
                display_name.as_deref(),
                &mods_dir,
                &storage_mods_dir,
                &profile_id,
            ).await;

            if ok {
                ok_counter.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
            } else {
                fail_counter.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
            }

            let current = completed.fetch_add(1, std::sync::atomic::Ordering::SeqCst) + 1;
            let percent = ((current as f64 / total_files as f64) * 100.0) as u32;

            let _ = app.emit("modpack-progress", serde_json::json!({
                "phase": "downloading",
                "current": current,
                "total": total_files,
                "percent": percent,
                "status": format!("Baixando mods ({}/{})...", current, total_files)
            }));
        }
    });

    files_stream.buffer_unordered(8).collect::<Vec<()>>().await;

    let final_ok = mods_ok.load(std::sync::atomic::Ordering::SeqCst);
    let final_fail = mods_fail.load(std::sync::atomic::Ordering::SeqCst);

    let _ = app.emit("modpack-progress", serde_json::json!({
        "phase": "complete",
        "current": total_files,
        "total": total_files,
        "percent": 100,
        "status": format!("{} mods instalados, {} falharam", final_ok, final_fail)
    }));
    tracing::info!(mods_ok = final_ok, mods_fail = final_fail, total = total_files, "curseforge modpack mod download summary");

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
        ram_mb: Some(smart_modpack_ram(ram_mb)),
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
pub async fn instance_repair_modpack(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
    profileId: String,
) -> AppResult<u32> {
    let db = crate::db::shared_db().await?;
    let row = sqlx::query_as::<_, ProfileRow>("SELECT * FROM profiles WHERE id = ?")
        .bind(&profileId)
        .fetch_optional(db.pool())
        .await?
        .ok_or_else(|| crate::error::AppError::NotFound(format!("profile {profileId} not found")))?;

    let instance_dir = std::path::PathBuf::from(&row.game_dir);
    let manifest_path = instance_dir.join("manifest.json");

    if !manifest_path.exists() {
        return Err(crate::error::AppError::NotFound(
            "Arquivo manifest.json do modpack não foi encontrado nesta instância.".into(),
        ));
    }

    let manifest_content = tokio::fs::read_to_string(&manifest_path).await?;
    let manifest: CfManifest = serde_json::from_str(&manifest_content).map_err(|e| {
        crate::error::AppError::InvalidState(format!("manifest.json inválido: {e}"))
    })?;

    let mods_dir = instance_dir.join("mods");
    tokio::fs::create_dir_all(&mods_dir).await?;

    let base_dir = directories::ProjectDirs::from("io", "github", "Luxmc").ok_or_else(|| {
        crate::error::AppError::InvalidState("could not determine data dir".into())
    })?;
    let storage_mods_dir = base_dir.data_dir().join("mods").join(&profileId);
    let _ = tokio::fs::create_dir_all(&storage_mods_dir).await;

    let mut existing_files = std::collections::HashSet::new();
    if let Ok(mut entries) = tokio::fs::read_dir(&mods_dir).await {
        while let Ok(Some(entry)) = entries.next_entry().await {
            let name = entry.file_name().to_string_lossy().to_string();
            if name.ends_with(".jar") || name.ends_with(".jar.disabled") {
                if let Ok(meta) = entry.metadata().await {
                    if meta.len() > 100 {
                        existing_files.insert(name);
                    }
                }
            }
        }
    }

    let project_ids: Vec<u64> = manifest.files.iter().map(|f| f.project_id).collect();
    let file_ids: Vec<u64> = manifest.files.iter().map(|f| f.file_id).collect();
    let mod_names = crate::core::mods::curseforge::get_mod_names_batch(&state.http, &project_ids).await;
    let file_infos = crate::core::mods::curseforge::get_files_batch(&state.http, &file_ids).await;

    let mut missing_files = Vec::new();
    for cf_file in &manifest.files {
        let file_info = file_infos.get(&cf_file.file_id);
        let project_id_str = cf_file.project_id.to_string();
        let file_id_str = cf_file.file_id.to_string();

        let filename = if let Some(info) = file_info {
            info.file_name.clone()
        } else if let Some(display_name) = mod_names.get(&cf_file.project_id) {
            let safe = display_name.chars().map(|c| if c.is_alphanumeric() || c == ' ' || c == '-' || c == '_' { c } else { '_' }).collect::<String>();
            format!("{}.jar", safe)
        } else {
            format!("{}_{}.jar", project_id_str, file_id_str)
        };

        let exists = existing_files.contains(&filename)
            || existing_files.contains(&format!("{}.disabled", filename))
            || existing_files.iter().any(|f| f.contains(&format!("{}_{}", project_id_str, file_id_str)));

        if !exists {
            missing_files.push(cf_file.clone());
        }
    }

    if missing_files.is_empty() {
        return Ok(0);
    }

    let total_missing = missing_files.len() as u32;
    let completed_count = std::sync::Arc::new(std::sync::atomic::AtomicU32::new(0));
    let repaired_count = std::sync::Arc::new(std::sync::atomic::AtomicU32::new(0));

    use futures_util::StreamExt;
    let stream = futures_util::stream::iter(missing_files).map(|cf_file| {
        let http = state.http.clone();
        let file_info = file_infos.get(&cf_file.file_id).cloned();
        let display_name = mod_names.get(&cf_file.project_id).cloned();
        let mods_dir = mods_dir.clone();
        let storage_mods_dir = storage_mods_dir.clone();
        let profile_id = profileId.clone();
        let app = app.clone();
        let completed = completed_count.clone();
        let repaired = repaired_count.clone();

        async move {
            let ok = download_cf_mod_file(
                &http,
                &cf_file,
                file_info.as_ref(),
                display_name.as_deref(),
                &mods_dir,
                &storage_mods_dir,
                &profile_id,
            ).await;

            if ok {
                repaired.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
            }

            let cur = completed.fetch_add(1, std::sync::atomic::Ordering::SeqCst) + 1;
            let percent = ((cur as f64 / total_missing as f64) * 100.0) as u32;
            let _ = app.emit("modpack-repair-progress", serde_json::json!({
                "current": cur,
                "total": total_missing,
                "percent": percent,
                "status": format!("Baixando mod ausente {}/{}...", cur, total_missing)
            }));
        }
    });

    stream.buffer_unordered(8).collect::<Vec<()>>().await;
    let total_repaired = repaired_count.load(std::sync::atomic::Ordering::SeqCst);

    let count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM mods WHERE profile_id = ?")
        .bind(&profileId)
        .fetch_one(db.pool())
        .await
        .unwrap_or((0,));
    let _ = sqlx::query("UPDATE profiles SET mod_count = ? WHERE id = ?")
        .bind(count.0)
        .bind(&profileId)
        .execute(db.pool())
        .await;

    Ok(total_repaired)
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
    app: tauri::AppHandle,
    state: State<'_, AppState>,
    file_path: String,
    profile_name: String,
    icon: Option<String>,
    ram_mb: Option<i64>,
) -> AppResult<ProfileRow> {
    tracing::info!(file_path = %file_path, profile_name = %profile_name, "instance_import_mrpack called");
    let file = std::fs::File::open(&file_path).map_err(|e| {
        tracing::error!(file_path = %file_path, error = %e, "failed to open mrpack zip");
        e
    })?;
    let mut archive = zip::ZipArchive::new(std::io::BufReader::new(file))
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
			let raw_name = file.name().replace('\\', "/");
			let clean_name = raw_name.trim_start_matches('/');
			let rel_str = if clean_name.to_lowercase().starts_with("overrides/") {
				clean_name.split_once('/').map(|x| x.1)
			} else if clean_name.to_lowercase().starts_with("client-overrides/") {
				clean_name.split_once('/').map(|x| x.1)
			} else {
				None
			};

			if let Some(rel) = rel_str {
				let rel_path = std::path::Path::new(rel);
				if rel.is_empty()
					|| rel_path.is_absolute()
					|| rel.contains("..")
					|| rel.contains(':')
				{
					continue;
				}
				let outpath = instance_dir.join(rel_path);
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

	let client = crate::core::mods::ModrinthClient::new(state.http.clone());
	let db = crate::db::shared_db().await?;
	let mut installed_mods_count = 0;

	let total_mrpack_files = manifest.files.len() as u32;
	let _ = app.emit("modpack-progress", serde_json::json!({
		"phase": "downloading",
		"current": 0,
		"total": total_mrpack_files,
		"status": format!("Preparando download de {} arquivos...", total_mrpack_files)
	}));

	for (idx, mrpack_file) in manifest.files.iter().enumerate() {
		let progress_pct = if total_mrpack_files > 0 {
			((idx as f64 / total_mrpack_files as f64) * 100.0) as u32
		} else {
			0
		};
		let _ = app.emit("modpack-progress", serde_json::json!({
			"phase": "downloading",
			"current": idx + 1,
			"total": total_mrpack_files,
			"percent": progress_pct,
			"status": format!("Baixando arquivo {}/{}...", idx + 1, total_mrpack_files)
		}));

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
		ram_mb: Some(smart_modpack_ram(ram_mb)),
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
    open_folder_safe(&mods_dir)?;
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
    open_folder_safe(&target_dir)?;
    Ok(())
}
