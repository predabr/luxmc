use super::*;
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::sync::OnceLock;

static AGENT: OnceLock<tokio::sync::Mutex<Option<(String, tokio::process::Child)>>> = OnceLock::new();
static SETUP: tokio::sync::Mutex<()> = tokio::sync::Mutex::const_new(());

#[derive(Default, Serialize, Deserialize)]
struct Link { claim: String, secret: String, address: String }

async fn api(http: &reqwest::Client, path: &str, secret: &str, body: Value) -> AppResult<Value> {
    let mut request = http.post(format!("https://api.playit.gg{path}")).json(&body).timeout(Duration::from_secs(20));
    if !secret.is_empty() { request = request.header("Authorization", format!("Agent-Key {secret}")); }
    let mut response = request.send().await?.error_for_status()?;
    let mut bytes = Vec::new();
    while let Some(chunk) = response.chunk().await? {
        if bytes.len() + chunk.len() > 2 * 1024 * 1024 { return Err(invalid("Resposta Playit muito grande.")); }
        bytes.extend_from_slice(&chunk);
    }
    let result: Value = serde_json::from_slice(&bytes)?;
    if result["status"] != "success" {
        let code = result["data"].as_str().unwrap_or("Pedido recusado");
        return Err(unavailable(format!("Playit: {}. Confira a vinculação e os limites da sua conta no painel oficial.", code.chars().take(100).collect::<String>())));
    }
    Ok(result["data"].clone())
}

async fn save(path: &Path, link: &Link) -> AppResult<()> {
    crate::core::mods::pack_download::atomic_write(path, &serde_json::to_vec(link)?).await?;
    #[cfg(unix)]
    { use std::os::unix::fs::PermissionsExt; tokio::fs::set_permissions(path, std::fs::Permissions::from_mode(0o600)).await?; }
    Ok(())
}

fn binary() -> AppResult<(&'static str, &'static str)> {
    if cfg!(all(target_os = "windows", target_arch = "x86_64")) {
        Ok(("playit-windows-x86_64-signed.exe", "2dbdaad119844cbbc062cc9774b8b462afa5f1b4b7832a9fc5ef4676cae887cf"))
    } else if cfg!(all(target_os = "linux", target_arch = "x86_64")) {
        Ok(("playit-linux-amd64", "2df7d9f10227ab312b1ad341853db4e8a8243df5cfcdbae58713a4271711c339"))
    } else { Err(unavailable("Integração Playit automática disponível para Windows e Linux x64. Use o agente oficial na sua plataforma.")) }
}

async fn executable(http: &reqwest::Client, directory: &Path) -> AppResult<PathBuf> {
    let (name, expected) = binary()?;
    let path = directory.join(name);
    crate::core::mods::pack_download::atomic_write(&directory.join(".luxmc-playit-LICENSE.txt"), include_bytes!("../../../licenses/playit-BSD-2-Clause.txt")).await?;
    if let Ok(bytes) = tokio::fs::read(&path).await {
        if format!("{:x}", Sha256::digest(&bytes)) == expected { return Ok(path); }
    }
    let mut response = http.get(format!("https://github.com/playit-cloud/playit-agent/releases/download/v1.0.10/{name}")).timeout(Duration::from_secs(120)).send().await?.error_for_status()?;
    let mut bytes = Vec::new();
    while let Some(chunk) = response.chunk().await? {
        if bytes.len() + chunk.len() > 40 * 1024 * 1024 { return Err(invalid("Agente Playit excede o limite de tamanho.")); }
        bytes.extend_from_slice(&chunk);
    }
    if format!("{:x}", Sha256::digest(&bytes)) != expected { return Err(invalid("O agente Playit não passou na verificação SHA-256. Não foi executado.")); }
    crate::core::mods::pack_download::atomic_write(&path, &bytes).await?;
    #[cfg(unix)]
    { use std::os::unix::fs::PermissionsExt; tokio::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o700)).await?; }
    Ok(path)
}

pub async fn stop(id: &str) -> AppResult<()> {
    let mut agent = AGENT.get_or_init(|| tokio::sync::Mutex::new(None)).lock().await;
    if agent.as_ref().is_some_and(|(owner, _)| owner == id) {
        if let Some((_, mut child)) = agent.take() { let _ = child.kill().await; let _ = child.wait().await; }
    }
    Ok(())
}

impl LocalServerManager {
    pub async fn public_tunnel(&self, id: &str, action: &str) -> AppResult<Value> {
        let _setup = SETUP.try_lock().map_err(|_| unavailable("Aguarde a configuração do Playit em andamento."))?;
        let directory = self.directory(id).await?;
        let path = directory.join(".luxmc-playit.json");
        let mut link: Link = self.read(&directory, ".luxmc-playit.json").await?.map(|bytes| serde_json::from_slice(&bytes)).transpose()?.unwrap_or_default();
        if action == "stop" { stop(id).await?; }
        if action == "claim" && link.secret.is_empty() {
            link.claim = uuid::Uuid::new_v4().simple().to_string()[..10].into();
            api(&self.http, "/claim/setup", "", json!({"code":link.claim,"agent_type":"assignable","version":"Luxmc / playit 1.0.10"})).await?;
            save(&path, &link).await?;
        }
        if action == "status" && link.secret.is_empty() && !link.claim.is_empty() {
            let state = api(&self.http, "/claim/setup", "", json!({"code":link.claim,"agent_type":"assignable","version":"Luxmc / playit 1.0.10"})).await?;
            if state == "UserAccepted" {
                let exchange = api(&self.http, "/claim/exchange", "", json!({"code":link.claim})).await?;
                link.secret = exchange["secret_key"].as_str().filter(|value| value.len() >= 16 && value.len() <= 512 && value.bytes().all(|byte| byte.is_ascii_hexdigit())).ok_or_else(|| unavailable("Chave Playit inválida."))?.into();
                save(&path,&link).await?;
            }
        }
        if action == "connect" {
            let port = self.running_port(id)?;
            if link.secret.is_empty() {
                if link.claim.is_empty() { return Err(unavailable("Vincule sua conta Playit primeiro.")); }
                api(&self.http, "/claim/setup", "", json!({"code":link.claim,"agent_type":"assignable","version":"Luxmc / playit 1.0.10"})).await?;
                let exchange = api(&self.http, "/claim/exchange", "", json!({"code":link.claim})).await?;
                link.secret = exchange["secret_key"].as_str().filter(|value| value.len() >= 16 && value.len() <= 512 && value.bytes().all(|byte| byte.is_ascii_hexdigit())).ok_or_else(|| unavailable("Chave Playit inválida."))?.into();
                save(&path, &link).await?;
            }
            let data = api(&self.http, "/agents/rundata", &link.secret, json!({})).await?;
            let tunnel = data["tunnels"].as_array().and_then(|tunnels| tunnels.iter().find(|tunnel| tunnel["tunnel_type"] == "minecraft-java" && tunnel["local_ip"] == "127.0.0.1" && tunnel["local_port"] == port)).cloned();
            if tunnel.is_none() {
                return Err(unavailable(format!("Crie um túnel Minecraft Java no painel Playit, selecione o agente Luxmc e use o destino 127.0.0.1:{port}. Depois clique em Conectar endereço público. A chave do agente permite consultar túneis, mas não criá-los na sua conta.")));
            }
            link.address = address(&data, port).ok_or_else(|| unavailable("O Playit ainda não informou o endereço. Confira o túnel Minecraft Java no painel oficial e tente conectar novamente."))?;
            save(&path, &link).await?;
            let executable = executable(&self.http, &directory).await?;
            if self.running_port(id).ok() != Some(port) { return Err(unavailable("O servidor parou durante a configuração. Inicie-o antes de conectar o endereço público.")); }
            let secret_path = directory.join(".luxmc-playit-secret");
            crate::core::mods::pack_download::atomic_write(&secret_path, link.secret.as_bytes()).await?;
            #[cfg(unix)]
            { use std::os::unix::fs::PermissionsExt; tokio::fs::set_permissions(&secret_path, std::fs::Permissions::from_mode(0o600)).await?; }
            let mut agent = AGENT.get_or_init(|| tokio::sync::Mutex::new(None)).lock().await;
            if agent.as_mut().is_some_and(|(owner, child)| owner == id && child.try_wait().ok().flatten().is_none()) {
                return Ok(json!({"linked":true,"address":link.address,"agentRunning":true}));
            }
            if let Some((_, mut child)) = agent.take() { let _ = child.kill().await; }
            let mut command = tokio::process::Command::new(executable);
            command.arg("--secret-path").arg(secret_path).arg("--socket-path").arg(format!("@luxmc-playit-{id}")).current_dir(&directory).stdin(Stdio::null()).stdout(Stdio::null()).stderr(Stdio::null()).kill_on_drop(true);
            #[cfg(windows)]
            command.creation_flags(0x08000000);
            *agent = Some((id.into(), command.spawn()?));
        }
        let mut agent = AGENT.get_or_init(|| tokio::sync::Mutex::new(None)).lock().await;
        let active = agent.as_mut().is_some_and(|(owner, child)| owner == id && child.try_wait().ok().flatten().is_none());
        Ok(json!({"linked":!link.secret.is_empty(),"claimUrl":if link.secret.is_empty() && !link.claim.is_empty() {Some(format!("https://playit.gg/claim/{}",link.claim))} else {None},"address":link.address,"agentRunning":active}))
    }
}

fn address(data: &Value, port: u16) -> Option<String> {
    let tunnel = data["tunnels"].as_array()?.iter().find(|tunnel| tunnel["tunnel_type"] == "minecraft-java" && tunnel["local_ip"] == "127.0.0.1" && tunnel["local_port"] == port && tunnel["disabled"].is_null())?;
    let domain = tunnel["custom_domain"].as_str().filter(|value| !value.is_empty()).or_else(|| tunnel["assigned_domain"].as_str())?;
    if domain.len() > 253 || !domain.contains('.') || !domain.bytes().all(|byte| byte.is_ascii_alphanumeric() || byte == b'.' || byte == b'-') { return None; }
    Some(domain.into())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn only_matching_enabled_loopback_minecraft_tunnel_is_advertised() {
        let mut data = json!({"tunnels":[{"tunnel_type":"minecraft-java","local_ip":"127.0.0.1","local_port":25565,"disabled":null,"assigned_domain":"example.joinmc.link"}]});
        assert_eq!(address(&data,25565).as_deref(),Some("example.joinmc.link"));
        assert!(address(&data,25566).is_none());
        data["tunnels"][0]["assigned_domain"] = json!("https://evil/path");
        assert!(address(&data,25565).is_none());
    }
}
