#![allow(non_snake_case)]

use serde::{Deserialize, Serialize};
use tauri::State;

use crate::db;
use crate::error::{AppError, AppResult};
use crate::state::AppState;

use crate::core::launcher::jvm_arg_allowed_on_current_os;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct JvmValidationResult {
    pub valid: bool,
    pub rejected: Vec<String>,
    pub normalized: String,
    pub suggestions: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct JavaRuntimeInfo {
    pub available: bool,
    pub major: u32,
    pub path: Option<String>,
    pub version: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CrashSummary {
    pub category: String,
    pub probable_cause: String,
    pub hint: String,
}

fn tokenize_jvm_args(input: &str) -> Vec<String> {
    let mut tokens = Vec::new();
    let mut current = String::new();
    let mut in_quotes = false;
    for ch in input.chars() {
        match ch {
            '"' => {
                in_quotes = !in_quotes;
            }
            ' ' | '\t' | '\n' if !in_quotes => {
                if !current.is_empty() {
                    tokens.push(current.clone());
                    current.clear();
                }
            }
            _ => current.push(ch),
        }
    }
    if !current.is_empty() {
        tokens.push(current);
    }
    tokens
}

#[tauri::command]
pub async fn jvm_args_validate(input: String) -> AppResult<JvmValidationResult> {
    let tokens = tokenize_jvm_args(&input);
    let mut rejected = Vec::new();
    let mut accepted = Vec::new();
    let mut suggestions = Vec::new();

    for token in &tokens {
        if jvm_arg_allowed_on_current_os(token) {
            accepted.push(token.clone());
        } else {
            rejected.push(token.clone());
            if token == "-XstartOnFirstThread" {
                suggestions.push(
					"-XstartOnFirstThread is a macOS-only flag. It is unsafe on Linux and ignored on Windows."
						.into(),
				);
            } else if token.starts_with("-XX:HeapDumpPath=MojangTricksIntelDriversForPerformance") {
                suggestions.push(
                    "Heap dump paths are Windows-only Mojang driver workarounds. Strip on Linux."
                        .into(),
                );
            } else {
                suggestions.push(format!(
                    "Remove the flag \"{token}\" — it is not safe on this OS."
                ));
            }
        }
    }

    let normalized = accepted.join(" ");
    Ok(JvmValidationResult {
        valid: rejected.is_empty(),
        rejected,
        normalized,
        suggestions,
    })
}

#[tauri::command]
pub async fn java_runtime_status(
    state: State<'_, AppState>,
    major: u32,
) -> AppResult<JavaRuntimeInfo> {
    use crate::core::java::JavaRuntimeManager;

    let base_dir = directories::ProjectDirs::from("io", "github", "Luxmc")
        .ok_or_else(|| AppError::InvalidState("could not determine data dir".into()))?;
    let manager = JavaRuntimeManager::new(state.http.clone(), base_dir.data_dir().to_path_buf());
    let path = manager.ensure_java(major).await.ok();
    let version = path
        .as_ref()
        .and_then(|p| std::process::Command::new(p).arg("-version").output().ok())
        .map(|out| String::from_utf8_lossy(&out.stderr).to_string());

    Ok(JavaRuntimeInfo {
        available: path.is_some(),
        major,
        path: path.map(|p| p.to_string_lossy().to_string()),
        version,
    })
}

#[tauri::command]
pub async fn crash_summary(lines: Vec<String>) -> AppResult<CrashSummary> {
    let joined = lines.join("\n");
    let lower = joined.to_lowercase();

    if lower.contains("libflite.so") || lower.contains("flite") {
        return Ok(CrashSummary {
			category: "narrator".into(),
			probable_cause: "The Minecraft narrator is missing libflite.so on Linux".into(),
			hint: "Install the system package 'flite' (Arch: pacman -S flite) to enable narration. The game still runs without it.".into(),
		});
    }
    if lower.contains("unsatisfiedlinkerror") || lower.contains("failed to load library") {
        return Ok(CrashSummary {
			category: "missing_native".into(),
			probable_cause: "A required native library is missing or not in java.library.path".into(),
			hint: "Re-run the version install to re-extract natives, or check that no custom JVM flags override the natives directory.".into(),
		});
    }
    if lower.contains("invalidcredentials")
        || lower.contains("401")
        || lower.contains("rejected by")
    {
        return Ok(CrashSummary {
			category: "auth".into(),
			probable_cause: "The Microsoft or session token was rejected".into(),
			hint: "Sign in again with Microsoft login, or use Dev/Offline mode if you do not need premium services.".into(),
		});
    }
    if lower.contains("outofmemory")
        || lower.contains("could not reserve enough space")
        || lower.contains("gc overhead")
    {
        return Ok(CrashSummary {
			category: "memory".into(),
			probable_cause: "JVM ran out of memory".into(),
			hint: "Increase the per-instance RAM in the instance settings (recommended 4–6 GB for modded 1.18+).".into(),
		});
    }
    if lower.contains("java.lang.unsupportedclassversionerror")
        || lower.contains("class file version")
    {
        return Ok(CrashSummary {
			category: "java_version".into(),
			probable_cause: "The installed Java is too old for this version".into(),
			hint: "Luxmc should auto-download the correct Java. If it failed, check the launcher log for download errors.".into(),
		});
    }
    if lower.contains("failed to bind") || lower.contains("address already in use") {
        return Ok(CrashSummary {
			category: "port".into(),
			probable_cause: "A required port is already in use".into(),
			hint: "Another game or tool is using the same port. Close other instances of Minecraft or change the server port.".into(),
		});
    }

    Ok(CrashSummary {
        category: "unknown".into(),
        probable_cause: "No specific cause was identified".into(),
        hint: "Check the full log for clues, or open an issue with the log attached.".into(),
    })
}

#[tauri::command]
pub async fn instance_export_zip(
    _state: State<'_, AppState>,
    profileId: String,
    outputPath: String,
) -> AppResult<String> {
    let conn = db::shared_db().await?;
    let row: crate::db::models::ProfileRow = sqlx::query_as("SELECT * FROM profiles WHERE id = ?")
        .bind(&profileId)
        .fetch_optional(conn.pool())
        .await?
        .ok_or_else(|| AppError::NotFound(format!("profile {profileId} not found")))?;

    let file = std::fs::File::create(&outputPath)?;
    let mut zip = zip::ZipWriter::new(file);
    let options = zip::write::FileOptions::default()
        .compression_method(zip::CompressionMethod::Deflated)
        .unix_permissions(0o644);

    {
        let manifest = serde_json::json!({
            "formatVersion": 1,
            "name": row.name,
            "icon": row.icon,
            "mcVersion": row.mc_version,
            "loader": row.loader,
            "loaderVersion": row.loader_version,
            "javaPath": row.java_path,
            "jvmArgs": row.jvm_args,
            "resolutionW": row.resolution_w,
            "resolutionH": row.resolution_h,
            "fullscreen": row.fullscreen,
            "ramMb": row.ram_mb,
            "instanceGroup": row.instance_group,
            "notes": row.notes,
        });
        zip.start_file("luxmc.profile.json", options)?;
        zip.write_all(manifest.to_string().as_bytes())?;
    }

    let game_dir = std::path::PathBuf::from(&row.game_dir);
    if game_dir.is_dir() {
        for entry in walkdir(&game_dir) {
            let path = entry.path();
            if path.is_file() {
                let name = path
                    .strip_prefix(&game_dir)
                    .map_err(|e| AppError::Internal(format!("strip prefix: {e}")))?;
                zip.start_file(name.to_string_lossy(), options)?;
                let bytes = std::fs::read(path)?;
                zip.write_all(&bytes)?;
            }
        }
    }

    zip.finish()?;
    Ok(outputPath)
}

fn walkdir(dir: &std::path::Path) -> Vec<std::fs::DirEntry> {
    let mut out = Vec::new();
    let entries = match std::fs::read_dir(dir) {
        Ok(e) => e,
        Err(_) => return out,
    };
    for entry in entries {
        let Ok(entry) = entry else { continue };
        let path = entry.path();
        out.push(entry);
        if path.is_dir() {
            out.extend(walkdir(&path));
        }
    }
    out
}

#[tauri::command]
pub async fn instance_backup_saves(
    _state: State<'_, AppState>,
    profileId: String,
    outputPath: String,
) -> AppResult<String> {
    let conn = db::shared_db().await?;
    let row: crate::db::models::ProfileRow = sqlx::query_as("SELECT * FROM profiles WHERE id = ?")
        .bind(&profileId)
        .fetch_optional(conn.pool())
        .await?
        .ok_or_else(|| AppError::NotFound(format!("profile {profileId} not found")))?;

    let saves_dir = std::path::PathBuf::from(&row.game_dir).join("saves");
    if !saves_dir.is_dir() {
        return Err(AppError::NotFound(format!(
            "saves directory does not exist for instance {}",
            row.name
        )));
    }

    let file = std::fs::File::create(&outputPath)?;
    let mut zip = zip::ZipWriter::new(file);
    let options = zip::write::FileOptions::default()
        .compression_method(zip::CompressionMethod::Deflated)
        .unix_permissions(0o644);

    for entry in walkdir(&saves_dir) {
        let path = entry.path();
        if path.is_file() {
            let name = path
                .strip_prefix(&saves_dir)
                .map_err(|e| AppError::Internal(format!("strip prefix: {e}")))?;
            zip.start_file(format!("saves/{}", name.to_string_lossy()), options)?;
            let bytes = std::fs::read(path)?;
            zip.write_all(&bytes)?;
        }
    }

    zip.finish()?;
    Ok(outputPath)
}

#[tauri::command]
pub async fn instance_restore_saves(
    _state: State<'_, AppState>,
    profileId: String,
    zipPath: String,
) -> AppResult<String> {
    use std::io::Read;
    let conn = db::shared_db().await?;
    let row: crate::db::models::ProfileRow = sqlx::query_as("SELECT * FROM profiles WHERE id = ?")
        .bind(&profileId)
        .fetch_optional(conn.pool())
        .await?
        .ok_or_else(|| AppError::NotFound(format!("profile {profileId} not found")))?;

    let saves_dir = std::path::PathBuf::from(&row.game_dir).join("saves");
    tokio::fs::create_dir_all(&saves_dir).await?;

    let file = std::fs::File::open(&zipPath)?;
    let mut archive = zip::ZipArchive::new(file)
        .map_err(|e| AppError::InvalidState(format!("invalid zip: {e}")))?;

    for i in 0..archive.len() {
        let mut entry = archive.by_index(i)?;
        let raw_name = entry.name().to_string();
        let stripped = raw_name
            .strip_prefix("saves/")
            .or_else(|| Some(raw_name.as_str()))
            .unwrap_or(raw_name.as_str());
        if stripped.is_empty() {
            continue;
        }
        let out_path = saves_dir.join(stripped);
        if entry.is_dir() {
            std::fs::create_dir_all(&out_path)?;
        } else {
            if let Some(parent) = out_path.parent() {
                std::fs::create_dir_all(parent)?;
            }
            let mut buffer = Vec::new();
            entry.read_to_end(&mut buffer)?;
            std::fs::write(&out_path, &buffer)?;
        }
    }

    Ok(saves_dir.to_string_lossy().to_string())
}

#[tauri::command]
pub async fn instance_repair(
    state: State<'_, AppState>,
    app: tauri::AppHandle,
    profileId: String,
) -> AppResult<()> {
    let conn = db::shared_db().await?;
    let row: crate::db::models::ProfileRow = sqlx::query_as("SELECT * FROM profiles WHERE id = ?")
        .bind(&profileId)
        .fetch_optional(conn.pool())
        .await?
        .ok_or_else(|| AppError::NotFound(format!("profile {profileId} not found")))?;

    use crate::core::minecraft;
    let version_row = crate::db::schema::versions::get(&conn, &row.mc_version)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("version {} not found", row.mc_version)))?;
    let detail = minecraft::fetch_version_detail(&state.http, &version_row.url).await?;

    let base_dir = directories::ProjectDirs::from("io", "github", "Luxmc")
        .ok_or_else(|| AppError::InvalidState("could not determine data dir".into()))?;
    let data_dir = base_dir.data_dir().to_path_buf();
    let downloader = crate::core::downloader::DownloadManager::new(state.http.clone(), data_dir)
        .with_app(app.clone());
    downloader.download_version(&detail).await?;
    downloader.validate_version(&detail).await?;
    Ok(())
}

#[tauri::command]
pub async fn instance_disk_usage(_state: State<'_, AppState>, profileId: String) -> AppResult<i64> {
    let conn = db::shared_db().await?;
    let row: crate::db::models::ProfileRow = sqlx::query_as("SELECT * FROM profiles WHERE id = ?")
        .bind(&profileId)
        .fetch_optional(conn.pool())
        .await?
        .ok_or_else(|| AppError::NotFound(format!("profile {profileId} not found")))?;

    let game_dir = std::path::PathBuf::from(&row.game_dir);
    let mut total: i64 = 0;
    for entry in walkdir(&game_dir) {
        let path = entry.path();
        if path.is_file() {
            if let Ok(meta) = path.metadata() {
                total += meta.len() as i64;
            }
        }
    }
    Ok(total)
}

#[tauri::command]
pub async fn version_repair(
    state: State<'_, AppState>,
    app: tauri::AppHandle,
    versionId: String,
) -> AppResult<()> {
    let conn = db::shared_db().await?;
    let version_row = crate::db::schema::versions::get(&conn, &versionId)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("version {versionId} not found")))?;

    use crate::core::minecraft;
    let detail = minecraft::fetch_version_detail(&state.http, &version_row.url).await?;
    let base_dir = directories::ProjectDirs::from("io", "github", "Luxmc")
        .ok_or_else(|| AppError::InvalidState("could not determine data dir".into()))?;
    let data_dir = base_dir.data_dir().to_path_buf();
    let downloader = crate::core::downloader::DownloadManager::new(state.http.clone(), data_dir)
        .with_app(app.clone());
    downloader.download_version(&detail).await?;
    downloader.validate_version(&detail).await?;
    Ok(())
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MclogsResponse {
    pub success: bool,
    pub id: Option<String>,
    pub url: Option<String>,
    pub error: Option<String>,
}

#[tauri::command]
pub async fn share_log_mclogs(
    state: State<'_, AppState>,
    content: String,
) -> AppResult<String> {
    if content.trim().is_empty() {
        return Err(AppError::InvalidInput("Log content cannot be empty".into()));
    }
    let form = [("content", content.as_str())];
    let resp = state
        .http
        .post("https://api.mclo.gs/1/log")
        .form(&form)
        .send()
        .await?
        .error_for_status()?;
    let data: MclogsResponse = resp.json().await?;
    if data.success {
        if let Some(url) = data.url {
            return Ok(url);
        }
    }
    Err(AppError::Internal(
        data.error.unwrap_or_else(|| "Failed to upload log to mclo.gs".to_string()),
    ))
}

use std::io::Write;

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn jvm_args_rejects_unsupported_flags() {
        let r = jvm_args_validate(
			"-Xms2G -Xmx4G -XstartOnFirstThread -XX:HeapDumpPath=MojangTricksIntelDriversForPerformance_javaw.exe_minecraft.exe.heapdump".to_string(),
		)
		.await
		.unwrap();
        assert!(!r.valid);
        assert!(r.rejected.iter().any(|x| x == "-XstartOnFirstThread"));
        assert!(r
            .rejected
            .iter()
            .any(|x| x.starts_with("-XX:HeapDumpPath=")));
        assert!(r.normalized.contains("-Xms2G"));
        assert!(r.normalized.contains("-Xmx4G"));
    }

    #[tokio::test]
    async fn jvm_args_keeps_safe_flags() {
        let r = jvm_args_validate("-Xms2G -Xmx4G -Dfile.encoding=UTF-8".to_string())
            .await
            .unwrap();
        assert!(r.valid);
        assert!(r.rejected.is_empty());
        assert_eq!(r.normalized, "-Xms2G -Xmx4G -Dfile.encoding=UTF-8");
    }

    #[tokio::test]
    async fn jvm_args_handles_quoted_paths() {
        let r =
            jvm_args_validate(r#"-Djava.library.path="/opt/mc with spaces/natives""#.to_string())
                .await
                .unwrap();
        assert!(r.valid);
        assert_eq!(
            r.normalized,
            "-Djava.library.path=/opt/mc with spaces/natives"
        );
    }

    #[tokio::test]
    async fn crash_summary_classifies_flite() {
        let r = crash_summary(vec![
            "[Client] INFO: Loading".into(),
            "[Client] ERROR: Failed to load library flite (libflite.so)".into(),
        ])
        .await
        .unwrap();
        assert_eq!(r.category, "narrator");
        assert!(r.hint.contains("flite"));
    }

    #[tokio::test]
    async fn crash_summary_classifies_out_of_memory() {
        let r = crash_summary(vec!["java.lang.OutOfMemoryError: Java heap space".into()])
            .await
            .unwrap();
        assert_eq!(r.category, "memory");
    }

    #[tokio::test]
    async fn crash_summary_classifies_auth_failure() {
        let r = crash_summary(vec![
            "com.mojang.authlib.exceptions.InvalidCredentialsException: Status: 401".into(),
        ])
        .await
        .unwrap();
        assert_eq!(r.category, "auth");
    }

    #[tokio::test]
    async fn crash_summary_classifies_port_in_use() {
        let r = crash_summary(vec!["BindException: Address already in use".into()])
            .await
            .unwrap();
        assert_eq!(r.category, "port");
    }

    #[tokio::test]
    async fn crash_summary_falls_back_to_unknown() {
        let r = crash_summary(vec!["some unrelated warning".into()])
            .await
            .unwrap();
        assert_eq!(r.category, "unknown");
    }
}
