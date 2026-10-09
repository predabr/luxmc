pub fn arguments(mut args: Vec<String>, version: &str, metadata: &str, host: &str, port: u16) -> Vec<String> {
    let mut index = 0;
    while index < args.len() {
        if matches!(args[index].as_str(), "--server" | "--port" | "--quickPlayMultiplayer" | "--quickPlaySingleplayer" | "--quickPlayRealms") {
            args.remove(index);
            if index < args.len() && !args[index].starts_with("--") { args.remove(index); }
        } else { index += 1; }
    }
    let parts: Vec<_> = version.split('.').filter_map(|value| value.parse::<u32>().ok()).collect();
    let modern = metadata.contains("--quickPlayMultiplayer") || parts.first().is_some_and(|major| *major >= 26) || (parts.first() == Some(&1) && parts.get(1).is_some_and(|minor| *minor >= 20));
    if modern { args.extend(["--quickPlayMultiplayer".into(), format!("{host}:{port}")]); }
    else { args.extend(["--server".into(), host.into(), "--port".into(), port.to_string()]); }
    args
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn chooses_supported_join_protocol_for_modern_and_legacy_clients() {
        for version in ["1.20.1", "1.21.8", "26.3"] { assert_eq!(arguments(Vec::new(), version, "", "127.0.0.1", 24444), vec!["--quickPlayMultiplayer", "127.0.0.1:24444"]); }
        assert_eq!(arguments(Vec::new(), "1.12.2", "", "127.0.0.1", 24444), vec!["--server", "127.0.0.1", "--port", "24444"]);
        assert_eq!(arguments(vec!["--server".into(), "old".into(), "--port".into(), "1234".into()], "23w14a", "--quickPlayMultiplayer", "127.0.0.1", 24444), vec!["--quickPlayMultiplayer", "127.0.0.1:24444"]);
    }
}
