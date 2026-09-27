use crate::{error::{AppError, AppResult}, state::AppState};

pub async fn minecraft_news_core(state: &AppState) -> AppResult<serde_json::Value> {
    let response = state.http.get("https://launchercontent.mojang.com/v2/news.json")
        .timeout(std::time::Duration::from_secs(15)).send().await?.error_for_status()?;
    if response.content_length().unwrap_or(0) > 2 * 1024 * 1024 {
        return Err(AppError::InvalidInput("Feed de notícias excedeu o limite".into()));
    }
    let mut response = response;
    let mut bytes = Vec::new();
    while let Some(chunk) = response.chunk().await? {
        if bytes.len() + chunk.len() > 2 * 1024 * 1024 {
            return Err(AppError::InvalidInput("Feed de notícias excedeu o limite".into()));
        }
        bytes.extend_from_slice(&chunk);
    }
    Ok(serde_json::from_slice(&bytes)?)
}

#[tauri::command]
pub async fn minecraft_news(state: tauri::State<'_, AppState>) -> AppResult<serde_json::Value> {
    minecraft_news_core(state.inner()).await
}
