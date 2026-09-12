use serde::{Deserialize, Serialize};
use tauri::State;

use crate::error::AppResult;
use crate::state::AppState;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StorageBreakdown {
    pub category: String,
    pub bytes: i64,
    pub path: String,
}

#[tauri::command]
pub async fn storage_breakdown(_state: State<'_, AppState>) -> AppResult<Vec<StorageBreakdown>> {
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
        ] {
            let path = base_dir.join(sub);
            let mut total: i64 = 0;
            if path.is_dir() {
                let mut stack = vec![path.clone()];
                while let Some(p) = stack.pop() {
                    if let Ok(rd) = std::fs::read_dir(&p) {
                        for entry in rd.flatten() {
                            if let Ok(ft) = entry.file_type() {
                                if ft.is_dir() {
                                    stack.push(entry.path());
                                } else if let Ok(meta) = entry.metadata() {
                                    total += meta.len() as i64;
                                }
                            }
                        }
                    }
                }
            }
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
