use luxmc_lib::core::loaders::forge::fetch_versions;
use luxmc_lib::core::loaders::prepare_loader;

#[tokio::test]
async fn test_forge_loader_versions_and_prepare() {
    let http = reqwest::Client::builder()
        .user_agent("Luxmc/1.3.0-beta")
        .build()
        .unwrap();

    let versions = fetch_versions(&http, "1.20.1").await.expect("fetch_versions failed");
    assert!(!versions.is_empty(), "Must find Forge versions for MC 1.20.1");
    assert!(
        versions.iter().any(|v| v.id.starts_with("47.")),
        "Forge version must match 47. prefix for MC 1.20.1"
    );

    let temp_dir = std::env::temp_dir().join(format!("luxmc_forge_test_{}", std::process::id()));
    let libraries_dir = temp_dir.join("libraries");
    tokio::fs::create_dir_all(&libraries_dir).await.unwrap();

    let prep = prepare_loader(&http, &libraries_dir, "forge", "1.20.1", Some("47.4.20"))
        .await
        .expect("prepare_loader for forge failed");

    assert!(
        prep.main_class.contains("BootstrapLauncher") || prep.main_class.contains("forge"),
        "Forge main_class must be BootstrapLauncher, got: {}",
        prep.main_class
    );

    assert!(
        !prep.classpath_entries.is_empty(),
        "Classpath must contain Forge libraries"
    );

    let has_installer = prep.classpath_entries.iter().any(|p| {
        p.to_string_lossy().contains("forge-1.20.1-47.4.20-installer.jar") && p.exists()
    });
    assert!(has_installer, "Forge installer jar must exist on disk");

    let _ = tokio::fs::remove_dir_all(&temp_dir).await;
}
