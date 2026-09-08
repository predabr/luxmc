use luxmc_lib::core::minecraft::{self, lib_url_from_name};
use luxmc_lib::core::launcher::{self, lib_path_from_name};
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

const JAVA_RUNTIME_MANIFEST_URL: &str =
    "https://launchermeta.mojang.com/v1/products/java-runtime/2ec0cc96c44e5a76b9c8b7c39df7210883d12871/all.json";

#[derive(Debug, serde::Deserialize)]
struct RuntimeManifest {
    linux: Option<std::collections::HashMap<String, Vec<RuntimeEntry>>>,
}

#[derive(Debug, serde::Deserialize)]
struct RuntimeEntry {
    manifest: Option<RuntimeManifestRef>,
    version: Option<RuntimeVersion>,
}

#[derive(Debug, serde::Deserialize)]
struct RuntimeManifestRef {
    url: Option<String>,
}

#[derive(Debug, serde::Deserialize)]
struct RuntimeVersion {
    component: Option<String>,
    name: Option<String>,
}

#[derive(Debug, serde::Deserialize)]
struct ComponentManifest {
    files: Option<std::collections::HashMap<String, ComponentFile>>,
}

#[derive(Debug, serde::Deserialize)]
struct ComponentFile {
    #[serde(rename = "type")]
    file_type: Option<String>,
    downloads: Option<ComponentFileDownloads>,
    executable: Option<bool>,
}

#[derive(Debug, serde::Deserialize)]
struct ComponentFileDownloads {
    lzma: Option<ComponentDownload>,
    raw: Option<ComponentDownload>,
}

#[derive(Debug, serde::Deserialize)]
struct ComponentDownload {
    url: Option<String>,
}

#[tokio::main]
async fn main() {
    let client = reqwest::Client::new();
    let base_dir = PathBuf::from("/tmp/luxmc_e2e_test");
    let _ = tokio::fs::remove_dir_all(&base_dir).await;
    tokio::fs::create_dir_all(&base_dir).await.unwrap();

    let total_bytes = Arc::new(AtomicU64::new(0u64));

    println!("\n{}", "=".repeat(70));
    println!("  END-TO-END INTEGRATION TEST");
    println!("  Version: 1.21.4 | Date: {}", chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC"));
    println!("{}", "=".repeat(70));

    // ===== STEP 1: Fetch version manifest =====
    println!("\n--- Step 1: Fetch version manifest ---");
    let manifest = minecraft::fetch_version_manifest(&client).await.expect("manifest fetch failed");
    println!("  Versions available: {}", manifest.versions.len());
    println!("  Latest release: {}", manifest.latest.release);
    println!("  Latest snapshot: {}", manifest.latest.snapshot);

    // ===== STEP 2: Fetch version detail =====
    println!("\n--- Step 2: Fetch version detail for 1.21.4 ---");
    let entry = manifest.versions.iter().find(|v| v.id == "1.21.4").expect("1.21.4 not found");
    let detail = minecraft::fetch_version_detail(&client, &entry.url).await.expect("detail fetch failed");
    println!("  ID: {}", detail.id);
    println!("  Type: {}", detail.version_type);
    println!("  Main class: {}", detail.main_class.as_deref().unwrap_or("none"));
    println!("  Java major: {}", detail.java_major_version());
    println!("  Libraries: {}", detail.libraries.len());
    println!("  Has downloads: {}", detail.downloads.is_some());
    println!("  Has asset_index: {}", detail.asset_index.is_some());

    // ===== STEP 3: Download client jar =====
    println!("\n--- Step 3: Download client jar ---");
    if let Some(ref downloads) = detail.downloads {
        let version_dir = base_dir.join("versions").join(&detail.id);
        tokio::fs::create_dir_all(&version_dir).await.unwrap();
        let client_path = version_dir.join(format!("{}.jar", detail.id));

        let bytes = download_with_retry(&client, &downloads.client.url, &client_path, "client jar").await.expect("client jar download failed");
        let size = downloads.client.size;
        let actual = tokio::fs::read(&client_path).await.unwrap().len() as u64;
        total_bytes.fetch_add(actual, Ordering::Relaxed);
        println!("  URL: {}", &downloads.client.url[..80.min(downloads.client.url.len())]);
        println!("  Expected size: {} bytes", size);
        println!("  Actual size: {} bytes", actual);
        println!("  SHA1 expected: {}", &downloads.client.sha1);
        println!("  SHA1 match: {}", sha1_hex(&client_path) == downloads.client.sha1);
        assert_eq!(actual, size, "client jar size mismatch");
        assert_eq!(sha1_hex(&client_path), downloads.client.sha1, "client jar sha1 mismatch");
        println!("  ✓ Client jar downloaded and verified");
    }

    // ===== STEP 4: Download asset index =====
    println!("\n--- Step 4: Download asset index ---");
    if let Some(ref asset_index) = detail.asset_index {
        let indexes_dir = base_dir.join("assets").join("indexes");
        tokio::fs::create_dir_all(&indexes_dir).await.unwrap();
        let index_path = indexes_dir.join(format!("{}.json", asset_index.id));

        let bytes = download_with_retry(&client, &asset_index.url, &index_path, "asset index").await.expect("asset index download failed");
        let actual = tokio::fs::read(&index_path).await.unwrap().len() as u64;
        total_bytes.fetch_add(actual, Ordering::Relaxed);
        println!("  Index ID: {}", asset_index.id);
        println!("  Size: {} bytes", actual);

        let index_content = tokio::fs::read_to_string(&index_path).await.unwrap();
        let index: serde_json::Value = serde_json::from_str(&index_content).unwrap();
        let objects = index.get("objects").and_then(|o| o.as_object());
        let total_assets = objects.map(|m| m.len()).unwrap_or(0);
        println!("  Total assets: {}", total_assets);
        println!("  ✓ Asset index downloaded");
    }

    // ===== STEP 5: Download libraries =====
    println!("\n--- Step 5: Download libraries ---");
    let lib_dir = base_dir.join("libraries");
    tokio::fs::create_dir_all(&lib_dir).await.unwrap();

    let mut allowed_libs = Vec::new();
    for lib in &detail.libraries {
        if !launcher::is_library_allowed(lib) {
            continue;
        }
        allowed_libs.push(lib);
    }
    println!("  Allowed libraries: {}", allowed_libs.len());

    let mut downloaded_count = 0u64;
    let mut skipped_count = 0u64;
    let mut failed_count = 0u64;

    for lib in &allowed_libs {
        let path = lib_path_from_name(&lib_dir, &lib.name);

        let url = if let Some(ref downloads) = lib.downloads {
            if let Some(ref artifact) = downloads.artifact {
                if !artifact.url.is_empty() {
                    Some(artifact.url.clone())
                } else {
                    None
                }
            } else {
                None
            }
        } else {
            let u = lib_url_from_name(&lib.name, lib.url.as_deref());
            if !u.is_empty() && u.starts_with("http") {
                Some(u)
            } else {
                None
            }
        };

        let url = match url {
            Some(u) => u,
            None => continue,
        };

        if path.exists() {
            skipped_count += 1;
            continue;
        }

        if let Some(parent) = path.parent() {
            tokio::fs::create_dir_all(parent).await.unwrap();
        }

        match download_with_retry(&client, &url, &path, &lib.name).await {
            Ok(bytes) => {
                total_bytes.fetch_add(bytes, Ordering::Relaxed);
                downloaded_count += 1;
            }
            Err(e) => {
                println!("  FAILED: {} - {}", lib.name, e);
                failed_count += 1;
            }
        }
    }

    println!("  Downloaded: {}", downloaded_count);
    println!("  Skipped (existing): {}", skipped_count);
    println!("  Failed: {}", failed_count);

    // ===== STEP 6: Download Java 21 runtime =====
    println!("\n--- Step 6: Download Java 21 runtime ---");
    let java_dir = base_dir.join("java").join("21");
    tokio::fs::create_dir_all(&java_dir).await.unwrap();

    let manifest_url = JAVA_RUNTIME_MANIFEST_URL;
    let runtime_manifest: RuntimeManifest = client.get(manifest_url)
        .send().await.unwrap()
        .error_for_status().unwrap()
        .json().await.unwrap();

    let linux = runtime_manifest.linux.expect("no linux runtimes");

    let mut component_url = None;
    let mut component_name = String::new();
    for (_name, entries) in &linux {
        for entry in entries {
            if let Some(ref version) = entry.version {
                if let Some(ref name_str) = version.name {
                    let major = name_str.split('.').next().and_then(|s| s.parse::<u32>().ok());
                    if major == Some(21) {
                        if let Some(ref manifest_ref) = entry.manifest {
                            if let Some(ref url) = manifest_ref.url {
                                component_url = Some(url.clone());
                                component_name = version.component.clone().unwrap_or_default();
                            }
                        }
                    }
                }
            }
        }
    }

    let component_url = component_url.expect("no Java 21 runtime found");
    println!("  Component: {}", component_name);
    println!("  Manifest URL: {}...", &component_url[..60.min(component_url.len())]);

    let component_manifest: ComponentManifest = client.get(&component_url)
        .send().await.unwrap()
        .error_for_status().unwrap()
        .json().await.unwrap();

    let files = component_manifest.files.expect("no files in java manifest");
    let total_java_files = files.len();
    let mut java_downloaded = 0u64;
    let mut java_skipped = 0u64;

    println!("  Files to process: {}", total_java_files);

    for (path, file) in &files {
        if file.file_type.as_deref() != Some("file") {
            continue;
        }

        let downloads = match file.downloads {
            Some(ref d) => d,
            None => continue,
        };

        let download = downloads.raw.as_ref().or(downloads.lzma.as_ref());
        let url = match download {
            Some(ref d) => d.url.as_ref(),
            None => continue,
        };
        let url = match url {
            Some(u) => u,
            None => continue,
        };

        let file_path = java_dir.join(path);
        if file_path.exists() {
            java_skipped += 1;
            continue;
        }

        if let Some(parent) = file_path.parent() {
            tokio::fs::create_dir_all(parent).await.unwrap();
        }

        match download_with_retry(&client, url, &file_path, path).await {
            Ok(bytes) => {
                total_bytes.fetch_add(bytes, Ordering::Relaxed);
                java_downloaded += 1;
                if file.executable == Some(true) {
                    #[cfg(unix)]
                    {
                        use std::os::unix::fs::PermissionsExt;
                        let _ = tokio::fs::set_permissions(&file_path,
                            std::fs::Permissions::from_mode(0o755)).await;
                    }
                }
            }
            Err(e) => {
                println!("  FAILED Java file: {} - {}", path, e);
            }
        }
    }

    println!("  Java files downloaded: {}", java_downloaded);
    println!("  Java files skipped: {}", java_skipped);

    let java_bin = java_dir.join("bin").join("java");
    println!("  Java binary exists: {}", java_bin.exists());

    // ===== STEP 7: Validate all files =====
    println!("\n--- Step 7: Validate all files ---");
    let mut validation_ok = true;

    if let Some(ref downloads) = detail.downloads {
        let client_path = base_dir.join("versions").join(&detail.id).join(format!("{}.jar", detail.id));
        if !client_path.exists() {
            println!("  ✗ Client jar missing");
            validation_ok = false;
        } else {
            println!("  ✓ Client jar present");
        }
    }

    let mut missing_libs = 0;
    for lib in &allowed_libs {
        let path = lib_path_from_name(&lib_dir, &lib.name);
        if !path.exists() {
            missing_libs += 1;
        }
    }
    if missing_libs > 0 {
        println!("  ✗ {} libraries missing", missing_libs);
        validation_ok = false;
    } else {
        println!("  ✓ All {} libraries present", allowed_libs.len());
    }

    if let Some(ref asset_index) = detail.asset_index {
        let index_path = base_dir.join("assets").join("indexes").join(format!("{}.json", asset_index.id));
        if !index_path.exists() {
            println!("  ✗ Asset index missing");
            validation_ok = false;
        } else {
            println!("  ✓ Asset index present");
        }
    }

    if java_bin.exists() {
        println!("  ✓ Java 21 binary present");
    } else {
        println!("  ✗ Java 21 binary missing");
        validation_ok = false;
    }

    // ===== STEP 8: Build launch command (dry run) =====
    println!("\n--- Step 8: Build launch command ---");
    let classpath = build_classpath_for_test(&detail, &lib_dir, &base_dir);
    println!("  Classpath entries: {}", classpath.len());

    let game_args = detail.effective_game_args();
    let jvm_args_raw = detail.effective_jvm_args();
    println!("  Game args: {}", game_args.len());
    println!("  JVM args: {}", jvm_args_raw.as_ref().map(|a| a.len()).unwrap_or(0));

    let main_class = detail.main_class.as_deref().unwrap_or("net.minecraft.client.Minecraft");
    println!("  Main class: {}", main_class);

    let cp_str = classpath.iter().map(|p| p.to_string_lossy().to_string()).collect::<Vec<_>>().join(":");

    let mut full_args: Vec<String> = Vec::new();
    if let Some(ref jvm_args) = jvm_args_raw {
        for arg in jvm_args {
            if let Some(s) = arg.as_str() {
                let resolved = s
                    .replace("${natives_directory}", "/tmp/natives")
                    .replace("${launcher_name}", "Luxmc")
                    .replace("${launcher_version}", "0.1.0")
                    .replace("${classpath}", &cp_str);
                full_args.push(resolved);
            }
        }
    }

    let assets_dir = base_dir.join("assets");
    let asset_index_id = detail.asset_index.as_ref().map(|ai| ai.id.as_str()).unwrap_or("legacy");
    for arg in &game_args {
        let resolved = arg
            .replace("${auth_player_name}", "TestPlayer")
            .replace("${auth_uuid}", "00000000-0000-0000-0000-000000000001")
            .replace("${auth_session}", "dev-token")
            .replace("${auth_access_token}", "dev-token")
            .replace("${user_properties}", "{}")
            .replace("${version_name}", &detail.id)
            .replace("${game_directory}", "/tmp/game")
            .replace("${assets_directory}", &assets_dir.to_string_lossy())
            .replace("${assets_root}", &assets_dir.to_string_lossy())
            .replace("${game_assets}", &assets_dir.to_string_lossy())
            .replace("${asset_index}", asset_index_id)
            .replace("${assets_index_name}", asset_index_id)
            .replace("${clientid}", "")
            .replace("${auth_xuid}", "")
            .replace("${user_type}", "msa")
            .replace("${version_type}", &detail.version_type);
        full_args.push(resolved);
    }

    full_args.push(main_class.to_string());

    println!("  Total JVM+game args: {}", full_args.len() - 1);

    // ===== STEP 9: Try to actually run Minecraft =====
    println!("\n--- Step 9: Attempt real Minecraft launch ---");
    if java_bin.exists() {
        let output = tokio::process::Command::new(&java_bin)
            .args(&full_args)
            .current_dir("/tmp/game")
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .output()
            .await;

        match output {
            Ok(out) => {
                let stdout = String::from_utf8_lossy(&out.stdout);
                let stderr = String::from_utf8_lossy(&out.stderr);
                let exit_code = out.status.code().unwrap_or(-1);

                println!("  Exit code: {}", exit_code);
                println!("  stdout lines: {}", stdout.lines().count());
                println!("  stderr lines: {}", stderr.lines().count());

                if !stdout.is_empty() {
                    println!("  --- stdout (first 30 lines) ---");
                    for line in stdout.lines().take(30) {
                        println!("    {}", line);
                    }
                }

                if !stderr.is_empty() {
                    println!("  --- stderr (first 50 lines) ---");
                    for line in stderr.lines().take(50) {
                        println!("    {}", line);
                    }
                }

                if exit_code == 0 {
                    println!("  ✓ Minecraft process exited normally");
                } else {
                    println!("  ✗ Minecraft process exited with code {}", exit_code);
                }
            }
            Err(e) => {
                println!("  ✗ Failed to spawn Java: {}", e);
            }
        }
    } else {
        println!("  ✗ Cannot launch — Java binary not found");
    }

    // ===== FINAL REPORT =====
    let total = total_bytes.load(Ordering::Relaxed);
    println!("\n{}", "=".repeat(70));
    println!("  FINAL REPORT");
    println!("{}", "=".repeat(70));
    println!("  Total bytes downloaded: {} ({:.1} MB)", total, total as f64 / 1024.0 / 1024.0);
    println!("  Client jar: {}", if detail.downloads.is_some() { "OK" } else { "N/A" });
    println!("  Libraries: {} allowed, {} downloaded", allowed_libs.len(), downloaded_count);
    println!("  Asset index: {}", detail.asset_index.as_ref().map(|a| a.id.as_str()).unwrap_or("N/A"));
    println!("  Java binary: {}", if java_bin.exists() { "OK" } else { "MISSING" });
    println!("  Environment: unzip={}, java={}",
        which_exists("unzip"),
        which_exists("java"),
    );
    println!("{}", "=".repeat(70));
}

fn build_classpath_for_test(
    detail: &minecraft::VersionDetail,
    lib_dir: &PathBuf,
    base_dir: &PathBuf,
) -> Vec<PathBuf> {
    let mut cp = Vec::new();
    for lib in &detail.libraries {
        if !launcher::is_library_allowed(lib) {
            continue;
        }
        let path = lib_path_from_name(lib_dir, &lib.name);
        if path.exists() {
            cp.push(path);
        }
    }
    if detail.downloads.is_some() {
        let client_jar = base_dir
            .join("versions")
            .join(&detail.id)
            .join(format!("{}.jar", detail.id));
        if client_jar.exists() {
            cp.push(client_jar);
        }
    }
    cp
}

async fn download_with_retry(
    client: &reqwest::Client,
    url: &str,
    path: &PathBuf,
    label: &str,
) -> Result<u64, String> {
    let mut last_err = None;
    for attempt in 1..=3u32 {
        match client.get(url).send().await {
            Ok(resp) => {
                match resp.error_for_status() {
                    Ok(validated) => {
                        match validated.bytes().await {
                            Ok(bytes) => {
                                let len = bytes.len() as u64;
                                tokio::fs::write(path, &bytes).await.map_err(|e| e.to_string())?;
                                if attempt > 1 {
                                    println!("  [retry {}/3] OK: {}", attempt, label);
                                }
                                return Ok(len);
                            }
                            Err(e) => { last_err = Some(format!("bytes: {}", e)); }
                        }
                    }
                    Err(e) => { last_err = Some(format!("http {}: {}", e.status().unwrap_or_default(), e)); }
                }
            }
            Err(e) => { last_err = Some(format!("send: {}", e)); }
        }
        if attempt < 3 {
            let delay = 1000 * 2u64.pow(attempt - 1);
            println!("  [retry {}/3] Waiting {} ms before retry for {}...", attempt, delay, label);
            tokio::time::sleep(tokio::time::Duration::from_millis(delay)).await;
        }
    }
    Err(format!("Download failed after 3 attempts for {}: {:?}", label, last_err))
}

fn sha1_hex(path: &PathBuf) -> String {
    let bytes = std::fs::read(path).unwrap();
    use sha1::{Digest, Sha1};
    let mut hasher = Sha1::new();
    hasher.update(&bytes);
    format!("{:x}", hasher.finalize())
}

fn which_exists(name: &str) -> bool {
    std::process::Command::new("which")
        .arg(name)
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}
