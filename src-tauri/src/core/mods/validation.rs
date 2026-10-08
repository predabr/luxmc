use std::{collections::{HashMap, HashSet}, io::Read, path::Path};
use crate::error::AppResult;

#[derive(Default)]
pub struct Audit { pub errors: Vec<String>, pub warnings: Vec<String> }

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
