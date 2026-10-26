use super::{LoaderVersion, PreparedLoader};
use crate::error::AppResult;

const FABRIC_META: &str = "https://meta.fabricmc.net/v2";
static GAME_VERSIONS: std::sync::LazyLock<std::sync::Mutex<std::collections::HashMap<String, (std::time::Instant, Vec<String>)>>> = std::sync::LazyLock::new(Default::default);

pub(super) async fn supported_game_versions(http: &reqwest::Client, meta: &str) -> AppResult<Vec<String>> {
    if let Some((at, versions)) = GAME_VERSIONS.lock().ok().and_then(|cache| cache.get(meta).cloned()) {
        if at.elapsed() < std::time::Duration::from_secs(300) { return Ok(versions); }
    }
    let entries: Vec<serde_json::Value> = http.get(format!("{meta}/versions/game")).timeout(std::time::Duration::from_secs(12)).send().await?.error_for_status()?.json().await?;
    let versions: Vec<String> = entries.iter().filter_map(|entry| entry.get("version").and_then(|value| value.as_str())).filter(|version| !version.is_empty()).map(str::to_owned).collect();
    if versions.is_empty() { return Err(crate::error::AppError::InvalidState("O provedor não informou as versões compatíveis. Tente novamente.".into())); }
    if let Ok(mut cache) = GAME_VERSIONS.lock() { cache.insert(meta.to_owned(), (std::time::Instant::now(), versions.clone())); }
    Ok(versions)
}

#[derive(Debug, Clone, serde::Deserialize)]
pub struct FabricLibrary {
    pub name: String,
    pub url: Option<String>,
    #[serde(default)]
    pub natives: Option<std::collections::HashMap<String, String>>,
}

#[derive(Debug, Clone, serde::Deserialize)]
pub struct FabricArguments {
    #[serde(default)]
    pub jvm: Vec<serde_json::Value>,
    #[serde(default)]
    pub game: Vec<serde_json::Value>,
}

#[derive(Debug, Clone, serde::Deserialize)]
pub struct FabricProfile {
    pub id: String,
    #[serde(rename = "mainClass")]
    pub main_class: String,
    #[serde(default)]
    pub arguments: Option<FabricArguments>,
    pub libraries: Vec<FabricLibrary>,
}

pub async fn fetch_versions(
    http: &reqwest::Client,
    mc_version: &str,
) -> AppResult<Vec<LoaderVersion>> {
    fetch_versions_from(http, mc_version, FABRIC_META).await
}

pub(super) async fn fetch_versions_from(http: &reqwest::Client, mc_version: &str, meta: &str) -> AppResult<Vec<LoaderVersion>> {
    if meta != FABRIC_META && !supported_game_versions(http, meta).await?.iter().any(|version| version == mc_version) { return Ok(Vec::new()); }
    let url = format!("{meta}/versions/loader/{mc_version}");
    let resp: Vec<serde_json::Value> = http
        .get(&url)
        .send()
        .await?
        .error_for_status()?
        .json()
        .await?;

    let versions = resp
        .iter()
        .map(|v| {
            let loader = v.get("loader").and_then(|l| l.as_object());
            let version = loader
                .and_then(|l| l.get("version"))
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            let stable = loader
                .and_then(|l| l.get("stable"))
                .and_then(|s| s.as_bool())
                .unwrap_or(false);
            LoaderVersion {
                id: version,
                stable,
            }
        })
        .filter(|v| !v.id.is_empty())
        .collect();

    Ok(versions)
}

pub async fn prepare_fabric(
    http: &reqwest::Client,
    libraries_dir: &std::path::Path,
    mc_version: &str,
    loader_version: Option<&str>,
) -> AppResult<PreparedLoader> {
    prepare_from_meta(http, libraries_dir, mc_version, loader_version, FABRIC_META).await
}

pub(super) async fn prepare_from_meta(http: &reqwest::Client, libraries_dir: &std::path::Path, mc_version: &str, loader_version: Option<&str>, meta: &str) -> AppResult<PreparedLoader> {
    let chosen_version = if let Some(v) = loader_version {
        if !v.trim().is_empty() {
            v.trim().to_string()
        } else {
            get_latest_loader_version(http, mc_version, meta).await?
        }
    } else {
        get_latest_loader_version(http, mc_version, meta).await?
    };

    let profile_url = format!(
        "{}/versions/loader/{}/{}/profile/json",
        meta, mc_version, chosen_version
    );

    let cache_key = format!("{:016x}.json", xxhash_rust::xxh3::xxh3_64(profile_url.as_bytes()));
    let cache_dir = libraries_dir.join(".luxmc-fabric");
    let cache_path = cache_dir.join(cache_key);
    let cached = tokio::fs::read(&cache_path).await.ok().and_then(|bytes| serde_json::from_slice::<FabricProfile>(&bytes).ok());
    let profile = match cached.filter(|profile| !profile.main_class.is_empty() && !profile.libraries.is_empty()) {
        Some(profile) => profile,
        None => {
            let bytes = http.get(&profile_url).send().await?.error_for_status()?.bytes().await?;
            let profile: FabricProfile = serde_json::from_slice(&bytes)?;
            if !profile.main_class.is_empty() && !profile.libraries.is_empty() {
                let _ = tokio::fs::create_dir_all(&cache_dir).await;
                let temporary = cache_path.with_extension(format!("{}.part", uuid::Uuid::new_v4()));
                if tokio::fs::write(&temporary, &bytes).await.is_ok() {
                    let _ = tokio::fs::rename(&temporary, &cache_path).await;
                }
            }
            profile
        }
    };

    use futures_util::{StreamExt, TryStreamExt};
    let mut seen = std::collections::HashSet::new();
    let os = crate::core::launcher::platform_mojang_name(crate::core::launcher::current_platform());
    let libraries = profile.libraries.into_iter().filter(|lib| seen.insert(lib.name.clone()) && lib.natives.as_ref().is_none_or(|natives| natives.contains_key(os))).collect::<Vec<_>>();
    let artifacts = futures_util::stream::iter(libraries).map(|lib| {
        let http = http.clone();
        let libraries_dir = libraries_dir.to_path_buf();
        async move {
        let classifier = lib.natives.as_ref().and_then(|natives| natives.get(os)).map(|value| value.replace("${arch}", if cfg!(target_arch = "x86") { "32" } else { "64" }));
        let coordinate = classifier.as_ref().map(|classifier| format!("{}:{classifier}", lib.name)).unwrap_or_else(|| lib.name.clone());
        let rel_path = crate::core::launcher::lib_path_from_name(&std::path::PathBuf::new(), &coordinate);
        if rel_path.components().any(|part| !matches!(part, std::path::Component::Normal(_))) {
            return Err(crate::error::AppError::InvalidInput("Caminho Maven inválido".into()));
        }
        let dest = libraries_dir.join(&rel_path);
        let base_url = lib.url.as_deref().unwrap_or("https://maven.fabricmc.net/");
        let relative = rel_path.to_string_lossy().replace('\\', "/");
        let url = format!("{}/{}", base_url.trim_end_matches('/'), relative);
        if super::ensure_maven_jar(&http, &dest, &url).await.is_err() {
            let central = format!("https://repo1.maven.org/maven2/{relative}");
            super::ensure_maven_jar(&http, &dest, &central).await?;
        }
        Ok::<_, crate::error::AppError>((dest, classifier.is_some()))
    }}).buffered(6).try_collect::<Vec<_>>().await?;

    let (native, regular): (Vec<_>, Vec<_>) = artifacts.into_iter().partition(|(_, native)| *native);
    let native_archives = native.into_iter().map(|(path, _)| path).collect();
    let classpath_entries = regular.into_iter().map(|(path, _)| path).collect();
    let mut jvm_args = Vec::new();
    if let Some(args) = profile.arguments {
        for arg in args.jvm {
            match &arg {
                serde_json::Value::String(s) => {
                    if crate::core::launcher::jvm_arg_allowed_on_current_os(s) {
                        jvm_args.push(s.to_string());
                    }
                }
                serde_json::Value::Object(obj) => {
                    if let Some(serde_json::Value::Array(rules)) = obj.get("rules") {
                        if !crate::core::launcher::evaluate_rules(rules) {
                            continue;
                        }
                    }
                    if let Some(serde_json::Value::Array(values)) = obj.get("value") {
                        for v in values {
                            if let Some(s) = v.as_str() {
                                if crate::core::launcher::jvm_arg_allowed_on_current_os(s) {
                                    jvm_args.push(s.to_string());
                                }
                            }
                        }
                    } else if let Some(serde_json::Value::String(s)) = obj.get("value") {
                        if crate::core::launcher::jvm_arg_allowed_on_current_os(s) {
                            jvm_args.push(s.to_string());
                        }
                    }
                }
                _ => {}
            }
        }
    }
    if jvm_args.is_empty() {
        jvm_args.push("-DFabricMcEmu=net.minecraft.client.main.Main".to_string());
    }

    Ok(PreparedLoader {
        native_archives,
        main_class: profile.main_class,
        classpath_entries,
        jvm_args,
        game_args: Vec::new(),
    })
}

async fn get_latest_loader_version(http: &reqwest::Client, mc_version: &str, meta: &str) -> AppResult<String> {
    let versions = fetch_versions_from(http, mc_version, meta).await?;
    let chosen = versions
        .iter()
        .find(|v| v.stable)
        .or_else(|| versions.first())
        .map(|v| v.id.clone())
        .ok_or_else(|| crate::error::AppError::NotFound(format!("Nenhuma versão de loader compatível com Minecraft {mc_version}")))?;
    Ok(chosen)
}

pub async fn ensure_fabric_api(
    http: &reqwest::Client,
    mods_dir: &std::path::Path,
    mc_version: &str,
) -> AppResult<()> {
    tokio::fs::create_dir_all(mods_dir).await?;

    let mut entries = match tokio::fs::read_dir(mods_dir).await {
        Ok(e) => e,
        Err(_) => return Ok(()),
    };

    let mut has_fabric_api = false;
    let mut has_any_mod = false;
    while let Ok(Some(entry)) = entries.next_entry().await {
        let name = entry.file_name().to_string_lossy().to_lowercase();
        if name.ends_with(".jar") {
            has_any_mod = true;
            if name.contains("fabric-api") || name.contains("fabric_api") {
                has_fabric_api = true;
                break;
            }
        }
    }

    if has_fabric_api || !has_any_mod {
        return Ok(());
    }

    tracing::info!(
        mc_version = %mc_version,
        "Fabric API missing in mods directory; downloading compatible version from Modrinth"
    );

    let url = format!(
        "https://api.modrinth.com/v2/project/fabric-api/version?game_versions=%5B%22{}%22%5D&loaders=%5B%22fabric%22%5D",
        urlencoding::encode(mc_version)
    );

    if let Ok(resp) = http
        .get(&url)
        .header("User-Agent", "Luxmc/1.3.0-beta")
        .send()
        .await
    {
        if resp.status().is_success() {
            if let Ok(versions) = resp.json::<Vec<serde_json::Value>>().await {
                if let Some(first_ver) = versions.first() {
                    if let Some(files) = first_ver.get("files").and_then(|f| f.as_array()) {
                        if let Some(file) = files.first() {
                            if let (Some(dl_url), Some(filename)) = (
                                file.get("url").and_then(|u| u.as_str()),
                                file.get("filename").and_then(|f| f.as_str()),
                            ) {
                                let dest = mods_dir.join(filename);
                                if let Ok(dl_resp) = http.get(dl_url).send().await {
                                    if dl_resp.status().is_success() {
                                        if let Ok(bytes) = dl_resp.bytes().await {
                                            let _ = tokio::fs::write(&dest, &bytes).await;
                                            tracing::info!(file = %filename, "Fabric API installed successfully");
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    Ok(())
}

#[cfg(test)]
mod fixed_loader_cache_tests {
    use super::*;
    #[tokio::test]
    async fn incompatible_game_is_rejected_before_querying_the_loader_endpoint() {
        use std::io::{Read, Write};
        let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
        listener.set_nonblocking(true).unwrap();
        let meta = format!("http://{}/v2", listener.local_addr().unwrap());
        let server = std::thread::spawn(move || {
            let started = std::time::Instant::now();
            let mut paths = Vec::new();
            while paths.len() < 2 && started.elapsed() < std::time::Duration::from_secs(5) {
                match listener.accept() {
                    Ok((mut stream,_)) => {
                        stream.set_read_timeout(Some(std::time::Duration::from_secs(2))).unwrap();
                        let mut buffer = [0;4096];
                        let size = stream.read(&mut buffer).unwrap();
                        let request = String::from_utf8_lossy(&buffer[..size]);
                        let path = request.lines().next().unwrap().split_whitespace().nth(1).unwrap().to_owned();
                        let body = match path.as_str() {
                            "/v2/versions/game" => r#"[{"version":"1.8.9","stable":true}]"#,
                            "/v2/versions/loader/1.8.9" => r#"[{"loader":{"version":"0.16.0","stable":true}}]"#,
                            _ => panic!("Unexpected unsupported version request: {path}"),
                        };
                        write!(stream,"HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",body.len(),body).unwrap();
                        paths.push(path);
                    }
                    Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => std::thread::sleep(std::time::Duration::from_millis(5)),
                    Err(error) => panic!("{error}"),
                }
            }
            paths
        });
        let http = reqwest::Client::builder().no_proxy().timeout(std::time::Duration::from_secs(2)).build().unwrap();
        assert!(fetch_versions_from(&http,"26.3",&meta).await.unwrap().is_empty());
        let supported = fetch_versions_from(&http,"1.8.9",&meta).await.unwrap();
        assert_eq!(supported[0].id,"0.16.0");
        assert_eq!(server.join().unwrap(),vec!["/v2/versions/game","/v2/versions/loader/1.8.9"]);
    }
    #[tokio::test]
    async fn windows_fabric_fixed_loader_uses_disk_cache_without_network() {
        let root = std::env::temp_dir().join(format!("luxmc-fabric-cache-{}", uuid::Uuid::new_v4()));
        let cache_dir = root.join(".luxmc-fabric");
        tokio::fs::create_dir_all(&cache_dir).await.unwrap();
        let profile_url = format!("{}/versions/loader/1.21.1/0.16.14/profile/json", FABRIC_META);
        let cache_key = format!("{:016x}.json", xxhash_rust::xxh3::xxh3_64(profile_url.as_bytes()));
        tokio::fs::write(cache_dir.join(cache_key), serde_json::to_vec(&serde_json::json!({"id":"fixture","mainClass":"net.fabricmc.loader.impl.launch.knot.KnotClient","libraries":[{"name":"fixture:test:1","url":"https://invalid.invalid/"}]})).unwrap()).await.unwrap();
        let library = root.join("fixture/test/1/test-1.jar");
        tokio::fs::create_dir_all(library.parent().unwrap()).await.unwrap();
        let mut writer = zip::ZipWriter::new(std::fs::File::create(&library).unwrap());
        writer.finish().unwrap();
        let client = reqwest::Client::builder().timeout(std::time::Duration::from_millis(100)).build().unwrap();
        let prepared = prepare_fabric(&client, &root, "1.21.1", Some("0.16.14")).await.unwrap();
        assert_eq!(prepared.main_class, "net.fabricmc.loader.impl.launch.knot.KnotClient");
        assert_eq!(prepared.classpath_entries, vec![library]);
        let resolved = root.canonicalize().unwrap();
        assert!(resolved.starts_with(std::env::temp_dir().canonicalize().unwrap()));
        std::fs::remove_dir_all(resolved).unwrap();
    }
}
