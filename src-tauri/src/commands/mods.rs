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
    #[serde(default)]
    pub world_name: Option<String>,
    #[serde(default)]
    pub icon_url: Option<String>,
}

fn default_source() -> String {
    "modrinth".into()
}

pub(crate) fn content_icon_key(path: &std::path::Path, metadata: &std::fs::Metadata) -> String {
    let modified = metadata.modified().ok().and_then(|time| time.duration_since(std::time::UNIX_EPOCH).ok()).map(|time| time.as_nanos()).unwrap_or(0);
    format!("content:{}:{}:{}", path.to_string_lossy(), metadata.len(), modified)
}

async fn cache_content_icon(pool: &sqlx::SqlitePool, path: &std::path::Path, icon: Option<&str>) {
    let Some(icon) = icon.filter(|icon| icon.len() <= 4096 && url::Url::parse(icon).ok().is_some_and(|url| url.scheme() == "https")) else { return; };
    if let Ok(metadata) = tokio::fs::metadata(path).await {
        let _ = sqlx::query("INSERT INTO mod_icons (key, icon_url) VALUES (?, ?) ON CONFLICT(key) DO UPDATE SET icon_url = excluded.icon_url")
            .bind(content_icon_key(path, &metadata)).bind(icon).execute(pool).await;
    }
}

fn normalize_slug(s: &str) -> String {
    s.to_lowercase()
        .replace(['-', '_'], "")
        .trim()
        .to_string()
}

fn safe_file_name(name: &str) -> String {
    std::path::Path::new(name)
        .file_name()
        .and_then(|n| n.to_str())
        .filter(|n| !n.is_empty() && *n != "." && *n != "..")
        .map(|n| n.to_string())
        .unwrap_or_else(|| "mod.jar".to_string())
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

use std::sync::OnceLock;
use std::time::{Duration, Instant};
use dashmap::DashMap;

static MODS_SEARCH_CACHE: OnceLock<DashMap<String, (Instant, Vec<ModSearchResult>)>> = OnceLock::new();

fn get_mods_search_cache() -> &'static DashMap<String, (Instant, Vec<ModSearchResult>)> {
    MODS_SEARCH_CACHE.get_or_init(DashMap::new)
}

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
    let src = source.as_deref().unwrap_or("all").to_lowercase();

    let cache_key = format!(
        "{}:{}:{}:{}:{}:{}:{}:{}:{}",
        query,
        mc_version,
        limit,
        offset,
        content_type,
        sort,
        loader.as_deref().unwrap_or(""),
        category.as_deref().unwrap_or(""),
        src
    );

    let cache = get_mods_search_cache();
    if let Some(entry) = cache.get(&cache_key) {
        if entry.0.elapsed() < Duration::from_secs(300) {
            return Ok(entry.1.clone());
        }
    }

    let modrinth_client = ModrinthClient::new(state.http.clone());

    let (modrinth_results, curseforge_results) = match src.as_str() {
        "modrinth" => {
            let m = tokio::time::timeout(Duration::from_millis(3500), async {
                modrinth_client
                    .search_mods(&query, &mc_version, &content_type, loader.as_deref(), category.as_deref(), limit, offset, Some(sort))
                    .await
            })
            .await
            .unwrap_or_else(|_| {
                tracing::warn!("Modrinth search timed out");
                Ok(Vec::new())
            })
            .unwrap_or_else(|e| {
                tracing::warn!(error = %e, "Modrinth search failed");
                Vec::new()
            });
            (m, Vec::new())
        }
        "curseforge" => {
            let c = tokio::time::timeout(Duration::from_millis(3000), async {
                curseforge::search_mods(&state.http, &query, &mc_version, &content_type, loader.as_deref(), limit, offset, Some(sort))
                    .await
            })
            .await
            .unwrap_or_else(|_| {
                tracing::warn!("CurseForge search timed out");
                Ok(Vec::new())
            })
            .unwrap_or_else(|e| {
                tracing::warn!(error = %e, "CurseForge search failed");
                Vec::new()
            });
            (Vec::new(), c)
        }
        _ => {
            tokio::join!(
                async {
                    tokio::time::timeout(Duration::from_millis(3500), async {
                        modrinth_client
                            .search_mods(&query, &mc_version, &content_type, loader.as_deref(), category.as_deref(), limit, offset, Some(sort))
                            .await
                    })
                    .await
                    .unwrap_or_else(|_| {
                        tracing::warn!("Modrinth search timed out");
                        Ok(Vec::new())
                    })
                    .unwrap_or_else(|e| {
                        tracing::warn!(error = %e, "Modrinth search failed");
                        Vec::new()
                    })
                },
                async {
                    tokio::time::timeout(Duration::from_millis(3000), async {
                        curseforge::search_mods(&state.http, &query, &mc_version, &content_type, loader.as_deref(), limit, offset, Some(sort))
                            .await
                    })
                    .await
                    .unwrap_or_else(|_| {
                        tracing::warn!("CurseForge search timed out");
                        Ok(Vec::new())
                    })
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

    if cache.len() > 300 {
        cache.clear();
    }
    cache.insert(cache_key, (Instant::now(), combined.clone()));

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

pub(crate) async fn datapack_directory(game_dir: &str, name: Option<&str>) -> AppResult<std::path::PathBuf> {
        let name = name.ok_or_else(|| crate::error::AppError::InvalidInput("Selecione o mundo que receberá o datapack".into()))?;
        if name.is_empty() || name.contains(['/', '\\']) || name == "." || name == ".." { return Err(crate::error::AppError::InvalidInput("Mundo inválido".into())); }
        let saves = tokio::fs::canonicalize(std::path::PathBuf::from(game_dir).join("saves")).await?;
        let world = tokio::fs::canonicalize(saves.join(name)).await?;
        if world.parent() != Some(saves.as_path()) || !world.join("level.dat").is_file() { return Err(crate::error::AppError::InvalidInput("Mundo inválido".into())); }
        let dir = world.join("datapacks");
        tokio::fs::create_dir_all(&dir).await?;
        if tokio::fs::canonicalize(&dir).await?.parent() != Some(world.as_path()) { return Err(crate::error::AppError::InvalidInput("Pasta de datapacks inválida".into())); }
        Ok(dir)
}

pub async fn mods_install_core(state: &AppState, request: ModInstallRequest) -> AppResult<()> {
    if request.content_type.as_deref().is_none_or(|kind| kind == "mod") {
        mods_install_with_deps_core(state, request).await.map(|_| ())
    } else {
        let _guard = state.launch_lock.try_lock().map_err(|_| crate::error::AppError::InvalidState("Aguarde a preparação ou instalação em andamento.".into()))?;
        mods_install_single_core(state, request).await
    }
}

async fn mods_install_single_core(state: &AppState, request: ModInstallRequest) -> AppResult<()> {
    tracing::info!(profile_id = %request.profile_id, project_id = %request.project_id, version_id = %request.version_id, source = %request.source, content_type = ?request.content_type, "mods_install called");
    let content_type = request.content_type.as_deref().unwrap_or("mod").to_lowercase();
    let is_world = matches!(
        content_type.as_str(),
        "world" | "worlds" | "world pack" | "world_pack" | "worldpack"
    );
    let target_subfolder = if is_world {
        "world-import"
    } else {
        match content_type.as_str() {
            "shader" | "shaders" => "shaderpacks",
            "resourcepack" | "resource pack" | "resource_pack" => "resourcepacks",
            "datapack" | "data pack" | "data_pack" | "datapacks" => "datapacks",
            _ => "mods",
        }
    };

    let db = crate::db::shared_db().await?;
    let profile = sqlx::query_as::<_, crate::db::models::ProfileRow>("SELECT * FROM profiles WHERE id = ?")
        .bind(&request.profile_id)
        .fetch_optional(db.pool())
        .await?
        .ok_or_else(|| crate::error::AppError::NotFound(format!("profile {} not found", request.profile_id)))?;
    ensure_content_idle(&profile.game_dir)?;
    if target_subfolder == "mods" && matches!(profile.loader.as_str(), "vanilla" | "") {
        return Err(crate::error::AppError::InvalidInput("Esta instância Vanilla aceita resource packs e data packs, mas mods precisam de um loader compatível.".into()));
    }
    let target_dir = if is_world {
        std::env::temp_dir().join(format!("luxmc-world-{}", uuid::Uuid::new_v4()))
    } else if target_subfolder == "datapacks" {
        datapack_directory(&profile.game_dir, request.world_name.as_deref()).await?
    } else {
        let dir = std::path::PathBuf::from(&profile.game_dir).join(target_subfolder);
        tokio::fs::create_dir_all(&dir).await?;
        dir
    };

    let (file_url, file_name, file_size, file_sha1) = match request.source.as_str() {
        "curseforge" => {
            let versions = curseforge::get_mod_versions(&state.http, &request.project_id, &profile.mc_version).await?;
            let version = versions.iter().find(|version| version.id == request.version_id)
                .ok_or_else(|| crate::error::AppError::InvalidInput(format!("Nenhum arquivo selecionado compatível com Minecraft {}. A instância foi preservada.", profile.mc_version)))?;
            if target_subfolder == "mods" && !version.loaders.is_empty() && !loader_matches(&version.loaders, &profile.loader) {
                return Err(crate::error::AppError::InvalidInput("O mod selecionado pertence a outro loader.".into()));
            }
            if let Ok(file) = curseforge::get_mod_file_details(&state.http, &request.project_id, &request.version_id).await { (file.url, file.filename, file.size, file.sha1) }
            else {
                let file = version.files.first().ok_or_else(|| crate::error::AppError::NotFound("Arquivo CurseForge indisponível".into()))?;
                (file.url.clone(), file.filename.clone(), file.size, file.sha1.clone())
            }
        }
        _ => {
            let version = ModrinthClient::new(state.http.clone()).get_version_detail(&request.project_id, &request.version_id, &profile.mc_version).await?;
            if target_subfolder == "mods" { validate_mod_version(&version, &profile.mc_version, &profile.loader)?; }
            let file = version.files.first().ok_or_else(|| crate::error::AppError::NotFound("Arquivo Modrinth indisponível".into()))?;
            (file.url.clone(), file.filename.clone(), file.size, file.sha1.clone())
        }
    };
    let file_name = safe_file_name(&file_name);

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
    use tokio::io::AsyncWriteExt;
    let fallback_name = if is_world { "world.zip" } else { "mod.jar" };
    let safe_name = std::path::Path::new(&file_name)
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or(fallback_name);
    let file_path = target_dir.join(safe_name);
    let tmp_path = target_dir.join(format!("{}.{}.part", safe_name, uuid::Uuid::new_v4()));
    let write_result: AppResult<()> = async {
        let mut stream = resp.bytes_stream();
        let mut total = 0u64;
        let mut digest = sha1::Sha1::default();
        let mut file = tokio::fs::File::create(&tmp_path).await.map_err(|e| {
            tracing::error!(path = %tmp_path.display(), error = %e, "failed to create temp file");
            e
        })?;
        while let Some(chunk) = stream.next().await {
            let bytes = chunk.map_err(|e| {
                tracing::error!(url = %file_url, error = %e, "stream error");
                crate::error::AppError::Http(e)
            })?;
            total += bytes.len() as u64;
            sha1::Digest::update(&mut digest, &bytes);
            file.write_all(&bytes).await?;
        }
        file.flush().await?;
        drop(file);
        if total == 0 || (file_size > 0 && total != file_size) || (!file_sha1.is_empty() && !format!("{:x}", sha1::Digest::finalize(digest)).eq_ignore_ascii_case(&file_sha1)) {
            return Err(crate::error::AppError::InvalidInput("O download do conteúdo está incompleto ou falhou na verificação. O arquivo anterior foi preservado.".into()));
        }
        tokio::fs::rename(&tmp_path, &file_path).await?;
        Ok(())
    }
    .await;
    if let Err(error) = write_result {
        let _ = tokio::fs::remove_file(&tmp_path).await;
        return Err(error);
    }

    if is_world {
        let source = file_path.to_string_lossy().to_string();
        let imported =
            crate::commands::instances::instance_world_import(profile.id.clone(), source).await;
        let _ = tokio::fs::remove_dir_all(&target_dir).await;
        return imported.map(|_| ());
    }

    let resolved_profile_id = profile.id.clone();
    cache_content_icon(db.pool(), &file_path, request.icon_url.as_deref()).await;

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
    let _guard = state.launch_lock.try_lock().map_err(|_| crate::error::AppError::InvalidState("Aguarde a preparação ou instalação em andamento.".into()))?;
    if request.content_type.as_deref().is_some_and(|kind| kind != "mod") {
        let project = request.project_id.clone();
        mods_install_single_core(state, request).await?;
        return Ok(vec![project]);
    }
    let db = crate::db::shared_db().await?;
    let profile = sqlx::query_as::<_, crate::db::models::ProfileRow>("SELECT * FROM profiles WHERE id = ?")
        .bind(&request.profile_id).fetch_optional(db.pool()).await?
        .ok_or_else(|| crate::error::AppError::NotFound("Instância não encontrada; selecione novamente a instância.".into()))?;
    if matches!(profile.loader.as_str(), "vanilla" | "") {
        return Err(crate::error::AppError::InvalidInput("Mods precisam de Fabric, Quilt, Forge ou NeoForge. Escolha um loader compatível nas configurações da instância.".into()));
    }
    ensure_content_idle(&profile.game_dir)?;
    let client = ModrinthClient::new(state.http.clone());
    let mut queue = vec![(request.project_id.clone(), request.version_id.clone())];
    let mut planned = std::collections::BTreeMap::<String, crate::core::mods::ModVersionDetail>::new();
    let existing = crate::db::schema::mods::list_by_profile(&db, &profile.id).await?;
    while let Some((project, version_id)) = queue.pop() {
        if planned.len() >= 256 { return Err(crate::error::AppError::InvalidInput("A árvore de dependências excedeu 256 projetos.".into())); }
        let version = if request.source == "curseforge" {
            let versions = curseforge::get_mod_versions(&state.http, &project, &profile.mc_version).await?;
            let selected = versions.into_iter().find(|version| version.id == version_id).ok_or_else(|| crate::error::AppError::InvalidInput("Arquivo CurseForge incompatível com a versão da instância.".into()))?;
            let file = curseforge::get_mod_file_details(&state.http, &project, &version_id).await?;
            crate::core::mods::ModVersionDetail { id: selected.id, project_id: project.clone(), name: selected.name, version_number: selected.version_number, loaders: selected.loaders, game_versions: vec![profile.mc_version.clone()], files: vec![file], dependencies: curseforge::file_dependencies(&state.http, &project, &version_id).await? }
        } else { client.get_version_detail(&project, &version_id, &profile.mc_version).await? };
        validate_mod_version(&version, &profile.mc_version, &profile.loader)?;
        if let Some(previous) = planned.get(&version.project_id) {
            if previous.id != version.id { return Err(crate::error::AppError::InvalidInput(format!("Dependências exigem versões diferentes de {}. A instância foi preservada.", version.project_id))); }
            continue;
        }
        for dependency in &version.dependencies {
            if dependency.dependency_type != "required" { continue; }
            let selected = if let Some(pin) = &dependency.version_id { pin.clone() } else {
                if dependency.project_id.is_empty() { return Err(crate::error::AppError::InvalidInput("Uma dependência externa precisa de instalação manual. A instância foi preservada.".into())); }
                let mut candidates = if request.source == "curseforge" { curseforge::get_mod_versions(&state.http, &dependency.project_id, &profile.mc_version).await?.into_iter().filter(|version| loader_matches(&version.loaders, &profile.loader)).collect() } else { client.get_mod_versions_filtered(&dependency.project_id, &profile.mc_version, Some(&profile.loader)).await? };
                if request.source != "curseforge" && candidates.is_empty() && profile.loader == "quilt" { candidates = client.get_mod_versions_filtered(&dependency.project_id, &profile.mc_version, Some("fabric")).await?; }
                candidates.first().map(|version| version.id.clone()).ok_or_else(|| crate::error::AppError::InvalidInput(format!("Dependência {} sem versão compatível com Minecraft {} / {}.", dependency.project_id, profile.mc_version, profile.loader)))?
            };
            queue.push((dependency.project_id.clone(), selected));
        }
        planned.insert(version.project_id.clone(), version);
    }
    for version in planned.values() {
        for dependency in version.dependencies.iter().filter(|dep| dep.dependency_type == "incompatible") {
            let conflict = planned.values().any(|item| (dependency.project_id.is_empty() || item.project_id == dependency.project_id) && dependency.version_id.as_ref().is_none_or(|pin| pin == &item.id))
                || existing.iter().any(|item| item.source == request.source && !planned.contains_key(&item.project_id) && (dependency.project_id.is_empty() || item.project_id == dependency.project_id) && dependency.version_id.as_ref().is_none_or(|pin| pin == &item.version_id));
            if conflict { return Err(crate::error::AppError::InvalidInput(format!("{} é incompatível com {}. Nenhum arquivo foi alterado.", version.name, dependency.project_id))); }
        }
    }
    let mut filenames = HashSet::new();
    for version in planned.values() {
        let file = version.files.first().ok_or_else(|| crate::error::AppError::NotFound("Arquivo instalável ausente".into()))?;
        if !filenames.insert(safe_file_name(&file.filename)) { return Err(crate::error::AppError::InvalidInput("Dois projetos usam o mesmo nome de arquivo. A instalação foi cancelada para preservar a instância.".into())); }
    }
    let mods_dir = std::path::PathBuf::from(&profile.game_dir).join("mods");
    let staging = std::path::PathBuf::from(&profile.game_dir).join(".luxmc").join(format!("install-{}", uuid::Uuid::new_v4()));
    let result: AppResult<Vec<String>> = async {
        for version in planned.values() {
            let file = version.files.first().ok_or_else(|| crate::error::AppError::NotFound(format!("{} não possui arquivo instalável", version.name)))?;
            crate::core::downloader::ensure_artifact(&state.http, &staging.join(safe_file_name(&file.filename)), &file.url, file.size, &file.sha1).await?;
        }
        tokio::fs::create_dir_all(&mods_dir).await?;
        for version in planned.values() {
            let file = &version.files[0];
            let filename = safe_file_name(&file.filename);
            if !file.sha1.is_empty() {
                if let Some(alias) = existing.iter().find(|old| old.source != request.source && old.sha1.eq_ignore_ascii_case(&file.sha1)) {
                    crate::core::downloader::ensure_artifact(&state.http, &mods_dir.join(safe_file_name(&alias.file_name)), &file.url, file.size, &file.sha1).await?;
                    continue;
                }
            }
            tokio::fs::rename(staging.join(&filename), mods_dir.join(&filename)).await?;
            if version.project_id == request.project_id { cache_content_icon(db.pool(), &mods_dir.join(&filename), request.icon_url.as_deref()).await; }
            crate::db::schema::mods::upsert(&db, &ModRow { profile_id: profile.id.clone(), project_id: version.project_id.clone(), version_id: version.id.clone(), file_name: filename.clone(), sha1: file.sha1.clone(), source: request.source.clone(), installed_at: String::new() }).await?;
            if let Some(old) = existing.iter().find(|old| old.source == request.source && old.project_id == version.project_id && old.file_name != filename) {
                let old_path = mods_dir.join(safe_file_name(&old.file_name));
                if old_path.is_file() { tokio::fs::remove_file(old_path).await?; }
            }
        }
        sqlx::query("UPDATE profiles SET mod_count = (SELECT COUNT(*) FROM mods WHERE profile_id = ?) WHERE id = ?")
            .bind(&profile.id).bind(&profile.id).execute(db.pool()).await?;
        Ok(planned.keys().cloned().collect())
    }.await;
    let _ = tokio::fs::remove_dir_all(&staging).await;
    result
}

fn validate_mod_version(version: &crate::core::mods::ModVersionDetail, minecraft: &str, loader: &str) -> AppResult<()> {
    if !version.game_versions.iter().any(|value| value == minecraft) || !loader_matches(&version.loaders, loader) {
        return Err(crate::error::AppError::InvalidInput(format!("{} não é compatível com Minecraft {} / {}. A instância foi preservada.", version.name, minecraft, loader)));
    }
    Ok(())
}

fn ensure_content_idle(game_dir: &str) -> AppResult<()> {
    if crate::core::launcher::get_active_game_dir().as_deref() == Some(std::path::Path::new(game_dir)) {
        return Err(crate::error::AppError::InvalidState("Feche o Minecraft desta instância antes de alterar seus conteúdos.".into()));
    }
    Ok(())
}

fn loader_matches(loaders: &[String], loader: &str) -> bool {
    loaders.iter().any(|value| value == loader || (loader == "quilt" && value == "fabric"))
}

#[cfg(test)]
mod compatibility_tests {
    use super::*;

    #[test]
    fn exact_game_version_and_loader_are_required_before_installation() {
        let version = crate::core::mods::ModVersionDetail { id: "pinned".into(), project_id: "project".into(), name: "Dependency".into(), version_number: "1".into(), game_versions: vec!["1.20.1".into()], loaders: vec!["fabric".into()], files: vec![], dependencies: vec![] };
        assert!(validate_mod_version(&version, "1.20.1", "fabric").is_ok());
        assert!(validate_mod_version(&version, "1.20.1", "quilt").is_ok());
        for loader in ["vanilla", "forge", "neoforge"] { assert!(validate_mod_version(&version, "1.20.1", loader).is_err()); }
        assert!(validate_mod_version(&version, "1.20.2", "fabric").is_err());
    }
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
        let file_name = safe_file_name(&m.file_name);
        if let Some(base_dir) = directories::ProjectDirs::from("io", "github", "Luxmc") {
            let p = base_dir.data_dir().join("mods").join(&profileId).join(&file_name);
            let _ = tokio::fs::remove_file(p).await;
        }
        let profile = sqlx::query_as::<_, crate::db::models::ProfileRow>("SELECT * FROM profiles WHERE id = ?")
            .bind(&profileId)
            .fetch_optional(db.pool())
            .await?;
        if let Some(prof) = profile {
            let p = std::path::PathBuf::from(&prof.game_dir).join("mods").join(&file_name);
            let _ = tokio::fs::remove_file(p).await;
            let p_disabled = std::path::PathBuf::from(&prof.game_dir).join("mods").join(format!("{}.disabled", file_name));
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
                            file_name: Some(mod_row.file_name.clone()),
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
                    file_name: Some(mod_row.file_name.clone()),
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
    .await?
    .ok_or_else(|| crate::error::AppError::NotFound(format!("profile {profile_id} not found")))?;

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
        let old_path = mods_dir.join(safe_file_name(&old_mod.file_name));
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
    let file_name = safe_file_name(&file_name);

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

    let prof_mods_dir = std::path::PathBuf::from(&profile.game_dir).join("mods");
    tokio::fs::create_dir_all(&prof_mods_dir).await?;
    if let Some(ref old_mod) = old {
        let old_file_name = safe_file_name(&old_mod.file_name);
        let old_game_path = prof_mods_dir.join(&old_file_name);
        let _ = tokio::fs::remove_file(&old_game_path).await;
        let old_disabled_path = prof_mods_dir.join(format!("{}.disabled", old_file_name));
        let _ = tokio::fs::remove_file(&old_disabled_path).await;
    }
    tokio::fs::copy(&file_path, prof_mods_dir.join(&file_name)).await?;

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
    if let Ok(mut entry) = archive.by_name("fabric.mod.json") {
        if let Ok(json) = serde_json::from_reader::<_, serde_json::Value>(std::io::Read::take(&mut entry, 4 * 1024 * 1024)) {
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
    if let Ok(mut entry) = archive.by_name("quilt.mod.json") {
        if let Ok(json) = serde_json::from_reader::<_, serde_json::Value>(std::io::Read::take(&mut entry, 4 * 1024 * 1024)) {
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
            if entry.by_ref().take(1024 * 1024).read_to_string(&mut content).is_ok() {
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
    if let Ok(mut entry) = archive.by_name("mcmod.info") {
        if let Ok(json) = serde_json::from_reader::<_, serde_json::Value>(std::io::Read::take(&mut entry, 4 * 1024 * 1024)) {
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

fn mod_icon_data_url(bytes: &[u8]) -> Option<String> {
    use base64::Engine;
    let mut reader = image::ImageReader::new(std::io::Cursor::new(bytes)).with_guessed_format().ok()?;
    let mut limits = image::Limits::default();
    limits.max_image_width = Some(8192);
    limits.max_image_height = Some(8192);
    limits.max_alloc = Some(32 * 1024 * 1024);
    reader.limits(limits);
    let source = reader.decode().ok()?;
    let icon = source.thumbnail(128, 128);
    let mut output = std::io::Cursor::new(Vec::new());
    icon.write_to(&mut output, image::ImageFormat::Png).ok()?;
    if output.get_ref().len() > 24_000 {
        output = std::io::Cursor::new(Vec::new());
        source.thumbnail(64, 64).write_to(&mut output, image::ImageFormat::Png).ok()?;
    }
    Some(format!("data:image/png;base64,{}", base64::engine::general_purpose::STANDARD.encode(output.into_inner())))
}

pub(crate) fn compact_mod_icon(icon: &str) -> String {
    use base64::Engine;
    if icon.len() > 32_768 && icon.len() < 2_700_000 && icon.starts_with("data:image/") {
        if let Some((header, encoded)) = icon.split_once(',') {
            if header.ends_with(";base64") {
                if let Ok(bytes) = base64::engine::general_purpose::STANDARD.decode(encoded) {
                    if let Some(compact) = mod_icon_data_url(&bytes) { return compact; }
                }
            }
        }
    }
    icon.to_owned()
}

pub fn extract_mod_icon_from_jar(jar_path: &std::path::Path) -> Option<String> {
    type IconKey = (std::path::PathBuf, u64, Option<std::time::SystemTime>);
    #[derive(Default)]
    struct IconCache {
        entries: std::collections::HashMap<IconKey, Option<String>>,
        bytes: usize,
    }
    static CACHE: std::sync::LazyLock<std::sync::Mutex<IconCache>> = std::sync::LazyLock::new(|| std::sync::Mutex::new(IconCache::default()));
    let directory = jar_path.is_dir();
    let image_path = if directory { jar_path.join("pack.png") } else { jar_path.to_path_buf() };
    let metadata = std::fs::metadata(&image_path).ok()?;
    let key = (image_path.clone(), metadata.len(), metadata.modified().ok());
    if let Ok(cache) = CACHE.lock() {
        if let Some(icon) = cache.entries.get(&key) { return icon.clone(); }
    }
    let icon = if directory {
        if metadata.len() > 0 && metadata.len() < 2_000_000 {
            std::fs::read(image_path).ok().and_then(|bytes| mod_icon_data_url(&bytes))
        } else { None }
    } else { read_mod_icon_from_jar(jar_path) };
    if let Ok(mut cache) = CACHE.lock() {
        let bytes = icon.as_ref().map_or(0, String::len);
        if cache.entries.len() >= 4096 || cache.bytes + bytes > 32 * 1024 * 1024 {
            cache.entries.clear();
            cache.bytes = 0;
        }
        if !cache.entries.contains_key(&key) {
            cache.bytes += bytes;
            cache.entries.insert(key, icon.clone());
        }
    }
    icon
}

fn metadata_icon_path(value: &serde_json::Value) -> Option<String> {
    let path = value.as_str().map(str::to_owned).or_else(|| {
        value.as_object()?.iter().filter_map(|(size, path)| Some((size.parse::<u32>().ok()?, path.as_str()?)))
            .min_by_key(|(size, _)| size.abs_diff(128)).map(|(_, path)| path.to_owned())
    })?;
    let normalized = path.replace('\\', "/");
    Some(normalized.trim_start_matches('/').trim_start_matches("./").to_owned())
}

fn read_mod_icon_from_jar(jar_path: &std::path::Path) -> Option<String> {
    use std::io::Read;
    let file = std::fs::File::open(jar_path).ok()?;
    let mut archive = zip::ZipArchive::new(std::io::BufReader::new(file)).ok()?;

    let mut icon_path: Option<String> = None;
    let mut mod_id: Option<String> = None;

    if let Ok(mut entry) = archive.by_name("fabric.mod.json") {
        if let Ok(json) = serde_json::from_reader::<_, serde_json::Value>(std::io::Read::take(&mut entry, 4 * 1024 * 1024)) {
            if let Some(id) = json.get("id").and_then(|i| i.as_str()) {
                mod_id = Some(id.trim().to_string());
            }
            icon_path = json.get("icon").and_then(metadata_icon_path);
        }
    }

    if icon_path.is_none() {
        if let Ok(mut entry) = archive.by_name("quilt.mod.json") {
            if let Ok(json) = serde_json::from_reader::<_, serde_json::Value>(std::io::Read::take(&mut entry, 4 * 1024 * 1024)) {
                if let Some(id) = json.pointer("/quilt_loader/id").and_then(|i| i.as_str()) {
                    mod_id = Some(id.trim().to_string());
                }
                icon_path = json.pointer("/quilt_loader/metadata/icon").and_then(metadata_icon_path);
            }
        }
    }

    if icon_path.is_none() || mod_id.is_none() {
        for toml_name in &["META-INF/neoforge.mods.toml", "META-INF/mods.toml"] {
            if let Ok(mut entry) = archive.by_name(toml_name) {
                use std::io::Read;
                let mut content = String::new();
                if entry.by_ref().take(1024 * 1024).read_to_string(&mut content).is_ok() {
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
            use std::io::Read;
            let mut content = String::new();
            if entry.by_ref().take(1024 * 1024).read_to_string(&mut content).is_ok() {
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
                if entry.by_ref().take(2_000_000).read_to_end(&mut buf).is_ok() && !buf.is_empty() {
                    if let Some(icon) = mod_icon_data_url(&buf) { return Some(icon); }
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
                if entry.by_ref().take(2_000_000).read_to_end(&mut buf).is_ok() && !buf.is_empty() {
                    if let Some(icon) = mod_icon_data_url(&buf) { return Some(icon); }
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

    let mut candidates: Vec<(std::path::PathBuf, String, bool)> = Vec::new();

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
            candidates.push((entry.path(), name, is_numeric_pair));
        }
    }

    let pending_renames: Vec<(std::path::PathBuf, std::path::PathBuf)> = {
        let mods_dir = mods_dir.clone();
        let scan_targets = candidates.clone();
        tokio::task::spawn_blocking(move || {
            let mut out = Vec::new();
            for (entry_path, _name, _is_pair) in &scan_targets {
                let Some(jar_name) = extract_mod_name_from_jar(entry_path) else {
                    continue;
                };
                let safe_name = jar_name.replace(['/', '\\', ':', '*', '?', '"', '<', '>', '|'], "_");
                let new_name = if _name.ends_with(".disabled") {
                    format!("{}.jar.disabled", safe_name)
                } else {
                    format!("{}.jar", safe_name)
                };
                let new_path = entry_path.parent().unwrap_or(&mods_dir).join(&new_name);
                if new_path != *entry_path && !new_path.exists() {
                    out.push((entry_path.clone(), new_path));
                }
            }
            out
        })
        .await
        .map_err(|error| crate::error::AppError::Internal(error.to_string()))?
    };

    let mut resolved: std::collections::HashSet<std::path::PathBuf> = std::collections::HashSet::new();
    for (entry_path, new_path) in pending_renames {
        if tokio::fs::rename(&entry_path, &new_path).await.is_ok() {
            renamed += 1;
            resolved.insert(entry_path);
        }
    }

    let mut project_ids: Vec<u64> = Vec::new();
    let mut file_map: Vec<(std::path::PathBuf, String)> = Vec::new();

    for (entry_path, name, is_numeric_pair) in candidates {
        if resolved.contains(&entry_path) || !is_numeric_pair {
            continue;
        }
        let bare = name.strip_suffix(".disabled").unwrap_or(&name);
        if let Some(dollar_pos) = bare.find('_') {
            if let Ok(pid) = bare[..dollar_pos].parse::<u64>() {
                project_ids.push(pid);
                file_map.push((entry_path, name));
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
                    if new_path != *path && !new_path.exists() {
                        if tokio::fs::rename(path, &new_path).await.is_ok() {
                            renamed += 1;
                            tracing::info!(old = %old_name, new = %new_name, "renamed mod file");
                        }
                    }
                }
            }
        }
    }

    if renamed > 0 {
        let _ = crate::db::schema::mods::reconcile_profile(&db, &profile_id, &mods_dir).await;
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
                                for installed in db_mods.iter().filter(|installed| installed.project_id == id) {
                                    let _ = sqlx::query("INSERT INTO mod_icons (key, icon_url) VALUES (?, ?) ON CONFLICT(key) DO UPDATE SET icon_url = excluded.icon_url")
                                        .bind(&installed.file_name).bind(icon).execute(db.pool()).await;
                                }
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

    #[test]
    fn mod_icons_are_bounded_and_source_is_preserved() {
        use base64::Engine;
        let source = image::DynamicImage::new_rgba8(1024, 512);
        let mut encoded = std::io::Cursor::new(Vec::new());
        source.write_to(&mut encoded, image::ImageFormat::Png).unwrap();
        let bytes = encoded.into_inner();
        let original = bytes.clone();
        let uri = mod_icon_data_url(&bytes).unwrap();
        let decoded = base64::engine::general_purpose::STANDARD.decode(uri.split_once(',').unwrap().1).unwrap();
        let thumbnail = image::load_from_memory(&decoded).unwrap();
        assert_eq!((thumbnail.width(), thumbnail.height()), (128, 64));
        assert_eq!(bytes, original);
        assert!(mod_icon_data_url(b"invalid image").is_none());
    }

    #[test]
    fn metadata_icons_use_best_resolution_and_normalized_paths() {
        assert_eq!(metadata_icon_path(&serde_json::json!({ "16": "tiny.png", "128": "./assets/logo.png", "512": "large.png" })), Some("assets/logo.png".into()));
        assert_eq!(metadata_icon_path(&serde_json::json!("/assets\\test\\logo.png")), Some("assets/test/logo.png".into()));
        assert_eq!(metadata_icon_path(&serde_json::json!({ "bad": 1 })), None);
    }

    #[test]
    fn missing_mod_icon_cache_expires_after_file_changes() {
        let path = std::env::temp_dir().join(format!("luxmc-icon-{}.jar", uuid::Uuid::new_v4()));
        std::fs::write(&path, b"not a jar").unwrap();
        assert!(extract_mod_icon_from_jar(&path).is_none());
        let file = std::fs::File::create(&path).unwrap();
        let mut archive = zip::ZipWriter::new(file);
        archive.start_file("icon.png", zip::write::FileOptions::default()).unwrap();
        let mut png = std::io::Cursor::new(Vec::new());
        image::DynamicImage::new_rgba8(16, 16).write_to(&mut png, image::ImageFormat::Png).unwrap();
        std::io::Write::write_all(&mut archive, &png.into_inner()).unwrap();
        archive.finish().unwrap();
        let original = std::fs::read(&path).unwrap();
        assert!(extract_mod_icon_from_jar(&path).is_some());
        assert_eq!(std::fs::read(&path).unwrap(), original);
        std::fs::remove_file(path).unwrap();
    }

    #[test]
    fn pack_images_are_read_from_archives_and_folders_without_changing_files() {
        let root = std::env::temp_dir().join(format!("luxmc-pack-icons-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&root).unwrap();
        let mut png = std::io::Cursor::new(Vec::new());
        image::DynamicImage::new_rgba8(64, 64).write_to(&mut png, image::ImageFormat::Png).unwrap();
        let bytes = png.into_inner();
        std::fs::write(root.join("pack.png"), &bytes).unwrap();
        assert!(extract_mod_icon_from_jar(&root).unwrap().starts_with("data:image/png;base64,"));
        let archive_path = root.join("resources.zip");
        let mut archive = zip::ZipWriter::new(std::fs::File::create(&archive_path).unwrap());
        archive.start_file("pack.png", zip::write::FileOptions::default()).unwrap();
        std::io::Write::write_all(&mut archive, &bytes).unwrap();
        archive.finish().unwrap();
        let original = std::fs::read(&archive_path).unwrap();
        assert!(extract_mod_icon_from_jar(&archive_path).is_some());
        assert_eq!(std::fs::read(&archive_path).unwrap(), original);
        let first = content_icon_key(&archive_path, &std::fs::metadata(&archive_path).unwrap());
        std::fs::write(&archive_path, b"changed archive").unwrap();
        assert_ne!(first, content_icon_key(&archive_path, &std::fs::metadata(&archive_path).unwrap()));
        assert!(extract_mod_icon_from_jar(&archive_path).is_none());
        std::fs::remove_file(&archive_path).unwrap();
        std::fs::remove_file(root.join("pack.png")).unwrap();
        std::fs::remove_dir(root).unwrap();
    }

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

#[cfg(test)]
mod datapack_target_tests {
    #[tokio::test]
    async fn installs_only_inside_the_selected_existing_world() {
        let temporary = std::fs::canonicalize(std::env::temp_dir()).unwrap();
        let root = temporary.join(format!("luxmc-datapack-test-{}", uuid::Uuid::new_v4()));
        let world = root.join("saves").join("Meu mundo");
        std::fs::create_dir_all(&world).unwrap();
        std::fs::write(world.join("level.dat"), b"fixture").unwrap();
        let game_dir = root.to_str().unwrap();
        let target = super::datapack_directory(game_dir, Some("Meu mundo")).await.unwrap();
        assert_eq!(target, std::fs::canonicalize(&world).unwrap().join("datapacks"));
        for invalid in [None, Some("../outro"), Some(".."), Some("missing"), Some("x\\y")] {
            assert!(super::datapack_directory(game_dir, invalid).await.is_err());
        }
        assert_eq!(std::fs::read(world.join("level.dat")).unwrap(), b"fixture");
        assert!(std::fs::canonicalize(&root).unwrap().starts_with(&temporary));
        std::fs::remove_dir_all(root).unwrap();
    }
}
