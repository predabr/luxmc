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
    #[serde(default)]
    pub icon: Option<String>,
    #[serde(default)]
    pub ram_mb: Option<i64>,
}

const MAX_ICON_DATA_URL: usize = 6_000_000;

fn data_url_from_file(path: &Path) -> Option<String> {
    let mime = match path.extension().and_then(|ext| ext.to_str()).map(|ext| ext.to_ascii_lowercase()).as_deref() {
        Some("png") => "image/png",
        Some("jpg") | Some("jpeg") => "image/jpeg",
        Some("gif") => "image/gif",
        Some("webp") => "image/webp",
        Some("bmp") => "image/bmp",
        _ => return None,
    };
    let bytes = std::fs::read(path).ok()?;
    if bytes.is_empty() || bytes.len() > MAX_ICON_DATA_URL / 4 * 3 { return None; }
    use base64::Engine as _;
    Some(format!("data:{mime};base64,{}", base64::engine::general_purpose::STANDARD.encode(bytes)))
}

fn sanitize_icon(icon: Option<String>) -> Option<String> {
    let trimmed = icon?.trim().to_string();
    if trimmed.is_empty() || trimmed.len() > MAX_ICON_DATA_URL { return None; }
    let builtin = trimmed.len() <= 64 && trimmed.bytes().all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'-'));
    let accepted = trimmed.starts_with("data:image/")
        || trimmed.starts_with("http://")
        || trimmed.starts_with("https://")
        || (trimmed.starts_with('/') && trimmed.len() < 512 && trimmed.bytes().all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'/' | b'.' | b'_' | b'-')));
    (accepted || builtin).then_some(trimmed)
}

fn resolve_prism_icon(instance_dir: &Path, icon_key: &str) -> Option<String> {
    let key = icon_key.trim().to_string();
    if key.is_empty() || key.contains("..") || key.contains('/') || key.contains('\\') { return None; }
    let mut roots: Vec<PathBuf> = Vec::new();
    if let Some(instances) = instance_dir.parent() {
        roots.push(instances.join("icons"));
        if let Some(root) = instances.parent() { roots.push(root.join("icons")); }
    }
    roots.push(instance_dir.join("icons"));
    for root in &roots {
        for candidate in [root.join(format!("{key}.png")), root.join(format!("{key}.jpg")), root.join(format!("{key}.jpeg")), root.join(format!("{key}.webp")), root.join(format!("{key}.gif")), root.join(&key)] {
            if candidate.is_file() {
                if let Some(url) = data_url_from_file(&candidate) { return Some(url); }
            }
        }
    }
    match key.as_str() {
        "grass" | "grass_block" => Some("grass_block".into()),
        _ => None,
    }
}

struct PrismExtras {
    name: Option<String>,
    icon: Option<String>,
    ram_mb: Option<i64>,
}

fn read_prism_extras(dir: &Path) -> PrismExtras {
    let mut name: Option<String> = None;
    let mut icon_key: Option<String> = None;
    let mut ram_mb: Option<i64> = None;
    if let Ok(content) = std::fs::read_to_string(dir.join("instance.cfg")) {
        for line in content.lines() {
            if let Some(value) = line.strip_prefix("name=") {
                let value = value.trim();
                if !value.is_empty() { name = Some(value.to_string()); }
            } else if let Some(value) = line.strip_prefix("iconKey=") {
                icon_key = Some(value.trim().to_string());
            } else if let Some(value) = line.strip_prefix("MaxMemAlloc=") {
                ram_mb = value.trim().parse::<i64>().ok().filter(|ram| *ram > 0);
            } else if ram_mb.is_none() {
                if let Some(value) = line.strip_prefix("AllocatedRam=") {
                    ram_mb = value.trim().parse::<i64>().ok().filter(|ram| *ram > 0);
                }
            }
        }
    }
    PrismExtras { name, icon: icon_key.and_then(|key| resolve_prism_icon(dir, &key)), ram_mb }
}

struct CurseForgeExtras {
    ram_mb: Option<i64>,
    icon: Option<String>,
}

fn read_curseforge_extras(dir: &Path) -> CurseForgeExtras {
    let ram_mb = std::fs::read_to_string(dir.join("minecraftinstance.json")).ok().and_then(|content| {
        serde_json::from_str::<serde_json::Value>(&content).ok().and_then(|value| {
            value.get("allocatedMemory").and_then(|memory| memory.as_i64()).filter(|ram| *ram > 0 && *ram < 65536)
        })
    });
    let icon = ["instance.png", "icon.png", "logo.png", "cover.png", "instance.jpg", "icon.jpg", "logo.jpg"]
        .iter()
        .find_map(|name| { let candidate = dir.join(name); candidate.is_file().then(|| data_url_from_file(&candidate)).flatten() });
    CurseForgeExtras { ram_mb, icon }
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
pub async fn importer_inspect_directory(path: String) -> AppResult<ExternalInstance> {
    tokio::task::spawn_blocking(move || {
        let directory = PathBuf::from(path);
        if !directory.is_dir() { return Err(crate::error::AppError::NotFound("Pasta de origem não encontrada".into())); }
        let game = game_directory(&directory);
        let is_prism = directory.join("mmc-pack.json").is_file() || directory.join("instance.cfg").is_file();
        let (version, loader) = if is_prism {
            parse_prism_instance_cfg(&directory)
        } else if directory.join("minecraftinstance.json").is_file() { parse_curseforge_manifest(&directory) }
        else { (String::new(), "vanilla".to_owned()) };
        let name = directory.file_name().unwrap_or_default().to_string_lossy().into_owned();
        let (icon, ram_mb) = if is_prism {
            let extras = read_prism_extras(&directory);
            (extras.icon, extras.ram_mb)
        } else if directory.join("minecraftinstance.json").is_file() {
            let extras = read_curseforge_extras(&directory);
            (extras.icon, extras.ram_mb)
        } else { (None, None) };
        Ok(ExternalInstance { launcher: "Pasta selecionada".into(), name, path: directory.to_string_lossy().into_owned(), mc_version: version, loader, mod_count: count_mods_in_dir(&game.join("mods")), has_saves: game.join("saves").is_dir(), icon, ram_mb })
    }).await.map_err(|error| crate::error::AppError::Internal(error.to_string()))?
}

fn game_directory(source: &Path) -> PathBuf {
    for name in [".minecraft", "minecraft"] { if source.join(name).is_dir() { return source.join(name); } }
    source.to_path_buf()
}

fn copy_game_data(source: &Path, destination: &Path) -> AppResult<()> {
    let mut pending = vec![(source.to_path_buf(), destination.to_path_buf())];
    while let Some((from, to)) = pending.pop() {
        let kind = std::fs::symlink_metadata(&from)?.file_type();
        if kind.is_symlink() { continue; }
        if kind.is_dir() {
            std::fs::create_dir_all(&to)?;
            for entry in std::fs::read_dir(from)? { let entry = entry?; pending.push((entry.path(), to.join(entry.file_name()))); }
        } else if kind.is_file() { std::fs::copy(from, to)?; }
    }
    Ok(())
}

#[tauri::command]
pub async fn importer_detect_launchers() -> AppResult<Vec<ExternalInstance>> {
    tokio::task::spawn_blocking(move || -> AppResult<Vec<ExternalInstance>> {
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
                            let extras = read_prism_extras(&p);
                            let name = extras.name.clone().filter(|value| !value.trim().is_empty()).unwrap_or(name);

                            instances.push(ExternalInstance {
                                launcher: "Prism Launcher".into(),
                                name,
                                path: p.to_string_lossy().to_string(),
                                mc_version,
                                loader,
                                mod_count,
                                has_saves,
                                icon: extras.icon,
                                ram_mb: extras.ram_mb,
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
                            let extras = read_curseforge_extras(&p);

                            instances.push(ExternalInstance {
                                launcher: "CurseForge".into(),
                                name,
                                path: p.to_string_lossy().to_string(),
                                mc_version,
                                loader,
                                mod_count,
                                has_saves,
                                icon: extras.icon,
                                ram_mb: extras.ram_mb,
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
                    icon: None,
                    ram_mb: None,
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
                icon: None,
                ram_mb: None,
            });
        }

        Ok(instances)
    })
    .await
    .map_err(|error| crate::error::AppError::Internal(error.to_string()))?
}

#[tauri::command]
#[allow(non_snake_case)]
pub async fn importer_execute_import(
    state: State<'_, AppState>,
    sourcePath: String,
    targetName: String,
    mcVersion: Option<String>,
    loader: Option<String>,
    icon: Option<String>,
    ramMb: Option<i64>,
) -> AppResult<ImportResult> {
    let src = PathBuf::from(&sourcePath);
    if !src.is_dir() {
        return Err(crate::error::AppError::NotFound("Diretório de origem não encontrado".into()));
    }

    let final_mc_version = mcVersion.filter(|value| !value.trim().is_empty()).ok_or_else(|| crate::error::AppError::InvalidInput("Selecione a versão do Minecraft da instância".into()))?;
    if final_mc_version.len() > 128 || final_mc_version.contains("..") || !final_mc_version.bytes().all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'-' | b'_')) { return Err(crate::error::AppError::InvalidInput("Versão do Minecraft inválida".into())); }
    let final_loader = loader.unwrap_or_else(|| "vanilla".into());
    let mut final_icon = sanitize_icon(icon);
    if final_icon.is_none() {
        if let Some((source, project)) = manifest_project(&game_directory(&src)) {
            final_icon = catalog_icon(&state, source, &project).await.and_then(|url| sanitize_icon(Some(url)));
        }
    }
    let final_ram = ramMb.filter(|ram| *ram > 0 && *ram < 65536);

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

    let dest_dir = data_dir.join("instances").join(&profile_id).join(".minecraft");
    tokio::fs::create_dir_all(&dest_dir).await?;

    let copy_candidates = vec!["mods", "coremods", "openloader", "config", "defaultconfigs", "kubejs", "scripts", "saves", "resourcepacks", "shaderpacks", "screenshots", "journeymap", "packmenu", "custommainmenu", "options.txt", "servers.dat"];
    let inner_mc = game_directory(&src);
    let copy_destination = dest_dir.clone();
    let imported_mods = tokio::task::spawn_blocking(move || -> AppResult<usize> {
    for item in copy_candidates {
        let from = inner_mc.join(item);
        let to = copy_destination.join(item);
        if from.is_dir() {
            copy_game_data(&from, &to)?;
        } else if from.is_file() {
            std::fs::copy(&from, &to)?;
        }
    }
    Ok(count_mods_in_dir(&copy_destination.join("mods")))
    }).await.map_err(|error| crate::error::AppError::Internal(error.to_string()))??;


    let db = crate::db::shared_db().await?;
    let now = chrono::Utc::now().to_rfc3339();

    sqlx::query(
        "INSERT INTO profiles (id, name, mc_version, loader, game_dir, mod_count, ram_mb, icon, created_at, updated_at) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)"
    )
    .bind(&profile_id)
    .bind(&safe_name)
    .bind(&final_mc_version)
    .bind(&final_loader)
    .bind(dest_dir.to_string_lossy().to_string())
    .bind(imported_mods as i64)
    .bind(final_ram)
    .bind(final_icon.filter(|value| !value.trim().is_empty()).unwrap_or_else(|| "grass_block".into()))
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

fn manifest_project(dir: &Path) -> Option<(&'static str, String)> {
    for file in ["minecraftinstance.json", "manifest.json"] {
        if let Ok(bytes) = std::fs::read(dir.join(file)) {
            if let Ok(value) = serde_json::from_slice::<serde_json::Value>(&bytes) {
                for pointer in ["/installedModpack/projectID", "/installedModpack/projectId", "/installedModpack/id", "/projectID", "/projectId"] {
                    if let Some(id) = value.pointer(pointer).and_then(|value| value.as_u64()) { return Some(("curseforge", id.to_string())); }
                }
            }
        }
    }
    if let Ok(bytes) = std::fs::read(dir.join("modrinth.index.json")) {
        if let Ok(value) = serde_json::from_slice::<serde_json::Value>(&bytes) {
            if let Some(id) = value["projectId"].as_str().map(str::trim).filter(|id| !id.is_empty() && id.len() < 64) { return Some(("modrinth", id.to_owned())); }
        }
    }
    None
}

async fn catalog_icon(state: &AppState, source: &str, project: &str) -> Option<String> {
    let details = if source == "curseforge" {
        crate::core::mods::curseforge::get_mod_details(&state.http, project).await.ok()
    } else {
        crate::core::mods::ModrinthClient::new(state.http.clone()).get_project_details(project).await.ok()
    };
    details.and_then(|details| details.icon_url)
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
    let mut version = String::new();
    let mut loader = "vanilla".to_string();

    if let Ok(content) = std::fs::read_to_string(cfg_path) {
        for line in content.lines() {
            if let Some(v) = line.strip_prefix("IntendedVersion=") {
                version = v.trim().to_string();
            }
            let lower = line.to_lowercase();
            if lower.contains("neoforge") {
                loader = "neoforge".to_string();
            } else if lower.contains("fabric") {
                loader = "fabric".to_string();
            } else if lower.contains("quilt") {
                loader = "quilt".to_string();
            } else if lower.contains("forge") {
                loader = "forge".to_string();
            }
        }
    }

    if let Ok(content) = std::fs::read_to_string(dir.join("mmc-pack.json")) {
        if let Ok(val) = serde_json::from_str::<serde_json::Value>(&content) {
            if let Some(components) = val.get("components").and_then(|c| c.as_array()) {
                for component in components {
                    let id = component.get("id").and_then(|i| i.as_str()).unwrap_or_default();
                    if id == "net.minecraft" && version.is_empty() {
                        version = component
                            .get("version")
                            .and_then(|v| v.as_str())
                            .unwrap_or_default()
                            .to_string();
                        continue;
                    }
                    let mapped = match id {
                        "net.neoforged" | "net.neoforged.modloader" => Some("neoforge"),
                        "net.fabricmc.fabric-loader" => Some("fabric"),
                        "org.quiltmc.quilt-loader" => Some("quilt"),
                        "net.minecraftforge" | "net.minecraftforge.fml" => Some("forge"),
                        _ => None,
                    };
                    if let Some(mapped) = mapped {
                        loader = mapped.to_string();
                    }
                }
            }
        }
    }

    (version, loader)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn generic_minecraft_folder_preserves_nested_worlds_and_scripts() {
        let root=std::env::temp_dir().join(format!("luxmc-import-{}",uuid::Uuid::new_v4()));
        let source=root.join("source/minecraft");
        std::fs::create_dir_all(source.join("saves/World/datapacks")).unwrap();
        std::fs::write(source.join("saves/World/datapacks/example.zip"),b"world data").unwrap();
        std::fs::create_dir_all(source.join("kubejs")).unwrap();
        std::fs::write(source.join("kubejs/server.js"),b"script").unwrap();
        assert_eq!(game_directory(&root.join("source")),source);
        copy_game_data(&source,&root.join("destination")).unwrap();
        assert_eq!(std::fs::read(root.join("destination/saves/World/datapacks/example.zip")).unwrap(),b"world data");
        assert_eq!(std::fs::read(root.join("destination/kubejs/server.js")).unwrap(),b"script");
        assert!(source.join("kubejs/server.js").is_file());
        std::fs::remove_dir_all(root).unwrap();
    }
}

fn parse_curseforge_manifest(dir: &Path) -> (String, String) {
    let manifest_path = dir.join("minecraftinstance.json");
    if let Ok(content) = std::fs::read_to_string(manifest_path) {
        if let Ok(val) = serde_json::from_str::<serde_json::Value>(&content) {
            let version = val["gameVersion"].as_str().or_else(|| val["minecraftVersion"].as_str()).or_else(|| val.pointer("/baseModLoader/minecraftVersion").and_then(|v| v.as_str())).unwrap_or_default().to_string();
            let loader = if let Some(loaders) = val["modLoaders"].as_array() {
                loaders.first().and_then(|l| l["id"].as_str()).or_else(|| val.pointer("/baseModLoader/name").and_then(|v| v.as_str())).unwrap_or("vanilla").to_lowercase()
            } else {
                val.pointer("/baseModLoader/name").and_then(|v| v.as_str()).unwrap_or("vanilla").to_string()
            };
            let clean_loader = if loader.contains("neoforge") {
                "neoforge"
            } else if loader.contains("quilt") {
                "quilt"
            } else if loader.contains("forge") {
                "forge"
            } else if loader.contains("fabric") {
                "fabric"
            } else {
                "vanilla"
            };
            return (version, clean_loader.to_string());
        }
    }
    (String::new(), "vanilla".to_string())
}
