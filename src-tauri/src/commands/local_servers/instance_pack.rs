use super::*;
use crate::core::mods::{curseforge, pack_download, ModrinthClient};

pub(super) async fn curseforge_file(
    http: &reqwest::Client,
    project: &str,
    file: &str,
) -> AppResult<serde_json::Value> {
    let key = curseforge::api_key().ok_or_else(|| AppError::InvalidState("Configure a chave CurseForge nos ajustes para consultar o pacote de servidor do autor.".into()))?;
    let mut response = http
        .get(format!(
            "https://api.curseforge.com/v1/mods/{project}/files/{file}"
        ))
        .header("x-api-key", key)
        .timeout(std::time::Duration::from_secs(20))
        .send()
        .await?
        .error_for_status()?;
    let mut bytes = Vec::new();
    while let Some(chunk) = response.chunk().await? {
        if bytes.len() + chunk.len() > 2 * 1024 * 1024 {
            return Err(AppError::InvalidInput(
                "Metadados de pacote muito grandes.".into(),
            ));
        }
        bytes.extend_from_slice(&chunk);
    }
    let value: serde_json::Value = serde_json::from_slice(&bytes)?;
    let data = value
        .get("data")
        .ok_or_else(|| AppError::InvalidInput("Arquivo CurseForge não encontrado.".into()))?;
    if data["modId"]
        .as_u64()
        .map(|value| value.to_string())
        .as_deref()
        != Some(project)
        || data["id"]
            .as_u64()
            .map(|value| value.to_string())
            .as_deref()
            != Some(file)
        || data["isAvailable"] == false
    {
        return Err(AppError::InvalidInput(
            "O pacote não pertence ao projeto ou não está disponível.".into(),
        ));
    }
    Ok(data.clone())
}

pub(super) async fn import_instance(
    http: &reqwest::Client,
    id: &str,
    profile_id: String,
) -> AppResult<serde_json::Value> {
    let manager = manager(http)?;
    let _job = manager.content_job()?;
    let config = manager.config(id).await?;
    let loader = manager.loader(id).await?;
    let profile = crate::commands::studio::profile(&profile_id).await?;
    if profile.mc_version != config.mc_version
        || profile.loader != loader.kind
        || (loader.kind != "vanilla"
            && profile.loader_version.as_deref() != Some(loader.version.as_str()))
    {
        return Err(AppError::InvalidInput("A instância e o servidor precisam usar exatamente o mesmo Minecraft e loader. Configure a versão correta antes de importar.".into()));
    }
    return manager.import_installed_files(id, std::path::PathBuf::from(profile.game_dir)).await;
}

#[allow(dead_code)]
pub(super) async fn import_author_pack(http: &reqwest::Client, id: &str, profile_id: String) -> AppResult<serde_json::Value> {
    let manager = manager(http)?;
    let _job = manager.content_job()?;
    let config = manager.config(id).await?;
    let reference = crate::commands::studio::instance_pack_reference(profile_id).await?.ok_or_else(|| AppError::InvalidInput("Esta instância não tem uma origem de modpack vinculada. Importe o arquivo .mrpack original ou o pacote de servidor fornecido pelo autor.".into()))?;
    if !crate::commands::studio::valid_reference(&reference) {
        return Err(AppError::InvalidInput("Origem de modpack inválida.".into()));
    }
    let (url, size, hash) = if reference.source == "modrinth" {
        let detail = ModrinthClient::new(http.clone())
            .get_version_detail(
                &reference.project_id,
                &reference.version_id,
                &config.mc_version,
            )
            .await?;
        let file = detail
            .files
            .into_iter()
            .find(|file| file.filename.ends_with(".mrpack"))
            .ok_or_else(|| {
                AppError::InvalidInput("A versão vinculada não oferece um arquivo .mrpack.".into())
            })?;
        (file.url, file.size, file.sha1)
    } else if reference.source == "curseforge" {
        let parent = curseforge_file(http, &reference.project_id, &reference.version_id).await?;
        let server_id = parent["serverPackFileId"].as_u64().filter(|value| *value > 0).ok_or_else(|| AppError::InvalidState("O autor não vinculou um pacote de servidor a esta versão. Não copie automaticamente os mods do cliente; use LAN ou prepare um servidor compatível separadamente.".into()))?;
        let server = curseforge_file(http, &reference.project_id, &server_id.to_string()).await?;
        if server["isServerPack"] != true {
            return Err(AppError::InvalidInput(
                "O arquivo vinculado não foi confirmado como pacote de servidor pelo CurseForge."
                    .into(),
            ));
        }
        let url = server["downloadUrl"].as_str().ok_or_else(|| AppError::InvalidState("O autor não permite download automático deste pacote. Baixe-o pela página oficial e use a opção de importar pacote de servidor.".into()))?.to_string();
        let size = server["fileLength"]
            .as_u64()
            .ok_or_else(|| AppError::InvalidInput("Pacote de servidor sem tamanho.".into()))?;
        let hash = server["hashes"]
            .as_array()
            .and_then(|values| values.iter().find(|value| value["algo"] == 1))
            .and_then(|value| value["value"].as_str())
            .ok_or_else(|| {
                AppError::InvalidInput("Pacote de servidor sem hash de integridade.".into())
            })?
            .to_string();
        (url, size, hash)
    } else {
        return Err(AppError::InvalidInput(
            "Provedor de modpack não suportado.".into(),
        ));
    };
    if size > 1024 * 1024 * 1024
        || hash.len() != 40
        || !hash.bytes().all(|value| value.is_ascii_hexdigit())
    {
        return Err(AppError::InvalidInput(
            "Pacote muito grande ou sem hash válido.".into(),
        ));
    }
    let temporary = manager
        .directory(id)
        .await?
        .join(format!(".luxmc-pack-import-{}", uuid::Uuid::new_v4()));
    let client = pack_download::client()?;
    crate::core::mods::resumable::download(&client, &url, &temporary, Some(manager.cancellation()))
        .await?;
    if !pack_download::verify_existing(&temporary, Some(size), Some(&hash), None, true).await {
        let _ = tokio::fs::remove_file(&temporary).await;
        return Err(AppError::InvalidInput(
            "O pacote não passou na verificação de integridade.".into(),
        ));
    }
    let result = if reference.source == "modrinth" {
        manager.import_modpack(id, temporary.clone()).await
    } else {
        manager.import_server_pack(id, temporary.clone()).await
    };
    let _ = tokio::fs::remove_file(temporary).await;
    result
}
