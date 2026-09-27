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
        if rel_path.components().any(|part| !matches!(part, std::path::Component::Normal(_))) {
            return Err(crate::error::AppError::InvalidInput("Caminho Maven inválido".into()));
        }
        let dest = libraries_dir.join(&rel_path);
        let base_url = lib.url.as_deref().unwrap_or("https://maven.quiltmc.org/repository/release/");
        let relative = rel_path.to_string_lossy().replace('\\', "/");
        let url = format!("{}/{}", base_url.trim_end_matches('/'), relative);
        if super::ensure_maven_jar(http, &dest, &url).await.is_err() {
            let central = format!("https://repo1.maven.org/maven2/{relative}");
            super::ensure_maven_jar(http, &dest, &central).await?;
        }
        classpath_entries.push(dest);
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
        game_args: Vec::new(),
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
