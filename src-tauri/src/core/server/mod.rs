use std::io::{Read, Write};
use std::net::TcpStream;
use std::time::Duration;

use serde::{Deserialize, Serialize};

use crate::error::{AppError, AppResult};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ServerStatus {
    pub online: bool,
    pub version: String,
    pub players_max: u32,
    pub players_online: u32,
    pub motd: String,
    pub favicon: Option<String>,
}

fn write_varint(buf: &mut Vec<u8>, mut value: i32) {
    loop {
        let mut byte = (value & 0x7F) as u8;
        value >>= 7;
        if value != 0 {
            byte |= 0x80;
        }
        buf.push(byte);
        if value == 0 {
            break;
        }
    }
}

fn write_string(buf: &mut Vec<u8>, s: &str) {
    let bytes = s.as_bytes();
    write_varint(buf, bytes.len() as i32);
    buf.extend_from_slice(bytes);
}

fn read_varint(data: &[u8], pos: &mut usize) -> AppResult<i32> {
    let mut result: i32 = 0;
    let mut shift = 0;
    loop {
        if *pos >= data.len() {
            return Err(AppError::Internal("varint: unexpected end of data".into()));
        }
        let byte = data[*pos];
        *pos += 1;
        result |= ((byte & 0x7F) as i32) << shift;
        if byte & 0x80 == 0 {
            break;
        }
        shift += 7;
        if shift >= 32 {
            return Err(AppError::Internal("varint: too many bytes".into()));
        }
    }
    Ok(result)
}

fn read_packet(stream: &mut TcpStream) -> AppResult<(i32, Vec<u8>)> {
    let mut length = 0i32;
    let mut shift = 0;

    loop {
        let mut byte = [0u8; 1];
        stream.read_exact(&mut byte)?;
        length |= ((byte[0] & 0x7F) as i32) << shift;
        if byte[0] & 0x80 == 0 {
            break;
        }
        shift += 7;
        if shift >= 32 {
            return Err(AppError::Internal("packet length varint too long".into()));
        }
    }

    if length <= 0 || length > 1024 * 1024 {
        return Err(AppError::Internal(format!(
            "invalid packet length: {}",
            length
        )));
    }

    let mut data = vec![0u8; length as usize];
    stream.read_exact(&mut data)?;

    let mut pos = 0;
    let packet_id = read_varint(&data, &mut pos)?;

    Ok((packet_id, data[pos..].to_vec()))
}

use std::net::ToSocketAddrs;

pub fn ping(host: &str, port: u16) -> AppResult<ServerStatus> {
    let target = (host, port);
    let addr = match target.to_socket_addrs() {
        Ok(mut iter) => iter.next(),
        Err(_) => None,
    };

    if let Some(sock_addr) = addr {
        if let Ok(mut stream) = TcpStream::connect_timeout(&sock_addr, Duration::from_secs(3)) {
            stream.set_read_timeout(Some(Duration::from_secs(3))).ok();
            stream.set_write_timeout(Some(Duration::from_secs(3))).ok();

            let mut handshake = Vec::new();
            write_varint(&mut handshake, 0x00);
            write_varint(&mut handshake, -1);
            write_string(&mut handshake, host);
            let mut port_buf = [0u8; 2];
            port_buf.copy_from_slice(&port.to_be_bytes());
            handshake.extend_from_slice(&port_buf);
            write_varint(&mut handshake, 1);

            let mut packet = Vec::new();
            write_varint(&mut packet, handshake.len() as i32);
            packet.extend_from_slice(&handshake);
            if stream.write_all(&packet).is_ok() {
                let mut status_req = Vec::new();
                write_varint(&mut status_req, 0x00);
                let mut status_packet = Vec::new();
                write_varint(&mut status_packet, status_req.len() as i32);
                status_packet.extend_from_slice(&status_req);
                if stream.write_all(&status_packet).is_ok() {
                    if let Ok((_, payload)) = read_packet(&mut stream) {
                        let mut pos = 0;
                        if let Ok(_json_len) = read_varint(&payload, &mut pos) {
                            if let Ok(json_str) = std::str::from_utf8(&payload[pos..]) {
                                if let Ok(v) = serde_json::from_str::<serde_json::Value>(json_str) {
                                    let version = v
                                        .get("version")
                                        .and_then(|v| v.get("name"))
                                        .and_then(|v| v.as_str())
                                        .unwrap_or("1.21.4")
                                        .to_string();

                                    let players = v.get("players").cloned().unwrap_or_default();
                                    let players_max = players.get("max").and_then(|v| v.as_u64()).unwrap_or(0) as u32;
                                    let players_online = players.get("online").and_then(|v| v.as_u64()).unwrap_or(0) as u32;

                                    let motd = v
                                        .get("description")
                                        .map(|d| {
                                            if let Some(text) = d.get("text").and_then(|t| t.as_str()) {
                                                text.to_string()
                                            } else if let Some(s) = d.as_str() {
                                                s.to_string()
                                            } else {
                                                d.to_string()
                                            }
                                        })
                                        .unwrap_or_default();

                                    let favicon = v
                                        .get("favicon")
                                        .and_then(|f| f.as_str())
                                        .map(|s| s.to_string());

                                    return Ok(ServerStatus {
                                        online: true,
                                        version,
                                        players_max,
                                        players_online,
                                        motd,
                                        favicon,
                                    });
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    // HTTP Status Fallback (mcsrvstat.us)
    static HTTP_CLIENT: std::sync::OnceLock<reqwest::blocking::Client> = std::sync::OnceLock::new();
    let client = HTTP_CLIENT.get_or_init(|| {
        reqwest::blocking::Client::builder()
            .timeout(Duration::from_secs(4))
            .build()
            .unwrap_or_default()
    });

    if let Ok(resp) = client.get(format!("https://api.mcsrvstat.us/3/{}", host)).send() {
        if resp.status().is_success() {
            if let Ok(v) = resp.json::<serde_json::Value>() {
                let online = v.get("online").and_then(|o| o.as_bool()).unwrap_or(false);
                let version = v.get("version").and_then(|s| s.as_str()).unwrap_or("Online").to_string();
                let players_online = v.get("players").and_then(|p| p.get("online")).and_then(|o| o.as_u64()).unwrap_or(0) as u32;
                let players_max = v.get("players").and_then(|p| p.get("max")).and_then(|m| m.as_u64()).unwrap_or(1000) as u32;
                let motd = v.get("motd").and_then(|m| m.get("clean")).and_then(|c| c.as_array())
                    .map(|arr| arr.iter().filter_map(|x| x.as_str()).collect::<Vec<_>>().join(" "))
                    .unwrap_or_default();

                return Ok(ServerStatus {
                    online,
                    version,
                    players_max,
                    players_online,
                    motd,
                    favicon: None,
                });
            }
        }
    }

    Ok(ServerStatus {
        online: false,
        version: "Offline".to_string(),
        players_max: 0,
        players_online: 0,
        motd: String::new(),
        favicon: None,
    })
}
