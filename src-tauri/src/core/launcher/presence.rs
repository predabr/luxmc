use std::collections::HashSet;

#[derive(Default)]
pub(super) struct Presence {
    status: &'static str,
    detail: String,
    players: HashSet<String>,
    observed_count: Option<u32>,
    pub(super) target: Option<(String, u16)>,
}

impl Presence {
    pub(super) fn event(&self) -> serde_json::Value {
        serde_json::json!({"status":if self.status.is_empty() {"menu"} else {self.status},"detail":self.detail,"playerCount":self.observed_count.or_else(|| (self.status == "singleplayer").then(|| self.players.len().max(1) as u32))})
    }

    pub(super) fn observe_count(&mut self, target: &(String, u16), count: u32) -> bool {
        if self.target.as_ref() != Some(target) || self.observed_count == Some(count) { return false; }
        self.observed_count = Some(count);
        true
    }

    pub(super) fn line(&mut self, line: &str) -> bool {
        let before = self.event();
        if let Some((_, rest)) = line.split_once("Preparing level \"") {
            if let Some((world, _)) = rest.split_once('"') { self.status = "singleplayer"; self.detail = world.into(); }
        } else if line.contains("Starting integrated minecraft server") {
            self.status = "singleplayer"; self.detail.clear(); self.players.clear(); self.observed_count = None; self.target = None;
        } else if let Some((_, target)) = line.split_once("Connecting to ") {
            let target = target.trim();
            let (host, port) = target.rsplit_once(',').map(|(host, port)| (host.trim(), port.trim().parse::<u16>().unwrap_or(25565)))
                .or_else(|| target.rsplit_once(':').and_then(|(host, port)| port.parse().ok().map(|port| (host.trim_matches(['[', ']']), port))))
                .unwrap_or((target, 25565));
            self.status = "multiplayer"; self.detail = host.into(); self.players.clear(); self.observed_count = None; self.target = Some((host.into(), port));
        } else if line.contains("Stopping server") || line.contains("Saving and disconnecting") {
            self.status = "menu"; self.detail.clear(); self.players.clear(); self.observed_count = None; self.target = None;
        } else if self.status == "singleplayer" && line.contains("Server thread") {
            let message = line.rsplit_once(": ").map_or(line, |(_, message)| message);
            let change = message.strip_suffix(" joined the game").map(|name| (name, true))
                .or_else(|| message.strip_suffix(" left the game").map(|name| (name, false)));
            if let Some((name, joined)) = change {
                if (3..=16).contains(&name.len()) && name.bytes().all(|byte| byte.is_ascii_alphanumeric() || byte == b'_') {
                    if joined { self.players.insert(name.into()); } else { self.players.remove(name); }
                    self.observed_count = None;
                }
            }
        }
        self.event() != before
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn counts_game_players_and_resets_between_worlds() {
        let mut state = Presence::default();
        state.line("[Server thread/INFO]: Starting integrated minecraft server version 1.21.1");
        state.line("[Server thread/INFO]: Preparing level \"Our world\"");
        for name in ["Owner", "Friend", "Third", "Friend"] { state.line(&format!("[Server thread/INFO]: {name} joined the game")); }
        assert_eq!(state.event()["playerCount"], 3);
        assert_eq!(state.event()["detail"], "Our world");
        state.line("[Server thread/INFO]: Friend left the game");
        assert_eq!(state.event()["playerCount"], 2);
        state.line("[Render thread/INFO]: FakePlayer joined the game");
        assert_eq!(state.event()["playerCount"], 2);
        state.line("[Server thread/INFO]: Stopping server");
        assert!(state.event()["playerCount"].is_null());
        state.line("Starting integrated minecraft server");
        assert_eq!(state.event()["playerCount"], 1);
    }
    #[test]
    fn server_count_is_applied_only_to_the_current_connection() {
        let mut state = Presence::default();
        state.line("[Render thread/INFO]: Connecting to 127.0.0.1, 41234");
        assert!(state.observe_count(&("127.0.0.1".into(), 41234), 3));
        assert_eq!(state.event()["playerCount"], 3);
        state.line("Saving and disconnecting");
        assert!(!state.observe_count(&("127.0.0.1".into(), 41234), 3));
        assert_eq!(state.event()["status"], "menu");
    }
}
