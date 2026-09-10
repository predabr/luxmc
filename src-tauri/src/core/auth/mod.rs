use async_trait::async_trait;
use base64::Engine;
use chrono::Utc;
use rand::RngCore;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::error::AppResult;

pub mod microsoft;
pub mod minecraft;
pub mod oauth_server;
pub mod xbox;

pub use microsoft::MicrosoftOAuth;
pub use minecraft::MinecraftClient;
pub use xbox::XboxClient;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AuthAccount {
    pub id: String,
    pub username: String,
    pub uuid: String,
    pub access_token: String,
    pub refresh_token: String,
    pub expires_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MicrosoftTokens {
    pub access_token: String,
    pub refresh_token: String,
    pub expires_in: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct XboxTokenSet {
    pub user_token: String,
    pub xsts_token: String,
    pub user_hash: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MinecraftProfile {
    pub id: String,
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Entitlement {
    pub name: String,
    pub signature: String,
}

#[async_trait]
pub trait AuthProvider: Send + Sync {
    async fn login(&self) -> AppResult<AuthAccount>;
    async fn refresh(&self, refresh_token: &str) -> AppResult<MicrosoftTokens>;
    async fn xbox_token(&self, ms_access_token: &str) -> AppResult<XboxTokenSet>;
    async fn minecraft_token(&self, user_hash: &str, xsts_token: &str) -> AppResult<String>;
    async fn check_entitlements(&self, mc_access_token: &str) -> AppResult<Vec<Entitlement>>;
    async fn profile(&self, mc_access_token: &str) -> AppResult<MinecraftProfile>;
}

pub fn pkce_verifier() -> String {
    let mut buf = [0u8; 48];
    rand::thread_rng().fill_bytes(&mut buf);
    base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(buf)
}

pub fn pkce_challenge(verifier: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(verifier.as_bytes());
    let digest = hasher.finalize();
    base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(digest)
}

pub fn default_client_id() -> String {
    std::env::var("LUXMC_MS_CLIENT_ID")
        .unwrap_or_else(|_| "00000000-0000-0000-0000-000000000000".to_string())
}

pub struct AuthService {
    microsoft: MicrosoftOAuth,
    xbox: XboxClient,
    minecraft: MinecraftClient,
}

impl AuthService {
    pub fn new(http: reqwest::Client) -> Self {
        Self {
            microsoft: MicrosoftOAuth::new(http.clone()),
            xbox: XboxClient::new(http.clone()),
            minecraft: MinecraftClient::new(http),
        }
    }

    pub fn begin(&self) -> microsoft::PendingAuth {
        self.microsoft.begin()
    }

    pub fn begin_with_client_id(&self, client_id: Option<&str>) -> microsoft::PendingAuth {
        self.microsoft.begin_with_client_id(client_id)
    }

    pub async fn exchange_code(&self, code: &str, verifier: &str) -> AppResult<MicrosoftTokens> {
        self.microsoft.exchange_code(code, verifier).await
    }

    pub async fn exchange_code_with_client_id(
        &self,
        code: &str,
        verifier: &str,
        client_id: Option<&str>,
    ) -> AppResult<MicrosoftTokens> {
        self.microsoft.exchange_code_with_client_id(code, verifier, client_id).await
    }

    pub async fn refresh(&self, refresh_token: &str) -> AppResult<MicrosoftTokens> {
        self.microsoft.refresh(refresh_token).await
    }

    pub async fn login_with_code(&self, code: &str, verifier: &str) -> AppResult<AuthAccount> {
        let ms_tokens = self.exchange_code(code, verifier).await?;
        self.complete_login(ms_tokens).await
    }

    pub async fn login_with_code_and_client_id(
        &self,
        code: &str,
        verifier: &str,
        client_id: Option<&str>,
    ) -> AppResult<AuthAccount> {
        let ms_tokens = self.exchange_code_with_client_id(code, verifier, client_id).await?;
        self.complete_login(ms_tokens).await
    }

    pub async fn refresh_account(&self, refresh_token: &str) -> AppResult<AuthAccount> {
        let ms_tokens = self.refresh(refresh_token).await?;
        self.complete_login(ms_tokens).await
    }

    async fn complete_login(&self, ms_tokens: MicrosoftTokens) -> AppResult<AuthAccount> {
        let xbox_tokens = self.xbox.user_token(&ms_tokens.access_token).await?;
        let mc_token = self
            .minecraft
            .login(&xbox_tokens.user_hash, &xbox_tokens.xsts_token)
            .await?;
        self.minecraft.check_entitlements(&mc_token).await?;
        let profile = self.minecraft.profile(&mc_token).await?;

        Ok(AuthAccount {
            id: profile.id.clone(),
            username: profile.name,
            uuid: profile.id,
            access_token: mc_token,
            refresh_token: ms_tokens.refresh_token,
            expires_at: Utc::now().timestamp() + ms_tokens.expires_in,
        })
    }
}
