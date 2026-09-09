use super::LoaderVersion;
use crate::error::AppResult;

const QUILT_META: &str = "https://meta.quiltmc.org/v3";

pub async fn fetch_versions(
    http: &reqwest::Client,
    mc_version: &str,
) -> AppResult<Vec<LoaderVersion>> {
    let url = format!("{}/versions/loader/{}", QUILT_META, mc_version);
    let resp: Vec<serde_json::Value> = http
        .get(&url)
        .send()
        .await?
        .error_for_status()?
        .json()
        .await?;

    let versions = resp
        .iter()
        .filter_map(|v| {
            let version = v.get("version").and_then(|v| v.as_str())?.to_string();
            let stable = v.get("stable").and_then(|s| s.as_bool()).unwrap_or(false);
            Some(LoaderVersion {
                id: version,
                stable,
            })
        })
        .filter(|v| !v.id.is_empty())
        .collect();

    Ok(versions)
}
