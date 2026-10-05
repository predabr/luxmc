use crate::error::{AppError, AppResult};
use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
use iroh::{endpoint::presets, Endpoint, EndpointAddr};
use serde::{Deserialize, Serialize};
use std::{
    collections::{BTreeMap, HashSet},
    sync::{Arc, LazyLock},
    time::{Duration, SystemTime, UNIX_EPOCH},
};
use tokio::{
    net::{TcpListener, TcpStream, UdpSocket},
    sync::Mutex,
    task::{JoinHandle, JoinSet},
};

const ALPN: &[u8] = b"luxmc/minecraft-tunnel/2";
const ROOM_CAPACITY: usize = 10;
static CLIENT_KEY: LazyLock<iroh::SecretKey> = LazyLock::new(iroh::SecretKey::generate);
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

#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RoomIdentity { pub username: String, pub uuid: String, pub avatar_url: Option<String> }
impl Default for RoomIdentity {
    fn default() -> Self { Self { username: "Jogador".into(), uuid: String::new(), avatar_url: None } }
}
fn valid_avatar(value: &str) -> bool {
    if value.len() > 2048 { return false; }
    if let Some(encoded) = value.strip_prefix("data:image/png;base64,") {
        let Ok(bytes) = base64::engine::general_purpose::STANDARD.decode(encoded) else { return false; };
        if bytes.len() < 24 || &bytes[..8] != b"\x89PNG\r\n\x1a\n" { return false; }
        let width = u32::from_be_bytes(bytes[16..20].try_into().unwrap());
        let height = u32::from_be_bytes(bytes[20..24].try_into().unwrap());
        return (1..=64).contains(&width) && (1..=64).contains(&height);
    }
    url::Url::parse(value).is_ok_and(|url| url.scheme() == "https" && url.host_str().is_some() && url.username().is_empty() && url.password().is_none())
}
impl RoomIdentity {
    fn validate(self) -> AppResult<Self> {
        if self.username.trim().is_empty() || self.username.chars().count() > 32 || self.username.chars().any(char::is_control) || self.uuid.len() > 64 { return Err(failure("Identidade da conta inválida")); }
        let avatar_url = self.avatar_url.filter(|url| valid_avatar(url));
        Ok(Self { username: self.username.trim().into(), avatar_url, ..self })
    }
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TunnelMember { pub id: String, pub username: String, pub uuid: String, pub avatar_url: Option<String>, pub joined_at: u64, pub is_host: bool }
impl TunnelMember {
    fn new(id: String, identity: RoomIdentity, is_host: bool) -> Self { Self { id, username: identity.username, uuid: identity.uuid, avatar_url: identity.avatar_url, joined_at: now(), is_host } }
}

#[derive(Default)]
struct RoomRegistry {
    members: BTreeMap<String, TunnelMember>,
    connections: BTreeMap<String, iroh::endpoint::Connection>,
    removed: HashSet<String>,
    locked: bool,
    secret: [u8; 32],
    expires: u64,
    host: Option<TunnelMember>,
    room_code: String,
}
impl RoomRegistry {
    fn snapshot(&self) -> RoomSnapshot {
        RoomSnapshot { room_code: self.room_code.clone(), members: self.host.iter().cloned().chain(self.members.values().cloned()).collect(), max_players: ROOM_CAPACITY, room_locked: self.locked }
    }
    fn admission(&self, id: &str) -> Result<(), u8> {
        if self.removed.contains(id) { Err(4) }
        else if self.locked { Err(3) }
        else if self.members.len() >= ROOM_CAPACITY - 1 || self.members.contains_key(id) { Err(2) }
        else { Ok(()) }
    }
    fn remove(&mut self, id: &str) { self.members.remove(id); self.connections.remove(id); }
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct RoomSnapshot { room_code: String, members: Vec<TunnelMember>, max_players: usize, room_locked: bool }

async fn read_room_snapshot(connection: &iroh::endpoint::Connection, secret: [u8; 32]) -> AppResult<RoomSnapshot> {
    let (mut send, mut recv) = connection.open_bi().await.map_err(failure)?;
    let mut request = [3u8; 33];
    request[1..].copy_from_slice(&secret);
    send.write_all(&request).await.map_err(failure)?;
    let _ = send.finish();
    let mut length = [0u8; 2];
    recv.read_exact(&mut length).await.map_err(failure)?;
    let length = u16::from_be_bytes(length) as usize;
    if length == 0 || length > 32768 { return Err(failure("Resposta da sala inválida")); }
    let mut bytes = vec![0; length];
    recv.read_exact(&mut bytes).await.map_err(failure)?;
    let mut snapshot: RoomSnapshot = serde_json::from_slice(&bytes)?;
    if snapshot.members.len() > ROOM_CAPACITY || snapshot.max_players != ROOM_CAPACITY || snapshot.room_code.len() > 32 { return Err(failure("Participantes da sala inválidos")); }
    for member in &mut snapshot.members {
        if member.id.len() > 128 { return Err(failure("Participante inválido")); }
        let identity = RoomIdentity { username: member.username.clone(), uuid: member.uuid.clone(), avatar_url: member.avatar_url.clone() }.validate()?;
        member.username = identity.username; member.avatar_url = identity.avatar_url;
    }
    Ok(snapshot)
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
    pub members: Vec<TunnelMember>,
    pub max_players: usize,
    pub room_locked: bool,
}

pub struct TunnelSession {
    endpoint: Endpoint,
    task: JoinHandle<()>,
    lan_broadcast: Option<JoinHandle<()>>,
    connection: Option<iroh::endpoint::Connection>,
    status: TunnelStatus,
    room: Option<Arc<Mutex<RoomRegistry>>>,
    public_code: Option<String>,
    client_room: Option<Arc<Mutex<RoomSnapshot>>>,
    room_poll: Option<JoinHandle<()>>,
}
impl Drop for TunnelSession {
    fn drop(&mut self) {
        self.task.abort();
        if let Some(task) = &self.room_poll { task.abort(); }
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
        if let Some(task) = &self.room_poll { task.abort(); }
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
    start_host_as(port, endpoint, RoomIdentity::default()).await
}

async fn start_host_as(port: u16, endpoint: Endpoint, identity: RoomIdentity) -> AppResult<TunnelSession> {
    let identity = identity.validate()?;
    let host_member = TunnelMember::new(endpoint.id().to_string(), identity, true);
    let room = Arc::new(Mutex::new(RoomRegistry::default()));
    let registry = room.clone();
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
    { let mut room = room.lock().await; room.secret = secret; room.expires = expires; }
    let invitation_payload = format!(
        "{PREFIX}{}",
        URL_SAFE_NO_PAD.encode(serde_json::to_vec(&Invitation {
            version: 2,
            addr: endpoint.addr(),
            secret,
            expires
        })?)
    );
    let room_code = format!("LUX-{:04X}", rand::random::<u16>());
    { let mut room = room.lock().await; room.host = Some(host_member.clone()); room.room_code = room_code.clone(); }
    let invitation = format!("{room_code}|{invitation_payload}");
    let ep = endpoint.clone();
    let task = tokio::spawn(async move {
        let mut clients = JoinSet::new();
        loop {
            tokio::select! {
                Some(_) = clients.join_next(), if !clients.is_empty() => {},
                incoming = ep.accept() => {
                    let Some(incoming) = incoming else { break };
                    if clients.len() >= 24 { incoming.refuse(); continue; }
                    let registry = registry.clone();
                    clients.spawn(async move {
                        let Ok(Ok(conn)) = tokio::time::timeout(Duration::from_secs(10), incoming).await else { return };
                        let id = conn.remote_id().to_string();
                        let mut admitted = false;
                        let admission = async {
                            let (mut send, mut recv) = conn.accept_bi().await.map_err(failure)?;
                            let mut handshake = [0u8; 33];
                            recv.read_exact(&mut handshake).await.map_err(failure)?;
                            let control = registry.lock().await;
                            let accepted_secret = control.secret;
                            let difference = handshake[1..].iter().zip(accepted_secret).fold(0u8, |acc, (a,b)| acc | (*a ^ b));
                            let rejection = if handshake[0] != 2 { Some(7) } else if difference != 0 { Some(6) } else if now() >= control.expires { Some(5) } else { None };
                            if let Some(code) = rejection {
                                drop(control);
                                send.write_all(&[code]).await.map_err(failure)?;
                                let _ = send.finish();
                                let _ = tokio::time::timeout(Duration::from_secs(1), send.stopped()).await;
                                return Err(failure("Entrada recusada"));
                            }
                            drop(control);
                            let mut length = [0u8; 2];
                            recv.read_exact(&mut length).await.map_err(failure)?;
                            let length = u16::from_be_bytes(length) as usize;
                            if length == 0 || length > 4096 { return Err(failure("Identidade inválida")); }
                            let mut bytes = vec![0; length];
                            recv.read_exact(&mut bytes).await.map_err(failure)?;
                            let identity: RoomIdentity = serde_json::from_slice(&bytes)?;
                            let identity = identity.validate()?;
                            let mut room = registry.lock().await;
                            if let Err(code) = room.admission(&id) {
                                drop(room);
                                let _ = send.write_all(&[code]).await;
                                let _ = send.finish();
                                return Err(failure("Entrada recusada"));
                            }
                            room.members.insert(id.clone(), TunnelMember::new(id.clone(), identity, false));
                            room.connections.insert(id.clone(), conn.clone());
                            admitted = true;
                            drop(room);
                            send.write_all(&[1]).await.map_err(failure)?;
                            let _ = send.finish();
                            Ok::<_, AppError>(accepted_secret)
                        };
                        let accepted_secret = match tokio::time::timeout(Duration::from_secs(6), admission).await {
                            Ok(Ok(secret)) => secret,
                            _ => {
                                if admitted { registry.lock().await.remove(&id); }
                                conn.close(1u8.into(), b"admission denied");
                                return;
                            }
                        };
                        let mut streams = JoinSet::new();
                        loop {
                            tokio::select! {
                                _ = conn.closed() => break,
                                Some(_) = streams.join_next(), if !streams.is_empty() => {},
                                stream = conn.accept_bi() => {
                                    let Ok((mut send, mut recv)) = stream else { break };
                                    if streams.len() >= 16 { conn.close(1u8.into(), b"stream limit"); break; }
                                    let registry = registry.clone();
                                    streams.spawn(async move {
                                        let mut handshake = [0u8; 33];
                                        let Ok(Ok(_)) = tokio::time::timeout(Duration::from_secs(5), recv.read_exact(&mut handshake)).await else { return };
                                        let difference = handshake[1..].iter().zip(accepted_secret).fold(0u8, |acc, (a,b)| acc | (*a ^ b));
                                        if difference != 0 { return; }
                                        if handshake[0] == 3 {
                                            let snapshot = registry.lock().await.snapshot();
                                            let Ok(bytes) = serde_json::to_vec(&snapshot) else { return };
                                            if bytes.len() > 32768 { return; }
                                            if send.write_all(&(bytes.len() as u16).to_be_bytes()).await.is_err() { return; }
                                            let _ = send.write_all(&bytes).await;
                                            let _ = send.finish();
                                            return;
                                        }
                                        if handshake[0] != 1 { return; }
                                        let Ok(Ok(tcp)) = tokio::time::timeout(Duration::from_secs(5), TcpStream::connect(("127.0.0.1", port))).await else { return };
                                        if send.write_all(&[1]).await.is_err() { return; }
                                        let _ = bridge(tcp, send, recv).await;
                                    });
                                }
                            }
                        }
                        streams.abort_all();
                        registry.lock().await.remove(&id);
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
            members: vec![host_member], max_players: ROOM_CAPACITY, room_locked: false,
        },
        room: Some(room),
        public_code: None, client_room: None, room_poll: None,
    })
}

fn parse_invitation(input: &str) -> AppResult<Invitation> {
    if input.len() > 4096 {
        return Err(AppError::InvalidInput("Convite muito longo".into()));
    }
    let link;
    let input = input.trim();
    let input = if input.to_ascii_lowercase().starts_with("luxmc://") {
        let url = url::Url::parse(input).map_err(failure)?;
        if url.host_str() != Some("join") || url.path() != "/world" { return Err(failure("Link de sala inválido")); }
        link = url.query_pairs().find(|(key, _)| key == "invitation").map(|(_, value)| value.into_owned()).ok_or_else(|| failure("O link não contém o convite completo"))?;
        link.as_str()
    } else { input };
    let input = input.rsplit_once('|').map(|(_, value)| value).unwrap_or(input);
    let data =
        URL_SAFE_NO_PAD
            .decode(input.strip_prefix(PREFIX).ok_or_else(|| {
                AppError::InvalidInput("Cole o convite completo luxmc-world:".into())
            })?)
            .map_err(failure)?;
    let invitation: Invitation = serde_json::from_slice(&data)?;
    if invitation.version != 2 { return Err(failure("Atualize o Luxmc nos dois computadores para usar esta sala")); }
    if invitation.expires == 0 {
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
    start_client_as(invite, endpoint, RoomIdentity::default()).await
}

async fn start_client_as(invite: Invitation, endpoint: Endpoint, identity: RoomIdentity) -> AppResult<TunnelSession> {
    let identity = identity.validate()?;
    let self_member = TunnelMember::new(endpoint.id().to_string(), identity.clone(), false);
    let setup = async {
        let conn = endpoint.connect(invite.addr, ALPN).await.map_err(failure)?;
        let (mut send, mut recv) = conn.open_bi().await.map_err(failure)?;
        let mut handshake = [2u8; 33];
        handshake[1..].copy_from_slice(&invite.secret);
        send.write_all(&handshake).await.map_err(failure)?;
        let metadata = serde_json::to_vec(&identity)?;
        send.write_all(&(metadata.len() as u16).to_be_bytes()).await.map_err(failure)?;
        send.write_all(&metadata).await.map_err(failure)?;
        let _ = send.finish();
        let mut accepted = [0];
        recv.read_exact(&mut accepted).await.map_err(failure)?;
        if accepted != [1] {
            return Err(failure(match accepted[0] { 2 => "Sala lotada: máximo de 10 pessoas, incluindo o hospedeiro", 3 => "O hospedeiro fechou a sala para novas entradas", 4 => "Você foi removido desta sala", 5 => "Este convite expirou. Peça ao hospedeiro um convite renovado", 6 => "Este convite foi substituído. Peça ao hospedeiro o link atual da sala", 7 => "Atualize o Luxmc nos dois computadores para usar esta sala", _ => "Convite recusado" }));
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
    let client_room = Arc::new(Mutex::new(RoomSnapshot { room_code: String::new(), members: vec![self_member], max_players: ROOM_CAPACITY, room_locked: false }));
    let polling_room = client_room.clone();
    let polling_connection = conn.clone();
    let secret = invite.secret;
    let room_poll = tokio::spawn(async move {
        let mut interval = tokio::time::interval(Duration::from_secs(2));
        loop {
            tokio::select! {
                _ = polling_connection.closed() => break,
                _ = interval.tick() => {
                    if let Ok(Ok(snapshot)) = tokio::time::timeout(Duration::from_secs(2), read_room_snapshot(&polling_connection, secret)).await { *polling_room.lock().await = snapshot; }
                }
            }
        }
    });
    let lan_broadcast = start_lan_broadcast(local_port);
    let connection = conn.clone();
    let ep = endpoint.clone();
    let task = tokio::spawn(async move {
        let mut streams = JoinSet::new();
        loop {
            tokio::select! {
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
            members: Vec::new(), max_players: ROOM_CAPACITY, room_locked: false,
        },
        room: None,
        public_code: None, client_room: Some(client_room), room_poll: Some(room_poll),
    })
}

#[tauri::command]
pub async fn host_world(port: u16, identity: Option<RoomIdentity>) -> AppResult<TunnelStatus> {
    let mut state = SESSION.lock().await;
    if state.is_some() {
        return Err(failure("Encerre a sessão atual primeiro"));
    }
    let mut session = start_host_as(port, endpoint().await?, identity.unwrap_or_default()).await?;
    let code = crate::core::share_codes::publish("room", session.status.invitation.as_deref().unwrap_or("")).await?;
    session.public_code = Some(code.clone());
    session.status.room_code = Some(code.clone());
    if let Some(room) = &session.room { room.lock().await.room_code = code.clone(); }
    let mut status = session.status.clone();
    status.invitation = Some(code);
    *state = Some(session);
    Ok(status)
}
#[tauri::command]
pub async fn join_world(invitation: String, identity: Option<RoomIdentity>) -> AppResult<TunnelStatus> {
    let mut state = SESSION.lock().await;
    if state.is_some() {
        return Err(failure("Encerre a sessão atual primeiro"));
    }
    let input = invitation.trim();
    let linked = url::Url::parse(input).ok().filter(|url| url.scheme() == "luxmc" && url.host_str() == Some("join") && url.path() == "/world").and_then(|url| url.query_pairs().find(|(key,_)| key == "invitation").map(|(_,value)| value.into_owned()));
    let code = linked.as_deref().unwrap_or(input).to_uppercase();
    let invitation = if crate::core::share_codes::is_short_code(&code) { crate::core::share_codes::resolve(&code, "room").await? } else { invitation };
    let ep = Endpoint::builder(presets::N0).secret_key(CLIENT_KEY.clone()).alpns(vec![ALPN.to_vec()]).bind().await.map_err(failure)?;
    let session = start_client_as(parse_invitation(&invitation)?, ep, identity.unwrap_or_default()).await?;
    let status = session.status.clone();
    *state = Some(session);
    Ok(status)
}
fn saved_server_bytes(existing: Option<&[u8]>, address: &str) -> AppResult<Vec<u8>> {
    use fastnbt::Value;
    use std::collections::HashMap;
    let mut root: HashMap<String, Value> = match existing { Some(bytes) => fastnbt::from_bytes(bytes).map_err(failure)?, None => HashMap::new() };
    let servers = root.entry("servers".into()).or_insert_with(|| Value::List(Vec::new()));
    let Value::List(servers) = servers else { return Err(failure("A lista de servidores do Minecraft está inválida e foi preservada")); };
    let name = "Luxmc · Mundo de amigo";
    let mut entry = None;
    for server in servers.iter_mut() {
        if let Value::Compound(fields) = server { if fields.get("name") == Some(&Value::String(name.into())) { entry = Some(fields); break; } }
    }
    if let Some(entry) = entry { entry.insert("ip".into(), Value::String(address.into())); }
    else { servers.push(Value::Compound(HashMap::from([("name".into(), Value::String(name.into())), ("ip".into(), Value::String(address.into()))]))); }
    fastnbt::to_bytes(&root).map_err(failure)
}

#[tauri::command]
pub async fn tunnel_save_server(profile_id: String) -> AppResult<String> {
    let address = {
        let state = SESSION.lock().await;
        state.as_ref().filter(|session| session.status.mode == "client").and_then(|session| session.status.local_address.clone()).ok_or_else(|| failure("Entre na sala antes de adicionar o mundo ao Minecraft"))?
    };
    let profile = crate::commands::profiles::profiles_get(profile_id).await?;
    let directory = std::path::PathBuf::from(profile.game_dir);
    tokio::fs::create_dir_all(&directory).await?;
    let directory = tokio::fs::canonicalize(directory).await?;
    let path = directory.join("servers.dat");
    let existing = match tokio::fs::metadata(&path).await {
        Ok(metadata) => {
            if metadata.len() > 2 * 1024 * 1024 || !tokio::fs::canonicalize(&path).await?.starts_with(&directory) { return Err(failure("A lista de servidores existente foi preservada")); }
            Some(tokio::fs::read(&path).await?)
        }
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => None,
        Err(error) => return Err(error.into()),
    };
    let bytes = saved_server_bytes(existing.as_deref(), &address)?;
    if existing.as_deref() == Some(bytes.as_slice()) { return Ok(address); }
    if let Some(previous) = &existing { tokio::fs::write(directory.join("servers.dat.luxmc.bak"), previous).await?; }
    let temporary = directory.join(format!("servers.dat.luxmc-{}.tmp", uuid::Uuid::new_v4()));
    tokio::fs::write(&temporary, bytes).await?;
    if let Err(error) = tokio::fs::rename(&temporary, &path).await { let _ = tokio::fs::remove_file(temporary).await; return Err(error.into()); }
    Ok(address)
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
    let room_state = if let Some(session) = state.as_ref() {
        if let Some(room) = &session.room { let room = room.lock().await; Some((room.members.values().cloned().collect::<Vec<_>>(), room.locked)) } else { None }
    } else { None };
    let client_state = if let Some(session) = state.as_ref() {
        if let Some(room) = &session.client_room { Some(room.lock().await.clone()) } else { None }
    } else { None };
    Ok(state.as_ref().map(|session| {
        let mut s = session.status.clone();
        if let Some(code) = &session.public_code { s.invitation = Some(code.clone()); }
        if let Some(room) = &client_state { s.members = room.members.clone(); s.room_code = (!room.room_code.is_empty()).then(|| room.room_code.clone()); s.max_players = room.max_players; s.room_locked = room.room_locked; }
        if let Some((members, locked)) = &room_state { s.members.extend(members.iter().cloned()); s.room_locked = *locked; }
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

#[tauri::command]
pub async fn tunnel_kick_member(member_id: String) -> AppResult<()> {
    let state = SESSION.lock().await;
    let room = state.as_ref().and_then(|session| session.room.as_ref()).ok_or_else(|| failure("Somente o hospedeiro pode remover participantes"))?;
    let mut room = room.lock().await;
    let conn = room.connections.remove(&member_id).ok_or_else(|| failure("Participante não está mais conectado"))?;
    room.members.remove(&member_id);
    room.removed.insert(member_id);
    conn.close(2u8.into(), b"removed by host");
    Ok(())
}

#[tauri::command]
pub async fn tunnel_set_locked(locked: bool) -> AppResult<()> {
    let state = SESSION.lock().await;
    let room = state.as_ref().and_then(|session| session.room.as_ref()).ok_or_else(|| failure("Somente o hospedeiro pode controlar a sala"))?;
    room.lock().await.locked = locked;
    Ok(())
}

#[tauri::command]
pub async fn tunnel_refresh_invitation() -> AppResult<TunnelStatus> {
    let mut state = SESSION.lock().await;
    let session = state.as_mut().filter(|session| session.room.is_some()).ok_or_else(|| failure("Somente o hospedeiro pode renovar o convite"))?;
    let mut secret = [0u8; 32];
    secret[..16].copy_from_slice(uuid::Uuid::new_v4().as_bytes());
    secret[16..].copy_from_slice(uuid::Uuid::new_v4().as_bytes());
    let expires = now() + 3600;
    let payload = URL_SAFE_NO_PAD.encode(serde_json::to_vec(&Invitation { version: 2, addr: session.endpoint.addr(), secret, expires })?);
    let raw = format!("{}{PREFIX}{payload}", session.status.room_code.as_deref().map(|code| format!("{code}|")).unwrap_or_default());
    let public = if session.public_code.is_some() { Some(crate::core::share_codes::publish("room", &raw).await?) } else { None };
    let mut room = session.room.as_ref().unwrap().lock().await;
    if let Some(code) = &public { room.room_code = code.clone(); session.status.room_code = Some(code.clone()); session.public_code = Some(code.clone()); }
    room.secret = secret;
    room.expires = expires;
    session.status.expires_at = expires;
    session.status.invitation = Some(format!("{}|{PREFIX}{payload}", session.status.room_code.as_deref().unwrap_or("LUX")));
    let mut status = session.status.clone();
    if let Some(code) = &session.public_code { status.invitation = Some(code.clone()); }
    status.members.extend(room.members.values().cloned());
    status.room_locked = room.locked;
    Ok(status)
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
        *SESSION.lock().await = Some(host);
        let guest_identity = RoomIdentity { username: "Alex".into(), uuid: "alex-uuid".into(), avatar_url: Some("https://example.com/alex.png".into()) };
        let mut different_clock_invite = parse_invitation(&invitation).unwrap();
        different_clock_invite.expires = 1;
        let client = start_client_as(different_clock_invite, endpoint().await.unwrap(), guest_identity).await.unwrap();
        let local = client.status.local_address.clone().unwrap();
        let status = tunnel_status().await.unwrap().unwrap();
        assert_eq!(status.members.len(), 2);
        assert_eq!(status.max_players, 10);
        assert!(status.members.iter().any(|member| member.username == "Alex" && !member.is_host));
        tunnel_set_locked(true).await.unwrap();
        assert!(start_client(&invitation).await.is_err());
        assert!(tunnel_status().await.unwrap().unwrap().room_locked);
        tunnel_set_locked(false).await.unwrap();
        let mut tcp = TcpStream::connect(&local).await.unwrap();
        let payload = vec![42u8; 1024 * 1024];
        let (mut read, mut write) = tcp.split();
        let mut received = vec![0; payload.len()];
        tokio::time::timeout(Duration::from_secs(20), async {
            let (sent, echoed) = tokio::join!(write.write_all(&payload), read.read_exact(&mut received));
            sent.unwrap(); echoed.unwrap();
        }).await.unwrap();
        assert_eq!(received, payload);
        let mut extra = TcpStream::connect(&local).await.unwrap();
        extra.write_all(b"status and login").await.unwrap();
        let mut extra_bytes = [0; 16];
        tokio::time::timeout(Duration::from_secs(10), extra.read_exact(&mut extra_bytes)).await.unwrap().unwrap();
        assert_eq!(&extra_bytes, b"status and login");
        drop(extra);
        let snapshot = tokio::time::timeout(Duration::from_secs(3), read_room_snapshot(client.connection.as_ref().unwrap(), parse_invitation(&invitation).unwrap().secret)).await.unwrap().unwrap();
        assert_eq!(snapshot.members.len(), 2);
        assert_eq!(snapshot.room_code, status.room_code.unwrap());
        let renewed = tunnel_refresh_invitation().await.unwrap();
        assert_ne!(renewed.invitation.as_ref().unwrap(), &invitation);
        assert!(start_client(&invitation).await.is_err());
        tcp.write_all(b"still connected").await.unwrap();
        let mut echo_bytes = [0; 15];
        tokio::time::timeout(Duration::from_secs(10), tcp.read_exact(&mut echo_bytes)).await.unwrap().unwrap();
        assert_eq!(&echo_bytes, b"still connected");
        let member_id = client.endpoint.id().to_string();
        tunnel_kick_member(member_id.clone()).await.unwrap();
        assert_eq!(tunnel_status().await.unwrap().unwrap().members.len(), 1);
        let mut ended = Vec::new();
        let _ = tokio::time::timeout(Duration::from_secs(10), tcp.read_to_end(&mut ended)).await.unwrap();
        assert!(ended.is_empty());
        let room = SESSION.lock().await.as_ref().unwrap().room.clone().unwrap();
        assert_eq!(room.lock().await.admission(&member_id), Err(4));
        client.stop().await;
        assert!(TcpStream::connect(local).await.is_err());
        room.lock().await.expires = 0;
        assert!(start_client(renewed.invitation.as_ref().unwrap()).await.err().unwrap().to_string().contains("expirou"));
        stop_session().await.unwrap();
        assert!(tunnel_status().await.unwrap().is_none());
        echo.abort();
    }

    #[test]
    fn room_capacity_counts_host_and_rejects_duplicates_removed_and_locked_members() {
        let mut room = RoomRegistry::default();
        for index in 0..9 {
            let id = format!("peer-{index}");
            assert!(room.admission(&id).is_ok());
            room.members.insert(id.clone(), TunnelMember::new(id, RoomIdentity::default(), false));
        }
        assert_eq!(room.admission("eleventh-person"), Err(2));
        assert_eq!(room.admission("peer-0"), Err(2));
        room.remove("peer-0");
        assert!(room.admission("new-guest").is_ok());
        room.removed.insert("peer-0".into());
        assert_eq!(room.admission("peer-0"), Err(4));
        room.locked = true;
        assert_eq!(room.admission("new-guest"), Err(3));
    }

    #[test]
    fn validates_account_metadata_without_accepting_script_avatar_urls() {
        let avatar = "data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAQAAAC1HAwCAAAAC0lEQVR42mP8/x8AAwMCAO+aX1cAAAAASUVORK5CYII=";
        assert!(valid_avatar(avatar));
        assert!(!valid_avatar("data:image/svg+xml,<svg/>"));
        assert!(!valid_avatar("https://user:secret@example.com/avatar.png"));
        assert!(RoomIdentity { username: "\n".into(), ..RoomIdentity::default() }.validate().is_err());
        assert!(RoomIdentity { username: "x".repeat(33), ..RoomIdentity::default() }.validate().is_err());
        assert!(RoomIdentity { avatar_url: Some("javascript:alert(1)".into()), ..RoomIdentity::default() }.validate().unwrap().avatar_url.is_none());
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
    fn invitation_accepts_links_and_different_computer_clocks() {
        let key = iroh::SecretKey::generate();
        for expires in [1, now() + 172800] {
            let invite = Invitation { version: 2, addr: EndpointAddr::new(key.public()), secret: [8; 32], expires };
            let raw = format!("LUX-ABCD|{PREFIX}{}", URL_SAFE_NO_PAD.encode(serde_json::to_vec(&invite).unwrap()));
            assert_eq!(parse_invitation(&raw).unwrap().expires, expires);
            let mut link = url::Url::parse("luxmc://join/world").unwrap();
            link.query_pairs_mut().append_pair("invitation", &raw);
            assert_eq!(parse_invitation(link.as_str()).unwrap().secret, [8; 32]);
        }
    }

    #[test]
    fn saves_local_world_without_losing_existing_servers_or_metadata() {
        use fastnbt::Value;
        use std::collections::HashMap;
        let original: HashMap<String, Value> = HashMap::from([("custom".into(), Value::Int(42)), ("servers".into(), Value::List(vec![Value::Compound(HashMap::from([("name".into(), Value::String("Meu servidor".into())), ("ip".into(), Value::String("example.org".into()))]))]))]);
        let bytes = fastnbt::to_bytes(&original).unwrap();
        let added = saved_server_bytes(Some(&bytes), "127.0.0.1:54321").unwrap();
        let updated = saved_server_bytes(Some(&added), "127.0.0.1:54322").unwrap();
        let root: HashMap<String, Value> = fastnbt::from_bytes(&updated).unwrap();
        assert_eq!(root.get("custom"), Some(&Value::Int(42)));
        let Value::List(servers) = &root["servers"] else { panic!("missing servers") };
        assert_eq!(servers.len(), 2);
        let Value::Compound(saved) = &servers[1] else { panic!("missing entry") };
        assert_eq!(saved["ip"], Value::String("127.0.0.1:54322".into()));
        assert!(saved_server_bytes(Some(b"invalid"), "127.0.0.1:1").is_err());
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
