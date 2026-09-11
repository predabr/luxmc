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
    pub port: u16,
    pub share_link: String,
    pub direct_address: String,
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
    let share_link = format!("luxmc://join/{}", direct_address);

    Ok(HostLinkInfo {
        local_ip,
        port: target_port,
        share_link,
        direct_address,
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

    let mut stream = TcpStream::connect(&addr)
        .await
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
