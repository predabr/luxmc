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

pub fn detect_gpu() -> GpuInfo {
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

    GpuInfo {
        vendor,
        renderer,
        driver,
        supports_zink,
    }
}

pub fn generate_aikar_flags(ram_mb: u64) -> Vec<String> {
    let mut flags = Vec::new();

    // Matching -Xms and -Xmx eliminates GC resize stutters completely
    flags.push(format!("-Xms{}M", ram_mb));
    flags.push(format!("-Xmx{}M", ram_mb));

    // Core G1GC flags established by Aikar
    flags.push("-XX:+UseG1GC".into());
    flags.push("-XX:+ParallelRefProcEnabled".into());
    flags.push("-XX:MaxGCPauseMillis=200".into());
    flags.push("-XX:+UnlockExperimentalVMOptions".into());
    flags.push("-XX:+DisableExplicitGC".into());
    flags.push("-XX:+AlwaysPreTouch".into());

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
    flags.push("-XX:MaxTenuringThreshold=1".into());

    flags
}

pub fn generate_standard_flags(ram_mb: u64) -> Vec<String> {
    vec![
        "-Xms1024M".to_string(),
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
            .get_mod_versions(&mod_entry.slug, mc_version)
            .await
            .unwrap_or_default();

        if let Some(ver) = versions.first() {
            if let Some(file) = ver.files.first() {
                let dest = mods_dir.join(&file.filename);
                if !dest.exists() {
                    let resp = http.get(&file.url).send().await?.error_for_status()?;
                    let bytes = resp.bytes().await?;
                    tokio::fs::write(&dest, &bytes).await?;
                    installed.push(file.filename.clone());
                } else {
                    installed.push(format!("{} (já instalado)", file.filename));
                }
            }
        }
    }

    Ok(installed)
}
