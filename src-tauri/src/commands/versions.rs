use serde::{Deserialize, Serialize};
use tauri::Emitter;
use tauri::State;

use crate::core::downloader::DownloadManager;
use crate::core::java::JavaRuntimeManager;
use crate::core::minecraft::{self, VersionDetail, VersionSummary};
use crate::error::AppResult;
use crate::state::AppState;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VersionListResponse {
    pub versions: Vec<VersionSummary>,
    pub latest_release: String,
    pub latest_snapshot: String,
}

pub async fn versions_list_core(http: &reqwest::Client) -> AppResult<VersionListResponse> {
    let db = crate::db::shared_db().await?;
    let manifest = match minecraft::fetch_version_manifest(http).await {
        Ok(manifest) => manifest,
        Err(error) => {
            let cached = crate::db::schema::versions::list(&db).await?;
            if cached.is_empty() {
                return Err(error);
            }
            let versions: Vec<VersionSummary> = cached
                .iter()
                .map(|v| VersionSummary {
                    id: v.id.clone(),
                    version_type: v.version_type.clone(),
                    url: v.url.clone(),
                    release_time: v.release_time.clone(),
                })
                .collect();
            return Ok(VersionListResponse {
                latest_release: versions
                    .iter()
                    .find(|v| v.version_type == "release")
                    .map(|v| v.id.clone())
                    .unwrap_or_default(),
                latest_snapshot: versions
                    .iter()
                    .find(|v| v.version_type == "snapshot")
                    .map(|v| v.id.clone())
                    .unwrap_or_default(),
                versions,
            });
        }
    };
    let rows: Vec<crate::db::schema::versions::VersionRow> = manifest
        .versions
        .iter()
        .map(|v| crate::db::schema::versions::VersionRow {
            id: v.id.clone(),
            version_type: v.version_type.clone(),
            url: v.url.clone(),
            time: v.release_time.clone(),
            release_time: v.release_time.clone(),
            fetched_at: String::new(),
        })
        .collect();
    crate::db::schema::versions::upsert_many(&db, &rows).await?;

    Ok(VersionListResponse {
        versions: manifest.versions,
        latest_release: manifest.latest.release,
        latest_snapshot: manifest.latest.snapshot,
    })
}

#[tauri::command]
pub async fn versions_list(state: State<'_, AppState>) -> AppResult<VersionListResponse> {
    versions_list_core(&state.http).await
}

async fn resolve_version_row(
    http: &reqwest::Client,
    db: &crate::db::Db,
    id: &str,
) -> AppResult<crate::db::schema::versions::VersionRow> {
    let clean = id.trim().trim_matches('\'').trim_matches('"');
    if let Some(row) = crate::db::schema::versions::get(db, clean).await? {
        return Ok(row);
    }
    let manifest = minecraft::fetch_version_manifest(http).await?;
    if let Some(target) = manifest.versions.iter().find(|v| v.id == clean) {
        let now = chrono::Utc::now().to_rfc3339();
        let row = crate::db::schema::versions::VersionRow {
            id: target.id.clone(),
            version_type: target.version_type.clone(),
            url: target.url.clone(),
            time: target.release_time.clone(),
            release_time: target.release_time.clone(),
            fetched_at: now,
        };
        crate::db::schema::versions::upsert(db, &row).await?;
        Ok(row)
    } else {
        Err(crate::error::AppError::NotFound(format!(
            "Versão {clean} não encontrada no banco de dados nem no manifesto oficial."
        )))
    }
}

pub async fn versions_detail_core(http: &reqwest::Client, id: &str) -> AppResult<VersionDetail> {
    let db = crate::db::shared_db().await?;
    let row = resolve_version_row(http, &db, id).await?;
    minecraft::fetch_version_detail(http, &row.url).await
}

#[tauri::command]
pub async fn versions_detail(state: State<'_, AppState>, id: String) -> AppResult<VersionDetail> {
    versions_detail_core(&state.http, &id).await
}

pub async fn versions_download_core(
    http: &reqwest::Client,
    id: &str,
) -> AppResult<()> {
    let db = crate::db::shared_db().await?;
    let row = resolve_version_row(http, &db, id).await?;

    let detail = minecraft::fetch_version_detail(http, &row.url).await?;

    let base_dir = directories::ProjectDirs::from("io", "github", "Luxmc").ok_or_else(|| {
        crate::error::AppError::InvalidState("could not determine data dir".into())
    })?;
    let data_dir = base_dir.data_dir().to_path_buf();

    let major = detail.java_major_version();
    let java = JavaRuntimeManager::new(http.clone(), data_dir.clone());
    let _ = java.ensure_java(major).await;

    let downloader = DownloadManager::new(http.clone(), data_dir);
    downloader.download_version(&detail).await
}

#[tauri::command]
pub async fn versions_download(
    state: State<'_, AppState>,
    app: tauri::AppHandle,
    id: String,
) -> AppResult<()> {
    let clean = id.trim().trim_matches('\'').trim_matches('"');
    app.emit(
        "launcher-log",
        format!("Preparing to download version {}", clean),
    )
    .ok();

    let db = crate::db::shared_db().await?;
    let row = resolve_version_row(&state.http, &db, clean).await?;

    app.emit(
        "launcher-log",
        format!("Fetching version manifest for {}...", clean),
    )
    .ok();
    let detail = minecraft::fetch_version_detail(&state.http, &row.url).await?;

    let base_dir = directories::ProjectDirs::from("io", "github", "Luxmc").ok_or_else(|| {
        crate::error::AppError::InvalidState("could not determine data dir".into())
    })?;
    let data_dir = base_dir.data_dir().to_path_buf();

    let major = detail.java_major_version();
    app.emit(
        "launcher-log",
        format!("Java {} required for version {}", major, clean),
    )
    .ok();

    let java = JavaRuntimeManager::new(state.http.clone(), data_dir.clone()).with_app(app.clone());
    let java_result = java.ensure_java(major).await;
    match &java_result {
        Ok(path) => {
            app.emit("launcher-log", format!("Java ready: {}", path.display()))
                .ok();
        }
        Err(e) => {
            app.emit("launcher-log", format!("Java warning: {}", e))
                .ok();
        }
    }

    let downloader = DownloadManager::new(state.http.clone(), data_dir).with_app(app.clone());
    downloader.download_version(&detail).await
}

pub async fn versions_check_installed_core(id: &str) -> AppResult<bool> {
    let clean = id.trim().trim_matches('\'').trim_matches('"');
    let base_dir = directories::ProjectDirs::from("io", "github", "Luxmc").ok_or_else(|| {
        crate::error::AppError::InvalidState("could not determine data dir".into())
    })?;
    let client_jar = base_dir
        .data_dir()
        .join("versions")
        .join(clean)
        .join(format!("{}.jar", clean));
    Ok(client_jar.exists())
}

#[tauri::command]
pub async fn versions_check_installed(id: String) -> AppResult<bool> {
    versions_check_installed_core(&id).await
}
