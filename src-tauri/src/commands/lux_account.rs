use crate::core::auth::AuthAccount;
use crate::db::models::AccountRow;
use crate::error::{AppError, AppResult};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tokio::io::AsyncWriteExt;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Preferences {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub theme: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub accent_theme: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub language: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub animations: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PortalAccount {
    pub id: String,
    pub username: String,
    pub social_id: String,
    pub preferences: Preferences,
    pub revision: i64,
}

#[derive(Deserialize)]
struct PortalResponse {
    account: PortalAccount,
    token: Option<String>,
}

pub(crate) fn portal_base() -> String {
    #[cfg(debug_assertions)]
    if let Ok(value) = std::env::var("LUXMC_TEST_PORTAL_URL") {
        if let Ok(url) = reqwest::Url::parse(&value) {
            if url.scheme() == "http" && matches!(url.host_str(), Some("127.0.0.1") | Some("localhost")) && url.username().is_empty() && url.password().is_none() {
                return value.trim_end_matches('/').to_string();
            }
        }
    }
    "https://luxmc-r92.pages.dev".into()
}

fn session_path(id: &str) -> AppResult<PathBuf> {
    let id = id.strip_prefix("luxmc:").ok_or_else(|| AppError::InvalidInput("Esta conta não está conectada ao Luxmc.".into()))?;
    let id = uuid::Uuid::parse_str(id).map_err(|_| AppError::InvalidInput("Conta inválida.".into()))?;
    let directory = directories::ProjectDirs::from("io", "github", "Luxmc").ok_or_else(|| AppError::InvalidState("Data directory unavailable".into()))?;
    Ok(directory.data_dir().join("lux-accounts").join(id.to_string()))
}

pub(crate) async fn token(id: &str) -> AppResult<String> {
    let value = tokio::fs::read_to_string(session_path(id)?).await.map_err(|_| AppError::InvalidState("Entre novamente com sua senha do Luxmc.".into()))?;
    if value.len() != 64 || !value.bytes().all(|b| b.is_ascii_hexdigit()) { return Err(AppError::InvalidState("Sessão inválida. Entre novamente.".into())); }
    Ok(value)
}

async fn request(action: &str, body: serde_json::Value, bearer: Option<&str>) -> AppResult<serde_json::Value> {
    let client = reqwest::Client::builder().redirect(reqwest::redirect::Policy::none()).timeout(std::time::Duration::from_secs(20)).build()?;
    let mut req = client.post(format!("{}/api/account/{action}", portal_base())).json(&body);
    if let Some(token) = bearer { req = req.bearer_auth(token); }
    let response = req.send().await?;
    let status = response.status();
    let value: serde_json::Value = response.json().await.map_err(|_| AppError::InvalidState("O portal de contas ainda não está disponível.".into()))?;
    if !status.is_success() { return Err(AppError::InvalidState(value.get("error").and_then(|v| v.as_str()).unwrap_or("Não foi possível conectar à conta.").into())); }
    Ok(value)
}

#[tauri::command]
pub async fn lux_account_login(username: String, password: String) -> AppResult<AuthAccount> {
    if username.len() < 3 || username.len() > 16 || !username.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'_') || password.len() < 8 || password.len() > 512 {
        return Err(AppError::InvalidInput("Confira o nickname e a senha da sua conta Luxmc.".into()));
    }
    let response: PortalResponse = serde_json::from_value(request("login", serde_json::json!({"username":username,"password":password,"client":"launcher"}), None).await?)?;
    let id = format!("luxmc:{}", response.account.id);
    let session = response.token.ok_or_else(|| AppError::InvalidState("Sessão não recebida.".into()))?;
    if session.len() != 64 || !session.bytes().all(|b| b.is_ascii_hexdigit()) { return Err(AppError::InvalidState("Sessão inválida.".into())); }
    let path = session_path(&id)?;
    tokio::fs::create_dir_all(path.parent().unwrap()).await?;
    let temporary = path.with_extension(uuid::Uuid::new_v4().simple().to_string());
    let mut options = tokio::fs::OpenOptions::new();
    options.write(true).create_new(true);
    #[cfg(unix)]
    options.mode(0o600);
    let mut file = options.open(&temporary).await?;
    file.write_all(session.as_bytes()).await?;
    file.flush().await?;
    drop(file);
    tokio::fs::rename(temporary, path).await?;
    let now = chrono::Utc::now();
    let skin = format!("https://minotar.net/skin/{}", response.account.username);
    let row = AccountRow {
        id: id.clone(), uuid: response.account.id.clone(), username: response.account.username.clone(), refresh_token: String::new(), access_token: Some(String::new()),
        expires_at: None, created_at: now, updated_at: now, skin_url: Some(skin.clone()), skin_variant: Some("classic".into()), cape_url: None,
    };
    let db = crate::db::shared_db().await?;
    crate::db::schema::accounts::upsert(&db, &row).await?;
    super::auth::set_active_account_id(&id).await?;
    Ok(AuthAccount { id, uuid: response.account.id, username: response.account.username, access_token: String::new(), refresh_token: String::new(), expires_at: 0, skin_url: Some(skin), skin_variant: Some("classic".into()), cape_url: None })
}

#[tauri::command]
pub async fn lux_account_sync(account_id: String, preferences: Option<Preferences>, revision: Option<i64>) -> AppResult<PortalAccount> {
    let token = token(&account_id).await?;
    let body = if let Some(prefs) = preferences { serde_json::json!({"preferences":prefs,"revision":revision}) } else { serde_json::json!({}) };
    let response: PortalResponse = serde_json::from_value(request("sync", body, Some(&token)).await?)?;
    Ok(response.account)
}

#[tauri::command]
pub async fn lux_account_logout(account_id: String) -> AppResult<()> {
    let session = token(&account_id).await?;
    request("logout", serde_json::json!({}), Some(&session)).await?;
    tokio::fs::remove_file(session_path(&account_id)?).await?;
    Ok(())
}
