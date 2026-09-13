use std::sync::Arc;
use std::sync::atomic::AtomicBool;
use crate::core::auth::AuthService;

pub struct AppState {
    pub http: reqwest::Client,
    pub auth: AuthService,
    pub import_cancel: Arc<AtomicBool>,
}

impl Default for AppState {
    fn default() -> Self {
        let http = reqwest::Client::builder()
            .user_agent("Luxmc/1.5.5-beta (Linux; Minecraft Launcher)")
            .build()
            .unwrap_or_else(|_| reqwest::Client::new());
        let auth = AuthService::new(http.clone());
        Self { http, auth, import_cancel: Arc::new(AtomicBool::new(false)) }
    }
}
