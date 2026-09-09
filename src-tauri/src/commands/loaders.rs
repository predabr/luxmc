use serde::{Deserialize, Serialize};
use tauri::State;

use crate::core::loaders::{self, LoaderVersion};
use crate::error::AppResult;
use crate::state::AppState;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LoaderVersionsResponse {
    pub versions: Vec<LoaderVersion>,
}

#[tauri::command]
#[allow(non_snake_case)]
pub async fn loaders_versions(
    state: State<'_, AppState>,
    loader: String,
    mcVersion: String,
) -> AppResult<LoaderVersionsResponse> {
    let versions = loaders::fetch_loader_versions(&state.http, &loader, &mcVersion).await?;
    Ok(LoaderVersionsResponse { versions })
}
