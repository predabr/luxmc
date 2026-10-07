use luxmc_repair::{file_hash, Recovery, MAIN, HELPER};
use std::{fs, path::PathBuf, time::{SystemTime, UNIX_EPOCH}};

struct Fixture { root: PathBuf, app: PathBuf, cache: PathBuf }
impl Fixture {
 fn new() -> Self {
  let root = std::env::temp_dir().join(format!("luxmc-recovery-test-{}-{}",std::process::id(),SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos()));
  let app=root.join("application"); let cache=root.join("recovery");
  fs::create_dir_all(&app).unwrap(); fs::create_dir_all(&cache).unwrap();
  fs::write(app.join(MAIN), b"healthy program").unwrap();
  fs::write(app.join(HELPER), b"healthy verifier").unwrap();
  fs::write(app.join("uninstall.exe"), b"healthy uninstaller").unwrap();
  fs::write(app.join("world.dat"), b"protected user data").unwrap();
  Self {root,app,cache}
 }
 fn recovery(&self, version: &str) -> Recovery { Recovery::new(self.app.clone(),self.cache.clone(),file_hash(&self.app.join(MAIN)).unwrap(),version.into()).unwrap() }
}
impl Drop for Fixture { fn drop(&mut self) { assert!(self.root.file_name().unwrap().to_string_lossy().starts_with("luxmc-recovery-test-")); let _=fs::remove_dir_all(&self.root); } }

#[test]
fn healthy_files_are_not_rewritten() {
 let f=Fixture::new(); let r=f.recovery("3.0.2"); r.initialize().unwrap();
 let before=fs::metadata(f.app.join(MAIN)).unwrap().modified().unwrap();
 let report=r.verify(true).unwrap(); assert!(report.healthy); assert!(!report.repaired);
 assert_eq!(before,fs::metadata(f.app.join(MAIN)).unwrap().modified().unwrap());
}

#[test]
fn corruption_is_detected_without_mutation_and_restored_from_baseline() {
 let f=Fixture::new(); let r=f.recovery("3.0.2"); let original=fs::read(f.app.join(MAIN)).unwrap(); r.initialize().unwrap();
 fs::write(f.app.join(MAIN),b"corrupt").unwrap();
 assert!(!r.verify(false).unwrap().healthy); assert_eq!(fs::read(f.app.join(MAIN)).unwrap(),b"corrupt");
 let report=r.verify(true).unwrap(); assert!(report.healthy && report.repaired && !report.rolled_back);
 assert_eq!(fs::read(f.app.join(MAIN)).unwrap(),original);
 assert_eq!(fs::read(f.app.join("world.dat")).unwrap(),b"protected user data");
}

#[test]
fn upgrade_retains_and_restores_previous_valid_program() {
 let f=Fixture::new(); let old=fs::read(f.app.join(MAIN)).unwrap(); f.recovery("3.0.1").initialize().unwrap();
 fs::write(f.app.join(MAIN),b"new healthy program").unwrap(); let r=f.recovery("3.0.2"); r.initialize().unwrap();
 fs::write(f.app.join(MAIN),b"bad new program").unwrap();
 let report=r.verify(true).unwrap(); assert!(report.healthy && report.rolled_back); assert_eq!(report.version,"3.0.1");
 assert_eq!(fs::read(f.app.join(MAIN)).unwrap(),old);
 assert!(r.verify(true).unwrap().healthy);
 assert_eq!(fs::read(f.app.join("world.dat")).unwrap(),b"protected user data");
}

#[test]
fn invalid_previous_copy_falls_back_to_current_baseline() {
 let f=Fixture::new(); f.recovery("3.0.1").initialize().unwrap();
 fs::write(f.app.join(MAIN),b"new program").unwrap(); let r=f.recovery("3.0.2"); r.initialize().unwrap();
 fs::write(f.cache.join("previous-main.exe"),b"bad previous").unwrap(); fs::write(f.app.join(MAIN),b"bad program").unwrap();
 let report=r.verify(true).unwrap(); assert!(report.healthy && !report.rolled_back);
 assert_eq!(fs::read(f.app.join(MAIN)).unwrap(),b"new program");
}

#[test]
fn no_valid_copy_never_overwrites_program_or_user_data() {
 let f=Fixture::new(); let r=f.recovery("3.0.2"); r.initialize().unwrap();
 fs::write(f.cache.join("current").join(MAIN),b"bad baseline").unwrap(); fs::write(f.app.join(MAIN),b"bad program").unwrap();
 assert!(r.verify(true).is_err()); assert_eq!(fs::read(f.app.join(MAIN)).unwrap(),b"bad program");
 assert_eq!(fs::read(f.app.join("world.dat")).unwrap(),b"protected user data");
}

#[test]
fn missing_uninstaller_is_restored_and_manifest_corruption_uses_mirror() {
 let f=Fixture::new(); let r=f.recovery("3.0.2"); r.initialize().unwrap();
 fs::write(f.cache.join("manifest.json"),b"broken json").unwrap(); fs::remove_file(f.app.join("uninstall.exe")).unwrap();
 let report=r.verify(true).unwrap(); assert!(report.healthy && report.repaired);
 assert_eq!(fs::read(f.app.join("uninstall.exe")).unwrap(),b"healthy uninstaller");
}

#[test]
fn missing_cache_can_be_initialized_only_from_verified_main() {
 let f=Fixture::new(); let r=f.recovery("3.0.2"); assert!(r.verify(true).unwrap().healthy);
}

#[test]
fn bundle_marker_is_normalized_across_stream_boundaries() {
 let f=Fixture::new(); let mut payload=vec![b'x';65531]; payload.extend_from_slice(b"PE_VAR_NSS"); payload.extend_from_slice(b"suffix");
 fs::write(f.app.join(MAIN),&payload).unwrap(); let before=file_hash(&f.app.join(MAIN)).unwrap();
 payload[65531..65541].copy_from_slice(b"PE_VAR_UNK"); fs::write(f.app.join(MAIN),payload).unwrap(); assert_eq!(before,file_hash(&f.app.join(MAIN)).unwrap());
 fs::write(f.app.join(HELPER),b"PE_VAR_NSS").unwrap(); let helper_before=file_hash(&f.app.join(HELPER)).unwrap();
 fs::write(f.app.join(HELPER),b"PE_VAR_UNK").unwrap(); assert_ne!(helper_before,file_hash(&f.app.join(HELPER)).unwrap());
}
