use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

use futures_util::StreamExt;
use sha2::Digest;
use tokio::io::AsyncWriteExt;

use crate::error::{AppError, AppResult};

pub fn relative_path(value: &str) -> AppResult<PathBuf> {
    let normalized = value.replace('\\', "/");
    if normalized.is_empty()
        || normalized.starts_with('/')
        || normalized.contains([':', '\0', '<', '>', '|', '?', '*', '"'])
        || normalized.split('/').any(|part| {
            let stem = part.split('.').next().unwrap_or_default().to_ascii_uppercase();
            part == ".." || part == "." || part.ends_with(['.', ' '])
                || part.chars().any(char::is_control)
                || ["CON", "PRN", "AUX", "NUL"].contains(&stem.as_str())
                || ((stem.starts_with("COM") || stem.starts_with("LPT")) && stem.len() == 4 && matches!(stem.as_bytes()[3], b'1'..=b'9'))
        })
    {
        return Err(AppError::InvalidInput(format!("Unsafe pack path: {value}")));
    }
    Ok(PathBuf::from(normalized))
}

pub fn destination(root: &Path, value: &str) -> AppResult<PathBuf> {
    let relative = relative_path(value)?;
    let mut path = root.to_path_buf();
    for component in relative.components() {
        path.push(component);
        match std::fs::symlink_metadata(&path) {
            Ok(meta) if meta.file_type().is_symlink() => {
                return Err(AppError::InvalidInput(format!("Symlink in pack path: {value}")));
            }
            Ok(_) => {}
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
            Err(error) => return Err(error.into()),
        }
    }
    Ok(path)
}

fn is_allowed_pack_host(host: &str) -> bool {
    let host = host.to_ascii_lowercase();
    let exact = [
        "api.curseforge.com", "www.curseforge.com", "curseforge.com",
        "edge.forgecdn.net", "mediafilez.forgecdn.net", "media.forgecdn.net",
        "cdn.modrinth.com", "api.modrinth.com", "modrinth.com",
        "github.com", "raw.githubusercontent.com", "objects.githubusercontent.com",
        "release-assets.githubusercontent.com", "gitlab.com", "codeberg.org",
        "bitbucket.org", "archive.org", "cursemeta.dries007.net", "dist.creeper.host",
        "maven.architectury.dev", "maven.shedaniel.me", "maven.terraformersmc.com",
        "maven.blamejared.com", "maven.theillusivec4.top", "files.minecraftforge.net",
        "maven.minecraftforge.net", "libraries.minecraft.net", "resources.download.minecraft.net",
        "jitpack.io",
    ];
    if exact.contains(&host.as_str()) {
        return true;
    }
    let suffixes = [
        ".forgecdn.net",
        ".curseforge.com",
        ".modrinth.com",
        ".github.com",
        ".githubusercontent.com",
        ".amazonaws.com",
        ".cloudflarestorage.com",
        ".creeperhost.net",
        ".creeper.host",
        ".fabricmc.net",
        ".quiltmc.org",
        ".neoforged.net",
        ".minecraftforge.net",
        ".parchmentmc.org",
        ".geysermc.org",
        ".papermc.io",
        ".spongepowered.org",
    ];
    suffixes.iter().any(|suffix| host.ends_with(suffix))
}

pub fn download_url(value: &str) -> AppResult<reqwest::Url> {
    let url = reqwest::Url::parse(value)
        .map_err(|_| AppError::InvalidInput("Invalid download URL".into()))?;
    let host = url.host_str().unwrap_or_default();
    if url.scheme() != "https" || !is_allowed_pack_host(host)
        || !url.username().is_empty() || url.password().is_some()
        || url.port().is_some_and(|port| port != 443)
    {
        return Err(AppError::InvalidInput("Untrusted pack download URL".into()));
    }
    Ok(url)
}

pub fn mirrors(value: &str) -> AppResult<Vec<String>> {
    let url = download_url(value)?;
    let mut urls = vec![url.to_string()];
    let hosts = ["edge.forgecdn.net", "mediafilez.forgecdn.net", "media.forgecdn.net"];
    if hosts.contains(&url.host_str().unwrap_or_default()) {
        for host in hosts {
            let mut mirror = url.clone();
            mirror.set_host(Some(host)).map_err(|_| AppError::InvalidInput("Invalid CDN".into()))?;
            if !urls.contains(&mirror.to_string()) {
                urls.push(mirror.to_string());
            }
        }
    }
    Ok(urls)
}

pub fn client() -> AppResult<reqwest::Client> {
    static CLIENT: std::sync::OnceLock<reqwest::Client> = std::sync::OnceLock::new();
    if let Some(client) = CLIENT.get() { return Ok(client.clone()); }
    let client = reqwest::Client::builder()
        .redirect(reqwest::redirect::Policy::none())
        .connect_timeout(Duration::from_secs(15))
        .read_timeout(Duration::from_secs(45))
        .timeout(Duration::from_secs(600))
        .pool_idle_timeout(Some(Duration::from_secs(90)))
        .pool_max_idle_per_host(32)
        .tcp_nodelay(true)
        .user_agent(concat!("Luxmc/", env!("CARGO_PKG_VERSION")))
        .build()?;
    Ok(CLIENT.get_or_init(|| client).clone())
}

pub fn cancelled(cancel: Option<&AtomicBool>) -> AppResult<()> {
    if cancel.is_some_and(|flag| flag.load(Ordering::SeqCst)) {
        return Err(AppError::InvalidState("Importação cancelada".into()));
    }
    Ok(())
}

pub async fn backoff(attempt: u32, cancel: Option<&AtomicBool>) -> AppResult<()> {
    cancelled(cancel)?;
    if attempt == 0 { return Ok(()); }
    let ms = (300 * (1 << (attempt.min(4) - 1))).min(1500);
    let deadline = tokio::time::Instant::now() + Duration::from_millis(ms);
    while tokio::time::Instant::now() < deadline {
        tokio::time::sleep_until(deadline.min(tokio::time::Instant::now() + Duration::from_millis(50))).await;
        cancelled(cancel)?;
    }
    Ok(())
}

pub async fn bytes(http: &reqwest::Client, value: &str, cancel: Option<&AtomicBool>) -> AppResult<Vec<u8>> {
    transfer_bytes(http, download_url(value)?, cancel).await
}

async fn transfer_bytes(http: &reqwest::Client, initial: reqwest::Url, cancel: Option<&AtomicBool>) -> AppResult<Vec<u8>> {
    let transfer = async {
        let mut url = initial;
        for _ in 0..6 {
            cancelled(cancel)?;
            let mut request = http.get(url.clone());
            if url.host_str() == Some("api.curseforge.com") {
                if let Some(key) = super::curseforge::api_key() {
                    request = request.header("x-api-key", key);
                }
            }
            let response = request.send().await?;
            if response.status().is_redirection() {
                let location = response.headers().get(reqwest::header::LOCATION)
                    .and_then(|v| v.to_str().ok())
                    .ok_or_else(|| AppError::InvalidState("Missing redirect location".into()))?;
                let next = url.join(location).map_err(|_| AppError::InvalidInput("Invalid redirect".into()))?;
                url = download_url(next.as_str())?;
                continue;
            }
            let response = response.error_for_status()?;
            let limit = 1024 * 1024 * 1024;
            if response.content_length().is_some_and(|size| size > limit as u64) {
                return Err(AppError::InvalidInput("Pack file exceeds 1 GiB".into()));
            }
            let mut stream = response.bytes_stream();
            let mut result = Vec::new();
            while let Some(chunk) = stream.next().await {
                cancelled(cancel)?;
                let chunk = chunk?;
                if result.len() + chunk.len() > limit {
                    return Err(AppError::InvalidInput("Pack file exceeds 1 GiB".into()));
                }
                result.extend_from_slice(&chunk);
            }
            if result.is_empty() {
                return Err(AppError::InvalidState("Empty pack file".into()));
            }
            return Ok(result);
        }
        Err(AppError::InvalidState("Too many download redirects".into()))
    };
    let monitor = async {
        loop {
            cancelled(cancel)?;
            tokio::time::sleep(Duration::from_millis(100)).await;
        }
        #[allow(unreachable_code)]
        Ok::<Vec<u8>, AppError>(Vec::new())
    };
    tokio::select! {
        result = tokio::time::timeout(Duration::from_secs(600), transfer) =>
            result.map_err(|_| AppError::InvalidState("Download excedeu 10 minutos; tente novamente ou verifique sua conexão".into()))?,
        result = monitor => result,
    }
}

pub async fn verify_existing(path: &Path, size: Option<u64>, sha1: Option<&str>, sha512: Option<&str>, archive: bool) -> bool {
    let path = path.to_path_buf();
    let sha1 = sha1.map(str::to_owned);
    let sha512 = sha512.map(str::to_owned);
    tokio::task::spawn_blocking(move || {
        use std::io::Read;
        use std::sync::{LazyLock, Mutex};
        static VERIFIED: LazyLock<Mutex<std::collections::HashSet<String>>> = LazyLock::new(|| Mutex::new(std::collections::HashSet::new()));
        let Ok(mut file) = std::fs::File::open(&path) else { return false; };
        let Ok(before) = file.metadata() else { return false; };
        if !before.is_file() || before.len() == 0 || size.is_some_and(|s| s != before.len()) { return false; }
        let fingerprint = |meta: &std::fs::Metadata| {
            #[cfg(unix)]
            {
                use std::os::unix::fs::MetadataExt;
                format!("{}:{}:{}:{}:{}:{}:{}", meta.dev(), meta.ino(), meta.len(), meta.mtime(), meta.mtime_nsec(), meta.ctime(), meta.ctime_nsec())
            }
            #[cfg(not(unix))]
            { format!("{}:{:?}", meta.len(), meta.modified()) }
        };
        let signature = fingerprint(&before);
        let key = format!("{:?}:{signature}:{sha1:?}:{sha512:?}:{archive}", path);
        if cfg!(unix) && (sha1.is_some() || sha512.is_some()) && VERIFIED.lock().is_ok_and(|cache| cache.contains(&key)) { return true; }
        if sha1.is_some() || sha512.is_some() {
            let mut first = sha1::Sha1::new();
            let mut second = sha2::Sha512::new();
            let mut buffer = [0u8; 65536];
            loop {
                let count = match file.read(&mut buffer) { Ok(0) => break, Ok(count) => count, Err(_) => return false };
                if sha1.is_some() { first.update(&buffer[..count]); }
                if sha512.is_some() { second.update(&buffer[..count]); }
            }
            if sha1.as_ref().is_some_and(|hash| !format!("{:x}", first.finalize()).eq_ignore_ascii_case(hash))
                || sha512.as_ref().is_some_and(|hash| !format!("{:x}", second.finalize()).eq_ignore_ascii_case(hash)) { return false; }
        }
        if archive && zip::ZipArchive::new(std::io::BufReader::new(&file)).is_err() { return false; }
        let Ok(after) = file.metadata() else { return false; };
        if signature != fingerprint(&after) { return false; }
        if cfg!(unix) && (sha1.is_some() || sha512.is_some()) {
            if let Ok(mut cache) = VERIFIED.lock() {
                if cache.len() >= 4096 { cache.clear(); }
                cache.insert(key);
            }
        }
        true
    }).await.unwrap_or(false)
}

pub fn verify(bytes: &[u8], size: Option<u64>, sha1: Option<&str>, sha512: Option<&str>) -> bool {
    !bytes.is_empty()
        && size.is_none_or(|size| size == bytes.len() as u64)
        && sha1.is_none_or(|hash| format!("{:x}", sha1::Sha1::digest(bytes)).eq_ignore_ascii_case(hash))
        && sha512.is_none_or(|hash| format!("{:x}", sha2::Sha512::digest(bytes)).eq_ignore_ascii_case(hash))
}

pub fn content_cache_root() -> Option<PathBuf> {
    directories::ProjectDirs::from("io", "github", "Luxmc")
        .map(|dirs| dirs.data_dir().join("cache").join("pack-files"))
}

fn content_cache_path(root: &Path, sha1: Option<&str>, sha512: Option<&str>) -> Option<PathBuf> {
    let (algorithm, hash, length) = sha1.map(|hash| ("sha1", hash, 40))
        .or_else(|| sha512.map(|hash| ("sha512", hash, 128)))?;
    if hash.len() != length || !hash.bytes().all(|byte| byte.is_ascii_hexdigit()) { return None; }
    Some(root.join(format!("{algorithm}-{}.bin", hash.to_ascii_lowercase())))
}

pub async fn cached_content(root: &Path, size: Option<u64>, sha1: Option<&str>, sha512: Option<&str>) -> Option<Vec<u8>> {
    let path = content_cache_path(root, sha1, sha512)?;
    if tokio::fs::metadata(&path).await.ok()?.len() > 64 * 1024 * 1024 { return None; }
    let bytes = tokio::fs::read(&path).await.ok()?;
    if verify(&bytes, size, sha1, sha512) { Some(bytes) } else { None }
}

pub async fn cache_verified_content(root: &Path, bytes: &[u8], sha1: Option<&str>, sha512: Option<&str>) {
    if bytes.len() > 64 * 1024 * 1024 { return; }
    if let Some(path) = content_cache_path(root, sha1, sha512) {
        if let Err(error) = atomic_write(&path, bytes).await { tracing::debug!(%error, "Pack cache write skipped"); }
    }
}

pub async fn atomic_write(path: &Path, bytes: &[u8]) -> AppResult<()> {
    let parent = path.parent().ok_or_else(|| AppError::InvalidInput("Invalid destination".into()))?;
    tokio::fs::create_dir_all(parent).await?;
    let temp = parent.join(format!(".luxmc-{}.part", uuid::Uuid::new_v4()));
    let result = async {
        let mut file = tokio::fs::OpenOptions::new().write(true).create_new(true).open(&temp).await?;
        file.write_all(bytes).await?;
        file.flush().await?;
        drop(file);
        tokio::fs::rename(&temp, path).await
    }.await;
    if result.is_err() {
        let _ = tokio::fs::remove_file(&temp).await;
    }
    result.map_err(Into::into)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn verified_cache_reuses_download_and_rejects_corruption() {
        let root = std::env::temp_dir().join(format!("luxmc-pack-cache-{}", uuid::Uuid::new_v4()));
        let bytes = b"verified modpack file";
        let hash = format!("{:x}", sha1::Sha1::digest(bytes));
        assert!(cached_content(&root, None, Some(&hash), None).await.is_none());
        cache_verified_content(&root, bytes, Some(&hash), None).await;
        let reused = cached_content(&root, Some(bytes.len() as u64), Some(&hash), None).await.unwrap();
        assert_eq!(reused, bytes);
        assert!(cached_content(&root, Some(1), Some(&hash), None).await.is_none());
        let path = content_cache_path(&root, Some(&hash), None).unwrap();
        tokio::fs::write(path, b"corrupt download").await.unwrap();
        assert!(cached_content(&root, None, Some(&hash), None).await.is_none());
        assert!(content_cache_path(&root, Some("../escape"), None).is_none());
        assert!(root.canonicalize().unwrap().starts_with(std::env::temp_dir().canonicalize().unwrap()));
        std::fs::remove_dir_all(root).unwrap();
    }

    #[tokio::test]
    async fn backoff_stops_when_import_is_cancelled() {
        let cancel = AtomicBool::new(false);
        let wait = backoff(3, Some(&cancel));
        let trigger = async {
            tokio::time::sleep(Duration::from_millis(10)).await;
            cancel.store(true, Ordering::SeqCst);
        };
        let result = tokio::time::timeout(Duration::from_millis(500), async {
            let (result, ()) = tokio::join!(wait, trigger);
            result
        }).await.expect("Cancellation must interrupt backoff");
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn slow_active_download_survives_old_fifteen_second_limit() {
        use tokio::io::AsyncReadExt;
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let url = reqwest::Url::parse(&format!("http://{}/pack", listener.local_addr().unwrap())).unwrap();
        let server = tokio::spawn(async move {
            let (mut socket, _) = listener.accept().await.unwrap();
            let mut request = [0; 2048];
            let read = socket.read(&mut request).await.unwrap();
            assert!(read > 0);
            socket.write_all(b"HTTP/1.1 200 OK\r\nContent-Length: 4\r\nConnection: close\r\n\r\np").await.unwrap();
            tokio::time::sleep(Duration::from_secs(16)).await;
            socket.write_all(b"ack").await.unwrap();
        });
        assert_eq!(transfer_bytes(&client().unwrap(), url, None).await.unwrap(), b"pack");
        server.await.unwrap();
    }

    #[tokio::test]
    async fn cancellation_interrupts_a_stalled_response() {
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let url = reqwest::Url::parse(&format!("http://{}/pack", listener.local_addr().unwrap())).unwrap();
        let cancel = AtomicBool::new(false);
        let http = client().unwrap();
        let download = transfer_bytes(&http, url, Some(&cancel));
        let trigger = async {
            let (_socket, _) = listener.accept().await.unwrap();
            tokio::time::sleep(Duration::from_millis(150)).await;
            cancel.store(true, Ordering::SeqCst);
        };
        let (result, ()) = tokio::time::timeout(Duration::from_secs(2), async { tokio::join!(download, trigger) }).await.unwrap();
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn verified_files_reject_same_size_edits_and_changed_hashes() {
        let path = std::env::temp_dir().join(format!("luxmc-verify-{}", uuid::Uuid::new_v4()));
        std::fs::write(&path, b"valid").unwrap();
        let hash = format!("{:x}", sha1::Sha1::digest(b"valid"));
        assert!(verify_existing(&path, Some(5), Some(&hash), None, false).await);
        assert!(verify_existing(&path, Some(5), Some(&hash), None, false).await);
        assert!(!verify_existing(&path, Some(5), Some("wrong"), None, false).await);
        tokio::time::sleep(Duration::from_millis(100)).await;
        std::fs::write(&path, b"other").unwrap();
        assert!(!verify_existing(&path, Some(5), Some(&hash), None, false).await);
        std::fs::remove_file(path).unwrap();
    }

    #[test]
    fn rejects_cross_platform_traversal() {
        for path in ["../evil.jar", "mods/../../evil", "C:\\evil", "\\\\server\\file", "/tmp/evil", "mods\\..\\evil", "mods/./evil", ""] {
            assert!(relative_path(path).is_err(), "{path}");
        }
        assert_eq!(relative_path("mods\\valid.jar").unwrap(), PathBuf::from("mods/valid.jar"));
        assert!(relative_path("mods/library..jar").is_ok());
    }

    #[test]
    fn rejects_credentials_and_host_spoofing() {
        for url in ["https://api.curseforge.com.evil.test/file", "https://evil.test/curseforge.com", "https://api.curseforge.com@evil.test/file", "http://cdn.modrinth.com/file", "https://127.0.0.1/file", "https://cdn.modrinth.com:8443/file"] {
            assert!(download_url(url).is_err(), "{url}");
        }
        assert_eq!(mirrors("https://media.forgecdn.net/files/1/2/a.jar").unwrap().len(), 3);
    }

    #[test]
    fn detects_truncated_and_corrupt_content() {
        let hash = format!("{:x}", sha1::Sha1::digest(b"valid"));
        assert!(verify(b"valid", Some(5), Some(&hash), None));
        assert!(!verify(b"bad", Some(5), Some(&hash), None));
        assert!(!verify(b"other", Some(5), Some(&hash), None));
        assert!(!verify(b"", None, None, None));
    }
}
