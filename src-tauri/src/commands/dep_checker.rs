use serde::{Deserialize, Serialize};
use tauri::State;

use crate::core::mods::ModrinthClient;
use crate::error::AppResult;
use crate::state::AppState;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MissingDep {
    pub slug: String,
    pub name: String,
    pub project_id: String,
    pub reason: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DepCheckResult {
    pub missing: Vec<MissingDep>,
    pub all_ok: bool,
}

static ESSENTIAL_DEPS: &[(&str, &str, &str)] = &[
    ("fabric-api", "Fabric API", "P7dR8mSH"),
    ("cloth-config", "Cloth Config API", "9s6osm5g"),
    ("architectury-api", "Architectury API", "lhGA9TYQ"),
    ("forge-config-api-port", "Forge Config API Port", "ohNO6lps"),
    ("modmenu", "Mod Menu", "mOgUt4GM"),
    ("kotlin-for-forge", "Kotlin for Forge", "ordsPcFz"),
];

#[tauri::command]
#[allow(non_snake_case)]
pub async fn mods_check_missing_deps(
    state: State<'_, AppState>,
    profileId: String,
    mcVersion: String,
    loader: String,
) -> AppResult<DepCheckResult> {
    let db = crate::db::shared_db().await?;
    let installed_rows = crate::db::schema::mods::list_by_profile(&db, &profileId).await?;

    let installed_names: std::collections::HashSet<String> = installed_rows
        .iter()
        .map(|r| r.file_name.to_lowercase().replace(['-', '_', ' '], ""))
        .collect();

    let installed_ids: std::collections::HashSet<String> = installed_rows
        .iter()
        .map(|r| r.project_id.clone())
        .collect();

    let loader_lower = loader.to_lowercase();
    let relevant_deps: Vec<(&str, &str, &str)> = ESSENTIAL_DEPS
        .iter()
        .filter(|(slug, _, _)| {
            match *slug {
                "fabric-api" | "modmenu" => loader_lower == "fabric" || loader_lower == "quilt",
                "forge-config-api-port" => loader_lower == "fabric" || loader_lower == "neoforge",
                "kotlin-for-forge" => loader_lower == "forge" || loader_lower == "neoforge",
                "cloth-config" | "architectury-api" => true,
                _ => true,
            }
        })
        .copied()
        .collect();

    let mods_dir = {
        let base = directories::ProjectDirs::from("io", "github", "Luxmc")
            .ok_or_else(|| crate::error::AppError::InvalidState("no data dir".into()))?;
        let row = sqlx::query_as::<_, crate::db::models::ProfileRow>(
            "SELECT * FROM profiles WHERE id = ?",
        )
        .bind(&profileId)
        .fetch_optional(db.pool())
        .await?;
        row.map(|r| std::path::PathBuf::from(&r.game_dir).join("mods"))
            .unwrap_or_else(|| base.data_dir().join("instances").join(&profileId).join("mods"))
    };

    let mut jar_names: std::collections::HashSet<String> = std::collections::HashSet::new();
    if let Ok(mut entries) = tokio::fs::read_dir(&mods_dir).await {
        while let Ok(Some(e)) = entries.next_entry().await {
            let n = e.file_name().to_string_lossy().to_lowercase().replace(['-', '_', ' '], "");
            jar_names.insert(n);
        }
    }

    let modrinth = ModrinthClient::new(state.http.clone());
    let mut missing = Vec::new();

    for (slug, name, project_id) in relevant_deps {
        let norm_slug = slug.replace(['-', '_'], "");
        let norm_name = name.to_lowercase().replace(['-', '_', ' '], "");

        let is_installed = installed_ids.contains(project_id)
            || installed_names.iter().any(|n| n.contains(&norm_slug) || n.contains(&norm_name))
            || jar_names.iter().any(|n| n.contains(&norm_slug) || n.contains(&norm_name));

        if is_installed {
            continue;
        }

        let versions = modrinth
            .get_mod_versions(project_id, &mcVersion)
            .await
            .unwrap_or_default();

        if versions.is_empty() {
            continue;
        }

        missing.push(MissingDep {
            slug: slug.to_string(),
            name: name.to_string(),
            project_id: project_id.to_string(),
            reason: format!("Required by mods in this modpack but not installed"),
        });
    }

    let all_ok = missing.is_empty();
    Ok(DepCheckResult { missing, all_ok })
}

#[tauri::command]
#[allow(non_snake_case)]
pub async fn mods_install_missing_deps(
    state: State<'_, AppState>,
    profileId: String,
    mcVersion: String,
    _loader: String,
    projectIds: Vec<String>,
) -> AppResult<u32> {
    let db = crate::db::shared_db().await?;
    let row = sqlx::query_as::<_, crate::db::models::ProfileRow>(
        "SELECT * FROM profiles WHERE id = ?",
    )
    .bind(&profileId)
    .fetch_optional(db.pool())
    .await?
    .ok_or_else(|| crate::error::AppError::NotFound(format!("profile {profileId} not found")))?;

    let mods_dir = std::path::PathBuf::from(&row.game_dir).join("mods");
    tokio::fs::create_dir_all(&mods_dir).await?;

    let modrinth = ModrinthClient::new(state.http.clone());
    let mut installed = 0u32;

    for project_id in &projectIds {
        let versions = modrinth
            .get_mod_versions(project_id, &mcVersion)
            .await
            .unwrap_or_default();

        let best = versions.first();
        if let Some(ver) = best {
            if let Some(file) = ver.files.first() {
                let url = &file.url;
                let filename = &file.filename;
                let dest = mods_dir.join(filename);
                if dest.exists() {
                    installed += 1;
                    continue;
                }
                if let Ok(resp) = state.http.get(url).send().await {
                    if resp.status().is_success() {
                        if let Ok(bytes) = resp.bytes().await {
                            if tokio::fs::write(&dest, &bytes).await.is_ok() {
                                let mod_row = crate::db::schema::mods::ModRow {
                                    profile_id: profileId.clone(),
                                    project_id: project_id.clone(),
                                    version_id: ver.id.clone(),
                                    file_name: filename.clone(),
                                    sha1: file.sha1.clone(),
                                    source: "modrinth".into(),
                                    installed_at: chrono::Utc::now().to_rfc3339(),
                                };
                                let _ = crate::db::schema::mods::upsert(&db, &mod_row).await;
                                installed += 1;
                            }
                        }
                    }
                }
            }
        }
    }

    Ok(installed)
}
