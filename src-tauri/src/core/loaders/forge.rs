use super::{LoaderVersion, PreparedLoader};
use crate::error::{AppError, AppResult};
use serde::Deserialize;
use std::io::{Cursor, Read, Write};
use std::path::Path;

const FORGE_MAVEN: &str = "https://maven.minecraftforge.net";
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
            if let Some(rec) = data.promos.get(&rec_key) {
                if seen.insert(rec.clone()) {
                    versions.push(LoaderVersion {
                        id: rec.clone(),
                        stable: true,
                    });
                }
            }
            if let Some(lat) = data.promos.get(&lat_key) {
                if seen.insert(lat.clone()) {
                    versions.push(LoaderVersion {
                        id: lat.clone(),
                        stable: false,
                    });
                }
            }
        }
    }

    let prefix = format!("{}-", mc_version);
    if let Ok(resp) = http.get(FORGE_METADATA).send().await {
        if let Ok(text) = resp.text().await {
            for line in text.lines() {
                let trimmed = line.trim();
                if trimmed.starts_with("<version>") && trimmed.ends_with("</version>") {
                    let v = trimmed
                        .trim_start_matches("<version>")
                        .trim_end_matches("</version>")
                        .trim();
                    if v.starts_with(&prefix) {
                        let forge_ver = v.trim_start_matches(&prefix).to_string();
                        if seen.insert(forge_ver.clone()) {
                            versions.push(LoaderVersion {
                                id: forge_ver,
                                stable: true,
                            });
                        }
                    }
                }
            }
        }
    }

    if versions.is_empty() {
        let default_ver = match mc_version {
            "1.20.1" => "47.4.20",
            "1.19.2" => "43.4.2",
            "1.18.2" => "40.2.21",
            "1.16.5" => "36.2.42",
            "1.12.2" => "14.23.5.2860",
            _ => "47.4.20",
        };
        versions.push(LoaderVersion {
            id: default_ver.to_string(),
            stable: true,
        });
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
                let dest = libraries_dir.join(rel);
                if let Some(parent) = dest.parent() {
                    let _ = std::fs::create_dir_all(parent);
                }
                if let Ok(mut outfile) = std::fs::File::create(&dest) {
                    let _ = std::io::copy(&mut file, &mut outfile);
                }
            }
        }
    }
    Ok(())
}

fn extract_version_json_from_bytes(bytes: &[u8]) -> AppResult<String> {
    let mut archive = zip::ZipArchive::new(Cursor::new(bytes))
        .map_err(|e| AppError::Internal(format!("Failed to open Forge installer zip: {e}")))?;

    let mut version_file = archive
        .by_name("version.json")
        .map_err(|e| AppError::Internal(format!("Forge installer missing version.json: {e}")))?;
    let mut s = String::new();
    version_file
        .read_to_string(&mut s)
        .map_err(|e| AppError::Internal(format!("Failed to read Forge version.json: {e}")))?;
    Ok(s)
}

fn find_java_binary(libraries_dir: &Path, mc_version: &str) -> std::path::PathBuf {
    let major = match mc_version {
        v if v.starts_with("1.20.5") || v.starts_with("1.20.6")
            || v.starts_with("1.21")
            || (v.starts_with("1.2") && {
                let patch: u32 = v.split('.').nth(1).and_then(|s| s.parse().ok()).unwrap_or(0);
                patch >= 22
            })
            || v.starts_with("2") => 21,
        v if v.starts_with("1.17") || v.starts_with("1.18") || v.starts_with("1.19")
            || v.starts_with("1.20") => 17,
        _ => 8,
    };
    if let Some(parent) = libraries_dir.parent() {
        let bin_name = if cfg!(windows) { "java.exe" } else { "java" };
        let preferred = parent.join("java").join(major.to_string()).join("bin").join(bin_name);
        if preferred.exists() {
            return preferred;
        }
        let fallbacks: &[u32] = match major {
            21 => &[21, 17],
            17 => &[17, 21],
            _ => &[8, 17, 21],
        };
        for &v in fallbacks {
            let b = parent.join("java").join(v.to_string()).join("bin").join(bin_name);
            if b.exists() {
                return b;
            }
        }
    }
    if let Ok(java_home) = std::env::var("JAVA_HOME") {
        let b = std::path::PathBuf::from(java_home).join("bin").join(if cfg!(windows) { "java.exe" } else { "java" });
        if b.exists() {
            return b;
        }
    }
    if let Ok(found) = which::which("java") {
        return found;
    }
    std::path::PathBuf::from(if cfg!(windows) { "java.exe" } else { "java" })
}

fn prepare_patched_installer(installer_path: &Path) -> AppResult<std::path::PathBuf> {
    let file = std::fs::File::open(installer_path)
        .map_err(|e| AppError::Internal(format!("Failed to open Forge installer: {e}")))?;
    let mut archive = zip::ZipArchive::new(std::io::BufReader::new(file))
        .map_err(|e| AppError::Internal(format!("Failed to read Forge installer zip: {e}")))?;

    let mut has_processor_outputs = false;
    for i in 0..archive.len() {
        if let Ok(mut f) = archive.by_index(i) {
            if f.name() == "install_profile.json" {
                let mut content = String::new();
                if f.read_to_string(&mut content).is_ok() {
                    if let Ok(json_val) = serde_json::from_str::<serde_json::Value>(&content) {
                        if let Some(processors) = json_val.get("processors").and_then(|p| p.as_array()) {
                            for proc in processors {
                                if let Some(outputs) = proc.get("outputs") {
                                    if outputs.as_object().map(|o| !o.is_empty()).unwrap_or(false) {
                                        has_processor_outputs = true;
                                        break;
                                    }
                                }
                            }
                        }
                    }
                }
                break;
            }
        }
    }

    if !has_processor_outputs {
        return Ok(installer_path.to_path_buf());
    }

    let patched_path = installer_path.with_extension("patched.jar");
    let out_file = std::fs::File::create(&patched_path)
        .map_err(|e| AppError::Internal(format!("Failed to create patched Forge installer: {e}")))?;
    let mut zip_writer = zip::ZipWriter::new(out_file);
    let options = zip::write::FileOptions::default()
        .compression_method(zip::CompressionMethod::Deflated);

    let file = std::fs::File::open(installer_path)
        .map_err(|e| AppError::Internal(format!("Failed to re-open Forge installer: {e}")))?;
    let mut archive = zip::ZipArchive::new(std::io::BufReader::new(file))
        .map_err(|e| AppError::Internal(format!("Failed to re-read Forge installer: {e}")))?;

    for i in 0..archive.len() {
        let mut entry = archive.by_index(i)
            .map_err(|e| AppError::Internal(format!("Failed to read entry from installer: {e}")))?;
        let name = entry.name().to_string();

        if name == "install_profile.json" {
            let mut content = String::new();
            entry.read_to_string(&mut content).map_err(|e| AppError::Internal(format!("Failed to read install_profile.json: {e}")))?;
            let mut json_val: serde_json::Value = serde_json::from_str(&content)
                .map_err(|e| AppError::Internal(format!("Failed to parse install_profile.json: {e}")))?;
            if let Some(processors) = json_val.get_mut("processors").and_then(|p| p.as_array_mut()) {
                for proc in processors {
                    if let Some(outputs) = proc.get_mut("outputs") {
                        *outputs = serde_json::json!({});
                    }
                }
            }
            let modified = serde_json::to_vec(&json_val)
                .map_err(|e| AppError::Internal(format!("Failed to serialize install_profile.json: {e}")))?;
            zip_writer.start_file(&name, options)
                .map_err(|e| AppError::Internal(format!("Failed to start zip entry: {e}")))?;
            zip_writer.write_all(&modified)
                .map_err(|e| AppError::Internal(format!("Failed to write zip entry: {e}")))?;
        } else {
            zip_writer.start_file(&name, options)
                .map_err(|e| AppError::Internal(format!("Failed to start zip entry: {e}")))?;
            std::io::copy(&mut entry, &mut zip_writer)
                .map_err(|e| AppError::Internal(format!("Failed to copy zip entry: {e}")))?;
        }
    }
    zip_writer.finish()
        .map_err(|e| AppError::Internal(format!("Failed to finish zip writer: {e}")))?;

    tracing::info!(patched = %patched_path.display(), "Successfully created patched Forge installer");
    Ok(patched_path)
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

    if !installer_dest.exists() {
        if let Some(parent) = installer_dest.parent() {
            tokio::fs::create_dir_all(parent).await?;
        }
        tracing::info!(version = %full_version, url = %installer_url, "Downloading Forge installer");
        let resp = http
            .get(&installer_url)
            .send()
            .await
            .map_err(|e| AppError::Internal(format!("Failed to download Forge installer: {e}")))?
            .error_for_status()
            .map_err(|e| AppError::Internal(format!("Forge installer not found ({full_version}): {e}")))?;
        {
            use futures_util::StreamExt;
            use tokio::io::AsyncWriteExt;
            let mut stream = resp.bytes_stream();
            let mut dest = tokio::fs::File::create(&installer_dest).await?;
            while let Some(chunk) = stream.next().await {
                let chunk = chunk
                    .map_err(|e| AppError::Internal(format!("Failed to read Forge installer: {e}")))?;
                dest.write_all(&chunk).await?;
            }
            dest.flush().await?;
        }
    }

    let data_dir = libraries_dir.parent().unwrap_or(libraries_dir);
    let client_rel = format!(
        "net/minecraftforge/forge/{}/forge-{}-client.jar",
        full_version, full_version
    );
    let client_dest = libraries_dir.join(&client_rel);

    if !client_dest.exists() && installer_dest.exists() {
        let profiles_file = data_dir.join("launcher_profiles.json");
        if !profiles_file.exists() {
            let _ = tokio::fs::write(&profiles_file, b"{\"profiles\":{}}").await;
        }

        let java_bin = find_java_binary(libraries_dir, mc_version);
        tracing::info!(java = %java_bin.display(), installer = %installer_dest.display(), "Executing Forge installer");
        let _ = tokio::process::Command::new(&java_bin)
            .arg("-jar")
            .arg(&installer_dest)
            .arg("--installClient")
            .arg(data_dir)
            .stdin(std::process::Stdio::null())
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .status()
            .await;

        if !client_dest.exists() {
            if let Ok(patched) = prepare_patched_installer(&installer_dest) {
                if patched != installer_dest && patched.exists() {
                    let _ = tokio::process::Command::new(&java_bin)
                        .arg("-jar")
                        .arg(&patched)
                        .arg("--installClient")
                        .arg(data_dir)
                        .stdin(std::process::Stdio::null())
                        .stdout(std::process::Stdio::null())
                        .stderr(std::process::Stdio::null())
                        .status()
                        .await;
                    let _ = tokio::fs::remove_file(&patched).await;
                }
            }
        }
    }

    let installer_bytes = tokio::fs::read(&installer_dest).await?;
    let _ = extract_installer_maven_files(&installer_bytes, libraries_dir);

    let versions_dir = data_dir.join("versions");
    let candidate_ids = [
        format!("{}-forge-{}", mc_version, chosen_version),
        format!("{}-forge{}", mc_version, chosen_version),
        format!("forge-{}-{}", mc_version, chosen_version),
        full_version.clone(),
        format!("{}-{}", mc_version, chosen_version),
        format!("forge-{}", chosen_version),
    ];

    let mut installed_json_path = None;
    for cand in &candidate_ids {
        let p = versions_dir.join(cand).join(format!("{}.json", cand));
        if p.exists() {
            installed_json_path = Some(p);
            break;
        }
    }

    if installed_json_path.is_none() && versions_dir.exists() {
        if let Ok(mut entries) = tokio::fs::read_dir(&versions_dir).await {
            while let Ok(Some(entry)) = entries.next_entry().await {
                let name = entry.file_name().to_string_lossy().to_string();
                if (name.contains(mc_version) || name.contains(&chosen_version)) && name.to_lowercase().contains("forge") {
                    let cand_file = entry.path().join(format!("{}.json", name));
                    if cand_file.exists() {
                        installed_json_path = Some(cand_file);
                        break;
                    }
                }
            }
        }
    }

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
        let (rel_path_str, download_url) = if let Some(ref d) = lib.downloads {
            if let Some(ref art) = d.artifact {
                let path = art.path.clone().unwrap_or_else(|| {
                    crate::core::launcher::lib_path_from_name(&std::path::PathBuf::new(), &lib.name)
                        .to_string_lossy()
                        .replace('\\', "/")
                });
                let url = art.url.clone().filter(|u| !u.trim().is_empty()).unwrap_or_else(|| {
                    format!("{}/{}", FORGE_MAVEN, path.trim_start_matches('/'))
                });
                (path, url)
            } else {
                let p = crate::core::launcher::lib_path_from_name(&std::path::PathBuf::new(), &lib.name)
                    .to_string_lossy()
                    .replace('\\', "/");
                let u = format!("{}/{}", FORGE_MAVEN, p.trim_start_matches('/'));
                (p, u)
            }
        } else {
            let p = crate::core::launcher::lib_path_from_name(&std::path::PathBuf::new(), &lib.name)
                .to_string_lossy()
                .replace('\\', "/");
            let u = format!("{}/{}", FORGE_MAVEN, p.trim_start_matches('/'));
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
                let fallback_mojang = format!("https://libraries.minecraft.net/{}", rel_path_str.trim_start_matches('/'));
                if let Ok(resp) = http.get(&fallback_mojang).send().await {
                    if resp.status().is_success() {
                        if let Ok(b) = resp.bytes().await {
                            let _ = tokio::fs::write(&dest, &b).await;
                            downloaded = true;
                        }
                    }
                }
            }

            if !downloaded {
                let fallback_central = format!("https://repo1.maven.org/maven2/{}", rel_path_str.trim_start_matches('/'));
                if let Ok(resp) = http.get(&fallback_central).send().await {
                    if resp.status().is_success() {
                        if let Ok(b) = resp.bytes().await {
                            let _ = tokio::fs::write(&dest, &b).await;
                        }
                    }
                }
            }
        }

        if dest.exists() && !classpath_entries.contains(&dest) {
            classpath_entries.push(dest);
        }
    }

    let universal_rel = format!(
        "net/minecraftforge/forge/{}/forge-{}-universal.jar",
        full_version, full_version
    );
    let universal_dest = libraries_dir.join(&universal_rel);
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

    let default_ver = match mc_version {
        "1.20.1" => "47.4.20",
        "1.19.2" => "43.4.2",
        "1.18.2" => "40.2.21",
        "1.16.5" => "36.2.42",
        "1.12.2" => "14.23.5.2860",
        _ => "47.4.20",
    };
    Ok(default_ver.to_string())
}
