use tauri::State;
use crate::core::optimizer::{
    detect_gpu, generate_aikar_flags, generate_standard_flags, get_performance_pack_info_for,
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

pub async fn optimizer_get_perf_pack_core(
    http: &reqwest::Client,
    loader: String,
    mc_version: String,
) -> PerformancePackInfo {
    get_performance_pack_info_for(http, &loader, &mc_version).await
}

#[tauri::command]
pub async fn optimizer_get_perf_pack(
    state: State<'_, AppState>,
    loader: String,
    mc_version: String,
) -> AppResult<PerformancePackInfo> {
    Ok(optimizer_get_perf_pack_core(&state.http, loader, mc_version).await)
}

pub async fn optimizer_install_perf_pack_core(
    http: &reqwest::Client,
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
        http,
        &game_dir,
        &profile.loader,
        &profile.mc_version,
    )
    .await?;

    // Update mod count in profile
    let mods_dir = game_dir.join("mods");
    if let Ok(count) = tokio::task::spawn_blocking(move || -> std::io::Result<usize> {
        let entries = std::fs::read_dir(mods_dir)?;
        Ok(entries
            .flatten()
            .filter(|e| e.path().extension().map_or(false, |ext| ext == "jar"))
            .count())
    })
    .await
    .map_err(|error| crate::error::AppError::Internal(error.to_string()))?
    {
        let _ = sqlx::query("UPDATE profiles SET mod_count = ? WHERE id = ?")
            .bind(count as i64)
            .bind(&instance_id)
            .execute(db.pool())
            .await;
    }

    Ok(installed)
}

#[tauri::command]
pub async fn optimizer_install_perf_pack(
    state: State<'_, AppState>,
    instance_id: String,
) -> AppResult<Vec<String>> {
    optimizer_install_perf_pack_core(&state.http, instance_id).await
}

#[tauri::command]
pub async fn optimizer_detect_gpu() -> GpuInfo {
    tokio::task::spawn_blocking(detect_gpu)
        .await
        .unwrap_or_else(|_| GpuInfo {
            vendor: "Desconhecido".to_string(),
            renderer: "Driver Padrão".to_string(),
            driver: "Desconhecido".to_string(),
            supports_zink: false,
        })
}

#[tauri::command]
pub fn optimizer_trim_memory() -> bool {
    crate::core::native_cpp::trim_memory_native()
}

#[tauri::command]
pub fn optimizer_native_cpu_profile() -> crate::core::native_cpp::NativeCpuProfile {
    crate::core::native_cpp::get_cpu_profile()
}

