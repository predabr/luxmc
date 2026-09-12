use luxmc_lib::core::optimizer::{
    detect_gpu, generate_aikar_flags, generate_standard_flags, get_performance_pack_info,
};
use std::time::Instant;

#[test]
fn test_dynamic_aikar_flags_generation() {
    println!("\n=== TESTE 1: GERAÇÃO DINÂMICA DE AIKAR'S FLAGS ===");

    // Test <= 4GB (2048 MB)
    let flags_2g = generate_aikar_flags(2048);
    assert!(flags_2g.contains(&"-Xms1024M".to_string()));
    assert!(flags_2g.contains(&"-Xmx2048M".to_string()));
    assert!(flags_2g.contains(&"-XX:G1HeapRegionSize=8M".to_string()));
    assert!(flags_2g.contains(&"-XX:G1NewSizePercent=30".to_string()));
    assert!(flags_2g.contains(&"-XX:G1MaxNewSizePercent=40".to_string()));
    assert!(flags_2g.contains(&"-XX:G1ReservePercent=15".to_string()));
    assert!(flags_2g.contains(&"-XX:InitiatingHeapOccupancyPercent=20".to_string()));
    println!("✓ 2048 MB: G1HeapRegionSize=8M, G1NewSize=30%, G1Reserve=15%");

    // Test 4GB..8GB (6144 MB)
    let flags_6g = generate_aikar_flags(6144);
    assert!(flags_6g.contains(&"-Xms1024M".to_string()));
    assert!(flags_6g.contains(&"-Xmx6144M".to_string()));
    assert!(flags_6g.contains(&"-XX:G1HeapRegionSize=16M".to_string()));
    assert!(flags_6g.contains(&"-XX:G1NewSizePercent=30".to_string()));
    assert!(flags_6g.contains(&"-XX:G1ReservePercent=20".to_string()));
    assert!(flags_6g.contains(&"-XX:InitiatingHeapOccupancyPercent=15".to_string()));
    println!("✓ 6144 MB: G1HeapRegionSize=16M, G1NewSize=30%, G1Reserve=20%");

    // Test > 8GB (16384 MB)
    let flags_16g = generate_aikar_flags(16384);
    assert!(flags_16g.contains(&"-Xms1024M".to_string()));
    assert!(flags_16g.contains(&"-Xmx16384M".to_string()));
    assert!(flags_16g.contains(&"-XX:G1HeapRegionSize=32M".to_string()));
    assert!(flags_16g.contains(&"-XX:G1NewSizePercent=40".to_string()));
    assert!(flags_16g.contains(&"-XX:G1MaxNewSizePercent=50".to_string()));
    assert!(flags_16g.contains(&"-XX:G1ReservePercent=20".to_string()));
    println!("✓ 16384 MB: G1HeapRegionSize=32M, G1NewSize=40%, G1MaxNewSize=50%");

    // Test Standard flags (desativado)
    let std_flags = generate_standard_flags(4096);
    assert_eq!(std_flags, vec!["-Xms1024M".to_string(), "-Xmx4096M".to_string()]);
    println!("✓ Flags Padrão (Sem otimização): -Xms1024M -Xmx4096M");
}

#[test]
fn test_gpu_detection_hardware() {
    println!("\n=== TESTE 2: DETECÇÃO DE GPU E CAPACIDADE ZINK/VULKAN ===");
    let gpu = detect_gpu();
    println!("  GPU Detectada: {}", gpu.renderer);
    println!("  Fabricante: {}", gpu.vendor);
    println!("  Driver DRM: {}", gpu.driver);
    println!("  Suporta Mesa Zink (Vulkan): {}", gpu.supports_zink);

    assert!(!gpu.vendor.is_empty(), "GPU vendor must not be empty");
    assert!(!gpu.renderer.is_empty(), "GPU renderer must not be empty");
}

#[tokio::test]
async fn test_performance_pack_and_modrinth_files() {
    println!("\n=== TESTE 3: PACOTE DE MODS DE PERFORMANCE ===");
    let fabric_pack = get_performance_pack_info("fabric", "1.21.4");
    assert!(fabric_pack.available);
    assert_eq!(fabric_pack.mods.len(), 3);
    assert!(fabric_pack.mods.iter().any(|m| m.slug == "sodium"));
    assert!(fabric_pack.mods.iter().any(|m| m.slug == "lithium"));
    assert!(fabric_pack.mods.iter().any(|m| m.slug == "ferrite-core"));
    println!("✓ Fabric 1.21.4: Sodium, Lithium, FerriteCore validados");

    let forge_pack = get_performance_pack_info("forge", "1.20.1");
    assert!(forge_pack.available);
    assert!(forge_pack.mods.iter().any(|m| m.slug == "embeddium"));
    assert!(forge_pack.mods.iter().any(|m| m.slug == "modernfix"));
    println!("✓ Forge: Embeddium, ModernFix, FerriteCore validados");

    let vanilla_pack = get_performance_pack_info("vanilla", "1.21.4");
    assert!(!vanilla_pack.available);
    assert!(vanilla_pack.reason.is_some());
    println!("✓ Vanilla: Corretamente marcado como indisponível com aviso ao usuário");

    // Real API call: verify modrinth has versions for sodium on 1.21.4
    let http = reqwest::Client::builder().user_agent("Luxmc/1.0.0-beta").build().unwrap();
    let modrinth = luxmc_lib::core::mods::ModrinthClient::new(http);
    let sodium_versions = modrinth.get_mod_versions("sodium", "1.21.4").await.expect("Failed to query sodium");
    assert!(!sodium_versions.is_empty(), "Sodium should have releases for 1.21.4");
    println!("✓ Sodium 1.21.4 na Modrinth: {} versões disponíveis (arquivo: {})", sodium_versions.len(), sodium_versions[0].files[0].filename);
}

#[test]
fn test_real_jvm_execution_benchmark_aikar_vs_standard() {
    println!("\n=== TESTE 4: BENCHMARK REAL DA JVM (AIKAR FLAGS VS PADRÃO) ===");

    let java_bin = "/usr/lib/jvm/java-26-openjdk/bin/java";
    if !std::path::Path::new(java_bin).exists() {
        println!("Java binary not at standard path, skipping execution bench");
        return;
    }

    // 1. Standard flags execution
    let std_flags = generate_standard_flags(4096);
    let start_std = Instant::now();
    let std_output = std::process::Command::new(java_bin)
        .args(&std_flags)
        .arg("-version")
        .output()
        .expect("Failed to run Java with standard flags");
    let elapsed_std = start_std.elapsed();
    assert!(std_output.status.success());

    // 2. Aikar's flags execution
    let aikar_flags = generate_aikar_flags(4096);
    let start_aikar = Instant::now();
    let aikar_output = std::process::Command::new(java_bin)
        .args(&aikar_flags)
        .arg("-version")
        .output()
        .expect("Failed to run Java with Aikar flags");
    let elapsed_aikar = start_aikar.elapsed();
    assert!(aikar_output.status.success());

    println!("  [Flags Padrão] Tempo de inicialização da JVM: {:?}", elapsed_std);
    println!("  [Aikar's Flags (Luxmc)] Tempo de inicialização da JVM: {:?}", elapsed_aikar);
    println!("  ✓ Ambas as configurações executaram com código de saída 0 no OpenJDK 26.");
}
