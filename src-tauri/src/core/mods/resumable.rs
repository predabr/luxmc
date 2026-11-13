use crate::error::{AppError, AppResult};
use futures_util::StreamExt;
use std::{path::Path, sync::atomic::AtomicBool, time::Duration};
use tokio::io::AsyncWriteExt;

pub async fn download(client: &reqwest::Client, value: &str, target: &Path, cancel: Option<&AtomicBool>) -> AppResult<()> {
    download_inner(client, value, target, cancel, None).await
}

pub async fn download_with_progress(
    client: &reqwest::Client,
    value: &str,
    target: &Path,
    cancel: Option<&AtomicBool>,
    app: &tauri::AppHandle,
    label: &str,
) -> AppResult<()> {
    download_inner(client, value, target, cancel, Some((app, label))).await
}

async fn download_inner(
    client: &reqwest::Client,
    value: &str,
    target: &Path,
    cancel: Option<&AtomicBool>,
    progress: Option<(&tauri::AppHandle, &str)>,
) -> AppResult<()> {
    let url = super::pack_download::download_url(value)?;
    let transfer = transfer_inner(client, url, target, cancel, progress);
    let monitor = async { loop { super::pack_download::cancelled(cancel)?; tokio::time::sleep(std::time::Duration::from_millis(100)).await; } #[allow(unreachable_code)] Ok::<(),AppError>(()) };
    tokio::select! { result = transfer => result, result = monitor => result }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::io::AsyncReadExt;
    async fn response(range: bool) {
        let root=std::env::temp_dir().join(format!("luxmc-resume-{}",uuid::Uuid::new_v4()));
        tokio::fs::create_dir_all(&root).await.unwrap(); let target=root.join("pack.zip");
        tokio::fs::write(target.with_extension("download-part"),b"com").await.unwrap();
        tokio::fs::write(target.with_extension("download-validator"),"\"v1\"").await.unwrap();
        let listener=tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let url=reqwest::Url::parse(&format!("http://{}/file",listener.local_addr().unwrap())).unwrap();
        let server=tokio::spawn(async move {
            let (mut stream,_)=listener.accept().await.unwrap(); let mut request=[0u8;2048]; let n=stream.read(&mut request).await.unwrap();
            let request=String::from_utf8_lossy(&request[..n]).to_lowercase(); assert!(request.contains("range: bytes=3-")); assert!(request.contains("if-range: \"v1\""));
            let body=if range {"HTTP/1.1 206 Partial Content\r\nContent-Length: 5\r\nContent-Range: bytes 3-7/8\r\nETag: \"v1\"\r\nConnection: close\r\n\r\nplete"} else {"HTTP/1.1 200 OK\r\nContent-Length: 8\r\nETag: \"v2\"\r\nConnection: close\r\n\r\ncomplete"};
            stream.write_all(body.as_bytes()).await.unwrap();
        });
        transfer_inner(&reqwest::Client::new(),url,&target,None,None).await.unwrap();
        assert_eq!(tokio::fs::read(&target).await.unwrap(),b"complete"); server.await.unwrap(); tokio::fs::remove_dir_all(root).await.unwrap();
    }
    #[tokio::test] async fn continues_validated_range() {response(true).await;}
    #[tokio::test] async fn ignored_range_restarts_without_concatenation() {response(false).await;}
}

async fn transfer_inner(
    client: &reqwest::Client,
    mut url: reqwest::Url,
    target: &Path,
    cancel: Option<&AtomicBool>,
    progress: Option<(&tauri::AppHandle, &str)>,
) -> AppResult<()> {
    let partial = target.with_extension("download-part");
    let validator_path = target.with_extension("download-validator");
    if let Some(parent) = target.parent() { tokio::fs::create_dir_all(parent).await?; }
    for _ in 0..6 {
        super::pack_download::cancelled(cancel)?;
        let offset = tokio::fs::metadata(&partial).await.map(|m| m.len()).unwrap_or(0);
        let validator = tokio::fs::read_to_string(&validator_path).await.unwrap_or_default();
        let mut request = client.get(url.clone()).header("Accept-Encoding", "identity");
        if url.host_str() == Some("api.curseforge.com") {
            if let Some(key) = super::curseforge::api_key() { request = request.header("x-api-key",key); }
        }
        if offset > 0 && !validator.is_empty() {
            request = request.header("Range", format!("bytes={offset}-")).header("If-Range", &validator);
        }
        let response = request.send().await?;
        if response.status().is_redirection() {
            let location = response.headers().get("location").and_then(|v| v.to_str().ok()).ok_or_else(|| AppError::InvalidInput("Redirecionamento sem destino".into()))?;
            url = super::pack_download::download_url(url.join(location).map_err(|e| AppError::InvalidInput(e.to_string()))?.as_str())?;
            continue;
        }
        if response.status() == reqwest::StatusCode::RANGE_NOT_SATISFIABLE {
            let _ = tokio::fs::remove_file(&partial).await;
            let _ = tokio::fs::remove_file(&validator_path).await;
            continue;
        }
        let response = response.error_for_status()?;
        let append = response.status() == reqwest::StatusCode::PARTIAL_CONTENT;
        let content_range_total = response.headers().get("content-range")
            .and_then(|value| value.to_str().ok())
            .and_then(|value| value.rsplit('/').next())
            .and_then(|value| value.parse::<u64>().ok());
        if append {
            let range = response.headers().get("content-range").and_then(|v| v.to_str().ok()).unwrap_or("");
            if offset == 0 || validator.is_empty() || !range.starts_with(&format!("bytes {offset}-")) {
                return Err(AppError::InvalidInput("O servidor devolveu um intervalo de download inválido".into()));
            }
        }
        let expected = response.content_length();
        let total_bytes = content_range_total.or_else(|| expected.map(|total| total.saturating_add(if append { offset } else { 0 }))).unwrap_or(0);
        let etag = response.headers().get("etag").and_then(|v| v.to_str().ok()).filter(|v| !v.starts_with("W/"));
        let validator = etag.or_else(|| response.headers().get("last-modified").and_then(|v| v.to_str().ok())).unwrap_or("");
        tokio::fs::write(&validator_path, validator).await?;
        let output = tokio::fs::OpenOptions::new().create(true).write(true).append(append).truncate(!append).open(&partial).await?;
        let mut output = tokio::io::BufWriter::with_capacity(1024 * 1024, output);
        let mut received = 0u64;
        let mut stream = response.bytes_stream();
        let started = std::time::Instant::now();
        let mut last_report = std::time::Instant::now();
        loop {
            super::pack_download::cancelled(cancel)?;
            let chunk = tokio::select! {
                chunk = stream.next() => chunk,
                _ = tokio::time::sleep(std::time::Duration::from_millis(100)) => { continue; }
            };
            let Some(chunk) = chunk else { break; };
            let chunk = chunk?;
            received += chunk.len() as u64;
            if received + if append { offset } else { 0 } > 1024 * 1024 * 1024 {
                return Err(AppError::InvalidInput("Download acima de 1 GiB".into()));
            }
            output.write_all(&chunk).await?;
            if last_report.elapsed() >= Duration::from_millis(100) {
                emit_progress(progress, label_from_path(target), offset.saturating_add(received), total_bytes, started.elapsed(), false);
                last_report = std::time::Instant::now();
            }
        }
        output.flush().await?;
        let output = output.into_inner();
        output.sync_all().await?;
        drop(output);
        if received == 0 || expected.is_some_and(|total| total != received) { return Err(AppError::InvalidState("Download incompleto; os bytes recebidos foram preservados para retomada".into())); }
        emit_progress(progress, label_from_path(target), offset.saturating_add(received), total_bytes, started.elapsed(), true);
        if target.exists() { tokio::fs::remove_file(target).await?; }
        tokio::fs::rename(&partial, target).await?;
        let _ = tokio::fs::remove_file(&validator_path).await;
        return Ok(());
    }
    Err(AppError::InvalidState("Não foi possível retomar o download".into()))
}

fn label_from_path(path: &Path) -> &str {
    path.file_name().and_then(|name| name.to_str()).unwrap_or("modpack")
}

fn emit_progress(
    progress: Option<(&tauri::AppHandle, &str)>,
    fallback_label: &str,
    bytes_downloaded: u64,
    total_bytes: u64,
    elapsed: Duration,
    complete: bool,
) {
    let Some((app, label)) = progress else { return; };
    use tauri::Emitter;
    let bytes_per_second = if elapsed.is_zero() { 0 } else { bytes_downloaded.saturating_mul(1000) / elapsed.as_millis().max(1) as u64 };
    let _ = app.emit("download-progress", crate::core::downloader::DownloadProgress {
        phase: if complete { "done" } else { "downloading" }.into(),
        total: 1,
        completed: u64::from(complete),
        current_file: if label.is_empty() { fallback_label.to_owned() } else { label.to_owned() },
        bytes_downloaded,
        total_bytes,
        speed: Some(crate::core::downloader::DownloadSpeed {
            bytes_per_second,
            total_downloaded: bytes_downloaded,
            elapsed_ms: elapsed.as_millis() as u64,
        }),
    });
}
