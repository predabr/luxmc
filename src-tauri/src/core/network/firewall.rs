use crate::error::{AppError, AppResult};

fn validate_port(port: u16) -> AppResult<()> {
    if port == 0 {
        return Err(AppError::InvalidInput(
            "A porta do Minecraft deve estar entre 1 e 65535.".into(),
        ));
    }
    Ok(())
}

#[cfg(windows)]
pub async fn set_minecraft_rule(port: u16, allow: bool) -> AppResult<()> {
    use tokio::process::Command;

    validate_port(port)?;
    let rule = format!("Luxmc-Minecraft-{port}");
    let arguments = if allow {
        format!("@('advfirewall','firewall','add','rule','name={rule}','dir=in','action=allow','protocol=TCP','localport={port}','profile=Private')")
    } else {
        format!(
            "@('advfirewall','firewall','delete','rule','name={rule}','dir=in','profile=Private')"
        )
    };
    let script = format!(
        "$arguments = {arguments}; try {{ $process = Start-Process -FilePath \"$env:SystemRoot\\System32\\netsh.exe\" -ArgumentList $arguments -Verb RunAs -Wait -PassThru; exit $process.ExitCode }} catch {{ [Console]::Error.WriteLine($_.Exception.Message); exit 1 }}"
    );
    let mut command = Command::new("powershell.exe");
    command.args(["-NoProfile", "-NonInteractive", "-Command", &script]);
    #[cfg(windows)]
    command.creation_flags(crate::core::process::CREATE_NO_WINDOW);
    let output = command.output().await?;
    if output.status.success() {
        return Ok(());
    }
    let detail = String::from_utf8_lossy(&output.stderr);
    Err(AppError::InvalidState(format!(
        "Não foi possível {} a regra TCP da porta {port} no perfil Privado do Firewall do Windows. A regra altera somente essa porta; aprove a solicitação do Windows ou tente novamente. {}",
        if allow { "criar" } else { "remover" },
        detail.trim()
    )))
}

#[cfg(target_os = "linux")]
pub async fn set_minecraft_rule(port: u16, allow: bool) -> AppResult<()> {
    validate_port(port)?;
    if which::which("pkexec").is_err() {
        return Err(AppError::InvalidState(
            "O pedido de autorização do sistema (pkexec) não está disponível. Nenhuma regra de firewall foi alterada.".into(),
        ));
    }

    if which::which("firewall-cmd").is_ok() {
        let operation = if allow { "add" } else { "remove" };
        let argument = format!("--{operation}-port={port}/tcp");
        run_privileged("firewall-cmd", &[&argument, "--permanent"]).await?;
        if let Err(error) = run_privileged("firewall-cmd", &["--reload"]).await {
            if allow {
                let remove = format!("--remove-port={port}/tcp");
                let rollback = run_privileged("firewall-cmd", &[&remove, "--permanent"]).await;
                let _ = run_privileged("firewall-cmd", &["--reload"]).await;
                if let Err(rollback_error) = rollback {
                    return Err(AppError::InvalidState(format!(
                        "{error} A limpeza da regra também falhou: {rollback_error}"
                    )));
                }
            }
            return Err(error);
        }
        return Ok(());
    }

    if which::which("ufw").is_ok() {
        let port = port.to_string();
        if allow {
            let status = run_privileged_output("ufw", &["status"]).await?;
            if !status.status.success() {
                let detail = String::from_utf8_lossy(&status.stderr);
                return Err(AppError::InvalidState(format!(
                    "Não foi possível verificar o estado do UFW: {}",
                    detail.trim()
                )));
            }
            if !String::from_utf8_lossy(&status.stdout)
                .to_ascii_lowercase()
                .contains("status: active")
            {
                return Err(AppError::InvalidState(
                    "O UFW está inativo. Ative o firewall do sistema antes de criar a regra TCP do Minecraft; nenhuma regra foi aberta.".into(),
                ));
            }
            run_privileged(
                "ufw",
                &["allow", "in", "proto", "tcp", "to", "any", "port", &port],
            )
            .await?;
        } else {
            run_privileged(
                "ufw",
                &[
                    "delete", "allow", "in", "proto", "tcp", "to", "any", "port", &port,
                ],
            )
            .await?;
        }
        return Ok(());
    }

    Err(AppError::InvalidState(
        "Não foi detectado UFW nem firewalld. Configure somente uma regra TCP para a porta do Minecraft no firewall do sistema e tente novamente.".into(),
    ))
}

#[cfg(target_os = "linux")]
async fn run_privileged(program: &str, arguments: &[&str]) -> AppResult<()> {
    let output = run_privileged_output(program, arguments).await?;
    if output.status.success() {
        return Ok(());
    }
    let detail = String::from_utf8_lossy(&output.stderr);
    Err(AppError::InvalidState(format!(
        "A autorização para alterar somente a regra TCP do Minecraft foi recusada ou falhou. {}",
        detail.trim()
    )))
}

#[cfg(target_os = "linux")]
async fn run_privileged_output(
    program: &str,
    arguments: &[&str],
) -> AppResult<std::process::Output> {
    use tokio::process::Command;

    let output = Command::new("pkexec")
        .arg(program)
        .args(arguments)
        .output()
        .await?;
    Ok(output)
}

#[cfg(not(any(windows, target_os = "linux")))]
pub async fn set_minecraft_rule(port: u16, _allow: bool) -> AppResult<()> {
    validate_port(port)?;
    Err(AppError::InvalidState(
        "A configuração automática e restrita do firewall está disponível no Windows e em Linux com UFW/firewalld. Nenhuma regra foi alterada.".into(),
    ))
}

#[cfg(test)]
mod tests {
    use super::set_minecraft_rule;

    #[tokio::test]
    async fn rejects_port_zero_without_requesting_firewall_authorization() {
        assert!(set_minecraft_rule(0, true).await.is_err());
    }
}
