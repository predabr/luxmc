use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;
use std::path::{Component, Path, PathBuf};

use crate::error::{AppError, AppResult};

const SCHEMA: u32 = 1;

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
struct FileStamp {
    path: PathBuf,
    length: u64,
    modified_ns: u128,
}

#[derive(Serialize, Deserialize)]
struct ExtractionState {
    schema: u32,
    architecture: String,
    archives: Vec<FileStamp>,
    outputs: Vec<FileStamp>,
}

pub(super) struct Inputs(Vec<FileStamp>);

fn stamp(path: &Path, stored_path: PathBuf) -> Option<FileStamp> {
    let metadata = std::fs::metadata(path).ok()?;
    if !metadata.is_file() {
        return None;
    }
    Some(FileStamp {
        path: stored_path,
        length: metadata.len(),
        modified_ns: metadata
            .modified()
            .ok()?
            .duration_since(std::time::UNIX_EPOCH)
            .ok()?
            .as_nanos(),
    })
}

fn archive_stamps(archives: &[PathBuf]) -> Option<Vec<FileStamp>> {
    archives
        .iter()
        .map(|path| stamp(path, path.clone()))
        .collect()
}

pub(super) fn capture(archives: &[PathBuf]) -> Option<Inputs> {
    archive_stamps(archives).map(Inputs)
}

fn state_path(directory: &Path) -> PathBuf {
    directory.join(".luxmc-native-cache.json")
}

pub(super) fn is_valid(directory: &Path, archives: &[PathBuf]) -> bool {
    let Some(current_archives) = archive_stamps(archives) else {
        return false;
    };
    let Ok(bytes) = std::fs::read(state_path(directory)) else {
        return false;
    };
    let Ok(state) = serde_json::from_slice::<ExtractionState>(&bytes) else {
        return false;
    };
    if state.schema != SCHEMA
        || state.architecture != std::env::consts::ARCH
        || state.archives != current_archives
        || (!archives.is_empty() && state.outputs.is_empty())
    {
        return false;
    }
    state.outputs.iter().all(|output| {
        output.path.components().count() == 1
            && matches!(output.path.components().next(), Some(Component::Normal(_)))
            && stamp(&directory.join(&output.path), output.path.clone()).as_ref() == Some(output)
    })
}

pub(super) fn store(
    directory: &Path,
    archives: &[PathBuf],
    output_names: &BTreeSet<PathBuf>,
    original_inputs: &Inputs,
) -> AppResult<()> {
    let archive_stamps = archive_stamps(archives)
        .ok_or_else(|| AppError::InvalidState("Native archive changed during extraction".into()))?;
    if archive_stamps != original_inputs.0 {
        return Err(AppError::InvalidState("Native archive changed during extraction".into()));
    }
    let outputs = output_names
        .iter()
        .map(|name| stamp(&directory.join(name), name.clone()))
        .collect::<Option<Vec<_>>>()
        .ok_or_else(|| AppError::InvalidState("Native library changed during extraction".into()))?;
    let state = ExtractionState {
        schema: SCHEMA,
        architecture: std::env::consts::ARCH.to_string(),
        archives: archive_stamps,
        outputs,
    };
    let path = state_path(directory);
    let temporary = directory.join(format!(".luxmc-native-{}.part", uuid::Uuid::new_v4()));
    let result = (|| -> AppResult<()> {
        std::fs::write(&temporary, serde_json::to_vec(&state)?)?;
        std::fs::rename(&temporary, path)?;
        Ok(())
    })();
    if result.is_err() {
        let _ = std::fs::remove_file(temporary);
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fixture() -> (PathBuf, Vec<PathBuf>, BTreeSet<PathBuf>) {
        let root = std::env::temp_dir().join(format!("luxmc-native-state-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&root).unwrap();
        let archive = root.join("native.jar");
        std::fs::write(&archive, b"archive").unwrap();
        std::fs::write(root.join("lwjgl.dll"), b"library").unwrap();
        (root, vec![archive], BTreeSet::from([PathBuf::from("lwjgl.dll")]))
    }

    fn clean(root: PathBuf) {
        let absolute = root.canonicalize().unwrap();
        assert!(absolute.starts_with(std::env::temp_dir().canonicalize().unwrap()));
        std::fs::remove_dir_all(absolute).unwrap();
    }

    #[test]
    fn unchanged_inputs_and_outputs_reuse_the_extraction() {
        let (root, archives, outputs) = fixture();
        assert!(!is_valid(&root, &archives));
        store(&root, &archives, &outputs, &capture(&archives).unwrap()).unwrap();
        assert!(is_valid(&root, &archives));
        std::fs::write(root.join("unrelated.txt"), b"preferences").unwrap();
        assert!(is_valid(&root, &archives));
        clean(root);
    }

    #[test]
    fn modified_or_missing_libraries_and_archives_invalidate_extraction() {
        let (root, archives, outputs) = fixture();
        store(&root, &archives, &outputs, &capture(&archives).unwrap()).unwrap();
        std::fs::write(root.join("lwjgl.dll"), b"damaged library").unwrap();
        assert!(!is_valid(&root, &archives));
        store(&root, &archives, &outputs, &capture(&archives).unwrap()).unwrap();
        std::fs::remove_file(root.join("lwjgl.dll")).unwrap();
        assert!(!is_valid(&root, &archives));
        std::fs::write(root.join("lwjgl.dll"), b"library").unwrap();
        store(&root, &archives, &outputs, &capture(&archives).unwrap()).unwrap();
        std::fs::write(&archives[0], b"updated archive").unwrap();
        assert!(!is_valid(&root, &archives));
        store(&root, &archives, &outputs, &capture(&archives).unwrap()).unwrap();
        assert!(!is_valid(&root, &[]));
        std::fs::remove_file(&archives[0]).unwrap();
        assert!(!is_valid(&root, &archives));
        clean(root);
    }

    #[test]
    fn changed_archive_during_extraction_cannot_be_marked_as_ready() {
        let (root, archives, outputs) = fixture();
        let original_inputs = capture(&archives).unwrap();
        std::fs::write(&archives[0], b"different archive").unwrap();
        assert!(store(&root, &archives, &outputs, &original_inputs).is_err());
        assert!(!is_valid(&root, &archives));
        clean(root);
    }

    #[test]
    fn malformed_state_and_unsafe_output_paths_force_extraction() {
        let (root, archives, outputs) = fixture();
        std::fs::write(state_path(&root), b"not json").unwrap();
        assert!(!is_valid(&root, &archives));
        store(&root, &archives, &outputs, &capture(&archives).unwrap()).unwrap();
        let mut state: ExtractionState = serde_json::from_slice(&std::fs::read(state_path(&root)).unwrap()).unwrap();
        state.outputs[0].path = PathBuf::from("../lwjgl.dll");
        std::fs::write(state_path(&root), serde_json::to_vec(&state).unwrap()).unwrap();
        assert!(!is_valid(&root, &archives));
        clean(root);
    }

    #[cfg(target_os = "windows")]
    #[tokio::test(flavor = "multi_thread")]
    async fn windows_warm_launch_preserves_dlls_and_restores_deleted_library() {
        use std::io::Write;

        let root = std::env::temp_dir().join(format!("luxmc-native-extract-{}", uuid::Uuid::new_v4()));
        let library = root.join("libraries/fixture/natives/1/natives-1-natives-windows.jar");
        std::fs::create_dir_all(library.parent().unwrap()).unwrap();
        let mut writer = zip::ZipWriter::new(std::fs::File::create(&library).unwrap());
        for index in 0..24 {
            writer.start_file(format!("library-{index}.dll"), zip::write::FileOptions::default()).unwrap();
            writer.write_all(&vec![index as u8; 256 * 1024]).unwrap();
        }
        writer.finish().unwrap();
        let detail: crate::core::minecraft::VersionDetail = serde_json::from_value(serde_json::json!({
            "id": "fixture",
            "type": "release",
            "libraries": [{"name": "fixture:natives:1", "natives": {"windows": "natives-windows"}}]
        })).unwrap();
        let manager = crate::core::downloader::DownloadManager::new(reqwest::Client::new(), root.clone());
        let cold_started = std::time::Instant::now();
        let natives = super::super::extract_natives(&manager, &detail, None).await.unwrap();
        let cold_ms = cold_started.elapsed().as_secs_f64() * 1000.0;
        let output = natives.join("library-0.dll");
        let original_modified = std::fs::metadata(&output).unwrap().modified().unwrap();
        let warm_started = std::time::Instant::now();
        super::super::extract_natives(&manager, &detail, None).await.unwrap();
        let warm_ms = warm_started.elapsed().as_secs_f64() * 1000.0;
        assert_eq!(std::fs::metadata(&output).unwrap().modified().unwrap(), original_modified);
        assert!(is_valid(&natives, &[library.clone()]));
        std::fs::remove_file(&output).unwrap();
        super::super::extract_natives(&manager, &detail, None).await.unwrap();
        assert_eq!(std::fs::metadata(&output).unwrap().len(), 256 * 1024);
        assert!(is_valid(&natives, &[library]));
        println!("Native preparation fixture: 24 DLLs / 6 MiB; cold {cold_ms:.2} ms, warm {warm_ms:.2} ms");
        clean(root);
    }
}
