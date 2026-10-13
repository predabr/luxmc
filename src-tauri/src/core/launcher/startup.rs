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

#[cfg(test)]
mod tests {
    use super::*;

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
