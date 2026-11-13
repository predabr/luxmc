use serde::{Deserialize, Serialize};
use std::net::SocketAddrV4;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpnpPortMappingResult {
    pub success: bool,
    pub external_ip: Option<String>,
    pub port: u16,
    pub message: String,
}

pub async fn open_port_upnp(local_port: u16, lease_duration_secs: u32) -> UpnpPortMappingResult {
    tokio::task::spawn_blocking(move || match igd_next::search_gateway(Default::default()) {
        Ok(gateway) => {
            let local_ip = match local_ip_address() {
                Some(ip) => ip,
                None => {
                    return UpnpPortMappingResult {
                        success: false,
                        external_ip: None,
                        port: local_port,
                        message: "Não foi possível determinar o endereço IP local".into(),
                    };
                }
            };

            let local_addr = SocketAddrV4::new(local_ip, local_port);
            let desc = "Luxmc Minecraft server";

            match gateway.add_port(
                igd_next::PortMappingProtocol::TCP,
                local_port,
                std::net::SocketAddr::V4(local_addr),
                lease_duration_secs,
                desc,
            ) {
                Ok(()) => {
                    let ext_ip = gateway.get_external_ip().ok().map(|ip| ip.to_string());
                    UpnpPortMappingResult {
                        success: true,
                        external_ip: ext_ip,
                        port: local_port,
                        message: format!("Porta {} aberta com sucesso via UPnP!", local_port),
                    }
                }
                Err(e) => UpnpPortMappingResult {
                    success: false,
                    external_ip: None,
                    port: local_port,
                    message: format!("Falha ao adicionar redirecionamento UPnP: {e}"),
                },
            }
        }
        Err(e) => UpnpPortMappingResult {
            success: false,
            external_ip: None,
            port: local_port,
            message: format!("Roteador/Gateway UPnP não encontrado: {e}"),
        },
    })
    .await
    .unwrap_or_else(|e| UpnpPortMappingResult {
        success: false,
        external_ip: None,
        port: local_port,
        message: format!("Erro interno ao executar UPnP: {e}"),
    })
}

pub async fn close_port_upnp(port: u16) -> bool {
    tokio::task::spawn_blocking(move || {
        if let Ok(gateway) = igd_next::search_gateway(Default::default()) {
            gateway
                .remove_port(igd_next::PortMappingProtocol::TCP, port)
                .is_ok()
        } else {
            false
        }
    })
    .await
    .unwrap_or(false)
}

pub async fn close_owned_minecraft_port_upnp(port: u16) -> bool {
    tokio::task::spawn_blocking(move || {
        let Ok(gateway) = igd_next::search_gateway(Default::default()) else {
            return false;
        };
        let local_ip = local_ip_address().map(|ip| ip.to_string());
        for index in 0..4096 {
            match gateway.get_generic_port_mapping_entry(index) {
                Ok(entry)
                    if entry.protocol == igd_next::PortMappingProtocol::TCP
                        && entry.external_port == port
                        && entry.internal_port == port
                        && entry.port_mapping_description == "Luxmc Minecraft server"
                        && entry.enabled
                        && local_ip.as_deref() == Some(entry.internal_client.as_str()) =>
                {
                    return gateway
                        .remove_port(igd_next::PortMappingProtocol::TCP, port)
                        .is_ok();
                }
                Ok(_) => {}
                Err(igd_next::GetGenericPortMappingEntryError::SpecifiedArrayIndexInvalid) => {
                    return true
                }
                Err(_) => return false,
            }
        }
        true
    })
    .await
    .unwrap_or(false)
}

fn local_ip_address() -> Option<std::net::Ipv4Addr> {
    let socket = std::net::UdpSocket::bind("0.0.0.0:0").ok()?;
    socket.connect("8.8.8.8:80").ok()?;
    match socket.local_addr().ok()?.ip() {
        std::net::IpAddr::V4(ipv4) => Some(ipv4),
        _ => None,
    }
}
