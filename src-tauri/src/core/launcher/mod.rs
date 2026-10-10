mod loader_selection;
mod quick_join;
mod classpath_paths;
#[cfg(any(target_os = "windows", test))]
mod native_cache;
pub mod launch_state;
use std::path::PathBuf;
use std::process::Stdio;
use tauri::Emitter;
use tokio::io::BufReader;
use tokio::sync::Mutex as TokioMutex;
use std::sync::Arc;

use crate::core::downloader::DownloadManager;
use crate::core::java::JavaRuntimeManager;
use crate::core::minecraft::{self, VersionDetail};
use crate::error::{AppError, AppResult};

const DEV_CLIENT_ID: &str = "00000000-0000-0000-0000-000000000002";
const DEV_XUID: &str = "0";
const LAUNCHER_NAME: &str = "Luxmc";
const LAUNCHER_VERSION: &str = env!("CARGO_PKG_VERSION");
static CLIENT_AGENT_JAR: &[u8] = include_bytes!("../../../assets/luxmc-client-agent.jar");
static ACTIVE_CAPE_BYTES: tokio::sync::RwLock<Vec<u8>> = tokio::sync::RwLock::const_new(Vec::new());

static ACTIVE_GAME_PID: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(0);

async fn finish_log_reader(mut reader: tokio::task::JoinHandle<()>) {
    if tokio::time::timeout(std::time::Duration::from_secs(2), &mut reader).await.is_err() {
        reader.abort();
        let _ = reader.await;
    }
}

pub fn set_active_game_pid(pid: u32) {
    ACTIVE_GAME_PID.store(pid, std::sync::atomic::Ordering::SeqCst);
}

pub fn get_active_game_pid() -> u32 {
    ACTIVE_GAME_PID.load(std::sync::atomic::Ordering::SeqCst)
}

pub fn clear_active_game_pid() {
    ACTIVE_GAME_PID.store(0, std::sync::atomic::Ordering::SeqCst);
}

static ACTIVE_GAME_DIR: std::sync::RwLock<Option<std::path::PathBuf>> = std::sync::RwLock::new(None);

pub fn set_active_game_dir(dir: std::path::PathBuf) {
    if let Ok(mut lock) = ACTIVE_GAME_DIR.write() {
        *lock = Some(dir);
    }
}

pub fn get_active_game_dir() -> Option<std::path::PathBuf> {
    if let Ok(lock) = ACTIVE_GAME_DIR.read() {
        lock.clone()
    } else {
        None
    }
}

pub fn clear_active_game_dir() {
    if let Ok(mut lock) = ACTIVE_GAME_DIR.write() {
        *lock = None;
    }
}

pub fn resolve_local_or_asset_path(src: &str) -> Option<std::path::PathBuf> {
    let raw = src.trim();
    if raw.is_empty() {
        return None;
    }
    if let Some(rest) = raw.strip_prefix("asset://localhost/") {
        if let Ok(decoded) = urlencoding::decode(rest) {
            let p = std::path::PathBuf::from(decoded.into_owned());
            if p.exists() {
                return Some(p);
            }
        }
    }
    if let Some(rest) = raw.strip_prefix("asset://") {
        if let Ok(decoded) = urlencoding::decode(rest) {
            let p = std::path::PathBuf::from(decoded.into_owned());
            if p.exists() {
                return Some(p);
            }
        }
    }
    let p = std::path::PathBuf::from(raw);
    if p.exists() {
        return Some(p);
    }
    None
}

pub async fn resolve_image_bytes(source: &str, http: &reqwest::Client) -> Vec<u8> {
    let s = source.trim();
    if s.is_empty() {
        return Vec::new();
    }
    if s.starts_with("http://") || s.starts_with("https://") {
        if let Ok(resp) = http.get(s).send().await {
            if resp.status().is_success() {
                return resp.bytes().await.map(|b| b.to_vec()).unwrap_or_default();
            }
        }
        return Vec::new();
    }
    if s.starts_with("data:image/") {
        if let Some(pos) = s.find(',') {
            use base64::Engine;
            return base64::engine::general_purpose::STANDARD
                .decode(&s[pos + 1..])
                .unwrap_or_default();
        }
        return Vec::new();
    }
    if let Some(p) = resolve_local_or_asset_path(s) {
        return tokio::fs::read(p).await.unwrap_or_default();
    }
    Vec::new()
}

pub async fn set_active_cape_bytes(bytes: Vec<u8>) {
    let mut lock = ACTIVE_CAPE_BYTES.write().await;
    *lock = bytes;
}

pub async fn get_active_cape_bytes() -> Vec<u8> {
    ACTIVE_CAPE_BYTES.read().await.clone()
}

/// Pipeline state machine. Every transition is emitted to the UI.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub enum LaunchStage {
    Preparing,
    CheckingJava,
    ResolvingClasspath,
    ExtractingNatives,
    ResolvingArgs,
    ValidatingArgs,
    Spawning,
    Running,
    Finished,
    Failed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Platform {
    Linux,
    Windows,
    Macos,
    Unknown,
}

pub const fn current_platform() -> Platform {
    if cfg!(target_os = "macos") {
        Platform::Macos
    } else if cfg!(target_os = "windows") {
        Platform::Windows
    } else if cfg!(target_os = "linux") {
        Platform::Linux
    } else {
        Platform::Unknown
    }
}

pub const fn platform_mojang_name(p: Platform) -> &'static str {
    match p {
        Platform::Linux => "linux",
        Platform::Windows => "windows",
        Platform::Macos => "osx",
        Platform::Unknown => "unknown",
    }
}

/// JVM argument values that are absolutely forbidden on each platform.
/// This is the *last* gate before a real Java process gets spawned.
fn forbidden_jvm_args_on(p: Platform) -> &'static [&'static str] {
    match p {
        // macOS-only: a no-op on Windows, a real crash on Linux.
        Platform::Linux | Platform::Windows | Platform::Unknown => &["-XstartOnFirstThread"],
        Platform::Macos => &[],
    }
}

fn forbidden_jvm_prefixes_on(p: Platform) -> &'static [&'static str] {
    match p {
        // The Mojang Intel driver workaround path is meaningless outside Windows.
        Platform::Linux | Platform::Macos | Platform::Unknown => {
            &["-XX:HeapDumpPath=MojangTricksIntelDriversForPerformance"]
        }
        Platform::Windows => &[],
    }
}

const OBSOLETE_HOTSPOT_FLAGS: &[&str] = &[
    "-XX:+UseFastAccessorMethods",
    "-XX:-UseFastAccessorMethods",
    "-XX:+AggressiveOpts",
    "-XX:-AggressiveOpts",
    "-XX:+UseConcMarkSweepGC",
    "-XX:+CMSParallelRemarkEnabled",
    "-XX:+UseCMSCompactAtFullCollection",
];

/// Final validation layer. Returns the indices of every arg that must NOT
/// be passed to the Java process on the current platform.
pub fn validate_platform_args(args: &[String]) -> Vec<usize> {
    let platform = current_platform();
    let forbidden = forbidden_jvm_args_on(platform);
    let forbidden_prefixes = forbidden_jvm_prefixes_on(platform);

    let mut rejected = Vec::new();
    for (idx, arg) in args.iter().enumerate() {
        let trimmed = arg.trim();
        if forbidden.iter().any(|f| trimmed == *f) {
            rejected.push(idx);
            continue;
        }
        if forbidden_prefixes.iter().any(|p| trimmed.starts_with(p)) {
            rejected.push(idx);
            continue;
        }
        if OBSOLETE_HOTSPOT_FLAGS.iter().any(|f| trimmed == *f) {
            rejected.push(idx);
            continue;
        }
    }
    rejected
}

pub fn jvm_arg_allowed_on_current_os(arg: &str) -> bool {
    !validate_platform_args(&[arg.to_string()]).contains(&0)
}

pub fn find_csharp_launcher() -> Option<PathBuf> {
    if let Ok(p) = std::env::var("LUXMC_CSHARP_PATH") {
        let pb = PathBuf::from(p);
        if pb.is_file() {
            return Some(pb);
        }
    }

    if std::env::var("LUXMC_USE_CSHARP_LAUNCHER").is_err() {
        return None;
    }

    let mut candidates = Vec::new();
    let binary_name = if cfg!(windows) { "Luxmc.Launcher.exe" } else { "Luxmc.Launcher" };

    if let Ok(exe) = std::env::current_exe() {
        if let Some(parent) = exe.parent() {
            candidates.push(parent.join(binary_name));
            candidates.push(parent.join("bin").join(binary_name));
            if let Some(grandparent) = parent.parent() {
                candidates.push(grandparent.join("bin").join(binary_name));
                candidates.push(grandparent.join("dist-electron").join("bin").join(binary_name));
                candidates.push(grandparent.join("app.asar.unpacked").join("dist-electron").join("bin").join(binary_name));
            }
        }
    }

    if let Some(dirs) = directories::ProjectDirs::from("io", "github", "Luxmc") {
        candidates.push(dirs.data_dir().join("bin").join(binary_name));
    }

    for c in candidates {
        if c.is_file() {
            return Some(c);
        }
    }
    None
}

pub struct GameLauncher {
    downloader: DownloadManager,
    java: JavaRuntimeManager,
    app: Option<tauri::AppHandle>,
}

impl GameLauncher {
    pub fn new(downloader: DownloadManager, java: JavaRuntimeManager) -> Self {
        Self {
            downloader,
            java,
            app: None,
        }
    }

    pub fn with_app(mut self, app: tauri::AppHandle) -> Self {
        self.app = Some(app);
        self
    }

    fn emit_log(&self, message: &str) {
        if let Some(ref app) = self.app {
            let _ = app.emit("launcher-log", message);
        }
        tracing::info!(target: "launch", "{}", message);
    }

    fn emit_stage(&self, stage: LaunchStage) {
        if let Some(ref app) = self.app {
            let _ = app.emit("launch-stage", stage);
        }
    }

    fn extract_actual_crash_reason(game_dir: &std::path::Path, fallback_stderr: Option<String>) -> Option<String> {
        let crash_reports_dir = game_dir.join("crash-reports");
        if let Ok(entries) = std::fs::read_dir(&crash_reports_dir) {
            let mut newest_report: Option<(std::time::SystemTime, std::path::PathBuf)> = None;
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_file() && path.extension().map_or(false, |ext| ext == "txt") {
                    if let Ok(meta) = entry.metadata() {
                        if let Ok(modified) = meta.modified() {
                            if let Ok(elapsed) = modified.elapsed() {
                                if elapsed.as_secs() < 300 {
                                    if newest_report.as_ref().map_or(true, |(m, _)| modified > *m) {
                                        newest_report = Some((modified, path));
                                    }
                                }
                            }
                        }
                    }
                }
            }
            if let Some((_, report_path)) = newest_report {
                if let Ok(content) = std::fs::read_to_string(report_path) {
                    let mut description = None;
                    let mut cause = None;
                    for line in content.lines().take(60) {
                        let line = line.trim();
                        if description.is_none() && line.starts_with("Description:") {
                            description = Some(line.trim_start_matches("Description:").trim().to_string());
                        } else if line.starts_with("Caused by:") || line.starts_with("java.lang.") || line.starts_with("org.spongepowered.") {
                            if cause.is_none() || line.starts_with("Caused by:") {
                                cause = Some(line.to_string());
                            }
                        }
                    }
                    if let Some(desc) = description {
                        if let Some(c) = cause {
                            return Some(format!("{}: {}", desc, c));
                        }
                        return Some(desc);
                    } else if let Some(c) = cause {
                        return Some(c);
                    }
                }
            }
        }

        for name in ["luxmc_game.log", "latest.log"] {
            let log_file = game_dir.join("logs").join(name);
            if let Ok(content) = std::fs::read_to_string(log_file) {
                let lines: Vec<&str> = content.lines().collect();
                let scan_start = lines.len().saturating_sub(60);
                for line in lines[scan_start..].iter().rev() {
                    let trimmed = line.trim();
                    if trimmed.contains("/FATAL]") || trimmed.contains("/ERROR]") || trimmed.starts_with("Caused by:") || trimmed.starts_with("Exception in thread") || trimmed.starts_with("Error:") || trimmed.contains("Could not reserve enough space") || trimmed.starts_with("Unrecognized VM option") {
                        let cleaned = if let Some(idx) = trimmed.find("]: ") {
                            &trimmed[idx + 3..]
                        } else {
                            trimmed
                        };
                        if !cleaned.is_empty() && !cleaned.contains("[ALSOFT]") {
                            return Some(cleaned.to_string());
                        }
                    }
                }
            }
        }

        fallback_stderr.filter(|s| !s.is_empty())
    }

    #[allow(dead_code)]
    fn compute_auto_ram(&self, is_heavy_modded: bool, is_pvp: bool, mod_count: i64) -> i64 {
        let total_ram_mb = crate::core::optimizer::get_total_memory_mb();
        let os_reserve: i64 = 3072;
        let available = (total_ram_mb - os_reserve).max(1024);

        if is_pvp {
            let want = if mod_count > 10 { 3072 } else { 2048 };
            let result = want.min(available);
            self.emit_log(&format!(
                "Luxmc Auto-RAM: {}MB alocados (PvP 1.7/1.8)",
                result
            ));
            return result;
        }

        let want = if is_heavy_modded {
            if total_ram_mb >= 32768 { 12288 }
            else if total_ram_mb >= 24576 { 10240 }
            else if total_ram_mb >= 16384 { 8192 }
            else if total_ram_mb >= 12288 { 6144 }
            else if total_ram_mb >= 8192 { 5120 }
            else { (total_ram_mb * 7 / 10).max(3072) }
        } else if mod_count >= 20 {
            if total_ram_mb >= 16384 { 5120 } else if total_ram_mb >= 8192 { 4096 } else { 3072 }
        } else {
            if total_ram_mb >= 8192 { 3072 } else { 2048 }
        };

        let result = want.min(available);
        self.emit_log(&format!(
            "Luxmc Auto-RAM: {}MB alocados (RAM Total: {}MB, Tipo: {})",
            result, total_ram_mb, if is_heavy_modded { "Modpack Pesado" } else { "Padrão" }
        ));
        result
    }

    pub async fn launch(
        &self,
        detail: &VersionDetail,
        username: &str,
        uuid: &str,
        access_token: &str,
        user_type: &str,
        game_dir: &PathBuf,
        profile: &crate::db::models::ProfileRow,
        skin_url: Option<&str>,
        skin_variant: Option<&str>,
        cape_url: Option<&str>,
        server_ip: Option<&str>,
        server_port: Option<u16>,
    ) -> AppResult<u32> {
        self.emit_stage(LaunchStage::Preparing);
        self.emit_log(&format!("Preparing to launch {}", detail.id));
        self.emit_log(&format!("Host platform: {:?}", current_platform()));

        let (loader, loader_version) = loader_selection::resolve(&game_dir, &profile.loader, profile.loader_version.as_deref())?;
        if !matches!(loader.as_str(), "vanilla" | "fabric" | "forge" | "neoforge" | "quilt") {
            return Err(crate::error::AppError::InvalidState(format!(
                "Loader desconhecido '{}' na instancia. Ajuste o loader nas configuracoes da instancia.",
                loader
            )));
        }

        self.emit_stage(LaunchStage::CheckingJava);
        let mut major = detail.java_major_version();
        let clean_ver = profile.clean_mc_version();
        if (clean_ver.starts_with("1.21") || clean_ver.starts_with("1.20.5") || clean_ver.starts_with("1.20.6")) && major < 21 {
            major = 21;
        }
        self.emit_log(&format!("Java major version required: {} (loader: {})", major, loader));

        let java_path = if let Some(custom_path) = profile.java_path.as_deref().filter(|p| !p.trim().is_empty()) {
            let p = PathBuf::from(custom_path);
            if p.exists() {
                self.emit_log(&format!("Using custom Java binary from profile: {}", p.display()));
                p
            } else {
                self.java.ensure_java(major).await?
            }
        } else {
            self.java.ensure_java(major).await?
        };
        self.emit_log(&format!("Java binary: {}", java_path.display()));

        self.emit_stage(LaunchStage::ResolvingClasspath);
        let mut classpath = self.build_classpath(detail);

        let mut main_class = detail
            .main_class
            .as_deref()
            .unwrap_or("net.minecraft.client.main.Main")
            .to_string();

        let mut extra_jvm_args = Vec::new();
        let mut extra_game_args = Vec::new();

        let clean_mc_ver = profile.clean_mc_version();
        if loader == "fabric" || loader == "quilt" || loader == "neoforge" || loader == "forge" {
            if loader == "fabric" {
                let mods_dir = game_dir.join("mods");
                let _ = crate::core::loaders::fabric::ensure_fabric_api(
                    self.downloader.http(),
                    &mods_dir,
                    clean_mc_ver,
                ).await;
            }

            self.emit_log(&format!("Resolving {} loader for Minecraft {}...", loader, clean_mc_ver));
            match crate::core::loaders::prepare_loader(
                self.downloader.http(),
                &self.downloader.libraries_dir(),
                &loader,
                clean_mc_ver,
                loader_version.as_deref(),
            ).await {
                Ok(prep) => {
                    self.emit_log(&format!("{} loader ready: mainClass = {}", loader, prep.main_class));
                    main_class = prep.main_class;
                    let mut new_cp = prep.classpath_entries;
                    new_cp.extend(classpath);
                    classpath = new_cp;
                    extra_jvm_args.extend(prep.jvm_args);
                    extra_game_args.extend(prep.game_args);
                }
                Err(e) => {
                    self.emit_log(&format!("ERROR: Failed to prepare {} loader: {}. Cannot launch with mods.", loader, e));
                    return Err(crate::error::AppError::InvalidState(format!(
                        "Falha ao preparar o loader {}: {}. Verifique se a versao do Minecraft e do loader sao compativeis.",
                        loader, e
                    )));
                }
            }

            if loader == "neoforge" || loader == "forge" {
                classpath.retain(|p| {
                    let s = p.to_string_lossy();
                    !s.ends_with("-installer.jar")
                });

                let mc_ver = clean_mc_ver;
                let versions_dir = self.downloader.versions_dir();
                let vanilla_jar = versions_dir.join(mc_ver).join(format!("{}.jar", mc_ver));

                let has_bundled_client = classpath.iter().any(|p| {
                    let s = p.to_string_lossy().to_lowercase();
                    p.exists() && (
                        (s.contains("neoforge-") && s.ends_with("-client.jar"))
                        || (s.contains("forge-") && s.ends_with("-client.jar"))
                        || (s.contains("minecraft-") && s.ends_with("-client.jar"))
                        || (s.contains("client-") && s.ends_with("-srg.jar"))
                        || (s.contains("client-") && s.ends_with("-slim.jar"))
                    )
                });

                if has_bundled_client {
                    classpath.retain(|p| {
                        let s = p.to_string_lossy().replace('\\', "/").to_lowercase();
                        let is_vanilla = s.ends_with(&format!("/{}.jar", mc_ver.to_lowercase()))
                            || s.ends_with(&format!("/versions/{}/{}.jar", mc_ver.to_lowercase(), mc_ver.to_lowercase()))
                            || s.ends_with(&format!("/{}.jar", detail.id.to_lowercase()))
                            || s.ends_with(&format!("/versions/{}/{}.jar", detail.id.to_lowercase(), detail.id.to_lowercase()))
                            || p == &vanilla_jar;
                        !is_vanilla
                    });
                    self.emit_log(&format!(
                        "Removed vanilla {}.jar from classpath (bundled client jar present)",
                        mc_ver
                    ));
                } else if !classpath.contains(&vanilla_jar) && vanilla_jar.exists() {
                    classpath.push(vanilla_jar);
                }

                if loader == "neoforge" {
                    let is_modern = clean_mc_ver.starts_with("1.20.4")
                        || clean_mc_ver.starts_with("1.20.5")
                        || clean_mc_ver.starts_with("1.20.6")
                        || clean_mc_ver.starts_with("1.21")
                        || clean_mc_ver.starts_with("1.22");
                    if is_modern {
                        classpath.retain(|p| {
                            let s = p.to_string_lossy().replace('\\', "/").to_lowercase();
                            !s.contains("net/neoforged/neoforge/")
                        });
                        self.emit_log("NeoForge module isolation: excluded neoforge runtime jars from classpath");
                    } else {
                        let has_neoforge_client = classpath.iter().any(|p| {
                            let s = p.to_string_lossy().to_lowercase();
                            s.contains("neoforge") && s.ends_with("-client.jar")
                        });
                        if has_neoforge_client {
                            classpath.retain(|p| {
                                let s = p.to_string_lossy().replace('\\', "/").to_lowercase();
                                if s.contains("net/neoforged/neoforge") {
                                    s.ends_with("-client.jar")
                                } else {
                                    true
                                }
                            });
                        }
                    }
                }
            }

        }

        classpath = classpath_paths::normalize(classpath);
        classpath_paths::normalize_jvm_paths(&mut extra_jvm_args);

        self.emit_log(&format!("Classpath entries: {}", classpath.len()));

        self.emit_stage(LaunchStage::ExtractingNatives);
        let natives_dir = self.prepare_natives(detail).await?;
        self.emit_log(&format!("Natives dir: {}", natives_dir.display()));

        // Apply customized player skin if configured
        let is_msa = user_type == "msa";
        let mut appearance_variant = skin_variant.unwrap_or("classic").to_string();

        let effective_cape = if let Some(c) = cape_url.filter(|c| !c.trim().is_empty()) {
            Some(c.to_string())
        } else if let Ok(db) = crate::db::shared_db().await {
            use sqlx::Row;
            if let Ok(Some(row)) = sqlx::query("SELECT cape_url FROM accounts WHERE username = ? OR uuid = ? OR id = ? LIMIT 1")
                .bind(username)
                .bind(uuid)
                .bind(uuid)
                .fetch_optional(db.pool())
                .await
            {
                row.try_get::<Option<String>, _>("cape_url").ok().flatten().filter(|c| !c.trim().is_empty())
            } else {
                None
            }
        } else {
            None
        };

        let has_explicit_cape = effective_cape
            .as_deref()
            .map(|c| !c.trim().is_empty())
            .unwrap_or(false);

        let skin_info = if let Some(s_url) = skin_url.filter(|s| !s.trim().is_empty()) {
            Some((s_url.to_string(), skin_variant.unwrap_or("classic").to_string()))
        } else if let Ok(db) = crate::db::shared_db().await {
            use sqlx::Row;
            if let Ok(Some(row)) = sqlx::query("SELECT skin_url, skin_variant FROM accounts WHERE username = ? OR uuid = ? OR id = ? LIMIT 1")
                .bind(username)
                .bind(uuid)
                .bind(uuid)
                .fetch_optional(db.pool())
                .await
            {
                let s_url: Option<String> = row.try_get::<Option<String>, _>("skin_url").ok().flatten().filter(|s| !s.trim().is_empty());
                let s_var: Option<String> = row.try_get::<Option<String>, _>("skin_variant").ok().flatten();
                s_url.map(|u| (u, s_var.unwrap_or_else(|| "classic".to_string())))
            } else {
                None
            }
        } else {
            None
        };

        if let Some((skin_source, variant)) = skin_info {
            appearance_variant = variant.clone();
            self.emit_log(&format!("Applying customized player skin ({}) for {}...", variant, username));
            inject_player_skin(self.downloader.http(), game_dir, username, &skin_source, &variant, effective_cape.as_deref(), clean_mc_ver).await?;
        } else if is_msa && !has_explicit_cape {
            self.emit_log(&format!("Using official Mojang account skin directly from session server for {}...", username));
            clean_skin_injection(game_dir).await?;
        } else if !is_msa || has_explicit_cape {
            use base64::Engine;
            let default_source = format!("data:image/png;base64,{}", base64::engine::general_purpose::STANDARD.encode(include_bytes!("../../../../static/steve.png")));
            appearance_variant = "classic".to_string();
            inject_player_skin(self.downloader.http(), game_dir, username, &default_source, "classic", effective_cape.as_deref(), clean_mc_ver).await?;
        }


        self.emit_stage(LaunchStage::ResolvingArgs);
        let mut jvm_args = self.build_jvm_args(detail, &classpath, &natives_dir, game_dir, profile);
        jvm_args.extend(extra_jvm_args);

        let mut game_args = extra_game_args;
        game_args.extend(self.build_game_args(
            detail,
            username,
            uuid,
            access_token,
            user_type,
            game_dir,
            profile,
        ));

        if let Some(ip) = server_ip.filter(|s| !s.trim().is_empty()) {
            game_args = quick_join::arguments(game_args, clean_mc_ver, &serde_json::to_string(&detail.arguments).unwrap_or_default(), ip.trim(), server_port.filter(|port| *port > 0).unwrap_or(25565));
            self.emit_log(&format!("Quick Join: connecting directly to {}:{}", ip.trim(), server_port.unwrap_or(25565)));
        }

        self.emit_log(&format!("Main class: {}", main_class));

        // === FASE 1: FINAL VALIDATION GATE ===
        self.emit_stage(LaunchStage::ValidatingArgs);
        let rejected = validate_platform_args(&jvm_args);
        if !rejected.is_empty() {
            for &idx in &rejected {
                self.emit_log(&format!(
                    "REJECTED for platform {:?}: {} (this is a known incompatible flag)",
                    current_platform(),
                    &jvm_args[idx]
                ));
            }
        }
        let mut safe_jvm_args: Vec<String> = jvm_args
            .iter()
            .enumerate()
            .filter_map(|(idx, a)| {
                if rejected.contains(&idx) {
                    None
                } else {
                    Some(a.clone())
                }
            })
            .collect();

        let rejected_game = validate_platform_args(&game_args);
        if !rejected_game.is_empty() {
            for &idx in &rejected_game {
                self.emit_log(&format!(
                    "REJECTED game arg for platform {:?}: {}",
                    current_platform(),
                    &game_args[idx]
                ));
            }
        }
        let safe_game_args: Vec<String> = game_args
            .iter()
            .enumerate()
            .filter_map(|(idx, a)| {
                if rejected_game.contains(&idx) {
                    None
                } else {
                    Some(a.clone())
                }
            })
            .filter(|a| a != "--demo")
            .collect();

        #[cfg(any(target_os = "linux", target_os = "windows"))]
        {
            debug_assert!(
                !safe_jvm_args.iter().any(|a| a == "-XstartOnFirstThread"),
                "validate_platform_args failed: -XstartOnFirstThread leaked into the spawn"
            );
        }

        let is_modpack = profile.loader == "forge"
            || profile.loader == "neoforge"
            || profile.loader == "fabric"
            || profile.loader == "quilt"
            || profile.mod_count > 0;

        let has_mods = if let Ok(entries) = std::fs::read_dir(game_dir.join("mods")) {
            entries.filter_map(|e| e.ok()).any(|e| {
                e.path().extension().map_or(false, |ext| ext == "jar")
            })
        } else {
            false
        };

        if has_mods {
            self.patch_modpack_configs(game_dir);
        }

        let appearance_root = game_dir.join("resourcepacks/LuxmcCustomSkin/assets/minecraft/textures/entity");
        let appearance_skin = appearance_root.join(if appearance_variant == "slim" { "player/slim/alex.png" } else { "player/wide/steve.png" });
        let appearance_cape = appearance_root.join("cape.png");
        let pvp = !is_modpack && !has_mods && profile.loader == "vanilla";
        {
            let agent_path = game_dir.join("luxmc-client-agent.jar");
            if tokio::fs::read(&agent_path).await.ok().as_deref() != Some(CLIENT_AGENT_JAR) {
                tokio::fs::write(&agent_path, CLIENT_AGENT_JAR).await?;
            }
            safe_jvm_args.push(format!("-Dluxmc.appearance.uuid={uuid}"));
            if let Some(path) = crate::network::p2p_tunnel::private_lan_session_path() {
                safe_jvm_args.push(format!("-Dluxmc.p2p.session={}", path.display()));
            }
            if appearance_skin.is_file() {
                safe_jvm_args.push(format!("-Dluxmc.appearance.skin={}", appearance_skin.display()));
                safe_jvm_args.push(format!("-Dluxmc.appearance.model={}", appearance_variant));
                if appearance_cape.is_file() { safe_jvm_args.push(format!("-Dluxmc.appearance.cape={}", appearance_cape.display())); }
            }
            safe_jvm_args.push(format!("-javaagent:{}{}", agent_path.display(), if pvp { "" } else { "=appearance-only" }));
            self.emit_log("Luxmc Custom Skin: aparência local e skins compartilhadas de contas Luxmc");
        }

        #[cfg(target_os = "linux")]
        {
            safe_jvm_args.push("-Dorg.lwjgl.glfw.checkFilename=false".to_string());
        }


        let cmd_display = format!(
            "{} {} {} {}",
            java_path.display(),
            safe_jvm_args.join(" "),
            main_class,
            safe_game_args.join(" ")
        );
        self.emit_log(&format!("Command: {}", cmd_display));

        // === FASE 2: SPAWN ===
        self.emit_stage(LaunchStage::Spawning);
        tokio::fs::create_dir_all(game_dir).await.ok();
        let csharp_launcher = find_csharp_launcher();
        let mut cmd = if let Some(ref cs_bin) = csharp_launcher {
            self.emit_log(&format!("Iniciando através do motor de lançamento C# (.NET 8): {}", cs_bin.display()));
            let launch_payload = serde_json::json!({
                "versionId": detail.id,
                "gameDir": game_dir.to_string_lossy(),
                "assetsDir": self.downloader.assets_dir().to_string_lossy(),
                "assetIndex": detail.asset_index.as_ref().map(|a| a.id.clone()).unwrap_or_default(),
                "javaPath": java_path.to_string_lossy(),
                "mainClass": main_class,
                "classpath": classpath.iter().map(|p| p.to_string_lossy().to_string()).collect::<Vec<_>>(),
                "jvmArgs": safe_jvm_args,
                "gameArgs": safe_game_args,
                "username": username,
                "uuid": uuid,
                "accessToken": access_token,
                "userType": user_type,
                "memoryMb": profile.ram_mb.unwrap_or(4096),
                "enableVulkan": profile.use_vulkan,
                "serverIp": server_ip,
                "serverPort": server_port,
                "resolutionWidth": profile.resolution_w.unwrap_or(1280),
                "resolutionHeight": profile.resolution_h.unwrap_or(720)
            });
            let payload_path = game_dir.join(".luxmc_launch.json");
            let _ = tokio::fs::write(&payload_path, serde_json::to_string(&launch_payload).unwrap_or_default()).await;

            let mut c = crate::core::process::tokio_command(cs_bin);
            c.arg("launch").arg(&payload_path);
            c
        } else {
            let effective_java = if cfg!(windows) {
                if java_path.file_name().map_or(false, |f| f == "java.exe") {
                    let javaw_path = java_path.with_file_name("javaw.exe");
                    if javaw_path.exists() {
                        javaw_path
                    } else {
                        java_path.clone()
                    }
                } else {
                    java_path.clone()
                }
            } else {
                java_path.clone()
            };

            #[cfg(target_os = "linux")]
            let mut wrappers: Vec<&str> = Vec::new();
            #[cfg(target_os = "linux")]
            {
                if profile.use_gamemode && which::which("gamemoderun").is_ok() {
                    wrappers.push("gamemoderun");
                    self.emit_log("GameMode ativo para esta instância.");
                }
                if profile.use_mangohud && which::which("mangohud").is_ok() {
                    wrappers.push("mangohud");
                    self.emit_log("MangoHud ativo para esta instância.");
                }
            }
            #[cfg(target_os = "linux")]
            let mut c = if profile.use_gamescope && which::which("gamescope").is_ok() {
                let mut command = crate::core::process::tokio_command("gamescope");
                if let Some(width) = profile.gamescope_width.filter(|value| *value > 0) {
                    command.arg("-W").arg(width.to_string());
                }
                if let Some(height) = profile.gamescope_height.filter(|value| *value > 0) {
                    command.arg("-H").arg(height.to_string());
                }
                command.arg("-U");
                if profile.gamescope_fsr {
                    command.arg("--fsr-upscaling");
                }
                command.arg("--").args(&wrappers).arg(&effective_java);
                self.emit_log("Gamescope ativo para esta instância.");
                command
            } else if let Some(wrapper) = wrappers.first() {
                let mut command = crate::core::process::tokio_command(wrapper);
                command.args(&wrappers[1..]).arg(&effective_java);
                command
            } else {
                crate::core::process::tokio_command(&effective_java)
            };
            #[cfg(not(target_os = "linux"))]
            let mut c = crate::core::process::tokio_command(&effective_java);
            c.args(&safe_jvm_args)
                .arg(main_class)
                .args(&safe_game_args);
            c
        };
        for variable in ["JAVA_TOOL_OPTIONS", "_JAVA_OPTIONS", "JAVA_OPTIONS", "JDK_JAVA_OPTIONS", "CLASSPATH"] { cmd.env_remove(variable); }
        cmd.current_dir(game_dir)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .kill_on_drop(true)
            .env("PATH", std::env::var("PATH").unwrap_or_default());

        #[cfg(unix)]
        {
            cmd.process_group(0);
        }

        #[cfg(target_os = "linux")]
        {
            let mut ld_dirs: Vec<String> = Vec::new();
            ld_dirs.push(natives_dir.to_string_lossy().to_string());

            if let Some(parent) = java_path.parent() {
                let java_home = parent.parent().unwrap_or(parent);
                let lib_dir = java_home.join("lib");
                let amd64_dir = lib_dir.join("amd64");
                let candidates = [
                    amd64_dir.join("server"),
                    amd64_dir.join("jli"),
                    amd64_dir,
                    lib_dir.join("server"),
                    lib_dir.join("jli"),
                    lib_dir,
                ];
                for c in candidates {
                    if c.exists() {
                        ld_dirs.push(c.to_string_lossy().to_string());
                    }
                }
            }

            if let Ok(orig_ld) = std::env::var("LD_LIBRARY_PATH_ORIG") {
                let filtered: Vec<&str> = orig_ld
                    .split(':')
                    .filter(|p| !p.is_empty() && !p.contains(".mount_") && !p.contains("/tmp/.mount"))
                    .collect();
                for f in filtered {
                    ld_dirs.push(f.to_string());
                }
            } else if let Ok(current_ld) = std::env::var("LD_LIBRARY_PATH") {
                let filtered: Vec<&str> = current_ld
                    .split(':')
                    .filter(|p| !p.is_empty() && !p.contains(".mount_") && !p.contains("/tmp/.mount"))
                    .collect();
                for f in filtered {
                    ld_dirs.push(f.to_string());
                }
            }

            for sys_dir in ["/usr/lib64", "/usr/lib", "/usr/local/lib"] {
                if std::path::Path::new(sys_dir).exists() && !ld_dirs.iter().any(|d| d == sys_dir) {
                    ld_dirs.push(sys_dir.to_string());
                }
            }

            if ld_dirs.is_empty() {
                cmd.env_remove("LD_LIBRARY_PATH");
            } else {
                cmd.env("LD_LIBRARY_PATH", ld_dirs.join(":"));
            }

            if let Ok(orig_dirs) = std::env::var("XDG_DATA_DIRS_ORIG") {
                cmd.env("XDG_DATA_DIRS", orig_dirs);
            }

            cmd.env_remove("APPDIR");
            cmd.env_remove("APPIMAGE");
            cmd.env_remove("OWD");
            cmd.env_remove("LD_PRELOAD");
            cmd.env_remove("GIO_MODULE_DIR");
            cmd.env_remove("GTK_PATH");
            cmd.env_remove("GSETTINGS_SCHEMA_DIR");

            cmd.env("_JAVA_AWT_WM_NONREPARENTING", "1");
            if let Ok(display) = std::env::var("DISPLAY") {
                cmd.env("DISPLAY", display);
                cmd.env_remove("WAYLAND_DISPLAY");
                cmd.env("GLFW_PLATFORM", "x11");
                cmd.env("GDK_BACKEND", "x11");
                cmd.env("SDL_VIDEODRIVER", "x11");
                cmd.env("QT_QPA_PLATFORM", "xcb");
            } else {
                return Err(anyhow::anyhow!("Minecraft requer X11/XWayland: DISPLAY não está definido. Ative o XWayland e tente novamente.").into());
            }

            if profile.use_vulkan {
                self.emit_log("Mesa Zink (OpenGL sobre Vulkan) aceleração ativa");
                cmd.env("MESA_LOADER_DRIVER_OVERRIDE", "zink");
                cmd.env("GALLIUM_DRIVER", "zink");
                cmd.env("MESA_SHADER_CACHE_MAX_SIZE", "100G");
                cmd.env("RADV_PERFTEST", "aco");
                cmd.env("mesa_glthread", "true");
                cmd.env("MESA_GLTHREAD", "true");
                cmd.env("__GL_THREADED_OPTIMIZATIONS", "1");
            } else {
                cmd.env("MESA_SHADER_CACHE_MAX_SIZE", "100G");
                cmd.env("mesa_glthread", "true");
                cmd.env("MESA_GLTHREAD", "true");
                cmd.env("__GL_THREADED_OPTIMIZATIONS", "1");
            }
        }

        let gpu = tokio::task::spawn_blocking(crate::core::optimizer::detect_gpu).await
            .map_err(|error| crate::error::AppError::Internal(error.to_string()))?;
        self.emit_log(&format!("GPU detectada para lançamento: {} (Fabricante: {}, Driver: {})", gpu.renderer, gpu.vendor, gpu.driver));

        #[cfg(target_os = "windows")]
        {
            if profile.auto_optimize {
                cmd.creation_flags(
                    crate::core::process::CREATE_NO_WINDOW
                        | crate::core::process::ABOVE_NORMAL_PRIORITY_CLASS,
                );
                self.emit_log("Windows: prioridade acima do normal ativada para o jogo.");
            }
            let system_root = std::env::var("SystemRoot").unwrap_or_else(|_| "C:\\Windows".to_string());
            let system32 = format!("{}\\System32", system_root);
            let mut win_paths = Vec::new();
            win_paths.push(natives_dir.to_string_lossy().to_string());
            if let Some(parent) = java_path.parent() {
                win_paths.push(parent.to_string_lossy().to_string());
            }
            win_paths.push(system32);
            if let Ok(current_path) = std::env::var("PATH").or_else(|_| std::env::var("Path")) {
                win_paths.push(current_path);
            }
            let joined_path = win_paths.join(";");
            cmd.env("PATH", &joined_path);
            cmd.env("Path", &joined_path);
            for key in ["SHIM_MCCOMPAT", "GPU_MAX_ALLOC_PERCENT", "GPU_USE_SYNC_OBJECTS", "GPU_NUM_COMPUTE_RINGS", "GPU_MAX_HEAP_SIZE", "GPU_FORCE_64BIT_PTR"] {
                cmd.env_remove(key);
            }

            if profile.force_dedicated_gpu {
                if let Ok((hkcu, _)) = winreg::RegKey::predef(winreg::enums::HKEY_CURRENT_USER)
                    .create_subkey("Software\\Microsoft\\DirectX\\UserGpuPreferences")
                {
                    for binary in [java_path.clone(), java_path.with_file_name("javaw.exe")] {
                        if binary.exists() {
                            let java_str = binary.to_string_lossy().to_string();
                            if let Err(error) = hkcu.set_value(&java_str, &"GpuPreference=2;") {
                                self.emit_log(&format!("Não foi possível definir preferência de GPU: {error}"));
                            }
                        }
                    }
                    self.emit_log("Windows: preferência de GPU de alto desempenho configurada para Java e Javaw.");
                }
            }
        }

        #[cfg(target_os = "linux")]
        {
            cmd.env("GPU_MAX_ALLOC_PERCENT", "100");
            cmd.env("GPU_USE_SYNC_OBJECTS", "1");
            cmd.env("GPU_NUM_COMPUTE_RINGS", "1");
            cmd.env("GPU_MAX_HEAP_SIZE", "100");
            cmd.env("GPU_FORCE_64BIT_PTR", "1");

            if profile.force_dedicated_gpu && gpu.vendor == "NVIDIA" {
                cmd.env("__NV_PRIME_RENDER_OFFLOAD", "1");
                cmd.env("__GLX_VENDOR_LIBRARY_NAME", "nvidia");
                cmd.env("__VK_LAYER_NV_optimus", "NVIDIA_only");
                cmd.env("__GL_THREADED_OPTIMIZATIONS", "1");
                if let Ok(entries) = std::fs::read_dir("/usr/share/vulkan/icd.d") {
                    let nvidia_icds = entries.flatten()
                        .map(|entry| entry.path())
                        .filter(|path| path.file_name().is_some_and(|name| name.to_string_lossy().to_lowercase().contains("nvidia")))
                        .map(|path| path.to_string_lossy().to_string())
                        .collect::<Vec<_>>();
                    if !nvidia_icds.is_empty() {
                        cmd.env("VK_ICD_FILENAMES", nvidia_icds.join(":"));
                    }
                }
            } else {
                cmd.env_remove("__GLX_VENDOR_LIBRARY_NAME");
                cmd.env_remove("__VK_LAYER_NV_optimus");
                cmd.env_remove("__NV_PRIME_RENDER_OFFLOAD");

                let drm_cards = std::fs::read_dir("/sys/class/drm").ok().map(|rd| {
                    rd.filter_map(|e| e.ok()).filter(|e| {
                        let name = e.file_name().to_string_lossy().to_string();
                        (name.starts_with("card0") || name.starts_with("card1") || name.starts_with("card2")) && !name.contains('-')
                    }).count()
                }).unwrap_or(0);
                if profile.force_dedicated_gpu && drm_cards > 1 {
                    cmd.env("DRI_PRIME", "1");
                } else {
                    cmd.env_remove("DRI_PRIME");
                }

                let has_sodium_like = if let Ok(mods) = std::fs::read_dir(game_dir.join("mods")) {
                    mods.filter_map(|e| e.ok()).any(|e| {
                        let name = e.file_name().to_string_lossy().to_lowercase();
                        name.contains("embeddium") || name.contains("sodium") || name.contains("rubidium")
                    })
                } else {
                    false
                };

                if has_sodium_like {
                    cmd.env_remove("mesa_glthread");
                    cmd.env_remove("MESA_GLTHREAD");
                    self.emit_log("Embeddium/Sodium detectado: mesa_glthread desativado para evitar conflito de renderização.");
                } else {
                    cmd.env("mesa_glthread", "true");
                    cmd.env("MESA_GLTHREAD", "true");
                }

                cmd.env("MESA_SHADER_CACHE_MAX_SIZE", "100G");
                if gpu.vendor == "AMD" {
                    cmd.env("AMD_POWERXPRESS_REQUEST_HIGH_PERFORMANCE", "1");
                } else if gpu.vendor == "Intel" {
                    cmd.env("ANV_ENABLE_PIPELINE_CACHE", "1");
                }
            }

            let gamemode_libs = [
                "/usr/lib/libgamemodeauto.so.0",
                "/usr/lib64/libgamemodeauto.so.0",
                "/usr/lib/x86_64-linux-gnu/libgamemodeauto.so.0",
            ];
            for lib in gamemode_libs {
                if !profile.use_gamemode {
                    break;
                }
                if std::path::Path::new(lib).exists() {
                    let existing = std::env::var("LD_PRELOAD").unwrap_or_default();
                    if existing.is_empty() {
                        cmd.env("LD_PRELOAD", lib);
                    } else if !existing.contains("libgamemodeauto") {
                        cmd.env("LD_PRELOAD", format!("{}:{}", lib, existing));
                    }
                    self.emit_log(&format!("GameMode ativo via preload: {}", lib));
                    break;
                }
            }
        }

        let mut child = cmd.spawn().map_err(|e| {
            let msg = format!("Failed to start Java process: {}", e);
            self.emit_log(&msg);
            self.emit_stage(LaunchStage::Failed);
            crate::error::AppError::Internal(msg)
        })?;

        let pid = child.id().unwrap_or(0);
        if pid > 0 {
            set_active_game_pid(pid);
            set_active_game_dir(game_dir.clone());
        }
        self.emit_log(&format!("Java process started, PID: {}", pid));
        self.emit_stage(LaunchStage::Running);

        // Ghost mode: immediately release unused launcher memory
        crate::commands::optimizer::optimizer_trim_memory();

        if pid > 0 && profile.use_gamemode {
            tokio::spawn(async move {
                crate::core::linux::gamemode::request_gamemode_for_pid(pid).await;
            });
        }

        let start_time = std::time::Instant::now();
        let peak_ram_bytes = std::sync::Arc::new(std::sync::atomic::AtomicU64::new(0));
        let peak_ram_tracker = peak_ram_bytes.clone();

        if pid > 0 {
            tokio::spawn(async move {
                let mut sys = sysinfo::System::new();
                let spid = sysinfo::Pid::from_u32(pid);
                loop {
                    tokio::time::sleep(std::time::Duration::from_secs(3)).await;
                    sys.refresh_processes(sysinfo::ProcessesToUpdate::Some(&[spid]), true);
                    if let Some(proc) = sys.process(spid) {
                        let mem = proc.memory();
                        let current_max = peak_ram_tracker.load(std::sync::atomic::Ordering::Relaxed);
                        if mem > current_max {
                            peak_ram_tracker.store(mem, std::sync::atomic::Ordering::Relaxed);
                        }
                        if !cfg!(windows) { crate::commands::optimizer::optimizer_trim_memory(); }
                    } else {
                        break;
                    }
                }
            });
        }

        let stdout = child.stdout.take().unwrap();
        let stderr = child.stderr.take().unwrap();

        let log_buffer: Arc<TokioMutex<Vec<String>>> = Arc::new(TokioMutex::new(Vec::with_capacity(200)));
        let lb_out = log_buffer.clone();
        let lb_err = log_buffer.clone();

        let last_stderr = std::sync::Arc::new(std::sync::Mutex::new(None::<String>));
        let last_stderr_writer = last_stderr.clone();

        let log_dir = game_dir.join("logs");
        let log_file_path = log_dir.join("luxmc_game.log");
        let _ = tokio::fs::create_dir_all(&log_dir).await;
        let mut log_file = tokio::fs::OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(&log_file_path)
            .await
            .ok();
        let mut log_file_err = if let Some(ref f) = log_file {
            f.try_clone().await.ok()
        } else {
            None
        };

        let app_world_notify = self.app.clone();
        let stdout_reader = tokio::spawn(async move {
            let mut reader = BufReader::new(stdout);
            while let Ok(Some(line)) = crate::core::process::read_log_line(&mut reader).await {
                if let Some(ref mut f) = log_file {
                    let _ = tokio::io::AsyncWriteExt::write_all(f, format!("{}\n", line).as_bytes()).await;
                }
                let mut buffer = lb_out.lock().await;
                if buffer.len() < 200 { buffer.push(line.clone()); }
                drop(buffer);

                if let Some(ref app) = app_world_notify {
                    if line.contains("Preparing level \"") {
                        if let Some(start) = line.find("Preparing level \"") {
                            let rest = &line[start + 17..];
                            if let Some(end) = rest.find('"') {
                                let world_name = &rest[..end];
                                let _ = app.emit("game-state-change", serde_json::json!({
                                    "status": "singleplayer",
                                    "detail": world_name
                                }));
                            }
                        }
                    } else if line.contains("Starting integrated minecraft server") {
                        let _ = app.emit("game-state-change", serde_json::json!({
                            "status": "singleplayer",
                            "detail": ""
                        }));
                    } else if line.contains("Connecting to ") {
                        let parts: Vec<&str> = line.split("Connecting to ").collect();
                        let target = parts.get(1).map(|s| s.trim()).unwrap_or("");
                        let host = target.split(',').next().unwrap_or(target).trim();
                        let _ = app.emit("game-state-change", serde_json::json!({
                            "status": "multiplayer",
                            "detail": host
                        }));
                    } else if line.contains("Stopping server") || line.contains("Saving and disconnecting") {
                        let _ = app.emit("game-state-change", serde_json::json!({
                            "status": "menu",
                            "detail": ""
                        }));
                    }
                }
            }
        });

        let stderr_reader = tokio::spawn(async move {
            let mut reader = BufReader::new(stderr);
            while let Ok(Some(line)) = crate::core::process::read_log_line(&mut reader).await {
                let trimmed = line.trim();
                if !trimmed.is_empty() {
                    let is_noise = trimmed.contains("[ALSOFT]")
                        || trimmed.contains("Failed to set real-time priority")
                        || trimmed.contains("AL_SOFT")
                        || trimmed.contains("MESA-LOADER")
                        || trimmed.contains("libEGL warning");
                    if !is_noise {
                        if let Ok(mut lock) = last_stderr_writer.lock() {
                            *lock = Some(trimmed.to_string());
                        }
                    }
                }
                if let Some(ref mut f) = log_file_err {
                    let _ = tokio::io::AsyncWriteExt::write_all(f, format!("{}\n", line).as_bytes()).await;
                }
                let mut buffer = lb_err.lock().await;
                if buffer.len() < 200 { buffer.push(line); }
            }
        });

        let is_game_active = Arc::new(std::sync::atomic::AtomicBool::new(true));
        let is_game_active_flusher = is_game_active.clone();

        // Batch flusher: emit accumulated logs every 500ms to avoid IPC flood
        let app_flush = self.app.clone();
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(std::time::Duration::from_millis(500));
            while is_game_active_flusher.load(std::sync::atomic::Ordering::Relaxed) {
                interval.tick().await;
                let mut buf = log_buffer.lock().await;
                if buf.is_empty() { continue; }
                let batch: Vec<String> = std::mem::take(&mut *buf);
                drop(buf);
                if let Some(ref app) = app_flush {
                    let _ = app.emit("game-log", GameLogEntry {
                        stream: "game".into(),
                        message: batch.join("\n"),
                    });
                }
            }
            // Final flush of any leftover logs after game exits
            let mut buf = log_buffer.lock().await;
            if !buf.is_empty() {
                let batch: Vec<String> = std::mem::take(&mut *buf);
                drop(buf);
                if let Some(ref app) = app_flush {
                    let _ = app.emit("game-log", GameLogEntry {
                        stream: "game".into(),
                        message: batch.join("\n"),
                    });
                }
            }
        });

        let app_exit = self.app.clone();
        let detail_id = detail.id.clone();
        let profile_id = profile.id.clone();
        let profile_name = profile.name.clone();
        let use_gamemode = profile.use_gamemode;
        let last_stderr_reader = last_stderr.clone();
        let is_game_active_waiter = is_game_active.clone();
        let game_dir_for_crash = game_dir.clone();
        let post_exit_hook = profile.post_exit_hook.clone();
        let hook_profile_id = profile.id.clone();
        let hook_profile_name = profile.name.clone();
        let hook_game_dir = game_dir.clone();
        let hook_version_id = detail.id.clone();
        tokio::spawn(async move {
            let result = child.wait().await;
            if result.as_ref().is_ok_and(|status| status.success()) || game_dir_for_crash.join("local/crash_assistant").join(format!("normal_stop_pid{pid}.tmp")).is_file() {
                if let Err(error) = crate::commands::launch::stop_owned_crash_assistant(game_dir_for_crash.clone(), pid).await { tracing::warn!(target: "launch", "{}", error); }
            }
            tokio::join!(finish_log_reader(stdout_reader), finish_log_reader(stderr_reader));
            match result {
                Ok(status) => {
                    if get_active_game_pid() == pid { clear_active_game_pid(); clear_active_game_dir(); }
                    is_game_active_waiter.store(false, std::sync::atomic::Ordering::Relaxed);
                    let code = status.code().unwrap_or(-1);
                    if pid > 0 && use_gamemode {
                        crate::core::linux::gamemode::release_gamemode_for_pid(pid).await;
                    }
                    let duration_secs = start_time.elapsed().as_secs();
                    let peak_mb = peak_ram_bytes.load(std::sync::atomic::Ordering::Relaxed) / (1024 * 1024);
                    let msg = format!("Game process exited with code {}", code);
                    tracing::info!(target: "launch", "{}", msg);
                    let raw_stderr = last_stderr_reader.lock().ok().and_then(|g| g.clone());
                    let error_message = if !status.success() {
                        Self::extract_actual_crash_reason(&game_dir_for_crash, raw_stderr)
                    } else {
                        None
                    };
                    let _ = crate::commands::experience::record_performance(&profile_id,serde_json::json!({"kind":"session","pid":pid,"durationSeconds":duration_secs,"peakRamMb":peak_mb,"exitCode":code,"cleanExit":status.success()})).await;
                    if let Some(ref app) = app_exit {
                        let _ = app.emit(
                            "game-exit",
                            GameExitEvent {
                                version_id: detail_id.clone(),
                                code,
                                success: status.success(),
                                error_message,
                            },
                        );
                        let _ = app.emit(
                            "game-telemetry-summary",
                            serde_json::json!({
                                "profileId": profile_id,
                                "profileName": profile_name,
                                "versionId": detail_id,
                                "durationSeconds": duration_secs,
                                "peakRamMb": peak_mb,
                                "exitCode": code,
                                "cleanExit": status.success(),
                                "timestamp": chrono::Utc::now().to_rfc3339(),
                            }),
                        );
                    }
                    if let Some(hook) = post_exit_hook.as_deref() {
                        let hook_env = crate::core::hooks::HookEnv {
                            profile_id: &hook_profile_id,
                            profile_name: &hook_profile_name,
                            version_id: &hook_version_id,
                            game_dir: &hook_game_dir,
                            exit_code: Some(code),
                        };
                        if let Err(e) = crate::core::hooks::run("pós-encerramento", hook, &hook_env).await
                        {
                            tracing::warn!(target: "hooks", "{}", e);
                            if let Some(ref app) = app_exit {
                                let _ = app.emit("launcher-log", format!("⚠️ {e}"));
                            }
                        }
                    }
                }
                Err(e) => {
                    if get_active_game_pid() == pid { clear_active_game_pid(); clear_active_game_dir(); }
                    is_game_active_waiter.store(false, std::sync::atomic::Ordering::Relaxed);
                    let msg = format!("Game process error: {}", e);
                    tracing::error!(target: "launch", "{}", msg);
                    if let Some(ref app) = app_exit {
                        let _ = app.emit(
                            "game-exit",
                            GameExitEvent {
                                version_id: detail_id,
                                code: -1,
                                success: false,
                                error_message: Some(e.to_string()),
                            },
                        );
                    }
                }
            }
        });

        Ok(pid)
    }

    fn build_classpath(&self, detail: &VersionDetail) -> Vec<PathBuf> {
        let base = self.downloader.libraries_dir();
        let mut cp = Vec::new();

        for lib in &detail.libraries {
            if !is_library_allowed(lib) {
                continue;
            }
            let path = lib_path_from_name(&base, &lib.name);
            if path.exists() {
                cp.push(path);
            }
        }

        if detail.downloads.is_some() {
            let versions_dir = self.downloader.versions_dir();
            let client_jar = versions_dir
                .join(&detail.id)
                .join(format!("{}.jar", detail.id));
            if client_jar.exists() {
                cp.push(client_jar);
            }
        }

        cp
    }

    async fn prepare_natives(&self, detail: &VersionDetail) -> AppResult<PathBuf> {
        extract_natives(&self.downloader, detail, self.app.as_ref()).await
    }
}

pub async fn extract_natives(
    downloader: &DownloadManager,
    detail: &VersionDetail,
    app: Option<&tauri::AppHandle>,
) -> AppResult<PathBuf> {
    let log = |message: &str| {
        if let Some(app) = app {
            let _ = app.emit("launcher-log", message);
        }
        tracing::info!(target: "natives", "{}", message);
    };
    let base = downloader.libraries_dir();
    let versions_dir = downloader.versions_dir();
    let natives_dir = versions_dir.join(&detail.id).join("natives");
    tokio::fs::create_dir_all(&natives_dir).await?;

    let os_name = platform_mojang_name(current_platform());
    let mut extract_error: Option<std::io::Error> = None;

    let mut archive_paths = Vec::new();
    let mut seen_archives = std::collections::HashSet::new();
    for lib in &detail.libraries {
        if !is_library_allowed(lib) {
            continue;
        }

        let mut candidate_paths = Vec::new();

        if lib.name.contains(&format!("natives-{}", os_name)) {
            candidate_paths.push(lib_path_from_name(&base, &lib.name));
        }

        if let Some(ref downloads) = lib.downloads {
            if let Some(ref classifiers) = downloads.classifiers {
                let mut classifier_keys = classifiers.keys().collect::<Vec<_>>();
                classifier_keys.sort();
                for classifier_key in classifier_keys {
                    let matches_os = match os_name {
                        "windows" => classifier_key.starts_with("natives-windows"),
                        "linux" => classifier_key.starts_with("natives-linux"),
                        "osx" => {
                            classifier_key.starts_with("natives-osx")
                                || classifier_key.starts_with("natives-macos")
                        }
                        _ => false,
                    };
                    if matches_os && is_native_classifier_allowed(classifier_key) {
                        let native_lib_name = format!("{}:{}", lib.name, classifier_key);
                        candidate_paths.push(lib_path_from_name(&base, &native_lib_name));
                    }
                }
            }
        }

        if let Some(ref natives_map) = lib.natives {
            if let Some(native_key) = natives_map.get(os_name) {
                let arch = if cfg!(target_arch = "x86") { "32" } else { "64" };
                let native_key = native_key.replace("${arch}", arch);
                let native_lib_name = format!("{}:{}", lib.name, native_key);
                candidate_paths.push(lib_path_from_name(&base, &native_lib_name));
            }
        }

        for path in candidate_paths {
            if seen_archives.insert(path.clone()) {
                archive_paths.push(path);
            }
        }
    }

    #[cfg(target_os = "windows")]
    if native_cache::is_valid(&natives_dir, &archive_paths) {
        log("Native libraries unchanged; reusing verified extraction without rewriting DLLs");
        return Ok(natives_dir);
    }

    #[cfg(target_os = "windows")]
    let original_inputs = native_cache::capture(&archive_paths);
    let extraction_started = std::time::Instant::now();
    let mut output_names = std::collections::BTreeSet::new();
    let mut all_archives_opened = true;
    tokio::task::block_in_place(|| {
        for path in &archive_paths {
            if path.exists() {
                if let Ok(file) = std::fs::File::open(path) {
                    if let Ok(mut archive) = zip::ZipArchive::new(file) {
                        for i in 0..archive.len() {
                            if let Ok(mut file) = archive.by_index(i) {
                                if file.is_dir() {
                                    continue;
                                }
                                let enclosed = match file.enclosed_name() {
                                    Some(name) => name.to_path_buf(),
                                    None => continue,
                                };
                                if enclosed.starts_with("META-INF") {
                                    continue;
                                }
                                if let Some(file_name) = enclosed.file_name() {
                                    let outpath = natives_dir.join(file_name);
                                    let copy_result = (|| -> std::io::Result<()> {
                                        let mut outfile = std::fs::File::create(&outpath)?;
                                        std::io::copy(&mut file, &mut outfile)?;
                                        std::io::Write::flush(&mut outfile)
                                    })();
                                    if copy_result.is_ok() {
                                        output_names.insert(PathBuf::from(file_name));
                                    }
                                    if let Err(e) = copy_result {
                                        let _ = std::fs::remove_file(&outpath);
                                        if extract_error.is_none() {
                                            extract_error = Some(e);
                                        }
                                    }
                                }
                            }
                        }
                    } else {
                        all_archives_opened = false;
                        log(&format!(
                            "Failed to open native zip archive: {}",
                            path.display()
                        ));
                    }
                } else {
                    all_archives_opened = false;
                    log(&format!(
                        "Failed to open native jar file: {}",
                        path.display()
                    ));
                }
            } else {
                all_archives_opened = false;
            }
        }
    });

    if cfg!(target_os = "linux") {
        match ensure_flite_library(&natives_dir).await {
            Ok(true) => log("Flite narrator library ready"),
            Ok(false) => log(
                "Flite narrator unavailable; Minecraft will continue without narration",
            ),
            Err(error) => log(&format!("Flite narrator unavailable: {}", error)),
        }
    }

    tokio::task::block_in_place(|| {
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            if let Ok(entries) = std::fs::read_dir(&natives_dir) {
                for entry in entries.flatten() {
                    let p = entry.path();
                    if p.is_file() {
                        let is_lib = p
                            .extension()
                            .map(|ext| ext == "so" || ext == "dylib")
                            .unwrap_or(false);
                        if is_lib {
                            let _ =
                                std::fs::set_permissions(&p, std::fs::Permissions::from_mode(0o755));
                        }
                    }
                }
            }
        }

        #[cfg(target_os = "linux")]
        {
            let candidate_glfw_paths = [
                "/usr/lib/libglfw.so",
                "/usr/lib/libglfw.so.3",
                "/usr/lib64/libglfw.so",
                "/usr/lib64/libglfw.so.3",
                "/usr/lib/x86_64-linux-gnu/libglfw.so",
                "/usr/lib/x86_64-linux-gnu/libglfw.so.3",
            ];
            for path in candidate_glfw_paths {
                let p = std::path::Path::new(path);
                if p.exists() {
                    let target = natives_dir.join("libglfw.so");
                    let _ = std::fs::remove_file(&target);
                    #[cfg(unix)]
                    if std::os::unix::fs::symlink(p, &target).is_err() {
                        let _ = std::fs::copy(p, &target);
                    }
                    #[cfg(not(unix))]
                    let _ = std::fs::copy(p, &target);
                    log(&format!("Using system GLFW library: {}", path));
                    break;
                }
            }
        }

        let native_count = std::fs::read_dir(&natives_dir)
            .map(|rd| {
                rd.filter(|e| {
                    e.as_ref()
                        .map(|f| {
                            f.path()
                                .extension()
                                .map(|ext| ext == "so" || ext == "dll" || ext == "dylib")
                                .unwrap_or(false)
                        })
                        .unwrap_or(false)
                })
                .count()
            })
            .unwrap_or(0);
        log(&format!(
            "Extracted {} native libraries to {}",
            native_count,
            natives_dir.display()
        ));
    });

    if let Some(e) = extract_error {
        return Err(crate::error::AppError::Io(e));
    }

    #[cfg(target_os = "windows")]
    if let Some(inputs) = original_inputs.filter(|_| all_archives_opened) {
        if let Err(error) = native_cache::store(&natives_dir, &archive_paths, &output_names, &inputs) {
            log(&format!("Native extraction cache could not be saved: {error}"));
        }
    }
    #[cfg(not(target_os = "windows"))]
    let _ = all_archives_opened;
    log(&format!("Native preparation completed in {:.0}ms", extraction_started.elapsed().as_secs_f64() * 1000.0));

    Ok(natives_dir)
}

impl GameLauncher {
    fn build_jvm_args(
        &self,
        detail: &VersionDetail,
        classpath: &[PathBuf],
        natives_dir: &PathBuf,
        game_dir: &PathBuf,
        profile: &crate::db::models::ProfileRow,
    ) -> Vec<String> {
        let mut args = Vec::new();

        let cp_str = classpath
            .iter()
            .map(|p| p.to_string_lossy().to_string())
            .collect::<Vec<_>>()
            .join(if cfg!(target_os = "windows") {
                ";"
            } else {
                ":"
            });

        if let Some(ref jvm_args) = detail.effective_jvm_args() {
            for arg in jvm_args {
                let values: Vec<String> = if let Some(s) = arg.as_str() {
                    vec![s.to_string()]
                } else if let Some(obj) = arg.as_object() {
                    if let Some(rules) = obj.get("rules").and_then(|r| r.as_array()) {
                        if !self.evaluate_jvm_rules(rules) {
                            continue;
                        }
                    }

                    if let Some(val) = obj.get("value") {
                        if let Some(s) = val.as_str() {
                            vec![s.to_string()]
                        } else if let Some(arr) = val.as_array() {
                            arr.iter()
                                .filter_map(|v| v.as_str().map(String::from))
                                .collect()
                        } else {
                            continue;
                        }
                    } else {
                        continue;
                    }
                } else {
                    continue;
                };

                for s in values {
                    let resolved = s
                        .replace("${natives_directory}", &natives_dir.to_string_lossy())
                        .replace("${launcher_name}", LAUNCHER_NAME)
                        .replace("${launcher_version}", LAUNCHER_VERSION)
                        .replace("${classpath}", &cp_str)
                        .replace(
                            "${library_directory}",
                            &self.downloader.libraries_dir().to_string_lossy(),
                        )
                        .replace("${version_name}", &detail.id)
                        .replace("${game_directory}", &game_dir.to_string_lossy());

                    // If it is a classpath, property (-D), or single parameter, don't split spaces!
                    if s.contains("${classpath}") || resolved.starts_with("-D") || resolved.starts_with("-Xbootclasspath") {
                        if jvm_arg_allowed_on_current_os(&resolved) {
                            args.push(resolved);
                        }
                    } else {
                        for sub in resolved.split_whitespace() {
                            if !jvm_arg_allowed_on_current_os(sub) {
                                self.emit_log(&format!("Dropped JVM arg on this platform: {}", sub));
                                continue;
                            }
                            args.push(sub.to_string());
                        }
                    }
                }
            }
        } else {
            args.push(format!(
                "-Djava.library.path={}",
                natives_dir.to_string_lossy()
            ));
            args.push("-cp".to_string());
            args.push(cp_str.clone());
        }

        let actual_mods_count = if let Ok(entries) = std::fs::read_dir(game_dir.join("mods")) {
            entries.filter_map(|e| e.ok()).filter(|e| {
                e.path().extension().map_or(false, |ext| {
                    let s = ext.to_string_lossy().to_lowercase();
                    s == "jar" || s == "disabled"
                })
            }).count() as i64
        } else {
            0
        };
        let effective_mod_count = profile.mod_count.max(actual_mods_count);

        // 100% Automatic RAM Allocation: dynamically inspects user hardware and mod weight
        let is_heavy_modded = profile.loader == "forge"
            || profile.loader == "neoforge"
            || profile.name.to_lowercase().contains("all the mods")
            || profile.name.to_lowercase().contains("atm")
            || profile.name.to_lowercase().contains("better mc")
            || profile.name.to_lowercase().contains("dawncraft")
            || profile.name.to_lowercase().contains("prominence")
            || profile.name.to_lowercase().contains("rlcraft")
            || effective_mod_count >= 50;

        let _is_pvp = profile.clean_mc_version().starts_with("1.8") || profile.clean_mc_version().starts_with("1.7");
        let ram_mb = if let Some(manual) = profile.ram_mb {
            if manual > 0 {
                self.emit_log(&format!(
                    "Luxmc RAM: {}MB alocados (configurado pelo usuário)",
                    manual
                ));
                manual
            } else {
                4096
            }
        } else {
            4096
        };

        // Apply Intelligent Luxmc Optimization (Aikar's Flags) or Standard Flags
        let custom_collector = profile.jvm_args.as_deref().is_some_and(|args|
            args.split_whitespace().any(|arg| matches!(arg, "-XX:+UseZGC" | "-XX:+UseShenandoahGC" | "-XX:+UseParallelGC" | "-XX:+UseSerialGC")));
        let mut generated_flags = if custom_collector {
            vec![format!("-Xms{}M", ram_mb.min(1024)), format!("-Xmx{}M", ram_mb)]
        } else if profile.auto_optimize {
            crate::core::optimizer::generate_optimized_flags(ram_mb.max(0) as u64, detail.java_major_version())
        } else {
            crate::core::optimizer::generate_standard_flags(ram_mb.max(0) as u64)
        };

        if !cfg!(windows) && (is_heavy_modded || profile.loader == "forge" || profile.loader == "neoforge" || profile.loader == "fabric" || profile.loader == "quilt") && !custom_collector {
            if let Some(pos) = generated_flags.iter().position(|f| f.starts_with("-Xms")) {
                let initial_ms = (ram_mb / 4).max(2048).min(ram_mb);
                generated_flags[pos] = format!("-Xms{}M", initial_ms);
            }
        }

        for flag in generated_flags {
            let key = if flag.starts_with("-Xms") {
                "-Xms"
            } else if flag.starts_with("-Xmx") {
                "-Xmx"
            } else if flag.starts_with("-Xss") {
                "-Xss"
            } else {
                flag.split('=').next().unwrap_or(&flag)
            };
            if !args.iter().any(|a| a.starts_with(key)) {
                args.push(flag);
            }
        }

        if is_heavy_modded || profile.loader == "forge" || profile.loader == "neoforge" || profile.loader == "fabric" || profile.loader == "quilt" {
            for opt in [
                "-XX:+TieredCompilation",
                "-Dfml.ignoreInvalidMinecraftCertificates=true",
                "-Dfml.ignorePatchDiscrepancies=true",
                "-Dio.netty.allocator.type=pooled",
                "-Dio.netty.recycler.maxCapacityPerThread=0",
                "-XX:+OptimizeStringConcat",
                "-Dsun.rmi.dgc.server.gcInterval=2147483646",
                "-Dsun.rmi.dgc.client.gcInterval=2147483646",
                "-Dlog4j2.formatMsgNoLookups=true",
            ] {
                if !args.iter().any(|a| a.starts_with(opt)) {
                    args.push(opt.to_string());
                }
            }
        }

        // Apply user-specified custom JVM arguments (take precedence)
        if let Some(ref custom_args) = profile.jvm_args {
            if !custom_args.is_empty() {
                for arg in custom_args.split_whitespace() {
                    // If user manually specifies -Xms or -Xmx, replace existing
                    if arg.starts_with("-Xmx") {
                        args.retain(|a| !a.starts_with("-Xmx"));
                    } else if arg.starts_with("-Xms") {
                        args.retain(|a| !a.starts_with("-Xms"));
                    }
                    args.push(arg.to_string());
                }
            }
        }

        args.retain(|a| !a.starts_with("-Dneoforge.earlydisplay="));
        args.push("-Dneoforge.earlydisplay=false".to_string());
        args.retain(|a| !a.starts_with("-Dfml.earlyprogresswindow="));
        args.push("-Dfml.earlyprogresswindow=false".to_string());

        if !args.iter().any(|a| a.starts_with("-Dorg.lwjgl.glfw.checkThread0=")) {
            args.push("-Dorg.lwjgl.glfw.checkThread0=false".to_string());
        }

        if !args.iter().any(|a| a.starts_with("-Dorg.lwjgl.opengl.Display.allowSoftwareOpenGL=")) {
            args.push("-Dorg.lwjgl.opengl.Display.allowSoftwareOpenGL=true".to_string());
        }

        if detail.java_major_version() >= 17 {
            for module in [
                "java.base/java.lang",
                "java.base/java.util",
                "java.base/java.io",
                "java.base/java.nio",
                "java.base/sun.security.util",
            ] {
                let open_flag = format!("--add-opens={}={}", module, "ALL-UNNAMED");
                if !args.contains(&open_flag) {
                    args.push(open_flag);
                }
            }
        }

        #[cfg(target_os = "windows")]
        {
            let intel_fix = "-XX:HeapDumpPath=MojangTricksIntelDriversForPerformance_javaw.exe_minecraft.exe.heapdump";
            if !args.iter().any(|a| a.starts_with("-XX:HeapDumpPath=")) {
                args.push(intel_fix.to_string());
            }
        }

        args
    }

    fn evaluate_jvm_rules(&self, rules: &[serde_json::Value]) -> bool {
        evaluate_rules(rules)
    }
}

pub fn evaluate_rules(rules: &[serde_json::Value]) -> bool {
    let current = current_platform();
    let current_name = platform_mojang_name(current);

    let mut allowed = false;
    for rule in rules {
        let action = rule
            .get("action")
            .and_then(|a| a.as_str())
            .unwrap_or("allow");
        let mut matches = true;

        if let Some(os_obj) = rule.get("os").and_then(|os| os.as_object()) {
            if let Some(os_name) = os_obj.get("name").and_then(|n| n.as_str()) {
                if os_name != current_name {
                    matches = false;
                }
            }
            if let Some(arch) = os_obj.get("arch").and_then(|a| a.as_str()) {
                let is_match = match arch {
                    "x86" => cfg!(target_arch = "x86"),
                    "x86_64" | "x64" => cfg!(target_arch = "x86_64"),
                    "arm64" | "aarch64" => cfg!(target_arch = "aarch64"),
                    _ => false,
                };
                if !is_match {
                    matches = false;
                }
            }
        }

        if matches {
            allowed = match action {
                "allow" => true,
                "disallow" => false,
                _ => allowed,
            };
        }
    }
    allowed
}

impl GameLauncher {
    fn patch_modpack_configs(&self, game_dir: &std::path::Path) {
        let crash_assistant_cfg = game_dir.join("config").join("crash_assistant").join("config.toml");
        if crash_assistant_cfg.exists() {
            if let Ok(content) = std::fs::read_to_string(&crash_assistant_cfg) {
                let mut modified = content;
                if modified.contains("[piracy]") {
                    modified = modified.replace("delay = 10", "delay = 0");
                    modified = modified.replace("piracy_notification = true", "piracy_notification = false");
                    if let Some(pos) = modified.find("[piracy]") {
                        if let Some(enabled_pos) = modified[pos..].find("enabled = true") {
                            let actual_pos = pos + enabled_pos;
                            modified.replace_range(actual_pos..actual_pos + 14, "enabled = false");
                        }
                    }
                    let _ = std::fs::write(&crash_assistant_cfg, modified);
                }
            }
        }
    }

    fn build_game_args(
        &self,
        detail: &VersionDetail,
        username: &str,
        uuid: &str,
        access_token: &str,
        user_type: &str,
        game_dir: &PathBuf,
        profile: &crate::db::models::ProfileRow,
    ) -> Vec<String> {
        let mut args = Vec::new();

        let assets_dir = self.downloader.assets_dir();
        let asset_index_id = detail
            .asset_index
            .as_ref()
            .map(|ai| ai.id.as_str())
            .unwrap_or("legacy");

        let game_args_raw = detail.effective_game_args();

        let res_w = profile.resolution_w.unwrap_or(854).to_string();
        let res_h = profile.resolution_h.unwrap_or(480).to_string();

        for arg in &game_args_raw {
            let resolved = arg
                .replace("${auth_player_name}", username)
                .replace("${auth_uuid}", uuid)
                .replace("${auth_session}", access_token)
                .replace("${auth_access_token}", access_token)
                .replace("${user_properties}", "{}")
                .replace("${version_name}", &detail.id)
                .replace("${game_directory}", &game_dir.to_string_lossy())
                .replace("${assets_directory}", &assets_dir.to_string_lossy())
                .replace("${assets_root}", &assets_dir.to_string_lossy())
                .replace("${game_assets}", &assets_dir.to_string_lossy())
                .replace("${asset_index}", asset_index_id)
                .replace("${assets_index_name}", asset_index_id)
                .replace("${clientid}", DEV_CLIENT_ID)
                .replace("${auth_xuid}", DEV_XUID)
                .replace("${user_type}", user_type)
                .replace("${version_type}", &detail.version_type)
                .replace("${resolution_width}", &res_w)
                .replace("${resolution_height}", &res_h)
                .replace("${quickPlayPath}", "")
                .replace("${quickPlaySingleplayer}", "")
                .replace("${quickPlayMultiplayer}", "")
                .replace("${quickPlayRealms}", "");
            args.push(resolved);
        }

        if !args.contains(&"--username".to_string()) {
            args.push("--username".to_string());
            args.push(username.to_string());
            args.push("--version".to_string());
            args.push(detail.id.clone());
            args.push("--gameDir".to_string());
            args.push(game_dir.to_string_lossy().to_string());
            args.push("--assetsDir".to_string());
            args.push(assets_dir.to_string_lossy().to_string());
            args.push("--assetIndex".to_string());
            args.push(asset_index_id.to_string());
            args.push("--uuid".to_string());
            args.push(uuid.to_string());
            args.push("--accessToken".to_string());
            args.push(access_token.to_string());
            args.push("--userType".to_string());
            args.push(user_type.to_string());
            args.push("--userProperties".to_string());
            args.push("{}".to_string());
        }

        if let Some(w) = profile.resolution_w {
            if !args.contains(&"--width".to_string()) {
                args.push("--width".to_string());
                args.push(w.to_string());
            }
        }
        if let Some(h) = profile.resolution_h {
            if !args.contains(&"--height".to_string()) {
                args.push("--height".to_string());
                args.push(h.to_string());
            }
        }
        if profile.fullscreen && !args.contains(&"--fullscreen".to_string()) {
            args.push("--fullscreen".to_string());
        }

        args
    }
}

async fn ensure_flite_library(natives_dir: &PathBuf) -> AppResult<bool> {
    let target = natives_dir.join("libflite.so");
    if target.exists() {
        return Ok(true);
    }

    // Flite is not shipped by Mojang; use only an existing distro-provided library.
    let candidates = [
        "/usr/lib/libflite.so",
        "/usr/lib/libflite.so.1",
        "/usr/lib/libflite.so.2",
        "/usr/lib64/libflite.so",
        "/usr/lib64/libflite.so.1",
        "/usr/lib64/libflite.so.2",
        "/lib/libflite.so",
        "/lib/libflite.so.1",
        "/lib/libflite.so.2",
        "/lib64/libflite.so",
        "/lib64/libflite.so.1",
        "/lib64/libflite.so.2",
    ];
    let Some(source) = candidates
        .iter()
        .map(PathBuf::from)
        .find(|path| path.is_file())
    else {
        return Ok(false);
    };

    let bytes = tokio::fs::read(&source).await?;
    if bytes.len() < 4 || &bytes[..4] != b"\x7fELF" {
        return Err(crate::error::AppError::Internal(format!(
            "invalid Flite library at {}",
            source.display()
        )));
    }
    tokio::fs::write(&target, bytes).await?;
    Ok(true)
}

#[cfg(test)]
mod tests {
    #[cfg(windows)]
    #[tokio::test]
    async fn stale_log_reader_releases_locked_files() {
        use std::os::windows::fs::OpenOptionsExt;
        let fixture = tempfile::tempdir().unwrap();
        let path = fixture.path().join("game.log");
        let file = std::fs::OpenOptions::new().write(true).create_new(true).share_mode(0).open(&path).unwrap();
        let task = tokio::spawn(async move { std::future::pending::<()>().await; drop(file); });
        super::finish_log_reader(task).await;
        std::fs::remove_file(path).unwrap();
    }
    use super::*;

    #[test]
    fn skin_pack_metadata_supports_legacy_and_minor_version_formats() {
        for format in [1, 15, 34, 64] {
            let value = skin_pack_metadata(format);
            assert_eq!(value["pack"]["pack_format"], format);
            assert!(value["pack"].get("min_format").is_none());
        }
        for format in [65, 69, 97] {
            let value = skin_pack_metadata(format);
            assert_eq!(value["pack"]["min_format"], format);
            assert_eq!(value["pack"]["max_format"], format);
            assert!(value["pack"].get("supported_formats").is_none());
            assert!(value["pack"].get("pack_format").is_none());
        }
    }

    #[test]
    fn early_jvm_failure_is_reported_without_a_minecraft_log() {
        let root = std::env::temp_dir().join(uuid::Uuid::new_v4().to_string());
        std::fs::create_dir_all(root.join("logs")).unwrap();
        std::fs::write(root.join("logs/luxmc_game.log"), "Error occurred during initialization of VM\nCould not reserve enough space for object heap\n").unwrap();
        assert_eq!(GameLauncher::extract_actual_crash_reason(&root, None).as_deref(), Some("Could not reserve enough space for object heap"));
        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn native_classifiers_do_not_mix_windows_cpu_architectures() {
        assert!(native_classifier_matches_arch("natives-windows", "x86_64"));
        assert!(native_classifier_matches_arch("natives-windows", "x86"));
        assert!(!native_classifier_matches_arch("natives-windows-arm64", "x86_64"));
        assert!(!native_classifier_matches_arch("natives-windows-x86", "x86_64"));
        assert!(native_classifier_matches_arch("natives-windows-x86", "x86"));
        assert!(native_classifier_matches_arch("natives-windows-arm64", "aarch64"));
        assert!(native_classifier_matches_arch("natives-windows-64", "x86_64"));
        assert!(!native_classifier_matches_arch("natives-windows-32", "x86_64"));
        assert!(native_classifier_matches_arch("sources", "x86_64"));
    }

    #[test]
    fn library_os_and_architecture_rules_preserve_last_matching_action() {
        let platform = platform_mojang_name(current_platform());
        let library = |rules: serde_json::Value| serde_json::from_value::<minecraft::Library>(serde_json::json!({"name":"fixture:library:1", "rules":rules})).unwrap();
        let mismatch = if cfg!(target_arch = "aarch64") { "x86_64" } else { "aarch64" };
        assert!(!is_library_allowed(&library(serde_json::json!([{"action":"allow","os":{"name":platform,"arch":mismatch}}]))));
        assert!(is_library_allowed(&library(serde_json::json!([{"action":"allow","os":{"name":platform,"arch":std::env::consts::ARCH}}]))));
        assert!(!is_library_allowed(&library(serde_json::json!([{"action":"allow"},{"action":"disallow","os":{"name":platform}}]))));
        assert!(is_library_allowed(&library(serde_json::json!([{"action":"disallow","os":{"name":platform}},{"action":"allow"}]))));
    }

    #[tokio::test]
    async fn appearance_keeps_other_packs_and_removes_stale_cape() {
        use base64::Engine;
        let directory = std::env::temp_dir().join(format!("luxmc-skin-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&directory).unwrap();
        std::fs::write(directory.join("options.txt"), "resourcePacks:[\"vanilla\",\"file/Other\",\"file/LuxmcCustomSkin\"]\n").unwrap();
        let skin = format!("data:image/png;base64,{}", base64::engine::general_purpose::STANDARD.encode(include_bytes!("../../../../static/steve.png")));
        let client = reqwest::Client::new();
        inject_player_skin(&client, &directory, "Player", &skin, "classic", Some(&skin), "1.21.1").await.unwrap();
        let cape = directory.join("resourcepacks/LuxmcCustomSkin/assets/minecraft/textures/entity/cape.png");
        assert!(cape.is_file());
        inject_player_skin(&client, &directory, "Player", &skin, "classic", None, "1.21.1").await.unwrap();
        assert!(!cape.exists());
        let options = std::fs::read_to_string(directory.join("options.txt")).unwrap();
        assert!(options.contains("resourcePacks:[\"vanilla\",\"file/Other\"]"));
        assert!(!options.contains("LuxmcCustomSkin"));
        std::fs::remove_dir_all(directory).unwrap();
    }
    #[tokio::test]
    async fn appearance_preserves_pack_defaults_on_first_launch_and_user_options_afterward() {
        use base64::Engine;
        let directory = std::env::temp_dir().join(format!("luxmc-yosbr-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(directory.join("config/yosbr")).unwrap();
        let defaults = "resourcePacks:[\"vanilla\",\"file/ProminenceFancyServerListing.zip\"]\nlang:pt_br\n";
        std::fs::write(directory.join("config/yosbr/options.txt"), defaults).unwrap();
        let skin = format!("data:image/png;base64,{}", base64::engine::general_purpose::STANDARD.encode(include_bytes!("../../../../static/steve.png")));
        let client = reqwest::Client::new();
        inject_player_skin(&client, &directory, "Player", &skin, "classic", None, "1.20.1").await.unwrap();
        let options = std::fs::read_to_string(directory.join("options.txt")).unwrap();
        assert!(options.contains("file/ProminenceFancyServerListing.zip"));
        assert!(options.contains("lang:pt_br"));
        let personal = "resourcePacks:[\"vanilla\",\"file/MyPack.zip\"]\nlang:en_us\n";
        std::fs::write(directory.join("options.txt"), personal).unwrap();
        inject_player_skin(&client, &directory, "Player", &skin, "classic", None, "1.20.1").await.unwrap();
        let options = std::fs::read_to_string(directory.join("options.txt")).unwrap();
        assert!(options.contains("file/MyPack.zip"));
        assert!(options.contains("lang:en_us"));
        assert!(!options.contains("ProminenceFancyServerListing"));
        assert_eq!(std::fs::read_to_string(directory.join("config/yosbr/options.txt")).unwrap(), defaults);
        std::fs::remove_dir_all(directory).unwrap();
    }
    use std::path::PathBuf;

    #[test]
    fn linux_forbids_macos_flag() {
        if cfg!(target_os = "linux") {
            let args = vec![
                "-Xms2G".to_string(),
                "-XstartOnFirstThread".to_string(),
                "-Xmx4G".to_string(),
            ];
            assert_eq!(validate_platform_args(&args), vec![1]);
        }
    }

    #[test]
    fn windows_forbids_macos_flag() {
        if cfg!(target_os = "windows") {
            let args = vec!["-XstartOnFirstThread".to_string()];
            assert_eq!(validate_platform_args(&args), vec![0]);
        }
    }

    #[test]
    fn macos_allows_start_on_first_thread() {
        if cfg!(target_os = "macos") {
            let args = vec!["-XstartOnFirstThread".to_string()];
            assert!(validate_platform_args(&args).is_empty());
        }
    }

    #[test]
    fn linux_forbids_windows_heapdump_prefix() {
        if cfg!(target_os = "linux") {
            let args = vec![
				"-Xms2G".to_string(),
				"-XX:HeapDumpPath=MojangTricksIntelDriversForPerformance_javaw.exe_minecraft.exe.heapdump".to_string(),
			];
            assert_eq!(validate_platform_args(&args), vec![1]);
        }
    }

    #[test]
    fn safe_args_pass_through() {
        let args = vec![
            "-Xss1M".to_string(),
            "-Xms2G".to_string(),
            "-Xmx4G".to_string(),
            "-Dfile.encoding=UTF-8".to_string(),
            "--sun-misc-unsafe-memory-access=allow".to_string(),
            "--enable-native-access=ALL-UNNAMED".to_string(),
        ];
        assert!(validate_platform_args(&args).is_empty());
    }

    #[test]
    fn platform_mojang_name_is_stable() {
        assert_eq!(platform_mojang_name(Platform::Linux), "linux");
        assert_eq!(platform_mojang_name(Platform::Windows), "windows");
        assert_eq!(platform_mojang_name(Platform::Macos), "osx");
    }

    #[test]
    fn current_platform_is_exactly_one() {
        // Exactly one of the three is the host.
        let known = [
            cfg!(target_os = "linux"),
            cfg!(target_os = "windows"),
            cfg!(target_os = "macos"),
        ];
        assert!(known.iter().filter(|b| **b).count() <= 1);
    }

    #[tokio::test]
    async fn missing_flite_is_a_non_fatal_fallback() {
        let path = PathBuf::from("/tmp/luxmc-test-natives-that-do-not-exist");
        let result = ensure_flite_library(&path).await;
        assert!(result.is_ok());
    }

    #[test]
    fn test_normalize_skin_64x32_to_64x64() {
        let legacy_img = image::RgbaImage::new(64, 32);
        let normalized = normalize_skin_image(legacy_img);
        assert_eq!(normalized.dimensions(), (64, 64));
    }

    #[test]
    fn test_normalize_skin_fixes_torso_back_alpha() {
        let mut img = image::RgbaImage::new(64, 64);
        img.put_pixel(24, 24, image::Rgba([200, 100, 50, 255]));
        img.put_pixel(36, 24, image::Rgba([0, 0, 0, 0]));

        let normalized = normalize_skin_image(img);
        let back_pixel = *normalized.get_pixel(36, 24);
        assert_eq!(back_pixel[3], 255);
        assert_eq!(back_pixel[0], 200);
        assert_eq!(back_pixel[1], 100);
        assert_eq!(back_pixel[2], 50);
    }

    #[test]
    fn test_normalize_skin_fixes_arm_back_rendering() {
        let mut img = image::RgbaImage::new(64, 64);
        img.put_pixel(46, 24, image::Rgba([180, 120, 80, 255]));
        img.put_pixel(54, 24, image::Rgba([0, 0, 0, 0]));

        img.put_pixel(38, 56, image::Rgba([170, 110, 75, 255]));
        img.put_pixel(46, 56, image::Rgba([0, 0, 0, 0]));

        let normalized = normalize_skin_image(img);
        let right_back = *normalized.get_pixel(54, 24);
        let left_back = *normalized.get_pixel(46, 56);

        assert_eq!(right_back[3], 255);
        assert_eq!(right_back[0], 180);
        assert_eq!(right_back[1], 120);
        assert_eq!(right_back[2], 80);

        assert_eq!(left_back[3], 255);
        assert_eq!(left_back[0], 170);
        assert_eq!(left_back[1], 110);
        assert_eq!(left_back[2], 75);
    }

    #[test]
    fn test_normalize_skin_fixes_placeholder_black_arm_back() {
        let mut img = image::RgbaImage::new(64, 64);
        img.put_pixel(45, 24, image::Rgba([190, 130, 90, 255]));
        img.put_pixel(53, 24, image::Rgba([0, 0, 0, 255]));

        img.put_pixel(37, 56, image::Rgba([195, 135, 95, 255]));
        img.put_pixel(45, 56, image::Rgba([45, 45, 45, 255]));

        let normalized = normalize_skin_image(img);
        let right_back = *normalized.get_pixel(53, 24);
        let left_back = *normalized.get_pixel(45, 56);

        assert_eq!(right_back[3], 255);
        assert_eq!(right_back[0], 0);
        assert_eq!(right_back[1], 0);
        assert_eq!(right_back[2], 0);

        assert_eq!(left_back[3], 255);
        assert_eq!(left_back[0], 45);
        assert_eq!(left_back[1], 45);
        assert_eq!(left_back[2], 45);
    }

    #[test]
    fn test_normalize_skin_alex_slim_arm_back() {
        let mut img = image::RgbaImage::new(64, 64);
        img.put_pixel(44, 24, image::Rgba([200, 140, 100, 255]));
        img.put_pixel(51, 24, image::Rgba([0, 0, 0, 0]));

        img.put_pixel(36, 56, image::Rgba([205, 145, 105, 255]));
        img.put_pixel(43, 56, image::Rgba([0, 0, 0, 0]));

        let normalized = normalize_skin_image(img);
        let right_slim_back = *normalized.get_pixel(51, 24);
        let left_slim_back = *normalized.get_pixel(43, 56);

        assert_eq!(right_slim_back[3], 255);
        assert_eq!(right_slim_back[0], 200);
        assert_eq!(left_slim_back[3], 255);
        assert_eq!(left_slim_back[0], 205);
    }

    #[test]
    fn test_normalize_skin_fixes_underside_hand() {
        let mut img = image::RgbaImage::new(64, 64);
        // Set right arm top to known color
        img.put_pixel(45, 18, image::Rgba([190, 130, 90, 255]));
        // Set right arm bottom (underside of hand) to transparent
        img.put_pixel(49, 18, image::Rgba([0, 0, 0, 0]));

        // Set left arm top to known color
        img.put_pixel(37, 50, image::Rgba([195, 135, 95, 255]));
        // Set left arm bottom (underside of hand) to transparent
        img.put_pixel(41, 50, image::Rgba([0, 0, 0, 0]));

        let normalized = normalize_skin_image(img);
        let right_underside = *normalized.get_pixel(49, 18);
        let left_underside = *normalized.get_pixel(41, 50);

        assert_eq!(right_underside[3], 255, "Right hand underside must be opaque");
        assert_eq!(right_underside[0], 190, "Right hand underside should match top of hand");
        assert_eq!(left_underside[3], 255, "Left hand underside must be opaque");
        assert_eq!(left_underside[0], 195, "Left hand underside should match top of hand");
    }

    fn make_64x32_skin(arm_back_color: image::Rgba<u8>) -> image::RgbaImage {
        let mut img = image::RgbaImage::new(64, 32);
        for y in 0..32 {
            for x in 0..64 {
                img.put_pixel(x, y, image::Rgba([210, 165, 130, 255]));
            }
        }
        for dy in 0..12 {
            for dx in 0..4 {
                img.put_pixel(52 + dx, 20 + dy, arm_back_color);
            }
        }
        img
    }

    #[test]
    fn left_arm_back_copies_from_right_arm_back_64x32() {
        let color = image::Rgba([200, 100, 50, 255]);
        let img = make_64x32_skin(color);
        let result = normalize_skin_image(img);
        assert_eq!(result.dimensions(), (64, 64));
        let p = *result.get_pixel(36 + (3 - 0), 20 + 0);
        assert_eq!(p[3], 255, "left arm back pixel must be opaque");
        assert_ne!(
            (p[0], p[1], p[2]),
            (0, 0, 0),
            "left arm back must not be black"
        );
    }

    #[test]
    fn transparent_back_64x32_uses_front_as_fallback() {
        let mut img = image::RgbaImage::new(64, 32);
        for y in 0..32 {
            for x in 0..64 {
                img.put_pixel(x, y, image::Rgba([210, 165, 130, 255]));
            }
        }
        for dy in 0..12 {
            for dx in 0..4 {
                img.put_pixel(52 + dx, 20 + dy, image::Rgba([0, 0, 0, 0]));
                img.put_pixel(44 + dx, 20 + dy, image::Rgba([180, 120, 80, 255]));
            }
        }
        let result = normalize_skin_image(img);
        let p = *result.get_pixel(36 + (3 - 0), 20 + 0);
        assert_eq!(p[3], 255, "fallback pixel must be opaque");
        assert_ne!((p[0], p[1], p[2]), (0, 0, 0), "fallback must not be black");
    }
}

#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GameLogEntry {
    pub stream: String,
    pub message: String,
}

#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GameExitEvent {
    pub version_id: String,
    pub code: i32,
    pub success: bool,
    #[serde(default)]
    pub error_message: Option<String>,
}

fn architecture_matches(required: &str, current: &str) -> bool {
    match required {
        "x86" | "i386" | "i686" => current == "x86",
        "x86_64" | "x64" | "amd64" => current == "x86_64",
        "arm64" | "aarch64" => current == "aarch64",
        "arm" | "arm32" => current == "arm",
        _ => false,
    }
}

fn native_classifier_matches_arch(classifier: &str, architecture: &str) -> bool {
    if !classifier.starts_with("natives-") {
        return true;
    }
    let suffix = classifier.rsplit('-').next().unwrap_or_default();
    match suffix {
        "arm64" | "aarch64" | "x86_64" | "x64" | "x86" | "arm32" => architecture_matches(suffix, architecture),
        "32" => architecture == "x86",
        "64" => architecture == "x86_64",
        _ => true,
    }
}

pub fn is_native_classifier_allowed(classifier: &str) -> bool {
    native_classifier_matches_arch(classifier, std::env::consts::ARCH)
}

pub fn is_library_allowed(lib: &minecraft::Library) -> bool {
    if let Some(classifier) = lib.name.split(':').nth(3) {
        if !is_native_classifier_allowed(classifier) {
            return false;
        }
    }
    if let Some(ref rules) = lib.rules {
        let platform = platform_mojang_name(current_platform());
        let mut allowed = false;
        for rule in rules {
            if let Some(ref os) = rule.os {
                if os.name != platform {
                    continue;
                }
                if let Some(architecture) = &os.arch {
                    if !architecture_matches(architecture, std::env::consts::ARCH) {
                        continue;
                    }
                }
            }
            match rule.action.as_str() {
                "allow" => allowed = true,
                "deny" | "disallow" => allowed = false,
                _ => {}
            }
        }
        return allowed;
    }
    true
}

pub fn lib_path_from_name(base: &PathBuf, name: &str) -> PathBuf {
    let parts: Vec<&str> = name.split(':').collect();
    if parts.len() < 3 {
        return base.join(name.replace(':', "/"));
    }
    let group = parts[0].replace('.', "/");
    let artifact = parts[1];
    let version = parts[2];
    let classifier = if parts.len() > 3 { Some(parts[3]) } else { None };
    let (ver_clean, filename) = crate::core::minecraft::maven_lib_path_and_filename(artifact, version, classifier);
    base.join(format!("{}/{}/{}/{}", group, artifact, ver_clean, filename))
}

fn extract_mc_version(version: &str) -> &str {
    let clean = version.trim();
    for part in clean.split(|c: char| c == '-' || c == '/' || c == '_') {
        if part.starts_with("1.") && part.len() >= 3 {
            return part;
        }
    }
    if clean.starts_with("20.4") || clean.starts_with("neoforge-20.4") {
        return "1.20.4";
    }
    if clean.starts_with("20.6") || clean.starts_with("neoforge-20.6") {
        return "1.20.6";
    }
    if clean.starts_with("21.1") || clean.starts_with("neoforge-21.1") {
        return "1.21.1";
    }
    if clean.starts_with("21.3") || clean.starts_with("neoforge-21.3") {
        return "1.21.3";
    }
    if clean.starts_with("21.4") || clean.starts_with("neoforge-21.4") {
        return "1.21.4";
    }
    clean
}

fn is_legacy_pack(version: &str) -> bool {
    let clean = extract_mc_version(version);
    clean.starts_with("1.12")
        || clean.starts_with("1.11")
        || clean.starts_with("1.10")
        || clean.starts_with("1.9")
        || clean.starts_with("1.8")
        || clean.starts_with("1.7")
        || clean.starts_with("1.6")
        || clean.starts_with("b1.")
        || clean.starts_with("a1.")
        || clean.starts_with("c0.")
        || clean.starts_with("rd-")
}

fn skin_pack_metadata(format: u32) -> serde_json::Value {
    if format >= 65 {
        serde_json::json!({"pack": {"min_format": format, "max_format": format, "description": "Luxmc Player Custom Skin & Cape"}})
    } else {
        serde_json::json!({"pack": {"pack_format": format, "description": "Luxmc Player Custom Skin & Cape"}})
    }
}

fn get_pack_format_for_version(version: &str) -> u32 {
    let clean = extract_mc_version(version);
    if let Some(dirs) = directories::ProjectDirs::from("io", "github", "Luxmc") {
        let jar = dirs.data_dir().join("versions").join(&clean).join(format!("{clean}.jar"));
        if let Ok(file) = std::fs::File::open(jar) {
            if let Ok(mut archive) = zip::ZipArchive::new(file) {
                if let Ok(entry) = archive.by_name("version.json") {
                    if entry.size() < 65536 {
                        if let Ok(value) = serde_json::from_reader::<_, serde_json::Value>(entry) {
                            if let Some(format) = value.pointer("/pack_version/resource_major").or_else(|| value.pointer("/pack_version/resource/major")).or_else(|| value.pointer("/pack_version/resource")).and_then(|value| value.as_u64()).and_then(|value| u32::try_from(value).ok()) { return format; }
                        }
                    }
                }
            }
        }
    }
    if clean.starts_with("1.21.4") {
        46
    } else if clean.starts_with("1.21.2") || clean.starts_with("1.21.3") {
        42
    } else if clean.starts_with("1.21") {
        34
    } else if clean.starts_with("1.20.5") || clean.starts_with("1.20.6") {
        32
    } else if clean.starts_with("1.20.3") || clean.starts_with("1.20.4") {
        22
    } else if clean.starts_with("1.20.2") {
        18
    } else if clean.starts_with("1.20") {
        15
    } else if clean.starts_with("1.19.4") {
        13
    } else if clean.starts_with("1.19.3") {
        12
    } else if clean.starts_with("1.19") {
        9
    } else if clean.starts_with("1.18") {
        8
    } else if clean.starts_with("1.17") {
        7
    } else if clean.starts_with("1.16") {
        6
    } else if clean.starts_with("1.15") {
        5
    } else if clean.starts_with("1.13") || clean.starts_with("1.14") {
        4
    } else if clean.starts_with("1.11") || clean.starts_with("1.12") {
        3
    } else if clean.starts_with("1.9") || clean.starts_with("1.10") {
        2
    } else {
        1
    }
}

pub async fn clean_skin_injection(game_dir: &std::path::Path) -> AppResult<()> {
    let pack_dir = game_dir.join("resourcepacks").join("LuxmcCustomSkin");
    if pack_dir.exists() {
        let _ = tokio::fs::remove_dir_all(&pack_dir).await;
    }

    let options_file = game_dir.join("options.txt");
    if options_file.exists() {
        if let Ok(content) = tokio::fs::read_to_string(&options_file).await {
            let mut lines: Vec<String> = content.lines().map(|s| s.to_string()).collect();
            let mut changed = false;
            for line in &mut lines {
                if line.starts_with("resourcePacks:") {
                    let inner = line.trim_start_matches("resourcePacks:").trim();
                    if inner.starts_with('[') && inner.ends_with(']') {
                        let array_content = &inner[1..inner.len() - 1];
                        let entries: Vec<String> = array_content
                            .split(',')
                            .map(|s| s.trim().to_string())
                            .filter(|s| {
                                !s.is_empty()
                                    && s != "\"LuxmcCustomSkin\""
                                    && s != "\"file/LuxmcCustomSkin\""
                            })
                            .collect();
                        *line = format!("resourcePacks:[{}]", entries.join(","));
                        changed = true;
                    }
                } else if line.starts_with("incompatibleResourcePacks:") {
                    let cleaned = line
                        .replace(",\"file/LuxmcCustomSkin\"", "")
                        .replace("\"file/LuxmcCustomSkin\",", "")
                        .replace("\"file/LuxmcCustomSkin\"", "")
                        .replace(",\"LuxmcCustomSkin\"", "")
                        .replace("\"LuxmcCustomSkin\",", "")
                        .replace("\"LuxmcCustomSkin\"", "");
                    if &cleaned != line {
                        *line = cleaned;
                        changed = true;
                    }
                }
            }
            if changed {
                let _ = tokio::fs::write(&options_file, lines.join("\n")).await;
            }
        }
    }
    Ok(())
}

pub(crate) fn is_slim_skin(img: &image::RgbaImage) -> bool {
    let (w, h) = img.dimensions();
    if w != h {
        return false;
    }
    let scale = (w / 64).max(1);
    let mut arm_opaque_pixels = 0;
    for y in (20 * scale)..(32 * scale) {
        for x in (44 * scale)..(50 * scale) {
            if img.get_pixel(x, y)[3] >= 128 {
                arm_opaque_pixels += 1;
            }
        }
    }
    if arm_opaque_pixels < 10 {
        return false;
    }

    let mut transparent_padding_count = 0;
    let mut total_padding_samples = 0;
    for y in (20 * scale)..(32 * scale) {
        for x in (54 * scale)..(56 * scale) {
            total_padding_samples += 1;
            if img.get_pixel(x, y)[3] < 50 {
                transparent_padding_count += 1;
            }
        }
    }
    total_padding_samples > 0 && transparent_padding_count >= (total_padding_samples * 8) / 10
}

pub(crate) fn convert_slim_to_wide(img: &image::RgbaImage) -> image::RgbaImage {
    let (w, h) = img.dimensions();
    if w != 64 || h != 64 {
        return img.clone();
    }
    let mut target = img.clone();
    // In Alex (slim):
    // Right Arm base (40..56, 16..32):
    for y in 16..20 {
        let t0 = *img.get_pixel(44, y);
        let t1 = *img.get_pixel(45, y);
        let t2 = *img.get_pixel(46, y);
        target.put_pixel(44, y, t0);
        target.put_pixel(45, y, t1);
        target.put_pixel(46, y, t1);
        target.put_pixel(47, y, t2);

        let b0 = *img.get_pixel(47, y);
        let b1 = *img.get_pixel(48, y);
        let b2 = *img.get_pixel(49, y);
        target.put_pixel(48, y, b0);
        target.put_pixel(49, y, b1);
        target.put_pixel(50, y, b1);
        target.put_pixel(51, y, b2);
    }
    for y in 20..32 {
        let f0 = *img.get_pixel(44, y);
        let f1 = *img.get_pixel(45, y);
        let f2 = *img.get_pixel(46, y);
        target.put_pixel(44, y, f0);
        target.put_pixel(45, y, f1);
        target.put_pixel(46, y, f1);
        target.put_pixel(47, y, f2);

        let i0 = *img.get_pixel(47, y);
        let i1 = *img.get_pixel(48, y);
        let i2 = *img.get_pixel(49, y);
        let i3 = *img.get_pixel(50, y);
        target.put_pixel(48, y, i0);
        target.put_pixel(49, y, i1);
        target.put_pixel(50, y, i2);
        target.put_pixel(51, y, i3);

        let b0 = *img.get_pixel(51, y);
        let b1 = *img.get_pixel(52, y);
        let b2 = *img.get_pixel(53, y);
        target.put_pixel(52, y, b0);
        target.put_pixel(53, y, b1);
        target.put_pixel(54, y, b1);
        target.put_pixel(55, y, b2);
    }
    // Left Arm base (32..48, 48..64):
    for y in 48..52 {
        let t0 = *img.get_pixel(36, y);
        let t1 = *img.get_pixel(37, y);
        let t2 = *img.get_pixel(38, y);
        target.put_pixel(36, y, t0);
        target.put_pixel(37, y, t1);
        target.put_pixel(38, y, t1);
        target.put_pixel(39, y, t2);

        let b0 = *img.get_pixel(39, y);
        let b1 = *img.get_pixel(40, y);
        let b2 = *img.get_pixel(41, y);
        target.put_pixel(40, y, b0);
        target.put_pixel(41, y, b1);
        target.put_pixel(42, y, b1);
        target.put_pixel(43, y, b2);
    }
    for y in 52..64 {
        let i0 = *img.get_pixel(32, y);
        let i1 = *img.get_pixel(33, y);
        let i2 = *img.get_pixel(34, y);
        let i3 = *img.get_pixel(35, y);
        target.put_pixel(32, y, i0);
        target.put_pixel(33, y, i1);
        target.put_pixel(34, y, i2);
        target.put_pixel(35, y, i3);

        let f0 = *img.get_pixel(36, y);
        let f1 = *img.get_pixel(37, y);
        let f2 = *img.get_pixel(38, y);
        target.put_pixel(36, y, f0);
        target.put_pixel(37, y, f1);
        target.put_pixel(38, y, f1);
        target.put_pixel(39, y, f2);

        let o0 = *img.get_pixel(39, y);
        let o1 = *img.get_pixel(40, y);
        let o2 = *img.get_pixel(41, y);
        let o3 = *img.get_pixel(42, y);
        target.put_pixel(40, y, o0);
        target.put_pixel(41, y, o1);
        target.put_pixel(42, y, o2);
        target.put_pixel(43, y, o3);

        let b0 = *img.get_pixel(43, y);
        let b1 = *img.get_pixel(44, y);
        let b2 = *img.get_pixel(45, y);
        target.put_pixel(44, y, b0);
        target.put_pixel(45, y, b1);
        target.put_pixel(46, y, b1);
        target.put_pixel(47, y, b2);
    }
    target
}

pub(crate) fn normalize_skin_image(img: image::RgbaImage) -> image::RgbaImage {
    let (orig_w, orig_h) = img.dimensions();
    let mut canvas = if orig_w == 64 && orig_h == 32 {
        let mut target = image::RgbaImage::new(64, 64);
        for y in 0..32 {
            for x in 0..64 {
                target.put_pixel(x, y, *img.get_pixel(x, y));
            }
        }
        for dy in 0..4 {
            for dx in 0..4 {
                target.put_pixel(20 + (3 - dx), 48 + dy, *target.get_pixel(4 + dx, 16 + dy));
                target.put_pixel(24 + (3 - dx), 48 + dy, *target.get_pixel(8 + dx, 16 + dy));
            }
        }
        for dy in 0..12 {
            for dx in 0..4 {
                target.put_pixel(20 + (3 - dx), 52 + dy, *target.get_pixel(4 + dx, 20 + dy));
                target.put_pixel(28 + (3 - dx), 52 + dy, *target.get_pixel(12 + dx, 20 + dy));
                target.put_pixel(24 + (3 - dx), 52 + dy, *target.get_pixel(dx, 20 + dy));
                target.put_pixel(16 + (3 - dx), 52 + dy, *target.get_pixel(8 + dx, 20 + dy));
            }
        }
        for dy in 0..4 {
            for dx in 0..4 {
                let right_top = *target.get_pixel(44 + dx, 16 + dy);
                let right_bottom = *target.get_pixel(48 + dx, 16 + dy);
                target.put_pixel(36 + (3 - dx), 48 + dy, right_top);
                target.put_pixel(40 + (3 - dx), 48 + dy, right_bottom);
            }
        }
        for dy in 0..12 {
            for dx in 0..4 {
                target.put_pixel(36 + (3 - dx), 52 + dy, *target.get_pixel(44 + dx, 20 + dy));
                let mut back_val = *target.get_pixel(52 + dx, 20 + dy);
                if back_val[3] < 128 {
                    back_val = *target.get_pixel(44 + dx, 20 + dy);
                }
                target.put_pixel(44 + (3 - dx), 52 + dy, back_val);
                target.put_pixel(40 + (3 - dx), 52 + dy, *target.get_pixel(40 + dx, 20 + dy));
                target.put_pixel(32 + (3 - dx), 52 + dy, *target.get_pixel(48 + dx, 20 + dy));
            }
        }
        target
    } else if orig_w != orig_h {
        image::imageops::resize(&img, 64, 64, image::imageops::FilterType::Nearest)
    } else {
        img
    };

    let (w, h) = canvas.dimensions();
    let scale = (w / 64).max(1);
    let is_slim = is_slim_skin(&canvas);

    let mut fallback_color = image::Rgba([20, 20, 20, 255]);
    for ty in [24 * scale, 22 * scale, 12 * scale] {
        for tx in [24 * scale, 22 * scale, 12 * scale] {
            if tx < w && ty < h {
                let p = *canvas.get_pixel(tx, ty);
                if p[3] >= 200 {
                    fallback_color = p;
                    break;
                }
            }
        }
    }

    let base_rects = [
        (0, 0, 32 * scale, 16 * scale),
        (16 * scale, 16 * scale, 40 * scale, 32 * scale),
        (40 * scale, 16 * scale, 56 * scale, 32 * scale),
        (0, 16 * scale, 16 * scale, 32 * scale),
        (16 * scale, 48 * scale, 32 * scale, 64 * scale),
        (32 * scale, 48 * scale, 48 * scale, 64 * scale),
    ];

    for &(x1, y1, x2, y2) in &base_rects {
        for y in y1..y2.min(h) {
            for x in x1..x2.min(w) {
                let pixel = *canvas.get_pixel(x, y);

                if is_slim {
                    let in_right_arm_padding = x >= 54 * scale && x < 56 * scale && y >= 16 * scale && y < 32 * scale;
                    let in_left_arm_padding = x >= 46 * scale && x < 48 * scale && y >= 48 * scale && y < 64 * scale;
                    if in_right_arm_padding || in_left_arm_padding {
                        continue;
                    }
                }

                if pixel[3] >= 200 {
                    continue;
                }

                let overlay_pos = if x >= 40 * scale && x < 56 * scale && y >= 16 * scale && y < 32 * scale {
                    Some((x, y + 16 * scale))
                } else if x >= 32 * scale && x < 48 * scale && y >= 48 * scale && y < 64 * scale {
                    Some((x + 16 * scale, y))
                } else if x >= 16 * scale && x < 40 * scale && y >= 16 * scale && y < 32 * scale {
                    Some((x, y + 16 * scale))
                } else if x < 32 * scale && y < 16 * scale {
                    Some((x + 32 * scale, y))
                } else if x < 16 * scale && y >= 16 * scale && y < 32 * scale {
                    Some((x, y + 16 * scale))
                } else if x >= 16 * scale && x < 32 * scale && y >= 48 * scale && y < 64 * scale {
                    Some((x - 16 * scale, y))
                } else {
                    None
                };

                let mut filled = false;
                if let Some((ox, oy)) = overlay_pos {
                    if ox < w && oy < h {
                        let op = *canvas.get_pixel(ox, oy);
                        if op[3] >= 128 {
                            canvas.put_pixel(x, y, image::Rgba([op[0], op[1], op[2], 255]));
                            filled = true;
                        }
                    }
                }

                if filled {
                    continue;
                }

                let is_right_arm_back = x >= 51 * scale && x < 56 * scale && y >= 20 * scale && y < 32 * scale;
                let is_left_arm_back = x >= 43 * scale && x < 48 * scale && y >= 52 * scale && y < 64 * scale;
                let is_torso_back = x >= 32 * scale && x < 40 * scale && y >= 20 * scale && y < 32 * scale;
                let is_right_hand_bottom = x >= 47 * scale && x < 52 * scale && y >= 16 * scale && y < 20 * scale;
                let is_left_hand_bottom = x >= 39 * scale && x < 44 * scale && y >= 48 * scale && y < 52 * scale;

                if is_right_hand_bottom {
                    let top_x = x.saturating_sub(4 * scale).max(44 * scale);
                    let top_p = *canvas.get_pixel(top_x, y);
                    if top_p[3] >= 128 {
                        canvas.put_pixel(x, y, image::Rgba([top_p[0], top_p[1], top_p[2], 255]));
                    } else {
                        canvas.put_pixel(x, y, fallback_color);
                    }
                } else if is_left_hand_bottom {
                    let top_x = x.saturating_sub(4 * scale).max(36 * scale);
                    let top_p = *canvas.get_pixel(top_x, y);
                    if top_p[3] >= 128 {
                        canvas.put_pixel(x, y, image::Rgba([top_p[0], top_p[1], top_p[2], 255]));
                    } else {
                        canvas.put_pixel(x, y, fallback_color);
                    }
                } else if is_right_arm_back {
                    let front_x = if x == 51 * scale {
                        44 * scale
                    } else {
                        44 * scale + x.saturating_sub(52 * scale).min(3 * scale)
                    };
                    let fp = *canvas.get_pixel(front_x, y);
                    if fp[3] >= 128 {
                        canvas.put_pixel(x, y, image::Rgba([fp[0], fp[1], fp[2], 255]));
                    } else {
                        canvas.put_pixel(x, y, fallback_color);
                    }
                } else if is_left_arm_back {
                    let front_x = if x == 43 * scale {
                        36 * scale
                    } else {
                        36 * scale + x.saturating_sub(44 * scale).min(3 * scale)
                    };
                    let fp = *canvas.get_pixel(front_x, y);
                    if fp[3] >= 128 {
                        canvas.put_pixel(x, y, image::Rgba([fp[0], fp[1], fp[2], 255]));
                    } else {
                        canvas.put_pixel(x, y, fallback_color);
                    }
                } else if is_torso_back {
                    let front_x = 20 * scale + x.saturating_sub(32 * scale);
                    let fp = *canvas.get_pixel(front_x, y);
                    if fp[3] >= 128 {
                        canvas.put_pixel(x, y, image::Rgba([fp[0], fp[1], fp[2], 255]));
                    } else {
                        canvas.put_pixel(x, y, fallback_color);
                    }
                } else {
                    canvas.put_pixel(x, y, fallback_color);
                }
            }
        }
    }

    canvas
}

async fn inject_player_skin(
    http: &reqwest::Client,
    game_dir: &std::path::Path,
    username: &str,
    skin_source: &str,
    variant: &str,
    cape_source: Option<&str>,
    mc_version: &str,
) -> AppResult<()> {
    let mut skin_bytes = resolve_image_bytes(skin_source, http).await;

    if skin_bytes.is_empty() {
        if let Some(u) = skin_source.split('/').last() {
            let clean_name = u.trim_end_matches(".png");
            if !clean_name.is_empty() {
                let fallback_urls = [
                    format!("https://mc-heads.net/skin/{}", clean_name),
                    format!("https://minotar.net/skin/{}", clean_name),
                    format!("https://crafatar.com/skins/{}", clean_name),
                ];
                for url in &fallback_urls {
                    if let Ok(resp) = http.get(url).send().await {
                        if resp.status().is_success() {
                            if let Ok(b) = resp.bytes().await {
                                if !b.is_empty() {
                                    skin_bytes = b.to_vec();
                                    break;
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    if skin_bytes.is_empty() || skin_bytes.len() < 8 || &skin_bytes[0..8] != b"\x89PNG\r\n\x1a\n" {
        tracing::warn!("No valid PNG skin bytes found for player skin injection. Cleaning up injection pack...");
        return Err(AppError::InvalidInput("A skin selecionada não pôde ser carregada. Selecione um PNG local antes de jogar.".into()));
    }

    let raw_dyn_img = Some(image::load_from_memory(&skin_bytes).map_err(|e| AppError::InvalidInput(format!("PNG da skin inválido: {e}")))?);
    let is_slim = match variant { "slim" => true, "classic" => false, _ => raw_dyn_img.as_ref().map(|d| is_slim_skin(&d.to_rgba8())).unwrap_or(false) };
    let normalized_skin_img = if let Some(ref dyn_img) = raw_dyn_img {
        normalize_skin_image(dyn_img.to_rgba8())
    } else {
        image::RgbaImage::new(64, 64)
    };

    let wide_skin_img = if is_slim {
        convert_slim_to_wide(&normalized_skin_img)
    } else {
        normalized_skin_img.clone()
    };
    let slim_skin_img = normalized_skin_img.clone();

    let mut wide_buf = Vec::new();
    wide_skin_img
        .write_to(&mut std::io::Cursor::new(&mut wide_buf), image::ImageFormat::Png)
        .map_err(|e| crate::error::AppError::Internal(format!("failed to encode wide skin: {e}")))?;
    let mut slim_buf = Vec::new();
    slim_skin_img
        .write_to(&mut std::io::Cursor::new(&mut slim_buf), image::ImageFormat::Png)
        .map_err(|e| crate::error::AppError::Internal(format!("failed to encode slim skin: {e}")))?;

    set_active_cape_bytes(Vec::new()).await;
    let pack_dir = game_dir.join("resourcepacks").join("LuxmcCustomSkin");
    if tokio::fs::try_exists(&pack_dir).await? { tokio::fs::remove_dir_all(&pack_dir).await?; }
    let entity_dir_wide = pack_dir.join("assets/minecraft/textures/entity/player/wide");
    let entity_dir_slim = pack_dir.join("assets/minecraft/textures/entity/player/slim");
    let entity_dir_player = pack_dir.join("assets/minecraft/textures/entity/player");
    let entity_dir_legacy = pack_dir.join("assets/minecraft/textures/entity");

    let _ = tokio::fs::create_dir_all(&entity_dir_wide).await;
    let _ = tokio::fs::create_dir_all(&entity_dir_slim).await;
    let _ = tokio::fs::create_dir_all(&entity_dir_player).await;
    let _ = tokio::fs::create_dir_all(&entity_dir_legacy).await;

    let pack_fmt = get_pack_format_for_version(mc_version);
    let mcmeta = skin_pack_metadata(pack_fmt);
    tokio::fs::write(pack_dir.join("pack.mcmeta"), serde_json::to_string_pretty(&mcmeta).unwrap_or_default()).await?;
    tokio::fs::write(pack_dir.join("pack.png"), include_bytes!("../../../icons/64x64.png")).await?;

    let skin_models = [
        "steve", "alex", "ari", "efe", "kai", "makena", "noor", "sunny", "zuri",
    ];
    for model in &skin_models {
        tokio::fs::write(entity_dir_wide.join(format!("{}.png", model)), &wide_buf).await?;
        tokio::fs::write(entity_dir_slim.join(format!("{}.png", model)), &slim_buf).await?;
        tokio::fs::write(entity_dir_player.join(format!("{}.png", model)), if is_slim { &slim_buf } else { &wide_buf }).await?;
        tokio::fs::write(entity_dir_legacy.join(format!("{}.png", model)), &wide_buf).await?;
    }

    tokio::fs::write(entity_dir_legacy.join("steve.png"), &wide_buf).await?;
    tokio::fs::write(entity_dir_legacy.join("alex.png"), &slim_buf).await?;

    let cape_bytes: Vec<u8> = if let Some(cs) = cape_source.filter(|s| !s.trim().is_empty()) {
        resolve_image_bytes(cs, http).await
    } else {
        Vec::new()
    };

    if cape_source.is_some_and(|source| !source.trim().is_empty()) && (cape_bytes.is_empty() || image::load_from_memory(&cape_bytes).is_err()) {
        return Err(AppError::InvalidInput("A capa selecionada não pôde ser carregada. Selecione um PNG local antes de jogar.".into()));
    }
    if !cape_bytes.is_empty() {
        let cape_source_bytes = cape_bytes;
        let (normalized_cape_bytes, optifine_cape_bytes) = tokio::task::spawn_blocking(
            move || -> AppResult<(Vec<u8>, Vec<u8>)> {
                if let Ok(dyn_cape) = image::load_from_memory(&cape_source_bytes) {
                    use image::GenericImageView;
                    let (cw, ch) = dyn_cape.dimensions();
                    let cape_rgba = dyn_cape.to_rgba8();

                    let (full_img, opti_img) = if (cw == 22 && ch == 17) || (cw == 44 && ch == 34)
                    {
                        let scale = if cw == 22 { 1 } else { 2 };
                        let mut canvas = image::RgbaImage::new(64 * scale, 32 * scale);
                        image::imageops::overlay(&mut canvas, &cape_rgba, 0, 0);
                        let opti = canvas.clone();
                        (canvas, opti)
                    } else if cw == ch * 2 {
                        (cape_rgba.clone(), cape_rgba)
                    } else {
                        format_custom_cape_texture(&cape_rgba, cw, ch)
                    };

                    let mut full_buf = Vec::new();
                    let mut cursor = std::io::Cursor::new(&mut full_buf);
                    let full_bytes = if full_img.write_to(&mut cursor, image::ImageFormat::Png).is_ok()
                    {
                        full_buf
                    } else {
                        cape_source_bytes.clone()
                    };

                    let mut opti_buf = Vec::new();
                    let mut opti_cursor = std::io::Cursor::new(&mut opti_buf);
                    let opti_bytes = if opti_img
                        .write_to(&mut opti_cursor, image::ImageFormat::Png)
                        .is_ok()
                    {
                        opti_buf
                    } else {
                        full_bytes.clone()
                    };

                    Ok((full_bytes, opti_bytes))
                } else {
                    Ok((cape_source_bytes.clone(), cape_source_bytes))
                }
            },
        )
        .await
        .map_err(|error| AppError::Internal(error.to_string()))??;

        set_active_cape_bytes(normalized_cape_bytes.clone()).await;

        let u_clean = username.trim();

        let u_lower = u_clean.to_lowercase();

        let optifine_users_dir = pack_dir.join("assets/minecraft/optifine/users");
        let _ = tokio::fs::create_dir_all(&optifine_users_dir).await;
        let user_prop = format!("cape=optifine/capes/{}.png\n", u_clean);
        tokio::fs::write(optifine_users_dir.join(format!("{}.properties", u_clean)), user_prop.as_bytes()).await?;
        tokio::fs::write(optifine_users_dir.join(format!("{}.properties", u_lower)), user_prop.as_bytes()).await?;

        let optifine_root_prop = pack_dir.join("assets/minecraft/optifine/cape.properties");
        let root_prop = format!("users={}\n", u_clean);
        if let Some(p) = optifine_root_prop.parent() { let _ = tokio::fs::create_dir_all(p).await; }
        tokio::fs::write(&optifine_root_prop, root_prop.as_bytes()).await?;

        let optifine_capes = [
            pack_dir.join("assets/minecraft/optifine/cape.png"),
            pack_dir.join("assets/minecraft/optifine/cape/cape.png"),
            pack_dir.join("assets/minecraft/optifine/capes/default.png"),
            pack_dir.join(format!("assets/minecraft/optifine/capes/{}.png", u_clean)),
            pack_dir.join(format!("assets/minecraft/optifine/capes/{}.png", u_lower)),
        ];
        for path in &optifine_capes {
            if let Some(parent) = path.parent() {
                let _ = tokio::fs::create_dir_all(parent).await;
            }
            tokio::fs::write(path, &optifine_cape_bytes).await?;
        }

        let cape_paths = [
            pack_dir.join("assets/minecraft/textures/entity/cape.png"),
            pack_dir.join("assets/minecraft/textures/entity/player/cape.png"),
            pack_dir.join("assets/minecraft/textures/entity/elytra.png"),
            pack_dir.join("assets/minecraft/textures/entity/player/elytra.png"),
            pack_dir.join("assets/minecraft/textures/cape/cape.png"),
            pack_dir.join("assets/minecraft/textures/capes/cape.png"),
            pack_dir.join(format!("assets/minecraft/textures/capes/{}.png", u_clean)),
            pack_dir.join(format!("assets/minecraft/textures/capes/{}.png", u_lower)),
            pack_dir.join("assets/minecraft/textures/entity/cape/cape.png"),
            pack_dir.join("assets/minecraft/textures/entity/player/wide/cape.png"),
            pack_dir.join("assets/minecraft/textures/entity/player/slim/cape.png"),
            pack_dir.join("assets/minecraft/textures/models/armor/cape.png"),
            pack_dir.join("assets/minecraft/textures/entity/player/wide/elytra.png"),
            pack_dir.join("assets/minecraft/textures/entity/player/slim/elytra.png"),
            pack_dir.join(format!("assets/minecraft/textures/entity/player/wide/{}.png", u_clean)),
            pack_dir.join(format!("assets/minecraft/textures/entity/player/slim/{}.png", u_clean)),
            pack_dir.join(format!("assets/minecraft/textures/entity/player/wide/{}.png", u_lower)),
            pack_dir.join(format!("assets/minecraft/textures/entity/player/slim/{}.png", u_lower)),
            pack_dir.join("assets/entity_model_features/textures/entity/cape.png"),
            pack_dir.join(format!("assets/entity_model_features/textures/entity/{}.png", u_clean)),
            pack_dir.join(format!("assets/entity_model_features/textures/entity/{}.png", u_lower)),
            pack_dir.join(format!("assets/entity_model_features/capes/{}.png", u_clean)),
            pack_dir.join(format!("assets/entity_model_features/capes/{}.png", u_lower)),
            pack_dir.join("assets/entity_texture_features/textures/entity/cape.png"),
            pack_dir.join(format!("assets/entity_texture_features/textures/entity/{}.png", u_clean)),
            pack_dir.join(format!("assets/entity_texture_features/textures/entity/{}.png", u_lower)),
            pack_dir.join(format!("assets/entity_texture_features/capes/{}.png", u_clean)),
            pack_dir.join(format!("assets/entity_texture_features/capes/{}.png", u_lower)),
            pack_dir.join(format!("assets/entity_texture_features/textures/capes/{}.png", u_clean)),
            pack_dir.join(format!("assets/entity_texture_features/textures/capes/{}.png", u_lower)),
            pack_dir.join("assets/minecraft/textures/player/cape.png"),
            pack_dir.join(format!("assets/minecraft/textures/player/{}.png", u_clean)),
            pack_dir.join(format!("assets/minecraft/textures/player/{}.png", u_lower)),
        ];

        for path in &cape_paths {
            if let Some(parent) = path.parent() {
                let _ = tokio::fs::create_dir_all(parent).await;
            }
            tokio::fs::write(path, &normalized_cape_bytes).await?;
        }
    }

    let is_legacy = is_legacy_pack(mc_version);
    let chosen_pack_entry = if is_legacy {
        "\"LuxmcCustomSkin\""
    } else {
        "\"file/LuxmcCustomSkin\""
    };
    let unchosen_pack_entry = if is_legacy {
        "\"file/LuxmcCustomSkin\""
    } else {
        "\"LuxmcCustomSkin\""
    };

    let skin_flags = [
        ("modelPart_cape", "true"),
        ("modelPart_jacket", "true"),
        ("modelPart_left_sleeve", "true"),
        ("modelPart_right_sleeve", "true"),
        ("modelPart_left_pants_leg", "true"),
        ("modelPart_right_pants_leg", "true"),
        ("modelPart_hat", "true"),
    ];

    let options_file = game_dir.join("options.txt");
    let pack_defaults = game_dir.join("config/yosbr/options.txt");
    if !options_file.exists() && pack_defaults.is_file() {
        tokio::fs::copy(&pack_defaults, &options_file).await?;
    }
    if options_file.exists() {
        if let Ok(content) = tokio::fs::read_to_string(&options_file).await {
            let mut lines: Vec<String> = content.lines().map(|s| s.to_string()).collect();
            let mut found_pack = false;
            for (key, val) in &skin_flags {
                let prefix = format!("{}:", key);
                let mut found_key = false;
                for line in &mut lines {
                    if line.starts_with(&prefix) {
                        let old_val = line.split(':').nth(1).unwrap_or("");
                        if old_val != *val {
                            tracing::debug!("Forcing {} from {} to {}", key, old_val, val);
                        }
                        *line = format!("{}:{}", key, val);
                        found_key = true;
                        break;
                    }
                }
                if !found_key {
                    tracing::debug!("Adding missing {} setting", key);
                    lines.push(format!("{}:{}", key, val));
                }
            }
            for line in &mut lines {
                if line.starts_with("resourcePacks:") {
                    found_pack = true;
                    let inner = line.trim_start_matches("resourcePacks:").trim();
                    if inner.starts_with('[') && inner.ends_with(']') {
                        let array_content = &inner[1..inner.len() - 1];
                        let entries: Vec<String> = array_content
                            .split(',')
                            .map(|s| s.trim().to_string())
                            .filter(|s| !s.is_empty() && s != unchosen_pack_entry && s != chosen_pack_entry)
                            .collect();
                        *line = format!("resourcePacks:[{}]", entries.join(","));
                    }
                } else if line.starts_with("incompatibleResourcePacks:") {
                    *line = line.replace(&format!(",{}", chosen_pack_entry), "")
                        .replace(&format!("{},", chosen_pack_entry), "")
                        .replace(chosen_pack_entry, "")
                        .replace(&format!(",{}", unchosen_pack_entry), "")
                        .replace(&format!("{},", unchosen_pack_entry), "")
                        .replace(unchosen_pack_entry, "");
                }
            }
            if !found_pack {
                let default_list = if is_legacy {
                    "resourcePacks:[]".to_string()
                } else {
                    "resourcePacks:[\"vanilla\"]".to_string()
                };
                lines.push(default_list);
            }
            tokio::fs::write(&options_file, lines.join("\n")).await?;
        }
    } else {
        let mut default_options = if is_legacy {
            "resourcePacks:[]\nincompatibleResourcePacks:[]\n".to_string()
        } else {
            "resourcePacks:[\"vanilla\"]\nincompatibleResourcePacks:[]\n".to_string()
        };
        for (key, val) in &skin_flags {
            default_options.push_str(&format!("{}:{}\n", key, val));
        }
        tokio::fs::write(&options_file, default_options).await?;
    }

    Ok(())
}

fn format_custom_cape_texture(cape_rgba: &image::RgbaImage, cw: u32, ch: u32) -> (image::RgbaImage, image::RgbaImage) {
    let mut min_x = cw;
    let mut min_y = ch;
    let mut max_x = 0;
    let mut max_y = 0;
    for y in 0..ch {
        for x in 0..cw {
            if cape_rgba.get_pixel(x, y)[3] > 10 {
                if x < min_x { min_x = x; }
                if x > max_x { max_x = x; }
                if y < min_y { min_y = y; }
                if y > max_y { max_y = y; }
            }
        }
    }

    let cropped = if min_x <= max_x && min_y <= max_y {
        let bw = (max_x - min_x + 1).max(1);
        let bh = (max_y - min_y + 1).max(1);
        image::imageops::crop_imm(cape_rgba, min_x, min_y, bw, bh).to_image()
    } else {
        cape_rgba.clone()
    };

    let target_w = if cw >= 128 || ch >= 128 { 128 } else { 64 };
    let target_h = target_w / 2;
    let s = target_w / 64;
    let mut canvas = image::RgbaImage::new(target_w, target_h);

    let art_w = 10 * s;
    let art_h = 16 * s;

    let cw_f = cropped.width() as f32;
    let ch_f = cropped.height() as f32;
    let scale = (art_w as f32 / cw_f).min(art_h as f32 / ch_f);
    let draw_w = ((cw_f * scale).round() as u32).max(1).min(art_w);
    let draw_h = ((ch_f * scale).round() as u32).max(1).min(art_h);
    let offset_x = (art_w - draw_w) / 2;
    let offset_y = (art_h - draw_h) / 2;

    let resized = image::imageops::resize(&cropped, draw_w, draw_h, image::imageops::FilterType::Nearest);

    let bg_pixel = if cropped.width() > 0 && cropped.height() > 0 {
        *cropped.get_pixel(0, 0)
    } else {
        image::Rgba([0, 0, 0, 0])
    };

    if bg_pixel[3] > 20 {
        for y in 0..art_h {
            for x in 0..art_w {
                canvas.put_pixel((1 * s) + x, (1 * s) + y, bg_pixel);
                canvas.put_pixel((12 * s) + x, (1 * s) + y, bg_pixel);
            }
        }
        for y in 0..art_h {
            for x in 0..s {
                canvas.put_pixel(x, (1 * s) + y, bg_pixel);
                canvas.put_pixel((11 * s) + x, (1 * s) + y, bg_pixel);
            }
        }
        for x in 0..art_w {
            for y in 0..s {
                canvas.put_pixel((1 * s) + x, y, bg_pixel);
                canvas.put_pixel((11 * s) + x, y, bg_pixel);
            }
        }
    }

    image::imageops::overlay(&mut canvas, &resized, ((1 * s) + offset_x) as i64, ((1 * s) + offset_y) as i64);
    image::imageops::overlay(&mut canvas, &resized, ((12 * s) + offset_x) as i64, ((1 * s) + offset_y) as i64);

    let opti = canvas.clone();
    (canvas, opti)
}
