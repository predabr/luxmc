use crate::error::{AppError, AppResult};
use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{path::PathBuf, sync::LazyLock};

const VERSION: &str = "v2.6.4";
const ARCHIVE_HASH: &str = "27af91e270e554709b048bd32327fefd2dfce5062ae1e8701af7550c6f525f84";
const FILES: [(&str, &str); 4] = [
    ("easytier-core.exe", "da7eb2d24b5416f3d3407636949e964a0750e3f9dc53a828cb6799a57ead445d"),
    ("easytier-cli.exe", "d8783e851e944b44a9b71b39fd02f227ec0a2a82b3165c55ead5dd32dcde53a1"),
    ("wintun.dll", "e5da8447dc2c320edc0fc52fa01885c103de8c118481f683643cacc3220dafce"),
    ("Packet.dll", "c7c03a87eac7243ccbe331554624b18803010b740e311fc8cfddb573096eacac"),
];
static SESSION: LazyLock<tokio::sync::Mutex<Option<Session>>> = LazyLock::new(|| tokio::sync::Mutex::new(None));
static SETUP: tokio::sync::Mutex<()> = tokio::sync::Mutex::const_new(());
static PREPARATION: std::sync::Mutex<Option<LanPreparation>> = std::sync::Mutex::new(None);

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LanPreparation { stage: &'static str, downloaded_bytes: u64, total_bytes: Option<u64> }

fn preparation(stage: &'static str, downloaded_bytes: u64, total_bytes: Option<u64>) {
    if let Ok(mut state) = PREPARATION.lock() { *state = Some(LanPreparation { stage, downloaded_bytes, total_bytes }); }
}

struct PreparationGuard;
impl Drop for PreparationGuard {
    fn drop(&mut self) { if let Ok(mut state) = PREPARATION.lock() { *state = None; } }
}

#[derive(Serialize, Deserialize)]
struct Invitation { secret: String, expires: u64 }
#[derive(Serialize, Deserialize)]
struct BrokerConfig { secret: String, expires: u64, name: String, parent_pid: u32, parent_start: u64, rpc_port: u16 }
struct Session { id: String, port: u16, invitation: String, started: u64, last_healthy: Option<(u64, String, Vec<LanPeer>)>, worlds: tokio::task::JoinHandle<()> }
#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LanPeer { name: String, address: String }
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LanStatus { pub installed: bool, pub supported: bool, pub active: bool, pub invitation: Option<String>, pub address: Option<String>, pub peers: Vec<LanPeer>, pub error: Option<String>, pub preparation: Option<LanPreparation> }

fn now() -> u64 { std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_secs() }
fn root() -> AppResult<PathBuf> {
    directories::ProjectDirs::from("io", "github", "Luxmc").map(|dirs| dirs.data_dir().join("virtual-lan"))
        .ok_or_else(|| AppError::InvalidState("Diretório de dados indisponível".into()))
}
fn runtime() -> AppResult<PathBuf> { Ok(root()?.join(VERSION)) }
fn session_path(id: &str, extension: &str) -> AppResult<PathBuf> {
    uuid::Uuid::parse_str(id).map_err(|_| AppError::InvalidInput("Sessão inválida".into()))?;
    Ok(root()?.join("sessions").join(format!("{id}.{extension}")))
}
fn check_runtime() -> AppResult<()> {
    let root = runtime()?;
    for (name, expected) in FILES {
        let metadata = std::fs::symlink_metadata(root.join(name))?;
        if !metadata.is_file() || metadata.file_type().is_symlink() || metadata.len() > 32 * 1024 * 1024 {
            return Err(AppError::InvalidState("Componente de rede inválido".into()));
        }
        let bytes = std::fs::read(root.join(name))?;
        if format!("{:x}", Sha256::digest(bytes)) != expected { return Err(AppError::InvalidState("Falha na integridade do componente de rede".into())); }
    }
    if root.join("WinDivert64.sys").exists() {
        return Err(AppError::InvalidState("A pasta da rede contém componentes não autorizados".into()));
    }
    Ok(())
}
fn invitation(value: &str) -> AppResult<Invitation> {
    if value.len() > 1024 { return Err(AppError::InvalidInput("Convite de rede inválido".into())); }
    let value = value.trim().strip_prefix("luxmc-lan:").ok_or_else(|| AppError::InvalidInput("Cole o convite completo luxmc-lan:".into()))?;
    let bytes = URL_SAFE_NO_PAD.decode(value).map_err(|_| AppError::InvalidInput("Convite de rede inválido".into()))?;
    let invite: Invitation = serde_json::from_slice(&bytes)?;
    if invite.secret.len() != 64 || !invite.secret.bytes().all(|byte| byte.is_ascii_hexdigit()) || invite.expires <= now() || invite.expires > now() + 86400 + 300 {
        return Err(AppError::InvalidInput("Convite expirado ou inválido".into()));
    }
    Ok(invite)
}
fn subnet(secret: &str) -> String {
    let bytes = Sha256::digest(secret.as_bytes());
    format!("10.{}.{}.1/24", 64 + bytes[0] % 64, 1 + bytes[1] % 253)
}

fn selected_components(bytes: Vec<u8>, files: &[(&str, &str)]) -> AppResult<Vec<(String, Vec<u8>)>> {
    use std::io::Read;
    let mut archive = zip::ZipArchive::new(std::io::Cursor::new(bytes))?;
    let mut result = Vec::new();
    for (name, expected) in files {
        let source = archive.by_name(&format!("easytier-windows-x86_64/{name}"))?;
        if source.size() > 32 * 1024 * 1024 { return Err(AppError::InvalidState("Componente de rede excedeu o limite".into())); }
        let mut content = Vec::new();
        source.take(32 * 1024 * 1024 + 1).read_to_end(&mut content)?;
        if content.len() > 32 * 1024 * 1024 || format!("{:x}", Sha256::digest(&content)) != *expected { return Err(AppError::InvalidState("Componente de rede não confere".into())); }
        result.push(((*name).into(), content));
    }
    Ok(result)
}

#[tauri::command]
pub async fn virtual_lan_install() -> AppResult<()> {
    if !cfg!(all(target_os = "windows", target_arch = "x86_64")) { return Err(AppError::InvalidState("A rede virtual integrada requer Windows de 64 bits nesta versão".into())); }
    let _guard = SETUP.lock().await;
    let _progress = PreparationGuard;
    preparation("checking", 0, None);
    if tokio::task::spawn_blocking(|| check_runtime().is_ok()).await.unwrap_or(false) { return Ok(()); }
    preparation("downloading", 0, None);
    let client = reqwest::Client::builder().timeout(std::time::Duration::from_secs(120)).build()?;
    let response = client.get("https://github.com/EasyTier/EasyTier/releases/download/v2.6.4/easytier-windows-x86_64-v2.6.4.zip").send().await?.error_for_status()?;
    let mut response = response;
    let total = response.content_length();
    let mut bytes = Vec::new();
    while let Some(chunk) = response.chunk().await? {
        if bytes.len() + chunk.len() > 40 * 1024 * 1024 { return Err(AppError::InvalidState("Pacote de rede excedeu o limite".into())); }
        bytes.extend_from_slice(&chunk);
        preparation("downloading", bytes.len() as u64, total);
    }
    preparation("verifying", bytes.len() as u64, total);
    if format!("{:x}", Sha256::digest(&bytes)) != ARCHIVE_HASH { return Err(AppError::InvalidState("Falha na integridade do pacote de rede".into())); }
    tokio::task::spawn_blocking(move || {
        let components = selected_components(bytes, &FILES)?;
        preparation("installing", 0, None);
        let directory = runtime()?;
        std::fs::create_dir_all(&directory)?;
        for (name, content) in components {
            let temporary = directory.join(format!("{name}.tmp"));
            std::fs::write(&temporary, content)?;
            std::fs::rename(temporary, directory.join(name))?;
        }
        std::fs::write(directory.join("SOURCE.txt"), "EasyTier v2.6.4\nhttps://github.com/EasyTier/EasyTier/tree/v2.6.4\nLGPL-3.0\nhttps://github.com/EasyTier/EasyTier/blob/v2.6.4/LICENSE\nWintun: https://www.wintun.net/\nPacket.dll: Npcap 1.79\nhttps://raw.githubusercontent.com/nmap/npcap/v1.79/LICENSE\nThird-party WinPcap: https://www.winpcap.org/misc/copyright.htm\n")?;
        std::fs::write(directory.join("EasyTier-LGPL-3.0.txt"), include_str!("../../licenses/EasyTier-LGPL-3.0.txt"))?;
        std::fs::write(directory.join("GPL-3.0.txt"), include_str!("../../licenses/GPL-3.0.txt"))?;
        std::fs::write(directory.join("Wintun-LICENSE.txt"), include_str!("../../licenses/Wintun-LICENSE.txt"))?;
        std::fs::write(directory.join("WinPcap-LICENSE.txt"), include_str!("../../licenses/WinPcap-LICENSE.txt"))?;
        std::fs::write(directory.join("Npcap-LICENSE.txt"), include_str!("../../licenses/Npcap-LICENSE.txt"))?;
        check_runtime()
    }).await.map_err(|error| AppError::Internal(error.to_string()))?
}

pub fn suppress_loader_dialogs() {
    #[cfg(windows)]
    {
        #[link(name = "kernel32")]
        unsafe extern "system" { fn SetErrorMode(mode: u32) -> u32; }
        unsafe { SetErrorMode(0x8003); }
    }
}

async fn preflight() -> AppResult<()> {
    suppress_loader_dialogs();
    preparation("testing", 0, None);
    for name in ["easytier-core.exe", "easytier-cli.exe"] {
        let directory = runtime()?;
        let mut command = tokio::process::Command::new(directory.join(name));
        command.current_dir(directory).arg("--version").kill_on_drop(true);
        #[cfg(windows)]
        command.creation_flags(0x08000000);
        let output = tokio::time::timeout(std::time::Duration::from_secs(8), command.output()).await
            .map_err(|_| AppError::InvalidState("O componente de rede não respondeu à verificação inicial".into()))??;
        if !output.status.success() {
            return Err(AppError::InvalidState(format!("Não foi possível carregar {name} (código {}). Os arquivos serão conferidos na próxima tentativa. Se o antivírus bloqueou uma dependência, consulte o alerta sem desativar a proteção", output.status.code().map(|code| format!("0x{:08X}", code as u32)).unwrap_or_else(|| "indisponível".into()))));
        }
    }
    Ok(())
}

#[cfg(windows)]
fn elevate(id: &str) -> AppResult<()> {
    use std::os::windows::ffi::OsStrExt;
    #[link(name = "shell32")]
    unsafe extern "system" { fn ShellExecuteW(window: *mut std::ffi::c_void, verb: *const u16, file: *const u16, parameters: *const u16, directory: *const u16, show: i32) -> isize; }
    let wide = |text: &std::ffi::OsStr| text.encode_wide().chain(Some(0)).collect::<Vec<_>>();
    let exe = wide(std::env::current_exe()?.as_os_str());
    let verb = wide(std::ffi::OsStr::new("runas"));
    let args = wide(std::ffi::OsStr::new(&format!("--virtual-lan-broker {id}")));
    let result = unsafe { ShellExecuteW(std::ptr::null_mut(), verb.as_ptr(), exe.as_ptr(), args.as_ptr(), std::ptr::null(), 0) };
    if result <= 32 { return Err(AppError::InvalidState("A autorização do Windows para criar a rede foi cancelada ou falhou".into())); }
    Ok(())
}
#[cfg(not(windows))]
fn elevate(_id: &str) -> AppResult<()> { Err(AppError::InvalidState("Rede virtual integrada indisponível nesta plataforma".into())) }

#[tauri::command]
pub async fn virtual_lan_connect(name: String, invite: Option<String>) -> AppResult<()> {
    if !cfg!(all(target_os = "windows", target_arch = "x86_64")) { return Err(AppError::InvalidState("Rede virtual integrada indisponível nesta plataforma".into())); }
    let name = name.trim().to_string();
    if name.is_empty() || name.chars().count() > 32 || name.chars().any(char::is_control) { return Err(AppError::InvalidInput("Use um nome de 1 a 32 caracteres".into())); }
    let invite = match invite { Some(value) => invitation(&value)?, None => Invitation { secret: format!("{}{}", uuid::Uuid::new_v4().simple(), uuid::Uuid::new_v4().simple()), expires: now() + 86400 } };
    if SESSION.lock().await.is_some() { return Err(AppError::InvalidState("Saia da rede atual antes de conectar a outra".into())); }
    virtual_lan_install().await?;
    let _setup = SETUP.lock().await;
    let _progress = PreparationGuard;
    tokio::task::spawn_blocking(check_runtime).await.map_err(|error| AppError::Internal(error.to_string()))??;
    preflight().await?;
    let session = SESSION.lock().await;
    if session.is_some() { return Err(AppError::InvalidState("Saia da rede atual antes de conectar a outra".into())); }
    let secret = format!("{:0>64}", invite.secret);
    let invite = Invitation { secret, ..invite };
    let encoded = format!("luxmc-lan:{}", URL_SAFE_NO_PAD.encode(serde_json::to_vec(&invite)?));
    let socket = std::net::TcpListener::bind("127.0.0.1:0")?;
    let rpc_port = socket.local_addr()?.port();
    drop(socket);
    let parent_pid = std::process::id();
    let parent_start = tokio::task::spawn_blocking(move || {
        let mut system = sysinfo::System::new();
        let pid = sysinfo::Pid::from_u32(parent_pid);
        system.refresh_processes(sysinfo::ProcessesToUpdate::Some(&[pid]), true);
        system.process(pid).map(|process| process.start_time()).unwrap_or_default()
    }).await.map_err(|error| AppError::Internal(error.to_string()))?;
    let id = uuid::Uuid::new_v4().to_string();
    let path = session_path(&id, "json")?;
    std::fs::create_dir_all(path.parent().unwrap())?;
    let world_secret = invite.secret.clone();
    std::fs::write(&path, serde_json::to_vec(&BrokerConfig { secret: invite.secret, expires: invite.expires, name, parent_pid, parent_start, rpc_port })?)?;
    preparation("authorizing", 0, None);
    drop(session);
    let broker_id = id.clone();
    let elevated = tokio::task::spawn_blocking(move || elevate(&broker_id)).await.map_err(|error| AppError::Internal(error.to_string()))?;
    if let Err(error) = elevated { let _ = std::fs::remove_file(path); return Err(error); }
    let mut session = SESSION.lock().await;
    super::virtual_world::reset().await;
    let worlds = tokio::spawn(super::virtual_world::run(rpc_port, world_secret));
    *session = Some(Session { id, port: rpc_port, invitation: encoded, started: now(), last_healthy: None, worlds });
    Ok(())
}

async fn query_command(port: u16, arguments: &[&str]) -> AppResult<serde_json::Value> {
    let mut command = tokio::process::Command::new(runtime()?.join("easytier-cli.exe"));
    command.args(["--rpc-portal", &format!("127.0.0.1:{port}"), "--output", "json"]).args(arguments);
    command.kill_on_drop(true);
    #[cfg(windows)]
    command.creation_flags(0x08000000);
    let output = tokio::time::timeout(std::time::Duration::from_secs(3), command.output()).await
        .map_err(|_| AppError::InvalidState("A rede ainda está iniciando ou não respondeu".into()))??;
    if !output.status.success() || output.stdout.len() > 1024 * 1024 { return Err(AppError::InvalidState("A rede ainda está iniciando ou o adaptador não pôde ser criado".into())); }
    Ok(serde_json::from_slice(&output.stdout)?)
}

fn queried_status(node: serde_json::Value, peers: serde_json::Value) -> AppResult<serde_json::Value> {
    if !node.is_object() || !peers.is_array() { return Err(AppError::InvalidState("O componente de rede retornou um formato inesperado".into())); }
    let routes: Vec<_> = peers.as_array().unwrap().iter().map(|peer| serde_json::json!({ "route": { "hostname": peer.get("hostname"), "ipv4_addr": peer.get("ipv4") } })).collect();
    Ok(serde_json::json!({ "node_info": { "ipv4_addr": node.get("ipv4_addr") }, "peer_routes": routes }))
}

pub(super) async fn query(port: u16) -> AppResult<serde_json::Value> {
    static CACHE: tokio::sync::Mutex<Option<(u16, tokio::time::Instant, serde_json::Value)>> = tokio::sync::Mutex::const_new(None);
    let mut cache = CACHE.lock().await;
    if let Some((cached_port, time, value)) = cache.as_ref() {
        if *cached_port == port && time.elapsed() < std::time::Duration::from_secs(2) { return Ok(value.clone()); }
    }
    let (node, peers) = tokio::try_join!(query_command(port, &["node", "info"]), query_command(port, &["peer"]))?;
    let value = queried_status(node, peers)?;
    *cache = Some((port, tokio::time::Instant::now(), value.clone()));
    Ok(value)
}

#[tauri::command]
pub async fn virtual_lan_status() -> AppResult<LanStatus> {
    let progress = PREPARATION.lock().ok().and_then(|state| state.clone());
    let snapshot = SESSION.lock().await.as_ref().map(|session| (session.id.clone(), session.port, session.invitation.clone(), session.started, session.last_healthy.clone()));
    let installed = if snapshot.is_some() { true } else if progress.is_some() { false } else { tokio::task::spawn_blocking(|| check_runtime().is_ok()).await.unwrap_or(false) };
    let mut status = LanStatus { installed, supported: cfg!(all(target_os = "windows", target_arch = "x86_64")), active: false, invitation: None, address: None, peers: Vec::new(), error: None, preparation: progress };
    let Some((id, port, encoded, started, healthy)) = snapshot else { return Ok(status); };
    let startup_timed_out = startup_expired(started, healthy.is_some(), now());
    status.invitation = Some(encoded.clone());
    let error_path = session_path(&id, "error")?;
    if error_path.exists() { status.error = Some(std::fs::read_to_string(error_path)?.chars().take(1024).collect()); return Ok(status); }
    if now() >= invitation(&encoded).map(|invite| invite.expires).unwrap_or(0) { status.error = Some("O convite expirou. Saia da sala e crie outra.".into()); return Ok(status); }
    match query(port).await {
        Ok(value) => {
            status.address = value.pointer("/node_info/ipv4_addr").and_then(|value| value.as_str()).and_then(normalize_address);
            status.active = status.address.as_ref().is_some_and(|address| std::net::UdpSocket::bind((address.as_str(), 0)).is_ok());
            if let Some(peers) = value.get("peer_routes").and_then(|value| value.as_array()) {
                for peer in peers {
                    let Some(route) = peer.get("route") else { continue; };
                    let name = route.get("hostname").and_then(|value| value.as_str()).unwrap_or("Jogador").to_string();
                    let address = route.get("ipv4_addr").and_then(|value| value.as_str()).and_then(normalize_address);
                    if let Some(address) = address { if status.address.as_ref() != Some(&address) { status.peers.push(LanPeer { name, address }); } }
                }
            }
        },
        Err(_) => { status.error = Some("Atualizando a conexão. Sua sala continua aberta.".into()); }
    }
    if status.active {
        if let Some(session) = SESSION.lock().await.as_mut().filter(|session| session.id == id) {
            session.last_healthy = Some((now(), status.address.clone().unwrap(), status.peers.clone()));
        }
    } else if let Some((seen, address, peers)) = healthy {
        if now().saturating_sub(seen) <= 20 && std::net::UdpSocket::bind((address.as_str(), 0)).is_ok() {
            status.active = true;
            status.address = Some(address);
            status.peers = peers;
        } else {
            status.error = Some("Reconectando os computadores. Mantenha a sala aberta; não é preciso criar outro convite.".into());
        }
    } else if startup_timed_out {
        let current = SESSION.lock().await;
        if !current.as_ref().is_some_and(|session| session.id == id && session.last_healthy.is_none()) { return Ok(status); }
        std::fs::write(session_path(&id, "stop")?, [])?;
        let message = "Não foi possível iniciar a sala. Saia e tente novamente; confirme a autorização do Windows quando ela aparecer.";
        std::fs::write(error_path, message)?;
        status.error = Some(message.into());
    }
    Ok(status)
}

fn startup_expired(started: u64, connected_before: bool, current: u64) -> bool {
    !connected_before && current.saturating_sub(started) >= 90
}

pub(super) async fn record_connection(port: u16, address: &str) {
    if std::net::UdpSocket::bind((address, 0)).is_err() { return; }
    if let Some(session) = SESSION.lock().await.as_mut().filter(|session| session.port == port) {
        let peers = session.last_healthy.as_ref().map(|(_, _, peers)| peers.clone()).unwrap_or_default();
        session.last_healthy = Some((now(), address.into(), peers));
    }
}

#[tauri::command]
pub async fn virtual_lan_stop() -> AppResult<()> {
    if let Some(session) = SESSION.lock().await.take() { session.worlds.abort(); std::fs::write(session_path(&session.id, "stop")?, [])?; }
    super::virtual_world::reset().await;
    Ok(())
}

pub fn run_broker(id: &str) -> AppResult<()> {
    suppress_loader_dialogs();
    let result = broker(id);
    if let Err(error) = &result { if let Ok(path) = session_path(id, "error") { let _ = std::fs::write(path, error.to_string()); } }
    result
}

pub(super) fn normalize_address(value: &str) -> Option<String> {
    let address: std::net::Ipv4Addr = value.split('/').next()?.parse().ok()?;
    (address.is_private() && !address.is_unspecified()).then(|| address.to_string())
}

fn broker(id: &str) -> AppResult<()> {
    check_runtime()?;
    let path = session_path(id, "json")?;
    let bytes = std::fs::read(&path)?;
    if bytes.len() > 4096 { return Err(AppError::InvalidInput("Configuração de rede inválida".into())); }
    let config: BrokerConfig = serde_json::from_slice(&bytes)?;
    invitation(&format!("luxmc-lan:{}", URL_SAFE_NO_PAD.encode(serde_json::to_vec(&Invitation { secret: config.secret.clone(), expires: config.expires })?)))?;
    if config.name.chars().any(char::is_control) || config.name.chars().count() > 32 || config.parent_start == 0 { return Err(AppError::InvalidInput("Configuração de rede inválida".into())); }
    let mut system = sysinfo::System::new();
    let pid = sysinfo::Pid::from_u32(config.parent_pid);
    system.refresh_processes(sysinfo::ProcessesToUpdate::Some(&[pid]), true);
    if !system.process(pid).is_some_and(|process| process.start_time() == config.parent_start && process.exe() == std::env::current_exe().ok().as_deref()) { return Err(AppError::InvalidState("Launcher não está mais aberto".into())); }
    let directory = runtime()?;
    #[cfg(windows)]
    struct FirewallRule { netsh: PathBuf, name: String }
    #[cfg(windows)]
    impl Drop for FirewallRule {
        fn drop(&mut self) {
            use std::os::windows::process::CommandExt;
            let _ = std::process::Command::new(&self.netsh).args(["advfirewall", "firewall", "delete", "rule", &self.name]).creation_flags(0x08000000).output();
        }
    }
    #[cfg(windows)]
    let _firewall_rule;
    #[cfg(windows)]
    {
        let system_root = std::env::var_os("SystemRoot").ok_or_else(|| AppError::InvalidState("Diretório do Windows indisponível".into()))?;
        let netsh = PathBuf::from(system_root).join("System32").join("netsh.exe");
        let exe = std::env::current_exe()?;
        let subnet = subnet(&config.secret).replace(".1/24", ".0/24");
        let name = format!("name=Luxmc LAN {id}");
        let mut firewall = std::process::Command::new(&netsh);
        firewall.args(["advfirewall", "firewall", "add", "rule", &name, "dir=in", "action=allow", "protocol=TCP", "localport=39945,39946", &format!("remoteip={subnet}"), &format!("program={}", exe.display()), "profile=any"]);
        use std::os::windows::process::CommandExt;
        firewall.creation_flags(0x08000000);
        if !firewall.output()?.status.success() { return Err(AppError::InvalidState("O Windows não permitiu preparar a regra de acesso aos mundos na rede virtual".into())); }
        _firewall_rule = FirewallRule { netsh, name };
    }
    let mut command = std::process::Command::new(directory.join("easytier-core.exe"));
    let network = format!("luxmc-{:x}", Sha256::digest(config.secret.as_bytes()));
    command.current_dir(&directory).env_clear()
        .env("SystemRoot", std::env::var_os("SystemRoot").unwrap_or_default())
        .env("ET_NETWORK_NAME", network).env("ET_NETWORK_SECRET", &config.secret)
        .args(["--dhcp", "--ipv4", &subnet(&config.secret), "--hostname", &config.name,
            "--dev-name", "Luxmc LAN", "--peers", "tcp://dreamlife.indevs.in:11010", "--no-listener",
            "--rpc-portal", &format!("127.0.0.1:{}", config.rpc_port), "--rpc-portal-whitelist", "127.0.0.1",
            "--relay-network-whitelist", "--console-log-level", "off"])
        .stdin(std::process::Stdio::null()).stdout(std::process::Stdio::null()).stderr(std::process::Stdio::null());
    for key in ["LOCALAPPDATA", "APPDATA", "USERPROFILE", "TEMP", "TMP"] {
        if let Some(value) = std::env::var_os(key) { command.env(key, value); }
    }
    #[cfg(windows)]
    { use std::os::windows::process::CommandExt; command.creation_flags(0x08000000); }
    struct NetworkChild(std::process::Child);
    impl Drop for NetworkChild { fn drop(&mut self) { let _ = self.0.kill(); let _ = self.0.wait(); } }
    let mut child = NetworkChild(command.spawn()?);
    loop {
        if let Some(exit) = child.0.try_wait()? { return Err(AppError::InvalidState(format!("O processo da rede encerrou (código {}). Confira os componentes de rede e a autorização do Windows", exit.code().map(|code| format!("0x{:08X}", code as u32)).unwrap_or_else(|| "indisponível".into())))); }
        system.refresh_processes(sysinfo::ProcessesToUpdate::Some(&[pid]), true);
        if session_path(id, "stop")?.exists() || now() >= config.expires || !system.process(pid).is_some_and(|process| process.start_time() == config.parent_start) {
            let _ = child.0.kill(); let _ = child.0.wait();
            let _ = std::fs::remove_file(path);
            break;
        }
        std::thread::sleep(std::time::Duration::from_secs(1));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn startup_deadline_never_stops_an_established_session() {
        assert!(!startup_expired(100, false, 160));
        assert!(startup_expired(100, false, 190));
        assert!(!startup_expired(100, true, 190));
        assert!(!startup_expired(100, true, 86500));
    }
    #[test]
    fn reads_actual_cli_formats_without_exposing_configuration() {
        let node = serde_json::json!({ "ipv4_addr": "10.100.1.1/24", "config": { "network_secret": "private" } });
        let peers = serde_json::json!([{ "ipv4": "10.100.1.2", "hostname": "Alex", "lat_ms": 12 }]);
        let status = queried_status(node, peers).unwrap();
        assert_eq!(status.pointer("/node_info/ipv4_addr").unwrap(), "10.100.1.1/24");
        assert_eq!(status.pointer("/peer_routes/0/route/ipv4_addr").unwrap(), "10.100.1.2");
        assert!(!status.to_string().contains("private"));
        assert!(queried_status(serde_json::json!({}), serde_json::json!({})).is_err());
    }
    #[test]
    fn rejects_expired_and_malformed_invites() {
        for invite in [Invitation { secret: "a".repeat(64), expires: now() - 1 }, Invitation { secret: "not-a-secret".into(), expires: now() + 60 }] {
            let encoded = format!("luxmc-lan:{}", URL_SAFE_NO_PAD.encode(serde_json::to_vec(&invite).unwrap()));
            assert!(invitation(&encoded).is_err());
        }
        assert!(invitation("luxmc-world:old").is_err());
    }
    #[test]
    fn permitted_runtime_never_contains_packet_interception_driver() {
        assert_eq!(FILES.len(), 4);
        assert!(!FILES.iter().any(|(name, _)| name.contains("WinDivert") || name.ends_with(".sys")));
        assert!(FILES.iter().any(|(name, _)| *name == "Packet.dll"));
        assert_eq!(subnet(&"a".repeat(64)), subnet(&"a".repeat(64)));
        assert!(session_path("../../escape", "json").is_err());
        assert_eq!(normalize_address("10.100.1.2/24").as_deref(), Some("10.100.1.2"));
        assert!(normalize_address("0.0.0.0").is_none());
        assert!(normalize_address("8.8.8.8").is_none());
    }
    #[test]
    fn archive_skips_unrequested_drivers_and_paths_and_rejects_changed_components() {
        use std::io::Write;
        let mut archive = zip::ZipWriter::new(std::io::Cursor::new(Vec::new()));
        for (name, bytes) in [("easytier-windows-x86_64/easytier-core.exe", b"fixture".as_slice()), ("easytier-windows-x86_64/WinDivert64.sys", b"unrequested".as_slice()), ("../../escape.exe", b"unrequested".as_slice())] {
            archive.start_file(name, zip::write::FileOptions::default()).unwrap();
            archive.write_all(bytes).unwrap();
        }
        let bytes = archive.finish().unwrap().into_inner();
        let digest = format!("{:x}", Sha256::digest(b"fixture"));
        let selected = selected_components(bytes.clone(), &[("easytier-core.exe", &digest)]).unwrap();
        assert_eq!(selected.len(), 1);
        assert_eq!(selected[0].0, "easytier-core.exe");
        assert_eq!(selected[0].1, b"fixture");
        assert!(selected_components(bytes, &[("easytier-core.exe", &"0".repeat(64))]).is_err());
    }
}
