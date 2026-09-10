use std::path::PathBuf;
use std::process::Stdio;
use tauri::Emitter;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::Command;

use crate::core::downloader::DownloadManager;
use crate::core::java::JavaRuntimeManager;
use crate::core::minecraft::{self, VersionDetail};
use crate::error::AppResult;

const DEV_CLIENT_ID: &str = "00000000-0000-0000-0000-000000000002";
const DEV_XUID: &str = "0";
const LAUNCHER_NAME: &str = "Luxmc";
const LAUNCHER_VERSION: &str = "1.1.0-BETA";

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

    pub async fn launch(
        &self,
        detail: &VersionDetail,
        username: &str,
        uuid: &str,
        access_token: &str,
        user_type: &str,
        game_dir: &PathBuf,
        profile: &crate::db::models::ProfileRow,
    ) -> AppResult<u32> {
        self.emit_stage(LaunchStage::Preparing);
        self.emit_log(&format!("Preparing to launch {}", detail.id));
        self.emit_log(&format!("Host platform: {:?}", current_platform()));

        self.emit_stage(LaunchStage::CheckingJava);
        let major = detail.java_major_version();
        self.emit_log(&format!("Java major version required: {}", major));

        let java_path = self.java.ensure_java(major).await?;
        self.emit_log(&format!("Java binary: {}", java_path.display()));

        self.emit_stage(LaunchStage::ResolvingClasspath);
        let mut classpath = self.build_classpath(detail);

        let mut main_class = detail
            .main_class
            .as_deref()
            .unwrap_or("net.minecraft.client.main.Main")
            .to_string();

        let mut extra_jvm_args = Vec::new();

        let mut loader = profile.loader.trim().to_lowercase();
        if loader == "vanilla" || loader.is_empty() {
            let mods_dir = game_dir.join("mods");
            if let Ok(mut entries) = tokio::fs::read_dir(&mods_dir).await {
                let mut has_jars = false;
                while let Ok(Some(entry)) = entries.next_entry().await {
                    if let Some(ext) = entry.path().extension() {
                        if ext == "jar" {
                            has_jars = true;
                            break;
                        }
                    }
                }
                if has_jars {
                    self.emit_log("Detected mods in mods/ directory; automatically enabling Fabric loader for this session");
                    loader = "fabric".to_string();
                }
            }
        }

        if loader == "fabric" || loader == "quilt" {
            if loader == "fabric" {
                let mods_dir = game_dir.join("mods");
                let _ = crate::core::loaders::fabric::ensure_fabric_api(
                    self.downloader.http(),
                    &mods_dir,
                    &profile.mc_version,
                ).await;
            }

            self.emit_log(&format!("Resolving {} loader for Minecraft {}...", loader, profile.mc_version));
            match crate::core::loaders::prepare_loader(
                self.downloader.http(),
                &self.downloader.libraries_dir(),
                &loader,
                &profile.mc_version,
                profile.loader_version.as_deref(),
            ).await {
                Ok(prep) => {
                    self.emit_log(&format!("{} loader ready: mainClass = {}", loader, prep.main_class));
                    main_class = prep.main_class;
                    // Prepend loader libraries to classpath
                    let mut new_cp = prep.classpath_entries;
                    new_cp.extend(classpath);
                    classpath = new_cp;
                    extra_jvm_args.extend(prep.jvm_args);
                }
                Err(e) => {
                    self.emit_log(&format!("WARNING: Failed to prepare {} loader: {}. Falling back to standard launch.", loader, e));
                }
            }
        }

        self.emit_log(&format!("Classpath entries: {}", classpath.len()));

        self.emit_stage(LaunchStage::ExtractingNatives);
        let natives_dir = self.prepare_natives(detail).await?;
        self.emit_log(&format!("Natives dir: {}", natives_dir.display()));

        self.emit_stage(LaunchStage::ResolvingArgs);
        let mut jvm_args = self.build_jvm_args(detail, &classpath, &natives_dir, game_dir, profile);
        jvm_args.extend(extra_jvm_args);

        let game_args =
            self.build_game_args(detail, username, uuid, access_token, user_type, game_dir, profile);

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
        let safe_jvm_args: Vec<String> = jvm_args
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
        let mut cmd = Command::new(&java_path);
        cmd.args(&safe_jvm_args)
            .arg(main_class)
            .args(&safe_game_args)
            .current_dir(game_dir)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .kill_on_drop(true)
            .env("PATH", std::env::var("PATH").unwrap_or_default());

        #[cfg(target_os = "linux")]
        {
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

        let mut child = cmd.spawn().map_err(|e| {
            let msg = format!("Failed to start Java process: {}", e);
            self.emit_log(&msg);
            self.emit_stage(LaunchStage::Failed);
            crate::error::AppError::Internal(msg)
        })?;

        let pid = child.id().unwrap_or(0);
        self.emit_log(&format!("Java process started, PID: {}", pid));
        self.emit_stage(LaunchStage::Running);

        let stdout = child.stdout.take().unwrap();
        let stderr = child.stderr.take().unwrap();

        let app_stdout = self.app.clone();
        tokio::spawn(async move {
            let reader = BufReader::new(stdout);
            let mut lines = reader.lines();
            while let Ok(Some(line)) = lines.next_line().await {
                tracing::info!(target: "game", "stdout: {}", line);
                if let Some(ref app) = app_stdout {
                    let _ = app.emit(
                        "game-log",
                        GameLogEntry {
                            stream: "stdout".into(),
                            message: line,
                        },
                    );
                }
            }
        });

        let app_stderr = self.app.clone();
        tokio::spawn(async move {
            let reader = BufReader::new(stderr);
            let mut lines = reader.lines();
            while let Ok(Some(line)) = lines.next_line().await {
                tracing::error!(target: "game", "stderr: {}", line);
                if let Some(ref app) = app_stderr {
                    let _ = app.emit(
                        "game-log",
                        GameLogEntry {
                            stream: "stderr".into(),
                            message: line,
                        },
                    );
                }
            }
        });

        let app_exit = self.app.clone();
        let detail_id = detail.id.clone();
        tokio::spawn(async move {
            match child.wait().await {
                Ok(status) => {
                    let code = status.code().unwrap_or(-1);
                    let msg = format!("Game process exited with code {}", code);
                    tracing::info!(target: "launch", "{}", msg);
                    if let Some(ref app) = app_exit {
                        let _ = app.emit(
                            "game-exit",
                            GameExitEvent {
                                version_id: detail_id,
                                code,
                                success: status.success(),
                            },
                        );
                    }
                }
                Err(e) => {
                    let msg = format!("Game process error: {}", e);
                    tracing::error!(target: "launch", "{}", msg);
                    if let Some(ref app) = app_exit {
                        let _ = app.emit(
                            "game-exit",
                            GameExitEvent {
                                version_id: detail_id,
                                code: -1,
                                success: false,
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

        for lib in &detail.libraries {
            if !is_library_allowed(lib) {
                continue;
            }

            if lib.name.contains("natives-linux") {
                let path = lib_path_from_name(&base, &lib.name);
                if path.exists() {
                    let output = Command::new("unzip")
                        .args([
                            "-o",
                            "-j",
                            path.to_str().unwrap_or(""),
                            "-d",
                            natives_dir.to_str().unwrap_or(""),
                        ])
                        .output()
                        .await;
                    match output {
                        Ok(out) => {
                            if !out.status.success() {
                                let stderr = String::from_utf8_lossy(&out.stderr);
                                self.emit_log(&format!(
                                    "unzip failed for {}: {}",
                                    lib.name, stderr
                                ));
                            }
                        }
                        Err(e) => {
                            self.emit_log(&format!("failed to run unzip for {}: {}", lib.name, e));
                        }
                    }
                } else {
                    self.emit_log(&format!("native jar not found: {}", path.display()));
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

        let so_count = std::fs::read_dir(&natives_dir)
            .map(|rd| {
                rd.filter(|e| {
                    e.as_ref()
                        .map(|f| f.path().extension().map(|ext| ext == "so").unwrap_or(false))
                        .unwrap_or(false)
                })
                .count()
            })
            .unwrap_or(0);
        self.emit_log(&format!(
            "Extracted {} native libraries to {}",
            so_count,
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

                    // Resolve vars first, then split on whitespace so compound args
                    // packed into one JSON string still get evaluated individually.
                    for sub in resolved.split_whitespace() {
                        if !jvm_arg_allowed_on_current_os(sub) {
                            self.emit_log(&format!("Dropped JVM arg on this platform: {}", sub));
                            continue;
                        }
                        args.push(sub.to_string());
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

        // Calculate RAM allocation from profile or dynamic system detection
        let ram_mb = if let Some(allocated) = profile.ram_mb {
            if allocated > 0 {
                allocated as u64
            } else {
                4096
            }
        } else {
            let mut sys = sysinfo::System::new_all();
            sys.refresh_memory();
            let total_ram_mb = sys.total_memory() / 1024 / 1024;
            std::cmp::min(std::cmp::max(total_ram_mb / 2, 2048), 8192)
        };

        // Apply Intelligent Luxmc Optimization (Aikar's Flags) or Standard Flags
        let generated_flags = if profile.auto_optimize {
            crate::core::optimizer::generate_aikar_flags(ram_mb)
        } else {
            crate::core::optimizer::generate_standard_flags(ram_mb)
        };

        for flag in generated_flags {
            if !args.iter().any(|a| a.starts_with(&flag.split('=').next().unwrap_or(&flag))) {
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

        args
    }

    fn evaluate_jvm_rules(&self, rules: &[serde_json::Value]) -> bool {
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
                    let actual_arch = if cfg!(target_arch = "x86_64") {
                        "x86"
                    } else if cfg!(target_arch = "aarch64") {
                        "arm64"
                    } else {
                        ""
                    };
                    if !actual_arch.is_empty() && arch != actual_arch {
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

    fn build_game_args(
        &self,
        detail: &VersionDetail,
        username: &str,
        uuid: &str,
        access_token: &str,
        user_type: &str,
        game_dir: &PathBuf,
        _profile: &crate::db::models::ProfileRow,
    ) -> Vec<String> {
        let mut args = Vec::new();

        let assets_dir = self.downloader.assets_dir();
        let asset_index_id = detail
            .asset_index
            .as_ref()
            .map(|ai| ai.id.as_str())
            .unwrap_or("legacy");

        let game_args_raw = detail.effective_game_args();

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
                .replace("${resolution_width}", "854")
                .replace("${resolution_height}", "480")
                .replace("${quickPlayPath}", "")
                .replace("${quickPlaySingleplayer}", "")
                .replace("${quickPlayMultiplayer}", "")
                .replace("${quickPlayRealms}", "");
            args.push(resolved);
        }

        if !args.contains(&"--username".to_string()) {
            args.insert(0, "--gameDir".to_string());
            args.insert(0, game_dir.to_string_lossy().to_string());
            args.insert(0, "--version".to_string());
            args.insert(0, detail.id.clone());
            args.insert(0, "--accessToken".to_string());
            args.insert(0, access_token.to_string());
            args.insert(0, "--uuid".to_string());
            args.insert(0, uuid.to_string());
            args.insert(0, "--username".to_string());
            args.insert(0, username.to_string());
            args.insert(0, "--userType".to_string());
            args.insert(0, user_type.to_string());
            args.insert(0, "--userProperties".to_string());
            args.insert(0, "{}".to_string());
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
}

pub fn is_library_allowed(lib: &minecraft::Library) -> bool {
    if let Some(ref rules) = lib.rules {
        let mut allowed = false;
        for rule in rules {
            match rule.action.as_str() {
                "allow" => {
                    if let Some(ref os) = rule.os {
                        if os.name == "linux" {
                            allowed = true;
                        }
                    } else {
                        allowed = true;
                    }
                }
                "deny" => {
                    if let Some(ref os) = rule.os {
                        if os.name == "linux" {
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
    let filename = if parts.len() > 3 {
        format!("{}-{}-{}", artifact, version, parts[3])
    } else {
        format!("{}-{}.jar", artifact, version)
    };
    base.join(format!("{}/{}/{}/{}", group, artifact, version, filename))
}
