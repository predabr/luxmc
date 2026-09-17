use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::io::Read;
use std::path::{Path, PathBuf};
use sha2::{Digest, Sha256};
use crate::db::models::ProfileRow;
use crate::error::{AppError, AppResult};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ShieldThreat {
    pub file_name: String,
    pub file_path: String,
    pub severity: String,
    pub threat_type: String,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ShieldScanResult {
    pub total_scanned: usize,
    pub is_clean: bool,
    pub threats: Vec<ShieldThreat>,
    pub scan_time_ms: u64,
}

fn get_known_malicious_hashes() -> HashSet<&'static str> {
    let mut s = HashSet::new();
    s.insert("3b006a7be7e1fe1ad50c82fb5a363cbda93c5d6447c22df5efca6a3eb77eb372");
    s.insert("d61689253457dc5b497040d34208a0a8bbbe977f6b907dbec598f82876615b31");
    s.insert("902e8697da4fd20286b20c9c7f12e2f3d6997a483e58316c80dfbe7754d9b626");
    s.insert("f8319f6f6d54fb4a77e5e3c734da2eef00c732483d09a062831c26ae7970d47d");
    s.insert("d4d420792ea7106093370f1a93e36127b46ff6a642ebda3efcbf59faea5370d0");
    s.insert("b8c009b0b46ebad084360e224e758da4f4c207902d3345d2b78ad06db7cb2245");
    s.insert("58fa1ef3d6067b5eb02685fb93c200938ff5d564bb3130bf7135e69e710b14c3");
    s.insert("4b1c7dc4901f65cfa8ff7b71933cb66b6c68b69324b172a5a51d95dc5ff3ad6a");
    s
}

pub fn scan_mods_directory(mods_dir: &Path) -> ShieldScanResult {
    let start = std::time::Instant::now();
    let known_hashes = get_known_malicious_hashes();
    let mut total_scanned = 0;
    let mut threats = Vec::new();

    if mods_dir.is_dir() {
        if let Ok(entries) = std::fs::read_dir(mods_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.extension().and_then(|e| e.to_str()) == Some("jar") {
                    total_scanned += 1;
                    let file_name = path
                        .file_name()
                        .map(|n| n.to_string_lossy().to_string())
                        .unwrap_or_else(|| "unknown.jar".to_string());

                    if let Ok(bytes) = std::fs::read(&path) {
                        let mut hasher = Sha256::new();
                        hasher.update(&bytes);
                        let hash_hex = format!("{:x}", hasher.finalize());

                        if known_hashes.contains(hash_hex.as_str()) {
                            threats.push(ShieldThreat {
                                file_name: file_name.clone(),
                                file_path: path.to_string_lossy().to_string(),
                                severity: "critical".to_string(),
                                threat_type: "Malware Conhecido (Hash Match)".to_string(),
                                description: format!(
                                    "O arquivo possui hash correspondente a payload malicioso conhecido ({})",
                                    &hash_hex[0..12]
                                ),
                            });
                            continue;
                        }

                        let reader = std::io::Cursor::new(&bytes);
                        if let Ok(mut zip) = zip::ZipArchive::new(reader) {
                            let mut has_l10 = false;
                            let mut has_suspicious_webhook = false;

                            for i in 0..zip.len() {
                                if let Ok(mut file) = zip.by_index(i) {
                                    let name = file.name().to_string();
                                    if name.ends_with("L10.class") || name.ends_with("updater.jar") {
                                        has_l10 = true;
                                    }

                                    if name.ends_with(".class") && file.size() < 100_000 {
                                        let mut buf = Vec::new();
                                        if file.read_to_end(&mut buf).is_ok() {
                                            if buf.windows(25).any(|w| {
                                                w == b"discord.com/api/webhooks"
                                                    || w == b"discordapp.com/api/webhooks"
                                            }) {
                                                has_suspicious_webhook = true;
                                            }
                                        }
                                    }
                                }
                            }

                            if has_l10 {
                                threats.push(ShieldThreat {
                                    file_name: file_name.clone(),
                                    file_path: path.to_string_lossy().to_string(),
                                    severity: "critical".to_string(),
                                    threat_type: "Fracturiser / Injetor de Malware".to_string(),
                                    description: "Detectado payload com estrutura interna similar ao exploit Fracturiser (L10/updater)".to_string(),
                                });
                            } else if has_suspicious_webhook {
                                threats.push(ShieldThreat {
                                    file_name: file_name.clone(),
                                    file_path: path.to_string_lossy().to_string(),
                                    severity: "warning".to_string(),
                                    threat_type: "Webhook Discord Embutido".to_string(),
                                    description: "Mod contém endpoints de webhook do Discord embutidos no bytecode. Possível stealer de credenciais.".to_string(),
                                });
                            }
                        }
                    }
                }
            }
        }
    }

    let is_clean = threats.is_empty();
    ShieldScanResult {
        total_scanned,
        is_clean,
        threats,
        scan_time_ms: start.elapsed().as_millis() as u64,
    }
}

#[tauri::command]
#[allow(non_snake_case)]
pub async fn instance_shield_scan(
    profileId: String,
) -> AppResult<ShieldScanResult> {
    let db = crate::db::shared_db().await?;
    let row = sqlx::query_as::<_, ProfileRow>("SELECT * FROM profiles WHERE id = ?")
        .bind(&profileId)
        .fetch_optional(db.pool())
        .await?
        .ok_or_else(|| AppError::NotFound(format!("profile {profileId} not found")))?;

    let mods_dir = PathBuf::from(&row.game_dir).join("mods");
    Ok(scan_mods_directory(&mods_dir))
}
