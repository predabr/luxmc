use std::{
    io::{Read, Seek, SeekFrom},
    path::{Path, PathBuf},
    sync::{LazyLock, Mutex},
};

#[derive(Default)]
struct Reader {
    path: PathBuf,
    pid: u32,
    offset: u64,
    created: Option<std::time::SystemTime>,
    pending: Vec<u8>,
    port: Option<u16>,
}

fn port_event(line: &str) -> Option<Option<u16>> {
    if line.contains("Stopping server") { return Some(None); }
    for marker in [
        "Started serving on ",
        "Started on ",
        "Started a LAN server on port ",
        "Published LAN server on port ",
        "Local game hosted on port ",
        "[LUXMC_LAN] Opened world on port ",
    ] {
        if let Some((_, value)) = line.split_once(marker) {
            let digits = value.trim_start_matches([' ', '[']).split(|value: char| !value.is_ascii_digit()).next()?;
            if let Some(port) = digits.parse::<u16>().ok().filter(|port| *port != 0) { return Some(Some(port)); }
        }
    }
    None
}

#[cfg(test)]
pub(super) fn logged_port(text: &str) -> Option<u16> {
    text.lines().rev().find_map(port_event).flatten()
}

impl Reader {
    fn read(&mut self, directory: &Path, pid: u32) -> Option<u16> {
        let path = directory.join("logs/latest.log");
        let mut file = std::fs::File::open(&path).ok()?;
        let metadata = file.metadata().ok()?;
        let length = metadata.len();
        let created = metadata.created().ok();
        if self.path != path || self.pid != pid || self.created != created || length < self.offset {
            *self = Self { path, pid, created, ..Self::default() };
        }
        file.seek(SeekFrom::Start(self.offset)).ok()?;
        let mut file = file.take(length.saturating_sub(self.offset).min(4 * 1024 * 1024));
        let mut chunk = [0; 8192];
        loop {
            let count = file.read(&mut chunk).ok()?;
            if count == 0 { break; }
            self.offset += count as u64;
            for byte in &chunk[..count] {
                if *byte == b'\n' {
                    if let Some(event) = port_event(&String::from_utf8_lossy(&self.pending)) { self.port = event; }
                    self.pending.clear();
                } else {
                    if self.pending.len() == 4096 { self.pending.drain(..2048); }
                    self.pending.push(*byte);
                }
            }
        }
        if let Some(event) = port_event(&String::from_utf8_lossy(&self.pending)) { self.port = event; }
        self.port
    }
}

fn reported_port(directory: &Path, pid: u32) -> Option<u16> {
    let file = std::fs::File::open(directory.join(".luxmc/lan-published.properties")).ok()?;
    let mut text = String::new();
    file.take(513).read_to_string(&mut text).ok()?;
    if text.len() > 512 { return None; }
    let properties: std::collections::BTreeMap<_, _> = text.lines().filter_map(|line| line.split_once('=')).collect();
    if properties.get("pid")?.parse::<u32>().ok()? != pid || pid == 0 { return None; }
    properties.get("port")?.parse::<u16>().ok().filter(|port| *port != 0)
}

pub(super) fn active_ports() -> Vec<u16> {
    static READER: LazyLock<Mutex<Reader>> = LazyLock::new(|| Mutex::new(Reader::default()));
    let pid = crate::core::launcher::get_active_game_pid();
    if pid == 0 { return Vec::new(); }
    let Some(directory) = crate::core::launcher::get_active_game_dir() else { return Vec::new(); };
    let reported = reported_port(&directory, pid);
    let logged = READER.lock().ok().and_then(|mut reader| reader.read(&directory, pid));
    candidate_ports(reported, logged)
}

fn candidate_ports(reported: Option<u16>, logged: Option<u16>) -> Vec<u16> {
    let mut ports = Vec::with_capacity(2);
    for port in [reported, logged].into_iter().flatten().filter(|port| *port != 0) {
        if !ports.contains(&port) { ports.push(port); }
    }
    ports
}

pub(super) fn active_port() -> Option<u16> {
    active_ports().into_iter().next()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_previous_report_does_not_discard_a_new_port_from_the_log() {
        assert_eq!(candidate_ports(Some(57813), Some(49219)), vec![57813, 49219]);
        assert_eq!(candidate_ports(Some(49219), Some(49219)), vec![49219]);
        assert_eq!(candidate_ports(Some(0), None), Vec::<u16>::new());
    }

    #[test]
    fn detects_forge_brackets_vanilla_and_closed_worlds() {
        assert_eq!(logged_port("[Render thread/INFO] Started serving on 57813\n[CHAT] Local game hosted on port [57813]"), Some(57813));
        assert_eq!(logged_port("Published LAN server on port 49219"), Some(49219));
        assert_eq!(logged_port("[Render thread/INFO] Started on 47123"), Some(47123));
        assert_eq!(logged_port("Started serving on 12345\nStopping server"), None);
        assert_eq!(logged_port("Started serving on 12345\nStopping server\nStarted serving on 25565"), Some(25565));
        assert_eq!(logged_port("Started serving on 999999"), None);
    }

    #[test]
    fn retains_a_modpack_port_beyond_the_old_log_tail_and_updates_it() {
        use std::io::Write;
        let temporary = tempfile::tempdir().unwrap();
        let directory = temporary.path();
        std::fs::create_dir(directory.join("logs")).unwrap();
        let path = directory.join("logs/latest.log");
        std::fs::write(&path, format!("Started serving on 57813\n{}", "[Server/INFO] Mod update\n".repeat(6000))).unwrap();
        let mut reader = Reader::default();
        assert_eq!(reader.read(directory, 1), Some(57813));
        assert_eq!(reader.read(directory, 1), Some(57813));
        let mut file = std::fs::OpenOptions::new().append(true).open(&path).unwrap();
        file.write_all(b"Stopping server\nPublished LAN server on port 49219\n").unwrap();
        assert_eq!(reader.read(directory, 1), Some(49219));
        file.write_all(b"Stopping server\n").unwrap();
        assert_eq!(reader.read(directory, 1), None);
        std::fs::write(&path, "Started serving on 25565\n").unwrap();
        assert_eq!(reader.read(directory, 2), Some(25565));
    }

    #[test]
    fn accepts_only_a_valid_port_reported_by_the_current_game() {
        let temporary = tempfile::tempdir().unwrap();
        let directory = temporary.path();
        std::fs::create_dir(directory.join(".luxmc")).unwrap();
        let path = directory.join(".luxmc/lan-published.properties");
        std::fs::write(&path, "pid=123\nport=57813\n").unwrap();
        assert_eq!(reported_port(directory, 123), Some(57813));
        assert_eq!(reported_port(directory, 124), None);
        std::fs::write(&path, "pid=123\nport=999999\n").unwrap();
        assert_eq!(reported_port(directory, 123), None);
    }
}
