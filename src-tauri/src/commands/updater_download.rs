use crate::error::{AppError, AppResult};
use futures_util::StreamExt;
use std::path::Path;
use tokio::io::AsyncWriteExt;

pub(super) async fn download(
    client: &reqwest::Client,
    url: &url::Url,
    destination: &Path,
    trusted: impl Fn(&url::Url) -> bool,
    progress: impl Fn(u64, u64),
) -> AppResult<(u64, u64)> {
    let mut last = None;
    for attempt in 0..3 {
        if attempt > 0 { tokio::time::sleep(std::time::Duration::from_millis(500 * attempt)).await; }
        let result: AppResult<(u64, u64)> = async {
            let offset = tokio::fs::metadata(destination).await.map(|m| m.len()).unwrap_or(0);
            let validator_path = destination.with_extension("validator");
            let validator = tokio::fs::read_to_string(&validator_path).await.unwrap_or_default();
            let mut request = client.get(url.clone()).header("Accept-Encoding","identity").header("User-Agent", "Luxmc-Launcher-Updater");
            if offset > 0 && !validator.is_empty() { request = request.header("Range",format!("bytes={offset}-")).header("If-Range",&validator); }
            let response = request.send().await?;
            if response.status() == reqwest::StatusCode::RANGE_NOT_SATISFIABLE { let _ = tokio::fs::remove_file(destination).await; let _ = tokio::fs::remove_file(&validator_path).await; return Err(AppError::Internal("Retomada indisponível; reiniciando download".into())); }
            let response = response.error_for_status()?;
            if !trusted(response.url()) { return Err(AppError::InvalidInput("Origem de atualização não confiável.".into())); }
            let append = response.status() == reqwest::StatusCode::PARTIAL_CONTENT;
            if append && (offset==0 || validator.is_empty() || !response.headers().get("content-range").and_then(|v| v.to_str().ok()).unwrap_or("").starts_with(&format!("bytes {offset}-"))) { return Err(AppError::InvalidInput("Intervalo de atualização inválido".into())); }
            let received_length = response.content_length().unwrap_or(0);
            let total = if received_length > 0 {received_length + if append {offset} else {0}} else {0};
            let next_validator = response.headers().get("etag").and_then(|v| v.to_str().ok()).filter(|v| !v.starts_with("W/")).or_else(|| response.headers().get("last-modified").and_then(|v| v.to_str().ok())).unwrap_or("");
            tokio::fs::write(&validator_path,next_validator).await?;
            if total > super::updater::MAX_UPDATE_BYTES { return Err(AppError::InvalidInput("Atualização acima do limite de tamanho.".into())); }
            let mut output = tokio::fs::OpenOptions::new().create(true).write(true).append(append).truncate(!append).open(destination).await?;
            let mut stream = response.bytes_stream();
            let mut downloaded = if append {offset} else {0};
            progress(downloaded, total);
            while let Some(chunk) = tokio::time::timeout(std::time::Duration::from_secs(30), stream.next()).await
                .map_err(|_| AppError::Internal("Download interrompido por falta de resposta.".into()))? {
                let chunk = chunk?;
                downloaded += chunk.len() as u64;
                if downloaded > super::updater::MAX_UPDATE_BYTES { return Err(AppError::InvalidInput("Atualização acima do limite de tamanho.".into())); }
                output.write_all(&chunk).await?;
                progress(downloaded, total);
            }
            if downloaded == 0 || (total > 0 && total != downloaded) { return Err(AppError::Internal("Atualização incompleta.".into())); }
            output.sync_all().await?;
            Ok((downloaded, total))
        }.await;
        match result {
            Ok(result) => return Ok(result),
            Err(error @ AppError::InvalidInput(_)) => { let _ = tokio::fs::remove_file(destination).await; return Err(error); }
            Err(error @ AppError::Http(_)) if matches!(&error, AppError::Http(inner) if inner.status().is_some_and(|status| status.is_client_error() && status != reqwest::StatusCode::TOO_MANY_REQUESTS)) => { let _ = tokio::fs::remove_file(destination).await; return Err(error); }
            Err(error) => last = Some(error),
        }
    }
    Err(last.unwrap_or_else(|| AppError::Internal("Falha no download da atualização.".into())))
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::io::AsyncReadExt;

    #[tokio::test]
    async fn restart_keeps_partial_and_validates_range() {
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let address = listener.local_addr().unwrap();
        let server = tokio::spawn(async move {
            for attempt in 0..4 {
                let (mut stream, _) = listener.accept().await.unwrap();
                let mut request = [0; 2048]; let size = stream.read(&mut request).await.unwrap();
                if attempt == 3 {
                    assert!(String::from_utf8_lossy(&request[..size]).to_lowercase().contains("range: bytes=3-"));
                    stream.write_all(b"HTTP/1.1 206 Partial Content\r\nContent-Length: 5\r\nContent-Range: bytes 3-7/8\r\nETag: \"v1\"\r\nConnection: close\r\n\r\nplete").await.unwrap();
                } else {
                    stream.write_all(b"HTTP/1.1 200 OK\r\nContent-Length: 8\r\nETag: \"v1\"\r\nConnection: close\r\n\r\ncom").await.unwrap();
                }
            }
        });
        let path = std::env::temp_dir().join(format!("luxmc-update-resume-{}",uuid::Uuid::new_v4()));
        let url = url::Url::parse(&format!("http://{address}/update.exe")).unwrap();
        let client = reqwest::Client::new();
        assert!(download(&client,&url,&path,|_|true,|_,_|{}).await.is_err());
        assert_eq!(tokio::fs::read(&path).await.unwrap(),b"com");
        assert_eq!(download(&client,&url,&path,|_|true,|_,_|{}).await.unwrap(),(8,8));
        assert_eq!(tokio::fs::read(&path).await.unwrap(),b"complete");
        tokio::fs::remove_file(&path).await.unwrap();tokio::fs::remove_file(path.with_extension("validator")).await.unwrap();server.await.unwrap();
    }

    #[tokio::test]
    async fn interrupted_update_retries_without_concatenating_partial_data() {
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let address = listener.local_addr().unwrap();
        let server = tokio::spawn(async move {
            for body in [b"bad".as_slice(), b"complete".as_slice()] {
                let (mut stream, _) = listener.accept().await.unwrap();
                let mut request = [0; 2048]; let _ = stream.read(&mut request).await;
                stream.write_all(b"HTTP/1.1 200 OK\r\nContent-Length: 8\r\nConnection: close\r\n\r\n").await.unwrap();
                stream.write_all(body).await.unwrap();
            }
        });
        let path = std::env::temp_dir().join(format!("luxmc-update-test-{}", uuid::Uuid::new_v4()));
        let url = url::Url::parse(&format!("http://{address}/update.exe")).unwrap();
        assert_eq!(download(&reqwest::Client::new(), &url, &path, |_| true, |_, _| {}).await.unwrap(), (8, 8));
        assert_eq!(tokio::fs::read(&path).await.unwrap(), b"complete");
        tokio::fs::remove_file(path).await.unwrap(); server.await.unwrap();
    }
}
