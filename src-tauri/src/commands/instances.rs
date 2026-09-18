use serde::{Deserialize, Serialize};
use tokio::io::AsyncWriteExt;
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
    #[serde(default, alias = "modLoaders")]
    pub(crate) mod_loaders: Vec<CfModLoader>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct CfModLoader {
    #[serde(default)]
    pub(crate) id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct CfFile {
    #[serde(alias = "projectID", alias = "projectId")]
    pub(crate) project_id: u64,
    #[serde(alias = "fileID", alias = "fileId")]
    pub(crate) file_id: u64,
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

#[derive(Debug, Clone, Deserialize)]
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

#[derive(Debug, Clone, Deserialize)]
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

async fn download_from_modrinth_fallback(
    http: &reqwest::Client,
    file_name: &str,
    display_name: Option<&str>,
    file_path: &std::path::Path,
    storage_path: &std::path::Path,
    mc_version: Option<&str>,
    loader: Option<&str>,
) -> bool {
    let raw_source = display_name.unwrap_or(file_name);
    let without_ext = raw_source.trim_end_matches(".jar");
    let clean_query = {
        let no_brackets = without_ext.replace(['(', '[', '{', ')', ']', '}'], " ");
        let mut words: Vec<&str> = Vec::new();
        for w in no_brackets.split_whitespace() {
            let lower = w.to_lowercase();
            if lower == "api" || lower == "forge" || lower == "fabric" || lower == "neoforge" || lower == "quilt" || lower == "mod" || lower.starts_with('v') && lower[1..].chars().all(|c| c.is_ascii_digit() || c == '.') || lower.chars().all(|c| c.is_ascii_digit() || c == '.') {
                continue;
            }
            words.push(w);
        }
        if words.is_empty() {
            without_ext.split(&['-', '_'][..]).next().unwrap_or(without_ext).trim().to_string()
        } else {
            words.join(" ")
        }
    };

    if clean_query.is_empty() || clean_query.len() < 2 {
        return false;
    }

    let search_urls = {
        let mut urls = Vec::new();
        if let (Some(mc), Some(ld)) = (mc_version, loader) {
            let facets = format!("[[\"project_type:mod\"],[\"versions:{}\"],[\"categories:{}\"]]", mc, ld.to_lowercase());
            urls.push(format!(
                "https://api.modrinth.com/v2/search?query={}&facets={}&limit=4",
                urlencoding::encode(&clean_query),
                urlencoding::encode(&facets)
            ));
        }
        urls.push(format!(
            "https://api.modrinth.com/v2/search?query={}&limit=4",
            urlencoding::encode(&clean_query)
        ));
        urls
    };

    let mut projects_found = Vec::new();
    for s_url in search_urls {
        if let Ok(resp) = http
            .get(&s_url)
            .header("User-Agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36")
            .timeout(std::time::Duration::from_secs(10))
            .send()
            .await
        {
            if resp.status().is_success() {
                if let Ok(hits) = resp.json::<serde_json::Value>().await {
                    if let Some(arr) = hits.get("hits").and_then(|h| h.as_array()) {
                        if !arr.is_empty() {
                            projects_found = arr.clone();
                            break;
                        }
                    }
                }
            }
        }
    }

    if projects_found.is_empty() {
        return false;
    }

    let query_lower = clean_query.to_lowercase();
    for proj in projects_found {
        let proj_id = match proj.get("project_id").and_then(|p| p.as_str()) {
            Some(id) => id,
            None => continue,
        };

        let proj_title = proj.get("title").and_then(|t| t.as_str()).unwrap_or("");
        let proj_slug = proj.get("slug").and_then(|s| s.as_str()).unwrap_or("");
        let title_match = proj_title.to_lowercase().contains(&query_lower) || proj_slug.to_lowercase().contains(&query_lower);

        let ver_url = if let (Some(mc), Some(ld)) = (mc_version, loader) {
            format!(
                "https://api.modrinth.com/v2/project/{}/version?game_versions=[\"{}\"]&loaders=[\"{}\"]",
                proj_id, mc, ld.to_lowercase()
            )
        } else {
            format!("https://api.modrinth.com/v2/project/{}/version", proj_id)
        };

        let ver_resp = match http
            .get(&ver_url)
            .header("User-Agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36")
            .timeout(std::time::Duration::from_secs(12))
            .send()
            .await
        {
            Ok(r) if r.status().is_success() => r,
            _ => continue,
        };

        let versions: serde_json::Value = match ver_resp.json().await {
            Ok(v) => v,
            Err(_) => continue,
        };

        let ver_list = match versions.as_array() {
            Some(v) => v,
            None => continue,
        };

        for ver in ver_list {
            if let Some(files) = ver.get("files").and_then(|f| f.as_array()) {
                for file_entry in files {
                    let dl_url = file_entry.get("url").and_then(|u| u.as_str()).unwrap_or("");
                    let mr_filename = file_entry.get("filename").and_then(|f| f.as_str()).unwrap_or("");

                    if dl_url.is_empty() {
                        continue;
                    }

                    let is_match = mr_filename.eq_ignore_ascii_case(file_name)
                        || mr_filename.to_lowercase().starts_with(&query_lower)
                        || (title_match && file_entry.get("primary").and_then(|p| p.as_bool()).unwrap_or(false));

                    if is_match {
                        if let Ok(dl_resp) = http
                            .get(dl_url)
                            .header("User-Agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36")
                            .timeout(std::time::Duration::from_secs(40))
                            .send()
                            .await
                        {
                            if dl_resp.status().is_success() {
                                let temp_path = file_path.with_extension("tmp");
                                let mut file = match tokio::fs::File::create(&temp_path).await {
                                    Ok(f) => f,
                                    Err(_) => continue,
                                };
                                let mut downloaded = Vec::new();
                                let mut stream = dl_resp.bytes_stream();
                                while let Some(chunk) = stream.next().await {
                                    let chunk = match chunk { Ok(c) => c, Err(_) => continue };
                                    downloaded.extend_from_slice(&chunk);
                                    let _ = file.write_all(&chunk).await;
                                }
                                let _ = file.flush().await;
                                drop(file);

                                if downloaded.len() > 200 && (downloaded.starts_with(b"PK") || zip::ZipArchive::new(std::io::Cursor::new(&downloaded)).is_ok()) {
                                    let _ = tokio::fs::rename(&temp_path, file_path).await;
                                    let _ = tokio::fs::copy(file_path, storage_path).await;
                                    tracing::info!(mod_name = %clean_query, "Downloaded missing mod from Modrinth CDN fallback");
                                    return true;
                                }
                                let _ = tokio::fs::remove_file(&temp_path).await;
                            }
                        }
                    }
                }
            }
        }
    }

    false
}

pub(crate) async fn download_cf_mod_file(
    http: &reqwest::Client,
    cf_file: &CfFile,
    file_info: Option<&crate::core::mods::curseforge::CurseForgeFileInfo>,
    display_name: Option<&str>,
    mods_dir: &std::path::Path,
    storage_mods_dir: &std::path::Path,
    profile_id: &str,
    mc_version: Option<&str>,
    loader: Option<&str>,
) -> bool {
    let project_id_str = cf_file.project_id.to_string();
    let file_id_str = cf_file.file_id.to_string();

    let single_info;
    let file_info = match file_info {
        Some(info) => Some(info),
        None => {
            let single = crate::core::mods::curseforge::get_files_batch(http, &[cf_file.file_id]).await;
            single_info = single.get(&cf_file.file_id).cloned();
            single_info.as_ref()
        }
    };

    let p1 = cf_file.file_id / 1000;
    let p2 = cf_file.file_id % 1000;
    let p2_pad = format!("{:03}", p2);

    let mut urls_to_try: Vec<String> = Vec::new();

    urls_to_try.push(format!(
        "https://www.curseforge.com/api/v1/mods/{}/files/{}/download",
        project_id_str, file_id_str
    ));

    if let Ok(url) = crate::core::mods::curseforge::get_download_url(http, &project_id_str, &file_id_str).await {
        if !url.is_empty() && !urls_to_try.contains(&url) {
            urls_to_try.push(url);
        }
    }

    let mut resolved_filename = file_info.map(|i| i.file_name.clone());

    if let Ok(details) = crate::core::mods::curseforge::get_mod_file_details(http, &project_id_str, &file_id_str).await {
        if !details.url.is_empty() && !urls_to_try.contains(&details.url) {
            urls_to_try.push(details.url);
        }
        if resolved_filename.is_none() && !details.filename.is_empty() {
            resolved_filename = Some(details.filename);
        }
    }

    let cdn_name_ref = resolved_filename.as_deref().or_else(|| file_info.map(|i| i.file_name.as_str()));
    if let Some(name_for_cdn) = cdn_name_ref {
        for host in ["mediafilez.forgecdn.net", "media.forgecdn.net", "edge.forgecdn.net"] {
            for sub in [format!("{}/{}", p1, p2), format!("{}/{}", p1, p2_pad)] {
                let u1 = format!("https://{}/files/{}/{}", host, sub, urlencoding::encode(name_for_cdn));
                if !urls_to_try.contains(&u1) { urls_to_try.push(u1); }
                let u2 = format!("https://{}/files/{}/{}", host, sub, name_for_cdn.replace(' ', "%20"));
                if !urls_to_try.contains(&u2) { urls_to_try.push(u2); }
            }
        }
    }

    if let Some(info) = file_info {
        if let Some(ref dl) = info.download_url {
            if !dl.is_empty() && !urls_to_try.contains(dl) {
                urls_to_try.push(dl.clone());
            }
        }
    }

    urls_to_try.push(format!("https://curse.nikky.moe/api/v2/pack/file/{}", file_id_str));
    urls_to_try.push(format!("https://cf.way2muchnoise.eu/file/{}", file_id_str));
    urls_to_try.push(format!("https://cursemeta.dries007.net/{}/{}", project_id_str, file_id_str));
    urls_to_try.push(format!("https://api.cfwidget.com/mc-mods/minecraft/{}", project_id_str));
    urls_to_try.push(format!("https://www.curseforge.com/minecraft/mc-mods/{}/files/{}", project_id_str, file_id_str));

    if let Some(name) = display_name {
        let guess = format!("https://edge.forgecdn.net/files/{}/{}/{}.jar", p1, p2, urlencoding::encode(name));
        if !urls_to_try.contains(&guess) { urls_to_try.push(guess); }
        let guess_pad = format!("https://edge.forgecdn.net/files/{}/{}/{}.jar", p1, p2_pad, urlencoding::encode(name));
        if !urls_to_try.contains(&guess_pad) { urls_to_try.push(guess_pad); }
    }

    let filename = if let Some(ref r_name) = resolved_filename {
        r_name.clone()
    } else if let Some(info) = file_info {
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
        if let Ok(meta) = std::fs::metadata(&file_path) {
            if meta.len() > 200 {
                if let Ok(f) = std::fs::File::open(&file_path) {
                    if zip::ZipArchive::new(std::io::BufReader::new(f)).is_ok() {
                        return true;
                    }
                }
            }
        }
        let _ = tokio::fs::remove_file(&file_path).await;
    }

    if let Some(p) = file_path.parent() { let _ = tokio::fs::create_dir_all(p).await; }
    if let Some(p) = storage_path.parent() { let _ = tokio::fs::create_dir_all(p).await; }

    for try_url in &urls_to_try {
        for attempt in 0..3 {
            if attempt > 0 {
                tokio::time::sleep(std::time::Duration::from_millis(250 * attempt as u64)).await;
            }
            let mut req = http
                .get(try_url)
                .header("User-Agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36")
                .header("Accept", "*/*")
                .timeout(std::time::Duration::from_secs(35));
            if let Some(key) = crate::core::mods::curseforge::api_key() {
                if try_url.contains("curseforge.com") {
                    req = req.header("x-api-key", key);
                }
            }
            let resp = match req.send().await {
                Ok(r) if r.status().is_success() => r,
                _ => continue,
            };

            let mut stream = resp.bytes_stream();
            let temp_path = file_path.with_extension("tmp");
            let mut file = match tokio::fs::File::create(&temp_path).await {
                Ok(f) => f,
                Err(_) => continue,
            };
            let mut downloaded = Vec::new();
            while let Some(chunk) = stream.next().await {
                let chunk = match chunk {
                    Ok(c) => c,
                    Err(_) => {
                        let _ = tokio::fs::remove_file(&temp_path).await;
                        continue;
                    }
                };
                downloaded.extend_from_slice(&chunk);
                let _ = file.write_all(&chunk).await;
            }
            let _ = file.flush().await;
            drop(file);

            if downloaded.len() < 200 {
                let _ = tokio::fs::remove_file(&temp_path).await;
                continue;
            }

            let is_valid = downloaded.len() > 200 && (zip::ZipArchive::new(std::io::Cursor::new(&downloaded)).is_ok() || downloaded.starts_with(b"PK"));
            if !is_valid {
                let _ = tokio::fs::remove_file(&temp_path).await;
                continue;
            }

            if tokio::fs::rename(&temp_path, &file_path).await.is_err() {
                let _ = tokio::fs::remove_file(&temp_path).await;
                continue;
            }
            let _ = tokio::fs::copy(&file_path, &storage_path).await;

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

    if download_from_modrinth_fallback(http, &filename, display_name, &file_path, &storage_path, mc_version, loader).await {
        if let Ok(db) = crate::db::shared_db().await {
            let mod_row = crate::db::schema::mods::ModRow {
                profile_id: profile_id.to_string(),
                project_id: project_id_str.clone(),
                version_id: file_id_str.clone(),
                file_name: filename.clone(),
                sha1: String::new(),
                source: "modrinth_fallback".into(),
                installed_at: String::new(),
            };
            let _ = crate::db::schema::mods::upsert(&db, &mod_row).await;
        }
        return true;
    }

    tracing::warn!(
        project_id = %project_id_str,
        file_id = %file_id_str,
        name = display_name.unwrap_or("unknown"),
        "CurseForge file could not be downloaded automatically (author distribution restricted)"
    );
    false
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

    let mut extracted_count = 0;
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
                        if std::io::copy(&mut file, &mut outfile).is_ok() {
                            extracted_count += 1;
                        }
                    }
                }
            }
        }
    }
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
                let _ = a.emit("modpack-progress", serde_json::json!({
                    "phase": "downloading",
                    "current": current,
                    "total": total_files,
                    "percent": percent,
                    "status": format!("Baixando mods ({}/{})...", current, total_files)
                }));
            }
        }
    });

    files_stream.buffer_unordered(4).for_each(|_| async {}).await;
    crate::commands::optimizer::optimizer_trim_memory();

    for retry_pass in 0..3 {
        let current_fail = mods_fail.load(std::sync::atomic::Ordering::SeqCst);
        if current_fail == 0 || state.import_cancel.load(std::sync::atomic::Ordering::SeqCst) {
            break;
        }

        tracing::info!(pass = retry_pass + 1, failed = current_fail, "retrying failed modpack mods in resilient pass");
        if let Some(ref a) = app {
            let _ = a.emit("modpack-progress", serde_json::json!({
                "phase": "downloading",
                "current": total_files.saturating_sub(current_fail),
                "total": total_files,
                "percent": 90 + retry_pass * 3,
                "status": format!("Repassando {} mods pendentes (tentativa {}/3)...", current_fail, retry_pass + 1)
            }));
        }

        let mut existing_valid = std::collections::HashSet::new();
        if let Ok(mut entries) = tokio::fs::read_dir(&mods_dir).await {
            while let Ok(Some(entry)) = entries.next_entry().await {
                let name = entry.file_name().to_string_lossy().to_string();
                if let Ok(meta) = entry.metadata().await {
                    if meta.len() > 200 {
                        if let Ok(f) = std::fs::File::open(entry.path()) {
                            if zip::ZipArchive::new(std::io::BufReader::new(f)).is_ok() {
                                existing_valid.insert(name);
                            }
                        }
                    }
                }
            }
        }

        let pending: Vec<CfFile> = manifest.files.iter().filter(|f| {
            let f_id = f.file_id;
            let p_id_str = f.project_id.to_string();
            let f_id_str = f_id.to_string();
            let fi = file_infos.get(&f_id);
            let expected_name = fi.map(|i| i.file_name.clone()).unwrap_or_default();
            !existing_valid.contains(&expected_name) && !existing_valid.iter().any(|e| e.contains(&format!("{}_{}", p_id_str, f_id_str)))
        }).cloned().collect();

        if pending.is_empty() {
            mods_fail.store(0, std::sync::atomic::Ordering::SeqCst);
            break;
        }

        let retry_stream = futures_util::stream::iter(pending).map(|cf_file| {
            let http = state.http.clone();
            let file_info = file_infos.get(&cf_file.file_id).cloned();
            let display_name = mod_names.get(&cf_file.project_id).cloned();
            let mods_dir = mods_dir.clone();
            let storage_mods_dir = storage_mods_dir.clone();
            let profile_id = profile_id.clone();
            let mc_ver = mc_version.clone();
            let ld = loader.clone();
            let ok_counter = mods_ok.clone();
            let fail_counter = mods_fail.clone();

            async move {
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
                ).await;
                if ok {
                    ok_counter.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                    let _ = fail_counter.fetch_update(std::sync::atomic::Ordering::SeqCst, std::sync::atomic::Ordering::SeqCst, |f| Some(f.saturating_sub(1)));
                }
            }
        });
        retry_stream.buffer_unordered(4).for_each(|_| async {}).await;
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

    let instance_dir = std::path::PathBuf::from(&row.game_dir);
    let manifest_path = instance_dir.join("manifest.json");

    if !manifest_path.exists() {
        if let Some(proj) = directories::ProjectDirs::from("io", "github", "Luxmc") {
            let modpack_cache = proj.cache_dir().join("modpacks");
            if let Ok(mut rd) = tokio::fs::read_dir(&modpack_cache).await {
                while let Ok(Some(entry)) = rd.next_entry().await {
                    let p = entry.path();
                    if p.extension().map_or(false, |e| e == "zip") {
                        if let Ok(f) = std::fs::File::open(&p) {
                            if let Ok(mut zip_arch) = zip::ZipArchive::new(std::io::BufReader::new(f)) {
                                for i in 0..zip_arch.len() {
                                    if let Ok(mut zf) = zip_arch.by_index(i) {
                                        if zf.name().ends_with("manifest.json") {
                                            let mut buf = Vec::new();
                                            if std::io::copy(&mut zf, &mut buf).is_ok() {
                                                let _ = std::fs::write(&manifest_path, &buf);
                                                break;
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                    if manifest_path.exists() { break; }
                }
            }
        }
    }

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
    let stream = futures_util::stream::iter(missing_files.clone()).map(|cf_file| {
        let http = state.http.clone();
        let file_info = file_infos.get(&cf_file.file_id).cloned();
        let display_name = mod_names.get(&cf_file.project_id).cloned();
        let mods_dir = mods_dir.clone();
        let storage_mods_dir = storage_mods_dir.clone();
        let profile_id = profileId.clone();
        let app_for_stream = app.clone();
        let completed = completed_count.clone();
        let repaired = repaired_count.clone();
        let mc_ver = row.mc_version.clone();
        let ld = row.loader.clone();

        async move {
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
            ).await;

            if ok {
                repaired.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
            }

            let cur = completed.fetch_add(1, std::sync::atomic::Ordering::SeqCst) + 1;
            let percent = ((cur as f64 / total_missing as f64) * 100.0) as u32;
            if let Some(ref a) = app_for_stream {
                let _ = a.emit("modpack-repair-progress", serde_json::json!({
                    "current": cur,
                    "total": total_missing,
                    "percent": percent,
                    "status": format!("Baixando mod ausente {}/{}...", cur, total_missing)
                }));
            }
        }
    });

    stream.buffer_unordered(8).collect::<Vec<()>>().await;
    crate::commands::optimizer::optimizer_trim_memory();
    let mut total_repaired = repaired_count.load(std::sync::atomic::Ordering::SeqCst);

    if total_repaired < total_missing {
        let mut still_missing = Vec::new();
        let mut check_existing = std::collections::HashSet::new();
        if let Ok(mut entries) = tokio::fs::read_dir(&mods_dir).await {
            while let Ok(Some(entry)) = entries.next_entry().await {
                let name = entry.file_name().to_string_lossy().to_string();
                if let Ok(meta) = entry.metadata().await {
                    if meta.len() > 100 {
                        check_existing.insert(name);
                    }
                }
            }
        }
        for cf in &missing_files {
            let p_str = cf.project_id.to_string();
            let f_str = cf.file_id.to_string();
            let fi = file_infos.get(&cf.file_id);
            let expected = fi.map(|i| i.file_name.clone()).unwrap_or_default();
            if !check_existing.contains(&expected) && !check_existing.iter().any(|e| e.contains(&format!("{}_{}", p_str, f_str))) {
                still_missing.push(cf.clone());
            }
        }

        if !still_missing.is_empty() {
            let retry_stream = futures_util::stream::iter(still_missing).map(|cf_file| {
                let http = state.http.clone();
                let file_info = file_infos.get(&cf_file.file_id).cloned();
                let display_name = mod_names.get(&cf_file.project_id).cloned();
                let mods_dir = mods_dir.clone();
                let storage_mods_dir = storage_mods_dir.clone();
                let profile_id = profileId.clone();
                let repaired = repaired_count.clone();
                let mc_ver = row.mc_version.clone();
                let ld = row.loader.clone();

                async move {
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
                    ).await;
                    if ok {
                        repaired.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                    }
                }
            });
            retry_stream.buffer_unordered(4).for_each(|_| async {}).await;
            total_repaired = repaired_count.load(std::sync::atomic::Ordering::SeqCst);
        }
    }

    let count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM mods WHERE profile_id = ?")
        .bind(&profileId)
        .fetch_one(db.pool())
        .await
        .unwrap_or((0,));

    let ram_boost = smart_modpack_ram(row.ram_mb, manifest.files.len());
    let _ = sqlx::query("UPDATE profiles SET mod_count = ?, ram_mb = ? WHERE id = ?")
        .bind(std::cmp::max(count.0, manifest.files.len() as i64))
        .bind(ram_boost)
        .bind(&profileId)
        .execute(db.pool())
        .await;

    Ok(total_repaired)
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

    let mut entries = Vec::new();

    let cached_icons: std::collections::HashMap<String, String> = {
        let rows: Vec<(String, String)> = sqlx::query_as("SELECT key, icon_url FROM mod_icons")
            .fetch_all(db.pool())
            .await
            .unwrap_or_default();
        rows.into_iter().collect()
    };

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

pub async fn instance_import_mrpack_core(
    app: Option<tauri::AppHandle>,
    state: &AppState,
    file_path: String,
    profile_name: String,
    icon: Option<String>,
    ram_mb: Option<i64>,
) -> AppResult<ProfileRow> {
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
            } else if clean.ends_with("/modrinth.index.json") && clean.matches('/').count() == 1 {
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

    let manifest: MrpackManifest = {
        let entry = archive.by_index(manifest_idx).map_err(|e| {
            crate::error::AppError::InvalidState(format!("failed to read modrinth.index.json entry: {e}"))
        })?;
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
    let instance_dir = base_dir.data_dir().join("instances").join(&profile_name);
    let mods_dir = instance_dir.join("mods");
    tokio::fs::create_dir_all(&mods_dir).await?;

    let storage_mods_dir = base_dir.data_dir().join("mods").join(&profile_id);
    let _ = tokio::fs::create_dir_all(&storage_mods_dir).await;

    let overrides_prefix = format!("{}overrides/", root_prefix).to_lowercase();
    let client_overrides_prefix = format!("{}client-overrides/", root_prefix).to_lowercase();

    for i in 0..archive.len() {
        if let Ok(mut file) = archive.by_index(i) {
            let raw_name = file.name().replace('\\', "/");
            let clean_name = raw_name.trim_start_matches('/');
            let lower_name = clean_name.to_lowercase();

            let rel_str = if lower_name.starts_with(&overrides_prefix) {
                clean_name.get(overrides_prefix.len()..)
            } else if lower_name.starts_with(&client_overrides_prefix) {
                clean_name.get(client_overrides_prefix.len()..)
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

    let client = crate::core::mods::ModrinthClient::new(state.http.clone());
    let db = crate::db::shared_db().await?;

    let total_mrpack_files = manifest.files.len() as u32;
    if let Some(ref a) = app {
        let _ = a.emit("modpack-progress", serde_json::json!({
            "phase": "downloading",
            "current": 0,
            "total": total_mrpack_files,
            "percent": 0,
            "status": format!("Preparando download concorrente de {} arquivos...", total_mrpack_files)
        }));
    }

    use futures_util::StreamExt;
    let completed_count = std::sync::Arc::new(std::sync::atomic::AtomicU32::new(0));
    let mods_installed_count = std::sync::Arc::new(std::sync::atomic::AtomicU32::new(0));

    let files_stream = futures_util::stream::iter(manifest.files.clone()).map(|mrpack_file| {
        let http = state.http.clone();
        let cancel = state.import_cancel.clone();
        let instance_dir = instance_dir.clone();
        let storage_mods_dir = storage_mods_dir.clone();
        let profile_id = profile_id.clone();
        let app_for_stream = app.clone();
        let completed = completed_count.clone();
        let mods_count = mods_installed_count.clone();
        let total = total_mrpack_files;
        let db = db.clone();

        async move {
            if cancel.load(std::sync::atomic::Ordering::SeqCst) {
                return;
            }

            if let Some(env) = &mrpack_file.env {
                if env.client.as_deref() == Some("unsupported") {
                    completed.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                    return;
                }
            }

            if mrpack_file.downloads.is_empty() {
                completed.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                return;
            }

            let clean_path = mrpack_file.path.replace('\\', "/");
            let outpath = instance_dir.join(&clean_path);
            if let Some(parent) = outpath.parent() {
                let _ = tokio::fs::create_dir_all(parent).await;
            }

            let mut success = false;
            for url in &mrpack_file.downloads {
                for attempt in 0..3 {
                    if attempt > 0 {
                        tokio::time::sleep(std::time::Duration::from_millis(250 * attempt as u64)).await;
                    }
                    if let Ok(resp) = http.get(url).timeout(std::time::Duration::from_secs(35)).send().await {
                        if resp.status().is_success() {
                            if let Ok(bytes) = resp.bytes().await {
                                if !bytes.is_empty() {
                                    if tokio::fs::write(&outpath, &bytes).await.is_ok() {
                                        success = true;
                                        if clean_path.starts_with("mods/") || clean_path.ends_with(".jar") {
                                            mods_count.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                                            let filename = std::path::Path::new(&clean_path)
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
                }
                if success {
                    break;
                }
            }

            let current = completed.fetch_add(1, std::sync::atomic::Ordering::SeqCst) + 1;
            let percent = if total > 0 {
                ((current as f64 / total as f64) * 100.0) as u32
            } else {
                100
            };

            if current % 16 == 0 {
                crate::commands::optimizer::optimizer_trim_memory();
            }

            if let Some(ref a) = app_for_stream {
                let _ = a.emit("modpack-progress", serde_json::json!({
                    "phase": "downloading",
                    "current": current,
                    "total": total,
                    "percent": percent,
                    "status": format!("Baixando arquivos ({}/{})...", current, total)
                }));
            }
        }
    });

    files_stream.buffer_unordered(6).for_each(|_| async {}).await;
    crate::commands::optimizer::optimizer_trim_memory();

    let mut installed_mods_count = mods_installed_count.load(std::sync::atomic::Ordering::SeqCst) as usize;

    if manifest.files.is_empty() && !manifest.mods.is_empty() {
        for mrpack_mod in &manifest.mods {
            if state.import_cancel.load(std::sync::atomic::Ordering::SeqCst) {
                break;
            }
            if let (Some(project_id), Some(version_id)) =
                (&mrpack_mod.project_id, &mrpack_mod.version_id)
            {
                let version_detail = client
                    .get_version_detail(project_id, version_id, &mc_version)
                    .await;
                if let Ok(version) = version_detail {
                    if let Some(file) = version.files.first() {
                        if let Ok(resp) = state.http.get(&file.url).timeout(std::time::Duration::from_secs(35)).send().await {
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

                    let total_size = dir_size_recursive(&path);

                    let last_played = entry
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
                            player_inventory = parse_player_inventory(data.player.as_ref(), &path);
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

                    list.push(WorldDetail {
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
                    });
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
