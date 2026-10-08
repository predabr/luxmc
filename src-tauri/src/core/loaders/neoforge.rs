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

fn versions_from_metadata(metadata: &str, prefix: &str) -> Vec<LoaderVersion> {
    let mut versions = metadata.split("<version>").skip(1)
        .filter_map(|entry| entry.split("</version>").next())
        .map(str::trim)
        .filter(|version| version.starts_with(prefix))
        .map(|version| LoaderVersion {
            id: version.to_string(),
            stable: !version.contains("beta") && !version.contains("alpha") && !version.contains("rc"),
        })
        .collect::<Vec<_>>();
    versions.reverse();
    versions
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

    let resp_text = match http
        .get(NEOFORGE_METADATA)
        .header("User-Agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/122.0.0.0 Safari/537.36 Luxmc/1.6.5")
        .header("Accept", "text/xml, application/xml, */*")
        .send()
        .await
    {
        Ok(r) if r.status().is_success() => r.text().await.unwrap_or_default(),
        _ => String::new(),
    };

    Ok(versions_from_metadata(&resp_text, &prefix))
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
            let mut clean = v.trim();
            if clean.to_ascii_lowercase().starts_with("neoforge-") {
                clean = &clean[9..];
            } else if clean.to_ascii_lowercase().starts_with("neo-forge-") {
                clean = &clean[10..];
            }
            if clean.starts_with(mc_version) && clean.len() > mc_version.len() + 1 && clean.as_bytes()[mc_version.len()] == b'-' {
                clean = &clean[mc_version.len() + 1..];
            }
            clean.to_string()
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

    crate::core::downloader::ensure_artifact(http, &installer_dest, &installer_url, 0, "").await?;

    let data_dir = libraries_dir.parent().unwrap_or(libraries_dir);
    let installed_json_path = data_dir
        .join("versions")
        .join(format!("neoforge-{}", chosen_version))
        .join(format!("neoforge-{}.json", chosen_version));

    if !installed_json_path.exists() || !installer_dest.with_extension("installed").exists() {
        let profiles_file = data_dir.join("launcher_profiles.json");
        if !profiles_file.exists() {
            let _ = tokio::fs::write(&profiles_file, b"{\"profiles\":{}}").await;
        }

        super::installer::run(http, &installer_dest, data_dir, mc_version).await?;
    }

    let version_json_str = if installed_json_path.exists() {
        tokio::fs::read_to_string(&installed_json_path)
            .await
            .unwrap_or_else(|_| {
                tokio::task::block_in_place(|| {
                    let b = std::fs::read(&installer_dest).unwrap_or_default();
                    extract_version_json_from_bytes(&b).unwrap_or_default()
                })
            })
    } else {
        let installer_bytes = tokio::fs::read(&installer_dest).await?;
        extract_version_json_from_bytes(&installer_bytes)?
    };

    let version_data: NeoForgeVersionJson = serde_json::from_str(&version_json_str)
        .map_err(|e| AppError::Internal(format!("Failed to parse NeoForge version.json: {e}")))?;

    use futures_util::{StreamExt, TryStreamExt};
    let mut classpath_entries = Vec::new();
    let mut downloads = Vec::new();

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

        let relative = Path::new(&rel_path_str);
        if relative.components().any(|part| !matches!(part, std::path::Component::Normal(_))) {
            return Err(AppError::InvalidInput("Unsafe loader library path".into()));
        }
        let dest = libraries_dir.join(relative);
        let artifact = lib.downloads.as_ref().and_then(|downloads| downloads.artifact.as_ref());
        let size = artifact.and_then(|entry| entry.size).unwrap_or(0);
        let sha1 = artifact.and_then(|entry| entry.sha1.as_deref()).unwrap_or_default();
        if !classpath_entries.contains(&dest) { downloads.push((dest.clone(), download_url, size, sha1.to_string())); classpath_entries.push(dest); }
    }

    futures_util::stream::iter(downloads).map(|(dest,urls,size,sha1)| {
        let http = http.clone();
        async move { crate::core::downloader::ensure_artifact(&http, &dest, &urls, size, &sha1).await }
    }).buffered(6).try_collect::<Vec<_>>().await?;

    let client_rel = format!(
        "net/neoforged/neoforge/{}/neoforge-{}-client.jar",
        chosen_version, chosen_version
    );
    let client_dest = libraries_dir.join(&client_rel);
    let universal_rel = format!(
        "net/neoforged/neoforge/{}/neoforge-{}-universal.jar",
        chosen_version, chosen_version
    );
    let universal_dest = libraries_dir.join(&universal_rel);

    let is_modern_neoforge = {
        let parts: Vec<&str> = chosen_version.split('.').collect();
        if let Some(first) = parts.first().and_then(|s| s.parse::<u32>().ok()) {
            first >= 20
        } else {
            true
        }
    };

    if !is_modern_neoforge {
        if client_dest.exists() {
            if !classpath_entries.contains(&client_dest) {
                classpath_entries.push(client_dest.clone());
            }
            classpath_entries.retain(|p| {
                let s = p.to_string_lossy().replace('\\', "/");
                !s.contains("net/neoforged/neoforge/") || s.ends_with("-client.jar")
            });
        } else if universal_dest.exists() {
            if !classpath_entries.contains(&universal_dest) {
                classpath_entries.push(universal_dest.clone());
            }
            classpath_entries.retain(|p| {
                let s = p.to_string_lossy().replace('\\', "/");
                !s.contains("net/neoforged/neoforge/") || s.ends_with("-universal.jar")
            });
        }
    } else {
        classpath_entries.retain(|p| {
            let s = p.to_string_lossy().replace('\\', "/");
            !s.contains("net/neoforged/neoforge/")
        });
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

    jvm_args.retain(|a| !a.starts_with("-Dneoforge.earlydisplay="));
    jvm_args.push("-Dneoforge.earlydisplay=false".to_string());
    jvm_args.retain(|a| !a.starts_with("-Dfml.earlyprogresswindow="));
    jvm_args.push("-Dfml.earlyprogresswindow=false".to_string());
    if !jvm_args.iter().any(|a| a.starts_with("-Dorg.lwjgl.glfw.checkThread0=")) {
        jvm_args.push("-Dorg.lwjgl.glfw.checkThread0=false".to_string());
    }

    if !jvm_args.iter().any(|a| a.contains("fml.neoForgeVersion") || a.contains("neoForgeVersion")) {
        jvm_args.push(format!("-Dfml.neoForgeVersion={}", chosen_version));
    }
    if !jvm_args.iter().any(|a| a.contains("fml.mcVersion") || a.contains("mcVersion")) {
        jvm_args.push(format!("-Dfml.mcVersion={}", mc_version));
    }
    if !jvm_args.iter().any(|a| a.contains("fml.neoFormVersion")) {
        jvm_args.push(format!("-Dfml.neoFormVersion={}", mc_version));
    }

    let mut found_ignore_list = false;
    for arg in &mut jvm_args {
        if arg.starts_with("-DignoreList=") {
            found_ignore_list = true;
            if !arg.contains(&format!("{}.jar", mc_version)) {
                arg.push_str(&format!(",{}.jar,{}", mc_version, mc_version));
            }
        }
    }
    if !found_ignore_list {
        jvm_args.push(format!("-DignoreList=client-extra,{}.jar,{}", mc_version, mc_version));
    }

    if !jvm_args.iter().any(|a| a.contains("java.base/java.lang")) {
        jvm_args.push("--add-opens=java.base/java.lang=ALL-UNNAMED".to_string());
    }
    if !jvm_args.iter().any(|a| a.contains("java.base/java.util")) {
        jvm_args.push("--add-opens=java.base/java.util=ALL-UNNAMED".to_string());
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
    if let Some(chosen) = versions.iter().find(|version| version.stable).or_else(|| versions.first()) {
        return Ok(chosen.id.clone());
    }

    Err(AppError::NotFound(format!("NeoForge indisponível para Minecraft {mc_version}")))
}

#[cfg(test)]
mod tests {
    #[test]
    fn reads_compact_maven_metadata() {
        let xml = "<metadata><versions><version>21.4.157</version><version>21.4.158</version><version>21.4.159-beta</version></versions></metadata>";
        let versions = super::versions_from_metadata(xml, "21.4.");
        assert_eq!(versions.iter().map(|version| version.id.as_str()).collect::<Vec<_>>(), ["21.4.159-beta", "21.4.158", "21.4.157"]);
        assert!(!versions[0].stable);
        assert!(versions[1].stable);
    }
}
