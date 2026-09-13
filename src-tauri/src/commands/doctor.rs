use tauri::State;
use crate::core::doctor::{analyze_crash_text, diagnose_instance, CrashDiagnosis};
use crate::error::{AppError, AppResult};
use crate::state::AppState;

#[tauri::command]
#[allow(non_snake_case)]
pub async fn crash_doctor_diagnose(
    _state: State<'_, AppState>,
    profileId: String,
    logContent: Option<String>,
) -> AppResult<CrashDiagnosis> {
    if let Some(text) = logContent.filter(|t| !t.trim().is_empty()) {
        let diagnosis = analyze_crash_text(&text);
        if diagnosis.has_error {
            return Ok(diagnosis);
        }
    }

    let db = crate::db::shared_db().await?;
    let row = sqlx::query_as::<_, crate::db::models::ProfileRow>("SELECT * FROM profiles WHERE id = ?")
        .bind(&profileId)
        .fetch_optional(db.pool())
        .await?
        .ok_or_else(|| AppError::NotFound(format!("Instância {profileId} não encontrada")))?;

    let game_dir = std::path::PathBuf::from(&row.game_dir);
    Ok(diagnose_instance(&game_dir))
}
