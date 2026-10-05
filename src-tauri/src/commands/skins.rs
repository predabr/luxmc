use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tauri::State;

use crate::db::schema::skins::SavedSkinRow;
use crate::error::{AppError, AppResult};
use crate::state::AppState;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SaveSkinRequest {
    pub id: String,
    pub name: String,
    pub skin_url: String,
    pub avatar_url: Option<String>,
    pub model_type: Option<String>,
    pub is_custom: Option<bool>,
}

fn get_skins_dir() -> AppResult<PathBuf> {
    let base_dir = directories::ProjectDirs::from("io", "github", "Luxmc")
        .ok_or_else(|| AppError::InvalidState("could not determine data dir".into()))?;
    let dir = base_dir.data_dir().join("skins");
    std::fs::create_dir_all(&dir)?;
    Ok(dir)
}

const MAX_IMAGE_BYTES: u64 = 2 * 1024 * 1024;

fn sanitize_asset_id(id: &str) -> String {
    let cleaned: String = id
        .trim()
        .chars()
        .filter(|c| c.is_ascii_alphanumeric() || *c == '-' || *c == '_')
        .collect();
    if cleaned.is_empty() || cleaned.len() > 64 {
        uuid::Uuid::new_v4().to_string()
    } else {
        cleaned
    }
}

fn asset_path(dir: &PathBuf, id: &str) -> AppResult<PathBuf> {
    let path = dir.join(format!("{}.png", id));
    if path.parent() != Some(dir.as_path()) {
        return Err(AppError::InvalidState("Caminho de imagem inválido".into()));
    }
    Ok(path)
}

async fn load_asset_bytes(http: &reqwest::Client, source: &str) -> Option<Vec<u8>> {
    let bytes: Vec<u8> = if source.starts_with("data:image/") {
        let pos = source.find(',')?;
        use base64::Engine;
        base64::engine::general_purpose::STANDARD
            .decode(&source[pos + 1..])
            .ok()?
    } else if source.starts_with("http://") || source.starts_with("https://") {
        let mut resp = http.get(source).send().await.ok()?;
        if resp.content_length().unwrap_or(0) > MAX_IMAGE_BYTES {
            return None;
        }
        let mut out: Vec<u8> = Vec::new();
        while let Ok(Some(chunk)) = resp.chunk().await {
            if out.len() as u64 + chunk.len() as u64 > MAX_IMAGE_BYTES {
                return None;
            }
            out.extend_from_slice(&chunk);
        }
        out
    } else {
        let p = std::path::Path::new(source);
        let meta = tokio::fs::metadata(p).await.ok()?;
        if !meta.is_file() || meta.len() > MAX_IMAGE_BYTES {
            return None;
        }
        tokio::fs::read(p).await.ok()?
    };
    if bytes.len() as u64 > MAX_IMAGE_BYTES || bytes.is_empty() {
        return None;
    }
    Some(bytes)
}

#[tauri::command]
pub async fn skins_list() -> AppResult<Vec<SavedSkinRow>> {
    let db = crate::db::shared_db().await?;
    let list = crate::db::schema::skins::list(&db).await?;
    Ok(list)
}

pub async fn skins_save_core(
    http: &reqwest::Client,
    request: SaveSkinRequest,
) -> AppResult<SavedSkinRow> {
    let db = crate::db::shared_db().await?;
    let skins_dir = get_skins_dir()?;

    let id = if request.id.trim().is_empty() {
        uuid::Uuid::new_v4().to_string()
    } else {
        sanitize_asset_id(&request.id)
    };

    let skin_file_path = asset_path(&skins_dir, &id)?;

    let skin_bytes = load_asset_bytes(http, &request.skin_url).await;

    if let Some(bytes) = skin_bytes {
        if let Err(e) = tokio::fs::write(&skin_file_path, &bytes).await {
            tracing::warn!("failed to write skin {}: {e}", skin_file_path.display());
        }
    }

    let avatar_url = request.avatar_url.unwrap_or_default();
    let model_type = request.model_type.unwrap_or_else(|| "steve".to_string());
    let is_custom = request.is_custom.unwrap_or(true);

    let row = SavedSkinRow {
        id: id.clone(),
        name: request.name,
        skin_url: request.skin_url,
        avatar_url,
        model_type,
        is_custom,
        created_at: chrono::Utc::now().to_rfc3339(),
    };

    crate::db::schema::skins::upsert(&db, &row).await?;
    Ok(row)
}

#[tauri::command]
pub async fn skins_save(
    state: State<'_, AppState>,
    request: SaveSkinRequest,
) -> AppResult<SavedSkinRow> {
    skins_save_core(&state.http, request).await
}

#[tauri::command]
pub async fn skins_delete(id: String) -> AppResult<()> {
    let db = crate::db::shared_db().await?;
    crate::db::schema::skins::delete(&db, &id).await?;

    let safe_id = sanitize_asset_id(&id);
    if safe_id == id.trim() {
        if let Ok(skins_dir) = get_skins_dir() {
            if let Ok(p) = asset_path(&skins_dir, &safe_id) {
                let _ = tokio::fs::remove_file(p).await;
            }
        }
    }

    Ok(())
}

#[tauri::command]
pub async fn skins_import_file(
    path: String,
    name: Option<String>,
    model_type: Option<String>,
) -> AppResult<SavedSkinRow> {
    let db = crate::db::shared_db().await?;
    let skins_dir = get_skins_dir()?;

    let p = std::path::Path::new(&path);
    if !p.exists() {
        return Err(AppError::NotFound("Arquivo de skin não encontrado".into()));
    }
    if p.metadata().map(|m| m.len() > 2 * 1024 * 1024).unwrap_or(false) {
        return Err(AppError::InvalidState("Arquivo de skin muito grande (máximo 2MB)".into()));
    }

    let bytes = tokio::fs::read(p).await?;
    if bytes.len() > 2 * 1024 * 1024 {
        return Err(AppError::InvalidState("Arquivo de skin muito grande (máximo 2MB)".into()));
    }
    if bytes.len() < 8 || &bytes[0..8] != b"\x89PNG\r\n\x1a\n" {
        return Err(AppError::InvalidState("Arquivo selecionado não é uma imagem PNG válida".into()));
    }

    let id = uuid::Uuid::new_v4().to_string();
    let skin_file_path = asset_path(&skins_dir, &id)?;
    tokio::fs::write(&skin_file_path, &bytes).await?;

    use base64::Engine;
    let b64 = base64::engine::general_purpose::STANDARD.encode(&bytes);
    let data_url = format!("data:image/png;base64,{}", b64);

    let skin_name = name.unwrap_or_else(|| {
        p.file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("Minha Skin")
            .to_string()
    });

    let row = SavedSkinRow {
        id,
        name: skin_name,
        skin_url: data_url.clone(),
        avatar_url: data_url,
        model_type: model_type.unwrap_or_else(|| "steve".to_string()),
        is_custom: true,
        created_at: chrono::Utc::now().to_rfc3339(),
    };

    crate::db::schema::skins::upsert(&db, &row).await?;
    Ok(row)
}

#[tauri::command]
pub async fn gaming_stats_get() -> AppResult<Option<String>> {
    let db = crate::db::shared_db().await?;
    use sqlx::Row;
    let row = sqlx::query("SELECT data FROM play_time_stats WHERE id = 'default' LIMIT 1")
        .fetch_optional(db.pool())
        .await?;
    let data: Option<String> = row.and_then(|r| r.try_get("data").ok());
    Ok(data)
}

#[tauri::command]
pub async fn gaming_stats_save(data: String) -> AppResult<()> {
    let db = crate::db::shared_db().await?;
    let now = chrono::Utc::now().to_rfc3339();
    sqlx::query(
        "INSERT INTO play_time_stats (id, data, updated_at) VALUES ('default', ?, ?)
         ON CONFLICT(id) DO UPDATE SET data = excluded.data, updated_at = excluded.updated_at"
    )
    .bind(data)
    .bind(now)
    .execute(db.pool())
    .await?;
    Ok(())
}

fn get_capes_dir() -> AppResult<PathBuf> {
    let base_dir = directories::ProjectDirs::from("io", "github", "Luxmc")
        .ok_or_else(|| AppError::InvalidState("could not determine data dir".into()))?;
    let dir = base_dir.data_dir().join("capes");
    std::fs::create_dir_all(&dir)?;
    Ok(dir)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SaveCapeRequest {
    pub id: String,
    pub name: String,
    pub cape_url: String,
}

#[tauri::command]
pub async fn capes_list() -> AppResult<Vec<crate::db::schema::skins::SavedCapeRow>> {
    let db = crate::db::shared_db().await?;
    let list = crate::db::schema::skins::list_capes(&db).await?;
    Ok(list)
}

pub async fn capes_save_core(
    http: &reqwest::Client,
    request: SaveCapeRequest,
) -> AppResult<crate::db::schema::skins::SavedCapeRow> {
    let db = crate::db::shared_db().await?;
    let capes_dir = get_capes_dir()?;

    let id = if request.id.trim().is_empty() {
        uuid::Uuid::new_v4().to_string()
    } else {
        sanitize_asset_id(&request.id)
    };

    let cape_file_path = asset_path(&capes_dir, &id)?;

    let cape_bytes = load_asset_bytes(http, &request.cape_url).await;

    if let Some(bytes) = cape_bytes {
        if let Err(e) = tokio::fs::write(&cape_file_path, &bytes).await {
            tracing::warn!("failed to write cape {}: {e}", cape_file_path.display());
        }
    }

    let row = crate::db::schema::skins::SavedCapeRow {
        id: id.clone(),
        name: request.name,
        cape_url: request.cape_url,
        created_at: chrono::Utc::now().to_rfc3339(),
    };

    crate::db::schema::skins::upsert_cape(&db, &row).await?;
    Ok(row)
}

#[tauri::command]
pub async fn capes_save(
    state: State<'_, AppState>,
    request: SaveCapeRequest,
) -> AppResult<crate::db::schema::skins::SavedCapeRow> {
    capes_save_core(&state.http, request).await
}

#[tauri::command]
pub async fn capes_delete(id: String) -> AppResult<()> {
    let db = crate::db::shared_db().await?;
    crate::db::schema::skins::delete_cape(&db, &id).await?;

    if let Ok(capes_dir) = get_capes_dir() {
        let safe_id = sanitize_asset_id(&id);
        if safe_id == id.trim() {
            if let Ok(p) = asset_path(&capes_dir, &safe_id) {
                let _ = tokio::fs::remove_file(p).await;
            }
        }
    }

    Ok(())
}

#[tauri::command]
pub async fn minecraft_uuid(username: String) -> AppResult<Option<String>> {
    if username.is_empty() || username.len() > 16 || !username.bytes().all(|value| value.is_ascii_alphanumeric() || value == b'_') {
        return Err(AppError::InvalidInput("Nickname inválido".into()));
    }
    let response = reqwest::Client::new()
        .get(format!("https://api.mojang.com/users/profiles/minecraft/{username}"))
        .timeout(std::time::Duration::from_secs(8)).send().await?;
    if response.status() == reqwest::StatusCode::NOT_FOUND || response.status() == reqwest::StatusCode::NO_CONTENT {
        return Ok(None);
    }
    #[derive(Deserialize)]
    struct Identity { id: String }
    let identity = response.error_for_status()?.json::<Identity>().await?;
    Ok(uuid::Uuid::parse_str(&identity.id).ok().map(|value| value.simple().to_string()))
}
