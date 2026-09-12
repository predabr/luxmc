use super::{LoaderVersion, PreparedLoader};
use crate::error::{AppError, AppResult};
use serde::Deserialize;
use std::io::{Cursor, Read};
use std::path::Path;

const NEOFORGE_MAVEN: &str = "https://maven.neoforged.net/releases";
const NEOFORGE_METADATA: &str =
    "https://maven.neoforged.net/releases/net/neoforged/neoforge/maven-metadata.xml";

#[derive(Debug, Clone, Deserialize)]
pub struct NeoForgeArtifact {
    pub path: Option<String>,
    pub url: Option<String>,
    pub sha1: Option<String>,
    pub size: Option<u64>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct NeoForgeDownloads {
    pub artifact: Option<NeoForgeArtifact>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct NeoForgeLibrary {
    pub name: String,
    pub downloads: Option<NeoForgeDownloads>,
    pub url: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct NeoForgeArguments {
    #[serde(default)]
    pub jvm: Vec<serde_json::Value>,
    #[serde(default)]
    pub game: Vec<serde_json::Value>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct NeoForgeVersionJson {
    pub id: String,
    #[serde(rename = "mainClass")]
    pub main_class: String,
    #[serde(default)]
    pub arguments: Option<NeoForgeArguments>,
    #[serde(default)]
    pub libraries: Vec<NeoForgeLibrary>,
}

pub async fn fetch_versions(
    http: &reqwest::Client,
    mc_version: &str,
) -> AppResult<Vec<LoaderVersion>> {
    let parts: Vec<&str> = mc_version.split('.').collect();
    let prefix = if parts.len() >= 2 && parts[0] == "1" {
        let minor = parts[1];
        let patch = if parts.len() > 2 { parts[2] } else { "0" };
        format!("{}.{}.", minor, patch)
    } else {
        format!("{}.", mc_version)
    };

    let resp_text = match http.get(NEOFORGE_METADATA).send().await {
        Ok(r) => match r.text().await {
            Ok(t) => t,
            Err(_) => String::new(),
        },
        Err(_) => String::new(),
    };

    let mut versions = Vec::new();
    if !resp_text.is_empty() {
        for line in resp_text.lines() {
            let trimmed = line.trim();
            if trimmed.starts_with("<version>") && trimmed.ends_with("</version>") {
                let v = trimmed
                    .trim_start_matches("<version>")
                    .trim_end_matches("</version>")
                    .trim();
                if v.starts_with(&prefix) {
                    versions.push(LoaderVersion {
                        id: v.to_string(),
                        stable: true,
                    });
                }
            }
        }
    }

    versions.reverse();
    Ok(versions)
}

fn extract_version_json_from_bytes(bytes: &[u8]) -> AppResult<String> {
    let mut archive = zip::ZipArchive::new(Cursor::new(bytes))
        .map_err(|e| AppError::Internal(format!("Failed to open NeoForge installer zip: {e}")))?;

    let mut version_file = archive
        .by_name("version.json")
        .map_err(|e| AppError::Internal(format!("NeoForge installer missing version.json: {e}")))?;
    let mut s = String::new();
    version_file
        .read_to_string(&mut s)
        .map_err(|e| AppError::Internal(format!("Failed to read NeoForge version.json: {e}")))?;
    Ok(s)
}

pub async fn prepare_neoforge(
    http: &reqwest::Client,
    libraries_dir: &Path,
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

    let installer_url = format!(
        "{}/net/neoforged/neoforge/{}/neoforge-{}-installer.jar",
        NEOFORGE_MAVEN, chosen_version, chosen_version
    );

    let installer_rel = format!(
        "net/neoforged/neoforge/{}/neoforge-{}-installer.jar",
        chosen_version, chosen_version
    );
    let installer_dest = libraries_dir.join(&installer_rel);

    if !installer_dest.exists() {
        if let Some(parent) = installer_dest.parent() {
            tokio::fs::create_dir_all(parent).await?;
        }
        tracing::info!(version = %chosen_version, url = %installer_url, "Downloading NeoForge installer");
        let resp = http
            .get(&installer_url)
            .send()
            .await
            .map_err(|e| AppError::Internal(format!("Failed to download NeoForge installer: {e}")))?;
        let bytes = resp
            .error_for_status()
            .map_err(|e| AppError::Internal(format!("NeoForge installer not found: {e}")))?
            .bytes()
            .await
            .map_err(|e| AppError::Internal(format!("Failed to read NeoForge installer: {e}")))?;
        tokio::fs::write(&installer_dest, &bytes).await?;
    }

    let installer_bytes = tokio::fs::read(&installer_dest).await?;
    let version_json_str = extract_version_json_from_bytes(&installer_bytes)?;

    let version_data: NeoForgeVersionJson = serde_json::from_str(&version_json_str)
        .map_err(|e| AppError::Internal(format!("Failed to parse NeoForge version.json: {e}")))?;

    let mut classpath_entries = Vec::new();

    for lib in &version_data.libraries {
        let (rel_path_str, download_url) = if let Some(ref d) = lib.downloads {
            if let Some(ref art) = d.artifact {
                let path = art.path.clone().unwrap_or_else(|| {
                    crate::core::launcher::lib_path_from_name(&std::path::PathBuf::new(), &lib.name)
                        .to_string_lossy()
                        .replace('\\', "/")
                });
                let url = art.url.clone().unwrap_or_else(|| {
                    format!("{}/{}", NEOFORGE_MAVEN, path.trim_start_matches('/'))
                });
                (path, url)
            } else {
                let p = crate::core::launcher::lib_path_from_name(&std::path::PathBuf::new(), &lib.name)
                    .to_string_lossy()
                    .replace('\\', "/");
                let u = format!("{}/{}", NEOFORGE_MAVEN, p.trim_start_matches('/'));
                (p, u)
            }
        } else {
            let p = crate::core::launcher::lib_path_from_name(&std::path::PathBuf::new(), &lib.name)
                .to_string_lossy()
                .replace('\\', "/");
            let u = format!("{}/{}", NEOFORGE_MAVEN, p.trim_start_matches('/'));
            (p, u)
        };

        let dest = libraries_dir.join(&rel_path_str);
        if !dest.exists() {
            if let Some(parent) = dest.parent() {
                let _ = tokio::fs::create_dir_all(parent).await;
            }

            let mut downloaded = false;
            if let Ok(resp) = http.get(&download_url).send().await {
                if resp.status().is_success() {
                    if let Ok(b) = resp.bytes().await {
                        if !b.is_empty() {
                            let _ = tokio::fs::write(&dest, &b).await;
                            downloaded = true;
                        }
                    }
                }
            }

            if !downloaded {
                let fallback = format!("https://repo1.maven.org/maven2/{}", rel_path_str.trim_start_matches('/'));
                if let Ok(resp) = http.get(&fallback).send().await {
                    if resp.status().is_success() {
                        if let Ok(b) = resp.bytes().await {
                            let _ = tokio::fs::write(&dest, &b).await;
                        }
                    }
                }
            }
        }

        if dest.exists() {
            classpath_entries.push(dest);
        }
    }

    if installer_dest.exists() && !classpath_entries.contains(&installer_dest) {
        classpath_entries.push(installer_dest);
    }

    let cp_sep = if cfg!(windows) { ";" } else { ":" };
    let lib_dir_str = libraries_dir.to_string_lossy().replace('\\', "/");

    let mut jvm_args = Vec::new();
    if let Some(ref args) = version_data.arguments {
        for arg in &args.jvm {
            match &arg {
                serde_json::Value::String(s) => {
                    let replaced = s
                        .replace("${library_directory}", &lib_dir_str)
                        .replace("${classpath_separator}", cp_sep)
                        .replace("${version_name}", &format!("neoforge-{}", chosen_version));
                    if crate::core::launcher::jvm_arg_allowed_on_current_os(&replaced) {
                        jvm_args.push(replaced);
                    }
                }
                serde_json::Value::Object(obj) => {
                    if let Some(serde_json::Value::Array(rules)) = obj.get("rules") {
                        if !crate::core::launcher::evaluate_rules(rules) {
                            continue;
                        }
                    }
                    if let Some(serde_json::Value::Array(values)) = obj.get("value") {
                        for v in values {
                            if let Some(s) = v.as_str() {
                                let replaced = s
                                    .replace("${library_directory}", &lib_dir_str)
                                    .replace("${classpath_separator}", cp_sep)
                                    .replace("${version_name}", &format!("neoforge-{}", chosen_version));
                                if crate::core::launcher::jvm_arg_allowed_on_current_os(&replaced) {
                                    jvm_args.push(replaced);
                                }
                            }
                        }
                    } else if let Some(serde_json::Value::String(s)) = obj.get("value") {
                        let replaced = s
                            .replace("${library_directory}", &lib_dir_str)
                            .replace("${classpath_separator}", cp_sep)
                            .replace("${version_name}", &format!("neoforge-{}", chosen_version));
                        if crate::core::launcher::jvm_arg_allowed_on_current_os(&replaced) {
                            jvm_args.push(replaced);
                        }
                    }
                }
                _ => {}
            }
        }
    }

    if jvm_args.is_empty() {
        jvm_args.push("-Dneoforge.enabled=true".to_string());
    }

    let mut game_args = Vec::new();
    if let Some(ref args) = version_data.arguments {
        for arg in &args.game {
            match arg {
                serde_json::Value::String(s) => {
                    game_args.push(s.clone());
                }
                serde_json::Value::Object(obj) => {
                    if let Some(serde_json::Value::Array(rules)) = obj.get("rules") {
                        if !crate::core::launcher::evaluate_rules(rules) {
                            continue;
                        }
                    }
                    if let Some(serde_json::Value::Array(values)) = obj.get("value") {
                        for v in values {
                            if let Some(s) = v.as_str() {
                                game_args.push(s.to_string());
                            }
                        }
                    } else if let Some(serde_json::Value::String(s)) = obj.get("value") {
                        game_args.push(s.to_string());
                    }
                }
                _ => {}
            }
        }
    }

    Ok(PreparedLoader {
        main_class: version_data.main_class,
        classpath_entries,
        jvm_args,
        game_args,
    })
}

async fn get_latest_loader_version(http: &reqwest::Client, mc_version: &str) -> AppResult<String> {
    let versions = fetch_versions(http, mc_version).await?;
    if let Some(first) = versions.first() {
        return Ok(first.id.clone());
    }

    let parts: Vec<&str> = mc_version.split('.').collect();
    if parts.len() >= 2 && parts[0] == "1" {
        let minor = parts[1];
        let patch = if parts.len() > 2 { parts[2] } else { "0" };
        Ok(format!("{}.{}.1", minor, patch))
    } else {
        Ok(format!("{}.1", mc_version))
    }
}
