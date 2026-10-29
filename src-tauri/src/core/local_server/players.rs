use super::*;
use std::collections::BTreeMap;

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PlayerRecord {
    pub uuid: String,
    pub name: String,
    pub first_joined_at: String,
    pub last_joined_at: String,
    pub last_left_at: Option<String>,
    pub visits: u32,
    #[serde(default)]
    pub online: bool,
}

#[derive(Default, Serialize, Deserialize)]
pub struct PlayerHistory {
    #[serde(default)]
    pub records: BTreeMap<String, PlayerRecord>,
    #[serde(skip)]
    identities: BTreeMap<String, String>,
    #[serde(skip)]
    pub complete: bool,
}

fn expression(
    pattern: &'static str,
    slot: &'static std::sync::OnceLock<regex::Regex>,
) -> &'static regex::Regex {
    slot.get_or_init(|| regex::Regex::new(pattern).unwrap())
}

impl PlayerHistory {
    pub fn online_count(&self) -> u32 {
        self.records.values().filter(|record| record.online).count() as u32
    }
    pub fn offline(&mut self) {
        for record in self.records.values_mut() {
            record.online = false;
        }
    }

    pub fn observe(&mut self, text: &str) -> bool {
        static AUTH: std::sync::OnceLock<regex::Regex> = std::sync::OnceLock::new();
        static JOIN: std::sync::OnceLock<regex::Regex> = std::sync::OnceLock::new();
        static LEAVE: std::sync::OnceLock<regex::Regex> = std::sync::OnceLock::new();
        if let Some(value) = expression(r"^\[[^\]\r\n]+\] \[User Authenticator #[0-9]+/INFO\]: UUID of player ([A-Za-z0-9_]{1,16}) is ([0-9a-fA-F-]{32,36})$", &AUTH).captures(text) {
            if let Ok(uuid) = uuid::Uuid::parse_str(&value[2]) { if self.identities.len() >= 2048 { self.identities.pop_first(); self.complete = false; } self.identities.insert(value[1].to_ascii_lowercase(), uuid.to_string()); }
            return false;
        }
        if let Some(value) = expression(r"^\[[^\]\r\n]+\] \[Server thread/INFO\]: ([A-Za-z0-9_]{1,16})\[[^\r\n]*\] logged in with entity id ", &JOIN).captures(text) {
            let Some(uuid) = self.identities.get(&value[1].to_ascii_lowercase()).cloned() else { self.complete = false; return false; };
            if !self.records.contains_key(&uuid) && self.records.len() >= 1024 { self.complete = false; return false; }
            let now = chrono::Utc::now().to_rfc3339();
            let record = self.records.entry(uuid.clone()).or_insert_with(|| PlayerRecord { uuid, name: value[1].into(), first_joined_at: now.clone(), last_joined_at: now.clone(), last_left_at: None, visits: 0, online: false });
            if !record.online { record.visits = record.visits.saturating_add(1); record.last_joined_at = now; }
            record.name = value[1].into(); record.online = true;
            return true;
        }
        if let Some(value) = expression(
            r"^\[[^\]\r\n]+\] \[Server thread/INFO\]: ([A-Za-z0-9_]{1,16}) lost connection:",
            &LEAVE,
        )
        .captures(text)
        {
            if let Some(uuid) = self.identities.get(&value[1].to_ascii_lowercase()) {
                if let Some(record) = self.records.get_mut(uuid) {
                    if record.online {
                        record.online = false;
                        record.last_left_at = Some(chrono::Utc::now().to_rfc3339());
                        return true;
                    }
                }
            }
        }
        false
    }
}

impl Runtime {
    pub async fn flush_history(&self) {
        if !self.history_dirty.swap(false, Ordering::AcqRel) {
            return;
        }
        let Some(path) = &self.history_path else {
            return;
        };
        let bytes = {
            let history = self
                .history
                .lock()
                .unwrap_or_else(|error| error.into_inner());
            serde_json::to_vec_pretty(&*history)
        };
        let result = match bytes {
            Ok(bytes) => crate::core::mods::pack_download::atomic_write(path, &bytes).await,
            Err(error) => Err(error.into()),
        };
        if let Err(error) = result {
            self.history_dirty.store(true, Ordering::Release);
            self.log(
                "luxmc",
                &format!("Não foi possível gravar o histórico de jogadores: {error}"),
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_successful_authenticated_logins_enter_unique_player_history() {
        let mut history = PlayerHistory::default();
        history.observe("[12:00:00] [User Authenticator #1/INFO]: UUID of player Player is 12345678-1234-1234-1234-123456789abc");
        history.observe(
            "[12:00:01] [Server thread/INFO]: Disconnecting Player: You are not white-listed",
        );
        assert!(history.records.is_empty());
        history.observe("[12:00:02] [Server thread/INFO]: Player[/127.0.0.1:40000] logged in with entity id 5 at (1, 2, 3)");
        history.observe("[12:00:02] [Server thread/INFO]: Player[/127.0.0.1:40000] logged in with entity id 5 at (1, 2, 3)");
        assert_eq!(history.records.len(), 1);
        assert_eq!(history.online_count(), 1);
        assert_eq!(history.records.values().next().unwrap().visits, 1);
        history.observe("[12:00:03] [Server thread/INFO]: Player lost connection: Disconnected");
        assert_eq!(history.online_count(), 0);
        assert!(history
            .records
            .values()
            .next()
            .unwrap()
            .last_left_at
            .is_some());
        let bytes = serde_json::to_vec(&history).unwrap();
        let restored: PlayerHistory = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(restored.online_count(), 0);
    }

    #[test]
    fn chat_and_failed_auth_cannot_spoof_a_connected_player() {
        let mut history = PlayerHistory::default();
        history.observe("[12:00:00] [Server thread/INFO]: <Attacker> Player[/127.0.0.1:4] logged in with entity id 2");
        history.observe(
            "[12:00:00] [Server thread/INFO]: Player[/127.0.0.1:4] logged in with entity id 2",
        );
        assert!(history.records.is_empty());
        assert!(!history.complete);
    }
}
