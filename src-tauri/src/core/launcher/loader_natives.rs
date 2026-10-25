use std::{collections::BTreeSet, io::Read, path::{Path, PathBuf}, sync::{LazyLock, Mutex}};
use crate::error::{AppError, AppResult};

static EXTRACTION: LazyLock<Mutex<()>> = LazyLock::new(|| Mutex::new(()));

fn native_name(path: &Path) -> bool {
    path.extension().and_then(|extension| extension.to_str()).is_some_and(|extension| match std::env::consts::OS {
        "windows" => extension.eq_ignore_ascii_case("dll"),
        "macos" => matches!(extension, "dylib" | "jnilib"),
        _ => extension == "so",
    })
}

pub(super) fn prepare(base: &Path, archives: &[PathBuf]) -> AppResult<PathBuf> {
    let _guard = EXTRACTION.lock().map_err(|error| AppError::Internal(error.to_string()))?;
    let mut originals = std::fs::read_dir(base)?.filter_map(Result::ok).filter(|entry| entry.path().is_file() && native_name(&entry.path())).map(|entry| entry.path()).collect::<Vec<_>>();
    originals.sort();
    let mut inputs = originals.clone();
    inputs.extend_from_slice(archives);
    let key = format!("{:016x}", xxhash_rust::xxh3::xxh3_64(serde_json::to_string(&inputs)?.as_bytes()));
    let target = base.join(".luxmc-loaders").join(key);
    if super::native_cache::is_valid(&target, &inputs) { return Ok(target); }
    let captured = super::native_cache::capture(&inputs).ok_or_else(|| AppError::InvalidState("Bibliotecas nativas do loader incompletas".into()))?;
    std::fs::create_dir_all(&target)?;
    let mut outputs = BTreeSet::new();
    for source in originals {
        let name = source.file_name().ok_or_else(|| AppError::InvalidInput("Biblioteca nativa sem nome".into()))?;
        std::fs::copy(&source, target.join(name))?;
        outputs.insert(PathBuf::from(name));
    }
    let mut extracted = 0usize;
    let mut total = 0u64;
    for archive in archives {
        let mut zip = zip::ZipArchive::new(std::io::BufReader::new(std::fs::File::open(archive)?))?;
        for index in 0..zip.len() {
            let mut file = zip.by_index(index)?;
            let Some(path) = file.enclosed_name().map(|path| path.to_path_buf()) else { continue; };
            if file.is_dir() || !native_name(&path) { continue; }
            if file.size() > 32 * 1024 * 1024 || extracted >= 256 || total + file.size() > 128 * 1024 * 1024 {
                return Err(AppError::InvalidInput("Bibliotecas nativas do loader excedem o limite".into()));
            }
            let Some(name) = path.file_name() else { continue; };
            let mut bytes = Vec::new();
            file.by_ref().take(32 * 1024 * 1024 + 1).read_to_end(&mut bytes)?;
            if bytes.len() > 32 * 1024 * 1024 { return Err(AppError::InvalidInput("Biblioteca nativa excede o limite".into())); }
            std::fs::write(target.join(name), &bytes)?;
            outputs.insert(PathBuf::from(name));
            total += bytes.len() as u64;
            extracted += 1;
        }
    }
    if extracted == 0 { return Err(AppError::InvalidState("O loader não contém bibliotecas nativas para este sistema".into())); }
    super::native_cache::store(&target, &inputs, &outputs, &captured)?;
    Ok(target)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    #[test]
    fn loader_overrides_are_isolated_cached_and_rebuilt_when_deleted() {
        let root = tempfile::tempdir().unwrap();
        let base = root.path().join("natives");
        std::fs::create_dir_all(&base).unwrap();
        let extension = if cfg!(windows) { "dll" } else if cfg!(target_os="macos") { "dylib" } else { "so" };
        let name = format!("lwjgl.{extension}");
        std::fs::write(base.join(&name), b"minecraft").unwrap();
        let archive = root.path().join("loader-native.jar");
        let mut zip = zip::ZipWriter::new(std::fs::File::create(&archive).unwrap());
        zip.start_file(&name, zip::write::FileOptions::default()).unwrap();
        zip.write_all(b"loader-native").unwrap();
        zip.start_file(format!("../escape.{extension}"), zip::write::FileOptions::default()).unwrap();
        zip.write_all(b"invalid").unwrap();
        zip.finish().unwrap();
        let archives = vec![archive];
        let target = prepare(&base, &archives).unwrap();
        assert_eq!(std::fs::read(base.join(&name)).unwrap(), b"minecraft");
        assert_eq!(std::fs::read(target.join(&name)).unwrap(), b"loader-native");
        assert!(!target.parent().unwrap().join(format!("escape.{extension}")).exists());
        let modified = std::fs::metadata(target.join(&name)).unwrap().modified().unwrap();
        assert_eq!(prepare(&base, &archives).unwrap(), target);
        assert_eq!(std::fs::metadata(target.join(&name)).unwrap().modified().unwrap(), modified);
        std::fs::remove_file(target.join(&name)).unwrap();
        assert_eq!(prepare(&base, &archives).unwrap(), target);
        assert_eq!(std::fs::read(target.join(&name)).unwrap(), b"loader-native");
    }
}
