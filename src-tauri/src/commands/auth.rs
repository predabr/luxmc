use crate::core::auth::microsoft::PendingAuth;
use crate::core::auth::oauth_server;
use crate::core::auth::AuthAccount;
use crate::db::models::AccountRow;
use crate::error::{AppError, AppResult};
use crate::state::AppState;
use tauri::Emitter;
use tauri::State;
use uuid::Uuid;

pub async fn get_configured_client_id() -> String {
    if let Ok(conn) = crate::db::shared_db().await {
        use sqlx::Row;
        if let Ok(Some(row)) = sqlx::query("SELECT value FROM app_settings WHERE key = 'app'")
            .fetch_optional(conn.pool())
            .await
        {
            if let Ok(raw) = row.try_get::<String, _>("value") {
                if let Ok(val) = serde_json::from_str::<serde_json::Value>(&raw) {
                    if let Some(cid) = val.get("customMicrosoftClientId").and_then(|v| v.as_str()) {
                        let trimmed = cid.trim();
                        if !trimmed.is_empty() && trimmed != "00000000-0000-0000-0000-000000000000" {
                            return trimmed.to_string();
                        }
                    }
                }
            }
        }
    }
    crate::core::auth::default_client_id()
}

#[tauri::command]
pub async fn auth_get_client_id() -> String {
    let cid = get_configured_client_id().await;
    if cid == "00000000-0000-0000-0000-000000000000" {
        "".to_string()
    } else {
        cid
    }
}

#[tauri::command]
pub async fn auth_get_tenant_id() -> String {
    crate::core::auth::default_tenant_id()
}

#[tauri::command]
pub async fn auth_set_client_id(client_id: String) -> AppResult<()> {
    let trimmed = client_id.trim();
    let conn = crate::db::shared_db().await?;
    use sqlx::Row;

    let current_val = if let Ok(Some(row)) = sqlx::query("SELECT value FROM app_settings WHERE key = 'app'")
        .fetch_optional(conn.pool())
        .await
    {
        row.try_get::<String, _>("value").unwrap_or_default()
    } else {
        String::new()
    };

    let mut map: serde_json::Map<String, serde_json::Value> = serde_json::from_str(&current_val).unwrap_or_default();
    if trimmed.is_empty() {
        map.remove("customMicrosoftClientId");
    } else {
        map.insert("customMicrosoftClientId".into(), serde_json::Value::String(trimmed.to_string()));
    }

    let serialized = serde_json::to_string(&map).map_err(|e| AppError::Internal(e.to_string()))?;
    sqlx::query("INSERT INTO app_settings (key, value) VALUES ('app', ?) ON CONFLICT(key) DO UPDATE SET value = excluded.value")
        .bind(serialized)
        .execute(conn.pool())
        .await
        .map_err(|e| AppError::Internal(format!("Failed to save client_id: {e}")))?;

    Ok(())
}

pub async fn auth_begin_core(state: &AppState) -> AppResult<PendingAuth> {
    let client_id = get_configured_client_id().await;
    Ok(state.auth.begin_with_client_id(Some(&client_id)))
}

#[tauri::command]
pub async fn auth_begin(state: State<'_, AppState>) -> AppResult<PendingAuth> {
    auth_begin_core(&state).await
}

#[tauri::command]
pub async fn auth_login(
    state: State<'_, AppState>,
    app: tauri::AppHandle,
) -> AppResult<AuthAccount> {
    let client_id = get_configured_client_id().await;
    if client_id == "00000000-0000-0000-0000-000000000000" {
        return Err(AppError::InvalidState(
            "client_id_required".into(),
        ));
    }
    let pending = state.auth.begin_with_client_id(Some(&client_id));
    let listener = oauth_server::start_callback_server().await.map_err(|e| {
        AppError::Internal(format!(
            "Failed to start callback server on port 8453: {}",
            e
        ))
    })?;

    app.emit("launcher-log", "Opening browser for Microsoft login...")
        .ok();
    crate::commands::instances::open_url_safe(&pending.url).map_err(|e| {
        let msg = format!(
            "Failed to open browser: {}. Please open this URL manually:\n{}",
            e, pending.url
        );
        AppError::Internal(msg)
    })?;

    app.emit(
        "launcher-log",
        "Waiting for login callback on http://localhost:8453/callback ...",
    )
    .ok();
    let callback = oauth_server::wait_for_callback(&listener, 300)
        .await
        .map_err(|e| AppError::Internal(e))?;

    app.emit(
        "launcher-log",
        "Login callback received, exchanging code...",
    )
    .ok();
    let account = state
        .auth
        .login_with_code_and_client_id(&callback.code, &pending.verifier, Some(&client_id))
        .await?;
    save_account(&state, &account).await?;
    app.emit(
        "launcher-log",
        format!("Login successful: {}", account.username),
    )
    .ok();
    Ok(account)
}

pub async fn auth_complete_core(
    state: &AppState,
    code: String,
    state_token: String,
    verifier: String,
) -> AppResult<AuthAccount> {
    let _ = &state_token;
    let client_id = get_configured_client_id().await;
    let account = state
        .auth
        .login_with_code_and_client_id(&code, &verifier, Some(&client_id))
        .await?;
    save_account(state, &account).await?;
    Ok(account)
}

#[tauri::command]
pub async fn auth_complete(
    state: State<'_, AppState>,
    code: String,
    state_token: String,
    verifier: String,
) -> AppResult<AuthAccount> {
    auth_complete_core(&state, code, state_token, verifier).await
}

pub async fn auth_refresh_core(
    state: &AppState,
    refresh_token: String,
) -> AppResult<AuthAccount> {
    let account = state.auth.refresh_account(&refresh_token).await?;
    save_account(state, &account).await?;
    Ok(account)
}

#[tauri::command]
#[allow(non_snake_case)]
pub async fn auth_refresh(
    state: State<'_, AppState>,
    refreshToken: String,
) -> AppResult<AuthAccount> {
    auth_refresh_core(&state, refreshToken).await
}

async fn set_active_account_id(account_id: &str) -> AppResult<()> {
    let conn = crate::db::shared_db().await?;
    use sqlx::Row;

    let current_val = if let Ok(Some(row)) = sqlx::query("SELECT value FROM app_settings WHERE key = 'app'")
        .fetch_optional(conn.pool())
        .await
    {
        let raw: String = row.try_get("value").unwrap_or_default();
        serde_json::from_str::<serde_json::Value>(&raw).unwrap_or(serde_json::json!({}))
    } else {
        serde_json::json!({})
    };

    let mut obj = match current_val {
        serde_json::Value::Object(m) => m,
        _ => serde_json::Map::new(),
    };

    obj.insert("activeAccountId".into(), serde_json::Value::String(account_id.to_string()));
    let new_raw = serde_json::to_string(&serde_json::Value::Object(obj)).unwrap_or_default();

    let _ = sqlx::query("INSERT OR REPLACE INTO app_settings (key, value) VALUES ('app', ?)")
        .bind(new_raw)
        .execute(conn.pool())
        .await;

    Ok(())
}

#[tauri::command]
pub async fn auth_accounts() -> AppResult<Vec<AccountRow>> {
    let db = crate::db::shared_db().await?;
    crate::db::schema::accounts::list(&db).await
}

#[tauri::command]
pub async fn auth_switch_account(
    uuid: String,
) -> AppResult<AuthAccount> {
    let db = crate::db::shared_db().await?;
    let row = crate::db::schema::accounts::get_by_uuid(&db, &uuid)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("Account with uuid {} not found", uuid)))?;
    let _ = crate::db::schema::accounts::touch_account(&db, &row.id).await;
    let _ = set_active_account_id(&row.id).await;
    Ok(AuthAccount {
        id: row.id,
        username: row.username,
        uuid: row.uuid,
        access_token: row.access_token.unwrap_or_default(),
        refresh_token: row.refresh_token,
        expires_at: row.expires_at.map(|dt| dt.timestamp()).unwrap_or(0),
        skin_url: row.skin_url,
        skin_variant: row.skin_variant,
        cape_url: row.cape_url,
    })
}

#[tauri::command]
pub async fn auth_remove(uuid: String) -> AppResult<()> {
    let db = crate::db::shared_db().await?;
    if let Ok(Some(_row)) = crate::db::schema::accounts::get_by_uuid(&db, &uuid).await {
        crate::db::schema::accounts::delete(&db, &uuid).await?;
        let remaining = crate::db::schema::accounts::list(&db).await?;
        if let Some(next_acc) = remaining.into_iter().next() {
            let _ = set_active_account_id(&next_acc.id).await;
        }
    } else {
        let _ = crate::db::schema::accounts::delete(&db, &uuid).await;
    }
    Ok(())
}

pub async fn auth_dev_login_core(state: &AppState) -> AppResult<AuthAccount> {
    let account = AuthAccount {
        id: uuid::Uuid::new_v4().to_string(),
        username: "DevPlayer".into(),
        uuid: uuid::Uuid::new_v4().to_string(),
        access_token: "dev-offline-token".into(),
        refresh_token: "".into(),
        expires_at: 0,
        skin_url: None,
        skin_variant: None,
        cape_url: None,
    };
    save_account(state, &account).await?;
    Ok(account)
}

#[tauri::command]
pub async fn auth_dev_login(state: State<'_, AppState>) -> AppResult<AuthAccount> {
    if std::env::var("LUXMC_DEV_MODE").unwrap_or_default() != "1" {
        return Err(crate::error::AppError::InvalidState(
            "dev login disabled: set LUXMC_DEV_MODE=1".into(),
        ));
    }
    let account = AuthAccount {
        id: "dev-00000000-0000-0000-0000-000000000001".into(),
        username: "DevPlayer".into(),
        uuid: "00000000-0000-0000-0000-000000000001".into(),
        access_token: "dev-access-token".into(),
        refresh_token: "dev-refresh-token".into(),
        expires_at: chrono::Utc::now().timestamp() + 86400,
        skin_url: Some("https://minotar.net/skin/MHF_Steve".into()),
        skin_variant: Some("classic".into()),
        cape_url: None,
    };
    save_account(&state, &account).await?;
    Ok(account)
}

#[tauri::command]
pub async fn auth_offline_login(username: String) -> AppResult<AuthAccount> {
    if username.is_empty() || username.len() < 3 || username.len() > 16 {
        return Err(AppError::InvalidInput(
            "Username must be between 3 and 16 characters".into(),
        ));
    }
    if !username
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '_')
    {
        return Err(AppError::InvalidInput(
            "Username must be alphanumeric or underscore".into(),
        ));
    }
    let uuid = Uuid::new_v5(
        &Uuid::NAMESPACE_URL,
        format!("offline:{}", username).as_bytes(),
    );
    let default_skin = format!("https://minotar.net/skin/{}", username);
    let account = AuthAccount {
        id: uuid.to_string(),
        username,
        uuid: uuid.to_string(),
        access_token: String::new(),
        refresh_token: String::new(),
        expires_at: chrono::Utc::now().timestamp() + 86400 * 365,
        skin_url: Some(default_skin),
        skin_variant: Some("classic".into()),
        cape_url: None,
    };
    let db = crate::db::shared_db().await?;
    let row = AccountRow {
        id: account.id.clone(),
        username: account.username.clone(),
        uuid: account.uuid.clone(),
        refresh_token: account.refresh_token.clone(),
        access_token: Some(account.access_token.clone()),
        expires_at: Some(
            chrono::DateTime::from_timestamp(account.expires_at, 0).unwrap_or_default(),
        ),
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
        skin_url: account.skin_url.clone(),
        skin_variant: account.skin_variant.clone(),
        cape_url: account.cape_url.clone(),
    };
    crate::db::schema::accounts::upsert(&db, &row).await?;
    let _ = set_active_account_id(&account.id).await;
    Ok(account)
}

async fn save_account(_state: &AppState, account: &AuthAccount) -> AppResult<()> {
    let db = crate::db::shared_db().await?;
    let row = AccountRow {
        id: account.id.clone(),
        username: account.username.clone(),
        uuid: account.uuid.clone(),
        refresh_token: account.refresh_token.clone(),
        access_token: Some(account.access_token.clone()),
        expires_at: Some(
            chrono::DateTime::from_timestamp(account.expires_at, 0).unwrap_or_default(),
        ),
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
        skin_url: account.skin_url.clone(),
        skin_variant: account.skin_variant.clone(),
        cape_url: account.cape_url.clone(),
    };
    crate::db::schema::accounts::upsert(&db, &row).await?;
    let _ = set_active_account_id(&account.id).await;
    Ok(())
}

#[tauri::command]
pub async fn auth_change_skin(
    uuid: String,
    variant: String,
    skin_url: String,
) -> AppResult<()> {
    let db = crate::db::shared_db().await?;
    let account = sqlx::query_as::<_, AccountRow>("SELECT * FROM accounts WHERE uuid = ? OR id = ?")
        .bind(&uuid)
        .bind(&uuid)
        .fetch_optional(db.pool())
        .await?
        .ok_or_else(|| AppError::NotFound(format!("Account not found: {}", uuid)))?;

    let norm_variant = if variant == "slim" || variant == "alex" { "slim" } else { "classic" };

    let mut updated = account.clone();
    updated.skin_url = Some(skin_url.clone());
    updated.skin_variant = Some(norm_variant.to_string());
    updated.updated_at = chrono::Utc::now();
    crate::db::schema::accounts::upsert(&db, &updated).await?;

    let token_str = account.access_token.as_deref().unwrap_or("");
    let is_real_msa = !token_str.is_empty()
        && !token_str.starts_with("offline")
        && !token_str.starts_with("token_")
        && !token_str.starts_with("dev-")
        && token_str.len() > 100;

    if is_real_msa {
        let client = reqwest::Client::new();
        if skin_url.starts_with("http://") || skin_url.starts_with("https://") {
            let body = serde_json::json!({
                "variant": norm_variant,
                "url": skin_url
            });

            let res = client
                .post("https://api.minecraftservices.com/minecraft/profile/skins")
                .bearer_auth(token_str)
                .json(&body)
                .send()
                .await;

            if let Ok(resp) = res {
                if !resp.status().is_success() {
                    let status = resp.status();
                    let err_text = resp.text().await.unwrap_or_default();
                    tracing::warn!(status = %status, err = %err_text, "Failed to sync skin URL with Mojang API");
                }
            }
        } else {
            let skin_bytes: Vec<u8> = if skin_url.starts_with("data:image/") {
                if let Some(comma_pos) = skin_url.find(',') {
                    use base64::Engine;
                    base64::engine::general_purpose::STANDARD
                        .decode(&skin_url[comma_pos + 1..])
                        .unwrap_or_default()
                } else {
                    Vec::new()
                }
            } else {
                tokio::fs::read(&skin_url).await.unwrap_or_default()
            };

            if !skin_bytes.is_empty() && skin_bytes.len() >= 8 && &skin_bytes[0..8] == b"\x89PNG\r\n\x1a\n" {
                let part = reqwest::multipart::Part::bytes(skin_bytes)
                    .file_name("skin.png")
                    .mime_str("image/png")
                    .unwrap_or_else(|_| reqwest::multipart::Part::bytes(Vec::new()));

                let form = reqwest::multipart::Form::new()
                    .text("variant", norm_variant.to_string())
                    .part("file", part);

                let res = client
                    .post("https://api.minecraftservices.com/minecraft/profile/skins")
                    .bearer_auth(token_str)
                    .multipart(form)
                    .send()
                    .await;

                if let Ok(resp) = res {
                    if !resp.status().is_success() {
                        let status = resp.status();
                        let err_text = resp.text().await.unwrap_or_default();
                        tracing::warn!(status = %status, err = %err_text, "Failed to sync multipart skin with Mojang API");
                    }
                }
            }
        }
    }

    Ok(())
}

