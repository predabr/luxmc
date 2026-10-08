use serde::Serialize;
use std::{collections::HashMap, sync::LazyLock, time::{Duration, SystemTime, UNIX_EPOCH}};
use tokio::sync::Semaphore;
use crate::{db::models::ProfileRow, error::{AppError, AppResult}, state::AppState};

static WORK: LazyLock<Semaphore> = LazyLock::new(|| Semaphore::new(4));

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ContentIcon { name: String, icon: Option<String>, icon_key: String, resolved: bool }

pub fn cache_is_resolved(value: &str) -> bool {
    if let Some(time) = value.strip_prefix("missing:").and_then(|value| value.parse::<u64>().ok()) {
        return now().saturating_sub(time) < 86400;
    }
    !value.is_empty()
}
fn now() -> u64 { SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_secs() }
fn allowed(path: &str) -> bool {
    matches!(path, "mods" | "resourcepacks" | "shaderpacks") || {
        let parts: Vec<_> = path.split('/').collect();
        parts.len() == 3 && parts[0] == "saves" && !parts[1].is_empty() && parts[1] != "." && parts[1] != ".." && !parts[1].contains('\\') && parts[2] == "datapacks"
    }
}

#[tauri::command]
#[allow(non_snake_case)]
pub async fn instance_content_icons(state: tauri::State<'_, AppState>, profileId: String, subPath: String, fileNames: Vec<String>, lookup: Option<bool>) -> AppResult<Vec<ContentIcon>> {
    instance_content_icons_core(&state, profileId, subPath, fileNames, lookup.unwrap_or(false)).await
}

pub async fn instance_content_icons_core(state: &AppState, profile_id: String, sub_path: String, names: Vec<String>, lookup: bool) -> AppResult<Vec<ContentIcon>> {
    if names.len() > 32 || !allowed(&sub_path) { return Err(AppError::InvalidInput("Invalid content thumbnail request".into())); }
    let db = crate::db::shared_db().await?;
    let row = sqlx::query_as::<_, ProfileRow>("SELECT * FROM profiles WHERE id = ?").bind(&profile_id).fetch_optional(db.pool()).await?.ok_or_else(|| AppError::NotFound("Instance not found".into()))?;
    let base = crate::core::instance_paths::resolve_within(std::path::Path::new(&row.game_dir), &sub_path)?;
    let mut results = Vec::new();
    let mut hashes = HashMap::<String, Vec<usize>>::new();
    for name in names {
        if name.is_empty() || name.contains(['/', '\\']) || name == "." || name == ".." { return Err(AppError::InvalidInput("Invalid content filename".into())); }
        let path = crate::core::instance_paths::resolve_within(&base, &name)?;
        let Ok(metadata) = tokio::fs::metadata(&path).await else { continue; };
        let key = super::mods::content_icon_key(&path, &metadata);
        let mut cached = sqlx::query_scalar::<_, String>("SELECT icon_url FROM mod_icons WHERE key = ?").bind(&key).fetch_optional(db.pool()).await?;
        if cached.is_none() && sub_path != "mods" && !metadata.is_dir() {
            let legacy_key = key.replacen("content:v2:", "content:", 1);
            if let Some(url) = sqlx::query_scalar::<_, String>("SELECT icon_url FROM mod_icons WHERE key = ?").bind(legacy_key).fetch_optional(db.pool()).await?.filter(|value| value.starts_with("https://")) {
                sqlx::query("INSERT INTO mod_icons(key, icon_url) VALUES (?, ?) ON CONFLICT(key) DO UPDATE SET icon_url = excluded.icon_url").bind(&key).bind(&url).execute(db.pool()).await?;
                cached = Some(url);
            }
        }
        if let Some(value) = cached.filter(|value| cache_is_resolved(value)) {
            results.push(ContentIcon { name, icon: (!value.starts_with("missing:")).then_some(value), icon_key: key, resolved: true });
            continue;
        }
        let _permit = WORK.acquire().await.map_err(|error| AppError::Internal(error.to_string()))?;
        let file = path.clone();
        let icon = tokio::task::spawn_blocking(move || super::mods::extract_mod_icon_from_jar(&file)).await.map_err(|error| AppError::Internal(error.to_string()))?;
        let resolved = icon.is_some() || metadata.is_dir();
        if resolved {
            let value = icon.clone().unwrap_or_else(|| format!("missing:{}", now()));
            sqlx::query("INSERT INTO mod_icons(key, icon_url) VALUES (?, ?) ON CONFLICT(key) DO UPDATE SET icon_url = excluded.icon_url").bind(&key).bind(value).execute(db.pool()).await?;
        } else if lookup && metadata.len() <= 256 * 1024 * 1024 {
            let hash = tokio::task::spawn_blocking(move || -> std::io::Result<String> {
                use sha1::{Digest, Sha1};
                use std::io::Read;
                let mut file = std::fs::File::open(path)?;
                let mut digest = Sha1::new();
                let mut buffer = [0u8; 65536];
                loop { let count = file.read(&mut buffer)?; if count == 0 { break; } digest.update(&buffer[..count]); }
                Ok(format!("{:x}", digest.finalize()))
            }).await.map_err(|error| AppError::Internal(error.to_string()))??;
            hashes.entry(hash).or_default().push(results.len());
        }
        results.push(ContentIcon { name, icon, icon_key: key, resolved });
    }
    if lookup && sub_path == "mods" && !hashes.is_empty() {
        let records = crate::db::schema::mods::list_by_profile(&db, &profile_id).await?;
        let exact: Vec<_> = records.iter().filter(|row| row.source == "curseforge" && hashes.contains_key(&row.sha1)).filter_map(|row| row.project_id.parse::<u64>().ok().map(|id| (id, row.sha1.clone()))).collect();
        let ids: Vec<_> = exact.iter().map(|(id,_)| *id).collect();
        if !ids.is_empty() {
            if let Ok(logos) = tokio::time::timeout(Duration::from_secs(10), crate::core::mods::curseforge::get_mod_logos_batch(&state.http, &ids)).await {
                for (id,hash) in exact {
                    if let Some(url) = logos.get(&id).filter(|url| url.starts_with("https://")) {
                        if let Some(indices) = hashes.remove(&hash) {
                          for index in indices {
                            let entry = &mut results[index]; entry.icon = Some(url.clone()); entry.resolved = true;
                            sqlx::query("INSERT INTO mod_icons(key, icon_url) VALUES (?, ?) ON CONFLICT(key) DO UPDATE SET icon_url = excluded.icon_url").bind(&entry.icon_key).bind(url).execute(db.pool()).await?;
                          }
                        }
                    }
                }
            }
        }
    }
    if !hashes.is_empty() {
        let response = state.http.post("https://api.modrinth.com/v2/version_files").timeout(Duration::from_secs(10)).json(&serde_json::json!({"hashes": hashes.keys().collect::<Vec<_>>(), "algorithm": "sha1"})).send().await;
        if let Ok(response) = response {
            if response.status().is_success() {
                if let Ok(versions) = response.json::<HashMap<String, serde_json::Value>>().await {
                    let ids: Vec<_> = versions.values().filter_map(|version| version["project_id"].as_str()).collect();
                    let projects = if ids.is_empty() { Some(Vec::new()) } else {
                        match state.http.get("https://api.modrinth.com/v2/projects").query(&[("ids", serde_json::to_string(&ids).unwrap_or_default())]).timeout(Duration::from_secs(10)).send().await {
                            Ok(response) if response.status().is_success() => response.json::<Vec<serde_json::Value>>().await.ok(),
                            _ => None
                        }
                    };
                    if let Some(projects) = projects {
                        for (hash, indices) in hashes {
                            let project = versions.get(&hash).and_then(|version| version["project_id"].as_str());
                            let icon = projects.iter().find(|entry| entry["id"].as_str() == project).and_then(|entry| entry["icon_url"].as_str()).filter(|url| url.starts_with("https://")).map(String::from);
                            if project.is_some() && icon.is_none() { continue; }
                            for index in indices {
                            let entry = &mut results[index];
                            entry.icon = icon.clone();
                            entry.resolved = true;
                            sqlx::query("INSERT INTO mod_icons(key, icon_url) VALUES (?, ?) ON CONFLICT(key) DO UPDATE SET icon_url = excluded.icon_url").bind(&entry.icon_key).bind(entry.icon.clone().unwrap_or_else(|| format!("missing:{}", now()))).execute(db.pool()).await?;
                            }
                        }
                    }
                }
            }
        }
    }
    Ok(results)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn content_paths_and_negative_cache_are_bounded() {
        assert!(allowed("mods")); assert!(allowed("saves/My world/datapacks"));
        assert!(!allowed("saves/../datapacks")); assert!(!allowed("config"));
        assert!(cache_is_resolved(&format!("missing:{}", now())));
        assert!(!cache_is_resolved("missing:0")); assert!(!cache_is_resolved(""));
    }
}
