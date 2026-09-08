use crate::core::auth::AuthService;

pub struct AppState {
	pub http: reqwest::Client,
	pub auth: AuthService,
}

impl Default for AppState {
	fn default() -> Self {
		let http = reqwest::Client::new();
		let auth = AuthService::new(http.clone());
		Self { http, auth }
	}
}
