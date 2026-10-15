use std::{collections::{HashMap, HashSet}, io::{Read, Seek}, path::Path};
use crate::error::AppResult;

#[derive(Clone, Default)]
pub struct Audit { pub errors: Vec<String>, pub warnings: Vec<String> }

type Fingerprint = (u64, std::time::SystemTime, Option<std::time::SystemTime>);
type Inspection = (Audit, HashMap<String, String>, Vec<(String, String)>);
type MetadataEntry = (Fingerprint, Option<ModMetadata>);
type AuditEntry = (Fingerprint, Inspection);
static METADATA_CACHE: std::sync::LazyLock<std::sync::Mutex<HashMap<std::path::PathBuf, MetadataEntry>>> = std::sync::LazyLock::new(|| std::sync::Mutex::new(HashMap::new()));
static AUDIT_CACHE: std::sync::LazyLock<std::sync::Mutex<HashMap<std::path::PathBuf, AuditEntry>>> = std::sync::LazyLock::new(|| std::sync::Mutex::new(HashMap::new()));

fn fingerprint(metadata: &std::fs::Metadata) -> Option<Fingerprint> {
    metadata.is_file().then(|| (metadata.len(), metadata.modified().ok(), metadata.created().ok()))
        .and_then(|(length, modified, created)| modified.map(|modified| (length, modified, created)))
}

struct ArchiveReader<'a> {
    reader: std::io::BufReader<std::fs::File>,
    before: std::fs::Metadata,
    cancel: Option<&'a std::sync::atomic::AtomicBool>,
    position: u64,
}

impl ArchiveReader<'_> {
    fn unchanged(&self) -> AppResult<()> {
        crate::core::mods::pack_download::cancelled(self.cancel)?;
        let after = self.reader.get_ref().metadata()?;
        if self.before.len() != after.len() || self.before.modified()? != after.modified()? {
            return Err(crate::error::AppError::InvalidInput("O arquivo mudou durante a conferência".into()));
        }
        Ok(())
    }

    fn check_cancel(&self) -> std::io::Result<()> {
        crate::core::mods::pack_download::cancelled(self.cancel)
            .map_err(|error| std::io::Error::other(error.to_string()))
    }
}

impl Read for ArchiveReader<'_> {
    fn read(&mut self, bytes: &mut [u8]) -> std::io::Result<usize> {
        self.check_cancel()?;
        let count = self.reader.read(bytes)?;
        self.position += count as u64;
        Ok(count)
    }
}

impl Seek for ArchiveReader<'_> {
    fn seek(&mut self, position: std::io::SeekFrom) -> std::io::Result<u64> {
        self.check_cancel()?;
        let target = match position {
            std::io::SeekFrom::Start(target) => target,
            std::io::SeekFrom::Current(offset) => self.position.checked_add_signed(offset)
                .ok_or_else(|| std::io::Error::new(std::io::ErrorKind::InvalidInput, "Posição inválida no arquivo"))?,
            std::io::SeekFrom::End(offset) => {
                self.position = self.reader.seek(std::io::SeekFrom::End(offset))?;
                return Ok(self.position);
            }
        };
        if let Ok(offset) = i64::try_from(i128::from(target) - i128::from(self.position)) {
            self.reader.seek_relative(offset)?;
        } else {
            self.reader.seek(std::io::SeekFrom::Start(target))?;
        }
        self.position = target;
        Ok(target)
    }
}

fn archive_reader<'a>(path: &Path, cancel: Option<&'a std::sync::atomic::AtomicBool>) -> AppResult<ArchiveReader<'a>> {
    crate::core::mods::pack_download::cancelled(cancel)?;
    let file = std::fs::File::open(path)?;
    let before = file.metadata()?;
    Ok(ArchiveReader { reader: std::io::BufReader::with_capacity(64 * 1024, file), before, cancel, position: 0 })
}

#[derive(Clone, Default)]
pub(crate) struct ModMetadata {
    pub ids: std::collections::BTreeSet<String>,
    pub loaders: std::collections::BTreeSet<String>,
    pub minecraft_declared: bool,
    pub versions: std::collections::BTreeMap<String, String>,
}

pub(crate) fn primary_ids(path: &Path) -> Option<std::collections::BTreeSet<String>> {
    mod_metadata(path).map(|metadata| metadata.ids)
}

pub(crate) fn mod_metadata(path: &Path) -> Option<ModMetadata> {
    let metadata = std::fs::metadata(path).ok()?;
    let fingerprint = fingerprint(&metadata)?;
    if let Ok(cache) = METADATA_CACHE.lock() {
        if let Some((saved, ids)) = cache.get(path) { if *saved == fingerprint { return ids.clone(); } }
    }
    let ids = read_mod_metadata(path);
    if let Ok(mut cache) = METADATA_CACHE.lock() {
        if cache.len() >= 4096 { cache.clear(); }
        cache.insert(path.to_owned(), (fingerprint, ids.clone()));
    }
    ids
}

fn read_mod_metadata(path: &Path) -> Option<ModMetadata> {
    let mut reader = archive_reader(path, None).ok()?;
    let metadata = read_mod_metadata_from_reader(&mut reader)?;
    reader.unchanged().ok()?;
    Some(metadata)
}

fn read_mod_metadata_from_reader<R: Read + Seek>(reader: R) -> Option<ModMetadata> {
    let mut archive = zip::ZipArchive::new(reader).ok()?;
    let mut metadata = ModMetadata::default();
    let jar_version = archive.by_name("META-INF/MANIFEST.MF").ok().and_then(|file| {
        let mut text = String::new();
        file.take(128 * 1024).read_to_string(&mut text).ok()?;
        let unfolded = text.replace("\r\n ", "").replace("\n ", "");
        unfolded.lines().find_map(|line| line.split_once(':').filter(|(key,_)|key.eq_ignore_ascii_case("Implementation-Version")).map(|(_,value)|value.trim().to_owned()))
    });
    for (name, pointer) in [("fabric.mod.json", "/id"), ("quilt.mod.json", "/quilt_loader/id")] {
        if let Ok(file) = archive.by_name(name) {
            if let Ok(value) = serde_json::from_reader::<_, serde_json::Value>(file.take(2 * 1024 * 1024)) {
                if let Some(id) = value.pointer(pointer).and_then(|v| v.as_str()) {
                    metadata.ids.insert(id.to_owned());
                    if let Some(version) = value.pointer(if name == "fabric.mod.json" { "/version" } else { "/quilt_loader/version" }).and_then(|value| value.as_str()) { metadata.versions.insert(id.to_owned(), version.to_owned()); }
                    metadata.loaders.insert(if name == "fabric.mod.json" { "fabric" } else { "quilt" }.into());
                    metadata.minecraft_declared |= value.pointer("/depends/minecraft").is_some_and(|version| version.is_string() || version.is_array())
                        || value.pointer("/quilt_loader/depends").and_then(|depends| depends.as_array()).is_some_and(|depends| depends.iter().any(|dependency| dependency["id"] == "minecraft" && dependency.get("versions").is_some()));
                }
            }
        }
    }
    for name in ["META-INF/neoforge.mods.toml", "META-INF/mods.toml"] {
        if let Ok(file) = archive.by_name(name) {
            let mut text = String::new();
            if file.take(2 * 1024 * 1024).read_to_string(&mut text).is_ok() {
                if let Ok(value) = text.parse::<toml::Value>() {
                    if let Some(mods) = value.get("mods").and_then(|v| v.as_array()) {
                        for module in mods { if let Some(id) = module.get("modId").and_then(|v| v.as_str()) { metadata.ids.insert(id.to_owned()); if let Some(version) = module.get("version").and_then(|value| value.as_str()) { metadata.versions.insert(id.to_owned(), version.to_owned()); } } }
                        let dependencies = value.get("dependencies").and_then(|deps| deps.as_table());
                        let has_dependency = |id: &str| dependencies.is_some_and(|deps| deps.values().filter_map(|deps| deps.as_array()).flatten().any(|dependency| dependency.get("modId").and_then(|value| value.as_str()) == Some(id)));
                        metadata.loaders.insert(if name.contains("neoforge") || has_dependency("neoforge") { "neoforge" } else { "forge" }.into());
                        metadata.minecraft_declared |= has_dependency("minecraft");
                    }
                }
            }
        }
    }
    for version in metadata.versions.values_mut() {
        if version == "${file.jarVersion}" { if let Some(value) = &jar_version { *version = value.clone(); } }
    }
    (!metadata.ids.is_empty()).then_some(metadata)
}

pub fn repair_duplicates(root: &Path, preferred: &HashSet<String>) -> AppResult<usize> { repair_duplicates_cancellable(root, preferred, None) }

pub(crate) fn select_pack_duplicate_versions(root: &Path, pinned: &HashSet<String>, preferred: &mut HashSet<String>) {
    select_pack_duplicate_versions_for_loader(root,pinned,preferred,"");
}

pub(crate) fn select_pack_duplicate_versions_for_loader(root: &Path, pinned: &HashSet<String>, preferred: &mut HashSet<String>, loader: &str) {
    let metadata: Vec<_> = pinned.iter().filter_map(|name|mod_metadata(&root.join("mods").join(name)).map(|metadata|(name,metadata))).collect();
    let compatible = |loaders: &std::collections::BTreeSet<String>| loaders.contains(loader) || loader == "quilt" && loaders.contains("fabric");
    for (name,item) in &metadata {
        if !item.loaders.is_empty() && !compatible(&item.loaders) && metadata.iter().any(|(_,other)|other.ids == item.ids && compatible(&other.loaders)) { preferred.remove(*name); }
    }
    let mut groups = std::collections::BTreeMap::<_, Vec<_>>::new();
    for (name,metadata) in metadata {
        if !preferred.contains(name) { continue; }
        if metadata.ids.len() != 1 { continue; }
        let id = metadata.ids.iter().next().unwrap();
        let Some(version) = metadata.versions.get(id).and_then(|version| {
            let value = version.trim();
            let value = if value.split('.').all(|part|!part.is_empty() && part.bytes().all(|byte|byte.is_ascii_digit())) {
                match value.split('.').count() { 1 => format!("{value}.0.0"), 2 => format!("{value}.0"), _ => value.to_owned() }
            } else { value.to_owned() };
            semver::Version::parse(&value).ok()
        }) else { continue; };
        if !version.pre.is_empty() { continue; }
        groups.entry((metadata.ids, metadata.loaders)).or_default().push((version, name));
    }
    for files in groups.values_mut() {
        files.sort_by(|a, b| b.0.cmp(&a.0));
        if files.len() > 1 && files[0].0 != files[1].0 {
            for (_, name) in &files[1..] { preferred.remove(*name); }
        }
    }
}

pub fn repair_duplicates_cancellable(root: &Path, preferred: &HashSet<String>, cancel: Option<&std::sync::atomic::AtomicBool>) -> AppResult<usize> {
    crate::core::mods::pack_download::cancelled(cancel)?;
    let directory = root.join("mods");
    if !directory.is_dir() { return Ok(0); }
    let mut paths = std::fs::read_dir(&directory)?.filter_map(Result::ok)
        .filter(|entry| entry.file_type().is_ok_and(|kind| kind.is_file()) && entry.path().extension().is_some_and(|extension| extension == "jar"))
        .map(|entry| entry.path()).collect::<Vec<_>>();
    paths.sort_by_key(|path| (!preferred.contains(&path.file_name().unwrap_or_default().to_string_lossy().into_owned()), path.file_name().map(|name| name.to_os_string())));
    let mut retained = std::collections::BTreeMap::<std::collections::BTreeSet<String>, std::path::PathBuf>::new();
    let mut removed = Vec::new();
    for path in paths {
        crate::core::mods::pack_download::cancelled(cancel)?;
        let Some(ids) = primary_ids(&path) else { continue; };
        if let Some(kept) = retained.get(&ids) {
            let kept_pinned = preferred.contains(&kept.file_name().unwrap_or_default().to_string_lossy().into_owned());
            let other_pinned = preferred.contains(&path.file_name().unwrap_or_default().to_string_lossy().into_owned());
            let identical = || -> AppResult<bool> {
                use sha2::Digest;
                fn digest(path: &Path, cancel: Option<&std::sync::atomic::AtomicBool>) -> AppResult<Vec<u8>> {
                    let mut file = std::fs::File::open(path)?;
                    let mut hash = sha2::Sha256::new();
                    let mut bytes = [0u8; 65536];
                    loop { crate::core::mods::pack_download::cancelled(cancel)?; let count = file.read(&mut bytes)?; if count == 0 { break; } hash.update(&bytes[..count]); }
                    Ok(hash.finalize().to_vec())
                }
                Ok(std::fs::metadata(kept)?.len() == std::fs::metadata(&path)?.len() && digest(kept, cancel)? == digest(&path, cancel)?)
            };
            if kept_pinned && !other_pinned || identical()? { removed.push((path, kept.clone())); }
        }
        else { retained.insert(ids, path); }
    }
    if removed.is_empty() { return Ok(0); }
    let backup = root.join(".luxmc/removed-duplicates").join(uuid::Uuid::new_v4().to_string());
    std::fs::create_dir_all(&backup)?;
    let mut records = Vec::new();
    for (path, kept) in &removed {
        crate::core::mods::pack_download::cancelled(cancel)?;
        let name = path.file_name().unwrap_or_default();
        std::fs::rename(path, backup.join(name))?;
        records.push(serde_json::json!({"removed": name.to_string_lossy(), "retained": kept.file_name().unwrap_or_default().to_string_lossy()}));
    }
    std::fs::write(backup.join("repair.json"), serde_json::to_vec_pretty(&records)?)?;
    Ok(removed.len())
}

pub fn audit(root: &Path, deep: bool) -> AppResult<Audit> { audit_cancellable(root, deep, None) }

fn inspect_archive(folder: &str, path: &Path, deep: bool, cancel: Option<&std::sync::atomic::AtomicBool>) -> AppResult<(Audit, HashMap<String,String>, Vec<(String,String)>)> {
    crate::core::mods::pack_download::cancelled(cancel)?;
    if !deep && folder == "mods" {
        if let Some(current) = std::fs::metadata(path).ok().and_then(|metadata| fingerprint(&metadata)) {
            let cached = AUDIT_CACHE.lock().ok().and_then(|cache| cache.get(path).filter(|(saved, _)| *saved == current).map(|(_, inspection)| inspection.clone()));
            if let Some(cached) = cached {
                crate::core::mods::pack_download::cancelled(cancel)?;
                if std::fs::metadata(path).ok().and_then(|metadata| fingerprint(&metadata)) == Some(current) { return Ok(cached); }
            }
        }
    }
    let started = std::time::Instant::now();
    let mut reader = archive_reader(path, cancel)?;
    let result = inspect_archive_from_reader(folder, path, deep, cancel, &mut reader);
    reader.unchanged()?;
    if started.elapsed().as_millis() > 500 { tracing::info!(file = ?path.file_name(), seconds = started.elapsed().as_secs_f64(), "Slow archive validation"); }
    result
}

pub(crate) fn remember_verified_mod_archive(path: &Path, bytes: &[u8]) {
    if path.parent().and_then(|parent| parent.file_name()).and_then(|name| name.to_str()) != Some("mods") || path.extension().and_then(|extension| extension.to_str()) != Some("jar") { return; }
    let Some(before) = std::fs::metadata(path).ok().and_then(|metadata| fingerprint(&metadata)) else { return; };
    if before.0 != bytes.len() as u64 { return; }
    let Ok(inspection) = inspect_archive_from_reader("mods", path, false, None, std::io::Cursor::new(bytes)) else { return; };
    let metadata = read_mod_metadata_from_reader(std::io::Cursor::new(bytes));
    if std::fs::metadata(path).ok().and_then(|metadata| fingerprint(&metadata)) != Some(before) { return; }
    if let Ok(mut cache) = AUDIT_CACHE.lock() {
        if cache.len() >= 4096 { cache.clear(); }
        cache.insert(path.to_owned(), (before, inspection));
    }
    if let Ok(mut cache) = METADATA_CACHE.lock() {
        if cache.len() >= 4096 { cache.clear(); }
        cache.insert(path.to_owned(), (before, metadata));
    }
}

fn inspect_archive_from_reader<R: Read + Seek>(folder: &str, path: &Path, deep: bool, cancel: Option<&std::sync::atomic::AtomicBool>, reader: R) -> AppResult<(Audit, HashMap<String,String>, Vec<(String,String)>)> {
    crate::core::mods::pack_download::cancelled(cancel)?;
    let mut report = Audit::default();
    let mut providers = HashMap::<String,String>::new();
    let mut dependencies = Vec::<(String,String)>::new();
    let name = path.file_name().unwrap_or_default().to_string_lossy().into_owned();
            let mut archive = match zip::ZipArchive::new(reader) { Ok(archive)=>archive, _=>{ report.errors.push(format!("Arquivo corrompido ou incompleto: {folder}/{name}"));return Ok((report, providers, dependencies));} };
            if deep {
                let mut expanded=0u64;
                for index in 0..archive.len() {
                    crate::core::mods::pack_download::cancelled(cancel)?;
                    let mut file=archive.by_index(index)?;
                    expanded=expanded.saturating_add(file.size());
                    if expanded>1024*1024*1024 { report.warnings.push(format!("{name}: validação CRC interrompida acima de 1 GiB descompactado"));break; }
                    if crate::core::mods::pack_download::copy_cancellable(&mut file,&mut std::io::sink(),cancel).is_err() {report.errors.push(format!("{name}: conteúdo interno corrompido (CRC)"));break;}
                }
            }
            if folder != "mods" { return Ok((report, providers, dependencies)); }
            if let Ok(mut metadata)=archive.by_name("fabric.mod.json") {
                let json=serde_json::from_reader::<_,serde_json::Value>(metadata.by_ref().take(2*1024*1024));
                if let Ok(json)=json {
                    if json["environment"] == "server" { return Ok((report, providers, dependencies)); }
                    if let Some(id)=json["id"].as_str() { if let Some(previous)=providers.insert(id.into(),name.clone()) {if previous != name {report.errors.push(format!("Mod duplicado {id}: {previous} e {name}"));}} }
                    if let Some(provides)=json["provides"].as_array() { for id in provides.iter().filter_map(|value| value.as_str()) {providers.entry(id.into()).or_insert_with(||name.clone());} }
                    if let Some(required)=json["depends"].as_object() { for id in required.keys() {dependencies.push((name.clone(),id.clone()));} }
                    if json["jars"].as_array().is_some_and(|jars| !jars.is_empty()) {report.warnings.push(format!("{name}: inclui dependências internas; o loader fará a validação completa"));}
                }
            };
            if let Ok(mut metadata)=archive.by_name("quilt.mod.json") {
                if let Ok(json)=serde_json::from_reader::<_,serde_json::Value>(metadata.by_ref().take(2*1024*1024)) {
                    if let Some(id)=json.pointer("/quilt_loader/id").and_then(|v|v.as_str()) {if let Some(previous)=providers.insert(id.into(),name.clone()) {if previous != name {report.errors.push(format!("Mod duplicado {id}: {previous} e {name}"));}}}
                    if let Some(required)=json.pointer("/quilt_loader/depends").and_then(|v|v.as_array()) {for dep in required {if dep["optional"]==true {continue;} if let Some(id)=dep["id"].as_str() {dependencies.push((name.clone(),id.into()));}}}
                }
            };
            for metadata_name in ["META-INF/neoforge.mods.toml","META-INF/mods.toml"] {
                if let Ok(mut metadata)=archive.by_name(metadata_name) {
                    let mut text=String::new();
                    if metadata.by_ref().take(2*1024*1024).read_to_string(&mut text).is_ok() {
                        if let Ok(json)=text.parse::<toml::Value>() {
                            if let Some(mods)=json.get("mods").and_then(|v|v.as_array()) {for module in mods {if let Some(id)=module.get("modId").and_then(|v|v.as_str()) {if let Some(previous)=providers.insert(id.into(),name.clone()) {if previous != name {report.errors.push(format!("Mod duplicado {id}: {previous} e {name}"));}}}}}
                            if let Some(deps)=json.get("dependencies").and_then(|v|v.as_table()) {for list in deps.values().filter_map(|v|v.as_array()) {for dep in list {if dep.get("side").and_then(|v|v.as_str())==Some("SERVER") {continue;} if dep.get("mandatory").and_then(|v|v.as_bool())==Some(true) || dep.get("type").and_then(|v|v.as_str())==Some("required") {if let Some(id)=dep.get("modId").and_then(|v|v.as_str()) {dependencies.push((name.clone(),id.into()));}}}}}
                        }
                    }
                    break;
                }
            }
    crate::core::mods::pack_download::cancelled(cancel)?;
    Ok((report, providers, dependencies))
}

pub fn audit_cancellable(root: &Path, deep: bool, cancel: Option<&std::sync::atomic::AtomicBool>) -> AppResult<Audit> {
    audit_with_progress(root, deep, cancel, &|_, _| {})
}

pub fn audit_with_progress(root: &Path, deep: bool, cancel: Option<&std::sync::atomic::AtomicBool>, progress: &(dyn Fn(usize, usize) + Sync)) -> AppResult<Audit> {
    crate::core::mods::pack_download::cancelled(cancel)?;
    let mut folders = vec!["mods".to_string(), "shaderpacks".to_string(), "resourcepacks".to_string()];
    if deep && root.join("saves").is_dir() {
        for world in std::fs::read_dir(root.join("saves"))?.take(128) {
            let world = world?;
            if world.file_type()?.is_dir() && !world.file_name().to_string_lossy().starts_with('.') { folders.push(format!("saves/{}/datapacks", world.file_name().to_string_lossy())); }
        }
    }
    let mut files = Vec::new();
    for folder in folders {
        let directory = root.join(&folder);
        if !directory.is_dir() { continue; }
        for entry in std::fs::read_dir(directory)? {
            crate::core::mods::pack_download::cancelled(cancel)?;
            let entry = entry?;
            let path = entry.path();
            if entry.file_type()?.is_file() && matches!(path.extension().and_then(|extension| extension.to_str()), Some("jar" | "zip")) { files.push((folder.clone(), path)); }
        }
    }
    files.sort_by(|a,b| a.1.cmp(&b.1));
    let total = files.len();
    let completed = std::sync::atomic::AtomicUsize::new(0);
    progress(0, total);
    let workers = if deep { 1 } else { std::thread::available_parallelism().map(|count| count.get()).unwrap_or(1).saturating_mul(2).min(8) };
    let progress_lock = std::sync::Mutex::new(());
    let queue = std::sync::Mutex::new(files.into_iter().enumerate());
    let parts = std::thread::scope(|scope| -> AppResult<Vec<_>> {
        let mut tasks = Vec::new();
        for _ in 0..workers {
            let queue = &queue;
            let completed = &completed;
            let progress_lock = &progress_lock;
            tasks.push(scope.spawn(move || -> AppResult<Vec<_>> {
                let mut inspected = Vec::new();
                loop {
                    crate::core::mods::pack_download::cancelled(cancel)?;
                    let next = queue.lock().map_err(|_| crate::error::AppError::Internal("Fila de validação indisponível".into()))?.next();
                    let Some((index,(folder,path))) = next else { break; };
                    inspected.push((index,inspect_archive(&folder,&path,deep,cancel)?));
                    let _progress = progress_lock.lock().map_err(|_| crate::error::AppError::Internal("Progresso da validação indisponível".into()))?;
                    let current = completed.fetch_add(1, std::sync::atomic::Ordering::AcqRel) + 1;
                    if current % 8 == 0 || current == total { progress(current, total); }
                }
                Ok(inspected)
            }));
        }
        let mut parts = Vec::new();
        for task in tasks { parts.extend(task.join().map_err(|_| crate::error::AppError::Internal("Falha na validação de arquivos".into()))??); }
        Ok(parts)
    })?;
    let mut parts = parts;
    parts.sort_by_key(|part| part.0);
    let mut report = Audit::default();
    let mut providers = HashMap::<String,String>::new();
    let mut dependencies = Vec::new();
    for (_, (part, ids, required)) in parts {
        crate::core::mods::pack_download::cancelled(cancel)?;
        report.errors.extend(part.errors); report.warnings.extend(part.warnings);
        for (id,name) in ids { if let Some(previous) = providers.insert(id.clone(),name.clone()) { if previous != name { report.errors.push(format!("Mod duplicado {id}: {previous} e {name}")); } } }
        dependencies.extend(required);
    }
    let built_in: HashSet<&str> = ["minecraft","java","fabricloader","quilt_loader","forge","neoforge"].into_iter().collect();
    for (name,id) in dependencies {
        if !providers.contains_key(&id) && !built_in.contains(id.as_str()) { report.warnings.push(format!("{name}: dependência {id} não encontrada entre os mods externos. Verifique também os JARs internos antes de instalar.")); }
    }
    Ok(report)
}

#[cfg(test)]
mod tests {
    #[test]
    fn verified_download_metadata_invalidates_when_the_file_changes() {
        let root = tempfile::tempdir().unwrap();
        std::fs::create_dir(root.path().join("mods")).unwrap();
        let path = root.path().join("mods/fixture.jar");
        mod_jar(&path, "fixture");
        let bytes = std::fs::read(&path).unwrap();
        super::remember_verified_mod_archive(&path, &bytes);
        assert!(super::AUDIT_CACHE.lock().unwrap().contains_key(&path));
        assert!(super::audit(root.path(), false).unwrap().errors.is_empty());
        assert!(super::mod_metadata(&path).unwrap().ids.contains("fixture"));
        let modified = std::fs::metadata(&path).unwrap().modified().unwrap();
        mod_jar(&path, "changed");
        assert_eq!(std::fs::metadata(&path).unwrap().len(), bytes.len() as u64);
        std::fs::File::options().write(true).open(&path).unwrap()
            .set_times(std::fs::FileTimes::new().set_modified(modified + std::time::Duration::from_secs(1))).unwrap();
        assert!(super::mod_metadata(&path).unwrap().ids.contains("changed"));
        std::fs::write(&path, b"corrupted").unwrap();
        assert!(super::audit(root.path(), false).unwrap().errors.iter().any(|error| error.contains("corrompido")));
        assert!(super::mod_metadata(&path).is_none());
    }

    #[test]
    fn cached_downloads_still_receive_deep_crc_validation() {
        use std::io::Write;
        let root = tempfile::tempdir().unwrap();
        std::fs::create_dir(root.path().join("mods")).unwrap();
        let path = root.path().join("mods/fixture.jar");
        let mut archive = zip::ZipWriter::new(std::io::Cursor::new(Vec::new()));
        let options = zip::write::FileOptions::default().compression_method(zip::CompressionMethod::Stored);
        archive.start_file("fabric.mod.json", options).unwrap();
        archive.write_all(br#"{"id":"fixture","version":"1"}"#).unwrap();
        archive.start_file("payload.bin", options).unwrap();
        archive.write_all(&[b'x'; 1024]).unwrap();
        let mut bytes = archive.finish().unwrap().into_inner();
        let payload = bytes.windows(64).position(|window| window.iter().all(|byte| *byte == b'x')).unwrap();
        bytes[payload] = b'y';
        std::fs::write(&path, &bytes).unwrap();
        super::remember_verified_mod_archive(&path, &bytes);
        assert!(super::audit(root.path(), false).unwrap().errors.is_empty());
        assert!(super::audit(root.path(), true).unwrap().errors.iter().any(|error| error.contains("CRC")));
    }

    #[test]
    fn archive_seeks_preserve_buffer_and_match_file_positions() {
        use std::io::{Read, Seek, SeekFrom};
        let root = tempfile::tempdir().unwrap();
        let path = root.path().join("fixture.jar");
        let bytes = (0..80_000).map(|index| (index % 251) as u8).collect::<Vec<_>>();
        std::fs::write(&path, &bytes).unwrap();
        let mut reader = super::archive_reader(&path, None).unwrap();
        let mut sample = [0; 8];
        reader.read_exact(&mut sample).unwrap();
        assert_eq!(sample, bytes[..8]);
        let buffered = reader.reader.buffer().len();
        assert!(buffered > 32_000);
        assert_eq!(reader.seek(SeekFrom::Current(0)).unwrap(), 8);
        assert_eq!(reader.reader.buffer().len(), buffered);
        assert_eq!(reader.seek(SeekFrom::Start(17)).unwrap(), 17);
        assert_eq!(reader.reader.buffer().len(), buffered - 9);
        reader.read_exact(&mut sample).unwrap();
        assert_eq!(sample, bytes[17..25]);
        assert_eq!(reader.seek(SeekFrom::Current(-20)).unwrap(), 5);
        reader.read_exact(&mut sample).unwrap();
        assert_eq!(sample, bytes[5..13]);
        assert_eq!(reader.seek(SeekFrom::End(-8)).unwrap(), 79_992);
        reader.read_exact(&mut sample).unwrap();
        assert_eq!(sample, bytes[79_992..]);
        assert_eq!(reader.read(&mut sample).unwrap(), 0);
        reader.seek(SeekFrom::Start(0)).unwrap();
        assert!(reader.seek(SeekFrom::Current(-1)).is_err());
        assert_eq!(reader.stream_position().unwrap(), 0);
        reader.unchanged().unwrap();
    }

    #[test]
    fn metadata_audit_skips_payload_but_deep_audit_checks_its_crc() {
        use std::io::{Read, Seek, Write};
        struct Meter {
            inner: std::io::Cursor<Vec<u8>>,
            bytes: std::rc::Rc<std::cell::Cell<usize>>,
        }
        impl Read for Meter {
            fn read(&mut self, buffer: &mut [u8]) -> std::io::Result<usize> {
                let count = self.inner.read(buffer)?;
                self.bytes.set(self.bytes.get() + count);
                Ok(count)
            }
        }
        impl Seek for Meter {
            fn seek(&mut self, position: std::io::SeekFrom) -> std::io::Result<u64> { self.inner.seek(position) }
        }
        let mut archive = zip::ZipWriter::new(std::io::Cursor::new(Vec::new()));
        let options = zip::write::FileOptions::default().compression_method(zip::CompressionMethod::Stored);
        archive.start_file("fabric.mod.json", options).unwrap();
        archive.write_all(br#"{"id":"fixture","version":"1"}"#).unwrap();
        archive.start_file("assets/fixture/large.bin", options).unwrap();
        archive.write_all(&vec![b'x'; 8 * 1024 * 1024]).unwrap();
        let mut bytes = archive.finish().unwrap().into_inner();
        let payload = bytes.windows(64).position(|window| window.iter().all(|byte| *byte == b'x')).unwrap();
        bytes[payload + 32] = b'y';
        let read = std::rc::Rc::new(std::cell::Cell::new(0));
        let reader = Meter { inner: std::io::Cursor::new(bytes.clone()), bytes: read.clone() };
        let (audit, ids, _) = super::inspect_archive_from_reader("mods", std::path::Path::new("fixture.jar"), false, None, reader).unwrap();
        assert!(audit.errors.is_empty());
        assert!(ids.contains_key("fixture"));
        assert!(read.get() < 128 * 1024, "Metadata read {} bytes", read.get());
        let reader = Meter { inner: std::io::Cursor::new(bytes), bytes: read.clone() };
        let (audit, _, _) = super::inspect_archive_from_reader("mods", std::path::Path::new("fixture.jar"), true, None, reader).unwrap();
        assert!(audit.errors.iter().any(|error| error.contains("CRC")));
        assert!(read.get() >= 8 * 1024 * 1024);
    }

    #[test]
    fn sparse_reader_rejects_modification_and_cancellation() {
        use std::io::Read;
        let root = tempfile::tempdir().unwrap();
        let path = root.path().join("fixture.jar");
        mod_jar(&path, "fixture");
        let cancel = std::sync::atomic::AtomicBool::new(false);
        let mut reader = super::archive_reader(&path, Some(&cancel)).unwrap();
        assert!(reader.unchanged().is_ok());
        std::fs::OpenOptions::new().write(true).open(&path).unwrap().set_len(2048).unwrap();
        assert!(reader.unchanged().is_err());
        cancel.store(true, std::sync::atomic::Ordering::SeqCst);
        assert!(reader.read(&mut [0; 10]).is_err());
        assert!(reader.unchanged().is_err());
    }

    #[test]
    #[ignore]
    fn benchmark_installed_archives_without_modifying_them() {
        let path = std::path::PathBuf::from(std::env::var("LUXMC_VALIDATION_BENCH_DIR").expect("Set LUXMC_VALIDATION_BENCH_DIR to an installed instance"));
        let mut files = std::fs::read_dir(path.join("mods")).unwrap().filter_map(Result::ok).map(|entry| entry.path()).filter(|path| path.extension().is_some_and(|value| value == "jar")).collect::<Vec<_>>();
        files.sort();
        let before = files.iter().map(|path| (path.clone(), std::fs::metadata(path).unwrap().len(), std::fs::metadata(path).unwrap().modified().unwrap())).collect::<Vec<_>>();
        let legacy_started = std::time::Instant::now();
        for file in &files { let _ = zip::ZipArchive::new(std::io::BufReader::new(std::fs::File::open(file).unwrap())); }
        let legacy = legacy_started.elapsed();
        let started = std::time::Instant::now();
        let report = super::audit(&path, false).unwrap();
        let after = files.iter().map(|path| (path.clone(), std::fs::metadata(path).unwrap().len(), std::fs::metadata(path).unwrap().modified().unwrap())).collect::<Vec<_>>();
        assert_eq!(before, after);
        println!("archives={} previous_central_directory_seconds={:.3} current_complete_audit_seconds={:.3} errors={} warnings={}", files.len(), legacy.as_secs_f64(), started.elapsed().as_secs_f64(), report.errors.len(), report.warnings.len());
    }
    fn mod_jar(path: &std::path::Path, id: &str) {
        use std::io::Write;
        let mut archive = zip::ZipWriter::new(std::fs::File::create(path).unwrap());
        archive.start_file("fabric.mod.json", zip::write::FileOptions::default()).unwrap();
        archive.write_all(serde_json::json!({"id":id,"version":"1"}).to_string().as_bytes()).unwrap();
        archive.finish().unwrap();
    }
    #[test]
    fn duplicate_repair_keeps_pinned_file_and_a_recoverable_copy() {
        let root=std::env::temp_dir().join(format!("luxmc-dedupe-{}",uuid::Uuid::new_v4()));
        std::fs::create_dir_all(root.join("mods")).unwrap();
        mod_jar(&root.join("mods/a-old.jar"), "example");
        mod_jar(&root.join("mods/pinned.jar"), "example");
        mod_jar(&root.join("mods/independent.jar"), "independent");
        let preferred = ["pinned.jar".to_owned()].into_iter().collect();
        assert_eq!(super::repair_duplicates(&root,&preferred).unwrap(),1);
        assert!(root.join("mods/pinned.jar").is_file());
        assert!(root.join("mods/independent.jar").is_file());
        assert!(!root.join("mods/a-old.jar").exists());
        let backup=std::fs::read_dir(root.join(".luxmc/removed-duplicates")).unwrap().next().unwrap().unwrap().path();
        assert!(backup.join("a-old.jar").is_file());
        assert!(super::audit(&root,false).unwrap().errors.is_empty());
        assert_eq!(super::repair_duplicates(&root,&preferred).unwrap(),0);
        std::fs::remove_dir_all(root).unwrap();
    }
    #[test]
    fn neoforge_duplicate_repair_keeps_manifest_version() {
        use std::io::Write;
        let root=std::env::temp_dir().join(format!("luxmc-neoforge-dedupe-{}",uuid::Uuid::new_v4()));
        std::fs::create_dir_all(root.join("mods")).unwrap();
        for (name,version) in [("old.jar","1"),("manifest.jar","2")] {
            let mut archive=zip::ZipWriter::new(std::fs::File::create(root.join("mods").join(name)).unwrap());
            archive.start_file("META-INF/neoforge.mods.toml",zip::write::FileOptions::default()).unwrap();
            archive.write_all(format!("[[mods]]\nmodId='example'\nversion='{version}'\n").as_bytes()).unwrap();
            archive.finish().unwrap();
        }
        assert_eq!(super::repair_duplicates(&root,&["manifest.jar".to_owned()].into_iter().collect()).unwrap(),1);
        assert!(root.join("mods/manifest.jar").is_file());
        assert!(!root.join("mods/old.jar").exists());
        assert!(super::audit(&root,false).unwrap().errors.is_empty());
        std::fs::remove_dir_all(root).unwrap();
    }
    #[test]
    fn pack_duplicates_with_two_provider_projects_keep_the_newer_declared_release() {
        let root = tempfile::tempdir().unwrap();
        std::fs::create_dir(root.path().join("mods")).unwrap();
        for (name, version) in [("old.jar", "1.113.1"), ("new.jar", "1.120.2")] {
            let mut archive = zip::ZipWriter::new(std::fs::File::create(root.path().join("mods").join(name)).unwrap());
            archive.start_file("META-INF/neoforge.mods.toml", zip::write::FileOptions::default()).unwrap();
            use std::io::Write;
            archive.write_all(format!("[[mods]]\nmodId='computercraft'\nversion='{version}'\n").as_bytes()).unwrap();
            archive.finish().unwrap();
        }
        let pinned: std::collections::HashSet<String> = ["old.jar".to_owned(), "new.jar".to_owned()].into_iter().collect();
        let mut preferred = pinned.clone();
        super::select_pack_duplicate_versions(root.path(), &pinned, &mut preferred);
        assert_eq!(super::repair_duplicates(root.path(), &preferred).unwrap(), 1);
        assert!(root.path().join("mods/new.jar").is_file());
        assert!(!root.path().join("mods/old.jar").exists());
        assert!(super::audit(root.path(), false).unwrap().errors.is_empty());
    }

    #[test]
    fn duplicate_repair_preserves_ambiguous_versions_and_extra_modules() {
        use std::io::Write;
        let root=std::env::temp_dir().join(format!("luxmc-dedupe-safe-{}",uuid::Uuid::new_v4()));
        std::fs::create_dir_all(root.join("mods")).unwrap();
        mod_jar(&root.join("mods/a.jar"), "example");
        let mut archive=zip::ZipWriter::new(std::fs::File::create(root.join("mods/b.jar")).unwrap());
        archive.start_file("fabric.mod.json",zip::write::FileOptions::default()).unwrap();
        archive.write_all(br#"{"id":"example","version":"2"}"#).unwrap();
        archive.finish().unwrap();
        let mut archive=zip::ZipWriter::new(std::fs::File::create(root.join("mods/multiple.jar")).unwrap());
        archive.start_file("META-INF/neoforge.mods.toml",zip::write::FileOptions::default()).unwrap();
        archive.write_all(b"[[mods]]\nmodId='example'\n[[mods]]\nmodId='extra'\n").unwrap();
        archive.finish().unwrap();
        assert_eq!(super::repair_duplicates(&root,&Default::default()).unwrap(),0);
        assert_eq!(super::repair_duplicates(&root,&["a.jar".to_owned(),"b.jar".to_owned()].into_iter().collect()).unwrap(),0);
        assert!(root.join("mods/a.jar").is_file());
        assert!(root.join("mods/b.jar").is_file());
        assert!(root.join("mods/multiple.jar").is_file());
        std::fs::remove_dir_all(root).unwrap();
    }
    #[test]
    fn pack_repair_resolves_forge_manifest_versions_and_wrong_loader_duplicates() {
        use std::io::Write;
        let root = tempfile::tempdir().unwrap();
        std::fs::create_dir(root.path().join("mods")).unwrap();
        for (name,id,version,loader) in [("fabric-gecko.jar","geckolib3","3.0.42","fabric"),("forge-gecko.jar","geckolib3","3.1.40","forge"),("old-cave.jar","cave_dweller","1.5","forge"),("new-cave.jar","cave_dweller","1.6.4","forge")] {
            let mut archive = zip::ZipWriter::new(std::fs::File::create(root.path().join("mods").join(name)).unwrap());
            if loader=="fabric" {
                archive.start_file("fabric.mod.json",zip::write::FileOptions::default()).unwrap();
                archive.write_all(serde_json::json!({"id":id,"version":version}).to_string().as_bytes()).unwrap();
            } else {
                archive.start_file("META-INF/mods.toml",zip::write::FileOptions::default()).unwrap();
                archive.write_all(format!("[[mods]]\nmodId='{id}'\nversion='${{file.jarVersion}}'\n").as_bytes()).unwrap();
                archive.start_file("META-INF/MANIFEST.MF",zip::write::FileOptions::default()).unwrap();
                archive.write_all(format!("Manifest-Version: 1.0\r\nImplementation-Version: {version}\r\n\r\n").as_bytes()).unwrap();
            }
            archive.finish().unwrap();
        }
        let pinned: std::collections::HashSet<_> = ["fabric-gecko.jar","forge-gecko.jar","old-cave.jar","new-cave.jar"].into_iter().map(str::to_owned).collect();
        let mut preferred = pinned.clone();
        super::select_pack_duplicate_versions_for_loader(root.path(),&pinned,&mut preferred,"forge");
        assert_eq!(preferred,["forge-gecko.jar","new-cave.jar"].into_iter().map(str::to_owned).collect());
        assert_eq!(super::repair_duplicates(root.path(),&preferred).unwrap(),2);
        assert!(super::audit(root.path(),false).unwrap().errors.is_empty());
        let backup = std::fs::read_dir(root.path().join(".luxmc/removed-duplicates")).unwrap().next().unwrap().unwrap().path();
        assert!(backup.join("fabric-gecko.jar").is_file());
        assert!(backup.join("old-cave.jar").is_file());
    }
    #[test]
    fn damaged_archive_is_reported_without_deleting_it() {
        let root=std::env::temp_dir().join(format!("luxmc-audit-{}",uuid::Uuid::new_v4()));
        std::fs::create_dir_all(root.join("mods")).unwrap();
        std::fs::write(root.join("mods/broken.jar"),b"broken").unwrap();
        let result=super::audit(&root,true).unwrap();
        assert_eq!(result.errors.len(),1); assert!(root.join("mods/broken.jar").exists());
        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn parallel_audit_keeps_cross_file_dependencies_and_detects_duplicates() {
        let root = std::env::temp_dir().join(format!("luxmc-parallel-audit-{}",uuid::Uuid::new_v4()));
        std::fs::create_dir_all(root.join("mods")).unwrap();
        for index in 0..128 { mod_jar(&root.join("mods").join(format!("mod-{index}.jar")), &format!("module_{index}")); }
        mod_jar(&root.join("mods/duplicate.jar"), "module_57");
        let report = super::audit(&root,false).unwrap();
        assert_eq!(report.errors.len(),1);
        assert!(report.errors[0].contains("module_57"));
        assert!(report.warnings.iter().all(|warning| !warning.contains("não encontrada")));
        let cancel = std::sync::atomic::AtomicBool::new(true);
        assert!(super::audit_cancellable(&root,false,Some(&cancel)).is_err());
        assert!(super::repair_duplicates_cancellable(&root,&Default::default(),Some(&cancel)).is_err());
        assert_eq!(std::fs::read_dir(root.join("mods")).unwrap().count(),129);
        std::fs::remove_dir_all(root).unwrap();
    }
}
