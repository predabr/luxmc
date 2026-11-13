use crate::error::{AppError, AppResult};
use serde::Serialize;
use std::{net::Ipv4Addr, sync::OnceLock, time::Duration};
use tokio::sync::{watch, Mutex};

const LEASE_SECONDS: u32 = 7200;
const RENEWAL_INTERVAL: Duration = Duration::from_secs(3600);

struct ActiveAccess {
    id: String,
    port: u16,
    external_ip: std::sync::Arc<Mutex<Option<String>>>,
    last_error: std::sync::Arc<Mutex<Option<String>>>,
    stop: watch::Sender<bool>,
    renewal: tokio::task::JoinHandle<()>,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PublicAccessStatus {
    pub enabled: bool,
    pub port: u16,
    pub external_ip: Option<String>,
    pub address: Option<String>,
    pub likely_behind_cgnat: bool,
    pub renewal_error: Option<String>,
}

static ACTIVE: OnceLock<Mutex<Option<ActiveAccess>>> = OnceLock::new();

fn active() -> &'static Mutex<Option<ActiveAccess>> {
    ACTIVE.get_or_init(|| Mutex::new(None))
}

pub async fn enable(id: &str, port: u16) -> AppResult<PublicAccessStatus> {
    if id.is_empty() || port == 0 {
        return Err(AppError::InvalidInput(
            "Servidor ou porta do Minecraft inválido.".into(),
        ));
    }

    let mut active = active().lock().await;
    if let Some(current) = active.as_ref() {
        if current.id == id && current.port == port {
            return Ok(status_for(current).await);
        }
        return Err(AppError::InvalidState(
            "Já existe um encaminhamento de servidor ativo no Luxmc. Desative-o antes de configurar outro.".into(),
        ));
    }

    super::firewall::set_minecraft_rule(port, true).await?;
    let mapping = super::upnp::open_port_upnp(port, LEASE_SECONDS).await;
    if !mapping.success {
        let mapping_cleanup = super::upnp::close_owned_minecraft_port_upnp(port).await;
        let cleanup = super::firewall::set_minecraft_rule(port, false).await;
        let mut message = mapping.message;
        if !mapping_cleanup {
            message.push_str(
                " O roteador não confirmou a limpeza de um possível encaminhamento parcial.",
            );
        }
        if let Err(error) = cleanup {
            message.push_str(&format!(
                " A regra do firewall não pôde ser removida automaticamente: {error}"
            ));
        }
        return Err(AppError::InvalidState(message));
    }

    let (stop, mut stopped) = watch::channel(false);
    let last_error = std::sync::Arc::new(Mutex::new(None));
    let renewal_error = last_error.clone();
    let external_ip = std::sync::Arc::new(Mutex::new(mapping.external_ip));
    let renewal_ip = external_ip.clone();
    let renewal = tokio::spawn(async move {
        loop {
            tokio::select! {
                _ = tokio::time::sleep(RENEWAL_INTERVAL) => {
                    let result = super::upnp::open_port_upnp(port, LEASE_SECONDS).await;
                    if result.success {
                        *renewal_error.lock().await = None;
                        if let Some(ip) = result.external_ip {
                            *renewal_ip.lock().await = Some(ip);
                        }
                    } else {
                        let error = result.message;
                        tracing::warn!(port, %error, "Não foi possível renovar o encaminhamento UPnP do Minecraft");
                        *renewal_error.lock().await = Some(error);
                    }
                }
                changed = stopped.changed() => {
                    if changed.is_err() || *stopped.borrow() {
                        break;
                    }
                }
            }
        }
    });

    *active = Some(ActiveAccess {
        id: id.into(),
        port,
        external_ip,
        last_error,
        stop,
        renewal,
    });
    let access = active.as_ref().ok_or_else(|| {
        AppError::Internal("O encaminhamento foi registrado sem estado ativo.".into())
    })?;
    Ok(status_for(access).await)
}

pub async fn status(id: &str) -> Option<PublicAccessStatus> {
    let active = active().lock().await;
    match active.as_ref().filter(|access| access.id == id) {
        Some(access) => Some(status_for(access).await),
        None => None,
    }
}

pub async fn is_active(id: &str, port: u16) -> bool {
    active()
        .lock()
        .await
        .as_ref()
        .is_some_and(|access| access.id == id && access.port == port)
}

pub async fn active_id() -> Option<String> {
    active()
        .lock()
        .await
        .as_ref()
        .map(|access| access.id.clone())
}

pub async fn disable(id: &str, port: u16) -> AppResult<()> {
    if port == 0 {
        return Err(AppError::InvalidInput(
            "A porta do Minecraft deve estar entre 1 e 65535.".into(),
        ));
    }

    let access = {
        let mut active = active().lock().await;
        if active.as_ref().is_some_and(|access| access.id != id) {
            return Err(AppError::InvalidState(
                "Desative o acesso externo do outro servidor antes de alterar este encaminhamento."
                    .into(),
            ));
        }
        if active.as_ref().is_some_and(|access| access.id == id) {
            active.take()
        } else {
            None
        }
    };

    let port = access.as_ref().map_or(port, |access| access.port);
    if let Some(access) = access {
        access.stop.send_replace(true);
        let _ = access.renewal.await;
    }

    let mapping_removed = super::upnp::close_owned_minecraft_port_upnp(port).await;
    let firewall_result = super::firewall::set_minecraft_rule(port, false).await;
    let mut errors = Vec::new();
    if !mapping_removed {
        errors.push(format!(
            "O roteador não confirmou a remoção da porta {port}; o encaminhamento temporário pode permanecer até expirar."
        ));
    }
    if let Err(error) = firewall_result {
        errors.push(error.to_string());
    }
    if errors.is_empty() {
        Ok(())
    } else {
        Err(AppError::InvalidState(errors.join(" ")))
    }
}

pub async fn shutdown() -> AppResult<()> {
    let access = active().lock().await.take();
    let Some(access) = access else {
        return Ok(());
    };
    access.stop.send_replace(true);
    let _ = access.renewal.await;
    let mapping_removed = super::upnp::close_owned_minecraft_port_upnp(access.port).await;
    let firewall_result = super::firewall::set_minecraft_rule(access.port, false).await;
    let mut errors = Vec::new();
    if !mapping_removed {
        errors.push(format!(
            "O roteador não confirmou a remoção da porta {}; o encaminhamento temporário pode permanecer até expirar.",
            access.port
        ));
    }
    if let Err(error) = firewall_result {
        errors.push(error.to_string());
    }
    if errors.is_empty() {
        Ok(())
    } else {
        Err(AppError::InvalidState(errors.join(" ")))
    }
}

async fn status_for(access: &ActiveAccess) -> PublicAccessStatus {
    let external_ip = access.external_ip.lock().await.clone();
    let address = external_ip
        .as_ref()
        .map(|ip| format!("{ip}:{}", access.port));
    PublicAccessStatus {
        enabled: true,
        port: access.port,
        likely_behind_cgnat: external_ip.as_deref().is_some_and(is_non_public_ipv4),
        external_ip,
        address,
        renewal_error: access.last_error.lock().await.clone(),
    }
}

fn is_non_public_ipv4(address: &str) -> bool {
    let Ok(ip) = address.parse::<Ipv4Addr>() else {
        return true;
    };
    ip.is_private()
        || ip.is_loopback()
        || ip.is_link_local()
        || ip.is_unspecified()
        || ip.octets()[0] == 0
        || ip.octets()[0] >= 224
        || (ip.octets()[0] == 100 && (64..=127).contains(&ip.octets()[1]))
}

#[cfg(test)]
mod tests {
    use super::is_non_public_ipv4;

    #[test]
    fn identifies_private_and_shared_address_ranges() {
        for address in ["10.0.0.1", "192.168.1.1", "100.64.0.1", "127.0.0.1"] {
            assert!(is_non_public_ipv4(address));
        }
        assert!(!is_non_public_ipv4("8.8.8.8"));
    }
}
