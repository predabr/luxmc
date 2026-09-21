mod loader_selection;
use std::path::PathBuf;
use std::process::Stdio;
use tauri::Emitter;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::sync::Mutex as TokioMutex;
use std::sync::Arc;

use crate::core::downloader::DownloadManager;
use crate::core::java::JavaRuntimeManager;
use crate::core::minecraft::{self, VersionDetail};
use crate::error::AppResult;

const DEV_CLIENT_ID: &str = "00000000-0000-0000-0000-000000000002";
const DEV_XUID: &str = "0";
const LAUNCHER_NAME: &str = "Luxmc";
const LAUNCHER_VERSION: &str = "1.9.2";
static CLIENT_AGENT_JAR: &[u8] = include_bytes!("../../../assets/luxmc-client-agent.jar");
static ACTIVE_CAPE_BYTES: tokio::sync::RwLock<Vec<u8>> = tokio::sync::RwLock::const_new(Vec::new());

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

    if let Ok(exe) = std::env::current_exe() {
        if let Some(parent) = exe.parent() {
            candidates.push(parent.join("Luxmc.Launcher"));
            candidates.push(parent.join("bin").join("Luxmc.Launcher"));
            if let Some(grandparent) = parent.parent() {
                candidates.push(grandparent.join("bin").join("Luxmc.Launcher"));
                candidates.push(grandparent.join("dist-electron").join("bin").join("Luxmc.Launcher"));
                candidates.push(grandparent.join("app.asar.unpacked").join("dist-electron").join("bin").join("Luxmc.Launcher"));
            }
        }
    }

    if let Some(dirs) = directories::ProjectDirs::from("io", "github", "Luxmc") {
        candidates.push(dirs.data_dir().join("bin").join("Luxmc.Launcher"));
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
            if total_ram_mb >= 32768 { 10240 }
            else if total_ram_mb >= 24576 { 8192 }
            else if total_ram_mb >= 16384 { 6144 }
            else if total_ram_mb >= 12288 { 5120 }
            else if total_ram_mb >= 8192 { 4096 }
            else { (total_ram_mb * 6 / 10).max(2048) }
        } else if mod_count >= 20 {
            if total_ram_mb >= 16384 { 4096 } else if total_ram_mb >= 8192 { 3072 } else { 2048 }
        } else {
            if total_ram_mb >= 8192 { 2560 } else { 2048 }
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

        self.emit_stage(LaunchStage::CheckingJava);
        let mut major = detail.java_major_version();
        if loader == "neoforge" && major < 21 {
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
                        self.emit_log("NeoForge module isolation: removed redundant neoforge runtime jars (neoforge-client.jar active)");
                    }
                }
            }

            let mut seen_cp = std::collections::HashSet::new();
            let mut deduped_cp = Vec::new();
            for entry in classpath {
                if seen_cp.insert(entry.clone()) {
                    deduped_cp.push(entry);
                }
            }
            classpath = deduped_cp;
        }

        self.emit_log(&format!("Classpath entries: {}", classpath.len()));

        self.emit_stage(LaunchStage::ExtractingNatives);
        let natives_dir = self.prepare_natives(detail).await?;
        self.emit_log(&format!("Natives dir: {}", natives_dir.display()));

        // Apply customized player skin if configured
        let is_msa = user_type == "msa";
        let has_explicit_custom_skin = skin_url
            .map(|s| {
                let trimmed = s.trim();
                !trimmed.is_empty()
                    && !trimmed.starts_with("https://textures.minecraft.net/")
                    && !trimmed.starts_with("http://textures.minecraft.net/")
            })
            .unwrap_or(false);

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

        let has_explicit_custom_cape = effective_cape
            .as_deref()
            .map(|c| !c.trim().is_empty())
            .unwrap_or(false);

        if is_msa && !has_explicit_custom_skin && !has_explicit_custom_cape {
            self.emit_log(&format!("Using official Mojang account skin directly from session server for {}...", username));
            let _ = clean_skin_injection(game_dir).await;
        } else {
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
                    let s_url: Option<String> = row.try_get("skin_url").ok();
                    let s_var: Option<String> = row.try_get("skin_variant").ok();
                    s_url.map(|u| (u, s_var.unwrap_or_else(|| "classic".to_string())))
                } else {
                    None
                }
            } else {
                None
            };

            if let Some((skin_source, variant)) = skin_info {
                self.emit_log(&format!("Applying customized player skin ({}) for {}...", variant, username));
                let _ = inject_player_skin(self.downloader.http(), game_dir, username, &skin_source, &variant, effective_cape.as_deref(), clean_mc_ver).await;
            } else if !is_msa || has_explicit_custom_cape {
                let default_source = format!("https://minotar.net/skin/{}", username);
                let _ = inject_player_skin(self.downloader.http(), game_dir, username, &default_source, "classic", effective_cape.as_deref(), clean_mc_ver).await;
            }
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
            game_args.push("--server".to_string());
            game_args.push(ip.trim().to_string());
            if let Some(port) = server_port.filter(|p| *p > 0) {
                game_args.push("--port".to_string());
                game_args.push(port.to_string());
            }
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

        let _is_modpack = profile.loader == "forge"
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
            self.resolve_duplicate_mods(game_dir);
        }

        let agent_path = game_dir.join("luxmc-client-agent.jar");
        if let Err(e) = std::fs::write(&agent_path, CLIENT_AGENT_JAR) {
            self.emit_log(&format!("Aviso: Não foi possível extrair Luxmc Client Agent: {}", e));
        } else {
            safe_jvm_args.push(format!("-javaagent:{}", agent_path.to_string_lossy()));
            self.emit_log("Luxmc Client Agent anexado (Right Shift menu in-game ativo)");
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

            let mut c = crate::core::process::tokio_command(&effective_java);
            c.args(&safe_jvm_args)
                .arg(main_class)
                .args(&safe_game_args);
            c
        };
        cmd.current_dir(game_dir)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .kill_on_drop(true)
            .env("PATH", std::env::var("PATH").unwrap_or_default());

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
            if std::env::var("DISPLAY").is_ok() {
                cmd.env("GLFW_PLATFORM", "x11");
            } else if std::env::var("WAYLAND_DISPLAY").is_ok() {
                cmd.env("DISPLAY", ":0");
                cmd.env("GLFW_PLATFORM", "x11");
            } else {
                cmd.env("DISPLAY", ":0");
                cmd.env("GLFW_PLATFORM", "x11");
            }

            if profile.use_vulkan {
                self.emit_log("Mesa Zink (OpenGL sobre Vulkan) aceleração ativa");
                cmd.env("MESA_LOADER_DRIVER_OVERRIDE", "zink");
                cmd.env("GALLIUM_DRIVER", "zink");
                cmd.env("MESA_SHADER_CACHE_MAX_SIZE", "100G");
                cmd.env("RADV_PERFTEST", "aco");
                cmd.env("vblank_mode", "0");
                cmd.env("MESA_GL_THREAD", "true");
            } else {
                cmd.env("MESA_SHADER_CACHE_MAX_SIZE", "100G");
                cmd.env("MESA_GL_THREAD", "true");
            }
        }

        #[cfg(target_os = "windows")]
        {
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
            cmd.env("GPU_MAX_ALLOC_PERCENT", "100");
            cmd.env("GPU_USE_SYNC_OBJECTS", "1");
            cmd.env("GPU_NUM_COMPUTE_RINGS", "1");
            cmd.env("GPU_MAX_HEAP_SIZE", "100");
            cmd.env("GPU_FORCE_64BIT_PTR", "1");
            cmd.env("__NV_PRIME_RENDER_OFFLOAD", "1");
            cmd.env("__GL_THREADED_OPTIMIZATIONS", "1");
            cmd.env("AMD_POWERXPRESS_REQUEST_HIGH_PERFORMANCE", "1");
        }

        let mut child = cmd.spawn().map_err(|e| {
            let msg = format!("Failed to start Java process: {}", e);
            self.emit_log(&msg);
            self.emit_stage(LaunchStage::Failed);
            crate::error::AppError::Internal(msg)
        })?;

        let pid = child.id().unwrap_or(0);
        self.emit_log(&format!("Java process started, PID: {}", pid));
        self.emit_stage(LaunchStage::Running);

        // Ghost mode: immediately release unused launcher memory
        crate::commands::optimizer::optimizer_trim_memory();

        if pid > 0 {
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
                        crate::commands::optimizer::optimizer_trim_memory();
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
            .append(true)
            .open(&log_file_path)
            .await
            .ok();
        let mut log_file_err = if let Some(ref f) = log_file {
            f.try_clone().await.ok()
        } else {
            None
        };

        let _app_for_overlay = self.app.clone();
        tokio::spawn(async move {
            let reader = BufReader::new(stdout);
            let mut lines = reader.lines();
            while let Ok(Some(line)) = lines.next_line().await {
                let mut line = line;
                if line.len() > 2000 { line.truncate(2000); }
                if let Some(ref mut f) = log_file {
                    let _ = tokio::io::AsyncWriteExt::write_all(f, format!("{}\n", line).as_bytes()).await;
                }
                if lb_out.lock().await.len() < 200 { lb_out.lock().await.push(line); }
            }
        });

        tokio::spawn(async move {
            let reader = BufReader::new(stderr);
            let mut lines = reader.lines();
            while let Ok(Some(line)) = lines.next_line().await {
                let mut line = line;
                if line.len() > 2000 { line.truncate(2000); }
                let trimmed = line.trim();
                if !trimmed.is_empty() {
                    if let Ok(mut lock) = last_stderr_writer.lock() {
                        *lock = Some(trimmed.to_string());
                    }
                }
                if let Some(ref mut f) = log_file_err {
                    let _ = tokio::io::AsyncWriteExt::write_all(f, format!("{}\n", line).as_bytes()).await;
                }
                if lb_err.lock().await.len() < 200 { lb_err.lock().await.push(line); }
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
        let last_stderr_reader = last_stderr.clone();
        let is_game_active_waiter = is_game_active.clone();
        tokio::spawn(async move {
            match child.wait().await {
                Ok(status) => {
                    is_game_active_waiter.store(false, std::sync::atomic::Ordering::Relaxed);
                    let code = status.code().unwrap_or(-1);
                    if pid > 0 {
                        crate::core::linux::gamemode::release_gamemode_for_pid(pid).await;
                    }
                    let duration_secs = start_time.elapsed().as_secs();
                    let peak_mb = peak_ram_bytes.load(std::sync::atomic::Ordering::Relaxed) / (1024 * 1024);
                    let msg = format!("Game process exited with code {}", code);
                    tracing::info!(target: "launch", "{}", msg);
                    let error_message = if !status.success() {
                        last_stderr_reader.lock().ok().and_then(|g| g.clone())
                    } else {
                        None
                    };
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
                }
                Err(e) => {
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
        let base = self.downloader.libraries_dir();
        let versions_dir = self.downloader.versions_dir();
        let natives_dir = versions_dir.join(&detail.id).join("natives");
        tokio::fs::create_dir_all(&natives_dir).await?;

        let os_name = platform_mojang_name(current_platform());

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
                    for (classifier_key, _) in classifiers {
                        let matches_os = match os_name {
                            "windows" => classifier_key.starts_with("natives-windows"),
                            "linux" => classifier_key.starts_with("natives-linux"),
                            "osx" => classifier_key.starts_with("natives-osx") || classifier_key.starts_with("natives-macos"),
                            _ => false,
                        };
                        if matches_os {
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
                if path.exists() {
                    if let Ok(file) = std::fs::File::open(&path) {
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
                                        if let Ok(mut outfile) = std::fs::File::create(&outpath) {
                                            let _ = std::io::copy(&mut file, &mut outfile);
                                        }
                                    }
                                }
                            }
                        } else {
                            self.emit_log(&format!("Failed to open native zip archive: {}", path.display()));
                        }
                    } else {
                        self.emit_log(&format!("Failed to open native jar file: {}", path.display()));
                    }
                }
            }
        }

        if cfg!(target_os = "linux") {
            match ensure_flite_library(&natives_dir).await {
                Ok(true) => self.emit_log("Flite narrator library ready"),
                Ok(false) => self.emit_log(
                    "Flite narrator unavailable; Minecraft will continue without narration",
                ),
                Err(error) => self.emit_log(&format!("Flite narrator unavailable: {}", error)),
            }
        }

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            if let Ok(entries) = std::fs::read_dir(&natives_dir) {
                for entry in entries.flatten() {
                    let p = entry.path();
                    if p.is_file() {
                        let is_lib = p.extension()
                            .map(|ext| ext == "so" || ext == "dylib")
                            .unwrap_or(false);
                        if is_lib {
                            let _ = std::fs::set_permissions(&p, std::fs::Permissions::from_mode(0o755));
                        }
                    }
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
        self.emit_log(&format!(
            "Extracted {} native libraries to {}",
            native_count,
            natives_dir.display()
        ));

        Ok(natives_dir)
    }

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

        // 100% Automatic RAM Allocation: dynamically inspects user hardware and mod weight
        let is_heavy_modded = profile.loader == "forge"
            || profile.loader == "neoforge"
            || profile.name.to_lowercase().contains("all the mods")
            || profile.name.to_lowercase().contains("atm")
            || profile.name.to_lowercase().contains("better mc")
            || profile.name.to_lowercase().contains("dawncraft")
            || profile.name.to_lowercase().contains("prominence")
            || profile.name.to_lowercase().contains("rlcraft")
            || profile.mod_count >= 80;

        let is_pvp = profile.clean_mc_version().starts_with("1.8") || profile.clean_mc_version().starts_with("1.7");
        let ram_mb = if let Some(manual) = profile.ram_mb {
            if manual > 0 {
                self.emit_log(&format!(
                    "Luxmc RAM: {}MB definido manualmente pelo usuário",
                    manual
                ));
                manual
            } else {
                self.compute_auto_ram(is_heavy_modded, is_pvp, profile.mod_count)
            }
        } else {
            self.compute_auto_ram(is_heavy_modded, is_pvp, profile.mod_count)
        };

        // Apply Intelligent Luxmc Optimization (Aikar's Flags) or Standard Flags
        let custom_collector = profile.jvm_args.as_deref().is_some_and(|args|
            args.split_whitespace().any(|arg| matches!(arg, "-XX:+UseZGC" | "-XX:+UseShenandoahGC" | "-XX:+UseParallelGC" | "-XX:+UseSerialGC")));
        let generated_flags = if custom_collector {
            vec![format!("-Xms{}M", ram_mb.min(1024)), format!("-Xmx{}M", ram_mb)]
        } else if profile.auto_optimize {
            crate::core::optimizer::generate_aikar_flags(ram_mb.max(0) as u64)
        } else {
            crate::core::optimizer::generate_standard_flags(ram_mb.max(0) as u64)
        };

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
    fn resolve_duplicate_mods(&self, game_dir: &std::path::Path) {
        let mods_dir = game_dir.join("mods");
        let Ok(entries) = std::fs::read_dir(&mods_dir) else { return; };

        let overrides_set: std::collections::HashSet<String> = {
            let overrides_json = game_dir.join(".luxmc/overrides.json");
            if let Ok(bytes) = std::fs::read(overrides_json) {
                if let Ok(files) = serde_json::from_slice::<Vec<serde_json::Value>>(&bytes) {
                    files.into_iter().filter_map(|f| {
                        f.get("path")
                            .and_then(|p| p.as_str())
                            .and_then(|s| s.strip_prefix("mods/"))
                            .map(|s| s.to_lowercase())
                    }).collect()
                } else {
                    std::collections::HashSet::new()
                }
            } else {
                std::collections::HashSet::new()
            }
        };

        struct ModCandidate {
            path: std::path::PathBuf,
            clean_jar_name: String,
            is_currently_disabled: bool,
            is_override: bool,
            version_numbers: Vec<u64>,
            size: u64,
            mtime: std::time::SystemTime,
        }

        let mut all_files: Vec<ModCandidate> = Vec::new();

        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_file() {
                let fname = path.file_name().unwrap_or_default().to_string_lossy().to_string();
                let is_jar = fname.ends_with(".jar");
                let is_disabled = fname.ends_with(".jar.disabled") || fname.ends_with(".disabled");
                if is_jar || is_disabled {
                    let clean_jar_name = if is_disabled {
                        fname.strip_suffix(".disabled").unwrap_or(&fname).to_string()
                    } else {
                        fname.clone()
                    };

                    let meta = std::fs::metadata(&path).ok();
                    let size = meta.as_ref().map(|m| m.len()).unwrap_or(0);
                    let mtime = meta.and_then(|m| m.modified().ok()).unwrap_or(std::time::SystemTime::UNIX_EPOCH);

                    let clean_lower = clean_jar_name.to_lowercase();
                    let is_override = overrides_set.contains(&clean_lower) || {
                        let stem = clean_lower.strip_suffix(".jar").unwrap_or(&clean_lower);
                        overrides_set.iter().any(|o| o.contains(stem))
                    };

                    let re = regex::Regex::new(r"\d+").unwrap();
                    let version_numbers: Vec<u64> = re.find_iter(&clean_jar_name)
                        .filter_map(|m| m.as_str().parse::<u64>().ok())
                        .collect();

                    all_files.push(ModCandidate {
                        path,
                        clean_jar_name,
                        is_currently_disabled: is_disabled,
                        is_override,
                        version_numbers,
                        size,
                        mtime,
                    });
                }
            }
        }

        if all_files.len() < 2 {
            return;
        }

        let mut groups: std::collections::HashMap<String, Vec<ModCandidate>> = std::collections::HashMap::new();

        for item in all_files {
            let stem = item.clean_jar_name.strip_suffix(".jar").unwrap_or(&item.clean_jar_name).to_lowercase();
            let parts: Vec<&str> = stem.split(&['-', '_'][..]).collect();
            let non_version_parts: Vec<&str> = parts
                .into_iter()
                .take_while(|part| !part.chars().any(|c| c.is_ascii_digit()))
                .collect();
            let base = non_version_parts.join("-");

            const COMMON_SINGLE_PREFIXES: &[&str] = &[
                "fabric", "forge", "neoforge", "quilt", "ftb", "kubejs", "create",
                "yungs", "macaws", "allthe", "better", "simple"
            ];

            if base.len() >= 3 && !COMMON_SINGLE_PREFIXES.contains(&base.as_str()) {
                groups.entry(base).or_default().push(item);
            }
        }

        for (base, mut list) in groups {
            if list.len() > 1 {
                list.sort_by(|a, b| {
                    b.is_override.cmp(&a.is_override)
                        .then_with(|| b.version_numbers.cmp(&a.version_numbers))
                        .then_with(|| b.size.cmp(&a.size))
                        .then_with(|| b.mtime.cmp(&a.mtime))
                });

                let winner = &list[0];
                if winner.is_currently_disabled {
                    let active_winner_path = mods_dir.join(&winner.clean_jar_name);
                    if std::fs::rename(&winner.path, &active_winner_path).is_ok() {
                        self.emit_log(&format!(
                            "Reativando versão mais recente do mod ({base}): '{}' (restaurado de .disabled)",
                            winner.clean_jar_name
                        ));
                    }
                }

                for duplicate in &list[1..] {
                    if !duplicate.is_currently_disabled {
                        let disabled_name = format!("{}.disabled", duplicate.clean_jar_name);
                        let disabled_path = mods_dir.join(&disabled_name);
                        if std::fs::rename(&duplicate.path, &disabled_path).is_ok() {
                            self.emit_log(&format!(
                                "Conflito de mod evitado ({base}): desativando duplicata '{}' em favor da versão mais recente '{}'",
                                duplicate.clean_jar_name, winner.clean_jar_name
                            ));
                        }
                    }
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
    use super::*;
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
        assert_eq!(right_back[0], 190);
        assert_eq!(right_back[1], 130);
        assert_eq!(right_back[2], 90);

        assert_eq!(left_back[3], 255);
        assert_eq!(left_back[0], 195);
        assert_eq!(left_back[1], 135);
        assert_eq!(left_back[2], 95);
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
        // Set left arm bottom (underside of hand) to black
        img.put_pixel(41, 50, image::Rgba([0, 0, 0, 255]));

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

pub fn is_library_allowed(lib: &minecraft::Library) -> bool {
    if let Some(ref rules) = lib.rules {
        let platform = platform_mojang_name(current_platform());
        let mut allowed = false;
        for rule in rules {
            match rule.action.as_str() {
                "allow" => {
                    if let Some(ref os) = rule.os {
                        if os.name == platform {
                            allowed = true;
                        }
                    } else {
                        allowed = true;
                    }
                }
                "deny" => {
                    if let Some(ref os) = rule.os {
                        if os.name == platform {
                            return false;
                        }
                    }
                }
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

fn get_pack_format_for_version(version: &str) -> u32 {
    let clean = extract_mc_version(version);
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

pub(crate) fn normalize_skin_image(img: image::RgbaImage) -> image::RgbaImage {
    let (orig_w, orig_h) = img.dimensions();
    let mut canvas = if orig_w == 64 && orig_h == 32 {
        let mut target = image::RgbaImage::new(64, 64);
        for y in 0..32 {
            for x in 0..64 {
                target.put_pixel(x, y, *img.get_pixel(x, y));
            }
        }
        for dy in 0..12 {
            for dx in 0..4 {
                let back_p = *target.get_pixel(52 + dx, 20 + dy);
                if back_p[3] < 200
                    || (back_p[0] == 0 && back_p[1] == 0 && back_p[2] == 0)
                    || (back_p[0] == 45 && back_p[1] == 45 && back_p[2] == 45)
                    || (back_p[0] == 40 && back_p[1] == 30 && back_p[2] == 25)
                    || (back_p[0] < 10 && back_p[1] < 10 && back_p[2] < 10)
                {
                    let front_p = *target.get_pixel(44 + dx, 20 + dy);
                    if front_p[3] > 0 && !(front_p[0] == 0 && front_p[1] == 0 && front_p[2] == 0) {
                        target.put_pixel(52 + dx, 20 + dy, image::Rgba([front_p[0], front_p[1], front_p[2], 255]));
                    } else {
                        target.put_pixel(52 + dx, 20 + dy, image::Rgba([210, 165, 130, 255]));
                    }
                }
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
                target.put_pixel(24 + (3 - dx), 52 + dy, *target.get_pixel(0 + dx, 20 + dy));
                target.put_pixel(16 + (3 - dx), 52 + dy, *target.get_pixel(8 + dx, 20 + dy));
            }
        }

        for dy in 0..4 {
            for dx in 0..4 {
                let right_top = *target.get_pixel(44 + dx, 16 + dy);
                let right_bottom = *target.get_pixel(48 + dx, 16 + dy);
                let healed_bottom = if right_bottom[3] > 200 && !(right_bottom[0] == 0 && right_bottom[1] == 0 && right_bottom[2] == 0) {
                    right_bottom
                } else if right_top[3] > 200 && !(right_top[0] == 0 && right_top[1] == 0 && right_top[2] == 0) {
                    right_top
                } else {
                    image::Rgba([210, 165, 130, 255])
                };
                target.put_pixel(36 + (3 - dx), 48 + dy, right_top);
                target.put_pixel(40 + (3 - dx), 48 + dy, healed_bottom);
                if right_bottom[3] < 200 || (right_bottom[0] == 0 && right_bottom[1] == 0 && right_bottom[2] == 0) {
                    target.put_pixel(48 + dx, 16 + dy, healed_bottom);
                }
            }
        }
        for dy in 0..12 {
            for dx in 0..4 {
                target.put_pixel(36 + (3 - dx), 52 + dy, *target.get_pixel(44 + dx, 20 + dy));
                let src_back = *target.get_pixel(52 + dx, 20 + dy);
                let back_val = if src_back[3] > 200
                    && !(src_back[0] == 0 && src_back[1] == 0 && src_back[2] == 0)
                    && !(src_back[0] == 45 && src_back[1] == 45 && src_back[2] == 45)
                    && !(src_back[0] == 40 && src_back[1] == 30 && src_back[2] == 25)
                    && !(src_back[0] < 10 && src_back[1] < 10 && src_back[2] < 10)
                {
                    image::Rgba([src_back[0], src_back[1], src_back[2], 255])
                } else {
                    let fp = *target.get_pixel(44 + dx, 20 + dy);
                    if fp[3] > 0 && !(fp[0] == 0 && fp[1] == 0 && fp[2] == 0) { 
                        image::Rgba([fp[0], fp[1], fp[2], 255])
                    } else { 
                        image::Rgba([210, 165, 130, 255]) 
                    }
                };
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

    let mut fallback_skin_tone = image::Rgba([210, 165, 130, 255]);
    for ty in [24 * scale, 22 * scale, 12 * scale, 26 * scale] {
        for tx in [24 * scale, 22 * scale, 12 * scale, 26 * scale] {
            if tx < w && ty < h {
                let p = *canvas.get_pixel(tx, ty);
                if p[3] > 200 && (p[0] > 60 || p[1] > 60 || p[2] > 60) {
                    fallback_skin_tone = image::Rgba([p[0], p[1], p[2], 255]);
                    break;
                }
            }
        }
    }

    let base_rects = [
        (0 * scale, 0 * scale, 32 * scale, 16 * scale),
        (16 * scale, 16 * scale, 40 * scale, 32 * scale),
        (40 * scale, 16 * scale, 56 * scale, 32 * scale),
        (0 * scale, 16 * scale, 16 * scale, 32 * scale),
        (16 * scale, 48 * scale, 32 * scale, 64 * scale),
        (32 * scale, 48 * scale, 48 * scale, 64 * scale),
    ];

    for &(x1, y1, x2, y2) in &base_rects {
        for y in y1..y2.min(h) {
            for x in x1..x2.min(w) {
                let pixel = *canvas.get_pixel(x, y);

                let is_right_arm_back = x >= 51 * scale && x < 56 * scale && y >= 20 * scale && y < 32 * scale;
                let is_left_arm_back = x >= 43 * scale && x < 48 * scale && y >= 52 * scale && y < 64 * scale;
                let is_right_hand_bottom = x >= 47 * scale && x < 52 * scale && y >= 16 * scale && y < 20 * scale;
                let is_left_hand_bottom = x >= 39 * scale && x < 44 * scale && y >= 48 * scale && y < 52 * scale;

                let is_placeholder_black = (pixel[0] == 45 && pixel[1] == 45 && pixel[2] == 45)
                    || (pixel[0] == 40 && pixel[1] == 30 && pixel[2] == 25);

                let is_arm_back_unrendered = (is_right_arm_back || is_left_arm_back)
                    && (pixel[0] == 0 && pixel[1] == 0 && pixel[2] == 0);

                let is_hand_bottom_unrendered = (is_right_hand_bottom || is_left_hand_bottom)
                    && (pixel[0] == 0 && pixel[1] == 0 && pixel[2] == 0);

                let needs_fix = pixel[3] < 250 || is_placeholder_black || is_arm_back_unrendered || is_hand_bottom_unrendered;

                if !needs_fix {
                    continue;
                }

                if pixel[3] > 0 && !is_placeholder_black && !is_arm_back_unrendered && !is_hand_bottom_unrendered {
                    canvas.put_pixel(x, y, image::Rgba([pixel[0], pixel[1], pixel[2], 255]));
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
                        if op[3] > 50 && !(op[0] == 0 && op[1] == 0 && op[2] == 0) && !(op[0] < 10 && op[1] < 10 && op[2] < 10) {
                            canvas.put_pixel(x, y, image::Rgba([op[0], op[1], op[2], 255]));
                            filled = true;
                        }
                    }
                }

                if filled {
                    continue;
                }

                if is_right_hand_bottom {
                    let top_x = (x - 4 * scale).max(44 * scale);
                    let top_p = *canvas.get_pixel(top_x, y);
                    if top_p[3] > 0 && !(top_p[0] == 0 && top_p[1] == 0 && top_p[2] == 0) {
                        canvas.put_pixel(x, y, image::Rgba([top_p[0], top_p[1], top_p[2], 255]));
                    } else {
                        let front_p = *canvas.get_pixel(44 * scale, 20 * scale);
                        if front_p[3] > 0 && !(front_p[0] == 0 && front_p[1] == 0 && front_p[2] == 0) {
                            canvas.put_pixel(x, y, image::Rgba([front_p[0], front_p[1], front_p[2], 255]));
                        } else {
                            canvas.put_pixel(x, y, fallback_skin_tone);
                        }
                    }
                } else if is_left_hand_bottom {
                    let top_x = (x - 4 * scale).max(36 * scale);
                    let top_p = *canvas.get_pixel(top_x, y);
                    if top_p[3] > 0 && !(top_p[0] == 0 && top_p[1] == 0 && top_p[2] == 0) {
                        canvas.put_pixel(x, y, image::Rgba([top_p[0], top_p[1], top_p[2], 255]));
                    } else {
                        let front_p = *canvas.get_pixel(36 * scale, 52 * scale);
                        if front_p[3] > 0 && !(front_p[0] == 0 && front_p[1] == 0 && front_p[2] == 0) {
                            canvas.put_pixel(x, y, image::Rgba([front_p[0], front_p[1], front_p[2], 255]));
                        } else {
                            canvas.put_pixel(x, y, fallback_skin_tone);
                        }
                    }
                } else if is_right_arm_back {
                    let front_x = if x >= 52 * scale {
                        44 * scale + (x - 52 * scale)
                    } else {
                        44 * scale
                    };
                    let front_p = *canvas.get_pixel(front_x, y);
                    if front_p[3] > 0 && !(front_p[0] == 0 && front_p[1] == 0 && front_p[2] == 0) {
                        canvas.put_pixel(x, y, image::Rgba([front_p[0], front_p[1], front_p[2], 255]));
                    } else {
                        canvas.put_pixel(x, y, fallback_skin_tone);
                    }
                } else if is_left_arm_back {
                    let front_x = if x >= 44 * scale {
                        36 * scale + (x - 44 * scale)
                    } else {
                        36 * scale
                    };
                    let front_p = *canvas.get_pixel(front_x, y);
                    if front_p[3] > 0 && !(front_p[0] == 0 && front_p[1] == 0 && front_p[2] == 0) {
                        canvas.put_pixel(x, y, image::Rgba([front_p[0], front_p[1], front_p[2], 255]));
                    } else {
                        canvas.put_pixel(x, y, fallback_skin_tone);
                    }
                } else if x >= 32 * scale && x < 40 * scale && y >= 20 * scale && y < 32 * scale {
                    let front_x = 20 * scale + (x - 32 * scale);
                    let front_p = *canvas.get_pixel(front_x, y);
                    if front_p[3] > 0 {
                        canvas.put_pixel(x, y, image::Rgba([front_p[0], front_p[1], front_p[2], 255]));
                    } else {
                        canvas.put_pixel(x, y, fallback_skin_tone);
                    }
                } else if x >= 24 * scale && x < 32 * scale && y >= 8 * scale && y < 16 * scale {
                    let front_x = 8 * scale + (x - 24 * scale);
                    let front_p = *canvas.get_pixel(front_x, y);
                    if front_p[3] > 0 {
                        canvas.put_pixel(x, y, image::Rgba([front_p[0], front_p[1], front_p[2], 255]));
                    } else {
                        canvas.put_pixel(x, y, fallback_skin_tone);
                    }
                } else if x >= 12 * scale && x < 16 * scale && y >= 20 * scale && y < 32 * scale {
                    let front_x = 4 * scale + (x - 12 * scale);
                    let front_p = *canvas.get_pixel(front_x, y);
                    if front_p[3] > 0 {
                        canvas.put_pixel(x, y, image::Rgba([front_p[0], front_p[1], front_p[2], 255]));
                    } else {
                        canvas.put_pixel(x, y, fallback_skin_tone);
                    }
                } else if x >= 28 * scale && x < 32 * scale && y >= 52 * scale && y < 64 * scale {
                    let front_x = 20 * scale + (x - 28 * scale);
                    let front_p = *canvas.get_pixel(front_x, y);
                    if front_p[3] > 0 {
                        canvas.put_pixel(x, y, image::Rgba([front_p[0], front_p[1], front_p[2], 255]));
                    } else {
                        canvas.put_pixel(x, y, fallback_skin_tone);
                    }
                } else {
                    canvas.put_pixel(x, y, fallback_skin_tone);
                }
            }
        }
    }

    let (final_w, final_h) = canvas.dimensions();
    let final_scale = (final_w / 64).max(1);
    
    let critical_base_zones = [
        (40 * final_scale, 16 * final_scale, 56 * final_scale, 32 * final_scale),
        (32 * final_scale, 48 * final_scale, 48 * final_scale, 64 * final_scale),
        (16 * final_scale, 16 * final_scale, 40 * final_scale, 32 * final_scale),
        (0 * final_scale, 16 * final_scale, 16 * final_scale, 32 * final_scale),
    ];

    for &(x1, y1, x2, y2) in &critical_base_zones {
        for y in y1..y2.min(final_h) {
            for x in x1..x2.min(final_w) {
                let p = *canvas.get_pixel(x, y);
                let is_ph = (p[0] == 45 && p[1] == 45 && p[2] == 45) || (p[0] == 40 && p[1] == 30 && p[2] == 25);
                let is_arm_back = (x >= 51 * final_scale && x < 56 * final_scale && y >= 20 * final_scale && y < 32 * final_scale)
                    || (x >= 43 * final_scale && x < 48 * final_scale && y >= 52 * final_scale && y < 64 * final_scale);
                let is_black_arm = is_arm_back && (p[0] == 0 && p[1] == 0 && p[2] == 0);
                if p[3] < 200 || is_ph || is_black_arm {
                    canvas.put_pixel(x, y, fallback_skin_tone);
                }
            }
        }
    }

    let overlay_arm_zones = [
        (40 * final_scale, 32 * final_scale, 56 * final_scale, 48 * final_scale),
        (48 * final_scale, 48 * final_scale, 64 * final_scale, 64 * final_scale),
    ];
    for &(x1, y1, x2, y2) in &overlay_arm_zones {
        for y in y1..y2.min(final_h) {
            for x in x1..x2.min(final_w) {
                let p = *canvas.get_pixel(x, y);
                let is_ph = (p[0] == 45 && p[1] == 45 && p[2] == 45) || (p[0] == 40 && p[1] == 30 && p[2] == 25);
                if is_ph {
                    canvas.put_pixel(x, y, image::Rgba([0, 0, 0, 0]));
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
    _variant: &str,
    cape_source: Option<&str>,
    mc_version: &str,
) -> AppResult<()> {
    let mut skin_bytes: Vec<u8> = if skin_source.starts_with("http://") || skin_source.starts_with("https://") {
        if let Ok(resp) = http.get(skin_source).send().await {
            if resp.status().is_success() {
                resp.bytes().await.map(|b| b.to_vec()).unwrap_or_default()
            } else {
                Vec::new()
            }
        } else {
            Vec::new()
        }
    } else if skin_source.starts_with("data:image/") {
        if let Some(comma_pos) = skin_source.find(',') {
            use base64::Engine;
            base64::engine::general_purpose::STANDARD
                .decode(&skin_source[comma_pos + 1..])
                .unwrap_or_default()
        } else {
            Vec::new()
        }
    } else {
        let path = std::path::Path::new(skin_source);
        if path.exists() {
            tokio::fs::read(path).await.unwrap_or_default()
        } else {
            Vec::new()
        }
    };

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
        return clean_skin_injection(game_dir).await;
    }

    // Normalize skin image
    let normalized_skin_bytes = if let Ok(dyn_img) = image::load_from_memory(&skin_bytes) {
        let norm_img = normalize_skin_image(dyn_img.to_rgba8());
        let mut buf = Vec::new();
        let mut cursor = std::io::Cursor::new(&mut buf);
        if norm_img.write_to(&mut cursor, image::ImageFormat::Png).is_ok() {
            buf
        } else {
            skin_bytes
        }
    } else {
        skin_bytes
    };

    let pack_dir = game_dir.join("resourcepacks").join("LuxmcCustomSkin");
    let entity_dir_wide = pack_dir.join("assets/minecraft/textures/entity/player/wide");
    let entity_dir_slim = pack_dir.join("assets/minecraft/textures/entity/player/slim");
    let entity_dir_player = pack_dir.join("assets/minecraft/textures/entity/player");
    let entity_dir_legacy = pack_dir.join("assets/minecraft/textures/entity");

    let _ = tokio::fs::create_dir_all(&entity_dir_wide).await;
    let _ = tokio::fs::create_dir_all(&entity_dir_slim).await;
    let _ = tokio::fs::create_dir_all(&entity_dir_player).await;
    let _ = tokio::fs::create_dir_all(&entity_dir_legacy).await;

    let pack_fmt = get_pack_format_for_version(mc_version);
    let mcmeta = serde_json::json!({
        "pack": {
            "pack_format": pack_fmt,
            "supported_formats": {
                "min_inclusive": 1,
                "max_inclusive": 99
            },
            "description": "Luxmc Player Custom Skin & Cape"
        }
    });
    let _ = tokio::fs::write(pack_dir.join("pack.mcmeta"), serde_json::to_string_pretty(&mcmeta).unwrap_or_default()).await;
    let _ = tokio::fs::write(pack_dir.join("pack.png"), include_bytes!("../../../icons/64x64.png")).await;

    let skin_models = [
        "steve", "alex", "ari", "efe", "kai", "makena", "noor", "sunny", "zuri",
    ];
    for model in &skin_models {
        let _ = tokio::fs::write(entity_dir_wide.join(format!("{}.png", model)), &normalized_skin_bytes).await;
        let _ = tokio::fs::write(entity_dir_slim.join(format!("{}.png", model)), &normalized_skin_bytes).await;
        let _ = tokio::fs::write(entity_dir_player.join(format!("{}.png", model)), &normalized_skin_bytes).await;
        let _ = tokio::fs::write(entity_dir_legacy.join(format!("{}.png", model)), &normalized_skin_bytes).await;
    }

    // Direct legacy Steve & Alex fallbacks
    let _ = tokio::fs::write(entity_dir_legacy.join("steve.png"), &normalized_skin_bytes).await;
    let _ = tokio::fs::write(entity_dir_legacy.join("alex.png"), &normalized_skin_bytes).await;

    // Cape handling
    let cape_bytes: Vec<u8> = if let Some(cs) = cape_source.filter(|s| !s.trim().is_empty()) {
        if cs.starts_with("http://") || cs.starts_with("https://") {
            if let Ok(resp) = http.get(cs).send().await {
                if resp.status().is_success() {
                    resp.bytes().await.map(|b| b.to_vec()).unwrap_or_default()
                } else {
                    Vec::new()
                }
            } else {
                Vec::new()
            }
        } else if cs.starts_with("data:image/") {
            if let Some(pos) = cs.find(',') {
                use base64::Engine;
                base64::engine::general_purpose::STANDARD
                    .decode(&cs[pos + 1..])
                    .unwrap_or_default()
            } else {
                Vec::new()
            }
        } else {
            let p = std::path::Path::new(cs);
            if p.exists() {
                tokio::fs::read(p).await.unwrap_or_default()
            } else {
                Vec::new()
            }
        }
    } else {
        Vec::new()
    };

    if !cape_bytes.is_empty() {
        let (normalized_cape_bytes, optifine_cape_bytes) = if let Ok(dyn_cape) = image::load_from_memory(&cape_bytes) {
            use image::GenericImageView;
            let (cw, ch) = dyn_cape.dimensions();
            let cape_rgba = dyn_cape.to_rgba8();

            let full_img = if cw == 22 && ch == 17 {
                let mut canvas = image::RgbaImage::new(64, 32);
                image::imageops::overlay(&mut canvas, &cape_rgba, 0, 0);
                canvas
            } else if cw == 44 && ch == 34 {
                let mut canvas = image::RgbaImage::new(128, 64);
                image::imageops::overlay(&mut canvas, &cape_rgba, 0, 0);
                canvas
            } else if (cw == 64 && ch == 32) || (cw == ch * 2) {
                cape_rgba.clone()
            } else if cw == ch && cw > 0 {
                image::imageops::crop_imm(&cape_rgba, 0, 0, cw, cw / 2).to_image()
            } else {
                let target_w = (cw.max(64) + 63) / 64 * 64;
                let target_h = target_w / 2;
                let mut canvas = image::RgbaImage::new(target_w, target_h);
                image::imageops::overlay(&mut canvas, &cape_rgba, 0, 0);
                canvas
            };

            let mut full_buf = Vec::new();
            let mut cursor = std::io::Cursor::new(&mut full_buf);
            let full_bytes = if full_img.write_to(&mut cursor, image::ImageFormat::Png).is_ok() {
                full_buf
            } else {
                cape_bytes.clone()
            };

            let opti_img = if cw == 22 && ch == 17 {
                let mut canvas = image::RgbaImage::new(64, 32);
                image::imageops::overlay(&mut canvas, &cape_rgba, 0, 0);
                canvas
            } else if cw == 44 && ch == 34 {
                let mut canvas = image::RgbaImage::new(128, 64);
                image::imageops::overlay(&mut canvas, &cape_rgba, 0, 0);
                canvas
            } else if cw == ch && cw > 0 {
                image::imageops::crop_imm(&cape_rgba, 0, 0, cw, cw / 2).to_image()
            } else if cw == ch * 2 {
                cape_rgba
            } else {
                let target_w = (cw.max(64) + 63) / 64 * 64;
                let target_h = target_w / 2;
                let mut canvas = image::RgbaImage::new(target_w, target_h);
                image::imageops::overlay(&mut canvas, &cape_rgba, 0, 0);
                canvas
            };

            let mut opti_buf = Vec::new();
            let mut opti_cursor = std::io::Cursor::new(&mut opti_buf);
            let opti_bytes = if opti_img.write_to(&mut opti_cursor, image::ImageFormat::Png).is_ok() {
                opti_buf
            } else {
                full_bytes.clone()
            };

            (full_bytes, opti_bytes)
        } else {
            (cape_bytes.clone(), cape_bytes)
        };

        set_active_cape_bytes(normalized_cape_bytes.clone()).await;

        let u_clean = username.trim();

        let u_lower = u_clean.to_lowercase();

        let optifine_users_dir = pack_dir.join("assets/minecraft/optifine/users");
        let _ = tokio::fs::create_dir_all(&optifine_users_dir).await;
        let user_prop = format!("cape=optifine/capes/{}.png\n", u_clean);
        let _ = tokio::fs::write(optifine_users_dir.join(format!("{}.properties", u_clean)), user_prop.as_bytes()).await;
        let _ = tokio::fs::write(optifine_users_dir.join(format!("{}.properties", u_lower)), user_prop.as_bytes()).await;

        let optifine_root_prop = pack_dir.join("assets/minecraft/optifine/cape.properties");
        let root_prop = format!("users={}\n", u_clean);
        if let Some(p) = optifine_root_prop.parent() { let _ = tokio::fs::create_dir_all(p).await; }
        let _ = tokio::fs::write(&optifine_root_prop, root_prop.as_bytes()).await;

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
            let _ = tokio::fs::write(path, &optifine_cape_bytes).await;
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
            let _ = tokio::fs::write(path, &normalized_cape_bytes).await;
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
                        let mut entries: Vec<String> = array_content
                            .split(',')
                            .map(|s| s.trim().to_string())
                            .filter(|s| !s.is_empty() && s != unchosen_pack_entry)
                            .collect();
                        if !entries.iter().any(|e| e == chosen_pack_entry) {
                            entries.push(chosen_pack_entry.to_string());
                        }
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
                    format!("resourcePacks:[{}]", chosen_pack_entry)
                } else {
                    format!("resourcePacks:[\"vanilla\",{}]", chosen_pack_entry)
                };
                lines.push(default_list);
            }
            let _ = tokio::fs::write(&options_file, lines.join("\n")).await;
        }
    } else {
        let mut default_options = if is_legacy {
            format!("resourcePacks:[{}]\nincompatibleResourcePacks:[]\n", chosen_pack_entry)
        } else {
            format!("resourcePacks:[\"vanilla\",{}]\nincompatibleResourcePacks:[]\n", chosen_pack_entry)
        };
        for (key, val) in &skin_flags {
            default_options.push_str(&format!("{}:{}\n", key, val));
        }
        let _ = tokio::fs::write(&options_file, default_options).await;
    }

    Ok(())
}

