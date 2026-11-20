use super::*;
use std::io::Write;

fn jar(path: &Path, name: &str, bytes: &[u8]) {
    let mut writer = zip::ZipWriter::new(std::fs::File::create(path).unwrap());
    writer.start_file(name, zip::write::FileOptions::default().compression_method(zip::CompressionMethod::Stored)).unwrap();
    writer.write_all(bytes).unwrap();
    writer.finish().unwrap();
}

#[test]
fn clean_archives_and_absent_mods_pass() {
    let root = tempfile::tempdir().unwrap();
    assert!(scan_mods_directory(&root.path().join("missing")).is_clean);
    jar(&root.path().join("normal.JAR"), "Example.class", b"benign test data");
    let result = scan_mods_directory(root.path());
    assert!(result.is_clean);
    assert_eq!(result.total_scanned, 1);
    assert!(enforce_scan(&result).is_ok());
    assert!(!result.purged);
    assert!(result.instance_name.is_none());
    assert!(!super::super::has_critical_threat(&result));
    let serialized = serde_json::to_string(&result).unwrap();
    assert!(serialized.contains("\"purged\":false"));
}

#[test]
#[test]
fn both_webhook_hosts_are_detected_in_large_classes() {
    let root = tempfile::tempdir().unwrap();
    let token = b"AAAABBBBCCCCDDDDEEEEFFFFGGGGHHHHIIIIJJJJKKKKLLLL";
    for (index, host) in [b"discord.com/api/webhooks".as_slice(), b"discordapp.com/api/webhooks".as_slice()].iter().enumerate() {
        let mut bytes = vec![b'x'; 110_000];
        bytes.extend_from_slice(host);
        bytes.extend_from_slice(b"/123456789012345678/");
        bytes.extend_from_slice(token);
        jar(&root.path().join(format!("webhook{index}.jar")), "Example.class", &bytes);
    }
    let result = scan_mods_directory(root.path());
    assert_eq!(result.threats.len(), 2);
    assert!(result.threats.iter().all(|threat| threat.severity == "warning"));
    assert!(enforce_scan(&result).is_ok());
}

#[test]
fn webhook_template_without_token_is_not_a_finding() {
    let root = tempfile::tempdir().unwrap();
    let mut bytes = vec![b'x'; 40_000];
    bytes.extend_from_slice(b"https://discord.com/api/webhooks/%s/%s");
    jar(&root.path().join("bridge.jar"), "WebhookManager.class", &bytes);
    let result = scan_mods_directory(root.path());
    assert!(result.is_clean);
    assert!(result.threats.is_empty());
}

#[test]
fn benign_markers_exercise_critical_launch_gate() {
    let root = tempfile::tempdir().unwrap();
    jar(&root.path().join("marker.jar"), "Example.class", b"inert test markers: java/net/URLClassLoader 85.217.144.130");
    let result = scan_mods_directory(root.path());
    assert!(!result.is_clean);
    assert!(enforce_scan(&result).is_err());
}

#[test]
fn invalid_and_oversized_classes_cannot_pass_as_clean() {
    let root = tempfile::tempdir().unwrap();
    std::fs::write(root.path().join("invalid.jar"), b"not an archive").unwrap();
    jar(&root.path().join("oversized.jar"), "Example.class", &vec![b'x'; MAX_CLASS_BYTES as usize + 1]);
    let result = scan_mods_directory(root.path());
    assert_eq!(result.threats.len(), 2);
    assert!(enforce_scan(&result).is_err());
    assert!(result.threats.iter().all(|threat| threat.threat_type == "Verificação incompleta"));
}

#[test]
fn content_hash_invalidates_cache_even_with_same_size_and_timestamp() {
    let root = tempfile::tempdir().unwrap();
    let path = root.path().join("rewrite.jar");
    let marker = b"inert test markers: java/net/URLClassLoader 85.217.144.130";
    jar(&path, "Example.class", &vec![b'x'; marker.len()]);
    let modified = path.metadata().unwrap().modified().unwrap();
    let size = path.metadata().unwrap().len();
    assert!(scan_mods_directory(root.path()).is_clean);
    jar(&path, "Example.class", marker);
    std::fs::File::options().write(true).open(&path).unwrap().set_times(std::fs::FileTimes::new().set_modified(modified)).unwrap();
    assert_eq!(path.metadata().unwrap().len(), size);
    let changed = scan_mods_directory(root.path());
    assert!(!changed.is_clean);
    assert!(enforce_scan(&changed).is_err());
}

#[test]
fn skyrage_indicators_block_launch_even_without_a_known_hash() {
    let root = tempfile::tempdir().unwrap();
    jar(&root.path().join("skyrage.jar"), "Example.class", b"inert markers: connect.skyrage.de 95.214.27.172 vmd-gnu");
    jar(&root.path().join("c2.jar"), "Other.class", b"download t23e7v6uz8idz87ehugwq.skyrage.de payload");
    let result = scan_mods_directory(root.path());
    assert_eq!(result.threats.iter().filter(|threat| threat.code == "skyrage").count(), 2);
    assert!(result.threats.iter().all(|threat| threat.severity == "critical"));
    assert!(enforce_scan(&result).is_err());
}

#[test]
fn fractureiser_download_hosts_require_remote_loader_bytecode() {
    let root = tempfile::tempdir().unwrap();
    jar(&root.path().join("loader.jar"), "Example.class", b"inert markers: 107.189.3.101 files-8ie.pages.dev java/net/URLClassLoader");
    jar(&root.path().join("hostonly.jar"), "Example.class", b"inert markers: 107.189.3.101");
    let result = scan_mods_directory(root.path());
    assert_eq!(result.threats.iter().filter(|threat| threat.code == "fractureiser").count(), 1);
    assert!(enforce_scan(&result).is_err());
}

#[test]
fn names_alone_are_warnings_and_documented_hashes_are_recognized() {
    let root = tempfile::tempdir().unwrap();
    jar(&root.path().join("names.jar"), "L10.class", b"benign example");
    let result = scan_mods_directory(root.path());
    assert_eq!(result.threats[0].severity, "warning");
    assert!(enforce_scan(&result).is_ok());
    assert!(known_hash(KNOWN_SHA256[0], ""));
    assert!(known_hash("", KNOWN_SHA1[0]));
    assert!(!known_hash("", ""));
    assert!(KNOWN_SHA256.iter().all(|hash| hash.len() == 64 && hash.bytes().all(|byte| byte.is_ascii_hexdigit())));
    assert!(KNOWN_SHA1.iter().all(|hash| hash.len() == 40 && hash.bytes().all(|byte| byte.is_ascii_hexdigit())));
}

#[test]
fn duplicate_class_paths_cannot_hide_an_earlier_indicator() {
    let root = tempfile::tempdir().unwrap();
    let path = root.path().join("duplicates.jar");
    let mut writer = zip::ZipWriter::new(std::fs::File::create(path).unwrap());
    for bytes in [b"inert markers java/net/URLClassLoader 85.217.144.130".as_slice(), b"benign replacement".as_slice()] {
        writer.start_file("Example.class", zip::write::FileOptions::default()).unwrap();
        writer.write_all(bytes).unwrap();
    }
    writer.finish().unwrap();
    let result = scan_mods_directory(root.path());
    assert!(enforce_scan(&result).is_err());
}

#[test]
fn large_resource_uses_streaming_reader_without_losing_class_detection() {
    let root = tempfile::tempdir().unwrap();
    let path = root.path().join("resources.jar");
    let mut writer = zip::ZipWriter::new(std::fs::File::create(path).unwrap());
    let options = zip::write::FileOptions::default().compression_method(zip::CompressionMethod::Stored);
    writer.start_file("assets/large.dat", options).unwrap();
    writer.write_all(&vec![0u8; BUFFER_BYTES as usize + 1]).unwrap();
    writer.start_file("Example.class", options).unwrap();
    writer.write_all(b"inert markers java/net/URLClassLoader 85.217.144.130").unwrap();
    writer.finish().unwrap();
    let result = scan_mods_directory(root.path());
    assert_eq!(result.total_scanned, 1);
    assert!(enforce_scan(&result).is_err());
}

#[test]
#[ignore = "Reads an explicitly selected installed modpack without executing its contents"]
fn installed_modpack_scan_is_read_only_and_reuses_content_cache() {
    let path = std::env::var_os("LUXMC_SHIELD_SCAN_DIR").expect("Select a mods directory");
    let root = Path::new(&path);
    let before = std::fs::read_dir(root).unwrap().map(|entry| {
        let entry = entry.unwrap();
        (entry.file_name(), entry.metadata().unwrap().len())
    }).collect::<std::collections::BTreeMap<_,_>>();
    let first = scan_mods_directory(root);
    let second = scan_mods_directory(root);
    assert!(first.total_scanned > 0);
    enforce_scan(&first).unwrap();
    enforce_scan(&second).unwrap();
    assert_eq!(first.total_scanned, second.total_scanned);
    let after = std::fs::read_dir(root).unwrap().map(|entry| {
        let entry = entry.unwrap();
        (entry.file_name(), entry.metadata().unwrap().len())
    }).collect::<std::collections::BTreeMap<_,_>>();
    assert_eq!(before, after);
    println!("SHIELD_READ_ONLY_OK mods={} warnings={} first_ms={} cached_ms={}", first.total_scanned, first.threats.len(), first.scan_time_ms, second.scan_time_ms);
}

#[test]
fn embedded_libraries_are_checked_and_cannot_hide_class_indicators() {
    let root = tempfile::tempdir().unwrap();
    let internal = root.path().join("internal.zip");
    jar(&internal, "Injected.class", b"inert markers java/net/URLClassLoader 85.217.144.130");
    jar(&root.path().join("outer.jar"), "META-INF/jarjar/library.jar", &std::fs::read(internal).unwrap());
    let result = scan_mods_directory(root.path());
    assert_eq!(result.total_scanned, 1);
    assert!(result.threats.iter().any(|finding| finding.code == "fractureiser"));
    assert!(enforce_scan(&result).is_err());
}

#[test]
fn clean_embedded_libraries_pass_and_excessive_nesting_fails_closed() {
    let root = tempfile::tempdir().unwrap();
    let internal = root.path().join("internal.zip");
    jar(&internal, "Example.class", b"benign fixture");
    jar(&root.path().join("outer.jar"), "META-INF/jars/library.jar", &std::fs::read(&internal).unwrap());
    assert!(scan_mods_directory(root.path()).is_clean);
    let mut bytes = std::fs::read(internal).unwrap();
    for _ in 0..=MAX_NESTED_DEPTH {
        let mut writer = zip::ZipWriter::new(Cursor::new(Vec::new()));
        writer.start_file("library.jar", zip::write::FileOptions::default()).unwrap();
        writer.write_all(&bytes).unwrap();
        bytes = writer.finish().unwrap().into_inner();
    }
    std::fs::write(root.path().join("outer.jar"), bytes).unwrap();
    let result = scan_mods_directory(root.path());
    assert!(result.threats.iter().any(|finding| finding.code == "scan_incomplete"));
    assert!(enforce_scan(&result).is_err());
}
