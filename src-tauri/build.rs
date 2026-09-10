fn main() {
    let mask: u8 = 0x5e;
    for path in &[".env", "../.env"] {
        if let Ok(content) = std::fs::read_to_string(path) {
            for line in content.lines() {
                let trimmed = line.trim();
                if let Some(val) = trimmed.strip_prefix("CURSEFORGE_API_KEY=") {
                    let cleaned = val.trim().trim_matches('"').trim_matches('\'');
                    if !cleaned.is_empty() {
                        let obfuscated: String = cleaned.bytes().map(|b| format!("{:02x}", b ^ mask)).collect();
                        println!("cargo:rustc-env=CURSEFORGE_KEY_OBFUSCATED={}", obfuscated);
                        println!("cargo:rustc-env=CURSEFORGE_KEY_MASK={}", mask);
                        break;
                    }
                }
            }
        }
    }
    println!("cargo:rerun-if-changed=../.env");
    println!("cargo:rerun-if-changed=.env");
    tauri_build::build()
}
