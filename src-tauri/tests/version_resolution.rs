use luxmc_lib::core::launcher::{self, lib_path_from_name};
use luxmc_lib::core::minecraft::{self, lib_url_from_name};
use std::path::PathBuf;

async fn fetch_detail(http: &reqwest::Client, id: &str) -> minecraft::VersionDetail {
    let manifest = minecraft::fetch_version_manifest(http).await.unwrap();
    let entry = manifest
        .versions
        .iter()
        .find(|v| v.id == id)
        .unwrap_or_else(|| panic!("version {} not in manifest", id));
    println!("  [OK] Found in manifest (type={})", entry.version_type);
    minecraft::fetch_version_detail(http, &entry.url)
        .await
        .unwrap()
}

fn resolve_game_args(detail: &minecraft::VersionDetail, game_args: &[String]) -> Vec<String> {
    let assets_dir = "/tmp/assets";
    let asset_index_id = detail
        .asset_index
        .as_ref()
        .map(|ai| ai.id.as_str())
        .unwrap_or("legacy");
    game_args
        .iter()
        .map(|a| {
            a.replace("${auth_player_name}", "Test")
                .replace("${auth_uuid}", "000")
                .replace("${auth_session}", "tok")
                .replace("${auth_access_token}", "tok")
                .replace("${user_properties}", "{}")
                .replace("${version_name}", &detail.id)
                .replace("${game_directory}", "/tmp/game")
                .replace("${assets_directory}", assets_dir)
                .replace("${assets_root}", assets_dir)
                .replace("${game_assets}", assets_dir)
                .replace("${asset_index}", asset_index_id)
                .replace("${assets_index_name}", asset_index_id)
                .replace("${clientid}", "")
                .replace("${auth_xuid}", "")
                .replace("${user_type}", "msa")
                .replace("${version_type}", &detail.version_type)
                .replace("${resolution_width}", "854")
                .replace("${resolution_height}", "480")
                .replace("${quickPlayPath}", "")
                .replace("${quickPlaySingleplayer}", "")
                .replace("${quickPlayMultiplayer}", "")
                .replace("${quickPlayRealms}", "")
        })
        .collect()
}

#[tokio::test]
async fn test_1_21_4() {
    let http = reqwest::Client::new();
    println!("\n{}", "=".repeat(60));
    println!("=== TESTING VERSION: 1.21.4 (modern release) ===");
    println!("{}", "=".repeat(60));

    let detail = fetch_detail(&http, "1.21.4").await;
    assert_eq!(detail.id, "1.21.4");
    println!("  [OK] Version detail fetched");

    let java_major = detail.java_major_version();
    println!(
        "  [JAVA] major_version={} (from javaVersion field)",
        java_major
    );
    assert!(
        java_major >= 17,
        "1.21.4 should require Java 17+, got {}",
        java_major
    );

    let has_modern = detail.arguments.is_some();
    println!(
        "  [ARGS] format = {}",
        if has_modern { "MODERN" } else { "OLD" }
    );
    assert!(has_modern, "1.21.4 should have modern arguments");

    let game_args = detail.effective_game_args();
    println!("  [ARGS] resolved {} game args", game_args.len());
    assert!(!game_args.is_empty());

    let jvm_args = detail.effective_jvm_args();
    assert!(jvm_args.is_some(), "1.21.4 should have JVM args");
    println!("  [ARGS] jvm args present={}", jvm_args.is_some());

    assert!(detail.downloads.is_some(), "1.21.4 should have downloads");
    assert!(
        detail.asset_index.is_some(),
        "1.21.4 should have asset_index"
    );
    assert!(detail.main_class.is_some(), "1.21.4 should have main_class");
    let mc = detail.main_class.as_deref().unwrap();
    println!("  [MAIN] {}", mc);
    assert!(
        mc.contains("Minecraft") || mc.contains("Main"),
        "1.21.4 main_class should be a Minecraft class, got {}",
        mc
    );

    let lib_count = detail.libraries.len();
    println!("  [LIBS] {} libraries", lib_count);
    assert!(
        lib_count > 50,
        "1.21.4 should have many libraries, got {}",
        lib_count
    );

    let mut url_ok = 0;
    for lib in &detail.libraries {
        if !launcher::is_library_allowed(lib) {
            continue;
        }
        if let Some(ref dl) = lib.downloads {
            if let Some(ref artifact) = dl.artifact {
                if !artifact.url.is_empty() {
                    url_ok += 1;
                }
            }
        }
    }
    println!("  [LIBS] url_valid={}", url_ok);
    assert!(url_ok > 40);

    let resolved = resolve_game_args(&detail, &game_args);
    let unresolved: Vec<_> = resolved.iter().filter(|a| a.contains("${")).collect();
    assert!(
        unresolved.is_empty(),
        "1.21.4 has unresolved vars: {:?}",
        unresolved
    );

    println!("  >>> 1.21.4 PASSED <<<");
}

#[tokio::test]
async fn test_1_16_5() {
    let http = reqwest::Client::new();
    println!("\n{}", "=".repeat(60));
    println!("=== TESTING VERSION: 1.16.5 (transitional) ===");
    println!("{}", "=".repeat(60));

    let detail = fetch_detail(&http, "1.16.5").await;
    assert_eq!(detail.id, "1.16.5");

    let java_major = detail.java_major_version();
    println!("  [JAVA] major_version={}", java_major);
    assert!(
        java_major >= 8 && java_major <= 16,
        "1.16.5 should be Java 8-16, got {}",
        java_major
    );

    let has_modern = detail.arguments.is_some();
    let has_old = detail.minecraft_arguments.is_some();
    println!("  [ARGS] modern={}, old={}", has_modern, has_old);
    let game_args = detail.effective_game_args();
    println!("  [ARGS] resolved {} game args", game_args.len());
    assert!(!game_args.is_empty());

    assert!(detail.downloads.is_some());
    assert!(detail.asset_index.is_some());
    assert!(detail.main_class.is_some());

    let lib_count = detail.libraries.len();
    println!("  [LIBS] {} libraries", lib_count);
    assert!(lib_count > 30);

    let mut url_ok = 0;
    for lib in &detail.libraries {
        if !launcher::is_library_allowed(lib) {
            continue;
        }
        if let Some(ref dl) = lib.downloads {
            if let Some(ref artifact) = dl.artifact {
                if !artifact.url.is_empty() {
                    url_ok += 1;
                }
            }
        }
    }
    println!("  [LIBS] url_valid={}", url_ok);
    assert!(url_ok > 20);

    let resolved = resolve_game_args(&detail, &game_args);
    let unresolved: Vec<_> = resolved.iter().filter(|a| a.contains("${")).collect();
    assert!(
        unresolved.is_empty(),
        "1.16.5 has unresolved vars: {:?}",
        unresolved
    );

    println!("  >>> 1.16.5 PASSED <<<");
}

#[tokio::test]
async fn test_1_8_9() {
    let http = reqwest::Client::new();
    println!("\n{}", "=".repeat(60));
    println!("=== TESTING VERSION: 1.8.9 (classic release) ===");
    println!("{}", "=".repeat(60));

    let detail = fetch_detail(&http, "1.8.9").await;
    assert_eq!(detail.id, "1.8.9");

    let java_major = detail.java_major_version();
    println!(
        "  [JAVA] major_version={} (default, no javaVersion field)",
        java_major
    );
    assert_eq!(java_major, 8, "1.8.9 should default to Java 8");

    let has_modern = detail.arguments.is_some();
    let has_old = detail.minecraft_arguments.is_some();
    println!("  [ARGS] modern={}, old={}", has_modern, has_old);
    let game_args = detail.effective_game_args();
    println!("  [ARGS] resolved {} game args", game_args.len());
    assert!(!game_args.is_empty());

    assert!(detail.downloads.is_some());
    assert!(detail.asset_index.is_some());
    assert!(detail.main_class.is_some());

    let lib_count = detail.libraries.len();
    println!("  [LIBS] {} libraries", lib_count);
    assert!(lib_count > 10);

    let mut url_ok = 0;
    for lib in &detail.libraries {
        if !launcher::is_library_allowed(lib) {
            continue;
        }
        if let Some(ref dl) = lib.downloads {
            if let Some(ref artifact) = dl.artifact {
                if !artifact.url.is_empty() {
                    url_ok += 1;
                }
            }
        } else {
            let url = lib_url_from_name(&lib.name, lib.url.as_deref());
            if !url.is_empty() && url.starts_with("http") {
                url_ok += 1;
            }
        }
    }
    println!("  [LIBS] url_valid={}", url_ok);
    assert!(url_ok > 15);

    let resolved = resolve_game_args(&detail, &game_args);
    let unresolved: Vec<_> = resolved.iter().filter(|a| a.contains("${")).collect();
    assert!(
        unresolved.is_empty(),
        "1.8.9 has unresolved vars: {:?}",
        unresolved
    );

    println!("  >>> 1.8.9 PASSED <<<");
}

#[tokio::test]
async fn test_b1_7_3() {
    let http = reqwest::Client::new();
    println!("\n{}", "=".repeat(60));
    println!("=== TESTING VERSION: b1.7.3 (old_beta) ===");
    println!("{}", "=".repeat(60));

    let detail = fetch_detail(&http, "b1.7.3").await;
    assert_eq!(detail.id, "b1.7.3");

    let java_major = detail.java_major_version();
    println!(
        "  [JAVA] major_version={} (default, no javaVersion field)",
        java_major
    );
    assert_eq!(java_major, 8, "b1.7.3 should default to Java 8");

    let has_modern = detail.arguments.is_some();
    let has_old = detail.minecraft_arguments.is_some();
    println!("  [ARGS] modern={}, old={}", has_modern, has_old);
    assert!(!has_modern, "b1.7.3 should NOT have modern arguments");
    assert!(has_old, "b1.7.3 should have minecraftArguments string");

    let game_args = detail.effective_game_args();
    println!(
        "  [ARGS] resolved {} game args from minecraftArguments",
        game_args.len()
    );
    assert!(
        !game_args.is_empty(),
        "b1.7.3 should resolve game args from minecraftArguments string"
    );

    println!("  [ARGS] raw = {:?}", detail.minecraft_arguments);
    println!("  [ARGS] resolved = {:?}", game_args);

    let main_class = detail.main_class.as_deref().unwrap_or("MISSING");
    println!("  [MAIN] {}", main_class);
    assert!(
        main_class.contains("Minecraft") || main_class.contains("Launch"),
        "b1.7.3 main_class should be a Minecraft/Launch class, got {}",
        main_class
    );

    assert!(
        detail.downloads.is_some(),
        "b1.7.3 should have downloads in Mojang manifest"
    );
    assert!(
        detail.asset_index.is_some(),
        "b1.7.3 should have asset_index"
    );
    assert!(detail.assets.is_some(), "b1.7.3 should have assets field");

    let lib_count = detail.libraries.len();
    println!("  [LIBS] {} libraries", lib_count);
    assert!(
        lib_count > 5,
        "b1.7.3 should have libraries, got {}",
        lib_count
    );

    let mut with_downloads = 0;
    let mut with_url = 0;
    let mut url_ok = 0;
    let mut url_bad = 0;

    for lib in &detail.libraries {
        if !launcher::is_library_allowed(lib) {
            continue;
        }
        if lib.downloads.is_some() {
            with_downloads += 1;
        }
        if lib.url.is_some() {
            with_url += 1;
        }

        if let Some(ref dl) = lib.downloads {
            if let Some(ref artifact) = dl.artifact {
                if !artifact.url.is_empty() {
                    url_ok += 1;
                } else {
                    url_bad += 1;
                }
            } else {
                let url = lib_url_from_name(&lib.name, lib.url.as_deref());
                if !url.is_empty() && url.starts_with("http") {
                    url_ok += 1;
                } else {
                    url_bad += 1;
                    println!(
                        "  WARN: lib {} no artifact, constructed URL: {}",
                        lib.name, url
                    );
                }
            }
        } else {
            let url = lib_url_from_name(&lib.name, lib.url.as_deref());
            if !url.is_empty() && url.starts_with("http") {
                url_ok += 1;
            } else {
                url_bad += 1;
                println!(
                    "  WARN: lib {} no downloads, constructed URL: {}",
                    lib.name, url
                );
            }
        }
    }
    println!(
        "  [LIBS] with_downloads={}, with_url_field={}, url_valid={}, url_invalid={}",
        with_downloads, with_url, url_ok, url_bad
    );
    assert_eq!(url_bad, 0, "b1.7.3 should have no invalid library URLs");

    let resolved = resolve_game_args(&detail, &game_args);
    let unresolved: Vec<_> = resolved.iter().filter(|a| a.contains("${")).collect();
    assert!(
        unresolved.is_empty(),
        "b1.7.3 has unresolved vars: {:?}",
        unresolved
    );

    println!("  >>> b1.7.3 PASSED <<<");
}
