use serde::Serialize;

use crate::error::{AppError, AppResult};

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EnvCheckResult {
    pub ok: bool,
    pub issues: Vec<EnvIssue>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EnvIssue {
    pub code: String,
    pub message: String,
    pub fix: String,
}

#[tauri::command]
pub async fn env_check() -> AppResult<EnvCheckResult> {
    let mut issues = Vec::new();

    let base_dir = directories::ProjectDirs::from("io", "github", "Luxmc")
        .ok_or_else(|| AppError::InvalidState("could not determine data dir".into()))?;
    let data_dir = base_dir.data_dir().to_path_buf();

    if let Err(_) = tokio::fs::create_dir_all(&data_dir).await {
        issues.push(EnvIssue {
            code: "NO_WRITE_ACCESS".into(),
            message: format!("Sem permissão de escrita em {}", data_dir.display()),
            fix: "Verifique as permissões do diretório de dados.".into(),
        });
    }

    match disk_free_bytes(&data_dir) {
        Some(bytes) => {
            let free_gb = bytes as f64 / (1024.0 * 1024.0 * 1024.0);
            if free_gb < 2.0 {
                issues.push(EnvIssue {
                    code: "LOW_DISK_SPACE".into(),
                    message: format!(
                        "Espaço insuficiente: {:.1} GB livres (mínimo 2 GB)",
                        free_gb
                    ),
                    fix: "Libere espaço em disco antes de continuar.".into(),
                });
            }
        }
        None => {
            issues.push(EnvIssue {
                code: "DISK_CHECK_FAILED".into(),
                message: "Não foi possível verificar o espaço em disco.".into(),
                fix: "Verifique manualmente o espaço disponível.".into(),
            });
        }
    }

    match java_version() {
        Some(v) => {
            tracing::info!(java_version = %v, "system java detected");
        }
        None => {
            issues.push(EnvIssue {
				code: "NO_JAVA".into(),
				message: "Java não encontrado no sistema. O Luxmc pode baixar automaticamente, mas é recomendado ter Java instalado.".into(),
				fix: "Instale o Java (JRE/JDK) no sistema se desejar usar Java global.".into(),
			});
        }
    }

    Ok(EnvCheckResult {
        ok: issues.is_empty(),
        issues,
    })
}

fn disk_free_bytes(path: &std::path::Path) -> Option<u64> {
    let disks = sysinfo::Disks::new_with_refreshed_list();
    let mut best_match: Option<(&sysinfo::Disk, usize)> = None;
    for disk in disks.list() {
        let mount = disk.mount_point();
        if path.starts_with(mount) {
            let len = mount.as_os_str().len();
            if best_match.map_or(true, |(_, best_len)| len > best_len) {
                best_match = Some((disk, len));
            }
        }
    }
    if let Some((disk, _)) = best_match {
        Some(disk.available_space())
    } else {
        disks.list().first().map(|d| d.available_space())
    }
}

fn java_version() -> Option<String> {
    let output = std::process::Command::new("java")
        .arg("-version")
        .stderr(std::process::Stdio::piped())
        .output()
        .ok()?;
    let stderr = String::from_utf8_lossy(&output.stderr);
    let version_line = stderr.lines().next()?;
    let start = version_line.find('"')? + 1;
    let end = version_line[start..].find('"')? + start;
    Some(version_line[start..end].to_string())
}
