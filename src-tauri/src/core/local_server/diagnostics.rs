use super::*;
use serde_json::{json, Value};
use std::net::Ipv4Addr;
use tokio::io::{AsyncReadExt, AsyncSeekExt};

fn status_label(status: &str) -> &'static str {
    match status {
        "approved" => "Aprovado",
        "attention" => "Atenção",
        "failed" => "Falhou",
        _ => "Não verificado",
    }
}

fn check(id: &str, label: &str, status: &str, detail: impl Into<String>) -> Value {
    json!({
        "id": id,
        "label": label,
        "status": status,
        "statusLabel": status_label(status),
        "detail": detail.into(),
    })
}

enum TcpProbe {
    Connected,
    Refused,
    Timeout,
    Failed(String),
}

async fn tcp_probe(host: &str, port: u16) -> TcpProbe {
    match tokio::time::timeout(
        Duration::from_secs(2),
        tokio::net::TcpStream::connect((host, port)),
    )
    .await
    {
        Err(_) => TcpProbe::Timeout,
        Ok(Ok(_stream)) => TcpProbe::Connected,
        Ok(Err(error)) if error.kind() == std::io::ErrorKind::ConnectionRefused => {
            TcpProbe::Refused
        }
        Ok(Err(error)) if error.kind() == std::io::ErrorKind::TimedOut => TcpProbe::Timeout,
        Ok(Err(error)) => TcpProbe::Failed(error.to_string()),
    }
}

fn lan_ipv4s() -> Vec<Ipv4Addr> {
    let networks = sysinfo::Networks::new_with_refreshed_list();
    let mut found: Vec<Ipv4Addr> = Vec::new();
    for (_, network) in networks.iter() {
        for ip in network.ip_networks() {
            if let std::net::IpAddr::V4(address) = ip.addr {
                if address.is_private() && !found.contains(&address) {
                    found.push(address);
                }
            }
        }
    }
    found.sort();
    found
}

async fn powershell(script: &str) -> Option<String> {
    let output = tokio::time::timeout(
        Duration::from_secs(12),
        {
            let mut command = tokio::process::Command::new("powershell.exe");
            command.args(["-NoProfile", "-NonInteractive", "-ExecutionPolicy", "Bypass", "-Command", script]);
            command.stdin(Stdio::null());
            command.stdout(Stdio::piped());
            command.stderr(Stdio::null());
            #[cfg(windows)]
            command.creation_flags(crate::core::process::CREATE_NO_WINDOW);
            command.output()
        },
    )
    .await
    .ok()?
    .ok()?;
    if !output.status.success() {
        return None;
    }
    Some(String::from_utf8_lossy(&output.stdout).into_owned())
}

async fn listening_sockets(port: u16) -> Option<Vec<(String, u32, String)>> {
    let script = format!(
        "$ErrorActionPreference='SilentlyContinue'; $r=@(); Get-NetTCPConnection -LocalPort {port} -State Listen -ErrorAction SilentlyContinue | Select-Object -First 8 | ForEach-Object {{ $n=(Get-Process -Id $_.OwningProcess -ErrorAction SilentlyContinue).ProcessName; $r+=,([pscustomobject]@{{address=$_.LocalAddress;pid=[int]$_.OwningProcess;name=$(if($n){{$n}} else {{'?'}})}}) }}; ConvertTo-Json -InputObject $r -Compress"
    );
    let text = powershell(&script).await?;
    let value: Value = serde_json::from_str(text.trim()).ok()?;
    let items = match value {
        Value::Array(items) => items,
        Value::Null => Vec::new(),
        other => vec![other],
    };
    Some(
        items
            .into_iter()
            .map(|item| {
                (
                    item.get("address")
                        .and_then(Value::as_str)
                        .unwrap_or("?")
                        .to_string(),
                    item.get("pid")
                        .and_then(Value::as_u64)
                        .and_then(|pid| u32::try_from(pid).ok())
                        .unwrap_or(0),
                    item.get("name").and_then(Value::as_str).unwrap_or("?").to_string(),
                )
            })
            .collect(),
    )
}

async fn firewall_report(port: u16, java: Option<&Path>) -> Option<Value> {
    let java = java
        .map(|path| path.to_string_lossy().replace('\'', "''"))
        .unwrap_or_default();
    let script = format!(
        "$ErrorActionPreference='SilentlyContinue'; $enabled=@(); Get-NetFirewallProfile | ForEach-Object {{ $enabled+=,(\"$($_.Name)=$($_.Enabled)\") }}; $rules=@(); Get-NetFirewallPortFilter -Protocol TCP | Where-Object {{ $_.LocalPort -eq '{port}' }} | ForEach-Object {{ $r=Get-NetFirewallRule -InstanceId $_.InstanceID -ErrorAction SilentlyContinue; if($r -and $r.Direction -eq 'Inbound' -and $r.Action -eq 'Allow' -and \"$($r.Enabled)\" -eq 'True') {{ $rules+=,$r.DisplayName }} }}; if('{java}') {{ Get-NetFirewallApplicationFilter -Program '{java}' -ErrorAction SilentlyContinue | ForEach-Object {{ $r=Get-NetFirewallRule -InstanceId $_.InstanceID -ErrorAction SilentlyContinue; if($r -and $r.Direction -eq 'Inbound' -and $r.Action -eq 'Allow' -and \"$($r.Enabled)\" -eq 'True') {{ $rules+=,$r.DisplayName }} }} }}; ConvertTo-Json -InputObject @{{profiles=$enabled;rules=$rules}} -Compress"
    );
    let text = powershell(&script).await?;
    serde_json::from_str(text.trim()).ok()
}

async fn sha1_of(path: PathBuf) -> AppResult<String> {
    tokio::task::spawn_blocking(move || -> AppResult<String> {
        use sha1::{Digest, Sha1};
        use std::io::Read;
        let mut file = std::fs::File::open(&path)?;
        let mut hasher = Sha1::new();
        let mut buffer = [0u8; 64 * 1024];
        loop {
            let read = file.read(&mut buffer)?;
            if read == 0 {
                break;
            }
            hasher.update(&buffer[..read]);
        }
        Ok(format!("{:x}", hasher.finalize()))
    })
    .await
    .map_err(|error| AppError::Internal(error.to_string()))?
}

async fn java_version(path: &Path) -> Result<String, String> {
    let output = tokio::time::timeout(
        Duration::from_secs(8),
        {
            let mut command = tokio::process::Command::new(path);
            command.arg("-version");
            command.stdin(Stdio::null());
            command.stdout(Stdio::piped());
            command.stderr(Stdio::piped());
            #[cfg(windows)]
            command.creation_flags(crate::core::process::CREATE_NO_WINDOW);
            command.output()
        },
    )
    .await;
    let output = match output {
        Err(_) => return Err("o Java não respondeu em 8 s".into()),
        Ok(Err(error)) => return Err(format!("não foi possível executar o Java: {error}")),
        Ok(Ok(output)) => output,
    };
    if !output.status.success() {
        return Err("o Java retornou erro ao executar -version".into());
    }
    let combined = format!(
        "{}{}",
        String::from_utf8_lossy(&output.stderr),
        String::from_utf8_lossy(&output.stdout)
    );
    combined
        .lines()
        .map(str::trim)
        .find(|line| line.contains("version"))
        .map(str::to_string)
        .ok_or_else(|| "a versão do Java não pôde ser identificada".into())
}

impl LocalServerManager {
    async fn public_ipv4(&self) -> Result<Ipv4Addr, String> {
        for url in ["https://api.ipify.org", "https://checkip.amazonaws.com"] {
            let response = self
                .http
                .get(url)
                .timeout(Duration::from_secs(4))
                .send()
                .await;
            if let Ok(response) = response {
                if response.status().is_success() {
                    if let Ok(text) = response.text().await {
                        if let Ok(address) = text.trim().parse::<Ipv4Addr>() {
                            return Ok(address);
                        }
                    }
                }
            }
        }
        Err("nenhum serviço de consulta respondeu".into())
    }

    fn is_cgnat(address: Ipv4Addr) -> bool {
        address.octets()[0] == 100 && (64..=127).contains(&address.octets()[1])
    }

    pub async fn connection_diagnostic(&self, id: &str) -> AppResult<Value> {
        let config = self.config(id).await?;
        let directory = self.directory(id).await?;
        let settings = self.settings(id).await.unwrap_or_default();
        let loader = self.loader(id).await.unwrap_or_default();
        let runtime = self.runtime_for(id);
        let state = runtime.as_ref().map(|runtime| runtime.snapshot(u64::MAX));
        let phase = state
            .as_ref()
            .map(|state| state.phase.clone())
            .unwrap_or_else(|| "stopped".into());
        let active = runtime.as_ref().is_some_and(|runtime| runtime.active());
        let pid = state.as_ref().and_then(|state| state.pid);
        let port = state
            .as_ref()
            .and_then(|state| state.port)
            .unwrap_or(if settings.port == 0 { 25565 } else { settings.port });
        let listening_phase = matches!(phase.as_str(), "running" | "unreachable");

        let jar = directory.join("server.jar");
        let eula_ok = tokio::fs::read_to_string(directory.join("eula.txt"))
            .await
            .map(|text| text.contains("eula=true"))
            .unwrap_or(false);
        let installation = match tokio::fs::metadata(&jar).await {
            Err(_) => check(
                "installation",
                "Integridade da instalação",
                "failed",
                "server.jar ausente. Reinstale o servidor em Conteúdos ou crie uma nova instância.",
            ),
            Ok(metadata) if metadata.len() == 0 => check(
                "installation",
                "Integridade da instalação",
                "failed",
                "server.jar existe mas está vazio; reinstale o servidor.",
            ),
            Ok(metadata) => {
                let vanilla = loader.kind == "vanilla"
                    && metadata.len() == config.artifact.size
                    && config.artifact.sha1.len() == 40;
                let verdict = if vanilla {
                    match sha1_of(jar.clone()).await {
                        Ok(hash) if hash.eq_ignore_ascii_case(&config.artifact.sha1) => Some(true),
                        Ok(_) => Some(false),
                        Err(_) => None,
                    }
                } else {
                    None
                };
                let mut detail = match verdict {
                    Some(true) => "server.jar verificado: o SHA-1 confere com o pacote oficial.".to_string(),
                    Some(false) => "O SHA-1 do server.jar difere do pacote oficial — o arquivo está corrompido; reinstale o servidor.".to_string(),
                    None if loader.kind != "vanilla" => format!(
                        "Instalação com {} presente; o hash oficial não se aplica a este loader.",
                        loader.kind
                    ),
                    None => "server.jar presente, mas o hash oficial não pôde ser verificado agora.".to_string(),
                };
                if !eula_ok {
                    detail.push_str(" eula.txt não foi aceito.");
                }
                let status = if !eula_ok || verdict == Some(false) {
                    "failed"
                } else if verdict.is_none() && loader.kind == "vanilla" {
                    "attention"
                } else {
                    "approved"
                };
                check("installation", "Integridade da instalação", status, detail)
            }
        };

        let data = self
            .root
            .parent()
            .ok_or_else(|| invalid("Pasta de dados inválida."))?;
        let java_manager = crate::core::java::JavaRuntimeManager::new(
            self.http.clone(),
            data.to_path_buf(),
        );
        let major = config.artifact.java_major;
        let java_path = java_manager.locate_existing(major);
        let java_check = match &java_path {
            None => check(
                "java",
                "Runtime Java",
                "attention",
                format!("Java {major} ainda não está instalado; ele será baixado no primeiro início do servidor."),
            ),
            Some(path) => match java_version(path).await {
                Ok(version) => check(
                    "java",
                    "Runtime Java",
                    "approved",
                    format!("Java {major} disponível: {version}"),
                ),
                Err(reason) => check(
                    "java",
                    "Runtime Java",
                    "failed",
                    format!("Java {major} encontrado em {} mas não executou: {reason}.", path.display()),
                ),
            },
        };

        let process_check = match &state {
            None => check(
                "process",
                "Processo do servidor",
                "unverified",
                "Servidor parado — nenhum processo em execução para verificar.",
            ),
            Some(state) if state.phase == "stopped" => check(
                "process",
                "Processo do servidor",
                "unverified",
                "Servidor parado — nenhum processo em execução para verificar.",
            ),
            Some(state) if state.phase == "failed" => check(
                "process",
                "Processo do servidor",
                "failed",
                format!(
                    "O servidor falhou: {}",
                    state.error.as_deref().unwrap_or("sem detalhe registrado.")
                ),
            ),
            Some(state) => match state.pid {
                None => check(
                    "process",
                    "Processo do servidor",
                    "attention",
                    format!("Servidor em preparação (fase {}); o processo Java ainda não foi criado.", state.phase),
                ),
                Some(pid) => {
                    let alive = {
                        let mut system = sysinfo::System::new();
                        let target = [sysinfo::Pid::from_u32(pid)];
                        system.refresh_processes_specifics(
                            sysinfo::ProcessesToUpdate::Some(&target),
                            true,
                            sysinfo::ProcessRefreshKind::nothing(),
                        );
                        system.process(sysinfo::Pid::from_u32(pid)).is_some()
                    };
                    if alive {
                        check(
                            "process",
                            "Processo do servidor",
                            "approved",
                            format!("Processo Java em execução (PID {pid}), fase {}.", state.phase),
                        )
                    } else {
                        check(
                            "process",
                            "Processo do servidor",
                            "failed",
                            format!("O PID {pid} não existe mais; o processo pode ter caído."),
                        )
                    }
                }
            },
        };

        let startup_check = match phase.as_str() {
            "running" => check(
                "startup",
                "Conclusão da inicialização",
                "approved",
                "Inicialização concluída: o servidor registrou o sinal de prontidão e responde ao ping de status.",
            ),
            "preparing" | "downloading" | "java" | "starting" => check(
                "startup",
                "Conclusão da inicialização",
                "attention",
                format!("Servidor ainda inicializando (fase {phase}); aguarde a mensagem de prontidão no console."),
            ),
            "unreachable" => check(
                "startup",
                "Conclusão da inicialização",
                "attention",
                "O processo está ativo, mas o servidor parou de responder ao ping de status.",
            ),
            "stopping" => check(
                "startup",
                "Conclusão da inicialização",
                "attention",
                "Servidor em encerramento.",
            ),
            "failed" => check(
                "startup",
                "Conclusão da inicialização",
                "failed",
                format!(
                    "A inicialização falhou: {}",
                    state.as_ref().and_then(|s| s.error.as_deref()).unwrap_or("sem detalhe registrado.")
                ),
            ),
            _ => check(
                "startup",
                "Conclusão da inicialização",
                "unverified",
                "Servidor parado — a inicialização não foi verificada.",
            ),
        };

        let (sockets, firewall, public, (local_check, ping, tcp_reachable)) = tokio::join!(
            listening_sockets(port),
            firewall_report(port, java_path.as_deref()),
            self.public_ipv4(),
            async {
                if !active {
                    return (
                        check(
                            "localhost",
                            "Conexão local (127.0.0.1) e Server List Ping",
                            "unverified",
                            "Servidor offline — o teste local não foi executado.",
                        ),
                        None,
                        false,
                    );
                }
                if !listening_phase {
                    return (
                        check(
                            "localhost",
                            "Conexão local (127.0.0.1) e Server List Ping",
                            "attention",
                            format!("Servidor em inicialização (fase {phase}); a porta ainda não aceita conexões."),
                        ),
                        None,
                        false,
                    );
                }
                match tcp_probe("127.0.0.1", port).await {
                    TcpProbe::Connected => {
                        let status = tokio::time::timeout(
                            Duration::from_secs(3),
                            crate::core::server::ping_async(&self.http, "127.0.0.1", port),
                        )
                        .await;
                        match status {
                            Ok(Ok(status)) if status.online => (
                                check(
                                    "localhost",
                                    "Conexão local (127.0.0.1) e Server List Ping",
                                    "approved",
                                    format!(
                                        "Server List Ping válida em 127.0.0.1:{port} — {} ({} / {} jogadores, {} ms).",
                                        status.version,
                                        status.players_online,
                                        status.players_max,
                                        status.latency_ms.unwrap_or(0)
                                    ),
                                ),
                                Some(status),
                                true,
                            ),
                            _ => (
                                check(
                                    "localhost",
                                    "Conexão local (127.0.0.1) e Server List Ping",
                                    "attention",
                                    "O TCP conectou, mas o protocolo Minecraft não respondeu — servidor inicializando ou a porta pertence a outro aplicativo.",
                                ),
                                None,
                                true,
                            ),
                        }
                    }
                    TcpProbe::Refused => (
                        check(
                            "localhost",
                            "Conexão local (127.0.0.1) e Server List Ping",
                            "failed",
                            format!("Conexão recusada em 127.0.0.1:{port} — o servidor não está aceitando conexões."),
                        ),
                        None,
                        false,
                    ),
                    TcpProbe::Timeout => (
                        check(
                            "localhost",
                            "Conexão local (127.0.0.1) e Server List Ping",
                            "failed",
                            format!("Tempo esgotado ao conectar em 127.0.0.1:{port} — firewall local bloqueando ou processo travado."),
                        ),
                        None,
                        false,
                    ),
                    TcpProbe::Failed(reason) => (
                        check(
                            "localhost",
                            "Conexão local (127.0.0.1) e Server List Ping",
                            "failed",
                            format!("Falha ao conectar em 127.0.0.1:{port}: {reason}"),
                        ),
                        None,
                        false,
                    ),
                }
            },
        );
        let port_check = if listening_phase {
            match &sockets {
                None => check(
                    "port",
                    "Disponibilidade da porta local",
                    "unverified",
                    format!("Não foi possível consultar a porta {port} com Get-NetTCPConnection."),
                ),
                Some(list) if list.iter().any(|(_, owner, _)| Some(*owner) == pid) => check(
                    "port",
                    "Disponibilidade da porta local",
                    "approved",
                    format!("A porta {port} está em uso pelo próprio processo do servidor."),
                ),
                Some(list) if !list.is_empty() => {
                    let (address, owner, name) = &list[0];
                    check(
                        "port",
                        "Disponibilidade da porta local",
                        "failed",
                        format!("A porta {port} está ocupada por {name} (PID {owner}) em {address}."),
                    )
                }
                Some(_) => check(
                    "port",
                    "Disponibilidade da porta local",
                    "attention",
                    format!("O processo ainda não abriu a porta {port}; ela foi validada como livre durante a preparação."),
                ),
            }
        } else {
            match tokio::net::TcpListener::bind(("0.0.0.0", port)).await {
                Ok(_free) => check(
                    "port",
                    "Disponibilidade da porta local",
                    "approved",
                    format!("A porta {port} está livre para o servidor."),
                ),
                Err(_) => {
                    let owner = sockets.as_ref().and_then(|list| list.first()).map(
                        |(address, owner, name)| format!(" Ocupada por {name} (PID {owner}) em {address}."),
                    ).unwrap_or_default();
                    let suggestion = match free_port_suggestion(port).await {
                        Some(candidate) => format!(" A porta {candidate} foi verificada e está livre; selecione-a nas configurações do servidor."),
                        None => " Nenhuma porta livre nas faixas usuais.".into(),
                    };
                    check(
                        "port",
                        "Disponibilidade da porta local",
                        "failed",
                        format!("A porta {port} já está em uso por outro processo.{owner}{suggestion}"),
                    )
                }
            }
        };

        let listening_check = if listening_phase {
            match &sockets {
                Some(list) if !list.is_empty() => {
                    let addresses = list
                        .iter()
                        .map(|(address, _, _)| format!("{address}:{port}"))
                        .collect::<Vec<_>>()
                        .join(", ");
                    check(
                        "listening",
                        "Socket de escuta real",
                        "approved",
                        format!("Socket de escuta ativo em {addresses}."),
                    )
                }
                Some(_) => check(
                    "listening",
                    "Socket de escuta real",
                    "failed",
                    format!("Nenhum socket de escuta na porta {port}; o processo ainda não abriu a porta."),
                ),
                None => check(
                    "listening",
                    "Socket de escuta real",
                    "unverified",
                    "Consulta de sockets indisponível neste sistema.",
                ),
            }
        } else {
            check(
                "listening",
                "Socket de escuta real",
                "unverified",
                "Servidor offline — sem socket de escuta para verificar.",
            )
        };

        let lan = lan_ipv4s();
        let lan_check = if lan.is_empty() {
            check(
                "lan",
                "Endereços da rede local",
                "attention",
                "Nenhuma interface de rede privada (10.x, 172.16-31.x ou 192.168.x) foi detectada neste computador.",
            )
        } else {
            let addresses = lan
                .iter()
                .map(|address| format!("{address}:{port}"))
                .collect::<Vec<_>>()
                .join(", ");
            check(
                "lan",
                "Endereços da rede local",
                "approved",
                format!("Use estes endereços em outros aparelhos da mesma rede: {addresses}."),
            )
        };

        let firewall_check = match firewall {
            None => check(
                "firewall",
                "Firewall do Windows",
                "unverified",
                "Não foi possível consultar as regras do firewall (Get-NetFirewallRule indisponível).",
            ),
            Some(report) => {
                let profiles = report
                    .get("profiles")
                    .and_then(Value::as_array)
                    .cloned()
                    .unwrap_or_default();
                let rules = report
                    .get("rules")
                    .and_then(Value::as_array)
                    .cloned()
                    .unwrap_or_default();
                let names = rules
                    .iter()
                    .filter_map(Value::as_str)
                    .collect::<Vec<_>>();
                let any_profile_enabled = profiles.iter().any(|profile| {
                    profile.as_str().is_some_and(|profile| profile.ends_with("=True"))
                });
                if !any_profile_enabled && !profiles.is_empty() {
                    check(
                        "firewall",
                        "Firewall do Windows",
                        "attention",
                        "Todos os perfis do firewall estão desativados. As conexões entram, mas a proteção do Windows está desligada; reative-o e use uma regra específica.",
                    )
                } else if !names.is_empty() {
                    check(
                        "firewall",
                        "Firewall do Windows",
                        "approved",
                        format!("Regra de entrada ativa para este servidor: {}.", names.join(", ")),
                    )
                } else {
                    check(
                        "firewall",
                        "Firewall do Windows",
                        "attention",
                        format!(
                            "Nenhuma regra de entrada conhecida para a porta {port} ou para o Java. Jogadores da rede local podem ser bloqueados. Se quiser permitir, execute manualmente como administrador: netsh advfirewall firewall add rule name=\"Luxmc Minecraft {port}\" dir=in action=allow protocol=TCP localport={port}. O Luxmc não altera o firewall automaticamente."
                        ),
                    )
                }
            }
        };

        let public_address = public.as_ref().ok().copied();
        let public_check = match &public {
            Err(_) => check(
                "public",
                "Endereço público",
                "unverified",
                "Não foi possível consultar o endereço público agora (rede ou serviço de consulta indisponível).",
            ),
            Ok(address) if Self::is_cgnat(*address) => check(
                "public",
                "Endereço público",
                "attention",
                format!("Seu provedor usa CGNAT ({address}); encaminhamento de portas padrão provavelmente não funcionará."),
            ),
            Ok(address) => check(
                "public",
                "Endereço público",
                "approved",
                format!("Endereço público deste computador: {address} (visível apenas nesta tela)."),
            ),
        };

        let router_check = match (&public, &lan) {
            (Ok(address), _) if Self::is_cgnat(*address) => check(
                "router",
                "Encaminhamento no roteador",
                "attention",
                "Como o endereço é CGNAT, pedir ao provedor ou usar o túnel Luxmc/hospedagem é mais confiável do que encaminhar portas.".to_string(),
            ),
            (Ok(address), lan) if !lan.is_empty() => check(
                "router",
                "Encaminhamento no roteador",
                "unverified",
                format!(
                    "Para jogadores de outra rede, encaminhe TCP {port} no roteador até {} e reserve um IP fixo para este computador. O Luxmc não acessa seu roteador e nada disso foi verificado.",
                    lan[0]
                ),
            ),
            (Ok(_), _) => check(
                "router",
                "Encaminhamento no roteador",
                "unverified",
                "Sem endereço de rede local detectado, não há para onde encaminhar a porta ainda.",
            ),
            (Err(_), _) => check(
                "router",
                "Encaminhamento no roteador",
                "unverified",
                "O endereço público é desconhecido agora; a orientação sobre o roteador fica pendente.",
            ),
        };

        let mut external_evidence = None;
        if listening_phase && active {
            if let Ok(address) = public {
                let probe = tcp_probe(&address.to_string(), port).await;
                external_evidence = Some(match probe {
                    TcpProbe::Connected => format!(
                        "Teste local ao endereço público {address}:{port}: conectou. Isso indica mapeamento ativo, mas NÃO substitui um teste de outra rede.",
                        address = address,
                        port = port
                    ),
                    TcpProbe::Refused => format!(
                        "Teste local ao endereço público {address}:{port}: recusado. Pode ser ausência de encaminhamento ou bloqueio; o resultado externo continua não verificado.",
                        address = address,
                        port = port
                    ),
                    TcpProbe::Timeout => format!(
                        "Teste local ao endereço público {address}:{port}: tempo esgotado (provável bloqueio ou CGNAT).",
                        address = address,
                        port = port
                    ),
                    TcpProbe::Failed(reason) => format!(
                        "Teste local ao endereço público {address}:{port}: {reason}"
                    ),
                });
            }
        }
        let mut external_detail = "Requer uma rede externa (por exemplo, dados móveis fora do Wi-Fi). O Luxmc não afirma conectividade externa sem esse teste. Procedimento: abra o Minecraft em outro aparelho fora da rede local e conecte em {public}:{port}.".to_string();
        if let Some(evidence) = external_evidence {
            external_detail.push_str(&format!(" {evidence}"));
        }
        let external_check = check(
            "external",
            "Conectividade externa",
            "unverified",
            external_detail
                .replace("{public}", public_address.map_or("SEU_IP_PUBLICO".into(), |address| address.to_string()).as_str())
                .replace("{port}", &port.to_string()),
        );

        let responding = ping.as_ref().is_some_and(|status| status.online);
        let ping_value = ping.as_ref().map(|status| {
            json!({
                "online": status.online,
                "version": status.version,
                "playersOnline": status.players_online,
                "playersMax": status.players_max,
                "motd": status.motd,
                "latencyMs": status.latency_ms,
            })
        });

        Ok(json!({
            "minecraftResponding": responding,
            "address": responding.then(|| format!("127.0.0.1:{port}")),
            "phase": phase,
            "tcpReachable": tcp_reachable,
            "port": port,
            "publicAddress": public_address.map(|address| address.to_string()),
            "lanAddresses": lan.iter().map(|address| format!("{address}:{port}")).collect::<Vec<_>>(),
            "checks": [installation, java_check, process_check, startup_check, port_check, listening_check, local_check, lan_check, firewall_check, public_check, router_check, external_check],
            "ping": ping_value,
            "generatedAt": chrono::Utc::now().to_rfc3339(),
        }))
    }

    pub async fn console_text(&self,id:&str)->AppResult<serde_json::Value> {
        let directory=self.directory(id).await?;
        let lines=self.runtime_for(id).map(|runtime|runtime.snapshot(0).console).unwrap_or_default();
        let path=directory.join("logs/latest.log");let mut text=String::new();let mut truncated=false;
        if path.exists() {
            backups::normal_file(&directory.join("logs"))?;let metadata=backups::normal_file(&path)?;
            if !metadata.is_file() || !tokio::fs::canonicalize(&path).await?.starts_with(&directory){return Err(invalid("Arquivo de console fora do servidor."));}
            let mut file=tokio::fs::File::open(path).await?;let limit=2*1024*1024;
            if metadata.len()>limit{truncated=true;file.seek(std::io::SeekFrom::End(-(limit as i64))).await?;}
            let mut bytes=Vec::new();file.take(limit).read_to_end(&mut bytes).await?;
            text=String::from_utf8_lossy(&bytes).lines().map(crate::core::process::sanitize_diagnostic).collect::<Vec<_>>().join("\n");
        }
        let extra=lines.iter().filter(|line|text.is_empty() || line.stream!="stdout").map(|line|format!("[{}] {}",line.stream,line.text)).collect::<Vec<_>>().join("\n");
        if !extra.is_empty(){if !text.is_empty(){text.push_str("\n\n--- Luxmc ---\n");}text.push_str(&extra);}
        Ok(serde_json::json!({"text":text,"truncated":truncated}))
    }

    pub async fn deletion_preview(&self,id:&str)->AppResult<serde_json::Value> {
        let directory=self.directory(id).await?;let root=self.root().await?;let id=id.to_string();
        tokio::task::spawn_blocking(move ||->AppResult<_>{
            let mut files=backups::tree_files(&directory,u64::MAX)?;
            let backup=root.join("backups").join(id);
            if backup.exists(){backups::normal_file(&root.join("backups"))?;files.extend(backups::tree_files(&backup,u64::MAX)?);}
            Ok(serde_json::json!({"files":files.len(),"bytes":files.iter().map(|(_,bytes)|*bytes).sum::<u64>()}))
        }).await.map_err(|error|AppError::Internal(error.to_string()))?
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn readiness_and_saving_require_trusted_server_messages() {
        let (runtime,_)=super::super::tests::fixture_runtime("fixture".into());
        runtime.log("stderr","Done (1s)!");
        runtime.log("stdout","[Server thread/INFO]: <Player> Done (1s)!");
        assert!(!runtime.ready_log.load(Ordering::Acquire));
        runtime.log("stdout","[Server thread/INFO]: Done (1s)! For help, type help");
        assert!(runtime.ready_log.load(Ordering::Acquire));
        runtime.log("stdout","[Server thread/INFO]: <Player> All dimensions are saved");
        assert!(runtime.snapshot(0).last_saved_at.is_none());
        runtime.log("stdout","[Server thread/INFO]: ThreadedAnvilChunkStorage: All dimensions are saved");
        assert!(runtime.snapshot(0).last_saved_at.is_some());
        let (forge,_)=super::super::tests::fixture_runtime("forge".into());
        forge.log("stdout","[21:00:00] [Server thread/INFO] [minecraft/MinecraftServer]: ThreadedAnvilChunkStorage: All dimensions are saved");
        assert!(forge.snapshot(0).last_saved_at.is_some());
    }

    #[tokio::test]
    async fn complete_deletion_preview_includes_owned_backups_and_preserves_other_servers() {
        let temporary=tempfile::tempdir().unwrap();
        let manager=LocalServerManager::new(temporary.path().join("servers"),reqwest::Client::new());
        let config=super::super::tests::fixture_config();let other=super::super::tests::fixture_config();
        let directory=super::super::tests::persist(&manager,&config).await;let preserved=super::super::tests::persist(&manager,&other).await;
        for name in ["world/region/world.mca","mods/mod.jar","libraries/a/a.jar"] {let path=directory.join(name);tokio::fs::create_dir_all(path.parent().unwrap()).await.unwrap();tokio::fs::write(path,b"owned").await.unwrap();}
        let backup=directory.parent().unwrap().join("backups").join(&config.id);tokio::fs::create_dir_all(&backup).await.unwrap();tokio::fs::write(backup.join("backup.zip"),b"backup").await.unwrap();
        let preview=manager.deletion_preview(&config.id).await.unwrap();assert_eq!(preview["files"],5);assert!(preview["bytes"].as_u64().unwrap()>21);
        manager.delete_server(&config.id,&config.name,true).await.unwrap();assert!(!directory.exists());assert!(!backup.exists());assert!(preserved.join("luxmc-server.json").exists());
    }

    #[tokio::test]
    async fn offline_diagnostic_reports_twelve_honest_checks_without_claiming_success() {
        let temporary=tempfile::tempdir().unwrap();
        let manager=LocalServerManager::new(temporary.path().join("servers"),reqwest::Client::new());
        let config=super::super::tests::fixture_config();
        let directory=super::super::tests::persist(&manager,&config).await;
        tokio::fs::write(directory.join("eula.txt"), b"eula=true\n").await.unwrap();
        let report=manager.connection_diagnostic(&config.id).await.unwrap();
        assert_eq!(report["checks"].as_array().unwrap().len(),12);
        assert_eq!(report["minecraftResponding"],false);
        assert!(report["address"].is_null());
        let ids=report["checks"].as_array().unwrap().iter().map(|item|item["id"].as_str().unwrap()).collect::<Vec<_>>();
        assert_eq!(ids,["installation","java","process","startup","port","listening","localhost","lan","firewall","public","router","external"]);
        for item in report["checks"].as_array().unwrap() {
            let status=item["status"].as_str().unwrap();
            assert!(["approved","attention","failed","unverified"].contains(&status));
            assert!(!item["detail"].as_str().unwrap().is_empty());
            if status=="approved" {
                assert!(item["statusLabel"].as_str().unwrap()=="Aprovado");
            }
        }
        let localhost=report["checks"].as_array().unwrap().iter().find(|item|item["id"]=="localhost").unwrap();
        assert_eq!(localhost["status"],"unverified");
        let external=report["checks"].as_array().unwrap().iter().find(|item|item["id"]=="external").unwrap();
        assert_eq!(external["status"],"unverified");
    }
}
