use super::*;
use serde::{Deserialize, Serialize};

const MB: u64 = 1_048_576;
const HEAP_CACHE: Duration = Duration::from_millis(1200);

#[derive(Clone)]
pub(crate) struct HeapReading {
    pub used_mb: u64,
    pub committed_mb: u64,
}

#[derive(Default)]
pub(crate) struct TelemetryCache {
    pub heap: Option<(std::time::Instant, HeapReading)>,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ServerTelemetry {
    pub server_id: String,
    pub phase: String,
    pub online: bool,
    pub pid: Option<u32>,
    pub uptime_seconds: Option<u64>,
    pub port: Option<u16>,
    pub configured_ram_mb: u32,
    pub players_online: u32,
    pub max_players: u16,
    pub process_memory_mb: Option<u64>,
    pub process_cpu_percent: Option<f32>,
    pub heap_used_mb: Option<u64>,
    pub heap_committed_mb: Option<u64>,
    pub heap_max_mb: Option<u64>,
    pub heap_source: Option<String>,
    pub tps: Option<f64>,
    pub mspt: Option<f64>,
    pub system_cpu_percent: f32,
    pub system_used_memory_mb: u64,
    pub system_total_memory_mb: u64,
    pub system_available_memory_mb: u64,
    pub detail: String,
}

fn parse_heap_output(text: &str) -> Option<HeapReading> {
    let modern = regex::Regex::new(
        r"total(?:\s+reserved)?\s+(\d+)K,\s*committed\s+(\d+)K,\s*used\s+(\d+)K",
    )
    .ok()?;
    let legacy = regex::Regex::new(r"total\s+(\d+)K,\s*used\s+(\d+)K").ok()?;
    let mut committed = 0u64;
    let mut used = 0u64;
    let mut matches = 0usize;
    for capture in modern.captures_iter(text) {
        let (Some(committed_k), Some(used_k)) = (capture.get(2), capture.get(3)) else {
            continue;
        };
        let (Ok(committed_k), Ok(used_k)) = (
            committed_k.as_str().parse::<u64>(),
            used_k.as_str().parse::<u64>(),
        ) else {
            continue;
        };
        committed += committed_k;
        used += used_k;
        matches += 1;
    }
    if matches == 0 {
        for capture in legacy.captures_iter(text) {
            let (Some(total), Some(used_k)) = (capture.get(1), capture.get(2)) else {
                continue;
            };
            let (Ok(total), Ok(used_k)) = (total.as_str().parse::<u64>(), used_k.as_str().parse::<u64>())
            else {
                continue;
            };
            committed += total;
            used += used_k;
            matches += 1;
        }
    }
    (matches > 0 && committed > 0).then_some(HeapReading {
        used_mb: used / 1024,
        committed_mb: committed / 1024,
    })
}

async fn read_heap(java: &Path, pid: u32) -> Result<HeapReading, String> {
    let jcmd = java.with_file_name(if cfg!(windows) { "jcmd.exe" } else { "jcmd" });
    if !jcmd.exists() {
        return Err("este runtime Java não acompanha o jcmd".into());
    }
    let result = tokio::time::timeout(
        Duration::from_secs(4),
        {
            let mut command = tokio::process::Command::new(&jcmd);
            command.arg(pid.to_string());
            command.arg("GC.heap_info");
            command.stdin(Stdio::null());
            command.stdout(Stdio::piped());
            command.stderr(Stdio::piped());
            #[cfg(windows)]
            command.creation_flags(crate::core::process::CREATE_NO_WINDOW);
            command.output()
        },
    )
    .await;
    let output = match result {
        Err(_) => return Err("o jcmd não respondeu em 4 s".into()),
        Ok(Err(error)) => return Err(format!("não foi possível executar o jcmd: {error}")),
        Ok(Ok(output)) => output,
    };
    if !output.status.success() {
        let detail = String::from_utf8_lossy(&output.stderr)
            .lines()
            .chain(String::from_utf8_lossy(&output.stdout).lines())
            .find(|line| !line.trim().is_empty())
            .unwrap_or("o jcmd não retornou dados")
            .trim()
            .to_string();
        return Err(crate::core::process::sanitize_diagnostic(&detail));
    }
    parse_heap_output(&String::from_utf8_lossy(&output.stdout))
        .ok_or_else(|| "a saída do jcmd não pôde ser interpretada".into())
}

impl LocalServerManager {
    pub async fn telemetry(&self, id: &str) -> AppResult<ServerTelemetry> {
        let config = self.config(id).await?;
        let runtime = self.runtime_for(id);
        let state = runtime.as_ref().map(|runtime| runtime.snapshot(u64::MAX));
        let phase = state
            .as_ref()
            .map(|state| state.phase.clone())
            .unwrap_or_else(|| "stopped".into());
        let pid = state.as_ref().and_then(|state| state.pid);
        let active = runtime.as_ref().is_some_and(|runtime| runtime.active()) && pid.is_some();

        let mut process_memory_mb = None;
        let mut process_cpu_percent = None;
        let mut uptime_seconds = None;
        let mut detail = String::new();

        if active {
            if let Some(pid) = pid {
                let refreshed = {
                    let mut system = self
                        .telemetry_sys
                        .lock()
                        .unwrap_or_else(|error| error.into_inner());
                    let target = [sysinfo::Pid::from_u32(pid)];
                    system.refresh_processes_specifics(
                        sysinfo::ProcessesToUpdate::Some(&target),
                        true,
                        sysinfo::ProcessRefreshKind::nothing().with_memory().with_cpu(),
                    ) > 0
                        && system.process(sysinfo::Pid::from_u32(pid)).is_some()
                };
                if refreshed {
                    let system = self
                        .telemetry_sys
                        .lock()
                        .unwrap_or_else(|error| error.into_inner());
                    if let Some(process) = system.process(sysinfo::Pid::from_u32(pid)) {
                        process_memory_mb = Some(process.memory() / MB);
                        let cores = std::thread::available_parallelism()
                            .map(|value| value.get() as f32)
                            .unwrap_or(1.0);
                        process_cpu_percent =
                            Some((process.cpu_usage() / cores).clamp(0.0, 100.0));
                        let started = process.start_time();
                        if started > 0 {
                            uptime_seconds = Some(
                                chrono::Utc::now().timestamp().saturating_sub(started as i64)
                                    as u64,
                            );
                        }
                    }
                } else {
                    detail = format!("O processo Java (PID {pid}) não foi encontrado no sistema.");
                }
            }
        } else {
            detail = "Servidor offline — sem métricas de processo.".into();
        }

        let mut heap_used_mb = None;
        let mut heap_committed_mb = None;
        let mut heap_source = None;
        if active {
            if let Some(pid) = pid {
                let java = runtime
                    .as_ref()
                    .and_then(|runtime| {
                        runtime
                            .java
                            .lock()
                            .unwrap_or_else(|error| error.into_inner())
                            .clone()
                    });
                if let Some(java) = java {
                    let cached = self
                        .telemetry_cache
                        .lock()
                        .unwrap_or_else(|error| error.into_inner())
                        .get(id)
                        .and_then(|cache| {
                            cache
                                .heap
                                .as_ref()
                                .filter(|(at, _)| at.elapsed() < HEAP_CACHE)
                                .map(|(_, reading)| reading.clone())
                        });
                    let reading = match cached {
                        Some(reading) => Some(Ok(reading)),
                        None => {
                            let outcome = read_heap(&java, pid).await;
                            let mut cache = self
                                .telemetry_cache
                                .lock()
                                .unwrap_or_else(|error| error.into_inner());
                            let entry = cache.entry(id.to_string()).or_default();
                            match &outcome {
                                Ok(reading) => {
                                    entry.heap =
                                        Some((std::time::Instant::now(), reading.clone()));
                                }
                                Err(_) => entry.heap = None,
                            }
                            Some(outcome)
                        }
                    };
                    match reading {
                        Some(Ok(reading)) => {
                            heap_used_mb = Some(reading.used_mb);
                            heap_committed_mb = Some(reading.committed_mb);
                            heap_source = Some("jcmd".into());
                        }
                        Some(Err(reason)) => {
                            if detail.is_empty() {
                                detail = format!("Heap JVM indisponível: {reason}.");
                            }
                        }
                        None => {}
                    }
                } else if detail.is_empty() {
                    detail = "O caminho do Java deste servidor ainda não foi registrado.".into();
                }
            }
        }

        let system = crate::commands::system::system_resource_sample().await;
        let available = system
            .total_memory_mb
            .saturating_sub(system.used_memory_mb);
        if detail.is_empty() {
            detail = format!(
                "Medições reais do processo Java (PID {}) via jcmd local; heap máximo = RAM configurada.",
                pid.map_or_else(|| "—".into(), |pid| pid.to_string())
            );
        }

        Ok(ServerTelemetry {
            server_id: config.id,
            phase,
            online: active,
            pid: if active { pid } else { None },
            uptime_seconds: if active { uptime_seconds } else { None },
            port: state.as_ref().and_then(|state| state.port),
            configured_ram_mb: config.ram_mb,
            players_online: state.as_ref().map_or(0, |state| state.players_online),
            max_players: state.as_ref().map_or(config.max_players, |state| state.max_players),
            process_memory_mb,
            process_cpu_percent,
            heap_used_mb,
            heap_committed_mb,
            heap_max_mb: Some(config.ram_mb as u64),
            heap_source,
            tps: None,
            mspt: None,
            system_cpu_percent: system.cpu_percent,
            system_used_memory_mb: system.used_memory_mb,
            system_total_memory_mb: system.total_memory_mb,
            system_available_memory_mb: available,
            detail,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn heap_output_is_parsed_as_used_and_committed_without_mixing_metaspace() {
        let g1 = "garbage-first heap   total 524288K, used 262144K [0x0000000080000000, 0x0000000100000000)\n  region size 1024K, 256 young (262144K), 9 survivors (9216K)\n  Metaspace       used 131072K, committed 133120K, reserved 1114112K\n  class space    used 16384K, committed 17408K, reserved 1048576K\n";
        let reading = parse_heap_output(g1).expect("G1 must parse");
        assert_eq!(reading.committed_mb, 512);
        assert_eq!(reading.used_mb, 256);

        let parallel = "PS Young Generation\n total 65536K, used 12288K [0x00000000c0000000, 0x00000000c0400000)\nPS Old Generation\n total 131072K, used 65536K [0x0000000080000000, 0x0000000088000000)\nMetaspace       used 100000K, committed 101000K, reserved 1114112K\n";
        let reading = parse_heap_output(parallel).expect("Parallel must parse");
        assert_eq!(reading.committed_mb, 192);
        assert_eq!(reading.used_mb, 76);

        assert!(parse_heap_output("no heap here").is_none());
    }

    #[test]
    fn java25_reserved_committed_used_output_is_parsed() {
        let text = "24280:\ngarbage-first heap   total reserved 2097152K, committed 315392K, used 139284K [0x0000000080000000, 0x0000000100000000)\n region size 1024K, 9 young (9216K), 6 survivors (6144K)\n";
        let reading = parse_heap_output(text).expect("Java 25 G1 output must parse");
        assert_eq!(reading.committed_mb, 308);
        assert_eq!(reading.used_mb, 136);
    }
}
