use crate::error::{AppError, AppResult};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{collections::BTreeMap, net::{Ipv4Addr, SocketAddrV4}, sync::{Arc, atomic::{AtomicU16, Ordering}}, time::Duration};
use tokio::{io::{AsyncBufReadExt, AsyncWriteExt, BufReader}, net::{TcpListener, TcpStream, UdpSocket}, sync::RwLock, task::{JoinHandle, JoinSet}};

pub const CONTROL_PORT: u16 = 39945;
pub const GAME_PORT: u16 = 39946;
static MANUAL_PORT: AtomicU16 = AtomicU16::new(0);
static STATE: RwLock<WorldState> = RwLock::const_new(WorldState { worlds: Vec::new(), local_world: None, error: None });

#[derive(Clone, Serialize, Deserialize)]
pub struct PublishedWorld { pub motd: String, pub port: u16 }

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LanWorld { pub id: String, pub owner: String, pub name: String, pub local_address: String, pub version: String, pub latency_ms: Option<u32>, pub available: bool }

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WorldState { pub worlds: Vec<LanWorld>, pub local_world: Option<PublishedWorld>, pub error: Option<String> }

#[tauri::command]
pub async fn virtual_lan_worlds() -> WorldState { STATE.read().await.clone() }

#[tauri::command]
pub async fn virtual_lan_prepare_world(id: String) -> AppResult<String> {
    let world = STATE.read().await.worlds.iter().find(|world| world.id == id).cloned()
        .ok_or_else(|| AppError::InvalidState("Esse mundo não está mais na sala. Aguarde o amigo abrir o mundo novamente.".into()))?;
    let address: std::net::SocketAddr = world.local_address.parse().map_err(|_| AppError::InvalidState("Endereço do mundo inválido".into()))?;
    minecraft(address.ip().to_string(), address.port()).await.map_err(|_| AppError::InvalidState("A conexão com o mundo está se recuperando. Mantenha os dois launchers abertos e tente novamente em alguns segundos.".into()))?;
    Ok(world.local_address)
}

#[tauri::command]
pub async fn virtual_lan_world_port(port: u16) -> AppResult<()> {
    if port == 0 { return Err(AppError::InvalidInput("Use a porta informada ao abrir o mundo para LAN".into())); }
    minecraft("127.0.0.1".into(), port).await?;
    MANUAL_PORT.store(port, Ordering::Release);
    Ok(())
}

pub async fn reset() {
    MANUAL_PORT.store(0, Ordering::Release);
    *STATE.write().await = WorldState { worlds: Vec::new(), local_world: None, error: None };
}

async fn minecraft(host: String, port: u16) -> AppResult<crate::core::server::ServerStatus> {
    let status = tokio::task::spawn_blocking(move || crate::core::server::ping(&host, port)).await.map_err(|error| AppError::Internal(error.to_string()))??;
    if !status.online { return Err(AppError::InvalidState("O Minecraft não respondeu nesta porta. Abra o mundo para LAN primeiro".into())); }
    Ok(status)
}

fn token(secret: &str) -> String { format!("{:x}", Sha256::digest(format!("{secret}:luxmc-world-v1"))) }
fn authorized(received: &str, expected: &str) -> bool {
    received.len() == expected.len() && received.bytes().zip(expected.bytes()).fold(0u8, |difference, (a, b)| difference | (a ^ b)) == 0
}

fn logged_port(text: &str) -> Option<u16> {
    text.lines().rev().find_map(|line| {
        ["Started serving on ", "Started a LAN server on port ", "Local game hosted on port "].iter().find_map(|marker| {
            let value = line.split_once(marker)?.1.split(|value: char| !value.is_ascii_digit()).next()?;
            value.parse::<u16>().ok().filter(|port| *port != 0)
        })
    })
}

fn active_port() -> Option<u16> {
    use std::io::{Read, Seek, SeekFrom};
    if crate::core::launcher::get_active_game_pid() == 0 { return None; }
    let path = crate::core::launcher::get_active_game_dir()?.join("logs").join("latest.log");
    let mut file = std::fs::File::open(path).ok()?;
    let start = file.metadata().ok()?.len().saturating_sub(65536);
    file.seek(SeekFrom::Start(start)).ok()?;
    let mut bytes = Vec::new(); file.take(65536).read_to_end(&mut bytes).ok()?;
    logged_port(&String::from_utf8_lossy(&bytes))
}

async fn request(address: &str, key: &str) -> AppResult<Option<PublishedWorld>> {
    tokio::time::timeout(Duration::from_secs(3), async {
        let mut socket = TcpStream::connect((address, CONTROL_PORT)).await?;
        socket.write_all(format!("{key}\n").as_bytes()).await?;
        let mut response = String::new();
        use tokio::io::AsyncReadExt;
        BufReader::new(socket).take(4096).read_line(&mut response).await?;
        let world: Option<PublishedWorld> = serde_json::from_str(&response)?;
        if let Some(world) = &world { if world.port == 0 || world.motd.len() > 1024 { return Err(AppError::InvalidState("Resposta de mundo inválida".into())); } }
        Ok(world)
    }).await.map_err(|_| AppError::InvalidState("O computador do amigo não respondeu. Confira se os dois estão na mesma rede do Luxmc".into()))?
}

fn advertise(port: u16, name: &str) -> String {
    let name: String = name.chars().filter(|value| !value.is_control()).take(140).collect::<String>().replace(['[', ']'], " ");
    format!("[MOTD]§b[Luxmc] §f{name}[/MOTD][AD]{port}[/AD]")
}

fn broadcast_socket() -> AppResult<UdpSocket> {
    let socket = socket2::Socket::new(socket2::Domain::IPV4, socket2::Type::DGRAM, Some(socket2::Protocol::UDP))?;
    socket.bind(&SocketAddrV4::new(Ipv4Addr::LOCALHOST, 0).into())?;
    socket.set_multicast_if_v4(&Ipv4Addr::LOCALHOST)?;
    socket.set_multicast_loop_v4(true)?;
    socket.set_multicast_ttl_v4(1)?;
    socket.set_nonblocking(true)?;
    Ok(UdpSocket::from_std(socket.into())?)
}

fn proxy(listener: TcpListener, remote: String, remote_port: u16, title: String) -> JoinHandle<()> {
    tokio::spawn(async move {
        let port = listener.local_addr().map(|value| value.port()).unwrap_or(0);
        let socket = broadcast_socket().ok();
        let mut interval = tokio::time::interval(Duration::from_millis(1500));
        let mut streams = JoinSet::new();
        loop {
            tokio::select! {
                _ = interval.tick() => { if let Some(socket) = &socket { let _ = socket.send_to(advertise(port, &title).as_bytes(), "224.0.2.60:4445").await; } }
                Some(_) = streams.join_next(), if !streams.is_empty() => {},
                accepted = listener.accept() => {
                    let Ok((mut client, _)) = accepted else { break; };
                    if streams.len() >= 16 { continue; }
                    let host = remote.clone();
                    streams.spawn(async move {
                        if let Ok(Ok(mut upstream)) = tokio::time::timeout(Duration::from_secs(5), TcpStream::connect((host.as_str(), remote_port))).await {
                            let _ = client.set_nodelay(true); let _ = upstream.set_nodelay(true);
                            let _ = tokio::io::copy_bidirectional(&mut client, &mut upstream).await;
                        }
                    });
                }
            }
        }
    })
}

struct Routes(BTreeMap<String, (LanWorld, JoinHandle<()>, tokio::time::Instant)>);
impl Routes {
    fn refresh(&mut self, found: &[String]) {
        self.0.retain(|id, (row, task, seen)| {
            row.available = found.contains(id);
            if !task.is_finished() && (row.available || seen.elapsed() < Duration::from_secs(120)) { true } else { task.abort(); false }
        });
    }
}
impl Drop for Routes { fn drop(&mut self) { for (_, task, _) in self.0.values() { task.abort(); } } }
struct Services(Vec<JoinHandle<()>>);
impl Drop for Services { fn drop(&mut self) { for task in &self.0 { task.abort(); } } }

async fn serve(address: &str, key: String, world: Arc<RwLock<Option<PublishedWorld>>>) -> AppResult<Services> {
    let control = TcpListener::bind((address, CONTROL_PORT)).await?;
    let game = TcpListener::bind((address, GAME_PORT)).await?;
    let control_world = world.clone();
    let controller = tokio::spawn(async move {
        let mut clients = JoinSet::new();
        loop { tokio::select! {
            Some(_) = clients.join_next(), if !clients.is_empty() => {},
            accepted = control.accept() => {
                let Ok((socket, _)) = accepted else { break; };
                if clients.len() >= 16 { continue; }
                let world = control_world.clone(); let key = key.clone();
                clients.spawn(async move {
                    let _ = tokio::time::timeout(Duration::from_secs(3), async {
                        use tokio::io::AsyncReadExt;
                        let mut reader = BufReader::new(socket).take(128);
                        let mut received = String::new(); reader.read_line(&mut received).await?;
                        if !authorized(received.trim(), &key) { return Ok::<(), std::io::Error>(()); }
                        let payload = serde_json::to_vec(&*world.read().await).unwrap_or_default();
                        let mut socket = reader.into_inner().into_inner();
                        socket.write_all(&payload).await?; socket.write_all(b"\n").await?;
                        Ok(())
                    }).await;
                });
            }
        } }
    });
    let bridge = tokio::spawn(async move {
        let mut clients = JoinSet::new();
        loop { tokio::select! {
            Some(_) = clients.join_next(), if !clients.is_empty() => {},
            accepted = game.accept() => {
                let Ok((mut socket, _)) = accepted else { break; };
                if clients.len() >= 16 { continue; }
                let Some(target) = world.read().await.clone() else { continue; };
                clients.spawn(async move {
                    if let Ok(Ok(mut upstream)) = tokio::time::timeout(Duration::from_secs(3), TcpStream::connect(("127.0.0.1", target.port))).await {
                        let _ = socket.set_nodelay(true); let _ = upstream.set_nodelay(true);
                        let _ = tokio::io::copy_bidirectional(&mut socket, &mut upstream).await;
                    }
                });
            }
        } }
    });
    Ok(Services(vec![controller, bridge]))
}

pub async fn run(port: u16, secret: String) {
    let key = token(&secret);
    let local = Arc::new(RwLock::new(None));
    let mut routes = Routes(BTreeMap::new());
    let mut services = None;
    let mut bound = String::new();
    loop {
        let Ok(node) = super::virtual_lan::query(port).await else { tokio::time::sleep(Duration::from_secs(2)).await; continue; };
        let Some(address) = node.pointer("/node_info/ipv4_addr").and_then(|value| value.as_str()).and_then(super::virtual_lan::normalize_address) else { tokio::time::sleep(Duration::from_secs(2)).await; continue; };
        if bound != address {
            services.take();
            match serve(&address, key.clone(), local.clone()).await {
                Ok(next) => { services = Some(next); bound = address.clone(); STATE.write().await.error = None; },
                Err(error) => { STATE.write().await.error = Some(format!("Não foi possível preparar o acesso ao mundo na rede virtual: {error}")); tokio::time::sleep(Duration::from_secs(2)).await; continue; }
            }
        }
        let mut candidates = Vec::new();
        super::virtual_lan::record_connection(port, &address).await;
        STATE.write().await.error = None;
        if let Ok(worlds) = crate::commands::p2p::p2p_scan_lan_worlds().await {
            for world in worlds.into_iter().filter(|world| super::lan_discovery::is_local_world(&world.host, &world.motd)).take(4) { candidates.push(PublishedWorld { motd: world.motd, port: world.port }); }
        }
        let manual = MANUAL_PORT.load(Ordering::Acquire);
        if let Ok(Some(port)) = tokio::task::spawn_blocking(active_port).await { candidates.insert(0, PublishedWorld { motd: "Meu mundo".into(), port }); }
        if manual != 0 { candidates.insert(0, PublishedWorld { motd: "Meu mundo".into(), port: manual }); }
        let mut published = None;
        for candidate in candidates { if minecraft("127.0.0.1".into(), candidate.port).await.is_ok() { published = Some(candidate); break; } }
        *local.write().await = published.clone(); STATE.write().await.local_world = published;
        let mut found = Vec::new();
        if let Some(peers) = node.get("peer_routes").and_then(|value| value.as_array()) {
            for peer in peers.iter().take(16) {
                let Some(ip) = peer.pointer("/route/ipv4_addr").and_then(|value| value.as_str()).and_then(super::virtual_lan::normalize_address) else { continue; };
                if ip == address { continue; }
                let owner = peer.pointer("/route/hostname").and_then(|value| value.as_str()).unwrap_or("Amigo").to_string();
                match request(&ip, &key).await {
                    Ok(Some(world)) => {
                        if let Ok(status) = minecraft(ip.clone(), GAME_PORT).await {
                            found.push(ip.clone());
                            if !routes.0.contains_key(&ip) {
                                if let Ok(listener) = TcpListener::bind("127.0.0.1:0").await {
                                    if let Ok(socket) = listener.local_addr() {
                                        let row = LanWorld { id: ip.clone(), owner: owner.clone(), name: world.motd.clone(), local_address: socket.to_string(), version: status.version.clone(), latency_ms: status.latency_ms, available: true };
                                        let task = proxy(listener, ip.clone(), GAME_PORT, format!("{owner} · {}", world.motd));
                                        routes.0.insert(ip.clone(), (row, task, tokio::time::Instant::now()));
                                    }
                                }
                            }
                            if let Some((row, _, seen)) = routes.0.get_mut(&ip) {
                                *seen = tokio::time::Instant::now(); row.available = true; row.version = status.version; row.latency_ms = status.latency_ms;
                            }
                            STATE.write().await.error = None;
                        } else { STATE.write().await.error = Some(format!("{owner} abriu o mundo, mas o Minecraft não respondeu pela rede. Confira a conexão do computador do anfitrião")); }
                    },
                    Ok(None) => {},
                    Err(_) => { STATE.write().await.error = Some(format!("A rede encontrou {owner}, mas o launcher dele não respondeu. Os dois PCs precisam instalar a mesma revisão")); }
                }
            }
        }
        routes.refresh(&found);
        STATE.write().await.worlds = routes.0.values().map(|(row, _, _)| row.clone()).collect();
        tokio::time::sleep(Duration::from_secs(2)).await;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[tokio::test]
    async fn world_address_survives_a_missed_poll_until_grace_expires() {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let address = listener.local_addr().unwrap();
        let row = LanWorld { id: "peer".into(), owner: "Friend".into(), name: "World".into(), local_address: address.to_string(), version: "1.21.1".into(), latency_ms: None, available: true };
        let mut routes = Routes(BTreeMap::from([("peer".into(), (row, proxy(listener, "127.0.0.1".into(), 9, "World".into()), tokio::time::Instant::now()))]));
        routes.refresh(&[]);
        assert!(!routes.0["peer"].0.available);
        assert!(TcpStream::connect(address).await.is_ok());
        routes.refresh(&["peer".into()]);
        assert_eq!(routes.0["peer"].0.local_address, address.to_string());
        routes.0.get_mut("peer").unwrap().2 = tokio::time::Instant::now() - Duration::from_secs(121);
        routes.refresh(&[]);
        assert!(routes.0.is_empty());
    }
    #[test]
    fn announces_only_sanitized_local_routes_and_checks_private_key() {
        assert!(authorized(&token("private"), &token("private")));
        assert!(!authorized(&token("other"), &token("private")));
        assert_eq!(advertise(25565, "World[AD]1[/AD]\n"), "[MOTD]§b[Luxmc] §fWorld AD 1 /AD [/MOTD][AD]25565[/AD]");
        assert_eq!(logged_port("[INFO] Started serving on 24444\n[INFO] Started serving on 25565"), Some(25565));
        assert!(logged_port("[INFO] Started serving on 999999").is_none());
    }
    #[tokio::test]
    async fn loopback_proxy_transfers_game_bytes_in_both_directions() {
        use tokio::io::AsyncReadExt;
        let upstream = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let upstream_port = upstream.local_addr().unwrap().port();
        let server = tokio::spawn(async move { let (mut socket, _) = upstream.accept().await.unwrap(); let mut bytes = [0; 4]; socket.read_exact(&mut bytes).await.unwrap(); assert_eq!(&bytes, b"game"); socket.write_all(b"okay").await.unwrap(); });
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap(); let port = listener.local_addr().unwrap().port();
        let task = proxy(listener, "127.0.0.1".into(), upstream_port, "World".into());
        let mut client = TcpStream::connect(("127.0.0.1", port)).await.unwrap(); client.write_all(b"game").await.unwrap(); let mut bytes = [0; 4]; client.read_exact(&mut bytes).await.unwrap(); assert_eq!(&bytes, b"okay");
        task.abort(); server.await.unwrap();
    }
    #[tokio::test]
    async fn authenticates_world_and_pings_minecraft_through_both_proxies() {
        use tokio::io::AsyncReadExt;
        let upstream = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let port = upstream.local_addr().unwrap().port();
        let server = tokio::spawn(async move {
            let (mut socket, _) = upstream.accept().await.unwrap();
            let length = socket.read_u8().await.unwrap(); assert!(length < 128);
            let mut handshake = vec![0u8; length as usize]; socket.read_exact(&mut handshake).await.unwrap(); assert_eq!(handshake[0], 0);
            assert_eq!(socket.read_u8().await.unwrap(), 1); assert_eq!(socket.read_u8().await.unwrap(), 0);
            let payload = br#"{"version":{"name":"1.21.1"},"players":{"max":8,"online":1},"description":{"text":"World"}}"#;
            let mut response = vec![(payload.len() + 2) as u8, 0, payload.len() as u8]; response.extend_from_slice(payload);
            socket.write_all(&response).await.unwrap();
        });
        let world = Arc::new(RwLock::new(Some(PublishedWorld { motd: "World".into(), port })));
        let key = token("secret"); let services = serve("127.0.0.1", key.clone(), world).await.unwrap();
        assert!(request("127.0.0.1", &token("wrong")).await.is_err());
        assert_eq!(request("127.0.0.1", &key).await.unwrap().unwrap().port, port);
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap(); let local_port = listener.local_addr().unwrap().port();
        let bridge = proxy(listener, "127.0.0.1".into(), GAME_PORT, "World".into());
        let status = minecraft("127.0.0.1".into(), local_port).await.unwrap(); assert_eq!(status.version, "1.21.1");
        bridge.abort(); drop(services); server.await.unwrap();
    }
}
