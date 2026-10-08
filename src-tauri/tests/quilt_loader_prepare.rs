use luxmc_lib::core::loaders::prepare_loader;
use std::time::{Duration, Instant};

#[tokio::test]
async fn quilt_real_download_and_offline_warm_preparation() {
    let root = std::env::temp_dir().join(format!("luxmc-quilt-real-{}", uuid::Uuid::new_v4()));
    let libraries = root.join("libraries");
    tokio::fs::create_dir_all(&libraries).await.unwrap();
    let http = reqwest::Client::builder().timeout(Duration::from_secs(90)).build().unwrap();
    let cold_start = Instant::now();
    let cold = prepare_loader(&http, &libraries, "quilt", "1.20.1", Some("0.28.1")).await.unwrap();
    let cold_time = cold_start.elapsed();
    assert_eq!(cold.main_class, "org.quiltmc.loader.impl.launch.knot.KnotClient");
    assert!(!cold.classpath_entries.is_empty());
    for path in &cold.classpath_entries {
        let jar = zip::ZipArchive::new(std::fs::File::open(path).unwrap()).unwrap();
        assert!(!jar.is_empty());
    }
    let offline = reqwest::Client::builder().proxy(reqwest::Proxy::all("http://127.0.0.1:1").unwrap()).timeout(Duration::from_millis(100)).build().unwrap();
    let warm_start = Instant::now();
    let warm = prepare_loader(&offline, &libraries, "quilt", "1.20.1", Some("0.28.1")).await.unwrap();
    assert_eq!(warm.classpath_entries, cold.classpath_entries);
    assert_eq!(warm.main_class, cold.main_class);
    println!("Quilt preparation: cold={cold_time:?}, warm={:?}, libraries={}", warm_start.elapsed(), warm.classpath_entries.len());
}
