use crate::error::AppResult;
use serde::{Deserialize, Serialize};
use std::sync::Mutex;

#[cfg(unix)]
use std::io::{Read, Write};
#[cfg(unix)]
use std::os::unix::net::UnixStream;

static DISCORD_STREAM: Mutex<Option<UnixStream>> = Mutex::new(None);
static CURRENT_CLIENT_ID: Mutex<Option<String>> = Mutex::new(None);

const MINECRAFT_CLIENT_ID: &str = "450485984333660181";

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
    smallText: Option<String>,
    smallImage: Option<String>,
    startTime: Option<i64>,
    inGame: Option<bool>,
    clientId: Option<String>,
) -> AppResult<bool> {
    #[cfg(unix)]
    {
        let is_game = inGame.unwrap_or(false);
        let custom_id = std::env::var("LUXMC_DISCORD_CLIENT_ID")
            .ok()
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty());

        let target_client_id = clientId
            .or(custom_id)
            .unwrap_or_else(|| MINECRAFT_CLIENT_ID.to_string());

        let mut stream_guard = DISCORD_STREAM.lock().unwrap();
        let mut client_id_guard = CURRENT_CLIENT_ID.lock().unwrap();

        let needs_new_connection = stream_guard.is_none()
            || client_id_guard.as_deref() != Some(&target_client_id);

        if needs_new_connection {
            *stream_guard = None;
            *client_id_guard = None;

            if let Some(path) = get_socket_path() {
                if let Ok(mut stream) = UnixStream::connect(path) {
                    stream
                        .set_read_timeout(Some(std::time::Duration::from_millis(200)))
                        .ok();
                    let handshake = serde_json::json!({
                        "v": 1,
                        "client_id": target_client_id
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
                            *stream_guard = Some(stream);
                            *client_id_guard = Some(target_client_id);
                        }
                    }
                }
            }
        }

        if let Some(ref mut stream) = *stream_guard {
            stream
                .set_read_timeout(Some(std::time::Duration::from_millis(200)))
                .ok();
            let now = startTime.unwrap_or_else(|| chrono::Utc::now().timestamp());

            let (def_img, def_text, def_details, def_state) = if is_game {
                (
                    "default".to_string(),
                    "Minecraft (Luxmc)".to_string(),
                    "Jogando Minecraft".to_string(),
                    "Luxmc Launcher".to_string(),
                )
            } else {
                (
                    "default".to_string(),
                    "Luxmc Launcher".to_string(),
                    "Luxmc Launcher v1.2.0-ALPHA".to_string(),
                    "No Menu Principal".to_string(),
                )
            };

            let img = largeImage.unwrap_or(def_img);
            let txt = largeText.unwrap_or(def_text);
            let det = details.unwrap_or(def_details);
            let st = state.unwrap_or(def_state);

            let mut assets = serde_json::json!({
                "large_image": img,
                "large_text": txt,
            });

            if let Some(s_img) = smallImage {
                assets["small_image"] = serde_json::Value::String(s_img);
                if let Some(s_txt) = smallText {
                    assets["small_text"] = serde_json::Value::String(s_txt);
                }
            }

            let act = serde_json::json!({
                "cmd": "SET_ACTIVITY",
                "args": {
                    "pid": std::process::id(),
                    "activity": {
                        "name": "Luxmc Launcher",
                        "state": st,
                        "details": det,
                        "timestamps": {
                            "start": now
                        },
                        "assets": assets
                    }
                },
                "nonce": format!("luxmc-rpc-{}", chrono::Utc::now().timestamp_millis())
            })
            .to_string();

            if send_frame(stream, 1, &act).is_ok() {
                let mut header = [0u8; 8];
                let _ = stream.read_exact(&mut header);
                return Ok(true);
            } else {
                *stream_guard = None;
                *client_id_guard = None;
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
