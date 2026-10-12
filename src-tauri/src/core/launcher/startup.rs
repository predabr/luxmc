pub async fn confirm_startup(
    mut exit: tokio::sync::oneshot::Receiver<String>,
    grace: std::time::Duration,
) -> Result<(), String> {
    match tokio::time::timeout(grace, &mut exit).await {
        Ok(Ok(reason)) => Err(reason),
        Ok(Err(_)) => Err("O monitor do Minecraft encerrou antes de confirmar o início.".into()),
        Err(_) => Ok(()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn early_exit_is_not_reported_as_a_successful_launch() {
        let (sender, receiver) = tokio::sync::oneshot::channel();
        sender.send("Java incompatível".into()).unwrap();
        assert_eq!(confirm_startup(receiver, std::time::Duration::from_millis(30)).await, Err("Java incompatível".into()));
    }

    #[tokio::test]
    async fn a_living_process_finishes_the_grace_period_without_waiting_for_game_exit() {
        let (sender, receiver) = tokio::sync::oneshot::channel();
        assert!(confirm_startup(receiver, std::time::Duration::from_millis(20)).await.is_ok());
        assert!(sender.send("Encerrado mais tarde".into()).is_err());
    }

    #[tokio::test]
    async fn a_lost_monitor_is_not_a_successful_launch() {
        let (sender, receiver) = tokio::sync::oneshot::channel();
        drop(sender);
        assert!(confirm_startup(receiver, std::time::Duration::from_millis(20)).await.is_err());
    }
}
