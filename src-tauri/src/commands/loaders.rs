use serde::{Deserialize, Serialize};
use tauri::State;

use crate::core::loaders::{self, LoaderVersion};
use crate::error::AppResult;
use crate::state::AppState;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LoaderVersionsResponse {
    pub versions: Vec<LoaderVersion>,
    pub supported_game_versions: Option<Vec<String>>,
}

#[tauri::command]
#[allow(non_snake_case)]
pub async fn loaders_versions(
    state: State<'_, AppState>,
    loader: String,
    mcVersion: String,
) -> AppResult<LoaderVersionsResponse> {
    loader_versions_core(&state, &loader, &mcVersion).await
}

pub async fn loader_versions_core(state: &AppState, loader: &str, mc_version: &str) -> AppResult<LoaderVersionsResponse> {
    let supported_game_versions = loaders::supported_game_versions(&state.http, loader).await?;
    let versions = if supported_game_versions.as_ref().is_some_and(|versions| !versions.iter().any(|version| version == mc_version)) { Vec::new() } else { loaders::fetch_loader_versions(&state.http, loader, mc_version).await? };
    Ok(LoaderVersionsResponse { versions, supported_game_versions })
}
