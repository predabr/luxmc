use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::path::{Path, PathBuf};

use crate::db::models::ProfileRow;
use crate::error::AppResult;

#[derive(Serialize, Deserialize)]
struct LaunchState {
    profile_id: String,
    mc_version: String,
    loader: String,
    loader_version: Option<String>,
    fingerprint: String,
}

fn state_path(game_dir: &Path) -> PathBuf {
    game_dir.join(".luxmc").join("instance_launch_state.json")
}

fn append_path(hasher: &mut Sha256, root: &Path, path: &Path) {
    let Ok(relative) = path.strip_prefix(root) else { return };
    hasher.update(relative.to_string_lossy().as_bytes());
    let Ok(metadata) = std::fs::metadata(path) else { return };
    hasher.update(metadata.len().to_le_bytes());
    if let Ok(modified) = metadata.modified() {
        if let Ok(duration) = modified.duration_since(std::time::UNIX_EPOCH) {
            hasher.update(duration.as_nanos().to_le_bytes());
        }
    }
}

fn fingerprint_tree(root: &Path, current: &Path, hasher: &mut Sha256) {
    let Ok(entries) = std::fs::read_dir(current) else { return };
    let mut entries = entries.flatten().collect::<Vec<_>>();
    entries.sort_by_key(|entry| entry.file_name());
    for entry in entries {
        let path = entry.path();
        let Ok(kind) = entry.file_type() else { continue; };
        if kind.is_dir() {
            append_path(hasher, root, &path);
            fingerprint_tree(root, &path, hasher);
        } else if kind.is_file() {
            append_path(hasher, root, &path);
        }
    }
}

fn fingerprint(profile: &ProfileRow) -> String {
    let game_dir = Path::new(&profile.game_dir);
    let mut hasher = Sha256::new();
    hasher.update(profile.id.as_bytes());
    hasher.update(profile.mc_version.as_bytes());
    hasher.update(profile.loader.as_bytes());
    hasher.update(profile.loader_version.as_deref().unwrap_or_default().as_bytes());
    for name in ["mods", "kubejs", "modrinth.index.json", "manifest.json"] {
        let path = game_dir.join(name);
        if path.is_dir() {
            fingerprint_tree(game_dir, &path, &mut hasher);
        } else if path.is_file() {
            append_path(&mut hasher, game_dir, &path);
        }
    }
    format!("{:x}", hasher.finalize())
}

pub fn is_valid(profile: &ProfileRow) -> bool {
    if profile.force_full_verification {
        return false;
    }
    let path = state_path(Path::new(&profile.game_dir));
    let Ok(content) = std::fs::read_to_string(path) else { return false };
    let Ok(state) = serde_json::from_str::<LaunchState>(&content) else { return false };
    state.profile_id == profile.id
        && state.mc_version == profile.mc_version
        && state.loader == profile.loader
        && state.loader_version == profile.loader_version
        && state.fingerprint == fingerprint(profile)
}

pub fn store(profile: &ProfileRow) -> AppResult<()> {
    let path = state_path(Path::new(&profile.game_dir));
    let parent = path.parent().ok_or_else(|| crate::error::AppError::InvalidState("estado de lançamento sem diretório".into()))?;
    std::fs::create_dir_all(parent)?;
    let state = LaunchState {
        profile_id: profile.id.clone(),
        mc_version: profile.mc_version.clone(),
        loader: profile.loader.clone(),
        loader_version: profile.loader_version.clone(),
        fingerprint: fingerprint(profile),
    };
    let temporary = path.with_extension("json.part");
    std::fs::write(&temporary, serde_json::to_vec(&state)?)?;
    std::fs::rename(temporary, path)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn game_preferences_do_not_invalidate_verified_mod_files() {
        let root = std::env::temp_dir().join(format!("luxmc-launch-state-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(root.join("mods")).unwrap();
        let profile: ProfileRow = serde_json::from_value(serde_json::json!({"id":"fixture","name":"Fixture","icon":"grass","mcVersion":"26.3","loader":"fabric","fullscreen":false,"gameDir":root.to_string_lossy(),"createdAt":"2026-10-03T00:00:00Z","updatedAt":"2026-10-03T00:00:00Z"})).unwrap();
        std::fs::write(root.join("mods/library.jar"), b"mod").unwrap();
        let original = fingerprint(&profile);
        std::fs::write(root.join("options.txt"), b"fullscreen:true").unwrap();
        std::fs::create_dir(root.join("config")).unwrap();
        std::fs::write(root.join("config/mod.json"), b"{}").unwrap();
        assert_eq!(original, fingerprint(&profile));
        std::fs::write(root.join("mods/library.jar"), b"changed mod").unwrap();
        assert_ne!(original, fingerprint(&profile));
        for path in ["mods/library.jar", "options.txt", "config/mod.json"] { std::fs::remove_file(root.join(path)).unwrap(); }
        std::fs::remove_dir(root.join("mods")).unwrap();
        std::fs::remove_dir(root.join("config")).unwrap();
        std::fs::remove_dir(root).unwrap();
    }
}
