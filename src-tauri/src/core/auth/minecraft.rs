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
pub struct MinecraftProfileSkin {
    pub id: Option<String>,
    pub state: Option<String>,
    pub url: String,
    pub variant: Option<String>,
    pub alias: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MinecraftProfileCape {
    pub id: Option<String>,
    pub state: Option<String>,
    pub url: String,
    pub alias: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct MinecraftProfileResponse {
    id: String,
    name: String,
    #[serde(default)]
    skins: Vec<MinecraftProfileSkin>,
    #[serde(default)]
    capes: Vec<MinecraftProfileCape>,
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
            .await?;
        parse_profile_response(resp).await
    }
}

async fn parse_profile_response(resp: reqwest::Response) -> AppResult<super::MinecraftProfile> {
    if resp.status() == reqwest::StatusCode::NOT_FOUND {
        return Err(AppError::NotFound(
            concat!(
                "A Microsoft autorizou o login, mas esta conta não tem um perfil do Minecraft Java. ",
                "Confirme se ela possui Minecraft: Java Edition e finalize a criação do perfil em minecraft.net. ",
                "Contas apenas de Bedrock não podem entrar neste launcher."
            )
            .into(),
        ));
    }
    let body: MinecraftProfileResponse = resp.error_for_status()?.json().await?;
    let active_skin = body
        .skins
        .iter()
        .find(|s| s.state.as_deref() == Some("ACTIVE"))
        .or_else(|| body.skins.first());
    let active_cape = body
        .capes
        .iter()
        .find(|c| c.state.as_deref() == Some("ACTIVE"))
        .or_else(|| body.capes.first());

    let fix_url = |u: Option<String>| {
        u.map(|s| {
            if s.starts_with("http://") {
                s.replacen("http://", "https://", 1)
            } else {
                s
            }
        })
    };

    Ok(super::MinecraftProfile {
        id: body.id,
        name: body.name,
        skin_url: fix_url(active_skin.map(|s| s.url.clone())),
        skin_variant: active_skin.and_then(|s| s.variant.clone()),
        cape_url: fix_url(active_cape.map(|c| c.url.clone())),
    })
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

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::io::{AsyncReadExt, AsyncWriteExt};

    #[tokio::test]
    async fn missing_java_profile_returns_an_actionable_login_error() {
        let listener = tokio::net::TcpListener::bind(("127.0.0.1", 0))
            .await
            .unwrap();
        let address = listener.local_addr().unwrap();
        tokio::spawn(async move {
            let (mut stream, _) = listener.accept().await.unwrap();
            let mut request = [0; 1024];
            let _ = stream.read(&mut request).await;
            stream
                .write_all(
                    b"HTTP/1.1 404 Not Found\r\nContent-Length: 0\r\nConnection: close\r\n\r\n",
                )
                .await
                .unwrap();
        });

        let response = reqwest::Client::new()
            .get(format!("http://{address}/minecraft/profile"))
            .send()
            .await
            .unwrap();
        let error = parse_profile_response(response).await.unwrap_err();

        assert!(matches!(error, AppError::NotFound(_)));
        assert!(error.to_string().contains("Minecraft: Java Edition"));
        assert!(error.to_string().contains("minecraft.net"));
    }
}
