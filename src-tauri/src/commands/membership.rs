use crate::error::{AppError, AppResult};
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
#[serde(tag = "action")]
pub enum MembershipRequest {
    #[serde(rename = "status")]
    Status,
    #[serde(rename = "portal_ticket")]
    PortalTicket { owner: bool, #[serde(default, skip_serializing_if = "Option::is_none")] destination: Option<String> },
    #[serde(rename = "preferences")]
    Preferences { preferences: serde_json::Value, presets: Vec<serde_json::Value>, revision: u32 },
}

#[tauri::command]
pub async fn membership_request(account_id: String, request: MembershipRequest) -> AppResult<serde_json::Value> {
    membership_request_core(account_id, request).await
}

pub(crate) async fn membership_request_core(account_id: String, request: MembershipRequest) -> AppResult<serde_json::Value> {
    if account_id.is_empty() || account_id.len() > 128 { return Err(AppError::InvalidInput("Conta inválida.".into())); }
    let token = if account_id.starts_with("luxmc:") { super::lux_account::token(&account_id).await? } else { super::social::identity(&account_id).await? };
    let body = serde_json::to_value(request)?;
    let action = body.get("action").and_then(|value| value.as_str()).ok_or_else(|| AppError::InvalidInput("Pedido inválido.".into()))?;
    if serde_json::to_vec(&body)?.len() > 24576 { return Err(AppError::InvalidInput("Preferências muito grandes.".into())); }
    let client = reqwest::Client::builder().redirect(reqwest::redirect::Policy::none()).timeout(std::time::Duration::from_secs(15)).build()?;
    let response = client.post(format!("{}/api/commerce/{action}", super::lux_account::portal_base())).bearer_auth(token).json(&body).send().await?;
    let status = response.status();
    if response.content_length().unwrap_or(0) > 262144 { return Err(AppError::InvalidState("Resposta de planos muito grande.".into())); }
    let bytes = response.bytes().await?;
    if bytes.len() > 262144 { return Err(AppError::InvalidState("Resposta de planos muito grande.".into())); }
    let value: serde_json::Value = serde_json::from_slice(&bytes).map_err(|_| AppError::InvalidState("O serviço de planos está indisponível.".into()))?;
    if !status.is_success() { return Err(AppError::InvalidState(value.get("error").and_then(|value| value.as_str()).unwrap_or("Não foi possível consultar seu plano.").into())); }
    Ok(value)
}
