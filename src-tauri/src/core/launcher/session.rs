use std::sync::{Arc, Mutex};
use serde::{Deserialize, Serialize};
use tauri::Emitter;

static CURRENT: Mutex<Option<Arc<Session>>> = Mutex::new(None);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Phase { Preparing, StartingJava, WaitingForWindow, Opened, Failed, Cancelled, Exited }

impl Phase {
    pub fn terminal(self) -> bool { matches!(self, Self::Failed | Self::Cancelled | Self::Exited) }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Snapshot {
    pub session_id: String,
    pub profile_id: String,
    pub version_id: String,
    pub phase: Phase,
    pub stage: String,
    pub pid: u32,
    pub process_started_at: Option<u64>,
    pub started_at: String,
    pub updated_at: String,
    pub elapsed_seconds: u64,
    pub error: Option<String>,
    pub failed_stage: Option<String>,
    pub exit_code: Option<i32>,
}

pub struct Session {
    value: Mutex<Snapshot>,
    started: std::time::Instant,
    cancel: tokio::sync::watch::Sender<bool>,
    app: Option<tauri::AppHandle>,
    journal: Option<std::path::PathBuf>,
}

pub fn current() -> Option<Arc<Session>> { CURRENT.lock().ok()?.clone() }

pub fn previous() -> Option<Snapshot> {
    let path = directories::ProjectDirs::from("io", "github", "Luxmc")?.data_dir().join("logs/launch-latest.json");
    if std::fs::metadata(&path).ok()?.len() > 64 * 1024 { return None; }
    serde_json::from_slice(&std::fs::read(path).ok()?).ok()
}

fn save(path: &std::path::Path, value: &Snapshot) -> std::io::Result<()> {
    let temporary = path.with_extension("json.tmp");
    let mut file = std::fs::OpenOptions::new().create(true).truncate(true).write(true).open(&temporary)?;
    serde_json::to_writer(&mut file, value)?;
    file.sync_all()?;
    drop(file);
    std::fs::rename(temporary, path)
}

impl Session {
    pub fn begin(profile_id: String, version_id: String, app: Option<tauri::AppHandle>, id: Option<uuid::Uuid>) -> Arc<Self> {
        let now = chrono::Utc::now().to_rfc3339();
        let (cancel, _) = tokio::sync::watch::channel(false);
        let session = Arc::new(Self {
            value: Mutex::new(Snapshot { session_id: id.unwrap_or_else(uuid::Uuid::new_v4).to_string(), profile_id, version_id,
                phase: Phase::Preparing, stage: "preparing".into(), pid: 0, process_started_at: None,
                started_at: now.clone(), updated_at: now, elapsed_seconds: 0, error: None, failed_stage: None, exit_code: None }),
            started: std::time::Instant::now(), cancel, app,
            journal: if cfg!(test) { None } else { directories::ProjectDirs::from("io", "github", "Luxmc").map(|dirs| dirs.data_dir().join("logs/launch-latest.json")) },
        });
        if let Ok(mut active) = CURRENT.lock() { *active = Some(session.clone()); }
        session.update(Phase::Preparing, "preparing", None, None);
        session
    }

    pub fn snapshot(&self) -> Snapshot {
        let mut value = self.value.lock().unwrap_or_else(|error| error.into_inner()).clone();
        if !value.phase.terminal() { value.elapsed_seconds = self.started.elapsed().as_secs(); }
        value
    }

    pub fn id(&self) -> String { self.snapshot().session_id }

    pub fn update(&self, phase: Phase, stage: &str, error: Option<String>, exit_code: Option<i32>) {
        let mut value = self.value.lock().unwrap_or_else(|error| error.into_inner());
        if value.phase.terminal() { return; }
        if phase == Phase::Failed { value.failed_stage = Some(value.stage.clone()); }
        value.phase = phase;
        value.stage = stage.into();
        value.updated_at = chrono::Utc::now().to_rfc3339();
        value.elapsed_seconds = self.started.elapsed().as_secs();
        value.error = error.map(|text| crate::core::process::sanitize_diagnostic(&text));
        value.exit_code = exit_code;
        if let Some(path) = &self.journal {
            if path.parent().is_some_and(|directory| std::fs::create_dir_all(directory).is_ok()) {
                let _ = save(path, &value);
                let _ = save(&path.with_file_name(format!("launch-{}.json", value.session_id)), &value);
            }
        }
        if let Some(app) = &self.app { let _ = app.emit("launch-session", &*value); }
    }

    pub fn process_started(&self, pid: u32) {
        if let Ok(mut value) = self.value.lock() {
            value.pid = pid;
            value.process_started_at = crate::core::process::process_started_at(pid);
        }
        self.update(Phase::WaitingForWindow, "waitingForWindow", None, None);
    }

    pub fn owns_process(&self, pid: u32) -> bool {
        let value = self.snapshot();
        pid > 0 && value.pid == pid && value.process_started_at.is_some()
            && value.process_started_at == crate::core::process::process_started_at(pid)
    }

    pub fn request_cancel(&self, session_id: &str) -> bool {
        let value = self.snapshot();
        if value.session_id != session_id || value.phase.terminal() { return false; }
        self.cancel.send_replace(true);
        true
    }

    pub async fn cancelled(&self) {
        let mut receiver = self.cancel.subscribe();
        while !*receiver.borrow_and_update() { if receiver.changed().await.is_err() { break; } }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn cancellation_is_scoped_and_not_lost_before_subscribing() {
        let session = Session::begin("fixture".into(), "26.3".into(), None, None);
        assert!(!session.request_cancel("older-session"));
        assert!(session.request_cancel(&session.id()));
        tokio::time::timeout(std::time::Duration::from_millis(50), session.cancelled()).await.unwrap();
        session.update(Phase::Cancelled, "cancelled", None, None);
        assert!(!session.request_cancel(&session.id()));
        session.update(Phase::Opened, "running", None, None);
        assert_eq!(session.snapshot().phase, Phase::Cancelled);
        assert!(!session.owns_process(0));
    }

    #[test]
    fn failures_preserve_the_stage_and_an_atomic_sanitized_record() {
        let session = Session::begin("fixture".into(), "26.3".into(), None, None);
        session.update(Phase::StartingJava, "spawning", None, None);
        session.update(Phase::Failed, "failed", Some("access_token=secret".into()), Some(1));
        let snapshot = session.snapshot();
        assert_eq!(snapshot.failed_stage.as_deref(), Some("spawning"));
        assert_eq!(snapshot.exit_code, Some(1));
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join("latest.json");
        save(&path, &snapshot).unwrap();
        save(&path, &snapshot).unwrap();
        let content = std::fs::read_to_string(path).unwrap();
        assert!(!content.contains("secret"));
        assert_eq!(serde_json::from_str::<Snapshot>(&content).unwrap().session_id, snapshot.session_id);
    }
}
