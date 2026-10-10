use std::net::{IpAddr, Ipv4Addr, SocketAddrV4};
use tokio::net::UdpSocket;

pub fn interfaces() -> Vec<Ipv4Addr> {
    let mut addresses = vec![Ipv4Addr::LOCALHOST];
    if let Ok(interfaces) = if_addrs::get_if_addrs() {
        for interface in interfaces {
            if let IpAddr::V4(ip) = interface.ip() {
                if !ip.is_unspecified() && !ip.is_multicast() && !addresses.contains(&ip) { addresses.push(ip); }
            }
        }
    }
    addresses
}

pub fn local_client(ip: IpAddr) -> bool {
    ip.is_loopback() || matches!(ip, IpAddr::V4(ip) if interfaces().contains(&ip))
}

pub struct Advertiser { sockets: Vec<UdpSocket> }
impl Advertiser {
    pub fn new() -> Self {
        let sockets = interfaces().into_iter().filter_map(|ip| {
            let socket = socket2::Socket::new(socket2::Domain::IPV4, socket2::Type::DGRAM, Some(socket2::Protocol::UDP)).ok()?;
            socket.bind(&SocketAddrV4::new(ip, 0).into()).ok()?;
            socket.set_multicast_if_v4(&ip).ok()?;
            socket.set_multicast_loop_v4(true).ok()?;
            socket.set_multicast_ttl_v4(1).ok()?;
            socket.set_nonblocking(true).ok()?;
            UdpSocket::from_std(socket.into()).ok()
        }).collect();
        Self { sockets }
    }
    pub async fn announce(&self, port: u16, title: &str) {
        let title: String = title.chars().filter(|value| !value.is_control()).take(140).collect::<String>().replace(['[', ']'], " ");
        let packet = format!("[MOTD]§b[Luxmc] §f{title}[/MOTD][AD]{port}[/AD]");
        for socket in &self.sockets { let _ = socket.send_to(packet.as_bytes(), "224.0.2.60:4445").await; }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn rejects_clients_from_other_computers() {
        assert!(super::local_client("127.0.0.1".parse().unwrap()));
        assert!(super::local_client("::1".parse().unwrap()));
        assert!(!super::local_client("203.0.113.37".parse().unwrap()));
    }
}
