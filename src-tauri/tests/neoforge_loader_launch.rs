use luxmc_lib::core::loaders::neoforge::fetch_versions;
use luxmc_lib::core::loaders::prepare_loader;

#[tokio::test]
async fn test_neoforge_loader_versions_and_prepare() {
    let http = reqwest::Client::builder()
        .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/122.0.0.0 Safari/537.36 Luxmc/1.6.5")
        .build()
        .unwrap();

    let versions = fetch_versions(&http, "1.20.4").await.expect("fetch_versions failed");
    assert!(!versions.is_empty(), "Must find NeoForge versions for MC 1.20.4");
    assert!(versions[0].id.starts_with("20.4."), "NeoForge version must match 20.4. prefix");

    let temp_dir = std::env::temp_dir().join(format!("luxmc_neoforge_test_{}", std::process::id()));
    let libraries_dir = temp_dir.join("libraries");
    tokio::fs::create_dir_all(&libraries_dir).await.unwrap();

    let prep = prepare_loader(&http, &libraries_dir, "neoforge", "1.20.4", Some("20.4.237"))
        .await
        .expect("prepare_loader for neoforge failed");

    assert!(
        prep.main_class.contains("BootstrapLauncher") || prep.main_class.contains("neoforge"),
        "NeoForge main_class must be BootstrapLauncher or neoforge launcher, got: {}",
        prep.main_class
    );

    assert!(
        !prep.classpath_entries.is_empty(),
        "Classpath must contain NeoForge libraries"
    );

    let loader_dir = libraries_dir.join("net/neoforged/neoforge/20.4.237");
    for name in ["neoforge-20.4.237-client.jar", "neoforge-20.4.237-universal.jar"] {
        let file = std::fs::File::open(loader_dir.join(name)).expect("NeoForge processor output missing");
        let archive = zip::ZipArchive::new(file).expect("NeoForge processor output is not a valid JAR");
        assert!(!archive.is_empty());
    }
    assert!(prep.game_args.iter().any(|arg| arg == "20.4.237"));
    assert!(prep.classpath_entries.iter().all(|path| path.exists()));

    let _ = tokio::fs::remove_dir_all(&temp_dir).await;
}


#[tokio::test]
async fn modern_neoforge_processors_produce_valid_client() {
    let http = reqwest::Client::builder().user_agent("Luxmc integration tests").build().unwrap();
    let root = std::env::temp_dir().join(format!("luxmc_neoforge21_test_{}", std::process::id()));
    let libraries = root.join("libraries");
    tokio::fs::create_dir_all(&libraries).await.unwrap();
    let prepared = prepare_loader(&http, &libraries, "neoforge", "1.21.1", Some("21.1.248")).await.expect("NeoForge 1.21.1 preparation failed");
    assert!(!prepared.classpath_entries.is_empty());
    assert!(prepared.classpath_entries.iter().all(|path| path.exists()));
    let client = libraries.join("net/neoforged/neoforge/21.1.248/neoforge-21.1.248-client.jar");
    let archive = zip::ZipArchive::new(std::fs::File::open(client).unwrap()).unwrap();
    assert!(!archive.is_empty());
    tokio::fs::remove_dir_all(root).await.unwrap();
}
