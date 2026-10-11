use std::sync::{Arc, atomic::{AtomicBool, Ordering}};
use std::time::{Duration, Instant};
use tauri::Emitter;
use tokio::sync::{Mutex, watch};
use crate::error::{AppError, AppResult};
use super::social::{SocialRequest, social_request_core};

const HEARTBEAT: Duration = Duration::from_secs(20);
static CURRENT: Mutex<Option<Worker>> = Mutex::const_new(None);

struct Session {
    account_id: String,
    session_id: String,
    active: AtomicBool,
    updates: watch::Sender<Option<SocialRequest>>,
    request: Mutex<Option<Instant>>,
}

struct Worker {
    session: Arc<Session>,
    task: tokio::task::AbortHandle,
}

impl Worker {
    fn stop(self) -> Arc<Session> {
        self.session.active.store(false, Ordering::Release);
        self.task.abort();
        self.session
    }
}

async fn heartbeat<F, Fut, N>(session: Arc<Session>, mut receiver: watch::Receiver<Option<SocialRequest>>, interval: Duration, send: F, notify: N)
where F: Fn(String, SocialRequest) -> Fut, Fut: std::future::Future<Output = AppResult<serde_json::Value>>, N: Fn(AppResult<serde_json::Value>) {
    loop {
        tokio::select! {
            changed = receiver.changed() => if changed.is_err() { return; },
            _ = tokio::time::sleep(interval) => {}
        }
        let mut previous = session.request.lock().await;
        if !session.active.load(Ordering::Acquire) { return; }
        if previous.is_some_and(|at| at.elapsed() < interval) { continue; }
        let Some(request) = receiver.borrow_and_update().clone() else { continue; };
        let result = send(session.account_id.clone(), request).await;
        if result.is_ok() { *previous = Some(Instant::now()); }
        if session.active.load(Ordering::Acquire) { notify(result); }
    }
}

#[tauri::command]
pub async fn social_presence_start(app: tauri::AppHandle, account_id: String, session_id: String) -> AppResult<bool> {
    if account_id.is_empty() || account_id.len() > 128 || uuid::Uuid::parse_str(&session_id).is_err() {
        return Err(AppError::InvalidInput("Invalid presence session".into()));
    }
    let mut current = CURRENT.lock().await;
    if current.as_ref().is_some_and(|worker| worker.session.account_id == account_id && worker.session.session_id == session_id) { return Ok(true); }
    if let Some(previous) = current.take() { previous.stop(); }
    let (updates, receiver) = watch::channel(None);
    let session = Arc::new(Session { account_id, session_id, active: AtomicBool::new(true), updates, request: Mutex::new(None) });
    let event_session = session.session_id.clone();
    let task = tokio::spawn(heartbeat(session.clone(), receiver, HEARTBEAT, social_request_core, move |result| {
        let payload = match result {
            Ok(snapshot) => serde_json::json!({"sessionId": event_session, "snapshot": snapshot}),
            Err(error) => serde_json::json!({"sessionId": event_session, "error": error.to_string()}),
        };
        let _ = app.emit("social-presence", payload);
    }));
    *current = Some(Worker { session, task: task.abort_handle() });
    Ok(true)
}

pub async fn sync(account_id: String, session_id: String, request: SocialRequest) -> AppResult<serde_json::Value> {
    if !matches!(&request, SocialRequest::Sync { status, .. } if matches!(status.as_str(), "online" | "in_game")) {
        return Err(AppError::InvalidInput("Invalid presence update".into()));
    }
    let session = CURRENT.lock().await.as_ref()
        .filter(|worker| worker.session.account_id == account_id && worker.session.session_id == session_id)
        .map(|worker| worker.session.clone())
        .ok_or_else(|| AppError::InvalidState("Presence session ended".into()))?;
    let mut previous = session.request.lock().await;
    if !session.active.load(Ordering::Acquire) { return Err(AppError::InvalidState("Presence session ended".into())); }
    session.updates.send_replace(Some(request.clone()));
    let result = social_request_core(account_id, request).await;
    if result.is_ok() { *previous = Some(Instant::now()); }
    result
}

#[tauri::command]
pub async fn social_presence_stop(account_id: String, session_id: String) -> AppResult<()> {
    let session = {
        let mut current = CURRENT.lock().await;
        if !current.as_ref().is_some_and(|worker| worker.session.account_id == account_id && worker.session.session_id == session_id) { return Ok(()); }
        current.take().map(Worker::stop).unwrap()
    };
    let _request = session.request.lock().await;
    let current = CURRENT.lock().await;
    if current.as_ref().is_some_and(|worker| worker.session.account_id == account_id) { return Ok(()); }
    let result = social_request_core(account_id, SocialRequest::Sync { status: "offline".into(), instance_name: None, mc_version: None, loader: None, server_host: None, server_port: None, avatar_url: None }).await;
    drop(current);
    result.map(|_| ())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::AtomicUsize;

    fn request(status: &str) -> SocialRequest {
        SocialRequest::Sync { status: status.into(), instance_name: None, mc_version: None, loader: None, server_host: None, server_port: None, avatar_url: None }
    }

    #[tokio::test]
    async fn native_heartbeat_keeps_sending_without_frontend_timers_and_uses_latest_activity() {
        let (updates, receiver) = watch::channel(None);
        let session = Arc::new(Session { account_id: "fixture".into(), session_id: uuid::Uuid::new_v4().to_string(), active: AtomicBool::new(true), updates, request: Mutex::new(None) });
        let (sent, mut received) = tokio::sync::mpsc::unbounded_channel();
        let notifications = Arc::new(AtomicUsize::new(0));
        let observed = notifications.clone();
        let task = tokio::spawn(heartbeat(session.clone(), receiver, Duration::from_millis(20), move |id, body| {
            sent.send((id, serde_json::to_value(body).unwrap())).unwrap();
            async { Ok(serde_json::json!({"me": {}, "friends": []})) }
        }, move |_| { observed.fetch_add(1, Ordering::Relaxed); }));
        session.updates.send_replace(Some(request("online")));
        for _ in 0..3 {
            let (id, body) = tokio::time::timeout(Duration::from_secs(2), received.recv()).await.unwrap().unwrap();
            assert_eq!(id, "fixture");
            assert_eq!(body["status"], "online");
            assert!(body.get("avatarUrl").is_none());
        }
        session.updates.send_replace(Some(request("in_game")));
        let (_, body) = tokio::time::timeout(Duration::from_secs(2), received.recv()).await.unwrap().unwrap();
        assert_eq!(body["status"], "in_game");
        session.active.store(false, Ordering::Release);
        session.updates.send_replace(Some(request("online")));
        tokio::time::timeout(Duration::from_secs(2), task).await.unwrap().unwrap();
        assert!(received.try_recv().is_err());
        assert_eq!(notifications.load(Ordering::Relaxed), 4);
    }

    #[tokio::test]
    async fn recent_foreground_sync_avoids_a_duplicate_background_request() {
        let (updates, receiver) = watch::channel(Some(request("online")));
        let session = Arc::new(Session { account_id: "fixture".into(), session_id: uuid::Uuid::new_v4().to_string(), active: AtomicBool::new(true), updates, request: Mutex::new(Some(Instant::now())) });
        let calls = Arc::new(AtomicUsize::new(0));
        let count = calls.clone();
        let task = tokio::spawn(heartbeat(session.clone(), receiver, Duration::from_secs(30), move |_, _| {
            count.fetch_add(1, Ordering::Relaxed);
            async { Ok(serde_json::Value::Null) }
        }, |_| {}));
        session.updates.send_replace(Some(request("in_game")));
        tokio::time::sleep(Duration::from_millis(30)).await;
        assert_eq!(calls.load(Ordering::Relaxed), 0);
        session.active.store(false, Ordering::Release);
        session.updates.send_replace(Some(request("online")));
        tokio::time::timeout(Duration::from_secs(2), task).await.unwrap().unwrap();
    }
}
