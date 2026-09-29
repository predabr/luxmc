use std::io::{Cursor, Read};
use std::path::Path;
use crate::error::{AppError, AppResult};

pub async fn run(http: &reqwest::Client, installer: &Path, data_dir: &Path, mc_version: &str) -> AppResult<()> {
    let installer_bytes = tokio::fs::read(installer).await?;
    if let Some(profile) = read_install_profile(&installer_bytes)? {
        if profile.get("versionInfo").is_some() {
            return install_legacy_client(installer, data_dir, &profile).await;
        }
    }

    let major = crate::core::minecraft::detect_java_major_from_version_id(mc_version);
    #[cfg(target_os = "linux")]
    let java = tokio::time::timeout(std::time::Duration::from_secs(300), super::installer_java::ensure(http, data_dir, major)).await
        .map_err(|_| AppError::InvalidState("Download do Java de instalação excedeu 5 minutos".into()))??;
    #[cfg(not(target_os = "linux"))]
    let java = crate::core::java::JavaRuntimeManager::new(http.clone(), data_dir.to_owned()).ensure_java(major).await?;
    let log_path = installer.with_extension("install.log");
    let log = std::fs::File::create(&log_path)?;
    let mut command = crate::core::process::tokio_command(java);
    command.env_remove("LD_PRELOAD").env_remove("LD_LIBRARY_PATH").arg("-Duser.timezone=UTC").arg("-jar").arg(installer).arg("--installClient").arg(data_dir)
        .current_dir(data_dir).stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::from(log.try_clone()?)).stderr(std::process::Stdio::from(log))
        .kill_on_drop(true);
    let status = tokio::time::timeout(std::time::Duration::from_secs(900), command.status()).await
        .map_err(|_| AppError::InvalidState(format!("Instalador excedeu 15 minutos. Consulte {}", log_path.display())))??;
    if !status.success() {
        return Err(AppError::InvalidState(format!("Instalador falhou ({status}). Consulte {}", log_path.display())));
    }
    tokio::fs::write(installer.with_extension("installed"), b"ok").await?;
    Ok(())
}

fn read_install_profile(bytes: &[u8]) -> AppResult<Option<serde_json::Value>> {
    let mut archive = zip::ZipArchive::new(Cursor::new(bytes))
        .map_err(|e| AppError::Internal(format!("Failed to open installer zip: {e}")))?;
    let mut entry = match archive.by_name("install_profile.json") {
        Ok(entry) => entry,
        Err(_) => return Ok(None),
    };
    let mut raw = String::new();
    entry
        .read_to_string(&mut raw)
        .map_err(|e| AppError::Internal(format!("Failed to read install_profile.json: {e}")))?;
    let value: serde_json::Value = serde_json::from_str(&raw)
        .map_err(|e| AppError::Internal(format!("Failed to parse install_profile.json: {e}")))?;
    Ok(Some(value))
}

async fn install_legacy_client(
    installer: &Path,
    data_dir: &Path,
    profile: &serde_json::Value,
) -> AppResult<()> {
    let mut version_info = profile
        .get("versionInfo")
        .cloned()
        .ok_or_else(|| AppError::InvalidState("install_profile.json sem versionInfo".into()))?;
    let id = version_info
        .get("id")
        .and_then(|v| v.as_str())
        .ok_or_else(|| AppError::InvalidState("install_profile.json sem versionInfo.id".into()))?
        .to_string();

    let install = profile.get("install");
    let coords = install.and_then(|i| i.get("path")).and_then(|v| v.as_str()).unwrap_or_default();
    let file_name = install
        .and_then(|i| i.get("filePath"))
        .and_then(|v| v.as_str())
        .unwrap_or_default();

    let parts: Vec<&str> = coords.split(':').collect();
    let unsafe_coords = parts.len() < 3
        || parts
            .iter()
            .any(|p| p.contains("..") || p.contains('/') || p.contains('\\'));
    if unsafe_coords {
        return Err(AppError::InvalidState(format!(
            "install_profile.json com coordenadas Maven inválidas: {coords}"
        )));
    }

    if let Some(libs) = version_info.get_mut("libraries").and_then(|v| v.as_array_mut()) {
        libs.retain(|lib| lib.get("name").and_then(|n| n.as_str()) != Some(coords));
    }

    let libraries_dir = data_dir.join("libraries");
    let universal_rel = format!(
        "{}/{}/{}/{}-{}-universal.jar",
        parts[0].replace('.', "/"),
        parts[1],
        parts[2],
        parts[1],
        parts[2]
    );
    let universal_dest = libraries_dir.join(&universal_rel);
    if !universal_dest.exists() {
        let mut archive =
            zip::ZipArchive::new(Cursor::new(tokio::fs::read(installer).await?))
                .map_err(|e| AppError::Internal(format!("Failed to open installer zip: {e}")))?;
        let mut entry = archive.by_name(file_name).map_err(|e| {
            AppError::Internal(format!("Forge installer missing {file_name}: {e}"))
        })?;
        if let Some(parent) = universal_dest.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let mut out = std::fs::File::create(&universal_dest)?;
        std::io::copy(&mut entry, &mut out)?;
    }

    let version_json = serde_json::to_vec_pretty(&version_info)?;
    let version_dir = data_dir.join("versions").join(&id);
    tokio::fs::create_dir_all(&version_dir).await?;
    tokio::fs::write(version_dir.join(format!("{id}.json")), version_json).await?;

    tokio::fs::write(installer.with_extension("installed"), b"ok").await?;
    Ok(())
}
