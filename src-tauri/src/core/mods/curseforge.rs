use super::{ModFile, ModSearchResult, ModVersion};
use crate::error::AppResult;

const CURSEFORGE_API: &str = "https://api.curseforge.com/v1";
const EMBEDDED_KEY: &str = "$2a$10$ZAl3a4/dJg9zsqJ2FJ.S/O0eEnOCDlPfAH81irnXt9GwXIsELgPuq";

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

    if !EMBEDDED_KEY.is_empty() {
        tracing::info!("CurseForge API key loaded from embedded default");
        return Some(EMBEDDED_KEY.to_string());
    }

    tracing::warn!("CurseForge API key not available — CurseForge results will be omitted");
    None
}

pub async fn search_mods(
    http: &reqwest::Client,
    query: &str,
    mc_version: &str,
    content_type: &str,
    limit: u32,
    offset: u32,
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

    let sort_param = if query.trim().is_empty() {
        "&sortField=2&sortOrder=desc"
    } else {
        "&sortField=1&sortOrder=desc"
    };

    let url = format!(
        "{}/mods/search?gameId={}&classId={}&searchFilter={}{}&index={}&pageSize={}{}",
        CURSEFORGE_API,
        game_id,
        class_id,
        urlencoding::encode(query.trim()),
        version_filter,
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
