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
        .get("projectId")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());

    let current_ver = index
        .get("sourceVersionId").or_else(|| index.get("versionId"))
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

    if let Some(ref pid) = project_id {
        if !pid.chars().all(|c| c.is_ascii_alphanumeric() || c == '-') { return Err(AppError::InvalidInput("ID de projeto inválido".into())); }
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

                        let has_update = latest_id.as_deref() != current_ver.as_deref() && latest_number.as_deref() != current_ver.as_deref();
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

    let source_version = manifest.get("sourceVersionId").and_then(|value| value.as_str());
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

                        let has_update = if let Some(current) = source_version { Some(current) != file_id.as_deref() } else { current_ver.as_deref() != display_name.as_deref() && display_name.is_some() };

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

static UPDATE_LOCK: tokio::sync::Mutex<()> = tokio::sync::Mutex::const_new(());

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
    use crate::core::mods::pack_download as pack;
    use crate::commands::instances::{instance_import_modpack_core, instance_import_mrpack_core};
    use tauri::Emitter;
    let _guard = UPDATE_LOCK.try_lock().map_err(|_| AppError::InvalidState("Uma atualização já está em andamento".into()))?;
    if !["modrinth", "curseforge"].contains(&source.as_str()) ||
        [&versionId, &projectId].iter().any(|id| id.is_empty() || !id.chars().all(|c| c.is_ascii_alphanumeric() || c == '-')) {
        return Err(AppError::InvalidInput("Identificador de atualização inválido".into()));
    }
    let db = crate::db::shared_db().await?;
    let row = sqlx::query_as::<_, crate::db::models::ProfileRow>("SELECT * FROM profiles WHERE id = ?")
        .bind(&profileId).fetch_optional(db.pool()).await?
        .ok_or_else(|| AppError::NotFound(format!("profile {profileId} not found")))?;
    let original = std::path::PathBuf::from(&row.game_dir);
    let managed = owned_pack_paths(&original).await?;
    state.import_cancel.store(false, std::sync::atomic::Ordering::SeqCst);
    let client = pack::client()?;
    let _ = app.emit("modpack-progress", serde_json::json!({"phase":"downloading","percent":5,"status":"Preparando nova versão sem alterar a instância atual..."}));
    let (urls, size, sha1, sha512) = if source == "modrinth" {
        let version: serde_json::Value = client.get(format!("https://api.modrinth.com/v2/version/{versionId}"))
            .send().await?.error_for_status()?.json().await?;
        if version.get("project_id").and_then(|v| v.as_str()) != Some(projectId.as_str()) {
            return Err(AppError::InvalidInput("A versão não pertence ao modpack selecionado".into()));
        }
        let files = version.get("files").and_then(|v| v.as_array()).ok_or_else(|| AppError::NotFound("Arquivos da versão ausentes".into()))?;
        let file = files.iter().find(|v| v.get("primary").and_then(|v| v.as_bool()) == Some(true)).or_else(|| files.first())
            .ok_or_else(|| AppError::NotFound("Arquivo mrpack ausente".into()))?;
        let url = file.get("url").and_then(|v| v.as_str()).ok_or_else(|| AppError::NotFound("URL do modpack ausente".into()))?;
        (pack::mirrors(url)?, file.get("size").and_then(|v| v.as_u64()),
         file.pointer("/hashes/sha1").and_then(|v| v.as_str()).map(str::to_owned),
         file.pointer("/hashes/sha512").and_then(|v| v.as_str()).map(str::to_owned))
    } else {
        let file_id = versionId.parse::<u64>().map_err(|_| AppError::InvalidInput("ID CurseForge inválido".into()))?;
        let info = crate::core::mods::curseforge::get_files_batch(&client, &[file_id]).await.remove(&file_id)
            .ok_or_else(|| AppError::NotFound("Metadados CurseForge indisponíveis; verifique a chave API".into()))?;
        if info.mod_id.to_string() != projectId { return Err(AppError::InvalidInput("Projeto CurseForge incorreto".into())); }
        let mut urls = info.download_url.as_deref().map(pack::mirrors).transpose()?.unwrap_or_default();
        for host in ["edge.forgecdn.net", "mediafilez.forgecdn.net", "media.forgecdn.net"] {
            urls.push(format!("https://{host}/files/{}/{}/{}", file_id / 1000, file_id % 1000, urlencoding::encode(&info.file_name)));
        }
        (urls, info.size, info.sha1, None)
    };
    let mut archive = None;
    for attempt in 0..3 {
        if attempt > 0 { tokio::time::sleep(std::time::Duration::from_millis(500 * (1 << (attempt - 1)))).await; }
        for url in &urls {
            pack::cancelled(Some(&state.import_cancel))?;
            if let Ok(bytes) = pack::bytes(&client, url, Some(&state.import_cancel)).await {
                if pack::verify(&bytes, size, sha1.as_deref(), sha512.as_deref()) && zip::ZipArchive::new(std::io::Cursor::new(&bytes)).is_ok() {
                    archive = Some(bytes); break;
                }
            }
        }
        if archive.is_some() { break; }
    }
    let bytes = archive.ok_or_else(|| AppError::InvalidState("Não foi possível baixar e validar a atualização".into()))?;
    let archive_path = std::env::temp_dir().join(format!("luxmc-update-{}.zip", uuid::Uuid::new_v4()));
    pack::atomic_write(&archive_path, &bytes).await?;
    let imported = if source == "modrinth" {
        instance_import_mrpack_core(None, &state, archive_path.to_string_lossy().into_owned(), row.name.clone(), Some(row.icon.clone()), row.ram_mb).await
    } else {
        instance_import_modpack_core(None, &state, archive_path.to_string_lossy().into_owned(), row.name.clone(), row.mc_version.clone(), row.loader.clone(), Some(row.icon.clone()), row.ram_mb).await
    };
    let _ = tokio::fs::remove_file(&archive_path).await;
    let staged = imported?;
    let staged_dir = std::path::PathBuf::from(&staged.game_dir);
    let result: AppResult<()> = async {
        let manifest_path = staged_dir.join(if source == "modrinth" { "modrinth.index.json" } else { "manifest.json" });
        let mut manifest: serde_json::Value = serde_json::from_slice(&tokio::fs::read(&manifest_path).await?)?;
        if source == "modrinth" {
            manifest["projectId"] = serde_json::json!(projectId);
            manifest["sourceVersionId"] = serde_json::json!(versionId);
        } else {
            manifest["projectID"] = serde_json::json!(projectId.parse::<u64>().map_err(|_| AppError::InvalidInput("ID CurseForge inválido".into()))?);
            manifest["sourceVersionId"] = serde_json::json!(versionId);
        }
        pack::atomic_write(&manifest_path, &serde_json::to_vec_pretty(&manifest)?).await?;
        let old = original.clone();
        let new = staged_dir.clone();
        let owned = managed.clone();
        tokio::task::spawn_blocking(move || preserve_user_files(&old, &new, &owned)).await
            .map_err(|error| AppError::Internal(error.to_string()))??;
        pack::cancelled(Some(&state.import_cancel))?;
        let mut tx = db.pool().begin().await?;
        sqlx::query("DELETE FROM mods WHERE profile_id = ? AND project_id IN (SELECT project_id FROM mods WHERE profile_id = ?)").bind(&profileId).bind(&staged.id).execute(&mut *tx).await?;
        for path in &managed {
            if let Some(name) = path.strip_prefix("mods/") {
                sqlx::query("DELETE FROM mods WHERE profile_id = ? AND file_name = ?").bind(&profileId).bind(name).execute(&mut *tx).await?;
            }
        }
        sqlx::query("UPDATE mods SET profile_id = ? WHERE profile_id = ?").bind(&profileId).bind(&staged.id).execute(&mut *tx).await?;
        sqlx::query("UPDATE profiles SET game_dir = ?, mc_version = ?, loader = ?, loader_version = ?, mod_count = ?, updated_at = ? WHERE id = ?")
            .bind(&staged.game_dir).bind(&staged.mc_version).bind(&staged.loader).bind(&staged.loader_version)
            .bind(staged.mod_count).bind(chrono::Utc::now()).bind(&profileId).execute(&mut *tx).await?;
        sqlx::query("DELETE FROM profiles WHERE id = ?").bind(&staged.id).execute(&mut *tx).await?;
        tx.commit().await?;
        Ok(())
    }.await;
    if let Err(error) = result {
        sqlx::query("DELETE FROM profiles WHERE id = ?").bind(&staged.id).execute(db.pool()).await?;
        return Err(error);
    }
    let _ = app.emit("modpack-progress", serde_json::json!({"phase":"complete","percent":100,"status":"Modpack atualizado; a pasta anterior foi preservada como backup.","backupPath":original,"projectId":projectId}));
    Ok(())
}

async fn owned_pack_paths(root: &std::path::Path) -> AppResult<std::collections::HashSet<String>> {
    use crate::core::mods::pack_download as pack;
    let mut owned = std::collections::HashSet::new();
    for manifest in ["modrinth.index.json", ".luxmc/overrides.json"] {
        let path = root.join(manifest);
        if !path.exists() { continue; }
        let value: serde_json::Value = serde_json::from_slice(&tokio::fs::read(path).await?)?;
        let files = value.as_array().or_else(|| value.get("files").and_then(|v| v.as_array()));
        if let Some(files) = files {
            for file in files {
                if let Some(path) = file.get("path").and_then(|v| v.as_str()) {
                    pack::relative_path(path)?;
                    owned.insert(path.replace('\\', "/"));
                }
            }
        }
    }
    let cf = root.join("manifest.json");
    if cf.exists() {
        let manifest: crate::commands::instances::CfManifest = serde_json::from_slice(&tokio::fs::read(cf).await?)?;
        for file in manifest.files {
            let data = tokio::fs::read(root.join(format!(".luxmc/curseforge/{}.json", file.file_id))).await
                .map_err(|_| AppError::InvalidState("Repare o modpack antes de atualizar: faltam metadados exatos dos mods antigos".into()))?;
            let info: crate::core::mods::curseforge::CurseForgeFileInfo = serde_json::from_slice(&data)?;
            if info.id != file.file_id || info.mod_id != file.project_id || info.file_name.contains(['/', '\\']) {
                return Err(AppError::InvalidState("Metadados CurseForge inconsistentes".into()));
            }
            let path = format!("mods/{}", info.file_name);
            pack::relative_path(&path)?;
            owned.insert(path);
        }
    }
    Ok(owned)
}

fn preserve_user_files(old: &std::path::Path, new: &std::path::Path, managed: &std::collections::HashSet<String>) -> AppResult<()> {
    use crate::core::mods::pack_download as pack;
    let mut pending = vec![old.to_path_buf()];
    while let Some(dir) = pending.pop() {
        for entry in std::fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();
            let relative = path.strip_prefix(old).map_err(|error| AppError::Internal(error.to_string()))?.to_string_lossy().replace('\\', "/");
            if relative == ".luxmc" || ["manifest.json", "modrinth.index.json"].contains(&relative.as_str()) { continue; }
            if entry.file_type()?.is_symlink() { return Err(AppError::InvalidInput(format!("Não é possível copiar link simbólico: {relative}"))); }
            let target = pack::destination(new, &relative)?;
            if entry.file_type()?.is_dir() { std::fs::create_dir_all(&target)?; pending.push(path); continue; }
            let enabled = relative.strip_suffix(".disabled").unwrap_or(&relative);
            if managed.contains(enabled) {
                if relative.ends_with(".disabled") {
                    let candidate = pack::destination(new, enabled)?;
                    if candidate.is_file() { std::fs::rename(candidate, target)?; }
                }
                continue;
            }
            let first = relative.split('/').next().unwrap_or_default();
            let personal = ["saves", "screenshots", "options.txt", "servers.dat", "resourcepacks", "shaderpacks"].contains(&first);
            if personal || !target.exists() { std::fs::copy(path, target)?; }
        }
    }
    Ok(())
}

#[cfg(test)]
mod update_tests {
    use super::*;
    #[test]
    fn preserves_user_data_and_exact_mod_ownership_without_touching_original() {
        let root = std::env::temp_dir().join(format!("luxmc-update-test-{}", uuid::Uuid::new_v4()));
        let old = root.join("old"); let new = root.join("new");
        std::fs::create_dir_all(old.join("mods")).unwrap(); std::fs::create_dir_all(new.join("mods")).unwrap();
        for (path, value) in [(old.join("mods/example-1.jar"), "old"), (old.join("mods/example-extra.jar"), "user"), (old.join("options.txt"), "settings"), (new.join("mods/example-2.jar"), "new"), (new.join("options.txt"), "pack")] {
            std::fs::write(path, value).unwrap();
        }
        preserve_user_files(&old, &new, &std::collections::HashSet::from(["mods/example-1.jar".into()])).unwrap();
        assert!(!new.join("mods/example-1.jar").exists());
        assert_eq!(std::fs::read_to_string(new.join("mods/example-extra.jar")).unwrap(), "user");
        assert_eq!(std::fs::read_to_string(new.join("options.txt")).unwrap(), "settings");
        assert_eq!(std::fs::read_to_string(old.join("mods/example-1.jar")).unwrap(), "old");
        std::fs::remove_dir_all(root).unwrap();
    }
}
