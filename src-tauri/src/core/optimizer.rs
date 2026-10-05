use serde::{Deserialize, Serialize};
use std::path::Path;

use crate::core::mods::ModrinthClient;
use crate::error::{AppError, AppResult};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GpuInfo {
    pub vendor: String,
    pub renderer: String,
    pub driver: String,
    pub supports_zink: bool,
}

static GPU_INFO: std::sync::OnceLock<GpuInfo> = std::sync::OnceLock::new();

pub fn detect_gpu() -> GpuInfo {
    GPU_INFO.get_or_init(detect_gpu_uncached).clone()
}

#[allow(unused_mut)]
fn detect_gpu_uncached() -> GpuInfo {
    let mut vendor = "Desconhecido".to_string();
    let mut renderer = "Driver Padrão".to_string();
    let mut driver = "Desconhecido".to_string();
    let mut supports_zink = false;

    #[cfg(target_os = "linux")]
    {
        // Try reading DRM device driver directly
        if let Ok(entries) = std::fs::read_dir("/sys/class/drm") {
            for entry in entries.flatten() {
                let name = entry.file_name().to_string_lossy().to_string();
                if (name.starts_with("card0") || name.starts_with("card1")) && !name.contains('-') {
                    let uevent_path = entry.path().join("device/uevent");
                    if let Ok(content) = std::fs::read_to_string(&uevent_path) {
                        for line in content.lines() {
                            if let Some(drv) = line.strip_prefix("DRIVER=") {
                                driver = drv.trim().to_string();
                                match driver.as_str() {
                                    "amdgpu" | "radeon" => {
                                        vendor = "AMD".to_string();
                                        renderer = "AMD Radeon (Mesa RADV)".to_string();
                                        supports_zink = true;
                                    }
                                    "i915" | "xe" => {
                                        vendor = "Intel".to_string();
                                        renderer = "Intel Graphics (Mesa ANV)".to_string();
                                        supports_zink = true;
                                    }
                                    "nvidia" => {
                                        vendor = "NVIDIA".to_string();
                                        renderer = "NVIDIA GeForce (Proprietário)".to_string();
                                        supports_zink = false;
                                    }
                                    "nouveau" => {
                                        vendor = "NVIDIA".to_string();
                                        renderer = "NVIDIA (Mesa NVK/Nouveau)".to_string();
                                        supports_zink = true;
                                    }
                                    _ => {}
                                }
                                break;
                            }
                        }
                    }
                    if vendor != "Desconhecido" {
                        break;
                    }
                }
            }
        }

        // Try reading lspci output for pretty GPU name
        if let Ok(output) = std::process::Command::new("lspci").output() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            for line in stdout.lines() {
                if line.contains("VGA compatible controller") || line.contains("3D controller") {
                    if let Some(idx) = line.find(": ") {
                        let desc = line[idx + 2..].trim();
                        renderer = desc.to_string();
                        if desc.contains("AMD") || desc.contains("Radeon") {
                            vendor = "AMD".to_string();
                            supports_zink = true;
                        } else if desc.contains("Intel") {
                            vendor = "Intel".to_string();
                            supports_zink = true;
                        } else if desc.contains("NVIDIA") {
                            vendor = "NVIDIA".to_string();
                        }
                        break;
                    }
                }
            }
        }
    }

    #[cfg(target_os = "windows")]
    {
        let command = "Get-CimInstance Win32_VideoController | Select-Object -First 1 Name,DriverVersion | ConvertTo-Json -Compress";
        if let Ok(output) = crate::core::process::std_command("powershell.exe")
            .args(["-NoProfile", "-NonInteractive", "-Command", command])
            .output()
        {
            if output.status.success() {
                if let Ok(value) = serde_json::from_slice::<serde_json::Value>(&output.stdout) {
                    renderer = value.get("Name").and_then(serde_json::Value::as_str).unwrap_or("Driver Padrão").to_string();
                    driver = value.get("DriverVersion").and_then(serde_json::Value::as_str).unwrap_or("Desconhecido").to_string();
                    let lower = renderer.to_ascii_lowercase();
                    if lower.contains("nvidia") {
                        vendor = "NVIDIA".to_string();
                    } else if lower.contains("amd") || lower.contains("radeon") {
                        vendor = "AMD".to_string();
                        supports_zink = true;
                    } else if lower.contains("intel") {
                        vendor = "Intel".to_string();
                        supports_zink = true;
                    }
                }
            }
        }
    }

    GpuInfo {
        vendor,
        renderer,
        driver,
        supports_zink,
    }
}

pub fn get_total_memory_mb() -> i64 {
    #[cfg(target_os = "linux")]
    {
        if let Ok(meminfo) = std::fs::read_to_string("/proc/meminfo") {
            for line in meminfo.lines() {
                if let Some(rest) = line.strip_prefix("MemTotal:") {
                    let kb_str = rest.trim().split_whitespace().next().unwrap_or("");
                    if let Ok(kb) = kb_str.parse::<i64>() {
                        return (kb / 1024).max(1024);
                    }
                }
            }
        }
    }
    let mut sys = sysinfo::System::new();
    sys.refresh_memory();
    (sys.total_memory() / 1024 / 1024).max(1024) as i64
}

pub fn generate_aikar_flags(ram_mb: u64) -> Vec<String> {
    let mut flags = Vec::new();
    let initial_ram = std::cmp::min(1024, ram_mb / 2).max(512);
    flags.push(format!("-Xms{}M", initial_ram));
    flags.push(format!("-Xmx{}M", ram_mb));

    flags.push("-XX:+UseG1GC".into());
    flags.push("-XX:+ParallelRefProcEnabled".into());
    flags.push("-XX:MaxGCPauseMillis=120".into());
    flags.push("-XX:+UnlockExperimentalVMOptions".into());
    flags.push("-XX:+DisableExplicitGC".into());

    // Dynamic region size and new generation sizing based on allocated memory
    if ram_mb <= 4096 {
        flags.push("-XX:G1NewSizePercent=30".into());
        flags.push("-XX:G1MaxNewSizePercent=40".into());
        flags.push("-XX:G1HeapRegionSize=8M".into());
        flags.push("-XX:G1ReservePercent=15".into());
        flags.push("-XX:InitiatingHeapOccupancyPercent=20".into());
    } else if ram_mb <= 8192 {
        flags.push("-XX:G1NewSizePercent=30".into());
        flags.push("-XX:G1MaxNewSizePercent=40".into());
        flags.push("-XX:G1HeapRegionSize=16M".into());
        flags.push("-XX:G1ReservePercent=20".into());
        flags.push("-XX:InitiatingHeapOccupancyPercent=15".into());
    } else {
        flags.push("-XX:G1NewSizePercent=40".into());
        flags.push("-XX:G1MaxNewSizePercent=50".into());
        flags.push("-XX:G1HeapRegionSize=32M".into());
        flags.push("-XX:G1ReservePercent=20".into());
        flags.push("-XX:InitiatingHeapOccupancyPercent=15".into());
    }

    flags.push("-XX:G1HeapWastePercent=5".into());
    flags.push("-XX:G1MixedGCCountTarget=4".into());
    flags.push("-XX:G1MixedGCLiveThresholdPercent=90".into());
    flags.push("-XX:G1RSetUpdatingPauseTimePercent=5".into());
    flags.push("-XX:SurvivorRatio=32".into());
    flags.push("-XX:+PerfDisableSharedMem".into());
    flags.push("-XX:+AlwaysPreTouch".into());
    flags.push("-XX:+OptimizeStringConcat".into());
    flags.push("-XX:+UseStringDeduplication".into());

    flags
}

pub fn generate_optimized_flags(ram_mb: u64, java_major: u32) -> Vec<String> {
    if cfg!(target_os = "windows") {
        return generate_client_flags(ram_mb);
    }
    if java_major >= 21 {
        let mut flags = Vec::new();
        let initial_ram = if ram_mb >= 8192 {
            (ram_mb * 3 / 4).max(4096)
        } else if ram_mb >= 4096 {
            (ram_mb / 2).max(2048)
        } else {
            std::cmp::min(1024, ram_mb / 2).max(512)
        };
        flags.push(format!("-Xms{}M", initial_ram));
        flags.push(format!("-Xmx{}M", ram_mb));
        flags.push("-XX:+UnlockExperimentalVMOptions".into());
        flags.push("-XX:+UseZGC".into());
        if java_major < 23 { flags.push("-XX:+ZGenerational".into()); }
        flags.push("-XX:+AlwaysPreTouch".into());
        flags.push("-XX:+OptimizeStringConcat".into());
        flags.push("-XX:+UseStringDeduplication".into());
        flags
    } else {
        generate_aikar_flags(ram_mb)
    }
}

pub fn generate_client_flags(ram_mb: u64) -> Vec<String> {
    let ram_mb = ram_mb.max(256);
    vec![
        format!("-Xms{}M", (ram_mb / 4).clamp(256, 1024)),
        format!("-Xmx{}M", ram_mb),
        "-XX:+UseG1GC".into(),
        "-XX:+ParallelRefProcEnabled".into(),
        "-XX:MaxGCPauseMillis=100".into(),
        "-XX:+UseStringDeduplication".into(),
    ]
}

#[cfg(test)]
mod windows_client_tests {
    use super::*;
    #[test]
    fn windows_client_heap_preserves_limits_without_startup_pretouch() {
        for ram in [256, 512, 1024, 4096, 8192, 16384] {
            let flags = generate_client_flags(ram);
            assert!(flags.contains(&format!("-Xmx{}M", ram)));
            assert!(flags.contains(&format!("-Xms{}M", (ram / 4).clamp(256, 1024))));
            assert!(!flags.iter().any(|flag| flag.contains("AlwaysPreTouch") || flag.contains("UseZGC")));
            assert_eq!(flags.iter().filter(|flag| flag.contains("UseG1GC")).count(), 1);
        }
    }

    #[test]
    fn standard_heap_stays_within_the_configured_maximum() {
        for ram in [0, 128, 256, 512, 1024, 4096, 16384] {
            let effective_maximum = ram.max(256);
            let flags = generate_standard_flags(ram);
            assert_eq!(flags, vec![
                format!("-Xms{}M", (effective_maximum / 4).clamp(256, 1024)),
                format!("-Xmx{}M", effective_maximum),
            ]);
            let initial = flags[0].trim_start_matches("-Xms").trim_end_matches('M').parse::<u64>().unwrap();
            assert!(initial <= effective_maximum);
        }
    }
}

pub fn generate_standard_flags(ram_mb: u64) -> Vec<String> {
    let ram_mb = ram_mb.max(256);
    vec![
        format!("-Xms{}M", (ram_mb / 4).clamp(256, 1024)),
        format!("-Xmx{}M", ram_mb),
    ]
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PerformanceModEntry {
    pub slug: String,
    pub title: String,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PerformancePackInfo {
    pub available: bool,
    pub loader: String,
    pub mc_version: String,
    pub reason: Option<String>,
    pub mods: Vec<PerformanceModEntry>,
}

pub fn get_performance_pack_info(loader: &str, mc_version: &str) -> PerformancePackInfo {
    let clean_loader = loader.trim().to_lowercase();
    match clean_loader.as_str() {
        "fabric" | "quilt" => PerformancePackInfo {
            available: true,
            loader: clean_loader,
            mc_version: mc_version.to_string(),
            reason: None,
            mods: vec![
                PerformanceModEntry {
                    slug: "sodium".into(),
                    title: "Sodium".into(),
                    description: "Motor de renderização moderno que substitui o pipeline gráfico do Minecraft, aumentando drasticamente o FPS (3x a 5x).".into(),
                },
                PerformanceModEntry {
                    slug: "lithium".into(),
                    title: "Lithium".into(),
                    description: "Otimização de física, IA de entidades e chunks sem alterar a mecânica vanilla.".into(),
                },
                PerformanceModEntry {
                    slug: "ferrite-core".into(),
                    title: "FerriteCore".into(),
                    description: "Reduz o consumo de memória RAM do Minecraft em 30% a 50% através da otimização de estados de blocos e modelos.".into(),
                },
                PerformanceModEntry {
                    slug: "modernfix".into(),
                    title: "ModernFix".into(),
                    description: "Reduz drasticamente o tempo de carregamento e corrige vazamentos de memória.".into(),
                },
                PerformanceModEntry {
                    slug: "entityculling".into(),
                    title: "Entity Culling".into(),
                    description: "Ignora a renderização de blocos e entidades fora do campo de visão, aumentando a fluidez gráfica.".into(),
                },
                PerformanceModEntry {
                    slug: "immediatelyfast".into(),
                    title: "ImmediatelyFast".into(),
                    description: "Otimiza a renderização imediata do HUD, texto, partículas e telas de menu.".into(),
                },
            ],
        },
        "neoforge" => {
            PerformancePackInfo {
                available: true,
                loader: clean_loader,
                mc_version: mc_version.to_string(),
                reason: None,
                mods: vec![
                    PerformanceModEntry {
                        slug: "sodium".into(),
                        title: "Sodium (NeoForge)".into(),
                        description: "Renderização gráfica ultrarrápida com suporte nativo a NeoForge.".into(),
                    },
                    PerformanceModEntry {
                        slug: "lithium".into(),
                        title: "Lithium (NeoForge)".into(),
                        description: "Otimização do motor de física e processamento de entidades.".into(),
                    },
                    PerformanceModEntry {
                        slug: "ferrite-core".into(),
                        title: "FerriteCore".into(),
                        description: "Redução drástica do consumo de RAM em instâncias NeoForge.".into(),
                    },
                    PerformanceModEntry {
                        slug: "modernfix".into(),
                        title: "ModernFix".into(),
                        description: "Reduz o consumo de RAM e acelera a inicialização no NeoForge.".into(),
                    },
                    PerformanceModEntry {
                        slug: "entityculling".into(),
                        title: "Entity Culling".into(),
                        description: "Otimiza a renderização de entidades e blocos.".into(),
                    },
                    PerformanceModEntry {
                        slug: "immediatelyfast".into(),
                        title: "ImmediatelyFast".into(),
                        description: "Acelera a renderização de interface e texto.".into(),
                    },
                ],
            }
        }
        "forge" => {
            PerformancePackInfo {
                available: true,
                loader: clean_loader,
                mc_version: mc_version.to_string(),
                reason: None,
                mods: vec![
                    PerformanceModEntry {
                        slug: "embeddium".into(),
                        title: "Embeddium".into(),
                        description: "Port estável do Sodium para o ecossistema Forge com alta compatibilidade.".into(),
                    },
                    PerformanceModEntry {
                        slug: "ferrite-core".into(),
                        title: "FerriteCore".into(),
                        description: "Otimização drástica do uso de memória RAM para modpacks Forge.".into(),
                    },
                    PerformanceModEntry {
                        slug: "modernfix".into(),
                        title: "ModernFix".into(),
                        description: "Reduz o tempo de carregamento do Forge e corrige vazamentos de memória.".into(),
                    },
                    PerformanceModEntry {
                        slug: "entityculling".into(),
                        title: "Entity Culling".into(),
                        description: "Ignora a renderização de blocos e entidades fora do campo de visão.".into(),
                    },
                    PerformanceModEntry {
                        slug: "immediatelyfast".into(),
                        title: "ImmediatelyFast".into(),
                        description: "Otimiza a renderização de HUD e menus no Forge.".into(),
                    },
                ],
            }
        }
        _ => PerformancePackInfo {
            available: false,
            loader: clean_loader,
            mc_version: mc_version.to_string(),
            reason: Some("O pacote de otimização requer um modloader (Fabric, NeoForge ou Forge). Crie uma instância com Fabric para utilizar.".into()),
            mods: vec![],
        },
    }
}

pub async fn install_performance_pack(
    http: &reqwest::Client,
    game_dir: &Path,
    loader: &str,
    mc_version: &str,
) -> AppResult<Vec<String>> {
    let pack = get_performance_pack_info(loader, mc_version);
    if !pack.available {
        return Err(AppError::InvalidState(
            pack.reason.unwrap_or_else(|| "Pacote indisponível".into()),
        ));
    }

    let mods_dir = game_dir.join("mods");
    tokio::fs::create_dir_all(&mods_dir).await?;

    let modrinth = ModrinthClient::new(http.clone());
    let mut installed = Vec::new();

    // If Fabric, make sure fabric-api is also installed if missing
    if loader.to_lowercase() == "fabric" {
        let _ = crate::core::loaders::fabric::ensure_fabric_api(http, &mods_dir, mc_version).await;
    }

    for mod_entry in &pack.mods {
        let versions = modrinth
            .get_mod_versions_filtered(&mod_entry.slug, mc_version, Some(loader))
            .await
            .unwrap_or_default();

        if let Some(ver) = versions.first() {
            if let Some(file) = ver.files.first() {
                let dest = mods_dir.join(&file.filename);
                let valid_existing = std::fs::File::open(&dest).ok()
                    .and_then(|existing| zip::ZipArchive::new(existing).ok())
                    .is_some();
                if valid_existing {
                    installed.push(format!("{} (já instalado)", file.filename));
                } else {
                    crate::core::downloader::ensure_artifact(http, &dest, &file.url, file.size, &file.sha1).await?;
                    installed.push(file.filename.clone());
                }
            }
        }
    }

    if installed.is_empty() {
        return Err(AppError::NotFound(format!("Nenhum mod de desempenho compatível com {loader} {mc_version}")));
    }

    Ok(installed)
}

static PERF_PACK_CACHE: std::sync::LazyLock<
    std::sync::Mutex<std::collections::HashMap<String, (std::time::Instant, PerformancePackInfo)>>,
> = std::sync::LazyLock::new(|| std::sync::Mutex::new(std::collections::HashMap::new()));

const PERF_PACK_CACHE_TTL: std::time::Duration = std::time::Duration::from_secs(900);

pub async fn get_performance_pack_info_for(
    http: &reqwest::Client,
    loader: &str,
    mc_version: &str,
) -> PerformancePackInfo {
    let base = get_performance_pack_info(loader, mc_version);
    if !base.available {
        return base;
    }

    let key = format!("{}:{}", base.loader, mc_version.trim());
    if let Ok(cache) = PERF_PACK_CACHE.lock() {
        if let Some((cached_at, info)) = cache.get(&key) {
            if cached_at.elapsed() < PERF_PACK_CACHE_TTL {
                return info.clone();
            }
        }
    }

    let checks = futures_util::future::join_all(base.mods.iter().map(|entry| {
        let slug = entry.slug.clone();
        let mc = base.mc_version.clone();
        let loader = base.loader.clone();
        let client = ModrinthClient::new(http.clone());
        async move {
            client
                .get_mod_versions_filtered(&slug, &mc, Some(&loader))
                .await
                .map(|versions| !versions.is_empty())
        }
    }))
    .await;

    let mut network_error = false;
    let mut compatible = Vec::new();
    for (entry, result) in base.mods.iter().zip(checks) {
        match result {
            Ok(true) => compatible.push(entry.clone()),
            Ok(false) => {}
            Err(_) => network_error = true,
        }
    }

    let info = if network_error {
        base
    } else if compatible.is_empty() {
        PerformancePackInfo {
            available: false,
            loader: base.loader.clone(),
            mc_version: base.mc_version.clone(),
            reason: Some(format!(
                "Nenhum mod compatível com {} {}",
                base.mc_version, base.loader
            )),
            mods: Vec::new(),
        }
    } else if compatible.len() == base.mods.len() {
        base
    } else {
        PerformancePackInfo {
            mods: compatible,
            ..base
        }
    };

    if let Ok(mut cache) = PERF_PACK_CACHE.lock() {
        if cache.len() > 64 {
            cache.clear();
        }
        cache.insert(key, (std::time::Instant::now(), info.clone()));
    }

    info
}
