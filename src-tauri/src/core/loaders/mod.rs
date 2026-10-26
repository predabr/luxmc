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
    LegacyFabric,
    Babric,
    Forge,
    NeoForge,
    Quilt,
    Vanilla,
    Unknown,
}

#[cfg(test)]
mod additional_loader_tests {
    use super::*;

    #[tokio::test]
    async fn loader_families_keep_separate_cached_profiles_and_pinned_versions() {
        let temporary = tempfile::tempdir().unwrap();
        let root = temporary.path();
        let library = root.join("fixture/library/1/library-1.jar");
        std::fs::create_dir_all(library.parent().unwrap()).unwrap();
        zip::ZipWriter::new(std::fs::File::create(library).unwrap()).finish().unwrap();
        let cache = root.join(".luxmc-fabric");
        std::fs::create_dir_all(&cache).unwrap();
        let http = reqwest::Client::builder().timeout(std::time::Duration::from_millis(50)).build().unwrap();
        for (loader, meta, minecraft) in [("legacyfabric", "https://meta.legacyfabric.net/v2", "1.8.9"), ("babric", "https://meta.babric.glass-launcher.net/v2", "b1.7.3")] {
            let url = format!("{meta}/versions/loader/{minecraft}/0.20.1/profile/json");
            let key = format!("{:016x}.json", xxhash_rust::xxh3::xxh3_64(url.as_bytes()));
            let main = format!("fixture.{loader}.Client");
            std::fs::write(cache.join(key), serde_json::to_vec(&serde_json::json!({"id":"fixture","mainClass":main,"libraries":[{"name":"fixture:library:1"}]})).unwrap()).unwrap();
            let result = prepare_loader(&http, root, loader, minecraft, Some("0.20.1")).await.unwrap();
            assert_eq!(result.main_class, main);
            assert_eq!(result.classpath_entries.len(), 1);
        }
        assert!(fetch_loader_versions(&http, "unknown-loader", "1.21.1").await.is_err());
    }

    #[tokio::test]
    #[ignore = "requires the official Legacy Fabric and Babric endpoints"]
    async fn official_legacy_loader_profiles_prepare_valid_archives() {
        let http = reqwest::Client::builder().user_agent("Luxmc/4.0.0 (+https://github.com/predabr/luxmc)").timeout(std::time::Duration::from_secs(20)).build().unwrap();
        let root = tempfile::tempdir().unwrap();
        for (loader, minecraft) in [("legacyfabric", "1.8.9"), ("babric", "b1.7.3")] {
            let versions = fetch_loader_versions(&http, loader, minecraft).await.unwrap();
            let version = versions.first().unwrap();
            let result = prepare_loader(&http, root.path(), loader, minecraft, Some(&version.id)).await.unwrap();
            assert!(result.main_class.contains("KnotClient"));
            assert!(!result.classpath_entries.is_empty());
            assert!(result.classpath_entries.iter().all(|path| is_valid_jar(path)));
            assert!(result.native_archives.iter().all(|path| is_valid_jar(path)));
            if loader == "legacyfabric" { assert!(!result.native_archives.is_empty()); }
        }
    }
}

impl LoaderKind {
    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "fabric" => LoaderKind::Fabric,
            "legacyfabric" => LoaderKind::LegacyFabric,
            "babric" => LoaderKind::Babric,
            "forge" => LoaderKind::Forge,
            "neoforge" => LoaderKind::NeoForge,
            "quilt" => LoaderKind::Quilt,
            "vanilla" => LoaderKind::Vanilla,
            _ => LoaderKind::Unknown,
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
        LoaderKind::LegacyFabric => fabric::fetch_versions_from(http, mc_version, "https://meta.legacyfabric.net/v2").await,
        LoaderKind::Babric => fabric::fetch_versions_from(http, mc_version, "https://meta.babric.glass-launcher.net/v2").await,
        LoaderKind::Forge => forge::fetch_versions(http, mc_version).await,
        LoaderKind::NeoForge => neoforge::fetch_versions(http, mc_version).await,
        LoaderKind::Quilt => quilt::fetch_versions(http, mc_version).await,
        LoaderKind::Vanilla => Ok(Vec::new()),
        LoaderKind::Unknown => Err(crate::error::AppError::InvalidInput(format!("Loader não suportado: {loader}"))),
    }
}

pub async fn supported_game_versions(http: &reqwest::Client, loader: &str) -> AppResult<Option<Vec<String>>> {
    let meta = match LoaderKind::from_str(loader) {
        LoaderKind::LegacyFabric => "https://meta.legacyfabric.net/v2",
        LoaderKind::Babric => "https://meta.babric.glass-launcher.net/v2",
        _ => return Ok(None),
    };
    Ok(Some(fabric::supported_game_versions(http, meta).await?))
}

#[derive(Debug, Clone)]
pub struct PreparedLoader {
    pub main_class: String,
    pub classpath_entries: Vec<std::path::PathBuf>,
    pub native_archives: Vec<std::path::PathBuf>,
    pub jvm_args: Vec<String>,
    pub game_args: Vec<String>,
}

pub fn prepare_loader<'a>(
    http: &'a reqwest::Client,
    libraries_dir: &'a std::path::Path,
    loader: &'a str,
    mc_version: &'a str,
    loader_version: Option<&'a str>,
) -> futures_util::future::BoxFuture<'a, AppResult<PreparedLoader>> {
    match LoaderKind::from_str(loader) {
        LoaderKind::Fabric => Box::pin(fabric::prepare_fabric(http, libraries_dir, mc_version, loader_version)),
        LoaderKind::LegacyFabric => Box::pin(fabric::prepare_from_meta(http, libraries_dir, mc_version, loader_version, "https://meta.legacyfabric.net/v2")),
        LoaderKind::Babric => Box::pin(fabric::prepare_from_meta(http, libraries_dir, mc_version, loader_version, "https://meta.babric.glass-launcher.net/v2")),
        LoaderKind::Forge => Box::pin(forge::prepare_forge(http, libraries_dir, mc_version, loader_version)),
        LoaderKind::Quilt => Box::pin(quilt::prepare_quilt(http, libraries_dir, mc_version, loader_version)),
        LoaderKind::NeoForge => Box::pin(neoforge::prepare_neoforge(http, libraries_dir, mc_version, loader_version)),
        _ => Box::pin(async move { Err(crate::error::AppError::NotFound(format!("Loader '{}' not supported for auto-injection", loader))) }),
    }
}

mod installer;

#[cfg(target_os = "linux")]
mod installer_java;

fn is_valid_jar(path: &std::path::Path) -> bool {
    let file = match std::fs::File::open(path) {
        Ok(f) => f,
        Err(_) => return false,
    };
    let meta = match file.metadata() {
        Ok(m) => m,
        Err(_) => return false,
    };
    if meta.len() < 22 {
        return false;
    }
    zip::ZipArchive::new(file).is_ok()
}

async fn ensure_maven_jar(http: &reqwest::Client, path: &std::path::Path, url: &str) -> crate::error::AppResult<()> {
    if path.exists() && is_valid_jar(path) {
        return Ok(());
    }
    let sha = match http.get(format!("{url}.sha1")).timeout(std::time::Duration::from_secs(6)).send().await {
        Ok(resp) if resp.status().is_success() => {
            if let Ok(bytes) = resp.bytes().await {
                let text = String::from_utf8_lossy(&bytes);
                let hash = text.split_whitespace().next().unwrap_or("").to_owned();
                if hash.len() == 40 && hash.bytes().all(|b| b.is_ascii_hexdigit()) {
                    hash.to_ascii_lowercase()
                } else {
                    String::new()
                }
            } else {
                String::new()
            }
        }
        _ => String::new(),
    };
    crate::core::downloader::ensure_artifact(http, path, url, 0, &sha).await
}
