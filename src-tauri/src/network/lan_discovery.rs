use std::{net::{IpAddr, UdpSocket}, sync::{Arc, atomic::{AtomicU16, Ordering}}, time::Duration};
use tokio::{sync::Mutex, task::JoinHandle};

pub fn is_local_world(host: &str, motd: &str) -> bool {
    let Ok(ip) = host.parse::<IpAddr>() else { return false; };
    !motd.contains("[Luxmc]") && (ip.is_loopback() || UdpSocket::bind((ip, 0)).is_ok())
}

pub fn monitor(target: Arc<AtomicU16>, ready: Arc<Mutex<super::p2p_tunnel::RoomRegistry>>) -> JoinHandle<()> {
    tokio::spawn(async move {
        let mut last_seen = tokio::time::Instant::now();
        loop {
            match crate::commands::p2p::p2p_scan_lan_worlds().await {
                Ok(worlds) => {
                    let current = target.load(Ordering::Acquire);
                    let mut candidates: Vec<_> = worlds.iter().rev().filter(|world| is_local_world(&world.host, &world.motd)).collect();
                    candidates.sort_by_key(|world| world.port != current);
                    for world in candidates {
                        if matches!(tokio::time::timeout(Duration::from_millis(300), tokio::net::TcpStream::connect(("127.0.0.1", world.port))).await, Ok(Ok(_))) {
                            target.store(world.port, Ordering::Release);
                            super::p2p_tunnel::write_private_lan_session(world.port).await;
                            let mut registry = ready.lock().await;
                            registry.world_ready = true;
                            if let Some(owner) = registry.host.clone() {
                                registry.worlds.insert(owner.id.clone(), super::p2p_tunnel::RoomWorld { owner_id: owner.id, owner_username: owner.username, motd: world.motd.chars().filter(|character| !character.is_control()).take(160).collect(), local_address: None });
                            }
                            last_seen = tokio::time::Instant::now();
                            break;
                        }
                    }
                }
                Err(error) => {
                    tracing::warn!("LAN discovery unavailable: {error}");
                    tokio::time::sleep(Duration::from_secs(2)).await;
                }
            }
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
    #[test]
    fn only_local_minecraft_worlds_are_forwarded() {
        assert!(super::is_local_world("127.0.0.1", "My Minecraft world"));
        assert!(!super::is_local_world("127.0.0.1", "§b[Luxmc] §fMundo de Amigo"));
        assert!(!super::is_local_world("203.0.113.25", "Someone else's world"));
        assert!(!super::is_local_world("invalid", "World"));
    }
}
