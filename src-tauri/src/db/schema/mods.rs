use chrono::Utc;
use serde::{Deserialize, Serialize};

use crate::db::Db;
use crate::error::AppResult;

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct ModRow {
    pub profile_id: String,
    pub project_id: String,
    pub version_id: String,
    pub file_name: String,
    pub sha1: String,
    pub source: String,
    pub installed_at: String,
}

pub async fn list_by_profile(db: &Db, profile_id: &str) -> AppResult<Vec<ModRow>> {
    let rows = sqlx::query_as::<_, ModRow>(
        "SELECT * FROM mods WHERE profile_id = ? ORDER BY installed_at DESC",
    )
    .bind(profile_id)
    .fetch_all(db.pool())
    .await?;
    Ok(rows)
}

pub async fn upsert(db: &Db, m: &ModRow) -> AppResult<()> {
    let now = Utc::now().to_rfc3339();
    sqlx::query(
		"INSERT INTO mods (profile_id, project_id, version_id, file_name, sha1, source, installed_at)
		 VALUES (?, ?, ?, ?, ?, ?, ?)
		 ON CONFLICT(profile_id, project_id) DO UPDATE SET
			version_id = excluded.version_id,
			file_name = excluded.file_name,
			sha1 = excluded.sha1,
			source = excluded.source,
			installed_at = excluded.installed_at",
	)
	.bind(&m.profile_id)
	.bind(&m.project_id)
	.bind(&m.version_id)
	.bind(&m.file_name)
	.bind(&m.sha1)
	.bind(&m.source)
	.bind(now)
	.execute(db.pool())
	.await?;
    Ok(())
}

pub async fn delete(db: &Db, profile_id: &str, project_id: &str) -> AppResult<()> {
    sqlx::query("DELETE FROM mods WHERE profile_id = ? AND project_id = ?")
        .bind(profile_id)
        .bind(project_id)
        .execute(db.pool())
        .await?;
    Ok(())
}

pub async fn delete_by_file_name(db: &Db, profile_id: &str, file_name: &str) -> AppResult<()> {
    let canonical = file_name.trim_end_matches(".disabled");
    sqlx::query("DELETE FROM mods WHERE profile_id = ? AND (file_name = ? OR file_name = ?)")
        .bind(profile_id)
        .bind(canonical)
        .bind(file_name)
        .execute(db.pool())
        .await?;
    Ok(())
}

fn canonical_mod_file(name: &str) -> Option<String> {
    let lower = name.to_lowercase();
    if lower.ends_with(".jar.disabled") {
        Some(name[..name.len() - ".disabled".len()].to_string())
    } else if lower.ends_with(".jar") {
        Some(name.to_string())
    } else {
        None
    }
}

pub async fn reconcile_profile(
    db: &Db,
    profile_id: &str,
    mods_dir: &std::path::Path,
) -> AppResult<Vec<ModRow>> {
    let scan_dir = mods_dir.to_path_buf();
    let present = tokio::task::spawn_blocking(move || -> std::collections::BTreeSet<String> {
        let mut present: std::collections::BTreeSet<String> = std::collections::BTreeSet::new();
        if let Ok(entries) = std::fs::read_dir(scan_dir) {
            for entry in entries.flatten() {
                if !entry.path().is_file() {
                    continue;
                }
                if let Some(canonical) = canonical_mod_file(&entry.file_name().to_string_lossy()) {
                    present.insert(canonical);
                }
            }
        }
        present
    })
    .await
    .map_err(|error| crate::error::AppError::Internal(error.to_string()))?;

    let rows = list_by_profile(db, profile_id).await?;
    let mut tracked: std::collections::BTreeSet<String> = std::collections::BTreeSet::new();

    for row in rows {
        if present.contains(&row.file_name) {
            tracked.insert(row.file_name);
        } else {
            delete(db, profile_id, &row.project_id).await?;
        }
    }

    for file in &present {
        if tracked.contains(file) {
            continue;
        }
        let stem = file.strip_suffix(".jar").unwrap_or(file.as_str());
        upsert(
            db,
            &ModRow {
                profile_id: profile_id.to_string(),
                project_id: format!("local:{stem}"),
                version_id: String::new(),
                file_name: file.clone(),
                sha1: String::new(),
                source: "local".into(),
                installed_at: Utc::now().to_rfc3339(),
            },
        )
        .await?;
        tracked.insert(file.clone());
    }

    let count = tracked.len() as i64;
    sqlx::query("UPDATE profiles SET mod_count = ? WHERE id = ?")
        .bind(count)
        .bind(profile_id)
        .execute(db.pool())
        .await?;

    list_by_profile(db, profile_id).await
}
