#![allow(non_snake_case)]

use crate::db;
use crate::db::models::ProfileRow;
use crate::error::{AppError, AppResult};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashSet;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::sync::{LazyLock, Mutex};

const CAPSULE_DIRS: [&str; 5] = ["saves", "mods", "config", "shaderpacks", "resourcepacks"];
const CAPSULE_FILES: [&str; 1] = ["options.txt"];
const MAX_CAPSULE_BYTES: u64 = 8 * 1024 * 1024 * 1024;
const MAX_CAPSULE_FILES: usize = 100_000;
static LAB_LOCK: LazyLock<Mutex<()>> = LazyLock::new(|| Mutex::new(()));

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CapsuleInfo {
    pub filename: String,
    pub label: String,
    pub created_at: String,
    pub size_bytes: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct CapsuleManifest {
    format_version: u32,
    profile_id: String,
    label: String,
    created_at: String,
    mc_version: String,
    loader: String,
    loader_version: Option<String>,
    ram_mb: Option<i64>,
    jvm_args: Option<String>,
}

async fn profile(profile_id: &str) -> AppResult<ProfileRow> {
    let database = db::shared_db().await?;
    sqlx::query_as::<_, ProfileRow>("SELECT * FROM profiles WHERE id = ?")
        .bind(profile_id)
        .fetch_optional(database.pool())
        .await?
        .ok_or_else(|| AppError::NotFound(format!("Instância {profile_id} não encontrada")))
}

fn capsule_directory(profile_id: &str) -> AppResult<PathBuf> {
    let project = directories::ProjectDirs::from("io", "github", "Luxmc")
        .ok_or_else(|| AppError::InvalidState("Diretório de dados indisponível".into()))?;
    let key = format!("{:x}", Sha256::digest(profile_id.as_bytes()));
    Ok(project.data_dir().join("capsules").join(key))
}

fn safe_label(input: &str) -> String {
    let label: String = input.chars().filter(|value| value.is_alphanumeric() || *value == ' ' || *value == '-' || *value == '_').take(48).collect();
    let label = label.trim();
    if label.is_empty() { "Ponto de restauração".into() } else { label.into() }
}

fn append_files(zip: &mut zip::ZipWriter<std::fs::File>, root: &Path, current: &Path, total: &mut u64, count: &mut usize) -> AppResult<()> {
    if current.is_dir() {
        for entry in std::fs::read_dir(current)? {
            let entry = entry?;
            if entry.file_type()?.is_symlink() { continue; }
            append_files(zip, root, &entry.path(), total, count)?;
        }
    } else if current.is_file() {
        let metadata = current.metadata()?;
        *total = total.saturating_add(metadata.len());
        *count += 1;
        if *total > MAX_CAPSULE_BYTES || *count > MAX_CAPSULE_FILES {
            return Err(AppError::InvalidInput("Cápsula excede o limite de 8 GB ou 100 mil arquivos".into()));
        }
        let relative = current.strip_prefix(root).map_err(|error| AppError::Internal(error.to_string()))?;
        let name = relative.to_string_lossy().replace('\\', "/");
        zip.start_file(name, zip::write::FileOptions::default().compression_method(zip::CompressionMethod::Deflated))?;
        let mut source = std::fs::File::open(current)?;
        std::io::copy(&mut source, zip)?;
    }
    Ok(())
}

fn create_capsule_inner(row: &ProfileRow, label: &str) -> AppResult<CapsuleInfo> {
    if crate::core::launcher::get_active_game_pid() != 0 {
        return Err(AppError::InvalidState("Feche o Minecraft antes de criar uma cápsula".into()));
    }
    if read_isolation(&row.id)?.is_some() {
        return Err(AppError::InvalidState("Finalize o diagnóstico de mods antes de criar uma cápsula".into()));
    }
    let directory = capsule_directory(&row.id)?;
    std::fs::create_dir_all(&directory)?;
    let filename = format!("capsule-{}.zip", uuid::Uuid::new_v4());
    let target = directory.join(&filename);
    let temporary = directory.join(format!("{filename}.part"));
    let created_at = chrono::Utc::now().to_rfc3339();
    let manifest = CapsuleManifest {
        format_version: 1,
        profile_id: row.id.clone(),
        label: safe_label(label),
        created_at: created_at.clone(),
        mc_version: row.mc_version.clone(),
        loader: row.loader.clone(),
        loader_version: row.loader_version.clone(),
        ram_mb: row.ram_mb,
        jvm_args: row.jvm_args.clone(),
    };
    let result = (|| -> AppResult<()> {
        let file = std::fs::File::create(&temporary)?;
        let mut zip = zip::ZipWriter::new(file);
        zip.start_file("luxmc.capsule.json", zip::write::FileOptions::default())?;
        zip.write_all(serde_json::to_string(&manifest)?.as_bytes())?;
        let game_dir = Path::new(&row.game_dir);
        let mut total = 0;
        let mut count = 0;
        for name in CAPSULE_DIRS.into_iter().chain(CAPSULE_FILES) {
            let path = game_dir.join(name);
            if path.is_symlink() { continue; }
            if path.exists() { append_files(&mut zip, game_dir, &path, &mut total, &mut count)?; }
        }
        zip.finish()?;
        std::fs::rename(&temporary, &target)?;
        Ok(())
    })();
    if result.is_err() { let _ = std::fs::remove_file(&temporary); }
    result?;
    Ok(CapsuleInfo { filename, label: manifest.label, created_at, size_bytes: target.metadata()?.len() })
}

#[tauri::command]
pub async fn instance_capsule_create(profileId: String, label: String) -> AppResult<CapsuleInfo> {
    let row = profile(&profileId).await?;
    tokio::task::spawn_blocking(move || create_capsule_inner(&row, &label)).await.map_err(|error| AppError::Internal(error.to_string()))?
}

#[tauri::command]
pub async fn instance_capsules_list(profileId: String) -> AppResult<Vec<CapsuleInfo>> {
    let _ = profile(&profileId).await?;
    tokio::task::spawn_blocking(move || -> AppResult<Vec<CapsuleInfo>> {
        let directory = capsule_directory(&profileId)?;
        let mut output = Vec::new();
        if !directory.exists() { return Ok(output); }
        for entry in std::fs::read_dir(directory)? {
            let entry = entry?;
            let filename = entry.file_name().to_string_lossy().to_string();
            if !filename.starts_with("capsule-") || !filename.ends_with(".zip") { continue; }
            let file = std::fs::File::open(entry.path())?;
            let Ok(mut archive) = zip::ZipArchive::new(file) else { continue; };
            let manifest = archive.by_name("luxmc.capsule.json").ok().and_then(|item| {
                let mut text = String::new();
                item.take(16 * 1024).read_to_string(&mut text).ok()?;
                serde_json::from_str::<CapsuleManifest>(&text).ok()
            });
            if let Some(manifest) = manifest.filter(|item| item.format_version == 1 && item.profile_id == profileId) {
                output.push(CapsuleInfo { filename, label: manifest.label, created_at: manifest.created_at, size_bytes: entry.metadata()?.len() });
            }
        }
        output.sort_by(|a, b| b.created_at.cmp(&a.created_at));
        Ok(output)
    }).await.map_err(|error| AppError::Internal(error.to_string()))?
}

fn restore_capsule_inner(row: &ProfileRow, filename: &str) -> AppResult<(CapsuleInfo, CapsuleManifest)> {
    if crate::core::launcher::get_active_game_pid() != 0 {
        return Err(AppError::InvalidState("Feche o Minecraft antes de restaurar uma cápsula".into()));
    }
    if read_isolation(&row.id)?.is_some() {
        return Err(AppError::InvalidState("Finalize o diagnóstico de mods antes de restaurar uma cápsula".into()));
    }
    if !filename.starts_with("capsule-") || !filename.ends_with(".zip") || filename.contains('/') || filename.contains('\\') || filename.contains("..") {
        return Err(AppError::InvalidInput("Nome de cápsula inválido".into()));
    }
    let source = capsule_directory(&row.id)?.join(filename);
    let file = std::fs::File::open(&source)?;
    let mut archive = zip::ZipArchive::new(file)?;
    let manifest: CapsuleManifest = {
        let mut item = archive.by_name("luxmc.capsule.json")?;
        if item.size() > 16 * 1024 { return Err(AppError::InvalidInput("Manifesto da cápsula excede o limite".into())); }
        let mut text = String::new();
        item.read_to_string(&mut text)?;
        serde_json::from_str(&text)?
    };
    if manifest.format_version != 1 || manifest.profile_id != row.id {
        return Err(AppError::InvalidInput("Cápsula de outra instância ou versão inválida".into()));
    }
    let game_dir = Path::new(&row.game_dir);
    let stage = game_dir.join(format!(".luxmc-capsule-stage-{}", uuid::Uuid::new_v4()));
    std::fs::create_dir_all(&stage)?;
    let result = (|| -> AppResult<(CapsuleInfo, CapsuleManifest)> {
        let mut total = 0u64;
        let mut names = HashSet::new();
        for index in 0..archive.len() {
            let mut item = archive.by_index(index)?;
            if item.name() == "luxmc.capsule.json" { continue; }
            total = total.saturating_add(item.size());
            if total > MAX_CAPSULE_BYTES || index > MAX_CAPSULE_FILES || item.unix_mode().is_some_and(|mode| mode & 0o170000 == 0o120000) {
                return Err(AppError::InvalidInput("Cápsula excede os limites de segurança".into()));
            }
            let name = item.name().replace('\\', "/");
            let relative = Path::new(&name);
            let top = relative.components().next().and_then(|part| part.as_os_str().to_str()).unwrap_or("");
            if !CAPSULE_DIRS.contains(&top) && !CAPSULE_FILES.contains(&top) {
                return Err(AppError::InvalidInput("Cápsula contém arquivo fora das pastas permitidas".into()));
            }
            if CAPSULE_FILES.contains(&top) && relative.components().count() != 1 {
                return Err(AppError::InvalidInput("Caminho inválido em opções".into()));
            }
            if !names.insert(name.clone()) { return Err(AppError::InvalidInput("Cápsula contém caminhos duplicados".into())); }
            let destination = crate::core::mods::pack_download::destination(&stage, &name)?;
            if item.is_dir() { std::fs::create_dir_all(&destination)?; }
            else {
                if let Some(parent) = destination.parent() { std::fs::create_dir_all(parent)?; }
                let mut output = std::fs::File::create(destination)?;
                std::io::copy(&mut item, &mut output)?;
            }
        }
        let backup = create_capsule_inner(row, "Antes da restauração")?;
        let rollback = game_dir.join(format!(".luxmc-capsule-rollback-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&rollback)?;
        let mut moved_old = Vec::new();
        let mut moved_new = Vec::new();
        let swap = (|| -> AppResult<()> {
            for name in CAPSULE_DIRS.into_iter().chain(CAPSULE_FILES) {
                let current = game_dir.join(name);
                if current.exists() {
                    std::fs::rename(&current, rollback.join(name))?;
                    moved_old.push(name);
                }
                let replacement = stage.join(name);
                if replacement.exists() {
                    std::fs::rename(&replacement, &current)?;
                    moved_new.push(name);
                }
            }
            Ok(())
        })();
        if let Err(error) = swap {
            for name in moved_new.into_iter().rev() { let _ = std::fs::rename(game_dir.join(name), stage.join(name)); }
            for name in moved_old.into_iter().rev() { let _ = std::fs::rename(rollback.join(name), game_dir.join(name)); }
            return Err(error);
        }
        std::fs::remove_dir_all(&rollback)?;
        Ok((backup, manifest))
    })();
    let _ = std::fs::remove_dir_all(&stage);
    result
}

#[tauri::command]
pub async fn instance_capsule_restore(profileId: String, filename: String) -> AppResult<CapsuleInfo> {
    let original = profile(&profileId).await?;
    let database = db::shared_db().await?;
    let restore_row = original.clone();
    let (backup, manifest) = tokio::task::spawn_blocking(move || restore_capsule_inner(&restore_row, &filename)).await.map_err(|error| AppError::Internal(error.to_string()))??;
    let mut restored = original.clone();
    restored.mc_version = manifest.mc_version;
    restored.loader = manifest.loader;
    restored.loader_version = manifest.loader_version;
    restored.ram_mb = manifest.ram_mb;
    restored.jvm_args = manifest.jvm_args;
    restored.updated_at = chrono::Utc::now();
    if let Err(error) = db::schema::profiles::upsert(&database, &restored).await {
        let rollback_name = backup.filename.clone();
        let rollback = tokio::task::spawn_blocking(move || restore_capsule_inner(&original, &rollback_name)).await;
        return Err(AppError::InvalidState(format!("Falha ao restaurar configurações: {error}. Reversão de arquivos: {}", if matches!(rollback, Ok(Ok(_))) { "concluída" } else { "falhou; use a cápsula anterior" })));
    }
    Ok(backup)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IsolationState {
    pub profile_id: String,
    pub original_enabled: Vec<String>,
    pub suspects: Vec<String>,
    pub trial_disabled: Vec<String>,
    pub round: u32,
    pub phase: String,
}

fn isolation_file(profile_id: &str) -> AppResult<PathBuf> {
    let project = directories::ProjectDirs::from("io", "github", "Luxmc")
        .ok_or_else(|| AppError::InvalidState("Diretório de dados indisponível".into()))?;
    let key = format!("{:x}", Sha256::digest(profile_id.as_bytes()));
    Ok(project.data_dir().join("mod_isolation").join(format!("{key}.json")))
}

fn read_isolation(profile_id: &str) -> AppResult<Option<IsolationState>> {
    let path = isolation_file(profile_id)?;
    if !path.exists() { return Ok(None); }
    let state: IsolationState = serde_json::from_slice(&std::fs::read(path)?)?;
    if state.profile_id != profile_id { return Err(AppError::InvalidState("Sessão de diagnóstico inválida".into())); }
    Ok(Some(state))
}

pub fn isolation_testing(profile_id: &str) -> AppResult<bool> {
    Ok(read_isolation(profile_id)?.is_some_and(|state| state.phase == "testing"))
}

pub fn ensure_isolation_idle(profile_id: &str) -> AppResult<()> {
    if isolation_testing(profile_id)? {
        Err(AppError::InvalidState("Finalize o diagnóstico de mods antes de alterar a instância".into()))
    } else {
        Ok(())
    }
}

fn write_isolation(state: &IsolationState) -> AppResult<()> {
    let path = isolation_file(&state.profile_id)?;
    if let Some(parent) = path.parent() { std::fs::create_dir_all(parent)?; }
    let temporary = path.with_extension(format!("{}.part", uuid::Uuid::new_v4()));
    std::fs::write(&temporary, serde_json::to_vec(state)?)?;
    std::fs::rename(temporary, path)?;
    Ok(())
}

fn apply_trial(mods_dir: &Path, state: &IsolationState) -> AppResult<()> {
    let disabled: HashSet<&str> = state.trial_disabled.iter().map(String::as_str).collect();
    if !disabled.is_empty() {
        let original: HashSet<&str> = state.original_enabled.iter().map(String::as_str).collect();
        for entry in std::fs::read_dir(mods_dir)? {
            let entry = entry?;
            let name = entry.file_name().to_string_lossy().to_string();
            if entry.file_type()?.is_file() && name.ends_with(".jar") && !original.contains(name.as_str()) {
                return Err(AppError::InvalidState(format!("Novo mod encontrado durante o diagnóstico: {name}. Restaure os mods e reinicie o teste")));
            }
        }
    }
    let mut moves = Vec::new();
    for filename in &state.original_enabled {
        if !filename.ends_with(".jar") || filename.contains('/') || filename.contains('\\') || filename.contains("..") {
            return Err(AppError::InvalidState("Arquivo de mod inválido na sessão".into()));
        }
        let enabled = mods_dir.join(filename);
        let inactive = mods_dir.join(format!("{filename}.disabled"));
        let (source, target) = if disabled.contains(filename.as_str()) { (&enabled, &inactive) } else { (&inactive, &enabled) };
        if target.exists() {
            if source.exists() { return Err(AppError::InvalidState(format!("Arquivos duplicados para {filename}"))); }
            continue;
        }
        if !source.is_file() || source.is_symlink() {
            return Err(AppError::InvalidState(format!("Mod alterado fora do diagnóstico: {filename}")));
        }
        moves.push((source.to_path_buf(), target.to_path_buf()));
    }
    let mut completed = Vec::new();
    for (source, target) in moves {
        if let Err(error) = std::fs::rename(&source, &target) {
            for (from, to) in completed.into_iter().rev() { let _ = std::fs::rename(to, from); }
            return Err(error.into());
        }
        completed.push((source, target));
    }
    Ok(())
}

fn ensure_game_closed() -> AppResult<()> {
    if crate::core::launcher::get_active_game_pid() != 0 {
        return Err(AppError::InvalidState("Feche o Minecraft antes de alterar os mods".into()));
    }
    Ok(())
}

#[tauri::command]
pub async fn instance_isolation_status(profileId: String) -> AppResult<Option<IsolationState>> {
    let _ = profile(&profileId).await?;
    tokio::task::spawn_blocking(move || read_isolation(&profileId)).await.map_err(|error| AppError::Internal(error.to_string()))?
}

#[tauri::command]
pub async fn instance_isolation_start(profileId: String) -> AppResult<IsolationState> {
    let row = profile(&profileId).await?;
    tokio::task::spawn_blocking(move || -> AppResult<IsolationState> {
        let _guard = LAB_LOCK.lock().map_err(|error| AppError::InvalidState(error.to_string()))?;
        ensure_game_closed()?;
        if read_isolation(&row.id)?.is_some() { return Err(AppError::InvalidState("Já existe um diagnóstico em andamento".into())); }
        let mods_dir = Path::new(&row.game_dir).join("mods");
        let mut enabled = Vec::new();
        for entry in std::fs::read_dir(&mods_dir)? {
            let entry = entry?;
            if !entry.file_type()?.is_file() { continue; }
            let name = entry.file_name().to_string_lossy().to_string();
            if name.ends_with(".jar") { enabled.push(name); }
        }
        enabled.sort();
        if enabled.len() < 2 { return Err(AppError::InvalidInput("É preciso ter pelo menos dois mods ativos para isolar um conflito".into())); }
        let state = IsolationState {
            profile_id: row.id.clone(),
            suspects: enabled.clone(),
            trial_disabled: enabled[..enabled.len() / 2].to_vec(),
            original_enabled: enabled,
            round: 1,
            phase: "testing".into(),
        };
        write_isolation(&state)?;
        if let Err(error) = apply_trial(&mods_dir, &state) {
            let _ = std::fs::remove_file(isolation_file(&row.id)?);
            return Err(error);
        }
        Ok(state)
    }).await.map_err(|error| AppError::Internal(error.to_string()))?
}

#[tauri::command]
pub async fn instance_isolation_report(profileId: String, crashed: bool) -> AppResult<IsolationState> {
    let row = profile(&profileId).await?;
    tokio::task::spawn_blocking(move || -> AppResult<IsolationState> {
        let _guard = LAB_LOCK.lock().map_err(|error| AppError::InvalidState(error.to_string()))?;
        ensure_game_closed()?;
        let mut state = read_isolation(&row.id)?.ok_or_else(|| AppError::NotFound("Diagnóstico não iniciado".into()))?;
        if state.phase != "testing" { return Err(AppError::InvalidState("Diagnóstico já concluído".into())); }
        let disabled: HashSet<&str> = state.trial_disabled.iter().map(String::as_str).collect();
        state.suspects.retain(|name| disabled.contains(name.as_str()) != crashed);
        if state.suspects.is_empty() { return Err(AppError::InvalidState("Resultado incompatível com os testes anteriores. Restaure os mods e reinicie o diagnóstico".into())); }
        state.round += 1;
        if state.suspects.len() == 1 {
            state.phase = "found".into();
            state.trial_disabled.clear();
        } else {
            state.trial_disabled = state.suspects[..state.suspects.len() / 2].to_vec();
        }
        apply_trial(&Path::new(&row.game_dir).join("mods"), &state)?;
        write_isolation(&state)?;
        Ok(state)
    }).await.map_err(|error| AppError::Internal(error.to_string()))?
}

#[tauri::command]
pub async fn instance_isolation_restore(profileId: String) -> AppResult<()> {
    let row = profile(&profileId).await?;
    tokio::task::spawn_blocking(move || -> AppResult<()> {
        let _guard = LAB_LOCK.lock().map_err(|error| AppError::InvalidState(error.to_string()))?;
        ensure_game_closed()?;
        if let Some(mut state) = read_isolation(&row.id)? {
            state.trial_disabled.clear();
            apply_trial(&Path::new(&row.game_dir).join("mods"), &state)?;
            std::fs::remove_file(isolation_file(&row.id)?)?;
        }
        Ok(())
    }).await.map_err(|error| AppError::Internal(error.to_string()))?
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BenchmarkResult {
    pub id: String,
    pub label: String,
    pub created_at: String,
    pub average_fps: f64,
    pub low_one_percent_fps: f64,
    pub fps_drops: usize,
    pub samples: usize,
    pub source: String,
}

fn benchmark_file(profile_id: &str) -> AppResult<PathBuf> {
    let project = directories::ProjectDirs::from("io", "github", "Luxmc")
        .ok_or_else(|| AppError::InvalidState("Diretório de dados indisponível".into()))?;
    let key = format!("{:x}", Sha256::digest(profile_id.as_bytes()));
    Ok(project.data_dir().join("benchmarks").join(format!("{key}.json")))
}

fn read_benchmarks(profile_id: &str) -> AppResult<Vec<BenchmarkResult>> {
    let path = benchmark_file(profile_id)?;
    if !path.exists() { return Ok(Vec::new()); }
    Ok(serde_json::from_slice(&std::fs::read(path)?)?)
}

fn parse_mangohud_csv(content: &str, label: &str) -> AppResult<BenchmarkResult> {
    let mut fps_column = 0;
    let mut frametime_column = None;
    let mut fps_values = Vec::new();
    let mut frame_values = Vec::new();
    for line in content.lines() {
        let fields: Vec<&str> = line.trim().split(',').map(str::trim).collect();
        if fields.iter().any(|field| field.eq_ignore_ascii_case("fps")) {
            fps_column = fields.iter().position(|field| field.eq_ignore_ascii_case("fps")).unwrap_or(0);
            frametime_column = fields.iter().position(|field| field.eq_ignore_ascii_case("frametime") || field.eq_ignore_ascii_case("frame_time"));
            continue;
        }
        let Some(raw_fps) = fields.get(fps_column) else { continue };
        let Ok(fps) = raw_fps.parse::<f64>() else { continue };
        if !fps.is_finite() || !(1.0..=2000.0).contains(&fps) { continue; }
        fps_values.push(fps);
        if let Some(index) = frametime_column {
            frame_values.push(fields.get(index).and_then(|value| value.parse::<f64>().ok()).filter(|value| value.is_finite() && *value > 0.0).unwrap_or(0.0));
        }
    }
    if fps_values.len() < 10 {
        return Err(AppError::InvalidInput("O log precisa ter pelo menos dez amostras válidas de FPS do MangoHud".into()));
    }
    fps_values.sort_by(f64::total_cmp);
    let low_count = (fps_values.len() / 100).max(1);
    let low_one_percent_fps = fps_values[..low_count].iter().sum::<f64>() / low_count as f64;
    let average_fps = fps_values.iter().sum::<f64>() / fps_values.len() as f64;
    let median_fps = fps_values[fps_values.len() / 2];
    let fps_drops = if frame_values.iter().any(|value| *value > 0.0) {
        let mut sorted = frame_values.iter().copied().filter(|value| *value > 0.0).collect::<Vec<_>>();
        sorted.sort_by(f64::total_cmp);
        let threshold = (sorted[sorted.len() / 2] * 2.0).max(50.0);
        frame_values.iter().filter(|value| **value > threshold).count()
    } else {
        fps_values.iter().filter(|value| **value < (median_fps * 0.5).max(20.0)).count()
    };
    Ok(BenchmarkResult {
        id: uuid::Uuid::new_v4().to_string(),
        label: safe_label(label),
        created_at: chrono::Utc::now().to_rfc3339(),
        average_fps,
        low_one_percent_fps,
        fps_drops,
        samples: fps_values.len(),
        source: "MangoHud CSV".into(),
    })
}

#[tauri::command]
pub async fn instance_benchmarks_list(profileId: String) -> AppResult<Vec<BenchmarkResult>> {
    let _ = profile(&profileId).await?;
    tokio::task::spawn_blocking(move || read_benchmarks(&profileId)).await.map_err(|error| AppError::Internal(error.to_string()))?
}

#[tauri::command]
pub async fn instance_benchmark_import(profileId: String, label: String, csvPath: String) -> AppResult<BenchmarkResult> {
    let _ = profile(&profileId).await?;
    tokio::task::spawn_blocking(move || -> AppResult<BenchmarkResult> {
        let source = Path::new(&csvPath);
        let metadata = source.metadata()?;
        if !metadata.is_file() || metadata.len() > 20 * 1024 * 1024 || source.extension().and_then(|value| value.to_str()).is_none_or(|value| !value.eq_ignore_ascii_case("csv")) {
            return Err(AppError::InvalidInput("Selecione um CSV de até 20 MB".into()));
        }
        let content = std::fs::read_to_string(source)?;
        let result = parse_mangohud_csv(&content, &label)?;
        let mut benchmarks = read_benchmarks(&profileId)?;
        benchmarks.insert(0, result.clone());
        benchmarks.truncate(30);
        let path = benchmark_file(&profileId)?;
        if let Some(parent) = path.parent() { std::fs::create_dir_all(parent)?; }
        let temporary = path.with_extension(format!("{}.part", uuid::Uuid::new_v4()));
        std::fs::write(&temporary, serde_json::to_vec(&benchmarks)?)?;
        std::fs::rename(temporary, path)?;
        Ok(result)
    }).await.map_err(|error| AppError::Internal(error.to_string()))?
}

#[cfg(test)]
mod tests {
    #[test]
    fn parses_explicit_fps_and_frametime_samples() {
        let mut csv = String::from("fps,frametime,gpu_load\n");
        for _ in 0..98 { csv.push_str("100,10,50\n"); }
        csv.push_str("20,80,50\n20,80,50\n");
        let result = super::parse_mangohud_csv(&csv, "Antes").unwrap();
        assert_eq!(result.samples, 100);
        assert_eq!(result.fps_drops, 2);
        assert_eq!(result.low_one_percent_fps, 20.0);
    }

    #[test]
    fn isolation_moves_and_restores_original_jars() {
        let directory = std::env::temp_dir().join(format!("luxmc-isolation-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&directory).unwrap();
        std::fs::write(directory.join("alpha.jar"), b"alpha").unwrap();
        std::fs::write(directory.join("beta.jar"), b"beta").unwrap();
        std::fs::write(directory.join("other.jar.disabled"), b"other").unwrap();
        let mut state = super::IsolationState {
            profile_id: "test".into(),
            original_enabled: vec!["alpha.jar".into(), "beta.jar".into()],
            suspects: vec!["alpha.jar".into(), "beta.jar".into()],
            trial_disabled: vec!["alpha.jar".into()],
            round: 1,
            phase: "testing".into(),
        };
        super::apply_trial(&directory, &state).unwrap();
        assert!(!directory.join("alpha.jar").exists());
        assert_eq!(std::fs::read(directory.join("alpha.jar.disabled")).unwrap(), b"alpha");
        state.trial_disabled.clear();
        super::apply_trial(&directory, &state).unwrap();
        assert_eq!(std::fs::read(directory.join("alpha.jar")).unwrap(), b"alpha");
        assert_eq!(std::fs::read(directory.join("beta.jar")).unwrap(), b"beta");
        assert_eq!(std::fs::read(directory.join("other.jar.disabled")).unwrap(), b"other");
        std::fs::remove_dir_all(directory).unwrap();
    }

    #[test]
    fn capsule_restores_mods_world_and_options_and_keeps_previous_state() {
        let game_dir = std::env::temp_dir().join(format!("luxmc-capsule-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(game_dir.join("mods")).unwrap();
        std::fs::create_dir_all(game_dir.join("saves/World")).unwrap();
        std::fs::write(game_dir.join("mods/a.jar"), b"original jar").unwrap();
        std::fs::write(game_dir.join("saves/World/level.dat"), b"original world").unwrap();
        std::fs::write(game_dir.join("options.txt"), b"renderDistance:12").unwrap();
        let id = uuid::Uuid::new_v4().to_string();
        let now = chrono::Utc::now();
        let row = crate::db::models::ProfileRow {
            id: id.clone(), name: "Test".into(), icon: "".into(), mc_version: "1.21.1".into(), loader: "fabric".into(), loader_version: None,
            java_path: None, jvm_args: None, resolution_w: None, resolution_h: None, fullscreen: false,
            game_dir: game_dir.to_string_lossy().into_owned(), created_at: now, updated_at: now,
            favorite: false, notes: None, last_played: None, launch_count: 0, mod_count: 1, disk_usage: 0,
            ram_mb: Some(4096), instance_group: None, auto_optimize: true, use_vulkan: false,
            use_gamemode: false, use_mangohud: false, force_dedicated_gpu: false, use_gamescope: false,
            gamescope_width: None, gamescope_height: None, gamescope_fsr: false, force_full_verification: false,
        };
        let original = super::create_capsule_inner(&row, "Original").unwrap();
        std::fs::write(game_dir.join("mods/a.jar"), b"changed jar").unwrap();
        std::fs::write(game_dir.join("saves/World/level.dat"), b"changed world").unwrap();
        std::fs::write(game_dir.join("options.txt"), b"renderDistance:2").unwrap();
        let previous = super::restore_capsule_inner(&row, &original.filename).unwrap().0;
        assert_eq!(std::fs::read(game_dir.join("mods/a.jar")).unwrap(), b"original jar");
        assert_eq!(std::fs::read(game_dir.join("saves/World/level.dat")).unwrap(), b"original world");
        assert_eq!(std::fs::read(game_dir.join("options.txt")).unwrap(), b"renderDistance:12");
        super::restore_capsule_inner(&row, &previous.filename).unwrap();
        assert_eq!(std::fs::read(game_dir.join("mods/a.jar")).unwrap(), b"changed jar");
        std::fs::remove_dir_all(game_dir).unwrap();
        std::fs::remove_dir_all(super::capsule_directory(&id).unwrap()).unwrap();
    }
}
