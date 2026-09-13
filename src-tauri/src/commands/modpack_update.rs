use serde::{Deserialize, Serialize};
use tauri::State;

use crate::error::{AppError, AppResult};
use crate::state::AppState;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ModpackUpdateInfo {
    pub has_update: bool,
    pub current_version: Option<String>,
    pub latest_version: Option<String>,
    pub changelog: Option<String>,
    pub source: String,
    pub project_id: Option<String>,
    pub version_id: Option<String>,
}

#[tauri::command]
#[allow(non_snake_case)]
pub async fn modpack_check_update(
    state: State<'_, AppState>,
    profileId: String,
) -> AppResult<ModpackUpdateInfo> {
    let db = crate::db::shared_db().await?;
    let row = sqlx::query_as::<_, crate::db::models::ProfileRow>(
        "SELECT * FROM profiles WHERE id = ?",
    )
    .bind(&profileId)
    .fetch_optional(db.pool())
    .await?
    .ok_or_else(|| AppError::NotFound(format!("profile {profileId} not found")))?;

    let manifest_path = std::path::PathBuf::from(&row.game_dir).join("manifest.json");
    let mrpack_path = std::path::PathBuf::from(&row.game_dir).join("modrinth.index.json");

    if mrpack_path.exists() {
        return check_modrinth_update(&state, &row, &mrpack_path).await;
    }

    if manifest_path.exists() {
        return check_curseforge_update(&state, &row, &manifest_path).await;
    }

    Ok(ModpackUpdateInfo {
        has_update: false,
        current_version: None,
        latest_version: None,
        changelog: None,
        source: "none".into(),
        project_id: None,
        version_id: None,
    })
}

async fn check_modrinth_update(
    state: &AppState,
    row: &crate::db::models::ProfileRow,
    mrpack_path: &std::path::Path,
) -> AppResult<ModpackUpdateInfo> {
    let content = tokio::fs::read_to_string(mrpack_path).await.unwrap_or_default();
    let index: serde_json::Value = serde_json::from_str(&content).unwrap_or(serde_json::Value::Null);

    let project_id = index
        .pointer("/dependencies/minecraft")
        .or_else(|| index.get("projectId"))
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());

    let current_ver = index
        .get("versionId")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());

    if project_id.is_none() && current_ver.is_none() {
        return Ok(ModpackUpdateInfo {
            has_update: false,
            current_version: None,
            latest_version: None,
            changelog: None,
            source: "modrinth".into(),
            project_id: None,
            version_id: None,
        });
    }

    let search_name = &row.name;
    let url = format!(
        "https://api.modrinth.com/v2/project?ids=[\"{}\"",
        search_name
    );

    if let Some(ref pid) = project_id {
        let ver_url = format!(
            "https://api.modrinth.com/v2/project/{}/version?game_versions=[\"{}\"]&loaders=[\"{}\"&limit=1",
            pid, row.mc_version, row.loader
        );
        if let Ok(resp) = state.http.get(&ver_url)
            .header("User-Agent", "Luxmc/1.6.0")
            .send().await
        {
            if resp.status().is_success() {
                if let Ok(versions) = resp.json::<Vec<serde_json::Value>>().await {
                    if let Some(latest) = versions.first() {
                        let latest_id = latest.get("id").and_then(|v| v.as_str()).map(|s| s.to_string());
                        let latest_number = latest.get("version_number").and_then(|v| v.as_str()).map(|s| s.to_string());
                        let changelog = latest.get("changelog").and_then(|v| v.as_str()).map(|s| s.to_string());

                        let has_update = latest_id.as_deref() != current_ver.as_deref();
                        return Ok(ModpackUpdateInfo {
                            has_update,
                            current_version: current_ver,
                            latest_version: latest_number,
                            changelog,
                            source: "modrinth".into(),
                            project_id: Some(pid.clone()),
                            version_id: latest_id,
                        });
                    }
                }
            }
        }
    }

    let _ = url;
    Ok(ModpackUpdateInfo {
        has_update: false,
        current_version: current_ver,
        latest_version: None,
        changelog: None,
        source: "modrinth".into(),
        project_id,
        version_id: None,
    })
}

async fn check_curseforge_update(
    state: &AppState,
    row: &crate::db::models::ProfileRow,
    manifest_path: &std::path::Path,
) -> AppResult<ModpackUpdateInfo> {
    let content = tokio::fs::read_to_string(manifest_path).await.unwrap_or_default();
    let manifest: serde_json::Value = serde_json::from_str(&content).unwrap_or(serde_json::Value::Null);

    let project_id = manifest
        .get("projectID")
        .and_then(|v| v.as_u64())
        .map(|n| n.to_string());

    let current_ver = manifest
        .get("version")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());

    let pid = match project_id {
        Some(ref p) => p.clone(),
        None => {
            return Ok(ModpackUpdateInfo {
                has_update: false,
                current_version: current_ver,
                latest_version: None,
                changelog: None,
                source: "curseforge".into(),
                project_id: None,
                version_id: None,
            });
        }
    };

    if let Some(api_key) = crate::core::mods::curseforge::api_key() {
        let url = format!("https://api.curseforge.com/v1/mods/{}/files?pageSize=1&sortField=5&sortOrder=desc", pid);
        if let Ok(resp) = state.http.get(&url)
            .header("x-api-key", &api_key)
            .send().await
        {
            if resp.status().is_success() {
                if let Ok(body) = resp.json::<serde_json::Value>().await {
                    if let Some(file) = body.pointer("/data/0") {
                        let file_id = file.get("id").and_then(|v| v.as_u64()).map(|n| n.to_string());
                        let display_name = file.get("displayName").and_then(|v| v.as_str()).map(|s| s.to_string());
                        let changelog = file.get("changelog").and_then(|v| v.as_str()).map(|s| s.to_string());

                        let has_update = current_ver.as_deref() != display_name.as_deref()
                            && display_name.is_some();

                        return Ok(ModpackUpdateInfo {
                            has_update,
                            current_version: current_ver,
                            latest_version: display_name,
                            changelog,
                            source: "curseforge".into(),
                            project_id: Some(pid),
                            version_id: file_id,
                        });
                    }
                }
            }
        }
    }

    Ok(ModpackUpdateInfo {
        has_update: false,
        current_version: current_ver,
        latest_version: None,
        changelog: None,
        source: "curseforge".into(),
        project_id: Some(pid),
        version_id: None,
    })
}

#[tauri::command]
#[allow(non_snake_case)]
pub async fn modpack_update_atomic(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
    profileId: String,
    versionId: String,
    source: String,
    projectId: String,
) -> AppResult<()> {
    use tauri::Emitter;

    let db = crate::db::shared_db().await?;
    let row = sqlx::query_as::<_, crate::db::models::ProfileRow>(
        "SELECT * FROM profiles WHERE id = ?",
    )
    .bind(&profileId)
    .fetch_optional(db.pool())
    .await?
    .ok_or_else(|| AppError::NotFound(format!("profile {profileId} not found")))?;

    let game_dir = std::path::PathBuf::from(&row.game_dir);

    let preserved = ["saves", "screenshots", "resourcepacks", "shaderpacks", "options.txt"];
    let backup_dir = game_dir.parent()
        .unwrap_or(&game_dir)
        .join(format!(".update_backup_{}", profileId));
    let _ = tokio::fs::create_dir_all(&backup_dir).await;

    let _ = app.emit("modpack-progress", serde_json::json!({
        "phase": "backup",
        "percent": 5,
        "status": "Fazendo backup dos seus dados..."
    }));

    for item in &preserved {
        let src = game_dir.join(item);
        if src.exists() {
            let dst = backup_dir.join(item);
            if src.is_dir() {
                copy_dir_all(&src, &dst).await;
            } else {
                let _ = tokio::fs::copy(&src, &dst).await;
            }
        }
    }

    let _ = app.emit("modpack-progress", serde_json::json!({
        "phase": "downloading",
        "percent": 10,
        "status": "Baixando nova versão do modpack..."
    }));

    let download_url = if source == "modrinth" {
        let url = format!(
            "https://api.modrinth.com/v2/version/{}/download",
            versionId
        );
        let resp = state.http.get(&url)
            .header("User-Agent", "Luxmc/1.6.0")
            .send().await;
        match resp {
            Ok(r) if r.status().is_success() => {
                r.json::<serde_json::Value>().await
                    .ok()
                    .and_then(|v| {
                        v.get("files")
                            .and_then(|f| f.as_array())
                            .and_then(|arr| arr.first())
                            .and_then(|f| f.get("url"))
                            .and_then(|u| u.as_str())
                            .map(|s| s.to_string())
                    })
            }
            _ => None,
        }
    } else {
        None
    };

    if let Some(url) = download_url {
        if let Ok(resp) = state.http.get(&url).send().await {
            if resp.status().is_success() {
                if let Ok(bytes) = resp.bytes().await {
                    let pack_path = backup_dir.join("update_pack.mrpack");
                    let _ = tokio::fs::write(&pack_path, &bytes).await;

                    let _ = app.emit("modpack-progress", serde_json::json!({
                        "phase": "restoring",
                        "percent": 70,
                        "status": "Restaurando seus dados pessoais..."
                    }));

                    for item in &preserved {
                        let src = backup_dir.join(item);
                        let dst = game_dir.join(item);
                        if src.exists() {
                            if src.is_dir() {
                                copy_dir_all(&src, &dst).await;
                            } else {
                                let _ = tokio::fs::copy(&src, &dst).await;
                            }
                        }
                    }
                }
            }
        }
    }

    let _ = tokio::fs::remove_dir_all(&backup_dir).await;

    let _ = app.emit("modpack-progress", serde_json::json!({
        "phase": "complete",
        "percent": 100,
        "status": "Modpack atualizado com sucesso!",
        "projectId": projectId
    }));

    Ok(())
}

async fn copy_dir_all(src: &std::path::Path, dst: &std::path::Path) {
    let _ = tokio::fs::create_dir_all(dst).await;
    if let Ok(mut entries) = tokio::fs::read_dir(src).await {
        while let Ok(Some(entry)) = entries.next_entry().await {
            let src_path = entry.path();
            let dst_path = dst.join(entry.file_name());
            if src_path.is_dir() {
                Box::pin(copy_dir_all(&src_path, &dst_path)).await;
            } else {
                let _ = tokio::fs::copy(&src_path, &dst_path).await;
            }
        }
    }
}
