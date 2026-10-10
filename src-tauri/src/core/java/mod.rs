use serde::Deserialize;
use std::path::PathBuf;
use tauri::Emitter;
use futures_util::{stream, StreamExt, TryStreamExt};
use sha1::Digest;

use crate::core::downloader::DownloadProgress;
use crate::error::{AppError, AppResult};

const JAVA_RUNTIME_MANIFEST_URL: &str =
	"https://launchermeta.mojang.com/v1/products/java-runtime/2ec0cc96c44e5a76b9c8b7c39df7210883d12871/all.json";

const MAX_RETRIES: u32 = 3;
const RETRY_BASE_DELAY_MS: u64 = 1000;

#[derive(Debug, Deserialize)]
struct RuntimeEntry {
    manifest: Option<RuntimeManifestRef>,
    version: Option<RuntimeVersion>,
}

#[derive(Debug, Deserialize)]
struct RuntimeManifestRef {
    url: Option<String>,
}

#[derive(Debug, Deserialize)]
struct RuntimeVersion {
    component: Option<String>,
    name: Option<String>,
}

#[derive(Debug, Deserialize)]
struct ComponentManifest {
    files: Option<std::collections::HashMap<String, ComponentFile>>,
}

#[derive(Debug, Deserialize)]
struct ComponentFile {
    #[serde(rename = "type")]
    file_type: Option<String>,
    downloads: Option<ComponentFileDownloads>,
    executable: Option<bool>,
    target: Option<String>,
}

#[derive(Debug, Deserialize)]
struct ComponentFileDownloads {
    raw: Option<ComponentDownload>,
}

#[derive(Debug, Deserialize)]
struct ComponentDownload {
    url: Option<String>,
    sha1: Option<String>,
    size: Option<u64>,
}

pub struct JavaRuntimeManager {
    http: reqwest::Client,
    data_dir: PathBuf,
    app: Option<tauri::AppHandle>,
}

fn current_mojang_platform_key() -> &'static str {
    if cfg!(target_os = "windows") {
        if cfg!(target_arch = "aarch64") {
            "windows-arm64"
        } else if cfg!(target_arch = "x86") {
            "windows-x86"
        } else {
            "windows-x64"
        }
    } else if cfg!(target_os = "macos") {
        if cfg!(target_arch = "aarch64") {
            "mac-os-arm64"
        } else {
            "mac-os"
        }
    } else if cfg!(target_arch = "x86") {
        "linux-i386"
    } else {
        "linux"
    }
}

impl JavaRuntimeManager {
    pub fn new(http: reqwest::Client, data_dir: PathBuf) -> Self {
        Self {
            http,
            data_dir,
            app: None,
        }
    }

    pub fn with_app(mut self, app: tauri::AppHandle) -> Self {
        self.app = Some(app);
        self
    }

    fn java_dir(&self, major: u32) -> PathBuf {
        self.data_dir.join("java").join(major.to_string())
    }

    fn java_bin(&self, major: u32) -> PathBuf {
        let bin_name = if cfg!(windows) { "java.exe" } else { "java" };
        self.java_dir(major).join("bin").join(bin_name)
    }

    fn emit_progress(&self, progress: &DownloadProgress) {
        if let Some(ref app) = self.app {
            let _ = app.emit("download-progress", progress);
        }
    }

    fn emit_log(&self, message: &str) {
        if let Some(ref app) = self.app {
            let _ = app.emit("launcher-log", message);
        }
        tracing::info!(target: "java", "{}", message);
    }

    fn verify_binary(&self, path: &std::path::Path, expected_major: u32) -> bool {
        verified_java(path, expected_major)
    }

    pub async fn ensure_java(&self, major_version: u32) -> AppResult<PathBuf> {
        static LOCKS: std::sync::LazyLock<std::sync::Mutex<std::collections::HashMap<u32, std::sync::Arc<tokio::sync::Mutex<()>>>>> = std::sync::LazyLock::new(|| std::sync::Mutex::new(std::collections::HashMap::new()));
        let lock = LOCKS.lock().map_err(|_| AppError::Internal("Runtime Java ocupado".into()))?.entry(major_version).or_default().clone();
        let _guard = lock.lock().await;
        let bin = self.java_bin(major_version);
        if bin.exists() && self.verify_binary(&bin, major_version) {
            self.emit_log(&format!(
                "Java {} found at {}",
                major_version,
                bin.display()
            ));
            return Ok(bin);
        }

        match self.find_system_java(major_version) {
            Ok(path) => {
                self.emit_log(&format!("Using system Java: {}", path.display()));
                Ok(path)
            }
            Err(_) => {
                self.emit_log(&format!(
                    "System Java {} not found, will download runtime for platform {}",
                    major_version,
                    current_mojang_platform_key()
                ));
                let primary = self.download_runtime(major_version).await;
                if let Err(error) = primary {
                    self.emit_log(&format!("Runtime Mojang indisponível: {error}. Tentando Temurin verificado."));
                    self.download_temurin(major_version).await.map_err(|fallback| AppError::Internal(format!("Não foi possível preparar Java {major_version}. Mojang: {error}. Temurin: {fallback}. Confira a conexão, o espaço em disco e os componentes do Windows; em Configurações → Java você também pode escolher um Java instalado.")))?;
                }

                let bin = self.java_bin(major_version);
                if bin.exists() && self.verify_binary(&bin, major_version) {
                    Ok(bin)
                } else {
                    let hint = if cfg!(windows) {
                        format!("Install Java {} manually from https://adoptium.net/", major_version)
                    } else {
                        format!("Install Java {} manually with: sudo pacman -S jre-openjdk", major_version)
                    };
                    Err(AppError::Internal(format!(
                        "Failed to install Java {} runtime. {}",
                        major_version, hint
                    )))
                }
            }
        }
    }

    pub fn find_system_java_pub(&self, major: u32) -> AppResult<PathBuf> {
        self.find_system_java(major)
    }

    fn find_system_java(&self, major: u32) -> AppResult<PathBuf> {
        let check_binary = verified_java;

        if cfg!(windows) {
            if major == 8 {
                let win_8_paths = [
                    r"C:\Program Files\Eclipse Adoptium\jre-8.0\bin\java.exe",
                    r"C:\Program Files\Eclipse Adoptium\jdk-8.0\bin\java.exe",
                    r"C:\Program Files\Java\jre1.8.0\bin\java.exe",
                    r"C:\Program Files\Java\jdk1.8.0\bin\java.exe",
                    r"C:\Program Files (x86)\Java\jre1.8.0\bin\java.exe",
                ];
                for p in win_8_paths {
                    let pb = PathBuf::from(p);
                    if pb.exists() && check_binary(&pb, 8) {
                        return Ok(pb);
                    }
                }
            }

            if let Ok(path_out) = crate::core::process::std_command("where.exe")
                .arg("java")
                .output()
            {
                for line in String::from_utf8_lossy(&path_out.stdout).lines() {
                    let trimmed = line.trim();
                    if !trimmed.is_empty() {
                        let pb = PathBuf::from(trimmed);
                        if pb.exists() && check_binary(&pb, major) {
                            return Ok(pb);
                        }
                    }
                }
            }

            if let Ok(java_home) = std::env::var("JAVA_HOME") {
                let p = PathBuf::from(java_home).join("bin").join("java.exe");
                if p.exists() && check_binary(&p, major) {
                    return Ok(p);
                }
            }
            for variable in ["ProgramW6432", "ProgramFiles", "ProgramFiles(x86)"] {
                let Some(base) = std::env::var_os(variable) else { continue };
                for vendor in ["Eclipse Adoptium", "Java", "Microsoft", "Amazon Corretto", "Zulu"] {
                    let Ok(entries) = std::fs::read_dir(PathBuf::from(&base).join(vendor)) else { continue };
                    for entry in entries.flatten() {
                        let binary = entry.path().join("bin").join("java.exe");
                        if binary.is_file() && check_binary(&binary, major) {
                            return Ok(binary);
                        }
                    }
                }
            }
        } else {
            let specific_linux_paths: &[&str] = match major {
                8 => &[
                    "/usr/lib/jvm/java-8-openjdk/bin/java",
                    "/usr/lib/jvm/java-1.8.0-openjdk/bin/java",
                    "/usr/lib/jvm/java-8-openjdk-amd64/bin/java",
                    "/usr/lib/jvm/temurin-8-jdk/bin/java",
                    "/usr/lib/jvm/zulu-8/bin/java",
                ],
                17 => &[
                    "/usr/lib/jvm/java-17-openjdk/bin/java",
                    "/usr/lib/jvm/java-17-openjdk-amd64/bin/java",
                    "/usr/lib/jvm/temurin-17-jdk/bin/java",
                    "/usr/lib/jvm/zulu-17/bin/java",
                ],
                21 => &[
                    "/usr/lib/jvm/java-21-openjdk/bin/java",
                    "/usr/lib/jvm/java-21-openjdk-amd64/bin/java",
                    "/usr/lib/jvm/temurin-21-jdk/bin/java",
                    "/usr/lib/jvm/zulu-21/bin/java",
                ],
                _ => &[],
            };

            for p in specific_linux_paths {
                let pb = PathBuf::from(p);
                if pb.exists() && check_binary(&pb, major) {
                    return Ok(pb);
                }
            }

            if let Ok(path_out) = crate::core::process::std_command("which").arg("java").output() {
                let path_str = String::from_utf8_lossy(&path_out.stdout).trim().to_string();
                if !path_str.is_empty() {
                    let pb = PathBuf::from(path_str);
                    if pb.exists() && check_binary(&pb, major) {
                        return Ok(pb);
                    }
                }
            }
        }

        Err(AppError::Internal(format!(
            "Compatible system Java {} not found",
            major
        )))
    }

    async fn download_runtime(&self, major_version: u32) -> AppResult<()> {
        self.emit_progress(&DownloadProgress {
            phase: "java".into(),
            total: 1,
            completed: 0,
            current_file: format!("java-{}", major_version),
            bytes_downloaded: 0,
            total_bytes: 0,
            speed: None,
        });

        let manifest: std::collections::HashMap<String, std::collections::HashMap<String, Vec<RuntimeEntry>>> =
            retry_get_json(&self.http, JAVA_RUNTIME_MANIFEST_URL).await?;

        let platform_key = current_mojang_platform_key();
        let platform_entries = manifest
            .get(platform_key)
            .ok_or_else(|| AppError::Internal(format!("no java runtimes available for platform {}", platform_key)))?;

        let component_url = self.find_component_url(platform_entries, major_version)?;

        let component_manifest: ComponentManifest =
            retry_get_json(&self.http, &component_url).await?;

        let java_dir = self.java_dir(major_version);
        tokio::fs::create_dir_all(&java_dir).await?;

        let files = component_manifest
            .files
            .ok_or_else(|| AppError::Internal("java runtime manifest has no files".into()))?;

        let total = files.len() as u64;
        let completed = std::sync::Arc::new(std::sync::atomic::AtomicU64::new(0));

        self.emit_log(&format!(
            "Downloading Java {} runtime ({} files)",
            major_version, total
        ));

        stream::iter(files.into_iter().map(|(path, file): (String, ComponentFile)| {
            let java_dir = java_dir.clone();
            let completed = completed.clone();
            async move {
            if !safe_runtime_path(&path) { return Err(AppError::InvalidInput("Caminho inválido no runtime Java".into())); }
            if file.file_type.as_deref() == Some("link") {
                #[cfg(unix)]
                if let Some(target) = file.target.as_deref() {
                    let file_path = java_dir.join(&path);
                    if let Some(parent) = file_path.parent() {
                        tokio::fs::create_dir_all(parent).await?;
                    }
                    let _ = tokio::fs::remove_file(&file_path).await;
                    let _ = std::os::unix::fs::symlink(target, &file_path);
                }
                return Ok(());
            }

            if file.file_type.as_deref() != Some("file") {
                return Ok(());
            }

            let downloads = match file.downloads {
                Some(ref d) => d,
                None => return Ok(()),
            };

            let download = downloads.raw.as_ref();

            let url = match download {
                Some(ref d) => d.url.as_ref(),
                None => return Err(AppError::Internal("Runtime Java sem arquivo raw verificável".into())),
            };

            let url = match url {
                Some(u) => u,
                None => return Err(AppError::Internal("Runtime Java sem URL de download".into())),
            };

            let file_path = java_dir.join(&path);
            if let Some(parent) = file_path.parent() {
                tokio::fs::create_dir_all(parent).await?;
            }

            self.emit_progress(&DownloadProgress {
                phase: "java".into(),
                total,
                completed: completed.load(std::sync::atomic::Ordering::Relaxed),
                current_file: path.clone(),
                bytes_downloaded: 0,
                total_bytes: 0,
                speed: None,
            });

            let valid = |bytes: &[u8]| download.as_ref().is_some_and(|entry| entry.sha1.as_ref().is_some_and(|expected| format!("{:x}", sha1::Sha1::digest(bytes)) == expected.to_ascii_lowercase()) && entry.size.is_none_or(|size| size == bytes.len() as u64));
            if tokio::fs::read(&file_path).await.is_ok_and(|bytes| valid(&bytes)) {
                completed.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                return Ok(());
            }
            match retry_download_bytes(&self.http, url).await {
                Ok(bytes) => {
                    if !valid(&bytes) { return Err(AppError::Internal(format!("Integridade do Java inválida: {path}"))); }
                    let temporary = file_path.with_file_name(format!("{}.{}.part", file_path.file_name().unwrap().to_string_lossy(), uuid::Uuid::new_v4()));
                    tokio::fs::write(&temporary, &bytes).await?;
                    if file_path.exists() { tokio::fs::remove_file(&file_path).await?; }
                    tokio::fs::rename(temporary, &file_path).await?;
                    if file.executable == Some(true) {
                        #[cfg(unix)]
                        {
                            use std::os::unix::fs::PermissionsExt;
                            let perms = std::fs::Permissions::from_mode(0o755);
                            tokio::fs::set_permissions(&file_path, perms).await?;
                        }
                    }
                }
                Err(e) => {
                    self.emit_log(&format!("Failed to download Java file {}: {}", path, e));
                    return Err(e);
                }
            }

            completed.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
            Ok::<(), AppError>(())
        }})).buffer_unordered(crate::core::downloader::max_concurrent_downloads().min(12)).try_collect::<Vec<_>>().await?;

        if !self.verify_binary(&self.java_bin(major_version), major_version) { return Err(AppError::Internal("Java baixado não executa corretamente neste sistema".into())); }

        let _ = tokio::fs::write(java_dir.join(".installed"), b"ok").await;
        self.emit_log(&format!("Java {} runtime download complete", major_version));
        self.emit_progress(&DownloadProgress {
            phase: "java".into(),
            total,
            completed: total,
            current_file: String::new(),
            bytes_downloaded: 0,
            total_bytes: 0,
            speed: None,
        });

        Ok(())
    }

    async fn download_temurin(&self, major: u32) -> AppResult<()> {
        if !cfg!(windows) { return Err(AppError::Internal("Escolha um Java do sistema nas configurações".into())); }
        let arch = if cfg!(target_arch = "aarch64") { "aarch64" } else if cfg!(target_arch = "x86") { "x32" } else { "x64" };
        let assets: serde_json::Value = retry_get_json(&self.http, &format!("https://api.adoptium.net/v3/assets/latest/{major}/hotspot?architecture={arch}&image_type=jre&os=windows&vendor=eclipse")).await?;
        let package = assets.as_array().and_then(|items| items.first()).and_then(|asset| asset.get("binary")).and_then(|binary| binary.get("package")).ok_or_else(|| AppError::NotFound("Temurin não oferece esse runtime para a arquitetura deste PC".into()))?;
        let url = package.get("link").and_then(|value| value.as_str()).filter(|url| url.starts_with("https://github.com/adoptium/")).ok_or_else(|| AppError::InvalidInput("Origem do Java não reconhecida".into()))?;
        let expected = package.get("checksum").and_then(|value| value.as_str()).ok_or_else(|| AppError::InvalidInput("Checksum Java ausente".into()))?;
        let bytes = retry_download_bytes(&self.http, url).await?;
        if format!("{:x}", sha2::Sha256::digest(&bytes)) != expected.to_ascii_lowercase() { return Err(AppError::InvalidInput("Integridade do Temurin inválida".into())); }
        let destination = self.java_dir(major);
        tokio::task::spawn_blocking(move || -> AppResult<()> {
            let mut zip = zip::ZipArchive::new(std::io::Cursor::new(bytes)).map_err(|error| AppError::Internal(error.to_string()))?;
            for index in 0..zip.len() {
                let mut entry = zip.by_index(index).map_err(|error| AppError::Internal(error.to_string()))?;
                let path = entry.enclosed_name().ok_or_else(|| AppError::InvalidInput("Caminho inválido no Temurin".into()))?.to_owned();
                let relative: PathBuf = path.components().skip(1).collect();
                if relative.as_os_str().is_empty() { continue; }
                let target = destination.join(relative);
                if entry.is_dir() { std::fs::create_dir_all(&target)?; }
                else { if let Some(parent) = target.parent() { std::fs::create_dir_all(parent)?; } std::io::copy(&mut entry, &mut std::fs::File::create(target)?)?; }
            }
            Ok(())
        }).await.map_err(|error| AppError::Internal(error.to_string()))??;
        if !self.verify_binary(&self.java_bin(major), major) { return Err(AppError::Internal("O runtime foi verificado, mas o Windows não conseguiu executá-lo. Confira componentes removidos do Windows e a quarentena do antivírus.".into())); }
        tokio::fs::write(self.java_dir(major).join(".installed"), b"verified").await?;
        self.emit_progress(&DownloadProgress { phase: "java".into(), total: 1, completed: 1, current_file: String::new(), bytes_downloaded: 0, total_bytes: 0, speed: None });
        Ok(())
    }

    fn find_component_url(
        &self,
        platform_entries: &std::collections::HashMap<String, Vec<RuntimeEntry>>,
        major_version: u32,
    ) -> AppResult<String> {
        let target_component_keys: &[&str] = match major_version {
            8 => &["jre-legacy"],
            16 => &["java-runtime-alpha"],
            17 => &["java-runtime-gamma", "java-runtime-beta"],
            21 => &["java-runtime-delta"],
            25 => &["java-runtime-epsilon"],
            _ => &[],
        };

        for target_key in target_component_keys {
            if let Some(entries) = platform_entries.get(*target_key) {
                for entry in entries {
                    if let Some(ref manifest_ref) = entry.manifest {
                        if let Some(ref url) = manifest_ref.url {
                            self.emit_log(&format!(
                                "Found Java {} runtime component: {}",
                                major_version, target_key
                            ));
                            return Ok(url.clone());
                        }
                    }
                }
            }
        }

        for (name, entries) in platform_entries {
            for entry in entries {
                if let Some(ref version) = entry.version {
                    let entry_major = version
                        .name
                        .as_ref()
                        .and_then(|n| parse_major_from_version_name(n));

                    if entry_major == Some(major_version) || (major_version == 8 && name == "jre-legacy") {
                        if let Some(ref manifest_ref) = entry.manifest {
                            if let Some(ref url) = manifest_ref.url {
                                self.emit_log(&format!(
                                    "Found Java {} runtime component: {}",
                                    major_version,
                                    version.component.as_deref().unwrap_or(name.as_str())
                                ));
                                return Ok(url.clone());
                            }
                        }
                    }
                }
            }
        }

        let hint = if cfg!(windows) {
            format!("Install Java {} manually from https://adoptium.net/", major_version)
        } else {
            format!("Install Java {} manually with: sudo pacman -S jre-openjdk", major_version)
        };
        Err(AppError::Internal(format!(
            "No Java runtime found for major version {}. {}",
            major_version, hint
        )))
    }
}

fn safe_runtime_path(path: &str) -> bool {
    !path.is_empty() && !path.contains(':') && !path.contains('\\') && std::path::Path::new(path).components().all(|part| matches!(part, std::path::Component::Normal(_)))
}

fn verified_java(path: &std::path::Path, expected_major: u32) -> bool {
    type Fingerprint = Vec<(u64, std::time::SystemTime)>;
    static VERIFIED: std::sync::LazyLock<std::sync::Mutex<std::collections::HashMap<(PathBuf, u32), Fingerprint>>> = std::sync::LazyLock::new(|| std::sync::Mutex::new(std::collections::HashMap::new()));
    let root = path.parent().and_then(std::path::Path::parent);
    let fingerprint = root.and_then(|root| {
        let vm = ["bin/server/jvm.dll", "bin/client/jvm.dll", "lib/server/libjvm.so", "lib/amd64/server/libjvm.so", "lib/server/libjvm.dylib"].iter().map(|name| root.join(name)).find(|file| file.is_file())?;
        let mut files = vec![path.to_owned(), vm];
        for name in ["bin/java.dll", "bin/zip.dll", "lib/rt.jar", "lib/modules"] { let file = root.join(name); if file.is_file() { files.push(file); } }
        files.into_iter().map(|file| std::fs::metadata(file).ok().and_then(|metadata| Some((metadata.len(), metadata.modified().ok()?)))).collect::<Option<Fingerprint>>()
    });
    let key = (path.to_owned(), expected_major);
    if let Some(current) = &fingerprint { if VERIFIED.lock().is_ok_and(|cache| cache.get(&key) == Some(current)) { return true; } }
    let Ok(mut child) = crate::core::process::std_command(path).arg("-version").stdout(std::process::Stdio::piped()).stderr(std::process::Stdio::piped()).spawn() else { return false; };
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(20);
    loop {
        match child.try_wait() {
            Ok(Some(_)) => break,
            Ok(None) if std::time::Instant::now() < deadline => std::thread::sleep(std::time::Duration::from_millis(15)),
            _ => { let _ = child.kill(); let _ = child.wait(); return false; }
        }
    }
    let Ok(output) = child.wait_with_output() else { return false; };
    if !output.status.success() { return false; }
    let text = format!("{} {}", String::from_utf8_lossy(&output.stderr), String::from_utf8_lossy(&output.stdout));
    let version = text.split('"').nth(1).unwrap_or("");
    let valid = !version.is_empty() && parse_java_major(version) == expected_major;
    if valid { if let Some(fingerprint) = fingerprint { if let Ok(mut cache) = VERIFIED.lock() { if cache.len() > 128 { cache.clear(); } cache.insert(key, fingerprint); } } }
    valid
}

fn parse_java_major(version_str: &str) -> u32 {
    let cleaned: String = version_str
        .chars()
        .take_while(|c| c.is_ascii_digit() || *c == '.')
        .collect();
    let parts: Vec<&str> = cleaned.split('.').collect();

    match parts.as_slice() {
        [major, ..] if *major == "1" && parts.len() >= 2 => {
            parts.get(1).and_then(|s| s.parse().ok()).unwrap_or(8)
        }
        [major, ..] => major.parse().unwrap_or(8),
        _ => 8,
    }
}

fn parse_major_from_version_name(name: &str) -> Option<u32> {
    let first_part: String = name.chars().take_while(|c| c.is_ascii_digit()).collect();
    first_part.parse().ok()
}

async fn retry_get_json<T: serde::de::DeserializeOwned>(
    http: &reqwest::Client,
    url: &str,
) -> AppResult<T> {
    let mut last_err = None;

    for attempt in 1..=MAX_RETRIES {
        match http.get(url).timeout(std::time::Duration::from_secs(30)).send().await {
            Ok(resp) => match resp.error_for_status() {
                Ok(validated) => match validated.json::<T>().await {
                    Ok(val) => return Ok(val),
                    Err(e) => {
                        last_err = Some(e.into());
                    }
                },
                Err(e) => {
                    last_err = Some(e.into());
                }
            },
            Err(e) => {
                last_err = Some(e.into());
            }
        }

        if attempt < MAX_RETRIES {
            let delay = RETRY_BASE_DELAY_MS * 2u64.pow(attempt - 1);
            tokio::time::sleep(tokio::time::Duration::from_millis(delay)).await;
        }
    }

    Err(last_err.unwrap_or_else(|| AppError::Internal("request failed after retries".into())))
}

async fn retry_download_bytes(http: &reqwest::Client, url: &str) -> AppResult<Vec<u8>> {
    let mut last_err = None;

    for attempt in 1..=MAX_RETRIES {
        match http.get(url).timeout(std::time::Duration::from_secs(120)).send().await {
            Ok(resp) => match resp.error_for_status() {
                Ok(validated) => match validated.bytes().await {
                    Ok(bytes) => return Ok(bytes.to_vec()),
                    Err(e) => {
                        last_err = Some(e.into());
                    }
                },
                Err(e) => {
                    last_err = Some(e.into());
                }
            },
            Err(e) => {
                last_err = Some(e.into());
            }
        }

        if attempt < MAX_RETRIES {
            let delay = RETRY_BASE_DELAY_MS * 2u64.pow(attempt - 1);
            tokio::time::sleep(tokio::time::Duration::from_millis(delay)).await;
        }
    }

    Err(last_err.unwrap_or_else(|| AppError::Internal("download failed after retries".into())))
}

#[cfg(test)]
mod runtime_tests {
    use super::*;
    #[test]
    fn runtime_paths_and_major_versions_are_validated() {
        for path in ["bin/java.exe", "lib/amd64/server/jvm.dll"] { assert!(safe_runtime_path(path)); }
        for path in ["../java.exe", "C:/java.exe", "/bin/java", "bin/../java", "bin\\java.exe"] { assert!(!safe_runtime_path(path)); }
        assert_eq!(parse_java_major("1.8.0_452"), 8);
        assert_eq!(parse_java_major("21.0.7"), 21);
    }
    #[tokio::test]
    #[ignore]
    async fn downloads_and_runs_verified_java_8_from_both_providers() {
        let temporary = tempfile::tempdir().unwrap();
        let client = reqwest::Client::builder().user_agent("Luxmc-runtime-verification").build().unwrap();
        let mojang = JavaRuntimeManager::new(client.clone(), temporary.path().join("mojang"));
        let started = std::time::Instant::now();
        mojang.download_runtime(8).await.unwrap();
        assert!(mojang.verify_binary(&mojang.java_bin(8), 8));
        println!("Mojang Java 8 verified in {:.2}s", started.elapsed().as_secs_f64());
        let temurin = JavaRuntimeManager::new(client, temporary.path().join("temurin"));
        let started = std::time::Instant::now();
        if let Err(error) = temurin.download_temurin(8).await {
            let diagnostic = crate::core::process::std_command(temurin.java_bin(8)).arg("-version").output();
            panic!("Temurin preparation failed: {error}; execution: {diagnostic:?}");
        }
        assert!(temurin.verify_binary(&temurin.java_bin(8), 8));
        println!("Temurin Java 8 verified in {:.2}s", started.elapsed().as_secs_f64());
    }
}
