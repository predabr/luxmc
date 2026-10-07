use std::collections::BTreeSet;
use std::path::Path;

use crate::error::{AppError, AppResult};

fn parse_loader_id(raw_id: &str) -> Option<(String, String)> {
    let lower = raw_id.to_ascii_lowercase();
    for loader in ["neoforged", "neoforge", "fabric", "quilt", "forge"] {
        if let Some(rest) = lower.strip_prefix(loader) {
            let rest = rest.strip_prefix("-loader").unwrap_or(rest);
            let rest = rest.strip_prefix('-').unwrap_or(rest);
            return Some((loader.to_string(), rest.to_string()));
        }
    }
    None
}

fn declared(value: &serde_json::Value) -> Option<(String, String)> {
    if let Some(loaders) = value.pointer("/minecraft/modLoaders").and_then(|v| v.as_array()) {
        let entry = loaders.iter().find(|v| v.get("primary").and_then(|v| v.as_bool()) == Some(true)).or_else(|| loaders.first())?;
        let raw_id = entry.get("id")?.as_str()?;
        if let Some((loader, version)) = parse_loader_id(raw_id) {
            if !version.is_empty() {
                return Some((loader, version));
            }
            return Some((loader, String::new()));
        }
    }
    for (key, loader) in [
        ("fabric-loader", "fabric"), ("fabric", "fabric"),
        ("forge", "forge"), ("minecraft-forge", "forge"),
        ("neoforge", "neoforge"), ("neo-forge", "neoforge"), ("neoforged", "neoforge"),
        ("quilt-loader", "quilt"), ("quilt", "quilt"),
    ] {
        if let Some(version) = value.get("dependencies").and_then(|v| v.get(key)).and_then(|v| v.as_str()) {
            return Some((loader.into(), version.into()));
        }
    }
    None
}

pub fn resolve(root: &Path, configured: &str, version: Option<&str>) -> AppResult<(String, Option<String>)> {
    for name in ["modrinth.index.json", "manifest.json"] {
        let file = root.join(name);
        if file.exists() {
            if file.metadata()?.len() > 8 * 1024 * 1024 { return Err(AppError::InvalidInput("Manifest exceeds 8 MiB".into())); }
            let value: serde_json::Value = serde_json::from_reader(std::fs::File::open(file)?)?;
            if let Some((loader, version)) = declared(&value) { return Ok((loader, Some(version))); }
        }
    }
    let configured = configured.trim().to_lowercase();
    if !configured.is_empty() && configured != "vanilla" { return Ok((configured, version.map(str::to_owned))); }
    let mut found = BTreeSet::new();
    if let Ok(entries) = std::fs::read_dir(root.join("mods")) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().is_none_or(|ext| ext != "jar") { continue; }
            let Ok(file) = std::fs::File::open(path) else { continue; };
            let Ok(mut jar) = zip::ZipArchive::new(file) else { continue; };
            for (name, loader) in [("fabric.mod.json", "fabric"), ("quilt.mod.json", "quilt"), ("META-INF/neoforge.mods.toml", "neoforge"), ("META-INF/mods.toml", "forge")] {
                if jar.by_name(name).is_ok() { found.insert(loader); }
            }
        }
    }
    if found.contains("quilt") { found.remove("fabric"); }
    if found.contains("neoforge") { found.remove("forge"); }
    if found.len() > 1 { return Err(AppError::InvalidInput("Mods de loaders diferentes encontrados. Selecione o loader nas configurações da instância.".into())); }
    Ok((found.first().copied().unwrap_or("vanilla").into(), version.map(str::to_owned)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn manifest_identifies_fabric_independently_of_pack_name() {
        let value = serde_json::json!({"name":"Smoke CurseForge", "minecraft":{"modLoaders":[{"id":"fabric-0.14.23"}]}});
        assert_eq!(declared(&value), Some(("fabric".into(), "0.14.23".into())));
    }

    #[test]
    fn every_supported_loader_retains_its_exact_pinned_version() {
        for (loader, key, version) in [("fabric", "fabric-loader", "0.16.10"), ("quilt", "quilt-loader", "0.27.1"), ("forge", "forge", "47.4.10"), ("neoforge", "neoforge", "21.1.256")] {
            let value = serde_json::json!({"dependencies": {key: version, "minecraft": "1.20.1"}});
            assert_eq!(declared(&value), Some((loader.into(), version.into())));
        }
        assert_eq!(declared(&serde_json::json!({"dependencies": {"minecraft": "1.8.9"}})), None);
    }

    #[test]
    fn primary_loader_wins_and_modrinth_uses_dependencies() {
        let value = serde_json::json!({"minecraft":{"modLoaders":[{"id":"forge-47.0.0"},{"id":"fabric-0.14.23","primary":true}]}});
        assert_eq!(declared(&value).unwrap().0, "fabric");
        let value = serde_json::json!({"dependencies":{"minecraft":"1.21.1","neoforge":"21.1.1"}});
        assert_eq!(declared(&value), Some(("neoforge".into(), "21.1.1".into())));
        let value_cf = serde_json::json!({"minecraft":{"modLoaders":[{"id":"NeoForge-21.1.137"}]}});
        assert_eq!(declared(&value_cf), Some(("neoforge".into(), "21.1.137".into())));
    }
}
