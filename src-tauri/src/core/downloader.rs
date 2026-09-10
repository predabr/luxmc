use serde::Serialize;
use sha1::{Digest, Sha1};
use std::path::PathBuf;
use std::sync::Arc;
use tauri::Emitter;
use tokio::sync::Semaphore;
use tokio::task::JoinSet;
use tokio::time::Instant;

use crate::core::launcher::lib_path_from_name;
use crate::core::minecraft::{lib_url_from_name, DownloadEntry, Library, VersionDetail};
use crate::error::{AppError, AppResult};

const MAX_RETRIES: u32 = 3;
const RETRY_BASE_DELAY_MS: u64 = 1000;
const MAX_CONCURRENT_DOWNLOADS: usize = 8;
const SPEED_UPDATE_INTERVAL_MS: u64 = 250;

pub struct DownloadManager {
    http: reqwest::Client,
    base_dir: PathBuf,
    app: Option<tauri::AppHandle>,
    semaphore: Arc<Semaphore>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DownloadSpeed {
    pub bytes_per_second: u64,
    pub total_downloaded: u64,
    pub elapsed_ms: u64,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DownloadProgress {
    pub phase: String,
    pub total: u64,
    pub completed: u64,
    pub current_file: String,
    pub bytes_downloaded: u64,
    pub total_bytes: u64,
    pub speed: Option<DownloadSpeed>,
}

impl DownloadManager {
    pub fn new(http: reqwest::Client, base_dir: PathBuf) -> Self {
        Self {
            http,
            base_dir,
            app: None,
            semaphore: Arc::new(Semaphore::new(MAX_CONCURRENT_DOWNLOADS)),
        }
    }

    pub fn with_app(mut self, app: tauri::AppHandle) -> Self {
        self.app = Some(app);
        self
    }

    pub fn http(&self) -> &reqwest::Client {
        &self.http
    }

    pub fn libraries_dir(&self) -> PathBuf {
        self.base_dir.join("libraries")
    }

    pub fn assets_dir(&self) -> PathBuf {
        self.base_dir.join("assets")
    }

    pub fn versions_dir(&self) -> PathBuf {
        self.base_dir.join("versions")
    }

    fn emit_progress(mgr: &DownloadManager, progress: &DownloadProgress) {
        if let Some(ref app) = mgr.app {
            let _ = app.emit("download-progress", progress);
        }
    }

    fn emit_log(mgr: &DownloadManager, message: &str) {
        if let Some(ref app) = mgr.app {
            let _ = app.emit("launcher-log", message);
        }
        tracing::info!(target: "download", "{}", message);
    }

    pub async fn download_version(&self, detail: &VersionDetail) -> AppResult<()> {
        Self::emit_log(
            self,
            &format!("Starting download for version {}", detail.id),
        );

        if let Some(ref downloads) = detail.downloads {
            let version_dir = self.versions_dir().join(&detail.id);
            tokio::fs::create_dir_all(&version_dir).await?;

            let client_jar = version_dir.join(format!("{}.jar", detail.id));
            let already = client_jar.exists();
            Self::emit_progress(
                self,
                &DownloadProgress {
                    phase: "client".into(),
                    total: 1,
                    completed: if already { 1 } else { 0 },
                    current_file: format!("{}.jar", detail.id),
                    bytes_downloaded: 0,
                    total_bytes: downloads.client.size,
                    speed: None,
                },
            );
            if !already {
                Self::emit_log(self, &format!("Downloading client jar: {}", detail.id));
                self.download_client_with_retry(downloads, &detail.id, &version_dir)
                    .await?;
            } else {
                Self::emit_log(self, &format!("Client jar already exists: {}", detail.id));
            }
            Self::emit_progress(
                self,
                &DownloadProgress {
                    phase: "client".into(),
                    total: 1,
                    completed: 1,
                    current_file: format!("{}.jar", detail.id),
                    bytes_downloaded: downloads.client.size,
                    total_bytes: downloads.client.size,
                    speed: None,
                },
            );
        }

        if let Some(ref asset_index) = detail.asset_index {
            self.download_assets(asset_index).await?;
        }

        self.download_libraries(&detail.libraries).await?;

        Self::emit_log(self, "Download complete");
        Self::emit_progress(
            self,
            &DownloadProgress {
                phase: "done".into(),
                total: 1,
                completed: 1,
                current_file: String::new(),
                bytes_downloaded: 0,
                total_bytes: 0,
                speed: None,
            },
        );

        Ok(())
    }

    async fn download_client_with_retry(
        &self,
        downloads: &crate::core::minecraft::VersionDownloads,
        version_id: &str,
        version_dir: &PathBuf,
    ) -> AppResult<()> {
        let client_path = version_dir.join(format!("{}.jar", version_id));
        let client_filename = format!("{}.jar", version_id);
        let entry = DownloadEntry {
            url: downloads.client.url.clone(),
            size: downloads.client.size,
            sha1: downloads.client.sha1.clone(),
        };
        download_file_retry(self, &entry, &client_path, &client_filename).await
    }

    async fn download_assets(
        &self,
        asset_index: &crate::core::minecraft::AssetIndex,
    ) -> AppResult<()> {
        let assets_dir = self.assets_dir();
        let indexes_dir = assets_dir.join("indexes");
        tokio::fs::create_dir_all(&indexes_dir).await?;

        let index_path = indexes_dir.join(format!("{}.json", asset_index.id));
        if !index_path.exists() {
            Self::emit_log(
                self,
                &format!("Downloading asset index: {}", asset_index.id),
            );
            Self::emit_progress(
                self,
                &DownloadProgress {
                    phase: "asset_index".into(),
                    total: 1,
                    completed: 0,
                    current_file: format!("{}.json", asset_index.id),
                    bytes_downloaded: 0,
                    total_bytes: asset_index.size,
                    speed: None,
                },
            );

            let entry = DownloadEntry {
                url: asset_index.url.clone(),
                size: asset_index.size,
                sha1: String::new(),
            };
            download_file_retry(self, &entry, &index_path, &format!("{}.json", asset_index.id)).await?;

            Self::emit_progress(
                self,
                &DownloadProgress {
                    phase: "asset_index".into(),
                    total: 1,
                    completed: 1,
                    current_file: format!("{}.json", asset_index.id),
                    bytes_downloaded: asset_index.size,
                    total_bytes: asset_index.size,
                    speed: None,
                },
            );
        } else {
            Self::emit_log(
                self,
                &format!("Asset index already exists: {}", asset_index.id),
            );
        }

        let objects_dir = assets_dir.join("objects");
        tokio::fs::create_dir_all(&objects_dir).await?;

        let index_content = tokio::fs::read_to_string(&index_path).await?;
        let index: serde_json::Value = serde_json::from_str(&index_content)?;

        let mut to_download: Vec<(String, u64, String)> = Vec::new();
        if let Some(objects) = index.get("objects") {
            if let Some(map) = objects.as_object() {
                for (_name, obj) in map {
                    if let Some(hash) = obj.get("hash").and_then(|h| h.as_str()) {
                        let size = obj.get("size").and_then(|s| s.as_u64()).unwrap_or(0);
                        let prefix = &hash[..2];
                        let object_dir = objects_dir.join(prefix);
                        let object_path = object_dir.join(hash);
                        if !object_path.exists() {
                            tokio::fs::create_dir_all(&object_dir).await?;
                            let url =
                                format!("https://resources.download.minecraft.net/{prefix}/{hash}");
                            to_download.push((url, size, hash.to_string()));
                        }
                    }
                }
            }
        }

        let total = to_download.len() as u64;
        let total_bytes: u64 = to_download.iter().map(|(_, s, _)| s).sum();
        Self::emit_log(
            self,
            &format!("Downloading {} assets ({} bytes)", total, total_bytes),
        );

        let mgr = Arc::new(self.clone_for_parallel());
        let mut completed = 0u64;
        let mut bytes_so_far = 0u64;
        let start_time = Instant::now();
        let mut join_set: JoinSet<(u64, Result<(), String>)> = JoinSet::new();

        if total > 0 {
            Self::emit_progress(
                &mgr,
                &DownloadProgress {
                    phase: "assets".into(),
                    total,
                    completed: 0,
                    current_file: "Iniciando download dos assets...".into(),
                    bytes_downloaded: 0,
                    total_bytes,
                    speed: None,
                },
            );
        }

        for (url, size, hash) in to_download {
            let permit = mgr.semaphore.clone().acquire_owned().await.unwrap();
            let entry = DownloadEntry {
                url: url.clone(),
                size,
                sha1: hash.clone(),
            };
            let path = objects_dir.join(&hash[..2]).join(&hash);
            let mgr_clone = mgr.clone();
            let label = hash.clone();

            join_set.spawn(async move {
                let result = download_file_retry(&mgr_clone, &entry, &path, &label).await;
                drop(permit);
                match result {
                    Ok(()) => (size, Ok(())),
                    Err(e) => (0, Err(e.to_string())),
                }
            });

            while join_set.len() >= MAX_CONCURRENT_DOWNLOADS {
                if let Some(result) = join_set.join_next().await {
                    match result {
                        Ok((dl_bytes, Ok(()))) => {
                            completed += 1;
                            bytes_so_far += dl_bytes;
                            let elapsed = start_time.elapsed().as_millis() as u64;
                            let bps = if elapsed > 0 {
                                bytes_so_far * 1000 / elapsed
                            } else {
                                0
                            };
                            Self::emit_progress(
                                &mgr,
                                &DownloadProgress {
                                    phase: "assets".into(),
                                    total,
                                    completed,
                                    current_file: format!("asset {:04}/{}", completed, total),
                                    bytes_downloaded: bytes_so_far,
                                    total_bytes,
                                    speed: Some(DownloadSpeed {
                                        bytes_per_second: bps,
                                        total_downloaded: bytes_so_far,
                                        elapsed_ms: elapsed,
                                    }),
                                },
                            );
                        }
                        Ok((_, Err(e))) => {
                            join_set.abort_all();
                            return Err(AppError::Internal(format!("asset download failed: {}", e)));
                        }
                        Err(e) => {
                            join_set.abort_all();
                            return Err(AppError::Internal(format!("asset task failed: {}", e)));
                        }
                    }
                }
            }
        }

        while let Some(result) = join_set.join_next().await {
            match result {
                Ok((dl_bytes, Ok(()))) => {
                    completed += 1;
                    bytes_so_far += dl_bytes;
                    let elapsed = start_time.elapsed().as_millis() as u64;
                    let bps = if elapsed > 0 {
                        bytes_so_far * 1000 / elapsed
                    } else {
                        0
                    };
                    Self::emit_progress(
                        &mgr,
                        &DownloadProgress {
                            phase: "assets".into(),
                            total,
                            completed,
                            current_file: format!("asset {:04}/{}", completed, total),
                            bytes_downloaded: bytes_so_far,
                            total_bytes,
                            speed: Some(DownloadSpeed {
                                bytes_per_second: bps,
                                total_downloaded: bytes_so_far,
                                elapsed_ms: elapsed,
                            }),
                        },
                    );
                }
                Ok((_, Err(e))) => {
                    join_set.abort_all();
                    return Err(AppError::Internal(format!("asset download failed: {}", e)));
                }
                Err(e) => {
                    join_set.abort_all();
                    return Err(AppError::Internal(format!("asset task failed: {}", e)));
                }
            }
        }

        Self::emit_progress(
            &mgr,
            &DownloadProgress {
                phase: "assets".into(),
                total,
                completed: total,
                current_file: String::new(),
                bytes_downloaded: total_bytes,
                total_bytes,
                speed: None,
            },
        );

        Ok(())
    }

    async fn download_libraries(&self, libraries: &[Library]) -> AppResult<()> {
        let lib_dir = self.libraries_dir();
        tokio::fs::create_dir_all(&lib_dir).await?;

        let mut to_download: Vec<(String, PathBuf, String, String, u64)> = Vec::new();

        for lib in libraries {
            if !crate::core::launcher::is_library_allowed(lib) {
                continue;
            }
            let path = lib_path_from_name(&lib_dir, &lib.name);
            if path.exists() {
                continue;
            }

            if let Some(ref downloads) = lib.downloads {
                if let Some(ref artifact) = downloads.artifact {
                    if let Some(parent) = path.parent() {
                        tokio::fs::create_dir_all(parent).await?;
                    }
                    to_download.push((
                        artifact.url.clone(),
                        path.clone(),
                        artifact.sha1.clone(),
                        lib.name.clone(),
                        artifact.size,
                    ));
                    continue;
                }
            }

            let url = lib_url_from_name(&lib.name, lib.url.as_deref());
            if !url.is_empty() {
                if let Some(parent) = path.parent() {
                    tokio::fs::create_dir_all(parent).await?;
                }
                to_download.push((url, path, String::new(), lib.name.clone(), 0));
            }
        }

        let total = to_download.len() as u64;
        let total_bytes: u64 = to_download.iter().map(|(_, _, _, _, s)| *s).sum();
        Self::emit_log(self, &format!("Downloading {} libraries ({} bytes)", total, total_bytes));

        let mgr = Arc::new(self.clone_for_parallel());
        let mut completed = 0u64;
        let mut bytes_so_far = 0u64;
        let start_time = Instant::now();
        let mut join_set: JoinSet<(String, u64, Result<(), String>)> = JoinSet::new();

        if total > 0 {
            Self::emit_progress(
                &mgr,
                &DownloadProgress {
                    phase: "libraries".into(),
                    total,
                    completed: 0,
                    current_file: "Verificando bibliotecas...".into(),
                    bytes_downloaded: 0,
                    total_bytes,
                    speed: None,
                },
            );
        }

        for (url, path, sha1, name, size) in to_download {
            let permit = mgr.semaphore.clone().acquire_owned().await.unwrap();
            let entry = DownloadEntry {
                url: url.clone(),
                size,
                sha1: sha1.clone(),
            };
            let path = path.clone();
            let mgr_clone = mgr.clone();
            let label = name.clone();

            join_set.spawn(async move {
                let result = download_file_retry(&mgr_clone, &entry, &path, &label).await;
                drop(permit);
                match result {
                    Ok(()) => (label, size, Ok(())),
                    Err(e) => (label, 0, Err(e.to_string())),
                }
            });

            while join_set.len() >= MAX_CONCURRENT_DOWNLOADS {
                if let Some(result) = join_set.join_next().await {
                    match result {
                        Ok((lib_name, dl_bytes, Ok(()))) => {
                            completed += 1;
                            bytes_so_far += dl_bytes;
                            let elapsed = start_time.elapsed().as_millis() as u64;
                            let bps = if elapsed > 0 {
                                bytes_so_far * 1000 / elapsed
                            } else {
                                0
                            };
                            Self::emit_progress(
                                &mgr,
                                &DownloadProgress {
                                    phase: "libraries".into(),
                                    total,
                                    completed,
                                    current_file: lib_name,
                                    bytes_downloaded: bytes_so_far,
                                    total_bytes,
                                    speed: Some(DownloadSpeed {
                                        bytes_per_second: bps,
                                        total_downloaded: bytes_so_far,
                                        elapsed_ms: elapsed,
                                    }),
                                },
                            );
                        }
                        Ok((_, _, Err(e))) => {
                            join_set.abort_all();
                            return Err(AppError::Internal(format!(
                                "library download failed: {}",
                                e
                            )));
                        }
                        Err(e) => {
                            join_set.abort_all();
                            return Err(AppError::Internal(format!("library task failed: {}", e)));
                        }
                    }
                }
            }
        }

        while let Some(result) = join_set.join_next().await {
            match result {
                Ok((lib_name, dl_bytes, Ok(()))) => {
                    completed += 1;
                    bytes_so_far += dl_bytes;
                    let elapsed = start_time.elapsed().as_millis() as u64;
                    let bps = if elapsed > 0 {
                        bytes_so_far * 1000 / elapsed
                    } else {
                        0
                    };
                    Self::emit_progress(
                        &mgr,
                        &DownloadProgress {
                            phase: "libraries".into(),
                            total,
                            completed,
                            current_file: lib_name,
                            bytes_downloaded: bytes_so_far,
                            total_bytes,
                            speed: Some(DownloadSpeed {
                                bytes_per_second: bps,
                                total_downloaded: bytes_so_far,
                                elapsed_ms: elapsed,
                            }),
                        },
                    );
                }
                Ok((_, _, Err(e))) => {
                    join_set.abort_all();
                    return Err(AppError::Internal(format!(
                        "library download failed: {}",
                        e
                    )));
                }
                Err(e) => {
                    join_set.abort_all();
                    return Err(AppError::Internal(format!("library task failed: {}", e)));
                }
            }
        }

        Self::emit_progress(
            &mgr,
            &DownloadProgress {
                phase: "libraries".into(),
                total,
                completed: total,
                current_file: String::new(),
                bytes_downloaded: total_bytes,
                total_bytes,
                speed: None,
            },
        );

        Ok(())
    }

    fn clone_for_parallel(&self) -> Self {
        Self {
            http: self.http.clone(),
            base_dir: self.base_dir.clone(),
            app: self.app.clone(),
            semaphore: self.semaphore.clone(),
        }
    }

    pub async fn validate_version(&self, detail: &VersionDetail) -> AppResult<()> {
        if let Some(ref downloads) = detail.downloads {
            let client_path = self
                .versions_dir()
                .join(&detail.id)
                .join(format!("{}.jar", detail.id));
            if !client_path.exists() {
                return Err(AppError::NotFound(format!(
                    "client jar for {} not downloaded",
                    detail.id
                )));
            }
            if !downloads.client.sha1.is_empty() {
                let bytes = tokio::fs::read(&client_path).await?;
                let mut hasher = Sha1::new();
                hasher.update(&bytes);
                let computed = format!("{:x}", hasher.finalize());
                if computed != downloads.client.sha1 {
                    return Err(AppError::Internal(format!(
                        "client jar sha1 mismatch for {}",
                        detail.id
                    )));
                }
            }
        }

        let lib_dir = self.libraries_dir();
        for lib in &detail.libraries {
            if !crate::core::launcher::is_library_allowed(lib) {
                continue;
            }
            let path = lib_path_from_name(&lib_dir, &lib.name);
            if !path.exists() {
                return Err(AppError::NotFound(format!(
                    "library {} not downloaded",
                    lib.name
                )));
            }
        }

        if let Some(ref asset_index) = detail.asset_index {
            let index_path = self
                .assets_dir()
                .join("indexes")
                .join(format!("{}.json", asset_index.id));
            if !index_path.exists() {
                return Err(AppError::NotFound(format!(
                    "asset index {} not downloaded",
                    asset_index.id
                )));
            }
        }

        Ok(())
    }
}

async fn download_file_retry(
    mgr: &DownloadManager,
    entry: &DownloadEntry,
    path: &PathBuf,
    label: &str,
) -> AppResult<()> {
    let mut last_err = None;

    for attempt in 1..=MAX_RETRIES {
        match download_file_once(mgr, entry, path, label).await {
            Ok(()) => {
                if attempt > 1 {
                    DownloadManager::emit_log(
                        mgr,
                        &format!("Download succeeded on attempt {} for {}", attempt, label),
                    );
                }
                return Ok(());
            }
            Err(e) => {
                DownloadManager::emit_log(
                    mgr,
                    &format!(
                        "Download attempt {}/{} failed for {}: {}",
                        attempt, MAX_RETRIES, label, e
                    ),
                );
                last_err = Some(e);

                if attempt < MAX_RETRIES {
                    let delay = RETRY_BASE_DELAY_MS * 2u64.pow(attempt - 1);
                    DownloadManager::emit_log(mgr, &format!("Retrying in {} ms...", delay));
                    tokio::time::sleep(tokio::time::Duration::from_millis(delay)).await;
                }
            }
        }
    }

    Err(last_err.unwrap_or_else(|| AppError::Internal(format!("download failed for {}", label))))
}

async fn download_file_once(
    mgr: &DownloadManager,
    entry: &DownloadEntry,
    path: &PathBuf,
    label: &str,
) -> AppResult<()> {
    let resp = mgr.http.get(&entry.url).send().await?.error_for_status()?;
    let total_size = resp.content_length().unwrap_or(entry.size);

    let mut stream = resp.bytes_stream();
    let mut bytes_downloaded = 0u64;
    let mut buffer = Vec::new();
    let start = Instant::now();
    let mut last_report = Instant::now();

    use futures_util::StreamExt;

    while let Some(chunk) = stream.next().await {
        let chunk =
            chunk.map_err(|e| AppError::Internal(format!("stream error for {}: {}", label, e)))?;
        bytes_downloaded += chunk.len() as u64;
        buffer.extend_from_slice(&chunk);

        if last_report.elapsed().as_millis() >= SPEED_UPDATE_INTERVAL_MS as u128 {
            let elapsed = start.elapsed().as_millis() as u64;
            let bps = if elapsed > 0 {
                bytes_downloaded * 1000 / elapsed
            } else {
                0
            };
            DownloadManager::emit_progress(
                mgr,
                &DownloadProgress {
                    phase: "downloading".into(),
                    total: 1,
                    completed: 0,
                    current_file: label.to_string(),
                    bytes_downloaded,
                    total_bytes: total_size,
                    speed: Some(DownloadSpeed {
                        bytes_per_second: bps,
                        total_downloaded: bytes_downloaded,
                        elapsed_ms: elapsed,
                    }),
                },
            );
            last_report = Instant::now();
        }
    }

    tokio::fs::write(path, &buffer).await?;

    if !entry.sha1.is_empty() {
        let mut hasher = Sha1::new();
        hasher.update(&buffer);
        let computed = format!("{:x}", hasher.finalize());
        if computed != entry.sha1 {
            tokio::fs::remove_file(path).await?;
            return Err(AppError::Internal(format!(
                "sha1 mismatch for {}: expected {}, got {}",
                entry.url, entry.sha1, computed
            )));
        }
    }

    let elapsed = start.elapsed().as_millis() as u64;
    let bps = if elapsed > 0 {
        bytes_downloaded * 1000 / elapsed
    } else {
        0
    };
    DownloadManager::emit_progress(
        mgr,
        &DownloadProgress {
            phase: "downloading".into(),
            total: 1,
            completed: 1,
            current_file: label.to_string(),
            bytes_downloaded,
            total_bytes: total_size,
            speed: Some(DownloadSpeed {
                bytes_per_second: bps,
                total_downloaded: bytes_downloaded,
                elapsed_ms: elapsed,
            }),
        },
    );

    Ok(())
}
