use crate::error::AppResult;
use serde::{Deserialize, Serialize};
use std::sync::Mutex;

#[cfg(unix)]
use std::io::{Read, Write};
#[cfg(unix)]
use std::os::unix::net::UnixStream;

static DISCORD_STREAM: Mutex<Option<UnixStream>> = Mutex::new(None);

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[allow(dead_code)]
pub struct DiscordActivityArgs {
    pub details: Option<String>,
    pub state: Option<String>,
    pub large_text: Option<String>,
    pub start_time: Option<i64>,
}

#[cfg(unix)]
fn get_socket_path() -> Option<std::path::PathBuf> {
    let runtime_dir = std::env::var("XDG_RUNTIME_DIR").unwrap_or_else(|_| "/tmp".to_string());
    let home_dir = std::env::var("HOME").unwrap_or_default();

    let candidates = vec![
        std::path::PathBuf::from(&runtime_dir),
        std::path::PathBuf::from(&runtime_dir).join("app/com.discordapp.Discord"),
        std::path::PathBuf::from(&runtime_dir).join("app/com.discordapp.DiscordCanary"),
        std::path::PathBuf::from(&runtime_dir).join("app/de.vencord.Vesktop"),
        std::path::PathBuf::from("/tmp"),
        std::path::PathBuf::from("/tmp/app/com.discordapp.Discord"),
        std::path::PathBuf::from(&home_dir).join(".var/app/com.discordapp.Discord/config"),
        std::path::PathBuf::from(&home_dir).join(".var/app/de.vencord.Vesktop/config"),
        std::path::PathBuf::from(&home_dir).join(".config/discord"),
    ];

    let prefixes = vec!["discord-ipc-", "ipc-"];

    for base in candidates {
        for prefix in &prefixes {
            for i in 0..10 {
                let p = base.join(format!("{}{}", prefix, i));
                if p.exists() {
                    return Some(p);
                }
            }
        }
    }

    None
}

#[cfg(unix)]
fn send_frame(stream: &mut UnixStream, opcode: u32, payload: &str) -> std::io::Result<()> {
    let len = payload.len() as u32;
    let mut header = Vec::with_capacity(8);
    header.extend_from_slice(&opcode.to_le_bytes());
    header.extend_from_slice(&len.to_le_bytes());
    stream.write_all(&header)?;
    stream.write_all(payload.as_bytes())?;
    stream.flush()?;
    Ok(())
}

#[tauri::command]
#[allow(non_snake_case)]
pub async fn discord_set_activity(
    details: Option<String>,
    state: Option<String>,
    largeText: Option<String>,
    largeImage: Option<String>,
) -> AppResult<bool> {
    #[cfg(unix)]
    {
        let mut guard = DISCORD_STREAM.lock().unwrap();

        let needs_connect = guard.is_none();
        if needs_connect {
            if let Some(path) = get_socket_path() {
                if let Ok(mut stream) = UnixStream::connect(path) {
                    stream
                        .set_read_timeout(Some(std::time::Duration::from_millis(200)))
                        .ok();
                    let handshake = serde_json::json!({
                        "v": 1,
                        "client_id": "1219293400582553650"
                    })
                    .to_string();

                    if send_frame(&mut stream, 0, &handshake).is_ok() {
                        let mut header = [0u8; 8];
                        if stream.read_exact(&mut header).is_ok() {
                            let resp_len =
                                u32::from_le_bytes([header[4], header[5], header[6], header[7]])
                                    as usize;
                            let mut buf = vec![0u8; resp_len];
                            let _ = stream.read_exact(&mut buf);
                            *guard = Some(stream);
                        }
                    }
                }
            }
        }

        if let Some(ref mut stream) = *guard {
            stream
                .set_read_timeout(Some(std::time::Duration::from_millis(200)))
                .ok();
            let now = chrono::Utc::now().timestamp();
            let img = largeImage.unwrap_or_else(|| {
                "https://raw.githubusercontent.com/luxmc/luxmc/main/static/icon.png".to_string()
            });
            let act = serde_json::json!({
                "cmd": "SET_ACTIVITY",
                "args": {
                    "pid": std::process::id(),
                    "activity": {
                        "state": state.unwrap_or_else(|| "No Menu Principal".to_string()),
                        "details": details.unwrap_or_else(|| "Luxmc Launcher v0.6.0-BETA".to_string()),
                        "timestamps": {
                            "start": now
                        },
                        "assets": {
                            "large_image": img,
                            "large_text": largeText.unwrap_or_else(|| "Luxmc Launcher (Linux)".to_string())
                        }
                    }
                },
                "nonce": "luxmc-rpc-1"
            })
            .to_string();

            if send_frame(stream, 1, &act).is_ok() {
                let mut header = [0u8; 8];
                let _ = stream.read_exact(&mut header);
                return Ok(true);
            } else {
                *guard = None;
            }
        }
    }

    Ok(false)
}

#[tauri::command]
pub async fn discord_clear_activity() -> AppResult<()> {
    #[cfg(unix)]
    {
        let mut guard = DISCORD_STREAM.lock().unwrap();
        if let Some(ref mut stream) = *guard {
            let act = serde_json::json!({
                "cmd": "SET_ACTIVITY",
                "args": {
                    "pid": std::process::id(),
                    "activity": null
                },
                "nonce": "1"
            })
            .to_string();
            let _ = send_frame(stream, 1, &act);
        }
        *guard = None;
    }
    Ok(())
}
