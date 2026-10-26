use crate::error::{AppError, AppResult};
use tokio::io::{AsyncReadExt, AsyncWriteExt};

const PORT: u16 = 38457;
const ORIGINS: [&str; 3] = ["https://luxmc-r92.pages.dev", "https://luxmc.top", "https://www.luxmc.top"];

pub(crate) async fn portal_url(path: &str) -> AppResult<String> {
    let base = format!("{}{}", super::lux_account::portal_base(), path);
    let Some(account) = active_account().await? else { return Ok(base); };
    let value = ticket(account, path).await?;
    let code = value.get("ticket").and_then(|value|value.as_str()).filter(|value|value.len() == 64 && value.bytes().all(|byte|byte.is_ascii_hexdigit()))
        .ok_or_else(|| AppError::InvalidState("Sessão do site indisponível.".into()))?;
    let mut url = url::Url::parse(&base).map_err(|_| AppError::InvalidInput("Página inválida.".into()))?;
    let anchor = url.fragment().unwrap_or_default().to_owned();
    url.set_fragment(Some(&format!("launcher_ticket={code}{}", if anchor.is_empty() { String::new() } else { format!("&section={anchor}") })));
    Ok(url.to_string())
}

async fn active_account() -> AppResult<Option<String>> {
    let settings = super::settings::settings_get().await?;
    let Some(id) = settings.get("activeAccountId").and_then(|value|value.as_str()).filter(|value|!value.is_empty()) else { return Ok(None); };
    let db = crate::db::shared_db().await?;
    Ok(crate::db::schema::accounts::get_by_id(&db, id).await?.map(|account|account.id))
}

async fn ticket(account: String, path: &str) -> AppResult<serde_json::Value> {
    super::membership::membership_request_core(account, super::membership::MembershipRequest::PortalTicket {owner: false, destination: Some(path.into())}).await
}

struct Request {
    method: String,
    origin: String,
    length: usize,
}

fn validate_request(header: &str) -> Option<Request> {
    let mut lines = header.split("\r\n");
    let parts = lines.next()?.split_whitespace().collect::<Vec<_>>();
    if parts.len() != 3 || parts[1] != "/v1/session" || parts[2] != "HTTP/1.1" || !matches!(parts[0], "POST" | "OPTIONS") { return None; }
    let mut headers = std::collections::HashMap::new();
    for line in lines.filter(|line|!line.is_empty()) {
        let (name, value) = line.split_once(':')?;
        if headers.insert(name.trim().to_ascii_lowercase(), value.trim()).is_some() { return None; }
    }
    if headers.get("host")? != &format!("127.0.0.1:{PORT}") || headers.contains_key("transfer-encoding") { return None; }
    let origin = *headers.get("origin")?;
    if !ORIGINS.contains(&origin) { return None; }
    let length = headers.get("content-length").map(|value|value.parse::<usize>().ok()).unwrap_or(Some(0))?;
    if length > 256 { return None; }
    if parts[0] == "POST" && (headers.get("x-luxmc-site") != Some(&"1") || !headers.get("content-type")?.starts_with("application/json") || length == 0) { return None; }
    if parts[0] == "OPTIONS" && headers.get("access-control-request-method") != Some(&"POST") { return None; }
    Some(Request {method: parts[0].into(), origin: origin.into(), length})
}

async fn reply(socket: &mut tokio::net::TcpStream, status: u16, origin: Option<&str>, body: &str) -> std::io::Result<()> {
    let cors = origin.map(|origin|format!("Access-Control-Allow-Origin: {origin}\r\nAccess-Control-Allow-Methods: POST, OPTIONS\r\nAccess-Control-Allow-Headers: Content-Type, X-Luxmc-Site\r\nAccess-Control-Allow-Private-Network: true\r\nVary: Origin\r\n")).unwrap_or_default();
    let response = format!("HTTP/1.1 {status} {}\r\n{cors}Content-Type: application/json\r\nCache-Control: no-store\r\nX-Content-Type-Options: nosniff\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}", if status == 200 {"OK"} else if status == 204 {"No Content"} else {"Unavailable"}, body.len());
    socket.write_all(response.as_bytes()).await
}

async fn handle(socket: &mut tokio::net::TcpStream, last_request: &std::sync::Mutex<std::time::Instant>) -> std::io::Result<()> {
    let mut data = Vec::new();
    let split = loop {
        let mut buffer = [0; 1024];
        let size = socket.read(&mut buffer).await?;
        if size == 0 { return Ok(()); }
        data.extend_from_slice(&buffer[..size]);
        if data.len() > 8192 { return reply(socket, 403, None, "{}").await; }
        if let Some(offset) = data.windows(4).position(|value|value == b"\r\n\r\n") { break offset + 4; }
    };
    let Some(request) = std::str::from_utf8(&data[..split]).ok().and_then(validate_request) else { return reply(socket, 403, None, "{}").await; };
    if request.method == "OPTIONS" { return reply(socket, 204, Some(&request.origin), "").await; }
    while data.len() < split + request.length {
        let mut buffer = [0; 256];
        let size = socket.read(&mut buffer).await?;
        if size == 0 { return Ok(()); }
        data.extend_from_slice(&buffer[..size]);
    }
    if serde_json::from_slice::<serde_json::Value>(&data[split..split + request.length]).ok() != Some(serde_json::json!({})) { return reply(socket, 403, None, "{}").await; }
    let allowed = last_request.lock().map(|mut time| {
        if time.elapsed() < std::time::Duration::from_secs(2) { return false; }
        *time = std::time::Instant::now();
        true
    }).unwrap_or(false);
    if !allowed { return reply(socket, 429, Some(&request.origin), "{}").await; }
    let value = async {
        let id = active_account().await.ok()??;
        ticket(id, "/conta").await.ok()
    }.await;
    match value { Some(value) => reply(socket, 200, Some(&request.origin), &value.to_string()).await, None => reply(socket, 404, Some(&request.origin), "{}").await }
}

pub(crate) async fn serve() {
    let Ok(listener) = tokio::net::TcpListener::bind(("127.0.0.1", PORT)).await else {
        tracing::warn!("Local site session connector unavailable; signed portal links remain available");
        return;
    };
    let connections = std::sync::Arc::new(tokio::sync::Semaphore::new(4));
    let last_request = std::sync::Arc::new(std::sync::Mutex::new(std::time::Instant::now() - std::time::Duration::from_secs(5)));
    while let Ok((mut socket, address)) = listener.accept().await {
        if !address.ip().is_loopback() { continue; }
        let Ok(permit) = connections.clone().try_acquire_owned() else { continue; };
        let last_request = last_request.clone();
        tokio::spawn(async move {
            let _permit = permit;
            let _ = tokio::time::timeout(std::time::Duration::from_secs(20), handle(&mut socket, &last_request)).await;
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn local_connector_accepts_only_the_official_site_and_explicit_json_requests() {
        let valid = format!("POST /v1/session HTTP/1.1\r\nHost: 127.0.0.1:{PORT}\r\nOrigin: {}\r\nContent-Type: application/json\r\nX-Luxmc-Site: 1\r\nContent-Length: 2\r\n\r\n", ORIGINS[0]);
        assert!(validate_request(&valid).is_some());
        for changed in [
            valid.replace(ORIGINS[0], "https://evil.example"), valid.replace(ORIGINS[0], "null"),
            valid.replace("127.0.0.1", "evil.example"), valid.replace("POST /", "GET /"),
            valid.replace("X-Luxmc-Site: 1\r\n", ""), valid.replace("application/json", "text/plain"),
            valid.replace("Content-Length: 2", "Content-Length: 10000"), valid.replace("Content-Length: 2", "Content-Length: 2\r\nTransfer-Encoding: chunked"),
            valid.replace("Content-Length: 2", "Content-Length: 2\r\nOrigin: https://evil.example"),
        ] { assert!(validate_request(&changed).is_none()); }
        let preflight = format!("OPTIONS /v1/session HTTP/1.1\r\nHost: 127.0.0.1:{PORT}\r\nOrigin: {}\r\nAccess-Control-Request-Method: POST\r\n\r\n", ORIGINS[0]);
        assert!(validate_request(&preflight).is_some());
    }
}
