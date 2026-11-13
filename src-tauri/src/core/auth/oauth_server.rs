use std::collections::HashMap;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::{TcpListener, TcpStream};

pub struct OAuthCallbackListeners {
    ipv4: TcpListener,
    ipv6: Option<TcpListener>,
}

impl OAuthCallbackListeners {
    async fn accept(&self) -> Result<(TcpStream, std::net::SocketAddr), std::io::Error> {
        if let Some(ipv6) = &self.ipv6 {
            tokio::select! {
                result = self.ipv4.accept() => result,
                result = ipv6.accept() => result,
            }
        } else {
            self.ipv4.accept().await
        }
    }
}

pub struct OAuthCallback {
    pub code: String,
    pub state: String,
}

pub async fn start_callback_server() -> Result<OAuthCallbackListeners, std::io::Error> {
    bind_callback_server(8453).await
}

async fn bind_callback_server(port: u16) -> Result<OAuthCallbackListeners, std::io::Error> {
    let ipv4 = TcpListener::bind(("127.0.0.1", port)).await?;
    let ipv6 = match TcpListener::bind(("::1", port)).await {
        Ok(listener) => Some(listener),
        Err(error)
            if matches!(
                error.kind(),
                std::io::ErrorKind::AddrNotAvailable | std::io::ErrorKind::Unsupported
            ) =>
        {
            tracing::warn!(%error, "IPv6 loopback is unavailable for Microsoft login");
            None
        }
        Err(error) => return Err(error),
    };
    Ok(OAuthCallbackListeners { ipv4, ipv6 })
}

pub async fn wait_for_callback(
    listeners: &OAuthCallbackListeners,
    timeout_secs: u64,
) -> Result<OAuthCallback, String> {
    let accept = tokio::time::timeout(
        std::time::Duration::from_secs(timeout_secs),
        listeners.accept(),
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

    if let Some(err) = params.get("error") {
        let desc = params.get("error_description").cloned().unwrap_or_else(|| err.clone());
        let _ = send_response(&mut writer_half, 400, &format!("Falha no Login Microsoft: {}", desc), false).await;
        return Err(format!("Microsoft OAuth error: {} ({})", err, desc));
    }

    let code = match params.get("code").cloned() {
        Some(c) => c,
        None => {
            let _ = send_response(&mut writer_half, 400, "Parâmetro de autorização ausente no retorno da Microsoft.", false).await;
            return Err("No code parameter in callback".to_string());
        }
    };
    let state = params.get("state").cloned().unwrap_or_default();

    let _ = send_response(
        &mut writer_half,
        200,
        "Login realizado com sucesso! Você já pode fechar esta aba e retornar ao Luxmc.",
        true,
    )
    .await;

    Ok(OAuthCallback { code, state })
}

async fn send_response(
    stream: &mut tokio::net::tcp::OwnedWriteHalf,
    status: u16,
    message: &str,
    success: bool,
) -> Result<(), std::io::Error> {
    let status_text = match status {
        200 => "OK",
        400 => "Bad Request",
        _ => "Error",
    };
    let accent_color = if success { "#e2b86b" } else { "#ef4444" };
    let status_title = if success { "Autenticação Concluída" } else { "Aviso de Autenticação" };
    let icon_char = if success { "✓" } else { "✕" };

    let html = format!(
        r#"<!DOCTYPE html>
<html lang="pt-BR">
<head>
    <meta charset="utf-8">
    <title>Luxmc - Login</title>
    <style>
        body {{
            background: #0f1013;
            color: #ffffff;
            font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif;
            display: flex;
            align-items: center;
            justify-content: center;
            height: 100vh;
            margin: 0;
            padding: 20px;
            box-sizing: border-box;
        }}
        .card {{
            background: #18191c;
            border: 1px solid rgba(255, 255, 255, 0.1);
            border-radius: 24px;
            padding: 40px;
            max-width: 480px;
            text-align: center;
            box-shadow: 0 20px 40px rgba(0, 0, 0, 0.6);
        }}
        .icon {{
            width: 56px;
            height: 56px;
            margin: 0 auto 20px;
            border-radius: 50%;
            display: flex;
            align-items: center;
            justify-content: center;
            font-size: 24px;
            font-weight: bold;
            color: #000;
            background: {accent_color};
            box-shadow: 0 0 24px {accent_color}66;
        }}
        h1 {{
            font-size: 20px;
            margin: 0 0 12px;
            color: #fff;
        }}
        p {{
            font-size: 14px;
            color: rgba(255, 255, 255, 0.6);
            line-height: 1.5;
            margin: 0;
        }}
    </style>
</head>
<body>
    <div class="card">
        <div class="icon">{icon_char}</div>
        <h1>{status_title}</h1>
        <p>{message}</p>
    </div>
</body>
</html>"#
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

#[cfg(test)]
mod tests {
    use super::*;

    async fn receive_callback(
        listeners: &OAuthCallbackListeners,
        address: std::net::SocketAddr,
    ) -> OAuthCallback {
        let accept = wait_for_callback(listeners, 2);
        let connect = async {
            let mut stream = TcpStream::connect(address).await.unwrap();
            stream
                .write_all(
                    b"GET /callback?code=test-code&state=test-state HTTP/1.1\r\nHost: localhost\r\n\r\n",
                )
                .await
                .unwrap();
        };
        let (callback, ()) = tokio::join!(accept, connect);
        callback.unwrap()
    }

    #[tokio::test]
    async fn microsoft_callback_accepts_both_loopback_address_families() {
        let reservation = TcpListener::bind(("127.0.0.1", 0)).await.unwrap();
        let port = reservation.local_addr().unwrap().port();
        drop(reservation);
        let listeners = bind_callback_server(port).await.unwrap();

        let ipv4 = receive_callback(
            &listeners,
            std::net::SocketAddr::from(([127, 0, 0, 1], port)),
        )
        .await;
        assert_eq!(ipv4.code, "test-code");
        assert_eq!(ipv4.state, "test-state");

        if listeners.ipv6.is_some() {
            let ipv6 = receive_callback(
                &listeners,
                std::net::SocketAddr::from(([0, 0, 0, 0, 0, 0, 0, 1], port)),
            )
            .await;
            assert_eq!(ipv6.code, "test-code");
            assert_eq!(ipv6.state, "test-state");
        }
    }
}
