use tauri::command;

#[command]
pub async fn teamwork_preview() -> Result<String, String> {
    Ok("Teamwork Preview iniciado com sucesso".to_string())
}
