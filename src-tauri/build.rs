fn main() {
    cc::Build::new()
        .cpp(true)
        .std("c++17")
        .file("native/luxmc_core.cpp")
        .compile("luxmc_native");

    println!("cargo:rerun-if-changed=native/luxmc_core.cpp");
    println!("cargo:rerun-if-changed=native/luxmc_core.hpp");

    tauri_build::build();
}
