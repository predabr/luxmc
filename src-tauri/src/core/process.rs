#[cfg(windows)]
pub const CREATE_NO_WINDOW: u32 = 0x08000000;
#[cfg(windows)]
pub const ABOVE_NORMAL_PRIORITY_CLASS: u32 = 0x00008000;

pub fn process_started_at(pid: u32) -> Option<u64> {
    if pid == 0 { return None; }
    #[cfg(windows)]
    { return windows_process::open(pid).ok().and_then(|handle| windows_process::created(&handle)); }
    #[cfg(not(windows))]
    {
    let pid = sysinfo::Pid::from_u32(pid);
    let mut system = sysinfo::System::new();
    system.refresh_processes(sysinfo::ProcessesToUpdate::Some(&[pid]), true);
    system.process(pid).map(|process| process.start_time())
    }
}

#[cfg(windows)]
mod windows_process {
    use std::os::windows::io::{AsRawHandle, FromRawHandle, OwnedHandle};
    #[repr(C)]
    #[derive(Clone, Copy, Default)]
    struct FileTime { low: u32, high: u32 }
    #[link(name = "kernel32")]
    extern "system" {
        fn OpenProcess(access: u32, inherit: i32, pid: u32) -> *mut std::ffi::c_void;
        fn GetProcessTimes(handle: *mut std::ffi::c_void, creation: *mut FileTime, exit: *mut FileTime, kernel: *mut FileTime, user: *mut FileTime) -> i32;
    }
    pub fn open(pid: u32) -> std::io::Result<OwnedHandle> {
        let handle = unsafe { OpenProcess(0x1000, 0, pid) };
        if handle.is_null() { return Err(std::io::Error::last_os_error()); }
        Ok(unsafe { OwnedHandle::from_raw_handle(handle) })
    }
    pub fn created(handle: &OwnedHandle) -> Option<u64> {
        let mut times = [FileTime::default(); 4];
        let success = unsafe { GetProcessTimes(handle.as_raw_handle(), &mut times[0], &mut times[1], &mut times[2], &mut times[3]) };
        (success != 0).then(|| (u64::from(times[0].high) << 32) | u64::from(times[0].low))
    }
}

#[cfg(windows)]
pub fn hold_owned_process(pid: u32, creation: u64) -> std::io::Result<std::os::windows::io::OwnedHandle> {
    let handle = windows_process::open(pid)?;
    if windows_process::created(&handle) != Some(creation) {
        return Err(std::io::Error::new(std::io::ErrorKind::InvalidInput, "A identidade do processo mudou."));
    }
    Ok(handle)
}

pub fn sanitize_diagnostic(text: &str) -> String {
    static SECRET: std::sync::OnceLock<regex::Regex> = std::sync::OnceLock::new();
    let pattern = SECRET.get_or_init(|| regex::Regex::new(r"(?i)(authorization[\s:=\x22]+(?:bearer\s+)?|bearer\s+|(?:access[_-]?token|refresh[_-]?token|session[_-]?id|password)[\s:=\x22]+)[^\s\x22,;]+|eyJ[A-Za-z0-9_-]+\.[A-Za-z0-9_-]+\.[A-Za-z0-9_-]+").unwrap());
    pattern.replace_all(text, "[redacted]").chars().take(4096).collect()
}

pub fn has_visible_window(pid: u32) -> bool {
    if pid == 0 { return false; }
    #[cfg(windows)]
    {
        struct Search { pid: u32, found: bool }
        #[link(name = "user32")]
        extern "system" {
            fn EnumWindows(callback: unsafe extern "system" fn(*mut std::ffi::c_void, isize) -> i32, data: isize) -> i32;
            fn IsWindowVisible(window: *mut std::ffi::c_void) -> i32;
            fn GetWindowThreadProcessId(window: *mut std::ffi::c_void, pid: *mut u32) -> u32;
            fn GetClassNameW(window: *mut std::ffi::c_void, name: *mut u16, capacity: i32) -> i32;
        }
        unsafe extern "system" fn visit(window: *mut std::ffi::c_void, data: isize) -> i32 {
            let search = &mut *(data as *mut Search);
            let mut owner = 0;
            if GetWindowThreadProcessId(window, &mut owner) != 0 && owner == search.pid && IsWindowVisible(window) != 0 {
                let mut class = [0u16; 128];
                let length = GetClassNameW(window, class.as_mut_ptr(), class.len() as i32);
                if length > 0 && is_game_window_class(&String::from_utf16_lossy(&class[..length as usize])) {
                    search.found = true;
                    return 0;
                }
            }
            1
        }
        let mut search = Search { pid, found: false };
        unsafe { EnumWindows(visit, &mut search as *mut Search as isize); }
        search.found
    }
    #[cfg(not(windows))]
    { true }
}

fn is_game_window_class(class: &str) -> bool {
    matches!(class, "GLFW30" | "LWJGL" | "SDL_app")
}

#[inline]
pub fn std_command<S: AsRef<std::ffi::OsStr>>(program: S) -> std::process::Command {
    #[allow(unused_mut)]
    let mut cmd = std::process::Command::new(program.as_ref());
    if is_java(program.as_ref()) { for variable in JAVA_ENVIRONMENT { cmd.env_remove(variable); } }
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
    let mut cmd = tokio::process::Command::new(program.as_ref());
    if is_java(program.as_ref()) { for variable in JAVA_ENVIRONMENT { cmd.env_remove(variable); } }
    #[cfg(windows)]
    {
        cmd.creation_flags(CREATE_NO_WINDOW);
    }
    cmd
}

const JAVA_ENVIRONMENT: [&str; 5] = ["JAVA_TOOL_OPTIONS", "_JAVA_OPTIONS", "JAVA_OPTIONS", "JDK_JAVA_OPTIONS", "CLASSPATH"];

fn is_java(program: &std::ffi::OsStr) -> bool {
    std::path::Path::new(program).file_name().is_some_and(|name| matches!(name.to_string_lossy().to_ascii_lowercase().as_str(), "java" | "java.exe" | "javaw" | "javaw.exe"))
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
    sanitize_diagnostic(line)
}

#[cfg(test)]
mod log_tests {
    #[test]
    fn only_rendering_classes_confirm_a_game_window() {
        assert!(super::is_game_window_class("GLFW30"));
        assert!(super::is_game_window_class("LWJGL"));
        assert!(super::is_game_window_class("SDL_app"));
        for class in ["#32770", "SunAwtFrame", "ConsoleWindowClass", ""] {
            assert!(!super::is_game_window_class(class));
        }
    }

    #[test]
    fn diagnostics_redact_credentials_and_limit_size() {
        let text = super::sanitize_diagnostic("Authorization: Bearer secret access_token=token refresh-token=refresh password:pass");
        for secret in ["secret", "=token", "=refresh", ":pass"] { assert!(!text.contains(secret)); }
        assert!(super::sanitize_diagnostic(&"界".repeat(5000)).chars().count() <= 4096);
    }
    #[test]
    fn java_commands_remove_inherited_jvm_injection_but_other_programs_keep_their_environment() {
        let command = super::std_command("java.exe");
        for variable in super::JAVA_ENVIRONMENT {
            assert!(command.get_envs().any(|(key, value)| key == variable && value.is_none()));
        }
        assert!(!super::std_command("git").get_envs().any(|(key, _)| key == "JAVA_TOOL_OPTIONS"));
    }
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
