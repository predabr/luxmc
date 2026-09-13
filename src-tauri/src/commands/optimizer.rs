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

#[tauri::command]
pub fn optimizer_trim_memory() -> bool {
    #[cfg(target_os = "linux")]
    {
        extern "C" {
            fn malloc_trim(pad: usize) -> i32;
            fn madvise(addr: *mut std::ffi::c_void, length: usize, advice: i32) -> i32;
        }
        const MADV_DONTNEED: i32 = 4;
        unsafe {
            malloc_trim(0);
        }
        if let Ok(maps) = std::fs::read_to_string("/proc/self/maps") {
            for line in maps.lines() {
                if line.contains("[heap]") || (line.ends_with(" 0") && line.contains("rw-p")) {
                    if let Some(range) = line.split_whitespace().next() {
                        let parts: Vec<&str> = range.split('-').collect();
                        if parts.len() == 2 {
                            if let (Ok(start), Ok(end)) = (
                                usize::from_str_radix(parts[0], 16),
                                usize::from_str_radix(parts[1], 16),
                            ) {
                                if end > start && (end - start) <= 64 * 1024 * 1024 {
                                    unsafe {
                                        madvise(
                                            start as *mut std::ffi::c_void,
                                            end - start,
                                            MADV_DONTNEED,
                                        );
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        true
    }
    #[cfg(target_os = "windows")]
    {
        use std::os::raw::c_void;
        extern "system" {
            fn GetCurrentProcess() -> *mut c_void;
            fn SetProcessWorkingSetSize(
                hProcess: *mut c_void,
                dwMinimumWorkingSetSize: usize,
                dwMaximumWorkingSetSize: usize,
            ) -> i32;
        }
        unsafe {
            let handle = GetCurrentProcess();
            SetProcessWorkingSetSize(handle, usize::MAX, usize::MAX);
        }
        true
    }
    #[cfg(not(any(target_os = "linux", target_os = "windows")))]
    {
        false
    }
}

