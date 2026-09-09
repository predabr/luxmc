use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use crate::error::{AppError, AppResult};

use super::AuthProvider;

const MC_LOGIN: &str = "https://api.minecraftservices.com/authentication/login_with_xbox";
const MC_ENTITLEMENT: &str = "https://api.minecraftservices.com/entitlements/mcstore";
const MC_PROFILE: &str = "https://api.minecraftservices.com/minecraft/profile";

#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(non_snake_case)]
struct MinecraftLoginRequest {
    identityToken: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct MinecraftLoginResponse {
    access_token: String,
    expires_in: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct MinecraftProfileResponse {
    id: String,
    name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(non_snake_case)]
struct EntitlementsResponse {
    items: Vec<super::Entitlement>,
    signature: String,
    keyId: String,
}

pub struct MinecraftClient {
    pub http: reqwest::Client,
}

impl MinecraftClient {
    pub fn new(http: reqwest::Client) -> Self {
        Self { http }
    }

    pub async fn login(&self, user_hash: &str, xsts: &str) -> AppResult<String> {
        let identity = format!("XBL3.0 x={user_hash};{xsts}");
        let resp = self
            .http
            .post(MC_LOGIN)
            .json(&MinecraftLoginRequest {
                identityToken: identity,
            })
            .send()
            .await?
            .error_for_status()?;
        let body: MinecraftLoginResponse = resp.json().await?;
        Ok(body.access_token)
    }

    pub async fn check_entitlements(&self, mc_token: &str) -> AppResult<Vec<super::Entitlement>> {
        let resp = self
            .http
            .get(MC_ENTITLEMENT)
            .bearer_auth(mc_token)
            .send()
            .await?
            .error_for_status()?;
        let body: EntitlementsResponse = resp.json().await?;
        Ok(body.items)
    }

    pub async fn profile(&self, mc_token: &str) -> AppResult<super::MinecraftProfile> {
        let resp = self
            .http
            .get(MC_PROFILE)
            .bearer_auth(mc_token)
            .send()
            .await?
            .error_for_status()?;
        if resp.status() == reqwest::StatusCode::NOT_FOUND {
            return Err(AppError::NotFound("minecraft profile not found".into()));
        }
        let body: MinecraftProfileResponse = resp.json().await?;
        Ok(super::MinecraftProfile {
            id: body.id,
            name: body.name,
        })
    }
}

#[async_trait]
impl AuthProvider for MinecraftClient {
    async fn login(&self) -> AppResult<super::AuthAccount> {
        Err(AppError::NotImplemented("compose steps manually"))
    }
    async fn refresh(&self, _refresh_token: &str) -> AppResult<super::MicrosoftTokens> {
        Err(AppError::NotImplemented("delegate to MicrosoftOAuth"))
    }
    async fn xbox_token(&self, _ms_access_token: &str) -> AppResult<super::XboxTokenSet> {
        Err(AppError::NotImplemented("delegate to XboxClient"))
    }
    async fn minecraft_token(&self, _user_hash: &str, _xsts_token: &str) -> AppResult<String> {
        Err(AppError::NotImplemented("use MinecraftClient::login"))
    }
    async fn check_entitlements(
        &self,
        mc_access_token: &str,
    ) -> AppResult<Vec<super::Entitlement>> {
        MinecraftClient::check_entitlements(self, mc_access_token).await
    }
    async fn profile(&self, mc_access_token: &str) -> AppResult<super::MinecraftProfile> {
        MinecraftClient::profile(self, mc_access_token).await
    }
}
