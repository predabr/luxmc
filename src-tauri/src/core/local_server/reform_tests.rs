use super::tests::{fixture_config, fixture_runtime, persist};
use super::*;
use std::io::Write;

fn mod_bytes(id:&str,environment:&str)->Vec<u8> {
    let mut zip=zip::ZipWriter::new(std::io::Cursor::new(Vec::new()));
    zip.start_file("fabric.mod.json",zip::write::FileOptions::default()).unwrap();
    zip.write_all(&serde_json::to_vec(&serde_json::json!({"schemaVersion":1,"id":id,"version":"1.0.0","name":id,"environment":environment})).unwrap()).unwrap();
    zip.finish().unwrap().into_inner()
}

#[tokio::test]
async fn mixed_server_packs_skip_client_jars_without_aborting_or_weakening_direct_imports() {
    for format in ["modrinth","server-zip"] {
        let temporary=tempfile::tempdir().unwrap();let manager=LocalServerManager::new(temporary.path().join("servers"),reqwest::Client::new());let config=fixture_config();let directory=persist(&manager,&config).await;
        tokio::fs::write(directory.join("luxmc-loader.json"),br#"{"kind":"fabric","version":"0.16.9"}"#).await.unwrap();
        let path=temporary.path().join("pack.zip");let mut zip=zip::ZipWriter::new(std::fs::File::create(&path).unwrap());
        if format=="modrinth" {zip.start_file("modrinth.index.json",zip::write::FileOptions::default()).unwrap();zip.write_all(&serde_json::to_vec(&serde_json::json!({"formatVersion":1,"game":"minecraft","dependencies":{"minecraft":"1.21.1","fabric-loader":"0.16.9"},"files":[]})).unwrap()).unwrap();}
        for (name,side) in [("shared","*"),("visual","client")] {zip.start_file(format!("{}mods/{name}.jar",if format=="modrinth"{"overrides/"}else{""}),zip::write::FileOptions::default()).unwrap();zip.write_all(&mod_bytes(name,side)).unwrap();}
        zip.start_file(format!("{}mods/.panorama/image.png",if format=="modrinth"{"overrides/"}else{""}),zip::write::FileOptions::default()).unwrap();zip.write_all(b"not a jar").unwrap();
        zip.finish().unwrap();
        let report=if format=="modrinth" {manager.import_modpack(&config.id,path).await.unwrap()}else{manager.import_server_pack(&config.id,path).await.unwrap()};
        assert_eq!(report["prepared"],1);assert_eq!(report["excluded"],2);assert!(report["excludedFiles"].as_array().unwrap().iter().any(|name|name=="mods/visual.jar"));assert!(!directory.join("mods/.panorama").exists());
        manager.apply_content(&config.id).await.unwrap();assert!(directory.join("mods/shared.jar").exists());assert!(!directory.join("mods/visual.jar").exists());
        let direct=temporary.path().join("visual.jar");std::fs::write(&direct,mod_bytes("visual","client")).unwrap();assert!(manager.stage_content(&config.id,"mod",&direct).await.is_err());
    }
}

#[tokio::test]
async fn content_controls_preserve_bytes_and_details_reject_foreign_paths() {
    let temporary=tempfile::tempdir().unwrap();let manager=LocalServerManager::new(temporary.path().join("servers"),reqwest::Client::new());let config=fixture_config();let directory=persist(&manager,&config).await;
    tokio::fs::write(directory.join("luxmc-loader.json"),br#"{"kind":"fabric","version":"0.16.9"}"#).await.unwrap();
    tokio::fs::create_dir_all(directory.join("mods")).await.unwrap();let bytes=mod_bytes("shared","*");tokio::fs::write(directory.join("mods/shared.jar"),&bytes).await.unwrap();
    manager.toggle_content(&config.id,"mod","shared.jar",false).await.unwrap();assert!(!directory.join("mods/shared.jar").exists());assert_eq!(tokio::fs::read(directory.join("mods/shared.jar.disabled")).await.unwrap(),bytes);
    assert!(manager.content_details(&config.id,"mod",vec!["../outside.jar".into()]).await.is_err());
    let projects=(0..1200).map(|index|(format!("mods/unrelated-project-with-a-long-file-name-{index}.jar"),"fixture-id".to_string())).collect::<std::collections::BTreeMap<_,_>>();let bytes_json=serde_json::to_vec(&projects).unwrap();assert!(bytes_json.len()>65536);tokio::fs::write(directory.join(".luxmc-content-projects.json"),bytes_json).await.unwrap();
    let details=manager.content_details(&config.id,"mod",vec!["shared.jar.disabled".into()]).await.unwrap();assert_eq!(details[0]["name"],"shared.jar.disabled");
    manager.toggle_content(&config.id,"mod","shared.jar.disabled",true).await.unwrap();assert_eq!(tokio::fs::read(directory.join("mods/shared.jar")).await.unwrap(),bytes);
    manager.stage_remove(&config.id,"mod","shared.jar").await.unwrap();assert!(manager.toggle_content(&config.id,"mod","shared.jar",false).await.is_err());
    manager.apply_content(&config.id).await.unwrap();assert!(!directory.join("mods/shared.jar").exists());
}

#[tokio::test]
async fn modpack_textures_are_bundled_and_datapacks_reach_the_world() {
    for format in ["modrinth","server-zip"] {
    let temporary=tempfile::tempdir().unwrap();let manager=LocalServerManager::new(temporary.path().join("servers"),reqwest::Client::new());let config=fixture_config();let directory=persist(&manager,&config).await;
    tokio::fs::write(directory.join("luxmc-loader.json"),br#"{"kind":"fabric","version":"0.16.9"}"#).await.unwrap();
    let path=temporary.path().join("pack.mrpack");let mut zip=zip::ZipWriter::new(std::fs::File::create(&path).unwrap());
    if format=="modrinth" {zip.start_file("modrinth.index.json",zip::write::FileOptions::default()).unwrap();zip.write_all(&serde_json::to_vec(&serde_json::json!({"formatVersion":1,"game":"minecraft","dependencies":{"minecraft":"1.21.1","fabric-loader":"0.16.9"},"files":[]})).unwrap()).unwrap();}
    for (name,bytes) in [("overrides/resourcepacks/Included/pack.mcmeta",br#"{"pack":{"pack_format":34,"description":"fixture"}}"#.as_slice()),("overrides/resourcepacks/Included/assets/test.txt",b"texture".as_slice()),("overrides/datapacks/example.zip",b"fixture datapack bytes".as_slice())] {zip.start_file(if format=="modrinth" {name}else{name.trim_start_matches("overrides/")},zip::write::FileOptions::default()).unwrap();zip.write_all(bytes).unwrap();}zip.start_file(if format=="modrinth" {"overrides/mods/shared.jar"}else{"mods/shared.jar"},zip::write::FileOptions::default()).unwrap();zip.write_all(&mod_bytes("shared","*")).unwrap();zip.finish().unwrap();
    if format=="modrinth" {manager.import_modpack(&config.id,path).await.unwrap();}else{manager.import_server_pack(&config.id,path).await.unwrap();}manager.apply_content(&config.id).await.unwrap();
    assert!(directory.join("world/datapacks/example.zip").exists());let pack=directory.join("resourcepacks/Included.zip");assert!(pack.exists());let mut archive=zip::ZipArchive::new(std::fs::File::open(&pack).unwrap()).unwrap();assert!(archive.by_name("pack.mcmeta").is_ok());assert!(archive.by_name("assets/test.txt").is_ok());
    manager.use_resource_pack(&config.id,"Included.zip".into(),false).await.unwrap();assert_eq!(manager.settings(&config.id).await.unwrap().resource_pack_file,"Included.zip");
    }
}

#[test]
fn dedicated_java_flags_are_version_gated_and_preserve_legacy_jvm_compatibility() {
    assert!(server_java_compatibility(8).is_empty());assert!(!server_java_compatibility(17).iter().any(|flag|flag.contains("unsafe")));assert!(server_java_compatibility(25).contains(&"--enable-native-access=ALL-UNNAMED"));assert!(server_java_compatibility(25).contains(&"--sun-misc-unsafe-memory-access=allow"));
    assert!(server_memory_arguments(8,true).is_empty());assert!(server_memory_arguments(21,false).is_empty());assert!(server_memory_arguments(21,true).contains(&"-XX:G1PeriodicGCInterval=60000"));
}

#[tokio::test]
#[ignore = "Downloads a real Modrinth server pack into a temporary directory without running Java"]
async fn real_catalog_pack_imports_applies_and_excludes_client_libraries() {
    let http=reqwest::Client::builder().user_agent("Luxmc/4.0.0 integration-check").timeout(Duration::from_secs(60)).build().unwrap();
    let project=std::env::var("LUXMC_REAL_PACK").unwrap_or_else(|_|"adrenaline".into());
    let versions:serde_json::Value=http.get(format!("https://api.modrinth.com/v2/project/{project}/version")).send().await.unwrap().error_for_status().unwrap().json().await.unwrap();
    let pin=std::env::var("LUXMC_REAL_PACK_VERSION").ok();
    let version=versions.as_array().unwrap().iter().find(|version|pin.as_ref().is_none_or(|pin|version["id"].as_str()==Some(pin))).unwrap();let file=version["files"].as_array().unwrap().iter().find(|file|file["filename"].as_str().unwrap_or("").ends_with(".mrpack")).unwrap();
    let bytes=http.get(file["url"].as_str().unwrap()).send().await.unwrap().error_for_status().unwrap().bytes().await.unwrap();
    let temporary=tempfile::tempdir().unwrap();let path=temporary.path().join("pack.mrpack");tokio::fs::write(&path,&bytes).await.unwrap();
    assert!(crate::core::mods::pack_download::verify_existing(&path,file["size"].as_u64(),file["hashes"]["sha1"].as_str(),None,true).await);
    let index:serde_json::Value={let mut zip=zip::ZipArchive::new(std::io::Cursor::new(bytes)).unwrap();let file=zip.by_name("modrinth.index.json").unwrap();serde_json::from_reader(file).unwrap()};
    let mut config=fixture_config();config.mc_version=index["dependencies"]["minecraft"].as_str().unwrap().into();let manager=LocalServerManager::new(temporary.path().join("servers"),http);let directory=persist(&manager,&config).await;
    let (kind,version)=index["dependencies"].as_object().unwrap().iter().find(|(name,_)|name.as_str()!="minecraft").unwrap();let kind=match kind.as_str(){"fabric-loader"=>"fabric","quilt-loader"=>"quilt",other=>other};tokio::fs::write(directory.join("luxmc-loader.json"),serde_json::to_vec(&serde_json::json!({"kind":kind,"version":version})).unwrap()).await.unwrap();
    let report=manager.import_modpack(&config.id,path).await.unwrap();let applied=manager.apply_content(&config.id).await.unwrap();
    assert!(report["prepared"].as_u64().unwrap()>0);assert!(applied.applied>0);manager.validate_installed_mods(&config.id).await.unwrap();
    eprintln!("REAL PACK: Minecraft {}, prepared {}, excluded {}, applied {}",config.mc_version,report["prepared"],report["excluded"],applied.applied);
}

#[tokio::test]
async fn legacy_settings_preserve_generation_properties_and_explicit_new_choices_win() {
    let temporary = tempfile::tempdir().unwrap();
    let manager = LocalServerManager::new(temporary.path().join("servers"), reqwest::Client::new());
    let config = fixture_config(); let directory = persist(&manager, &config).await;
    tokio::fs::write(directory.join("server.properties"),b"level-seed=42\nhardcore=true\ngenerate-structures=false\n").await.unwrap();
    tokio::fs::write(directory.join("luxmc-settings.json"),b"{\"motd\":\"legacy\"}").await.unwrap();
    let mut settings = manager.settings(&config.id).await.unwrap();
    assert_eq!(settings.seed.as_deref(),Some("42")); assert!(settings.hardcore); assert!(!settings.generate_structures);
    settings.seed = Some(String::new()); settings.hardcore=false; settings.generate_structures=true;
    manager.update_settings(&config.id,settings).await.unwrap();
    let explicit = manager.settings(&config.id).await.unwrap();
    assert_eq!(explicit.seed.as_deref(),Some("")); assert!(!explicit.hardcore); assert!(explicit.generate_structures);
}

#[tokio::test]
async fn permanent_deletion_checks_name_and_jobs_and_preserves_independent_backups() {
    let temporary = tempfile::tempdir().unwrap();
    let manager = LocalServerManager::new(temporary.path().join("servers"), reqwest::Client::new());
    let config = fixture_config(); let directory = persist(&manager, &config).await;
    let backup = directory.parent().unwrap().join("backups").join(&config.id);
    tokio::fs::create_dir_all(&backup).await.unwrap(); tokio::fs::write(backup.join("keep.zip"),b"backup").await.unwrap();
    assert!(manager.delete_server(&config.id,"wrong",false).await.is_err());
    let job = manager.content_job().unwrap();
    assert!(manager.delete_server(&config.id,&config.name,false).await.is_err()); drop(job);
    manager.delete_server(&config.id,&config.name,false).await.unwrap();
    assert!(!directory.exists()); assert!(backup.join("keep.zip").exists());
    assert!(manager.deleted_servers().await.unwrap().is_empty());
    let directory = persist(&manager, &config).await;
    manager.delete_server(&config.id,&config.name,true).await.unwrap();
    assert!(!directory.exists()); assert!(!backup.exists());
}

#[tokio::test]
async fn deleting_server_requires_confirmation_preserves_world_and_can_be_undone() {
    let temporary = tempfile::tempdir().unwrap();
    let manager = LocalServerManager::new(temporary.path().join("servers"), reqwest::Client::new());
    let config = fixture_config(); let directory = persist(&manager, &config).await;
    tokio::fs::create_dir(directory.join("world")).await.unwrap(); tokio::fs::write(directory.join("world/level.dat"), b"preserve-world").await.unwrap();
    assert!(manager.archive_server(&config.id, "wrong name").await.is_err());
    manager.archive_server(&config.id, &config.name).await.unwrap();
    assert!(manager.list(0).await.unwrap().servers.is_empty());
    manager.restore_server(&config.id).await.unwrap();
    assert_eq!(tokio::fs::read(directory.join("world/level.dat")).await.unwrap(), b"preserve-world");
    assert_eq!(manager.list(0).await.unwrap().servers.len(), 1);
    assert!(manager.browse_files(&config.id, "../").await.is_err());
    assert!(manager.browse_files(&config.id, ".luxmc-playit.json").await.is_err());
}

#[tokio::test]
async fn interrupted_instance_copy_cannot_launch_an_empty_modded_server() {
    let temporary = tempfile::tempdir().unwrap();
    let manager = LocalServerManager::new(temporary.path().join("servers"), reqwest::Client::new());
    let config = fixture_config();
    let directory = persist(&manager, &config).await;
    tokio::fs::write(directory.join(".luxmc-instance-pending"), b"pending").await.unwrap();
    let result = manager.start(config.id,20).await;
    assert!(result.is_err()); assert!(!manager.has_active());
}

fn archive(path: &Path, name: &str, bytes: &[u8]) {
    let mut file = zip::ZipWriter::new(std::fs::File::create(path).unwrap());
    file.start_file(name, zip::write::FileOptions::default())
        .unwrap();
    file.write_all(bytes).unwrap();
    file.finish().unwrap();
}

#[tokio::test]
async fn start_fails_promptly_during_content_work_and_does_not_queue_a_later_launch() {
    let temporary = tempfile::tempdir().unwrap();
    let manager = LocalServerManager::new(temporary.path().join("servers"), reqwest::Client::new());
    let config = fixture_config();
    persist(&manager, &config).await;
    {
        let _operation = manager.changes.lock().await;
        let result = tokio::time::timeout(
            Duration::from_millis(100),
            manager.start(config.id.clone(), 20),
        )
        .await;
        assert!(result
            .expect("start must not queue behind another operation")
            .is_err());
        assert!(!manager.has_active());
    }
    {
        let _download = manager.content_job().unwrap();
        assert!(manager.start(config.id.clone(), 20).await.is_err());
        assert!(!manager.has_active());
        assert!(manager.stop(&config.id).await.is_ok());
    }
    assert_eq!(manager.content_jobs.load(Ordering::Acquire), 0);
    assert!(manager.change().is_ok());
}

#[tokio::test]
async fn backup_restore_preserves_world_settings_and_generated_libraries_and_protects_the_last_copy(
) {
    let temporary = tempfile::tempdir().unwrap();
    let manager = LocalServerManager::new(temporary.path().join("servers"), reqwest::Client::new());
    let config = fixture_config();
    let directory = persist(&manager, &config).await;
    tokio::fs::create_dir_all(directory.join("world"))
        .await
        .unwrap();
    tokio::fs::write(directory.join("world/level.dat"), b"original-world")
        .await
        .unwrap();
    tokio::fs::create_dir_all(directory.join("libraries"))
        .await
        .unwrap();
    tokio::fs::write(directory.join("libraries/generated.jar"), b"library")
        .await
        .unwrap();
    let first = manager.backup(&config.id).await.unwrap();
    assert_eq!(first.files, 2);
    assert!(manager.remove_backup(&config.id, &first.id).await.is_err());
    tokio::fs::write(directory.join("world/level.dat"), b"changed-world")
        .await
        .unwrap();
    manager.restore_backup(&config.id, &first.id).await.unwrap();
    assert_eq!(
        tokio::fs::read(directory.join("world/level.dat"))
            .await
            .unwrap(),
        b"original-world"
    );
    assert_eq!(
        tokio::fs::read(directory.join("libraries/generated.jar"))
            .await
            .unwrap(),
        b"library"
    );
    assert_eq!(manager.backups(&config.id).await.unwrap().len(), 2);
    manager.remove_backup(&config.id, &first.id).await.unwrap();
    assert_eq!(manager.backups(&config.id).await.unwrap().len(), 1);
}

#[tokio::test]
async fn resource_pack_import_can_stage_while_running_but_cannot_replace_active_files() {
    let temporary = tempfile::tempdir().unwrap();
    let manager = LocalServerManager::new(temporary.path().join("servers"), reqwest::Client::new());
    let config = fixture_config();
    let directory = persist(&manager, &config).await;
    let source = temporary.path().join("pack.zip");
    archive(
        &source,
        "pack.mcmeta",
        br#"{"pack":{"pack_format":34,"description":"Fixture"}}"#,
    );
    let (runtime, _) = fixture_runtime(config.id.clone());
    runtime.phase("running");
    *manager.runtime.lock().unwrap() = Some(runtime.clone());
    manager
        .stage_content(&config.id, "resourcepack", &source)
        .await
        .unwrap();
    assert!(!directory.join("resourcepacks/pack.zip").exists());
    assert!(manager.apply_content(&config.id).await.is_err());
    assert!(manager
        .contents(&config.id)
        .await
        .unwrap()
        .iter()
        .any(|item| item.staged));
    runtime.finished.send_replace(true);
    manager.apply_content(&config.id).await.unwrap();
    assert!(directory.join("resourcepacks/pack.zip").is_file());
    assert!(manager
        .contents(&config.id)
        .await
        .unwrap()
        .iter()
        .all(|item| !item.staged));
    assert_eq!(manager.backups(&config.id).await.unwrap().len(), 0);
}

#[tokio::test]
async fn pending_validation_is_complete_before_any_active_file_is_replaced() {
    let temporary = tempfile::tempdir().unwrap();
    let manager = LocalServerManager::new(temporary.path().join("servers"), reqwest::Client::new());
    let config = fixture_config();
    let directory = persist(&manager, &config).await;
    let one = temporary.path().join("one.zip");
    archive(&one, "pack.mcmeta", b"{}");
    let two = temporary.path().join("two.zip");
    archive(&two, "pack.mcmeta", b"{}");
    manager
        .stage_content(&config.id, "resourcepack", &one)
        .await
        .unwrap();
    manager
        .stage_content(&config.id, "resourcepack", &two)
        .await
        .unwrap();
    let mut pending = tokio::fs::read_dir(directory.join(".luxmc-staging"))
        .await
        .unwrap();
    let entry = pending.next_entry().await.unwrap().unwrap();
    tokio::fs::write(entry.path().join("payload"), b"corrupt")
        .await
        .unwrap();
    assert!(manager.apply_content(&config.id).await.is_err());
    assert!(!directory.join("resourcepacks/one.zip").exists());
    assert!(!directory.join("resourcepacks/two.zip").exists());
    manager.clear_pending(&config.id).await.unwrap();
    assert!(!directory.join(".luxmc-staging").exists());
}

#[tokio::test]
async fn client_only_mod_and_unsafe_pack_paths_are_rejected_without_touching_the_world() {
    let temporary = tempfile::tempdir().unwrap();
    let manager = LocalServerManager::new(temporary.path().join("servers"), reqwest::Client::new());
    let config = fixture_config();
    let directory = persist(&manager, &config).await;
    tokio::fs::write(
        directory.join("luxmc-loader.json"),
        br#"{"kind":"fabric","version":"0.16.14"}"#,
    )
    .await
    .unwrap();
    let source = temporary.path().join("client.jar");
    archive(
        &source,
        "fabric.mod.json",
        br#"{"id":"client","version":"1","environment":"client"}"#,
    );
    assert!(manager
        .stage_content(&config.id, "mod", &source)
        .await
        .is_err());
    for name in [
        "../world/level.dat",
        "mods/../../bad.jar",
        "mods/a.bat",
        "server.properties",
        "config/C:bad",
        "libraries/evil.jar",
    ] {
        assert!(content::pack_path(name).is_err(), "{name}");
    }
    assert!(content::pack_path("config/example/options.json").is_ok());
    assert!(content::pack_path("mods/example.jar").is_ok());
    assert!(!directory.join(".luxmc-staging").exists());
}

#[tokio::test]
async fn verified_owner_is_added_without_erasing_whitelist_bans_or_operators() {
    let temporary = tempfile::tempdir().unwrap();
    let manager = LocalServerManager::new(temporary.path().join("servers"), reqwest::Client::new());
    let config = fixture_config();
    let directory = persist(&manager, &config).await;
    let friend = uuid::Uuid::new_v4().to_string();
    let owner = uuid::Uuid::new_v4().to_string();
    tokio::fs::write(
        directory.join("whitelist.json"),
        serde_json::to_vec(&serde_json::json!([{"name":"Friend","uuid":friend}])).unwrap(),
    )
    .await
    .unwrap();
    tokio::fs::write(directory.join("banned-players.json"), b"[]")
        .await
        .unwrap();
    tokio::fs::write(directory.join("ops.json"), b"[]")
        .await
        .unwrap();
    manager
        .include_owner(&config.id, &owner, "Owner")
        .await
        .unwrap();
    manager
        .include_owner(&config.id, &owner, "RenamedOwner")
        .await
        .unwrap();
    let entries: Vec<serde_json::Value> = serde_json::from_slice(
        &tokio::fs::read(directory.join("whitelist.json"))
            .await
            .unwrap(),
    )
    .unwrap();
    assert_eq!(entries.len(), 2);
    assert!(entries.iter().any(|entry| entry["uuid"] == friend));
    assert!(entries.iter().any(|entry| entry["name"] == "RenamedOwner"));
    assert_eq!(
        tokio::fs::read(directory.join("banned-players.json"))
            .await
            .unwrap(),
        b"[]"
    );
    assert_eq!(
        tokio::fs::read(directory.join("ops.json")).await.unwrap(),
        b"[]"
    );
}

fn mrpack(path: &Path, minecraft: &str, value: &str) {
    let mut file = zip::ZipWriter::new(std::fs::File::create(path).unwrap());
    let options = zip::write::FileOptions::default();
    let index = serde_json::json!({"formatVersion":1,"game":"minecraft","versionId":"fixture","name":"Fixture","dependencies":{"minecraft":minecraft},"files":[{"path":"mods/client-only.jar","env":{"client":"required","server":"unsupported"},"downloads":[],"fileSize":1,"hashes":{}}]});
    for (name, bytes) in [
        ("modrinth.index.json", serde_json::to_vec(&index).unwrap()),
        ("overrides/config/example.json", b"base".to_vec()),
        (
            "server-overrides/config/example.json",
            value.as_bytes().to_vec(),
        ),
        (
            "client-overrides/config/client-only.json",
            b"client".to_vec(),
        ),
        ("overrides/options.txt", b"client options".to_vec()),
    ] {
        file.start_file(name, options).unwrap();
        file.write_all(&bytes).unwrap();
    }
    file.finish().unwrap();
}

#[tokio::test]
async fn client_performance_packs_can_use_a_server_without_copying_client_only_mods() {
    let temporary = tempfile::tempdir().unwrap();
    let manager = LocalServerManager::new(temporary.path().join("servers"), reqwest::Client::new());
    let config = fixture_config();
    let directory = persist(&manager, &config).await;
    let source = temporary.path().join("client.mrpack");
    let index = serde_json::json!({"formatVersion":1,"game":"minecraft","dependencies":{"minecraft":"1.21.1"},"files":[{"path":"mods/client.jar","env":{"server":"unsupported"},"downloads":[],"fileSize":1,"hashes":{}}]});
    archive(
        &source,
        "modrinth.index.json",
        &serde_json::to_vec(&index).unwrap(),
    );
    let result = manager
        .import_modpack(&config.id, source.clone())
        .await
        .unwrap();
    assert_eq!(result["prepared"], 0);
    assert_eq!(result["noRequiredServerFiles"], true);
    assert!(!directory.join("mods/client.jar").exists());
    tokio::fs::create_dir(directory.join("mods")).await.unwrap();
    tokio::fs::write(directory.join("mods/existing.jar"), b"existing")
        .await
        .unwrap();
    assert!(manager.import_modpack(&config.id, source).await.is_err());
    assert_eq!(
        tokio::fs::read(directory.join("mods/existing.jar"))
            .await
            .unwrap(),
        b"existing"
    );
}

#[tokio::test]
async fn modrinth_server_overrides_win_client_only_files_are_excluded_and_local_edits_survive_updates(
) {
    let temporary = tempfile::tempdir().unwrap();
    let manager = LocalServerManager::new(temporary.path().join("servers"), reqwest::Client::new());
    let config = fixture_config();
    let directory = persist(&manager, &config).await;
    let source = temporary.path().join("fixture.mrpack");
    mrpack(&source, "1.21.1", "server defaults v1");
    let result = manager
        .import_modpack(&config.id, source.clone())
        .await
        .unwrap();
    assert_eq!(result["prepared"], 1);
    assert_eq!(result["excluded"], 1);
    assert!(!directory.join("config/example.json").exists());
    let applied = manager.apply_content(&config.id).await.unwrap();
    assert_eq!(applied.applied, 1);
    assert!(applied.preserved.is_empty());
    assert_eq!(
        tokio::fs::read(directory.join("config/example.json"))
            .await
            .unwrap(),
        b"server defaults v1"
    );
    assert!(!directory.join("mods/client-only.jar").exists());
    assert!(!directory.join("options.txt").exists());
    assert!(!directory.join("config/client-only.json").exists());
    tokio::fs::write(directory.join("config/example.json"), b"owner preferences")
        .await
        .unwrap();
    mrpack(&source, "1.21.1", "server defaults v2");
    manager
        .import_modpack(&config.id, source.clone())
        .await
        .unwrap();
    let applied = manager.apply_content(&config.id).await.unwrap();
    assert_eq!(applied.preserved, vec!["config/example.json"]);
    assert_eq!(
        tokio::fs::read(directory.join("config/example.json"))
            .await
            .unwrap(),
        b"owner preferences"
    );
    mrpack(&source, "1.20.1", "wrong version");
    assert!(manager.import_modpack(&config.id, source).await.is_err());
    assert_eq!(
        tokio::fs::read(directory.join("config/example.json"))
            .await
            .unwrap(),
        b"owner preferences"
    );
}

#[tokio::test]
async fn quilt_and_forge_client_only_flags_are_rejected_before_staging() {
    let temporary = tempfile::tempdir().unwrap();
    let manager = LocalServerManager::new(temporary.path().join("servers"), reqwest::Client::new());
    let config = fixture_config();
    let directory = persist(&manager, &config).await;
    tokio::fs::write(
        directory.join("luxmc-loader.json"),
        br#"{"kind":"quilt","version":"0.28.1"}"#,
    )
    .await
    .unwrap();
    let source = temporary.path().join("client.jar");
    archive(
        &source,
        "quilt.mod.json",
        br#"{"quilt_loader":{"id":"client","version":"1"},"minecraft":{"environment":"client"}}"#,
    );
    assert!(manager
        .stage_content(&config.id, "mod", &source)
        .await
        .is_err());
    archive(&source, "META-INF/mods.toml", b"modLoader='javafml'\nloaderVersion='[1,)'\nclientSideOnly=true\n[[mods]]\nmodId='client'\nversion='1'\n");
    assert!(manager
        .stage_content(&config.id, "mod", &source)
        .await
        .is_err());
    assert!(!directory.join(".luxmc-staging").exists());
}
