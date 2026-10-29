use crate::{
    core::local_server::{
        CreateServer, LocalServerConfig, LocalServerManager, ServerOverview, ServerState,
    },
    error::{AppError, AppResult},
    network::p2p_tunnel::{RoomIdentity, TunnelStatus},
    state::AppState,
};
use std::sync::{Arc, Mutex, OnceLock};
use tauri::State;
mod instance_pack;
mod server_catalog;

static MANAGER: OnceLock<Arc<LocalServerManager>> = OnceLock::new();
static PRO_ACCESS: Mutex<Option<(String, i64)>> = Mutex::new(None);

pub(crate) fn manager(http: &reqwest::Client) -> AppResult<Arc<LocalServerManager>> {
    let root = directories::ProjectDirs::from("io", "github", "Luxmc")
        .ok_or_else(|| AppError::InvalidState("Pasta de dados indisponível.".into()))?
        .data_dir()
        .join("local-servers");
    Ok(MANAGER
        .get_or_init(|| LocalServerManager::new(root, http.clone()))
        .clone())
}

pub(crate) fn has_active() -> bool {
    MANAGER.get().is_some_and(|manager| manager.has_active())
}
pub(crate) async fn shutdown() -> AppResult<()> {
    if let Some(manager) = MANAGER.get() {
        manager.shutdown().await?;
    }
    Ok(())
}

pub(crate) fn pro_expiration(status: &serde_json::Value, now: i64) -> Option<i64> {
    if status
        .pointer("/membership/plan")
        .and_then(|value| value.as_str())
        != Some("pro")
    {
        return None;
    }
    status
        .pointer("/membership/expiresAt")
        .and_then(|value| value.as_i64())
        .filter(|expires| *expires > now)
}

pub(crate) async fn require_pro(account_id: Option<&str>) -> AppResult<()> {
    let id = account_id.filter(|id| !id.is_empty()).ok_or_else(|| AppError::InvalidState("Entre na sua conta Pro para hospedar um servidor dedicado. A hospedagem de mundo LAN continua disponível separadamente.".into()))?;
    let now = chrono::Utc::now().timestamp();
    if PRO_ACCESS
        .lock()
        .unwrap_or_else(|error| error.into_inner())
        .as_ref()
        .is_some_and(|(account, until)| account == id && *until > now)
    {
        return Ok(());
    }
    let status = super::membership::membership_request_core(id.into(), super::membership::MembershipRequest::Status).await.map_err(|_| AppError::InvalidState("Não foi possível confirmar o Pro. Tente novamente com conexão; seus mundos e a opção Parar e salvar continuam disponíveis.".into()))?;
    let expires = pro_expiration(&status, now).ok_or_else(|| AppError::InvalidState("Hospedar e administrar servidor dedicado é um recurso Pro. Plus pode hospedar mundos pela rede LAN.".into()))?;
    *PRO_ACCESS.lock().unwrap_or_else(|error| error.into_inner()) =
        Some((id.into(), expires.min(now + 30)));
    Ok(())
}

pub(crate) async fn allow_console(account_id: Option<&str>, text: &str) -> AppResult<()> {
    if ["stop", "save-all flush", "save-on", "list"].contains(&text.trim().trim_start_matches('/'))
    {
        return Ok(());
    }
    require_pro(account_id).await
}

pub(crate) async fn include_owner(
    http: &reqwest::Client,
    id: &str,
    account_id: Option<&str>,
) -> AppResult<()> {
    let manager = manager(http)?;
    if !manager.settings(id).await?.whitelist {
        return Ok(());
    }
    let account_id = account_id.ok_or_else(|| {
        AppError::InvalidState(
            "Selecione sua conta Microsoft para liberar o dono na whitelist.".into(),
        )
    })?;
    let db = crate::db::shared_db().await?;
    let account = crate::db::schema::accounts::get_by_id(&db, account_id).await?.ok_or_else(|| AppError::InvalidState("Para servidor restrito, selecione a conta Microsoft usada no Minecraft ou escolha acesso aberto nas configurações.".into()))?;
    let token = account.access_token.as_deref().ok_or_else(|| {
        AppError::InvalidState(
            "Selecione uma conta Microsoft autenticada para liberar a whitelist.".into(),
        )
    })?;
    let profile = crate::core::auth::MinecraftClient::new(http.clone()).profile(token).await.map_err(|_| AppError::InvalidState("Entre novamente na conta Microsoft para confirmar a identidade do dono antes de iniciar o servidor restrito.".into()))?;
    let uuid = uuid::Uuid::parse_str(&profile.id)
        .map_err(|_| AppError::InvalidState("Identidade Minecraft inválida.".into()))?;
    if uuid::Uuid::parse_str(&account.uuid).ok() != Some(uuid) {
        return Err(AppError::InvalidState(
            "A conta Minecraft recebida não corresponde ao perfil selecionado.".into(),
        ));
    }
    manager
        .include_owner(id, &uuid.to_string(), &profile.name)
        .await
}

#[derive(serde::Deserialize, serde::Serialize)]
#[serde(
    tag = "action",
    rename_all = "snake_case",
    rename_all_fields = "camelCase"
)]
pub enum ServerRequest {
    DeleteServer { id: String, name: String, #[serde(default)] delete_backups: bool },
    ConnectionDiagnostic { id:String },
    ConsoleText { id:String },
    DeletionPreview { id:String },
    ManagedComponents { id:String },
    CatalogVersions { project: String, source: String },
    InstallCatalogPack { id: String, project: String, version: String, source: String },
    CatalogPending { id: String },
    UseResourcePack { id:String, file:String, required:bool },
    RestoreServer { id: String },
    DeletedServers,
    InstanceReport { id: String },
    BrowseFiles { id: String, path: String },
    PublicTunnel { id: String, operation: String },
    SharingStatus,
    SharingKick { member_id:String },
    SharingLock { locked:bool },
    SharingInvitation,
    StopSharing,
    Settings {
        id: String,
    },
    SaveSettings {
        id: String,
        settings: crate::core::local_server::settings::ServerSettings,
    },
    Moderate {
        id: String,
        command: String,
        name: String,
        #[serde(default)]
        reason: String,
        uuid: Option<String>,
    },
    Backups {
        id: String,
    },
    Backup {
        id: String,
    },
    RestoreBackup {
        id: String,
        backup: String,
    },
    RemoveBackup {
        id: String,
        backup: String,
    },
    ExportBackup {
        id: String,
        backup: String,
        path: String,
    },
    Contents {
        id: String,
    },
    ContentDetails { id: String, kind: String, names: Vec<String> },
    ToggleContent { id: String, kind: String, name: String, enabled: bool },
    ImportContent {
        id: String,
        kind: String,
        path: String,
    },
    RemoveContent {
        id: String,
        kind: String,
        name: String,
    },
    ApplyContent {
        id: String,
    },
    Loader {
        id: String,
    },
    LoaderVersions {
        id: String,
        kind: String,
    },
    SetLoader {
        id: String,
        loader: crate::core::local_server::loaders::ServerLoader,
    },
    DownloadContent {
        id: String,
        project: String,
        source: String,
        kind: String,
    },
    Players {
        id: String,
    },
    SetIcon {
        id: String,
        path: String,
    },
    ImportWorld {
        id: String,
        path: String,
    },
    ImportModpack {
        id: String,
        path: String,
    },
    ClearPending {
        id: String,
    },
    ImportInstance {
        id: String,
        profile_id: String,
    },
    ImportServerPack {
        id: String,
        path: String,
    },
}

pub(crate) async fn manage(
    http: &reqwest::Client,
    account_id: Option<String>,
    request: ServerRequest,
) -> AppResult<serde_json::Value> {
    let manager = manager(http)?;
    match request {
        ServerRequest::ConnectionDiagnostic {id}=>manager.connection_diagnostic(&id).await,
        ServerRequest::ConsoleText {id}=>manager.console_text(&id).await,
        ServerRequest::DeletionPreview {id}=>manager.deletion_preview(&id).await,
        ServerRequest::ManagedComponents {id}=>{let root=manager.directory(&id).await?;Ok(crate::core::managed_skin::status(&root).await)},
        ServerRequest::ContentDetails {id,kind,names} => manager.content_details(&id,&kind,names).await,
        ServerRequest::ToggleContent {id,kind,name,enabled} => {require_pro(account_id.as_deref()).await?;manager.toggle_content(&id,&kind,&name,enabled).await?;Ok(serde_json::Value::Null)},
        ServerRequest::DeleteServer { id, name, delete_backups } => { manager.delete_server(&id, &name, delete_backups).await?; Ok(serde_json::Value::Null) },
        ServerRequest::CatalogVersions { project, source } => server_catalog::versions(http, &project, &source).await,
        ServerRequest::InstallCatalogPack { id, project, version, source } => { require_pro(account_id.as_deref()).await?; server_catalog::install(http, &id, &project, &version, &source).await },
        ServerRequest::CatalogPending { id } => server_catalog::pending(http, &id).await,
        ServerRequest::UseResourcePack {id,file,required}=>{require_pro(account_id.as_deref()).await?;manager.use_resource_pack(&id,file,required).await?;Ok(serde_json::Value::Null)},
        ServerRequest::RestoreServer { id } => { manager.restore_server(&id).await?; Ok(serde_json::Value::Null) },
        ServerRequest::DeletedServers => Ok(serde_json::to_value(manager.deleted_servers().await?)?),
        ServerRequest::InstanceReport { id } => manager.instance_report(&id).await,
        ServerRequest::BrowseFiles { id, path } => manager.browse_files(&id, &path).await,
        ServerRequest::PublicTunnel { id, operation } => {
            if operation == "stop" { return manager.public_tunnel(&id, "stop").await; }
            if operation == "status" { return Ok(serde_json::json!({"linked":false,"address":"","agentRunning":false,"retired":true})); }
            Err(AppError::InvalidState("A integração externa foi retirada. Use a conexão P2P integrada do Luxmc.".into()))
        },
        ServerRequest::SharingStatus => Ok(serde_json::to_value(crate::network::p2p_tunnel::tunnel_status().await?)?),
        ServerRequest::SharingKick {member_id} => {crate::network::p2p_tunnel::tunnel_kick_member(member_id).await?;Ok(serde_json::Value::Null)},
        ServerRequest::SharingLock {locked} => {crate::network::p2p_tunnel::tunnel_set_locked(locked).await?;Ok(serde_json::Value::Null)},
        ServerRequest::SharingInvitation => Ok(serde_json::to_value(crate::network::p2p_tunnel::tunnel_refresh_invitation().await?)?),
        ServerRequest::StopSharing => { crate::network::p2p_tunnel::stop_session().await?; Ok(serde_json::Value::Null) },
        ServerRequest::Settings { id } => Ok(serde_json::to_value(manager.settings(&id).await?)?),
        ServerRequest::SaveSettings { id, settings } => {
            require_pro(account_id.as_deref()).await?;
            Ok(serde_json::to_value(
                manager.update_settings(&id, settings).await?,
            )?)
        }
        ServerRequest::Backups { id } => Ok(serde_json::to_value(manager.backups(&id).await?)?),
        ServerRequest::Backup { id } => Ok(serde_json::to_value(manager.backup(&id).await?)?),
        ServerRequest::RestoreBackup { id, backup } => {
            manager.restore_backup(&id, &backup).await?;
            Ok(serde_json::Value::Null)
        }
        ServerRequest::RemoveBackup { id, backup } => {
            manager.remove_backup(&id, &backup).await?;
            Ok(serde_json::Value::Null)
        }
        ServerRequest::ExportBackup { id, backup, path } => {
            manager.export_backup(&id, &backup, path.into()).await?;
            Ok(serde_json::Value::Null)
        }
        ServerRequest::Contents { id } => Ok(serde_json::to_value(manager.contents(&id).await?)?),
        ServerRequest::ImportContent { id, kind, path } => {
            require_pro(account_id.as_deref()).await?;
            manager
                .stage_content(&id, &kind, std::path::Path::new(&path))
                .await?;
            Ok(serde_json::Value::Null)
        }
        ServerRequest::RemoveContent { id, kind, name } => {
            require_pro(account_id.as_deref()).await?;
            manager.stage_remove(&id, &kind, &name).await?;
            Ok(serde_json::Value::Null)
        }
        ServerRequest::ApplyContent { id } => {
            require_pro(account_id.as_deref()).await?;
            Ok(serde_json::to_value(manager.apply_content(&id).await?)?)
        }
        ServerRequest::Loader { id } => Ok(serde_json::to_value(manager.loader(&id).await?)?),
        ServerRequest::LoaderVersions { id, kind } => Ok(serde_json::to_value(
            manager.loader_versions(&id, &kind).await?,
        )?),
        ServerRequest::SetLoader { id, loader } => {
            require_pro(account_id.as_deref()).await?;
            Ok(serde_json::to_value(
                manager.set_loader(&id, loader).await?,
            )?)
        }
        ServerRequest::DownloadContent {
            id,
            project,
            source,
            kind,
        } => {
            require_pro(account_id.as_deref()).await?;
            Ok(serde_json::to_value(
                manager
                    .download_content(&id, &project, &source, &kind)
                    .await?,
            )?)
        }
        ServerRequest::Players { id } => manager.player_details(&id).await,
        ServerRequest::SetIcon { id, path } => {
            require_pro(account_id.as_deref()).await?;
            manager.set_icon(&id, path.into()).await?;
            Ok(serde_json::Value::Null)
        }
        ServerRequest::ImportWorld { id, path } => {
            require_pro(account_id.as_deref()).await?;
            manager.import_world(&id, path.into()).await?;
            Ok(serde_json::Value::Null)
        }
        ServerRequest::ImportModpack { id, path } => {
            require_pro(account_id.as_deref()).await?;
            manager.import_modpack(&id, path.into()).await
        }
        ServerRequest::ClearPending { id } => {
            require_pro(account_id.as_deref()).await?;
            manager.clear_pending(&id).await?;
            Ok(serde_json::Value::Null)
        }
        ServerRequest::ImportInstance { id, profile_id } => {
            require_pro(account_id.as_deref()).await?;
            instance_pack::import_instance(http, &id, profile_id).await
        }
        ServerRequest::ImportServerPack { id, path } => {
            require_pro(account_id.as_deref()).await?;
            manager.import_server_pack(&id, path.into()).await
        }
        ServerRequest::Moderate {
            id,
            command,
            name,
            reason,
            uuid,
        } => {
            require_pro(account_id.as_deref()).await?;
            if ![
                "kick",
                "ban",
                "pardon",
                "whitelist add",
                "whitelist remove",
                "op",
                "deop",
            ]
            .contains(&command.as_str())
                || name.is_empty()
                || name.len() > 16
                || !name
                    .bytes()
                    .all(|byte| byte.is_ascii_alphanumeric() || byte == b'_')
                || reason.len() > 200
                || reason.chars().any(char::is_control)
            {
                return Err(AppError::InvalidInput("Ação ou jogador inválido.".into()));
            }
            if let Some(expected) = uuid {
                let expected = uuid::Uuid::parse_str(&expected)
                    .map_err(|_| AppError::InvalidInput("UUID de jogador inválido.".into()))?;
                let details = manager.player_details(&id).await?;
                let online = details["players"].as_array().is_some_and(|players| {
                    players.iter().any(|player| {
                        player["online"] == true
                            && player["name"]
                                .as_str()
                                .is_some_and(|value| value.eq_ignore_ascii_case(&name))
                            && player["uuid"]
                                .as_str()
                                .and_then(|value| uuid::Uuid::parse_str(value).ok())
                                == Some(expected)
                    })
                });
                if !online
                    && super::skins::minecraft_uuid(name.clone())
                        .await?
                        .as_deref()
                        .and_then(|value| uuid::Uuid::parse_str(value).ok())
                        != Some(expected)
                {
                    return Err(AppError::InvalidState("Este nome não corresponde mais ao UUID selecionado. Confira o nome atual do jogador antes de administrar o acesso.".into()));
                }
            }
            let reason = if ["kick", "ban"].contains(&command.as_str()) {
                reason.trim()
            } else {
                ""
            };
            manager
                .console(&id, format!("{command} {name} {reason}").trim().into())
                .await?;
            Ok(serde_json::Value::Null)
        }
    }
}

#[tauri::command]
pub async fn local_server_manage(
    state: State<'_, AppState>,
    request: ServerRequest,
    account_id: Option<String>,
) -> AppResult<serde_json::Value> {
    let _ = state;
    if matches!(request,ServerRequest::SharingStatus) {
        return Ok(crate::server_worker::invoke_if_running::<serde_json::Value>("local_server_manage",serde_json::json!({"request":request})).await?.unwrap_or(serde_json::Value::Null));
    }
    crate::server_worker::invoke("local_server_manage", serde_json::json!({"request":request,"accountId":account_id})).await
}

#[tauri::command]
pub async fn local_server_list(
    state: State<'_, AppState>,
    after: Option<u64>,
) -> AppResult<ServerOverview> {
    let _ = state;
    crate::server_worker::invoke("local_server_list", serde_json::json!({"after":after.unwrap_or(0)})).await
}

#[tauri::command]
pub async fn local_server_create(
    state: State<'_, AppState>,
    input: CreateServer,
    account_id: Option<String>,
) -> AppResult<LocalServerConfig> {
    let _ = state;
    crate::server_worker::invoke("local_server_create", serde_json::json!({"input":input,"accountId":account_id})).await
}

#[tauri::command]
pub async fn local_server_start(
    state: State<'_, AppState>,
    id: String,
    account_id: Option<String>,
) -> AppResult<ServerState> {
    let _guard = state.launch_lock.try_lock().map_err(|_| AppError::InvalidState("Aguarde a preparação do Minecraft.".into()))?;
    crate::server_worker::invoke("local_server_start", serde_json::json!({"id":id,"accountId":account_id})).await
}

#[tauri::command]
pub async fn local_server_stop(state: State<'_, AppState>, id: String) -> AppResult<()> {
    let _ = state;
    crate::server_worker::invoke("local_server_stop", serde_json::json!({"id":id})).await
}

#[tauri::command]
pub async fn local_server_console(
    state: State<'_, AppState>,
    id: String,
    text: String,
    account_id: Option<String>,
) -> AppResult<()> {
    let _ = state;
    crate::server_worker::invoke("local_server_console", serde_json::json!({"id":id,"text":text,"accountId":account_id})).await
}

#[tauri::command]
pub async fn local_server_update(
    state: State<'_, AppState>,
    id: String,
    name: String,
    ram_mb: u32,
    max_players: u16,
    account_id: Option<String>,
) -> AppResult<LocalServerConfig> {
    let _ = state;
    crate::server_worker::invoke("local_server_update", serde_json::json!({"id":id,"name":name,"ramMb":ram_mb,"maxPlayers":max_players,"accountId":account_id})).await
}

#[tauri::command]
pub async fn local_server_open_folder(state: State<'_, AppState>, id: String) -> AppResult<()> {
    let _ = state;
    crate::server_worker::invoke("local_server_open_folder", serde_json::json!({"id":id})).await
}

pub(crate) async fn open_folder(http: &reqwest::Client, id: String) -> AppResult<()> {
    let path = manager(http)?.directory(&id).await?;
    tokio::task::spawn_blocking(move || open::that(path))
        .await
        .map_err(|error| AppError::Internal(error.to_string()))??;
    Ok(())
}

#[tauri::command]
pub async fn local_server_share(
    state: State<'_, AppState>,
    id: String,
    identity: Option<RoomIdentity>,
    account_id: Option<String>,
) -> AppResult<serde_json::Value> {
    let _ = state;
    crate::server_worker::invoke("local_server_share", serde_json::json!({"id":id,"identity":identity,"accountId":account_id})).await
}

pub(crate) async fn share(
    http: &reqwest::Client,
    id: String,
    identity: Option<RoomIdentity>,
    account_id: Option<String>,
) -> AppResult<TunnelStatus> {
    require_pro(account_id.as_deref()).await?;
    let manager = manager(http)?;
    let config = manager.config(&id).await?;
    let port = manager.running_port(&id)?;
    let owner = manager.room_owner(&id)?;
    let mut stopped = manager.stop_signal(&id)?;
    let status = tokio::select! {
        biased;
        _ = crate::core::local_server::wait_stop(&mut stopped) => return Err(AppError::InvalidState("O servidor encerrou durante o compartilhamento.".into())),
        result = crate::network::p2p_tunnel::host_owned_server(port, owner.clone(), config.name, config.mc_version, identity, account_id) => result?,
    };
    if manager.running_port(&id).ok() != Some(port) {
        let _ = crate::network::p2p_tunnel::stop_owned_server(&owner).await;
        return Err(AppError::InvalidState(
            "O servidor encerrou durante o compartilhamento.".into(),
        ));
    }
    Ok(status)
}

#[tauri::command]
pub async fn local_server_share_existing(
    state: State<'_, AppState>,
    port: u16,
    identity: Option<RoomIdentity>,
    account_id: Option<String>,
) -> AppResult<serde_json::Value> {
    let _ = state;
    crate::server_worker::invoke("local_server_share_existing", serde_json::json!({"port":port,"identity":identity,"accountId":account_id})).await
}

pub(crate) async fn share_existing(
    http: &reqwest::Client,
    port: u16,
    identity: Option<RoomIdentity>,
    account_id: Option<String>,
) -> AppResult<TunnelStatus> {
    require_pro(account_id.as_deref()).await?;
    if port == 0 {
        return Err(AppError::InvalidInput(
            "Informe uma porta local de 1 a 65535.".into(),
        ));
    }
    let status = tokio::time::timeout(std::time::Duration::from_secs(7), async {
        let first = crate::core::server::ping_async(http, "127.0.0.1", port).await?;
        if first.online {
            Ok::<_, AppError>(first)
        } else {
            crate::core::server::ping_async(http, "::1", port).await
        }
    })
    .await
    .map_err(|_| AppError::InvalidState("O servidor local não respondeu.".into()))??;
    if !status.online {
        return Err(AppError::InvalidState(
            "Nenhum servidor Minecraft respondeu nesta porta local.".into(),
        ));
    }
    crate::network::p2p_tunnel::host_owned_server(
        port,
        format!("external:{port}"),
        "Minecraft Server".into(),
        status
            .version
            .chars()
            .filter(|character| !character.is_control())
            .take(96)
            .collect(),
        identity,
        account_id,
    )
    .await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dedicated_access_requires_an_unexpired_verified_pro_membership() {
        for value in [
            serde_json::json!({}),
            serde_json::json!({"membership":{"plan":"plus","expiresAt":200}}),
            serde_json::json!({"membership":{"plan":"pro","expiresAt":99}}),
            serde_json::json!({"membership":{"plan":"pro","expiresAt":null}}),
        ] {
            assert_eq!(pro_expiration(&value, 100), None);
        }
        assert_eq!(
            pro_expiration(
                &serde_json::json!({"membership":{"plan":"pro","expiresAt":200}}),
                100
            ),
            Some(200)
        );
    }

    #[tokio::test]
    async fn safety_controls_do_not_require_an_active_plan() {
        for command in ["stop", "/save-all flush", "save-on", "list"] {
            assert!(allow_console(None, command).await.is_ok());
        }
        assert!(allow_console(None, "op Player").await.is_err());
        assert!(require_pro(None).await.is_err());
    }
}
