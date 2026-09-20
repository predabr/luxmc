use crate::error::AppResult;
use serde::{Deserialize, Serialize};
use std::io::{Read, Write};
use std::sync::Mutex;

#[cfg(unix)]
use std::os::unix::net::UnixStream;

#[cfg(windows)]
use std::fs::OpenOptions;

const MINECRAFT_CLIENT_ID: &str = "450485984333660181";
const LUXMC_ICON_URL: &str =
    "https://raw.githubusercontent.com/predabr/luxmc/main/src-tauri/icons/icon.png";
const LUXMC_VERSION: &str = "1.9.0";
const MINECRAFT_GRASS_ASSET: &str = "grass";

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DiscordButton {
    pub label: String,
    pub url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[allow(dead_code)]
pub struct DiscordActivityArgs {
    pub details: Option<String>,
    pub state: Option<String>,
    pub large_text: Option<String>,
    pub large_image: Option<String>,
    pub small_text: Option<String>,
    pub small_image: Option<String>,
    pub start_time: Option<i64>,
    pub in_game: Option<bool>,
    pub client_id: Option<String>,
    pub buttons: Option<Vec<DiscordButton>>,
}

enum IpcStream {
    #[cfg(unix)]
    Unix(UnixStream),
    #[cfg(windows)]
    Pipe(std::fs::File),
}

impl IpcStream {
    fn write_all(&mut self, buf: &[u8]) -> std::io::Result<()> {
        match self {
            #[cfg(unix)]
            IpcStream::Unix(s) => s.write_all(buf),
            #[cfg(windows)]
            IpcStream::Pipe(f) => f.write_all(buf),
        }
    }

    fn flush(&mut self) -> std::io::Result<()> {
        match self {
            #[cfg(unix)]
            IpcStream::Unix(s) => s.flush(),
            #[cfg(windows)]
            IpcStream::Pipe(f) => f.flush(),
        }
    }

    fn read_exact(&mut self, buf: &mut [u8]) -> std::io::Result<()> {
        match self {
            #[cfg(unix)]
            IpcStream::Unix(s) => s.read_exact(buf),
            #[cfg(windows)]
            IpcStream::Pipe(f) => f.read_exact(buf),
        }
    }

    #[cfg(unix)]
    fn set_read_timeout(&mut self, dur: Option<std::time::Duration>) -> std::io::Result<()> {
        match self {
            IpcStream::Unix(s) => s.set_read_timeout(dur),
        }
    }

    #[cfg(windows)]
    fn set_read_timeout(&mut self, _dur: Option<std::time::Duration>) -> std::io::Result<()> {
        Ok(())
    }
}

struct DiscordConn {
    stream: IpcStream,
    client_id: String,
}

static DISCORD_CONN: Mutex<Option<DiscordConn>> = Mutex::new(None);

fn send_frame(stream: &mut IpcStream, opcode: u32, payload: &str) -> std::io::Result<()> {
    let len = payload.len() as u32;
    let mut header = [0u8; 8];
    header[0..4].copy_from_slice(&opcode.to_le_bytes());
    header[4..8].copy_from_slice(&len.to_le_bytes());
    stream.write_all(&header)?;
    stream.write_all(payload.as_bytes())?;
    stream.flush()
}

fn read_frame_header(stream: &mut IpcStream) -> std::io::Result<u32> {
    let mut header = [0u8; 8];
    stream.read_exact(&mut header)?;
    Ok(u32::from_le_bytes([header[4], header[5], header[6], header[7]]))
}

fn drain_response(stream: &mut IpcStream) {
    if let Ok(len) = read_frame_header(stream) {
        let mut buf = vec![0u8; len as usize];
        let _ = stream.read_exact(&mut buf);
    }
}

#[cfg(unix)]
fn open_unix_stream() -> Option<IpcStream> {
    let runtime_dir = std::env::var("XDG_RUNTIME_DIR").unwrap_or_else(|_| "/tmp".to_string());
    let tmp_dir = std::env::var("TMPDIR").unwrap_or_else(|_| "/tmp".to_string());
    let home_dir = std::env::var("HOME").unwrap_or_default();

    let mut candidates = vec![
        std::path::PathBuf::from(&runtime_dir),
        std::path::PathBuf::from(&tmp_dir),
        std::path::PathBuf::from(&runtime_dir).join("app/com.discordapp.Discord"),
        std::path::PathBuf::from(&runtime_dir).join("app/com.discordapp.DiscordCanary"),
        std::path::PathBuf::from(&runtime_dir).join("app/com.discordapp.DiscordPTB"),
        std::path::PathBuf::from(&runtime_dir).join("app/de.vencord.Vesktop"),
        std::path::PathBuf::from(&runtime_dir).join("app/dev.vencord.Vesktop"),
        std::path::PathBuf::from("/tmp"),
        std::path::PathBuf::from("/tmp/app/com.discordapp.Discord"),
        std::path::PathBuf::from(&home_dir).join(".var/app/com.discordapp.Discord/config"),
        std::path::PathBuf::from(&home_dir).join(".var/app/de.vencord.Vesktop/config"),
        std::path::PathBuf::from(&home_dir).join(".config/discord"),
    ];

    if let Ok(entries) = std::fs::read_dir("/run/user") {
        for entry in entries.flatten() {
            candidates.push(entry.path());
            candidates.push(entry.path().join("snap.discord"));
            candidates.push(entry.path().join("app/com.discordapp.Discord"));
            candidates.push(entry.path().join("app/de.vencord.Vesktop"));
        }
    }

    for base in &candidates {
        for prefix in &["discord-ipc-", "ipc-"] {
            for i in 0..10u32 {
                let path = base.join(format!("{}{}", prefix, i));
                if path.exists() {
                    if let Ok(stream) = UnixStream::connect(&path) {
                        return Some(IpcStream::Unix(stream));
                    }
                }
            }
        }
    }
    None
}

#[cfg(windows)]
fn open_windows_pipe() -> Option<IpcStream> {
    for i in 0..10u32 {
        let pipe_path = format!(r"\\.\pipe\discord-ipc-{}", i);
        if let Ok(file) = OpenOptions::new()
            .read(true)
            .write(true)
            .open(&pipe_path)
        {
            return Some(IpcStream::Pipe(file));
        }
    }
    None
}

fn open_ipc_stream() -> Option<IpcStream> {
    #[cfg(unix)]
    {
        open_unix_stream()
    }
    #[cfg(windows)]
    {
        open_windows_pipe()
    }
    #[cfg(not(any(unix, windows)))]
    {
        None
    }
}

fn handshake(stream: &mut IpcStream, client_id: &str) -> bool {
    let _ = stream.set_read_timeout(Some(std::time::Duration::from_millis(300)));
    let payload = serde_json::json!({ "v": 1, "client_id": client_id }).to_string();
    if send_frame(stream, 0, &payload).is_err() {
        return false;
    }
    drain_response(stream);
    true
}

fn ensure_connection(guard: &mut Option<DiscordConn>, target_id: &str) {
    let needs_reconnect = guard
        .as_ref()
        .map(|c| c.client_id != target_id)
        .unwrap_or(true);

    if needs_reconnect {
        *guard = None;
        if let Some(mut stream) = open_ipc_stream() {
            if handshake(&mut stream, target_id) {
                *guard = Some(DiscordConn {
                    stream,
                    client_id: target_id.to_string(),
                });
            }
        }
    }
}

#[tauri::command]
#[allow(non_snake_case, unused_variables)]
pub async fn discord_set_activity(
    details: Option<String>,
    state: Option<String>,
    largeText: Option<String>,
    largeImage: Option<String>,
    smallText: Option<String>,
    smallImage: Option<String>,
    startTime: Option<i64>,
    inGame: Option<bool>,
    clientId: Option<String>,
    buttons: Option<Vec<DiscordButton>>,
) -> AppResult<bool> {
    let is_game = inGame.unwrap_or(false);

    let custom_env_id = std::env::var("LUXMC_DISCORD_CLIENT_ID")
        .ok()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty());

    let target_id = clientId
        .or(custom_env_id)
        .unwrap_or_else(|| MINECRAFT_CLIENT_ID.to_string());

    let mut guard = DISCORD_CONN.lock().unwrap_or_else(|e| e.into_inner());
    ensure_connection(&mut guard, &target_id);

    let conn = match guard.as_mut() {
        Some(c) => c,
        None => return Ok(false),
    };

    let _ = conn.stream.set_read_timeout(Some(std::time::Duration::from_millis(200)));

    let now = startTime.unwrap_or_else(|| chrono::Utc::now().timestamp());

    let det = details.unwrap_or_else(|| {
        if is_game {
            "Jogando Minecraft".to_string()
        } else {
            "No Menu Principal".to_string()
        }
    });

    let st = state.unwrap_or_else(|| {
        if is_game {
            "Luxmc Launcher".to_string()
        } else {
            format!("Luxmc v{}", LUXMC_VERSION)
        }
    });

    let img = match largeImage.as_deref() {
        Some("default") | Some("luxmc") | Some("") | None => LUXMC_ICON_URL.to_string(),
        Some(val) => val.to_string(),
    };

    let txt = largeText.unwrap_or_else(|| {
        if is_game {
            "Minecraft via Luxmc".to_string()
        } else {
            format!("Luxmc Launcher v{}", LUXMC_VERSION)
        }
    });

    let s_img = match smallImage.as_deref() {
        Some("") | None => MINECRAFT_GRASS_ASSET.to_string(),
        Some(val) => val.to_string(),
    };

    let s_txt = smallText.unwrap_or_else(|| {
        if is_game {
            "Minecraft".to_string()
        } else {
            format!("Luxmc v{}", LUXMC_VERSION)
        }
    });

    let mut activity_obj = serde_json::json!({
        "details": det,
        "state": st,
        "timestamps": { "start": now },
        "assets": {
            "large_image": img,
            "large_text": txt,
            "small_image": s_img,
            "small_text": s_txt
        }
    });

    if let Some(btns) = buttons {
        if !btns.is_empty() {
            activity_obj["buttons"] = serde_json::json!(btns);
        }
    } else {
        activity_obj["buttons"] = serde_json::json!([
            { "label": "Baixar Luxmc", "url": "https://luxmc-r92.pages.dev" }
        ]);
    }

    let payload = serde_json::json!({
        "cmd": "SET_ACTIVITY",
        "args": {
            "pid": std::process::id(),
            "activity": activity_obj
        },
        "nonce": format!("{}", chrono::Utc::now().timestamp_millis())
    })
    .to_string();

    if send_frame(&mut conn.stream, 1, &payload).is_ok() {
        drain_response(&mut conn.stream);
        Ok(true)
    } else {
        *guard = None;
        if let Some(mut stream) = open_ipc_stream() {
            if handshake(&mut stream, &target_id) {
                if send_frame(&mut stream, 1, &payload).is_ok() {
                    drain_response(&mut stream);
                    *guard = Some(DiscordConn {
                        stream,
                        client_id: target_id,
                    });
                    return Ok(true);
                }
            }
        }
        Ok(false)
    }
}

#[tauri::command]
pub async fn discord_clear_activity() -> AppResult<()> {
    let mut guard = DISCORD_CONN.lock().unwrap_or_else(|e| e.into_inner());
    if let Some(ref mut conn) = *guard {
        let payload = serde_json::json!({
            "cmd": "SET_ACTIVITY",
            "args": { "pid": std::process::id(), "activity": null },
            "nonce": "clear"
        })
        .to_string();
        let _ = send_frame(&mut conn.stream, 1, &payload);
    }
    *guard = None;
    Ok(())
}
