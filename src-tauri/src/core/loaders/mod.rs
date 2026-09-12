use serde::{Deserialize, Serialize};

use crate::error::AppResult;

pub mod fabric;
pub mod forge;
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
    Forge,
    NeoForge,
    Quilt,
    Vanilla,
}

impl LoaderKind {
    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "fabric" => LoaderKind::Fabric,
            "forge" => LoaderKind::Forge,
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
        LoaderKind::Forge => forge::fetch_versions(http, mc_version).await,
        LoaderKind::NeoForge => neoforge::fetch_versions(http, mc_version).await,
        LoaderKind::Quilt => quilt::fetch_versions(http, mc_version).await,
        LoaderKind::Vanilla => Ok(Vec::new()),
    }
}

#[derive(Debug, Clone)]
pub struct PreparedLoader {
    pub main_class: String,
    pub classpath_entries: Vec<std::path::PathBuf>,
    pub jvm_args: Vec<String>,
}

pub async fn prepare_loader(
    http: &reqwest::Client,
    libraries_dir: &std::path::Path,
    loader: &str,
    mc_version: &str,
    loader_version: Option<&str>,
) -> AppResult<PreparedLoader> {
    match LoaderKind::from_str(loader) {
        LoaderKind::Fabric => fabric::prepare_fabric(http, libraries_dir, mc_version, loader_version).await,
        LoaderKind::Forge => forge::prepare_forge(http, libraries_dir, mc_version, loader_version).await,
        LoaderKind::Quilt => quilt::prepare_quilt(http, libraries_dir, mc_version, loader_version).await,
        LoaderKind::NeoForge => neoforge::prepare_neoforge(http, libraries_dir, mc_version, loader_version).await,
        _ => Err(crate::error::AppError::NotFound(format!("Loader '{}' not supported for auto-injection", loader))),
    }
}
