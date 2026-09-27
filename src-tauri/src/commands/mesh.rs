use crate::error::{AppError, AppResult};
use serde::Serialize;
use serde_json::Value;
use std::time::Duration;

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MeshPeer {
    id: String,
    name: String,
    ip: String,
    online: bool,
    transport: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MeshStatus {
    available: bool,
    state: String,
    ip: Option<String>,
    peers: Vec<MeshPeer>,
}

async fn tailscale(args: &[&str]) -> AppResult<std::process::Output> {
    let mut command = crate::core::process::tokio_command("tailscale");
    command.args(args).kill_on_drop(true);
    tokio::time::timeout(Duration::from_secs(8), command.output()).await
        .map_err(|_| AppError::InvalidState("Tailscale não respondeu em 8 segundos".into()))?
        .map_err(|e| AppError::InvalidState(format!("Instale e conecte o Tailscale para usar a rede mesh: {e}")))
}

fn ipv4(value: &Value) -> Option<String> {
    value.get("TailscaleIPs")?.as_array()?.iter()
        .filter_map(Value::as_str).find(|ip| ip.parse::<std::net::Ipv4Addr>().is_ok()).map(str::to_owned)
}

#[tauri::command]
pub async fn mesh_status() -> AppResult<MeshStatus> {
    let output = match tailscale(&["status", "--json"]).await {
        Ok(output) => output,
        Err(_) => return Ok(MeshStatus { available: false, state: "Instale e conecte o Tailscale nos dois computadores".into(), ip: None, peers: vec![] }),
    };
    if !output.status.success() {
        return Ok(MeshStatus { available: true, state: "Conecte o Tailscale à mesma rede privada dos seus amigos".into(), ip: None, peers: vec![] });
    }
    let status: Value = serde_json::from_slice(&output.stdout)?;
    let peers = status.get("Peer").and_then(Value::as_object).map(|peers| peers.iter().filter_map(|(id, value)| {
        Some(MeshPeer {
            id: id.clone(), name: value.get("HostName").and_then(Value::as_str).unwrap_or("Amigo").to_owned(),
            ip: ipv4(value)?, online: value.get("Online").and_then(Value::as_bool).unwrap_or(false),
            transport: if value.get("CurAddr").and_then(Value::as_str).is_some_and(|s| !s.is_empty()) { "Direto P2P" }
                else if value.get("Relay").and_then(Value::as_str).is_some_and(|s| !s.is_empty()) { "Relay DERP" } else { "Aguardando conexão" }.into(),
        })
    }).collect()).unwrap_or_default();
    Ok(MeshStatus {
        available: true, state: status.get("BackendState").and_then(Value::as_str).unwrap_or("Desconhecido").to_owned(),
        ip: status.get("Self").and_then(ipv4), peers,
    })
}

#[tauri::command]
pub async fn mesh_ping(ip: String) -> AppResult<u64> {
    let status = mesh_status().await?;
    if !status.peers.iter().any(|peer| peer.ip == ip && peer.online) {
        return Err(AppError::InvalidInput("Selecione um peer conectado à sua rede privada".into()));
    }
    let output = tailscale(&["ping", "--c=1", "--until-direct=false", "--timeout=5s", &ip]).await?;
    if !output.status.success() { return Err(AppError::InvalidState("Peer não respondeu".into())); }
    parse_latency(&String::from_utf8_lossy(&output.stdout))
        .ok_or_else(|| AppError::InvalidState("Resposta de ping sem medida de latência".into()))
}


fn parse_latency(output: &str) -> Option<u64> {
    let value = output.lines().find(|line| line.starts_with("pong from "))?.rsplit_once(" in ")?.1.trim();
    let milliseconds = if let Some(ms) = value.strip_suffix("ms") { ms.parse::<f64>().ok()? }
        else { value.strip_suffix('s')?.parse::<f64>().ok()? * 1000.0 };
    (milliseconds.is_finite() && milliseconds >= 0.0).then_some(milliseconds.round() as u64)
}

#[cfg(test)]
mod tests {
    #[test]
    fn reports_network_round_trip_including_relay() {
        assert_eq!(super::parse_latency("pong from friend (100.64.0.2) via DERP(gru) in 28ms"), Some(28));
        assert_eq!(super::parse_latency("pong from friend (100.64.0.2) via 10.0.0.2:123 in 1.25s"), Some(1250));
        assert_eq!(super::parse_latency("ping timed out"), None);
    }
}
