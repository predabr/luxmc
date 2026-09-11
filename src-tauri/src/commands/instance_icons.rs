use std::path::Path;

use crate::error::{AppError, AppResult};

fn is_safe_path(path: &str) -> bool {
    let p = Path::new(path);
    for component in p.components() {
        if matches!(component, std::path::Component::ParentDir) {
            return false;
        }
    }
    true
}

#[tauri::command]
pub async fn storage_total(_paths: Vec<String>) -> AppResult<i64> {
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
}

#[tauri::command]
pub async fn directory_exists(path: String) -> AppResult<bool> {
    if !is_safe_path(&path) {
        return Ok(false);
    }
    Ok(Path::new(&path).exists())
}

#[tauri::command]
pub async fn ensure_directory(path: String) -> AppResult<()> {
    if !is_safe_path(&path) {
        return Err(AppError::Internal("Invalid path".to_string()));
    }
    let p = Path::new(&path);
    if !p.exists() {
        std::fs::create_dir_all(p)?;
    }
    Ok(())
}

#[tauri::command]
pub async fn read_text_file(path: String) -> AppResult<String> {
    if !is_safe_path(&path) {
        return Err(AppError::Internal("Invalid path".to_string()));
    }
    std::fs::read_to_string(&path).map_err(|e| AppError::Internal(format!("read {path}: {e}")))
}

#[tauri::command]
pub async fn write_text_file(path: String, contents: String) -> AppResult<()> {
    if !is_safe_path(&path) {
        return Err(AppError::Internal("Invalid path".to_string()));
    }
    if let Some(parent) = Path::new(&path).parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(&path, contents)
        .map_err(|e| AppError::Internal(format!("write {path}: {e}")))?;
    Ok(())
}

#[tauri::command]
pub async fn delete_file_or_dir(path: String) -> AppResult<()> {
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
}
