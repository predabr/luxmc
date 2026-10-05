fn main() {
    cc::Build::new()
        .cpp(true)
        .std("c++17")
        .file("native/luxmc_core.cpp")
        .compile("luxmc_native");

    println!("cargo:rerun-if-changed=native/luxmc_core.cpp");
    println!("cargo:rerun-if-changed=native/luxmc_core.hpp");
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("windows") {
        println!("cargo:rustc-link-lib=psapi");
        println!("cargo:rustc-link-arg=/MANIFEST:EMBED");
        println!("cargo:rustc-link-arg=/MANIFESTDEPENDENCY:type='win32' name='Microsoft.Windows.Common-Controls' version='6.0.0.0' processorArchitecture='*' publicKeyToken='6595b64144ccf1df' language='*'");
    }

    tauri_build::try_build(tauri_build::Attributes::new().windows_attributes(tauri_build::WindowsAttributes::new_without_app_manifest())).expect("failed to build Tauri resources");
}

