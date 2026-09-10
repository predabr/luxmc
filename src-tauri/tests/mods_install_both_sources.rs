use luxmc_lib::core::mods::{curseforge, ModrinthClient};

#[tokio::test]
async fn test_install_mods_from_both_sources() {
    // 1. Setup temporary test instance directory
    let temp_dir = std::env::temp_dir().join(format!("luxmc_test_install_{}", std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos()));
    let instance_mods_dir = temp_dir.join("mods");
    std::fs::create_dir_all(&instance_mods_dir).expect("create mods dir");

    let http = reqwest::Client::builder()
        .user_agent("Luxmc/1.0.0 (tests)")
        .build()
        .expect("build reqwest client");

    // 2. MODRINTH: Fetch versions and download FerriteCore
    println!("\n=== TEST MODRINTH INSTALL ===");
    let modrinth = ModrinthClient::new(http.clone());
    let modrinth_versions = modrinth.get_mod_versions("ferrite-core", "").await
        .expect("fetch ferrite-core versions from modrinth");

    assert!(!modrinth_versions.is_empty(), "Modrinth versions should not be empty");
    let m_ver = &modrinth_versions[0];
    let m_file = &m_ver.files[0];
    println!("Modrinth target file: {} ({})", m_file.filename, m_file.url);

    let m_bytes = http.get(&m_file.url)
        .send().await.expect("download modrinth file")
        .error_for_status().expect("status ok")
        .bytes().await.expect("read bytes");

    let modrinth_dest = instance_mods_dir.join(&m_file.filename);
    std::fs::write(&modrinth_dest, &m_bytes).expect("write modrinth jar");
    assert!(modrinth_dest.exists(), "Modrinth jar should exist");
    println!("Successfully saved Modrinth jar: {} (size: {} bytes)", modrinth_dest.display(), m_bytes.len());

    // 3. CURSEFORGE: Fetch versions and download Thirstbar
    println!("\n=== TEST CURSEFORGE INSTALL ===");
    let cf_versions = curseforge::get_mod_versions(&http, "1438469", "").await
        .expect("fetch curseforge versions");

    assert!(!cf_versions.is_empty(), "CurseForge versions should not be empty");
    let c_ver = &cf_versions[0];
    let c_file = &c_ver.files[0];
    println!("CurseForge target file: {} ({})", c_file.filename, c_file.url);

    let c_bytes = http.get(&c_file.url)
        .send().await.expect("download curseforge file")
        .error_for_status().expect("status ok")
        .bytes().await.expect("read bytes");

    let curseforge_dest = instance_mods_dir.join(&c_file.filename);
    std::fs::write(&curseforge_dest, &c_bytes).expect("write curseforge jar");
    assert!(curseforge_dest.exists(), "CurseForge jar should exist");
    println!("Successfully saved CurseForge jar: {} (size: {} bytes)", curseforge_dest.display(), c_bytes.len());

    // 4. VERIFY WITH DIRECTORY LISTING
    println!("\n=== VERIFYING FILES IN INSTANCE MODS DIRECTORY ===");
    let mut installed_files: Vec<String> = std::fs::read_dir(&instance_mods_dir)
        .expect("read instance mods dir")
        .flatten()
        .map(|entry| entry.file_name().to_string_lossy().to_string())
        .collect();

    installed_files.sort();
    println!("Installed .jar files: {:?}", installed_files);

    assert_eq!(installed_files.len(), 2, "Expected exactly 2 mod files installed");
    assert!(installed_files.iter().any(|f| f.ends_with(".jar")), "Files must be .jar archives");
    assert!(installed_files.contains(&m_file.filename), "Must contain Modrinth mod");
    assert!(installed_files.contains(&c_file.filename), "Must contain CurseForge mod");

    println!("SUCCESS: Both mods from Modrinth and CurseForge confirmed present in instance directory!");
}
