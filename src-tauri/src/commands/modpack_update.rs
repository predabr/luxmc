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
            "https://api.modrinth.com/v2/project/{}/version?game_versions=[\"{}\"]&loaders=[\"{}\"]&limit=1",
            pid, row.mc_version, row.loader.to_lowercase()
        );
        if let Ok(resp) = state.http.get(&ver_url)
            .header("User-Agent", "Luxmc/1.6.5")
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
    _row: &crate::db::models::ProfileRow,
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
    use futures_util::StreamExt;

    let db = crate::db::shared_db().await?;
    let row = sqlx::query_as::<_, crate::db::models::ProfileRow>(
        "SELECT * FROM profiles WHERE id = ?",
    )
    .bind(&profileId)
    .fetch_optional(db.pool())
    .await?
    .ok_or_else(|| AppError::NotFound(format!("profile {profileId} not found")))?;

    let game_dir = std::path::PathBuf::from(&row.game_dir);

    let preserved = ["saves", "screenshots", "options.txt", "servers.dat", "resourcepacks", "shaderpacks"];
    let backup_dir = game_dir.parent()
        .unwrap_or(&game_dir)
        .join(format!(".update_backup_{}", profileId));
    let _ = tokio::fs::create_dir_all(&backup_dir).await;

    let _ = app.emit("modpack-progress", serde_json::json!({
        "phase": "backup",
        "percent": 5,
        "status": "Fazendo backup dos seus dados (saves, prints, opções)..."
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
        "percent": 15,
        "status": "Baixando nova versão do modpack..."
    }));

    if source == "modrinth" {
        let url = format!("https://api.modrinth.com/v2/version/{}", versionId);
        let resp = state.http.get(&url)
            .header("User-Agent", "Luxmc/1.6.5")
            .send().await
            .map_err(|e| AppError::Internal(format!("Erro ao buscar versão do modpack: {e}")))?;

        let ver_json: serde_json::Value = resp.json().await
            .map_err(|e| AppError::InvalidState(format!("JSON inválido: {e}")))?;

        let mrpack_dl_url = ver_json.get("files")
            .and_then(|f| f.as_array())
            .and_then(|arr| {
                arr.iter().find(|f| f.get("primary").and_then(|p| p.as_bool()).unwrap_or(false))
                    .or_else(|| arr.first())
            })
            .and_then(|f| f.get("url"))
            .and_then(|u| u.as_str())
            .ok_or_else(|| AppError::NotFound("URL do arquivo .mrpack não encontrada".into()))?;

        let mrpack_resp = state.http.get(mrpack_dl_url)
            .header("User-Agent", "Luxmc/1.6.5")
            .send().await
            .map_err(|e| AppError::Internal(format!("Erro ao baixar arquivo .mrpack: {e}")))?;

        let bytes = mrpack_resp.bytes().await
            .map_err(|e| AppError::Internal(format!("Erro ao ler bytes do .mrpack: {e}")))?;

        let mrpack_path = backup_dir.join("update.mrpack");
        tokio::fs::write(&mrpack_path, &bytes).await?;

        let file = std::fs::File::open(&mrpack_path)?;
        let mut archive = zip::ZipArchive::new(std::io::BufReader::new(file))
            .map_err(|e| AppError::InvalidState(format!("Arquivo mrpack inválido: {e}")))?;

        let mut index_opt: Option<serde_json::Value> = None;
        if let Ok(entry) = archive.by_name("modrinth.index.json") {
            if let Ok(idx) = serde_json::from_reader(entry) {
                index_opt = Some(idx);
            }
        }

        let index = index_opt.ok_or_else(|| AppError::NotFound("modrinth.index.json ausente no .mrpack".into()))?;
        let _ = tokio::fs::write(game_dir.join("modrinth.index.json"), serde_json::to_vec_pretty(&index).unwrap_or_default()).await;

        let mods_dir = game_dir.join("mods");
        let _ = tokio::fs::remove_dir_all(&mods_dir).await;
        let _ = tokio::fs::create_dir_all(&mods_dir).await;

        for i in 0..archive.len() {
            if let Ok(mut file) = archive.by_index(i) {
                let raw_name = file.name().replace('\\', "/");
                let clean = raw_name.trim_start_matches('/');
                let rel = if clean.starts_with("overrides/") {
                    Some(&clean["overrides/".len()..])
                } else if clean.starts_with("client-overrides/") {
                    Some(&clean["client-overrides/".len()..])
                } else {
                    None
                };

                if let Some(sub) = rel {
                    if !sub.is_empty() && !sub.contains("..") {
                        let out_path = game_dir.join(sub);
                        if file.is_dir() || clean.ends_with('/') {
                            let _ = std::fs::create_dir_all(&out_path);
                        } else {
                            if let Some(parent) = out_path.parent() {
                                let _ = std::fs::create_dir_all(parent);
                            }
                            if let Ok(mut out) = std::fs::File::create(&out_path) {
                                let _ = std::io::copy(&mut file, &mut out);
                            }
                        }
                    }
                }
            }
        }

        if let Some(files) = index.get("files").and_then(|f| f.as_array()) {
            let total = files.len() as u32;
            let count = std::sync::Arc::new(std::sync::atomic::AtomicU32::new(0));

            let download_stream = futures_util::stream::iter(files.clone()).map(|file_meta| {
                let http = state.http.clone();
                let g_dir = game_dir.clone();
                let app = app.clone();
                let c = count.clone();
                async move {
                    let path_str = file_meta.get("path").and_then(|p| p.as_str()).unwrap_or("");
                    let downloads = file_meta.get("downloads").and_then(|d| d.as_array());
                    if !path_str.is_empty() {
                        let target = g_dir.join(path_str);
                        if let Some(p) = target.parent() { let _ = tokio::fs::create_dir_all(p).await; }
                        if let Some(dl_arr) = downloads {
                            for dl_val in dl_arr {
                                if let Some(dl_url) = dl_val.as_str() {
                                    if let Ok(resp) = http.get(dl_url).header("User-Agent", "Luxmc/1.6.5").send().await {
                                        if resp.status().is_success() {
                                            if let Ok(b) = resp.bytes().await {
                                                let _ = tokio::fs::write(&target, &b).await;
                                                break;
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                    let cur = c.fetch_add(1, std::sync::atomic::Ordering::SeqCst) + 1;
                    let pct = 20 + ((cur as f64 / total.max(1) as f64) * 60.0) as u32;
                    let _ = app.emit("modpack-progress", serde_json::json!({
                        "phase": "downloading",
                        "current": cur,
                        "total": total,
                        "percent": pct,
                        "status": format!("Atualizando mods ({}/{})...", cur, total)
                    }));
                }
            });
            download_stream.buffer_unordered(8).collect::<Vec<()>>().await;
        }
    } else {
        let key = crate::core::mods::curseforge::api_key();
        let file_dl_url = if let Some(ref k) = key {
            let url = format!("https://api.curseforge.com/v1/mods/{}/files/{}/download-url", projectId, versionId);
            if let Ok(resp) = state.http.get(&url).header("x-api-key", k).send().await {
                if resp.status().is_success() {
                    resp.json::<serde_json::Value>().await.ok()
                        .and_then(|v| v.get("data").and_then(|d| d.as_str()).map(|s| s.to_string()))
                } else {
                    None
                }
            } else {
                None
            }
        } else {
            None
        };

        let file_dl_url = file_dl_url.unwrap_or_else(|| {
            format!("https://www.curseforge.com/api/v1/mods/{}/files/{}/download", projectId, versionId)
        });

        let mut zip_bytes = Vec::new();
        if let Ok(resp) = state.http.get(&file_dl_url).header("User-Agent", "Mozilla/5.0").send().await {
            if resp.status().is_success() {
                if let Ok(b) = resp.bytes().await {
                    zip_bytes = b.to_vec();
                }
            }
        }

        if zip_bytes.len() < 200 {
            if let Ok(v_id) = versionId.parse::<u64>() {
                let p1 = v_id / 1000;
                let p2 = v_id % 1000;
                for host in ["mediafilez.forgecdn.net", "edge.forgecdn.net"] {
                    let u = format!("https://{}/files/{}/{}/modpack.zip", host, p1, p2);
                    if let Ok(resp) = state.http.get(&u).send().await {
                        if resp.status().is_success() {
                            if let Ok(b) = resp.bytes().await {
                                if b.len() > 200 {
                                    zip_bytes = b.to_vec();
                                    break;
                                }
                            }
                        }
                    }
                }
            }
        }

        if zip_bytes.len() > 200 {
            let pack_path = backup_dir.join("update.zip");
            tokio::fs::write(&pack_path, &zip_bytes).await?;

            let file = std::fs::File::open(&pack_path)?;
            let mut archive = zip::ZipArchive::new(std::io::BufReader::new(file))
                .map_err(|e| AppError::InvalidState(format!("Zip inválido: {e}")))?;

            let mut manifest_idx = None;
            for i in 0..archive.len() {
                if let Ok(f) = archive.by_index(i) {
                    let n = f.name().replace('\\', "/");
                    if n.trim_start_matches('/') == "manifest.json" {
                        manifest_idx = Some(i);
                        break;
                    }
                }
            }

            if let Some(idx) = manifest_idx {
                let manifest: crate::commands::instances::CfManifest = {
                    let entry = archive.by_index(idx)
                        .map_err(|e| AppError::InvalidState(format!("Erro ao ler manifest: {e}")))?;
                    serde_json::from_reader(entry)
                        .map_err(|e| AppError::InvalidState(format!("Manifest corrompido: {e}")))?
                };

                let _ = tokio::fs::write(game_dir.join("manifest.json"), serde_json::to_vec_pretty(&manifest).unwrap_or_default()).await;

                let mods_dir = game_dir.join("mods");
                let _ = tokio::fs::remove_dir_all(&mods_dir).await;
                let _ = tokio::fs::create_dir_all(&mods_dir).await;

                let storage_mods_dir = directories::ProjectDirs::from("io", "github", "Luxmc")
                    .map(|d| d.data_dir().join("mods").join(&profileId))
                    .unwrap_or_else(|| mods_dir.clone());
                let _ = tokio::fs::create_dir_all(&storage_mods_dir).await;

                for i in 0..archive.len() {
                    if let Ok(mut file) = archive.by_index(i) {
                        let raw_name = file.name().replace('\\', "/");
                        let clean = raw_name.trim_start_matches('/');
                        let rel = if clean.starts_with("overrides/") {
                            Some(&clean["overrides/".len()..])
                        } else if clean.starts_with("client-overrides/") {
                            Some(&clean["client-overrides/".len()..])
                        } else {
                            None
                        };

                        if let Some(sub) = rel {
                            if !sub.is_empty() && !sub.contains("..") {
                                let out_path = game_dir.join(sub);
                                if file.is_dir() || clean.ends_with('/') {
                                    let _ = std::fs::create_dir_all(&out_path);
                                } else {
                                    if let Some(parent) = out_path.parent() {
                                        let _ = std::fs::create_dir_all(parent);
                                    }
                                    if let Ok(mut out) = std::fs::File::create(&out_path) {
                                        let _ = std::io::copy(&mut file, &mut out);
                                    }
                                }
                            }
                        }
                    }
                }

                let file_ids: Vec<u64> = manifest.files.iter().map(|f| f.file_id).collect();
                let project_ids: Vec<u64> = manifest.files.iter().map(|f| f.project_id).collect();
                let file_infos = crate::core::mods::curseforge::get_files_batch(&state.http, &file_ids).await;
                let mod_names = crate::core::mods::curseforge::get_mod_names_batch(&state.http, &project_ids).await;

                let total = manifest.files.len() as u32;
                let count = std::sync::Arc::new(std::sync::atomic::AtomicU32::new(0));

                let cf_stream = futures_util::stream::iter(manifest.files.clone()).map(|cf_file| {
                    let http = state.http.clone();
                    let file_info = file_infos.get(&cf_file.file_id).cloned();
                    let display_name = mod_names.get(&cf_file.project_id).cloned();
                    let m_dir = mods_dir.clone();
                    let s_dir = storage_mods_dir.clone();
                    let p_id = profileId.clone();
                    let mc_ver = row.mc_version.clone();
                    let ld = row.loader.clone();
                    let c = count.clone();
                    let app = app.clone();

                    async move {
                        crate::commands::instances::download_cf_mod_file(
                            &http,
                            &cf_file,
                            file_info.as_ref(),
                            display_name.as_deref(),
                            &m_dir,
                            &s_dir,
                            &p_id,
                            Some(&mc_ver),
                            Some(&ld),
                        ).await;

                        let cur = c.fetch_add(1, std::sync::atomic::Ordering::SeqCst) + 1;
                        let pct = 20 + ((cur as f64 / total.max(1) as f64) * 60.0) as u32;
                        let _ = app.emit("modpack-progress", serde_json::json!({
                            "phase": "downloading",
                            "current": cur,
                            "total": total,
                            "percent": pct,
                            "status": format!("Atualizando mods ({}/{})...", cur, total)
                        }));
                    }
                });
                cf_stream.buffer_unordered(8).collect::<Vec<()>>().await;
            }
        }
    }

    let _ = app.emit("modpack-progress", serde_json::json!({
        "phase": "restoring",
        "percent": 85,
        "status": "Restaurando seus dados pessoais (saves, prints, opções)..."
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

    let _ = tokio::fs::remove_dir_all(&backup_dir).await;

    let _ = sqlx::query("UPDATE profiles SET updated_at = ? WHERE id = ?")
        .bind(chrono::Utc::now())
        .bind(&profileId)
        .execute(db.pool())
        .await;

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
