pub async fn confirm_startup(
    mut exit: tokio::sync::oneshot::Receiver<String>,
    mut ready: impl FnMut() -> bool,
    interval: std::time::Duration,
) -> Result<(), String> {
    let mut consecutive = 0;
    loop {
        tokio::select! {
            biased;
            result = &mut exit => return Err(result.unwrap_or_else(|_| "O monitor do Minecraft encerrou antes de confirmar o início.".into())),
            _ = tokio::time::sleep(interval) => {
                consecutive = if ready() { consecutive + 1 } else { 0 };
                if consecutive >= 2 { return Ok(()); }
            }
        }
    }
}

pub(super) fn supports_lwjgl_overlay<'a>(libraries: impl Iterator<Item = &'a str>) -> bool {
    !libraries.into_iter().any(|name| name.starts_with("org.lwjgl:lwjgl-sdl:"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sdl_clients_keep_appearance_and_lan_without_starting_glfw_hooks() {
        assert!(supports_lwjgl_overlay(["org.lwjgl.lwjgl:lwjgl:2.9.4"].into_iter()));
        assert!(supports_lwjgl_overlay(["org.lwjgl:lwjgl-glfw:3.3.3"].into_iter()));
        assert!(!supports_lwjgl_overlay(["org.lwjgl:lwjgl:3.4.3", "org.lwjgl:lwjgl-sdl:3.4.3"].into_iter()));
    }

    #[test]
    fn counter_fallback_only_applies_to_missing_counters_and_respects_explicit_options() {
        let mut args = vec!["-Xmx4096M".into()];
        append_counter_fallback(&mut args, false);
        assert_eq!(args, ["-Xmx4096M"]);
        append_counter_fallback(&mut args, true);
        assert!(args.contains(&"-Doshi.os.windows.hkeyperfdata=false".into()));
        append_counter_fallback(&mut args, true);
        assert_eq!(args.len(), 2);
        let mut explicit = vec!["-Doshi.os.windows.hkeyperfdata=true".into(), "-Doshi.os.windows.perfproc.disabled=false".into()];
        append_counter_fallback(&mut explicit, true);
        assert_eq!(explicit.len(), 2);
    }

    #[tokio::test]
    async fn early_exit_is_not_reported_as_a_successful_launch() {
        let (sender, receiver) = tokio::sync::oneshot::channel();
        sender.send("Java incompatível".into()).unwrap();
        assert_eq!(confirm_startup(receiver, || true, std::time::Duration::from_millis(1)).await, Err("Java incompatível".into()));
    }

    #[tokio::test]
    async fn readiness_requires_a_stable_game_window() {
        let (sender, receiver) = tokio::sync::oneshot::channel();
        let mut calls = 0;
        assert!(confirm_startup(receiver, || { calls += 1; matches!(calls, 1 | 3 | 4) }, std::time::Duration::from_millis(1)).await.is_ok());
        assert_eq!(calls, 4);
        assert!(sender.send("Encerrado mais tarde".into()).is_err());
    }

    #[tokio::test]
    async fn a_lost_monitor_is_not_a_successful_launch() {
        let (sender, receiver) = tokio::sync::oneshot::channel();
        drop(sender);
        assert!(confirm_startup(receiver, || false, std::time::Duration::from_millis(1)).await.is_err());
    }

    #[tokio::test]
    async fn a_living_process_without_a_game_window_is_not_success() {
        let (sender, receiver) = tokio::sync::oneshot::channel();
        let attempt = confirm_startup(receiver, || false, std::time::Duration::from_millis(1));
        assert!(tokio::time::timeout(std::time::Duration::from_millis(20), attempt).await.is_err());
        drop(sender);
    }
}
pub(super) fn append_counter_fallback(args: &mut Vec<String>, unavailable: bool) {
    if !unavailable { return; }
    for name in ["oshi.os.windows.hkeyperfdata"] {
        if !args.iter().any(|arg| arg.starts_with(&format!("-D{name}="))) {
            args.push(format!("-D{name}=false"));
        }
    }
}

#[cfg(windows)]
pub(super) fn windows_counters_unavailable() -> bool {
    static UNAVAILABLE: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
    *UNAVAILABLE.get_or_init(|| {
        winreg::RegKey::predef(winreg::enums::HKEY_LOCAL_MACHINE)
            .open_subkey("SOFTWARE\\Microsoft\\Windows NT\\CurrentVersion\\Perflib\\009")
            .and_then(|key| key.get_value::<Vec<String>, _>("Counter"))
            .map_or(true, |names| names.is_empty())
    })
}
