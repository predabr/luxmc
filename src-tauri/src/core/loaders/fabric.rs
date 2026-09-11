use super::{LoaderVersion, PreparedLoader};
use crate::error::AppResult;

const FABRIC_META: &str = "https://meta.fabricmc.net/v2";

#[derive(Debug, Clone, serde::Deserialize)]
pub struct FabricLibrary {
    pub name: String,
    pub url: Option<String>,
}

#[derive(Debug, Clone, serde::Deserialize)]
pub struct FabricArguments {
    #[serde(default)]
    pub jvm: Vec<serde_json::Value>,
    #[serde(default)]
    pub game: Vec<serde_json::Value>,
}

#[derive(Debug, Clone, serde::Deserialize)]
pub struct FabricProfile {
    pub id: String,
    #[serde(rename = "mainClass")]
    pub main_class: String,
    #[serde(default)]
    pub arguments: Option<FabricArguments>,
    pub libraries: Vec<FabricLibrary>,
}

pub async fn fetch_versions(
    http: &reqwest::Client,
    mc_version: &str,
) -> AppResult<Vec<LoaderVersion>> {
    let url = format!("{}/versions/loader/{}", FABRIC_META, mc_version);
    let resp: Vec<serde_json::Value> = http
        .get(&url)
        .send()
        .await?
        .error_for_status()?
        .json()
        .await?;

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

pub async fn prepare_fabric(
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
        FABRIC_META, mc_version, chosen_version
    );

    let profile: FabricProfile = http
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

            let base_url = lib.url.as_deref().unwrap_or("https://maven.fabricmc.net/");
            let rel_str = rel_path.to_string_lossy().replace('\\', "/");
            let download_url = format!("{}/{}", base_url.trim_end_matches('/'), rel_str.trim_start_matches('/'));

            tracing::info!(lib = %lib.name, url = %download_url, "downloading Fabric library");

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
            match &arg {
                serde_json::Value::String(s) => {
                    jvm_args.push(s.to_string());
                }
                serde_json::Value::Object(obj) => {
                    if let Some(serde_json::Value::Array(rules)) = obj.get("rules") {
                        let mut allow = false;
                        let mut deny = false;
                        for rule in rules {
                            if let Some(action) = rule.get("action").and_then(|a| a.as_str()) {
                                if action == "allow" {
                                    allow = true;
                                } else if action == "deny" {
                                    deny = true;
                                }
                            }
                        }
                        if deny && !allow {
                            continue;
                        }
                    }
                    if let Some(serde_json::Value::Array(values)) = obj.get("value") {
                        for v in values {
                            if let Some(s) = v.as_str() {
                                jvm_args.push(s.to_string());
                            }
                        }
                    } else if let Some(serde_json::Value::String(s)) = obj.get("value") {
                        jvm_args.push(s.to_string());
                    }
                }
                _ => {}
            }
        }
    }
    if jvm_args.is_empty() {
        jvm_args.push("-DFabricMcEmu= net.minecraft.client.main.Main ".to_string());
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
        .unwrap_or_else(|| "0.16.10".to_string());
    Ok(chosen)
}

pub async fn ensure_fabric_api(
    http: &reqwest::Client,
    mods_dir: &std::path::Path,
    mc_version: &str,
) -> AppResult<()> {
    tokio::fs::create_dir_all(mods_dir).await?;

    let mut entries = match tokio::fs::read_dir(mods_dir).await {
        Ok(e) => e,
        Err(_) => return Ok(()),
    };

    let mut has_fabric_api = false;
    let mut has_any_mod = false;
    while let Ok(Some(entry)) = entries.next_entry().await {
        let name = entry.file_name().to_string_lossy().to_lowercase();
        if name.ends_with(".jar") {
            has_any_mod = true;
            if name.contains("fabric-api") || name.contains("fabric_api") {
                has_fabric_api = true;
                break;
            }
        }
    }

    if has_fabric_api || !has_any_mod {
        return Ok(());
    }

    tracing::info!(
        mc_version = %mc_version,
        "Fabric API missing in mods directory; downloading compatible version from Modrinth"
    );

    let url = format!(
        "https://api.modrinth.com/v2/project/fabric-api/version?game_versions=%5B%22{}%22%5D&loaders=%5B%22fabric%22%5D",
        urlencoding::encode(mc_version)
    );

    if let Ok(resp) = http
        .get(&url)
        .header("User-Agent", "Luxmc/1.2.0-alpha")
        .send()
        .await
    {
        if resp.status().is_success() {
            if let Ok(versions) = resp.json::<Vec<serde_json::Value>>().await {
                if let Some(first_ver) = versions.first() {
                    if let Some(files) = first_ver.get("files").and_then(|f| f.as_array()) {
                        if let Some(file) = files.first() {
                            if let (Some(dl_url), Some(filename)) = (
                                file.get("url").and_then(|u| u.as_str()),
                                file.get("filename").and_then(|f| f.as_str()),
                            ) {
                                let dest = mods_dir.join(filename);
                                if let Ok(dl_resp) = http.get(dl_url).send().await {
                                    if dl_resp.status().is_success() {
                                        if let Ok(bytes) = dl_resp.bytes().await {
                                            let _ = tokio::fs::write(&dest, &bytes).await;
                                            tracing::info!(file = %filename, "Fabric API installed successfully");
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    Ok(())
}

