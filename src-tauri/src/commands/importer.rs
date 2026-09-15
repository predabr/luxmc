use std::path::{Path, PathBuf};
use serde::{Deserialize, Serialize};
use tauri::State;

use etcetera::AppStrategy;

use crate::error::AppResult;
use crate::state::AppState;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExternalInstance {
    pub launcher: String,
    pub name: String,
    pub path: String,
    pub mc_version: String,
    pub loader: String,
    pub mod_count: usize,
    pub has_saves: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ImportResult {
    pub success: bool,
    pub profile_id: String,
    pub name: String,
    pub message: String,
}

#[tauri::command]
pub async fn importer_detect_launchers() -> AppResult<Vec<ExternalInstance>> {
    let mut instances = Vec::new();
    let home = etcetera::home_dir().unwrap_or_else(|_| PathBuf::from("/"));

    let prism_dirs = vec![
        home.join(".local/share/PrismLauncher/instances"),
        home.join(".var/app/org.prismlauncher.PrismLauncher/data/PrismLauncher/instances"),
        home.join("AppData/Roaming/PrismLauncher/instances"),
    ];

    for base in prism_dirs {
        if base.is_dir() {
            if let Ok(entries) = std::fs::read_dir(&base) {
                for entry in entries.flatten() {
                    let p = entry.path();
                    if p.is_dir() {
                        let name = p.file_name().unwrap_or_default().to_string_lossy().to_string();
                        if name.starts_with('.') || name == "_MMC_TEMP" {
                            continue;
                        }
                        let (mc_version, loader) = parse_prism_instance_cfg(&p);
                        let mod_count = count_mods_in_dir(&p.join(".minecraft/mods")).max(count_mods_in_dir(&p.join("mods")));
                        let has_saves = p.join(".minecraft/saves").is_dir() || p.join("saves").is_dir();

                        instances.push(ExternalInstance {
                            launcher: "Prism Launcher".into(),
                            name,
                            path: p.to_string_lossy().to_string(),
                            mc_version,
                            loader,
                            mod_count,
                            has_saves,
                        });
                    }
                }
            }
        }
    }

    let curseforge_dirs = vec![
        home.join("curseforge/minecraft/Instances"),
        home.join(".local/share/curseforge/minecraft/Instances"),
        home.join("Documents/curseforge/minecraft/Instances"),
        home.join("curseforge/minecraft/instances"),
    ];

    for base in curseforge_dirs {
        if base.is_dir() {
            if let Ok(entries) = std::fs::read_dir(&base) {
                for entry in entries.flatten() {
                    let p = entry.path();
                    if p.is_dir() {
                        let name = p.file_name().unwrap_or_default().to_string_lossy().to_string();
                        if name.starts_with('.') {
                            continue;
                        }
                        let (mc_version, loader) = parse_curseforge_manifest(&p);
                        let mod_count = count_mods_in_dir(&p.join("mods"));
                        let has_saves = p.join("saves").is_dir();

                        instances.push(ExternalInstance {
                            launcher: "CurseForge".into(),
                            name,
                            path: p.to_string_lossy().to_string(),
                            mc_version,
                            loader,
                            mod_count,
                            has_saves,
                        });
                    }
                }
            }
        }
    }

    let lunar_dirs = vec![
        home.join(".lunarclient/offline/multiver"),
        home.join(".lunarclient/offline"),
        home.join(".lunarclient"),
    ];

    for base in lunar_dirs {
        if base.is_dir() {
            let saves_dir = home.join(".minecraft/saves");
            let has_saves = saves_dir.is_dir();
            instances.push(ExternalInstance {
                launcher: "Lunar Client".into(),
                name: "Lunar Client (Perfil Padrão)".into(),
                path: base.to_string_lossy().to_string(),
                mc_version: "1.8.9".into(),
                loader: "vanilla".into(),
                mod_count: 0,
                has_saves,
            });
            break;
        }
    }

    let badlion_dir = home.join(".badlion");
    if badlion_dir.is_dir() {
        instances.push(ExternalInstance {
            launcher: "Badlion Client".into(),
            name: "Badlion Client (Instalação Local)".into(),
            path: badlion_dir.to_string_lossy().to_string(),
            mc_version: "1.8.9".into(),
            loader: "vanilla".into(),
            mod_count: 0,
            has_saves: home.join(".minecraft/saves").is_dir(),
        });
    }

    Ok(instances)
}

#[tauri::command]
#[allow(non_snake_case)]
pub async fn importer_execute_import(
    _state: State<'_, AppState>,
    sourcePath: String,
    targetName: String,
    mcVersion: Option<String>,
    loader: Option<String>,
) -> AppResult<ImportResult> {
    let src = PathBuf::from(&sourcePath);
    if !src.is_dir() {
        return Err(crate::error::AppError::NotFound("Diretório de origem não encontrado".into()));
    }

    let profile_id = uuid::Uuid::new_v4().to_string();
    let safe_name = if targetName.trim().is_empty() {
        src.file_name().unwrap_or_default().to_string_lossy().to_string()
    } else {
        targetName.trim().to_string()
    };

    let data_dir = etcetera::app_strategy::choose_app_strategy(etcetera::app_strategy::AppStrategyArgs {
        top_level_domain: "io.github.luxmc".into(),
        author: "luxmc".into(),
        app_name: "luxmc".into(),
    })
    .map(|s| s.data_dir())
    .unwrap_or_else(|_| etcetera::home_dir().unwrap_or_default().join(".local/share/luxmc"));

    let dest_dir = data_dir.join("instances").join(&safe_name);
    tokio::fs::create_dir_all(&dest_dir).await?;

    let copy_candidates = vec!["mods", "config", "saves", "resourcepacks", "shaderpacks", "options.txt"];
    let inner_mc = if src.join(".minecraft").is_dir() {
        src.join(".minecraft")
    } else {
        src.clone()
    };

    let mut imported_mods = 0;
    for item in copy_candidates {
        let from = inner_mc.join(item);
        let to = dest_dir.join(item);
        if from.is_dir() {
            let _ = fs_extra::dir::copy(&from, &dest_dir, &fs_extra::dir::CopyOptions::new().overwrite(true));
            if item == "mods" {
                imported_mods = count_mods_in_dir(&to);
            }
        } else if from.is_file() {
            let _ = tokio::fs::copy(&from, &to).await;
        }
    }

    let final_mc_version = mcVersion.unwrap_or_else(|| "1.20.1".into());
    let final_loader = loader.unwrap_or_else(|| "fabric".into());

    let db = crate::db::shared_db().await?;
    let now = chrono::Utc::now().to_rfc3339();

    sqlx::query(
        "INSERT INTO profiles (id, name, mc_version, loader, game_dir, mod_count, created_at, updated_at) VALUES (?, ?, ?, ?, ?, ?, ?, ?)"
    )
    .bind(&profile_id)
    .bind(&safe_name)
    .bind(&final_mc_version)
    .bind(&final_loader)
    .bind(dest_dir.to_string_lossy().to_string())
    .bind(imported_mods as i64)
    .bind(&now)
    .bind(&now)
    .execute(db.pool())
    .await?;

    Ok(ImportResult {
        success: true,
        profile_id,
        name: safe_name,
        message: format!("Instância importada com sucesso com {} mods!", imported_mods),
    })
}

fn count_mods_in_dir(dir: &Path) -> usize {
    if !dir.is_dir() {
        return 0;
    }
    std::fs::read_dir(dir)
        .map(|entries| {
            entries
                .flatten()
                .filter(|e| {
                    let p = e.path();
                    p.is_file() && p.extension().map(|ext| ext == "jar").unwrap_or(false)
                })
                .count()
        })
        .unwrap_or(0)
}

fn parse_prism_instance_cfg(dir: &Path) -> (String, String) {
    let cfg_path = dir.join("instance.cfg");
    let mut version = "1.20.1".to_string();
    let mut loader = "fabric".to_string();

    if let Ok(content) = std::fs::read_to_string(cfg_path) {
        for line in content.lines() {
            if let Some(v) = line.strip_prefix("IntendedVersion=") {
                version = v.trim().to_string();
            } else if line.contains("Fabric") {
                loader = "fabric".to_string();
            } else if line.contains("Forge") {
                loader = "forge".to_string();
            } else if line.contains("NeoForge") {
                loader = "neoforge".to_string();
            } else if line.contains("Quilt") {
                loader = "quilt".to_string();
            }
        }
    }
    (version, loader)
}

fn parse_curseforge_manifest(dir: &Path) -> (String, String) {
    let manifest_path = dir.join("minecraftinstance.json");
    if let Ok(content) = std::fs::read_to_string(manifest_path) {
        if let Ok(val) = serde_json::from_str::<serde_json::Value>(&content) {
            let version = val["gameVersion"].as_str().unwrap_or("1.20.1").to_string();
            let loader = if let Some(loaders) = val["modLoaders"].as_array() {
                loaders.first().and_then(|l| l["id"].as_str()).unwrap_or("fabric").to_lowercase()
            } else {
                "fabric".to_string()
            };
            let clean_loader = if loader.contains("forge") && !loader.contains("neoforge") {
                "forge"
            } else if loader.contains("neoforge") {
                "neoforge"
            } else {
                "fabric"
            };
            return (version, clean_loader.to_string());
        }
    }
    ("1.20.1".to_string(), "fabric".to_string())
}
