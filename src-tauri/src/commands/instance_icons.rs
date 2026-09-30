use std::path::Path;

use crate::error::{AppError, AppResult};

fn allowed_app_dirs() -> Vec<std::path::PathBuf> {
    let mut dirs = Vec::new();
    if let Some(d) = directories::ProjectDirs::from("io", "github", "Luxmc") {
        dirs.push(d.data_dir().to_path_buf());
        dirs.push(d.config_dir().to_path_buf());
        dirs.push(d.cache_dir().to_path_buf());
    }
    if let Some(user) = directories::UserDirs::new() {
        dirs.push(user.home_dir().join(".local/share/luxmc"));
        dirs.push(user.home_dir().join(".config/luxmc"));
        dirs.push(user.home_dir().join(".cache/luxmc"));
    }
    dirs
}

fn is_safe_path(path: &str) -> bool {
    let p = Path::new(path);
    for component in p.components() {
        if matches!(component, std::path::Component::ParentDir) {
            return false;
        }
    }
    for dir in allowed_app_dirs() {
        let Ok(canonical_dir) = std::fs::canonicalize(dir) else {
            continue;
        };
        if let Ok(canonical_path) = std::fs::canonicalize(p) {
            if canonical_path.starts_with(&canonical_dir) {
                return true;
            }
            continue;
        }
        let mut parent = p.parent();
        while let Some(candidate) = parent {
            if let Ok(canonical_parent) = std::fs::canonicalize(candidate) {
                if canonical_parent.starts_with(&canonical_dir) {
                    return true;
                }
                break;
            }
            parent = candidate.parent();
        }
    }
    false
}

#[tauri::command]
pub async fn storage_total(_paths: Vec<String>) -> AppResult<i64> {
    tokio::task::spawn_blocking(move || -> AppResult<i64> {
        let mut total: i64 = 0;
        for path in _paths {
            if !is_safe_path(&path) {
                continue;
            }
            let p = Path::new(&path);
            if !p.exists() {
                continue;
            }
            if p.is_file() {
                if let Ok(meta) = p.metadata() {
                    total += meta.len() as i64;
                }
            } else if p.is_dir() {
                let mut stack = vec![p.to_path_buf()];
                while let Some(d) = stack.pop() {
                    if let Ok(rd) = std::fs::read_dir(&d) {
                        for entry in rd.flatten() {
                            let ep = entry.path();
                            if ep.is_dir() {
                                stack.push(ep);
                            } else if let Ok(meta) = ep.metadata() {
                                total += meta.len() as i64;
                            }
                        }
                    }
                }
            }
        }
        Ok(total)
    })
    .await
    .map_err(|error| crate::error::AppError::Internal(error.to_string()))?
}

#[tauri::command]
pub async fn directory_exists(path: String) -> AppResult<bool> {
    tokio::task::spawn_blocking(move || -> AppResult<bool> {
        if !is_safe_path(&path) {
            return Ok(false);
        }
        Ok(Path::new(&path).exists())
    })
    .await
    .map_err(|error| crate::error::AppError::Internal(error.to_string()))?
}

#[tauri::command]
pub async fn ensure_directory(path: String) -> AppResult<()> {
    tokio::task::spawn_blocking(move || -> AppResult<()> {
        if !is_safe_path(&path) {
            return Err(AppError::Internal("Invalid path".to_string()));
        }
        let p = Path::new(&path);
        if !p.exists() {
            std::fs::create_dir_all(p)?;
        }
        Ok(())
    })
    .await
    .map_err(|error| crate::error::AppError::Internal(error.to_string()))?
}

#[tauri::command]
pub async fn read_text_file(path: String) -> AppResult<String> {
    tokio::task::spawn_blocking(move || -> AppResult<String> {
        if !is_safe_path(&path) {
            return Err(AppError::Internal("Invalid path".to_string()));
        }
        std::fs::read_to_string(&path).map_err(|e| AppError::Internal(format!("read {path}: {e}")))
    })
    .await
    .map_err(|error| crate::error::AppError::Internal(error.to_string()))?
}

#[tauri::command]
pub async fn write_text_file(path: String, contents: String) -> AppResult<()> {
    tokio::task::spawn_blocking(move || -> AppResult<()> {
        if !is_safe_path(&path) {
            return Err(AppError::Internal("Invalid path".to_string()));
        }
        if let Some(parent) = Path::new(&path).parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(&path, contents)
            .map_err(|e| AppError::Internal(format!("write {path}: {e}")))?;
        Ok(())
    })
    .await
    .map_err(|error| crate::error::AppError::Internal(error.to_string()))?
}

#[tauri::command]
pub async fn delete_file_or_dir(path: String) -> AppResult<()> {
    tokio::task::spawn_blocking(move || -> AppResult<()> {
        if !is_safe_path(&path) {
            return Err(AppError::Internal("Invalid path".to_string()));
        }
        let p = Path::new(&path);
        if p.is_file() {
            std::fs::remove_file(p)?;
        } else if p.is_dir() {
            std::fs::remove_dir_all(p)?;
        }
        Ok(())
    })
    .await
    .map_err(|error| crate::error::AppError::Internal(error.to_string()))?
}
