use crate::error::AppResult;
use super::LoaderVersion;

const FABRIC_META: &str = "https://meta.fabricmc.net/v2";

pub async fn fetch_versions(
	http: &reqwest::Client,
	mc_version: &str,
) -> AppResult<Vec<LoaderVersion>> {
	let url = format!("{}/versions/loader/{}", FABRIC_META, mc_version);
	let resp: Vec<serde_json::Value> = http.get(&url).send().await?.error_for_status()?.json().await?;

	let versions = resp
		.iter()
		.map(|v| {
			let loader = v.get("loader").and_then(|l| l.as_object());
			let version = loader
				.and_then(|l| l.get("version"))
				.and_then(|v| v.as_str())
				.unwrap_or("")
				.to_string();
			let stable = loader
				.and_then(|l| l.get("stable"))
				.and_then(|s| s.as_bool())
				.unwrap_or(false);
			LoaderVersion {
				id: version,
				stable,
			}
		})
		.filter(|v| !v.id.is_empty())
		.collect();

	Ok(versions)
}
