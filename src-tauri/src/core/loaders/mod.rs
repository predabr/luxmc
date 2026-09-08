use serde::{Deserialize, Serialize};

use crate::error::AppResult;

pub mod fabric;
pub mod neoforge;
pub mod quilt;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LoaderVersion {
	pub id: String,
	pub stable: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LoaderInstallResult {
	pub loader_version: String,
	pub main_class: String,
	pub libraries: Vec<String>,
}

pub enum LoaderKind {
	Fabric,
	NeoForge,
	Quilt,
	Vanilla,
}

impl LoaderKind {
	pub fn from_str(s: &str) -> Self {
		match s.to_lowercase().as_str() {
			"fabric" => LoaderKind::Fabric,
			"neoforge" => LoaderKind::NeoForge,
			"quilt" => LoaderKind::Quilt,
			_ => LoaderKind::Vanilla,
		}
	}
}

pub async fn fetch_loader_versions(
	http: &reqwest::Client,
	loader: &str,
	mc_version: &str,
) -> AppResult<Vec<LoaderVersion>> {
	match LoaderKind::from_str(loader) {
		LoaderKind::Fabric => fabric::fetch_versions(http, mc_version).await,
		LoaderKind::NeoForge => neoforge::fetch_versions(http, mc_version).await,
		LoaderKind::Quilt => quilt::fetch_versions(http, mc_version).await,
		LoaderKind::Vanilla => Ok(Vec::new()),
	}
}
