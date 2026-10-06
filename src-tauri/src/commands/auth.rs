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

    if callback.state != pending.state {
        return Err(AppError::InvalidState(
            "O estado do retorno de login não corresponde à solicitação iniciada. Tente novamente.".into(),
        ));
    }

    app.emit(
        "launcher-log",
        "Login callback received, exchanging code...",
    )
    .ok();
    let account = state
        .auth
        .login_with_code_and_client_id(&callback.code, &pending.verifier, Some(&client_id))
        .await?;
    let account = save_account(&state, &account).await?;
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
    if state_token.trim().is_empty() {
        return Err(AppError::InvalidState("Estado de login ausente.".into()));
    }
    let client_id = get_configured_client_id().await;
    let account = state
        .auth
        .login_with_code_and_client_id(&code, &verifier, Some(&client_id))
        .await?;
    let account = save_account(state, &account).await?;
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
    let account = save_account(state, &account).await?;
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

pub(crate) async fn set_active_account_id(account_id: &str) -> AppResult<()> {
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

    sqlx::query("INSERT OR REPLACE INTO app_settings (key, value) VALUES ('app', ?)")
        .bind(new_raw)
        .execute(conn.pool())
        .await?;

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
    if row.id.starts_with("luxmc:") { super::lux_account::lux_account_sync(row.id.clone(), None, None).await?; }
    let _ = crate::db::schema::accounts::touch_account(&db, &row.id).await;
    set_active_account_id(&row.id).await?;
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
            set_active_account_id(&next_acc.id).await?;
        }
    } else {
        crate::db::schema::accounts::delete(&db, &uuid).await?;
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
    set_active_account_id(&account.id).await?;
    Ok(account)
}

#[tauri::command]
pub async fn auth_rename_offline(id: String, username: String) -> AppResult<()> {
    if !(3..=16).contains(&username.len()) || !username.bytes().all(|c| c.is_ascii_alphanumeric() || c == b'_') {
        return Err(AppError::InvalidInput("Use de 3 a 16 letras, números ou _.".into()));
    }
    let db = crate::db::shared_db().await?;
    let row = crate::db::schema::accounts::get_by_id(&db, &id).await?.ok_or_else(|| AppError::NotFound("Conta não encontrada".into()))?;
    if id.starts_with("luxmc:") || !row.refresh_token.is_empty() || row.access_token.as_ref().is_some_and(|token| !token.is_empty()) {
        return Err(AppError::InvalidInput("Altere o nome da conta pelo seu provedor.".into()));
    }
    sqlx::query("UPDATE accounts SET username = ?, updated_at = ? WHERE id = ?").bind(username).bind(chrono::Utc::now()).bind(id).execute(db.pool()).await?;
    Ok(())
}

pub(crate) fn preserve_local_appearance(account: &mut AuthAccount, existing: &AccountRow) {
    if existing.skin_url.as_deref().is_some_and(|skin| skin.starts_with("data:image/png;base64,")) {
        account.skin_url = existing.skin_url.clone();
        account.skin_variant = existing.skin_variant.clone();
    }
    if existing.cape_url.as_deref().is_some_and(|cape| cape.starts_with("data:image/png;base64,")) {
        account.cape_url = existing.cape_url.clone();
    }
}

async fn save_account(_state: &AppState, account: &AuthAccount) -> AppResult<AuthAccount> {
    let db = crate::db::shared_db().await?;
    let existing_row = crate::db::schema::accounts::get_by_uuid(&db, &account.uuid).await.ok().flatten();
    let mut account = account.clone();
    if let Some(existing) = existing_row.as_ref() { preserve_local_appearance(&mut account, existing); }
    let is_custom_cape = existing_row.as_ref().and_then(|r| r.cape_url.as_deref()).map(|c| {
        let trimmed = c.trim();
        !trimmed.is_empty()
            && !trimmed.starts_with("https://textures.minecraft.net/")
            && !trimmed.starts_with("http://textures.minecraft.net/")
    }).unwrap_or(false);

    let preserved_cape = if is_custom_cape {
        existing_row.as_ref().and_then(|r| r.cape_url.clone())
    } else {
        account.cape_url.clone()
    };

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
        cape_url: preserved_cape,
    };
    crate::db::schema::accounts::upsert(&db, &row).await?;
    set_active_account_id(&account.id).await?;
    Ok(account)
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

    let token_str = account.access_token.as_deref().unwrap_or("");
    let is_real_msa = !token_str.is_empty()
        && !token_str.starts_with("offline")
        && !token_str.starts_with("token_")
        && !token_str.starts_with("dev-")
        && token_str.len() > 100;

    if is_real_msa {
        let client = reqwest::Client::builder().timeout(std::time::Duration::from_secs(35)).build()?;
        if skin_url.starts_with("http://") || skin_url.starts_with("https://") {
            let body = serde_json::json!({
                "variant": norm_variant,
                "url": skin_url
            });

            client
                .post("https://api.minecraftservices.com/minecraft/profile/skins")
                .bearer_auth(token_str)
                .json(&body)
                .send().await?.error_for_status()?;
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

                client
                    .post("https://api.minecraftservices.com/minecraft/profile/skins")
                    .bearer_auth(token_str)
                    .multipart(form)
                    .send().await?.error_for_status()?;
            } else {
                return Err(AppError::InvalidInput("Arquivo de skin PNG inválido".into()));
            }
        }
    }

    let mut updated = account.clone();
    updated.skin_url = Some(skin_url.clone());
    updated.skin_variant = Some(norm_variant.to_string());
    updated.updated_at = chrono::Utc::now();
    crate::db::schema::accounts::upsert(&db, &updated).await?;

    Ok(())
}

#[tauri::command]
pub async fn auth_set_account_cape(
    uuid: String,
    cape_url: Option<String>,
) -> AppResult<()> {
    let db = crate::db::shared_db().await?;
    let res = sqlx::query("UPDATE accounts SET cape_url = ?, updated_at = ? WHERE uuid = ? OR id = ?")
        .bind(&cape_url)
        .bind(chrono::Utc::now())
        .bind(&uuid)
        .bind(&uuid)
        .execute(db.pool())
        .await?;

    if res.rows_affected() == 0 {
        return Err(AppError::NotFound(format!("Account not found: {}", uuid)));
    }

    if let Some(ref cu) = cape_url {
        if cu.starts_with("data:image/") {
            if let Some(pos) = cu.find(',') {
                use base64::Engine;
                if let Ok(bytes) = base64::engine::general_purpose::STANDARD.decode(&cu[pos + 1..]) {
                    crate::core::launcher::set_active_cape_bytes(bytes).await;
                }
            }
        }
    } else {
        crate::core::launcher::set_active_cape_bytes(Vec::new()).await;
    }

    // If MSA account has access token and explicitly unset cape, clear it
    let acc_opt = crate::db::schema::accounts::get_by_id(&db, &uuid).await.ok().flatten();
    if let Some(acc) = acc_opt {
        if let Some(token) = acc.access_token.filter(|t| !t.is_empty() && !t.starts_with("offline") && t.len() > 100) {
            let http = reqwest::Client::new();
            if cape_url.is_none() {
                let _ = http.delete("https://api.minecraftservices.com/minecraft/profile/capes/active")
                    .bearer_auth(&token)
                    .send()
                    .await;
            }
        }
    }

    Ok(())
}



#[tauri::command]
pub async fn auth_save_appearance(uuid: String, skin_url: String, variant: String, cape_url: Option<String>) -> AppResult<()> {
    fn validate_texture(value: &str) -> AppResult<()> {
        if value.len() > 3 * 1024 * 1024 { return Err(AppError::InvalidInput("Textura excede 3 MB".into())); }
        if value.starts_with("data:image/png;base64,") {
            use base64::Engine;
            let bytes = base64::engine::general_purpose::STANDARD.decode(value.split_once(',').unwrap().1)
                .map_err(|_| AppError::InvalidInput("PNG inválido".into()))?;
            if !bytes.starts_with(b"\x89PNG\r\n\x1a\n") { return Err(AppError::InvalidInput("PNG inválido".into())); }
        } else {
            let url = url::Url::parse(value).map_err(|_| AppError::InvalidInput("URL de textura inválida".into()))?;
            if url.scheme() != "https" || url.host_str().is_none() || !url.username().is_empty() || url.password().is_some() {
                return Err(AppError::InvalidInput("Use PNG ou HTTPS".into()));
            }
        }
        Ok(())
    }
    validate_texture(&skin_url)?;
    if let Some(cape) = &cape_url { validate_texture(cape)?; }
    if variant != "classic" && variant != "slim" { return Err(AppError::InvalidInput("Modelo de skin inválido".into())); }
    let db = crate::db::shared_db().await?;
    let row = sqlx::query_as::<_, AccountRow>("SELECT * FROM accounts WHERE id = ? OR uuid = ?").bind(&uuid).bind(&uuid).fetch_optional(db.pool()).await?.ok_or_else(|| AppError::NotFound("Conta não encontrada".into()))?;
    if row.id.starts_with("luxmc:") {
        super::lux_account::publish_appearance(&row.id, &skin_url, &variant, cape_url.as_deref()).await?;
    }
    let result = sqlx::query("UPDATE accounts SET skin_url = ?, skin_variant = ?, cape_url = ?, updated_at = ? WHERE id = ? OR uuid = ?")
        .bind(skin_url).bind(variant).bind(cape_url).bind(chrono::Utc::now()).bind(&uuid).bind(&uuid)
        .execute(db.pool()).await?;
    if result.rows_affected() == 0 { return Err(AppError::NotFound("Conta não encontrada".into())); }
    Ok(())
}

#[tauri::command]
pub async fn auth_resolve_texture(url: String) -> AppResult<String> {
    use base64::Engine;
    use futures_util::StreamExt;
    let http = reqwest::Client::builder().timeout(std::time::Duration::from_secs(6)).redirect(reqwest::redirect::Policy::none()).build()?;
    let mut target = url::Url::parse(&url).map_err(|_| AppError::InvalidInput("URL de textura inválida".into()))?;
    for _ in 0..4 {
        if target.scheme() != "https" || !target.username().is_empty() || target.password().is_some() || !matches!(target.host_str(), Some("mineskin.eu" | "mc-heads.net" | "minotar.net" | "crafatar.com" | "textures.minecraft.net")) {
            return Err(AppError::InvalidInput("Provedor de textura não permitido".into()));
        }
        let response = http.get(target.clone()).send().await?;
        if response.status().is_redirection() {
            let location = response.headers().get(reqwest::header::LOCATION).and_then(|v| v.to_str().ok()).ok_or_else(|| AppError::InvalidInput("Redirecionamento inválido".into()))?;
            target = target.join(location).map_err(|_| AppError::InvalidInput("Redirecionamento inválido".into()))?;
            continue;
        }
        let mut stream = response.error_for_status()?.bytes_stream();
        let mut bytes = Vec::new();
        while let Some(chunk) = stream.next().await { let chunk = chunk?; if bytes.len() + chunk.len() > 3 * 1024 * 1024 { return Err(AppError::InvalidInput("Textura excede 3 MB".into())); } bytes.extend_from_slice(&chunk); }
        if !bytes.starts_with(b"\x89PNG\r\n\x1a\n") { return Err(AppError::InvalidInput("Textura não é PNG".into())); }
        return Ok(format!("data:image/png;base64,{}", base64::engine::general_purpose::STANDARD.encode(bytes)));
    }
    Err(AppError::InvalidInput("Redirecionamentos demais".into()))
}

#[tauri::command]
pub async fn auth_read_local_texture(path: String) -> AppResult<String> {
    use base64::Engine;
    use tokio::io::AsyncReadExt;
    let file = tokio::fs::File::open(path).await?;
    if !file.metadata().await?.is_file() {
        return Err(AppError::InvalidInput("Selecione um arquivo de imagem válido".into()));
    }
    let mut bytes = Vec::new();
    file.take(10 * 1024 * 1024 + 1).read_to_end(&mut bytes).await?;
    let png_bytes = convert_or_validate_local_texture(&bytes)?;
    Ok(format!("data:image/png;base64,{}", base64::engine::general_purpose::STANDARD.encode(png_bytes)))
}

fn convert_or_validate_local_texture(bytes: &[u8]) -> AppResult<Vec<u8>> {
    if bytes.len() > 10 * 1024 * 1024 {
        return Err(AppError::InvalidInput("A textura deve ter até 10 MB".into()));
    }
    let dyn_img = image::load_from_memory(bytes)
        .map_err(|_| AppError::InvalidInput("Formato de imagem não suportado (use PNG ou WebP)".into()))?;
    let (width, height) = (dyn_img.width(), dyn_img.height());
    if width == 0 || height == 0 || width > 2048 || height > 2048 {
        return Err(AppError::InvalidInput("A textura deve ter no máximo 2048 × 2048 pixels".into()));
    }

    if bytes.starts_with(b"\x89PNG\r\n\x1a\n") {
        return Ok(bytes.to_vec());
    }

    let mut png_buf = Vec::new();
    dyn_img.to_rgba8().write_to(&mut std::io::Cursor::new(&mut png_buf), image::ImageFormat::Png)
        .map_err(|e| AppError::Internal(format!("Erro ao converter textura para PNG: {e}")))?;
    Ok(png_buf)
}

#[cfg(test)]
mod local_texture_tests {
    use super::convert_or_validate_local_texture;

    #[test]
    fn accepts_skin_and_rejects_non_png_or_oversize() {
        assert!(convert_or_validate_local_texture(include_bytes!("../../../static/steve.png")).is_ok());
        assert!(convert_or_validate_local_texture(b"not a png").is_err());
        assert!(convert_or_validate_local_texture(&vec![0; 10 * 1024 * 1024 + 1]).is_err());
    }
}

#[cfg(test)]
mod appearance_refresh_tests {
    use super::*;
    #[test]
    fn windows_refresh_keeps_local_skin_without_reverting_official_profiles() {
        let mut account: AuthAccount = serde_json::from_value(serde_json::json!({"id":"fixture","uuid":"fixture","username":"Player","accessToken":"new","refreshToken":"rotated","expiresAt":42,"skinUrl":"https://textures.minecraft.net/new","skinVariant":"classic"})).unwrap();
        let mut existing: AccountRow = serde_json::from_value(serde_json::json!({"id":"fixture","uuid":"fixture","username":"Player","refreshToken":"old","createdAt":"2026-10-04T00:00:00Z","updatedAt":"2026-10-04T00:00:00Z","skinUrl":"data:image/png;base64,local","skinVariant":"slim","capeUrl":"data:image/png;base64,cape"})).unwrap();
        preserve_local_appearance(&mut account, &existing);
        assert_eq!(account.skin_url, existing.skin_url);
        assert_eq!(account.skin_variant.as_deref(), Some("slim"));
        assert_eq!(account.refresh_token, "rotated");
        existing.skin_url = Some("https://textures.minecraft.net/old".into());
        account.skin_url = Some("https://textures.minecraft.net/new".into());
        preserve_local_appearance(&mut account, &existing);
        assert_eq!(account.skin_url.as_deref(), Some("https://textures.minecraft.net/new"));
    }
}
