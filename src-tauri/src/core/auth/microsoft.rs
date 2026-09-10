use async_trait::async_trait;
use base64::Engine;
use rand::RngCore;
use serde::{Deserialize, Serialize};

use crate::error::{AppError, AppResult};

use super::{default_client_id, pkce_challenge, pkce_verifier, AuthProvider, MicrosoftTokens};

const AUTH_URL: &str = "https://login.microsoftonline.com/consumers/oauth2/v2.0/authorize";
const TOKEN_URL: &str = "https://login.microsoftonline.com/consumers/oauth2/v2.0/token";
const REDIRECT_URI: &str = "http://localhost:8453/callback";
const SCOPES: &str = "offline_access XBoxLive.signin";

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PendingAuth {
    pub state: String,
    pub verifier: String,
    pub url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MicrosoftTokenResponse {
    pub token_type: String,
    pub scope: String,
    pub expires_in: i64,
    pub ext_expires_in: i64,
    pub access_token: String,
    pub refresh_token: String,
}

pub struct MicrosoftOAuth {
    pub client_id: String,
    pub http: reqwest::Client,
}

impl MicrosoftOAuth {
    pub fn new(http: reqwest::Client) -> Self {
        Self {
            client_id: default_client_id(),
            http,
        }
    }

    pub fn begin(&self) -> PendingAuth {
        self.begin_with_client_id(None)
    }

    pub fn begin_with_client_id(&self, client_id_override: Option<&str>) -> PendingAuth {
        let cid = client_id_override.filter(|s| !s.is_empty()).unwrap_or(&self.client_id);
        let verifier = pkce_verifier();
        let challenge = pkce_challenge(&verifier);
        let state = random_state();
        let url = format!(
			"{AUTH_URL}?client_id={cid}&response_type=code&redirect_uri={ru}&response_mode=query&scope={scopes}&state={state}&code_challenge={challenge}&code_challenge_method=S256",
			cid = urlencoding(cid),
			ru = urlencoding(REDIRECT_URI),
			scopes = urlencoding(SCOPES),
			state = state,
			challenge = challenge,
		);
        PendingAuth {
            state,
            verifier,
            url,
        }
    }

    pub async fn exchange_code(&self, code: &str, verifier: &str) -> AppResult<MicrosoftTokens> {
        self.exchange_code_with_client_id(code, verifier, None).await
    }

    pub async fn exchange_code_with_client_id(
        &self,
        code: &str,
        verifier: &str,
        client_id_override: Option<&str>,
    ) -> AppResult<MicrosoftTokens> {
        let cid = client_id_override.filter(|s| !s.is_empty()).unwrap_or(&self.client_id);
        let form = [
            ("client_id", cid),
            ("code", code),
            ("code_verifier", verifier),
            ("grant_type", "authorization_code"),
            ("redirect_uri", REDIRECT_URI),
        ];
        let resp = self
            .http
            .post(TOKEN_URL)
            .form(&form)
            .send()
            .await?
            .error_for_status()?;
        let body: MicrosoftTokenResponse = resp.json().await?;
        Ok(MicrosoftTokens {
            access_token: body.access_token,
            refresh_token: body.refresh_token,
            expires_in: body.expires_in,
        })
    }
}

fn random_state() -> String {
    let mut buf = [0u8; 24];
    rand::thread_rng().fill_bytes(&mut buf);
    base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(buf)
}

fn urlencoding(s: &str) -> String {
    s.chars()
        .flat_map(|c| {
            if c.is_ascii_alphanumeric() || matches!(c, '-' | '_' | '.' | '~') {
                vec![c]
            } else {
                let mut out = vec![];
                let mut buf = [0u8; 4];
                let s = c.encode_utf8(&mut buf);
                for b in s.bytes() {
                    out.push('%');
                    out.push(nibble((b >> 4) & 0xF) as char);
                    out.push(nibble(b & 0xF) as char);
                }
                out
            }
        })
        .collect()
}

fn nibble(b: u8) -> u8 {
    match b {
        0..=9 => b'0' + b,
        10..=15 => b'A' + (b - 10),
        _ => b'0',
    }
}

#[async_trait]
impl AuthProvider for MicrosoftOAuth {
    async fn login(&self) -> AppResult<super::AuthAccount> {
        Err(AppError::NotImplemented(
            "login flow must go through begin + exchange_code (UI-driven)",
        ))
    }

    async fn refresh(&self, refresh_token: &str) -> AppResult<MicrosoftTokens> {
        let form = [
            ("client_id", self.client_id.as_str()),
            ("refresh_token", refresh_token),
            ("grant_type", "refresh_token"),
            ("redirect_uri", REDIRECT_URI),
        ];
        let resp = self
            .http
            .post(TOKEN_URL)
            .form(&form)
            .send()
            .await?
            .error_for_status()?;
        let body: MicrosoftTokenResponse = resp.json().await?;
        Ok(MicrosoftTokens {
            access_token: body.access_token,
            refresh_token: body.refresh_token,
            expires_in: body.expires_in,
        })
    }

    async fn xbox_token(&self, _ms_access_token: &str) -> AppResult<super::XboxTokenSet> {
        Err(AppError::NotImplemented("use XboxClient directly"))
    }

    async fn minecraft_token(&self, _user_hash: &str, _xsts_token: &str) -> AppResult<String> {
        Err(AppError::NotImplemented("use MinecraftClient directly"))
    }

    async fn check_entitlements(
        &self,
        _mc_access_token: &str,
    ) -> AppResult<Vec<super::Entitlement>> {
        Err(AppError::NotImplemented("use MinecraftClient directly"))
    }

    async fn profile(&self, _mc_access_token: &str) -> AppResult<super::MinecraftProfile> {
        Err(AppError::NotImplemented("use MinecraftClient directly"))
    }
}
