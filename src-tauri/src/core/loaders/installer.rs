use std::path::Path;
use crate::error::{AppError, AppResult};

pub async fn run(http: &reqwest::Client, installer: &Path, data_dir: &Path, mc_version: &str) -> AppResult<()> {
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
