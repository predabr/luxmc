use super::{LoaderVersion, PreparedLoader};
use crate::error::AppResult;

const QUILT_META: &str = "https://meta.quiltmc.org/v3";

#[derive(Debug, Clone, serde::Deserialize)]
pub struct QuiltLibrary {
    pub name: String,
    pub url: Option<String>,
}

#[derive(Debug, Clone, serde::Deserialize)]
pub struct QuiltArguments {
    #[serde(default)]
    pub jvm: Vec<serde_json::Value>,
    #[serde(default)]
    pub game: Vec<serde_json::Value>,
}

#[derive(Debug, Clone, serde::Deserialize)]
pub struct QuiltProfile {
    pub id: String,
    #[serde(rename = "mainClass")]
    pub main_class: String,
    #[serde(default)]
    pub arguments: Option<QuiltArguments>,
    pub libraries: Vec<QuiltLibrary>,
}

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

pub async fn prepare_quilt(
    http: &reqwest::Client,
    libraries_dir: &std::path::Path,
    mc_version: &str,
    loader_version: Option<&str>,
) -> AppResult<PreparedLoader> {
    let chosen_version = if let Some(v) = loader_version {
        if !v.trim().is_empty() {
            v.trim().to_string()
        } else {
            get_latest_loader_version(http, mc_version).await?
        }
    } else {
        get_latest_loader_version(http, mc_version).await?
    };

    let profile_url = format!(
        "{}/versions/loader/{}/{}/profile/json",
        QUILT_META, mc_version, chosen_version
    );

    let profile: QuiltProfile = http
        .get(&profile_url)
        .send()
        .await?
        .error_for_status()?
        .json()
        .await?;

    let mut classpath_entries = Vec::new();

    for lib in &profile.libraries {
        let rel_path = crate::core::launcher::lib_path_from_name(&std::path::PathBuf::new(), &lib.name);
        let dest = libraries_dir.join(&rel_path);

        if !dest.exists() {
            if let Some(parent) = dest.parent() {
                tokio::fs::create_dir_all(parent).await?;
            }

            let base_url = lib.url.as_deref().unwrap_or("https://maven.quiltmc.org/repository/release/");
            let rel_str = rel_path.to_string_lossy().replace('\\', "/");
            let download_url = format!("{}/{}", base_url.trim_end_matches('/'), rel_str.trim_start_matches('/'));

            tracing::info!(lib = %lib.name, url = %download_url, "downloading Quilt library");

            let mut downloaded = false;
            if let Ok(resp) = http.get(&download_url).send().await {
                if resp.status().is_success() {
                    if let Ok(bytes) = resp.bytes().await {
                        if !bytes.is_empty() {
                            let _ = tokio::fs::write(&dest, &bytes).await;
                            downloaded = true;
                        }
                    }
                }
            }

            if !downloaded {
                let central_url = format!("https://repo1.maven.org/maven2/{}", rel_str.trim_start_matches('/'));
                if let Ok(resp) = http.get(&central_url).send().await {
                    if resp.status().is_success() {
                        if let Ok(bytes) = resp.bytes().await {
                            let _ = tokio::fs::write(&dest, &bytes).await;
                        }
                    }
                }
            }
        }

        if dest.exists() {
            classpath_entries.push(dest);
        }
    }

    let mut jvm_args = Vec::new();
    if let Some(args) = profile.arguments {
        for arg in args.jvm {
            if let Some(s) = arg.as_str() {
                jvm_args.push(s.to_string());
            }
        }
    }

    Ok(PreparedLoader {
        main_class: profile.main_class,
        classpath_entries,
        jvm_args,
    })
}

async fn get_latest_loader_version(http: &reqwest::Client, mc_version: &str) -> AppResult<String> {
    let versions = fetch_versions(http, mc_version).await?;
    let chosen = versions
        .iter()
        .find(|v| v.stable)
        .or_else(|| versions.first())
        .map(|v| v.id.clone())
        .unwrap_or_else(|| "0.26.0".to_string());
    Ok(chosen)
}
