use std::collections::VecDeque;
use std::path::Path;
use std::time::Duration;

use tokio::io::AsyncReadExt;

use crate::error::{AppError, AppResult};

const OUTPUT_CAP: usize = 8 * 1024;
const DEFAULT_TIMEOUT: Duration = Duration::from_secs(300);

pub struct HookEnv<'a> {
    pub profile_id: &'a str,
    pub profile_name: &'a str,
    pub version_id: &'a str,
    pub game_dir: &'a Path,
    pub exit_code: Option<i32>,
}

impl HookEnv<'_> {
    fn vars(&self) -> Vec<(String, String)> {
        let mut vars: Vec<(String, String)> = vec![
            ("LUXMC_PROFILE_ID".into(), self.profile_id.to_string()),
            ("LUXMC_PROFILE_NAME".into(), self.profile_name.to_string()),
            ("LUXMC_VERSION_ID".into(), self.version_id.to_string()),
            (
                "LUXMC_GAME_DIR".into(),
                self.game_dir.to_string_lossy().to_string(),
            ),
        ];
        if let Some(code) = self.exit_code {
            vars.push(("LUXMC_EXIT_CODE".into(), code.to_string()));
            vars.push((
                "LUXMC_SUCCESS".into(),
                if code == 0 { "1".into() } else { "0".into() },
            ));
        }
        vars
    }
}

async fn drain<R: tokio::io::AsyncRead + Unpin>(mut pipe: R) -> Vec<u8> {
    let mut kept: VecDeque<u8> = VecDeque::new();
    let mut chunk = [0u8; 8192];
    loop {
        match pipe.read(&mut chunk).await {
            Ok(0) => break,
            Ok(n) => {
                kept.extend(&chunk[..n]);
                while kept.len() > OUTPUT_CAP {
                    kept.pop_front();
                }
            }
            Err(_) => break,
        }
    }
    kept.into_iter().collect()
}

fn last_line(bytes: &[u8]) -> Option<String> {
    String::from_utf8_lossy(bytes)
        .lines()
        .rev()
        .find(|l| !l.trim().is_empty())
        .map(|l| l.trim().to_string())
}

fn run_command(script: &str) -> tokio::process::Command {
    let mut cmd = if cfg!(windows) {
        let mut c = crate::core::process::tokio_command("cmd");
        c.arg("/C").arg(script);
        c
    } else {
        let mut c = crate::core::process::tokio_command("sh");
        c.arg("-c").arg(script);
        c
    };
    cmd.kill_on_drop(true);
    cmd
}

pub async fn run(label: &str, script: &str, env: &HookEnv<'_>) -> AppResult<()> {
    let trimmed = script.trim();
    if trimmed.is_empty() {
        return Ok(());
    }

    let mut cmd = run_command(trimmed);
    cmd.current_dir(env.game_dir)
        .envs(env.vars())
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped());

    tracing::info!(target: "hooks", "Executando hook {} em {}: {}", label, env.game_dir.display(), trimmed);

    let mut child = cmd.spawn().map_err(|e| {
        AppError::InvalidState(format!("Não foi possível executar o hook {label} ({trimmed}): {e}"))
    })?;

    let stdout_pipe = child.stdout.take();
    let stderr_pipe = child.stderr.take();
    let out_task = tokio::spawn(async move {
        match stdout_pipe {
            Some(p) => drain(p).await,
            None => Vec::new(),
        }
    });
    let err_task = tokio::spawn(async move {
        match stderr_pipe {
            Some(p) => drain(p).await,
            None => Vec::new(),
        }
    });

    let wait_result = tokio::time::timeout(DEFAULT_TIMEOUT, child.wait()).await;

    if wait_result.is_err() {
        let _ = child.kill().await;
    }

    let drain_grace = Duration::from_secs(5);
    let stdout = match tokio::time::timeout(drain_grace, out_task).await {
        Ok(Ok(bytes)) => bytes,
        Ok(Err(error)) => {
            tracing::warn!(target: "hooks", "[{label}] falha ao ler stdout do hook: {error}");
            Vec::new()
        }
        Err(_) => Vec::new(),
    };
    let stderr = match tokio::time::timeout(drain_grace, err_task).await {
        Ok(Ok(bytes)) => bytes,
        Ok(Err(error)) => {
            tracing::warn!(target: "hooks", "[{label}] falha ao ler stderr do hook: {error}");
            Vec::new()
        }
        Err(_) => Vec::new(),
    };

    if !stdout.is_empty() {
        tracing::info!(target: "hooks", "[{label}] {}", String::from_utf8_lossy(&stdout));
    }
    if !stderr.is_empty() {
        tracing::warn!(target: "hooks", "[{label}] {}", String::from_utf8_lossy(&stderr));
    }

    match wait_result {
        Err(_) => Err(AppError::InvalidState(format!(
            "Hook {label} excedeu o tempo limite de {}s e foi interrompido ({trimmed})",
            DEFAULT_TIMEOUT.as_secs()
        ))),
        Ok(Err(e)) => Err(AppError::InvalidState(format!(
            "Falha ao aguardar o hook {label} ({trimmed}): {e}"
        ))),
        Ok(Ok(status)) if !status.success() => {
            let code = status
                .code()
                .map(|c| c.to_string())
                .unwrap_or_else(|| "sinal".into());
            let detail = last_line(&stderr).or_else(|| last_line(&stdout));
            Err(AppError::InvalidState(match detail {
                Some(d) => format!("Hook {label} falhou com código {code} ({trimmed}): {d}"),
                None => format!("Hook {label} falhou com código {code} ({trimmed})"),
            }))
        }
        Ok(Ok(_)) => Ok(()),
    }
}
