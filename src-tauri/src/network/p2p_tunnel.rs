use crate::error::{AppError, AppResult};
use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
use iroh::{endpoint::presets, Endpoint, EndpointAddr};
use serde::{Deserialize, Serialize};
use std::{
    sync::LazyLock,
    time::{Duration, SystemTime, UNIX_EPOCH},
};
use tokio::{
    net::{TcpListener, TcpStream, UdpSocket},
    sync::Mutex,
    task::{JoinHandle, JoinSet},
};

const ALPN: &[u8] = b"luxmc/minecraft-tunnel/1";
const PREFIX: &str = "luxmc-world:";
const MINECRAFT_LAN_MULTICAST: &str = "224.0.2.60:4445";
static SESSION: LazyLock<Mutex<Option<TunnelSession>>> = LazyLock::new(|| Mutex::new(None));

fn failure(error: impl std::fmt::Display) -> AppError {
    AppError::InvalidState(format!("Túnel: {error}"))
}
fn now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

#[derive(Serialize, Deserialize)]
struct Invitation {
    version: u8,
    addr: EndpointAddr,
    secret: [u8; 32],
    expires: u64,
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TunnelStatus {
    pub mode: String,
    pub invitation: Option<String>,
    pub local_address: Option<String>,
    pub expires_at: u64,
    pub ping_ms: Option<u64>,
    pub transport: String,
    pub room_code: Option<String>,
}

pub struct TunnelSession {
    endpoint: Endpoint,
    task: JoinHandle<()>,
    lan_broadcast: Option<JoinHandle<()>>,
    connection: Option<iroh::endpoint::Connection>,
    status: TunnelStatus,
}
impl Drop for TunnelSession {
    fn drop(&mut self) {
        self.task.abort();
        if let Some(task) = &self.lan_broadcast {
            task.abort();
        }
        if let Some(c) = &self.connection {
            c.close(0u8.into(), b"session stopped");
        }
    }
}
impl TunnelSession {
    async fn stop(mut self) {
        self.task.abort();
        if let Some(task) = &self.lan_broadcast {
            task.abort();
        }
        let _ = (&mut self.task).await;
        self.endpoint.close().await;
    }
}

async fn endpoint() -> AppResult<Endpoint> {
    Endpoint::builder(presets::N0)
        .alpns(vec![ALPN.to_vec()])
        .bind()
        .await
        .map_err(failure)
}

async fn bridge(
    mut tcp: TcpStream,
    send: iroh::endpoint::SendStream,
    recv: iroh::endpoint::RecvStream,
) -> AppResult<()> {
    tcp.set_nodelay(true)?;
    let mut quic = tokio::io::join(recv, send);
    tokio::io::copy_bidirectional(&mut tcp, &mut quic).await?;
    Ok(())
}

fn start_lan_broadcast(local_port: u16) -> JoinHandle<()> {
    tokio::spawn(async move {
        let Ok(socket) = socket2::Socket::new(
            socket2::Domain::IPV4,
            socket2::Type::DGRAM,
            Some(socket2::Protocol::UDP),
        ) else {
            return;
        };
        let loopback = std::net::Ipv4Addr::LOCALHOST;
        if socket.bind(&std::net::SocketAddrV4::new(loopback, 0).into()).is_err()
            || socket.set_multicast_if_v4(&loopback).is_err()
            || socket.set_nonblocking(true).is_err()
        {
            return;
        }
        let Ok(socket) = UdpSocket::from_std(socket.into()) else {
            return;
        };
        let _ = socket.set_multicast_loop_v4(true);
        let _ = socket.set_multicast_ttl_v4(1);
        let payload = lan_advertisement(local_port);
        let mut interval = tokio::time::interval(Duration::from_millis(1500));
        loop {
            interval.tick().await;
            let _ = socket
                .send_to(payload.as_bytes(), MINECRAFT_LAN_MULTICAST)
                .await;
        }
    })
}

fn lan_advertisement(local_port: u16) -> String {
    format!("[MOTD]§b[Luxmc] §fMundo de Amigo[/MOTD][AD]{local_port}[/AD]")
}

pub async fn start_host(port: u16) -> AppResult<TunnelSession> {
    if port == 0 {
        return Err(AppError::InvalidInput("Porta inválida".into()));
    }
    start_host_with(port, endpoint().await?).await
}

async fn start_host_with(port: u16, endpoint: Endpoint) -> AppResult<TunnelSession> {
    if port == 0 {
        return Err(AppError::InvalidInput(
            "Informe a porta LAN do Minecraft".into(),
        ));
    }
    tokio::time::timeout(
        Duration::from_secs(3),
        TcpStream::connect(("127.0.0.1", port)),
    )
    .await
    .map_err(failure)??;
    let _ = tokio::time::timeout(Duration::from_secs(8), endpoint.online()).await;
    let mut secret = [0u8; 32];
    secret[..16].copy_from_slice(uuid::Uuid::new_v4().as_bytes());
    secret[16..].copy_from_slice(uuid::Uuid::new_v4().as_bytes());
    let expires = now() + 3600;
    let invitation_payload = format!(
        "{PREFIX}{}",
        URL_SAFE_NO_PAD.encode(serde_json::to_vec(&Invitation {
            version: 1,
            addr: endpoint.addr(),
            secret,
            expires
        })?)
    );
    let room_code = format!("LUX-{:04X}", rand::random::<u16>());
    let invitation = format!("{room_code}|{invitation_payload}");
    let ep = endpoint.clone();
    let task = tokio::spawn(async move {
        let deadline = tokio::time::sleep(Duration::from_secs(3600));
        tokio::pin!(deadline);
        let mut clients = JoinSet::new();
        loop {
            tokio::select! {
                _ = &mut deadline => break,
                Some(_) = clients.join_next(), if !clients.is_empty() => {},
                incoming = ep.accept() => {
                    let Some(incoming) = incoming else { break };
                    if clients.len() >= 16 { incoming.refuse(); continue; }
                    clients.spawn(async move {
                        let Ok(Ok(conn)) = tokio::time::timeout(Duration::from_secs(10), incoming).await else { return };
                        let mut streams = JoinSet::new();
                        loop {
                            tokio::select! {
                                Some(_) = streams.join_next(), if !streams.is_empty() => {},
                                stream = conn.accept_bi() => {
                                    let Ok((mut send, mut recv)) = stream else { break };
                                    if streams.len() >= 16 { conn.close(1u8.into(), b"stream limit"); break; }
                                    streams.spawn(async move {
                                        let mut handshake = [0u8; 33];
                                        let Ok(Ok(_)) = tokio::time::timeout(Duration::from_secs(5), recv.read_exact(&mut handshake)).await else { return };
                                        let difference = handshake[1..].iter().zip(secret).fold(0u8, |acc, (a,b)| acc | (*a ^ b));
                                        if difference != 0 || now() >= expires || handshake[0] > 1 { return; }
                                        if handshake[0] == 0 { let _ = send.write_all(&[1]).await; let _ = send.finish(); return; }
                                        let Ok(Ok(tcp)) = tokio::time::timeout(Duration::from_secs(5), TcpStream::connect(("127.0.0.1", port))).await else { return };
                                        if send.write_all(&[1]).await.is_err() { return; }
                                        let _ = bridge(tcp, send, recv).await;
                                    });
                                }
                            }
                        }
                    });
                }
            }
        }
        clients.abort_all();
        ep.close().await;
    });
    Ok(TunnelSession {
        endpoint,
        task,
        lan_broadcast: None,
        connection: None,
        status: TunnelStatus {
            mode: "host".into(),
            invitation: Some(invitation),
            local_address: None,
            expires_at: expires,
            ping_ms: None,
            transport: "QUIC automático".into(),
            room_code: Some(room_code),
        },
    })
}

fn parse_invitation(input: &str) -> AppResult<Invitation> {
    if input.len() > 4096 {
        return Err(AppError::InvalidInput("Convite muito longo".into()));
    }
    let input = input.trim().rsplit_once('|').map(|(_, value)| value).unwrap_or(input.trim());
    let data =
        URL_SAFE_NO_PAD
            .decode(input.strip_prefix(PREFIX).ok_or_else(|| {
                AppError::InvalidInput("Cole o convite completo luxmc-world:".into())
            })?)
            .map_err(failure)?;
    let invitation: Invitation = serde_json::from_slice(&data)?;
    if invitation.version != 1 || invitation.expires <= now() || invitation.expires > now() + 3660 {
        return Err(AppError::InvalidInput(
            "Convite inválido ou expirado".into(),
        ));
    }
    Ok(invitation)
}

pub async fn start_client(input: &str) -> AppResult<TunnelSession> {
    let invite = parse_invitation(input)?;
    start_client_with(invite, endpoint().await?).await
}

async fn start_client_with(invite: Invitation, endpoint: Endpoint) -> AppResult<TunnelSession> {
    let setup = async {
        let conn = endpoint.connect(invite.addr, ALPN).await.map_err(failure)?;
        let (mut send, mut recv) = conn.open_bi().await.map_err(failure)?;
        let mut handshake = [0u8; 33];
        handshake[1..].copy_from_slice(&invite.secret);
        send.write_all(&handshake).await.map_err(failure)?;
        let mut accepted = [0];
        recv.read_exact(&mut accepted).await.map_err(failure)?;
        if accepted != [1] {
            return Err(failure("Convite recusado"));
        }
        Ok::<_, AppError>(conn)
    };
    let conn = match tokio::time::timeout(Duration::from_secs(25), setup).await {
        Ok(Ok(conn)) => conn,
        result => {
            endpoint.close().await;
            return Err(failure(format!("Host indisponível: {result:?}")));
        }
    };
    let listener = TcpListener::bind(("127.0.0.1", 0)).await?;
    let local_port = listener.local_addr()?.port();
    let address = format!("127.0.0.1:{local_port}");
    let lan_broadcast = start_lan_broadcast(local_port);
    let connection = conn.clone();
    let ep = endpoint.clone();
    let task = tokio::spawn(async move {
        let deadline =
            tokio::time::sleep(Duration::from_secs(invite.expires.saturating_sub(now())));
        tokio::pin!(deadline);
        let mut streams = JoinSet::new();
        loop {
            tokio::select! {
                _ = &mut deadline => break,
                _ = connection.closed() => break,
                Some(_) = streams.join_next(), if !streams.is_empty() => {},
                accepted = listener.accept() => {
                    let Ok((tcp, _)) = accepted else { break };
                    if streams.len() >= 16 { continue; }
                    let c = connection.clone();
                    streams.spawn(async move {
                        let open = async {
                            let (mut send, mut recv) = c.open_bi().await.map_err(failure)?;
                            let mut handshake = [1u8; 33]; handshake[1..].copy_from_slice(&invite.secret);
                            send.write_all(&handshake).await.map_err(failure)?;
                            let mut ack = [0]; recv.read_exact(&mut ack).await.map_err(failure)?;
                            if ack != [1] { return Err(failure("Conexão recusada")); }
                            Ok::<_, AppError>((send, recv))
                        };
                        if let Ok(Ok((send, recv))) = tokio::time::timeout(Duration::from_secs(10), open).await { let _ = bridge(tcp, send, recv).await; }
                    });
                }
            }
        }
        streams.abort_all();
        ep.close().await;
    });
    Ok(TunnelSession {
        endpoint,
        task,
        lan_broadcast: Some(lan_broadcast),
        connection: Some(conn),
        status: TunnelStatus {
            mode: "client".into(),
            invitation: None,
            local_address: Some(address),
            expires_at: invite.expires,
            ping_ms: None,
            transport: "QUIC automático".into(),
            room_code: None,
        },
    })
}

#[tauri::command]
pub async fn host_world(port: u16) -> AppResult<TunnelStatus> {
    let mut state = SESSION.lock().await;
    if state.is_some() {
        return Err(failure("Encerre a sessão atual primeiro"));
    }
    let session = start_host(port).await?;
    let status = session.status.clone();
    *state = Some(session);
    Ok(status)
}
#[tauri::command]
pub async fn join_world(invitation: String) -> AppResult<TunnelStatus> {
    let mut state = SESSION.lock().await;
    if state.is_some() {
        return Err(failure("Encerre a sessão atual primeiro"));
    }
    let session = start_client(&invitation).await?;
    let status = session.status.clone();
    *state = Some(session);
    Ok(status)
}
#[tauri::command]
pub async fn stop_session() -> AppResult<()> {
    if let Some(session) = SESSION.lock().await.take() {
        session.stop().await;
    }
    Ok(())
}
#[tauri::command]
pub async fn tunnel_status() -> AppResult<Option<TunnelStatus>> {
    let mut state = SESSION.lock().await;
    if state
        .as_ref()
        .is_some_and(|session| session.task.is_finished())
    {
        if let Some(session) = state.take() {
            session.stop().await;
        }
    }
    Ok(state.as_ref().map(|session| {
        let mut s = session.status.clone();
        if let Some(connection) = &session.connection {
            if let Some(path) = connection.paths().iter().find(|p| p.is_selected()) {
                s.transport = if path.is_relay() {
                    "Relay criptografado"
                } else {
                    "Direto P2P"
                }
                .into();
            }
        }
        s.ping_ms = session.connection.as_ref().and_then(|c| {
            c.paths()
                .iter()
                .find(|p| p.is_selected())
                .map(|p| p.rtt().as_millis() as u64)
        });
        s
    }))
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    #[tokio::test]
    async fn encrypted_tunnel_transfers_bytes_and_stops() {
        let listener = TcpListener::bind(("127.0.0.1", 0)).await.unwrap();
        let port = listener.local_addr().unwrap().port();
        let echo = tokio::spawn(async move {
            let mut jobs = JoinSet::new();
            loop {
                let (mut tcp, _) = listener.accept().await.unwrap();
                jobs.spawn(async move {
                    let (mut r, mut w) = tcp.split();
                    let _ = tokio::io::copy(&mut r, &mut w).await;
                });
            }
        });
        assert!(start_host(0).await.is_err());
        let host = start_host(port).await.unwrap();
        let invitation = host.status.invitation.clone().unwrap();
        let mut invalid = parse_invitation(&invitation).unwrap();
        invalid.secret[0] ^= 1;
        let wrong = format!(
            "{PREFIX}{}",
            URL_SAFE_NO_PAD.encode(serde_json::to_vec(&invalid).unwrap())
        );
        assert!(start_client(&wrong).await.is_err());
        let client = start_client(&invitation).await.unwrap();
        let local = client.status.local_address.clone().unwrap();
        let mut sockets = JoinSet::new();
        for byte in 0..3u8 {
            let local = local.clone();
            sockets.spawn(async move {
                let mut tcp = TcpStream::connect(local).await.unwrap();
                let payload = vec![byte; 1024 * 1024];
                let (mut read, mut write) = tcp.split();
                let (sent, received) = tokio::join!(
                    async {
                        write.write_all(&payload).await.unwrap();
                        write.shutdown().await.unwrap();
                    },
                    async {
                        let mut bytes = Vec::new();
                        read.read_to_end(&mut bytes).await.unwrap();
                        bytes
                    }
                );
                let _ = sent;
                assert_eq!(received, payload);
            });
        }
        while let Some(result) = tokio::time::timeout(Duration::from_secs(20), sockets.join_next())
            .await
            .unwrap()
        {
            result.unwrap();
        }
        client.stop().await;
        assert!(TcpStream::connect(local).await.is_err());
        host.stop().await;
        echo.abort();
    }
    #[tokio::test]
    #[ignore = "uses public relay infrastructure"]
    async fn relay_only_transfers_without_direct_ip_transport() {
        let listener = TcpListener::bind(("127.0.0.1", 0)).await.unwrap();
        let port = listener.local_addr().unwrap().port();
        let echo = tokio::spawn(async move {
            let mut jobs = JoinSet::new();
            loop {
                let (mut tcp, _) = listener.accept().await.unwrap();
                jobs.spawn(async move {
                    let (mut r, mut w) = tcp.split();
                    let _ = tokio::io::copy(&mut r, &mut w).await;
                });
            }
        });
        let host_ep = Endpoint::builder(presets::N0)
            .clear_ip_transports()
            .alpns(vec![ALPN.to_vec()])
            .bind()
            .await
            .unwrap();
        let host = start_host_with(port, host_ep).await.unwrap();
        let client_ep = Endpoint::builder(presets::N0)
            .clear_ip_transports()
            .bind()
            .await
            .unwrap();
        let invite = parse_invitation(host.status.invitation.as_ref().unwrap()).unwrap();
        let client = start_client_with(invite, client_ep).await.unwrap();
        let mut socket = TcpStream::connect(client.status.local_address.as_ref().unwrap())
            .await
            .unwrap();
        socket.write_all(b"luxmc relay verified").await.unwrap();
        let mut received = [0; 20];
        tokio::time::timeout(Duration::from_secs(15), socket.read_exact(&mut received))
            .await
            .unwrap()
            .unwrap();
        assert_eq!(&received, b"luxmc relay verified");
        assert!(client
            .connection
            .as_ref()
            .unwrap()
            .paths()
            .iter()
            .any(|p| p.is_selected() && p.is_relay()));
        drop(socket);
        client.stop().await;
        host.stop().await;
        echo.abort();
    }
    #[test]
    fn rejects_malformed_and_oversized_invitations() {
        assert!(parse_invitation("abc").is_err());
        assert!(parse_invitation(&"x".repeat(4097)).is_err());
    }

    #[test]
    fn lan_advertisement_uses_the_local_tunnel_port() {
        assert_eq!(
            lan_advertisement(42875),
            "[MOTD]§b[Luxmc] §fMundo de Amigo[/MOTD][AD]42875[/AD]"
        );
    }
}
