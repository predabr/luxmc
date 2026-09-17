use serde::{Deserialize, Serialize};
use std::path::PathBuf;

use crate::db::models::ProfileRow;
use crate::error::{AppError, AppResult};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InstanceMinecraftOptions {
    pub gamma: f64,
    pub fov: f64,
    pub render_distance: i32,
    pub simulation_distance: i32,
    pub max_fps: i32,
    pub gui_scale: i32,
    pub fullscreen: bool,
    pub vsync: bool,
    pub auto_jump: bool,
    pub bob_view: bool,
    pub sound_master: f32,
    pub sound_music: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConfigFileInfo {
    pub relative_path: String,
    pub content: String,
    pub language: String,
}

#[tauri::command]
#[allow(non_snake_case)]
pub async fn instance_options_get(
    profileId: String,
) -> AppResult<InstanceMinecraftOptions> {
    let db = crate::db::shared_db().await?;
    let row = sqlx::query_as::<_, ProfileRow>("SELECT * FROM profiles WHERE id = ?")
        .bind(&profileId)
        .fetch_optional(db.pool())
        .await?
        .ok_or_else(|| AppError::NotFound(format!("profile {profileId} not found")))?;

    let options_path = PathBuf::from(&row.game_dir).join("options.txt");
    let mut gamma = 1.0;
    let mut fov = 70.0;
    let mut gui_scale = 0;
    let mut render_distance = 12;
    let mut simulation_distance = 12;
    let mut max_fps = 120;
    let mut vsync = true;
    let mut fullscreen = false;
    let mut auto_jump = false;
    let mut bob_view = true;
    let mut sound_master = 1.0;
    let mut sound_music = 1.0;

    if options_path.exists() {
        if let Ok(content) = tokio::fs::read_to_string(&options_path).await {
            for line in content.lines() {
                if let Some((k, v)) = line.split_once(':') {
                    match k.trim() {
                        "gamma" => gamma = v.trim().parse().unwrap_or(gamma),
                        "fov" => fov = v.trim().parse().unwrap_or(fov),
                        "guiScale" => gui_scale = v.trim().parse().unwrap_or(gui_scale),
                        "renderDistance" => render_distance = v.trim().parse().unwrap_or(render_distance),
                        "simulationDistance" => simulation_distance = v.trim().parse().unwrap_or(simulation_distance),
                        "maxFps" => max_fps = v.trim().parse().unwrap_or(max_fps),
                        "enableVsync" => vsync = v.trim() == "true",
                        "fullscreen" => fullscreen = v.trim() == "true",
                        "autoJump" => auto_jump = v.trim() == "true",
                        "bobView" => bob_view = v.trim() == "true",
                        "soundCategory_master" => sound_master = v.trim().parse().unwrap_or(sound_master),
                        "soundCategory_music" => sound_music = v.trim().parse().unwrap_or(sound_music),
                        _ => {}
                    }
                }
            }
        }
    }

    Ok(InstanceMinecraftOptions {
        gamma,
        fov,
        render_distance,
        simulation_distance,
        max_fps,
        gui_scale,
        fullscreen,
        vsync,
        auto_jump,
        bob_view,
        sound_master,
        sound_music,
    })
}

#[tauri::command]
#[allow(non_snake_case)]
pub async fn instance_options_set(
    profileId: String,
    options: InstanceMinecraftOptions,
) -> AppResult<()> {
    let db = crate::db::shared_db().await?;
    let row = sqlx::query_as::<_, ProfileRow>("SELECT * FROM profiles WHERE id = ?")
        .bind(&profileId)
        .fetch_optional(db.pool())
        .await?
        .ok_or_else(|| AppError::NotFound(format!("profile {profileId} not found")))?;

    let options_path = PathBuf::from(&row.game_dir).join("options.txt");
    let mut lines = Vec::new();
    let mut existing_keys = std::collections::HashSet::new();

    if options_path.is_file() {
        if let Ok(content) = tokio::fs::read_to_string(&options_path).await {
            for line in content.lines() {
                let trimmed = line.trim();
                if let Some((k, _)) = trimmed.split_once(':') {
                    let k = k.trim();
                    existing_keys.insert(k.to_string());
                    match k {
                        "gamma" => lines.push(format!("gamma:{:.2}", options.gamma)),
                        "fov" => lines.push(format!("fov:{:.2}", options.fov)),
                        "renderDistance" => lines.push(format!("renderDistance:{}", options.render_distance)),
                        "simulationDistance" => lines.push(format!("simulationDistance:{}", options.simulation_distance)),
                        "maxFps" => lines.push(format!("maxFps:{}", options.max_fps)),
                        "guiScale" => lines.push(format!("guiScale:{}", options.gui_scale)),
                        "fullscreen" => lines.push(format!("fullscreen:{}", options.fullscreen)),
                        "enableVsync" => lines.push(format!("enableVsync:{}", options.vsync)),
                        "autoJump" => lines.push(format!("autoJump:{}", options.auto_jump)),
                        "bobView" => lines.push(format!("bobView:{}", options.bob_view)),
                        "soundCategory_master" => lines.push(format!("soundCategory_master:{:.2}", options.sound_master)),
                        "soundCategory_music" => lines.push(format!("soundCategory_music:{:.2}", options.sound_music)),
                        _ => lines.push(line.to_string()),
                    }
                } else {
                    lines.push(line.to_string());
                }
            }
        }
    }

    // Insert keys that weren't present
    let to_insert = [
        ("gamma", format!("gamma:{:.2}", options.gamma)),
        ("fov", format!("fov:{:.2}", options.fov)),
        ("renderDistance", format!("renderDistance:{}", options.render_distance)),
        ("simulationDistance", format!("simulationDistance:{}", options.simulation_distance)),
        ("maxFps", format!("maxFps:{}", options.max_fps)),
        ("guiScale", format!("guiScale:{}", options.gui_scale)),
        ("fullscreen", format!("fullscreen:{}", options.fullscreen)),
        ("enableVsync", format!("enableVsync:{}", options.vsync)),
        ("autoJump", format!("autoJump:{}", options.auto_jump)),
        ("bobView", format!("bobView:{}", options.bob_view)),
        ("soundCategory_master", format!("soundCategory_master:{:.2}", options.sound_master)),
        ("soundCategory_music", format!("soundCategory_music:{:.2}", options.sound_music)),
    ];

    for (k, entry) in to_insert {
        if !existing_keys.contains(k) {
            lines.push(entry);
        }
    }

    if let Some(parent) = options_path.parent() {
        tokio::fs::create_dir_all(parent).await?;
    }
    tokio::fs::write(&options_path, lines.join("\n")).await?;
    Ok(())
}

#[tauri::command]
#[allow(non_snake_case)]
pub async fn instance_config_read(
    profileId: String,
    relativePath: String,
) -> AppResult<ConfigFileInfo> {
    if relativePath.contains("..") || relativePath.starts_with('/') || relativePath.starts_with('\\') {
        return Err(AppError::InvalidInput("Invalid relative path".into()));
    }

    let db = crate::db::shared_db().await?;
    let row = sqlx::query_as::<_, ProfileRow>("SELECT * FROM profiles WHERE id = ?")
        .bind(&profileId)
        .fetch_optional(db.pool())
        .await?
        .ok_or_else(|| AppError::NotFound(format!("profile {profileId} not found")))?;

    let target = PathBuf::from(&row.game_dir).join(&relativePath);
    let language = if relativePath.ends_with(".toml") {
        "toml".to_string()
    } else if relativePath.ends_with(".json") {
        "json".to_string()
    } else {
        "ini".to_string()
    };

    let content = if target.is_file() {
        tokio::fs::read_to_string(&target).await.unwrap_or_default()
    } else {
        String::new()
    };

    Ok(ConfigFileInfo {
        relative_path: relativePath,
        content,
        language,
    })
}

#[tauri::command]
#[allow(non_snake_case)]
pub async fn instance_config_write(
    profileId: String,
    relativePath: String,
    content: String,
) -> AppResult<()> {
    if relativePath.contains("..") || relativePath.starts_with('/') || relativePath.starts_with('\\') {
        return Err(AppError::InvalidInput("Invalid relative path".into()));
    }

    let db = crate::db::shared_db().await?;
    let row = sqlx::query_as::<_, ProfileRow>("SELECT * FROM profiles WHERE id = ?")
        .bind(&profileId)
        .fetch_optional(db.pool())
        .await?
        .ok_or_else(|| AppError::NotFound(format!("profile {profileId} not found")))?;

    let target = PathBuf::from(&row.game_dir).join(&relativePath);
    if let Some(parent) = target.parent() {
        tokio::fs::create_dir_all(parent).await?;
    }

    tokio::fs::write(&target, content).await?;
    Ok(())
}
