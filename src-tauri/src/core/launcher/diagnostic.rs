use std::{io::{Read, Seek, SeekFrom}, path::Path};

pub(super) fn warning(line: &str) -> bool {
    let line = line.trim();
    line.starts_with("WARNING:") || line.contains("[ALSOFT]")
        || line.contains("Failed to set real-time priority") || line.contains("AL_SOFT")
        || line.contains("MESA-LOADER") || line.contains("libEGL warning")
}

pub(super) fn read_tail(path: &Path) -> std::io::Result<String> {
    let mut file = std::fs::File::open(path)?;
    let length = file.metadata()?.len();
    let offset = length.saturating_sub(256 * 1024);
    file.seek(SeekFrom::Start(offset))?;
    let mut bytes = Vec::new();
    file.take(256 * 1024).read_to_end(&mut bytes)?;
    let text = String::from_utf8_lossy(&bytes);
    Ok(if offset > 0 { text.split_once('\n').map_or("", |(_, tail)| tail) } else { &text }.to_owned())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn warnings_do_not_replace_crash_causes() {
        assert!(warning("WARNING: sun.misc.Unsafe::putFloat will be removed in a future release"));
        assert!(!warning("java.lang.OutOfMemoryError: Java heap space"));
        assert!(!warning("Error: Could not create the Java Virtual Machine."));
    }
    #[test]
    fn large_log_reads_keep_the_bounded_tail() {
        let path = std::env::temp_dir().join(format!("luxmc-tail-{}.log", uuid::Uuid::new_v4()));
        std::fs::write(&path, format!("{}\nCaused by: actual crash\n", "x".repeat(300 * 1024))).unwrap();
        let text = read_tail(&path).unwrap();
        std::fs::remove_file(path).unwrap();
        assert_eq!(text, "Caused by: actual crash\n");
    }
}
