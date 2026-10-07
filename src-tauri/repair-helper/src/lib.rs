use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{collections::BTreeMap, fs, io::{self, Read, Write}, path::{Path, PathBuf}};

pub const MAIN: &str = if cfg!(windows) { "luxmc.exe" } else { "luxmc" };
pub const HELPER: &str = if cfg!(windows) { "luxmc-repair.exe" } else { "luxmc-repair" };
const FILES: [&str; 3] = [MAIN, HELPER, "uninstall.exe"];

#[derive(Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Snapshot {
    version: String,
    files: BTreeMap<String, String>,
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Manifest {
    format: u32,
    current: Snapshot,
    previous: Option<Snapshot>,
    active_previous: bool,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Report {
    pub healthy: bool,
    pub repaired: bool,
    pub rolled_back: bool,
    pub version: String,
    pub changed_files: Vec<String>,
    pub invalid_files: Vec<String>,
    pub previous_available: bool,
}

pub struct Recovery {
    pub app: PathBuf,
    pub cache: PathBuf,
    expected: String,
    version: String,
}

fn invalid(message: &str) -> io::Error { io::Error::new(io::ErrorKind::InvalidData, message) }

pub fn file_hash(path: &Path) -> io::Result<String> {
    if fs::symlink_metadata(path)?.file_type().is_symlink() { return Err(invalid("Arquivo de programa redirecionado.")); }
    let file = fs::File::open(path)?;
    let mut reader = io::BufReader::new(file);
    let mut digest = Sha256::new();
    let mut tail = Vec::new();
    let mut buffer = [0u8; 65536];
    let name = path.file_name().and_then(|name| name.to_str()).unwrap_or("");
    let main_payload = matches!(name, "luxmc.exe" | "luxmc" | "luxmc.repair-stage" | "previous-main.exe" | "previous-main.repair-stage");
    loop {
        let count = reader.read(&mut buffer)?;
        tail.extend_from_slice(&buffer[..count]);
        for index in 0..tail.len().saturating_sub(9) {
            if main_payload && &tail[index..index + 10] == b"PE_VAR_NSS" { tail[index..index + 10].copy_from_slice(b"PE_VAR_UNK"); }
        }
        let ready = if count == 0 { tail.len() } else { tail.len().saturating_sub(9) };
        digest.update(&tail[..ready]);
        tail.drain(..ready);
        if count == 0 { break; }
    }
    Ok(format!("{:x}", digest.finalize()))
}

fn atomic_write(path: &Path, bytes: &[u8]) -> io::Result<()> {
    let stage = path.with_extension("repair-stage");
    let mut file = fs::OpenOptions::new().write(true).create_new(true).open(&stage).or_else(|error| {
        if error.kind() != io::ErrorKind::AlreadyExists { return Err(error); }
        if fs::symlink_metadata(&stage)?.file_type().is_symlink() { return Err(invalid("Arquivo temporário redirecionado.")); }
        fs::OpenOptions::new().write(true).truncate(true).open(&stage)
    })?;
    file.write_all(bytes)?;
    file.sync_all()?;
    drop(file);
    fs::rename(&stage, path)
}

fn copy_verified(source: &Path, target: &Path, expected: &str) -> io::Result<()> {
    if file_hash(source)? != expected { return Err(invalid("A cópia de recuperação também está corrompida.")); }
    let stage = target.with_extension("repair-stage");
    if stage.exists() && fs::symlink_metadata(&stage)?.file_type().is_symlink() { return Err(invalid("Arquivo temporário redirecionado.")); }
    fs::copy(source, &stage)?;
    fs::OpenOptions::new().write(true).open(&stage)?.sync_all()?;
    if file_hash(&stage)? != expected { return Err(invalid("Falha ao verificar o arquivo restaurado.")); }
    fs::rename(stage, target)
}

impl Recovery {
    pub fn new(app: PathBuf, cache: PathBuf, expected: String, version: String) -> io::Result<Self> {
        if expected.len() != 64 || !expected.bytes().all(|byte| byte.is_ascii_hexdigit()) { return Err(invalid("Verificador sem hash de distribuição válido.")); }
        let app = app.canonicalize()?;
        fs::create_dir_all(&cache)?;
        let cache = cache.canonicalize()?;
        if cache == app || app.starts_with(&cache) || cache.starts_with(&app) { return Err(invalid("A recuperação precisa usar uma pasta separada.")); }
        Ok(Self { app, cache, expected: expected.to_ascii_lowercase(), version })
    }

    fn manifest(&self) -> io::Result<Manifest> {
        self.read_manifest("manifest.json").or_else(|_| self.read_manifest("manifest-backup.json"))
    }

    fn read_manifest(&self, name: &str) -> io::Result<Manifest> {
        let file = self.cache.join(name);
        if fs::metadata(&file)?.len() > 65536 { return Err(invalid("Manifesto de recuperação inválido.")); }
        let manifest: Manifest = serde_json::from_slice(&fs::read(file)?).map_err(|error| invalid(&error.to_string()))?;
        if manifest.format != 1 || manifest.current.files.get(MAIN) != Some(&self.expected) { return Err(invalid("Manifesto não corresponde a esta instalação.")); }
        for snapshot in std::iter::once(&manifest.current).chain(manifest.previous.iter()) {
            if snapshot.files.keys().any(|name| !FILES.contains(&name.as_str())) || snapshot.files.values().any(|hash| hash.len() != 64 || !hash.bytes().all(|byte| byte.is_ascii_hexdigit())) { return Err(invalid("Arquivos de recuperação inválidos.")); }
        }
        Ok(manifest)
    }

    fn save(&self, manifest: &Manifest) -> io::Result<()> {
        let bytes = serde_json::to_vec(manifest).map_err(|error| invalid(&error.to_string()))?;
        atomic_write(&self.cache.join("manifest-backup.json"), &bytes)?;
        atomic_write(&self.cache.join("manifest.json"), &bytes)
    }

    pub fn initialize(&self) -> io::Result<Report> {
        if file_hash(&self.app.join(MAIN))? != self.expected { return Err(invalid("O programa instalado não corresponde ao instalador verificado.")); }
        let previous_manifest: Option<Manifest> = fs::read(self.cache.join("manifest.json")).ok().filter(|bytes| bytes.len() <= 65536).and_then(|bytes| serde_json::from_slice(&bytes).ok());
        let mut previous = None;
        if let Some(old) = previous_manifest {
            if let Some(snapshot) = old.previous.clone() {
                if snapshot.files.len() == 1 && snapshot.files.get(MAIN).is_some_and(|hash| file_hash(&self.cache.join("previous-main.exe")).ok().as_ref() == Some(hash)) { previous = Some(snapshot); }
            }
            if let Some(hash) = old.current.files.get(MAIN).filter(|hash| *hash != &self.expected) {
                if copy_verified(&self.cache.join("current").join(MAIN), &self.cache.join("previous-main.exe"), hash).is_ok() {
                    previous = Some(Snapshot { version: old.current.version, files: BTreeMap::from([(MAIN.to_string(), hash.clone())]) });
                }
            }
        }
        fs::create_dir_all(self.cache.join("current"))?;
        let mut files = BTreeMap::new();
        for name in FILES {
            let source = self.app.join(name);
            if !source.is_file() { continue; }
            let hash = file_hash(&source)?;
            copy_verified(&source, &self.cache.join("current").join(name), &hash)?;
            files.insert(name.to_owned(), hash);
        }
        let manifest = Manifest { format: 1, current: Snapshot { version: self.version.clone(), files }, previous, active_previous: false };
        self.save(&manifest)?;
        self.verify(false)
    }

    pub fn verify(&self, repair: bool) -> io::Result<Report> {
        let mut manifest = match self.manifest() {
            Ok(manifest) => manifest,
            Err(_) if repair && file_hash(&self.app.join(MAIN)).ok().as_ref() == Some(&self.expected) => return self.initialize(),
            Err(error) => return Err(error),
        };
        let previous_valid = manifest.previous.as_ref().is_some_and(|snapshot| snapshot.files.get(MAIN).is_some_and(|hash| file_hash(&self.cache.join("previous-main.exe")).ok().as_ref() == Some(hash)));
        let active = if manifest.active_previous { manifest.previous.as_ref().ok_or_else(|| invalid("Cópia anterior ausente."))? } else { &manifest.current };
        let mut expected_main = active.files.get(MAIN).ok_or_else(|| invalid("Programa ausente do manifesto."))?.clone();
        let mut report = Report { healthy: true, repaired: false, rolled_back: manifest.active_previous, version: active.version.clone(), changed_files: Vec::new(), invalid_files: Vec::new(), previous_available: previous_valid };
        for name in manifest.current.files.keys() {
            let expected = if name == MAIN { &expected_main } else { &manifest.current.files[name] };
            if file_hash(&self.app.join(name)).ok().as_ref() == Some(expected) { continue; }
            report.invalid_files.push(name.clone());
            if !repair { continue; }
            let (source, hash) = if name == MAIN && previous_valid {
                let snapshot = manifest.previous.as_ref().unwrap();
                let hash = snapshot.files[MAIN].clone();
                report.rolled_back = true;
                report.version = snapshot.version.clone();
                manifest.active_previous = true;
                (self.cache.join("previous-main.exe"), hash)
            } else {
                if name == MAIN { report.rolled_back = false; report.version = manifest.current.version.clone(); manifest.active_previous = false; }
                (self.cache.join("current").join(name), manifest.current.files[name].clone())
            };
            copy_verified(&source, &self.app.join(name), &hash)?;
            if name == MAIN { expected_main = hash; }
            report.changed_files.push(name.clone());
        }
        report.repaired = !report.changed_files.is_empty();
        report.healthy = report.invalid_files.is_empty() || report.changed_files.len() == report.invalid_files.len();
        if repair {
            self.save(&manifest)?;
            atomic_write(&self.cache.join("last-report.json"), &serde_json::to_vec(&report).map_err(|error| invalid(&error.to_string()))?)?;
        }
        Ok(report)
    }
}

pub fn cache_for(app: &Path) -> io::Result<PathBuf> {
    let base = std::env::var_os(if cfg!(windows) { "LOCALAPPDATA" } else { "XDG_STATE_HOME" }).map(PathBuf::from).or_else(|| std::env::var_os("HOME").map(|home| PathBuf::from(home).join(".local/state"))).ok_or_else(|| invalid("Pasta de recuperação indisponível."))?;
    let identity = app.canonicalize()?.to_string_lossy().to_lowercase();
    let hash = format!("{:x}", Sha256::digest(identity.as_bytes()));
    Ok(base.join("LuxmcRecovery").join(&hash[..24]))
}
