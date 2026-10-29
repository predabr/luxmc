use crate::error::{AppError, AppResult};
use sha2::Digest;
use std::path::{Path, PathBuf};

pub const AGENT: &[u8] = include_bytes!("../../assets/luxmc-client-agent.jar");
pub const FILE: &str = "luxmc-client-agent.jar";
const MANIFEST: &str = ".luxmc/managed-components.json";

pub async fn prepare(root: &Path, server: bool) -> AppResult<PathBuf> {
    tokio::fs::create_dir_all(root).await?;
    let path = crate::core::mods::pack_download::destination(root, FILE)?;
    if tokio::fs::read(&path).await.ok().as_deref() != Some(AGENT) {
        crate::core::mods::pack_download::atomic_write(&path, AGENT).await?;
    }
    let manifest = crate::core::mods::pack_download::destination(root, MANIFEST)?;
    let text = serde_json::to_vec_pretty(
        &serde_json::json!({"schema":1,"name":"Luxmc Skin Bridge","role":if server{"server-client-support"}else{"client"},"required":true,"file":FILE,"sha256":format!("{:x}",sha2::Sha256::digest(AGENT)),"clientComponentRequired":true}),
    )?;
    if tokio::fs::read(&manifest).await.ok().as_deref() != Some(text.as_slice()) {
        crate::core::mods::pack_download::atomic_write(&manifest, &text).await?;
    }
    Ok(path)
}

pub fn is_managed(path: &Path) -> bool {
    let reserved = path.components().any(|component| {
        matches!(
            component
                .as_os_str()
                .to_string_lossy()
                .to_ascii_lowercase()
                .as_str(),
            "luxmc-client-agent.jar"
                | "luxmcessentials"
                | "luxmccustomskin"
                | ".luxmc"
                | "resourcepacks"
        )
    });
    if !reserved {
        return false;
    }
    for root in path.ancestors().skip(1) {
        if !root.join(MANIFEST).is_file() {
            continue;
        }
        let Ok(relative) = path.strip_prefix(root) else {
            continue;
        };
        let relative = relative
            .to_string_lossy()
            .replace('\\', "/")
            .to_ascii_lowercase();
        return relative == FILE
            || relative == ".luxmc"
            || relative == ".luxmc/managed-components.json"
            || relative == ".luxmc/essentials-installed"
            || relative == ".luxmc/appearance-fingerprint"
            || (root.join(".luxmc/appearance-fingerprint").is_file()
                && (relative == "resourcepacks/luxmccustomskin"
                    || relative.starts_with("resourcepacks/luxmccustomskin/")))
            || (root
                .join("resourcepacks/LuxmcEssentials/.luxmc-owned")
                .is_file()
                && (relative == "resourcepacks/luxmcessentials"
                    || relative.starts_with("resourcepacks/luxmcessentials/")))
            || (relative == "resourcepacks"
                && (root
                    .join("resourcepacks/LuxmcEssentials/.luxmc-owned")
                    .is_file()
                    || root.join(".luxmc/appearance-fingerprint").is_file()));
    }
    false
}

pub fn guard_change(path: &Path) -> AppResult<()> {
    if is_managed(path) {
        return Err(AppError::InvalidInput("Componente obrigatório do Luxmc. Ele é gerenciado e reparado automaticamente; exclua a instância ou servidor inteiro se quiser removê-lo.".into()));
    }
    Ok(())
}

pub async fn status(root: &Path) -> serde_json::Value {
    let valid = tokio::fs::read(root.join(FILE))
        .await
        .is_ok_and(|bytes| bytes == AGENT);
    serde_json::json!({"name":"Luxmc Skin Bridge","required":true,"valid":valid,"clientComponentRequired":true})
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn repairs_missing_and_changed_components_and_protects_only_the_managed_assets() {
        let root = tempfile::tempdir().unwrap();
        let path = prepare(root.path(), false).await.unwrap();
        assert!(status(root.path()).await["valid"].as_bool().unwrap());
        assert!(guard_change(&path).is_err());
        tokio::fs::write(&path, b"changed").await.unwrap();
        prepare(root.path(), false).await.unwrap();
        assert_eq!(tokio::fs::read(&path).await.unwrap(), AGENT);
        tokio::fs::remove_file(&path).await.unwrap();
        prepare(root.path(), true).await.unwrap();
        assert_eq!(tokio::fs::read(&path).await.unwrap(), AGENT);
        assert!(guard_change(&root.path().join(MANIFEST)).is_err());
        assert!(guard_change(root.path()).is_ok());
        assert!(guard_change(&root.path().join("mods/personal.jar")).is_ok());
        assert!(guard_change(&root.path().join(".luxmc/removed-duplicates/personal.jar")).is_ok());
        let own = root.path().join("resourcepacks/LuxmcEssentials");
        tokio::fs::create_dir_all(&own).await.unwrap();
        assert!(guard_change(&own).is_ok());
        tokio::fs::write(own.join(".luxmc-owned"), b"1")
            .await
            .unwrap();
        assert!(guard_change(&own).is_err());
        assert!(guard_change(&root.path().join("resourcepacks")).is_err());
        assert!(guard_change(&root.path().join("resourcepacks/personal.zip")).is_ok());
    }
}
