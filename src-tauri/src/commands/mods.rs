use std::collections::HashSet;

use serde::{Deserialize, Serialize};
use tauri::State;

use crate::core::mods::{ModrinthClient, ModSearchResult, ModUpdate, ModVersion, curseforge};
use crate::db::schema::mods::ModRow;
use crate::state::AppState;
use crate::error::AppResult;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ModInstallRequest {
	pub profile_id: String,
	pub project_id: String,
	pub version_id: String,
}

#[tauri::command]
#[allow(non_snake_case)]
pub async fn mods_search(
	state: State<'_, AppState>,
	query: String,
	mcVersion: String,
	limit: Option<u32>,
) -> AppResult<Vec<ModSearchResult>> {
	let limit = limit.unwrap_or(20);

	let modrinth_client = ModrinthClient::new(state.http.clone());
	let modrinth_results = modrinth_client.search_mods(&query, &mcVersion, limit).await.unwrap_or_default();

	let curseforge_results = curseforge::search_mods(&state.http, &query, &mcVersion, limit).await.unwrap_or_default();

	let mut combined = modrinth_results;
	let existing_slugs: std::collections::HashSet<String> = combined.iter().map(|r| r.slug.clone()).collect();
	for result in curseforge_results {
		if !existing_slugs.contains(&result.slug) {
			combined.push(result);
		}
	}

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
) -> AppResult<Vec<ModSearchResult>> {
	let limit = limit.unwrap_or(20);

	let modrinth_client = ModrinthClient::new(state.http.clone());

	match contentType.as_str() {
		"resourcepack" => {
			let results = modrinth_client.search_typed(&query, &mcVersion, "resourcepack", limit).await.unwrap_or_default();
			Ok(results)
		}
		"shader" => {
			let results = modrinth_client.search_typed(&query, &mcVersion, "shader", limit).await.unwrap_or_default();
			Ok(results)
		}
		_ => {
			let modrinth_results = modrinth_client.search_mods(&query, &mcVersion, limit).await.unwrap_or_default();
			let curseforge_results = curseforge::search_mods(&state.http, &query, &mcVersion, limit).await.unwrap_or_default();

			let mut combined = modrinth_results;
			let existing_slugs: std::collections::HashSet<String> = combined.iter().map(|r| r.slug.clone()).collect();
			for result in curseforge_results {
				if !existing_slugs.contains(&result.slug) {
					combined.push(result);
				}
			}

			combined.sort_by(|a, b| b.downloads.cmp(&a.downloads));
			combined.truncate(limit as usize);
			Ok(combined)
		}
	}
}

#[tauri::command]
#[allow(non_snake_case)]
pub async fn mods_versions(
	state: State<'_, AppState>,
	projectId: String,
	mcVersion: String,
) -> AppResult<Vec<ModVersion>> {
	let modrinth_client = ModrinthClient::new(state.http.clone());
	let modrinth_versions = modrinth_client.get_mod_versions(&projectId, &mcVersion).await.unwrap_or_default();

	let curseforge_versions = curseforge::get_mod_versions(&state.http, &projectId, &mcVersion).await.unwrap_or_default();

	let mut combined = modrinth_versions;
	combined.extend(curseforge_versions);
	Ok(combined)
}

#[tauri::command]
#[allow(non_snake_case)]
pub async fn mods_list(
	_state: State<'_, AppState>,
	profileId: String,
) -> AppResult<Vec<ModRow>> {
	let db = crate::db::shared_db().await?;
	crate::db::schema::mods::list_by_profile(&db, &profileId).await
}

#[tauri::command]
pub async fn mods_install(
	state: State<'_, AppState>,
	request: ModInstallRequest,
) -> AppResult<()> {
	let client = ModrinthClient::new(state.http.clone());
	let versions = client.get_mod_versions(&request.project_id, "").await?;

	let version = versions.iter().find(|v| v.id == request.version_id)
		.ok_or_else(|| crate::error::AppError::NotFound("mod version not found".into()))?;

	let file = version.files.first()
		.ok_or_else(|| crate::error::AppError::NotFound("mod file not found".into()))?;

	let base_dir = directories::ProjectDirs::from("io", "github", "Luxmc")
		.ok_or_else(|| crate::error::AppError::InvalidState("could not determine data dir".into()))?;
	let mods_dir = base_dir.data_dir().join("mods").join(&request.profile_id);
	tokio::fs::create_dir_all(&mods_dir).await?;

	let resp = state.http.get(&file.url).send().await?.error_for_status()?;
	let bytes = resp.bytes().await?;
	let file_path = mods_dir.join(&file.filename);
	tokio::fs::write(&file_path, &bytes).await?;

	let db = crate::db::shared_db().await?;
	let mod_row = ModRow {
		profile_id: request.profile_id,
		project_id: request.project_id,
		version_id: request.version_id,
		file_name: file.filename.clone(),
		sha1: file.sha1.clone(),
		source: "modrinth".into(),
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
	let mut to_install: Vec<(String, String)> = vec![(request.project_id.clone(), request.version_id.clone())];

	let base_dir = directories::ProjectDirs::from("io", "github", "Luxmc")
		.ok_or_else(|| crate::error::AppError::InvalidState("could not determine data dir".into()))?;
	let mods_dir = base_dir.data_dir().join("mods").join(&request.profile_id);
	tokio::fs::create_dir_all(&mods_dir).await?;

	let db = crate::db::shared_db().await?;

	let mut depth = 0;
	while !to_install.is_empty() && depth < 3 {
		let batch = std::mem::take(&mut to_install);
		depth += 1;

		for (project_id, version_id) in batch {
			if installed.contains(&project_id) {
				continue;
			}

			let version = client.get_version_detail(&project_id, &version_id, "").await;
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
					source: "modrinth".into(),
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
			.ok_or_else(|| crate::error::AppError::NotFound(format!("profile {profileId} not found")))?;

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
		if let Ok(Some((latest_id, latest_num, download_url))) =
			client.get_latest_version(&mod_row.project_id, &profile.mc_version, &loaders).await
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
	let client = ModrinthClient::new(state.http.clone());
	let versions = client.get_mod_versions(&projectId, "").await?;

	let version = versions.iter().find(|v| v.id == versionId)
		.ok_or_else(|| crate::error::AppError::NotFound("mod version not found".into()))?;

	let file = version.files.first()
		.ok_or_else(|| crate::error::AppError::NotFound("mod file not found".into()))?;

	let base_dir = directories::ProjectDirs::from("io", "github", "Luxmc")
		.ok_or_else(|| crate::error::AppError::InvalidState("could not determine data dir".into()))?;
	let mods_dir = base_dir.data_dir().join("mods").join(&profileId);
	tokio::fs::create_dir_all(&mods_dir).await?;

	let db = crate::db::shared_db().await?;

	let old = sqlx::query_as::<_, crate::db::schema::mods::ModRow>(
		"SELECT * FROM mods WHERE profile_id = ? AND project_id = ?",
	)
	.bind(&profileId)
	.bind(&projectId)
	.fetch_optional(db.pool())
	.await?;

	if let Some(ref old_mod) = old {
		let old_path = mods_dir.join(&old_mod.file_name);
		let _ = tokio::fs::remove_file(&old_path).await;
	}

	let resp = state.http.get(&file.url).send().await?.error_for_status()?;
	let bytes = resp.bytes().await?;
	let file_path = mods_dir.join(&file.filename);
	tokio::fs::write(&file_path, &bytes).await?;

	let mod_row = crate::db::schema::mods::ModRow {
		profile_id: profileId,
		project_id: projectId,
		version_id: versionId,
		file_name: file.filename.clone(),
		sha1: file.sha1.clone(),
		source: "modrinth".into(),
		installed_at: String::new(),
	};
	crate::db::schema::mods::upsert(&db, &mod_row).await
}
