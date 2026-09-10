use super::{ModAuthor, ModFile, ModGalleryImage, ModProjectDetails, ModSearchResult, ModVersion};
use crate::error::AppResult;

const CURSEFORGE_API: &str = "https://api.curseforge.com/v1";

pub fn api_key() -> Option<String> {
    let from_env = std::env::var("CURSEFORGE_API_KEY")
        .ok()
        .filter(|k| !k.is_empty());

    if let Some(k) = from_env {
        tracing::info!(
            "CurseForge API key loaded from environment ({}…)",
            &k[..k.len().min(8)]
        );
        return Some(k);
    }

    tracing::warn!("CurseForge API key not configured (set CURSEFORGE_API_KEY) — CurseForge results will be omitted");
    None
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
        .send()
        .await?;

    if !resp.status().is_success() {
        tracing::warn!(
            status = %resp.status(),
            "CurseForge search request failed — omitting CurseForge results"
        );
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

    let url = format!(
        "{}/mods/{}/files?gameVersion={}",
        CURSEFORGE_API, project_id, mc_version
    );

    let resp = http
        .get(&url)
        .header("x-api-key", &key)
        .send()
        .await?;

    if !resp.status().is_success() {
        tracing::warn!(
            status = %resp.status(),
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
    let body_html = if let Ok(d_resp) = http.get(&desc_url).header("x-api-key", &key).send().await {
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
