use std::path::PathBuf;
use serde::Deserialize;
use tauri::Emitter;

use crate::error::{AppError, AppResult};
use crate::core::downloader::DownloadProgress;

const JAVA_RUNTIME_MANIFEST_URL: &str =
	"https://launchermeta.mojang.com/v1/products/java-runtime/2ec0cc96c44e5a76b9c8b7c39df7210883d12871/all.json";

const MAX_RETRIES: u32 = 3;
const RETRY_BASE_DELAY_MS: u64 = 1000;

#[derive(Debug, Deserialize)]
struct RuntimeManifest {
	linux: Option<std::collections::HashMap<String, Vec<RuntimeEntry>>>,
}

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
}

#[derive(Debug, Deserialize)]
struct ComponentFileDownloads {
	lzma: Option<ComponentDownload>,
	raw: Option<ComponentDownload>,
}

#[derive(Debug, Deserialize)]
struct ComponentDownload {
	url: Option<String>,
}

pub struct JavaRuntimeManager {
	http: reqwest::Client,
	data_dir: PathBuf,
	app: Option<tauri::AppHandle>,
}

impl JavaRuntimeManager {
	pub fn new(http: reqwest::Client, data_dir: PathBuf) -> Self {
		Self { http, data_dir, app: None }
	}

	pub fn with_app(mut self, app: tauri::AppHandle) -> Self {
		self.app = Some(app);
		self
	}

	fn java_dir(&self, major: u32) -> PathBuf {
		self.data_dir.join("java").join(major.to_string())
	}

	fn java_bin(&self, major: u32) -> PathBuf {
		self.java_dir(major).join("bin").join("java")
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

	pub async fn ensure_java(&self, major_version: u32) -> AppResult<PathBuf> {
		let bin = self.java_bin(major_version);
		if bin.exists() {
			self.emit_log(&format!("Java {} found at {}", major_version, bin.display()));
			return Ok(bin);
		}

		match self.find_system_java(major_version) {
			Ok(path) => {
				self.emit_log(&format!("Using system Java: {}", path.display()));
				Ok(path)
			}
			Err(_) => {
				self.emit_log(&format!(
					"System Java {} not found, will download runtime", major_version
				));
				self.download_runtime(major_version).await?;

				let bin = self.java_bin(major_version);
				if bin.exists() {
					Ok(bin)
				} else {
					Err(AppError::Internal(format!(
						"Failed to install Java {} runtime. Install Java {} manually with: sudo pacman -S jre-openjdk",
						major_version, major_version
					)))
				}
			}
		}
	}

	fn find_system_java(&self, major: u32) -> AppResult<PathBuf> {
		let output = std::process::Command::new("java")
			.arg("-version")
			.stderr(std::process::Stdio::piped())
			.output()
			.map_err(|_| AppError::Internal("java not found on PATH".into()))?;

		let stderr = String::from_utf8_lossy(&output.stderr);
		let version_str = stderr.split('"').nth(1).unwrap_or("");
		let detected = parse_java_major(version_str);

		if detected >= major {
			let path_out = std::process::Command::new("which")
				.arg("java")
				.output()
				.map_err(|_| AppError::Internal("which java failed".into()))?;
			let path = String::from_utf8_lossy(&path_out.stdout).trim().to_string();
			if !path.is_empty() {
				return Ok(PathBuf::from(path));
			}
		}

		Err(AppError::Internal(format!(
			"system java is version {}, need >= {}", detected, major
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

		let manifest: RuntimeManifest = retry_get_json(&self.http, JAVA_RUNTIME_MANIFEST_URL).await?;

		let linux = manifest.linux
			.ok_or_else(|| AppError::Internal("no linux java runtimes available".into()))?;

		let component_url = self.find_component_url(&linux, major_version)?;

		let component_manifest: ComponentManifest = retry_get_json(&self.http, &component_url).await?;

		let java_dir = self.java_dir(major_version);
		tokio::fs::create_dir_all(&java_dir).await?;

		let files = component_manifest.files
			.ok_or_else(|| AppError::Internal("java runtime manifest has no files".into()))?;

		let total = files.len() as u64;
		let mut completed = 0u64;

		self.emit_log(&format!("Downloading Java {} runtime ({} files)", major_version, total));

		for (path, file) in &files {
			if file.file_type.as_deref() != Some("file") {
				continue;
			}

			let downloads = match file.downloads {
				Some(ref d) => d,
				None => continue,
			};

			let download = downloads.raw.as_ref()
				.or(downloads.lzma.as_ref());

			let url = match download {
				Some(ref d) => d.url.as_ref(),
				None => continue,
			};

			let url = match url {
				Some(u) => u,
				None => continue,
			};

			let file_path = java_dir.join(path);
			if let Some(parent) = file_path.parent() {
				tokio::fs::create_dir_all(parent).await?;
			}

			self.emit_progress(&DownloadProgress {
				phase: "java".into(),
				total,
				completed,
				current_file: path.clone(),
				bytes_downloaded: 0,
				total_bytes: 0,
				speed: None,
			});

			match retry_download_bytes(&self.http, url).await {
				Ok(bytes) => {
					tokio::fs::write(&file_path, &bytes).await?;
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

			completed += 1;
		}

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

	fn find_component_url(
		&self,
		linux: &std::collections::HashMap<String, Vec<RuntimeEntry>>,
		major_version: u32,
	) -> AppResult<String> {
		for (_name, entries) in linux {
			for entry in entries {
				if let Some(ref version) = entry.version {
					let entry_major = version.name.as_ref()
						.and_then(|n| parse_major_from_version_name(n));

					if entry_major == Some(major_version) {
						if let Some(ref manifest_ref) = entry.manifest {
							if let Some(ref url) = manifest_ref.url {
								self.emit_log(&format!(
									"Found Java {} runtime component: {}",
									major_version,
									version.component.as_deref().unwrap_or("unknown")
								));
								return Ok(url.clone());
							}
						}
					}
				}
			}
		}

		Err(AppError::Internal(format!(
			"No Java runtime found for major version {}. \
			 Install manually with: sudo pacman -S jre-openjdk", major_version
		)))
	}
}

fn parse_java_major(version_str: &str) -> u32 {
	let cleaned: String = version_str.chars().take_while(|c| c.is_ascii_digit() || *c == '.').collect();
	let parts: Vec<&str> = cleaned.split('.').collect();

	match parts.as_slice() {
		[major, ..] if major.starts_with('1') && parts.len() >= 2 => {
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
		match http.get(url).send().await {
			Ok(resp) => {
				match resp.error_for_status() {
					Ok(validated) => {
						match validated.json::<T>().await {
							Ok(val) => return Ok(val),
							Err(e) => {
								last_err = Some(e.into());
							}
						}
					}
					Err(e) => {
						last_err = Some(e.into());
					}
				}
			}
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

async fn retry_download_bytes(
	http: &reqwest::Client,
	url: &str,
) -> AppResult<Vec<u8>> {
	let mut last_err = None;

	for attempt in 1..=MAX_RETRIES {
		match http.get(url).send().await {
			Ok(resp) => {
				match resp.error_for_status() {
					Ok(validated) => {
						match validated.bytes().await {
							Ok(bytes) => return Ok(bytes.to_vec()),
							Err(e) => {
								last_err = Some(e.into());
							}
						}
					}
					Err(e) => {
						last_err = Some(e.into());
					}
				}
			}
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
