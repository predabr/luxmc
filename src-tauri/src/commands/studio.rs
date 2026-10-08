use crate::db::models::ProfileRow;
use crate::error::{AppError, AppResult};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::{BTreeMap, BTreeSet};
use std::io::Read;
use std::path::Path;

pub async fn profile(id: &str) -> AppResult<ProfileRow> {
    let db = crate::db::shared_db().await?;
    sqlx::query_as("SELECT * FROM profiles WHERE id = ?").bind(id).fetch_optional(db.pool()).await?
        .ok_or_else(|| AppError::NotFound("Instância não encontrada".into()))
}

#[derive(Clone, Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct DependencyNode {
    pub id: String,
    pub name: String,
    pub file: String,
    pub version: String,
    pub requires: Vec<String>,
    pub provides: Vec<String>,
    pub required_by: Vec<String>,
    pub missing: Vec<String>,
    pub metadata_known: bool,
}

fn metadata(bytes: &[u8], file: &str, depth: usize) -> AppResult<Vec<DependencyNode>> {
    let mut zip = zip::ZipArchive::new(std::io::Cursor::new(bytes))?;
    let mut nodes = Vec::new();
    let mut nested = Vec::new();
    if let Ok(entry)=zip.by_name("META-INF/jarjar/metadata.json") {
        if entry.size()<=2*1024*1024 {
            if let Ok(value)=serde_json::from_reader::<_,Value>(entry.take(2*1024*1024)) {
                if let Some(jars)=value["jars"].as_array(){nested.extend(jars.iter().filter_map(|jar|jar["path"].as_str().map(str::to_owned)));}
            }
        }
    }
    for name in ["fabric.mod.json", "quilt.mod.json", "META-INF/neoforge.mods.toml", "META-INF/mods.toml"] {
        let Ok(entry) = zip.by_name(name) else { continue; };
        if entry.size() > 2 * 1024 * 1024 { return Err(AppError::InvalidInput("Metadados de mod muito grandes".into())); }
        let mut text = String::new();
        entry.take(2 * 1024 * 1024).read_to_string(&mut text)?;
        if name.ends_with("json") {
            let value: Value = serde_json::from_str(&text)?;
            let module = if name.starts_with("quilt") { &value["quilt_loader"] } else { &value };
            let Some(id) = module["id"].as_str() else { continue; };
            let mut requires = Vec::new();
            if let Some(deps) = module["depends"].as_object() { requires.extend(deps.keys().cloned()); }
            if let Some(deps) = module["depends"].as_array() { requires.extend(deps.iter().filter(|dep| dep["optional"] != true).filter_map(|dep| dep["id"].as_str().map(str::to_owned))); }
            let provides = module["provides"].as_array().map(|items| items.iter().filter_map(|item| item.as_str().or_else(|| item["id"].as_str()).map(str::to_owned)).collect()).unwrap_or_default();
            nodes.push(DependencyNode { id:id.into(), name:module["name"].as_str().or_else(|| module.pointer("/metadata/name").and_then(Value::as_str)).unwrap_or(id).into(), file:file.into(), version:module["version"].as_str().unwrap_or("").into(), requires, provides, required_by:Vec::new(), missing:Vec::new(), metadata_known:true });
            if let Some(jars) = value["jars"].as_array() { nested.extend(jars.iter().filter_map(|jar| jar["file"].as_str().map(str::to_owned))); }
            if let Some(jars) = module["jars"].as_array() { nested.extend(jars.iter().filter_map(|jar| jar.as_str().map(str::to_owned))); }
        } else {
            let value: toml::Value = text.parse().map_err(|error: toml::de::Error| AppError::InvalidInput(error.to_string()))?;
            if let Some(modules) = value.get("mods").and_then(toml::Value::as_array) {
                for module in modules {
                    let Some(id) = module.get("modId").and_then(toml::Value::as_str) else { continue; };
                    let requires = value.get("dependencies").and_then(|deps| deps.get(id)).and_then(toml::Value::as_array).map(|deps| deps.iter().filter(|dep| dep.get("side").and_then(toml::Value::as_str) != Some("SERVER") && (dep.get("mandatory").and_then(toml::Value::as_bool)==Some(true) || dep.get("type").and_then(toml::Value::as_str)==Some("required"))).filter_map(|dep| dep.get("modId").and_then(toml::Value::as_str).map(str::to_owned)).collect()).unwrap_or_default();
                    nodes.push(DependencyNode { id:id.into(),name:module.get("displayName").and_then(toml::Value::as_str).unwrap_or(id).into(),file:file.into(),version:module.get("version").and_then(toml::Value::as_str).unwrap_or("").into(),requires,provides:Vec::new(),required_by:Vec::new(),missing:Vec::new(),metadata_known:true });
                }
            }
        }
        break;
    }
    if depth < 2 {
        for name in nested.into_iter().take(64) {
            if let Ok(entry) = zip.by_name(&name) {
                if entry.size() > 8 * 1024 * 1024 { continue; }
                let mut bytes = Vec::new(); entry.take(8 * 1024 * 1024).read_to_end(&mut bytes)?;
                if let Ok(children) = metadata(&bytes, file, depth + 1) { nodes.extend(children); }
            }
        }
    }
    Ok(nodes)
}

pub fn graph(root: &Path) -> AppResult<Vec<DependencyNode>> {
    let directory = root.join("mods");
    if !directory.is_dir() { return Ok(Vec::new()); }
    let mut nodes = Vec::new();
    for entry in std::fs::read_dir(directory)?.take(10000) {
        let entry = entry?;
        if !entry.file_type()?.is_file() || entry.path().extension().is_none_or(|ext| ext != "jar") { continue; }
        let name = entry.file_name().to_string_lossy().into_owned();
        if entry.metadata()?.len() > 256 * 1024 * 1024 { continue; }
        match metadata(&std::fs::read(entry.path())?, &name, 0) {
            Ok(found) if !found.is_empty() => nodes.extend(found),
            _ => nodes.push(DependencyNode {id:name.clone(),name:name.clone(),file:name,version:String::new(),requires:Vec::new(),provides:Vec::new(),required_by:Vec::new(),missing:Vec::new(),metadata_known:false}),
        }
    }
    let providers: BTreeMap<String,String> = nodes.iter().flat_map(|node| std::iter::once(node.id.clone()).chain(node.provides.clone()).map(|id| (id,node.id.clone()))).collect();
    let builtins = ["minecraft","java","fabricloader","quilt_loader","forge","neoforge"];
    let edges: Vec<(String,String)> = nodes.iter().flat_map(|node| node.requires.iter().filter_map(|id| providers.get(id).map(|provider| (node.id.clone(),provider.clone())))).collect();
    for node in &mut nodes {
        node.required_by = edges.iter().filter(|(_,to)| to == &node.id).map(|(from,_)| from.clone()).collect();
        node.missing = node.requires.iter().filter(|id| !providers.contains_key(*id) && !builtins.contains(&id.as_str())).cloned().collect();
    }
    nodes.sort_by(|a,b| a.name.cmp(&b.name));
    Ok(nodes)
}

#[tauri::command]
pub async fn dependency_graph(profile_id: String) -> AppResult<Vec<DependencyNode>> {
    let row = profile(&profile_id).await?;
    tokio::task::spawn_blocking(move || graph(Path::new(&row.game_dir))).await.map_err(|error| AppError::Internal(error.to_string()))?
}

#[tauri::command]
pub async fn pc_recommendations() -> AppResult<Value> {
    let history = super::experience::performance_history().await?;
    let db = crate::db::shared_db().await?;
    let profiles: Vec<ProfileRow> = sqlx::query_as("SELECT * FROM profiles").fetch_all(db.pool()).await?;
    let mut system = sysinfo::System::new(); system.refresh_memory();
    let total_mb = system.total_memory() / 1048576;
    let available_mb = system.available_memory() / 1048576;
    let budget = available_mb.saturating_sub(1536).min(total_mb.saturating_sub(2048));
    let mut items = Vec::new();
    for profile in profiles {
        let entries: Vec<&Value> = history.as_array().into_iter().flatten().filter(|entry| entry["profileId"] == profile.id).collect();
        let peaks: Vec<f64> = entries.iter().filter(|entry| entry["metrics"]["kind"]=="session").filter_map(|entry| entry["metrics"]["peakRamMb"].as_f64()).filter(|peak| *peak>0.0).collect();
        let prep: Vec<f64> = entries.iter().filter(|entry| entry["metrics"]["kind"]=="preparation").filter_map(|entry| entry["metrics"]["seconds"].as_f64()).collect();
        if peaks.is_empty() { continue; }
        let peak = peaks.iter().copied().fold(0.0,f64::max);
        items.push(json!({"profileId":profile.id,"name":profile.name,"icon":profile.icon,"samples":peaks.len(),"peakRamMb":peak,"preparationSeconds":if prep.is_empty(){None}else{Some(prep.iter().sum::<f64>() / prep.len() as f64)},"fitsNow":peak * 1.2 <= budget as f64,"suggestedRamMb":((peak * 1.2 / 512.0).ceil() as u64 * 512).clamp(1024,total_mb.saturating_sub(2048).max(1024))}));
    }
    items.sort_by(|a,b| b["fitsNow"].as_bool().cmp(&a["fitsNow"].as_bool()).then_with(|| a["peakRamMb"].as_f64().partial_cmp(&b["peakRamMb"].as_f64()).unwrap_or(std::cmp::Ordering::Equal)));
    Ok(json!({"totalRamMb":total_mb,"availableRamMb":available_mb,"budgetMb":budget,"items":items}))
}

#[tauri::command]
pub async fn launcher_resource_sample() -> AppResult<Value> {
    let mut system=sysinfo::System::new();
    let refresh=sysinfo::ProcessRefreshKind::nothing().with_cpu().with_memory();
    system.refresh_processes_specifics(sysinfo::ProcessesToUpdate::All,true,refresh);
    tokio::time::sleep(std::time::Duration::from_millis(250)).await;
    system.refresh_processes_specifics(sysinfo::ProcessesToUpdate::All,true,refresh);
    let pid=sysinfo::Pid::from_u32(std::process::id());
    let mut included=std::collections::HashSet::from([pid]);
    loop {let previous=included.len();for (id,process) in system.processes(){if process.parent().is_some_and(|parent|included.contains(&parent)) && !process.name().to_string_lossy().to_lowercase().starts_with("java"){included.insert(*id);}}if included.len()==previous{break;}}
    let memory:u64=included.iter().filter_map(|id|system.process(*id)).map(|process|process.memory()).sum();
    let cpu:f32=included.iter().filter_map(|id|system.process(*id)).map(|process|process.cpu_usage()).sum();
    let cores=std::thread::available_parallelism().map(|value|value.get()).unwrap_or(1) as f32;
    Ok(json!({"ramMb":memory/1048576,"cpuPercent":cpu/cores,"processes":included.len(),"sampleMilliseconds":250}))
}

#[derive(Clone, Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct PackReference { pub source:String, pub project_id:String, pub version_id:String, pub name:String }

pub fn valid_reference(reference: &PackReference) -> bool {
    ["modrinth","curseforge"].contains(&reference.source.as_str()) && [&reference.project_id,&reference.version_id].iter().all(|id| !id.is_empty() && id.len()<=80 && id.chars().all(|ch| ch.is_ascii_alphanumeric() || ch=='-')) && reference.name.len()<=200 && (reference.source!="curseforge" || [&reference.project_id,&reference.version_id].iter().all(|id|id.parse::<u64>().is_ok_and(|number|number>0)))
}

#[derive(Clone, Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct RoomRecipe { pub name:String, pub mc_version:String, pub loader:String, pub loader_version:Option<String>, pub fingerprint:String, pub mods:Vec<PackReference>, pub pack:Option<PackReference>, pub unavailable:Vec<String> }

pub async fn recipe(profile_id: &str) -> AppResult<RoomRecipe> {
    let row = profile(profile_id).await?;
    let compatibility = super::experience::instance_compatibility(profile_id.into()).await?;
    let db = crate::db::shared_db().await?;
    let tracked = crate::db::schema::mods::list_by_profile(&db,profile_id).await?;
    let root = Path::new(&row.game_dir);
    let mut mods = Vec::new(); let mut available = BTreeSet::new();
    for item in tracked {
        let file = root.join("mods").join(&item.file_name);
        if !file.is_file() || item.file_name.contains(['/', '\\']) { continue; }
        let reference = PackReference {source:item.source,project_id:item.project_id,version_id:item.version_id,name:item.file_name.clone()};
        if valid_reference(&reference) { available.insert(item.file_name); mods.push(reference); }
    }
    let mut unavailable = Vec::new();
    if let Ok(entries) = std::fs::read_dir(root.join("mods")) { for entry in entries.flatten() { if entry.file_type()?.is_file() && entry.path().extension().is_some_and(|ext| ext=="jar") && !available.contains(&entry.file_name().to_string_lossy().into_owned()) { unavailable.push(entry.file_name().to_string_lossy().into_owned()); } } }
    let pack=linked_pack(&row).await?;
    Ok(RoomRecipe {name:row.name,mc_version:row.mc_version,loader:row.loader,loader_version:row.loader_version,fingerprint:compatibility.mod_fingerprint,mods,pack,unavailable})
}

static RECIPE_CACHE:tokio::sync::Mutex<Option<(String,std::time::Instant,RoomRecipe)>>=tokio::sync::Mutex::const_new(None);

pub async fn active_recipe() -> AppResult<RoomRecipe> {
    let directory = crate::core::launcher::get_active_game_dir().ok_or_else(|| AppError::InvalidState("Abra o Minecraft e o mundo LAN para compartilhar a receita".into()))?;
    let key=format!("{}:{}",crate::core::launcher::get_active_game_pid(),directory.display());
    let mut cache=RECIPE_CACHE.lock().await;
    if let Some((cached,when,recipe))=&*cache {if cached==&key && when.elapsed().as_secs()<15{return Ok(recipe.clone());}}
    let db = crate::db::shared_db().await?;
    let id:String = sqlx::query_scalar("SELECT id FROM profiles WHERE game_dir = ? LIMIT 1").bind(directory.to_string_lossy().as_ref()).fetch_one(db.pool()).await?;
    let result=recipe(&id).await?;
    *cache=Some((key,std::time::Instant::now(),result.clone()));
    Ok(result)
}

async fn linked_pack(row:&ProfileRow) -> AppResult<Option<PackReference>> {
    let root=Path::new(&row.game_dir);
    let mut pack = None;
    for (source,file,project) in [("modrinth","modrinth.index.json","projectId"),("curseforge","manifest.json","projectID")] {
        if let Ok(bytes) = tokio::fs::read(root.join(file)).await {
            if let Ok(value) = serde_json::from_slice::<Value>(&bytes) {
                let project_id = value[project].as_str().map(str::to_owned).or_else(|| value[project].as_u64().map(|n| n.to_string()));
                if let (Some(project_id),Some(version)) = (project_id,value["sourceVersionId"].as_str()) {
                    let reference = PackReference {source:source.into(),project_id,version_id:version.into(),name:row.name.clone()};
                    if valid_reference(&reference) {pack=Some(reference);}
                }
            }
        }
    }
    Ok(pack)
}

#[tauri::command]
pub async fn instance_pack_reference(profile_id:String) -> AppResult<Option<PackReference>> { linked_pack(&profile(&profile_id).await?).await }

pub async fn pack_file(state:&crate::state::AppState,reference:&PackReference) -> AppResult<crate::core::mods::ModFile> {
    if !valid_reference(reference) {return Err(AppError::InvalidInput("Referência de modpack inválida".into()));}
    let (url,name,size,sha1)=if reference.source=="modrinth" {
        let value:Value=state.http.get(format!("https://api.modrinth.com/v2/version/{}",reference.version_id)).send().await?.error_for_status()?.json().await?;
        if value["project_id"].as_str()!=Some(&reference.project_id){return Err(AppError::InvalidInput("A versão não pertence ao projeto".into()));}
        let files=value["files"].as_array().ok_or_else(|| AppError::NotFound("Arquivos da versão ausentes".into()))?;
        let file=files.iter().find(|file| file["primary"]==true).or_else(|| files.first()).ok_or_else(|| AppError::NotFound("Arquivo ausente".into()))?;
        (file["url"].as_str().unwrap_or("").to_owned(),format!("room-{}.mrpack",reference.version_id),file["size"].as_u64(),file.pointer("/hashes/sha1").and_then(Value::as_str).map(str::to_owned))
    } else {
        let id=reference.version_id.parse::<u64>().map_err(|_| AppError::InvalidInput("ID de arquivo inválido".into()))?;
        let file=crate::core::mods::curseforge::get_files_batch(&state.http,&[id]).await.remove(&id).ok_or_else(|| AppError::NotFound("Arquivo CurseForge indisponível".into()))?;
        if file.mod_id.to_string()!=reference.project_id{return Err(AppError::InvalidInput("A versão não pertence ao projeto".into()));}
        let url=file.download_url.clone().ok_or_else(|| AppError::InvalidInput("O autor restringiu o download automático deste arquivo".into()))?;
        (url,format!("room-{id}.zip"),file.size,file.sha1)
    };
    Ok(crate::core::mods::ModFile {url,filename:name,size:size.unwrap_or(0),sha1:sha1.unwrap_or_default()})
}

#[tauri::command]
pub async fn pack_reference_version(state:tauri::State<'_,crate::state::AppState>,reference:PackReference) -> AppResult<crate::core::mods::ModVersion> { pack_reference_version_core(&state,reference).await }

pub async fn pack_reference_version_core(state:&crate::state::AppState,reference:PackReference) -> AppResult<crate::core::mods::ModVersion> {
    let file=pack_file(state,&reference).await?;
    Ok(crate::core::mods::ModVersion {id:reference.version_id.clone(),name:reference.name,version_number:reference.version_id,files:vec![file],loaders:Vec::new()})
}

pub async fn pack_archive(state:&crate::state::AppState,reference:&PackReference) -> AppResult<String> {
    let file=pack_file(state,reference).await?;
    let path=super::mods::mods_download_to_temp_core(state,file.url,file.filename).await?;
    if !crate::core::mods::pack_download::verify_existing(Path::new(&path),(file.size>0).then_some(file.size),(!file.sha1.is_empty()).then_some(file.sha1.as_str()),None,true).await{return Err(AppError::InvalidInput("Integridade do pacote divergente".into()));}
    Ok(path)
}

static PREPARING:tokio::sync::Mutex<()>=tokio::sync::Mutex::const_new(());

#[tauri::command]
pub async fn room_prepare_instance(state:tauri::State<'_,crate::state::AppState>,app:tauri::AppHandle,owner_id:String) -> AppResult<ProfileRow> {
    use tauri::Emitter;
    use crate::network::p2p_tunnel::coordination::{tunnel_room_recipe,tunnel_set_preparation,Preparation};
    let _guard=PREPARING.try_lock().map_err(|_| AppError::InvalidState("Uma sala já está sendo preparada".into()))?;
    let update=|phase:&str,percent:u8,fingerprint:&str| Preparation {phase:phase.into(),percent,owner_id:owner_id.clone(),fingerprint:fingerprint.into()};
    tunnel_set_preparation(update("planning",0,"")).await?;
    let result:AppResult<ProfileRow>=async {
        let recipe=tunnel_room_recipe(owner_id.clone()).await?;
        if recipe.name.len()>200 || recipe.mc_version.is_empty() || recipe.mc_version.len()>64 || !recipe.mc_version.chars().all(|ch|ch.is_ascii_alphanumeric() || "._+-".contains(ch)) || recipe.mc_version.contains("..") || recipe.loader_version.as_ref().is_some_and(|version|version.len()>80 || version.is_empty() || version.contains("..") || !version.chars().all(|ch|ch.is_ascii_alphanumeric() || "._+-".contains(ch))) || recipe.fingerprint.len()!=64 || !recipe.fingerprint.chars().all(|ch| ch.is_ascii_hexdigit()) || recipe.mods.len()>2000 || !["vanilla","fabric","quilt","forge","neoforge"].contains(&recipe.loader.as_str()) || recipe.mods.iter().any(|item| !valid_reference(item)) {return Err(AppError::InvalidInput("Receita da sala inválida".into()));}
        if !recipe.unavailable.is_empty() && recipe.pack.is_none(){return Err(AppError::InvalidInput(format!("O anfitrião tem {} mods locais sem referência de provedor. Publique um pack ou vincule esses arquivos antes de preparar automaticamente.",recipe.unavailable.len())));}
        state.import_cancel.store(false,std::sync::atomic::Ordering::SeqCst);
        tunnel_set_preparation(update("downloading",5,"")).await?;
        let name=format!("Sala · {}",recipe.name.chars().take(80).collect::<String>());
        let mut row=if let Some(pack)=&recipe.pack {
            let archive=pack_archive(&state,pack).await?;
            if pack.source=="modrinth" {super::instances::instance_import_mrpack_core(Some(app.clone()),&state,archive,name.clone(),None,Some(4096)).await?}
            else {super::instances::instance_import_modpack_core(Some(app.clone()),&state,archive,name.clone(),recipe.mc_version.clone(),recipe.loader.clone(),None,Some(4096)).await?}
        } else {
            let input=serde_json::from_value(json!({"name":name,"mcVersion":recipe.mc_version,"loader":recipe.loader,"loaderVersion":recipe.loader_version,"instanceGroup":"Salas","ramMb":4096}))?;
            super::profiles::profiles_create(input).await?
        };
        if row.mc_version!=recipe.mc_version || row.loader!=recipe.loader {
            return Err(AppError::InvalidState(format!("Instância {} preservada para revisão: Minecraft ou loader do pack diferem da sala.",row.name)));
        }
        if row.loader_version!=recipe.loader_version {
            row.loader_version=recipe.loader_version.clone();
            let database=crate::db::shared_db().await?;
            crate::db::schema::profiles::upsert(&database,&row).await?;
        }
        for (index,item) in recipe.mods.iter().enumerate() {
            crate::core::mods::pack_download::cancelled(Some(&state.import_cancel))?;
            let current=super::mods::mods_list(row.id.clone()).await?;
            if !current.iter().any(|installed| installed.project_id==item.project_id && installed.version_id==item.version_id && installed.source==item.source) {
                super::mods::mods_install_core(&state,super::mods::ModInstallRequest {profile_id:row.id.clone(),project_id:item.project_id.clone(),version_id:item.version_id.clone(),source:item.source.clone(),content_type:Some("mod".into()),world_name:None,icon_url:None}).await?;
            }
            let percent=10+(index+1)*75/recipe.mods.len().max(1);
            tunnel_set_preparation(update("downloading",percent as u8,"")).await?;
            let _=app.emit("room-preparation-progress",json!({"percent":percent,"name":name,"profileId":row.id}));
        }
        tunnel_set_preparation(update("validating",90,"")).await?;
        let local=super::experience::instance_compatibility(row.id.clone()).await?;
        if local.mod_fingerprint!=recipe.fingerprint || local.mc_version!=recipe.mc_version || local.loader!=recipe.loader {return Err(AppError::InvalidState(format!("Instância {} preservada para revisão: os arquivos diferem da sala. Confira mods locais ou personalizados do anfitrião.",row.name)));}
        let graph=dependency_graph(row.id.clone()).await?;
        let dependency_warnings=graph.iter().filter(|node|!node.missing.is_empty()).count();
        let mut row=profile(&row.id).await?;
        if dependency_warnings>0 {row.notes=Some(format!("Receita da sala: {dependency_warnings} avisos de metadados de dependência. Consulte o mapa e o diagnóstico do loader antes de jogar."));}
        row.instance_group=Some("Salas".into());
        let database=crate::db::shared_db().await?;
        crate::db::schema::profiles::upsert(&database,&row).await?;
        if let Some(pack)=&recipe.pack {
            let path=Path::new(&row.game_dir).join(if pack.source=="modrinth"{"modrinth.index.json"}else{"manifest.json"});
            let mut value:Value=serde_json::from_slice(&tokio::fs::read(&path).await?)?;
            value[if pack.source=="modrinth"{"projectId"}else{"projectID"}]=if pack.source=="modrinth"{json!(pack.project_id)}else{json!(pack.project_id.parse::<u64>().map_err(|_|AppError::InvalidInput("Projeto CurseForge inválido".into()))?)};
            value["sourceVersionId"]=json!(pack.version_id);
            crate::core::mods::pack_download::atomic_write(&path,&serde_json::to_vec(&value)?).await?;
        }
        crate::core::launcher::launch_state::store(&row)?;
        tunnel_set_preparation(update("ready",100,&local.mod_fingerprint)).await?;
        Ok(row)
    }.await;
    if result.is_err(){let _=tunnel_set_preparation(update("failed",0,"")).await;}
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn references_reject_urls_and_traversal() {
        let mut value = PackReference {source:"modrinth".into(),project_id:"project".into(),version_id:"version".into(),name:"Pack".into()};
        assert!(valid_reference(&value)); value.version_id="../token".into(); assert!(!valid_reference(&value));
    }
    #[test]
    fn dependencies_and_removal_impact_are_read_from_real_archives() {
        use std::io::Write;
        let root = std::env::temp_dir().join(format!("luxmc-graph-{}",uuid::Uuid::new_v4()));
        std::fs::create_dir_all(root.join("mods")).unwrap();
        for (name,value) in [("api",json!({"id":"api","version":"1"})),("client",json!({"id":"client","version":"1","depends":{"api":"*","minecraft":"*","missing":"*"}}))] {
            let mut zip = zip::ZipWriter::new(std::fs::File::create(root.join("mods").join(format!("{name}.jar"))).unwrap());
            zip.start_file("fabric.mod.json",zip::write::FileOptions::default()).unwrap(); zip.write_all(value.to_string().as_bytes()).unwrap(); zip.finish().unwrap();
        }
        let nodes = graph(&root).unwrap();
        assert_eq!(nodes.iter().find(|node| node.id=="api").unwrap().required_by,vec!["client"]);
        assert_eq!(nodes.iter().find(|node| node.id=="client").unwrap().missing,vec!["missing"]);
    }
}
