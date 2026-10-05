use std::{collections::HashSet, path::PathBuf};

#[cfg(windows)]
fn java_path(path: PathBuf) -> PathBuf {
    let value = path.to_string_lossy();
    if let Some(rest) = value.strip_prefix(r"\\?\UNC\") {
        PathBuf::from(format!(r"\\{}", rest))
    } else if let Some(rest) = value.strip_prefix(r"\\?\") {
        PathBuf::from(rest)
    } else {
        path
    }
}

pub fn normalize(entries: Vec<PathBuf>) -> Vec<PathBuf> {
    let mut seen = HashSet::new();
    entries.into_iter().filter_map(|entry| {
        let path = entry.canonicalize().unwrap_or(entry);
        #[cfg(windows)]
        let path = java_path(path);
        seen.insert(path.clone()).then_some(path)
    }).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn library_aliases_share_one_physical_classpath_entry() {
        let fixture = std::env::temp_dir().join(format!("luxmc-classpath-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(fixture.join("inner")).unwrap();
        let library = fixture.join("library.jar");
        std::fs::write(&library, b"fixture").unwrap();
        let normalized = normalize(vec![fixture.join("inner/../library.jar"), library.clone()]);
        assert_eq!(normalized.len(), 1);
        assert!(normalized[0].is_absolute());
        assert_eq!(std::fs::read(&normalized[0]).unwrap(), b"fixture");
        assert!(!normalized[0].to_string_lossy().starts_with(r"\\?\"));
        std::fs::remove_file(library).unwrap();
        std::fs::remove_dir(fixture.join("inner")).unwrap();
        std::fs::remove_dir(fixture).unwrap();
    }
    #[cfg(windows)]
    #[test]
    fn extended_drive_and_unc_paths_remain_usable_by_java() {
        assert_eq!(java_path(PathBuf::from(r"\\?\C:\Game Data\library.jar")), PathBuf::from(r"C:\Game Data\library.jar"));
        assert_eq!(java_path(PathBuf::from(r"\\?\UNC\server\share\library.jar")), PathBuf::from(r"\\server\share\library.jar"));
    }
    #[test]
    fn missing_entries_are_preserved_for_normal_error_reporting() {
        let missing = PathBuf::from("missing-classpath-entry.jar");
        assert_eq!(normalize(vec![missing.clone()]), vec![missing]);
    }
}
