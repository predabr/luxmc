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
		return Err(AppError::Internal(format!("invalid packet length: {}", length)));
	}

	let mut data = vec![0u8; length as usize];
	stream.read_exact(&mut data)?;

	let mut pos = 0;
	let packet_id = read_varint(&data, &mut pos)?;

	Ok((packet_id, data[pos..].to_vec()))
}

pub fn ping(host: &str, port: u16) -> AppResult<ServerStatus> {
	let address = format!("{}:{}", host, port);
	let mut stream = TcpStream::connect_timeout(
		&address.parse().map_err(|_| AppError::Internal(format!("invalid address: {}", address)))?,
		Duration::from_secs(5),
	)?;
	stream.set_read_timeout(Some(Duration::from_secs(5)))?;
	stream.set_write_timeout(Some(Duration::from_secs(5)))?;

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
	stream.write_all(&packet)?;

	let mut status_req = Vec::new();
	write_varint(&mut status_req, 0x00);
	let mut status_packet = Vec::new();
	write_varint(&mut status_packet, status_req.len() as i32);
	status_packet.extend_from_slice(&status_req);
	stream.write_all(&status_packet)?;

	let (_, payload) = read_packet(&mut stream)?;

	let mut pos = 0;
	let _json_len = read_varint(&payload, &mut pos)?;
	let json_str = std::str::from_utf8(&payload[pos..])
		.map_err(|e| AppError::Internal(format!("invalid utf8: {}", e)))?;

	let v: serde_json::Value = serde_json::from_str(json_str)?;

	let version = v.get("version")
		.and_then(|v| v.get("name"))
		.and_then(|v| v.as_str())
		.unwrap_or("unknown")
		.to_string();

	let players = v.get("players").cloned().unwrap_or_default();
	let players_max = players.get("max").and_then(|v| v.as_u64()).unwrap_or(0) as u32;
	let players_online = players.get("online").and_then(|v| v.as_u64()).unwrap_or(0) as u32;

	let motd = v.get("description")
		.and_then(|d| d.get("text"))
		.and_then(|t| t.as_str())
		.unwrap_or("")
		.to_string();

	let favicon = v.get("favicon").and_then(|f| f.as_str()).map(|s| s.to_string());

	Ok(ServerStatus {
		online: true,
		version,
		players_max,
		players_online,
		motd,
		favicon,
	})
}
