fn main() {
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("windows") {
        let mut resource = tauri_winres::WindowsResource::new();
        resource
            .set("ProductName", "Luxmc")
            .set("FileDescription", "Luxmc update recovery")
            .set("CompanyName", "Luxmc")
            .set("OriginalFilename", "luxmc-repair.exe")
            .set("LegalCopyright", "Copyright 2026 Pedro (predabr) and Luxmc contributors");
        resource.compile().expect("failed to build Luxmc recovery version information");
    }
}
