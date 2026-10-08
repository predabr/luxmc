use crate::error::{AppError, AppResult};
use futures_util::StreamExt;
use std::{path::Path, sync::atomic::AtomicBool};
use tokio::io::AsyncWriteExt;

pub async fn download(client: &reqwest::Client, value: &str, target: &Path, cancel: Option<&AtomicBool>) -> AppResult<()> {
    let url = super::pack_download::download_url(value)?;
    let transfer = transfer(client,url,target,cancel);
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
        transfer(&reqwest::Client::new(),url,&target,None).await.unwrap();
        assert_eq!(tokio::fs::read(&target).await.unwrap(),b"complete"); server.await.unwrap(); tokio::fs::remove_dir_all(root).await.unwrap();
    }
    #[tokio::test] async fn continues_validated_range() {response(true).await;}
    #[tokio::test] async fn ignored_range_restarts_without_concatenation() {response(false).await;}
}

async fn transfer(client: &reqwest::Client, mut url: reqwest::Url, target: &Path, cancel: Option<&AtomicBool>) -> AppResult<()> {
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
        if append {
            let range = response.headers().get("content-range").and_then(|v| v.to_str().ok()).unwrap_or("");
            if offset == 0 || validator.is_empty() || !range.starts_with(&format!("bytes {offset}-")) {
                return Err(AppError::InvalidInput("O servidor devolveu um intervalo de download inválido".into()));
            }
        }
        let expected = response.content_length();
        let etag = response.headers().get("etag").and_then(|v| v.to_str().ok()).filter(|v| !v.starts_with("W/"));
        let validator = etag.or_else(|| response.headers().get("last-modified").and_then(|v| v.to_str().ok())).unwrap_or("");
        tokio::fs::write(&validator_path, validator).await?;
        let mut output = tokio::fs::OpenOptions::new().create(true).write(true).append(append).truncate(!append).open(&partial).await?;
        let mut received = 0u64;
        let mut stream = response.bytes_stream();
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
        }
        output.sync_all().await?;
        drop(output);
        if received == 0 || expected.is_some_and(|total| total != received) { return Err(AppError::InvalidState("Download incompleto; os bytes recebidos foram preservados para retomada".into())); }
        if target.exists() { tokio::fs::remove_file(target).await?; }
        tokio::fs::rename(&partial, target).await?;
        let _ = tokio::fs::remove_file(&validator_path).await;
        return Ok(());
    }
    Err(AppError::InvalidState("Não foi possível retomar o download".into()))
}
