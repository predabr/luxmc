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

pub fn download_url(value: &str) -> AppResult<reqwest::Url> {
    let url = reqwest::Url::parse(value)
        .map_err(|_| AppError::InvalidInput("Invalid download URL".into()))?;
    let host = url.host_str().unwrap_or_default();
    let allowed = [
        "api.curseforge.com", "www.curseforge.com", "edge.forgecdn.net",
        "mediafilez.forgecdn.net", "media.forgecdn.net", "cdn.modrinth.com",
        "github.com", "raw.githubusercontent.com", "objects.githubusercontent.com",
        "release-assets.githubusercontent.com", "gitlab.com",
    ];
    if url.scheme() != "https" || !allowed.contains(&host)
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
    Ok(reqwest::Client::builder()
        .redirect(reqwest::redirect::Policy::none())
        .timeout(Duration::from_secs(35))
        .user_agent("Luxmc/1.9.1")
        .build()?)
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
    let deadline = tokio::time::Instant::now() + Duration::from_millis(500 * (1 << (attempt.min(6) - 1)));
    while tokio::time::Instant::now() < deadline {
        tokio::time::sleep_until(deadline.min(tokio::time::Instant::now() + Duration::from_millis(50))).await;
        cancelled(cancel)?;
    }
    Ok(())
}

pub async fn bytes(http: &reqwest::Client, value: &str, cancel: Option<&AtomicBool>) -> AppResult<Vec<u8>> {
    let transfer = async {
        let mut url = download_url(value)?;
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
        result = tokio::time::timeout(Duration::from_secs(35), transfer) =>
            result.map_err(|_| AppError::InvalidState("Download timed out after 35s".into()))?,
        result = monitor => result,
    }
}

pub fn verify(bytes: &[u8], size: Option<u64>, sha1: Option<&str>, sha512: Option<&str>) -> bool {
    !bytes.is_empty()
        && size.is_none_or(|size| size == bytes.len() as u64)
        && sha1.is_none_or(|hash| format!("{:x}", sha1::Sha1::digest(bytes)).eq_ignore_ascii_case(hash))
        && sha512.is_none_or(|hash| format!("{:x}", sha2::Sha512::digest(bytes)).eq_ignore_ascii_case(hash))
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
