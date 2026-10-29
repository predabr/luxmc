use crate::error::{AppError, AppResult};
use serde::{Deserialize, Serialize};
use std::{
    collections::VecDeque,
    path::{Path, PathBuf},
    process::Stdio,
    sync::{
        atomic::{AtomicBool, AtomicU64, AtomicUsize, Ordering},
        Arc, Mutex,
    },
    time::Duration,
};
use tokio::{
    io::{AsyncRead, AsyncWriteExt, BufReader},
    sync::{mpsc, oneshot, watch},
};
pub mod backups;
pub mod catalog;
pub mod content;
mod control;
mod diagnostics;
mod artifact_reuse;
#[cfg(test)]
mod live_tests;
mod content_display;
mod instance_copy;
pub mod loaders;
pub mod management;
pub mod modpacks;
pub mod players;
pub mod public_tunnel;
mod process_guard;
#[cfg(test)]
mod reform_tests;
pub mod settings;

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ServerArtifact {
    pub url: String,
    pub sha1: String,
    pub size: u64,
    pub java_major: u32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LocalServerConfig {
    pub id: String,
    pub name: String,
    pub mc_version: String,
    pub ram_mb: u32,
    pub max_players: u16,
    pub eula_accepted: bool,
    pub artifact: ServerArtifact,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateServer {
    pub name: String,
    pub mc_version: String,
    pub ram_mb: u32,
    pub max_players: u16,
    pub eula_accepted: bool,
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConsoleLine {
    pub sequence: u64,
    pub stream: String,
    pub text: String,
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ServerState {
    pub id: String,
    pub generation: String,
    pub phase: String,
    pub port: Option<u16>,
    pub pid: Option<u32>,
    pub max_players: u16,
    pub players_online: u32,
    pub error: Option<String>,
    pub exit_code: Option<i32>,
    pub console: Vec<ConsoleLine>,
    pub cursor: u64,
    pub last_saved_at: Option<String>,
    pub forced_stop: bool,
    pub players: Vec<players::PlayerRecord>,
    pub history_complete: bool,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ServerOverview {
    pub servers: Vec<LocalServerConfig>,
    pub active: Option<ServerState>,
    pub warnings: Vec<String>,
    pub icons: std::collections::BTreeMap<String, String>,
}

struct ConsoleCommand {
    text: String,
    reply: oneshot::Sender<Result<(), String>>,
}

struct Runtime {
    state: Mutex<ServerState>,
    lines: Mutex<VecDeque<ConsoleLine>>,
    sequence: AtomicU64,
    stop: watch::Sender<bool>,
    finished: watch::Sender<bool>,
    commands: mpsc::Sender<ConsoleCommand>,
    ready_log: AtomicBool,
    history: Mutex<players::PlayerHistory>,
    history_path: Option<PathBuf>,
    history_dirty: AtomicBool,
    autosave_minutes: u16,
    control: Mutex<Option<control::Endpoint>>,
    startup_commands: Vec<String>,
}

impl Runtime {
    fn active(&self) -> bool {
        !*self.finished.borrow()
    }
    fn phase(&self, phase: &str) {
        self.state
            .lock()
            .unwrap_or_else(|error| error.into_inner())
            .phase = phase.into();
    }
    fn log(&self, stream: &str, text: &str) {
        let text = crate::core::process::sanitize_diagnostic(text);
        let message=text.split_once("]: ").map_or(text.as_str(),|(_,message)|message);
        if stream=="stdout" && (message.starts_with("Done (") || message.starts_with("Done!")) {
            self.ready_log.store(true, Ordering::Release);
        }
        {
            let mut history = self
                .history
                .lock()
                .unwrap_or_else(|error| error.into_inner());
            if history.observe(&text) {
                self.history_dirty.store(true, Ordering::Release);
                self.state
                    .lock()
                    .unwrap_or_else(|error| error.into_inner())
                    .players_online = history.online_count();
            }
        }
        if stream=="stdout" && text.contains("[Server thread/INFO]")
            && matches!(message,"Saved the game" | "All dimensions are saved" | "ThreadedAnvilChunkStorage: All dimensions are saved")
        {
            self.state
                .lock()
                .unwrap_or_else(|error| error.into_inner())
                .last_saved_at = Some(chrono::Utc::now().to_rfc3339());
        }
        let mut lines = self.lines.lock().unwrap_or_else(|error| error.into_inner());
        let sequence = self.sequence.fetch_add(1, Ordering::Relaxed) + 1;
        lines.push_back(ConsoleLine {
            sequence,
            stream: stream.into(),
            text,
        });
        while lines.len() > 500 {
            lines.pop_front();
        }
    }
    fn snapshot(&self, after: u64) -> ServerState {
        let mut state = self
            .state
            .lock()
            .unwrap_or_else(|error| error.into_inner())
            .clone();
        {
            let history = self
                .history
                .lock()
                .unwrap_or_else(|error| error.into_inner());
            state.players = history.records.values().cloned().collect();
            state.history_complete =
                history.complete && history.online_count() == state.players_online;
        }
        let lines = self.lines.lock().unwrap_or_else(|error| error.into_inner());
        state.console = lines
            .iter()
            .filter(|line| line.sequence > after)
            .cloned()
            .collect();
        state.cursor = lines.back().map_or(0, |line| line.sequence);
        state
    }
}

pub struct LocalServerManager {
    root: PathBuf,
    http: reqwest::Client,
    runtime: Mutex<Option<Arc<Runtime>>>,
    changes: tokio::sync::Mutex<()>,
    shutting_down: AtomicBool,
    content_jobs: AtomicUsize,
}

pub(crate) struct ContentJob<'a>(&'a AtomicUsize);
impl Drop for ContentJob<'_> {
    fn drop(&mut self) {
        self.0.fetch_sub(1, Ordering::AcqRel);
    }
}

fn invalid(message: impl Into<String>) -> AppError {
    AppError::InvalidInput(message.into())
}
fn unavailable(message: impl Into<String>) -> AppError {
    AppError::InvalidState(message.into())
}

fn validate_memory(ram_mb: u32) -> AppResult<()> {
    if !(512..=8192).contains(&ram_mb) {
        return Err(invalid("Configure de 512 MiB a 8 GiB de RAM para o servidor. Configurações antigas acima desse limite precisam ser ajustadas antes de iniciar."));
    }
    Ok(())
}

fn valid_official_url(value: &str, metadata: bool) -> bool {
    url::Url::parse(value).is_ok_and(|url| {
        url.scheme() == "https"
            && url.username().is_empty()
            && url.password().is_none()
            && url.port().is_none()
            && match url.host_str() {
                Some("piston-meta.mojang.com" | "launchermeta.mojang.com") => metadata,
                Some("piston-data.mojang.com" | "launcher.mojang.com") => !metadata,
                Some("s3.amazonaws.com") => {
                    !metadata && url.path().starts_with("/Minecraft.Download/")
                }
                _ => false,
            }
    })
}

fn validate_config(config: &LocalServerConfig) -> AppResult<()> {
    if uuid::Uuid::parse_str(&config.id).is_err() || config.id.len() != 36 {
        return Err(invalid("Identificador de servidor inválido."));
    }
    if config.name.trim().is_empty()
        || config.name.chars().count() > 64
        || config.name.chars().any(char::is_control)
    {
        return Err(invalid("Use um nome de servidor com até 64 caracteres."));
    }
    if config.mc_version.is_empty()
        || config.mc_version.len() > 96
        || !config
            .mc_version
            .bytes()
            .all(|value| value.is_ascii_alphanumeric() || b"._-".contains(&value))
    {
        return Err(invalid("Versão do servidor inválida."));
    }
    if !(512..=65536).contains(&config.ram_mb) || !(1..=20).contains(&config.max_players) {
        return Err(invalid(
            "Configure de 512 MB a 64 GB de RAM e de 1 a 20 jogadores.",
        ));
    }
    if !config.eula_accepted {
        return Err(invalid(
            "Leia e aceite a EULA do Minecraft para criar o servidor.",
        ));
    }
    let artifact = &config.artifact;
    if !valid_official_url(&artifact.url, false)
        || artifact.size == 0
        || artifact.size > 1024 * 1024 * 1024
        || artifact.sha1.len() != 40
        || !artifact.sha1.bytes().all(|value| value.is_ascii_hexdigit())
        || !(8..=99).contains(&artifact.java_major)
    {
        return Err(invalid("Os metadados do servidor oficial são inválidos."));
    }
    Ok(())
}

fn server_properties(previous: &str, port: u16, players: u16) -> String {
    let overrides = [
        "server-ip",
        "server-port",
        "max-players",
        "online-mode",
        "prevent-proxy-connections",
        "enable-rcon",
        "enable-query",
        "enable-status",
        "level-name",
    ];
    let mut result = previous
        .lines()
        .filter(|line| {
            !overrides.contains(
                &line
                    .trim_start()
                    .split(['=', ':', ' ', '\t'])
                    .next()
                    .unwrap_or_default(),
            )
        })
        .collect::<Vec<_>>()
        .join("\n");
    result.push_str(&format!("\nserver-ip=127.0.0.1\nserver-port={port}\nmax-players={players}\nonline-mode=true\nprevent-proxy-connections=false\nenable-rcon=false\nenable-query=false\nenable-status=true\nlevel-name=world\n"));
    result
}

fn server_memory_arguments(major:u32, economy:bool)->Vec<&'static str> {
    if economy && major>=17 {vec!["-XX:+UseG1GC","-XX:G1PeriodicGCInterval=60000"]} else {Vec::new()}
}

fn server_java_compatibility(major:u32)->Vec<&'static str> {
    let mut flags=Vec::new();
    if major>=17 {flags.push("--enable-native-access=ALL-UNNAMED");}
    if major>=24 {flags.push("--sun-misc-unsafe-memory-access=allow");}
    flags
}

impl LocalServerManager {
    pub fn new(root: PathBuf, http: reqwest::Client) -> Arc<Self> {
        Arc::new(Self {
            root,
            http,
            runtime: Mutex::new(None),
            changes: tokio::sync::Mutex::new(()),
            shutting_down: AtomicBool::new(false),
            content_jobs: AtomicUsize::new(0),
        })
    }

    fn change(&self) -> AppResult<tokio::sync::MutexGuard<'_, ()>> {
        self.changes.try_lock().map_err(|_| unavailable("Há uma operação de servidor em andamento. Aguarde a preparação atual antes de iniciar ou alterar conteúdos; Parar e salvar continua disponível."))
    }

    pub(crate) fn content_job(&self) -> AppResult<ContentJob<'_>> {
        let _guard = self.change()?;
        if self.shutting_down.load(Ordering::Acquire) {
            return Err(unavailable("O Luxmc está encerrando."));
        }
        self.content_jobs.fetch_add(1, Ordering::AcqRel);
        Ok(ContentJob(&self.content_jobs))
    }

    pub(crate) fn cancellation(&self) -> &AtomicBool {
        &self.shutting_down
    }

    pub fn has_active(&self) -> bool {
        self.runtime
            .lock()
            .unwrap_or_else(|error| error.into_inner())
            .as_ref()
            .is_some_and(|runtime| runtime.active())
    }

    async fn root(&self) -> AppResult<PathBuf> {
        tokio::fs::create_dir_all(&self.root).await?;
        Ok(tokio::fs::canonicalize(&self.root).await?)
    }

    pub async fn directory(&self, id: &str) -> AppResult<PathBuf> {
        if id.len() != 36 || uuid::Uuid::parse_str(id).is_err() {
            return Err(invalid("Identificador de servidor inválido."));
        }
        let root = self.root().await?;
        let path = tokio::fs::canonicalize(root.join(id)).await?;
        if !path.starts_with(&root) || path == root {
            return Err(invalid("A pasta do servidor está fora dos dados do Luxmc."));
        }
        Ok(path)
    }

    async fn read(&self, directory: &Path, name: &str) -> AppResult<Option<Vec<u8>>> {
        let path = directory.join(name);
        match tokio::fs::metadata(&path).await {
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(None),
            Err(error) => Err(error.into()),
            Ok(metadata) => {
                if !metadata.is_file()
                    || metadata.len() > 65536
                    || !tokio::fs::canonicalize(&path).await?.starts_with(directory)
                {
                    return Err(invalid("Arquivo de configuração inválido; os arquivos existentes foram preservados."));
                }
                Ok(Some(tokio::fs::read(path).await?))
            }
        }
    }

    pub async fn config(&self, id: &str) -> AppResult<LocalServerConfig> {
        let directory = self.directory(id).await?;
        let bytes = self
            .read(&directory, "luxmc-server.json")
            .await?
            .ok_or_else(|| unavailable("Configuração do servidor não encontrada."))?;
        let config: LocalServerConfig = serde_json::from_slice(&bytes)?;
        validate_config(&config)?;
        if config.id != id {
            return Err(invalid("A configuração pertence a outro servidor."));
        }
        Ok(config)
    }

    pub async fn list(&self, after: u64) -> AppResult<ServerOverview> {
        let mut entries = tokio::fs::read_dir(self.root().await?).await?;
        let mut servers = Vec::new();
        let mut warnings = Vec::new();
        let mut icons = std::collections::BTreeMap::new();
        while let Some(entry) = entries.next_entry().await? {
            let id = entry.file_name().to_string_lossy().into_owned();
            if uuid::Uuid::parse_str(&id).is_err() {
                continue;
            }
            if servers.len() + warnings.len() >= 64 {
                warnings.push(
                    "Há mais de 64 pastas de servidor. Abra a pasta de dados para conferir.".into(),
                );
                break;
            }
            match self.config(&id).await {
                Ok(config) => {
                    if let Ok(directory) = self.directory(&id).await {
                        if let Ok(Some(bytes)) = self.read(&directory, "server-icon.png").await {
                            if bytes.starts_with(b"\x89PNG\r\n\x1a\n") {
                                use base64::Engine;
                                icons.insert(
                                    id.clone(),
                                    format!(
                                        "data:image/png;base64,{}",
                                        base64::engine::general_purpose::STANDARD.encode(bytes)
                                    ),
                                );
                            }
                        }
                    }
                    servers.push(config);
                }
                Err(error) => warnings.push(format!("{id}: {error}")),
            }
        }
        servers.sort_by(|first, second| {
            first
                .name
                .to_lowercase()
                .cmp(&second.name.to_lowercase())
                .then(first.id.cmp(&second.id))
        });
        Ok(ServerOverview {
            servers,
            warnings,
            icons,
            active: self.status(after),
        })
    }

    pub fn status(&self, after: u64) -> Option<ServerState> {
        self.runtime
            .lock()
            .unwrap_or_else(|error| error.into_inner())
            .as_ref()
            .map(|runtime| runtime.snapshot(after))
    }

    pub async fn create(&self, input: CreateServer) -> AppResult<LocalServerConfig> {
        validate_memory(input.ram_mb)?;
        let _guard = self.change()?;
        if self.shutting_down.load(Ordering::Acquire) {
            return Err(unavailable("O Luxmc está encerrando."));
        }
        let mut config = LocalServerConfig {
            id: uuid::Uuid::new_v4().to_string(),
            name: input.name.trim().into(),
            mc_version: input.mc_version,
            ram_mb: input.ram_mb,
            max_players: input.max_players,
            eula_accepted: input.eula_accepted,
            artifact: ServerArtifact {
                url: "https://piston-data.mojang.com/server.jar".into(),
                sha1: "0".repeat(40),
                size: 1,
                java_major: 21,
            },
        };
        validate_config(&config)?;
        if self.list(0).await?.servers.len() >= 32 {
            return Err(unavailable(
                "O limite de 32 servidores locais foi atingido.",
            ));
        }
        let manifest = crate::core::minecraft::fetch_version_manifest(&self.http).await?;
        let version = manifest
            .versions
            .iter()
            .find(|version| version.id == config.mc_version)
            .ok_or_else(|| invalid("A versão não está no catálogo oficial do Minecraft."))?;
        if !valid_official_url(&version.url, true) {
            return Err(invalid("Endereço dos metadados inválido."));
        }
        let response = self
            .http
            .get(&version.url)
            .timeout(Duration::from_secs(20))
            .send()
            .await?
            .error_for_status()?;
        if response.content_length().unwrap_or(0) > 4 * 1024 * 1024 {
            return Err(invalid("Metadados do Minecraft muito grandes."));
        }
        let bytes = response.bytes().await?;
        if bytes.len() > 4 * 1024 * 1024 {
            return Err(invalid("Metadados do Minecraft muito grandes."));
        }
        let detail: serde_json::Value = serde_json::from_slice(&bytes)?;
        if detail.get("id").and_then(|value| value.as_str()) != Some(config.mc_version.as_str()) {
            return Err(invalid(
                "Os metadados recebidos pertencem a outra versão do Minecraft.",
            ));
        }
        let server = detail.pointer("/downloads/server").ok_or_else(|| {
            invalid("Esta versão não oferece um servidor oficial para instalação automática.")
        })?;
        let artifact: crate::core::minecraft::DownloadEntry =
            serde_json::from_value(server.clone())?;
        config.artifact = ServerArtifact {
            url: artifact.url,
            size: artifact.size,
            sha1: artifact.sha1.to_ascii_lowercase(),
            java_major: detail
                .pointer("/javaVersion/majorVersion")
                .and_then(|value| value.as_u64())
                .and_then(|value| u32::try_from(value).ok())
                .unwrap_or_else(|| {
                    crate::core::minecraft::detect_java_major_from_version_id(&config.mc_version)
                }),
        };
        validate_config(&config)?;
        let base=self.root().await?;
        let final_directory=base.join(&config.id);
        let directory=base.join(format!(".luxmc-creating-{}",config.id));
        tokio::fs::create_dir(&directory).await?;
        let result=async {
            crate::core::managed_skin::prepare(&directory,true).await?;
        crate::core::mods::pack_download::atomic_write(
            &directory.join("luxmc-server.json"),
            &serde_json::to_vec_pretty(&config)?,
        )
        .await?;
        let settings = settings::ServerSettings {
            motd: config.name.clone(),
            ..Default::default()
        };
        crate::core::mods::pack_download::atomic_write(
            &directory.join("luxmc-settings.json"),
            &serde_json::to_vec_pretty(&settings)?,
        )
        .await?;
            tokio::fs::rename(&directory,&final_directory).await?;
            Ok::<_,AppError>(())
        }.await;
        if let Err(error)=result {
            let _=tokio::task::spawn_blocking(move ||backups::remove_owned_tree(&base,&directory)).await;
            return Err(error);
        }
        Ok(config)
    }

    pub async fn start(self: &Arc<Self>, id: String, capacity: u16) -> AppResult<ServerState> {
        let _guard = self.change()?;
        if self.content_jobs.load(Ordering::Acquire) > 0 {
            return Err(unavailable("Aguarde o download ou a importação de conteúdos terminar antes de iniciar o servidor."));
        }
        if self.shutting_down.load(Ordering::Acquire) || self.has_active() {
            return Err(unavailable("Pare o servidor atual antes de iniciar outro."));
        }
        let config = self.config(&id).await?;
        let directory = self.directory(&id).await?;
        validate_memory(config.ram_mb)?;
        if directory.join(".luxmc-instance-pending").exists() {
            return Err(unavailable("A cópia da instância ainda não foi concluída. Em Conteúdos → Arquivos, importe novamente a instância e aplique os arquivos antes de iniciar."));
        }
        if self.contents(&id).await?.iter().any(|item| item.staged) {
            return Err(unavailable("Existem conteúdos preparados. Aplique ou cancele as pendências na aba Conteúdos antes de iniciar."));
        }
        self.validate_installed_mods(&id).await?;
        if config.ram_mb as i64 > (crate::core::optimizer::get_total_memory_mb() - 1024).max(512) {
            return Err(invalid(
                "Reserve pelo menos 1 GB de RAM para o sistema e reduza a memória do servidor.",
            ));
        }
        let (stop, _) = watch::channel(false);
        let (finished, _) = watch::channel(false);
        let (commands, receiver) = mpsc::channel(16);
        let history_path = directory.join("luxmc-players.json");
        let mut history: players::PlayerHistory = match tokio::fs::metadata(&history_path).await {
            Ok(metadata)
                if metadata.len() <= 1024 * 1024
                    && tokio::fs::canonicalize(&history_path)
                        .await?
                        .starts_with(&directory) =>
            {
                serde_json::from_slice(&tokio::fs::read(&history_path).await?)?
            }
            Ok(_) => {
                return Err(invalid(
                    "Histórico de jogadores inválido; arquivo preservado.",
                ))
            }
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                players::PlayerHistory::default()
            }
            Err(error) => return Err(error.into()),
        };
        history.offline();
        history.complete = true;
        let settings = self.settings(&id).await?;
        let autosave_minutes = settings.autosave_minutes;
        let runtime = Arc::new(Runtime {
            state: Mutex::new(ServerState {
                id,
                generation: uuid::Uuid::new_v4().to_string(),
                phase: "preparing".into(),
                port: None,
                pid: None,
                max_players: config.max_players.min(capacity.clamp(1, 20)),
                players_online: 0,
                error: None,
                exit_code: None,
                console: Vec::new(),
                cursor: 0,
                last_saved_at: None,
                forced_stop: false,
                players: Vec::new(),
                history_complete: true,
            }),
            lines: Mutex::new(VecDeque::new()),
            sequence: AtomicU64::new(0),
            stop,
            finished,
            commands,
            ready_log: AtomicBool::new(false),
            history: Mutex::new(history),
            history_path: Some(history_path),
            history_dirty: AtomicBool::new(false),
            autosave_minutes,
            control: Mutex::new(None),
            startup_commands: settings.startup_commands(&config.mc_version),
        });
        *self
            .runtime
            .lock()
            .unwrap_or_else(|error| error.into_inner()) = Some(runtime.clone());
        let manager = self.clone();
        let task_runtime = runtime.clone();
        tokio::spawn(async move {
            let result = manager
                .run(&config, &directory, &task_runtime, receiver)
                .await;
            task_runtime.stop.send_replace(true);
            let owner = {
                let state = task_runtime
                    .state
                    .lock()
                    .unwrap_or_else(|error| error.into_inner());
                format!("{}:{}", state.id, state.generation)
            };
            {
                let mut state = task_runtime
                    .state
                    .lock()
                    .unwrap_or_else(|error| error.into_inner());
                state.pid = None;
                state.players_online = 0;
                match result {
                    Ok(code) => {
                        state.exit_code = code;
                        state.phase = "stopped".into();
                        if state.forced_stop {
                            state.error = Some("O servidor precisou ser encerrado à força. O último salvamento não foi confirmado; confira o mundo e os backups antes de continuar.".into());
                        }
                    }
                    Err(error) => {
                        state.phase = "failed".into();
                        state.error = Some(crate::core::process::sanitize_diagnostic(
                            &error.to_string(),
                        ));
                    }
                }
            }
            task_runtime
                .history
                .lock()
                .unwrap_or_else(|error| error.into_inner())
                .offline();
            task_runtime.history_dirty.store(true, Ordering::Release);
            task_runtime.flush_history().await;
            task_runtime.finished.send_replace(true);
            crate::network::resource_packs::clear_host(&owner).await;
            let _ = crate::network::p2p_tunnel::stop_owned_server(&owner).await;
        });
        Ok(runtime.snapshot(0))
    }

    pub async fn update(
        &self,
        id: &str,
        name: String,
        ram_mb: u32,
        max_players: u16,
    ) -> AppResult<LocalServerConfig> {
        validate_memory(ram_mb)?;
        let _guard = self.change()?;
        if self.shutting_down.load(Ordering::Acquire) || self.current(id).is_ok() {
            return Err(unavailable(
                "Pare o servidor antes de alterar suas configurações.",
            ));
        }
        let mut config = self.config(id).await?;
        config.name = name.trim().into();
        config.ram_mb = ram_mb;
        config.max_players = max_players;
        validate_config(&config)?;
        let directory = self.directory(id).await?;
        crate::core::mods::pack_download::atomic_write(
            &directory.join("luxmc-server.json"),
            &serde_json::to_vec_pretty(&config)?,
        )
        .await?;
        Ok(config)
    }

    async fn prepare(
        &self,
        config: &LocalServerConfig,
        directory: &Path,
        runtime: &Runtime,
    ) -> AppResult<(PathBuf, u16)> {
        runtime.phase("downloading");
        crate::core::managed_skin::prepare(directory,true).await?;
        let jar = directory.join("server.jar");
        if self.reuse_server_artifact(config,&jar).await? {runtime.log("luxmc","Servidor oficial reaproveitado de outra instalação após verificar a integridade; download repetido evitado.");}
        if jar.exists() && !tokio::fs::canonicalize(&jar).await?.starts_with(directory) {
            return Err(invalid(
                "O arquivo do servidor está fora da pasta gerenciada.",
            ));
        }
        crate::core::downloader::ensure_artifact(
            &self.http,
            &jar,
            &config.artifact.url,
            config.artifact.size,
            &config.artifact.sha1,
        )
        .await?;
        runtime.phase("java");
        let data = self
            .root
            .parent()
            .ok_or_else(|| invalid("Pasta de dados inválida."))?;
        let java =
            crate::core::java::JavaRuntimeManager::new(self.http.clone(), data.to_path_buf())
                .ensure_java(config.artifact.java_major)
                .await?;
        let mut settings = self.settings(&config.id).await?;
        settings.validate()?;
        if !settings.resource_pack_file.is_empty() {
            let path=directory.join("resourcepacks").join(&settings.resource_pack_file);
            let check=path.clone();
            tokio::task::spawn_blocking(move || content::validate_archive(&check,"resourcepack")).await.map_err(|error|AppError::Internal(error.to_string()))??;
            let pack=crate::network::resource_packs::host(&config.id,path).await?;
            settings.resource_pack_url=pack.url();settings.resource_pack_sha1=pack.sha1;
        }
        let listener = tokio::net::TcpListener::bind(("127.0.0.1", settings.port)).await.map_err(|error| unavailable(format!("A porta local {} não está disponível: {error}. Confira as configurações do servidor.", settings.port)))?;
        let port = listener.local_addr()?.port();
        let control_listener=tokio::net::TcpListener::bind(("127.0.0.1",0)).await?;
        let endpoint=control::Endpoint{port:control_listener.local_addr()?.port(),password:uuid::Uuid::new_v4().simple().to_string()};
        drop(control_listener);
        drop(listener);
        *runtime.control.lock().unwrap_or_else(|error|error.into_inner())=Some(endpoint.clone());
        settings.port = port;
        crate::core::mods::pack_download::atomic_write(
            &directory.join("luxmc-settings.json"),
            &serde_json::to_vec_pretty(&settings)?,
        )
        .await?;
        for name in ["world", "logs"] {
            let path = directory.join(name);
            if path.exists() && !tokio::fs::canonicalize(&path).await?.starts_with(directory) {
                return Err(invalid("Uma pasta do servidor aponta para dados externos."));
            }
        }
        let previous = self.read(directory, "server.properties").await?.map(|bytes| String::from_utf8_lossy(&bytes).into_owned()).unwrap_or_else(|| "motd=Luxmc Local Server\ndifficulty=normal\ngamemode=survival\nview-distance=8\nsimulation-distance=6\n".into());
        let players = runtime
            .state
            .lock()
            .unwrap_or_else(|error| error.into_inner())
            .max_players;
        crate::core::mods::pack_download::atomic_write(
            &directory.join("server.properties"),
            control::properties(&settings.apply(&server_properties(&previous, port, players)),&endpoint).as_bytes(),
        )
        .await?;
        self.read(directory, "eula.txt").await?;
        crate::core::mods::pack_download::atomic_write(&directory.join("eula.txt"), b"eula=true\n")
            .await?;
        Ok((java, port))
    }

    async fn run(
        &self,
        config: &LocalServerConfig,
        directory: &Path,
        runtime: &Arc<Runtime>,
        receiver: mpsc::Receiver<ConsoleCommand>,
    ) -> AppResult<Option<i32>> {
        let mut stop = runtime.stop.subscribe();
        let prepared = tokio::select! { biased; _ = wait_stop(&mut stop) => return Ok(None), result = self.prepare(config, directory, runtime) => result? };
        if *stop.borrow() {
            return Ok(None);
        }
        runtime.phase("starting");
        let (java, port) = prepared;
        let loader_arguments = tokio::select! { biased; _ = wait_stop(&mut stop) => return Ok(None), result = self.prepare_loader(config, directory, &java, runtime) => result? };
        if *stop.borrow() {
            return Ok(None);
        }
        let settings = self.settings(&config.id).await?;
        let mut command = crate::core::process::tokio_command(java);
        command
            .current_dir(crate::core::process::java_working_directory(directory))
            .args(["-Dterminal.jline=false","-Dterminal.ansi=false"])
            .args(server_memory_arguments(config.artifact.java_major, settings.economy_mode))
            .args(["-Xms256M", &format!("-Xmx{}M", config.ram_mb)])
            .args(server_java_compatibility(config.artifact.java_major))
            .args(loader_arguments)
            .arg("nogui")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .kill_on_drop(true);
        #[cfg(target_os = "linux")]
        for variable in [
            "LD_LIBRARY_PATH",
            "LD_PRELOAD",
            "APPDIR",
            "APPIMAGE",
            "OWD",
            "GIO_MODULE_DIR",
            "GTK_PATH",
            "GSETTINGS_SCHEMA_DIR",
        ] {
            command.env_remove(variable);
        }
        #[cfg(target_os = "linux")]
        process_guard::prepare(&mut command);
        let child = command.spawn()?;
        let result = supervise(
            child,
            port,
            runtime.clone(),
            receiver,
            &self.http,
            Duration::from_secs(300),
            Duration::from_secs(20),
        )
        .await;
        let _ = public_tunnel::stop(&config.id).await;
        result
    }

    fn current(&self, id: &str) -> AppResult<Arc<Runtime>> {
        self.runtime
            .lock()
            .unwrap_or_else(|error| error.into_inner())
            .as_ref()
            .filter(|runtime| runtime.snapshot(u64::MAX).id == id && runtime.active())
            .cloned()
            .ok_or_else(|| unavailable("Este servidor não está em execução."))
    }

    pub fn running_port(&self, id: &str) -> AppResult<u16> {
        let runtime = self.current(id)?;
        let state = runtime.snapshot(u64::MAX);
        if state.phase != "running" {
            return Err(unavailable("Aguarde o servidor ficar pronto."));
        }
        state
            .port
            .ok_or_else(|| unavailable("Porta do servidor indisponível."))
    }

    pub(crate) fn stop_signal(&self, id: &str) -> AppResult<watch::Receiver<bool>> {
        Ok(self.current(id)?.stop.subscribe())
    }

    pub(crate) fn room_owner(&self, id: &str) -> AppResult<String> {
        let runtime = self.current(id)?;
        let state = runtime.snapshot(u64::MAX);
        Ok(format!("{}:{}", state.id, state.generation))
    }

    pub async fn stop(&self, id: &str) -> AppResult<()> {
        let runtime = match self.current(id) {
            Ok(runtime) => runtime,
            Err(_) => return Ok(()),
        };
        runtime.phase("stopping");
        let mut finished = runtime.finished.subscribe();
        runtime.stop.send_replace(true);
        tokio::time::timeout(Duration::from_secs(25), async {
            while !*finished.borrow_and_update() {
                if finished.changed().await.is_err() {
                    break;
                }
            }
        })
        .await
        .map_err(|_| {
            unavailable("O servidor ainda está encerrando. Aguarde antes de fechar o Luxmc.")
        })?;
        Ok(())
    }

    pub async fn console(&self, id: &str, text: String) -> AppResult<()> {
        let text = text.trim().trim_start_matches('/').to_string();
        if text.is_empty() || text.len() > 1024 || text.chars().any(char::is_control) {
            return Err(invalid("Digite um único comando de até 1024 bytes."));
        }
        if text.eq_ignore_ascii_case("stop") {
            return self.stop(id).await;
        }
        let runtime = self.current(id)?;
        if !["running","unreachable"].contains(&runtime.snapshot(u64::MAX).phase.as_str()) {
            return Err(unavailable(
                "Aguarde o servidor ficar pronto antes de enviar comandos.",
            ));
        }
        let (reply, result) = oneshot::channel();
        runtime
            .commands
            .try_send(ConsoleCommand { text, reply })
            .map_err(|_| unavailable("O console está ocupado. Aguarde e tente novamente."))?;
        tokio::time::timeout(Duration::from_secs(5), result)
            .await
            .map_err(|_| unavailable("O console não respondeu."))?
            .map_err(|_| unavailable("O servidor encerrou antes de receber o comando."))?
            .map_err(unavailable)
    }

    pub async fn shutdown(&self) -> AppResult<()> {
        self.shutting_down.store(true, Ordering::Release);
        let _guard = self.changes.lock().await;
        let id = self
            .status(u64::MAX)
            .filter(|_| self.has_active())
            .map(|state| state.id);
        if let Some(id) = id {
            self.stop(&id).await?;
        }
        Ok(())
    }
}

pub(crate) async fn wait_stop(receiver: &mut watch::Receiver<bool>) {
    while !*receiver.borrow_and_update() {
        if receiver.changed().await.is_err() {
            break;
        }
    }
}

async fn read_console(reader: impl AsyncRead + Unpin, runtime: Arc<Runtime>, stream: &'static str) {
    let mut reader = BufReader::new(reader);
    loop {
        match crate::core::process::read_log_line(&mut reader).await {
            Ok(Some(line)) => runtime.log(stream, &line),
            Ok(None) => break,
            Err(error) => {
                runtime.log("luxmc", &error.to_string());
                break;
            }
        }
    }
}

async fn supervise(
    mut child: tokio::process::Child,
    port: u16,
    runtime: Arc<Runtime>,
    mut receiver: mpsc::Receiver<ConsoleCommand>,
    http: &reqwest::Client,
    startup_timeout: Duration,
    stop_timeout: Duration,
) -> AppResult<Option<i32>> {
    #[cfg(windows)]
    let _job = process_guard::contain(&child)?;
    let mut input = child
        .stdin
        .take()
        .ok_or_else(|| unavailable("O console do servidor não abriu."))?;
    let stdout = child
        .stdout
        .take()
        .ok_or_else(|| unavailable("A saída do servidor não abriu."))?;
    let stderr = child
        .stderr
        .take()
        .ok_or_else(|| unavailable("A saída de erros do servidor não abriu."))?;
    {
        let mut state = runtime
            .state
            .lock()
            .unwrap_or_else(|error| error.into_inner());
        state.pid = child.id();
        state.port = Some(port);
    }
    let readers = [
        tokio::spawn(read_console(stdout, runtime.clone(), "stdout")),
        tokio::spawn(read_console(stderr, runtime.clone(), "stderr")),
    ];
    let mut stop = runtime.stop.subscribe();
    let started = tokio::time::Instant::now();
    let mut stopping = None;
    let mut failure = None;
    let mut interval = tokio::time::interval(Duration::from_secs(2));
    let mut last_autosave = tokio::time::Instant::now();
    let mut startup_applied = false;
    let mut missed_checks=0u8;
    let result = loop {
        tokio::select! {
            biased;
            result = child.wait() => break result.map_err(AppError::from),
            _ = wait_stop(&mut stop), if stopping.is_none() => {
                runtime.phase("stopping");
                let _ = control::command(&runtime,&mut input,"stop").await;
                stopping = Some(tokio::time::Instant::now());
            }
            Some(command) = receiver.recv(), if stopping.is_none() => {
                let result = control::command(&runtime,&mut input,&command.text).await;
                let outcome = match result { Ok(()) => { runtime.log("command", &command.text); Ok(()) }, Err(error) => Err(error.to_string()) };
                let _ = command.reply.send(outcome);
            }
            _ = interval.tick() => {
                if stopping.is_some_and(|time| time.elapsed() >= stop_timeout) {
                    runtime.state.lock().unwrap_or_else(|error| error.into_inner()).forced_stop = true;
                    runtime.log("luxmc", "O processo não respondeu ao comando stop; encerrando somente este servidor.");
                    if let Err(error) = child.start_kill() { break Err(error.into()); }
                    break child.wait().await.map_err(AppError::from);
                }
                if stopping.is_none() {
                    runtime.flush_history().await;
                    if runtime.autosave_minutes > 0 && last_autosave.elapsed() >= Duration::from_secs(u64::from(runtime.autosave_minutes) * 60) && runtime.snapshot(u64::MAX).phase == "running" {
                        if control::command(&runtime,&mut input,"save-all flush").await.is_ok() { runtime.log("luxmc", "Salvamento periódico solicitado; aguardando confirmação do servidor."); }
                        else { runtime.log("luxmc", "Não foi possível enviar o salvamento periódico. Confira o console."); }
                        last_autosave = tokio::time::Instant::now();
                    }
                    let ping = tokio::time::timeout(Duration::from_millis(1200), crate::core::server::ping_async(http, "127.0.0.1", port)).await;
                    let online = match ping { Ok(Ok(status)) if status.online => { runtime.state.lock().unwrap_or_else(|error| error.into_inner()).players_online = status.players_online; true }, _ => false };
                    if runtime.ready_log.load(Ordering::Acquire) && online {
                        missed_checks=0;
                        if !startup_applied {
                            for command in &runtime.startup_commands {
                                match control::command(&runtime,&mut input,command).await {Ok(())=>runtime.log("command",command),Err(error)=>runtime.log("luxmc",&format!("Não foi possível confirmar a aplicação de {command}: {error}"))};
                            }
                            startup_applied = true;
                        }
                        runtime.phase("running");
                    }
                    else if startup_applied {missed_checks=missed_checks.saturating_add(1);if missed_checks>=3{runtime.phase("unreachable");}}
                    else if started.elapsed() >= startup_timeout && runtime.snapshot(u64::MAX).phase == "starting" { failure = Some(unavailable("O servidor não ficou pronto no prazo. Confira o console e a memória disponível.")); runtime.stop.send_replace(true); }
                }
            }
        }
    };
    for mut reader in readers {
        if tokio::time::timeout(Duration::from_secs(1), &mut reader)
            .await
            .is_err()
        {
            reader.abort();
            let _ = reader.await;
        }
    }
    if let Some(error) = failure {
        return Err(error);
    }
    let status = result?;
    if stopping.is_none() && (!status.success() || runtime.snapshot(u64::MAX).phase == "starting") {
        return Err(unavailable(format!(
            "O servidor encerrou com código {}. Confira o console.",
            status
                .code()
                .map_or_else(|| "desconhecido".into(), |code| code.to_string())
        )));
    }
    Ok(status.code())
}

#[cfg(test)]
mod tests {
    use super::*;

    pub(super) fn fixture_config() -> LocalServerConfig {
        LocalServerConfig {
            id: uuid::Uuid::new_v4().to_string(),
            name: "Test server".into(),
            mc_version: "1.21.1".into(),
            ram_mb: 512,
            max_players: 20,
            eula_accepted: true,
            artifact: ServerArtifact {
                url: "https://piston-data.mojang.com/v1/objects/fixture/server.jar".into(),
                sha1: "a".repeat(40),
                size: 100,
                java_major: 21,
            },
        }
    }

    pub(super) fn fixture_runtime(id: String) -> (Arc<Runtime>, mpsc::Receiver<ConsoleCommand>) {
        let (stop, _) = watch::channel(false);
        let (finished, _) = watch::channel(false);
        let (commands, receiver) = mpsc::channel(16);
        (
            Arc::new(Runtime {
                state: Mutex::new(ServerState {
                    id,
                    generation: uuid::Uuid::new_v4().to_string(),
                    phase: "starting".into(),
                    port: None,
                    pid: None,
                    max_players: 10,
                    players_online: 0,
                    error: None,
                    exit_code: None,
                    console: Vec::new(),
                    cursor: 0,
                    last_saved_at: None,
                    forced_stop: false,
                    players: Vec::new(),
                    history_complete: true,
                }),
                stop,
                finished,
                commands,
                sequence: AtomicU64::new(0),
                lines: Mutex::new(VecDeque::new()),
                ready_log: AtomicBool::new(false),
                history: Mutex::new(players::PlayerHistory::default()),
                history_path: None,
                history_dirty: AtomicBool::new(false),
                autosave_minutes: 0,
                control: Mutex::new(None),
                startup_commands: Vec::new(),
            }),
            receiver,
        )
    }

    pub(super) async fn persist(
        manager: &LocalServerManager,
        config: &LocalServerConfig,
    ) -> PathBuf {
        let path = manager.root().await.unwrap().join(&config.id);
        tokio::fs::create_dir(&path).await.unwrap();
        tokio::fs::write(
            path.join("luxmc-server.json"),
            serde_json::to_vec(config).unwrap(),
        )
        .await
        .unwrap();
        path
    }

    #[tokio::test]
    async fn eula_and_configuration_are_checked_before_network_or_world_writes() {
        let root = tempfile::tempdir().unwrap();
        let manager = LocalServerManager::new(root.path().join("servers"), reqwest::Client::new());
        let input = CreateServer {
            name: "World".into(),
            mc_version: "1.21.1".into(),
            ram_mb: 512,
            max_players: 10,
            eula_accepted: false,
        };
        assert!(manager
            .create(input)
            .await
            .unwrap_err()
            .to_string()
            .contains("EULA"));
        assert!(!root.path().join("servers").exists());
        let mut config = fixture_config();
        assert!(validate_config(&config).is_ok());
        config.artifact.url = "https://piston-data.mojang.com.evil.test/server.jar".into();
        assert!(validate_config(&config).is_err());
        config = fixture_config();
        config.mc_version = "../world".into();
        assert!(validate_config(&config).is_err());
        assert!(manager.directory("../outside").await.is_err());
    }

    #[test]
    fn loopback_and_authentication_override_old_properties_without_changing_world_settings() {
        let text = server_properties("server-ip:0.0.0.0\nserver-port=25565\nonline-mode=false\nmax-players=99\nlevel-name=../personal-world\nlevel-seed=42\ndifficulty=hard\n", 45000, 20);
        for expected in [
            "server-ip=127.0.0.1",
            "server-port=45000",
            "online-mode=true",
            "max-players=20",
            "level-name=world",
            "enable-rcon=false",
            "level-seed=42",
            "difficulty=hard",
        ] {
            assert!(text.lines().any(|line| line == expected));
        }
        assert!(
            !text.contains("0.0.0.0")
                && !text.contains("online-mode=false")
                && !text.contains("personal-world")
        );
        assert_eq!(
            text.lines()
                .filter(|line| line.starts_with("max-players="))
                .count(),
            1
        );
    }

    #[tokio::test]
    async fn configuration_changes_and_cancelled_preparation_preserve_worlds_and_exclusive_process_ownership(
    ) {
        let root = tempfile::tempdir().unwrap();
        let manager = LocalServerManager::new(root.path().join("servers"), reqwest::Client::new());
        let config = fixture_config();
        let path = persist(&manager, &config).await;
        tokio::fs::create_dir(path.join("world")).await.unwrap();
        tokio::fs::write(path.join("world/level.dat"), b"personal-world")
            .await
            .unwrap();
        let updated = manager
            .update(&config.id, "New name".into(), 1024, 15)
            .await
            .unwrap();
        assert_eq!(updated.mc_version, config.mc_version);
        assert_eq!(updated.artifact.sha1, config.artifact.sha1);
        let status = manager.start(config.id.clone(), 10).await.unwrap();
        assert_eq!(status.max_players, 10);
        assert!(manager.start(config.id.clone(), 20).await.is_err());
        assert!(manager
            .update(&config.id, "Busy".into(), 512, 10)
            .await
            .is_err());
        assert!(manager
            .console(&config.id, "say hello\nstop".into())
            .await
            .is_err());
        manager.stop("another-server").await.unwrap();
        assert!(manager.has_active());
        manager.current(&config.id).unwrap().stop.send_replace(true);
        manager.stop(&config.id).await.unwrap();
        assert!(!manager.has_active());
        assert_eq!(manager.status(0).unwrap().phase, "stopped");
        assert!(!path.join("server.jar").exists());
        assert_eq!(
            tokio::fs::read(path.join("world/level.dat")).await.unwrap(),
            b"personal-world"
        );
        manager.shutdown().await.unwrap();
        assert!(manager.start(config.id, 10).await.is_err());
    }

    #[test]
    fn memory_migration_reads_old_configs_but_requires_explicit_adjustment_before_launch() {
        let mut config = fixture_config();
        config.ram_mb = 16384;
        assert!(validate_config(&config).is_ok());
        assert!(validate_memory(config.ram_mb).is_err());
        for value in [512, 2048, 8192] {
            assert!(validate_memory(value).is_ok());
        }
        for value in [0, 511, 8193, 65536] {
            assert!(validate_memory(value).is_err());
        }
    }

    #[test]
    fn save_timestamp_requires_a_minecraft_confirmation_and_not_just_a_request() {
        let (runtime, _) = fixture_runtime("fixture".into());
        runtime.log("command", "save-all flush");
        assert!(runtime.snapshot(0).last_saved_at.is_none());
        runtime.log(
            "stdout",
            "[12:00:00] [Server thread/INFO]: <Player> Saved the game",
        );
        assert!(runtime.snapshot(0).last_saved_at.is_none());
        runtime.log("stdout", "[12:00:00] [Server thread/INFO]: Saved the game");
        assert!(runtime.snapshot(0).last_saved_at.is_some());
    }

    #[test]
    fn console_is_bounded_incremental_and_redacts_credentials() {
        let (runtime, _) = fixture_runtime("fixture".into());
        for index in 0..550 {
            runtime.log("stdout", &format!("line {index}"));
        }
        runtime.log("stderr", "access_token=private-secret");
        let state = runtime.snapshot(540);
        assert_eq!(state.console.len(), 11);
        assert_eq!(state.cursor, 551);
        assert_eq!(runtime.snapshot(0).console.len(), 500);
        assert!(!state
            .console
            .last()
            .unwrap()
            .text
            .contains("private-secret"));
    }

    #[test]
    #[ignore = "child-process fixture, invoked only by the isolated supervisor tests"]
    fn server_process_fixture() {
        use std::io::{BufRead, Write};
        let Ok(mode) = std::env::var("LUXMC_LOCAL_SERVER_FIXTURE") else {
            return;
        };
        if mode == "crash" {
            eprintln!("Fixture server startup failure");
            std::process::exit(7);
        }
        if mode != "slow" {
            let port: u16 = std::env::var("LUXMC_LOCAL_SERVER_FIXTURE_PORT")
                .unwrap()
                .parse()
                .unwrap();
            let listener = std::net::TcpListener::bind(("127.0.0.1", port)).unwrap();
            std::thread::spawn(move || {
                fn variable(bytes:&mut Vec<u8>,mut value:usize){loop{let mut byte=(value&127) as u8;value>>=7;if value>0{byte|=128;}bytes.push(byte);if value==0{break;}}}
                for connection in listener.incoming() {
                    if let Ok(mut stream)=connection {
                        let _=stream.set_read_timeout(Some(Duration::from_millis(500)));
                        fn packet(stream:&mut std::net::TcpStream)->std::io::Result<()> {
                            let mut length=0usize;let mut shift=0;
                            loop {let mut byte=[0u8;1];std::io::Read::read_exact(stream,&mut byte)?;length|=usize::from(byte[0]&127)<<shift;if byte[0]&128==0{break;}shift+=7;if shift>28{return Err(std::io::ErrorKind::InvalidData.into());}}
                            if length>1024{return Err(std::io::ErrorKind::InvalidData.into());}
                            let mut body=vec![0u8;length];std::io::Read::read_exact(stream,&mut body)
                        }
                        if packet(&mut stream).is_err() || packet(&mut stream).is_err(){continue;}
                        let json=br#"{"version":{"name":"fixture","protocol":767},"players":{"max":10,"online":0},"description":{"text":"fixture"}}"#;
                        let mut body=vec![0];variable(&mut body,json.len());body.extend_from_slice(json);let mut packet=Vec::new();variable(&mut packet,body.len());packet.extend_from_slice(&body);let _=std::io::Write::write_all(&mut stream,&packet);
                    }
                }
            });
            println!("Done (0.1s)! For help, type help");
            std::io::stdout().flush().unwrap();
        }
        for line in std::io::stdin().lock().lines() {
            let line = line.unwrap();
            if line == "stop" && mode != "hang" {
                std::fs::write(
                    std::env::var_os("LUXMC_LOCAL_SERVER_FIXTURE_SAVED").unwrap(),
                    b"saved",
                )
                .unwrap();
                std::process::exit(0);
            }
            println!("received: {line}");
            std::io::stdout().flush().unwrap();
        }
    }

    #[tokio::test]
    async fn supervisor_saves_on_stop_force_stops_only_its_child_and_reports_early_exit() {
        for mode in ["graceful", "hang", "crash", "slow"] {
            let root = tempfile::tempdir().unwrap();
            let saved = root.path().join("saved.txt");
            let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
            let port = listener.local_addr().unwrap().port();
            drop(listener);
            let (runtime, receiver) = fixture_runtime(mode.into());
            let mut command = crate::core::process::tokio_command(std::env::current_exe().unwrap());
            command
                .args([
                    "--ignored",
                    "--exact",
                    "core::local_server::tests::server_process_fixture",
                    "--nocapture",
                ])
                .env("LUXMC_LOCAL_SERVER_FIXTURE", mode)
                .env("LUXMC_LOCAL_SERVER_FIXTURE_PORT", port.to_string())
                .env("LUXMC_LOCAL_SERVER_FIXTURE_SAVED", &saved)
                .stdin(Stdio::piped())
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .kill_on_drop(true);
            let child = command.spawn().unwrap();
            let observed = runtime.clone();
            let stop = async {
                if matches!(mode, "graceful" | "hang") {
                    tokio::time::timeout(Duration::from_secs(15), async {
                        while observed.snapshot(0).phase != "running" {
                            tokio::time::sleep(Duration::from_millis(10)).await;
                        }
                    })
                    .await
                    .unwrap();
                    observed.stop.send_replace(true);
                }
            };
            let http = reqwest::Client::new();
            let run = supervise(
                child,
                port,
                runtime.clone(),
                receiver,
                &http,
                if mode == "slow" {
                    Duration::from_millis(20)
                } else {
                    Duration::from_secs(20)
                },
                Duration::from_millis(100),
            );
            let (result, ()) =
                tokio::time::timeout(Duration::from_secs(25), async { tokio::join!(run, stop) })
                    .await
                    .unwrap();
            if mode == "graceful" {
                assert_eq!(result.unwrap(), Some(0));
                assert_eq!(std::fs::read(&saved).unwrap(), b"saved");
            } else if mode == "hang" {
                assert!(result.is_ok());
                assert!(!saved.exists());
                assert!(runtime
                    .snapshot(0)
                    .console
                    .iter()
                    .any(|line| line.text.contains("somente este servidor")));
            } else {
                assert!(result.is_err());
            }
        }
    }
}
