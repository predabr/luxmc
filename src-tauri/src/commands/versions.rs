use serde::{Deserialize, Serialize};
use tauri::Emitter;
use tauri::State;

use crate::core::downloader::DownloadManager;
use crate::core::java::JavaRuntimeManager;
use crate::core::minecraft::{self, VersionDetail, VersionSummary};
use crate::state::AppState;
use crate::error::AppResult;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VersionListResponse {
	pub versions: Vec<VersionSummary>,
	pub latest_release: String,
	pub latest_snapshot: String,
}

#[tauri::command]
pub async fn versions_list(_state: State<'_, AppState>) -> AppResult<VersionListResponse> {
	let db = crate::db::shared_db().await?;
	let manifest = match minecraft::fetch_version_manifest(&_state.http).await {
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
				latest_release: versions.iter().find(|v| v.version_type == "release").map(|v| v.id.clone()).unwrap_or_default(),
				latest_snapshot: versions.iter().find(|v| v.version_type == "snapshot").map(|v| v.id.clone()).unwrap_or_default(),
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
pub async fn versions_detail(
	state: State<'_, AppState>,
	id: String,
) -> AppResult<VersionDetail> {
	let db = crate::db::shared_db().await?;
	let row = crate::db::schema::versions::get(&db, &id)
		.await?
		.ok_or_else(|| crate::error::AppError::NotFound(format!("version {id} not found")))?;
	minecraft::fetch_version_detail(&state.http, &row.url).await
}

#[tauri::command]
pub async fn versions_download(
	state: State<'_, AppState>,
	app: tauri::AppHandle,
	id: String,
) -> AppResult<()> {
	app.emit("launcher-log", format!("Preparing to download version {}", id)).ok();

	let db = crate::db::shared_db().await?;
	let row = crate::db::schema::versions::get(&db, &id)
		.await?
		.ok_or_else(|| crate::error::AppError::NotFound(format!("version {id} not found in database. Try refreshing versions.")))?;

	app.emit("launcher-log", format!("Fetching version manifest for {}...", id)).ok();
	let detail = minecraft::fetch_version_detail(&state.http, &row.url).await?;

	let base_dir = directories::ProjectDirs::from("io", "github", "Luxmc")
		.ok_or_else(|| crate::error::AppError::InvalidState("could not determine data dir".into()))?;
	let data_dir = base_dir.data_dir().to_path_buf();

	let major = detail.java_major_version();
	app.emit("launcher-log", format!("Java {} required for version {}", major, id)).ok();

	let java = JavaRuntimeManager::new(state.http.clone(), data_dir.clone())
		.with_app(app.clone());
	let java_result = java.ensure_java(major).await;
	match &java_result {
		Ok(path) => {
			app.emit("launcher-log", format!("Java ready: {}", path.display())).ok();
		}
		Err(e) => {
			app.emit("launcher-log", format!("Java warning: {}", e)).ok();
		}
	}

	let downloader = DownloadManager::new(state.http.clone(), data_dir)
		.with_app(app.clone());
	downloader.download_version(&detail).await
}

#[tauri::command]
pub async fn versions_check_installed(
	_state: State<'_, AppState>,
	id: String,
) -> AppResult<bool> {
	let base_dir = directories::ProjectDirs::from("io", "github", "Luxmc")
		.ok_or_else(|| crate::error::AppError::InvalidState("could not determine data dir".into()))?;
	let client_jar = base_dir
		.data_dir()
		.join("versions")
		.join(&id)
		.join(format!("{}.jar", id));
	Ok(client_jar.exists())
}
