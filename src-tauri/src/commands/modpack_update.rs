use serde::{Deserialize, Serialize};
use tauri::State;

use crate::error::{AppError, AppResult};
use crate::state::AppState;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ModpackUpdateInfo {
    pub has_update: bool,
    pub current_version: Option<String>,
    pub current_version_id: Option<String>,
    pub latest_version: Option<String>,
    pub changelog: Option<String>,
    pub source: String,
    pub project_id: Option<String>,
    pub version_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ModpackVersionDiff {
    pub available: bool,
    pub added: Vec<String>,
    pub removed: Vec<String>,
    pub updated: Vec<String>,
    pub risks: Vec<String>,
    pub configuration: Vec<String>,
    pub dependencies: Vec<String>,
    pub target_minecraft: Option<String>,
    pub target_loader: Option<String>,
    pub worlds: usize,
}

fn modpack_entries(value: &serde_json::Value) -> std::collections::HashMap<String, String> {
    value
        .get("files")
        .and_then(serde_json::Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(|file| {
            let path = file.get("path")?.as_str()?.replace('\\', "/");
            let hash = file
                .get("hashes")
                .and_then(serde_json::Value::as_object)
                .and_then(|hashes| hashes.get("sha512").or_else(|| hashes.get("sha1")))
                .and_then(serde_json::Value::as_str)
                .unwrap_or_default()
                .to_string();
            Some((path, hash))
        })
        .collect()
}

fn display_mod_path(path: &str) -> String {
    std::path::Path::new(path)
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or(path)
        .to_string()
}

#[tauri::command]
#[allow(non_snake_case)]
pub async fn modpack_version_diff(
    state: State<'_, AppState>,
    profileId: String,
    versionId: String,
    source: String,
) -> AppResult<ModpackVersionDiff> {
    modpack_version_diff_core(&state, profileId, versionId, source).await
}

pub async fn modpack_version_diff_core(
    state: &AppState,
    profile_id: String,
    version_id: String,
    source: String,
) -> AppResult<ModpackVersionDiff> {
    use std::io::Read;
    use sha2::Digest;
    let row=super::studio::profile(&profile_id).await?;
    let reference=super::studio::PackReference {source:source.clone(),project_id:linked_project(&row.game_dir,&source).await?,version_id,name:row.name.clone()};
    let path=super::studio::pack_archive(state,&reference).await?;
    let root=std::path::PathBuf::from(&row.game_dir);
    let manifest_name=if source=="modrinth" {"modrinth.index.json"} else {"manifest.json"};
    let current:serde_json::Value=serde_json::from_slice(&tokio::fs::read(root.join(manifest_name)).await?)?;
    let mut archive=zip::ZipArchive::new(std::fs::File::open(path)?)?;
    let mut remote=None;let mut configuration=Vec::new();
    for index in 0..archive.len() {
        let mut entry=archive.by_index(index)?;
        let name=entry.name().replace('\\',"/");
        if name.rsplit('/').next()==Some(manifest_name) {
            if entry.size()>8*1024*1024{return Err(AppError::InvalidInput("Manifesto remoto muito grande".into()));}
            let mut bytes=Vec::new();entry.by_ref().take(8*1024*1024).read_to_end(&mut bytes)?;remote=Some(serde_json::from_slice::<serde_json::Value>(&bytes)?);
        }
        if let Some(relative)=name.strip_prefix("overrides/").or_else(|| name.strip_prefix("client-overrides/")) {
            if entry.is_dir() || !relative.starts_with("config/") {continue;}
            crate::core::mods::pack_download::relative_path(relative)?;
            if configuration.len()>=5000{return Err(AppError::InvalidInput("O pack tem configuraÃ§Ãµes demais para planejar".into()));}
            let local=root.join(relative);
            let changed=if entry.size()<=2*1024*1024 && local.is_file() && std::fs::metadata(&local)?.len()<=2*1024*1024 && std::fs::canonicalize(&local)?.starts_with(std::fs::canonicalize(&root)?) {
                let mut bytes=Vec::new();entry.by_ref().take(2*1024*1024).read_to_end(&mut bytes)?;
                sha2::Sha256::digest(&bytes)!=sha2::Sha256::digest(std::fs::read(&local)?)
            }else{true};
            if changed{configuration.push(relative.into());}
        }
    }
    let remote=remote.ok_or_else(|| AppError::NotFound("Manifesto remoto nÃ£o encontrado".into()))?;
    let entries=|value:&serde_json::Value| if source=="modrinth" {modpack_entries(value)} else {value["files"].as_array().into_iter().flatten().filter_map(|file|Some((format!("CurseForge #{}",file["projectID"].as_u64()?),file["fileID"].as_u64()?.to_string()))).collect()};
    let old=entries(&current);let new=entries(&remote);
    let mut added:Vec<_>=new.keys().filter(|path| !old.contains_key(*path)).map(|path|display_mod_path(path)).collect();
    let mut removed:Vec<_>=old.keys().filter(|path| !new.contains_key(*path)).map(|path|display_mod_path(path)).collect();
    let mut updated:Vec<_>=new.iter().filter(|(path,hash)|old.get(*path).is_some_and(|previous|previous!=*hash)).map(|(path,_)|display_mod_path(path)).collect();
    added.sort();removed.sort();updated.sort();configuration.sort();configuration.dedup();
    let target_minecraft=remote.pointer("/dependencies/minecraft").or_else(||remote.pointer("/minecraft/version")).and_then(|value|value.as_str()).map(str::to_owned);
    let target_loader=if source=="modrinth" {[("fabric-loader","fabric"),("quilt-loader","quilt"),("forge","forge"),("neoforge","neoforge")].into_iter().find(|(key,_)|remote["dependencies"].get(*key).is_some()).map(|(_,name)|name.to_owned()).or_else(||Some("vanilla".into()))} else {remote.pointer("/minecraft/modLoaders").and_then(|value|value.as_array()).and_then(|items|items.first()).and_then(|item|item["id"].as_str()).map(|id|id.split('-').next().unwrap_or("vanilla").to_owned())};
    let worlds=std::fs::read_dir(root.join("saves")).map(|entries|entries.flatten().filter(|entry|entry.path().join("level.dat").is_file()).count()).unwrap_or(0);
    let mut risks=Vec::new();
    if target_minecraft.as_deref().is_some_and(|target|target!=row.mc_version){risks.push("A versÃ£o do Minecraft muda. FaÃ§a uma cÃ³pia antes de abrir mundos existentes; voltar de versÃ£o pode perder blocos e dados.".into());}
    if target_loader.as_deref().is_some_and(|target|target!=row.loader){risks.push("O loader muda. Mods extras da instÃ¢ncia precisam ser revistos.".into());}
    if !removed.is_empty(){risks.push("Arquivos sÃ£o removidos. ConteÃºdo salvo por esses mods pode deixar de existir nos mundos.".into());}
    if !configuration.is_empty(){risks.push("O pack altera configuraÃ§Ãµes. Revise ajustes pessoais depois de atualizar.".into());}
    let graph=super::studio::dependency_graph(profile_id).await?;
    let mut dependencies:Vec<String>=graph.iter().filter(|node|removed.contains(&node.file)).flat_map(|node|node.required_by.iter().map(move |dependent|format!("{dependent} depende de {}",node.name))).collect();
    dependencies.extend(graph.iter().filter(|node|!node.missing.is_empty()).map(|node|format!("{}: revisar {}",node.name,node.missing.join(", "))));
    Ok(ModpackVersionDiff {available:true,added,removed,updated,risks,configuration,dependencies,target_minecraft,target_loader,worlds})
}

async fn linked_project(directory:&str,source:&str) -> AppResult<String> {
    let file=if source=="modrinth" {"modrinth.index.json"} else if source=="curseforge" {"manifest.json"} else {return Err(AppError::InvalidInput("Provedor invÃ¡lido".into()));};
    let value:serde_json::Value=serde_json::from_slice(&tokio::fs::read(std::path::Path::new(directory).join(file)).await?)?;
    value[if source=="modrinth" {"projectId"} else {"projectID"}].as_str().map(str::to_owned).or_else(||value["projectID"].as_u64().map(|id|id.to_string())).ok_or_else(||AppError::InvalidInput("Vincule a instÃ¢ncia ao projeto antes de planejar".into()))
}

#[tauri::command]
#[allow(non_snake_case)]
pub async fn modpack_check_update(
    state: State<'_, AppState>,
    profileId: String,
) -> AppResult<ModpackUpdateInfo> {
    modpack_check_update_core(&state,profileId).await
}

#[allow(non_snake_case)]
pub async fn modpack_check_update_core(state:&AppState,profileId:String) -> AppResult<ModpackUpdateInfo> {
    let db = crate::db::shared_db().await?;
    let row = sqlx::query_as::<_, crate::db::models::ProfileRow>(
        "SELECT * FROM profiles WHERE id = ?",
    )
    .bind(&profileId)
    .fetch_optional(db.pool())
    .await?
    .ok_or_else(|| AppError::NotFound(format!("profile {profileId} not found")))?;

    let manifest_path = std::path::PathBuf::from(&row.game_dir).join("manifest.json");
    let mrpack_path = std::path::PathBuf::from(&row.game_dir).join("modrinth.index.json");

    if mrpack_path.exists() {
        return check_modrinth_update(&state, &row, &mrpack_path).await;
    }

    if manifest_path.exists() {
        return check_curseforge_update(&state, &row, &manifest_path).await;
    }

    Ok(ModpackUpdateInfo {
        has_update: false,
        current_version: None,
        current_version_id: None,
        latest_version: None,
        changelog: None,
        source: "none".into(),
        project_id: None,
        version_id: None,
    })
}

fn release_date(value: &serde_json::Value, field: &str) -> i64 {
    value.get(field).and_then(|value| value.as_str()).and_then(|value| chrono::DateTime::parse_from_rfc3339(value).ok()).map(|date| date.timestamp_millis()).unwrap_or(0)
}

fn matching_release(value: &serde_json::Value, source: &str, minecraft: &str, loader: &str) -> bool {
    if source == "curseforge" && (value["isServerPack"] == true || value["isAvailable"] == false) { return false; }
    let field = if source == "modrinth" { "game_versions" } else { "gameVersions" };
    let Some(games) = value.get(field).and_then(|value| value.as_array()) else { return false; };
    if !games.iter().any(|value| value.as_str() == Some(minecraft)) { return false; }
    let loaders = if source == "modrinth" { value.get("loaders").and_then(|value| value.as_array()).unwrap_or(games) } else { games };
    let declared: Vec<_> = loaders.iter().filter_map(|value| value.as_str()).map(str::to_lowercase).filter(|value| matches!(value.as_str(), "forge" | "neoforge" | "fabric" | "quilt")).collect();
    declared.is_empty() || declared.iter().any(|value| value == &loader.to_lowercase())
}

fn is_newer_release(latest: &serde_json::Value, current: Option<&serde_json::Value>, installed_id: Option<&str>, installed_number: Option<&str>, source: &str) -> bool {
    let id = if source == "modrinth" { latest["id"].as_str().map(str::to_owned) } else { latest["id"].as_u64().map(|id| id.to_string()) };
    if installed_id.is_some() && installed_id == id.as_deref() { return false; }
    if installed_id.is_none() && installed_number.is_some() && installed_number == latest["version_number"].as_str() { return false; }
    let Some(current) = current else { return false; };
    let field = if source == "modrinth" { "date_published" } else { "fileDate" };
    let before = release_date(current, field);
    before > 0 && release_date(latest, field) > before
}

async fn check_modrinth_update(state: &AppState, row: &crate::db::models::ProfileRow, path: &std::path::Path) -> AppResult<ModpackUpdateInfo> {
    let index: serde_json::Value = serde_json::from_slice(&tokio::fs::read(path).await?)?;
    let project = index["projectId"].as_str().filter(|id| !id.is_empty() && id.chars().all(|c| c.is_ascii_alphanumeric() || c == '-'));
    let pinned = index["sourceVersionId"].as_str();
    let number = index["versionId"].as_str();
    let mut result = ModpackUpdateInfo { has_update: false, current_version: number.or(pinned).map(str::to_owned), current_version_id: pinned.map(str::to_owned), latest_version: None, changelog: None, source: "modrinth".into(), project_id: project.map(str::to_owned), version_id: None };
    let Some(project) = project else { return Ok(result); };
    let url = format!("https://api.modrinth.com/v2/project/{project}/version?game_versions={}&loaders={}", urlencoding::encode(&serde_json::to_string(&[&row.mc_version])?), urlencoding::encode(&serde_json::to_string(&[row.loader.to_lowercase()])?));
    let versions: Vec<serde_json::Value> = state.http.get(url).timeout(std::time::Duration::from_secs(15)).send().await?.error_for_status()?.json().await?;
    let current = versions.iter().find(|value| pinned.is_some() && value["id"].as_str() == pinned || pinned.is_none() && value["version_number"].as_str() == number);
    let latest = versions.iter().filter(|value| matching_release(value, "modrinth", &row.mc_version, &row.loader)).max_by_key(|value| release_date(value, "date_published"));
    if let Some(latest) = latest {
        result.has_update = is_newer_release(latest, current, pinned, number, "modrinth");
        result.latest_version = latest["version_number"].as_str().map(str::to_owned);
        result.version_id = latest["id"].as_str().map(str::to_owned);
        result.changelog = latest["changelog"].as_str().map(str::to_owned);
    }
    Ok(result)
}

async fn check_curseforge_update(state: &AppState, row: &crate::db::models::ProfileRow, path: &std::path::Path) -> AppResult<ModpackUpdateInfo> {
    let index: serde_json::Value = serde_json::from_slice(&tokio::fs::read(path).await?)?;
    let project = index["projectID"].as_u64();
    let pinned = index["sourceVersionId"].as_str();
    let mut result = ModpackUpdateInfo { has_update: false, current_version: index["version"].as_str().map(str::to_owned), current_version_id: pinned.map(str::to_owned), latest_version: None, changelog: None, source: "curseforge".into(), project_id: project.map(|id| id.to_string()), version_id: None };
    let (Some(project), Some(key)) = (project, crate::core::mods::curseforge::api_key()) else { return Ok(result); };
    let url = format!("https://api.curseforge.com/v1/mods/{project}/files?pageSize=50&gameVersion={}", urlencoding::encode(&row.mc_version));
    let body: serde_json::Value = state.http.get(url).header("x-api-key", &key).timeout(std::time::Duration::from_secs(15)).send().await?.error_for_status()?.json().await?;
    let files = body["data"].as_array().map(Vec::as_slice).unwrap_or_default();
    let current = files.iter().find(|file| file["id"].as_u64().map(|id| id.to_string()).as_deref() == pinned);
    let latest = files.iter().filter(|value| matching_release(value, "curseforge", &row.mc_version, &row.loader)).max_by_key(|value| release_date(value, "fileDate"));
    if let Some(latest) = latest {
        result.has_update = is_newer_release(latest, current, pinned, None, "curseforge");
        if current.is_none() {
            if let (Some(before), Some(after)) = (pinned.and_then(|id| id.parse::<u64>().ok()), latest["id"].as_u64()) { result.has_update = after > before; }
        }
        result.latest_version = latest["displayName"].as_str().map(str::to_owned);
        result.version_id = latest["id"].as_u64().map(|id| id.to_string());
    }
    Ok(result)
}

static UPDATE_LOCK: tokio::sync::Mutex<()> = tokio::sync::Mutex::const_new(());

#[tauri::command]
#[allow(non_snake_case)]
pub async fn modpack_update_atomic(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
    profileId: String,
    versionId: String,
    source: String,
    projectId: String,
) -> AppResult<()> {
    modpack_update_atomic_core(Some(app),&state,profileId,versionId,source,projectId).await
}

#[allow(non_snake_case)]
pub async fn modpack_update_atomic_core(app:Option<tauri::AppHandle>,state:&AppState,profileId:String,versionId:String,source:String,projectId:String) -> AppResult<()> {
    super::instances::instance_check_import_job(&state,None).await?;
    use crate::core::mods::pack_download as pack;
    use crate::commands::instances::{instance_import_modpack_core, instance_import_mrpack_core};
    use tauri::Emitter;
    let _launch_guard = state.launch_lock.try_lock().map_err(|_|AppError::InvalidState("Aguarde o lanÃ§amento do Minecraft terminar".into()))?;
    if crate::core::launcher::get_active_game_pid()!=0 {return Err(AppError::InvalidState("Feche o Minecraft antes de atualizar o modpack".into()));}
    let _guard = UPDATE_LOCK.try_lock().map_err(|_| AppError::InvalidState("Uma atualizaÃ§Ã£o jÃ¡ estÃ¡ em andamento".into()))?;
    let operation_id = super::instances::instance_begin_import_core(state).await?;
    let result: AppResult<()> = async {
    if !["modrinth", "curseforge"].contains(&source.as_str()) ||
        [&versionId, &projectId].iter().any(|id| id.is_empty() || !id.chars().all(|c| c.is_ascii_alphanumeric() || c == '-')) {
        return Err(AppError::InvalidInput("Identificador de atualizaÃ§Ã£o invÃ¡lido".into()));
    }
    let db = crate::db::shared_db().await?;
    let row = sqlx::query_as::<_, crate::db::models::ProfileRow>("SELECT * FROM profiles WHERE id = ?")
        .bind(&profileId).fetch_optional(db.pool()).await?
        .ok_or_else(|| AppError::NotFound(format!("profile {profileId} not found")))?;
    let original = std::path::PathBuf::from(&row.game_dir);
    if linked_project(&row.game_dir,&source).await? != projectId { return Err(AppError::InvalidInput("A atualizaÃ§Ã£o pertence a outro modpack. A instÃ¢ncia atual foi mantida.".into())); }
    let saves_dir = original.join("saves");
    if saves_dir.is_dir() {
        if let Some(app)=&app {let _ = app.emit("modpack-progress", serde_json::json!({"phase":"backup","current":0,"total":0,"percent":2,"status":"Criando backups versionados dos mundos antes da atualizaÃ§Ã£o..."}));}
        for entry in std::fs::read_dir(&saves_dir)? {
            let entry = entry?;
            if entry.file_type()?.is_dir() {
                let world = entry.file_name().to_string_lossy().to_string();
                crate::commands::world_backup::instance_backup_world_core(profileId.clone(), world).await?;
            }
        }
    }
    let managed = owned_pack_paths(&original).await?;
    let client = pack::client()?;
    if let Some(app)=&app {let _ = app.emit("modpack-progress", serde_json::json!({"phase":"downloading","current":0,"total":0,"percent":5,"status":"Preparando nova versÃ£o sem alterar a instÃ¢ncia atual..."}));}
    let (urls, size, sha1, sha512) = if source == "modrinth" {
        let version: serde_json::Value = client.get(format!("https://api.modrinth.com/v2/version/{versionId}"))
            .send().await?.error_for_status()?.json().await?;
        if version.get("project_id").and_then(|v| v.as_str()) != Some(projectId.as_str()) {
            return Err(AppError::InvalidInput("A versÃ£o nÃ£o pertence ao modpack selecionado".into()));
        }
        let files = version.get("files").and_then(|v| v.as_array()).ok_or_else(|| AppError::NotFound("Arquivos da versÃ£o ausentes".into()))?;
        let file = files.iter().find(|v| v.get("primary").and_then(|v| v.as_bool()) == Some(true)).or_else(|| files.first())
            .ok_or_else(|| AppError::NotFound("Arquivo mrpack ausente".into()))?;
        let url = file.get("url").and_then(|v| v.as_str()).ok_or_else(|| AppError::NotFound("URL do modpack ausente".into()))?;
        (pack::mirrors(url)?, file.get("size").and_then(|v| v.as_u64()),
         file.pointer("/hashes/sha1").and_then(|v| v.as_str()).map(str::to_owned),
         file.pointer("/hashes/sha512").and_then(|v| v.as_str()).map(str::to_owned))
    } else {
        let file_id = versionId.parse::<u64>().map_err(|_| AppError::InvalidInput("ID CurseForge invÃ¡lido".into()))?;
        let info = crate::core::mods::curseforge::get_files_batch(&client, &[file_id]).await.remove(&file_id)
            .ok_or_else(|| AppError::NotFound("Metadados CurseForge indisponÃ­veis; verifique a chave API".into()))?;
        if info.mod_id.to_string() != projectId { return Err(AppError::InvalidInput("Projeto CurseForge incorreto".into())); }
        let mut urls = info.download_url.as_deref().map(pack::mirrors).transpose()?.unwrap_or_default();
        for host in ["edge.forgecdn.net", "mediafilez.forgecdn.net", "media.forgecdn.net"] {
            urls.push(format!("https://{host}/files/{}/{}/{}", file_id / 1000, file_id % 1000, urlencoding::encode(&info.file_name)));
        }
        (urls, info.size, info.sha1, None)
    };
    let mut archive = None;
    for attempt in 0..3 {
        if attempt > 0 { tokio::time::sleep(std::time::Duration::from_millis(500 * (1 << (attempt - 1)))).await; }
        for url in &urls {
            pack::cancelled(Some(&state.import_cancel))?;
            if let Ok(bytes) = pack::bytes(&client, url, Some(&state.import_cancel)).await {
                if pack::verify(&bytes, size, sha1.as_deref(), sha512.as_deref()) && zip::ZipArchive::new(std::io::Cursor::new(&bytes)).is_ok() {
                    archive = Some(bytes); break;
                }
            }
        }
        if archive.is_some() { break; }
    }
    let bytes = archive.ok_or_else(|| AppError::InvalidState("NÃ£o foi possÃ­vel baixar e validar a atualizaÃ§Ã£o".into()))?;
    let archive_path = std::env::temp_dir().join(format!("luxmc-update-{}.zip", uuid::Uuid::new_v4()));
    pack::atomic_write(&archive_path, &bytes).await?;
    let imported = if source == "modrinth" {
        instance_import_mrpack_core(app.clone(), &state, archive_path.to_string_lossy().into_owned(), row.name.clone(), Some(row.icon.clone()), row.ram_mb).await
    } else {
        instance_import_modpack_core(app.clone(), &state, archive_path.to_string_lossy().into_owned(), row.name.clone(), row.mc_version.clone(), row.loader.clone(), Some(row.icon.clone()), row.ram_mb).await
    };
    let _ = tokio::fs::remove_file(&archive_path).await;
    let staged = imported?;
    let staged_dir = std::path::PathBuf::from(&staged.game_dir);
    let result: AppResult<()> = async {
        let manifest_path = staged_dir.join(if source == "modrinth" { "modrinth.index.json" } else { "manifest.json" });
        let mut manifest: serde_json::Value = serde_json::from_slice(&tokio::fs::read(&manifest_path).await?)?;
        if source == "modrinth" {
            manifest["projectId"] = serde_json::json!(projectId);
            manifest["sourceVersionId"] = serde_json::json!(versionId);
        } else {
            manifest["projectID"] = serde_json::json!(projectId.parse::<u64>().map_err(|_| AppError::InvalidInput("ID CurseForge invÃ¡lido".into()))?);
            manifest["sourceVersionId"] = serde_json::json!(versionId);
        }
        pack::atomic_write(&manifest_path, &serde_json::to_vec_pretty(&manifest)?).await?;
        let old = original.clone();
        let new = staged_dir.clone();
        let owned = managed.clone();
        let cancel = state.import_cancel.clone();
        let mod_count = tokio::task::spawn_blocking(move || -> AppResult<i64> {
            preserve_user_files(&old, &new, &owned)?;
            preserve_disabled_mods(&old,&new,&owned)?;
            let audit = crate::core::mods::validation::audit_cancellable(&new,false,Some(&cancel))?;
            if !audit.errors.is_empty() { return Err(AppError::InvalidInput(format!("A nova versÃ£o conflita com os arquivos preservados. A instÃ¢ncia anterior foi mantida.\n{}",audit.errors.join("\n")))); }
            let mut count = 0;
            for entry in std::fs::read_dir(new.join("mods"))? {
                let entry = entry?;
                if entry.file_type()?.is_file() && entry.path().extension().is_some_and(|extension|extension.eq_ignore_ascii_case("jar")) { count += 1; }
            }
            Ok(count)
        }).await
            .map_err(|error| AppError::Internal(error.to_string()))??;
        pack::cancelled(Some(&state.import_cancel))?;
        pack::atomic_write(&staged_dir.join(".luxmc/profile-owner.json"),&serde_json::to_vec(&serde_json::json!({"profileId":profileId}))?).await?;
        let mut tx = db.pool().begin().await?;
        sqlx::query("DELETE FROM mods WHERE profile_id = ? AND project_id IN (SELECT project_id FROM mods WHERE profile_id = ?)").bind(&profileId).bind(&staged.id).execute(&mut *tx).await?;
        for path in &managed {
            if let Some(name) = path.strip_prefix("mods/") {
                sqlx::query("DELETE FROM mods WHERE profile_id = ? AND file_name = ?").bind(&profileId).bind(name).execute(&mut *tx).await?;
            }
        }
        sqlx::query("UPDATE mods SET profile_id = ? WHERE profile_id = ?").bind(&profileId).bind(&staged.id).execute(&mut *tx).await?;
        sqlx::query("UPDATE profiles SET game_dir = ?, mc_version = ?, loader = ?, loader_version = ?, mod_count = ?, disk_usage = 0, updated_at = ? WHERE id = ?")
            .bind(&staged.game_dir).bind(&staged.mc_version).bind(&staged.loader).bind(&staged.loader_version)
            .bind(mod_count).bind(chrono::Utc::now()).bind(&profileId).execute(&mut *tx).await?;
        sqlx::query("DELETE FROM profiles WHERE id = ?").bind(&staged.id).execute(&mut *tx).await?;
        tx.commit().await?;
        Ok(())
    }.await;
    if let Err(error) = result {
        sqlx::query("DELETE FROM profiles WHERE id = ?").bind(&staged.id).execute(db.pool()).await?;
        return Err(error);
    }
    if let Some(app)=&app {let _ = app.emit("modpack-progress", serde_json::json!({"phase":"complete","current":1,"total":1,"percent":100,"status":"Modpack atualizado; a pasta anterior foi preservada como backup.","backupPath":original,"projectId":projectId}));}
    Ok(())
    }.await;
    let finished = super::instances::instance_end_import_core(state,operation_id).await;
    result?;
    finished
}

async fn owned_pack_paths(root: &std::path::Path) -> AppResult<std::collections::HashSet<String>> {
    use crate::core::mods::pack_download as pack;
    let mut owned = std::collections::HashSet::new();
    for manifest in ["modrinth.index.json", ".luxmc/overrides.json"] {
        let path = root.join(manifest);
        if !path.exists() { continue; }
        let value: serde_json::Value = serde_json::from_slice(&tokio::fs::read(path).await?)?;
        let files = value.as_array().or_else(|| value.get("files").and_then(|v| v.as_array()));
        if let Some(files) = files {
            for file in files {
                if let Some(path) = file.get("path").and_then(|v| v.as_str()) {
                    pack::relative_path(path)?;
                    owned.insert(path.replace('\\', "/"));
                }
            }
        }
    }
    let cf = root.join("manifest.json");
    if cf.exists() {
        let manifest: crate::commands::instances::CfManifest = serde_json::from_slice(&tokio::fs::read(cf).await?)?;
        for file in manifest.files {
            let data = tokio::fs::read(root.join(format!(".luxmc/curseforge/{}.json", file.file_id))).await
                .map_err(|_| AppError::InvalidState("Repare o modpack antes de atualizar: faltam metadados exatos dos mods antigos".into()))?;
            let info: crate::core::mods::curseforge::CurseForgeFileInfo = serde_json::from_slice(&data)?;
            if info.id != file.file_id || info.mod_id != file.project_id || info.file_name.contains(['/', '\\']) {
                return Err(AppError::InvalidState("Metadados CurseForge inconsistentes".into()));
            }
            let path = format!("mods/{}", info.file_name);
            pack::relative_path(&path)?;
            owned.insert(path);
        }
    }
    Ok(owned)
}

fn preserve_user_files(old: &std::path::Path, new: &std::path::Path, managed: &std::collections::HashSet<String>) -> AppResult<()> {
    use crate::core::mods::pack_download as pack;
    pack::destination(old,"saves")?;
    if old.join("saves").is_dir() {
        for world in std::fs::read_dir(old.join("saves"))? {
            let world = world?;
            if !world.file_type()?.is_dir() { continue; }
            let name = world.file_name().to_string_lossy().into_owned();
            pack::destination(old,&format!("saves/{name}"))?;
            let target = pack::destination(new,&format!("saves/{name}"))?;
            if target.exists() {
                let backup = pack::destination(new,&format!(".luxmc/update-default-worlds/{}/{name}",uuid::Uuid::new_v4()))?;
                std::fs::create_dir_all(backup.parent().unwrap())?;
                std::fs::rename(target,backup)?;
            }
        }
    }
    let mut pending = vec![old.to_path_buf()];
    while let Some(dir) = pending.pop() {
        for entry in std::fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();
            let relative = path.strip_prefix(old).map_err(|error| AppError::Internal(error.to_string()))?.to_string_lossy().replace('\\', "/");
            if relative == ".luxmc" || ["manifest.json", "modrinth.index.json"].contains(&relative.as_str()) { continue; }
            if pack::metadata_is_link(&std::fs::symlink_metadata(&path)?) { return Err(AppError::InvalidInput(format!("NÃ£o Ã© possÃ­vel copiar link simbÃ³lico: {relative}"))); }
            let target = pack::destination(new, &relative)?;
            if entry.file_type()?.is_dir() { std::fs::create_dir_all(&target)?; pending.push(path); continue; }
            let first = relative.split('/').next().unwrap_or_default();
            if ["saves","screenshots","options.txt","optionsof.txt","servers.dat","servers.dat_old"].contains(&first) {
                std::fs::copy(path,target)?;
                continue;
            }
            let enabled = relative.strip_suffix(".disabled").unwrap_or(&relative);
            if managed.contains(enabled) {
                if relative.ends_with(".disabled") {
                    let candidate = pack::destination(new, enabled)?;
                    if candidate.is_file() { std::fs::rename(candidate, target)?; }
                }
                continue;
            }
            let personal = ["saves", "screenshots", "options.txt", "servers.dat", "resourcepacks", "shaderpacks"].contains(&first);
            if personal || !target.exists() { std::fs::copy(path, target)?; }
        }
    }
    Ok(())
}

fn preserve_disabled_mods(old: &std::path::Path, new: &std::path::Path, managed: &std::collections::HashSet<String>) -> AppResult<()> {
    use crate::core::mods::{pack_download as pack,validation};
    let mut disabled = std::collections::BTreeSet::new();
    for path in managed.iter().filter(|path|path.starts_with("mods/") && path.ends_with(".jar")) {
        let enabled = pack::destination(old,path)?;
        let inactive = pack::destination(old,&format!("{path}.disabled"))?;
        if !enabled.exists() && inactive.is_file() {
            if let Some(ids) = validation::primary_ids(&inactive) { disabled.insert(ids); }
        }
    }
    if disabled.is_empty() || !new.join("mods").is_dir() { return Ok(()); }
    for entry in std::fs::read_dir(new.join("mods"))? {
        let entry = entry?;
        if !entry.file_type()?.is_file() || !entry.path().extension().is_some_and(|extension|extension.eq_ignore_ascii_case("jar")) { continue; }
        if validation::primary_ids(&entry.path()).is_some_and(|ids|disabled.contains(&ids)) {
            let name = entry.file_name().to_string_lossy().into_owned();
            let source = pack::destination(new,&format!("mods/{name}"))?;
            let target = pack::destination(new,&format!("mods/{name}.disabled"))?;
            if target.exists() { return Err(AppError::InvalidState(format!("Conflito ao preservar o mod desativado: {name}"))); }
            std::fs::rename(source,target)?;
        }
    }
    Ok(())
}

#[cfg(test)]
mod update_tests {
    use super::*;
    #[tokio::test]
    async fn failed_update_releases_its_reservation_without_clearing_another_job() {
        let state = AppState::default();
        let result = modpack_update_atomic_core(None,&state,"profile".into(),"version".into(),"invalid".into(),"project".into()).await;
        assert!(result.is_err());
        assert!(!state.import_job.load(std::sync::atomic::Ordering::SeqCst));
        assert!(state.import_job_id.lock().await.is_empty());
        let operation = super::super::instances::instance_begin_import_core(&state).await.unwrap();
        assert!(modpack_update_atomic_core(None,&state,"profile".into(),"version".into(),"modrinth".into(),"project".into()).await.is_err());
        assert!(state.import_job.load(std::sync::atomic::Ordering::SeqCst));
        assert_eq!(*state.import_job_id.lock().await,operation);
        super::super::instances::instance_end_import_core(&state,operation).await.unwrap();
    }

    #[test]
    fn preserves_user_data_and_exact_mod_ownership_without_touching_original() {
        let root = std::env::temp_dir().join(format!("luxmc-update-test-{}", uuid::Uuid::new_v4()));
        let old = root.join("old"); let new = root.join("new");
        std::fs::create_dir_all(old.join("mods")).unwrap(); std::fs::create_dir_all(new.join("mods")).unwrap();
        for (path, value) in [(old.join("mods/example-1.jar"), "old"), (old.join("mods/example-extra.jar"), "user"), (old.join("options.txt"), "settings"), (new.join("mods/example-2.jar"), "new"), (new.join("options.txt"), "pack")] {
            std::fs::write(path, value).unwrap();
        }
        preserve_user_files(&old, &new, &std::collections::HashSet::from(["mods/example-1.jar".into(),"options.txt".into()])).unwrap();
        assert!(!new.join("mods/example-1.jar").exists());
        assert_eq!(std::fs::read_to_string(new.join("mods/example-extra.jar")).unwrap(), "user");
        assert_eq!(std::fs::read_to_string(new.join("options.txt")).unwrap(), "settings");
        assert_eq!(std::fs::read_to_string(old.join("mods/example-1.jar")).unwrap(), "old");
        std::fs::remove_dir_all(root).unwrap();
    }

    fn mod_jar(path: &std::path::Path, id: &str, version: &str) {
        use std::io::Write;
        let mut archive = zip::ZipWriter::new(std::fs::File::create(path).unwrap());
        archive.start_file("fabric.mod.json",zip::write::FileOptions::default()).unwrap();
        archive.write_all(serde_json::json!({"id":id,"version":version}).to_string().as_bytes()).unwrap();
        archive.finish().unwrap();
    }

    #[test]
    fn disabled_choice_survives_changed_filename_without_disabling_other_mods() {
        let root = tempfile::tempdir().unwrap();
        let old = root.path().join("old"); let new = root.path().join("new");
        std::fs::create_dir_all(old.join("mods")).unwrap(); std::fs::create_dir_all(new.join("mods")).unwrap();
        mod_jar(&old.join("mods/example-1.jar.disabled"),"example","1");
        mod_jar(&new.join("mods/example-2.jar"),"example","2");
        mod_jar(&new.join("mods/other.jar"),"other","1");
        let managed = std::collections::HashSet::from(["mods/example-1.jar".into()]);
        preserve_user_files(&old,&new,&managed).unwrap();
        preserve_disabled_mods(&old,&new,&managed).unwrap();
        assert!(!new.join("mods/example-2.jar").exists());
        assert!(new.join("mods/example-2.jar.disabled").is_file());
        assert!(new.join("mods/other.jar").is_file());
        assert!(old.join("mods/example-1.jar.disabled").is_file());
    }

    #[test]
    fn preserves_world_without_merging_publisher_default_regions() {
        let root = tempfile::tempdir().unwrap();
        let old = root.path().join("old"); let new = root.path().join("new");
        std::fs::create_dir_all(old.join("saves/World/region")).unwrap();
        std::fs::create_dir_all(new.join("saves/World/region")).unwrap();
        std::fs::write(old.join("saves/World/region/user.mca"),b"original").unwrap();
        std::fs::write(new.join("saves/World/region/default.mca"),b"publisher").unwrap();
        preserve_user_files(&old,&new,&std::collections::HashSet::new()).unwrap();
        assert_eq!(std::fs::read(new.join("saves/World/region/user.mca")).unwrap(),b"original");
        assert!(!new.join("saves/World/region/default.mca").exists());
        let backup = std::fs::read_dir(new.join(".luxmc/update-default-worlds")).unwrap().next().unwrap().unwrap().path();
        assert_eq!(std::fs::read(backup.join("World/region/default.mca")).unwrap(),b"publisher");
        assert_eq!(std::fs::read(old.join("saves/World/region/user.mca")).unwrap(),b"original");
    }

    #[test]
    fn final_audit_detects_conflicts_from_preserved_personal_mods() {
        let root = tempfile::tempdir().unwrap();
        let old = root.path().join("old"); let new = root.path().join("new");
        std::fs::create_dir_all(old.join("mods")).unwrap(); std::fs::create_dir_all(new.join("mods")).unwrap();
        mod_jar(&old.join("mods/personal.jar"),"example","1");
        mod_jar(&new.join("mods/updated.jar"),"example","2");
        preserve_user_files(&old,&new,&std::collections::HashSet::new()).unwrap();
        let audit = crate::core::mods::validation::audit(&new,false).unwrap();
        assert!(!audit.errors.is_empty());
        assert!(old.join("mods/personal.jar").is_file());
    }
}

#[cfg(test)]
mod release_selection_tests {
    use super::*;
    #[test]
    fn newest_release_stays_current_and_old_responses_cannot_offer_a_downgrade() {
        let old = serde_json::json!({"id":"alpha","version_number":"15-alpha","date_published":"2026-10-04T12:00:00Z"});
        let new = serde_json::json!({"id":"beta","version_number":"15-beta","date_published":"2026-10-11T12:00:00Z"});
        assert!(!is_newer_release(&new,Some(&new),Some("beta"),Some("15-beta"),"modrinth"));
        assert!(!is_newer_release(&old,Some(&new),Some("beta"),Some("15-beta"),"modrinth"));
        assert!(is_newer_release(&new,Some(&old),Some("alpha"),Some("15-alpha"),"modrinth"));
        assert!(!is_newer_release(&new,None,None,Some("unrecognized"),"modrinth"));
        let revision=serde_json::json!({"id":"beta-revised","version_number":"15-beta","date_published":"2026-10-12T12:00:00Z"});
        assert!(is_newer_release(&revision,Some(&new),Some("beta"),Some("15-beta"),"modrinth"));
    }
    #[test]
    fn updates_match_minecraft_and_loader_instead_of_the_projects_other_editions() {
        let version=serde_json::json!({"game_versions":["26.3"],"loaders":["fabric"]});
        assert!(matching_release(&version,"modrinth","26.3","fabric"));
        assert!(!matching_release(&version,"modrinth","26.2","fabric"));
        assert!(!matching_release(&version,"modrinth","26.3","forge"));
        let version=serde_json::json!({"gameVersions":["1.21.1","NeoForge"]});
        assert!(matching_release(&version,"curseforge","1.21.1","neoforge"));
        assert!(!matching_release(&version,"curseforge","1.21.1","forge"));
        assert!(!matching_release(&version,"curseforge","1.20.1","neoforge"));
        let server=serde_json::json!({"gameVersions":["1.21.1","NeoForge"],"isServerPack":true});
        assert!(!matching_release(&server,"curseforge","1.21.1","neoforge"));
    }
}
