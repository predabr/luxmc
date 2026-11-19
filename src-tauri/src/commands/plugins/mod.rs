use crate::error::{AppError, AppResult};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;
use std::io::{Cursor, Read};
use std::path::{Path, PathBuf};
use crate::state::AppState;
use tauri::State;

const MAX_ARCHIVE_BYTES: usize = 16 * 1024 * 1024;
const MAX_FILE_BYTES: usize = 3 * 1024 * 1024;
const MAX_ENTRIES: usize = 400;
const MAX_SCRIPT_BYTES: u64 = 2 * 1024 * 1024;
const KNOWN_MALWARE_SHA256: &[&str] = &["1d1aaccdc13244e980c0c024610ecc77ea2674a33a52129edf1bb4ce3b2cc2fc"];

const ALLOWED_EXTENSIONS: &[&str] = &["js", "mjs", "json", "css", "md", "txt", "svg", "html"];

const PERMISSIONS: &[&str] = &["network", "storage", "profiles"];

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct PluginManifest {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub version: String,
    #[serde(default)]
    pub author: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub homepage: Option<String>,
    #[serde(default = "default_main")]
    pub main: String,
    #[serde(default)]
    pub permissions: Vec<String>,
}

fn default_main() -> String {
    "main.js".into()
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct PluginFinding {
    pub severity: String,
    pub code: String,
    pub message: String,
    pub file: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct PluginScan {
    pub clean: bool,
    pub scanned_at: String,
    pub findings: Vec<PluginFinding>,
    pub files: usize,
    pub bytes: u64,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct PluginRecord {
    pub manifest: PluginManifest,
    pub enabled: bool,
    pub installed_at: String,
    pub sha256: String,
    pub size_bytes: u64,
    pub scan: PluginScan,
    #[serde(default)]
    pub cover: Option<String>,
    #[serde(default)]
    pub community: bool,
}

#[derive(Serialize, Deserialize, Default)]
struct PluginState {
    records: BTreeMap<String, PluginRecord>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct RegistryPlugin {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub version: String,
    #[serde(default)]
    pub author: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub homepage: Option<String>,
    #[serde(default)]
    pub download_url: Option<String>,
    #[serde(default)]
    pub sha256: Option<String>,
    #[serde(default)]
    pub icon_url: Option<String>,
    #[serde(default)]
    pub tags: Vec<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct PluginRegistry {
    #[serde(default)]
    pub plugins: Vec<RegistryPlugin>,
    #[serde(default)]
    pub source: String,
    #[serde(default)]
    pub fetched_at: String,
}

pub fn plugins_root() -> AppResult<PathBuf> {
    let dirs = directories::ProjectDirs::from("io", "github", "Luxmc")
        .ok_or_else(|| AppError::Internal("Diretorio de dados do Luxmc indisponivel.".into()))?;
    Ok(dirs.data_dir().join("plugins"))
}

fn state_path(root: &Path) -> PathBuf {
    root.join("state.json")
}

fn now_stamp() -> String {
    let now = std::time::SystemTime::now();
    let seconds = now.duration_since(std::time::UNIX_EPOCH).map(|value| value.as_secs()).unwrap_or_default();
    format!("{seconds}")
}

fn read_state(root: &Path) -> PluginState {
    std::fs::read_to_string(state_path(root))
        .ok()
        .and_then(|raw| serde_json::from_str(&raw).ok())
        .unwrap_or_default()
}

fn write_state(root: &Path, state: &PluginState) -> AppResult<()> {
    std::fs::create_dir_all(root).map_err(|error| AppError::Internal(error.to_string()))?;
    let raw = serde_json::to_string_pretty(state).map_err(|error| AppError::Internal(error.to_string()))?;
    let temp = root.join("state.json.tmp");
    std::fs::write(&temp, raw).map_err(|error| AppError::Internal(error.to_string()))?;
    std::fs::rename(&temp, state_path(root)).map_err(|error| AppError::Internal(error.to_string()))
}

fn valid_id(id: &str) -> bool {
    !id.is_empty()
        && id.len() <= 80
        && id.chars().all(|character| character.is_ascii_alphanumeric() || matches!(character, '.' | '-' | '_'))
        && !id.starts_with('.')
        && id != "state"
}

fn sanitize_entry(name: &str) -> Option<String> {
    if name.is_empty() || name.len() > 240 { return None; }
    let normalized = name.replace('\\', "/");
    if normalized.starts_with('/') { return None; }
    let mut parts = Vec::new();
    for part in normalized.split('/') {
        if part.is_empty() || part == "." { continue; }
        if part == ".." { return None; }
        if part.contains(':') { return None; }
        parts.push(part);
    }
    if parts.is_empty() { return None; }
    Some(parts.join("/"))
}

fn read_entries(bytes: &[u8]) -> AppResult<BTreeMap<String, Vec<u8>>> {
    let mut archive = zip::ZipArchive::new(Cursor::new(bytes)).map_err(|error| AppError::InvalidState(format!("Pacote de plugin invalido: {error}")))?;
    if archive.len() > MAX_ENTRIES {
        return Err(AppError::InvalidState(format!("O pacote tem arquivos demais ({}) para um plugin.", archive.len())));
    }
    let mut files = BTreeMap::new();
    let mut total = 0usize;
    for index in 0..archive.len() {
        let mut entry = archive.by_index(index).map_err(|error| AppError::InvalidState(error.to_string()))?;
        if entry.is_dir() { continue; }
        let Some(path) = sanitize_entry(entry.name()) else {
            return Err(AppError::InvalidState("O pacote contem caminhos invalidos e foi recusado.".into()));
        };
        let extension = path.rsplit('.').next().unwrap_or("").to_ascii_lowercase();
        if !ALLOWED_EXTENSIONS.contains(&extension.as_str()) {
            return Err(AppError::InvalidState(format!("Arquivo nao permitido no plugin: {path}")));
        }
        if entry.size() as usize > MAX_FILE_BYTES {
            return Err(AppError::InvalidState(format!("Arquivo grande demais no plugin: {path}")));
        }
        let mut buffer = Vec::with_capacity(entry.size() as usize);
        let mut limited = (&mut entry).take(MAX_FILE_BYTES as u64 + 1);
        limited.read_to_end(&mut buffer).map_err(|error| AppError::Internal(error.to_string()))?;
        if buffer.len() > MAX_FILE_BYTES {
            return Err(AppError::InvalidState(format!("Arquivo grande demais no plugin: {path}")));
        }
        total += buffer.len();
        if total > MAX_ARCHIVE_BYTES {
            return Err(AppError::InvalidState("O plugin descompactado e grande demais.".into()));
        }
        files.insert(path, buffer);
    }
    Ok(strip_wrapper(files))
}

fn strip_wrapper(files: BTreeMap<String, Vec<u8>>) -> BTreeMap<String, Vec<u8>> {
    if files.contains_key("plugin.json") { return files; }
    let prefix = match files.keys().next().and_then(|key| key.split_once('/')).map(|(prefix, _)| prefix.to_string()) {
        Some(prefix) => prefix,
        None => return files,
    };
    let marker = format!("{prefix}/");
    if !files.keys().all(|key| key.starts_with(&marker)) { return files; }
    files
        .into_iter()
        .filter_map(|(key, value)| key.strip_prefix(&marker).map(|rest| (rest.to_string(), value)))
        .collect()
}

const IOC_PATTERNS: &[(&str, &str, &str)] = &[
    ("skyrage.de", "critical", "Endereco de comando e controle do malware SkyRage."),
    ("files-8ie.pages.dev", "critical", "Host usado pelo Fractureiser para baixar cargas maliciosas."),
    ("85.217.144.130", "critical", "Endereco IP documentado do Fractureiser."),
    ("107.189.3.101", "critical", "Endereco IP documentado do Fractureiser."),
    ("95.214.27.172", "critical", "Endereco IP documentado do SkyRage."),
    ("vmd-gnu", "critical", "Artefato de persistencia do SkyRage."),
    ("microsoft-vm-core", "critical", "Agendador falso criado pelo SkyRage."),
    ("fractureiser", "warning", "O plugin menciona o malware Fractureiser; confirme a intencao."),
];

const SCRIPT_PATTERNS: &[(&str, &str, &str, &str)] = &[
    ("child_process", "critical", "exec_child_process", "O plugin tenta executar processos no sistema."),
    ("require(\"fs\")", "critical", "fs_require", "O plugin tenta acessar arquivos do sistema."),
    ("require('fs')", "critical", "fs_require", "O plugin tenta acessar arquivos do sistema."),
    ("process.env", "warning", "env_access", "O plugin le variaveis de ambiente."),
    ("eval(atob(", "critical", "obfuscated_eval", "Codigo ofuscado em base64 executado com eval."),
    ("eval(String.fromCharCode", "critical", "obfuscated_eval", "Codigo ofuscado executado com eval."),
    ("new Function(", "warning", "dynamic_function", "Codigo montado dinamicamente com new Function."),
    ("document.cookie", "warning", "cookie_access", "O plugin tenta ler cookies."),
    ("localStorage.setItem('luxmc_account'", "critical", "token_theft", "O plugin tenta gravar dados de conta."),
];

fn scan_files(files: &BTreeMap<String, Vec<u8>>) -> PluginScan {
    let mut findings = Vec::new();
    let mut bytes = 0u64;
    for (path, content) in files {
        bytes += content.len() as u64;
        let hash = format!("{:x}", Sha256::digest(content));
        if KNOWN_MALWARE_SHA256.contains(&hash.as_str()) {
            findings.push(PluginFinding {
                severity: "critical".into(),
                code: "known_malware".into(),
                message: "Arquivo com hash de malware conhecido.".into(),
                file: path.clone(),
            });
        }
        if path.ends_with(".png") || path.ends_with(".jpg") { continue; }
        let text = String::from_utf8_lossy(content);
        let lower = text.to_ascii_lowercase();
        for (needle, severity, message) in IOC_PATTERNS {
            if lower.contains(needle) {
                findings.push(PluginFinding {
                    severity: severity.to_string(),
                    code: "known_indicator".into(),
                    message: (*message).to_string(),
                    file: path.clone(),
                });
            }
        }
        for (needle, severity, code, message) in SCRIPT_PATTERNS {
            if text.contains(needle) || lower.contains(&needle.to_ascii_lowercase()) {
                findings.push(PluginFinding {
                    severity: severity.to_string(),
                    code: (*code).to_string(),
                    message: (*message).to_string(),
                    file: path.clone(),
                });
            }
        }
        if path.ends_with(".js") || path.ends_with(".mjs") {
            let longest = text
                .split(|character: char| !character.is_ascii_alphanumeric() && character != '+' && character != '/' && character != '=')
                .map(|token| token.len())
                .max()
                .unwrap_or(0);
            if longest > 4_000 {
                findings.push(PluginFinding {
                    severity: "warning".into(),
                    code: "obfuscated_payload".into(),
                    message: "Bloco codificado muito longo, tipico de codigo empacotado.".into(),
                    file: path.clone(),
                });
            }
        }
    }
    let clean = findings.iter().all(|finding| finding.severity != "critical");
    PluginScan { clean, scanned_at: now_stamp(), findings, files: files.len(), bytes }
}

fn parse_manifest(files: &BTreeMap<String, Vec<u8>>) -> AppResult<PluginManifest> {
    let raw = files
        .get("plugin.json")
        .ok_or_else(|| AppError::InvalidState("O plugin nao tem plugin.json.".into()))?;
    let manifest: PluginManifest = serde_json::from_slice(raw)
        .map_err(|error| AppError::InvalidState(format!("plugin.json invalido: {error}")))?;
    if !valid_id(&manifest.id) {
        return Err(AppError::InvalidState("Identificador de plugin invalido.".into()));
    }
    if manifest.name.trim().is_empty() {
        return Err(AppError::InvalidState("O plugin precisa de um nome.".into()));
    }
    let main = sanitize_entry(&manifest.main)
        .ok_or_else(|| AppError::InvalidState("Arquivo principal do plugin invalido.".into()))?;
    if !files.contains_key(&main) {
        return Err(AppError::InvalidState(format!("O arquivo principal {main} nao existe no pacote.")));
    }
    for permission in &manifest.permissions {
        if !PERMISSIONS.contains(&permission.as_str()) {
            return Err(AppError::InvalidState(format!("Permissao desconhecida: {permission}")));
        }
    }
    Ok(manifest)
}

fn write_plugin(root: &Path, files: &BTreeMap<String, Vec<u8>>) -> AppResult<PathBuf> {
    std::fs::create_dir_all(root).map_err(|error| AppError::Internal(error.to_string()))?;
    let staging = root.join(".staging");
    let _ = std::fs::remove_dir_all(&staging);
    for (path, content) in files {
        let target = staging.join(path);
        if let Some(parent) = target.parent() {
            std::fs::create_dir_all(parent).map_err(|error| AppError::Internal(error.to_string()))?;
        }
        std::fs::write(&target, content).map_err(|error| AppError::Internal(error.to_string()))?;
    }
    Ok(staging)
}

fn commit_plugin(root: &Path, id: &str, files: &BTreeMap<String, Vec<u8>>) -> AppResult<()> {
    let target = root.join(id);
    let staging = write_plugin(root, files)?;
    let _ = std::fs::remove_dir_all(&target);
    std::fs::rename(&staging, &target).map_err(|error| AppError::Internal(error.to_string()))
}

fn find_zip_start(bytes: &[u8]) -> Option<usize> {
    let needle = b"PK\x03\x04";
    bytes.windows(4).position(|window| window == needle)
}

pub fn install_from_bytes(bytes: &[u8], expected_sha256: Option<&str>) -> AppResult<PluginRecord> {
    let root = plugins_root()?;
    install_at(&root, bytes, expected_sha256)
}

pub fn install_at(root: &Path, bytes: &[u8], expected_sha256: Option<&str>) -> AppResult<PluginRecord> {
    if bytes.is_empty() {
        return Err(AppError::InvalidState("O download do plugin veio vazio.".into()));
    }
    if bytes.len() > MAX_ARCHIVE_BYTES {
        return Err(AppError::InvalidState("O pacote do plugin e grande demais.".into()));
    }
    let offset = find_zip_start(bytes).ok_or_else(|| AppError::InvalidState("O arquivo nao e um pacote .zip valido.".into()))?;
    let archive = &bytes[offset..];
    let digest = format!("{:x}", Sha256::digest(archive));
    if let Some(expected) = expected_sha256 {
        let expected = expected.trim().to_ascii_lowercase();
        if !expected.is_empty() && expected != digest {
            return Err(AppError::InvalidState("A assinatura SHA-256 do plugin nao confere com o catalogo.".into()));
        }
    }
    let files = read_entries(archive)?;
    register_files(root, &files, &digest)
}

pub(crate) fn register_files(root: &Path, files: &BTreeMap<String, Vec<u8>>, digest: &str) -> AppResult<PluginRecord> {
    let manifest = parse_manifest(files)?;
    let scan = scan_files(files);
    if !scan.clean {
        let first = scan.findings.iter().find(|finding| finding.severity == "critical").map(|finding| finding.message.clone()).unwrap_or_default();
        return Err(AppError::InvalidState(format!("Luxmc Shield bloqueou este plugin: {first}")));
    }
    commit_plugin(root, &manifest.id, files)?;
    let record = PluginRecord {
        manifest,
        enabled: true,
        installed_at: now_stamp(),
        sha256: digest.to_string(),
        size_bytes: scan.bytes,
        scan,
        cover: None,
        community: false,
    };
    let mut state = read_state(root);
    state.records.insert(record.manifest.id.clone(), record.clone());
    write_state(root, &state)?;
    Ok(record)
}

pub fn create_at(root: &Path, plugin_id: &str, name: &str, author: &str, description: &str, permissions: &[String], source: &str) -> AppResult<PluginRecord> {
    if !valid_id(plugin_id) {
        return Err(AppError::InvalidState("Identificador invalido. Use letras, numeros, ponto, hifen ou underline.".into()));
    }
    if name.trim().is_empty() {
        return Err(AppError::InvalidState("O plugin precisa de um nome.".into()));
    }
    if source.len() > MAX_SCRIPT_BYTES as usize {
        return Err(AppError::InvalidState("O codigo do plugin e grande demais.".into()));
    }
    let state = read_state(root);
    if state.records.contains_key(plugin_id) {
        return Err(AppError::InvalidState(format!("Ja existe um plugin chamado {plugin_id}.")));
    }
    let permissions: Vec<String> = permissions.iter().filter(|value| PERMISSIONS.contains(&value.as_str())).cloned().collect();
    let manifest = PluginManifest {
        id: plugin_id.to_string(),
        name: name.trim().to_string(),
        version: "0.1.0".into(),
        author: author.trim().to_string(),
        description: description.trim().to_string(),
        homepage: None,
        main: "main.js".into(),
        permissions,
    };
    let manifest_json = serde_json::to_vec_pretty(&manifest).map_err(|error| AppError::Internal(error.to_string()))?;
    let mut files = BTreeMap::new();
    files.insert("plugin.json".to_string(), manifest_json);
    files.insert("main.js".to_string(), source.as_bytes().to_vec());
    let digest = format!("{:x}", Sha256::digest(source.as_bytes()));
    register_files(root, &files, &digest)
}

pub fn update_source_at(root: &Path, plugin_id: &str, source: &str) -> AppResult<PluginRecord> {
    let mut state = read_state(root);
    let record = state.records.get(plugin_id).cloned().ok_or_else(|| AppError::NotFound(format!("Plugin {plugin_id} nao instalado.")))?;
    if source.len() > MAX_SCRIPT_BYTES as usize {
        return Err(AppError::InvalidState("O codigo do plugin e grande demais.".into()));
    }
    let mut files = read_plugin_files(root, plugin_id)?;
    let main = sanitize_entry(&record.manifest.main).ok_or_else(|| AppError::InvalidState("Arquivo principal invalido.".into()))?;
    files.insert(main, source.as_bytes().to_vec());
    let manifest_json = serde_json::to_vec_pretty(&record.manifest).map_err(|error| AppError::Internal(error.to_string()))?;
    files.insert("plugin.json".to_string(), manifest_json);
    let scan = scan_files(&files);
    commit_plugin(root, plugin_id, &files)?;
    let mut updated = record;
    updated.scan = scan;
    updated.sha256 = format!("{:x}", Sha256::digest(source.as_bytes()));
    updated.size_bytes = updated.scan.bytes;
    if !updated.scan.clean {
        updated.enabled = false;
        state.records.insert(plugin_id.to_string(), updated);
        write_state(root, &state)?;
        return Err(AppError::InvalidState("Luxmc Shield encontrou ameacas neste codigo; o plugin foi desativado.".into()));
    }
    updated.enabled = true;
    state.records.insert(plugin_id.to_string(), updated.clone());
    write_state(root, &state)?;
    Ok(updated)
}

pub fn read_source_at(root: &Path, plugin_id: &str) -> AppResult<String> {
    let state = read_state(root);
    let record = state.records.get(plugin_id).cloned().ok_or_else(|| AppError::NotFound(format!("Plugin {plugin_id} nao instalado.")))?;
    let files = read_plugin_files(root, plugin_id)?;
    let main = sanitize_entry(&record.manifest.main).ok_or_else(|| AppError::InvalidState("Arquivo principal invalido.".into()))?;
    let bytes = files.get(&main).ok_or_else(|| AppError::InvalidState("Arquivo principal ausente.".into()))?;
    Ok(String::from_utf8_lossy(bytes).into_owned())
}

fn read_plugin_files(root: &Path, id: &str) -> AppResult<BTreeMap<String, Vec<u8>>> {
    let dir = root.join(id);
    let mut files = BTreeMap::new();
    let mut stack = vec![dir.clone()];
    while let Some(current) = stack.pop() {
        let entries = std::fs::read_dir(&current).map_err(|error| AppError::Internal(error.to_string()))?;
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() { stack.push(path); continue; }
            let Ok(relative) = path.strip_prefix(&dir) else { continue };
            let Some(name) = sanitize_entry(&relative.to_string_lossy()) else { continue };
            if let Ok(bytes) = std::fs::read(&path) { files.insert(name, bytes); }
        }
    }
    Ok(files)
}

pub fn rescan_plugin(id: &str) -> AppResult<PluginRecord> {
    let root = plugins_root()?;
    rescan_at(&root, id)
}

pub fn rescan_at(root: &Path, id: &str) -> AppResult<PluginRecord> {
    let mut state = read_state(root);
    let record = state.records.get(id).cloned().ok_or_else(|| AppError::NotFound(format!("Plugin {id} nao instalado.")))?;
    let files = read_plugin_files(root, id)?;
    let scan = scan_files(&files);
    let mut updated = record;
    updated.scan = scan;
    if !updated.scan.clean { updated.enabled = false; }
    state.records.insert(id.to_string(), updated.clone());
    write_state(root, &state)?;
    Ok(updated)
}

pub fn list_installed() -> AppResult<Vec<PluginRecord>> {
    let root = plugins_root()?;
    list_at(&root)
}

pub fn list_at(root: &Path) -> AppResult<Vec<PluginRecord>> {
    let state = read_state(root);
    let mut records: Vec<PluginRecord> = state.records.into_values().collect();
    records.sort_by(|first, second| first.manifest.name.to_lowercase().cmp(&second.manifest.name.to_lowercase()));
    Ok(records)
}

pub fn uninstall(id: &str) -> AppResult<()> {
    let root = plugins_root()?;
    uninstall_at(&root, id)
}

pub fn uninstall_at(root: &Path, id: &str) -> AppResult<()> {
    let mut state = read_state(root);
    if state.records.remove(id).is_none() {
        return Err(AppError::NotFound(format!("Plugin {id} nao instalado.")));
    }
    write_state(root, &state)?;
    let _ = std::fs::remove_dir_all(root.join(id));
    Ok(())
}

pub fn set_enabled(id: &str, enabled: bool) -> AppResult<PluginRecord> {
    let root = plugins_root()?;
    set_enabled_at(&root, id, enabled)
}

pub fn set_enabled_at(root: &Path, id: &str, enabled: bool) -> AppResult<PluginRecord> {
    let mut state = read_state(root);
    let record = state.records.get(id).cloned().ok_or_else(|| AppError::NotFound(format!("Plugin {id} nao instalado.")))?;
    if enabled {
        let files = read_plugin_files(root, id)?;
        let scan = scan_files(&files);
        if !scan.clean {
            let mut blocked = record;
            blocked.enabled = false;
            blocked.scan = scan;
            state.records.insert(id.to_string(), blocked);
            write_state(root, &state)?;
            return Err(AppError::InvalidState("Luxmc Shield encontrou ameacas neste plugin; ele foi desativado.".into()));
        }
        let mut updated = record;
        updated.enabled = true;
        updated.scan = scan;
        state.records.insert(id.to_string(), updated.clone());
        write_state(root, &state)?;
        return Ok(updated);
    }
    let mut updated = record;
    updated.enabled = false;
    state.records.insert(id.to_string(), updated.clone());
    write_state(root, &state)?;
    Ok(updated)
}

#[allow(dead_code)]
pub fn read_plugin_script(id: &str) -> AppResult<PluginManifest> {
    Ok(plugin_bundle(id)?.0)
}

pub fn plugin_bundle(id: &str) -> AppResult<(PluginManifest, String)> {
    let root = plugins_root()?;
    bundle_at(&root, id)
}

pub fn bundle_at(root: &Path, id: &str) -> AppResult<(PluginManifest, String)> {
    let state = read_state(root);
    let record = state.records.get(id).cloned().ok_or_else(|| AppError::NotFound(format!("Plugin {id} nao instalado.")))?;
    if !record.enabled {
        return Err(AppError::InvalidState("Este plugin esta desativado.".into()));
    }
    if !record.scan.clean {
        return Err(AppError::InvalidState("Este plugin foi bloqueado pelo Luxmc Shield.".into()));
    }
    let files = read_plugin_files(root, id)?;
    let fresh = scan_files(&files);
    if !fresh.clean {
        return Err(AppError::InvalidState("Luxmc Shield bloqueou este plugin: os arquivos mudaram desde a instalacao.".into()));
    }
    let main = sanitize_entry(&record.manifest.main).ok_or_else(|| AppError::InvalidState("Arquivo principal invalido.".into()))?;
    let script = files
        .get(&main)
        .ok_or_else(|| AppError::InvalidState("O plugin nao tem mais o arquivo principal.".into()))?;
    if script.len() as u64 > MAX_SCRIPT_BYTES {
        return Err(AppError::InvalidState("O script do plugin e grande demais.".into()));
    }
    let text = String::from_utf8_lossy(script).into_owned();
    Ok((record.manifest, text))
}


#[cfg(test)]
mod tests;



const COVER_EXTENSIONS: &[&str] = &["png", "jpg", "jpeg", "gif", "webp"];
const MAX_COVER_BYTES: usize = 4 * 1024 * 1024;

fn cover_mime(path: &str) -> &'static str {
    match path.rsplit('.').next().unwrap_or("").to_ascii_lowercase().as_str() {
        "png" => "image/png",
        "jpg" | "jpeg" => "image/jpeg",
        "gif" => "image/gif",
        "webp" => "image/webp",
        _ => "application/octet-stream",
    }
}

pub fn set_cover_at(root: &Path, plugin_id: &str, source: &Path, premium: bool) -> AppResult<PluginRecord> {
    if !premium {
        return Err(AppError::InvalidState("Capa personalizada e um recurso do Luxmc Plus e Pro.".into()));
    }
    let extension = source.extension().and_then(|value| value.to_str()).unwrap_or("").to_ascii_lowercase();
    if !COVER_EXTENSIONS.contains(&extension.as_str()) {
        return Err(AppError::InvalidState("Use uma imagem PNG, JPG, WEBP ou um GIF.".into()));
    }
    let metadata = std::fs::metadata(source).map_err(|error| AppError::InvalidState(format!("Nao consegui ler a imagem: {error}")))?;
    if metadata.len() as usize > MAX_COVER_BYTES {
        return Err(AppError::InvalidState("A capa precisa ter no maximo 4 MB.".into()));
    }
    let bytes = std::fs::read(source).map_err(|error| AppError::InvalidState(error.to_string()))?;
    let mut state = read_state(root);
    let record = state.records.get(plugin_id).cloned().ok_or_else(|| AppError::NotFound(format!("Plugin {plugin_id} nao instalado.")))?;
    let directory = root.join(plugin_id);
    std::fs::create_dir_all(&directory).map_err(|error| AppError::Internal(error.to_string()))?;
    for existing in COVER_EXTENSIONS {
        let _ = std::fs::remove_file(directory.join(format!("cover.{existing}")));
    }
    let file_name = format!("cover.{extension}");
    std::fs::write(directory.join(&file_name), &bytes).map_err(|error| AppError::Internal(error.to_string()))?;
    let mut updated = record;
    updated.cover = Some(file_name);
    state.records.insert(plugin_id.to_string(), updated.clone());
    write_state(root, &state)?;
    Ok(updated)
}

pub fn clear_cover_at(root: &Path, plugin_id: &str) -> AppResult<PluginRecord> {
    let mut state = read_state(root);
    let record = state.records.get(plugin_id).cloned().ok_or_else(|| AppError::NotFound(format!("Plugin {plugin_id} nao instalado.")))?;
    let directory = root.join(plugin_id);
    for existing in COVER_EXTENSIONS {
        let _ = std::fs::remove_file(directory.join(format!("cover.{existing}")));
    }
    let mut updated = record;
    updated.cover = None;
    state.records.insert(plugin_id.to_string(), updated.clone());
    write_state(root, &state)?;
    Ok(updated)
}

pub fn cover_data_url_at(root: &Path, plugin_id: &str) -> AppResult<Option<String>> {
    let state = read_state(root);
    let Some(record) = state.records.get(plugin_id) else { return Ok(None) };
    let Some(name) = record.cover.clone() else { return Ok(None) };
    if sanitize_entry(&name).map(|value| value == name).unwrap_or(false) == false {
        return Ok(None);
    }
    let path = root.join(plugin_id).join(&name);
    let bytes = match std::fs::read(&path) { Ok(bytes) => bytes, Err(_) => return Ok(None) };
    if bytes.len() > MAX_COVER_BYTES { return Ok(None); }
    let encoded = base64_encode(&bytes);
    Ok(Some(format!("data:{};base64,{encoded}", cover_mime(&name))))
}

fn base64_encode(bytes: &[u8]) -> String {
    const TABLE: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut out = String::with_capacity((bytes.len() + 2) / 3 * 4);
    for chunk in bytes.chunks(3) {
        let first = chunk[0] as u32;
        let second = *chunk.get(1).unwrap_or(&0) as u32;
        let third = *chunk.get(2).unwrap_or(&0) as u32;
        let value = (first << 16) | (second << 8) | third;
        out.push(TABLE[((value >> 18) & 0x3f) as usize] as char);
        out.push(TABLE[((value >> 12) & 0x3f) as usize] as char);
        out.push(if chunk.len() > 1 { TABLE[((value >> 6) & 0x3f) as usize] as char } else { '=' });
        out.push(if chunk.len() > 2 { TABLE[(value & 0x3f) as usize] as char } else { '=' });
    }
    out
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct PublishedPlugin {
    pub zip_path: String,
    pub entry_json: String,
    pub sha256: String,
    pub size_bytes: u64,
    pub cover_data_url: Option<String>,
}

pub fn publish_at(root: &Path, plugin_id: &str, public_base: &str) -> AppResult<PublishedPlugin> {
    let state = read_state(root);
    let record = state.records.get(plugin_id).cloned().ok_or_else(|| AppError::NotFound(format!("Plugin {plugin_id} nao instalado.")))?;
    if !record.scan.clean {
        return Err(AppError::InvalidState("Luxmc Shield bloqueou este plugin; ele nao pode ser publicado.".into()));
    }
    let files = read_plugin_files(root, plugin_id)?;
    archive_plugin(root, &record, &files, public_base)
}

fn archive_plugin(root: &Path, record: &PluginRecord, files: &BTreeMap<String, Vec<u8>>, public_base: &str) -> AppResult<PublishedPlugin> {
    use std::io::Write as _;
    use zip::write::FileOptions;
    let exports = root.join("exports");
    std::fs::create_dir_all(&exports).map_err(|error| AppError::Internal(error.to_string()))?;
    let file_name = format!("{}-{}.zip", record.manifest.id, record.manifest.version);
    let zip_path = exports.join(&file_name);
    let mut buffer = Vec::new();
    {
        let mut writer = zip::ZipWriter::new(std::io::Cursor::new(&mut buffer));
        let options = FileOptions::default().compression_method(zip::CompressionMethod::Deflated);
        for (name, content) in files {
            if !ALLOWED_EXTENSIONS.iter().any(|extension| name.to_ascii_lowercase().ends_with(&format!(".{extension}"))) {
                continue;
            }
            writer.start_file(name.clone(), options).map_err(|error| AppError::Internal(error.to_string()))?;
            writer.write_all(content).map_err(|error| AppError::Internal(error.to_string()))?;
        }
        writer.finish().map_err(|error| AppError::Internal(error.to_string()))?;
    }
    std::fs::write(&zip_path, &buffer).map_err(|error| AppError::Internal(error.to_string()))?;
    let digest = format!("{:x}", Sha256::digest(&buffer));
    let base = public_base.trim_end_matches('/');
    let entry = RegistryPlugin {
        id: record.manifest.id.clone(),
        name: record.manifest.name.clone(),
        version: record.manifest.version.clone(),
        author: record.manifest.author.clone(),
        description: record.manifest.description.clone(),
        homepage: record.manifest.homepage.clone(),
        download_url: Some(format!("{base}/{}", file_name)),
        sha256: Some(digest.clone()),
        icon_url: None,
        tags: Vec::new(),
    };
    let entry_json = serde_json::to_string_pretty(&entry).map_err(|error| AppError::Internal(error.to_string()))?;
    Ok(PublishedPlugin {
        zip_path: zip_path.to_string_lossy().into_owned(),
        entry_json,
        sha256: digest,
        size_bytes: buffer.len() as u64,
        cover_data_url: cover_data_url_at(root, &record.manifest.id)?,
    })
}

#[tauri::command]
pub fn plugins_set_cover(id: String, path: String, premium: bool) -> AppResult<PluginRecord> {
    let root = plugins_root()?;
    set_cover_at(&root, &id, Path::new(&path), premium)
}

#[tauri::command]
pub fn plugins_clear_cover(id: String) -> AppResult<PluginRecord> {
    let root = plugins_root()?;
    clear_cover_at(&root, &id)
}

#[tauri::command]
pub fn plugins_cover(id: String) -> AppResult<Option<String>> {
    let root = plugins_root()?;
    cover_data_url_at(&root, &id)
}

#[tauri::command]
#[allow(non_snake_case)]
pub fn plugins_publish(id: String, publicBase: Option<String>) -> AppResult<PublishedPlugin> {
    let root = plugins_root()?;
    let base = publicBase
        .filter(|value| value.starts_with("https://"))
        .unwrap_or_else(|| "https://luxmc-r92.pages.dev/plugins".to_string());
    publish_at(&root, &id, &base)
}

#[tauri::command]
pub fn plugins_create(id: String, name: String, author: String, description: String, permissions: Vec<String>, source: String) -> AppResult<PluginRecord> {
    let root = plugins_root()?;
    create_at(&root, &id, &name, &author, &description, &permissions, &source)
}

#[tauri::command]
pub fn plugins_update_source(id: String, source: String) -> AppResult<PluginRecord> {
    let root = plugins_root()?;
    update_source_at(&root, &id, &source)
}

#[tauri::command]
pub fn plugins_source(id: String) -> AppResult<String> {
    let root = plugins_root()?;
    read_source_at(&root, &id)
}

#[tauri::command]
pub fn plugins_list() -> AppResult<Vec<PluginRecord>> {
    list_installed()
}

#[tauri::command]
pub fn plugins_uninstall(id: String) -> AppResult<()> {
    uninstall(&id)
}

#[tauri::command]
pub fn plugins_set_enabled(id: String, enabled: bool) -> AppResult<PluginRecord> {
    set_enabled(&id, enabled)
}

pub const DEFAULT_REGISTRY_URL: &str =
    "https://raw.githubusercontent.com/predabr/luxmc/main/plugins-registry.json";

const BUNDLED_REGISTRY: &[u8] = include_bytes!("../../../assets/plugins/registry.json");

fn bundled_archive(url: &str) -> Option<&'static [u8]> {
    match url.trim_start_matches("luxmc://bundled/") {
        "pomodoro-luxmc" => Some(include_bytes!("../../../assets/plugins/pomodoro-luxmc.zip")),
        "notas-rapidas" => Some(include_bytes!("../../../assets/plugins/notas-rapidas.zip")),
        _ => None,
    }
}

pub fn bundled_catalog() -> AppResult<PluginRegistry> {
    let mut registry: PluginRegistry = serde_json::from_slice(BUNDLED_REGISTRY)
        .map_err(|error| AppError::Internal(format!("Catalogo embutido invalido: {error}")))?;
    registry.source = "luxmc://oficial".into();
    registry.fetched_at = now_stamp();
    Ok(registry)
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct PluginScript {
    pub manifest: PluginManifest,
    pub script: String,
}

#[tauri::command]
pub async fn plugins_install(state: State<'_, AppState>, id: String, url: String, sha256: Option<String>) -> AppResult<PluginRecord> {
    if let Some(bytes) = bundled_archive(&url) {
        return install_from_bytes(bytes, sha256.as_deref());
    }
    if !url.starts_with("https://") {
        return Err(AppError::InvalidState("Plugins so podem ser baixados por HTTPS.".into()));
    }
    let response = state
        .http
        .get(&url)
        .header(reqwest::header::USER_AGENT, "Luxmc/4.1 (+plugins)")
        .send()
        .await
        .map_err(|error| AppError::Internal(format!("Falha ao baixar o plugin {id}: {error}")))?;
    if !response.status().is_success() {
        return Err(AppError::Internal(format!("O servidor recusou o download do plugin {id} ({}).", response.status())));
    }
    let bytes = response
        .bytes()
        .await
        .map_err(|error| AppError::Internal(format!("Falha ao ler o plugin {id}: {error}")))?;
    install_from_bytes(&bytes, sha256.as_deref())
}

#[tauri::command]
pub async fn plugins_install_local(path: String) -> AppResult<PluginRecord> {
    let bytes = tokio::fs::read(&path).await.map_err(|error| AppError::InvalidState(format!("Nao consegui ler {path}: {error}")))?;
    install_from_bytes(&bytes, None)
}

#[tauri::command]
pub fn plugins_rescan(id: Option<String>) -> AppResult<Vec<PluginRecord>> {
    match id {
        Some(plugin_id) => {
            rescan_plugin(&plugin_id)?;
            list_installed()
        }
        None => {
            for record in plugins_list()? {
                let _ = rescan_plugin(&record.manifest.id);
            }
            plugins_list()
        }
    }
}

#[tauri::command]
pub fn plugins_script(id: String) -> AppResult<PluginScript> {
    let (manifest, script) = plugin_bundle(&id)?;
    Ok(PluginScript { manifest, script })
}

#[tauri::command]
pub fn plugins_open_folder(app: tauri::AppHandle) -> AppResult<String> {
    use tauri_plugin_opener::OpenerExt;
    let root = plugins_root()?;
    std::fs::create_dir_all(&root).map_err(|error| AppError::Internal(error.to_string()))?;
    app.opener()
        .open_path(root.to_string_lossy().into_owned(), None::<&str>)
        .map_err(|error| AppError::Internal(error.to_string()))?;
    Ok(root.to_string_lossy().into_owned())
}

#[tauri::command]
pub async fn plugins_registry(state: State<'_, AppState>, url: Option<String>) -> AppResult<PluginRegistry> {
    let remote = url.filter(|value| value.starts_with("https://")).unwrap_or_else(|| DEFAULT_REGISTRY_URL.to_string());
    match fetch_registry(&state.http, &remote).await {
        Ok(registry) => Ok(registry),
        Err(_) => bundled_catalog(),
    }
}

async fn fetch_registry(client: &reqwest::Client, source: &str) -> AppResult<PluginRegistry> {
    let response = client
        .get(source)
        .header(reqwest::header::USER_AGENT, "Luxmc/4.1 (+plugins)")
        .send()
        .await
        .map_err(|error| AppError::Internal(format!("Catalogo de plugins indisponivel: {error}")))?;
    if !response.status().is_success() {
        return Err(AppError::Internal(format!("Catalogo de plugins indisponivel ({}).", response.status())));
    }
    let mut registry: PluginRegistry = response
        .json()
        .await
        .map_err(|error| AppError::Internal(format!("Catalogo de plugins invalido: {error}")))?;
    registry.source = source.to_string();
    registry.fetched_at = now_stamp();
    Ok(registry)
}



