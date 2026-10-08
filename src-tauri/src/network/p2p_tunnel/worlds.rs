use super::*;

#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RoomWorld {
    pub owner_id: String,
    pub owner_username: String,
    pub motd: String,
    #[serde(default)]
    pub local_address: Option<String>,
    #[serde(default)]
    pub compatibility: Option<crate::commands::experience::Compatibility>,
}

struct Routes(BTreeMap<String, JoinHandle<()>>);
impl Drop for Routes {
    fn drop(&mut self) { for task in self.0.values() { task.abort(); } }
}

pub(super) async fn publish(connection: &iroh::endpoint::Connection, secret: [u8; 32], world: Option<RoomWorld>) -> AppResult<()> {
    let enriched = serde_json::to_vec(&world)?;
    if matches!(tokio::time::timeout(Duration::from_millis(900),publish_bytes(connection,secret,&enriched)).await,Ok(Ok(()))) { return Ok(()); }
    let legacy = serde_json::to_vec(&world.map(|world| world.motd))?;
    publish_bytes(connection,secret,&legacy).await
}

async fn publish_bytes(connection: &iroh::endpoint::Connection, secret: [u8; 32], bytes: &[u8]) -> AppResult<()> {
    let (mut send, mut recv) = connection.open_bi().await.map_err(failure)?;
    let mut header = [4u8; 33]; header[1..].copy_from_slice(&secret);
    send.write_all(&header).await.map_err(failure)?;
    send.write_all(&(bytes.len() as u16).to_be_bytes()).await.map_err(failure)?;
    send.write_all(&bytes).await.map_err(failure)?;
    let _ = send.finish();
    let mut ack = [0]; recv.read_exact(&mut ack).await.map_err(failure)?;
    if ack != [1] { return Err(failure("O mundo não foi anunciado na sala")); }
    Ok(())
}

pub(super) async fn request(connection: &iroh::endpoint::Connection, secret: [u8; 32], owner: Option<&str>) -> AppResult<(iroh::endpoint::SendStream, iroh::endpoint::RecvStream)> {
    let (mut send, mut recv) = connection.open_bi().await.map_err(failure)?;
    let mut header = [if owner.is_some() { 5u8 } else { 6u8 }; 33]; header[1..].copy_from_slice(&secret);
    send.write_all(&header).await.map_err(failure)?;
    if let Some(owner) = owner {
        if owner.len() > 128 { return Err(failure("Participante inválido")); }
        send.write_all(&[owner.len() as u8]).await.map_err(failure)?;
        send.write_all(owner.as_bytes()).await.map_err(failure)?;
    }
    let mut ack = [0]; recv.read_exact(&mut ack).await.map_err(failure)?;
    if ack != [1] { return Err(failure("O participante fechou o mundo LAN")); }
    Ok((send, recv))
}

pub(super) async fn control(op: u8, id: &str, mut send: iroh::endpoint::SendStream, mut recv: iroh::endpoint::RecvStream, registry: Arc<Mutex<RoomRegistry>>) {
    if op == 4 {
        let mut length = [0; 2];
        if recv.read_exact(&mut length).await.is_err() { return; }
        let length = u16::from_be_bytes(length) as usize;
        if length > 1024 { return; }
        let mut bytes = vec![0; length];
        if recv.read_exact(&mut bytes).await.is_err() { return; }
        let parsed = serde_json::from_slice::<Option<RoomWorld>>(&bytes);
        let (motd,compatibility) = match parsed {
            Ok(Some(world)) => (Some(world.motd),world.compatibility),
            Ok(None) => (None,None),
            Err(_) => match serde_json::from_slice::<Option<String>>(&bytes) { Ok(motd) => (motd,None), Err(_) => return },
        };
        if compatibility.as_ref().is_some_and(|c| c.mc_version.len()>64 || c.loader.len()>32 || c.mod_fingerprint.len()!=64 || !c.mod_fingerprint.chars().all(|v| v.is_ascii_hexdigit()) || c.mod_count>10000) {return;}
        let mut room = registry.lock().await;
        let Some(member) = room.members.get(id).cloned() else { return; };
        if let Some(motd) = motd {
            if motd.chars().count() > 160 || motd.chars().any(char::is_control) { return; }
            room.worlds.insert(id.into(), RoomWorld { owner_id: id.into(), owner_username: member.username, motd, local_address: None, compatibility });
        } else { room.worlds.remove(id); }
        drop(room);
        let _ = send.write_all(&[1]).await; let _ = send.finish();
    } else if op == 5 {
        let mut length = [0];
        if recv.read_exact(&mut length).await.is_err() || length[0] == 0 || length[0] > 128 { return; }
        let mut bytes = vec![0; length[0] as usize];
        if recv.read_exact(&mut bytes).await.is_err() { return; }
        let Ok(owner) = String::from_utf8(bytes) else { return; };
        let target = {
            let room = registry.lock().await;
            if !room.members.contains_key(id) || !room.worlds.contains_key(&owner) || owner == id { None }
            else { room.connections.get(&owner).cloned().zip(room.member_secrets.get(&owner).copied()) }
        };
        let Some((target, target_secret)) = target else { let _ = send.write_all(&[0]).await; return; };
        if let Ok(Ok((target_send, target_recv))) = tokio::time::timeout(Duration::from_secs(8), request(&target, target_secret, None)).await {
            if send.write_all(&[1]).await.is_err() { return; }
            let mut source = tokio::io::join(recv, send);
            let mut destination = tokio::io::join(target_recv, target_send);
            let _ = tokio::io::copy_bidirectional(&mut source, &mut destination).await;
        } else { let _ = send.write_all(&[0]).await; }
    }
}

pub(super) fn reverse(connection: iroh::endpoint::Connection, target: Arc<AtomicU16>, secret: [u8; 32]) -> JoinHandle<()> {
    tokio::spawn(async move {
        let mut streams = JoinSet::new();
        loop {
            tokio::select! {
                _ = connection.closed() => break,
                Some(_) = streams.join_next(), if !streams.is_empty() => {},
                stream = connection.accept_bi() => {
                    let Ok((mut send, mut recv)) = stream else { break; };
                    if streams.len() >= 16 { continue; }
                    let target = target.clone();
                    streams.spawn(async move {
                        let mut header = [0; 33];
                        if !matches!(tokio::time::timeout(Duration::from_secs(5), recv.read_exact(&mut header)).await, Ok(Ok(_))) { return; }
                        if header[1..].iter().zip(secret).fold(0u8, |difference, (a, b)| difference | (*a ^ b)) != 0 { return; }
                        if header[0] == 7 {
                            let mut length=[0]; if recv.read_exact(&mut length).await.is_err() || length[0]!=0 {return;}
                            coordination::local_recipe(send).await;return;
                        }
                        if header[0] != 6 {return;}
                        let port = target.load(Ordering::Acquire);
                        if port == 0 { let _ = send.write_all(&[0]).await; return; }
                        if let Ok(Ok(tcp)) = tokio::time::timeout(Duration::from_secs(5), TcpStream::connect(("127.0.0.1", port))).await {
                            if send.write_all(&[1]).await.is_ok() { let _ = bridge(tcp, send, recv).await; }
                        } else { let _ = send.write_all(&[0]).await; }
                    });
                }
            }
        }
    })
}

pub(super) fn manage(self_id: String, host_room: Option<Arc<Mutex<RoomRegistry>>>, client_room: Option<Arc<Mutex<RoomSnapshot>>>, host_connection: Option<iroh::endpoint::Connection>, secret: [u8; 32], addresses: Arc<Mutex<BTreeMap<String, String>>>) -> JoinHandle<()> {
    tokio::spawn(async move {
        let mut routes = Routes(BTreeMap::new());
        let mut interval = tokio::time::interval(Duration::from_secs(1));
        loop {
            interval.tick().await;
            let snapshot = if let Some(room) = &host_room { room.lock().await.snapshot() }
                else if let Some(room) = &client_room { room.lock().await.clone() } else { break; };
            let visible: BTreeMap<_, _> = snapshot.worlds.into_iter().filter(|world| world.owner_id != self_id && (host_room.is_some() || !snapshot.members.iter().any(|member| member.id == world.owner_id && member.is_host))).map(|world| (world.owner_id.clone(), world)).collect();
            routes.0.retain(|id, task| { let keep = visible.contains_key(id) && !task.is_finished(); if !keep { task.abort(); } keep });
            addresses.lock().await.retain(|id, _| visible.contains_key(id));
            for (id, world) in visible {
                if routes.0.contains_key(&id) { continue; }
                let destination = if let Some(room) = &host_room { let room = room.lock().await; room.connections.get(&id).cloned().zip(room.member_secrets.get(&id).copied()) } else { host_connection.clone().map(|connection| (connection, secret)) };
                let Some((connection, route_secret)) = destination else { continue; };
                let Ok(listener) = TcpListener::bind(("127.0.0.1", 0)).await else { continue; };
                let Ok(address) = listener.local_addr() else { continue; };
                addresses.lock().await.insert(id.clone(), address.to_string());
                let via_host = host_room.is_none();
                routes.0.insert(id.clone(), tokio::spawn(async move {
                    let socket = socket2::Socket::new(socket2::Domain::IPV4, socket2::Type::DGRAM, Some(socket2::Protocol::UDP)).ok().and_then(|socket| {
                        let loopback = std::net::Ipv4Addr::LOCALHOST;
                        socket.bind(&std::net::SocketAddrV4::new(loopback, 0).into()).ok()?;
                        socket.set_multicast_if_v4(&loopback).ok()?;
                        socket.set_nonblocking(true).ok()?;
                        UdpSocket::from_std(socket.into()).ok()
                    });
                    let mut interval = tokio::time::interval(Duration::from_millis(1500));
                    let mut streams = JoinSet::new();
                    loop {
                        tokio::select! {
                            _ = connection.closed() => break,
                            _ = interval.tick() => {
                                if let Some(socket) = &socket {
                                    let packet = format!("[MOTD]§b[Luxmc] §f{} · {}[/MOTD][AD]{}[/AD]", world.owner_username, world.motd.replace('[', "(").replace(']', ")"), address.port());
                                    let _ = socket.send_to(packet.as_bytes(), MINECRAFT_LAN_MULTICAST).await;
                                }
                            },
                            Some(_) = streams.join_next(), if !streams.is_empty() => {},
                            accepted = listener.accept() => {
                                let Ok((tcp, _)) = accepted else { break; };
                                if streams.len() >= 16 { continue; }
                                let connection = connection.clone(); let id = id.clone();
                                streams.spawn(async move {
                                    if let Ok(Ok((send, recv))) = tokio::time::timeout(Duration::from_secs(10), request(&connection, route_secret, via_host.then_some(id.as_str()))).await { let _ = bridge(tcp, send, recv).await; }
                                });
                            }
                        }
                    }
                }));
            }
        }
    })
}
