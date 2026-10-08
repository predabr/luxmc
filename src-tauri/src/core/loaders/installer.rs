use std::io::{Cursor, Read};
use std::path::Path;
use crate::error::{AppError, AppResult};
use futures_util::StreamExt;

pub async fn run(http: &reqwest::Client, installer: &Path, data_dir: &Path, mc_version: &str) -> AppResult<()> {
    let installer_bytes = tokio::fs::read(installer).await?;
    let install_profile=read_install_profile(&installer_bytes)?;
    if let Some(profile) = &install_profile {
        if profile.get("versionInfo").is_some() {
            return install_legacy_client(installer, data_dir, &profile).await;
        }
    }

    let major = crate::core::minecraft::detect_java_major_from_version_id(mc_version);
    let java_future=async {
        #[cfg(target_os = "linux")]
        {tokio::time::timeout(std::time::Duration::from_secs(300), super::installer_java::ensure(http, data_dir, major)).await.map_err(|_| AppError::InvalidState("Download do Java de instalação excedeu 5 minutos".into()))?}
        #[cfg(not(target_os = "linux"))]
        {crate::core::java::JavaRuntimeManager::new(http.clone(), data_dir.to_owned()).ensure_java(major).await}
    };
    let (java,())=tokio::join!(java_future,prefetch(http,data_dir,install_profile.as_ref()));
    let java=java?;
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
    let entry = match archive.by_name("install_profile.json") {
        Ok(entry) => entry,
        Err(_) => return Ok(None),
    };
    let mut raw = String::new();
    if entry.size()>8*1024*1024{return Err(AppError::InvalidInput("Metadados do instalador acima de 8 MiB".into()));}
    entry.take(8*1024*1024)
        .read_to_string(&mut raw)
        .map_err(|e| AppError::Internal(format!("Failed to read install_profile.json: {e}")))?;
    let value: serde_json::Value = serde_json::from_str(&raw)
        .map_err(|e| AppError::Internal(format!("Failed to parse install_profile.json: {e}")))?;
    Ok(Some(value))
}

async fn prefetch(http:&reqwest::Client,data_dir:&Path,profile:Option<&serde_json::Value>) {
    let Some(libraries)=profile.and_then(|profile|profile["libraries"].as_array()) else{return;};
    let mut targets=std::collections::HashSet::new();let mut jobs=Vec::new();
    for library in libraries {
        let artifact=&library["downloads"]["artifact"];
        let (Some(path),Some(address))=(artifact["path"].as_str(),artifact["url"].as_str()) else{continue;};
        if address.is_empty() || crate::core::mods::pack_download::relative_path(path).is_err() || !targets.insert(path.to_owned()){continue;}
        let Ok(url)=url::Url::parse(address) else{continue;};
        if url.scheme()!="https" || !url.username().is_empty() || url.password().is_some() || !["maven.minecraftforge.net","maven.neoforged.net","libraries.minecraft.net","repo.maven.apache.org","repo1.maven.org"].contains(&url.host_str().unwrap_or("")){continue;}
        jobs.push((data_dir.join("libraries").join(path),address.to_owned(),artifact["size"].as_u64().unwrap_or(0),artifact["sha1"].as_str().unwrap_or("").to_owned()));
    }
    futures_util::stream::iter(jobs.into_iter().map(|(path,url,size,sha1)|async move {
        if let Err(error)=crate::core::downloader::ensure_artifact(http,&path,&url,size,&sha1).await{tracing::warn!(%error,"Biblioteca de instalação será tentada pelo instalador oficial");}
    })).buffer_unordered(6).for_each(|_|async{}).await;
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
        .unwrap_or_default()
        .to_string();

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
        let installer_bytes = tokio::fs::read(installer).await?;
        let dest_for_worker = universal_dest.clone();
        tokio::task::spawn_blocking(move || -> AppResult<()> {
            let mut archive = zip::ZipArchive::new(Cursor::new(installer_bytes))
                .map_err(|e| AppError::Internal(format!("Failed to open installer zip: {e}")))?;
            let mut entry = archive.by_name(&file_name).map_err(|e| {
                AppError::Internal(format!("Forge installer missing {file_name}: {e}"))
            })?;
            if let Some(parent) = dest_for_worker.parent() {
                std::fs::create_dir_all(parent)?;
            }
            let mut out = std::fs::File::create(&dest_for_worker)?;
            std::io::copy(&mut entry, &mut out)?;
            Ok(())
        })
        .await
        .map_err(|error| AppError::Internal(error.to_string()))??;
    }

    let version_json = serde_json::to_vec_pretty(&version_info)?;
    let version_dir = data_dir.join("versions").join(&id);
    tokio::fs::create_dir_all(&version_dir).await?;
    tokio::fs::write(version_dir.join(format!("{id}.json")), version_json).await?;

    tokio::fs::write(installer.with_extension("installed"), b"ok").await?;
    Ok(())
}
