use serde::{Deserialize, Serialize};
use crate::error::{AppError, AppResult};

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", tag = "action")]
pub enum SocialRequest {
    #[serde(rename = "register")]
    Register { username: String },
    #[serde(rename = "sync", rename_all = "camelCase")]
    Sync { status: String, instance_name: Option<String>, mc_version: Option<String>, loader: Option<String>, server_host: Option<String>, server_port: Option<u16> },
    #[serde(rename = "search")]
    Search { query: String },
    #[serde(rename = "invite", rename_all = "camelCase")]
    Invite { target_id: String },
    #[serde(rename = "accept", rename_all = "camelCase")]
    Accept { target_id: String },
    #[serde(rename = "remove", rename_all = "camelCase")]
    Remove { target_id: String },
}

static IDENTITY_LOCK: tokio::sync::Mutex<()> = tokio::sync::Mutex::const_new(());

async fn identity(account_id: &str) -> AppResult<String> {
    use sha2::Digest;
    use tokio::io::AsyncWriteExt;
    let _guard = IDENTITY_LOCK.lock().await;
    let base = directories::ProjectDirs::from("io", "github", "Luxmc")
        .ok_or_else(|| AppError::InvalidState("Data directory unavailable".into()))?;
    let directory = base.data_dir().join("social");
    tokio::fs::create_dir_all(&directory).await?;
    let name = format!("{:x}", sha2::Sha256::digest(account_id.as_bytes()));
    let path = directory.join(name);
    match tokio::fs::read_to_string(&path).await {
        Ok(token) if token.len() == 64 && token.bytes().all(|b| b.is_ascii_hexdigit()) => return Ok(token),
        Ok(_) => return Err(AppError::InvalidState("Invalid social identity".into())),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
        Err(error) => return Err(error.into()),
    }
    let token = format!("{}{}", uuid::Uuid::new_v4().simple(), uuid::Uuid::new_v4().simple());
    let mut options = tokio::fs::OpenOptions::new();
    options.write(true).create_new(true);
    #[cfg(unix)]
    options.mode(0o600);
    let mut file = options.open(path).await?;
    file.write_all(token.as_bytes()).await?;
    file.flush().await?;
    Ok(token)
}

pub async fn social_request_core(account_id: String, request: SocialRequest) -> AppResult<serde_json::Value> {
    if account_id.is_empty() || account_id.len() > 128 { return Err(AppError::InvalidInput("Invalid account".into())); }
    let token = if account_id.starts_with("luxmc:") { super::lux_account::token(&account_id).await? } else { identity(&account_id).await? };
    let mut body = serde_json::to_value(request)?;
    if account_id.starts_with("luxmc:") { body["registeredAccount"] = serde_json::Value::Bool(true); }
    let action = body.get("action").and_then(|value| value.as_str()).unwrap_or_default();
    let client = reqwest::Client::builder()
        .redirect(reqwest::redirect::Policy::none())
        .timeout(std::time::Duration::from_secs(15)).build()?;
    let response = client.post(format!("{}/api/social/{action}", super::lux_account::portal_base()))
        .bearer_auth(token).json(&body).send().await?;
    let status = response.status();
    let value: serde_json::Value = response.json().await
        .map_err(|_| AppError::InvalidState("O serviço social ainda não está disponível no portal.".into()))?;
    if !status.is_success() {
        return Err(AppError::InvalidState(value.get("error").and_then(|v| v.as_str()).unwrap_or("Social request failed").to_owned()));
    }
    Ok(value)
}

#[tauri::command]
pub async fn social_request(account_id: String, request: SocialRequest) -> AppResult<serde_json::Value> {
    social_request_core(account_id, request).await
}
