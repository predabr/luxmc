use super::{ShieldScanResult, ShieldThreat};
use crate::error::{AppError, AppResult};
use sha1::Sha1;
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::io::{BufReader, Cursor, Read, Seek};
use std::path::{Path, PathBuf};
use std::sync::Mutex;

const CACHE_MAX: usize = 8192;
const MAX_JAR_BYTES: u64 = 8 * 1024 * 1024 * 1024;
const MAX_CLASS_BYTES: u64 = 4 * 1024 * 1024;
const MAX_SCAN_BYTES: u64 = 128 * 1024 * 1024;
const MAX_ENTRIES: usize = 100_000;
const BUFFER_BYTES: u64 = 16 * 1024 * 1024;
static CLEAN_CACHE: Mutex<Option<HashMap<PathBuf, String>>> = Mutex::new(None);
trait ArchiveReader: Read + Seek {}
impl<T: Read + Seek> ArchiveReader for T {}

const KNOWN_SHA256: &[&str] = &[
    "1d1aaccdc13244e980c0c024610ecc77ea2674a33a52129edf1bb4ce3b2cc2fc",
];
const KNOWN_SHA1: &[&str] = &[
    "dc43c4685c3f47808ac207d1667cc1eb915b2d82",
    "c2d0c87a1fe99e3c44a52c48d8bcf65a67b3e9a5",
    "e299bf5a025f5c3fff45d017c3c2f467fa599915",
    "33677ca0e4c565b1f34baa74a79c09a3b690bf41",
    "2db855a7f40c015f8c9ca7cbab69e1f1aafa210b",
    "284a4449e58868036b2bafdfb5a210fd0480ef4a",
    "0c6576bdc6d1b92d581c18f3a150905ad97fa080",
    "c55c3e9d6a4355f36b0710ab189d5131a290df26",
    "32536577d5bb074abd493ad98dc12ccc86f30172",
    "a4b6385d1140c111549d95eab25cb51922eefba2",
];

fn contains(bytes: &[u8], needle: &[u8]) -> bool {
    memchr::memmem::find(bytes, needle).is_some()
}

fn known_hash(sha256: &str, sha1: &str) -> bool {
    KNOWN_SHA256.contains(&sha256) || KNOWN_SHA1.contains(&sha1)
}

fn cache_update(path: &Path, hash: Option<String>) {
    let mut guard = CLEAN_CACHE.lock().unwrap_or_else(|error| error.into_inner());
    let cache = guard.get_or_insert_with(HashMap::new);
    if cache.len() >= CACHE_MAX { cache.clear(); }
    if let Some(hash) = hash { cache.insert(path.to_owned(), hash); } else { cache.remove(path); }
}

fn scan_file(path: &Path) -> Result<Vec<(&'static str, &'static str, &'static str)>, String> {
    let mut file = std::fs::File::open(path).map_err(|error| error.to_string())?;
    let before = file.metadata().map_err(|error| error.to_string())?;
    if before.len() > MAX_JAR_BYTES { return Err("O arquivo excede o limite de 8 GiB da verificação.".into()); }
    let mut sha256 = Sha256::new();
    let mut sha1 = Sha1::new();
    let mut chunk = [0u8; 64 * 1024];
    let mut hashed = 0u64;
    let mut buffered = (before.len() <= BUFFER_BYTES).then(|| Vec::with_capacity(before.len() as usize));
    loop {
        let length = file.read(&mut chunk).map_err(|error| error.to_string())?;
        if length == 0 { break; }
        hashed = hashed.saturating_add(length as u64);
        if hashed > MAX_JAR_BYTES { return Err("O arquivo excede o limite da verificação.".into()); }
        if hashed > BUFFER_BYTES { buffered = None; }
        if let Some(bytes) = &mut buffered { bytes.extend_from_slice(&chunk[..length]); }
        sha256.update(&chunk[..length]);
        sha1.update(&chunk[..length]);
    }
    let after_hash = file.metadata().map_err(|error| error.to_string())?;
    if before.len() != after_hash.len() || before.modified().ok() != after_hash.modified().ok() {
        return Err("O arquivo foi alterado durante a verificação. Tente novamente.".into());
    }
    let hash = format!("{:x}", sha256.finalize());
    let sha1 = format!("{:x}", sha1.finalize());
    if known_hash(&hash, &sha1) {
        return Ok(vec![("critical", "Malware conhecido", "O hash corresponde a um arquivo malicioso documentado pela investigação do Fractureiser.")]);
    }
    let cached = CLEAN_CACHE.lock().unwrap_or_else(|error| error.into_inner())
        .as_ref().and_then(|cache| cache.get(path)).is_some_and(|value| value == &hash);
    if cached { return Ok(Vec::new()); }
    file.rewind().map_err(|error| error.to_string())?;
    let reader: Box<dyn ArchiveReader> = match buffered {
        Some(bytes) => Box::new(Cursor::new(bytes)),
        None => Box::new(BufReader::with_capacity(64 * 1024, file.try_clone().map_err(|error| error.to_string())?)),
    };
    let mut archive = zip::ZipArchive::new(reader).map_err(|error| format!("JAR inválido: {error}"))?;
    if archive.len() > MAX_ENTRIES { return Err("O arquivo excede o limite de entradas da verificação.".into()); }
    let mut inspected = 0u64;
    let mut webhook = false;
    let mut suspicious_name = false;
    let mut fractureiser = false;
    let mut names = 0usize;
    let mut classes = Vec::new();
    for name in archive.file_names() {
        names += 1;
        let normalized = name.to_ascii_lowercase();
        suspicious_name |= normalized.rsplit('/').next() == Some("l10.class") || normalized.ends_with("/updater.jar") || normalized == "updater.jar";
        if normalized.ends_with(".class") { classes.push(name.to_owned()); }
    }
    classes.sort();
    let duplicate_names = names != archive.len();
    let count = if duplicate_names { archive.len() } else { classes.len() };
    for index in 0..count {
        let mut entry = if duplicate_names { archive.by_index(index) } else { archive.by_name(&classes[index]) }.map_err(|error| error.to_string())?;
        let name = entry.name().to_ascii_lowercase();
        suspicious_name |= name.rsplit('/').next() == Some("l10.class") || name.ends_with("/updater.jar") || name == "updater.jar";
        if !name.ends_with(".class") { continue; }
        if entry.size() > MAX_CLASS_BYTES { return Err("Uma classe excede o limite de 4 MiB da verificação.".into()); }
        inspected = inspected.saturating_add(entry.size());
        if inspected > MAX_SCAN_BYTES { return Err("O bytecode excede o limite de 128 MiB da verificação.".into()); }
        let mut bytes = Vec::new();
        entry.by_ref().take(MAX_CLASS_BYTES + 1).read_to_end(&mut bytes).map_err(|error| error.to_string())?;
        if bytes.len() as u64 > MAX_CLASS_BYTES { return Err("Uma classe excede o limite da verificação.".into()); }
        webhook |= contains(&bytes, b"discord.com/api/webhooks") || contains(&bytes, b"discordapp.com/api/webhooks");
        fractureiser |= (contains(&bytes, b"85.217.144.130") || contains(&bytes, b"files-8ie.pages.dev"))
            && (contains(&bytes, b"java/net/URLClassLoader") || contains(&bytes, b"java.net.URLClassLoader"));
        fractureiser |= contains(&bytes, b"dev/neko/nekoclient/Client") && contains(&bytes, b"neko.run");
    }
    let after = file.metadata().map_err(|error| error.to_string())?;
    if before.len() != after.len() || before.modified().ok() != after.modified().ok() {
        return Err("O arquivo foi alterado durante a verificação. Tente novamente.".into());
    }
    let mut findings = Vec::new();
    if fractureiser { findings.push(("critical", "Indicadores de Fractureiser", "O bytecode combina indicadores documentados de carregamento remoto do Fractureiser.")); }
    if suspicious_name { findings.push(("warning", "Estrutura suspeita", "O JAR contém L10.class ou updater.jar. O nome isolado não comprova malware; confira a origem do mod.")); }
    if webhook { findings.push(("warning", "Webhook Discord embutido", "O bytecode contém um webhook do Discord. Pode ser legítimo; confira a finalidade e a origem do mod.")); }
    if findings.is_empty() { cache_update(path, Some(hash)); }
    Ok(findings)
}

fn threat(path: &Path, severity: &str, threat_type: &str, description: String) -> ShieldThreat {
    ShieldThreat {
        file_name: path.file_name().map(|name| name.to_string_lossy().into_owned()).unwrap_or_else(|| "mods".into()),
        file_path: path.to_string_lossy().into_owned(),
        severity: severity.into(),
        threat_type: threat_type.into(),
        description,
    }
}

pub fn scan_mods_directory(mods_dir: &Path) -> ShieldScanResult {
    let start = std::time::Instant::now();
    let mut paths = Vec::new();
    let mut threats = Vec::new();
    match std::fs::read_dir(mods_dir) {
        Ok(entries) => for entry in entries {
            let entry = match entry {
                Ok(entry) => entry,
                Err(error) => { threats.push(threat(mods_dir, "critical", "Verificação incompleta", error.to_string())); continue; }
            };
            let path = entry.path();
            if !path.extension().is_some_and(|extension| extension.eq_ignore_ascii_case("jar")) { continue; }
            paths.push(path);
        },
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {},
        Err(error) => threats.push(threat(mods_dir, "critical", "Verificação incompleta", error.to_string())),
    }
    paths.sort();
    if !paths.is_empty() {
        let workers = std::thread::available_parallelism().map_or(1, |count| count.get()).min(4).min(paths.len());
        let chunk_size = paths.len().div_ceil(workers);
        std::thread::scope(|scope| {
            let handles = paths.chunks(chunk_size).map(|chunk| scope.spawn(move || {
                chunk.iter().flat_map(|path| match scan_file(path) {
                    Ok(findings) => {
                        if !findings.is_empty() { cache_update(path, None); }
                        findings.into_iter().map(|(severity, kind, description)| threat(path, severity, kind, description.into())).collect::<Vec<_>>()
                    }
                    Err(error) => {
                        cache_update(path, None);
                        vec![threat(path, "critical", "Verificação incompleta", error)]
                    }
                }).collect::<Vec<_>>()
            })).collect::<Vec<_>>();
            for handle in handles {
                match handle.join() {
                    Ok(findings) => threats.extend(findings),
                    Err(_) => threats.push(threat(mods_dir, "critical", "Verificação incompleta", "Uma tarefa de verificação foi interrompida. Tente novamente.".into())),
                }
            }
        });
    }
    ShieldScanResult { total_scanned: paths.len(), is_clean: threats.is_empty(), threats, scan_time_ms: start.elapsed().as_millis() as u64 }
}

pub(crate) fn enforce_scan(result: &ShieldScanResult) -> AppResult<()> {
    if let Some(threat) = result.threats.iter().find(|threat| threat.severity == "critical") {
        return Err(AppError::InvalidState(format!("Luxmc Shield bloqueou o lançamento: {} — {}. {}", threat.file_name, threat.threat_type, threat.description)));
    }
    Ok(())
}

#[cfg(test)]
mod tests;
