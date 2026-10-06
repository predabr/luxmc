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
                    let selected = worlds.iter().filter(|world| is_local_world(&world.host, &world.motd))
                        .find(|world| world.port == current)
                        .or_else(|| worlds.iter().rev().find(|world| is_local_world(&world.host, &world.motd)));
                    if let Some(world) = selected {
                        if matches!(tokio::time::timeout(Duration::from_millis(300), tokio::net::TcpStream::connect(("127.0.0.1", world.port))).await, Ok(Ok(_))) {
                            target.store(world.port, Ordering::Release);
                            ready.lock().await.world_ready = true;
                            last_seen = tokio::time::Instant::now();
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
                ready.lock().await.world_ready = false;
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
