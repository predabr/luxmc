use luxmc_lib::core::mods::ModrinthClient;

#[tokio::test]
async fn test_mod_details_and_all_tabs() {
    let http = reqwest::Client::builder()
        .user_agent("Luxmc/1.0.0 (tests)")
        .build()
        .expect("build reqwest client");

    let modrinth = ModrinthClient::new(http.clone());

    println!("\n=== TEST MOD DETAILS & TABS FOR REAL MOD (SODIUM) ===");
    let details = modrinth.get_project_details("sodium").await
        .expect("fetch sodium details");

    // 1. TAB: VISÃO GERAL
    println!("TAB 1: Visão Geral");
    println!("  Title: {}", details.title);
    println!("  Body length: {} chars", details.body.len());
    assert!(!details.title.is_empty(), "Mod title must not be empty");
    assert!(!details.body.is_empty(), "Mod body (overview markdown) must not be empty");
    assert!(details.body.contains("Sodium") || details.body.contains("rendering"), "Body should contain descriptive text");

    // 2. TAB: GALERIA
    println!("TAB 2: Galeria");
    println!("  Gallery screenshots count: {}", details.gallery.len());
    assert!(!details.gallery.is_empty(), "Sodium should have gallery screenshots");
    for (i, img) in details.gallery.iter().take(3).enumerate() {
        println!("  Screenshot #{}: url={} title={:?}", i + 1, img.url, img.title);
        assert!(!img.url.is_empty(), "Screenshot url must be valid");
    }

    // 3. TAB: VERSÕES
    println!("TAB 3: Versões");
    let versions = modrinth.get_mod_versions("sodium", "").await
        .expect("fetch sodium versions");

    println!("  Versions count: {}", versions.len());
    assert!(!versions.is_empty(), "Sodium must have versions list");
    for (i, ver) in versions.iter().take(3).enumerate() {
        println!("  Version #{}: id={} name={} files={}", i + 1, ver.id, ver.name, ver.files.len());
        assert!(!ver.files.is_empty(), "Version must have downloadable files");
        assert!(ver.files[0].url.starts_with("http"), "File URL must be http/https");
        assert!(ver.files[0].filename.ends_with(".jar"), "File must be a .jar");
    }

    println!("SUCCESS: All 3 tabs (Visão Geral, Galeria, Versões) confirmed with real data from Modrinth!");
}
