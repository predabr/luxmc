use super::*;
use std::io::Read;

fn title(path: &Path) -> Option<String> {
    let mut zip = zip::ZipArchive::new(std::fs::File::open(path).ok()?).ok()?;
    for (name, pointer) in [("fabric.mod.json", "/name"), ("quilt.mod.json", "/quilt_loader/metadata/name")] {
        if let Ok(file) = zip.by_name(name) {
            let value: serde_json::Value = serde_json::from_reader(file.take(1024 * 1024)).ok()?;
            if let Some(value) = value.pointer(pointer).and_then(|value| value.as_str()) {
                return Some(value.chars().filter(|value| !value.is_control()).take(120).collect());
            }
        }
    }
    for name in ["META-INF/neoforge.mods.toml", "META-INF/mods.toml"] {
        if let Ok(file) = zip.by_name(name) {
            let mut text = String::new();
            file.take(1024 * 1024).read_to_string(&mut text).ok()?;
            let value: toml::Value = toml::from_str(&text).ok()?;
            if let Some(name) = value.get("mods").and_then(|value| value.as_array()).and_then(|values| values.first()).and_then(|value| value.get("displayName")).and_then(|value| value.as_str()) {
                return Some(name.chars().filter(|value| !value.is_control()).take(120).collect());
            }
        }
    }
    None
}

impl LocalServerManager {
    pub async fn content_details(&self, id: &str, kind: &str, names: Vec<String>) -> AppResult<serde_json::Value> {
        if !["mod", "datapack", "resourcepack"].contains(&kind) || names.len() > 24 {
            return Err(invalid("Solicite até 24 capas de conteúdos por vez."));
        }
        let current = self.contents(id).await?;
        let directory = self.directory(id).await?;
        let folder = match kind { "mod" => "mods", "datapack" => "world/datapacks", _ => "resourcepacks" };
        let project_path=directory.join(".luxmc-content-projects.json");
        let projects:std::collections::BTreeMap<String,String>=if project_path.exists() {
            let metadata=backups::normal_file(&project_path)?;
            if !metadata.is_file() || metadata.len()>1024*1024 {return Err(invalid("Os metadados de capas excedem o limite de 1 MiB."));}
            serde_json::from_slice(&tokio::fs::read(project_path).await?).unwrap_or_default()
        }else{Default::default()};
        let ids=names.iter().filter_map(|name|projects.get(&format!("{folder}/{}",name.trim_end_matches(".disabled")))).cloned().collect::<std::collections::BTreeSet<_>>();
        let mut remote=std::collections::BTreeMap::<String,serde_json::Value>::new();
        if !ids.is_empty() {
            if let Ok(response)=self.http.get("https://api.modrinth.com/v2/projects").query(&[("ids",serde_json::to_string(&ids)?)]).timeout(Duration::from_secs(15)).send().await {
                if let Ok(mut response)=response.error_for_status() {
                    let mut bytes=Vec::new();
                    while let Ok(Some(chunk))=response.chunk().await {if bytes.len()+chunk.len()>2*1024*1024 {bytes.clear();break;}bytes.extend_from_slice(&chunk);}
                    if let Ok(values)=serde_json::from_slice::<Vec<serde_json::Value>>(&bytes) {for value in values {if let Some(id)=value["id"].as_str(){remote.insert(id.to_string(),value);}}}
                }
            }
        }
        let mut paths = Vec::new();
        for name in names {
            if !current.iter().any(|item| !item.staged && item.kind == kind && item.name == name) {
                return Err(invalid("Conteúdo não encontrado neste servidor."));
            }
            let path = crate::core::mods::pack_download::destination(&directory.join(folder), &name)?;
            backups::normal_file(&path)?;
            let project=projects.get(&format!("{folder}/{}",name.trim_end_matches(".disabled"))).and_then(|id|remote.get(id)).cloned();
            paths.push((name, path, project));
        }
        tokio::task::spawn_blocking(move || {
            serde_json::Value::Array(paths.into_iter().map(|(name, path, project)| {
                let icon=project.as_ref().and_then(|value|value["icon_url"].as_str()).filter(|value|value.starts_with("https://")).map(str::to_string).or_else(||crate::commands::mods::extract_mod_icon_from_jar(&path));
                let display=project.as_ref().and_then(|value|value["title"].as_str()).map(|value|value.chars().filter(|value|!value.is_control()).take(120).collect::<String>()).or_else(||title(&path));
                serde_json::json!({"name":name,"title":display,"icon":icon})
            }).collect())
        }).await.map_err(|error| AppError::Internal(error.to_string()))
    }

    pub async fn toggle_content(&self, id: &str, kind: &str, name: &str, enabled: bool) -> AppResult<()> {
        let _guard = self.change()?;
        if self.current(id).is_ok() || self.content_jobs.load(Ordering::Acquire) != 0 {
            return Err(unavailable("Pare e salve o servidor e aguarde as importações antes de ativar ou desativar conteúdos."));
        }
        if !["mod", "datapack"].contains(&kind) { return Err(invalid("Selecione mods ou datapacks.")); }
        let current = self.contents(id).await?;
        if current.iter().any(|item| item.staged) { return Err(unavailable("Aplique ou cancele as pendências antes de alterar a ativação.")); }
        if !current.iter().any(|item| !item.staged && item.kind == kind && item.name == name) { return Err(invalid("Conteúdo não encontrado.")); }
        if enabled == !name.ends_with(".disabled") { return Ok(()); }
        let directory = self.directory(id).await?;
        let folder = directory.join(if kind == "mod" { "mods" } else { "world/datapacks" });
        let source = crate::core::mods::pack_download::destination(&folder, name)?;
        let target_name = if enabled { name.trim_end_matches(".disabled").to_string() } else { format!("{name}.disabled") };
        let target = crate::core::mods::pack_download::destination(&folder, &target_name)?;
        backups::normal_file(&source)?;
        if target.exists() { return Err(invalid("Já existe um arquivo com esse nome de destino.")); }
        if enabled {
            let path = source.clone(); let kind = kind.to_string();
            tokio::task::spawn_blocking(move || content::validate_archive(&path, &kind)).await.map_err(|error| AppError::Internal(error.to_string()))??;
        }
        tokio::fs::rename(source, target).await?;
        Ok(())
    }
}
