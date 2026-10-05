use crate::error::{AppError, AppResult};
use serde::Deserialize;
const PORTAL: &str = "https://luxmc-r92.pages.dev/api/invites";
fn error(message: impl std::fmt::Display) -> AppError { AppError::InvalidState(format!("Código Luxmc: {message}")) }
pub fn is_short_code(value: &str) -> bool { value.strip_prefix("LUXMC1-").is_some_and(|digits| digits.len() == 16 && digits.bytes().all(|b| b.is_ascii_digit())) }
fn client() -> AppResult<reqwest::Client> { reqwest::Client::builder().timeout(std::time::Duration::from_secs(12)).user_agent(concat!("Luxmc/", env!("CARGO_PKG_VERSION"))).build().map_err(error) }
#[derive(Deserialize)]
struct Published { code: String }
#[derive(Deserialize)]
struct Resolved { kind: String, payload: String }
pub async fn publish(kind: &str, payload: &str) -> AppResult<String> {
    let response = client()?.post(format!("{PORTAL}/create")).json(&serde_json::json!({"kind":kind,"payload":payload})).send().await.map_err(error)?;
    if !response.status().is_success() { return Err(error("Não foi possível registrar o código no portal. Confira a conexão e tente novamente.")); }
    let published: Published = response.json().await.map_err(error)?;
    if !is_short_code(&published.code) { return Err(error("Resposta inválida do portal")); }
    Ok(published.code)
}
pub async fn resolve(code: &str, kind: &str) -> AppResult<String> {
    let normalized = code.trim().to_uppercase();
    if !is_short_code(&normalized) { return Err(error("Use LUXMC1- seguido de 16 números")); }
    let response = client()?.get(format!("{PORTAL}/{normalized}")).send().await.map_err(error)?;
    if !response.status().is_success() { return Err(error("Código não encontrado, expirado ou portal indisponível")); }
    let resolved: Resolved = response.json().await.map_err(error)?;
    if resolved.kind != kind || resolved.payload.len() > 131072 { return Err(error("Este código pertence a outro tipo de compartilhamento")); }
    Ok(resolved.payload)
}
#[cfg(test)]
mod tests {
    #[test]
    fn accepts_only_sixteen_decimal_digits() {
        assert!(super::is_short_code("LUXMC1-1234567890123456"));
        for invalid in ["LUXMC1-1", "LUXMC1-123456789012345a", "LUXMC1-12345678901234567", "../etc/passwd"] { assert!(!super::is_short_code(invalid)); }
    }
}
