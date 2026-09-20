use serde::{Deserialize, Serialize};
use tauri::State;

use crate::core::java::JavaRuntimeManager;
use crate::error::AppResult;
use crate::state::AppState;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct JavaInstallStatus {
    pub major: u32,
    pub installed: bool,
    pub path: Option<String>,
    pub version_string: Option<String>,
    pub is_system: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct JavaScanResult {
    pub runtimes: Vec<JavaInstallStatus>,
}

#[tauri::command]
pub async fn java_scan(state: State<'_, AppState>, _app: tauri::AppHandle) -> AppResult<JavaScanResult> {
    java_scan_core(&state).await
}

pub async fn java_scan_core(state: &AppState) -> AppResult<JavaScanResult> {
    let base_dir = directories::ProjectDirs::from("io", "github", "Luxmc")
        .ok_or_else(|| crate::error::AppError::InvalidState("could not determine data dir".into()))?;
    let mgr = JavaRuntimeManager::new(state.http.clone(), base_dir.data_dir().to_path_buf());

    let mut runtimes = Vec::new();
    for major in [8u32, 17, 21] {
        let bin = base_dir.data_dir().join("java").join(major.to_string()).join("bin")
            .join(if cfg!(windows) { "java.exe" } else { "java" });
        let is_managed = bin.exists();

        let (found_path, version_string, is_system) = if is_managed {
            let vs = get_version_string(&bin);
            (Some(bin.to_string_lossy().to_string()), vs, false)
        } else {
            match mgr.find_system_java_pub(major) {
                Ok(p) => {
                    let vs = get_version_string(&p);
                    let path_str = p.to_string_lossy().to_string();
                    (Some(path_str), vs, true)
                }
                Err(_) => (None, None, false),
            }
        };

        runtimes.push(JavaInstallStatus {
            major,
            installed: found_path.is_some(),
            path: found_path,
            version_string,
            is_system,
        });
    }

    Ok(JavaScanResult { runtimes })
}

#[tauri::command]
pub async fn java_install(
    state: State<'_, AppState>,
    app: tauri::AppHandle,
    major: u32,
) -> AppResult<JavaInstallStatus> {
    if major != 8 && major != 17 && major != 21 {
        return Err(crate::error::AppError::InvalidState(format!("unsupported Java major version: {major}")));
    }

    let base_dir = directories::ProjectDirs::from("io", "github", "Luxmc")
        .ok_or_else(|| crate::error::AppError::InvalidState("could not determine data dir".into()))?;
    let mgr = JavaRuntimeManager::new(state.http.clone(), base_dir.data_dir().to_path_buf())
        .with_app(app);

    let path = mgr.ensure_java(major).await?;
    let version_string = get_version_string(&path);

    Ok(JavaInstallStatus {
        major,
        installed: true,
        path: Some(path.to_string_lossy().to_string()),
        version_string,
        is_system: false,
    })
}

#[tauri::command]
pub async fn java_uninstall(major: u32) -> AppResult<()> {
    if major != 8 && major != 17 && major != 21 {
        return Err(crate::error::AppError::InvalidState(format!("unsupported Java major version: {major}")));
    }
    let base_dir = directories::ProjectDirs::from("io", "github", "Luxmc")
        .ok_or_else(|| crate::error::AppError::InvalidState("could not determine data dir".into()))?;
    let java_dir = base_dir.data_dir().join("java").join(major.to_string());
    if java_dir.exists() {
        tokio::fs::remove_dir_all(&java_dir).await?;
    }
    Ok(())
}

fn get_version_string(path: &std::path::Path) -> Option<String> {
    crate::core::process::std_command(path)
        .arg("-version")
        .stderr(std::process::Stdio::piped())
        .output()
        .ok()
        .map(|o| {
            String::from_utf8_lossy(&o.stderr)
                .lines()
                .next()
                .unwrap_or("")
                .to_string()
        })
}
