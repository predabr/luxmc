use super::AuthAccount;
use crate::error::{AppError, AppResult};

pub(super) fn queue(account: AuthAccount, proof: (String, String)) {
    tokio::spawn(async move {
        if synchronize(&account, &proof).await.is_err() {
            tracing::warn!("Microsoft login completed; portal synchronization needs another sign-in");
        }
    });
}

async fn synchronize(account: &AuthAccount, proof: &(String, String)) -> AppResult<()> {
    use crate::commands::{lux_account, social};
    social::social_request_core(account.id.clone(), social::SocialRequest::Register { username: account.username.clone() }).await?;
    let identity = social::identity(&account.id).await?;
    let http = reqwest::Client::builder().redirect(reqwest::redirect::Policy::none())
        .timeout(std::time::Duration::from_secs(15)).build()?;
    let start = http.post(format!("{}/api/microsoft/launcher_start", lux_account::portal_base()))
        .bearer_auth(&identity).json(&serde_json::json!({"idToken":proof.0,"verifier":proof.1,"uuid":account.uuid,"username":account.username}))
        .send().await?.error_for_status()?;
    if start.content_length().is_some_and(|size| size > 4096) {
        return Err(AppError::InvalidState("Resposta de sincronização inválida.".into()));
    }
    let challenge: serde_json::Value = start.json().await?;
    let ticket = challenge.get("ticket").and_then(|value|value.as_str()).unwrap_or_default();
    let server_id = challenge.get("serverId").and_then(|value|value.as_str()).unwrap_or_default();
    if ticket.len() != 64 || server_id.len() != 40 || !ticket.bytes().chain(server_id.bytes()).all(|value|value.is_ascii_hexdigit()) {
        return Err(AppError::InvalidState("Confirmação de sincronização inválida.".into()));
    }
    http.post("https://sessionserver.mojang.com/session/minecraft/join")
        .json(&serde_json::json!({"accessToken":account.access_token,"selectedProfile":account.uuid,"serverId":server_id}))
        .send().await?.error_for_status()?;
    for attempt in 0..3 {
        let response = http.post(format!("{}/api/microsoft/launcher_complete", lux_account::portal_base()))
            .bearer_auth(&identity).json(&serde_json::json!({"ticket":ticket}))
            .send().await?;
        if response.status().is_success() { return Ok(()); }
        if attempt == 2 { response.error_for_status()?; }
        tokio::time::sleep(std::time::Duration::from_millis(500)).await;
    }
    Err(AppError::InvalidState("Sincronização Microsoft indisponível.".into()))
}
