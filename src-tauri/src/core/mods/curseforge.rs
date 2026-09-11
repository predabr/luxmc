use std::sync::OnceLock;
use std::time::{Duration, Instant};

use super::{ModAuthor, ModFile, ModGalleryImage, ModProjectDetails, ModSearchResult, ModVersion};
use crate::error::AppResult;

const CURSEFORGE_API: &str = "https://api.curseforge.com/v1";
const KEYRING_SERVICE: &str = "io.github.Luxmc";
const KEYRING_USER: &str = "curseforge_api_key";
const MIN_KEY_LENGTH: usize = 20;
const MAX_KEY_LENGTH: usize = 128;

static API_KEY_CACHE: OnceLock<std::sync::Mutex<Option<(String, Instant)>>> = OnceLock::new();

fn keyring_entry() -> Result<keyring::Entry, String> {
    keyring::Entry::new(KEYRING_SERVICE, KEYRING_USER)
        .or_else(|_| {
            keyring::Entry::new_with_target(
                &format!("{}-{}", KEYRING_SERVICE, KEYRING_USER),
                KEYRING_SERVICE,
                KEYRING_USER,
            )
        })
        .map_err(|e| format!("Keyring error: {e}"))
}

fn is_valid_curseforge_key(key: &str) -> bool {
    let trimmed = key.trim();
    if trimmed.is_empty() || trimmed.len() < MIN_KEY_LENGTH || trimmed.len() > MAX_KEY_LENGTH {
        return false;
    }
    if trimmed.chars().any(|c| c.is_ascii_whitespace()) {
        return false;
    }
    true
}

fn get_cached_key() -> Option<String> {
    let cache = API_KEY_CACHE.get_or_init(|| std::sync::Mutex::new(None));
    let guard = cache.lock().unwrap();
    if let Some((ref key, ts)) = *guard {
        if ts.elapsed() < Duration::from_secs(300) {
            return Some(key.clone());
        }
    }
    None
}

fn set_cached_key(key: String) {
    let cache = API_KEY_CACHE.get_or_init(|| std::sync::Mutex::new(None));
    let mut guard = cache.lock().unwrap();
    *guard = Some((key, Instant::now()));
}

fn clear_cached_key() {
    let cache = API_KEY_CACHE.get_or_init(|| std::sync::Mutex::new(None));
    let mut guard = cache.lock().unwrap();
    *guard = None;
}

pub fn api_key() -> Option<String> {
    if let Some(key) = get_cached_key() {
        return Some(key);
    }

    if let Ok(entry) = keyring_entry() {
        if let Ok(key) = entry.get_password() {
            let trimmed = key.trim().to_string();
            if is_valid_curseforge_key(&trimmed) {
                set_cached_key(trimmed.clone());
                return Some(trimmed);
            }
        }
    }

    if let Ok(k) = std::env::var("CURSEFORGE_API_KEY") {
        let trimmed = k.trim().to_string();
        if is_valid_curseforge_key(&trimmed) {
            set_cached_key(trimmed.clone());
            return Some(trimmed);
        }
    }

    let mut search_paths: Vec<std::path::PathBuf> = Vec::new();

    if let Ok(exe) = std::env::current_exe() {
        if let Some(exe_dir) = exe.parent() {
            search_paths.push(exe_dir.join(".env"));
            search_paths.push(exe_dir.parent().unwrap_or(exe_dir).join(".env"));
            search_paths.push(exe_dir.parent().unwrap_or(exe_dir).parent().unwrap_or(exe_dir).join(".env"));
        }
    }

    if let Ok(cwd) = std::env::current_dir() {
        search_paths.push(cwd.join(".env"));
        search_paths.push(cwd.parent().unwrap_or(&cwd).join(".env"));
    }

    if let Some(config_dir) = directories::ProjectDirs::from("io", "github", "Luxmc") {
        search_paths.push(config_dir.config_dir().join(".env"));
        search_paths.push(config_dir.data_dir().join(".env"));
    }

    search_paths.dedup();

    for env_path in &search_paths {
        if let Ok(content) = std::fs::read_to_string(env_path) {
            for line in content.lines() {
                let trimmed = line.trim();
                if trimmed.starts_with('#') || trimmed.is_empty() {
                    continue;
                }
                if let Some(val) = trimmed.strip_prefix("CURSEFORGE_API_KEY=") {
                    let cleaned = val.trim().trim_matches('"').trim_matches('\'').to_string();
                    if is_valid_curseforge_key(&cleaned) {
                        tracing::info!(path = %env_path.display(), "CurseForge API key loaded");
                        set_cached_key(cleaned.clone());
                        return Some(cleaned);
                    }
                }
            }
        }
    }

    None
}

pub fn store_key(key: &str) -> Result<(), String> {
    let trimmed = key.trim();
    if !is_valid_curseforge_key(trimmed) {
        return Err(format!(
            "Invalid key: must be {}-{} alphanumeric characters, no whitespace",
            MIN_KEY_LENGTH, MAX_KEY_LENGTH
        ));
    }
    let entry = keyring_entry()?;
    entry.set_password(trimmed).map_err(|e| format!("Keyring write error: {e}"))?;
    set_cached_key(trimmed.to_string());
    Ok(())
}

pub fn remove_key() -> Result<(), String> {
    let entry = keyring_entry()?;
    entry.delete_credential().map_err(|e| format!("Keyring delete error: {e}"))?;
    clear_cached_key();
    Ok(())
}

pub fn has_key() -> bool {
    api_key().is_some()
}

pub async fn validate_key(client: &reqwest::Client) -> Result<bool, String> {
    let key = api_key().ok_or("No CurseForge API key configured")?;
    let resp = client
        .get(format!("{}/mods/search?gameId=432&pageSize=1&index=0", CURSEFORGE_API))
        .header("x-api-key", &key)
        .send()
        .await
        .map_err(|e| format!("Network error: {e}"))?;

    match resp.status().as_u16() {
        200 => Ok(true),
        403 => Err("Invalid API key (403 Forbidden)".into()),
        429 => Err("Rate limited (429) — try again later".into()),
        code => Err(format!("Unexpected status: {code}")),
    }
}

pub async fn search_mods(
    http: &reqwest::Client,
    query: &str,
    mc_version: &str,
    content_type: &str,
    loader: Option<&str>,
    limit: u32,
    offset: u32,
    sort_by: Option<&str>,
) -> AppResult<Vec<ModSearchResult>> {
    let key = match api_key() {
        Some(k) => k,
        None => return Ok(Vec::new()),
    };

    let game_id = 432;
    let class_id = match content_type.to_lowercase().as_str() {
        "modpack" => 4471,
        "resource pack" | "resourcepack" => 12,
        "shader" => 6552,
        "data pack" | "datapack" => 6945,
        "world" => 17,
        _ => 6,
    };

    let version_filter = if mc_version.is_empty() || mc_version == "Qualquer Versão" {
        "".to_string()
    } else {
        format!("&gameVersion={}", urlencoding::encode(mc_version))
    };

    let loader_param = match loader.map(|s| s.to_lowercase()).as_deref() {
        Some("forge") => "&modLoaderType=1",
        Some("fabric") => "&modLoaderType=4",
        Some("quilt") => "&modLoaderType=5",
        Some("neoforge") => "&modLoaderType=6",
        _ => "",
    };

    let sort_field = match sort_by.unwrap_or("downloads") {
        "relevance" => 1,
        "updated" => 3,
        "newest" => 4,
        _ => 2,
    };
    let sort_param = format!("&sortField={}&sortOrder=desc", sort_field);

    let url = format!(
        "{}/mods/search?gameId={}&classId={}&searchFilter={}{}{}&index={}&pageSize={}{}",
        CURSEFORGE_API,
        game_id,
        class_id,
        urlencoding::encode(query.trim()),
        version_filter,
        loader_param,
        offset,
        limit,
        sort_param,
    );

    let resp = http
        .get(&url)
        .header("x-api-key", &key)
        .timeout(Duration::from_secs(15))
        .send()
        .await?;

    if !resp.status().is_success() {
        let status = resp.status();
        if status.as_u16() == 403 {
            clear_cached_key();
            tracing::warn!("CurseForge API key rejected (403) — key cleared from cache");
        } else if status.as_u16() == 429 {
            tracing::warn!("CurseForge rate limited (429) — retrying later");
        } else {
            tracing::warn!(status = %status, "CurseForge search failed");
        }
        return Ok(Vec::new());
    }

    let body: serde_json::Value = resp.json().await?;

    let data = body
        .get("data")
        .and_then(|d| d.as_array())
        .cloned()
        .unwrap_or_default();

    let results = data
        .iter()
        .map(|m| {
            let cf_id = m.get("id").and_then(|i| i.as_u64()).unwrap_or(0);
            let slug = m
                .get("slug")
                .and_then(|s| s.as_str())
                .unwrap_or("")
                .to_string();
            let name = m
                .get("name")
                .and_then(|n| n.as_str())
                .unwrap_or("")
                .to_string();
            let summary = m
                .get("summary")
                .and_then(|s| s.as_str())
                .unwrap_or("")
                .to_string();
            let downloads = m.get("downloadCount").and_then(|d| d.as_u64()).unwrap_or(0);

            let logo = m.get("logo").and_then(|l| l.as_object());
            let icon_url = logo
                .and_then(|l| l.get("thumbnailUrl"))
                .and_then(|u| u.as_str())
                .map(|s| s.to_string());

            let banner_url = m
                .get("screenshots")
                .and_then(|s| s.as_array())
                .and_then(|arr| arr.first())
                .and_then(|sc| sc.get("url"))
                .and_then(|u| u.as_str())
                .map(|s| s.to_string());

            let author = m
                .get("authors")
                .and_then(|a| a.as_array())
                .and_then(|arr| arr.first())
                .and_then(|au| au.get("name"))
                .and_then(|n| n.as_str())
                .map(|s| s.to_string());

            let categories: Vec<String> = m
                .get("categories")
                .and_then(|c| c.as_array())
                .map(|arr| {
                    arr.iter()
                        .filter_map(|c| {
                            c.get("name")
                                .and_then(|n| n.as_str())
                                .map(|s| s.to_string())
                        })
                        .collect()
                })
                .unwrap_or_default();

            let versions: Vec<String> = m
                .get("latestFiles")
                .and_then(|f| f.as_array())
                .map(|arr| {
                    arr.iter()
                        .flat_map(|f| {
                            f.get("gameVersions")
                                .and_then(|gv| gv.as_array())
                                .map(|varr| {
                                    varr.iter()
                                        .filter_map(|v| v.as_str().map(|s| s.to_string()))
                                        .collect::<Vec<_>>()
                                })
                                .unwrap_or_default()
                        })
                        .collect::<std::collections::HashSet<_>>()
                        .into_iter()
                        .collect()
                })
                .unwrap_or_default();

            ModSearchResult {
                slug,
                title: name,
                description: summary,
                downloads,
                icon_url,
                banner_url,
                author,
                categories,
                versions,
                source: "curseforge".into(),
                source_id: cf_id.to_string(),
            }
        })
        .collect();

    Ok(results)
}

pub async fn get_mod_versions(
    http: &reqwest::Client,
    project_id: &str,
    mc_version: &str,
) -> AppResult<Vec<ModVersion>> {
    let key = match api_key() {
        Some(k) => k,
        None => return Ok(Vec::new()),
    };

    let version_param = if mc_version.is_empty() || mc_version == "Qualquer Versão" {
        String::new()
    } else {
        format!("&gameVersion={}", urlencoding::encode(mc_version))
    };

    let url = format!(
        "{}/mods/{}/files?{}",
        CURSEFORGE_API, project_id, version_param
    );

    let resp = http
        .get(&url)
        .header("x-api-key", &key)
        .timeout(Duration::from_secs(15))
        .send()
        .await?;

    if !resp.status().is_success() {
        let status = resp.status();
        if status.as_u16() == 403 {
            clear_cached_key();
        }
        tracing::warn!(
            status = %status,
            project_id = %project_id,
            "CurseForge get_mod_versions failed"
        );
        return Ok(Vec::new());
    }

    let body: serde_json::Value = resp.json().await?;

    let data = body
        .get("data")
        .and_then(|d| d.as_array())
        .cloned()
        .unwrap_or_default();

    let versions: Vec<ModVersion> = data
        .iter()
        .map(|f| {
            let id = f
                .get("id")
                .and_then(|i| i.as_u64())
                .unwrap_or(0)
                .to_string();
            let display = f
                .get("displayName")
                .and_then(|n| n.as_str())
                .unwrap_or("")
                .to_string();
            let download_url = f
                .get("downloadUrl")
                .and_then(|u| u.as_str())
                .unwrap_or("")
                .to_string();
            let file_name = f
                .get("fileName")
                .and_then(|n| n.as_str())
                .unwrap_or("")
                .to_string();
            let file_length = f.get("fileLength").and_then(|s| s.as_u64()).unwrap_or(0);

            ModVersion {
                id,
                name: display,
                version_number: String::new(),
                files: vec![ModFile {
                    url: download_url,
                    filename: file_name,
                    size: file_length,
                    sha1: String::new(),
                }],
            }
        })
        .filter(|v| !v.files.is_empty() && !v.files[0].url.is_empty())
        .collect();

    Ok(versions)
}

pub async fn get_mod_details(
    http: &reqwest::Client,
    mod_id: &str,
) -> AppResult<ModProjectDetails> {
    let key = match api_key() {
        Some(k) => k,
        None => return Err(crate::error::AppError::NotFound("CurseForge API key missing".into())),
    };

    let url = format!("{}/mods/{}", CURSEFORGE_API, mod_id);
    let resp = http
        .get(&url)
        .header("x-api-key", &key)
        .timeout(Duration::from_secs(15))
        .send()
        .await?
        .error_for_status()?;
    let body: serde_json::Value = resp.json().await?;
    let m = body
        .get("data")
        .ok_or_else(|| crate::error::AppError::NotFound("Mod not found on CurseForge".into()))?;

    let id = m.get("id").and_then(|i| i.as_u64()).unwrap_or(0).to_string();
    let slug = m.get("slug").and_then(|s| s.as_str()).unwrap_or("").to_string();
    let title = m.get("name").and_then(|n| n.as_str()).unwrap_or("").to_string();
    let description = m.get("summary").and_then(|s| s.as_str()).unwrap_or("").to_string();
    let downloads = m.get("downloadCount").and_then(|d| d.as_u64()).unwrap_or(0);
    let created_at = m.get("dateCreated").and_then(|d| d.as_str()).map(|s| s.to_string());
    let updated_at = m.get("dateModified").and_then(|d| d.as_str()).map(|s| s.to_string());

    let logo = m.get("logo").and_then(|l| l.as_object());
    let icon_url = logo
        .and_then(|l| l.get("thumbnailUrl"))
        .and_then(|u| u.as_str())
        .map(|s| s.to_string());

    let categories: Vec<String> = m
        .get("categories")
        .and_then(|c| c.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|c| c.get("name").and_then(|n| n.as_str()).map(|s| s.to_string()))
                .collect()
        })
        .unwrap_or_default();

    let authors_arr = m.get("authors").and_then(|a| a.as_array());
    let author = authors_arr.and_then(|a| a.first()).map(|a| {
        let name = a.get("name").and_then(|n| n.as_str()).unwrap_or("Autor").to_string();
        let avatar_url = a.get("avatarUrl").and_then(|u| u.as_str()).map(|s| s.to_string());
        ModAuthor {
            name,
            avatar_url,
            role: Some("Autor".to_string()),
        }
    });

    let links = m.get("links").and_then(|l| l.as_object());
    let source_url = links
        .and_then(|l| l.get("sourceUrl"))
        .and_then(|u| u.as_str())
        .map(|s| s.to_string());
    let issues_url = links
        .and_then(|l| l.get("issuesUrl"))
        .and_then(|u| u.as_str())
        .map(|s| s.to_string());
    let wiki_url = links
        .and_then(|l| l.get("wikiUrl"))
        .and_then(|u| u.as_str())
        .map(|s| s.to_string());
    let donation_url = links
        .and_then(|l| l.get("websiteUrl"))
        .and_then(|u| u.as_str())
        .map(|s| s.to_string());

    let gallery: Vec<ModGalleryImage> = m
        .get("screenshots")
        .and_then(|s| s.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|s| {
                    let url = s.get("url")?.as_str()?.to_string();
                    let title = s.get("title").and_then(|t| t.as_str()).map(|s| s.to_string());
                    let description = s
                        .get("description")
                        .and_then(|d| d.as_str())
                        .map(|s| s.to_string());
                    Some(ModGalleryImage {
                        url,
                        title,
                        description,
                    })
                })
                .collect()
        })
        .unwrap_or_default();

    let mut game_versions = Vec::new();
    let mut loaders = Vec::new();
    if let Some(indexes) = m.get("latestFilesIndexes").and_then(|i| i.as_array()) {
        for idx in indexes {
            if let Some(gv) = idx.get("gameVersion").and_then(|g| g.as_str()) {
                if !game_versions.contains(&gv.to_string()) {
                    game_versions.push(gv.to_string());
                }
            }
            if let Some(mod_loader) = idx.get("modLoader").and_then(|l| l.as_u64()) {
                let loader_name = match mod_loader {
                    1 => "Forge",
                    4 => "Fabric",
                    5 => "Quilt",
                    6 => "NeoForge",
                    _ => "",
                };
                if !loader_name.is_empty() && !loaders.contains(&loader_name.to_string()) {
                    loaders.push(loader_name.to_string());
                }
            }
        }
    }

    let desc_url = format!("{}/mods/{}/description", CURSEFORGE_API, mod_id);
    let body_html = if let Ok(d_resp) = http
        .get(&desc_url)
        .header("x-api-key", &key)
        .timeout(Duration::from_secs(10))
        .send()
        .await
    {
        if let Ok(d_json) = d_resp.json::<serde_json::Value>().await {
            d_json.get("data").and_then(|d| d.as_str()).unwrap_or("").to_string()
        } else {
            description.clone()
        }
    } else {
        description.clone()
    };

    Ok(ModProjectDetails {
        id,
        slug,
        title,
        description,
        body: body_html,
        body_type: "html".to_string(),
        icon_url,
        downloads,
        categories,
        loaders,
        game_versions,
        latest_version: None,
        updated_at,
        created_at,
        source: "curseforge".to_string(),
        source_url,
        issues_url,
        discord_url: None,
        wiki_url,
        donation_url,
        author,
        gallery,
    })
}

pub async fn get_download_url(
    http: &reqwest::Client,
    project_id: &str,
    file_id: &str,
) -> AppResult<String> {
    let key = match api_key() {
        Some(k) => k,
        None => return Err(crate::error::AppError::NotFound("CurseForge API key missing".into())),
    };

    let url = format!(
        "{}/mods/{}/files/{}/download-url",
        CURSEFORGE_API, project_id, file_id
    );

    let resp = http
        .get(&url)
        .header("x-api-key", &key)
        .timeout(Duration::from_secs(15))
        .send()
        .await?
        .error_for_status()?;
    let body: serde_json::Value = resp.json().await?;

    body.get("data")
        .and_then(|d| d.as_str())
        .filter(|s| !s.is_empty())
        .map(|s| s.to_string())
        .ok_or_else(|| crate::error::AppError::NotFound("Download URL not found on CurseForge".into()))
}

pub async fn get_mod_names_batch(
    http: &reqwest::Client,
    mod_ids: &[u64],
) -> std::collections::HashMap<u64, String> {
    let key = match api_key() {
        Some(k) => k,
        None => return std::collections::HashMap::new(),
    };

    let mut names = std::collections::HashMap::new();
    for chunk in mod_ids.chunks(50) {
        let ids_str: Vec<String> = chunk.iter().map(|id| id.to_string()).collect();
        let url = format!("{}/mods/{}", CURSEFORGE_API, ids_str.join(","));
        if let Ok(resp) = http
            .get(&url)
            .header("x-api-key", &key)
            .timeout(Duration::from_secs(20))
            .send()
            .await
        {
            if let Ok(body) = resp.json::<serde_json::Value>().await {
                if let Some(data) = body.get("data").and_then(|d| d.as_array()) {
                    for m in data {
                        if let Some(id) = m.get("id").and_then(|i| i.as_u64()) {
                            if let Some(name) = m.get("name").and_then(|n| n.as_str()) {
                                names.insert(id, name.to_string());
                            }
                        }
                    }
                }
            }
        }
    }
    names
}
