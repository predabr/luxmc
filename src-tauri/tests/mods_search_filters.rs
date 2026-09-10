use luxmc_lib::core::mods::{curseforge, ModrinthClient};

#[tokio::test]
async fn test_filters_individually() {
    let http = reqwest::Client::builder()
        .user_agent("Luxmc/1.0.0 (tests)")
        .build()
        .expect("build reqwest client");

    let modrinth = ModrinthClient::new(http.clone());

    // 1. TEST TEXT SEARCH FILTER
    println!("\n=== 1. TEST SEARCH QUERY ===");
    let search_sodium = modrinth.search_mods(
        "sodium",
        "",
        "mod",
        None,
        None,
        5,
        0,
        Some("downloads"),
    ).await.expect("search sodium");

    let search_iris = modrinth.search_mods(
        "iris",
        "",
        "mod",
        None,
        None,
        5,
        0,
        Some("downloads"),
    ).await.expect("search iris");

    assert!(!search_sodium.is_empty(), "Sodium search should return results");
    assert!(!search_iris.is_empty(), "Iris search should return results");
    assert_ne!(search_sodium[0].slug, search_iris[0].slug, "Different queries must return different top mods");
    println!("Query 'sodium' top result: {} ({})", search_sodium[0].title, search_sodium[0].slug);
    println!("Query 'iris' top result: {} ({})", search_iris[0].title, search_iris[0].slug);

    // 2. TEST MOD LOADER FILTER (Fabric vs Forge)
    println!("\n=== 2. TEST MOD LOADER FILTER ===");
    let fabric_mods = modrinth.search_mods(
        "",
        "1.20.1",
        "mod",
        Some("fabric"),
        None,
        10,
        0,
        Some("downloads"),
    ).await.expect("search fabric mods");

    let forge_mods = modrinth.search_mods(
        "",
        "1.20.1",
        "mod",
        Some("forge"),
        None,
        10,
        0,
        Some("downloads"),
    ).await.expect("search forge mods");

    assert!(!fabric_mods.is_empty(), "Fabric mods search should return results");
    assert!(!forge_mods.is_empty(), "Forge mods search should return results");

    let fabric_categories: Vec<_> = fabric_mods.iter().flat_map(|m| m.categories.clone()).collect();
    let forge_categories: Vec<_> = forge_mods.iter().flat_map(|m| m.categories.clone()).collect();

    assert!(fabric_categories.contains(&"fabric".to_string()), "Results for fabric loader should include fabric tag");
    assert!(forge_categories.contains(&"forge".to_string()), "Results for forge loader should include forge tag");
    println!("Fabric top 3: {:?}", fabric_mods.iter().take(3).map(|m| &m.title).collect::<Vec<_>>());
    println!("Forge top 3: {:?}", forge_mods.iter().take(3).map(|m| &m.title).collect::<Vec<_>>());

    // 3. TEST MINECRAFT VERSION FILTER
    println!("\n=== 3. TEST MC VERSION FILTER ===");
    let mods_1_21 = modrinth.search_mods(
        "",
        "1.21.4",
        "mod",
        Some("fabric"),
        None,
        10,
        0,
        Some("downloads"),
    ).await.expect("search 1.21.4");

    let mods_1_16 = modrinth.search_mods(
        "",
        "1.16.5",
        "mod",
        Some("forge"),
        None,
        10,
        0,
        Some("downloads"),
    ).await.expect("search 1.16.5");

    assert!(!mods_1_21.is_empty(), "1.21.4 mods should return results");
    assert!(!mods_1_16.is_empty(), "1.16.5 mods should return results");
    println!("1.21.4 top mod: {} (versions: {:?})", mods_1_21[0].title, mods_1_21[0].versions.iter().take(2).collect::<Vec<_>>());
    println!("1.16.5 top mod: {} (versions: {:?})", mods_1_16[0].title, mods_1_16[0].versions.iter().take(2).collect::<Vec<_>>());

    // 4. TEST SOURCE FILTER (Modrinth vs CurseForge)
    println!("\n=== 4. TEST SOURCE FILTER ===");
    let modrinth_res = modrinth.search_mods(
        "jei",
        "1.20.1",
        "mod",
        None,
        None,
        5,
        0,
        Some("downloads"),
    ).await.expect("modrinth jei");

    let cf_res = curseforge::search_mods(
        &http,
        "jei",
        "1.20.1",
        "mod",
        None,
        5,
        0,
        Some("downloads"),
    ).await.expect("curseforge jei");

    assert!(!modrinth_res.is_empty(), "Modrinth should return results");
    println!("Modrinth source check: {} (source: {})", modrinth_res[0].title, modrinth_res[0].source);

    if !cf_res.is_empty() {
        println!("CurseForge source check: {} (source: {}, downloads: {})", cf_res[0].title, cf_res[0].source, cf_res[0].downloads);
        assert_eq!(cf_res[0].source, "curseforge");
    } else {
        println!("CurseForge returned empty or key omitted, which is handled gracefully.");
    }
}
