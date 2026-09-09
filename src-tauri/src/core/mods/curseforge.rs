use super::{ModFile, ModSearchResult, ModVersion};
use crate::error::AppResult;

const CURSEFORGE_API: &str = "https://api.curseforge.com/v1";

pub fn api_key() -> Option<String> {
    std::env::var("CURSEFORGE_API_KEY")
        .ok()
        .filter(|k| !k.is_empty())
}

pub async fn search_mods(
    http: &reqwest::Client,
    query: &str,
    mc_version: &str,
    limit: u32,
) -> AppResult<Vec<ModSearchResult>> {
    let key = match api_key() {
        Some(k) => k,
        None => return Ok(Vec::new()),
    };

    let game_id = 432;
    let class_id = 6;

    let url = format!(
        "{}/mods/search?gameId={}&classId={}&searchFilter={}&gameVersion={}&pageSize={}",
        CURSEFORGE_API,
        game_id,
        class_id,
        urlencoding::encode(query),
        urlencoding::encode(mc_version),
        limit,
    );

    let resp: serde_json::Value = http
        .get(&url)
        .header("x-api-key", &key)
        .send()
        .await?
        .error_for_status()?
        .json()
        .await?;

    let data = resp
        .get("data")
        .and_then(|d| d.as_array())
        .cloned()
        .unwrap_or_default();

    let results = data
        .iter()
        .map(|m| {
            let _id = m.get("id").and_then(|i| i.as_u64()).unwrap_or(0);
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
                        .filter_map(|f| {
                            f.get("gameVersion")
                                .and_then(|v| v.as_str())
                                .map(|s| s.to_string())
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

    let resp: serde_json::Value = http
        .get(&url)
        .header("x-api-key", &key)
        .send()
        .await?
        .error_for_status()?
        .json()
        .await?;

    let data = resp
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
