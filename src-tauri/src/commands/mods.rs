use std::collections::HashSet;

use serde::{Deserialize, Serialize};
use tauri::State;

use crate::core::mods::{
    curseforge, ModProjectDetails, ModSearchResult, ModUpdate, ModVersion, ModrinthClient,
};
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
    pub content_type: Option<String>,
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
    sortBy: Option<String>,
    loader: Option<String>,
    category: Option<String>,
    source: Option<String>,
) -> AppResult<Vec<ModSearchResult>> {
    let limit = limit.unwrap_or(21);
    let offset = offset.unwrap_or(0);
    let content_type = contentType.unwrap_or_else(|| "mod".to_string());
    let sort = sortBy.as_deref().unwrap_or("downloads");

    let modrinth_client = ModrinthClient::new(state.http.clone());

    let src = source.as_deref().unwrap_or("all").to_lowercase();
    let (modrinth_results, curseforge_results) = match src.as_str() {
        "modrinth" => {
            let m = modrinth_client
                .search_mods(&query, &mcVersion, &content_type, loader.as_deref(), category.as_deref(), limit, offset, Some(sort))
                .await
                .unwrap_or_else(|e| {
                    tracing::warn!(error = %e, "Modrinth search failed");
                    Vec::new()
                });
            (m, Vec::new())
        }
        "curseforge" => {
            let c = curseforge::search_mods(&state.http, &query, &mcVersion, &content_type, loader.as_deref(), limit, offset, Some(sort))
                .await
                .unwrap_or_else(|e| {
                    tracing::warn!(error = %e, "CurseForge search failed");
                    Vec::new()
                });
            (Vec::new(), c)
        }
        _ => {
            tokio::join!(
                async {
                    modrinth_client
                        .search_mods(&query, &mcVersion, &content_type, loader.as_deref(), category.as_deref(), limit, offset, Some(sort))
                        .await
                        .unwrap_or_else(|e| {
                            tracing::warn!(error = %e, "Modrinth search failed");
                            Vec::new()
                        })
                },
                async {
                    curseforge::search_mods(&state.http, &query, &mcVersion, &content_type, loader.as_deref(), limit, offset, Some(sort))
                        .await
                        .unwrap_or_else(|e| {
                            tracing::warn!(error = %e, "CurseForge search failed");
                            Vec::new()
                        })
                }
            )
        }
    };

    tracing::info!(
        modrinth_count = modrinth_results.len(),
        curseforge_count = curseforge_results.len(),
        query = %query,
        offset = %offset,
        content_type = %content_type,
        sort = %sort,
        source = %src,
        "mods_search completed"
    );

    let mut combined = modrinth_results;
    combined.extend(curseforge_results);
    let mut combined = dedup_results(combined);
    if sort == "downloads" {
        combined.sort_by(|a, b| b.downloads.cmp(&a.downloads));
    }
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
    sortBy: Option<String>,
    loader: Option<String>,
    category: Option<String>,
    source: Option<String>,
) -> AppResult<Vec<ModSearchResult>> {
    mods_search(state, query, mcVersion, limit, offset, Some(contentType), sortBy, loader, category, source).await
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
pub async fn mods_project_details(
    state: State<'_, AppState>,
    projectId: String,
    source: Option<String>,
) -> AppResult<ModProjectDetails> {
    let src = source.as_deref().unwrap_or("modrinth");
    if src == "curseforge" {
        curseforge::get_mod_details(&state.http, &projectId).await
    } else {
        let client = ModrinthClient::new(state.http.clone());
        client.get_project_details(&projectId).await
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

    let target_subfolder = match request.content_type.as_deref().unwrap_or("mod").to_lowercase().as_str() {
        "shader" | "shaders" => "shaderpacks",
        "resourcepack" | "resource pack" | "resource_pack" => "resourcepacks",
        _ => "mods",
    };

    let target_dir = base_dir.data_dir().join(target_subfolder).join(&request.profile_id);
    tokio::fs::create_dir_all(&target_dir).await?;

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
        target_subfolder = %target_subfolder,
        file = %file_name,
        "downloading item"
    );

    let resp = state.http.get(&file_url).send().await?.error_for_status()?;
    let bytes = resp.bytes().await?;
    let file_path = target_dir.join(&file_name);
    tokio::fs::write(&file_path, &bytes).await?;

    let db = crate::db::shared_db().await?;
    let profile = if let Some(p) = sqlx::query_as::<_, crate::db::models::ProfileRow>("SELECT * FROM profiles WHERE id = ?")
        .bind(&request.profile_id)
        .fetch_optional(db.pool())
        .await? {
        Some(p)
    } else {
        sqlx::query_as::<_, crate::db::models::ProfileRow>("SELECT * FROM profiles ORDER BY last_played DESC LIMIT 1")
            .fetch_optional(db.pool())
            .await?
    };
    let resolved_profile_id = profile.as_ref().map(|p| p.id.clone()).unwrap_or_else(|| request.profile_id.clone());

    if let Some(ref prof) = profile {
        let prof_dest_dir = std::path::PathBuf::from(&prof.game_dir).join(target_subfolder);
        let _ = tokio::fs::create_dir_all(&prof_dest_dir).await;
        let _ = tokio::fs::write(prof_dest_dir.join(&file_name), &bytes).await;
    }

    let mod_row = ModRow {
        profile_id: resolved_profile_id.clone(),
        project_id: request.project_id,
        version_id: request.version_id,
        file_name: file_name.clone(),
        sha1: file_sha1,
        source: request.source,
        installed_at: String::new(),
    };
    crate::db::schema::mods::upsert(&db, &mod_row).await?;

    let count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM mods WHERE profile_id = ?")
        .bind(&resolved_profile_id)
        .fetch_one(db.pool())
        .await
        .unwrap_or((0,));
    let _ = sqlx::query("UPDATE profiles SET mod_count = ? WHERE id = ?")
        .bind(count.0)
        .bind(&resolved_profile_id)
        .execute(db.pool())
        .await;

    // Automatically promote profile loader to Fabric if installing a .jar mod onto a vanilla instance
    if target_subfolder == "mods" && file_name.ends_with(".jar") {
        if let Some(ref prof) = profile {
            if prof.loader == "vanilla" || prof.loader.is_empty() {
                tracing::info!(profile_id = %resolved_profile_id, "Promoting profile loader from vanilla to fabric");
                let _ = sqlx::query("UPDATE profiles SET loader = 'fabric' WHERE id = ?")
                    .bind(&resolved_profile_id)
                    .execute(db.pool())
                    .await;
            }
        }
    }

    Ok(())
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

    let db = crate::db::shared_db().await?;
    let profile = if let Some(p) = sqlx::query_as::<_, crate::db::models::ProfileRow>("SELECT * FROM profiles WHERE id = ?")
        .bind(&request.profile_id)
        .fetch_optional(db.pool())
        .await? {
        Some(p)
    } else {
        sqlx::query_as::<_, crate::db::models::ProfileRow>("SELECT * FROM profiles ORDER BY last_played DESC LIMIT 1")
            .fetch_optional(db.pool())
            .await?
    };
    let resolved_profile_id = profile.as_ref().map(|p| p.id.clone()).unwrap_or_else(|| request.profile_id.clone());

    let base_dir = directories::ProjectDirs::from("io", "github", "Luxmc").ok_or_else(|| {
        crate::error::AppError::InvalidState("could not determine data dir".into())
    })?;
    let mods_dir = base_dir.data_dir().join("mods").join(&resolved_profile_id);
    tokio::fs::create_dir_all(&mods_dir).await?;

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

                if let Some(ref prof) = profile {
                    let prof_mods_dir = std::path::PathBuf::from(&prof.game_dir).join("mods");
                    let _ = tokio::fs::create_dir_all(&prof_mods_dir).await;
                    let _ = tokio::fs::write(prof_mods_dir.join(&file.filename), &bytes).await;
                }

                let mod_row = ModRow {
                    profile_id: resolved_profile_id.clone(),
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

    let count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM mods WHERE profile_id = ?")
        .bind(&resolved_profile_id)
        .fetch_one(db.pool())
        .await
        .unwrap_or((0,));
    let _ = sqlx::query("UPDATE profiles SET mod_count = ? WHERE id = ?")
        .bind(count.0)
        .bind(&resolved_profile_id)
        .execute(db.pool())
        .await;

    if let Some(ref prof) = profile {
        if prof.loader == "vanilla" || prof.loader.is_empty() {
            tracing::info!(profile_id = %resolved_profile_id, "Promoting profile loader from vanilla to fabric");
            let _ = sqlx::query("UPDATE profiles SET loader = 'fabric' WHERE id = ?")
                .bind(&resolved_profile_id)
                .execute(db.pool())
                .await;
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
    let old = sqlx::query_as::<_, crate::db::schema::mods::ModRow>(
        "SELECT * FROM mods WHERE profile_id = ? AND project_id = ?",
    )
    .bind(&profileId)
    .bind(&projectId)
    .fetch_optional(db.pool())
    .await?;

    if let Some(m) = old {
        if let Some(base_dir) = directories::ProjectDirs::from("io", "github", "Luxmc") {
            let p = base_dir.data_dir().join("mods").join(&profileId).join(&m.file_name);
            let _ = tokio::fs::remove_file(p).await;
        }
        let profile = sqlx::query_as::<_, crate::db::models::ProfileRow>("SELECT * FROM profiles WHERE id = ?")
            .bind(&profileId)
            .fetch_optional(db.pool())
            .await?;
        if let Some(prof) = profile {
            let p = std::path::PathBuf::from(&prof.game_dir).join("mods").join(&m.file_name);
            let _ = tokio::fs::remove_file(p).await;
            let p_disabled = std::path::PathBuf::from(&prof.game_dir).join("mods").join(format!("{}.disabled", m.file_name));
            let _ = tokio::fs::remove_file(p_disabled).await;
        }
    }

    crate::db::schema::mods::delete(&db, &profileId, &projectId).await?;
    let count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM mods WHERE profile_id = ?")
        .bind(&profileId)
        .fetch_one(db.pool())
        .await
        .unwrap_or((0,));
    let _ = sqlx::query("UPDATE profiles SET mod_count = ? WHERE id = ?")
        .bind(count.0)
        .bind(&profileId)
        .execute(db.pool())
        .await;
    Ok(())
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
