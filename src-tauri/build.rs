fn main() {
    println!("cargo:rerun-if-env-changed=LUXMC_BUILD_ID");
    let build_id = std::env::var("LUXMC_BUILD_ID").unwrap_or_default();
    let revision = build_id.split_once('.').and_then(|(run,attempt)| {
        let run = run.parse::<u64>().ok()?;
        let attempt = attempt.parse::<u64>().ok().filter(|value|*value<1000)?;
        run.checked_mul(1000)?.checked_add(attempt).filter(|value|*value<=9_007_199_254_740_991)
    }).unwrap_or(0);
    println!("cargo:rustc-env=LUXMC_BUILD_REVISION={revision}");
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("windows") {
        println!("cargo:rustc-link-lib=psapi");
        println!("cargo:rustc-link-arg=/MANIFEST:EMBED");
        println!("cargo:rustc-link-arg=/MANIFESTDEPENDENCY:type='win32' name='Microsoft.Windows.Common-Controls' version='6.0.0.0' processorArchitecture='*' publicKeyToken='6595b64144ccf1df' language='*'");
    }

    tauri_build::try_build(tauri_build::Attributes::new().windows_attributes(tauri_build::WindowsAttributes::new_without_app_manifest())).expect("failed to build Tauri resources");
}

