use std::path::{Path, PathBuf};
use crate::error::{AppError, AppResult};
use futures_util::StreamExt;
use sha2::{Digest, Sha256};
use tokio::io::AsyncWriteExt;

static INSTALL_LOCK: tokio::sync::Mutex<()> = tokio::sync::Mutex::const_new(());

pub async fn ensure(http: &reqwest::Client, data_dir: &Path, major: u32) -> AppResult<PathBuf> {
    let _guard = INSTALL_LOCK.lock().await;
    let destination = data_dir.join("installer-java").join(major.to_string());
    let binary = destination.join("bin/java");
    if binary.is_file() { return Ok(binary); }
    let arch = match std::env::consts::ARCH {
        "x86_64" => "x64", "aarch64" => "aarch64",
        _ => return Err(AppError::InvalidState("Arquitetura sem runtime de instalação gerenciado".into())),
    };
    let repository = format!("adoptium/temurin{major}-binaries");
    let metadata: serde_json::Value = http.get(format!("https://api.github.com/repos/{repository}/releases/latest"))
        .header("User-Agent", "Luxmc").send().await?.error_for_status()?.json().await?;
    let prefix = format!("OpenJDK{major}U-jre_{arch}_linux_hotspot_");
    let assets = metadata["assets"].as_array().ok_or_else(|| AppError::InvalidState("Release Java inválido".into()))?;
    let package = assets.iter().find(|a| a["name"].as_str().is_some_and(|name| name.starts_with(&prefix) && name.ends_with(".tar.gz")))
        .ok_or_else(|| AppError::NotFound("Runtime Temurin não disponível para esta arquitetura".into()))?;
    let name = package["name"].as_str().unwrap();
    let checksum = assets.iter().find(|a| a["name"].as_str() == Some(&format!("{name}.sha256.txt")))
        .ok_or_else(|| AppError::InvalidState("Checksum do runtime não disponível".into()))?;
    let trusted_url = |asset: &serde_json::Value| -> AppResult<String> {
        let value = asset["browser_download_url"].as_str().unwrap_or_default();
        if !value.starts_with(&format!("https://github.com/{repository}/releases/download/")) { return Err(AppError::InvalidInput("Origem de runtime inválida".into())); }
        Ok(value.to_owned())
    };
    let expected = http.get(trusted_url(checksum)?).send().await?.error_for_status()?.text().await?;
    let expected = expected.split_whitespace().next().unwrap_or_default();
    if expected.len() != 64 || !expected.bytes().all(|b| b.is_ascii_hexdigit()) { return Err(AppError::InvalidInput("Checksum Java inválido".into())); }
    let parent = destination.parent().unwrap();
    tokio::fs::create_dir_all(parent).await?;
    let staging = parent.join(format!(".install-{}", uuid::Uuid::new_v4()));
    tokio::fs::create_dir(&staging).await?;
    let result = async {
        let archive_path = staging.join("runtime.tar.gz");
        let mut file = tokio::fs::File::create(&archive_path).await?;
        let mut stream = http.get(trusted_url(package)?).send().await?.error_for_status()?.bytes_stream();
        let mut digest = Sha256::new();
        let mut size = 0u64;
        while let Some(chunk) = stream.next().await {
            let chunk = chunk?;
            size += chunk.len() as u64;
            if size > 256 * 1024 * 1024 { return Err(AppError::InvalidInput("Runtime excede 256 MiB".into())); }
            digest.update(&chunk);
            file.write_all(&chunk).await?;
        }
        file.flush().await?;
        drop(file);
        if !format!("{:x}", digest.finalize()).eq_ignore_ascii_case(expected) { return Err(AppError::InvalidState("Checksum do runtime não confere".into())); }
        let extraction = staging.join("extracted");
        let target = extraction.clone();
        tokio::task::spawn_blocking(move || -> AppResult<()> {
            let archive = std::fs::File::open(archive_path)?;
            tar::Archive::new(flate2::read::GzDecoder::new(archive)).unpack(target)?;
            Ok(())
        }).await.map_err(|e| AppError::Internal(e.to_string()))??;
        let mut roots = tokio::fs::read_dir(extraction).await?;
        while let Some(root) = roots.next_entry().await? {
            if root.path().join("bin/java").is_file() {
                tokio::fs::rename(root.path(), &destination).await?;
                return Ok(binary.clone());
            }
        }
        Err(AppError::InvalidState("Runtime Java sem executável".into()))
    }.await;
    let _ = tokio::fs::remove_dir_all(staging).await;
    result
}
