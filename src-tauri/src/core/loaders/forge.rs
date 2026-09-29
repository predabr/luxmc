use super::{LoaderVersion, PreparedLoader};
use crate::error::{AppError, AppResult};
use serde::Deserialize;
use std::io::{Cursor, Read};
use std::path::Path;

const FORGE_MAVEN: &str = "https://maven.minecraftforge.net";
const MAVEN_REPOS: [&str; 3] = [
    "https://maven.minecraftforge.net",
    "https://libraries.minecraft.net",
    "https://repo1.maven.org/maven2",
];
const FORGE_METADATA: &str =
    "https://maven.minecraftforge.net/net/minecraftforge/forge/maven-metadata.xml";
const FORGE_PROMOTIONS: &str =
    "https://files.minecraftforge.net/net/minecraftforge/forge/promotions_slim.json";

#[derive(Debug, Clone, Deserialize)]
pub struct ForgeArtifact {
    pub path: Option<String>,
    pub url: Option<String>,
    pub sha1: Option<String>,
    pub size: Option<u64>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ForgeDownloads {
    pub artifact: Option<ForgeArtifact>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ForgeLibrary {
    pub name: String,
    pub downloads: Option<ForgeDownloads>,
    pub url: Option<String>,
    #[serde(default, rename = "clientreq")]
    pub clientreq: Option<bool>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ForgeArguments {
    #[serde(default)]
    pub jvm: Vec<serde_json::Value>,
    #[serde(default)]
    pub game: Vec<serde_json::Value>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ForgeVersionJson {
    pub id: String,
    #[serde(rename = "mainClass")]
    pub main_class: String,
    #[serde(default)]
    pub arguments: Option<ForgeArguments>,
    #[serde(default, rename = "minecraftArguments")]
    pub minecraft_arguments: Option<String>,
    #[serde(default)]
    pub libraries: Vec<ForgeLibrary>,
}

#[derive(Debug, Clone, Deserialize)]
struct ForgePromotions {
    #[serde(default)]
    promos: std::collections::HashMap<String, String>,
}

fn versions_from_metadata(metadata: &str, mc_version: &str) -> Vec<String> {
    let prefix = format!("{mc_version}-");
    let mut versions: Vec<String> = metadata.split("<version>").skip(1)
        .filter_map(|entry| entry.split("</version>").next())
        .map(str::trim)
        .filter_map(|version| version.strip_prefix(&prefix).map(str::to_string))
        .collect();
    versions.sort_by(|a, b| compare_loader_versions(a, b));
    versions
}

fn version_key(version: &str) -> Vec<u64> {
    let mut key = Vec::new();
    let mut digits = String::new();
    for ch in version.chars() {
        if ch.is_ascii_digit() {
            digits.push(ch);
        } else if !digits.is_empty() {
            key.push(digits.parse::<u64>().unwrap_or(0));
            digits.clear();
        }
    }
    if !digits.is_empty() {
        key.push(digits.parse::<u64>().unwrap_or(0));
    }
    key
}

pub(crate) fn compare_loader_versions(a: &str, b: &str) -> std::cmp::Ordering {
    let (key_a, key_b) = (version_key(a), version_key(b));
    key_b
        .cmp(&key_a)
        .then_with(|| b.cmp(a))
}

pub async fn fetch_versions(
    http: &reqwest::Client,
    mc_version: &str,
) -> AppResult<Vec<LoaderVersion>> {
    let mut versions = Vec::new();
    let mut seen = std::collections::HashSet::new();

    let rec_key = format!("{}-recommended", mc_version);
    let lat_key = format!("{}-latest", mc_version);

    if let Ok(resp) = http.get(FORGE_PROMOTIONS).send().await {
        if let Ok(data) = resp.json::<ForgePromotions>().await {
            if let Some(lat) = data.promos.get(&lat_key) {
                if seen.insert(lat.clone()) {
                    versions.push(LoaderVersion {
                        id: lat.clone(),
                        stable: data.promos.get(&rec_key) == Some(lat),
                    });
                }
            }
            if let Some(rec) = data.promos.get(&rec_key) {
                if seen.insert(rec.clone()) {
                    versions.push(LoaderVersion {
                        id: rec.clone(),
                        stable: true,
                    });
                }
            }
        }
    }

    if let Ok(resp) = http.get(FORGE_METADATA).send().await {
        if let Ok(text) = resp.text().await {
            for forge_ver in versions_from_metadata(&text, mc_version) {
                if seen.insert(forge_ver.clone()) {
                    versions.push(LoaderVersion {
                        id: forge_ver,
                        stable: true,
                    });
                }
            }
        }
    }

    Ok(versions)
}

fn extract_installer_maven_files(installer_bytes: &[u8], libraries_dir: &Path) -> AppResult<()> {
    let mut archive = zip::ZipArchive::new(Cursor::new(installer_bytes))
        .map_err(|e| AppError::Internal(format!("Failed to open Forge installer zip: {e}")))?;

    for i in 0..archive.len() {
        if let Ok(mut file) = archive.by_index(i) {
            let name = file.name().to_string();
            if let Some(rel) = name.strip_prefix("maven/") {
                if rel.is_empty() || file.is_dir() {
                    continue;
                }
                let relative = Path::new(rel);
                if relative.components().any(|part| !matches!(part, std::path::Component::Normal(_))) {
                    return Err(AppError::InvalidInput("Unsafe path in Forge installer".into()));
                }
                let dest = libraries_dir.join(relative);
                if let Some(parent) = dest.parent() {
                    std::fs::create_dir_all(parent)?;
                }
                let mut outfile = std::fs::File::create(&dest)?;
                std::io::copy(&mut file, &mut outfile)?;
            }
        }
    }
    Ok(())
}

fn extract_version_json_from_bytes(bytes: &[u8]) -> AppResult<String> {
    let mut archive = zip::ZipArchive::new(Cursor::new(bytes))
        .map_err(|e| AppError::Internal(format!("Failed to open Forge installer zip: {e}")))?;

    if let Ok(mut version_file) = archive.by_name("version.json") {
        let mut s = String::new();
        version_file.read_to_string(&mut s)?;
        return Ok(s);
    }

    if let Ok(mut profile_file) = archive.by_name("install_profile.json") {
        let mut raw = String::new();
        profile_file.read_to_string(&mut raw)?;
        if let Ok(value) = serde_json::from_str::<serde_json::Value>(&raw) {
            if let Some(info) = value.get("versionInfo") {
                return Ok(info.to_string());
            }
        }
    }

    Err(AppError::Internal(
        "Forge installer sem version.json nem install_profile.json".into(),
    ))
}

fn primary_maven_url(lib: &ForgeLibrary, rel_path: &str) -> String {
    let normalized = rel_path.trim_start_matches('/');
    if let Some(url) = lib
        .url
        .as_deref()
        .map(str::trim)
        .filter(|u| !u.is_empty())
    {
        return format!("{}/{}", url.trim_end_matches('/'), normalized);
    }
    format!("{FORGE_MAVEN}/{normalized}")
}

async fn find_installed_json(
    versions_dir: &Path,
    mc_version: &str,
    chosen_version: &str,
    candidate_ids: &[String],
) -> Option<std::path::PathBuf> {
    for cand in candidate_ids {
        let p = versions_dir.join(cand).join(format!("{cand}.json"));
        if p.exists() {
            return Some(p);
        }
    }

    if !versions_dir.exists() {
        return None;
    }
    let mut entries = tokio::fs::read_dir(versions_dir).await.ok()?;
    while let Ok(Some(entry)) = entries.next_entry().await {
        let name = entry.file_name().to_string_lossy().to_string();
        if name.contains(mc_version)
            && name.contains(chosen_version)
            && name.to_lowercase().contains("forge")
        {
            let cand_file = entry.path().join(format!("{name}.json"));
            if cand_file.exists() {
                return Some(cand_file);
            }
        }
    }
    None
}

pub async fn prepare_forge(
    http: &reqwest::Client,
    libraries_dir: &Path,
    mc_version: &str,
    loader_version: Option<&str>,
) -> AppResult<PreparedLoader> {
    let chosen_version = if let Some(v) = loader_version {
        if !v.trim().is_empty() {
            let clean = v.trim();
            clean.strip_prefix("forge-").unwrap_or(clean).to_string()
        } else {
            get_latest_loader_version(http, mc_version).await?
        }
    } else {
        get_latest_loader_version(http, mc_version).await?
    };

    let clean_chosen = chosen_version.strip_prefix("forge-").unwrap_or(&chosen_version);
    let full_version = if clean_chosen.starts_with(mc_version) {
        clean_chosen.to_string()
    } else {
        format!("{}-{}", mc_version, clean_chosen)
    };

    let installer_url = format!(
        "{}/net/minecraftforge/forge/{}/forge-{}-installer.jar",
        FORGE_MAVEN, full_version, full_version
    );

    let installer_rel = format!(
        "net/minecraftforge/forge/{}/forge-{}-installer.jar",
        full_version, full_version
    );
    let installer_dest = libraries_dir.join(&installer_rel);

    crate::core::downloader::ensure_artifact(http, &installer_dest, &installer_url, 0, "").await?;

    let data_dir = libraries_dir.parent().unwrap_or(libraries_dir);
    let client_rel = format!(
        "net/minecraftforge/forge/{}/forge-{}-client.jar",
        full_version, full_version
    );
    let client_dest = libraries_dir.join(&client_rel);
    let universal_rel = format!(
        "net/minecraftforge/forge/{}/forge-{}-universal.jar",
        full_version, full_version
    );
    let universal_dest = libraries_dir.join(&universal_rel);
    let installer_marker = installer_dest.with_extension("installed");

    let versions_dir = data_dir.join("versions");
    let candidate_ids = [
        format!("{}-forge{}", mc_version, full_version),
        format!("{}-forge-{}", mc_version, chosen_version),
        format!("{}-forge{}", mc_version, chosen_version),
        format!("forge-{}-{}", mc_version, chosen_version),
        full_version.clone(),
        format!("{}-{}", mc_version, chosen_version),
        format!("forge-{}", chosen_version),
    ];

    let mut installed_json_path =
        find_installed_json(&versions_dir, mc_version, &chosen_version, &candidate_ids).await;

    let already_installed =
        installed_json_path.is_some() && (client_dest.exists() || installer_marker.exists());

    if !already_installed {
        let profiles_file = data_dir.join("launcher_profiles.json");
        if !profiles_file.exists() {
            let _ = tokio::fs::write(&profiles_file, b"{\"profiles\":{}}").await;
        }

        super::installer::run(http, &installer_dest, data_dir, mc_version).await?;
        installed_json_path =
            find_installed_json(&versions_dir, mc_version, &chosen_version, &candidate_ids).await;
    }

    let installer_bytes = tokio::fs::read(&installer_dest).await?;
    extract_installer_maven_files(&installer_bytes, libraries_dir)?;

    let version_json_str = if let Some(ref p) = installed_json_path {
        tokio::fs::read_to_string(p).await.unwrap_or_else(|_| {
            extract_version_json_from_bytes(&installer_bytes).unwrap_or_default()
        })
    } else {
        extract_version_json_from_bytes(&installer_bytes)?
    };

    let version_data: ForgeVersionJson = serde_json::from_str(&version_json_str)
        .map_err(|e| AppError::Internal(format!("Failed to parse Forge version.json: {e}")))?;

    let mut classpath_entries = Vec::new();

    for lib in &version_data.libraries {
        if lib.clientreq == Some(false) {
            continue;
        }

        let legacy_self_forge = lib.downloads.is_none()
            && lib.name == format!("net.minecraftforge:forge:{}", full_version);

        let (mut rel_path_str, primary_url) = if let Some(ref d) = lib.downloads {
            if let Some(ref art) = d.artifact {
                let path = art.path.clone().unwrap_or_else(|| {
                    crate::core::launcher::lib_path_from_name(&std::path::PathBuf::new(), &lib.name)
                        .to_string_lossy()
                        .replace('\\', "/")
                });
                let url = art
                    .url
                    .clone()
                    .filter(|u| !u.trim().is_empty())
                    .unwrap_or_else(|| primary_maven_url(lib, &path));
                (path, url)
            } else {
                let p = crate::core::launcher::lib_path_from_name(&std::path::PathBuf::new(), &lib.name)
                    .to_string_lossy()
                    .replace('\\', "/");
                let u = primary_maven_url(lib, &p);
                (p, u)
            }
        } else {
            let p = crate::core::launcher::lib_path_from_name(&std::path::PathBuf::new(), &lib.name)
                .to_string_lossy()
                .replace('\\', "/");
            let u = primary_maven_url(lib, &p);
            (p, u)
        };

        if legacy_self_forge {
            rel_path_str = universal_rel.clone();
        }

        let relative = Path::new(&rel_path_str);
        if relative.components().any(|part| !matches!(part, std::path::Component::Normal(_))) {
            return Err(AppError::InvalidInput("Unsafe loader library path".into()));
        }
        let dest = libraries_dir.join(relative);

        let mut urls = vec![primary_url.clone()];
        let normalized = rel_path_str.trim_start_matches('/').to_string();
        for repo in MAVEN_REPOS {
            let candidate = format!("{repo}/{normalized}");
            if !urls.contains(&candidate) {
                urls.push(candidate);
            }
        }

        let artifact = lib.downloads.as_ref().and_then(|downloads| downloads.artifact.as_ref());
        let size = artifact.and_then(|entry| entry.size).unwrap_or(0);
        let sha1 = artifact.and_then(|entry| entry.sha1.as_deref()).unwrap_or_default();
        crate::core::downloader::ensure_artifact_any(http, &dest, &urls, size, sha1).await?;
        if !classpath_entries.contains(&dest) { classpath_entries.push(dest); }
    }

    if !universal_dest.exists() {
        let universal_url = format!("{}/{}", FORGE_MAVEN, universal_rel);
        if let Ok(resp) = http.get(&universal_url).send().await {
            if resp.status().is_success() {
                if let Ok(b) = resp.bytes().await {
                    if let Some(p) = universal_dest.parent() {
                        let _ = tokio::fs::create_dir_all(p).await;
                    }
                    let _ = tokio::fs::write(&universal_dest, &b).await;
                }
            }
        }
    }
    if client_dest.exists() && !classpath_entries.contains(&client_dest) {
        classpath_entries.push(client_dest);
    } else if universal_dest.exists() && !classpath_entries.contains(&universal_dest) {
        classpath_entries.push(universal_dest);
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
                        .replace("${version_name}", &version_data.id);
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
                                    .replace("${version_name}", &version_data.id);
                                if crate::core::launcher::jvm_arg_allowed_on_current_os(&replaced) {
                                    jvm_args.push(replaced);
                                }
                            }
                        }
                    } else if let Some(serde_json::Value::String(s)) = obj.get("value") {
                        let replaced = s
                            .replace("${library_directory}", &lib_dir_str)
                            .replace("${classpath_separator}", cp_sep)
                            .replace("${version_name}", &version_data.id);
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
        jvm_args.push("-Dforge.enabled=true".to_string());
    }

    jvm_args.retain(|a| !a.starts_with("-Dfml.earlyprogresswindow="));
    jvm_args.push("-Dfml.earlyprogresswindow=false".to_string());
    jvm_args.retain(|a| !a.starts_with("-Dneoforge.earlydisplay="));
    jvm_args.push("-Dneoforge.earlydisplay=false".to_string());
    if !jvm_args.iter().any(|a| a.starts_with("-Dorg.lwjgl.glfw.checkThread0=")) {
        jvm_args.push("-Dorg.lwjgl.glfw.checkThread0=false".to_string());
    }

    let forge_only_version = full_version
        .strip_prefix(&format!("{}-", mc_version))
        .unwrap_or(&full_version);
    if !jvm_args.iter().any(|a| a.contains("fml.forgeVersion") || a.contains("forge.version")) {
        jvm_args.push(format!("-Dfml.forgeVersion={}", forge_only_version));
    }
    if !jvm_args.iter().any(|a| a.contains("fml.mcVersion") || a.contains("minecraft.version")) {
        jvm_args.push(format!("-Dfml.mcVersion={}", mc_version));
    }
    if !jvm_args.iter().any(|a| a.contains("fml.mcpVersion")) {
        jvm_args.push(format!("-Dfml.mcpVersion={}", mc_version));
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

    let java_major = match mc_version {
        v if v.starts_with("1.20.5") || v.starts_with("1.20.6") || v.starts_with("1.21") || v.starts_with("2") => 21,
        v if v.starts_with("1.17") || v.starts_with("1.18") || v.starts_with("1.19") || v.starts_with("1.20") => 17,
        _ => 8,
    };
    if java_major >= 17 {
        if !jvm_args.iter().any(|a| a.contains("java.base/java.lang")) {
            jvm_args.push("--add-opens=java.base/java.lang=ALL-UNNAMED".to_string());
        }
        if !jvm_args.iter().any(|a| a.contains("java.base/java.util")) {
            jvm_args.push("--add-opens=java.base/java.util=ALL-UNNAMED".to_string());
        }
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

    if let Some(ref mc_args) = version_data.minecraft_arguments {
        let parts: Vec<&str> = mc_args.split_whitespace().collect();
        let mut idx = 0;
        while idx < parts.len() {
            let p = parts[idx];
            if p == "--tweakClass" && idx + 1 < parts.len() {
                game_args.push(p.to_string());
                game_args.push(parts[idx + 1].to_string());
                idx += 2;
                continue;
            }
            idx += 1;
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

    Err(AppError::NotFound(format!("Forge indisponível para Minecraft {mc_version}")))
}

#[cfg(test)]
mod tests {
    #[test]
    fn reads_compact_maven_metadata_without_other_versions() {
        let xml = "<versions><version>1.20.1-47.4.20</version><version>1.21.4-54.1.0</version></versions>";
        assert_eq!(super::versions_from_metadata(xml, "1.21.4"), ["54.1.0"]);
    }
}
