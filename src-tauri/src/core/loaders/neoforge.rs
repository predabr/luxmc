use crate::error::AppResult;
use super::LoaderVersion;

const NEOFORGED_API: &str = "https://api.neoforged.net/v2";

pub async fn fetch_versions(
	http: &reqwest::Client,
	mc_version: &str,
) -> AppResult<Vec<LoaderVersion>> {
	let url = format!("{}/versions/loader/{}", NEOFORGED_API, mc_version);
	let resp: Vec<serde_json::Value> = http.get(&url).send().await?.error_for_status()?.json().await?;

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
