use chrono::Utc;
use luxmc_lib::core::downloader::{DownloadProgress, DownloadSpeed};
use luxmc_lib::db::models::ProfileRow;
use luxmc_lib::db::Db;

#[tokio::test]
async fn test_instance_crud_and_persistence() {
    let db = Db::init().await.expect("Failed to initialize database");
    let test_id = format!("test-inst-{}", uuid::Uuid::new_v4());
    let temp_game_dir = std::env::temp_dir().join(&test_id);

    tokio::fs::create_dir_all(&temp_game_dir)
        .await
        .expect("Failed to create test instance directory");
    tokio::fs::write(temp_game_dir.join("options.txt"), "gamma:1.0\n")
        .await
        .expect("Failed to write options.txt");
    assert!(temp_game_dir.exists(), "Instance folder must exist on disk");

    let now = Utc::now();
    let initial_profile = ProfileRow {
        id: test_id.clone(),
        name: "Instância Teste 1.21.4".to_string(),
        icon: "grass".to_string(),
        mc_version: "1.21.4".to_string(),
        loader: "fabric".to_string(),
        loader_version: Some("0.16.9".to_string()),
        java_path: None,
        jvm_args: Some("-Xmx6G -XX:+UseG1GC".to_string()),
        resolution_w: Some(1280),
        resolution_h: Some(720),
        fullscreen: false,
        game_dir: temp_game_dir.to_string_lossy().to_string(),
        created_at: now,
        updated_at: now,
        favorite: false,
        notes: Some("Notas da instância teste".to_string()),
        last_played: None,
        launch_count: 0,
        mod_count: 5,
        disk_usage: 1024 * 1024 * 50,
        ram_mb: Some(6144),
        instance_group: Some("Modded".to_string()),
        auto_optimize: true,
        use_vulkan: false,
    };

    luxmc_lib::db::schema::profiles::upsert(&db, &initial_profile)
        .await
        .expect("Failed to upsert initial profile");

    let fetched = sqlx::query_as::<_, ProfileRow>("SELECT * FROM profiles WHERE id = ?")
        .bind(&test_id)
        .fetch_optional(db.pool())
        .await
        .expect("Failed to query profile")
        .expect("Profile must exist in DB");

    assert_eq!(fetched.name, "Instância Teste 1.21.4");
    assert_eq!(fetched.icon, "grass");
    assert_eq!(fetched.mc_version, "1.21.4");
    assert_eq!(fetched.loader, "fabric");
    assert_eq!(fetched.ram_mb, Some(6144));
    assert_eq!(fetched.jvm_args.as_deref(), Some("-Xmx6G -XX:+UseG1GC"));

    let updated_now = Utc::now();
    let mut updated_profile = fetched;
    updated_profile.name = "Instância Teste 1.21.4 (Atualizada)".to_string();
    updated_profile.ram_mb = Some(8192);
    updated_profile.jvm_args = Some("-Xmx8G -XX:+UseG1GC -XX:+AlwaysPreTouch".to_string());
    updated_profile.icon = "diamond_block".to_string();
    updated_profile.updated_at = updated_now;

    luxmc_lib::db::schema::profiles::upsert(&db, &updated_profile)
        .await
        .expect("Failed to update profile");

    let fetched_again = sqlx::query_as::<_, ProfileRow>("SELECT * FROM profiles WHERE id = ?")
        .bind(&test_id)
        .fetch_optional(db.pool())
        .await
        .expect("Failed to query profile after update")
        .expect("Profile must exist in DB");

    assert_eq!(fetched_again.name, "Instância Teste 1.21.4 (Atualizada)");
    assert_eq!(fetched_again.ram_mb, Some(8192));
    assert_eq!(
        fetched_again.jvm_args.as_deref(),
        Some("-Xmx8G -XX:+UseG1GC -XX:+AlwaysPreTouch")
    );
    assert_eq!(fetched_again.icon, "diamond_block");

    if temp_game_dir.is_dir() {
        tokio::fs::remove_dir_all(&temp_game_dir)
            .await
            .expect("Failed to delete game_dir from disk");
    }
    luxmc_lib::db::schema::profiles::delete(&db, &test_id)
        .await
        .expect("Failed to delete profile from db");

    assert!(
        !temp_game_dir.exists(),
        "Instance folder must be removed from disk after deletion"
    );

    let deleted_check = sqlx::query_as::<_, ProfileRow>("SELECT * FROM profiles WHERE id = ?")
        .bind(&test_id)
        .fetch_optional(db.pool())
        .await
        .expect("Failed to query deleted profile");

    assert!(
        deleted_check.is_none(),
        "Deleted profile must not exist in SQLite"
    );
}

#[test]
fn test_download_progress_payload_and_calculations() {
    let speed = DownloadSpeed {
        bytes_per_second: 12 * 1024 * 1024,
        total_downloaded: 48 * 1024 * 1024,
        elapsed_ms: 4000,
    };

    let client_progress = DownloadProgress {
        phase: "client".to_string(),
        total: 1,
        completed: 1,
        current_file: "1.21.4.jar".to_string(),
        bytes_downloaded: 25 * 1024 * 1024,
        total_bytes: 25 * 1024 * 1024,
        speed: Some(speed.clone()),
    };

    let json = serde_json::to_string(&client_progress).expect("Must serialize DownloadProgress");
    assert!(json.contains("\"phase\":\"client\""));
    assert!(json.contains("\"currentFile\":\"1.21.4.jar\""));
    assert!(json.contains("\"bytesPerSecond\":12582912"));

    let library_progress = DownloadProgress {
        phase: "libraries".to_string(),
        total: 64,
        completed: 32,
        current_file: "authlib-6.0.55.jar".to_string(),
        bytes_downloaded: 15 * 1024 * 1024,
        total_bytes: 30 * 1024 * 1024,
        speed: Some(speed),
    };

    let lib_json =
        serde_json::to_string(&library_progress).expect("Must serialize library DownloadProgress");
    assert!(lib_json.contains("\"phase\":\"libraries\""));
    assert!(lib_json.contains("\"completed\":32"));
    assert!(lib_json.contains("\"total\":64"));
    assert!(lib_json.contains("\"currentFile\":\"authlib-6.0.55.jar\""));

    let remaining_bytes = library_progress.total_bytes - library_progress.bytes_downloaded;
    let bps = library_progress
        .speed
        .as_ref()
        .map(|s| s.bytes_per_second)
        .unwrap_or(1);
    let eta_seconds = remaining_bytes as f64 / bps as f64;
    assert!(
        (eta_seconds - 1.25).abs() < 0.01,
        "ETA in seconds should be approx 1.25s"
    );
}
