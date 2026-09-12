use crate::core::auth::AuthService;

pub struct AppState {
    pub http: reqwest::Client,
    pub auth: AuthService,
}

impl Default for AppState {
    fn default() -> Self {
        let http = reqwest::Client::builder()
            .user_agent("Luxmc/1.5.4-beta (Linux; Minecraft Launcher)")
            .build()
            .unwrap_or_else(|_| reqwest::Client::new());
        let auth = AuthService::new(http.clone());
        Self { http, auth }
    }
}
