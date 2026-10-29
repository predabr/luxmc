use crate::{error::{AppError, AppResult}, state::AppState};
use serde::{Deserialize, Serialize, de::DeserializeOwned};
use serde_json::Value;
use std::{path::PathBuf, sync::{Arc, OnceLock}, time::Duration};
use tokio::{io::{AsyncReadExt, AsyncWriteExt}, net::{TcpListener, TcpStream}};

const MAX_FRAME: usize = 16 * 1024 * 1024;
static CONNECT: OnceLock<tokio::sync::Mutex<()>> = OnceLock::new();

#[derive(Serialize, Deserialize)]
struct Endpoint { port: u16, secret: String, revision: String }

#[derive(Serialize, Deserialize)]
struct Request { secret: String, command: String, args: Value }

#[derive(Serialize, Deserialize)]
struct Response { result: Option<Value>, error: Option<String> }

fn authorized(request: &Request, secret: &str) -> bool {
    request.secret == secret && request.command.starts_with("local_server_")
}

fn failure(message: impl Into<String>) -> AppError { AppError::InvalidState(message.into()) }

fn root() -> AppResult<PathBuf> {
    Ok(directories::ProjectDirs::from("io", "github", "Luxmc")
        .ok_or_else(|| failure("Pasta de dados indisponível."))?.data_dir().join("server-worker"))
}

pub fn is_worker() -> bool { std::env::args().any(|arg| arg == "--server-worker") }

async fn frame<T: DeserializeOwned>(stream: &mut TcpStream) -> AppResult<T> {
    let size = stream.read_u32().await? as usize;
    if size > MAX_FRAME { return Err(failure("Resposta do serviço de servidores muito grande.")); }
    let mut bytes = vec![0; size];
    stream.read_exact(&mut bytes).await?;
    Ok(serde_json::from_slice(&bytes)?)
}

async fn write<T: Serialize>(stream: &mut TcpStream, value: &T) -> AppResult<()> {
    let bytes = serde_json::to_vec(value)?;
    if bytes.len() > MAX_FRAME { return Err(failure("Pedido do serviço de servidores muito grande.")); }
    stream.write_u32(bytes.len() as u32).await?;
    stream.write_all(&bytes).await?;
    Ok(())
}

async fn existing(directory:&std::path::Path)->AppResult<Option<(TcpStream,Endpoint)>> {
    let bytes=match tokio::fs::read(directory.join("endpoint.json")).await {Ok(bytes)=>bytes,Err(error) if error.kind()==std::io::ErrorKind::NotFound=>return Ok(None),Err(error)=>return Err(error.into())};
    let endpoint:Endpoint=serde_json::from_slice(&bytes)?;
    let stream=match tokio::time::timeout(Duration::from_secs(1),TcpStream::connect(("127.0.0.1",endpoint.port))).await {Ok(Ok(stream))=>stream,_=>return Ok(None)};
    Ok(Some((stream,endpoint)))
}

pub(crate) async fn invoke_if_running<T:DeserializeOwned>(command:&str,args:Value)->AppResult<Option<T>> {
    let Some((mut stream,endpoint))=existing(&root()?).await? else{return Ok(None)};
    let response:Response=tokio::time::timeout(Duration::from_secs(5),async {
        write(&mut stream,&Request{secret:endpoint.secret,command:command.into(),args}).await?;
        frame(&mut stream).await
    }).await.map_err(|_|failure("Não foi possível conferir a sala do servidor. Tente novamente."))??;
    if let Some(error)=response.error{return Err(failure(error));}
    Ok(Some(serde_json::from_value(response.result.unwrap_or(Value::Null))?))
}

async fn connect() -> AppResult<(TcpStream, Endpoint)> {
    let _guard = CONNECT.get_or_init(|| tokio::sync::Mutex::new(())).lock().await;
    let directory = root()?;
    tokio::fs::create_dir_all(&directory).await?;
    let path = directory.join("endpoint.json");
    if let Ok(Some(value)) = existing(&directory).await { return Ok(value); }
    let source = std::env::current_exe()?;
    let runtime = directory.join(env!("LUXMC_BUILD_REVISION"));
    tokio::fs::create_dir_all(&runtime).await?;
    let executable = runtime.join(if cfg!(windows) { "luxmc-server.exe" } else { "luxmc-server" });
    if !executable.exists() { tokio::fs::copy(source, &executable).await?; }
    let mut command = std::process::Command::new(executable);
    command.arg("--server-worker").stdin(std::process::Stdio::null()).stdout(std::process::Stdio::null()).stderr(std::process::Stdio::null());
    #[cfg(windows)]
    { use std::os::windows::process::CommandExt; command.creation_flags(0x08000000 | 0x00000200); }
    command.spawn()?;
    for _ in 0..100 {
        tokio::time::sleep(Duration::from_millis(100)).await;
        if let Ok(bytes) = tokio::fs::read(&path).await {
            if let Ok(endpoint) = serde_json::from_slice::<Endpoint>(&bytes) {
                if let Ok(stream) = TcpStream::connect(("127.0.0.1", endpoint.port)).await { return Ok((stream, endpoint)); }
            }
        }
    }
    Err(failure("Não foi possível abrir o serviço de servidores. Tente novamente."))
}

pub(crate) async fn invoke<T: DeserializeOwned>(command: &str, args: Value) -> AppResult<T> {
    let (mut stream, endpoint) = connect().await?;
    let timeout = if command == "local_server_manage" { Duration::from_secs(900) } else { Duration::from_secs(50) };
    let response: Response = tokio::time::timeout(timeout, async {
        write(&mut stream, &Request { secret: endpoint.secret, command: command.into(), args }).await?;
        frame(&mut stream).await
    }).await.map_err(|_| failure("O serviço de servidores demorou a responder. Reabra o painel para acompanhar a operação; não repita a importação."))??;
    if let Some(error) = response.error { return Err(failure(error)); }
    Ok(serde_json::from_value(response.result.unwrap_or(Value::Null))?)
}

pub async fn run() -> AppResult<()> {
    let directory = root()?;
    tokio::fs::create_dir_all(&directory).await?;
    let lock = std::fs::OpenOptions::new().create(true).truncate(false).write(true).open(directory.join("worker.lock"))?;
    if lock.try_lock().is_err() { return Ok(()); }
    let listener = TcpListener::bind("127.0.0.1:0").await?;
    let endpoint = Endpoint { port: listener.local_addr()?.port(), secret: format!("{}{}", uuid::Uuid::new_v4(), uuid::Uuid::new_v4()), revision: env!("LUXMC_BUILD_REVISION").into() };
    let bytes = serde_json::to_vec(&endpoint)?;
    #[cfg(unix)]
    { use std::os::unix::fs::PermissionsExt; tokio::fs::set_permissions(&directory, std::fs::Permissions::from_mode(0o700)).await?; }
    crate::core::mods::pack_download::atomic_write(&directory.join("endpoint.json"), &bytes).await?;
    let state = Arc::new(AppState::default());
    if let Ok(db)=crate::db::shared_db().await { let _=crate::commands::settings::apply_stored_download_limit(&db).await; }
    let secret = Arc::new(endpoint.secret);
    let permits = Arc::new(tokio::sync::Semaphore::new(16));
    let _lock = lock;
    loop {
        let (mut stream, _) = match tokio::time::timeout(Duration::from_secs(120), listener.accept()).await {
            Ok(value) => value?,
            Err(_) if permits.available_permits() == 16 && !crate::commands::local_servers::has_active() && crate::network::p2p_tunnel::tunnel_status().await?.is_none() => {
                let _ = tokio::fs::remove_file(directory.join("endpoint.json")).await;
                return Ok(());
            }
            Err(_) => continue,
        };
        let Ok(permit) = permits.clone().try_acquire_owned() else { continue; };
        let state = state.clone(); let secret = secret.clone();
        tokio::spawn(async move {
            let _permit = permit;
            let request: Request = match tokio::time::timeout(Duration::from_secs(5), frame(&mut stream)).await { Ok(Ok(value)) => value, _ => return };
            if !authorized(&request, &secret) { return; }
            let result = crate::daemon::dispatch_local_server(&request.command, request.args, &state).await;
            let response = match result { Ok(value) => Response { result: Some(value), error: None }, Err(error) => Response { result: None, error: Some(error.to_string()) } };
            let _ = write(&mut stream, &response).await;
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[tokio::test]
    async fn looking_for_an_existing_room_does_not_create_worker_files_or_start_a_service() {
        let directory=tempfile::tempdir().unwrap();
        assert!(existing(directory.path()).await.unwrap().is_none());
        assert_eq!(std::fs::read_dir(directory.path()).unwrap().count(),0);
    }
    #[test]
    fn server_worker_requires_its_private_secret_and_rejects_general_launcher_commands() {
        let mut request = Request { secret:"private".into(), command:"local_server_list".into(), args:Value::Null };
        assert!(authorized(&request,"private"));
        assert!(!authorized(&request,"other"));
        request.command = "launch_game".into();
        assert!(!authorized(&request,"private"));
    }

    #[tokio::test]
    async fn server_worker_rejects_oversized_frames_before_allocating_or_dispatching() {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let port = listener.local_addr().unwrap().port();
        let sender = tokio::spawn(async move { let mut stream = TcpStream::connect(("127.0.0.1",port)).await.unwrap(); stream.write_u32((MAX_FRAME + 1) as u32).await.unwrap(); });
        let (mut stream,_) = listener.accept().await.unwrap();
        let result = tokio::time::timeout(Duration::from_secs(1),frame::<Request>(&mut stream)).await.unwrap();
        assert!(result.is_err()); sender.await.unwrap();
    }
}
