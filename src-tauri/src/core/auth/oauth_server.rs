use std::collections::HashMap;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::TcpListener;

pub struct OAuthCallback {
    pub code: String,
    pub state: String,
}

pub async fn start_callback_server() -> Result<TcpListener, std::io::Error> {
    let listener = TcpListener::bind("127.0.0.1:8453").await?;
    Ok(listener)
}

pub async fn wait_for_callback(
    listener: &TcpListener,
    timeout_secs: u64,
) -> Result<OAuthCallback, String> {
    let accept = tokio::time::timeout(
        std::time::Duration::from_secs(timeout_secs),
        listener.accept(),
    )
    .await
    .map_err(|_| "Login timed out waiting for browser redirect".to_string())?;

    let (stream, _addr) = accept.map_err(|e| format!("Failed to accept connection: {}", e))?;

    let (reader_half, mut writer_half) = stream.into_split();
    let mut reader = BufReader::new(reader_half);
    let mut request_line = String::new();
    reader
        .read_line(&mut request_line)
        .await
        .map_err(|e| e.to_string())?;

    loop {
        let mut line = String::new();
        reader
            .read_line(&mut line)
            .await
            .map_err(|e| e.to_string())?;
        if line.trim().is_empty() {
            break;
        }
    }

    let path = request_line.split_whitespace().nth(1).unwrap_or("/");

    let params = parse_query_string(path);
    let code = match params.get("code").cloned() {
        Some(c) => c,
        None => {
            let _ = send_response(&mut writer_half, 400, "Missing code parameter").await;
            return Err("No code parameter in callback".to_string());
        }
    };
    let state = params.get("state").cloned().unwrap_or_default();

    let _ = send_response(
        &mut writer_half,
        200,
        "Login successful! You can close this tab.",
    )
    .await;

    Ok(OAuthCallback { code, state })
}

async fn send_response(
    stream: &mut tokio::net::tcp::OwnedWriteHalf,
    status: u16,
    body: &str,
) -> Result<(), std::io::Error> {
    let status_text = match status {
        200 => "OK",
        400 => "Bad Request",
        _ => "Error",
    };
    let html = format!(
        "<!DOCTYPE html><html><head><title>Luxmc</title></head><body><h1>{}</h1></body></html>",
        body
    );
    let response = format!(
		"HTTP/1.1 {} {}\r\nContent-Type: text/html; charset=utf-8\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
		status,
		status_text,
		html.len(),
		html
	);
    stream.write_all(response.as_bytes()).await?;
    stream.flush().await?;
    Ok(())
}

fn parse_query_string(path: &str) -> HashMap<String, String> {
    let mut params = HashMap::new();
    if let Some(query_start) = path.find('?') {
        let query = &path[query_start + 1..];
        for pair in query.split('&') {
            if let Some((key, value)) = pair.split_once('=') {
                params.insert(
                    urlencoding::decode(key).unwrap_or_default().to_string(),
                    urlencoding::decode(value).unwrap_or_default().to_string(),
                );
            }
        }
    }
    params
}
