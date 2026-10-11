use std::{net::{IpAddr, UdpSocket}, sync::{Arc, atomic::{AtomicU16, Ordering}}, time::Duration};
use tokio::{sync::Mutex, task::JoinHandle};

pub fn is_local_world(host: &str, motd: &str) -> bool {
    let Ok(ip) = host.parse::<IpAddr>() else { return false; };
    !motd.contains("[Luxmc]") && (ip.is_loopback() || UdpSocket::bind((ip, 0)).is_ok())
}

struct CompatibilityLoad(Option<JoinHandle<Option<crate::commands::experience::Compatibility>>>);
impl Drop for CompatibilityLoad {
    fn drop(&mut self) { if let Some(task) = &self.0 { task.abort(); } }
}

pub(super) async fn connect_local_game(port: u16) -> std::io::Result<tokio::net::TcpStream> {
    if port == 0 { return Err(std::io::Error::new(std::io::ErrorKind::InvalidInput, "Invalid local game port")); }
    let ipv4 = tokio::net::TcpStream::connect(("127.0.0.1", port));
    let ipv6 = tokio::net::TcpStream::connect(("::1", port));
    tokio::pin!(ipv4, ipv6);
    tokio::select! {
        biased;
        result = &mut ipv4 => match result { Ok(stream) => Ok(stream), Err(_) => ipv6.await },
        result = &mut ipv6 => match result { Ok(stream) => Ok(stream), Err(_) => ipv4.await },
    }
}

fn discovery_candidates(
    scanned: crate::error::AppResult<Vec<crate::commands::p2p::DiscoveredLanWorld>>,
    game_port: Option<u16>,
) -> Vec<crate::commands::p2p::DiscoveredLanWorld> {
    let mut worlds = match scanned {
        Ok(worlds) => worlds,
        Err(error) => { tracing::warn!("LAN multicast unavailable; using game discovery: {error}"); Vec::new() }
    };
    if let Some(port) = game_port.filter(|port| *port != 0) {
        worlds.insert(0, crate::commands::p2p::DiscoveredLanWorld { host: "127.0.0.1".into(), port, motd: "Meu mundo".into() });
    }
    worlds
}

async fn available_game_port(ports: Vec<u16>) -> Option<u16> {
    for port in ports.into_iter().filter(|port| *port != 0) {
        if matches!(tokio::time::timeout(Duration::from_millis(300), connect_local_game(port)).await, Ok(Ok(_))) {
            return Some(port);
        }
    }
    None
}

pub fn monitor(target: Arc<AtomicU16>, ready: Arc<Mutex<super::p2p_tunnel::RoomRegistry>>) -> JoinHandle<()> {
    tokio::spawn(async move {
        let mut signature = String::new();
        let mut compatibility = None;
        let mut compatibility_load = CompatibilityLoad(None);
        let mut last_seen = tokio::time::Instant::now();
        let mut interval = tokio::time::interval(Duration::from_secs(2));
        loop {
            interval.tick().await;
            if compatibility_load.0.as_ref().is_some_and(|task| task.is_finished()) {
                if let Some(task) = compatibility_load.0.take() { compatibility = task.await.ok().flatten(); }
            }
            let ports = tokio::task::spawn_blocking(super::lan_log::active_ports).await.unwrap_or_default();
            let game_port = available_game_port(ports).await;
            let scanned = if game_port.is_some() { Ok(Vec::new()) } else { crate::commands::p2p::p2p_scan_lan_worlds().await };
            let worlds = discovery_candidates(scanned, game_port);
            let current = target.load(Ordering::Acquire);
            let mut candidates: Vec<_> = worlds.iter().rev().filter(|world| is_local_world(&world.host, &world.motd)).collect();
            candidates.sort_by_key(|world| (Some(world.port) != game_port, world.port != current));
            let mut found = false;
            for world in candidates {
                if Some(world.port) == game_port || matches!(tokio::time::timeout(Duration::from_millis(300), connect_local_game(world.port)).await, Ok(Ok(_))) {
                    target.store(world.port, Ordering::Release);
                    super::p2p_tunnel::write_private_lan_session(world.port).await;
                    let next_signature = format!("{:?}:{}:{}",crate::core::launcher::get_active_game_dir(),world.port,crate::core::launcher::get_active_game_pid());
                    if signature != next_signature {
                        compatibility = None;
                        compatibility_load = CompatibilityLoad(Some(tokio::spawn(crate::commands::experience::active_compatibility())));
                        signature = next_signature;
                    }
                    let mut registry = ready.lock().await;
                    registry.world_ready = true;
                    if let Some(owner) = registry.host.clone() {
                        registry.worlds.insert(owner.id.clone(), super::p2p_tunnel::RoomWorld { owner_id: owner.id, owner_username: owner.username, motd: world.motd.chars().filter(|character| !character.is_control()).take(160).collect(), local_address: None, compatibility: compatibility.clone() });
                    }
                    last_seen = tokio::time::Instant::now();
                    found = true;
                    break;
                }
            }
            let current = target.load(Ordering::Acquire);
            if !found && current != 0 && matches!(tokio::time::timeout(Duration::from_millis(300), connect_local_game(current)).await, Ok(Ok(_))) { last_seen = tokio::time::Instant::now(); }
            if last_seen.elapsed() > Duration::from_secs(7) {
                target.store(0, Ordering::Release);
                let mut registry = ready.lock().await;
                registry.world_ready = false;
                if let Some(owner) = registry.host.clone() { registry.worlds.remove(&owner.id); }
            }
        }
    })
}

#[cfg(test)]
mod tests {
    #[tokio::test]
    async fn local_game_forwarding_supports_ipv4_and_ipv6_listeners() {
        use tokio::io::{AsyncReadExt, AsyncWriteExt};
        for address in ["127.0.0.1", "::1"] {
            let listener = match tokio::net::TcpListener::bind((address, 0)).await {
                Ok(listener) => listener,
                Err(error) if address == "::1" && matches!(error.kind(), std::io::ErrorKind::AddrNotAvailable | std::io::ErrorKind::Unsupported) => continue,
                Err(error) => panic!("Local listener unavailable: {error}"),
            };
            let port = listener.local_addr().unwrap().port();
            assert_eq!(super::available_game_port(vec![port]).await, Some(port));
            let probe = tokio::time::timeout(std::time::Duration::from_secs(2), listener.accept()).await.unwrap().unwrap().0;
            drop(probe);
            let server = tokio::spawn(async move {
                let (mut stream, _) = listener.accept().await.unwrap();
                let mut bytes = [0; 4];
                stream.read_exact(&mut bytes).await.unwrap();
                assert_eq!(&bytes, b"game");
                stream.write_all(b"okay").await.unwrap();
            });
            let mut client = tokio::time::timeout(std::time::Duration::from_secs(2), super::connect_local_game(port)).await.unwrap().unwrap();
            client.write_all(b"game").await.unwrap();
            let mut bytes = [0; 4];
            tokio::time::timeout(std::time::Duration::from_secs(2), client.read_exact(&mut bytes)).await.unwrap().unwrap();
            assert_eq!(&bytes, b"okay");
            tokio::time::timeout(std::time::Duration::from_secs(2), server).await.unwrap().unwrap();
        }
    }

    #[tokio::test]
    async fn a_closed_previous_world_does_not_hide_the_current_listener() {
        let previous = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let closed_port = previous.local_addr().unwrap().port();
        let current = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let current_port = current.local_addr().unwrap().port();
        drop(previous);
        assert_eq!(super::available_game_port(vec![closed_port, current_port]).await, Some(current_port));
    }

    #[test]
    fn a_blocked_multicast_socket_does_not_hide_a_published_game() {
        let blocked = Err(crate::error::AppError::InvalidState("UDP socket unavailable".into()));
        let worlds = super::discovery_candidates(blocked, Some(57813));
        assert_eq!(worlds.len(), 1);
        assert_eq!(worlds[0].host, "127.0.0.1");
        assert_eq!(worlds[0].port, 57813);
        assert!(super::discovery_candidates(Ok(Vec::new()), None).is_empty());
    }

    #[test]
    fn only_local_minecraft_worlds_are_forwarded() {
        assert!(super::is_local_world("127.0.0.1", "My Minecraft world"));
        assert!(!super::is_local_world("127.0.0.1", "§b[Luxmc] §fMundo de Amigo"));
        assert!(!super::is_local_world("203.0.113.25", "Someone else's world"));
        assert!(!super::is_local_world("invalid", "World"));
    }
}
