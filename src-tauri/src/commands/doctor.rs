use tauri::State;
use crate::core::doctor::{analyze_crash_text, diagnose_instance, CrashDiagnosis};
use crate::error::{AppError, AppResult};
use crate::state::AppState;

#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct InstanceReadiness {
    pub ready: bool,
    pub required_java: u32,
    pub allocated_ram_mb: i64,
    pub recommended_ram_mb: i64,
    pub gpu_vendor: String,
    pub gpu_driver: String,
    pub blockers: Vec<String>,
    pub warnings: Vec<String>,
    pub conflicts: crate::core::doctor::PreLaunchCheckResult,
}

#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RepairAllOutcome {
    pub repaired_mods: u32,
    pub warnings: Vec<String>,
}

fn recommended_ram_mb(mod_count: i64, total_ram_mb: i64) -> i64 {
    let requested = if mod_count >= 100 {
        6144
    } else if mod_count >= 30 {
        4096
    } else {
        2048
    };
    requested.min((total_ram_mb - 3072).max(1024))
}

#[tauri::command]
#[allow(non_snake_case)]
pub async fn doctor_instance_readiness(profileId: String) -> AppResult<InstanceReadiness> {
    let db = crate::db::shared_db().await?;
    let row = sqlx::query_as::<_, crate::db::models::ProfileRow>("SELECT * FROM profiles WHERE id = ?")
        .bind(&profileId)
        .fetch_optional(db.pool())
        .await?
        .ok_or_else(|| AppError::NotFound(format!("Instância {profileId} não encontrada")))?;
    let health = crate::commands::instances::instance_health_check(profileId.clone()).await?;
    let conflicts = doctor_check_instance_conflicts(profileId).await?;
    let required_java = crate::core::minecraft::detect_java_major_from_version_id(&row.mc_version);
    let total_ram_mb = crate::core::optimizer::get_total_memory_mb();
    let recommended_ram_mb = recommended_ram_mb(row.mod_count, total_ram_mb);
    let allocated_ram_mb = row.ram_mb.unwrap_or(recommended_ram_mb);
    let gpu = crate::core::optimizer::detect_gpu();
    let mut blockers = Vec::new();
    let mut warnings = Vec::new();
    if !health.client_jar {
        blockers.push("Os arquivos principais do Minecraft estão ausentes.".into());
    }
    if !health.natives {
        blockers.push("As bibliotecas nativas da versão não estão prontas.".into());
    }
    if !health.mods_ok {
        warnings.extend(health.issues);
    }
    if conflicts.has_conflicts {
        blockers.push("Há conflitos ou duplicatas de mods que impedem a inicialização segura.".into());
    }
    if !matches!(row.loader.to_ascii_lowercase().as_str(), "vanilla" | "fabric" | "forge" | "neoforge" | "quilt" | "optifine") {
        blockers.push("O loader configurado para a instância não é suportado.".into());
    }
    if allocated_ram_mb < recommended_ram_mb {
        warnings.push(format!("Esta instância tem {} MB; recomenda-se pelo menos {} MB.", allocated_ram_mb, recommended_ram_mb));
    }
    if allocated_ram_mb > (total_ram_mb - 1536).max(1024) {
        warnings.push("A RAM configurada deixa pouca memória para o sistema operacional.".into());
    }
    if gpu.vendor == "Desconhecido" {
        warnings.push("Não foi possível identificar a GPU; o Luxmc usará a configuração segura padrão.".into());
    }
    #[cfg(target_os = "linux")]
    if gpu.vendor == "NVIDIA" && std::env::var_os("WAYLAND_DISPLAY").is_some() {
        warnings.push("NVIDIA em Wayland detectada; use o modo de compatibilidade XWayland se ocorrer falha gráfica.".into());
    }
    Ok(InstanceReadiness {
        ready: blockers.is_empty(),
        required_java,
        allocated_ram_mb,
        recommended_ram_mb,
        gpu_vendor: gpu.vendor,
        gpu_driver: gpu.driver,
        blockers,
        warnings,
        conflicts,
    })
}

#[tauri::command]
#[allow(non_snake_case)]
pub async fn doctor_repair_all(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
    profileId: String,
) -> AppResult<RepairAllOutcome> {
    crate::commands::instance_tools::instance_repair_core(
        Some(app.clone()),
        &state.http,
        profileId.clone(),
    )
    .await?;
    let mut warnings = Vec::new();
    let repaired_mods = match crate::commands::instances::instance_repair_modpack_core(
        Some(app),
        &state,
        profileId,
    )
    .await
    {
        Ok(repaired) => repaired,
        Err(error) => {
            warnings.push(format!("Os arquivos do jogo foram reparados, mas os mods não precisaram ou não puderam ser reparados: {error}"));
            0
        }
    };
    Ok(RepairAllOutcome { repaired_mods, warnings })
}

#[tauri::command]
#[allow(non_snake_case)]
pub async fn crash_doctor_diagnose(
    _state: State<'_, AppState>,
    profileId: String,
    logContent: Option<String>,
) -> AppResult<CrashDiagnosis> {
    crash_doctor_diagnose_core(profileId, logContent).await
}

pub async fn crash_doctor_diagnose_core(
    profile_id: String,
    log_content: Option<String>,
) -> AppResult<CrashDiagnosis> {
    if let Some(text) = log_content.filter(|t| !t.trim().is_empty()) {
        let diagnosis = analyze_crash_text(&text);
        if diagnosis.has_error {
            return Ok(diagnosis);
        }
    }

    let db = crate::db::shared_db().await?;
    let row = sqlx::query_as::<_, crate::db::models::ProfileRow>("SELECT * FROM profiles WHERE id = ?")
        .bind(&profile_id)
        .fetch_optional(db.pool())
        .await?
        .ok_or_else(|| AppError::NotFound(format!("Instância {profile_id} não encontrada")))?;

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
    let mut jar_paths = Vec::new();
    if let Ok(mut entries) = tokio::fs::read_dir(&mods_dir).await {
        while let Ok(Some(entry)) = entries.next_entry().await {
            let name = entry.file_name().to_string_lossy().to_string();
            if name.ends_with(".jar") || name.ends_with(".disabled") {
                jar_paths.push(entry.path());
                jar_names.push(name);
            }
        }
    }

    Ok(crate::core::doctor::check_mod_conflicts_with_paths(&jar_names, &jar_paths))
}
