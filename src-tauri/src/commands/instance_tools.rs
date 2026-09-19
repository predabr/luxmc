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

pub async fn java_runtime_status_core(
    http: reqwest::Client,
    major: u32,
) -> AppResult<JavaRuntimeInfo> {
    use crate::core::java::JavaRuntimeManager;

    let base_dir = directories::ProjectDirs::from("io", "github", "Luxmc")
        .ok_or_else(|| AppError::InvalidState("could not determine data dir".into()))?;
    let manager = JavaRuntimeManager::new(http, base_dir.data_dir().to_path_buf());
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
pub async fn java_runtime_status(
    state: State<'_, AppState>,
    major: u32,
) -> AppResult<JavaRuntimeInfo> {
    java_runtime_status_core(state.http.clone(), major).await
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

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InstanceShareManifest {
    pub name: String,
    pub mc_version: String,
    pub loader: String,
    pub loader_version: Option<String>,
    pub ram_mb: Option<i64>,
    pub jvm_args: Option<String>,
    pub mods: Vec<String>,
}

#[tauri::command]
pub async fn instance_export_share_code(
    profileId: String,
) -> AppResult<String> {
    let db = db::shared_db().await?;
    let row: crate::db::models::ProfileRow = sqlx::query_as("SELECT * FROM profiles WHERE id = ?")
        .bind(&profileId)
        .fetch_optional(db.pool())
        .await?
        .ok_or_else(|| AppError::NotFound(format!("profile {profileId} not found")))?;

    let game_dir = std::path::PathBuf::from(&row.game_dir);
    let mods_dir = game_dir.join("mods");
    let mut mods = Vec::new();
    if mods_dir.is_dir() {
        if let Ok(entries) = std::fs::read_dir(&mods_dir) {
            for entry in entries.flatten() {
                let name = entry.file_name().to_string_lossy().to_string();
                if name.ends_with(".jar") || name.ends_with(".jar.disabled") {
                    mods.push(name);
                }
            }
        }
    }

    let manifest = InstanceShareManifest {
        name: row.name,
        mc_version: row.mc_version,
        loader: row.loader,
        loader_version: row.loader_version,
        ram_mb: row.ram_mb,
        jvm_args: row.jvm_args,
        mods,
    };

    let manifest_json = serde_json::to_string(&manifest)
        .map_err(|e| AppError::Internal(format!("serialize manifest: {e}")))?;

    let num: u32 = rand::random::<u32>() & 0xFFFFFF;
    let short_code = format!("LUX-{:06X}", num);

    use flate2::write::GzEncoder;
    use flate2::Compression;
    use std::io::Write;
    use base64::Engine;

    let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
    let _ = encoder.write_all(manifest_json.as_bytes());
    let compressed = encoder.finish().unwrap_or_else(|_| manifest_json.as_bytes().to_vec());
    let b64_code = format!("LUX-B64:{}", base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(&compressed));

    let now = chrono::Utc::now().to_rfc3339();
    let _ = sqlx::query(
        "INSERT INTO share_codes (code, data, created_at) VALUES (?, ?, ?)
         ON CONFLICT(code) DO UPDATE SET data = excluded.data, created_at = excluded.created_at",
    )
    .bind(&short_code)
    .bind(&manifest_json)
    .bind(&now)
    .execute(db.pool())
    .await;

    let _ = sqlx::query(
        "INSERT INTO share_codes (code, data, created_at) VALUES (?, ?, ?)
         ON CONFLICT(code) DO UPDATE SET data = excluded.data, created_at = excluded.created_at",
    )
    .bind(&b64_code)
    .bind(&manifest_json)
    .bind(&now)
    .execute(db.pool())
    .await;

    if let Some(base_dir) = directories::ProjectDirs::from("io", "github", "Luxmc") {
        let codes_dir = base_dir.data_dir().join("share_codes");
        let _ = std::fs::create_dir_all(&codes_dir);
        let _ = std::fs::write(codes_dir.join(format!("{}.json", short_code)), &manifest_json);
    }

    Ok(b64_code)
}

#[tauri::command]
pub async fn instance_import_share_code(
    shareCode: String,
) -> AppResult<crate::db::models::ProfileRow> {
    let raw_trimmed = shareCode.trim();
    if raw_trimmed.is_empty() {
        return Err(AppError::InvalidInput("Código de compartilhamento não pode ser vazio.".into()));
    }

    let db = db::shared_db().await?;

    let json_data: String = if let Some(encoded) = raw_trimmed.strip_prefix("LUX-B64:").or_else(|| raw_trimmed.strip_prefix("lux-b64:")) {
        use base64::Engine;
        use flate2::read::GzDecoder;
        use std::io::Read;

        let bytes = base64::engine::general_purpose::URL_SAFE_NO_PAD
            .decode(encoded)
            .or_else(|_| base64::engine::general_purpose::URL_SAFE.decode(encoded))
            .or_else(|_| base64::engine::general_purpose::STANDARD.decode(encoded))
            .map_err(|_| AppError::InvalidInput("Código de compartilhamento Base64 inválido.".into()))?;

        let mut decoder = GzDecoder::new(&bytes[..]);
        let mut decompressed = String::new();
        if decoder.read_to_string(&mut decompressed).is_ok() && !decompressed.is_empty() {
            decompressed
        } else if let Ok(s) = String::from_utf8(bytes) {
            s
        } else {
            return Err(AppError::InvalidInput("Dados do código corrompidos.".into()));
        }
    } else {
        let clean_code = raw_trimmed.to_uppercase();
        if let Some(row) = sqlx::query_as::<_, (String,)>("SELECT data FROM share_codes WHERE code = ?")
            .bind(&clean_code)
            .fetch_optional(db.pool())
            .await?
        {
            row.0
        } else if let Some(base_dir) = directories::ProjectDirs::from("io", "github", "Luxmc") {
            let file_path = base_dir.data_dir().join("share_codes").join(format!("{}.json", clean_code));
            if file_path.exists() {
                std::fs::read_to_string(&file_path)
                    .map_err(|e| AppError::Internal(format!("ler arquivo de código: {e}")))?
            } else {
                return Err(AppError::NotFound(format!("Código de compartilhamento '{clean_code}' não encontrado.")));
            }
        } else {
            return Err(AppError::NotFound(format!("Código de compartilhamento '{clean_code}' não encontrado.")));
        }
    };

    let manifest: InstanceShareManifest = serde_json::from_str(&json_data)
        .map_err(|e| AppError::InvalidInput(format!("Estrutura do código inválida: {e}")))?;

    let input = crate::commands::profiles::ProfileCreate {
        name: format!("{} (Importado)", manifest.name),
        icon: None,
        mc_version: manifest.mc_version,
        loader: manifest.loader,
        loader_version: manifest.loader_version,
        java_path: None,
        jvm_args: manifest.jvm_args,
        resolution_w: None,
        resolution_h: None,
        fullscreen: None,
        game_dir: None,
        favorite: Some(false),
        notes: Some(format!("Importado via código {}", raw_trimmed)),
        ram_mb: manifest.ram_mb,
        instance_group: Some("Importados".into()),
        auto_optimize: Some(true),
        use_vulkan: Some(false),
    };

    let new_profile = crate::commands::profiles::profiles_create(input).await?;
    let game_dir = std::path::PathBuf::from(&new_profile.game_dir);
    let mods_dir = game_dir.join("mods");
    let _ = std::fs::create_dir_all(&mods_dir);

    Ok(new_profile)
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
    profileId: String,
    zipPath: String,
) -> AppResult<String> {
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

    let mut extracted_size = 0u64;
    for i in 0..archive.len() {
        let mut entry = archive.by_index(i)?;
        let raw_name = entry.name().replace('\\', "/");
        extracted_size = extracted_size.saturating_add(entry.size());
        if entry.size() > 1024 * 1024 * 1024 || extracted_size > 8 * 1024 * 1024 * 1024 || entry.unix_mode().is_some_and(|mode| mode & 0o170000 == 0o120000) {
            return Err(AppError::InvalidInput("Arquivo de backup inseguro ou grande demais".into()));
        }
        let stripped = raw_name
            .strip_prefix("saves/")
            .or_else(|| Some(raw_name.as_str()))
            .unwrap_or(raw_name.as_str());
        if stripped.is_empty() {
            continue;
        }
        let out_path = crate::core::mods::pack_download::destination(&saves_dir, stripped)?;
        if entry.is_dir() {
            std::fs::create_dir_all(&out_path)?;
        } else {
            if let Some(parent) = out_path.parent() {
                std::fs::create_dir_all(parent)?;
            }
            let mut output = std::fs::File::create(&out_path)?;
            std::io::copy(&mut entry, &mut output)?;
        }
    }

    Ok(saves_dir.to_string_lossy().to_string())
}

pub async fn instance_repair_core(
    app: Option<tauri::AppHandle>,
    http: &reqwest::Client,
    profile_id: String,
) -> AppResult<()> {
    let conn = db::shared_db().await?;
    let row: crate::db::models::ProfileRow = sqlx::query_as("SELECT * FROM profiles WHERE id = ?")
        .bind(&profile_id)
        .fetch_optional(conn.pool())
        .await?
        .ok_or_else(|| AppError::NotFound(format!("profile {profile_id} not found")))?;

    use crate::core::minecraft;
    let version_row = crate::db::schema::versions::get(&conn, &row.mc_version)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("version {} not found", row.mc_version)))?;
    let detail = minecraft::fetch_version_detail(http, &version_row.url).await?;

    let base_dir = directories::ProjectDirs::from("io", "github", "Luxmc")
        .ok_or_else(|| AppError::InvalidState("could not determine data dir".into()))?;
    let data_dir = base_dir.data_dir().to_path_buf();
    let mut downloader = crate::core::downloader::DownloadManager::new(http.clone(), data_dir);
    if let Some(a) = app {
        downloader = downloader.with_app(a);
    }
    downloader.download_version(&detail).await?;
    downloader.validate_version(&detail).await?;
    Ok(())
}

#[tauri::command]
pub async fn instance_repair(
    state: State<'_, AppState>,
    app: tauri::AppHandle,
    profileId: String,
) -> AppResult<()> {
    instance_repair_core(Some(app), &state.http, profileId).await
}

#[tauri::command]
pub async fn instance_disk_usage(profileId: String) -> AppResult<i64> {
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

pub async fn version_repair_core(
    app: Option<tauri::AppHandle>,
    http: &reqwest::Client,
    version_id: String,
) -> AppResult<()> {
    let conn = db::shared_db().await?;
    let version_row = crate::db::schema::versions::get(&conn, &version_id)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("version {version_id} not found")))?;

    use crate::core::minecraft;
    let detail = minecraft::fetch_version_detail(http, &version_row.url).await?;
    let base_dir = directories::ProjectDirs::from("io", "github", "Luxmc")
        .ok_or_else(|| AppError::InvalidState("could not determine data dir".into()))?;
    let data_dir = base_dir.data_dir().to_path_buf();
    let mut downloader = crate::core::downloader::DownloadManager::new(http.clone(), data_dir);
    if let Some(a) = app {
        downloader = downloader.with_app(a);
    }
    downloader.download_version(&detail).await?;
    downloader.validate_version(&detail).await?;
    Ok(())
}

#[tauri::command]
pub async fn version_repair(
    state: State<'_, AppState>,
    app: tauri::AppHandle,
    versionId: String,
) -> AppResult<()> {
    version_repair_core(Some(app), &state.http, versionId).await
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MclogsResponse {
    pub success: bool,
    pub id: Option<String>,
    pub url: Option<String>,
    pub error: Option<String>,
}

pub async fn share_log_mclogs_core(
    http: &reqwest::Client,
    content: String,
) -> AppResult<String> {
    if content.trim().is_empty() {
        return Err(AppError::InvalidInput("Log content cannot be empty".into()));
    }
    let form = [("content", content.as_str())];
    let resp = http
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

#[tauri::command]
pub async fn share_log_mclogs(
    state: State<'_, AppState>,
    content: String,
) -> AppResult<String> {
    share_log_mclogs_core(&state.http, content).await
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
        if cfg!(not(target_os = "windows")) {
            assert!(r
                .rejected
                .iter()
                .any(|x| x.starts_with("-XX:HeapDumpPath=")));
        }
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
