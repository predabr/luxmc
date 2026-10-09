use crate::{db::models::ProfileRow, error::{AppError, AppResult}};
use std::{path::{Path, PathBuf}, sync::LazyLock};
use tokio::sync::Mutex;
static MIGRATION: LazyLock<Mutex<()>> = LazyLock::new(|| Mutex::new(()));

pub async fn migration_guard() -> tokio::sync::MutexGuard<'static, ()> {
    MIGRATION.lock().await
}

pub fn game_dir(base: &Path, id: &str) -> AppResult<PathBuf> {
    if id.is_empty() || id.len() > 128 || !id.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'-' || b == b'_') {
        return Err(AppError::InvalidInput("ID de instância inválido".into()));
    }
    Ok(base.join("instances").join(id).join(".minecraft"))
}

pub fn resolve_within(base: &Path, relative: &str) -> AppResult<PathBuf> {
    let rel_path = Path::new(relative);
    for component in rel_path.components() {
        if matches!(
            component,
            std::path::Component::ParentDir
                | std::path::Component::RootDir
                | std::path::Component::Prefix(_)
        ) {
            return Err(AppError::InvalidInput("Path traversal detected".into()));
        }
    }
    let target = base.join(relative);
    if let Ok(canonical_base) = base.canonicalize() {
        if let Ok(canonical_target) = target.canonicalize() {
            if !canonical_target.starts_with(&canonical_base) {
                return Err(AppError::InvalidInput("Path escapes base directory".into()));
            }
        }
    }
    Ok(target)
}

pub async fn remove_instance_data(base: &Path, id: &str, game: &Path, other_games: &[PathBuf], active_game: Option<&Path>) -> AppResult<()> {
    let owned = game_dir(base, id)?.parent().unwrap().to_path_buf();
    let canonical_base = tokio::fs::canonicalize(base).await?;
    let mut candidates = vec![owned, base.join("mods").join(id), game.to_path_buf()];
    let mut checked = Vec::new();
    for path in candidates.drain(..) {
        let canonical = match tokio::fs::canonicalize(&path).await {
            Ok(path) => path,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => continue,
            Err(error) => return Err(error.into()),
        };
        if canonical == canonical_base || !canonical.starts_with(&canonical_base) {
            return Err(AppError::InvalidInput("A pasta está fora dos dados do Luxmc. A instância não foi removida; seus documentos e o projeto foram preservados.".into()));
        }
        for other in other_games {
            if let Ok(other) = tokio::fs::canonicalize(other).await {
                if other.starts_with(&canonical) || canonical.starts_with(&other) {
                    return Err(AppError::InvalidState("Outra instância usa essa pasta. Separe os arquivos antes de excluir.".into()));
                }
            }
        }
        if let Some(active) = active_game {
            if let Ok(active) = tokio::fs::canonicalize(active).await {
                if active.starts_with(&canonical) || canonical.starts_with(&active) {
                    return Err(AppError::InvalidState("Feche o Minecraft desta instância antes de excluir seus arquivos.".into()));
                }
            }
        }
        if !checked.iter().any(|parent: &PathBuf| canonical.starts_with(parent)) {
            checked.retain(|child: &PathBuf| !child.starts_with(&canonical));
            checked.push(canonical);
        }
    }
    for path in checked {
        tokio::fs::remove_dir_all(&path).await.map_err(|error| AppError::InvalidState(format!("Não foi possível apagar os arquivos da instância: {error}. Feche o jogo e tente novamente.")))?;
        if tokio::fs::try_exists(&path).await? { return Err(AppError::InvalidState("A pasta da instância ainda existe. A remoção não foi concluída.".into())); }
    }
    Ok(())
}

fn copy_tree(source: &Path, target: &Path) -> std::io::Result<()> {
    std::fs::create_dir_all(target)?;
    for entry in std::fs::read_dir(source)? {
        let entry = entry?;
        let from = entry.path();
        let to = target.join(entry.file_name());
        let metadata = std::fs::metadata(&from)?;
        if metadata.is_dir() {
            if entry.file_type()?.is_symlink() { return Err(std::io::Error::other("Diretório simbólico na instância; migração cancelada sem alterar a origem")); }
            copy_tree(&from, &to)?;
        } else if metadata.is_file() { std::fs::copy(from, to)?; }
    }
    Ok(())
}

pub async fn isolate(profile: &mut ProfileRow, base: &Path) -> AppResult<()> {
    let _guard = MIGRATION.lock().await;
    let target = game_dir(base, &profile.id)?;
    let source = PathBuf::from(&profile.game_dir);
    if source == target { tokio::fs::create_dir_all(&target).await?; return Ok(()); }
    if target.exists() { return Err(AppError::InvalidState("Já existe uma pasta de destino; migração interrompida para preservar ambas as instâncias".into())); }
    let staging = base.join("migration-staging").join(uuid::Uuid::new_v4().to_string());
    if source.exists() {
        let source = tokio::fs::canonicalize(&source).await?;
        let base = tokio::fs::canonicalize(base).await?;
        if base.starts_with(&source) { return Err(AppError::InvalidInput("A pasta do jogo não pode conter todos os dados do launcher".into())); }
        let destination = staging.clone();
        let result = tokio::task::spawn_blocking(move || copy_tree(&source, &destination)).await.map_err(|e| AppError::Internal(e.to_string()))?;
        if let Err(error) = result { let _ = tokio::fs::remove_dir_all(&staging).await; return Err(error.into()); }
    } else { tokio::fs::create_dir_all(&staging).await?; }
    tokio::fs::create_dir_all(target.parent().unwrap()).await?;
    tokio::fs::rename(&staging, &target).await?;
    let db = crate::db::shared_db().await?;
    let value = target.to_string_lossy().to_string();
    if let Err(error) = sqlx::query("UPDATE profiles SET game_dir = ? WHERE id = ?").bind(&value).bind(&profile.id).execute(db.pool()).await {
        let _ = tokio::fs::remove_dir_all(&target).await;
        return Err(error.into());
    }
    profile.game_dir = value;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn paths_are_unique_and_cannot_escape_instances() {
        let base = Path::new("/launcher");
        assert_eq!(game_dir(base, "one").unwrap(), base.join("instances/one/.minecraft"));
        assert_ne!(game_dir(base, "one").unwrap(), game_dir(base, "two").unwrap());
        assert!(game_dir(base, "../escape").is_err());
        assert!(game_dir(base, "").is_err());
    }
    #[test]
    fn copied_options_are_independent() {
        let root = std::env::temp_dir().join(uuid::Uuid::new_v4().to_string());
        let source = root.join("old"); let dest = root.join("new");
        std::fs::create_dir_all(source.join("config")).unwrap();
        std::fs::write(source.join("options.txt"), "old").unwrap();
        copy_tree(&source, &dest).unwrap();
        std::fs::write(dest.join("options.txt"), "new").unwrap();
        assert_eq!(std::fs::read_to_string(source.join("options.txt")).unwrap(), "old");
        std::fs::remove_dir_all(root).unwrap();
    }
    #[test]
    fn resolve_within_rejects_traversal_attempts() {
        let base = std::env::temp_dir();
        assert!(resolve_within(&base, "../etc/passwd").is_err());
        assert!(resolve_within(&base, "/etc/passwd").is_err());
        assert!(resolve_within(&base, "foo/../../bar").is_err());
        assert!(resolve_within(&base, "safe/file.txt").is_ok());
    }
    #[tokio::test]
    async fn deletion_removes_owned_files_and_preserves_other_instances() {
        let root = tempfile::tempdir().unwrap();
        let game = game_dir(root.path(), "one").unwrap();
        let other = game_dir(root.path(), "two").unwrap();
        std::fs::create_dir_all(game.join("saves/test")).unwrap();
        std::fs::create_dir_all(&other).unwrap();
        std::fs::write(game.join("saves/test/level.dat"), "world").unwrap();
        std::fs::write(other.join("options.txt"), "keep").unwrap();
        remove_instance_data(root.path(), "one", &game, &[other.clone()], None).await.unwrap();
        assert!(!game.parent().unwrap().exists());
        assert_eq!(std::fs::read_to_string(other.join("options.txt")).unwrap(), "keep");
    }
    #[tokio::test]
    async fn deletion_rejects_external_shared_and_running_game_directories() {
        let root = tempfile::tempdir().unwrap();
        let external = tempfile::tempdir().unwrap();
        let game = game_dir(root.path(), "one").unwrap();
        std::fs::create_dir_all(&game).unwrap();
        assert!(remove_instance_data(root.path(), "one", external.path(), &[], None).await.is_err());
        assert!(game.exists());
        assert!(remove_instance_data(root.path(), "one", &game, &[game.clone()], None).await.is_err());
        assert!(remove_instance_data(root.path(), "one", &game, &[], Some(&game)).await.is_err());
        assert!(remove_instance_data(root.path(), "one", root.path(), &[], None).await.is_err());
        assert!(game.exists());
    }
}
