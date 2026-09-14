use crate::error::{AppError, AppResult};
use serde::{Deserialize, Serialize};
use std::net::UdpSocket;
use tauri::Emitter;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::{TcpListener, TcpStream};

const DEFAULT_P2P_PORT: u16 = 25575;
const MAX_MESSAGE_SIZE: usize = 4096;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct P2PMessagePayload {
    pub sender: String,
    pub text: String,
    pub timestamp: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct P2PConnectionInfo {
    pub ip: String,
    pub port: u16,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HostLinkInfo {
    pub local_ip: String,
    pub public_ip: Option<String>,
    pub port: u16,
    pub share_link: String,
    pub direct_address: String,
    pub public_address: Option<String>,
    pub share_code: String,
    pub motd: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DiscoveredLanWorld {
    pub motd: String,
    pub port: u16,
    pub host: String,
}

#[tauri::command]
pub async fn p2p_scan_lan_worlds() -> AppResult<Vec<DiscoveredLanWorld>> {
    use std::net::Ipv4Addr;

    let multicast_addr = Ipv4Addr::new(224, 0, 2, 60);
    let socket = match tokio::net::UdpSocket::bind("0.0.0.0:4445").await {
        Ok(s) => s,
        Err(_) => {
            return Ok(Vec::new());
        }
    };

    let _ = socket.join_multicast_v4(multicast_addr, Ipv4Addr::UNSPECIFIED);

    let mut worlds = Vec::new();
    let mut buf = [0u8; 1024];

    let end_time = tokio::time::Instant::now() + std::time::Duration::from_millis(1500);
    while tokio::time::Instant::now() < end_time {
        let remaining = end_time - tokio::time::Instant::now();
        if let Ok(Ok((len, src))) = tokio::time::timeout(remaining, socket.recv_from(&mut buf)).await {
            let msg = String::from_utf8_lossy(&buf[..len]);
            if let (Some(motd_start), Some(motd_end)) = (msg.find("[MOTD]"), msg.find("[/MOTD]")) {
                let motd = msg[motd_start + 6..motd_end].to_string();
                if let (Some(ad_start), Some(ad_end)) = (msg.find("[AD]"), msg.find("[/AD]")) {
                    let port_str = &msg[ad_start + 4..ad_end];
                    if let Ok(port) = port_str.parse::<u16>() {
                        let host = src.ip().to_string();
                        if !worlds.iter().any(|w: &DiscoveredLanWorld| w.port == port && w.host == host) {
                            worlds.push(DiscoveredLanWorld { motd, port, host });
                        }
                    }
                }
            }
        } else {
            break;
        }
    }

    Ok(worlds)
}

#[tauri::command]
pub async fn p2p_get_host_link(port: Option<u16>) -> AppResult<HostLinkInfo> {
    let target_port = port.unwrap_or(25565);
    let local_ip = match UdpSocket::bind("0.0.0.0:0") {
        Ok(socket) => match socket.connect("8.8.8.8:80") {
            Ok(_) => match socket.local_addr() {
                Ok(addr) => addr.ip().to_string(),
                Err(_) => "127.0.0.1".to_string(),
            },
            Err(_) => "127.0.0.1".to_string(),
        },
        Err(_) => "127.0.0.1".to_string(),
    };

    let direct_address = format!("{}:{}", local_ip, target_port);

    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(2))
        .build()
        .unwrap_or_default();

    let public_ip = match client.get("https://api.ipify.org").send().await {
        Ok(resp) if resp.status().is_success() => {
            resp.text().await.ok().map(|s| s.trim().to_string())
        }
        _ => None,
    };

    let public_address = public_ip.as_ref().map(|pip| format!("{}:{}", pip, target_port));
    let share_addr = public_address.clone().unwrap_or_else(|| direct_address.clone());
    let share_link = format!("luxmc://join/{}", share_addr);

    let code_num = (target_port as u32).wrapping_mul(17) % 9000 + 1000;
    let share_code = format!("LUX-{}", code_num);

    Ok(HostLinkInfo {
        local_ip,
        public_ip,
        port: target_port,
        share_link,
        direct_address,
        public_address,
        share_code,
        motd: Some("Mundo do Amigo no Luxmc".into()),
    })
}

#[tauri::command]
pub fn p2p_get_local_info() -> AppResult<P2PConnectionInfo> {
    let ip = match UdpSocket::bind("0.0.0.0:0") {
        Ok(socket) => match socket.connect("8.8.8.8:80") {
            Ok(_) => match socket.local_addr() {
                Ok(addr) => addr.ip().to_string(),
                Err(_) => "127.0.0.1".to_string(),
            },
            Err(_) => "127.0.0.1".to_string(),
        },
        Err(_) => "127.0.0.1".to_string(),
    };

    Ok(P2PConnectionInfo {
        ip,
        port: DEFAULT_P2P_PORT,
    })
}

static LISTENER_INITIALIZED: std::sync::atomic::AtomicBool =
    std::sync::atomic::AtomicBool::new(false);

#[tauri::command]
pub async fn p2p_start_listener(app: tauri::AppHandle) -> AppResult<bool> {
    if LISTENER_INITIALIZED.swap(true, std::sync::atomic::Ordering::SeqCst) {
        return Ok(true);
    }

    let addr = format!("0.0.0.0:{}", DEFAULT_P2P_PORT);
    let listener = match TcpListener::bind(&addr).await {
        Ok(l) => l,
        Err(e) => {
            tracing::warn!("Could not bind P2P listener on {}: {}", addr, e);
            LISTENER_INITIALIZED.store(false, std::sync::atomic::Ordering::SeqCst);
            return Ok(false);
        }
    };

    tracing::info!("Luxmc P2P listener active on {}", addr);

    tokio::spawn(async move {
        while let Ok((stream, peer)) = listener.accept().await {
            tracing::debug!(peer = %peer, "P2P connection accepted");
            let app_clone = app.clone();
            tokio::spawn(async move {
                let mut reader = BufReader::new(stream);
                let mut line = String::new();
                match tokio::time::timeout(std::time::Duration::from_secs(5), reader.read_line(&mut line)).await {
                    Ok(Ok(0)) => {}
                    Ok(Ok(_)) => {
                        if line.len() > MAX_MESSAGE_SIZE {
                            tracing::warn!("P2P message too large: {} bytes", line.len());
                            return;
                        }
                        if let Ok(payload) = serde_json::from_str::<P2PMessagePayload>(&line) {
                            if payload.sender.len() > 64 || payload.text.len() > 2048 {
                                tracing::warn!("P2P payload fields too large");
                                return;
                            }
                            let _ = app_clone.emit("p2p-chat-message", payload);
                        }
                    }
                    _ => {}
                }
            });
        }
    });

    Ok(true)
}

#[tauri::command]
pub async fn p2p_send_message(
    target_address: String,
    sender: String,
    text: String,
) -> AppResult<bool> {
    let addr = if target_address.contains(':') {
        target_address
    } else {
        format!("{}:{}", target_address, DEFAULT_P2P_PORT)
    };

    let payload = P2PMessagePayload {
        sender,
        text,
        timestamp: chrono::Local::now().format("%H:%M").to_string(),
    };

    let serialized =
        serde_json::to_string(&payload).map_err(|e| AppError::Internal(e.to_string()))? + "\n";

    let connect_future = TcpStream::connect(&addr);
    let mut stream = tokio::time::timeout(std::time::Duration::from_secs(3), connect_future)
        .await
        .map_err(|_| AppError::Internal(format!("Tempo limite esgotado ao conectar a {}", addr)))?
        .map_err(|e| AppError::Internal(format!("Não foi possível conectar a {}: {}", addr, e)))?;

    stream
        .write_all(serialized.as_bytes())
        .await
        .map_err(|e| AppError::Internal(format!("Falha ao enviar mensagem: {}", e)))?;

    stream
        .flush()
        .await
        .map_err(|e| AppError::Internal(format!("Falha ao descarregar buffer: {}", e)))?;

    Ok(true)
}
