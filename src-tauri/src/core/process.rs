#[cfg(windows)]
pub const CREATE_NO_WINDOW: u32 = 0x08000000;
#[cfg(windows)]
pub const ABOVE_NORMAL_PRIORITY_CLASS: u32 = 0x00008000;

#[inline]
pub fn std_command<S: AsRef<std::ffi::OsStr>>(program: S) -> std::process::Command {
    #[allow(unused_mut)]
    let mut cmd = std::process::Command::new(program);
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        cmd.creation_flags(CREATE_NO_WINDOW);
    }
    cmd
}

#[inline]
pub fn tokio_command<S: AsRef<std::ffi::OsStr>>(program: S) -> tokio::process::Command {
    #[allow(unused_mut)]
    let mut cmd = tokio::process::Command::new(program);
    #[cfg(windows)]
    {
        cmd.creation_flags(CREATE_NO_WINDOW);
    }
    cmd
}

pub async fn read_log_line<R: tokio::io::AsyncBufRead + Unpin>(reader: &mut R) -> std::io::Result<Option<String>> {
    use tokio::io::AsyncBufReadExt;
    let mut line = Vec::with_capacity(2048);
    let mut saw_data = false;
    loop {
        let buffer = reader.fill_buf().await?;
        if buffer.is_empty() {
            return Ok(saw_data.then(|| redact_log_line(&String::from_utf8_lossy(&line))));
        }
        saw_data = true;
        let newline = buffer.iter().position(|byte| *byte == b'\n');
        let available = newline.unwrap_or(buffer.len());
        let take = available.min(2048 - line.len());
        line.extend_from_slice(&buffer[..take]);
        reader.consume(available + usize::from(newline.is_some()));
        if newline.is_some() { return Ok(Some(redact_log_line(String::from_utf8_lossy(&line).trim_end_matches('\r')))); }
    }
}

fn redact_log_line(line: &str) -> String {
    if let Some(position) = line.find("(Session ID is ") {
        return format!("{}(Session ID is [redacted])", &line[..position]);
    }
    line.to_owned()
}

#[cfg(test)]
mod log_tests {
    #[tokio::test]
    async fn legacy_session_tokens_are_removed_before_logging() {
        let mut reader = tokio::io::BufReader::new(&b"[INFO]: (Session ID is token:secret:uuid)\nnext\n"[..]);
        let line = super::read_log_line(&mut reader).await.unwrap().unwrap();
        assert_eq!(line, "[INFO]: (Session ID is [redacted])");
        assert_eq!(super::read_log_line(&mut reader).await.unwrap().unwrap(), "next");
    }
    #[tokio::test]
    async fn oversized_unicode_lines_are_bounded_and_next_line_survives() {
        let input = format!("{}\nnext\n", "界".repeat(100000));
        let mut reader = tokio::io::BufReader::new(input.as_bytes());
        let line = super::read_log_line(&mut reader).await.unwrap().unwrap();
        assert!(line.len() <= 2050);
        assert_eq!(super::read_log_line(&mut reader).await.unwrap().as_deref(), Some("next"));
        assert!(super::read_log_line(&mut reader).await.unwrap().is_none());
    }
}
