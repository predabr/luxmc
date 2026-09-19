use std::collections::HashSet;
use futures_util::StreamExt;
use tokio::io::AsyncWriteExt;

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
pub async fn mods_search_core(
    state: &AppState,
    query: String,
    mc_version: String,
    limit: Option<u32>,
    offset: Option<u32>,
    content_type: Option<String>,
    sort_by: Option<String>,
    loader: Option<String>,
    category: Option<String>,
    source: Option<String>,
) -> AppResult<Vec<ModSearchResult>> {
    let limit = limit.unwrap_or(21);
    let offset = offset.unwrap_or(0);
    let content_type = content_type.unwrap_or_else(|| "mod".to_string());
    let sort = sort_by.as_deref().unwrap_or("downloads");

    let modrinth_client = ModrinthClient::new(state.http.clone());

    let src = source.as_deref().unwrap_or("all").to_lowercase();
    let (modrinth_results, curseforge_results) = match src.as_str() {
        "modrinth" => {
            let m = modrinth_client
                .search_mods(&query, &mc_version, &content_type, loader.as_deref(), category.as_deref(), limit, offset, Some(sort))
                .await
                .unwrap_or_else(|e| {
                    tracing::warn!(error = %e, "Modrinth search failed");
                    Vec::new()
                });
            (m, Vec::new())
        }
        "curseforge" => {
            let c = curseforge::search_mods(&state.http, &query, &mc_version, &content_type, loader.as_deref(), limit, offset, Some(sort))
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
                        .search_mods(&query, &mc_version, &content_type, loader.as_deref(), category.as_deref(), limit, offset, Some(sort))
                        .await
                        .unwrap_or_else(|e| {
                            tracing::warn!(error = %e, "Modrinth search failed");
                            Vec::new()
                        })
                },
                async {
                    curseforge::search_mods(&state.http, &query, &mc_version, &content_type, loader.as_deref(), limit, offset, Some(sort))
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
    mods_search_core(&state, query, mcVersion, limit, offset, contentType, sortBy, loader, category, source).await
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

pub async fn mods_versions_core(
    state: &AppState,
    project_id: String,
    mc_version: String,
    source: Option<String>,
) -> AppResult<Vec<ModVersion>> {
    let src = source.as_deref().unwrap_or("modrinth");
    tracing::info!(project_id = %project_id, mc_version = %mc_version, source = %src, "mods_versions called");

    let result = match src {
        "curseforge" => {
            curseforge::get_mod_versions(&state.http, &project_id, &mc_version).await
        }
        _ => {
            let modrinth_client = ModrinthClient::new(state.http.clone());
            modrinth_client.get_mod_versions(&project_id, &mc_version).await
        }
    };

    result
}

#[tauri::command]
#[allow(non_snake_case)]
pub async fn mods_versions(
    state: State<'_, AppState>,
    projectId: String,
    mcVersion: String,
    source: Option<String>,
) -> AppResult<Vec<ModVersion>> {
    mods_versions_core(&state, projectId, mcVersion, source).await
}

pub async fn mods_project_details_core(
    state: &AppState,
    project_id: String,
    source: Option<String>,
) -> AppResult<ModProjectDetails> {
    let src = source.as_deref().unwrap_or("modrinth");
    if src == "curseforge" {
        curseforge::get_mod_details(&state.http, &project_id).await
    } else {
        let client = ModrinthClient::new(state.http.clone());
        client.get_project_details(&project_id).await
    }
}

#[tauri::command]
#[allow(non_snake_case)]
pub async fn mods_project_details(
    state: State<'_, AppState>,
    projectId: String,
    source: Option<String>,
) -> AppResult<ModProjectDetails> {
    mods_project_details_core(&state, projectId, source).await
}

#[tauri::command]
#[allow(non_snake_case)]
pub async fn mods_list(profileId: String) -> AppResult<Vec<ModRow>> {
    let db = crate::db::shared_db().await?;
    crate::db::schema::mods::list_by_profile(&db, &profileId).await
}

pub async fn mods_install_core(state: &AppState, request: ModInstallRequest) -> AppResult<()> {
    tracing::info!(profile_id = %request.profile_id, project_id = %request.project_id, version_id = %request.version_id, source = %request.source, content_type = ?request.content_type, "mods_install called");
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
            if let Ok(file) = curseforge::get_mod_file_details(&state.http, &request.project_id, &request.version_id).await {
                (file.url, file.filename, file.size, file.sha1)
            } else {
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
        version_id = %request.version_id,
        target_subfolder = %target_subfolder,
        file = %file_name,
        url = %file_url,
        "downloading item"
    );

    let edge_fallback_url = if request.source == "curseforge" {
        if let Ok(id_num) = request.version_id.parse::<u64>() {
            let p1 = id_num / 1000;
            let p2 = id_num % 1000;
            Some(format!(
                "https://edge.forgecdn.net/files/{}/{}/{}",
                p1,
                p2,
                urlencoding::encode(&file_name)
            ))
        } else {
            None
        }
    } else {
        None
    };

    let download_target = if file_url.is_empty() {
        edge_fallback_url.clone().unwrap_or_default()
    } else {
        file_url.clone()
    };

    let resp_res = state.http.get(&download_target).send().await;
    let resp = match resp_res {
        Ok(r) if r.status().is_success() => r,
        _ => {
            if let Some(ref fallback) = edge_fallback_url {
                if fallback != &download_target {
                    tracing::info!(fallback_url = %fallback, "Primary download failed, attempting CurseForge Edge CDN fallback");
                    state.http.get(fallback).send().await?.error_for_status()?
                } else if let Ok(r) = resp_res {
                    r.error_for_status()?
                } else {
                    return Err(crate::error::AppError::NotFound(format!("Failed to download file from {}", download_target)));
                }
            } else {
                resp_res?.error_for_status()?
            }
        }
    };

    use futures_util::StreamExt;
    let safe_name = std::path::Path::new(&file_name)
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("mod.jar");
    let file_path = target_dir.join(safe_name);
    let mut stream = resp.bytes_stream();
    let mut file = tokio::fs::File::create(&file_path).await.map_err(|e| {
        tracing::error!(path = %file_path.display(), error = %e, "failed to create file");
        e
    })?;
    while let Some(chunk) = stream.next().await {
        let bytes = chunk.map_err(|e| {
            tracing::error!(url = %file_url, error = %e, "stream error");
            crate::error::AppError::Http(e)
        })?;
        let _ = tokio::io::AsyncWriteExt::write_all(&mut file, &bytes).await;
    }
    let _ = tokio::io::AsyncWriteExt::flush(&mut file).await;
    drop(file);

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
        let _ = tokio::fs::copy(&file_path, prof_dest_dir.join(&file_name)).await;
    }

    if target_subfolder == "mods" {
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
        if file_name.ends_with(".jar") {
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
    }

    Ok(())
}

#[tauri::command]
pub async fn mods_install(state: State<'_, AppState>, request: ModInstallRequest) -> AppResult<()> {
    mods_install_core(&state, request).await
}

pub async fn mods_install_with_deps_core(
    state: &AppState,
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
    while !to_install.is_empty() && depth < 5 {
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
                let safe_dep_name = std::path::Path::new(&file.filename)
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("mod.jar");
                let file_path = mods_dir.join(safe_dep_name);

                let mut file_stream = resp.bytes_stream();
                let mut dest_file = tokio::fs::File::create(&file_path).await?;
                while let Some(chunk) = file_stream.next().await {
                    let chunk = chunk?;
                    dest_file.write_all(&chunk).await?;
                }
                dest_file.flush().await?;
                drop(dest_file);

                if let Some(ref prof) = profile {
                    let prof_mods_dir = std::path::PathBuf::from(&prof.game_dir).join("mods");
                    let _ = tokio::fs::create_dir_all(&prof_mods_dir).await;
                    let _ = tokio::fs::copy(&file_path, prof_mods_dir.join(safe_dep_name)).await;
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
pub async fn mods_install_with_deps(
    state: State<'_, AppState>,
    request: ModInstallRequest,
) -> AppResult<Vec<String>> {
    mods_install_with_deps_core(&state, request).await
}

#[tauri::command]
#[allow(non_snake_case)]
pub async fn mods_remove(
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

pub async fn mods_check_updates_core(
    state: &AppState,
    profile_id: String,
) -> AppResult<Vec<ModUpdate>> {
    let db = crate::db::shared_db().await?;
    let installed = crate::db::schema::mods::list_by_profile(&db, &profile_id).await?;

    let profile =
        sqlx::query_as::<_, crate::db::models::ProfileRow>("SELECT * FROM profiles WHERE id = ?")
            .bind(&profile_id)
            .fetch_optional(db.pool())
            .await?
            .ok_or_else(|| {
                crate::error::AppError::NotFound(format!("profile {profile_id} not found"))
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
            if let Ok(versions) = curseforge::get_mod_versions(&state.http, &mod_row.project_id, &profile.mc_version).await {
                if let Some(latest) = versions.first() {
                    if latest.id != mod_row.version_id {
                        let dl_url = latest.files.first().map(|f| f.url.clone()).unwrap_or_default();
                        updates.push(ModUpdate {
                            project_id: mod_row.project_id.clone(),
                            project_name: latest.name.clone(),
                            current_version_id: mod_row.version_id.clone(),
                            latest_version_id: latest.id.clone(),
                            latest_version_number: latest.version_number.clone(),
                            download_url: dl_url,
                        });
                    }
                }
            }
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
pub async fn mods_check_updates(
    state: State<'_, AppState>,
    profileId: String,
) -> AppResult<Vec<ModUpdate>> {
    mods_check_updates_core(&state, profileId).await
}

pub async fn mods_update_core(
    state: &AppState,
    project_id: String,
    version_id: String,
    profile_id: String,
) -> AppResult<()> {
    let db = crate::db::shared_db().await?;

    let old = sqlx::query_as::<_, crate::db::schema::mods::ModRow>(
        "SELECT * FROM mods WHERE profile_id = ? AND project_id = ?",
    )
    .bind(&profile_id)
    .bind(&project_id)
    .fetch_optional(db.pool())
    .await?;

    let profile = sqlx::query_as::<_, crate::db::models::ProfileRow>(
        "SELECT * FROM profiles WHERE id = ?",
    )
    .bind(&profile_id)
    .fetch_optional(db.pool())
    .await?;

    let source = old
        .as_ref()
        .map(|m| m.source.clone())
        .unwrap_or_else(|| "modrinth".into());

    let base_dir = directories::ProjectDirs::from("io", "github", "Luxmc").ok_or_else(|| {
        crate::error::AppError::InvalidState("could not determine data dir".into())
    })?;
    let mods_dir = base_dir.data_dir().join("mods").join(&profile_id);
    tokio::fs::create_dir_all(&mods_dir).await?;

    if let Some(ref old_mod) = old {
        let old_path = mods_dir.join(&old_mod.file_name);
        let _ = tokio::fs::remove_file(&old_path).await;
    }

    let (file_url, file_name, file_sha1) = match source.as_str() {
        "curseforge" => {
            let versions = curseforge::get_mod_versions(&state.http, &project_id, "").await?;
            let version = versions
                .iter()
                .find(|v| v.id == version_id)
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
            let versions = client.get_mod_versions(&project_id, "").await?;
            let version = versions
                .iter()
                .find(|v| v.id == version_id)
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
    let file_path = mods_dir.join(&file_name);
    let mut file_stream = resp.bytes_stream();
    let mut dest_file = tokio::fs::File::create(&file_path).await?;
    while let Some(chunk) = file_stream.next().await {
        let chunk = chunk?;
        dest_file.write_all(&chunk).await?;
    }
    dest_file.flush().await?;
    drop(dest_file);

    if let Some(ref prof) = profile {
        let prof_mods_dir = std::path::PathBuf::from(&prof.game_dir).join("mods");
        let _ = tokio::fs::create_dir_all(&prof_mods_dir).await;
        if let Some(ref old_mod) = old {
            let old_game_path = prof_mods_dir.join(&old_mod.file_name);
            let _ = tokio::fs::remove_file(&old_game_path).await;
            let old_disabled_path = prof_mods_dir.join(format!("{}.disabled", old_mod.file_name));
            let _ = tokio::fs::remove_file(&old_disabled_path).await;
        }
        let _ = tokio::fs::copy(&file_path, prof_mods_dir.join(&file_name)).await;
    }

    let mod_row = crate::db::schema::mods::ModRow {
        profile_id: profile_id.clone(),
        project_id: project_id.clone(),
        version_id: version_id.clone(),
        file_name,
        sha1: file_sha1,
        source,
        installed_at: String::new(),
    };
    crate::db::schema::mods::upsert(&db, &mod_row).await
}

#[tauri::command]
#[allow(non_snake_case)]
pub async fn mods_update(
    state: State<'_, AppState>,
    projectId: String,
    versionId: String,
    profileId: String,
) -> AppResult<()> {
    mods_update_core(&state, projectId, versionId, profileId).await
}

pub async fn mods_download_to_temp_core(
    state: &AppState,
    url: String,
    file_name: String,
) -> AppResult<String> {
    use crate::core::mods::pack_download as pack;
    let _import = state.import_lock.try_lock().map_err(|_| crate::error::AppError::InvalidState("Já existe uma importação em andamento.".into()))?;
    state.import_cancel.store(false, std::sync::atomic::Ordering::SeqCst);
    let urls = pack::mirrors(&url)?;
    if file_name.contains(['/', '\\']) {
        return Err(crate::error::AppError::InvalidInput("Invalid filename".into()));
    }
    let base = directories::ProjectDirs::from("io", "github", "Luxmc")
        .ok_or_else(|| crate::error::AppError::InvalidState("could not determine cache dir".into()))?;
    let directory = base.cache_dir().join("modpacks").join(uuid::Uuid::new_v4().to_string());
    let target = pack::destination(&directory, &file_name)?;
    let client = pack::client()?;
    let mut last_error = crate::error::AppError::InvalidState("Download failed".into());
    for attempt in 0..3 {
        pack::backoff(attempt, Some(&state.import_cancel)).await?;
        for url in &urls {
            match pack::bytes(&client, url, Some(&state.import_cancel)).await {
                Ok(bytes) => {
                    if zip::ZipArchive::new(std::io::Cursor::new(&bytes)).is_err() {
                        last_error = crate::error::AppError::InvalidInput("Invalid pack archive".into());
                        continue;
                    }
                    pack::atomic_write(&target, &bytes).await?;
                    return Ok(target.to_string_lossy().into_owned());
                }
                Err(error) => last_error = error,
            }
            pack::cancelled(Some(&state.import_cancel))?;
        }
    }
    Err(last_error)
}

#[tauri::command]
#[allow(non_snake_case)]
pub async fn mods_download_to_temp(
    state: State<'_, AppState>,
    url: String,
    fileName: String,
) -> AppResult<String> {
    mods_download_to_temp_core(&state, url, fileName).await
}

#[tauri::command]
pub fn curseforge_status() -> bool {
    crate::core::mods::curseforge::has_key()
}

#[tauri::command]
pub fn curseforge_get_key() -> Option<String> {
    crate::core::mods::curseforge::api_key()
}

#[tauri::command]
pub async fn curseforge_set_key(key: String) -> Result<(), String> {
    let trimmed = key.trim().to_string();
    crate::core::mods::curseforge::store_key(&trimmed)?;
    if let Ok(conn) = crate::db::shared_db().await {
        let _ = sqlx::query("INSERT INTO app_settings (key, value) VALUES ('curseforge_api_key', ?) ON CONFLICT(key) DO UPDATE SET value = excluded.value")
            .bind(&trimmed)
            .execute(conn.pool())
            .await;
    }
    Ok(())
}

#[tauri::command]
pub async fn curseforge_remove_key() -> Result<(), String> {
    crate::core::mods::curseforge::remove_key()?;
    if let Ok(conn) = crate::db::shared_db().await {
        let _ = sqlx::query("DELETE FROM app_settings WHERE key = 'curseforge_api_key'")
            .execute(conn.pool())
            .await;
    }
    Ok(())
}

pub async fn curseforge_validate_key_core(http: &reqwest::Client) -> Result<bool, String> {
    crate::core::mods::curseforge::validate_key(http).await
}

#[tauri::command]
pub async fn curseforge_validate_key(state: State<'_, AppState>) -> Result<bool, String> {
    curseforge_validate_key_core(&state.http).await
}

pub fn extract_mod_name_from_jar(jar_path: &std::path::Path) -> Option<String> {
    let file = std::fs::File::open(jar_path).ok()?;
    let mut archive = zip::ZipArchive::new(std::io::BufReader::new(file)).ok()?;

    // 1. Try fabric.mod.json
    if let Ok(entry) = archive.by_name("fabric.mod.json") {
        if let Ok(json) = serde_json::from_reader::<_, serde_json::Value>(entry) {
            if let Some(name) = json.get("name").and_then(|n| n.as_str()) {
                if !name.trim().is_empty() {
                    return Some(name.trim().to_string());
                }
            }
            if let Some(id) = json.get("id").and_then(|i| i.as_str()) {
                if !id.trim().is_empty() {
                    return Some(id.trim().to_string());
                }
            }
        }
    }

    // 2. Try quilt.mod.json
    if let Ok(entry) = archive.by_name("quilt.mod.json") {
        if let Ok(json) = serde_json::from_reader::<_, serde_json::Value>(entry) {
            if let Some(name) = json.pointer("/quilt_loader/metadata/name").and_then(|n| n.as_str()) {
                if !name.trim().is_empty() {
                    return Some(name.trim().to_string());
                }
            }
        }
    }

    // 3. Try META-INF/neoforge.mods.toml or META-INF/mods.toml
    for toml_name in &["META-INF/neoforge.mods.toml", "META-INF/mods.toml"] {
        if let Ok(mut entry) = archive.by_name(toml_name) {
            use std::io::Read;
            let mut content = String::new();
            if entry.read_to_string(&mut content).is_ok() {
                for line in content.lines() {
                    let trimmed = line.trim();
                    if trimmed.starts_with("displayName") {
                        if let Some(val) = trimmed.split('=').nth(1) {
                            let clean = val.trim().trim_matches('"').trim_matches('\'').trim();
                            if !clean.is_empty() && !clean.starts_with("${") {
                                return Some(clean.to_string());
                            }
                        }
                    }
                }
            }
        }
    }

    // 4. Try mcmod.info
    if let Ok(entry) = archive.by_name("mcmod.info") {
        if let Ok(json) = serde_json::from_reader::<_, serde_json::Value>(entry) {
            let item = if let Some(arr) = json.as_array() {
                arr.first()
            } else if let Some(modlist) = json.get("modList").and_then(|m| m.as_array()) {
                modlist.first()
            } else {
                Some(&json)
            };
            if let Some(item) = item {
                if let Some(name) = item.get("name").and_then(|n| n.as_str()) {
                    if !name.trim().is_empty() {
                        return Some(name.trim().to_string());
                    }
                }
            }
        }
    }

    None
}

pub fn extract_mod_icon_from_jar(jar_path: &std::path::Path) -> Option<String> {
    use base64::Engine;
    use std::io::Read;
    let file = std::fs::File::open(jar_path).ok()?;
    let mut archive = zip::ZipArchive::new(std::io::BufReader::new(file)).ok()?;

    let mut icon_path: Option<String> = None;
    let mut mod_id: Option<String> = None;

    if let Ok(entry) = archive.by_name("fabric.mod.json") {
        if let Ok(json) = serde_json::from_reader::<_, serde_json::Value>(entry) {
            if let Some(id) = json.get("id").and_then(|i| i.as_str()) {
                mod_id = Some(id.trim().to_string());
            }
            if let Some(icon) = json.get("icon").and_then(|i| i.as_str()) {
                icon_path = Some(icon.trim_start_matches('/').to_string());
            } else if let Some(icons) = json.get("icon").and_then(|i| i.as_object()) {
                if let Some(first) = icons.values().next().and_then(|v| v.as_str()) {
                    icon_path = Some(first.trim_start_matches('/').to_string());
                }
            }
        }
    }

    if icon_path.is_none() {
        if let Ok(entry) = archive.by_name("quilt.mod.json") {
            if let Ok(json) = serde_json::from_reader::<_, serde_json::Value>(entry) {
                if let Some(id) = json.pointer("/quilt_loader/id").and_then(|i| i.as_str()) {
                    mod_id = Some(id.trim().to_string());
                }
                if let Some(icon) = json.pointer("/quilt_loader/metadata/icon").and_then(|i| i.as_str()) {
                    icon_path = Some(icon.trim_start_matches('/').to_string());
                }
            }
        }
    }

    if icon_path.is_none() || mod_id.is_none() {
        for toml_name in &["META-INF/neoforge.mods.toml", "META-INF/mods.toml"] {
            if let Ok(mut entry) = archive.by_name(toml_name) {
                let mut content = String::new();
                if entry.read_to_string(&mut content).is_ok() {
                    for line in content.lines() {
                        let trimmed = line.trim();
                        if (trimmed.starts_with("modId") || trimmed.starts_with("mod_id")) && mod_id.is_none() {
                            if let Some(val) = trimmed.split('=').nth(1) {
                                let clean = val.trim().trim_matches('"').trim_matches('\'').trim();
                                if !clean.is_empty() {
                                    mod_id = Some(clean.to_string());
                                }
                            }
                        }
                        if trimmed.starts_with("logoFile") && icon_path.is_none() {
                            if let Some(val) = trimmed.split('=').nth(1) {
                                let clean = val.trim().trim_matches('"').trim_matches('\'').trim();
                                if !clean.is_empty() {
                                    icon_path = Some(clean.trim_start_matches('/').to_string());
                                }
                            }
                        }
                    }
                }
            }
            if icon_path.is_some() && mod_id.is_some() {
                break;
            }
        }
    }

    if icon_path.is_none() {
        if let Ok(mut entry) = archive.by_name("mcmod.info") {
            let mut content = String::new();
            if entry.read_to_string(&mut content).is_ok() {
                if let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) {
                    let first = if let Some(arr) = json.as_array() {
                        arr.first()
                    } else if let Some(arr) = json.get("modList").and_then(|l| l.as_array()) {
                        arr.first()
                    } else {
                        Some(&json)
                    };
                    if let Some(obj) = first {
                        if let Some(id) = obj.get("modid").and_then(|i| i.as_str()) {
                            mod_id = Some(id.trim().to_string());
                        }
                        if let Some(lf) = obj.get("logoFile").and_then(|l| l.as_str()) {
                            let clean = lf.trim().trim_matches('"').trim_matches('\'').trim();
                            if !clean.is_empty() {
                                icon_path = Some(clean.trim_start_matches('/').to_string());
                            }
                        }
                    }
                }
            }
        }
    }

    let mut candidate_paths: Vec<String> = Vec::new();
    if let Some(ref p) = icon_path {
        candidate_paths.push(p.clone());
        if let Some(ref mid) = mod_id {
            candidate_paths.push(format!("assets/{}/{}", mid, p));
            candidate_paths.push(format!("assets/{}/textures/{}", mid, p));
            candidate_paths.push(format!("assets/{}/textures/gui/{}", mid, p));
        }
    }
    if let Some(ref mid) = mod_id {
        candidate_paths.push(format!("assets/{}/icon.png", mid));
        candidate_paths.push(format!("assets/{}/logo.png", mid));
        candidate_paths.push(format!("assets/{}/textures/icon.png", mid));
        candidate_paths.push(format!("assets/{}/textures/logo.png", mid));
        candidate_paths.push(format!("assets/{}/textures/gui/icon.png", mid));
    }
    candidate_paths.push("icon.png".into());
    candidate_paths.push("assets/icon.png".into());
    candidate_paths.push("logo.png".into());
    candidate_paths.push("assets/logo.png".into());
    candidate_paths.push("pack.png".into());

    for candidate in &candidate_paths {
        if let Ok(mut entry) = archive.by_name(candidate) {
            if entry.size() > 0 && entry.size() < 2_000_000 {
                let mut buf = Vec::new();
                if entry.read_to_end(&mut buf).is_ok() && !buf.is_empty() {
                    let mime = if candidate.ends_with(".jpg") || candidate.ends_with(".jpeg") {
                        "jpeg"
                    } else if candidate.ends_with(".webp") {
                        "webp"
                    } else {
                        "png"
                    };
                    let b64 = base64::engine::general_purpose::STANDARD.encode(&buf);
                    return Some(format!("data:image/{};base64,{}", mime, b64));
                }
            }
        }
    }

    for i in 0..archive.len() {
        if let Ok(mut entry) = archive.by_index(i) {
            let name = entry.name().to_lowercase();
            let matches_icon_name = icon_path.as_ref().map_or(false, |ip| {
                let lower_ip = ip.to_lowercase();
                name == lower_ip || name.ends_with(&format!("/{}", lower_ip))
            });

            let is_generic_icon = name.ends_with("/icon.png")
                || name.ends_with("icon.png")
                || name.ends_with("/logo.png")
                || name.ends_with("logo.png")
                || name.ends_with("/pack.png")
                || (name.starts_with("assets/") && name.ends_with(".png") && (name.contains("icon") || name.contains("logo")));

            if (matches_icon_name || is_generic_icon) && entry.size() > 0 && entry.size() < 2_000_000 {
                let mut buf = Vec::new();
                if entry.read_to_end(&mut buf).is_ok() && !buf.is_empty() {
                    let mime = if name.ends_with(".jpg") || name.ends_with(".jpeg") {
                        "jpeg"
                    } else if name.ends_with(".webp") {
                        "webp"
                    } else {
                        "png"
                    };
                    let b64 = base64::engine::general_purpose::STANDARD.encode(&buf);
                    return Some(format!("data:image/{};base64,{}", mime, b64));
                }
            }
        }
    }

    None
}

pub async fn mods_resolve_names_core(
    state: &AppState,
    profile_id: String,
) -> AppResult<u32> {
    let db = crate::db::shared_db().await?;
    let row = sqlx::query_as::<_, crate::db::models::ProfileRow>("SELECT * FROM profiles WHERE id = ?")
        .bind(&profile_id)
        .fetch_optional(db.pool())
        .await?
        .ok_or_else(|| crate::error::AppError::NotFound(format!("profile {profile_id} not found")))?;

    let mods_dir = std::path::PathBuf::from(&row.game_dir).join("mods");
    if !mods_dir.exists() {
        return Ok(0);
    }

    let mut renamed = 0u32;
    let mut entries = tokio::fs::read_dir(&mods_dir).await?;

    let mut project_ids: Vec<u64> = Vec::new();
    let mut file_map: Vec<(std::path::PathBuf, String)> = Vec::new();

    while let Ok(Some(entry)) = entries.next_entry().await {
        let name = entry.file_name().to_string_lossy().to_string();
        let bare = name.strip_suffix(".disabled").unwrap_or(&name);
        let is_numeric_pair = if let Some(dollar_pos) = bare.find('_') {
            if bare[dollar_pos + 1..].ends_with(".jar") {
                let prefix = &bare[..dollar_pos];
                let suffix = bare[dollar_pos + 1..].strip_suffix(".jar").unwrap_or("");
                !prefix.is_empty() && !suffix.is_empty() && prefix.parse::<u64>().is_ok() && suffix.parse::<u64>().is_ok()
            } else {
                false
            }
        } else {
            false
        };

        let is_numeric_single = bare.strip_suffix(".jar").map(|s| !s.is_empty() && s.chars().all(|c| c.is_ascii_digit())).unwrap_or(false);

        if is_numeric_pair || is_numeric_single {
            let entry_path = entry.path();
            if let Some(jar_name) = extract_mod_name_from_jar(&entry_path) {
                let safe_name = jar_name.replace(['/', '\\', ':', '*', '?', '"', '<', '>', '|'], "_");
                let is_disabled = name.ends_with(".disabled");
                let new_name = if is_disabled {
                    format!("{}.jar.disabled", safe_name)
                } else {
                    format!("{}.jar", safe_name)
                };
                let new_path = entry_path.parent().unwrap_or(&mods_dir).join(&new_name);
                if new_path != entry_path {
                    if tokio::fs::rename(&entry_path, &new_path).await.is_ok() {
                        renamed += 1;
                        continue;
                    }
                }
            }

            if is_numeric_pair {
                if let Some(dollar_pos) = bare.find('_') {
                    if let Ok(pid) = bare[..dollar_pos].parse::<u64>() {
                        project_ids.push(pid);
                        file_map.push((entry_path, name));
                    }
                }
            }
        }
    }

    if !project_ids.is_empty() {
        let mod_names = crate::core::mods::curseforge::get_mod_names_batch(&state.http, &project_ids).await;

        for (i, (path, old_name)) in file_map.iter().enumerate() {
            if let Some(name) = mod_names.get(&project_ids[i]) {
                if !name.is_empty() && name != &project_ids[i].to_string() {
                    let safe_name = name.replace(['/', '\\', ':', '*', '?', '"', '<', '>', '|'], "_");
                    let is_disabled = old_name.ends_with(".disabled");
                    let new_name = if is_disabled {
                        format!("{}.jar.disabled", safe_name)
                    } else {
                        format!("{}.jar", safe_name)
                    };
                    let new_path = path.parent().unwrap_or(&mods_dir).join(&new_name);
                    if new_path != *path {
                        if tokio::fs::rename(path, &new_path).await.is_ok() {
                            renamed += 1;
                            tracing::info!(old = %old_name, new = %new_name, "renamed mod file");
                        }
                    }
                }
            }
        }
    }

    Ok(renamed)
}

#[tauri::command]
pub async fn mods_resolve_names(
    state: State<'_, AppState>,
    profile_id: String,
) -> AppResult<u32> {
    mods_resolve_names_core(&state, profile_id).await
}

pub async fn mods_resolve_icons_core(
    state: &AppState,
    profile_id: String,
) -> AppResult<u32> {
    let db = crate::db::shared_db().await?;
    let row = sqlx::query_as::<_, crate::db::models::ProfileRow>("SELECT * FROM profiles WHERE id = ?")
        .bind(&profile_id)
        .fetch_optional(db.pool())
        .await?
        .ok_or_else(|| crate::error::AppError::NotFound(format!("profile {profile_id} not found")))?;

    let mods_dir = std::path::PathBuf::from(&row.game_dir).join("mods");
    if !mods_dir.exists() {
        return Ok(0);
    }

    let db_mods = crate::db::schema::mods::list_by_profile(&db, &profile_id).await.unwrap_or_default();
    let mut resolved_count = 0u32;
    let mut cf_project_ids: Vec<u64> = Vec::new();
    let mut mr_slugs: Vec<String> = Vec::new();

    for m in &db_mods {
        if m.source == "curseforge" {
            if let Ok(pid) = m.project_id.parse::<u64>() {
                cf_project_ids.push(pid);
            }
        } else if m.source == "modrinth" || m.source == "modrinth_fallback" {
            mr_slugs.push(m.project_id.clone());
        }
    }

    let manifest_path = std::path::PathBuf::from(&row.game_dir).join("manifest.json");
    if manifest_path.exists() {
        if let Ok(content) = tokio::fs::read_to_string(&manifest_path).await {
            if let Ok(manifest) = serde_json::from_str::<crate::commands::instances::CfManifest>(&content) {
                for f in manifest.files {
                    cf_project_ids.push(f.project_id);
                }
            }
        }
    }

    cf_project_ids.dedup();
    if !cf_project_ids.is_empty() {
        let logos = crate::core::mods::curseforge::get_mod_logos_batch(&state.http, &cf_project_ids).await;
        for (pid, url) in logos {
            let pid_str = pid.to_string();
            let _ = sqlx::query(
                "INSERT INTO mod_icons (key, icon_url) VALUES (?, ?) ON CONFLICT(key) DO UPDATE SET icon_url = excluded.icon_url"
            )
            .bind(&pid_str)
            .bind(&url)
            .execute(db.pool())
            .await;
            resolved_count += 1;

            for m in &db_mods {
                if m.project_id == pid_str {
                    let _ = sqlx::query(
                        "INSERT INTO mod_icons (key, icon_url) VALUES (?, ?) ON CONFLICT(key) DO UPDATE SET icon_url = excluded.icon_url"
                    )
                    .bind(&m.file_name)
                    .bind(&url)
                    .execute(db.pool())
                    .await;
                }
            }
        }
    }

    mr_slugs.dedup();
    if !mr_slugs.is_empty() {
        for chunk in mr_slugs.chunks(20) {
            let ids_param = serde_json::to_string(&chunk).unwrap_or_default();
            let url = format!("https://api.modrinth.com/v2/projects?ids={}", urlencoding::encode(&ids_param));
            if let Ok(resp) = state.http.get(&url).header("User-Agent", "Luxmc/1.6.5").send().await {
                if resp.status().is_success() {
                    if let Ok(projects) = resp.json::<Vec<serde_json::Value>>().await {
                        for p in projects {
                            if let (Some(id), Some(icon)) = (
                                p.get("id").and_then(|i| i.as_str()),
                                p.get("icon_url").and_then(|u| u.as_str())
                            ) {
                                let _ = sqlx::query(
                                    "INSERT INTO mod_icons (key, icon_url) VALUES (?, ?) ON CONFLICT(key) DO UPDATE SET icon_url = excluded.icon_url"
                                )
                                .bind(id)
                                .bind(icon)
                                .execute(db.pool())
                                .await;
                                resolved_count += 1;
                            }
                        }
                    }
                }
            }
        }
    }

    Ok(resolved_count)
}

#[tauri::command]
pub async fn mods_resolve_icons(
    state: State<'_, AppState>,
    profile_id: String,
) -> AppResult<u32> {
    mods_resolve_icons_core(&state, profile_id).await
}

#[cfg(test)]
mod download_cancellation_tests {
    use super::*;

    #[tokio::test]
    async fn concurrent_download_cannot_reset_import_cancellation() {
        let state = AppState::default();
        let _guard = state.import_lock.lock().await;
        state.import_cancel.store(true, std::sync::atomic::Ordering::SeqCst);
        let result = mods_download_to_temp_core(&state, "https://cdn.modrinth.com/test.mrpack".into(), "test.mrpack".into()).await;
        assert!(result.is_err());
        assert!(state.import_cancel.load(std::sync::atomic::Ordering::SeqCst));
    }
}
