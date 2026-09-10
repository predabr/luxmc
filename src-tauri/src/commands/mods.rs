use std::collections::HashSet;

use serde::{Deserialize, Serialize};
use tauri::State;

use crate::core::mods::{curseforge, ModSearchResult, ModUpdate, ModVersion, ModrinthClient};
use crate::db::schema::mods::ModRow;
use crate::error::AppResult;
use crate::state::AppState;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ModInstallRequest {
    pub profile_id: String,
    pub project_id: String,
    pub version_id: String,
    #[serde(default = "default_source")]
    pub source: String,
}

fn default_source() -> String {
    "modrinth".into()
}

fn normalize_slug(s: &str) -> String {
    s.to_lowercase()
        .replace(['-', '_'], "")
        .trim()
        .to_string()
}

fn dedup_results(mut combined: Vec<ModSearchResult>) -> Vec<ModSearchResult> {
    let mut seen_slugs: HashSet<String> = HashSet::new();
    let mut seen_titles: HashSet<String> = HashSet::new();
    combined.retain(|r| {
        let norm_slug = normalize_slug(&r.slug);
        let norm_title = normalize_slug(&r.title);
        let is_new = seen_slugs.insert(norm_slug) && seen_titles.insert(norm_title);
        is_new
    });
    combined
}

#[tauri::command]
#[allow(non_snake_case)]
pub async fn mods_search(
    state: State<'_, AppState>,
    query: String,
    mcVersion: String,
    limit: Option<u32>,
    offset: Option<u32>,
    contentType: Option<String>,
) -> AppResult<Vec<ModSearchResult>> {
    let limit = limit.unwrap_or(24);
    let offset = offset.unwrap_or(0);
    let content_type = contentType.unwrap_or_else(|| "mod".to_string());

    let modrinth_client = ModrinthClient::new(state.http.clone());

    let (modrinth_results, curseforge_results) = tokio::join!(
        async {
            modrinth_client
                .search_mods(&query, &mcVersion, &content_type, limit, offset)
                .await
                .unwrap_or_else(|e| {
                    tracing::warn!(error = %e, "Modrinth search failed");
                    Vec::new()
                })
        },
        async {
            curseforge::search_mods(&state.http, &query, &mcVersion, &content_type, limit, offset)
                .await
                .unwrap_or_else(|e| {
                    tracing::warn!(error = %e, "CurseForge search failed");
                    Vec::new()
                })
        }
    );

    tracing::info!(
        modrinth_count = modrinth_results.len(),
        curseforge_count = curseforge_results.len(),
        query = %query,
        offset = %offset,
        content_type = %content_type,
        "mods_search completed"
    );

    let mut combined = modrinth_results;
    combined.extend(curseforge_results);
    let mut combined = dedup_results(combined);
    combined.sort_by(|a, b| b.downloads.cmp(&a.downloads));
    combined.truncate(limit as usize);
    Ok(combined)
}

#[tauri::command]
#[allow(non_snake_case)]
pub async fn mods_search_typed(
    state: State<'_, AppState>,
    query: String,
    mcVersion: String,
    contentType: String,
    limit: Option<u32>,
    offset: Option<u32>,
) -> AppResult<Vec<ModSearchResult>> {
    mods_search(state, query, mcVersion, limit, offset, Some(contentType)).await
}

#[tauri::command]
#[allow(non_snake_case)]
pub async fn mods_versions(
    state: State<'_, AppState>,
    projectId: String,
    mcVersion: String,
    source: Option<String>,
) -> AppResult<Vec<ModVersion>> {
    let src = source.as_deref().unwrap_or("modrinth");

    match src {
        "curseforge" => {
            curseforge::get_mod_versions(&state.http, &projectId, &mcVersion).await
        }
        _ => {
            let modrinth_client = ModrinthClient::new(state.http.clone());
            modrinth_client.get_mod_versions(&projectId, &mcVersion).await
        }
    }
}

#[tauri::command]
#[allow(non_snake_case)]
pub async fn mods_list(_state: State<'_, AppState>, profileId: String) -> AppResult<Vec<ModRow>> {
    let db = crate::db::shared_db().await?;
    crate::db::schema::mods::list_by_profile(&db, &profileId).await
}

#[tauri::command]
pub async fn mods_install(state: State<'_, AppState>, request: ModInstallRequest) -> AppResult<()> {
    let base_dir = directories::ProjectDirs::from("io", "github", "Luxmc").ok_or_else(|| {
        crate::error::AppError::InvalidState("could not determine data dir".into())
    })?;
    let mods_dir = base_dir.data_dir().join("mods").join(&request.profile_id);
    tokio::fs::create_dir_all(&mods_dir).await?;

    let (file_url, file_name, _file_size, file_sha1) = match request.source.as_str() {
        "curseforge" => {
            let versions = curseforge::get_mod_versions(&state.http, &request.project_id, "").await?;
            let version = versions
                .iter()
                .find(|v| v.id == request.version_id)
                .ok_or_else(|| {
                    crate::error::AppError::NotFound("CurseForge mod version not found".into())
                })?;
            let file = version.files.first().ok_or_else(|| {
                crate::error::AppError::NotFound("CurseForge mod file not found".into())
            })?;
            (
                file.url.clone(),
                file.filename.clone(),
                file.size,
                file.sha1.clone(),
            )
        }
        _ => {
            let client = ModrinthClient::new(state.http.clone());
            let versions = client.get_mod_versions(&request.project_id, "").await?;
            let version = versions
                .iter()
                .find(|v| v.id == request.version_id)
                .ok_or_else(|| {
                    crate::error::AppError::NotFound("Modrinth mod version not found".into())
                })?;
            let file = version.files.first().ok_or_else(|| {
                crate::error::AppError::NotFound("Modrinth mod file not found".into())
            })?;
            (
                file.url.clone(),
                file.filename.clone(),
                file.size,
                file.sha1.clone(),
            )
        }
    };

    tracing::info!(
        source = %request.source,
        project_id = %request.project_id,
        file = %file_name,
        "downloading mod"
    );

    let resp = state.http.get(&file_url).send().await?.error_for_status()?;
    let bytes = resp.bytes().await?;
    let file_path = mods_dir.join(&file_name);
    tokio::fs::write(&file_path, &bytes).await?;

    let db = crate::db::shared_db().await?;
    let mod_row = ModRow {
        profile_id: request.profile_id,
        project_id: request.project_id,
        version_id: request.version_id,
        file_name,
        sha1: file_sha1,
        source: request.source,
        installed_at: String::new(),
    };
    crate::db::schema::mods::upsert(&db, &mod_row).await
}

#[tauri::command]
#[allow(non_snake_case)]
pub async fn mods_install_with_deps(
    state: State<'_, AppState>,
    request: ModInstallRequest,
) -> AppResult<Vec<String>> {
    let client = ModrinthClient::new(state.http.clone());
    let mut installed = HashSet::new();
    let mut to_install: Vec<(String, String)> =
        vec![(request.project_id.clone(), request.version_id.clone())];

    let base_dir = directories::ProjectDirs::from("io", "github", "Luxmc").ok_or_else(|| {
        crate::error::AppError::InvalidState("could not determine data dir".into())
    })?;
    let mods_dir = base_dir.data_dir().join("mods").join(&request.profile_id);
    tokio::fs::create_dir_all(&mods_dir).await?;

    let db = crate::db::shared_db().await?;
    let source = request.source.clone();

    let mut depth = 0;
    while !to_install.is_empty() && depth < 3 {
        let batch = std::mem::take(&mut to_install);
        depth += 1;

        for (project_id, version_id) in batch {
            if installed.contains(&project_id) {
                continue;
            }

            let version = client
                .get_version_detail(&project_id, &version_id, "")
                .await;
            let version = match version {
                Ok(v) => v,
                Err(_) => continue,
            };

            if let Some(file) = version.files.first() {
                let resp = state.http.get(&file.url).send().await?.error_for_status()?;
                let bytes = resp.bytes().await?;
                let file_path = mods_dir.join(&file.filename);
                tokio::fs::write(&file_path, &bytes).await?;

                let mod_row = ModRow {
                    profile_id: request.profile_id.clone(),
                    project_id: project_id.clone(),
                    version_id: version.id.clone(),
                    file_name: file.filename.clone(),
                    sha1: file.sha1.clone(),
                    source: source.clone(),
                    installed_at: String::new(),
                };
                crate::db::schema::mods::upsert(&db, &mod_row).await?;
            }

            installed.insert(project_id.clone());

            for dep in &version.dependencies {
                if dep.dependency_type == "required" && !installed.contains(&dep.project_id) {
                    let dep_versions = client.get_mod_versions(&dep.project_id, "").await;
                    if let Ok(dep_versions) = dep_versions {
                        if let Some(dep_version) = dep_versions.first() {
                            to_install.push((dep.project_id.clone(), dep_version.id.clone()));
                        }
                    }
                }
            }
        }
    }

    Ok(installed.into_iter().collect())
}

#[tauri::command]
#[allow(non_snake_case)]
pub async fn mods_remove(
    _state: State<'_, AppState>,
    profileId: String,
    projectId: String,
) -> AppResult<()> {
    let db = crate::db::shared_db().await?;
    crate::db::schema::mods::delete(&db, &profileId, &projectId).await
}

#[tauri::command]
#[allow(non_snake_case)]
pub async fn mods_check_updates(
    state: State<'_, AppState>,
    profileId: String,
) -> AppResult<Vec<ModUpdate>> {
    let db = crate::db::shared_db().await?;
    let installed = crate::db::schema::mods::list_by_profile(&db, &profileId).await?;

    let profile =
        sqlx::query_as::<_, crate::db::models::ProfileRow>("SELECT * FROM profiles WHERE id = ?")
            .bind(&profileId)
            .fetch_optional(db.pool())
            .await?
            .ok_or_else(|| {
                crate::error::AppError::NotFound(format!("profile {profileId} not found"))
            })?;

    let client = ModrinthClient::new(state.http.clone());
    let loader = profile.loader.as_str();
    let loaders: Vec<&str> = match loader {
        "fabric" => vec!["fabric"],
        "forge" => vec!["forge"],
        "neoforge" => vec!["neoforge"],
        "quilt" => vec!["quilt"],
        _ => vec!["fabric", "forge", "neoforge", "quilt"],
    };

    let mut updates = Vec::new();
    for mod_row in &installed {
        if mod_row.source == "curseforge" {
            continue;
        }
        if let Ok(Some((latest_id, latest_num, download_url))) = client
            .get_latest_version(&mod_row.project_id, &profile.mc_version, &loaders)
            .await
        {
            if latest_id != mod_row.version_id {
                let project_name = client
                    .get_mod_versions(&mod_row.project_id, &profile.mc_version)
                    .await
                    .ok()
                    .and_then(|v| v.first().map(|v| v.name.clone()))
                    .unwrap_or_else(|| mod_row.project_id.clone());

                updates.push(ModUpdate {
                    project_id: mod_row.project_id.clone(),
                    project_name,
                    current_version_id: mod_row.version_id.clone(),
                    latest_version_id: latest_id,
                    latest_version_number: latest_num,
                    download_url,
                });
            }
        }
    }

    Ok(updates)
}

#[tauri::command]
#[allow(non_snake_case)]
pub async fn mods_update(
    state: State<'_, AppState>,
    projectId: String,
    versionId: String,
    profileId: String,
) -> AppResult<()> {
    let db = crate::db::shared_db().await?;

    let old = sqlx::query_as::<_, crate::db::schema::mods::ModRow>(
        "SELECT * FROM mods WHERE profile_id = ? AND project_id = ?",
    )
    .bind(&profileId)
    .bind(&projectId)
    .fetch_optional(db.pool())
    .await?;

    let source = old
        .as_ref()
        .map(|m| m.source.clone())
        .unwrap_or_else(|| "modrinth".into());

    let base_dir = directories::ProjectDirs::from("io", "github", "Luxmc").ok_or_else(|| {
        crate::error::AppError::InvalidState("could not determine data dir".into())
    })?;
    let mods_dir = base_dir.data_dir().join("mods").join(&profileId);
    tokio::fs::create_dir_all(&mods_dir).await?;

    if let Some(ref old_mod) = old {
        let old_path = mods_dir.join(&old_mod.file_name);
        let _ = tokio::fs::remove_file(&old_path).await;
    }

    let (file_url, file_name, file_sha1) = match source.as_str() {
        "curseforge" => {
            let versions = curseforge::get_mod_versions(&state.http, &projectId, "").await?;
            let version = versions
                .iter()
                .find(|v| v.id == versionId)
                .ok_or_else(|| {
                    crate::error::AppError::NotFound("CurseForge mod version not found".into())
                })?;
            let file = version.files.first().ok_or_else(|| {
                crate::error::AppError::NotFound("CurseForge mod file not found".into())
            })?;
            (file.url.clone(), file.filename.clone(), file.sha1.clone())
        }
        _ => {
            let client = ModrinthClient::new(state.http.clone());
            let versions = client.get_mod_versions(&projectId, "").await?;
            let version = versions
                .iter()
                .find(|v| v.id == versionId)
                .ok_or_else(|| {
                    crate::error::AppError::NotFound("Modrinth mod version not found".into())
                })?;
            let file = version.files.first().ok_or_else(|| {
                crate::error::AppError::NotFound("Modrinth mod file not found".into())
            })?;
            (file.url.clone(), file.filename.clone(), file.sha1.clone())
        }
    };

    let resp = state.http.get(&file_url).send().await?.error_for_status()?;
    let bytes = resp.bytes().await?;
    let file_path = mods_dir.join(&file_name);
    tokio::fs::write(&file_path, &bytes).await?;

    let mod_row = crate::db::schema::mods::ModRow {
        profile_id: profileId,
        project_id: projectId,
        version_id: versionId,
        file_name,
        sha1: file_sha1,
        source,
        installed_at: String::new(),
    };
    crate::db::schema::mods::upsert(&db, &mod_row).await
}
