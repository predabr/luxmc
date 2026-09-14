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

#[tauri::command]
#[allow(non_snake_case)]
pub async fn doctor_check_instance_conflicts(
    profileId: String,
) -> AppResult<crate::core::doctor::PreLaunchCheckResult> {
    let db = crate::db::shared_db().await?;
    let row = sqlx::query_as::<_, crate::db::models::ProfileRow>("SELECT * FROM profiles WHERE id = ?")
        .bind(&profileId)
        .fetch_optional(db.pool())
        .await?
        .ok_or_else(|| AppError::NotFound(format!("Instância {profileId} não encontrada")))?;

    let mods_dir = std::path::PathBuf::from(&row.game_dir).join("mods");
    if !mods_dir.exists() {
        return Ok(crate::core::doctor::PreLaunchCheckResult {
            has_conflicts: false,
            conflicts: Vec::new(),
            duplicates: Vec::new(),
        });
    }

    let mut jar_names = Vec::new();
    if let Ok(mut entries) = tokio::fs::read_dir(&mods_dir).await {
        while let Ok(Some(entry)) = entries.next_entry().await {
            let name = entry.file_name().to_string_lossy().to_string();
            if name.ends_with(".jar") || name.ends_with(".disabled") {
                jar_names.push(name);
            }
        }
    }

    Ok(crate::core::doctor::check_mod_conflicts(&jar_names))
}

