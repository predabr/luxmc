use tauri::State;
use crate::core::optimizer::{
    detect_gpu, generate_aikar_flags, generate_standard_flags, get_performance_pack_info,
    install_performance_pack, GpuInfo, PerformancePackInfo,
};
use crate::error::{AppError, AppResult};
use crate::state::AppState;

#[tauri::command]
pub fn optimizer_get_flags(ram_mb: u64, auto_optimize: bool) -> Vec<String> {
    if auto_optimize {
        generate_aikar_flags(ram_mb)
    } else {
        generate_standard_flags(ram_mb)
    }
}

#[tauri::command]
pub fn optimizer_get_perf_pack(loader: String, mc_version: String) -> PerformancePackInfo {
    get_performance_pack_info(&loader, &mc_version)
}

#[tauri::command]
pub async fn optimizer_install_perf_pack(
    state: State<'_, AppState>,
    instance_id: String,
) -> AppResult<Vec<String>> {
    let db = crate::db::shared_db().await?;
    let profile = sqlx::query_as::<_, crate::db::models::ProfileRow>(
        "SELECT * FROM profiles WHERE id = ?",
    )
    .bind(&instance_id)
    .fetch_optional(db.pool())
    .await?
    .ok_or_else(|| AppError::NotFound(format!("Instância {} não encontrada", instance_id)))?;

    let game_dir = std::path::PathBuf::from(&profile.game_dir);
    let installed = install_performance_pack(
        &state.http,
        &game_dir,
        &profile.loader,
        &profile.mc_version,
    )
    .await?;

    // Update mod count in profile
    let mods_dir = game_dir.join("mods");
    if let Ok(entries) = std::fs::read_dir(&mods_dir) {
        let count = entries
            .flatten()
            .filter(|e| e.path().extension().map_or(false, |ext| ext == "jar"))
            .count() as i64;
        let _ = sqlx::query("UPDATE profiles SET mod_count = ? WHERE id = ?")
            .bind(count)
            .bind(&instance_id)
            .execute(db.pool())
            .await;
    }

    Ok(installed)
}

#[tauri::command]
pub fn optimizer_detect_gpu() -> GpuInfo {
    detect_gpu()
}
