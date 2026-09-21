use serde::{Deserialize, Serialize};
use futures_util::StreamExt;
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
    pub icon: Option<String>,
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
pub async fn instances_list() -> AppResult<Vec<ProfileRow>> {
    let db = crate::db::shared_db().await?;
    crate::db::schema::profiles::list(&db).await
}

#[tauri::command]
pub async fn instances_duplicate(id: String) -> AppResult<ProfileRow> {
    let db = crate::db::shared_db().await?;
    let existing = sqlx::query_as::<_, ProfileRow>("SELECT * FROM profiles WHERE id = ?")
        .bind(&id)
        .fetch_optional(db.pool())
        .await?
        .ok_or_else(|| crate::error::AppError::NotFound(format!("profile {id} not found")))?;

    let now = chrono::Utc::now();
    let new_name = format!("{} (copy)", existing.name);

    let base_instances_dir = directories::ProjectDirs::from("io", "github", "Luxmc")
        .map(|d| d.data_dir().join("instances"))
        .unwrap_or_else(|| {
            let home = std::env::var("HOME").unwrap_or_else(|_| "/tmp".into());
            std::path::PathBuf::from(format!("{}/.local/share/luxmc/instances", home))
        });

    let target_game_dir = base_instances_dir.join(&new_name);
    let target_dir_str = target_game_dir.to_string_lossy().to_string();

    let old_dir = std::path::PathBuf::from(&existing.game_dir);
    if old_dir.exists() && old_dir.is_dir() {
        let _ = tokio::fs::create_dir_all(&target_game_dir).await;
        let mut copy_options = fs_extra::dir::CopyOptions::new();
        copy_options.content_only = true;
        copy_options.overwrite = true;
        let _ = fs_extra::dir::copy(&old_dir, &target_game_dir, &copy_options);
    }

    let row = ProfileRow {
        id: Uuid::new_v4().to_string(),
        name: new_name,
        icon: existing.icon,
        mc_version: existing.mc_version,
        loader: existing.loader,
        loader_version: existing.loader_version,
        java_path: existing.java_path,
        jvm_args: existing.jvm_args,
        resolution_w: existing.resolution_w,
        resolution_h: existing.resolution_h,
        fullscreen: existing.fullscreen,
        game_dir: target_dir_str,
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

    #[cfg(target_os = "windows")]
    {
        use std::os::windows::process::CommandExt;
        let mut cmd = std::process::Command::new("explorer.exe");
        cmd.arg(path);
        cmd.creation_flags(0x08000000);
        if let Ok(mut child) = cmd.spawn() {
            tokio::spawn(async move {
                let _ = child.wait();
            });
            return Ok(());
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

    #[cfg(target_os = "windows")]
    {
        use std::os::windows::process::CommandExt;
        let mut cmd = std::process::Command::new("rundll32.exe");
        cmd.args(["url.dll,FileProtocolHandler", url]);
        cmd.creation_flags(0x08000000);
        if let Ok(mut child) = cmd.spawn() {
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
pub async fn instances_open_folder(id: String) -> AppResult<()> {
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
pub async fn screenshots_open_folder(profile_id: String) -> AppResult<()> {
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
pub(crate) struct CfManifest {
    #[serde(flatten)]
    pub(crate) extra: std::collections::HashMap<String, serde_json::Value>,
    #[serde(default)]
    pub(crate) minecraft: CfMinecraft,
    #[serde(default)]
    pub(crate) files: Vec<CfFile>,
    #[serde(default)]
    pub(crate) overrides: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub(crate) struct CfMinecraft {
    #[serde(default)]
    pub(crate) version: String,
    #[serde(default, rename = "modLoaders", alias = "mod_loaders")]
    pub(crate) mod_loaders: Vec<CfModLoader>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct CfModLoader {
    #[serde(default)]
    pub(crate) primary: bool,
    #[serde(default)]
    pub(crate) id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct CfFile {
    #[serde(rename = "projectID", alias = "projectId", alias = "project_id")]
    pub(crate) project_id: u64,
    #[serde(rename = "fileID", alias = "fileId", alias = "file_id")]
    pub(crate) file_id: u64,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[allow(dead_code)]
struct MrpackManifest {
    #[serde(flatten)]
    extra: std::collections::HashMap<String, serde_json::Value>,
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

#[derive(Debug, Clone, Serialize, Deserialize)]
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

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[allow(dead_code)]
struct MrpackEnv {
    #[serde(default)]
    client: Option<String>,
    #[serde(default)]
    server: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
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

fn smart_modpack_ram(user_ram: Option<i64>, mod_count: usize) -> i64 {
    if let Some(ram) = user_ram {
        if ram > 0 {
            return ram;
        }
    }

    let total = crate::core::optimizer::get_total_memory_mb();
    let os_reserve: i64 = 3072;
    let available = (total - os_reserve).max(1024);

    if mod_count >= 100 {
        let want = if total >= 32768 { 10240 } else if total >= 24576 { 8192 } else if total >= 16384 { 6144 } else if total >= 12288 { 5120 } else if total >= 8192 { 4096 } else { (total * 6 / 10).max(2048) };
        want.min(available)
    } else if mod_count >= 20 {
        let want = if total >= 16384 { 4096 } else if total >= 8192 { 3072 } else { 2048 };
        want.min(available)
    } else {
        let want = if total >= 8192 { 2560 } else { 2048 };
        want.min(available)
    }
}

fn smart_pvp_ram(user_ram: Option<i64>, mc_version: &str, mod_count: usize) -> i64 {
    if let Some(ram) = user_ram {
        if ram > 0 {
            return ram;
        }
    }
    let is_pvp = mc_version.starts_with("1.8") || mc_version.starts_with("1.7");
    if is_pvp {
        if mod_count > 10 { 3072 } else { 2048 }
    } else {
        smart_modpack_ram(None, mod_count)
    }
}

pub fn instance_cancel_import_core(state: &AppState) -> AppResult<()> {
    state.import_cancel.store(true, std::sync::atomic::Ordering::SeqCst);
    Ok(())
}

#[tauri::command]
pub async fn instance_cancel_import(state: State<'_, AppState>) -> AppResult<()> {
    instance_cancel_import_core(&state)
}

pub(crate) async fn download_cf_mod_file(
    http: &reqwest::Client,
    cf_file: &CfFile,
    file_info: Option<&crate::core::mods::curseforge::CurseForgeFileInfo>,
    _display_name: Option<&str>,
    mods_dir: &std::path::Path,
    storage_mods_dir: &std::path::Path,
    profile_id: &str,
    _mc_version: Option<&str>,
    _loader: Option<&str>,
    cancel: Option<&std::sync::atomic::AtomicBool>,
) -> bool {
    use crate::core::mods::{curseforge, pack_download as pack};
    use sha1::Digest;

    let result: AppResult<()> = async {
        let metadata_path = mods_dir.parent().unwrap_or(mods_dir)
            .join(".luxmc").join("curseforge").join(format!("{}.json", cf_file.file_id));
        let cached = tokio::fs::read(&metadata_path).await.ok()
            .and_then(|bytes| serde_json::from_slice::<curseforge::CurseForgeFileInfo>(&bytes).ok());
        let info = if let Some(info) = file_info.cloned().or(cached) {
            info
        } else {
            curseforge::get_files_batch(http, &[cf_file.file_id]).await.remove(&cf_file.file_id)
                .ok_or_else(|| crate::error::AppError::InvalidState(format!(
                    "Metadados indisponíveis para CurseForge {}/{}. Configure a chave de API.", cf_file.project_id, cf_file.file_id)))?
        };
        if info.id != cf_file.file_id || info.mod_id != cf_file.project_id
            || info.file_name.contains(['/', '\\'])
        {
            return Err(crate::error::AppError::InvalidInput("Invalid CurseForge file metadata".into()));
        }
        let path = pack::destination(mods_dir, &info.file_name)?;
        let disabled = pack::destination(mods_dir, &format!("{}.disabled", info.file_name))?;
        let root = mods_dir.parent().unwrap_or(mods_dir);
        if let Ok(bytes) = tokio::fs::read(root.join(".luxmc/overrides.json")).await {
            if let Ok(overrides) = serde_json::from_slice::<Vec<MrpackFile>>(&bytes) {
                let info_stem = info.file_name.strip_suffix(".jar").unwrap_or(&info.file_name).to_lowercase();
                let info_parts: Vec<&str> = info_stem.split(&['-', '_'][..]).collect();
                let info_base: String = info_parts.into_iter().take_while(|p| !p.chars().any(|c| c.is_ascii_digit())).collect::<Vec<_>>().join("-");
                if let Some(file) = overrides.iter().find(|file| {
                    if file.path == format!("mods/{}", info.file_name) {
                        return true;
                    }
                    if !info_base.is_empty() && info_base.len() >= 3 {
                        if let Some(over_name) = file.path.strip_prefix("mods/") {
                            let over_stem = over_name.strip_suffix(".jar").unwrap_or(over_name).to_lowercase();
                            let over_parts: Vec<&str> = over_stem.split(&['-', '_'][..]).collect();
                            let over_base: String = over_parts.into_iter().take_while(|p| !p.chars().any(|c| c.is_ascii_digit())).collect::<Vec<_>>().join("-");
                            return over_base == info_base;
                        }
                    }
                    false
                }) {
                    ensure_mrpack_file(&pack::client()?, root, file, None).await?;
                    pack::atomic_write(&metadata_path, &serde_json::to_vec(&info)?).await?;
                    return Ok(());
                }
            }
        }
        let valid = |bytes: &[u8]| pack::verify(bytes, info.size, info.sha1.as_deref(), None)
            && zip::ZipArchive::new(std::io::Cursor::new(bytes)).is_ok();
        for existing in [&path, &disabled] {
            if let Ok(bytes) = tokio::fs::read(existing).await {
                if valid(&bytes) {
                    let mut persisted = info.clone();
                    persisted.sha1 = Some(format!("{:x}", sha1::Sha1::digest(&bytes)));
                    persisted.size = Some(bytes.len() as u64);
                    pack::atomic_write(&metadata_path, &serde_json::to_vec(&persisted)?).await?;
                    return Ok(());
                }
            }
        }
        let mut urls = Vec::new();
        if let Some(url) = info.download_url.as_ref().filter(|url| !url.is_empty()) {
            urls.extend(pack::mirrors(url)?);
        }
        for host in ["edge.forgecdn.net", "mediafilez.forgecdn.net", "media.forgecdn.net"] {
            let url = format!("https://{host}/files/{}/{}/{}", cf_file.file_id / 1000,
                cf_file.file_id % 1000, urlencoding::encode(&info.file_name));
            if !urls.contains(&url) { urls.push(url); }
        }
        if let Some(hash) = &info.sha1 {
            if let Ok(response) = http.get(format!("https://api.modrinth.com/v2/version_file/{hash}?algorithm=sha1"))
                .timeout(std::time::Duration::from_secs(35)).send().await {
                if response.status().is_success() {
                    if let Ok(data) = response.json::<serde_json::Value>().await {
                        if let Some(files) = data.get("files").and_then(|v| v.as_array()) {
                            for file in files {
                                if file.get("hashes").and_then(|v| v.get("sha1")).and_then(|v| v.as_str()) == Some(hash.as_str()) {
                                    if let Some(url) = file.get("url").and_then(|v| v.as_str()) { urls.push(url.to_owned()); }
                                }
                            }
                        }
                    }
                }
            }
        }
        let client = pack::client()?;
        for attempt in 0..3 {
            pack::cancelled(cancel)?;
            pack::backoff(attempt, cancel).await?;
            for url in &urls {
                pack::cancelled(cancel)?;
                let Ok(bytes) = pack::bytes(&client, url, cancel).await else { continue };
                if !valid(&bytes) { continue; }
                let target = if disabled.exists() { &disabled } else { &path };
                pack::atomic_write(target, &bytes).await?;
                let storage = pack::destination(storage_mods_dir, &info.file_name)?;
                pack::atomic_write(&storage, &bytes).await?;
                let mut persisted = info.clone();
                persisted.sha1 = Some(format!("{:x}", sha1::Sha1::digest(&bytes)));
                persisted.size = Some(bytes.len() as u64);
                pack::atomic_write(&metadata_path, &serde_json::to_vec(&persisted)?).await?;
                if let Ok(db) = crate::db::shared_db().await {
                    let row = crate::db::schema::mods::ModRow {
                        profile_id: profile_id.to_string(), project_id: cf_file.project_id.to_string(),
                        version_id: cf_file.file_id.to_string(), file_name: info.file_name.clone(),
                        sha1: persisted.sha1.unwrap_or_default(), source: "curseforge".into(), installed_at: String::new(),
                    };
                    let _ = crate::db::schema::mods::upsert(&db, &row).await;
                }
                return Ok(());
            }
        }
        Err(crate::error::AppError::InvalidState(format!("Falha ao baixar {} (CurseForge {})", info.file_name, cf_file.file_id)))
    }.await;
    if let Err(error) = &result { tracing::warn!(%error, "CurseForge download failed"); }
    result.is_ok()
}

pub async fn instance_import_modpack_core(
    app: Option<tauri::AppHandle>,
    state: &AppState,
    file_path: String,
    profile_name: String,
    mc_version: String,
    loader: String,
    icon: Option<String>,
    ram_mb: Option<i64>,
) -> AppResult<ProfileRow> {
    let _import = state.import_lock.try_lock().map_err(|_| crate::error::AppError::InvalidState("Já existe uma importação em andamento.".into()))?;
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
            } else if clean.ends_with("/manifest.json")  {
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
        if entry.size() > 8 * 1024 * 1024 { return Err(crate::error::AppError::InvalidInput("Manifest exceeds 8 MiB".into())); }
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
    let primary_loader = manifest.minecraft.mod_loaders.iter().find(|loader| loader.primary).or_else(|| manifest.minecraft.mod_loaders.first());
    let manifest_loader_type = primary_loader.map(|ml| {
        let parts: Vec<&str> = ml.id.split('-').collect();
        parts[0].trim().to_lowercase()
    });
    let loader_version = primary_loader
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
    let mc_version = mc_version.trim().trim_matches('\'').trim_matches('"').to_string();
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
    let instance_dir = base_dir.data_dir().join("instances").join(&profile_id);
    tokio::fs::create_dir_all(&instance_dir).await?;
    tokio::fs::write(instance_dir.join("manifest.json"), serde_json::to_vec_pretty(&manifest)?).await?;
    let mods_dir = instance_dir.join("mods");
    tokio::fs::create_dir_all(&mods_dir).await?;

    let storage_mods_dir = base_dir.data_dir().join("mods").join(&profile_id);
    let _ = tokio::fs::create_dir_all(&storage_mods_dir).await;
    tracing::info!("directories created, extracting overrides");

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
    let kubejs_prefix = format!("{}kubejs/", root_prefix).to_lowercase();
    let config_prefix = format!("{}config/", root_prefix).to_lowercase();
    let defaultconfigs_prefix = format!("{}defaultconfigs/", root_prefix).to_lowercase();
    let scripts_prefix = format!("{}scripts/", root_prefix).to_lowercase();
    let patchouli_prefix = format!("{}patchouli_books/", root_prefix).to_lowercase();
    let openloader_prefix = format!("{}openloader/", root_prefix).to_lowercase();
    let resourcepacks_prefix = format!("{}resourcepacks/", root_prefix).to_lowercase();
    let shaderpacks_prefix = format!("{}shaderpacks/", root_prefix).to_lowercase();
    let mods_prefix = format!("{}mods/", root_prefix).to_lowercase();

    let mut override_paths = std::collections::HashSet::new();
    let mut extracted_count = 0;
    let mut extracted_size = 0u64;
    for i in 0..archive.len() {
        crate::core::mods::pack_download::cancelled(Some(&state.import_cancel))?;
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
            } else if lower_name.starts_with(&kubejs_prefix) {
                Some(&clean_name[root_prefix.len()..])
            } else if lower_name.starts_with(&config_prefix) {
                Some(&clean_name[root_prefix.len()..])
            } else if lower_name.starts_with(&defaultconfigs_prefix) {
                Some(&clean_name[root_prefix.len()..])
            } else if lower_name.starts_with(&scripts_prefix) {
                Some(&clean_name[root_prefix.len()..])
            } else if lower_name.starts_with(&patchouli_prefix) {
                Some(&clean_name[root_prefix.len()..])
            } else if lower_name.starts_with(&openloader_prefix) {
                Some(&clean_name[root_prefix.len()..])
            } else if lower_name.starts_with(&resourcepacks_prefix) {
                Some(&clean_name[root_prefix.len()..])
            } else if lower_name.starts_with(&shaderpacks_prefix) {
                Some(&clean_name[root_prefix.len()..])
            } else if lower_name.starts_with(&mods_prefix) {
                Some(&clean_name[root_prefix.len()..])
            } else {
                None
            };

            if let Some(rel) = rel_str {
                if rel.is_empty() { continue; }
                validate_pack_entry(rel)?;
                let outpath = crate::core::mods::pack_download::destination(&instance_dir, rel)?;
                extracted_size = extracted_size.saturating_add(file.size());
                if file.unix_mode().is_some_and(|mode| mode & 0o170000 == 0o120000)
                    || file.size() > 1024 * 1024 * 1024 || extracted_size > 8 * 1024 * 1024 * 1024 {
                    return Err(crate::error::AppError::InvalidInput("Unsafe pack archive entry".into()));
                }
                if file.is_dir() || clean_name.ends_with('/') {
                    std::fs::create_dir_all(&outpath)?;
                } else {
                    if let Some(parent) = outpath.parent() { std::fs::create_dir_all(parent)?; }
                    let mut output = std::fs::File::create(&outpath)?;
                    std::io::copy(&mut file, &mut output)?;
                    extracted_count += 1;
                    override_paths.insert(rel.to_owned());
                }
            }
        }
    }
    record_override_jars(&instance_dir, &override_paths).await?;
    tracing::info!(extracted = extracted_count, "overrides extraction completed");

    let project_ids: Vec<u64> = manifest.files.iter().map(|f| f.project_id).collect();
    let file_ids: Vec<u64> = manifest.files.iter().map(|f| f.file_id).collect();
    let mod_names = crate::core::mods::curseforge::get_mod_names_batch(&state.http, &project_ids).await;
    let file_infos = crate::core::mods::curseforge::get_files_batch(&state.http, &file_ids).await;
    tracing::info!(resolved_names = mod_names.len(), resolved_files = file_infos.len(), total = project_ids.len(), "resolved CurseForge batch metadata");

    let total_files = manifest.files.len() as u32;

    if let Some(ref a) = app {
        let _ = a.emit("modpack-progress", serde_json::json!({
            "phase": "downloading",
            "current": 0,
            "total": total_files,
            "percent": 0,
            "status": format!("Preparando download concorrente de {} mods...", total_files)
        }));
    }

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
        let app_for_stream = app.clone();
        let completed = completed_count.clone();
        let ok_counter = mods_ok.clone();
        let fail_counter = mods_fail.clone();

        let mc_ver = mc_version.clone();
        let ld = loader.clone();

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
                Some(&mc_ver),
                Some(&ld),
                Some(&cancel),
            ).await;

            if ok {
                ok_counter.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
            } else {
                fail_counter.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
            }

            let current = completed.fetch_add(1, std::sync::atomic::Ordering::SeqCst) + 1;
            let percent = ((current as f64 / total_files as f64) * 100.0) as u32;

            if current % 16 == 0 {
                crate::commands::optimizer::optimizer_trim_memory();
            }

            if let Some(ref a) = app_for_stream {
                if current % 4 == 0 || current == total_files {
                    let _ = a.emit("modpack-progress", serde_json::json!({
                        "phase": "downloading",
                        "current": current,
                        "total": total_files,
                        "percent": percent,
                        "status": format!("Baixando mods ({}/{})...", current, total_files)
                    }));
                }
            }
        }
    });

    files_stream.buffer_unordered(12).for_each(|_| async {}).await;
    crate::commands::optimizer::optimizer_trim_memory();

    crate::core::mods::pack_download::cancelled(Some(&state.import_cancel))?;
    if mods_fail.load(std::sync::atomic::Ordering::SeqCst) > 0 {
        return Err(crate::error::AppError::InvalidState(format!("Importação incompleta: {} arquivos pendentes. Verifique a chave CurseForge e tente novamente.", mods_fail.load(std::sync::atomic::Ordering::SeqCst))));
    }

    let final_ok = mods_ok.load(std::sync::atomic::Ordering::SeqCst);
    let final_fail = mods_fail.load(std::sync::atomic::Ordering::SeqCst);

    if let Some(ref a) = app {
        let _ = a.emit("modpack-progress", serde_json::json!({
            "phase": "complete",
            "current": total_files,
            "total": total_files,
            "percent": 100,
            "status": format!("{} mods instalados, {} falharam", final_ok, final_fail)
        }));
    }
    tracing::info!(mods_ok = final_ok, mods_fail = final_fail, total = total_files, "curseforge modpack mod download summary");

    let now = chrono::Utc::now();
    let profile_row = ProfileRow {
        id: profile_id,
        name: profile_name,
        icon: icon.unwrap_or_else(|| "default".into()),
        mc_version: mc_version.clone(),
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
        ram_mb: Some(smart_pvp_ram(ram_mb, &mc_version, manifest.files.len())),
        instance_group: None,
        auto_optimize: true,
        use_vulkan: false,
    };

    let db = crate::db::shared_db().await?;
    crate::db::schema::profiles::upsert(&db, &profile_row).await?;
    index_pack_mods(&profile_row).await?;
    Ok(profile_row)
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
    instance_import_modpack_core(Some(app), &state, file_path, profile_name, mc_version, loader, icon, ram_mb).await
}

#[allow(non_snake_case)]
pub async fn instance_repair_modpack_core(
    app: Option<tauri::AppHandle>,
    state: &AppState,
    profileId: String,
) -> AppResult<u32> {
    let db = crate::db::shared_db().await?;
    let row = sqlx::query_as::<_, ProfileRow>("SELECT * FROM profiles WHERE id = ?")
        .bind(&profileId)
        .fetch_optional(db.pool())
        .await?
        .ok_or_else(|| crate::error::AppError::NotFound(format!("profile {profileId} not found")))?;

    let root = std::path::Path::new(&row.game_dir);
    let mods_dir = root.join("mods");
    let before: std::collections::HashMap<_, _> = std::fs::read_dir(&mods_dir).into_iter().flatten().filter_map(Result::ok)
        .filter_map(|entry| entry.metadata().ok().map(|meta| (entry.file_name(), (meta.len(), meta.modified().ok())))).collect();
    heal_modpack(state, &row).await?;
    let repaired = std::fs::read_dir(&mods_dir)?.filter_map(Result::ok).filter(|entry| {
        entry.metadata().ok().is_some_and(|meta| before.get(&entry.file_name()) != Some(&(meta.len(), meta.modified().ok())))
    }).count() as u32;
    if let Some(app) = app {
        let _ = app.emit("modpack-progress", serde_json::json!({ "phase": "complete", "percent": 100, "current": repaired, "total": repaired, "status": format!("{repaired} arquivos recuperados") }));
    }
    Ok(repaired)
}

#[tauri::command]
#[allow(non_snake_case)]
pub async fn instance_repair_modpack(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
    profileId: String,
) -> AppResult<u32> {
    instance_repair_modpack_core(Some(app), &state, profileId).await
}

#[tauri::command]
#[allow(non_snake_case)]
pub async fn instance_health_check(
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

    let mut raw_entries = Vec::new();
    let mut keys_needed = std::collections::HashSet::new();

    if base.is_dir() {
        for entry in std::fs::read_dir(&base)? {
            let entry = entry?;
            let path = entry.path();
            let metadata = entry.metadata()?;
            let is_dir = metadata.is_dir();
            let is_jar_or_zip = !is_dir && path.extension().map_or(false, |e| {
                let s = e.to_string_lossy().to_lowercase();
                s == "jar" || s == "zip" || s == "disabled"
            });
            let fname = path
                .file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_default();

            if is_jar_or_zip {
                keys_needed.insert(fname.clone());
                if let Some(stem) = path.file_stem().map(|s| s.to_string_lossy().to_string()) {
                    let base_key = stem.split('-').next().unwrap_or(&stem).split('_').next().unwrap_or(&stem).to_lowercase();
                    keys_needed.insert(stem);
                    keys_needed.insert(base_key);
                }
            }

            raw_entries.push((fname, path, is_dir, metadata, is_jar_or_zip));
        }
    }

    let cached_icons: std::collections::HashMap<String, String> = if !keys_needed.is_empty() {
        let keys_vec: Vec<String> = keys_needed.into_iter().collect();
        let mut map = std::collections::HashMap::new();
        for chunk in keys_vec.chunks(200) {
            let placeholders = chunk.iter().map(|_| "?").collect::<Vec<_>>().join(",");
            let query_str = format!("SELECT key, icon_url FROM mod_icons WHERE key IN ({})", placeholders);
            let mut query = sqlx::query_as::<_, (String, String)>(&query_str);
            for k in chunk {
                query = query.bind(k);
            }
            if let Ok(rows) = query.fetch_all(db.pool()).await {
                for (k, v) in rows {
                    map.insert(k, v);
                }
            }
        }
        map
    } else {
        std::collections::HashMap::new()
    };

    let mut entries = Vec::new();
    for (fname, path, is_dir, metadata, is_jar_or_zip) in raw_entries {
        let icon = if is_jar_or_zip {
            if let Some(cached) = cached_icons.get(&fname) {
                Some(cached.clone())
            } else if let Some(jar_icon) = crate::commands::mods::extract_mod_icon_from_jar(&path) {
                let _ = sqlx::query("INSERT INTO mod_icons (key, icon_url) VALUES (?, ?) ON CONFLICT(key) DO UPDATE SET icon_url = excluded.icon_url")
                    .bind(&fname)
                    .bind(&jar_icon)
                    .execute(db.pool())
                    .await;
                Some(jar_icon)
            } else {
                let stem = path.file_stem().map(|s| s.to_string_lossy().to_string()).unwrap_or_default();
                let base_key = stem.split('-').next().unwrap_or(&stem).split('_').next().unwrap_or(&stem).to_lowercase();
                cached_icons.get(&stem).cloned().or_else(|| cached_icons.get(&base_key).cloned())
            }
        } else {
            None
        };
        let size = if is_dir { 0 } else { metadata.len() };

        entries.push(FileTreeEntry {
            name: fname,
            path: path.to_string_lossy().to_string(),
            is_dir,
            size,
            icon,
        });
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

async fn record_override_jars(root: &std::path::Path, paths: &std::collections::HashSet<String>) -> AppResult<()> {
    use crate::core::mods::pack_download as pack;
    use sha2::Digest;
    let mut files = Vec::new();
    for path in paths.iter().filter(|path| path.to_lowercase().ends_with(".jar")) {
        let source = pack::destination(root, path)?;
        let bytes = tokio::fs::read(source).await?;
        let target = pack::destination(&root.join(".luxmc/overrides"), path)?;
        pack::atomic_write(&target, &bytes).await?;
        files.push(MrpackFile {
            path: path.clone(), hashes: [("sha512".to_string(), format!("{:x}", sha2::Sha512::digest(&bytes)))].into_iter().collect(),
            env: None, downloads: Vec::new(), file_size: Some(bytes.len() as u64),
        });
    }
    pack::atomic_write(&root.join(".luxmc/overrides.json"), &serde_json::to_vec(&files)?).await
}

fn validate_pack_entry(path: &str) -> AppResult<()> {
    let normalized = path.replace('\\', "/").to_lowercase();
    let first = normalized.split('/').next().unwrap_or_default();
    if [".luxmc", "manifest.json", "modrinth.index.json"].contains(&first) {
        return Err(crate::error::AppError::InvalidInput("Pack attempts to overwrite launcher metadata".into()));
    }
    crate::core::mods::pack_download::relative_path(path)?;
    Ok(())
}

async fn ensure_mrpack_file(
    http: &reqwest::Client,
    root: &std::path::Path,
    file: &MrpackFile,
    cancel: Option<&std::sync::atomic::AtomicBool>,
) -> AppResult<()> {
    use crate::core::mods::pack_download as pack;
    pack::cancelled(cancel)?;
    if file.env.as_ref().and_then(|env| env.client.as_deref()) == Some("unsupported") { return Ok(()); }
    let path = pack::destination(root, &file.path)?;
    let sha1 = file.hashes.get("sha1").map(String::as_str);
    let sha512 = file.hashes.get("sha512").map(String::as_str);
    if sha1.is_none() && sha512.is_none() {
        return Err(crate::error::AppError::InvalidInput(format!("Hash ausente: {}", file.path)));
    }
    let valid = |bytes: &[u8]| pack::verify(bytes, file.file_size, sha1, sha512);
    let disabled = pack::destination(root, &format!("{}.disabled", file.path))?;
    for existing in [&path, &disabled] {
        if let Ok(bytes) = tokio::fs::read(existing).await {
            if valid(&bytes) { return Ok(()); }
        }
    }
    let target = if disabled.exists() { &disabled } else { &path };
    let cache = pack::destination(&root.join(".luxmc/overrides"), &file.path)?;
    if let Ok(bytes) = tokio::fs::read(cache).await {
        if valid(&bytes) { pack::atomic_write(target, &bytes).await?; return Ok(()); }
    }
    for attempt in 0..3 {
        pack::cancelled(cancel)?;
        pack::backoff(attempt, cancel).await?;
        for url in &file.downloads {
            let Ok(bytes) = pack::bytes(http, url, cancel).await else { continue };
            if valid(&bytes) {
                pack::cancelled(cancel)?;
                pack::atomic_write(target, &bytes).await?;
                return Ok(());
            }
        }
    }
    pack::cancelled(cancel)?;
    Err(crate::error::AppError::InvalidState(format!("Arquivo ausente ou inválido: {}", file.path)))
}

async fn index_pack_mods(profile: &ProfileRow) -> AppResult<()> {
    let db = crate::db::shared_db().await?;
    let root = std::path::Path::new(&profile.game_dir);
    let cf = root.join("manifest.json");
    let mut rows_to_insert: Vec<crate::db::schema::mods::ModRow> = Vec::new();

    if cf.exists() {
        if let Ok(bytes) = tokio::fs::read(&cf).await {
            if let Ok(manifest) = serde_json::from_slice::<CfManifest>(&bytes) {
                for file in manifest.files {
                    let meta_path = root.join(format!(".luxmc/curseforge/{}.json", file.file_id));
                    if let Ok(info_bytes) = tokio::fs::read(&meta_path).await {
                        if let Ok(info) = serde_json::from_slice::<crate::core::mods::curseforge::CurseForgeFileInfo>(&info_bytes) {
                            rows_to_insert.push(crate::db::schema::mods::ModRow {
                                profile_id: profile.id.clone(),
                                project_id: file.project_id.to_string(),
                                version_id: file.file_id.to_string(),
                                file_name: info.file_name,
                                sha1: info.sha1.unwrap_or_default(),
                                source: "curseforge".into(),
                                installed_at: String::new(),
                            });
                        }
                    }
                }
            }
        }
    }

    let mr = root.join("modrinth.index.json");
    if mr.exists() {
        if let Ok(bytes) = tokio::fs::read(&mr).await {
            if let Ok(manifest) = serde_json::from_slice::<MrpackManifest>(&bytes) {
                for file in manifest.files {
                    if !file.path.starts_with("mods/") || !file.path.ends_with(".jar") || file.env.as_ref().and_then(|env| env.client.as_deref()) == Some("unsupported") { continue; }
                    for url in &file.downloads {
                        let Ok(url) = reqwest::Url::parse(url) else { continue };
                        if url.host_str() != Some("cdn.modrinth.com") { continue; }
                        let parts: Vec<_> = url.path_segments().into_iter().flatten().collect();
                        if parts.len() < 5 || parts[0] != "data" || parts[2] != "versions" { continue; }
                        rows_to_insert.push(crate::db::schema::mods::ModRow {
                            profile_id: profile.id.clone(),
                            project_id: parts[1].to_owned(),
                            version_id: parts[3].to_owned(),
                            file_name: file.path.trim_start_matches("mods/").to_owned(),
                            sha1: file.hashes.get("sha1").cloned().unwrap_or_default(),
                            source: "modrinth".into(),
                            installed_at: String::new(),
                        });
                        break;
                    }
                }
            }
        }
    }

    if !rows_to_insert.is_empty() {
        let mut tx = db.pool().begin().await?;
        for row in rows_to_insert {
            let _ = sqlx::query(
                "INSERT INTO mods (profile_id, project_id, version_id, file_name, sha1, source, installed_at)
                 VALUES (?, ?, ?, ?, ?, ?, datetime('now'))
                 ON CONFLICT(profile_id, project_id) DO UPDATE SET
                    version_id = excluded.version_id,
                    file_name = excluded.file_name,
                    sha1 = excluded.sha1,
                    source = excluded.source"
            )
            .bind(&row.profile_id)
            .bind(&row.project_id)
            .bind(&row.version_id)
            .bind(&row.file_name)
            .bind(&row.sha1)
            .bind(&row.source)
            .execute(&mut *tx)
            .await;
        }
        let _ = tx.commit().await;
    }

    Ok(())
}

pub(crate) async fn heal_modpack(state: &AppState, profile: &ProfileRow) -> AppResult<()> {
    use crate::core::mods::pack_download as pack;
    let root = std::path::Path::new(&profile.game_dir);
    let override_index = root.join(".luxmc/overrides.json");
    if override_index.exists() {
        let files: Vec<MrpackFile> = serde_json::from_slice(&tokio::fs::read(override_index).await?)?;
        let client = pack::client()?;
        for file in files { ensure_mrpack_file(&client, root, &file, None).await?; }
    }
    let mr_path = root.join("modrinth.index.json");
    if mr_path.exists() {
        let manifest: MrpackManifest = serde_json::from_slice(&tokio::fs::read(mr_path).await?)?;
        let client = pack::client()?;
        let results = futures_util::stream::iter(manifest.files)
            .map(|file| {
                let client = client.clone();
                let root = root.to_path_buf();
                async move { ensure_mrpack_file(&client, &root, &file, None).await }
            })
            .buffer_unordered(12).collect::<Vec<_>>().await;
        for result in results { result?; }
    }
    let cf_path = root.join("manifest.json");
    if cf_path.exists() {
        let manifest: CfManifest = serde_json::from_slice(&tokio::fs::read(cf_path).await?)?;
        let storage = root.join(".luxmc").join("mods");
        for file in &manifest.files {
            if !download_cf_mod_file(&state.http, file, None, None, &root.join("mods"), &storage,
                &profile.id, Some(&profile.mc_version), Some(&profile.loader), None).await {
                tracing::warn!(
                    project_id = file.project_id,
                    file_id = file.file_id,
                    "Arquivo CurseForge opcional ou bloqueado não pôde ser recuperado. Continuando..."
                );
            }
        }
    }
    Ok(())
}

pub async fn instance_import_mrpack_core(
    app: Option<tauri::AppHandle>,
    state: &AppState,
    file_path: String,
    profile_name: String,
    icon: Option<String>,
    ram_mb: Option<i64>,
) -> AppResult<ProfileRow> {
    let _import = state.import_lock.try_lock().map_err(|_| crate::error::AppError::InvalidState("Já existe uma importação em andamento.".into()))?;
    state.import_cancel.store(false, std::sync::atomic::Ordering::SeqCst);
    tracing::info!(file_path = %file_path, profile_name = %profile_name, "instance_import_mrpack called");
    let file = std::fs::File::open(&file_path).map_err(|e| {
        tracing::error!(file_path = %file_path, error = %e, "failed to open mrpack zip");
        e
    })?;
    let mut archive = zip::ZipArchive::new(std::io::BufReader::new(file))
        .map_err(|e| crate::error::AppError::InvalidState(format!("invalid zip: {e}")))?;

    let mut manifest_idx = None;
    let mut root_prefix = String::new();

    for i in 0..archive.len() {
        if let Ok(f) = archive.by_index(i) {
            let name = f.name().replace('\\', "/");
            let clean = name.trim_start_matches('/');
            if clean == "modrinth.index.json" {
                manifest_idx = Some(i);
                root_prefix = String::new();
                break;
            } else if clean.ends_with("/modrinth.index.json")  {
                if let Some(prefix) = clean.strip_suffix("modrinth.index.json") {
                    manifest_idx = Some(i);
                    root_prefix = prefix.to_string();
                }
            }
        }
    }

    let manifest_idx = manifest_idx.ok_or_else(|| {
        crate::error::AppError::NotFound("modrinth.index.json not found in mrpack".into())
    })?;

    let mut manifest: MrpackManifest = {
        let entry = archive.by_index(manifest_idx).map_err(|e| {
            crate::error::AppError::InvalidState(format!("failed to read modrinth.index.json entry: {e}"))
        })?;
        if entry.size() > 8 * 1024 * 1024 { return Err(crate::error::AppError::InvalidInput("Manifest exceeds 8 MiB".into())); }
        serde_json::from_reader(entry).map_err(|e| {
            crate::error::AppError::InvalidState(format!("invalid mrpack manifest: {e}"))
        })?
    };

    let mc_version = manifest
        .dependencies
        .get("minecraft")
        .and_then(|v| v.as_str().map(|s| s.to_string()).or_else(|| Some(v.to_string().trim_matches('"').to_string())))
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
    let instance_dir = base_dir.data_dir().join("instances").join(&profile_id);
    let mods_dir = instance_dir.join("mods");
    tokio::fs::create_dir_all(&mods_dir).await?;

    let storage_mods_dir = base_dir.data_dir().join("mods").join(&profile_id);
    let _ = tokio::fs::create_dir_all(&storage_mods_dir).await;

    use crate::core::mods::pack_download as pack;
    let db = crate::db::shared_db().await?;
    let client = crate::core::mods::ModrinthClient::new(state.http.clone());
    if manifest.files.is_empty() && !manifest.mods.is_empty() {
        for entry in &manifest.mods {
            pack::cancelled(Some(&state.import_cancel))?;
            if entry.env.get("client").and_then(|value| value.as_str()) == Some("unsupported") { continue; }
            let project = entry.project_id.as_deref().ok_or_else(|| crate::error::AppError::InvalidInput("Missing project ID".into()))?;
            let version = entry.version_id.as_deref().ok_or_else(|| crate::error::AppError::InvalidInput("Missing version ID".into()))?;
            let detail = client.get_version_detail(project, version, &mc_version).await?;
            let file = detail.files.first().ok_or_else(|| crate::error::AppError::InvalidInput("Missing mod file".into()))?;
            manifest.files.push(MrpackFile {
                path: entry.path.clone().unwrap_or_else(|| format!("mods/{}", entry.file_name.as_deref().unwrap_or(&file.filename))),
                hashes: [("sha1".to_string(), file.sha1.clone())].into_iter().collect(),
                env: None, downloads: vec![file.url.clone()], file_size: Some(file.size),
            });
        }
    }
    pack::atomic_write(&instance_dir.join(".luxmc/source-modrinth.index.json"), &serde_json::to_vec(&manifest)?).await?;
    let mut paths = std::collections::HashSet::new();
    for file in &manifest.files {
        validate_pack_entry(&file.path)?;
        let path = pack::relative_path(&file.path)?;
        if !paths.insert(path.to_string_lossy().to_lowercase()) {
            return Err(crate::error::AppError::InvalidInput(format!("Duplicate pack path: {}", file.path)));
        }
    }
    let total_mrpack_files = manifest.files.len() as u32;
    let download_client = pack::client()?;
    let completed = std::sync::atomic::AtomicU32::new(0);
    let results = futures_util::stream::iter(manifest.files.clone()).map(|file| {
        let http = &download_client;
        let root = &instance_dir;
        let completed = &completed;
        let app = &app;
        async move {
            let result = ensure_mrpack_file(http, root, &file, Some(&state.import_cancel)).await;
            let current = completed.fetch_add(1, std::sync::atomic::Ordering::SeqCst) + 1;
            if let Some(app) = app {
                let _ = app.emit("modpack-progress", serde_json::json!({
                    "phase": "downloading", "current": current, "total": total_mrpack_files,
                    "percent": current * 100 / total_mrpack_files.max(1),
                    "status": format!("Verificando arquivos ({current}/{total_mrpack_files})")
                }));
            }
            result
        }
    }).buffer_unordered(12).collect::<Vec<_>>().await;
    pack::cancelled(Some(&state.import_cancel))?;
    for result in results { result?; }

    let mut overridden = std::collections::HashSet::new();
    let mut extracted_size = 0u64;
    for folder in ["overrides", "client-overrides"] {
        let prefix = format!("{root_prefix}{folder}/");
        for i in 0..archive.len() {
            pack::cancelled(Some(&state.import_cancel))?;
            let mut file = archive.by_index(i)?;
            let name = file.name().replace('\\', "/");
            let Some(relative) = name.strip_prefix(&prefix).filter(|value| !value.is_empty()) else { continue };
            validate_pack_entry(relative)?;
            let outpath = pack::destination(&instance_dir, relative)?;
            if file.unix_mode().is_some_and(|mode| mode & 0o170000 == 0o120000) {
                return Err(crate::error::AppError::InvalidInput("Symlinks are not allowed in packs".into()));
            }
            extracted_size = extracted_size.saturating_add(file.size());
            if file.size() > 1024 * 1024 * 1024 || extracted_size > 8 * 1024 * 1024 * 1024 {
                return Err(crate::error::AppError::InvalidInput("Pack extraction limit exceeded".into()));
            }
            if file.is_dir() || name.ends_with('/') {
                std::fs::create_dir_all(outpath)?;
            } else {
                if let Some(parent) = outpath.parent() { std::fs::create_dir_all(parent)?; }
                let mut output = std::fs::File::create(&outpath)?;
                std::io::copy(&mut file, &mut output)?;
                overridden.insert(relative.to_owned());
            }
        }
    }
    record_override_jars(&instance_dir, &overridden).await?;
    manifest.files.retain(|file| !overridden.contains(&file.path.replace('\\', "/")));
    pack::atomic_write(&instance_dir.join("modrinth.index.json"), &serde_json::to_vec_pretty(&manifest)?).await?;
    let installed_mods_count = std::fs::read_dir(&mods_dir)?.filter_map(Result::ok)
        .filter(|entry| entry.path().extension().is_some_and(|ext| ext == "jar")).count();
    pack::cancelled(Some(&state.import_cancel))?;

    let (loader, loader_version) = if let Some(v) = manifest
        .dependencies
        .get("fabric-loader")
        .or_else(|| manifest.dependencies.get("fabric"))
        .and_then(|v| v.as_str().map(|s| s.to_string()).or_else(|| Some(v.to_string().trim_matches('"').to_string())))
    {
        ("fabric", Some(v))
    } else if let Some(v) = manifest
        .dependencies
        .get("neoforge")
        .or_else(|| manifest.dependencies.get("neo-forge"))
        .or_else(|| manifest.dependencies.get("neoforged"))
        .and_then(|v| v.as_str().map(|s| s.to_string()).or_else(|| Some(v.to_string().trim_matches('"').to_string())))
    {
        ("neoforge", Some(v))
    } else if let Some(v) = manifest
        .dependencies
        .get("forge")
        .or_else(|| manifest.dependencies.get("minecraft-forge"))
        .and_then(|v| v.as_str().map(|s| s.to_string()).or_else(|| Some(v.to_string().trim_matches('"').to_string())))
    {
        ("forge", Some(v))
    } else if let Some(v) = manifest
        .dependencies
        .get("quilt-loader")
        .or_else(|| manifest.dependencies.get("quilt"))
        .and_then(|v| v.as_str().map(|s| s.to_string()).or_else(|| Some(v.to_string().trim_matches('"').to_string())))
    {
        ("quilt", Some(v))
    } else {
        ("vanilla", None)
    };

    if let Some(ref a) = app {
        let _ = a.emit("modpack-progress", serde_json::json!({
            "phase": "complete",
            "current": total_mrpack_files,
            "total": total_mrpack_files,
            "percent": 100,
            "status": format!("Modpack instalado com sucesso ({} mods configurados)", installed_mods_count)
        }));
    }

    let now = chrono::Utc::now();
    let profile_row = ProfileRow {
        id: profile_id,
        name: profile_name,
        icon: icon.unwrap_or_else(|| "default".into()),
        mc_version: mc_version.clone(),
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
        ram_mb: Some(smart_pvp_ram(ram_mb, &mc_version, installed_mods_count)),
        instance_group: None,
        auto_optimize: true,
        use_vulkan: false,
    };

    crate::db::schema::profiles::upsert(&db, &profile_row).await?;
    index_pack_mods(&profile_row).await?;
    Ok(profile_row)
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
    instance_import_mrpack_core(Some(app), &state, file_path, profile_name, icon, ram_mb).await
}

#[tauri::command]
#[allow(non_snake_case)]
pub async fn instance_export(
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
pub struct WorldPlayerItem {
    pub slot: i32,
    pub id: String,
    pub count: i32,
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
    pub seed: Option<i64>,
    pub spawn_x: Option<i32>,
    pub spawn_y: Option<i32>,
    pub spawn_z: Option<i32>,
    pub version_name: Option<String>,
    pub difficulty: Option<String>,
    pub hardcore: Option<bool>,
    pub player_health: Option<f32>,
    pub player_level: Option<i32>,
    pub day_count: Option<i64>,
    pub snapshots_count: Option<usize>,
    pub player_inventory: Option<Vec<WorldPlayerItem>>,
}

#[derive(Debug, Clone, Deserialize)]
struct LevelDatRoot {
    #[serde(rename = "Data")]
    data: Option<LevelDataCompound>,
}

#[derive(Debug, Clone, Deserialize)]
struct LevelDataCompound {
    #[serde(rename = "LevelName")]
    level_name: Option<String>,
    #[serde(rename = "GameType")]
    game_type: Option<i32>,
    hardcore: Option<u8>,
    #[serde(rename = "SpawnX")]
    spawn_x: Option<i32>,
    #[serde(rename = "SpawnY")]
    spawn_y: Option<i32>,
    #[serde(rename = "SpawnZ")]
    spawn_z: Option<i32>,
    #[serde(rename = "RandomSeed")]
    random_seed: Option<i64>,
    #[serde(rename = "WorldGenSettings")]
    world_gen_settings: Option<WorldGenSettingsCompound>,
    #[serde(rename = "Time")]
    time: Option<i64>,
    #[serde(rename = "DayTime")]
    day_time: Option<i64>,
    #[serde(rename = "Difficulty")]
    difficulty: Option<u8>,
    #[serde(rename = "Version")]
    version: Option<VersionCompound>,
    #[serde(rename = "Player")]
    player: Option<PlayerDataCompound>,
}

#[derive(Debug, Clone, Deserialize)]
struct WorldGenSettingsCompound {
    seed: Option<i64>,
}

#[derive(Debug, Clone, Deserialize)]
struct VersionCompound {
    #[serde(rename = "Name")]
    name: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
struct NbtInventoryItem {
    #[serde(rename = "Slot", default)]
    slot: Option<i8>,
    #[serde(rename = "id", default)]
    id: Option<String>,
    #[serde(rename = "Count", default)]
    count_byte: Option<i8>,
    #[serde(rename = "count", default)]
    count_int: Option<i32>,
}

#[derive(Debug, Clone, Deserialize)]
struct PlayerDataCompound {
    #[serde(rename = "Health")]
    health: Option<f32>,
    #[serde(rename = "XpLevel")]
    xp_level: Option<i32>,
    #[serde(rename = "Inventory", default)]
    inventory: Option<Vec<NbtInventoryItem>>,
}

fn parse_level_dat(level_dat_path: &std::path::Path) -> Option<LevelDataCompound> {
    use std::io::Read;
    let file = std::fs::File::open(level_dat_path).ok()?;
    let mut decoder = flate2::read::GzDecoder::new(file);
    let mut decompressed = Vec::new();
    decoder.read_to_end(&mut decompressed).ok()?;
    let root: LevelDatRoot = fastnbt::from_bytes(&decompressed).ok()?;
    root.data
}

fn parse_player_inventory(
    player_data: Option<&PlayerDataCompound>,
    world_dir: &std::path::Path,
) -> Option<Vec<WorldPlayerItem>> {
    let mut raw_items = player_data
        .and_then(|p| p.inventory.clone())
        .unwrap_or_default();

    if raw_items.is_empty() {
        let playerdata_dir = world_dir.join("playerdata");
        if playerdata_dir.is_dir() {
            if let Ok(entries) = std::fs::read_dir(&playerdata_dir) {
                let mut best_file: Option<std::path::PathBuf> = None;
                let mut best_time = std::time::UNIX_EPOCH;
                for entry in entries.flatten() {
                    let path = entry.path();
                    if path.extension().and_then(|e| e.to_str()) == Some("dat") {
                        if let Ok(meta) = path.metadata() {
                            if let Ok(mtime) = meta.modified() {
                                if mtime > best_time {
                                    best_time = mtime;
                                    best_file = Some(path);
                                }
                            }
                        }
                    }
                }
                if let Some(p) = best_file {
                    use std::io::Read;
                    if let Ok(file) = std::fs::File::open(&p) {
                        let mut decoder = flate2::read::GzDecoder::new(file);
                        let mut decompressed = Vec::new();
                        if decoder.read_to_end(&mut decompressed).is_ok() {
                            if let Ok(player) = fastnbt::from_bytes::<PlayerDataCompound>(&decompressed) {
                                if let Some(inv) = player.inventory {
                                    raw_items = inv;
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    if raw_items.is_empty() {
        return None;
    }

    let mut items: Vec<WorldPlayerItem> = raw_items
        .into_iter()
        .filter_map(|it| {
            let id = it.id?;
            if id.is_empty() {
                return None;
            }
            let count = it
                .count_int
                .or_else(|| it.count_byte.map(|b| b as i32))
                .unwrap_or(1);
            let slot = it.slot.map(|s| s as i32).unwrap_or(0);
            Some(WorldPlayerItem { slot, id, count })
        })
        .collect();

    items.sort_by_key(|it| it.slot);
    if items.is_empty() {
        None
    } else {
        Some(items)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WorldSnapshotInfo {
    pub id: String,
    pub filename: String,
    pub folder_name: String,
    pub label: String,
    pub created_at: i64,
    pub size_bytes: u64,
}

pub(crate) fn inspect_world_dir(
    path: &std::path::Path,
    snapshots_base: &std::path::Path,
) -> Option<WorldDetail> {
    if !path.is_dir() {
        return None;
    }
    let folder_name = path
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_default();
    let mut world_name = folder_name.clone();

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

    let total_size = dir_size_recursive(path);

    let last_played = path
        .metadata()
        .ok()
        .and_then(|m| m.modified().ok())
        .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
        .map(|d| d.as_secs() as i64);

    let mut game_mode = "Sobrevivência".to_string();
    let mut seed = None;
    let mut spawn_x = None;
    let mut spawn_y = None;
    let mut spawn_z = None;
    let mut version_name = None;
    let mut difficulty = None;
    let mut hardcore = None;
    let mut player_health = None;
    let mut player_level = None;
    let mut day_count = None;
    let mut player_inventory = None;

    let level_dat_path = path.join("level.dat");
    if level_dat_path.is_file() {
        if let Some(data) = parse_level_dat(&level_dat_path) {
            if let Some(lvl_name) = data.level_name {
                if !lvl_name.trim().is_empty() {
                    world_name = lvl_name;
                }
            }

            if data.hardcore == Some(1) {
                game_mode = "Hardcore".to_string();
                hardcore = Some(true);
            } else {
                game_mode = match data.game_type {
                    Some(1) => "Criativo".to_string(),
                    Some(2) => "Aventura".to_string(),
                    Some(3) => "Espectador".to_string(),
                    _ => "Sobrevivência".to_string(),
                };
                hardcore = Some(false);
            }

            seed = data.random_seed.or_else(|| data.world_gen_settings.and_then(|w| w.seed));
            spawn_x = data.spawn_x;
            spawn_y = data.spawn_y;
            spawn_z = data.spawn_z;
            version_name = data.version.and_then(|v| v.name);
            difficulty = match data.difficulty {
                Some(0) => Some("Pacífico".to_string()),
                Some(1) => Some("Fácil".to_string()),
                Some(2) => Some("Normal".to_string()),
                Some(3) => Some("Difícil".to_string()),
                _ => None,
            };
            player_health = data.player.as_ref().and_then(|p| p.health);
            player_level = data.player.as_ref().and_then(|p| p.xp_level);
            day_count = data.day_time.or(data.time).map(|t| (t / 24000).max(1));
            player_inventory = parse_player_inventory(data.player.as_ref(), path);
        }
    }

    let world_snapshots_dir = snapshots_base.join(&folder_name);
    let snapshots_count = if world_snapshots_dir.is_dir() {
        std::fs::read_dir(&world_snapshots_dir)
            .ok()
            .map(|entries| {
                entries
                    .flatten()
                    .filter(|e| e.path().extension().map_or(false, |ext| ext == "zst"))
                    .count()
            })
    } else {
        Some(0)
    };

    Some(WorldDetail {
        name: world_name,
        folder_name,
        icon_base64,
        last_played,
        game_mode: Some(game_mode),
        size_bytes: total_size,
        seed,
        spawn_x,
        spawn_y,
        spawn_z,
        version_name,
        difficulty,
        hardcore,
        player_health,
        player_level,
        day_count,
        snapshots_count,
        player_inventory,
    })
}

#[tauri::command]
#[allow(non_snake_case)]
pub async fn instance_worlds_list(
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
    let snapshots_base = std::path::PathBuf::from(&row.game_dir).join("snapshots");
    let mut list = Vec::new();

    if saves_dir.is_dir() {
        if let Ok(entries) = std::fs::read_dir(&saves_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    if let Some(detail) = inspect_world_dir(&path, &snapshots_base) {
                        list.push(detail);
                    }
                }
            }
        }
    }

    Ok(list)
}

#[tauri::command]
#[allow(non_snake_case)]
pub async fn instance_world_snapshot_create(
    profileId: String,
    folderName: String,
    label: Option<String>,
) -> AppResult<WorldSnapshotInfo> {
    if folderName.is_empty() || folderName.contains("..") || folderName.contains('/') || folderName.contains('\\') {
        return Err(crate::error::AppError::InvalidInput("Invalid folder name".into()));
    }
    let db = crate::db::shared_db().await?;
    let row = sqlx::query_as::<_, ProfileRow>("SELECT * FROM profiles WHERE id = ?")
        .bind(&profileId)
        .fetch_optional(db.pool())
        .await?
        .ok_or_else(|| crate::error::AppError::NotFound(format!("profile {profileId} not found")))?;

    let world_dir = std::path::PathBuf::from(&row.game_dir).join("saves").join(&folderName);
    if !world_dir.is_dir() {
        return Err(crate::error::AppError::NotFound("World directory not found".into()));
    }

    let snapshots_dir = std::path::PathBuf::from(&row.game_dir).join("snapshots").join(&folderName);
    tokio::fs::create_dir_all(&snapshots_dir).await?;

    let timestamp = chrono::Utc::now().timestamp();
    let safe_label = label
        .as_deref()
        .unwrap_or("Manual")
        .trim()
        .replace(|c: char| !c.is_alphanumeric() && c != '_' && c != '-', "");
    let filename = format!("snapshot_{}_{}.tar.zst", timestamp, if safe_label.is_empty() { "Auto" } else { &safe_label });
    let dest_path = snapshots_dir.join(&filename);

    let file = std::fs::File::create(&dest_path)?;
    let zstd_writer = zstd::Encoder::new(file, 3)?.auto_finish();
    let mut tar_builder = tar::Builder::new(zstd_writer);
    tar_builder.append_dir_all(".", &world_dir)?;
    tar_builder.finish()?;

    let size_bytes = dest_path.metadata().map(|m| m.len()).unwrap_or(0);
    Ok(WorldSnapshotInfo {
        id: format!("{}_{}", folderName, timestamp),
        filename,
        folder_name: folderName,
        label: if safe_label.is_empty() { "Auto".to_string() } else { safe_label },
        created_at: timestamp,
        size_bytes,
    })
}

#[tauri::command]
#[allow(non_snake_case)]
pub async fn instance_world_snapshots_list(
    profileId: String,
    folderName: String,
) -> AppResult<Vec<WorldSnapshotInfo>> {
    if folderName.is_empty() || folderName.contains("..") || folderName.contains('/') || folderName.contains('\\') {
        return Err(crate::error::AppError::InvalidInput("Invalid folder name".into()));
    }
    let db = crate::db::shared_db().await?;
    let row = sqlx::query_as::<_, ProfileRow>("SELECT * FROM profiles WHERE id = ?")
        .bind(&profileId)
        .fetch_optional(db.pool())
        .await?
        .ok_or_else(|| crate::error::AppError::NotFound(format!("profile {profileId} not found")))?;

    let snapshots_dir = std::path::PathBuf::from(&row.game_dir).join("snapshots").join(&folderName);
    let mut list = Vec::new();
    if snapshots_dir.is_dir() {
        if let Ok(entries) = std::fs::read_dir(&snapshots_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                let fname = path.file_name().map(|n| n.to_string_lossy().to_string()).unwrap_or_default();
                if fname.ends_with(".tar.zst") && fname.starts_with("snapshot_") {
                    let meta = entry.metadata().ok();
                    let size_bytes = meta.as_ref().map(|m| m.len()).unwrap_or(0);
                    let created_at = meta
                        .and_then(|m| m.modified().ok())
                        .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
                        .map(|d| d.as_secs() as i64)
                        .unwrap_or_else(|| chrono::Utc::now().timestamp());

                    let parts: Vec<&str> = fname.trim_end_matches(".tar.zst").splitn(3, '_').collect();
                    let label = if parts.len() >= 3 { parts[2].to_string() } else { "Snapshot".to_string() };

                    list.push(WorldSnapshotInfo {
                        id: fname.clone(),
                        filename: fname,
                        folder_name: folderName.clone(),
                        label,
                        created_at,
                        size_bytes,
                    });
                }
            }
        }
    }

    list.sort_by(|a, b| b.created_at.cmp(&a.created_at));
    Ok(list)
}

#[tauri::command]
#[allow(non_snake_case)]
pub async fn instance_world_snapshot_restore(
    profileId: String,
    folderName: String,
    filename: String,
) -> AppResult<()> {
    if folderName.is_empty() || folderName.contains("..") || filename.contains("..") || filename.contains('/') || filename.contains('\\') {
        return Err(crate::error::AppError::InvalidInput("Invalid arguments".into()));
    }
    let db = crate::db::shared_db().await?;
    let row = sqlx::query_as::<_, ProfileRow>("SELECT * FROM profiles WHERE id = ?")
        .bind(&profileId)
        .fetch_optional(db.pool())
        .await?
        .ok_or_else(|| crate::error::AppError::NotFound(format!("profile {profileId} not found")))?;

    let snapshot_file = std::path::PathBuf::from(&row.game_dir).join("snapshots").join(&folderName).join(&filename);
    if !snapshot_file.is_file() {
        return Err(crate::error::AppError::NotFound("Snapshot archive not found".into()));
    }

    let world_dir = std::path::PathBuf::from(&row.game_dir).join("saves").join(&folderName);
    if world_dir.is_dir() {
        let _ = tokio::fs::remove_dir_all(&world_dir).await;
    }
    tokio::fs::create_dir_all(&world_dir).await?;

    let file = std::fs::File::open(&snapshot_file)?;
    let zstd_reader = zstd::Decoder::new(file)?;
    let mut archive = tar::Archive::new(zstd_reader);
    archive.unpack(&world_dir)?;

    Ok(())
}

#[tauri::command]
#[allow(non_snake_case)]
pub async fn instance_world_snapshot_delete(
    profileId: String,
    folderName: String,
    filename: String,
) -> AppResult<()> {
    if folderName.is_empty() || folderName.contains("..") || filename.contains("..") || filename.contains('/') || filename.contains('\\') {
        return Err(crate::error::AppError::InvalidInput("Invalid arguments".into()));
    }
    let db = crate::db::shared_db().await?;
    let row = sqlx::query_as::<_, ProfileRow>("SELECT * FROM profiles WHERE id = ?")
        .bind(&profileId)
        .fetch_optional(db.pool())
        .await?
        .ok_or_else(|| crate::error::AppError::NotFound(format!("profile {profileId} not found")))?;

    let snapshot_file = std::path::PathBuf::from(&row.game_dir).join("snapshots").join(&folderName).join(&filename);
    if snapshot_file.is_file() {
        std::fs::remove_file(&snapshot_file)?;
    }
    Ok(())
}

#[tauri::command]
#[allow(non_snake_case)]
pub async fn instance_world_inspect_region(
    profileId: String,
    folderName: String,
    regionFile: Option<String>,
) -> AppResult<Option<crate::core::minecraft::anvil::RegionSummary>> {
    let db = crate::db::shared_db().await?;
    let row = sqlx::query_as::<_, ProfileRow>("SELECT * FROM profiles WHERE id = ?")
        .bind(&profileId)
        .fetch_optional(db.pool())
        .await?
        .ok_or_else(|| crate::error::AppError::NotFound(format!("profile {profileId} not found")))?;

    let target_file = regionFile.unwrap_or_else(|| "r.0.0.mca".to_string());
    let region_path = std::path::PathBuf::from(&row.game_dir)
        .join("saves")
        .join(&folderName)
        .join("region")
        .join(&target_file);

    if region_path.is_file() {
        Ok(crate::core::minecraft::anvil::inspect_region_file(&region_path))
    } else {
        Ok(None)
    }
}

#[tauri::command]
#[allow(non_snake_case)]
pub async fn upnp_open_port(port: u16, leaseDurationSecs: Option<u32>) -> crate::core::network::upnp::UpnpPortMappingResult {
    crate::core::network::upnp::open_port_upnp(port, leaseDurationSecs.unwrap_or(7200)).await
}

#[tauri::command]
pub async fn upnp_close_port(port: u16) -> bool {
    crate::core::network::upnp::close_port_upnp(port).await
}


#[tauri::command]
#[allow(non_snake_case)]
pub async fn instance_world_delete(
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
pub async fn instance_world_import(
    profileId: String,
    sourcePath: String,
) -> AppResult<WorldDetail> {
    let db = crate::db::shared_db().await?;
    let row = sqlx::query_as::<_, ProfileRow>("SELECT * FROM profiles WHERE id = ?")
        .bind(&profileId)
        .fetch_optional(db.pool())
        .await?
        .ok_or_else(|| {
            crate::error::AppError::NotFound(format!("profile {profileId} not found"))
        })?;

    let src = std::path::PathBuf::from(&sourcePath);
    if !src.exists() {
        return Err(crate::error::AppError::NotFound(format!(
            "Source path not found: {}",
            sourcePath
        )));
    }

    let saves_dir = std::path::PathBuf::from(&row.game_dir).join("saves");
    std::fs::create_dir_all(&saves_dir)?;
    let snapshots_base = std::path::PathBuf::from(&row.game_dir).join("snapshots");

    let get_unique_dir = |base_name: &str| -> std::path::PathBuf {
        let clean: String = base_name
            .chars()
            .map(|c| {
                if c.is_alphanumeric() || c == '-' || c == '_' || c == ' ' {
                    c
                } else {
                    '_'
                }
            })
            .collect();
        let clean = clean.trim();
        let clean = if clean.is_empty() {
            "Mundo_Importado"
        } else {
            clean
        };
        let mut target = saves_dir.join(clean);
        let mut counter = 1;
        while target.exists() {
            target = saves_dir.join(format!("{} ({})", clean, counter));
            counter += 1;
        }
        target
    };

    let target_dir: std::path::PathBuf;

    if src.is_file()
        && src
            .extension()
            .map_or(false, |ext| ext.eq_ignore_ascii_case("zip"))
    {
        let file = std::fs::File::open(&src)?;
        let mut archive = zip::ZipArchive::new(file)
            .map_err(|e| crate::error::AppError::Io(std::io::Error::new(std::io::ErrorKind::InvalidData, e)))?;

        let mut level_dat_prefix: Option<String> = None;
        let mut default_folder_name = src
            .file_stem()
            .map(|s| s.to_string_lossy().to_string())
            .unwrap_or_else(|| "Mundo_Importado".into());

        for i in 0..archive.len() {
            if let Ok(entry) = archive.by_index(i) {
                let name = entry.name().replace('\\', "/");
                if name.ends_with("level.dat") {
                    let prefix = if name == "level.dat" {
                        "".to_string()
                    } else {
                        name.trim_end_matches("level.dat").to_string()
                    };
                    if let Some(parent) = prefix.trim_end_matches('/').split('/').last() {
                        if !parent.is_empty() {
                            default_folder_name = parent.to_string();
                        }
                    }
                    level_dat_prefix = Some(prefix);
                    break;
                }
            }
        }

        target_dir = get_unique_dir(&default_folder_name);
        std::fs::create_dir_all(&target_dir)?;

        let prefix = level_dat_prefix.unwrap_or_default();

        for i in 0..archive.len() {
            let mut file = archive.by_index(i)
                .map_err(|e| crate::error::AppError::Io(std::io::Error::new(std::io::ErrorKind::InvalidData, e)))?;
            let entry_name = file.name().replace('\\', "/");

            if !prefix.is_empty() && !entry_name.starts_with(&prefix) {
                continue;
            }

            let rel_name = if !prefix.is_empty() {
                entry_name.strip_prefix(&prefix).unwrap_or(&entry_name)
            } else {
                &entry_name
            };

            let rel_path = std::path::Path::new(rel_name);
            if rel_path
                .components()
                .any(|c| matches!(c, std::path::Component::ParentDir))
            {
                continue;
            }

            let out_path = target_dir.join(rel_path);
            if file.is_dir() {
                std::fs::create_dir_all(&out_path)?;
            } else {
                if let Some(parent) = out_path.parent() {
                    std::fs::create_dir_all(parent)?;
                }
                let mut out_file = std::fs::File::create(&out_path)?;
                std::io::copy(&mut file, &mut out_file)?;
            }
        }
    } else if src.is_dir() {
        let actual_world_src = if src.join("level.dat").is_file() {
            src.clone()
        } else {
            let mut found = None;
            if let Ok(entries) = std::fs::read_dir(&src) {
                for e in entries.flatten() {
                    if e.path().is_dir() && e.path().join("level.dat").is_file() {
                        found = Some(e.path());
                        break;
                    }
                }
            }
            found.unwrap_or(src.clone())
        };

        let folder_name = actual_world_src
            .file_name()
            .map(|s| s.to_string_lossy().to_string())
            .unwrap_or_else(|| "Mundo_Importado".into());
        target_dir = get_unique_dir(&folder_name);

        let mut options = fs_extra::dir::CopyOptions::new();
        options.copy_inside = true;
        std::fs::create_dir_all(&target_dir)?;
        fs_extra::dir::copy(&actual_world_src, &target_dir, &options)
            .map_err(|e| crate::error::AppError::Io(std::io::Error::new(std::io::ErrorKind::Other, e)))?;

        let sub = target_dir.join(&folder_name);
        if sub.join("level.dat").is_file() {
            if let Ok(entries) = std::fs::read_dir(&sub) {
                for e in entries.flatten() {
                    let dest = target_dir.join(e.file_name());
                    let _ = std::fs::rename(e.path(), dest);
                }
            }
            let _ = std::fs::remove_dir(sub);
        }
    } else {
        return Err(crate::error::AppError::InvalidInput(
            "Invalid world source: must be a directory or .zip file".into(),
        ));
    }

    inspect_world_dir(&target_dir, &snapshots_base)
        .ok_or_else(|| crate::error::AppError::InvalidInput("Failed to read imported world level.dat".into()))
}

#[tauri::command]
#[allow(non_snake_case)]
pub async fn instance_mod_toggle(
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

pub async fn instance_install_quick_pack_core(
    http: &reqwest::Client,
    profile_id: String,
    pack_type: String,
    project_id: String,
) -> AppResult<String> {
    let db = crate::db::shared_db().await?;
    let row = sqlx::query_as::<_, ProfileRow>("SELECT * FROM profiles WHERE id = ?")
        .bind(&profile_id)
        .fetch_optional(db.pool())
        .await?
        .ok_or_else(|| crate::error::AppError::NotFound(format!("profile {profile_id} not found")))?;

    let folder_name = match pack_type.as_str() {
        "shaders" | "shaderpacks" => "shaderpacks",
        "datapacks" => "datapacks",
        _ => "resourcepacks",
    };

    let target_dir = std::path::PathBuf::from(&row.game_dir).join(folder_name);
    tokio::fs::create_dir_all(&target_dir).await?;

    let version_url = format!("https://api.modrinth.com/v2/project/{}/version", project_id);
    let resp = http.get(&version_url)
        .header("User-Agent", "Luxmc/1.6.5")
        .timeout(std::time::Duration::from_secs(20))
        .send()
        .await
        .map_err(|e| crate::error::AppError::Internal(format!("Falha ao conectar com Modrinth: {e}")))?;

    let versions: serde_json::Value = resp.json().await
        .map_err(|e| crate::error::AppError::InvalidState(format!("Resposta inválida de versões: {e}")))?;

    let ver_list = versions.as_array()
        .ok_or_else(|| crate::error::AppError::NotFound("Nenhuma versão encontrada para este pacote".into()))?;

    let mut download_info: Option<(String, String)> = None;
    for ver in ver_list {
        if let Some(files) = ver.get("files").and_then(|f| f.as_array()) {
            for f in files {
                let url = f.get("url").and_then(|u| u.as_str()).unwrap_or("");
                let filename = f.get("filename").and_then(|n| n.as_str()).unwrap_or("");
                if !url.is_empty() && !filename.is_empty() {
                    download_info = Some((url.to_string(), filename.to_string()));
                    break;
                }
            }
        }
        if download_info.is_some() {
            break;
        }
    }

    let (dl_url, filename) = download_info
        .ok_or_else(|| crate::error::AppError::NotFound("Arquivo para download não encontrado".into()))?;

    let file_resp = http.get(&dl_url)
        .header("User-Agent", "Luxmc/1.6.5")
        .timeout(std::time::Duration::from_secs(60))
        .send()
        .await
        .map_err(|e| crate::error::AppError::Internal(format!("Falha no download: {e}")))?;

    let bytes = file_resp.bytes().await
        .map_err(|e| crate::error::AppError::Internal(format!("Falha ao ler dados do arquivo: {e}")))?;

    let dest = target_dir.join(&filename);
    tokio::fs::write(&dest, &bytes).await?;

    Ok(filename)
}

#[tauri::command]
#[allow(non_snake_case)]
pub async fn instance_install_quick_pack(
    state: State<'_, AppState>,
    profileId: String,
    packType: String,
    projectId: String,
) -> AppResult<String> {
    instance_install_quick_pack_core(&state.http, profileId, packType, projectId).await
}

#[cfg(test)]
mod modpack_integrity_tests {
    use super::*;
    use sha2::Digest;

    fn root() -> std::path::PathBuf {
        let root = std::env::temp_dir().join(format!("luxmc-pack-test-{}", Uuid::new_v4()));
        std::fs::create_dir_all(&root).unwrap();
        root
    }

    fn entry(path: &str, bytes: &[u8]) -> MrpackFile {
        MrpackFile {
            path: path.into(), hashes: [("sha512".into(), format!("{:x}", sha2::Sha512::digest(bytes)))].into_iter().collect(),
            env: None, downloads: Vec::new(), file_size: Some(bytes.len() as u64),
        }
    }

    #[tokio::test]
    async fn preserves_disabled_mods_and_restores_override_bytes() {
        let root = root();
        let bytes = b"the exact override jar bytes";
        std::fs::create_dir_all(root.join("mods")).unwrap();
        std::fs::write(root.join("mods/a.jar"), bytes).unwrap();
        record_override_jars(&root, &["mods/a.jar".into()].into_iter().collect()).await.unwrap();
        std::fs::rename(root.join("mods/a.jar"), root.join("mods/a.jar.disabled")).unwrap();
        let client = crate::core::mods::pack_download::client().unwrap();
        ensure_mrpack_file(&client, &root, &entry("mods/a.jar", bytes), None).await.unwrap();
        assert!(!root.join("mods/a.jar").exists());
        std::fs::write(root.join("mods/a.jar.disabled"), b"").unwrap();
        ensure_mrpack_file(&client, &root, &entry("mods/a.jar", bytes), None).await.unwrap();
        assert_eq!(std::fs::read(root.join("mods/a.jar.disabled")).unwrap(), bytes);
        assert!(!root.join("mods/a.jar").exists());
        std::fs::remove_dir_all(root).unwrap();
    }

    #[tokio::test]
    async fn unrelated_jars_cannot_satisfy_a_missing_manifest_entry() {
        let root = root();
        std::fs::create_dir_all(root.join("mods")).unwrap();
        std::fs::write(root.join("mods/library-extra.jar"), b"unrelated").unwrap();
        std::fs::write(root.join("mods/library-helper.jar"), b"unrelated").unwrap();
        let client = crate::core::mods::pack_download::client().unwrap();
        assert!(ensure_mrpack_file(&client, &root, &entry("mods/library.jar", b"expected"), None).await.is_err());
        assert!(root.join("mods/library-extra.jar").exists());
        assert!(root.join("mods/library-helper.jar").exists());
        std::fs::remove_dir_all(root).unwrap();
    }

    #[tokio::test]
    async fn cancellation_prevents_writes_and_reserved_metadata_is_rejected() {
        let root = root();
        let cancel = std::sync::atomic::AtomicBool::new(true);
        let client = crate::core::mods::pack_download::client().unwrap();
        assert!(ensure_mrpack_file(&client, &root, &entry("mods/a.jar", b"valid"), Some(&cancel)).await.is_err());
        assert!(!root.join("mods").exists());
        for path in [".luxmc/overrides.json", "manifest.json", "MODRINTH.INDEX.JSON", "mods/../../escape"] {
            assert!(validate_pack_entry(path).is_err());
        }
        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn curseforge_manifest_roundtrip_preserves_ids_and_loader() {
        let manifest: CfManifest = serde_json::from_str(r#"{"minecraft":{"version":"1.20.1","modLoaders":[{"id":"forge-47.2.0"}]},"files":[{"projectID":42,"fileID":1234}]}"#).unwrap();
        let serialized = serde_json::to_value(&manifest).unwrap();
        assert_eq!(serialized["files"][0]["projectID"], 42);
        let legacy: CfManifest = serde_json::from_str(r#"{"minecraft":{"mod_loaders":[{"id":"fabric-0.16.0"}]},"files":[{"project_id":42,"file_id":1234}]}"#).unwrap();
        assert_eq!(legacy.files[0].file_id, 1234);
        assert_eq!(legacy.minecraft.mod_loaders[0].id, "fabric-0.16.0");
    }
}
