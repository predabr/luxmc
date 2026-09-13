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

#[tauri::command]
pub async fn skins_list() -> AppResult<Vec<SavedSkinRow>> {
    let db = crate::db::shared_db().await?;
    let list = crate::db::schema::skins::list(&db).await?;
    Ok(list)
}

#[tauri::command]
pub async fn skins_save(
    state: State<'_, AppState>,
    request: SaveSkinRequest,
) -> AppResult<SavedSkinRow> {
    let db = crate::db::shared_db().await?;
    let skins_dir = get_skins_dir()?;

    let id = if request.id.trim().is_empty() {
        uuid::Uuid::new_v4().to_string()
    } else {
        request.id.trim().to_string()
    };

    let skin_file_path = skins_dir.join(format!("{}.png", id));

    // Decode and save skin PNG bytes to disk
    let skin_bytes: Vec<u8> = if request.skin_url.starts_with("data:image/") {
        if let Some(pos) = request.skin_url.find(',') {
            use base64::Engine;
            base64::engine::general_purpose::STANDARD
                .decode(&request.skin_url[pos + 1..])
                .unwrap_or_default()
        } else {
            Vec::new()
        }
    } else if request.skin_url.starts_with("http://") || request.skin_url.starts_with("https://") {
        if let Ok(resp) = state.http.get(&request.skin_url).send().await {
            resp.bytes().await.map(|b| b.to_vec()).unwrap_or_default()
        } else {
            Vec::new()
        }
    } else {
        let p = std::path::Path::new(&request.skin_url);
        if p.exists() {
            tokio::fs::read(p).await.unwrap_or_default()
        } else {
            Vec::new()
        }
    };

    if !skin_bytes.is_empty() {
        let _ = tokio::fs::write(&skin_file_path, &skin_bytes).await;
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
pub async fn skins_delete(id: String) -> AppResult<()> {
    let db = crate::db::shared_db().await?;
    crate::db::schema::skins::delete(&db, &id).await?;

    if let Ok(skins_dir) = get_skins_dir() {
        let p = skins_dir.join(format!("{}.png", id));
        let _ = tokio::fs::remove_file(p).await;
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

    let bytes = tokio::fs::read(p).await?;
    if bytes.len() < 8 || &bytes[0..8] != b"\x89PNG\r\n\x1a\n" {
        return Err(AppError::InvalidState("Arquivo selecionado não é uma imagem PNG válida".into()));
    }

    let id = uuid::Uuid::new_v4().to_string();
    let skin_file_path = skins_dir.join(format!("{}.png", id));
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

#[tauri::command]
pub async fn capes_save(
    state: State<'_, AppState>,
    request: SaveCapeRequest,
) -> AppResult<crate::db::schema::skins::SavedCapeRow> {
    let db = crate::db::shared_db().await?;
    let capes_dir = get_capes_dir()?;

    let id = if request.id.trim().is_empty() {
        uuid::Uuid::new_v4().to_string()
    } else {
        request.id.trim().to_string()
    };

    let cape_file_path = capes_dir.join(format!("{}.png", id));

    let cape_bytes: Vec<u8> = if request.cape_url.starts_with("data:image/") {
        if let Some(pos) = request.cape_url.find(',') {
            use base64::Engine;
            base64::engine::general_purpose::STANDARD
                .decode(&request.cape_url[pos + 1..])
                .unwrap_or_default()
        } else {
            Vec::new()
        }
    } else if request.cape_url.starts_with("http://") || request.cape_url.starts_with("https://") {
        if let Ok(resp) = state.http.get(&request.cape_url).send().await {
            resp.bytes().await.map(|b| b.to_vec()).unwrap_or_default()
        } else {
            Vec::new()
        }
    } else {
        let p = std::path::Path::new(&request.cape_url);
        if p.exists() {
            tokio::fs::read(p).await.unwrap_or_default()
        } else {
            Vec::new()
        }
    };

    if !cape_bytes.is_empty() {
        let _ = tokio::fs::write(&cape_file_path, &cape_bytes).await;
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
pub async fn capes_delete(id: String) -> AppResult<()> {
    let db = crate::db::shared_db().await?;
    crate::db::schema::skins::delete_cape(&db, &id).await?;

    if let Ok(capes_dir) = get_capes_dir() {
        let p = capes_dir.join(format!("{}.png", id));
        let _ = tokio::fs::remove_file(p).await;
    }

    Ok(())
}
