use luxmc_lib::core::loaders::neoforge::fetch_versions;
use luxmc_lib::core::loaders::prepare_loader;

#[tokio::test]
async fn test_neoforge_loader_versions_and_prepare() {
    let http = reqwest::Client::builder()
        .user_agent("Luxmc/1.3.0-beta")
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

    let has_installer = prep.classpath_entries.iter().any(|p| {
        p.to_string_lossy().contains("neoforge-20.4.237-installer.jar") && p.exists()
    });
    assert!(has_installer, "NeoForge installer jar must exist on disk");

    let _ = tokio::fs::remove_dir_all(&temp_dir).await;
}
