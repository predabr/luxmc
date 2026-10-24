use serde::Serialize;
use std::{collections::HashMap, sync::LazyLock, time::{Duration, SystemTime, UNIX_EPOCH}};
use tokio::sync::Semaphore;
use crate::{db::models::ProfileRow, error::{AppError, AppResult}, state::AppState};

static WORK: LazyLock<Semaphore> = LazyLock::new(|| Semaphore::new(4));
static IMAGE_WORK: LazyLock<Semaphore> = LazyLock::new(|| Semaphore::new(2));
static IMAGE_HTTP: LazyLock<reqwest::Client> = LazyLock::new(|| reqwest::Client::builder()
    .user_agent(concat!("Luxmc/", env!("CARGO_PKG_VERSION")))
    .redirect(reqwest::redirect::Policy::none())
    .timeout(Duration::from_secs(8))
    .build().expect("thumbnail HTTP client"));

fn image_url(source: &str) -> Option<reqwest::Url> {
    let url = reqwest::Url::parse(source).ok()?;
    (url.scheme() == "https" && url.username().is_empty() && url.password().is_none()
        && url.port_or_known_default() == Some(443)
        && matches!(url.host_str(), Some("media.forgecdn.net" | "cdn.modrinth.com" | "cdn-raw.modrinth.com")))
        .then_some(url)
}

fn thumbnail(bytes: &[u8]) -> Option<String> {
    use base64::Engine;
    let mut reader = image::ImageReader::new(std::io::Cursor::new(bytes)).with_guessed_format().ok()?;
    let mut limits = image::Limits::default();
    limits.max_image_width = Some(2048);
    limits.max_image_height = Some(2048);
    limits.max_alloc = Some(32 * 1024 * 1024);
    reader.limits(limits);
    let image = reader.decode().ok()?.thumbnail(96, 96);
    let mut png = std::io::Cursor::new(Vec::new());
    image.write_to(&mut png, image::ImageFormat::Png).ok()?;
    Some(format!("data:image/png;base64,{}", base64::engine::general_purpose::STANDARD.encode(png.into_inner())))
}

async fn cached_image(db: &crate::db::Db, source: &str) -> Option<String> {
    use sha2::Digest;
    let url = image_url(source)?;
    let key = format!("thumbnail:v1:{:x}", sha2::Sha256::digest(source.as_bytes()));
    let _permit = IMAGE_WORK.acquire().await.ok()?;
    if let Some(image) = sqlx::query_scalar::<_, String>("SELECT icon_url FROM mod_icons WHERE key = ?")
        .bind(&key).fetch_optional(db.pool()).await.ok().flatten() {
        return Some(image);
    }
    let mut response = IMAGE_HTTP.get(url).send().await.ok()?.error_for_status().ok()?;
    if response.content_length().is_some_and(|size| size > 2 * 1024 * 1024) { return None; }
    let mut bytes = Vec::new();
    while let Some(chunk) = response.chunk().await.ok()? {
        if bytes.len() + chunk.len() > 2 * 1024 * 1024 { return None; }
        bytes.extend_from_slice(&chunk);
    }
    let image = tokio::task::spawn_blocking(move || thumbnail(&bytes)).await.ok()??;
    let _ = sqlx::query("INSERT INTO mod_icons(key, icon_url) VALUES (?, ?) ON CONFLICT(key) DO UPDATE SET icon_url = excluded.icon_url")
        .bind(key).bind(&image).execute(db.pool()).await;
    Some(image)
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ContentIcon { name: String, icon: Option<String>, icon_key: String, resolved: bool }

pub fn cache_is_resolved(value: &str) -> bool {
    if let Some(time) = value.strip_prefix("missing:").and_then(|value| value.parse::<u64>().ok()) {
        return now().saturating_sub(time) < 86400;
    }
    !value.is_empty()
}
fn now() -> u64 { SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_secs() }
fn matching_shader_archive(directory: &std::path::Path) -> Option<std::path::PathBuf> {
    use std::io::Read;
    let base = directory.parent()?;
    let mut candidates: Vec<_> = std::fs::read_dir(base).ok()?.filter_map(Result::ok)
        .filter(|entry| entry.path().extension().is_some_and(|extension| extension.eq_ignore_ascii_case("zip")))
        .take(32).map(|entry| entry.path()).collect();
    candidates.sort();
    for candidate in candidates {
        if !std::fs::metadata(&candidate).is_ok_and(|metadata| metadata.is_file() && metadata.len() <= 64 * 1024 * 1024) { continue; }
        let Ok(file) = std::fs::File::open(&candidate) else { continue; };
        let Ok(mut archive) = zip::ZipArchive::new(file) else { continue; };
        if archive.len() > 10000 { continue; }
        let mut matched = 0;
        let mut different = false;
        for index in 0..archive.len() {
            let Ok(mut entry) = archive.by_index(index) else { continue; };
            let Some(relative) = entry.enclosed_name().map(|path| path.to_path_buf()) else { continue; };
            if !relative.starts_with("shaders") || !relative.extension().is_some_and(|extension| matches!(extension.to_str(), Some("glsl" | "fsh" | "vsh"))) || entry.size() > 1024 * 1024 { continue; }
            let Ok(path) = crate::core::instance_paths::resolve_within(directory, &relative.to_string_lossy()) else { different = true; break; };
            if !std::fs::metadata(&path).is_ok_and(|metadata| metadata.is_file() && metadata.len() == entry.size()) { different = true; break; }
            let Ok(local) = std::fs::read(path) else { different = true; break; };
            let mut zipped = Vec::new();
            if entry.by_ref().take(1024 * 1024 + 1).read_to_end(&mut zipped).is_err() || zipped != local { different = true; break; }
            matched += 1;
            if matched == 3 { break; }
        }
        if !different && matched == 3 { return Some(candidate); }
    }
    None
}
fn allowed(path: &str) -> bool {
    matches!(path, "mods" | "resourcepacks" | "shaderpacks") || {
        let parts: Vec<_> = path.split('/').collect();
        parts.len() == 3 && parts[0] == "saves" && !parts[1].is_empty() && parts[1] != "." && parts[1] != ".." && !parts[1].contains('\\') && parts[2] == "datapacks"
    }
}

#[tauri::command]
#[allow(non_snake_case)]
pub async fn instance_content_icons(state: tauri::State<'_, AppState>, profileId: String, subPath: String, fileNames: Vec<String>, lookup: Option<bool>, cacheImages: Option<bool>) -> AppResult<Vec<ContentIcon>> {
    instance_content_icons_core(&state, profileId, subPath, fileNames, lookup.unwrap_or(false), cacheImages.unwrap_or(false)).await
}

pub async fn instance_content_icons_core(state: &AppState, profile_id: String, sub_path: String, names: Vec<String>, lookup: bool, cache_images: bool) -> AppResult<Vec<ContentIcon>> {
    if names.len() > 32 || !allowed(&sub_path) { return Err(AppError::InvalidInput("Invalid content thumbnail request".into())); }
    let db = crate::db::shared_db().await?;
    let row = sqlx::query_as::<_, ProfileRow>("SELECT * FROM profiles WHERE id = ?").bind(&profile_id).fetch_optional(db.pool()).await?.ok_or_else(|| AppError::NotFound("Instance not found".into()))?;
    let base = crate::core::instance_paths::resolve_within(std::path::Path::new(&row.game_dir), &sub_path)?;
    let records = if sub_path == "mods" { crate::db::schema::mods::list_by_profile(&db, &profile_id).await? } else { Vec::new() };
    let mut known = HashMap::<(String, String), Vec<usize>>::new();
    let mut results = Vec::new();
    let mut hashes = HashMap::<String, Vec<usize>>::new();
    for name in names {
        if name.is_empty() || name.contains(['/', '\\']) || name == "." || name == ".." { return Err(AppError::InvalidInput("Invalid content filename".into())); }
        let path = crate::core::instance_paths::resolve_within(&base, &name)?;
        let Ok(metadata) = tokio::fs::metadata(&path).await else { continue; };
        let key = super::mods::content_icon_key(&path, &metadata);
        let mut cached = sqlx::query_scalar::<_, String>("SELECT icon_url FROM mod_icons WHERE key = ?").bind(&key).fetch_optional(db.pool()).await?;
        if cached.is_none() {
            cached = sqlx::query_scalar::<_, String>("SELECT icon_url FROM mod_icons WHERE key = ?").bind(key.replacen("content:v3:", "content:v2:", 1)).fetch_optional(db.pool()).await?.filter(|value| value.starts_with("https://") || value.starts_with("data:image/"));
        }
        if cached.is_none() {
            if let Some(record) = records.iter().find(|record| record.file_name.trim_end_matches(".disabled") == name.trim_end_matches(".disabled")) {
                cached = sqlx::query_scalar::<_, String>("SELECT icon_url FROM mod_icons WHERE key = ?").bind(&record.project_id).fetch_optional(db.pool()).await?.filter(|value| value.starts_with("https://"));
            }
        }
        if cached.is_none() && sub_path != "mods" && !metadata.is_dir() {
            let legacy_key = key.replacen("content:v3:", "content:", 1);
            if let Some(url) = sqlx::query_scalar::<_, String>("SELECT icon_url FROM mod_icons WHERE key = ?").bind(legacy_key).fetch_optional(db.pool()).await?.filter(|value| value.starts_with("https://")) {
                sqlx::query("INSERT INTO mod_icons(key, icon_url) VALUES (?, ?) ON CONFLICT(key) DO UPDATE SET icon_url = excluded.icon_url").bind(&key).bind(&url).execute(db.pool()).await?;
                cached = Some(url);
            }
        }
        if let Some(value) = cached.filter(|value| cache_is_resolved(value) && !(sub_path == "shaderpacks" && metadata.is_dir() && value.starts_with("missing:"))) {
            sqlx::query("INSERT INTO mod_icons(key, icon_url) VALUES (?, ?) ON CONFLICT(key) DO NOTHING").bind(&key).bind(&value).execute(db.pool()).await?;
            results.push(ContentIcon { name, icon: (!value.starts_with("missing:")).then_some(value), icon_key: key, resolved: true });
            continue;
        }
        let file = path.clone();
        let icon = {
            let _permit = WORK.acquire().await.map_err(|error| AppError::Internal(error.to_string()))?;
            tokio::task::spawn_blocking(move || super::mods::extract_mod_icon_from_jar(&file)).await.map_err(|error| AppError::Internal(error.to_string()))?
        };
        let shader_directory = sub_path == "shaderpacks" && metadata.is_dir();
        let resolved = icon.is_some() || metadata.is_dir() && !shader_directory;
        if resolved {
            let value = icon.clone().unwrap_or_else(|| format!("missing:{}", now()));
            sqlx::query("INSERT INTO mod_icons(key, icon_url) VALUES (?, ?) ON CONFLICT(key) DO UPDATE SET icon_url = excluded.icon_url").bind(&key).bind(value).execute(db.pool()).await?;
        } else if lookup && metadata.len() <= 256 * 1024 * 1024 {
            let path = if shader_directory {
                let directory = path.clone();
                let _permit = WORK.acquire().await.map_err(|error| AppError::Internal(error.to_string()))?;
                match tokio::task::spawn_blocking(move || matching_shader_archive(&directory)).await.map_err(|error| AppError::Internal(error.to_string()))? {
                    Some(archive) => archive,
                    None => { results.push(ContentIcon { name, icon, icon_key: key, resolved: true }); continue; }
                }
            } else { path };
            if let Some(record) = records.iter().find(|record| record.file_name.trim_end_matches(".disabled") == name.trim_end_matches(".disabled") && matches!(record.source.as_str(), "curseforge" | "modrinth" | "modrinth_fallback")) {
                known.entry((record.source.clone(), record.project_id.clone())).or_default().push(results.len());
                results.push(ContentIcon { name, icon, icon_key: key, resolved });
                continue;
            }
            let _permit = WORK.acquire().await.map_err(|error| AppError::Internal(error.to_string()))?;
            let hash = tokio::task::spawn_blocking(move || -> std::io::Result<String> {
                use sha1::{Digest, Sha1};
                use std::io::Read;
                let mut file = std::fs::File::open(path)?;
                let mut digest = Sha1::new();
                let mut buffer = [0u8; 65536];
                loop { let count = file.read(&mut buffer)?; if count == 0 { break; } digest.update(&buffer[..count]); }
                Ok(format!("{:x}", digest.finalize()))
            }).await.map_err(|error| AppError::Internal(error.to_string()))??;
            hashes.entry(hash).or_default().push(results.len());
        }
        results.push(ContentIcon { name, icon, icon_key: key, resolved });
    }
    if !known.is_empty() {
        let cf_ids: Vec<_> = known.keys().filter(|(source, _)| source == "curseforge").filter_map(|(_, id)| id.parse::<u64>().ok()).collect();
        let mr_ids: Vec<_> = known.keys().filter(|(source, _)| source != "curseforge").map(|(_, id)| id.clone()).collect();
        let (cf, mr) = tokio::join!(async {
            if cf_ids.is_empty() { return HashMap::new(); }
            tokio::time::timeout(Duration::from_secs(10), crate::core::mods::curseforge::get_mod_logos_batch(&state.http, &cf_ids)).await.unwrap_or_default()
        }, async {
            if mr_ids.is_empty() { return Vec::new(); }
            match state.http.get("https://api.modrinth.com/v2/projects").query(&[("ids", serde_json::to_string(&mr_ids).unwrap_or_default())]).timeout(Duration::from_secs(10)).send().await {
                Ok(response) if response.status().is_success() => response.json::<Vec<serde_json::Value>>().await.unwrap_or_default(),
                _ => Vec::new()
            }
        });
        for ((source, project), indices) in known {
            let url = if source == "curseforge" {
                project.parse::<u64>().ok().and_then(|id| cf.get(&id)).cloned()
            } else {
                mr.iter().find(|entry| entry["id"].as_str() == Some(&project) || entry["slug"].as_str() == Some(&project)).and_then(|entry| entry["icon_url"].as_str()).map(str::to_owned)
            };
            if let Some(url) = url.filter(|url| url.starts_with("https://")) {
                for index in indices {
                    let entry = &mut results[index]; entry.icon = Some(url.clone()); entry.resolved = true;
                    sqlx::query("INSERT INTO mod_icons(key, icon_url) VALUES (?, ?) ON CONFLICT(key) DO UPDATE SET icon_url = excluded.icon_url").bind(&entry.icon_key).bind(&url).execute(db.pool()).await?;
                }
            } else if source == "curseforge" {
                for index in indices {
                    if let Some(record) = records.iter().find(|record| record.project_id == project && record.file_name.trim_end_matches(".disabled") == results[index].name.trim_end_matches(".disabled") && record.sha1.len() == 40 && record.sha1.bytes().all(|byte| byte.is_ascii_hexdigit())) {
                        hashes.entry(record.sha1.to_ascii_lowercase()).or_default().push(index);
                    }
                }
            }
        }
    }
    if lookup && sub_path == "mods" && !hashes.is_empty() {
        let records = crate::db::schema::mods::list_by_profile(&db, &profile_id).await?;
        let exact: Vec<_> = records.iter().filter(|row| row.source == "curseforge" && hashes.contains_key(&row.sha1)).filter_map(|row| row.project_id.parse::<u64>().ok().map(|id| (id, row.sha1.clone()))).collect();
        let ids: Vec<_> = exact.iter().map(|(id,_)| *id).collect();
        if !ids.is_empty() {
            if let Ok(logos) = tokio::time::timeout(Duration::from_secs(10), crate::core::mods::curseforge::get_mod_logos_batch(&state.http, &ids)).await {
                for (id,hash) in exact {
                    if let Some(url) = logos.get(&id).filter(|url| url.starts_with("https://")) {
                        if let Some(indices) = hashes.remove(&hash) {
                          for index in indices {
                            let entry = &mut results[index]; entry.icon = Some(url.clone()); entry.resolved = true;
                            sqlx::query("INSERT INTO mod_icons(key, icon_url) VALUES (?, ?) ON CONFLICT(key) DO UPDATE SET icon_url = excluded.icon_url").bind(&entry.icon_key).bind(url).execute(db.pool()).await?;
                          }
                        }
                    }
                }
            }
        }
    }
    if !hashes.is_empty() {
        let response = state.http.post("https://api.modrinth.com/v2/version_files").timeout(Duration::from_secs(10)).json(&serde_json::json!({"hashes": hashes.keys().collect::<Vec<_>>(), "algorithm": "sha1"})).send().await;
        if let Ok(response) = response {
            if response.status().is_success() {
                if let Ok(versions) = response.json::<HashMap<String, serde_json::Value>>().await {
                    let ids: Vec<_> = versions.values().filter_map(|version| version["project_id"].as_str()).collect();
                    let projects = if ids.is_empty() { Some(Vec::new()) } else {
                        match state.http.get("https://api.modrinth.com/v2/projects").query(&[("ids", serde_json::to_string(&ids).unwrap_or_default())]).timeout(Duration::from_secs(10)).send().await {
                            Ok(response) if response.status().is_success() => response.json::<Vec<serde_json::Value>>().await.ok(),
                            _ => None
                        }
                    };
                    if let Some(projects) = projects {
                        for (hash, indices) in hashes {
                            let project = versions.get(&hash).and_then(|version| version["project_id"].as_str());
                            let icon = projects.iter().find(|entry| entry["id"].as_str() == project).and_then(|entry| entry["icon_url"].as_str()).filter(|url| url.starts_with("https://")).map(String::from);
                            if project.is_some() && icon.is_none() { continue; }
                            for index in indices {
                            let entry = &mut results[index];
                            entry.icon = icon.clone();
                            entry.resolved = true;
                            sqlx::query("INSERT INTO mod_icons(key, icon_url) VALUES (?, ?) ON CONFLICT(key) DO UPDATE SET icon_url = excluded.icon_url").bind(&entry.icon_key).bind(entry.icon.clone().unwrap_or_else(|| format!("missing:{}", now()))).execute(db.pool()).await?;
                            }
                        }
                    }
                }
            }
        }
    }
    if cache_images {
        for entry in &mut results {
            if let Some(source) = entry.icon.as_deref().filter(|source| source.starts_with("https://")) {
                if let Some(image) = cached_image(&db, source).await {
                    let _ = sqlx::query("UPDATE mod_icons SET icon_url = ? WHERE key = ?")
                        .bind(&image).bind(&entry.icon_key).execute(db.pool()).await;
                    entry.icon = Some(image);
                }
            }
        }
    }
    Ok(results)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn extracted_shader_variants_match_actual_code_instead_of_names() {
        use std::io::Write;
        let root = tempfile::tempdir().unwrap();
        let folder = root.path().join("My performance preset");
        std::fs::create_dir_all(folder.join("shaders")).unwrap();
        let source = root.path().join("Author shader.zip");
        let mut archive = zip::ZipWriter::new(std::fs::File::create(&source).unwrap());
        for name in ["a.glsl", "b.fsh", "c.vsh"] {
            let content = format!("shader program {name}");
            archive.start_file(format!("shaders/{name}"),zip::write::FileOptions::default()).unwrap();
            archive.write_all(content.as_bytes()).unwrap();
            std::fs::write(folder.join("shaders").join(name),content).unwrap();
        }
        archive.finish().unwrap();
        assert_eq!(matching_shader_archive(&folder),Some(source));
        std::fs::write(folder.join("shaders/a.glsl"),"another shader").unwrap();
        assert!(matching_shader_archive(&folder).is_none());
    }
    #[test]
    fn thumbnail_sources_and_decoding_are_bounded() {
        assert!(image_url("https://media.forgecdn.net/avatars/test.jpeg").is_some());
        assert!(image_url("https://cdn.modrinth.com/data/test/icon.png").is_some());
        for source in ["http://media.forgecdn.net/a", "https://media.forgecdn.net.evil.test/a", "https://127.0.0.1/a", "https://user@media.forgecdn.net/a", "https://media.forgecdn.net:8443/a"] {
            assert!(image_url(source).is_none());
        }
        assert!(thumbnail(b"not an image").is_none());
        let mut png = std::io::Cursor::new(Vec::new());
        image::DynamicImage::new_rgb8(256, 128).write_to(&mut png, image::ImageFormat::Png).unwrap();
        let encoded = thumbnail(png.get_ref()).unwrap();
        use base64::Engine;
        let bytes = base64::engine::general_purpose::STANDARD.decode(encoded.split_once(',').unwrap().1).unwrap();
        let image = image::load_from_memory(&bytes).unwrap();
        assert_eq!((image.width(), image.height()), (96, 48));
    }
    #[test]
    fn content_paths_and_negative_cache_are_bounded() {
        assert!(allowed("mods")); assert!(allowed("saves/My world/datapacks"));
        assert!(!allowed("saves/../datapacks")); assert!(!allowed("config"));
        assert!(cache_is_resolved(&format!("missing:{}", now())));
        assert!(!cache_is_resolved("missing:0")); assert!(!cache_is_resolved(""));
    }
}
