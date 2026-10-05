use std::io::Write;
use std::path::PathBuf;

const MAX_LOG_BYTES: u64 = 1_048_576;

fn panic_log_path() -> Option<PathBuf> {
    let dirs = directories::ProjectDirs::from("io", "github", "Luxmc")?;
    let dir = dirs.data_dir().join("logs");
    std::fs::create_dir_all(&dir).ok()?;
    Some(dir.join("panic.log"))
}

fn payload_message(info: &std::panic::PanicHookInfo<'_>) -> String {
    let payload = info.payload();
    if let Some(message) = payload.downcast_ref::<&str>() {
        (*message).to_string()
    } else if let Some(message) = payload.downcast_ref::<String>() {
        message.clone()
    } else {
        "panic sem mensagem".to_string()
    }
}

fn append_entry(message: &str, location: &str) {
    let Some(path) = panic_log_path() else { return; };
    append_entry_at(&path, message, location);
}

fn append_entry_at(path: &std::path::Path, message: &str, location: &str) {
    if let Ok(meta) = std::fs::metadata(path) {
        if meta.len() >= MAX_LOG_BYTES {
            let _ = std::fs::rename(&path, path.with_extension("log.1"));
        }
    }

    let entry = format!(
        "\n=== panic {} ===\n{}\nlocal: {}\nbacktrace:\n{}\n",
        chrono::Utc::now().to_rfc3339(),
        message,
        location,
        std::backtrace::Backtrace::force_capture()
    );

    if let Ok(mut file) = std::fs::OpenOptions::new().create(true).append(true).open(path) {
        let _ = file.write_all(entry.as_bytes());
        let _ = file.flush();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn writes_and_rotates_panic_entries() {
        let dir = std::env::temp_dir().join(format!("luxmc-panic-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("panic.log");

        append_entry_at(&path, "mensagem de teste", "arquivo.rs:1:1");
        let content = std::fs::read_to_string(&path).unwrap();
        assert!(content.contains("mensagem de teste"));
        assert!(content.contains("arquivo.rs:1:1"));

        let oversized = "x".repeat((MAX_LOG_BYTES + 16) as usize);
        std::fs::write(&path, oversized).unwrap();
        append_entry_at(&path, "após rotação", "outro.rs:2:2");
        assert!(std::fs::read_to_string(dir.join("panic.log.1")).unwrap().len() >= MAX_LOG_BYTES as usize);
        assert!(std::fs::read_to_string(&path).unwrap().contains("após rotação"));

        std::fs::remove_dir_all(&dir).unwrap();
    }
}

pub fn install() {
    let default_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        let location = info
            .location()
            .map(|l| format!("{}:{}:{}", l.file(), l.line(), l.column()))
            .unwrap_or_else(|| "desconhecida".to_string());
        append_entry(&payload_message(info), &location);
        default_hook(info);
    }));
}
