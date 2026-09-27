use crate::error::AppResult;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StorageBreakdown {
    pub category: String,
    pub bytes: i64,
    pub path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InstanceStorageInfo {
    pub id: String,
    pub name: String,
    pub mc_version: String,
    pub loader: String,
    pub icon: String,
    pub bytes: u64,
    pub path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StorageFullReport {
    pub total_bytes: u64,
    pub categories: Vec<StorageBreakdown>,
    pub instances: Vec<InstanceStorageInfo>,
    pub logs_bytes: u64,
    pub cache_bytes: u64,
}

fn dir_size_recursive(p: &std::path::Path) -> u64 {
    let mut total = 0u64;
    if let Ok(entries) = std::fs::read_dir(p) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                total += dir_size_recursive(&path);
            } else if let Ok(meta) = path.metadata() {
                total += meta.len();
            }
        }
    }
    total
}

#[tauri::command]
pub async fn storage_breakdown() -> AppResult<Vec<StorageBreakdown>> {
    let base_dir = directories::ProjectDirs::from("io", "github", "Luxmc")
        .map(|d| d.data_dir().to_path_buf())
        .unwrap_or_else(|| std::path::PathBuf::from("/tmp/luxmc"));

    let out = tokio::task::spawn_blocking(move || {
        let mut results = Vec::new();
        for (category, sub) in &[
            ("libraries", "libraries"),
            ("assets", "assets"),
            ("versions", "versions"),
            ("instances", "instances"),
            ("mods", "mods"),
            ("logs", "logs"),
            ("cache", "cache"),
        ] {
            let path = base_dir.join(sub);
            let total = if path.is_dir() {
                dir_size_recursive(&path) as i64
            } else {
                0
            };
            results.push(StorageBreakdown {
                category: category.to_string(),
                bytes: total,
                path: path.to_string_lossy().to_string(),
            });
        }
        results
    })
    .await
    .map_err(|e| crate::error::AppError::Internal(e.to_string()))?;

    Ok(out)
}

#[tauri::command]
pub async fn storage_full_report() -> AppResult<StorageFullReport> {
    let base_dir = directories::ProjectDirs::from("io", "github", "Luxmc")
        .map(|d| d.data_dir().to_path_buf())
        .unwrap_or_else(|| std::path::PathBuf::from("/tmp/luxmc"));

    let db = crate::db::shared_db().await?;
    let profiles = crate::db::schema::profiles::list(&db).await.unwrap_or_default();

    let base_dir_clone = base_dir.clone();
    let report = tokio::task::spawn_blocking(move || {
        let mut categories = Vec::new();
        let mut total_bytes = 0u64;

        for (category, sub) in &[
            ("libraries", "libraries"),
            ("assets", "assets"),
            ("versions", "versions"),
            ("instances", "instances"),
            ("mods", "mods"),
            ("logs", "logs"),
            ("cache", "cache"),
        ] {
            let path = base_dir_clone.join(sub);
            let cat_bytes = if path.is_dir() {
                dir_size_recursive(&path)
            } else {
                0
            };
            total_bytes += cat_bytes;
            categories.push(StorageBreakdown {
                category: category.to_string(),
                bytes: cat_bytes as i64,
                path: path.to_string_lossy().to_string(),
            });
        }

        let mut instance_items = Vec::new();
        let instances_dir = base_dir_clone.join("instances");

        for p in profiles {
            let mut inst_dir = instances_dir.join(&p.id);
            if !inst_dir.is_dir() {
                let alt = std::path::PathBuf::from(&p.game_dir);
                if alt.is_dir() {
                    inst_dir = alt;
                }
            }

            let inst_bytes = if inst_dir.is_dir() {
                dir_size_recursive(&inst_dir)
            } else {
                0
            };

            instance_items.push(InstanceStorageInfo {
                id: p.id,
                name: p.name,
                mc_version: p.mc_version,
                loader: p.loader,
                icon: p.icon,
                bytes: inst_bytes,
                path: inst_dir.to_string_lossy().to_string(),
            });
        }

        instance_items.sort_by(|a, b| b.bytes.cmp(&a.bytes));

        let logs_path = base_dir_clone.join("logs");
        let logs_bytes = if logs_path.is_dir() {
            dir_size_recursive(&logs_path)
        } else {
            0
        };

        let cache_path = base_dir_clone.join("cache");
        let dl_path = base_dir_clone.join("downloads");
        let cache_bytes = (if cache_path.is_dir() {
            dir_size_recursive(&cache_path)
        } else {
            0
        }) + (if dl_path.is_dir() {
            dir_size_recursive(&dl_path)
        } else {
            0
        });

        StorageFullReport {
            total_bytes,
            categories,
            instances: instance_items,
            logs_bytes,
            cache_bytes,
        }
    })
    .await
    .map_err(|e| crate::error::AppError::Internal(e.to_string()))?;

    Ok(report)
}

#[tauri::command]
pub async fn storage_clear_logs() -> AppResult<u64> {
    let base_dir = directories::ProjectDirs::from("io", "github", "Luxmc")
        .map(|d| d.data_dir().to_path_buf())
        .unwrap_or_else(|| std::path::PathBuf::from("/tmp/luxmc"));

    let freed = tokio::task::spawn_blocking(move || {
        let mut total_freed = 0u64;

        let logs_dir = base_dir.join("logs");
        if logs_dir.is_dir() {
            if let Ok(entries) = std::fs::read_dir(&logs_dir) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if path.is_file() {
                        if let Ok(meta) = path.metadata() {
                            total_freed += meta.len();
                        }
                        let _ = std::fs::remove_file(path);
                    }
                }
            }
        }

        let instances_dir = base_dir.join("instances");
        if instances_dir.is_dir() {
            if let Ok(inst_entries) = std::fs::read_dir(&instances_dir) {
                for inst_entry in inst_entries.flatten() {
                    let inst_logs = inst_entry.path().join(".minecraft").join("logs");
                    if inst_logs.is_dir() {
                        if let Ok(files) = std::fs::read_dir(&inst_logs) {
                            for f in files.flatten() {
                                let path = f.path();
                                if path.is_file() {
                                    if let Ok(meta) = path.metadata() {
                                        total_freed += meta.len();
                                    }
                                    let _ = std::fs::remove_file(path);
                                }
                            }
                        }
                    }
                    let inst_crash_reports = inst_entry.path().join(".minecraft").join("crash-reports");
                    if inst_crash_reports.is_dir() {
                        if let Ok(files) = std::fs::read_dir(&inst_crash_reports) {
                            for f in files.flatten() {
                                let path = f.path();
                                if path.is_file() {
                                    if let Ok(meta) = path.metadata() {
                                        total_freed += meta.len();
                                    }
                                    let _ = std::fs::remove_file(path);
                                }
                            }
                        }
                    }
                }
            }
        }

        total_freed
    })
    .await
    .map_err(|e| crate::error::AppError::Internal(e.to_string()))?;

    Ok(freed)
}

#[tauri::command]
pub async fn storage_clear_cache() -> AppResult<u64> {
    let base_dir = directories::ProjectDirs::from("io", "github", "Luxmc")
        .map(|d| d.data_dir().to_path_buf())
        .unwrap_or_else(|| std::path::PathBuf::from("/tmp/luxmc"));

    let freed = tokio::task::spawn_blocking(move || {
        let mut total_freed = 0u64;

        for sub in &["cache", "downloads", "migration-staging"] {
            let path = base_dir.join(sub);
            if path.is_dir() {
                total_freed += dir_size_recursive(&path);
                let _ = std::fs::remove_dir_all(&path);
                let _ = std::fs::create_dir_all(&path);
            }
        }

        total_freed
    })
    .await
    .map_err(|e| crate::error::AppError::Internal(e.to_string()))?;

    Ok(freed)
}

#[tauri::command]
pub async fn storage_delete_instance(id: String) -> AppResult<()> {
    crate::commands::profiles::profiles_delete(id).await
}
