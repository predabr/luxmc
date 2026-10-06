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

fn normalize_list(value: &str) -> String {
    std::env::join_paths(normalize(std::env::split_paths(value).collect()))
        .map(|paths| paths.to_string_lossy().into_owned())
        .unwrap_or_else(|_| value.to_owned())
}

pub fn normalize_jvm_paths(args: &mut [String]) {
    let mut next_is_path = false;
    for arg in args {
        if next_is_path {
            *arg = normalize_list(arg);
            next_is_path = false;
        } else if arg == "-p" || arg == "--module-path" {
            next_is_path = true;
        } else if let Some(value) = arg.strip_prefix("--module-path=") {
            *arg = format!("--module-path={}", normalize_list(value));
        } else if let Some(value) = arg.strip_prefix("-DlibraryDirectory=") {
            if let Some(path) = normalize(vec![PathBuf::from(value)]).pop() {
                *arg = format!("-DlibraryDirectory={}", path.display());
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn module_path_and_classpath_use_the_same_physical_libraries() {
        let fixture = std::env::temp_dir().join(format!("luxmc-modules-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(fixture.join("inner")).unwrap();
        let library = fixture.join("bootstraplauncher.jar");
        std::fs::write(&library, b"fixture").unwrap();
        let alias = fixture.join("inner/../bootstraplauncher.jar");
        let paths = std::env::join_paths([alias.clone(), library.clone()]).unwrap().to_string_lossy().into_owned();
        let mut args = vec!["-p".into(), paths.clone(), format!("--module-path={paths}"), format!("-DlibraryDirectory={}", fixture.join("inner/..").display()), "--add-modules".into(), "ALL-MODULE-PATH".into()];
        normalize_jvm_paths(&mut args);
        let expected = normalize(vec![library.clone()])[0].clone();
        assert_eq!(std::env::split_paths(&args[1]).collect::<Vec<_>>(), vec![expected]);
        assert_eq!(args[2], format!("--module-path={}", args[1]));
        assert_eq!(args[3], format!("-DlibraryDirectory={}", normalize(vec![fixture.clone()])[0].display()));
        assert_eq!(&args[4..], &["--add-modules", "ALL-MODULE-PATH"]);
        std::fs::remove_file(library).unwrap();
        std::fs::remove_dir(fixture.join("inner")).unwrap();
        std::fs::remove_dir(fixture).unwrap();
    }
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
