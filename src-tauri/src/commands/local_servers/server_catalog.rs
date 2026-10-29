use super::*;
use crate::core::{local_server::loaders::ServerLoader, mods::{curseforge, pack_download}};
use serde_json::{json, Value};
use std::{io::Read, path::Path};

static INSTALLING: tokio::sync::Mutex<()> = tokio::sync::Mutex::const_new(());

pub(super) async fn pending(http: &reqwest::Client, id: &str) -> AppResult<Value> {
    let root = manager(http)?.directory(id).await?;
    let path = root.join(".luxmc-catalog-request.json");
    if !path.exists() { return Ok(Value::Null); }
    let metadata = tokio::fs::symlink_metadata(&path).await?;
    if metadata.file_type().is_symlink() || metadata.len() > 4096 || !tokio::fs::canonicalize(&path).await?.starts_with(&root) { return Err(AppError::InvalidInput("Registro de preparação inválido.".into())); }
    let request:Value=serde_json::from_slice(&tokio::fs::read(&path).await?)?;
    let receipt=root.join(".luxmc-catalog.json");
    if receipt.exists() {
        let metadata=tokio::fs::symlink_metadata(&receipt).await?;
        if crate::core::mods::pack_download::metadata_is_link(&metadata) || metadata.len()>1024*1024 || !tokio::fs::canonicalize(&receipt).await?.starts_with(&root){return Err(AppError::InvalidInput("Registro de catálogo inválido.".into()));}
        let completed:Value=serde_json::from_slice(&tokio::fs::read(receipt).await?)?;
        if request_completed(&request,&completed){return Ok(Value::Null);}
    }
    Ok(request)
}

fn request_completed(request:&Value,receipt:&Value)->bool {
    receipt["pipelineVersion"]==2 && ["source","project","version"].iter().all(|key|request[*key].as_str().is_some() && request[*key]==receipt[*key])
}

fn valid_id(value: &str) -> AppResult<()> {
    if value.is_empty() || value.len() > 96 || !value.bytes().all(|byte| byte.is_ascii_alphanumeric() || b"_-".contains(&byte)) {
        return Err(AppError::InvalidInput("Identificador de catálogo inválido.".into()));
    }
    Ok(())
}

async fn metadata(http: &reqwest::Client, url: String, source: &str) -> AppResult<Value> {
    let mut request = http.get(url).timeout(std::time::Duration::from_secs(25));
    if source == "curseforge" { request = request.header("x-api-key", curseforge::api_key().ok_or_else(|| AppError::InvalidState("CurseForge indisponível. Configure a integração ou use Modrinth.".into()))?); }
    let mut response = request.send().await?.error_for_status()?;
    let mut bytes = Vec::new();
    while let Some(chunk) = response.chunk().await? {
        if bytes.len() + chunk.len() > 8 * 1024 * 1024 { return Err(AppError::InvalidInput("Catálogo muito grande.".into())); }
        bytes.extend_from_slice(&chunk);
    }
    Ok(serde_json::from_slice(&bytes)?)
}

async fn project(http: &reqwest::Client, project: &str, source: &str) -> AppResult<Value> {
    valid_id(project)?;
    match source {
        "modrinth" => {
            let value = metadata(http, format!("https://api.modrinth.com/v2/project/{project}"), source).await?;
            if value["project_type"] != "modpack" || value["server_side"] == "unsupported" {
                return Err(AppError::InvalidInput("Este projeto não oferece suporte a servidor. Escolha um modpack de servidor ou crie Vanilla separadamente.".into()));
            }
            Ok(value)
        }
        "curseforge" => {
            let value = metadata(http, format!("https://api.curseforge.com/v1/mods/{project}"), source).await?;
            if value["data"]["classId"] != 4471 { return Err(AppError::InvalidInput("Escolha um modpack Minecraft.".into())); }
            Ok(value["data"].clone())
        }
        _ => Err(AppError::InvalidInput("Provedor não suportado.".into())),
    }
}

pub(super) async fn versions(http: &reqwest::Client, id: &str, source: &str) -> AppResult<Value> {
    project(http, id, source).await?;
    let values = if source == "modrinth" {
        metadata(http, format!("https://api.modrinth.com/v2/project/{id}/version"), source).await?
    } else {
        metadata(http, format!("https://api.curseforge.com/v1/mods/{id}/files?pageSize=50&index=0"), source).await?["data"].clone()
    };
    let items = values.as_array().ok_or_else(|| AppError::InvalidInput("Resposta de catálogo inválida.".into()))?;
    Ok(Value::Array(items.iter().filter_map(|item| {
        if source == "modrinth" {
            let version = item["id"].as_str()?;
            if !item["files"].as_array()?.iter().any(|file| file["filename"].as_str().is_some_and(|name| name.ends_with(".mrpack"))) { return None; }
            Some(json!({"id":version,"name":item["name"],"gameVersions":item["game_versions"],"loaders":item["loaders"]}))
        } else {
            if item["isAvailable"] == false || item["isServerPack"] == true || item["serverPackFileId"].as_u64().unwrap_or(0) == 0 { return None; }
            let games = item["gameVersions"].as_array()?.iter().filter(|value| value.as_str().is_some_and(|value| value.starts_with(|c:char| c.is_ascii_digit()))).cloned().collect::<Vec<_>>();
            Some(json!({"id":item["id"].as_u64()?.to_string(),"name":item["displayName"],"gameVersions":games,"loaders":[]}))
        }
    }).take(100).collect()))
}

async fn download(http: &reqwest::Client, file: &Value, source: &str, path: &Path, cancelled: &std::sync::atomic::AtomicBool) -> AppResult<()> {
    let (url, size, hash) = if source == "modrinth" {
        (file["url"].as_str(), file["size"].as_u64(), file["hashes"]["sha1"].as_str())
    } else {
        (file["downloadUrl"].as_str(), file["fileLength"].as_u64(), file["hashes"].as_array().and_then(|values| values.iter().find(|value| value["algo"] == 1)).and_then(|value| value["value"].as_str()))
    };
    let _ = http;
    let (url, size, hash) = url.zip(size).zip(hash).map(|((url,size),hash)|(url,size,hash)).ok_or_else(|| AppError::InvalidInput("Download automático indisponível ou sem integridade declarada pelo autor.".into()))?;
    if size > 1024 * 1024 * 1024 || hash.len() != 40 || !hash.bytes().all(|byte| byte.is_ascii_hexdigit()) { return Err(AppError::InvalidInput("Pacote muito grande ou hash inválido.".into())); }
    let client = pack_download::client()?;
    crate::core::mods::resumable::download(&client, url, path, Some(cancelled)).await?;
    if !pack_download::verify_existing(path, Some(size), Some(hash), None, true).await { return Err(AppError::InvalidInput("O pacote falhou na verificação de integridade.".into())); }
    Ok(())
}

fn dependencies(path: &Path, source: &str, mc: &str) -> AppResult<ServerLoader> {
    let mut zip = zip::ZipArchive::new(std::fs::File::open(path)?)?;
    let mut file = zip.by_name(if source == "modrinth" { "modrinth.index.json" } else { "manifest.json" })?;
    if file.size() > 4 * 1024 * 1024 { return Err(AppError::InvalidInput("Manifesto muito grande.".into())); }
    let mut bytes = Vec::new(); file.by_ref().take(4 * 1024 * 1024 + 1).read_to_end(&mut bytes)?;
    let value: Value = serde_json::from_slice(&bytes)?;
    let mut loader = ServerLoader { kind:"vanilla".into(), version:String::new() };
    if source == "modrinth" {
        if value["dependencies"]["minecraft"] != mc { return Err(AppError::InvalidInput("A versão do pacote não corresponde ao Minecraft selecionado.".into())); }
        for (name, version) in value["dependencies"].as_object().ok_or_else(|| AppError::InvalidInput("Manifesto sem dependências.".into()))? {
            if name == "minecraft" { continue; }
            if loader.kind != "vanilla" { return Err(AppError::InvalidInput("Pacote com múltiplos loaders não suportado.".into())); }
            loader.kind = match name.as_str() { "fabric-loader"=>"fabric", "quilt-loader"=>"quilt", "forge"=>"forge", "neoforge"=>"neoforge", _=>return Err(AppError::InvalidInput("Loader do pacote não suportado.".into())) }.into();
            loader.version = version.as_str().unwrap_or_default().into();
        }
    } else {
        if value["minecraft"]["version"] != mc { return Err(AppError::InvalidInput("O pacote usa outro Minecraft.".into())); }
        let loaders = value["minecraft"]["modLoaders"].as_array().ok_or_else(|| AppError::InvalidInput("Manifesto sem loader.".into()))?;
        if loaders.len() != 1 { return Err(AppError::InvalidInput("O pacote precisa de exatamente um loader suportado.".into())); }
        let (kind, version) = loaders[0]["id"].as_str().and_then(|value| value.split_once('-')).ok_or_else(|| AppError::InvalidInput("Loader inválido.".into()))?;
        loader = ServerLoader { kind:kind.into(), version:version.into() };
    }
    loader.validate()?;
    Ok(loader)
}

pub(super) async fn install(http: &reqwest::Client, id: &str, project_id: &str, version: &str, source: &str) -> AppResult<Value> {
    let _install = INSTALLING.try_lock().map_err(|_| AppError::InvalidState("Já existe uma preparação de catálogo em andamento.".into()))?;
    valid_id(version)?;
    project(http, project_id, source).await?;
    let manager = manager(http)?;
    let _job = manager.content_job()?;
    let config = manager.config(id).await?;
    if manager.has_active() { return Err(AppError::InvalidState("Pare o servidor antes de preparar um modpack.".into())); }
    let root = manager.directory(id).await?;
    let marker = root.join(".luxmc-instance-pending");
    let request = root.join(".luxmc-catalog-request.json");
    let receipt = root.join(".luxmc-catalog.json");
    let mut repairing = false;
    if receipt.exists() {
        let previous: Value = serde_json::from_slice(&tokio::fs::read(&receipt).await?)?;
        if previous["source"] == source && previous["project"] == project_id && previous["version"] == version {
            if previous["pipelineVersion"] == 2 { clear_preparation(&marker, &request).await?; return Ok(previous["report"].clone()); }
            repairing = true;
        }
    }
    if !repairing && manager.contents(id).await?.iter().any(|item| !item.staged) { return Err(AppError::InvalidState("Instale o modpack em um servidor novo para preservar conteúdos existentes.".into())); }
    pack_download::atomic_write(&request, &serde_json::to_vec(&json!({"source":source,"project":project_id,"version":version}))?).await?;
    pack_download::atomic_write(&marker, b"catalog").await?;
    let temporary = root.join(format!(".luxmc-catalog-{}", uuid::Uuid::new_v4()));
    let result = async {
        let detail = if source == "modrinth" {
            let detail = metadata(http, format!("https://api.modrinth.com/v2/version/{version}"), source).await?;
            let canonical = project(http, project_id, source).await?;
            if detail["project_id"] != canonical["id"] { return Err(AppError::InvalidInput("Versão não pertence ao modpack.".into())); }
            detail
        } else { super::instance_pack::curseforge_file(http, project_id, version).await? };
        let file = if source == "modrinth" { detail["files"].as_array().and_then(|files| files.iter().find(|file| file["filename"].as_str().is_some_and(|name| name.ends_with(".mrpack")))).ok_or_else(|| AppError::InvalidInput("Arquivo .mrpack indisponível.".into()))? } else { &detail };
        download(http, file, source, &temporary, manager.cancellation()).await?;
        let inspect = temporary.clone(); let game = config.mc_version.clone(); let provider = source.to_string();
        let loader = tokio::task::spawn_blocking(move || dependencies(&inspect, &provider, &game)).await.map_err(|error| AppError::Internal(error.to_string()))??;
        manager.set_loader(id, loader).await?;
        let mut report = if source == "modrinth" { manager.import_modpack(id, temporary.clone()).await? } else {
            let server_id = detail["serverPackFileId"].as_u64().filter(|value| *value > 0).ok_or_else(|| AppError::InvalidInput("O autor não disponibilizou pacote de servidor para esta versão.".into()))?;
            let server = super::instance_pack::curseforge_file(http, project_id, &server_id.to_string()).await?;
            if server["isServerPack"] != true { return Err(AppError::InvalidInput("Arquivo não confirmado como pacote de servidor.".into())); }
            tokio::fs::remove_file(&temporary).await?;
            download(http, &server, source, &temporary, manager.cancellation()).await?;
            manager.import_server_pack(id, temporary.clone()).await?
        };
        manager.apply_content(id).await?;
        let settings = manager.settings(id).await?;
        if settings.resource_pack_file.is_empty() && settings.resource_pack_url.is_empty() {
            let packs = manager.contents(id).await?.into_iter().filter(|item| !item.staged && item.kind == "resourcepack" && item.name.to_ascii_lowercase().ends_with(".zip")).collect::<Vec<_>>();
            if packs.len() == 1 && packs[0].bytes <= 128 * 1024 * 1024 {
                if let Err(error) = manager.use_resource_pack(id, packs[0].name.clone(), false).await { report["resourcePackWarning"] = Value::String(error.to_string()); }
            }
        }
        pack_download::atomic_write(&root.join(".luxmc-catalog.json"), &serde_json::to_vec(&json!({"pipelineVersion":2,"source":source,"project":project_id,"version":version,"report":report}))?).await?;
        clear_preparation(&marker, &request).await?;
        Ok(report)
    }.await;
    let _ = tokio::fs::remove_file(&temporary).await;
    result
}

async fn clear_preparation(marker: &Path, request: &Path) -> AppResult<()> {
    for path in [marker, request] {
        match tokio::fs::remove_file(path).await {
            Ok(()) => {},
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {},
            Err(error) => return Err(error.into()),
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_completed_catalog_receipt_does_not_offer_to_resume_the_same_pack() {
        let request=json!({"source":"modrinth","project":"pixelmon","version":"release"});
        let mut receipt=json!({"pipelineVersion":2,"source":"modrinth","project":"pixelmon","version":"release"});
        assert!(request_completed(&request,&receipt));receipt["version"]=json!("another");assert!(!request_completed(&request,&receipt));assert!(!request_completed(&json!({}),&json!({"pipelineVersion":2})));
    }
    use std::io::Write;

    #[tokio::test]
    async fn catalog_completion_accepts_marker_already_removed_by_content_application() {
        let temp = tempfile::tempdir().unwrap();
        let marker = temp.path().join(".luxmc-instance-pending");
        let request = temp.path().join(".luxmc-catalog-request.json");
        tokio::fs::write(&request, b"{}").await.unwrap();
        clear_preparation(&marker, &request).await.unwrap();
        assert!(!request.exists());
        clear_preparation(&marker, &request).await.unwrap();
        tokio::fs::create_dir(&marker).await.unwrap();
        assert!(clear_preparation(&marker, &request).await.is_err());
    }

    fn archive(root: &Path, source: &str, value: Value) -> std::path::PathBuf {
        let path = root.join(format!("{source}.zip"));
        let mut zip = zip::ZipWriter::new(std::fs::File::create(&path).unwrap());
        zip.start_file(if source == "modrinth" { "modrinth.index.json" } else { "manifest.json" }, zip::write::FileOptions::default()).unwrap();
        zip.write_all(&serde_json::to_vec(&value).unwrap()).unwrap(); zip.finish().unwrap(); path
    }

    #[test]
    fn catalog_dependencies_pin_exact_loader_and_reject_wrong_world_version() {
        let temp = tempfile::tempdir().unwrap();
        let path = archive(temp.path(), "modrinth", json!({"dependencies":{"minecraft":"1.21.1","neoforge":"21.1.248"}}));
        let loader = dependencies(&path,"modrinth","1.21.1").unwrap();
        assert_eq!(loader.kind,"neoforge"); assert_eq!(loader.version,"21.1.248");
        assert!(dependencies(&path,"modrinth","1.20.1").is_err());
        let path = archive(temp.path(), "curseforge", json!({"minecraft":{"version":"1.20.1","modLoaders":[{"id":"forge-47.2.0"}]}}));
        assert_eq!(dependencies(&path,"curseforge","1.20.1").unwrap().version,"47.2.0");
        let path = archive(temp.path(), "modrinth", json!({"dependencies":{"minecraft":"1.21.1","fabric-loader":"0.16.0","forge":"47.2.0"}}));
        assert!(dependencies(&path,"modrinth","1.21.1").is_err());
        assert!(valid_id("../../other").is_err());
    }
}
