use std::{collections::{HashMap, HashSet}, io::Read, path::Path};
use crate::error::AppResult;

#[derive(Default)]
pub struct Audit { pub errors: Vec<String>, pub warnings: Vec<String> }

#[derive(Clone, Default)]
pub(crate) struct ModMetadata {
    pub ids: std::collections::BTreeSet<String>,
    pub loaders: std::collections::BTreeSet<String>,
    pub minecraft_declared: bool,
}

pub(crate) fn primary_ids(path: &Path) -> Option<std::collections::BTreeSet<String>> {
    mod_metadata(path).map(|metadata| metadata.ids)
}

pub(crate) fn mod_metadata(path: &Path) -> Option<ModMetadata> {
    type Entry = ((u64, std::time::SystemTime), Option<ModMetadata>);
    static CACHE: std::sync::LazyLock<std::sync::Mutex<HashMap<std::path::PathBuf, Entry>>> = std::sync::LazyLock::new(|| std::sync::Mutex::new(HashMap::new()));
    let metadata = std::fs::metadata(path).ok()?;
    let fingerprint = (metadata.len(), metadata.modified().ok()?);
    if let Ok(cache) = CACHE.lock() {
        if let Some((saved, ids)) = cache.get(path) { if *saved == fingerprint { return ids.clone(); } }
    }
    let ids = read_mod_metadata(path);
    if let Ok(mut cache) = CACHE.lock() {
        if cache.len() >= 4096 { cache.clear(); }
        cache.insert(path.to_owned(), (fingerprint, ids.clone()));
    }
    ids
}

fn read_mod_metadata(path: &Path) -> Option<ModMetadata> {
    let mut archive = zip::ZipArchive::new(std::io::BufReader::new(std::fs::File::open(path).ok()?)).ok()?;
    let mut metadata = ModMetadata::default();
    for (name, pointer) in [("fabric.mod.json", "/id"), ("quilt.mod.json", "/quilt_loader/id")] {
        if let Ok(file) = archive.by_name(name) {
            if let Ok(value) = serde_json::from_reader::<_, serde_json::Value>(file.take(2 * 1024 * 1024)) {
                if let Some(id) = value.pointer(pointer).and_then(|v| v.as_str()) {
                    metadata.ids.insert(id.to_owned());
                    metadata.loaders.insert(if name == "fabric.mod.json" { "fabric" } else { "quilt" }.into());
                    metadata.minecraft_declared |= value.pointer("/depends/minecraft").is_some_and(|version| version.is_string() || version.is_array())
                        || value.pointer("/quilt_loader/depends").and_then(|depends| depends.as_array()).is_some_and(|depends| depends.iter().any(|dependency| dependency["id"] == "minecraft" && dependency.get("versions").is_some()));
                }
            }
        }
    }
    for name in ["META-INF/neoforge.mods.toml", "META-INF/mods.toml"] {
        if let Ok(file) = archive.by_name(name) {
            let mut text = String::new();
            if file.take(2 * 1024 * 1024).read_to_string(&mut text).is_ok() {
                if let Ok(value) = text.parse::<toml::Value>() {
                    if let Some(mods) = value.get("mods").and_then(|v| v.as_array()) {
                        for module in mods { if let Some(id) = module.get("modId").and_then(|v| v.as_str()) { metadata.ids.insert(id.to_owned()); } }
                        let dependencies = value.get("dependencies").and_then(|deps| deps.as_table());
                        let has_dependency = |id: &str| dependencies.is_some_and(|deps| deps.values().filter_map(|deps| deps.as_array()).flatten().any(|dependency| dependency.get("modId").and_then(|value| value.as_str()) == Some(id)));
                        metadata.loaders.insert(if name.contains("neoforge") || has_dependency("neoforge") { "neoforge" } else { "forge" }.into());
                        metadata.minecraft_declared |= has_dependency("minecraft");
                    }
                }
            }
        }
    }
    (!metadata.ids.is_empty()).then_some(metadata)
}

pub fn repair_duplicates(root: &Path, preferred: &HashSet<String>) -> AppResult<usize> {
    let directory = root.join("mods");
    if !directory.is_dir() { return Ok(0); }
    let mut paths = std::fs::read_dir(&directory)?.filter_map(Result::ok)
        .filter(|entry| entry.file_type().is_ok_and(|kind| kind.is_file()) && entry.path().extension().is_some_and(|extension| extension == "jar"))
        .map(|entry| entry.path()).collect::<Vec<_>>();
    paths.sort_by_key(|path| (!preferred.contains(&path.file_name().unwrap_or_default().to_string_lossy().into_owned()), path.file_name().map(|name| name.to_os_string())));
    let mut retained = std::collections::BTreeMap::<std::collections::BTreeSet<String>, std::path::PathBuf>::new();
    let mut removed = Vec::new();
    for path in paths {
        let Some(ids) = primary_ids(&path) else { continue; };
        if let Some(kept) = retained.get(&ids) {
            let kept_pinned = preferred.contains(&kept.file_name().unwrap_or_default().to_string_lossy().into_owned());
            let other_pinned = preferred.contains(&path.file_name().unwrap_or_default().to_string_lossy().into_owned());
            let identical = || -> std::io::Result<bool> {
                use sha2::Digest;
                fn digest(path: &Path) -> std::io::Result<Vec<u8>> {
                    let mut file = std::fs::File::open(path)?;
                    let mut hash = sha2::Sha256::new();
                    let mut bytes = [0u8; 65536];
                    loop { let count = file.read(&mut bytes)?; if count == 0 { break; } hash.update(&bytes[..count]); }
                    Ok(hash.finalize().to_vec())
                }
                Ok(std::fs::metadata(kept)?.len() == std::fs::metadata(&path)?.len() && digest(kept)? == digest(&path)?)
            };
            if kept_pinned && !other_pinned || identical()? { removed.push((path, kept.clone())); }
        }
        else { retained.insert(ids, path); }
    }
    if removed.is_empty() { return Ok(0); }
    let backup = root.join(".luxmc/removed-duplicates").join(uuid::Uuid::new_v4().to_string());
    std::fs::create_dir_all(&backup)?;
    let mut records = Vec::new();
    for (path, kept) in &removed {
        let name = path.file_name().unwrap_or_default();
        std::fs::rename(path, backup.join(name))?;
        records.push(serde_json::json!({"removed": name.to_string_lossy(), "retained": kept.file_name().unwrap_or_default().to_string_lossy()}));
    }
    std::fs::write(backup.join("repair.json"), serde_json::to_vec_pretty(&records)?)?;
    Ok(removed.len())
}

pub fn audit(root: &Path, deep: bool) -> AppResult<Audit> {
    let mut report = Audit::default();
    let mut providers = HashMap::<String,String>::new();
    let mut dependencies = Vec::<(String,String)>::new();
    let mut folders=vec!["mods".to_string(),"shaderpacks".to_string(),"resourcepacks".to_string()];
    if deep && root.join("saves").is_dir() {
        for world in std::fs::read_dir(root.join("saves"))?.take(128) {
            let world=world?;
            if world.file_type()?.is_dir() && !world.file_name().to_string_lossy().starts_with('.') {folders.push(format!("saves/{}/datapacks",world.file_name().to_string_lossy()));}
        }
    }
    for folder in folders {
        let directory = root.join(&folder);
        if !directory.is_dir() {continue;}
        for entry in std::fs::read_dir(directory)? {
            let entry=entry?; let path=entry.path();
            if !entry.file_type()?.is_file() || !matches!(path.extension().and_then(|e| e.to_str()),Some("jar"|"zip")) {continue;}
            let name=entry.file_name().to_string_lossy().into_owned();
            let archive = std::fs::File::open(&path).map(std::io::BufReader::new).map(zip::ZipArchive::new);
            let mut archive = match archive { Ok(Ok(archive))=>archive, _=>{ report.errors.push(format!("Arquivo corrompido ou incompleto: {folder}/{name}"));continue;} };
            if deep {
                let mut expanded=0u64;
                for index in 0..archive.len() {
                    let mut file=archive.by_index(index)?;
                    expanded=expanded.saturating_add(file.size());
                    if expanded>1024*1024*1024 { report.warnings.push(format!("{name}: validação CRC interrompida acima de 1 GiB descompactado"));break; }
                    if std::io::copy(&mut file,&mut std::io::sink()).is_err() {report.errors.push(format!("{name}: conteúdo interno corrompido (CRC)"));break;}
                }
            }
            if folder != "mods" {continue;}
            if let Ok(mut metadata)=archive.by_name("fabric.mod.json") {
                let json=serde_json::from_reader::<_,serde_json::Value>(metadata.by_ref().take(2*1024*1024));
                if let Ok(json)=json {
                    if json["environment"] == "server" {continue;}
                    if let Some(id)=json["id"].as_str() { if let Some(previous)=providers.insert(id.into(),name.clone()) {if previous != name {report.errors.push(format!("Mod duplicado {id}: {previous} e {name}"));}} }
                    if let Some(provides)=json["provides"].as_array() { for id in provides.iter().filter_map(|value| value.as_str()) {providers.entry(id.into()).or_insert_with(||name.clone());} }
                    if let Some(required)=json["depends"].as_object() { for id in required.keys() {dependencies.push((name.clone(),id.clone()));} }
                    if json["jars"].as_array().is_some_and(|jars| !jars.is_empty()) {report.warnings.push(format!("{name}: inclui dependências internas; o loader fará a validação completa"));}
                }
            };
            if let Ok(mut metadata)=archive.by_name("quilt.mod.json") {
                if let Ok(json)=serde_json::from_reader::<_,serde_json::Value>(metadata.by_ref().take(2*1024*1024)) {
                    if let Some(id)=json.pointer("/quilt_loader/id").and_then(|v|v.as_str()) {if let Some(previous)=providers.insert(id.into(),name.clone()) {if previous != name {report.errors.push(format!("Mod duplicado {id}: {previous} e {name}"));}}}
                    if let Some(required)=json.pointer("/quilt_loader/depends").and_then(|v|v.as_array()) {for dep in required {if dep["optional"]==true {continue;} if let Some(id)=dep["id"].as_str() {dependencies.push((name.clone(),id.into()));}}}
                }
            };
            for metadata_name in ["META-INF/neoforge.mods.toml","META-INF/mods.toml"] {
                if let Ok(mut metadata)=archive.by_name(metadata_name) {
                    let mut text=String::new();
                    if metadata.by_ref().take(2*1024*1024).read_to_string(&mut text).is_ok() {
                        if let Ok(json)=text.parse::<toml::Value>() {
                            if let Some(mods)=json.get("mods").and_then(|v|v.as_array()) {for module in mods {if let Some(id)=module.get("modId").and_then(|v|v.as_str()) {if let Some(previous)=providers.insert(id.into(),name.clone()) {if previous != name {report.errors.push(format!("Mod duplicado {id}: {previous} e {name}"));}}}}}
                            if let Some(deps)=json.get("dependencies").and_then(|v|v.as_table()) {for list in deps.values().filter_map(|v|v.as_array()) {for dep in list {if dep.get("side").and_then(|v|v.as_str())==Some("SERVER") {continue;} if dep.get("mandatory").and_then(|v|v.as_bool())==Some(true) || dep.get("type").and_then(|v|v.as_str())==Some("required") {if let Some(id)=dep.get("modId").and_then(|v|v.as_str()) {dependencies.push((name.clone(),id.into()));}}}}}
                        }
                    }
                    break;
                }
            }
        }
    }
    let built_in: HashSet<&str> = ["minecraft","java","fabricloader","quilt_loader","forge","neoforge"].into_iter().collect();
    for (name,id) in dependencies {
        if !providers.contains_key(&id) && !built_in.contains(id.as_str()) { report.warnings.push(format!("{name}: dependência {id} não encontrada entre os mods externos. Verifique também os JARs internos antes de instalar.")); }
    }
    Ok(report)
}

#[cfg(test)]
mod tests {
    fn mod_jar(path: &std::path::Path, id: &str) {
        use std::io::Write;
        let mut archive = zip::ZipWriter::new(std::fs::File::create(path).unwrap());
        archive.start_file("fabric.mod.json", zip::write::FileOptions::default()).unwrap();
        archive.write_all(serde_json::json!({"id":id,"version":"1"}).to_string().as_bytes()).unwrap();
        archive.finish().unwrap();
    }
    #[test]
    fn duplicate_repair_keeps_pinned_file_and_a_recoverable_copy() {
        let root=std::env::temp_dir().join(format!("luxmc-dedupe-{}",uuid::Uuid::new_v4()));
        std::fs::create_dir_all(root.join("mods")).unwrap();
        mod_jar(&root.join("mods/a-old.jar"), "example");
        mod_jar(&root.join("mods/pinned.jar"), "example");
        mod_jar(&root.join("mods/independent.jar"), "independent");
        let preferred = ["pinned.jar".to_owned()].into_iter().collect();
        assert_eq!(super::repair_duplicates(&root,&preferred).unwrap(),1);
        assert!(root.join("mods/pinned.jar").is_file());
        assert!(root.join("mods/independent.jar").is_file());
        assert!(!root.join("mods/a-old.jar").exists());
        let backup=std::fs::read_dir(root.join(".luxmc/removed-duplicates")).unwrap().next().unwrap().unwrap().path();
        assert!(backup.join("a-old.jar").is_file());
        assert!(super::audit(&root,false).unwrap().errors.is_empty());
        assert_eq!(super::repair_duplicates(&root,&preferred).unwrap(),0);
        std::fs::remove_dir_all(root).unwrap();
    }
    #[test]
    fn neoforge_duplicate_repair_keeps_manifest_version() {
        use std::io::Write;
        let root=std::env::temp_dir().join(format!("luxmc-neoforge-dedupe-{}",uuid::Uuid::new_v4()));
        std::fs::create_dir_all(root.join("mods")).unwrap();
        for (name,version) in [("old.jar","1"),("manifest.jar","2")] {
            let mut archive=zip::ZipWriter::new(std::fs::File::create(root.join("mods").join(name)).unwrap());
            archive.start_file("META-INF/neoforge.mods.toml",zip::write::FileOptions::default()).unwrap();
            archive.write_all(format!("[[mods]]\nmodId='example'\nversion='{version}'\n").as_bytes()).unwrap();
            archive.finish().unwrap();
        }
        assert_eq!(super::repair_duplicates(&root,&["manifest.jar".to_owned()].into_iter().collect()).unwrap(),1);
        assert!(root.join("mods/manifest.jar").is_file());
        assert!(!root.join("mods/old.jar").exists());
        assert!(super::audit(&root,false).unwrap().errors.is_empty());
        std::fs::remove_dir_all(root).unwrap();
    }
    #[test]
    fn duplicate_repair_preserves_ambiguous_versions_and_extra_modules() {
        use std::io::Write;
        let root=std::env::temp_dir().join(format!("luxmc-dedupe-safe-{}",uuid::Uuid::new_v4()));
        std::fs::create_dir_all(root.join("mods")).unwrap();
        mod_jar(&root.join("mods/a.jar"), "example");
        let mut archive=zip::ZipWriter::new(std::fs::File::create(root.join("mods/b.jar")).unwrap());
        archive.start_file("fabric.mod.json",zip::write::FileOptions::default()).unwrap();
        archive.write_all(br#"{"id":"example","version":"2"}"#).unwrap();
        archive.finish().unwrap();
        let mut archive=zip::ZipWriter::new(std::fs::File::create(root.join("mods/multiple.jar")).unwrap());
        archive.start_file("META-INF/neoforge.mods.toml",zip::write::FileOptions::default()).unwrap();
        archive.write_all(b"[[mods]]\nmodId='example'\n[[mods]]\nmodId='extra'\n").unwrap();
        archive.finish().unwrap();
        assert_eq!(super::repair_duplicates(&root,&Default::default()).unwrap(),0);
        assert_eq!(super::repair_duplicates(&root,&["a.jar".to_owned(),"b.jar".to_owned()].into_iter().collect()).unwrap(),0);
        assert!(root.join("mods/a.jar").is_file());
        assert!(root.join("mods/b.jar").is_file());
        assert!(root.join("mods/multiple.jar").is_file());
        std::fs::remove_dir_all(root).unwrap();
    }
    #[test]
    fn damaged_archive_is_reported_without_deleting_it() {
        let root=std::env::temp_dir().join(format!("luxmc-audit-{}",uuid::Uuid::new_v4()));
        std::fs::create_dir_all(root.join("mods")).unwrap();
        std::fs::write(root.join("mods/broken.jar"),b"broken").unwrap();
        let result=super::audit(&root,true).unwrap();
        assert_eq!(result.errors.len(),1); assert!(root.join("mods/broken.jar").exists());
        std::fs::remove_dir_all(root).unwrap();
    }
}
