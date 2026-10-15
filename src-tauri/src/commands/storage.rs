use crate::error::{AppError, AppResult};
use serde::{Deserialize, Serialize};
use std::{collections::HashSet, path::{Path, PathBuf}};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StorageBreakdown { pub category: String, pub bytes: i64, pub path: String }

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InstanceStorageInfo {
    pub id: String, pub name: String, pub mc_version: String, pub loader: String,
    pub icon: String, pub bytes: u64, pub path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StorageFullReport {
    pub total_bytes: u64, pub categories: Vec<StorageBreakdown>, pub instances: Vec<InstanceStorageInfo>,
    pub logs_bytes: u64, pub cache_bytes: u64, pub warnings: Vec<String>,
}

#[derive(Default)]
struct DiskScan { seen: HashSet<PathBuf>, files: HashSet<(u64, u64)>, logical: std::collections::HashMap<PathBuf,u64>, warnings: Vec<String> }

#[cfg(unix)]
fn file_identity(_path: &Path, metadata: &std::fs::Metadata) -> Option<(u64, u64)> {
    use std::os::unix::fs::MetadataExt;
    Some((metadata.dev(), metadata.ino()))
}

#[cfg(windows)]
fn file_identity(path: &Path, _metadata: &std::fs::Metadata) -> Option<(u64, u64)> {
    use std::os::windows::io::AsRawHandle;
    #[repr(C)]
    #[derive(Default)]
    struct FileInfo {
        attributes: u32, creation: [u32; 2], access: [u32; 2], write: [u32; 2],
        volume: u32, size_high: u32, size_low: u32, links: u32, index_high: u32, index_low: u32,
    }
    #[link(name = "kernel32")]
    unsafe extern "system" { fn GetFileInformationByHandle(handle: *mut std::ffi::c_void, info: *mut FileInfo) -> i32; }
    let file = std::fs::File::open(path).ok()?;
    let mut info = FileInfo::default();
    if unsafe { GetFileInformationByHandle(file.as_raw_handle(), &mut info) } == 0 { return None; }
    Some((u64::from(info.volume), (u64::from(info.index_high) << 32) | u64::from(info.index_low)))
}

#[cfg(not(any(unix, windows)))]
fn file_identity(_path: &Path, _metadata: &std::fs::Metadata) -> Option<(u64, u64)> { None }

fn is_link(metadata: &std::fs::Metadata) -> bool {
    #[cfg(windows)]
    {
        use std::os::windows::fs::MetadataExt;
        metadata.file_attributes() & 0x400 != 0
    }
    #[cfg(not(windows))]
    { metadata.file_type().is_symlink() }
}
impl DiskScan {
    fn size(&mut self, root: &Path) -> u64 {
        self.visit(root,0).1
    }
    fn logical_size(&self, path: &Path) -> u64 {
        path.canonicalize().ok().and_then(|path| self.logical.get(&path).copied()).unwrap_or(0)
    }
    fn warn(&mut self, path: &Path) {
        if self.warnings.len() < 64 { self.warnings.push(format!("Leitura parcial de {}",path.display())); }
    }
    fn visit(&mut self, path: &Path, depth: usize) -> (u64,u64) {
        if depth > 256 { self.warn(path); return (0,0); }
        let metadata = match std::fs::symlink_metadata(path) {
            Ok(value) => value,
            Err(error) if error.kind()==std::io::ErrorKind::NotFound => return (0,0),
            Err(_) => { self.warn(path); return (0,0); }
        };
        if is_link(&metadata) { return (0,0); }
        if metadata.is_file() {
            let fresh = file_identity(path,&metadata).map(|identity| self.files.insert(identity))
                .unwrap_or_else(|| self.seen.insert(path.to_path_buf()));
            return (metadata.len(),if fresh {metadata.len()} else {0});
        }
        let canonical = path.canonicalize().unwrap_or_else(|_|path.to_path_buf());
        if !self.seen.insert(canonical.clone()) { return (self.logical.get(&canonical).copied().unwrap_or(0),0); }
        let mut logical=0u64;
        let mut physical=0u64;
        match std::fs::read_dir(path) {
            Ok(entries) => for entry in entries {
                match entry {
                    Ok(entry) => { let (size,unique)=self.visit(&entry.path(),depth+1); logical=logical.saturating_add(size); physical=physical.saturating_add(unique); },
                    Err(_) => self.warn(path)
                }
            },
            Err(_) => self.warn(path)
        }
        self.logical.insert(canonical,logical);
        (logical,physical)
    }
}

pub(crate) fn instance_directory_bytes(path: &Path) -> AppResult<u64> {
    let mut scan = DiskScan::default();
    let bytes = scan.size(path);
    if !scan.warnings.is_empty() {
        return Err(AppError::InvalidState(scan.warnings.join("; ")));
    }
    Ok(bytes)
}

fn storage_dirs() -> AppResult<directories::ProjectDirs> {
    directories::ProjectDirs::from("io", "github", "Luxmc").ok_or_else(|| AppError::InvalidState("Diretório de dados indisponível".into()))
}

static REPORT: std::sync::LazyLock<tokio::sync::Mutex<Option<(std::time::Instant,StorageFullReport)>>> = std::sync::LazyLock::new(||tokio::sync::Mutex::new(None));
static CLEANUP: tokio::sync::Mutex<()> = tokio::sync::Mutex::const_new(());

fn collect_report(base: &Path, config: &Path, cache: &Path, executable: Option<&Path>, profiles: Vec<crate::db::models::ProfileRow>, bedrock: Vec<InstanceStorageInfo>) -> StorageFullReport {
    let mut scan = DiskScan::default();
    let mut categories = Vec::new();
    let mut add = |scan: &mut DiskScan, category: &str, path: &Path| {
        let bytes = scan.size(path);
        categories.push(StorageBreakdown { category: category.into(), bytes: bytes.min(i64::MAX as u64) as i64, path: path.to_string_lossy().into_owned() });
    };
    let known = ["libraries", "assets", "versions", "instances", "mods", "logs", "cache", "downloads", "java"];
    for category in known { add(&mut scan, category, &base.join(category)); }
    if let Ok(entries) = std::fs::read_dir(base) {
        for entry in entries.flatten() {
            if !known.contains(&entry.file_name().to_string_lossy().as_ref()) { add(&mut scan,"launcher_data", &entry.path()); }
        }
    }
    add(&mut scan,"configuration", config);
    add(&mut scan,"launcher_cache", cache);
    if let Some(path) = executable {
        add(&mut scan,"application", path);
        if let Some(parent) = path.parent() {
            add(&mut scan,"application", &parent.join("uninstall.exe"));
        }
    }
    let mut instance_items = Vec::new();
    let mut log_roots = HashSet::from([base.join("logs")]);
    for profile in profiles {
        let configured = PathBuf::from(&profile.game_dir);
        let path = if !profile.game_dir.is_empty() && configured.is_dir() { configured } else { base.join("instances").join(&profile.id).join(".minecraft") };
        add(&mut scan,"external_instances", &path);
        log_roots.extend([path.join("logs"),path.join("crash-reports")]);
        instance_items.push(InstanceStorageInfo { id: profile.id, name: profile.name, mc_version: profile.mc_version, loader: profile.loader, icon: profile.icon, bytes: scan.logical_size(&path), path: path.to_string_lossy().into_owned() });
    }
    for mut instance in bedrock {
        let path = PathBuf::from(&instance.path);
        add(&mut scan,"external_instances", &path);
        instance.bytes = scan.logical_size(&path);
        instance_items.push(instance);
    }
    instance_items.sort_by(|a,b| b.bytes.cmp(&a.bytes));
    let mut grouped: std::collections::BTreeMap<String, StorageBreakdown> = std::collections::BTreeMap::new();
    for category in categories.into_iter().filter(|category| category.bytes > 0) {
        grouped.entry(category.category.clone()).and_modify(|existing| { existing.bytes = existing.bytes.saturating_add(category.bytes); existing.path.push_str("; "); existing.path.push_str(&category.path); }).or_insert(category);
    }
    let categories: Vec<_> = grouped.into_values().collect();
    let cache_bytes: u64 = ["cache", "downloads", "migration-staging"].iter().map(|sub| scan.logical_size(&base.join(sub))).sum();
    let cache_bytes = if cache != base && !cache.starts_with(base) { cache_bytes.saturating_add(scan.logical_size(cache)) } else { cache_bytes };
    let logs_bytes = log_roots.into_iter().filter_map(|path|path.canonicalize().ok()).collect::<HashSet<_>>().iter().map(|path|scan.logical_size(path)).sum();
    StorageFullReport { total_bytes: categories.iter().map(|category| category.bytes as u64).sum(), categories, instances: instance_items, logs_bytes, cache_bytes, warnings: scan.warnings }
}

#[tauri::command]
pub async fn storage_breakdown() -> AppResult<Vec<StorageBreakdown>> { Ok(storage_full_report(None).await?.categories) }

#[tauri::command]
pub async fn storage_full_report(refresh: Option<bool>) -> AppResult<StorageFullReport> {
    let mut saved = REPORT.lock().await;
    if refresh!=Some(true) { if let Some((at,report))=saved.as_ref() { if at.elapsed().as_secs()<10 { return Ok(report.clone()); } } }
    let report = compute_report().await?;
    *saved=Some((std::time::Instant::now(),report.clone()));
    Ok(report)
}

async fn compute_report() -> AppResult<StorageFullReport> {
    let dirs = storage_dirs()?;
    let base = dirs.data_dir().to_path_buf();
    let config = dirs.config_dir().to_path_buf();
    let cache = dirs.cache_dir().to_path_buf();
    let executable = std::env::current_exe().ok();
    let db = crate::db::shared_db().await?;
    let profiles = crate::db::schema::profiles::list(&db).await?;
    let (bedrock, bedrock_warning) = match crate::commands::bedrock::bedrock_state().await {
        Ok(state) => (Some(state), None), Err(error) => (None, Some(format!("Não foi possível medir as instâncias Bedrock: {error}"))),
    };
    let bedrock = bedrock.iter().flat_map(|state| state.instances.iter().filter_map(move |instance| {
        state.installations.iter().find(|installation| installation.id == instance.installation_id && installation.profile_id == instance.profile_id).map(|installation| InstanceStorageInfo {
            id: instance.id.clone(), name: instance.name.clone(), mc_version: installation.version.clone(), loader: "bedrock".into(), icon: "grass_block".into(), bytes: 0, path: installation.directory.clone(),
        })
    })).collect();
    tokio::task::spawn_blocking(move || {
        let mut report = collect_report(&base, &config, &cache, executable.as_deref(), profiles, bedrock);
        if let Some(warning) = bedrock_warning { report.warnings.push(warning); }
        report
    }).await.map_err(|error| AppError::Internal(error.to_string()))
}

fn clear_log_files(roots: Vec<PathBuf>) -> u64 {
    let mut seen = HashSet::new();
    let mut freed = 0u64;
    for root in roots {
        let Ok(root_meta) = std::fs::symlink_metadata(&root) else { continue; };
        if !root_meta.is_dir() || is_link(&root_meta) { continue; }
        let Ok(boundary) = root.canonicalize() else { continue; };
        let mut pending = vec![root];
        while let Some(path) = pending.pop() {
            let Ok(metadata) = std::fs::symlink_metadata(&path) else { continue; };
            if is_link(&metadata) { continue; }
            let Ok(canonical) = path.canonicalize() else { continue; };
            if !canonical.starts_with(&boundary) || !seen.insert(canonical) { continue; }
            if metadata.is_file() {
                if std::fs::remove_file(&path).is_ok() { freed = freed.saturating_add(metadata.len()); }
            } else if metadata.is_dir() {
                if let Ok(entries) = std::fs::read_dir(&path) { pending.extend(entries.flatten().map(|entry| entry.path())); }
            }
        }
    }
    freed
}

#[tauri::command]
pub async fn storage_clear_logs() -> AppResult<u64> {
    let _cleanup = CLEANUP.try_lock().map_err(|_|AppError::InvalidState("Uma limpeza já está em andamento.".into()))?;
    *REPORT.lock().await=None;
    let base = storage_dirs()?.data_dir().to_path_buf();
    let db = crate::db::shared_db().await?;
    let profiles = crate::db::schema::profiles::list(&db).await?;
    let mut roots = vec![base.join("logs")];
    for profile in profiles {
        let configured = PathBuf::from(&profile.game_dir);
        let game_dir = if !profile.game_dir.is_empty() && configured.is_dir() { configured } else { base.join("instances").join(&profile.id).join(".minecraft") };
        roots.extend([game_dir.join("logs"), game_dir.join("crash-reports")]);
    }
    tokio::task::spawn_blocking(move || clear_log_files(roots)).await.map_err(|error| AppError::Internal(error.to_string()))
}

#[tauri::command]
pub async fn storage_clear_cache(state: tauri::State<'_,crate::state::AppState>) -> AppResult<u64> { storage_clear_cache_core(&state).await }

pub async fn storage_clear_cache_core(state: &crate::state::AppState) -> AppResult<u64> {
    let _cleanup=CLEANUP.try_lock().map_err(|_|AppError::InvalidState("Uma limpeza já está em andamento.".into()))?;
    let _launch=state.launch_lock.try_lock().map_err(|_|AppError::InvalidState("Aguarde a abertura do Minecraft antes de limpar o cache.".into()))?;
    let _import=state.import_lock.try_lock().map_err(|_|AppError::InvalidState("Aguarde a instalação antes de limpar o cache.".into()))?;
    if state.import_job.load(std::sync::atomic::Ordering::SeqCst) || crate::core::launcher::get_active_game_pid()!=0 { return Err(AppError::InvalidState("Pause os downloads e feche o Minecraft antes de limpar o cache.".into())); }
    *REPORT.lock().await=None;
    let dirs = storage_dirs()?;
    let base_dir = dirs.data_dir().to_path_buf();
    let cache_dir = dirs.cache_dir().to_path_buf();

    let _migration_guard = crate::core::instance_paths::migration_guard().await;

    let freed = tokio::task::spawn_blocking(move || {
        let mut roots: Vec<_> = ["cache", "downloads", "migration-staging"].iter().map(|sub| base_dir.join(sub)).collect();
        if cache_dir != base_dir { roots.push(cache_dir); }
        clear_log_files(roots)
    })
    .await
    .map_err(|e| crate::error::AppError::Internal(e.to_string()))?;

    Ok(freed)
}

#[tauri::command]
pub async fn storage_delete_instance(id: String) -> AppResult<()> {
    if crate::commands::bedrock::bedrock_state().await.is_ok_and(|bedrock| bedrock.instances.iter().any(|instance| instance.id == id)) { return crate::commands::bedrock::bedrock_remove(id).await; }
    crate::commands::profiles::profiles_delete(id).await
}

#[cfg(test)]
mod tests {
    use super::*;
    fn file(root: &Path, path: &str, bytes: usize) {
        let path = root.join(path); std::fs::create_dir_all(path.parent().unwrap()).unwrap(); std::fs::write(path, vec![0; bytes]).unwrap();
    }
    fn profile(id: &str, path: &Path) -> crate::db::models::ProfileRow {
        serde_json::from_value(serde_json::json!({ "id": id, "name": id, "icon": "grass", "mcVersion": "1.21.1", "loader": "vanilla", "fullscreen": false, "gameDir": path.to_string_lossy(), "createdAt": "2026-10-03T00:00:00Z", "updatedAt": "2026-10-03T00:00:00Z" })).unwrap()
    }
    #[test]
    fn instance_size_includes_deep_world_files() {
        let root = std::env::temp_dir().join(format!("luxmc-world-size-{}", uuid::Uuid::new_v4()));
        file(&root, "saves/world/dimensions/mod/dimension/region/world.mca", 521);
        file(&root, "mods/file.jar", 43);
        assert_eq!(instance_directory_bytes(&root).unwrap(), 564);
        assert_eq!(instance_directory_bytes(&root.join("missing")).unwrap(), 0);
        assert!(root.canonicalize().unwrap().starts_with(std::env::temp_dir().canonicalize().unwrap()));
        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn reports_java_database_executable_caches_and_external_profiles_without_double_counting() {
        let root = std::env::temp_dir().join(format!("luxmc-storage-{}", uuid::Uuid::new_v4()));
        let base = root.join("data"); let cache = root.join("cache"); let external = root.join("external");
        for (path, bytes) in [("assets/file", 8), ("java/runtime", 17), ("luxmc.db", 31), ("instances/inside/.minecraft/worlds/world", 42), ("instances/inside/.minecraft/logs/log", 4), ("cache/file", 5), ("downloads/file", 7), ("migration-staging/file", 9), ("logs/log", 3)] { file(&base, path, bytes); }
        file(&external, "worlds/world", 49); file(&external, "logs/log", 6); file(&cache, "file", 23); file(&root, "Luxmc.exe", 47); file(&root, "uninstall.exe", 11);
        let inside = base.join("instances/inside/.minecraft");
        let report = collect_report(&base, &base, &cache, Some(&root.join("Luxmc.exe")), vec![profile("inside", &inside), profile("external", &external), profile("legacy-duplicate", &external)], vec![]);
        assert_eq!(report.total_bytes, 262);
        assert_eq!(report.categories.iter().find(|category| category.category == "java").unwrap().bytes, 17);
        assert_eq!(report.categories.iter().find(|category| category.category == "application").unwrap().bytes, 58);
        assert_eq!(report.instances.iter().find(|item| item.id == "inside").unwrap().bytes, 46);
        assert_eq!(report.instances.iter().find(|item| item.id == "external").unwrap().bytes, 55);
        assert_eq!(report.cache_bytes, 44); assert_eq!(report.logs_bytes, 13); assert!(report.warnings.is_empty());
        assert_eq!(clear_log_files(vec![base.join("logs"), inside.join("logs"), external.join("logs"), external.join("logs")]), 13);
        assert!(external.join("worlds/world").is_file());
        assert_eq!(clear_log_files(vec![external.join("logs")]), 0);
        assert!(root.canonicalize().unwrap().starts_with(std::env::temp_dir().canonicalize().unwrap()));
        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn hard_linked_files_are_counted_once_across_categories() {
        let root = std::env::temp_dir().join(format!("luxmc-hardlinks-{}", uuid::Uuid::new_v4()));
        file(&root, "libraries/shared.jar", 512);
        std::fs::create_dir_all(root.join("instances/one/mods")).unwrap();
        std::fs::hard_link(root.join("libraries/shared.jar"), root.join("instances/one/mods/shared.jar")).unwrap();
        let mut scan = DiskScan::default();
        assert_eq!(scan.size(&root.join("libraries")), 512);
        assert_eq!(scan.size(&root.join("instances")), 0);
        assert_eq!(instance_directory_bytes(&root.join("instances/one")).unwrap(), 512);
        assert!(root.canonicalize().unwrap().starts_with(std::env::temp_dir().canonicalize().unwrap()));
        std::fs::remove_dir_all(root).unwrap();
    }
    #[test]
    fn cache_cleanup_preserves_roots_and_unrelated_instance_data() {
        let root = tempfile::tempdir().unwrap();
        for (path,bytes) in [("cache/nested/thumbnail",12),("downloads/pack.part",23),("instances/game/saves/world/level.dat",45),("instances/game/mods/mod.jar",67)] { file(root.path(),path,bytes); }
        assert_eq!(clear_log_files(vec![root.path().join("cache"),root.path().join("downloads")]),35);
        assert!(root.path().join("cache/nested").is_dir());
        assert!(root.path().join("downloads").is_dir());
        assert!(root.path().join("instances/game/saves/world/level.dat").is_file());
        assert!(root.path().join("instances/game/mods/mod.jar").is_file());
    }
    #[cfg(unix)]
    #[test]
    fn scan_and_cleanup_do_not_follow_links_outside_the_selected_cache() {
        let root = tempfile::tempdir().unwrap();
        file(root.path(),"cache/local",12);
        file(root.path(),"world/level.dat",45);
        std::os::unix::fs::symlink(root.path().join("world"),root.path().join("cache/link")).unwrap();
        assert_eq!(DiskScan::default().size(&root.path().join("cache")),12);
        assert_eq!(clear_log_files(vec![root.path().join("cache")]),12);
        assert!(root.path().join("world/level.dat").is_file());
    }
    #[cfg(windows)]
    #[test]
    fn cleanup_reports_only_removed_files_when_a_log_is_locked() {
        use std::os::windows::fs::OpenOptionsExt;
        let root = tempfile::tempdir().unwrap();
        file(root.path(),"logs/current.log",14);
        file(root.path(),"logs/old.log",8);
        let locked = std::fs::OpenOptions::new().read(true).share_mode(0).open(root.path().join("logs/current.log")).unwrap();
        assert_eq!(clear_log_files(vec![root.path().join("logs")]),8);
        assert!(root.path().join("logs/current.log").is_file());
        drop(locked);
        assert_eq!(clear_log_files(vec![root.path().join("logs")]),14);
    }
    #[test]
    fn bedrock_worlds_are_measured_without_merging_them_with_java_profiles() {
        let root = std::env::temp_dir().join(format!("luxmc-bedrock-storage-{}", uuid::Uuid::new_v4()));
        let data = root.join("launcher");
        let bedrock = root.join("bedrock");
        file(&bedrock, "packageData/games/com.mojang/minecraftWorlds/world/db/data", 789);
        let report = collect_report(&data, &data, &root.join("cache"), None, vec![], vec![InstanceStorageInfo { id: "bedrock-link".into(), name: "My Bedrock".into(), mc_version: "1.20".into(), loader: "bedrock".into(), icon: "grass_block".into(), bytes: 0, path: bedrock.to_string_lossy().into_owned() }]);
        assert_eq!(report.total_bytes, 789);
        assert_eq!(report.instances.len(), 1);
        assert_eq!(report.instances[0].loader, "bedrock");
        assert_eq!(report.instances[0].bytes, 789);
        assert!(root.canonicalize().unwrap().starts_with(std::env::temp_dir().canonicalize().unwrap()));
        std::fs::remove_dir_all(root).unwrap();
    }
}
