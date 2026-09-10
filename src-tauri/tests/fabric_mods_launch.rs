use luxmc_lib::core::loaders::fabric::ensure_fabric_api;
use luxmc_lib::core::loaders::prepare_loader;
use luxmc_lib::core::mods::ModrinthClient;

#[tokio::test]
async fn test_fabric_loader_resolution_and_knot_client() {
    let http = reqwest::Client::builder()
        .user_agent("Luxmc/1.0.0-beta")
        .build()
        .unwrap();

    let temp_dir = std::env::temp_dir().join(format!("luxmc_fabric_test_{}", std::process::id()));
    let libraries_dir = temp_dir.join("libraries");
    let mods_dir = temp_dir.join("mods");
    tokio::fs::create_dir_all(&libraries_dir).await.unwrap();
    tokio::fs::create_dir_all(&mods_dir).await.unwrap();

    println!("Testing Fabric loader resolution for Minecraft 1.21.4...");
    let prep = prepare_loader(&http, &libraries_dir, "fabric", "1.21.4", None)
        .await
        .expect("prepare_loader failed");

    assert_eq!(
        prep.main_class,
        "net.fabricmc.loader.impl.launch.knot.KnotClient",
        "Main class must be Fabric KnotClient"
    );

    assert!(
        !prep.classpath_entries.is_empty(),
        "Classpath must contain Fabric loader libraries"
    );

    // Verify critical Fabric libraries were downloaded and exist on disk
    let has_loader = prep.classpath_entries.iter().any(|p| {
        p.to_string_lossy().contains("fabric-loader") && p.exists()
    });
    assert!(has_loader, "fabric-loader jar must exist in classpath and on disk");

    let has_mixin = prep.classpath_entries.iter().any(|p| {
        p.to_string_lossy().contains("sponge-mixin") && p.exists()
    });
    assert!(has_mixin, "sponge-mixin jar must exist in classpath and on disk");

    println!("✓ Fabric loader resolved: main_class = {}", prep.main_class);
    println!("✓ Classpath entries on disk: {}", prep.classpath_entries.len());

    // Test real mod retrieval from Modrinth: search for 'ferrite-core'
    let modrinth = ModrinthClient::new(http.clone());
    let search_results = modrinth
        .search_mods("ferrite-core", "1.21.4", "mod", Some("fabric"), None, 5, 0, Some("downloads"))
        .await
        .expect("Modrinth search for ferrite-core failed");

    assert!(
        !search_results.is_empty(),
        "Should find ferrite-core on Modrinth"
    );

    let ferrite = &search_results[0];
    println!("✓ Found mod on Modrinth: {} ({})", ferrite.title, ferrite.slug);

    // Get versions for ferrite-core
    let versions = modrinth
        .get_mod_versions(&ferrite.slug, "1.21.4")
        .await
        .expect("Failed to get versions for ferrite-core");

    assert!(!versions.is_empty(), "Versions should not be empty for ferrite-core 1.21.4");
    let version = &versions[0];
    let file = version.files.first().expect("FerriteCore file not found");

    // Download mod to mods directory
    let mod_jar_path = mods_dir.join(&file.filename);
    let mod_bytes = http
        .get(&file.url)
        .send()
        .await
        .unwrap()
        .error_for_status()
        .unwrap()
        .bytes()
        .await
        .unwrap();

    tokio::fs::write(&mod_jar_path, &mod_bytes).await.unwrap();
    assert!(mod_jar_path.exists(), "Mod jar must exist on disk");
    let mod_meta = tokio::fs::metadata(&mod_jar_path).await.unwrap();
    assert!(mod_meta.len() > 1000, "Mod jar file size must be > 1KB");
    println!("✓ Installed mod {} ({} bytes)", file.filename, mod_meta.len());

    // Test ensure_fabric_api: automatic promotion/installation of Fabric API
    println!("Testing ensure_fabric_api auto-download...");
    ensure_fabric_api(&http, &mods_dir, "1.21.4")
        .await
        .expect("ensure_fabric_api failed");

    // Check that fabric-api jar was downloaded into mods_dir
    let mut entries = tokio::fs::read_dir(&mods_dir).await.unwrap();
    let mut found_fabric_api = false;
    let mut total_mods = 0;
    while let Ok(Some(entry)) = entries.next_entry().await {
        let name = entry.file_name().to_string_lossy().to_string();
        if name.ends_with(".jar") {
            total_mods += 1;
            println!("  Found installed mod jar: {}", name);
            if name.contains("fabric-api") || name.contains("fabric_api") {
                found_fabric_api = true;
            }
        }
    }

    assert!(found_fabric_api, "Fabric API must be automatically downloaded to mods dir");
    assert!(total_mods >= 2, "Must have at least the mod and Fabric API in mods dir");
    println!("✓ All mods verified in instance directory: {} jars found", total_mods);

    // Clean up test directory
    let _ = tokio::fs::remove_dir_all(&temp_dir).await;
}
