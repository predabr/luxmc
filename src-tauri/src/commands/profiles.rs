use chrono::Utc;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::db::models::ProfileRow;
use crate::error::{AppError, AppResult};

const PROTECTED_ROOTS: &[&str] = &[
    "/etc", "/usr", "/bin", "/sbin", "/boot", "/dev", "/proc", "/sys", "/run", "/var", "/lib",
    "/lib64", "/snap", "/System", "/Windows", "/Program Files",
];

fn validate_game_dir(path: &str) -> AppResult<()> {
    let trimmed = path.trim();
    if trimmed.is_empty() || trimmed == "." || trimmed == ".." || trimmed.contains('\0') {
        return Err(AppError::InvalidInput("Caminho de diretório inválido".into()));
    }
    let candidate = std::path::Path::new(trimmed);
    if !candidate.is_absolute() {
        return Err(AppError::InvalidInput(
            "O diretório do jogo precisa ser um caminho absoluto".into(),
        ));
    }
    let mut parts = 0usize;
    let mut text = String::new();
    for component in candidate.components() {
        match component {
            std::path::Component::RootDir => {}
            std::path::Component::Prefix(_) => {}
            std::path::Component::CurDir | std::path::Component::ParentDir => {
                return Err(AppError::InvalidInput("Caminho de diretório inválido".into()));
            }
            std::path::Component::Normal(value) => {
                parts += 1;
                text.push('/');
                text.push_str(&value.to_string_lossy());
            }
        }
    }
    if parts < 3 {
        return Err(AppError::InvalidInput(
            "O diretório do jogo é profundo demais para ser usado com segurança".into(),
        ));
    }
    if PROTECTED_ROOTS.iter().any(|root| text == *root || text.starts_with(&format!("{root}/"))) {
        return Err(AppError::InvalidInput(
            "O diretório do jogo não pode ficar em uma pasta do sistema".into(),
        ));
    }
    Ok(())
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProfileCreate {
    pub name: String,
    pub icon: Option<String>,
    pub mc_version: String,
    pub loader: String,
    pub loader_version: Option<String>,
    pub java_path: Option<String>,
    pub jvm_args: Option<String>,
    pub resolution_w: Option<i64>,
    pub resolution_h: Option<i64>,
    pub fullscreen: Option<bool>,
    pub game_dir: Option<String>,
    #[serde(default)]
    pub favorite: Option<bool>,
    #[serde(default)]
    pub notes: Option<String>,
    #[serde(default)]
    pub ram_mb: Option<i64>,
    #[serde(default)]
    pub instance_group: Option<String>,
    #[serde(default)]
    pub auto_optimize: Option<bool>,
    #[serde(default)]
    pub use_vulkan: Option<bool>,
    #[serde(default)]
    pub use_gamemode: Option<bool>,
    #[serde(default)]
    pub use_mangohud: Option<bool>,
    #[serde(default)]
    pub force_dedicated_gpu: Option<bool>,
    #[serde(default)]
    pub use_gamescope: Option<bool>,
    #[serde(default)]
    pub gamescope_width: Option<Option<i64>>,
    #[serde(default)]
    pub gamescope_height: Option<Option<i64>>,
    #[serde(default)]
    pub gamescope_fsr: Option<bool>,
    #[serde(default)]
    pub force_full_verification: Option<bool>,
    #[serde(default)]
    pub pre_launch_hook: Option<Option<String>>,
    #[serde(default)]
    pub post_exit_hook: Option<Option<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProfileUpdate {
    pub id: String,
    pub name: Option<String>,
    pub icon: Option<String>,
    pub mc_version: Option<String>,
    pub loader: Option<String>,
    pub loader_version: Option<Option<String>>,
    pub java_path: Option<Option<String>>,
    pub jvm_args: Option<Option<String>>,
    pub resolution_w: Option<Option<i64>>,
    pub resolution_h: Option<Option<i64>>,
    pub fullscreen: Option<bool>,
    pub game_dir: Option<String>,
    pub favorite: Option<bool>,
    pub notes: Option<Option<String>>,
    pub last_played: Option<String>,
    pub launch_count: Option<i64>,
    pub mod_count: Option<i64>,
    pub disk_usage: Option<i64>,
    pub ram_mb: Option<Option<i64>>,
    pub instance_group: Option<Option<String>>,
    pub auto_optimize: Option<bool>,
    pub use_vulkan: Option<bool>,
    pub use_gamemode: Option<bool>,
    pub use_mangohud: Option<bool>,
    pub force_dedicated_gpu: Option<bool>,
    pub use_gamescope: Option<bool>,
    pub gamescope_width: Option<Option<i64>>,
    pub gamescope_height: Option<Option<i64>>,
    pub gamescope_fsr: Option<bool>,
    pub force_full_verification: Option<bool>,
    #[serde(default)]
    pub pre_launch_hook: Option<Option<String>>,
    #[serde(default)]
    pub post_exit_hook: Option<Option<String>>,
}

#[tauri::command]
pub async fn profiles_list() -> AppResult<Vec<ProfileRow>> {
    let db = crate::db::shared_db().await?;
    crate::db::schema::profiles::list(&db).await
}

#[tauri::command]
pub async fn profiles_get(id: String) -> AppResult<ProfileRow> {
    let db = crate::db::shared_db().await?;
    let row = sqlx::query_as::<_, ProfileRow>("SELECT * FROM profiles WHERE id = ?")
        .bind(&id)
        .fetch_optional(db.pool())
        .await?
        .ok_or_else(|| crate::error::AppError::NotFound(format!("profile {id} not found")))?;
    Ok(row)
}

#[tauri::command]
pub async fn profiles_create(
    input: ProfileCreate,
) -> AppResult<ProfileRow> {
    let db = crate::db::shared_db().await?;
    let now = Utc::now();
    let id = Uuid::new_v4().to_string();
    let game_dir = match input.game_dir {
        Some(value) => {
            let trimmed = value.trim().to_string();
            validate_game_dir(&trimmed)?;
            trimmed
        }
        None => {
            let base = directories::ProjectDirs::from("io", "github", "Luxmc")
                .map(|d| d.data_dir().to_string_lossy().to_string())
                .unwrap_or_else(|| {
                    let home = std::env::var("HOME").unwrap_or_else(|_| "/tmp".into());
                    format!("{}/.local/share/luxmc", home)
                });
            format!("{}/instances/{}/.minecraft", base, id)
        }
    };
    let mut row = ProfileRow {
        id,
        name: input.name,
        icon: input.icon.unwrap_or_else(|| "grass_block".into()),
        mc_version: input.mc_version.trim().trim_matches('\'').trim_matches('"').to_string(),
        loader: input.loader,
        loader_version: input.loader_version,
        java_path: input.java_path,
        jvm_args: input.jvm_args,
        resolution_w: input.resolution_w,
        resolution_h: input.resolution_h,
        fullscreen: input.fullscreen.unwrap_or(false),
        game_dir,
        created_at: now,
        updated_at: now,
        favorite: input.favorite.unwrap_or(false),
        notes: input.notes,
        last_played: None,
        launch_count: 0,
        mod_count: 0,
        disk_usage: 0,
        ram_mb: input.ram_mb,
        instance_group: input.instance_group,
        auto_optimize: input.auto_optimize.unwrap_or(true),
        use_vulkan: input.use_vulkan.unwrap_or(false),
        use_gamemode: input.use_gamemode.unwrap_or(cfg!(target_os = "linux")),
        use_mangohud: input.use_mangohud.unwrap_or(false),
        force_dedicated_gpu: input.force_dedicated_gpu.unwrap_or(false),
        use_gamescope: input.use_gamescope.unwrap_or(false),
        gamescope_width: input.gamescope_width.unwrap_or(None),
        gamescope_height: input.gamescope_height.unwrap_or(None),
        gamescope_fsr: input.gamescope_fsr.unwrap_or(false),
        force_full_verification: input.force_full_verification.unwrap_or(false),
        pre_launch_hook: input.pre_launch_hook.flatten().filter(|v| !v.trim().is_empty()),
        post_exit_hook: input.post_exit_hook.flatten().filter(|v| !v.trim().is_empty()),
    };
    crate::db::schema::profiles::upsert(&db, &row).await?;
    let base = directories::ProjectDirs::from("io", "github", "Luxmc").ok_or_else(|| crate::error::AppError::InvalidState("Diretório de dados indisponível".into()))?;
    crate::core::instance_paths::isolate(&mut row, base.data_dir()).await?;
    Ok(row)
}

#[tauri::command]
pub async fn profiles_update(
    input: ProfileUpdate,
) -> AppResult<ProfileRow> {
    let db = crate::db::shared_db().await?;
    let existing = sqlx::query_as::<_, ProfileRow>("SELECT * FROM profiles WHERE id = ?")
        .bind(&input.id)
        .fetch_optional(db.pool())
        .await?
        .ok_or_else(|| {
            crate::error::AppError::NotFound(format!("profile {} not found", input.id))
        })?;

    let game_dir = match input.game_dir {
        Some(value) => {
            let trimmed = value.trim().to_string();
            validate_game_dir(&trimmed)?;
            trimmed
        }
        None => existing.game_dir.clone(),
    };

    let now = Utc::now();
    let row = ProfileRow {
        id: existing.id,
        name: input.name.unwrap_or(existing.name),
        icon: input.icon.unwrap_or(existing.icon),
        mc_version: input.mc_version.map(|v| v.trim().trim_matches('\'').trim_matches('"').to_string()).unwrap_or(existing.mc_version),
        loader: input.loader.unwrap_or(existing.loader),
        loader_version: input.loader_version.unwrap_or(existing.loader_version),
        java_path: input.java_path.unwrap_or(existing.java_path),
        jvm_args: input.jvm_args.unwrap_or(existing.jvm_args),
        resolution_w: input.resolution_w.unwrap_or(existing.resolution_w),
        resolution_h: input.resolution_h.unwrap_or(existing.resolution_h),
        fullscreen: input.fullscreen.unwrap_or(existing.fullscreen),
        game_dir,
        created_at: existing.created_at,
        updated_at: now,
        favorite: input.favorite.unwrap_or(existing.favorite),
        notes: input.notes.unwrap_or(existing.notes),
        last_played: input
            .last_played
            .and_then(|s| s.parse().ok())
            .or(existing.last_played),
        launch_count: input.launch_count.unwrap_or(existing.launch_count),
        mod_count: input.mod_count.unwrap_or(existing.mod_count),
        disk_usage: input.disk_usage.unwrap_or(existing.disk_usage),
        ram_mb: input.ram_mb.unwrap_or(existing.ram_mb),
        instance_group: input.instance_group.unwrap_or(existing.instance_group),
        auto_optimize: input.auto_optimize.unwrap_or(existing.auto_optimize),
        use_vulkan: input.use_vulkan.unwrap_or(existing.use_vulkan),
        use_gamemode: input.use_gamemode.unwrap_or(existing.use_gamemode),
        use_mangohud: input.use_mangohud.unwrap_or(existing.use_mangohud),
        force_dedicated_gpu: input.force_dedicated_gpu.unwrap_or(existing.force_dedicated_gpu),
        use_gamescope: input.use_gamescope.unwrap_or(existing.use_gamescope),
        gamescope_width: input.gamescope_width.unwrap_or(existing.gamescope_width),
        gamescope_height: input.gamescope_height.unwrap_or(existing.gamescope_height),
        gamescope_fsr: input.gamescope_fsr.unwrap_or(existing.gamescope_fsr),
        force_full_verification: input.force_full_verification.unwrap_or(existing.force_full_verification),
        pre_launch_hook: input.pre_launch_hook.unwrap_or(existing.pre_launch_hook).filter(|v| !v.trim().is_empty()),
        post_exit_hook: input.post_exit_hook.unwrap_or(existing.post_exit_hook).filter(|v| !v.trim().is_empty()),
    };
    crate::db::schema::profiles::upsert(&db, &row).await?;
    Ok(row)
}

#[tauri::command]
pub async fn profiles_delete(id: String) -> AppResult<()> {
    if id.is_empty()
        || id.len() > 128
        || !id.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'-' || b == b'_')
    {
        return Err(AppError::InvalidInput("ID de instância inválido".into()));
    }
    let db = crate::db::shared_db().await?;
    if let Ok(Some(row)) = sqlx::query_as::<_, ProfileRow>("SELECT * FROM profiles WHERE id = ?")
        .bind(&id)
        .fetch_optional(db.pool())
        .await
    {
        if let Some(base_dir) = directories::ProjectDirs::from("io", "github", "Luxmc") {
            let path = std::path::PathBuf::from(&row.game_dir);
            let managed = path
                .canonicalize()
                .map(|canonical| canonical.starts_with(base_dir.data_dir()))
                .unwrap_or(false);
            if managed && path.is_dir() {
                let _ = tokio::fs::remove_dir_all(&path).await;
            } else if !managed && path.is_dir() {
                tracing::warn!(
                    target: "profiles",
                    "Diretório de jogo fora do diretório do launcher; remoção ignorada: {}",
                    path.display()
                );
            }

            let storage_mods_dir = base_dir.data_dir().join("mods").join(&id);
            if storage_mods_dir.is_dir() {
                let _ = tokio::fs::remove_dir_all(&storage_mods_dir).await;
            }

            let instance_dir_by_id = base_dir.data_dir().join("instances").join(&id);
            if instance_dir_by_id.is_dir() {
                let _ = tokio::fs::remove_dir_all(&instance_dir_by_id).await;
            }

            if let Ok(safe_name_dir) = crate::core::instance_paths::resolve_within(
                &base_dir.data_dir().join("instances"),
                &row.name,
            ) {
                if safe_name_dir.is_dir() {
                    let _ = tokio::fs::remove_dir_all(&safe_name_dir).await;
                }
            }
        }
    }
    crate::db::schema::profiles::delete(&db, &id).await
}
