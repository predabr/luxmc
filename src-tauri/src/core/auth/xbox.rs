use serde::{Deserialize, Serialize};

use crate::error::{AppError, AppResult};

const XBOX_USER_AUTH: &str = "https://user.auth.xboxlive.com/user/authenticate";
const XBOX_XSTS_AUTH: &str = "https://xsts.auth.xboxlive.com/xsts/authorize";

#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(non_snake_case)]
struct XboxUserRequest<'a> {
    RelyingParty: &'a str,
    TokenType: &'a str,
    Properties: XboxUserProperties<'a>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(non_snake_case)]
struct XboxUserProperties<'a> {
    AuthMethod: &'a str,
    SiteName: &'a str,
    RpsTicket: &'a str,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(non_snake_case)]
struct XstsRequest<'a> {
    RelyingParty: &'a str,
    TokenType: &'a str,
    Properties: XstsProperties<'a>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(non_snake_case)]
struct XstsProperties<'a> {
    SandboxId: &'a str,
    UserTokens: Vec<&'a str>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(non_snake_case)]
struct XboxTokenResponse {
    Token: String,
    DisplayClaims: serde_json::Value,
}

pub struct XboxClient {
    pub http: reqwest::Client,
}

impl XboxClient {
    pub fn new(http: reqwest::Client) -> Self {
        Self { http }
    }

    pub async fn user_token(&self, ms_access_token: &str) -> AppResult<super::XboxTokenSet> {
        let rps = format!("d={ms_access_token}");
        let req = XboxUserRequest {
            RelyingParty: "http://auth.xboxlive.com",
            TokenType: "JWT",
            Properties: XboxUserProperties {
                AuthMethod: "RPS",
                SiteName: "user.auth.xboxlive.com",
                RpsTicket: &rps,
            },
        };
        let resp = self
            .http
            .post(XBOX_USER_AUTH)
            .json(&req)
            .header("Accept", "application/json")
            .send()
            .await?;
        if !resp.status().is_success() {
            let status = resp.status();
            let text = resp.text().await.unwrap_or_default();
            return Err(AppError::Internal(format!(
                "Xbox Live user authenticate failed ({status}): {text}"
            )));
        }
        let body: XboxTokenResponse = resp.json().await?;
        let user_hash = extract_xuid(&body.DisplayClaims)
            .ok_or_else(|| AppError::InvalidState("xuid missing from xbox response".into()))?;
        self.xsts(&body.Token, &user_hash).await
    }

    async fn xsts(&self, user_token: &str, user_hash: &str) -> AppResult<super::XboxTokenSet> {
        let req = XstsRequest {
            RelyingParty: "rpc://api.minecraftservices.com/",
            TokenType: "JWT",
            Properties: XstsProperties {
                SandboxId: "RETAIL",
                UserTokens: vec![user_token],
            },
        };
        let resp = self
            .http
            .post(XBOX_XSTS_AUTH)
            .json(&req)
            .header("Accept", "application/json")
            .send()
            .await?;
        if !resp.status().is_success() {
            let status = resp.status();
            let text = resp.text().await.unwrap_or_default();
            return Err(AppError::Internal(format!(
                "Xbox Live XSTS authorize failed ({status}): {text}"
            )));
        }
        let body: XboxTokenResponse = resp.json().await?;
        Ok(super::XboxTokenSet {
            user_token: user_token.to_string(),
            xsts_token: body.Token,
            user_hash: user_hash.to_string(),
        })
    }
}

fn extract_xuid(claims: &serde_json::Value) -> Option<String> {
    claims
        .get("xui")
        .and_then(|v| v.as_array())
        .and_then(|arr| arr.first())
        .and_then(|obj| obj.get("uhs"))
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
}
