use crate::error::{AppError, AppResult};

pub(crate) fn parse(input: &str) -> AppResult<Vec<String>> {
    let mut args = Vec::new();
    let mut current = String::new();
    let mut quoted = false;
    let mut started = false;
    let mut chars = input.chars().peekable();
    while let Some(ch) = chars.next() {
        match ch {
            '\\' => {
                let mut count = 1;
                while chars.peek() == Some(&'\\') { chars.next(); count += 1; }
                if chars.peek() == Some(&'"') {
                    chars.next();
                    current.extend(std::iter::repeat_n('\\', count / 2));
                    if count % 2 == 0 { quoted = !quoted; } else { current.push('"'); }
                } else {
                    current.extend(std::iter::repeat_n('\\', count));
                }
                started = true;
            }
            '"' => { quoted = !quoted; started = true; }
            value if value.is_whitespace() && !quoted => {
                if started { args.push(std::mem::take(&mut current)); started = false; }
            }
            value => { current.push(value); started = true; }
        }
    }
    if quoted { return Err(AppError::InvalidInput("Feche as aspas dos argumentos do Java antes de guardar as alterações.".into())); }
    if started { args.push(current); }
    Ok(args)
}

pub(crate) fn format(args: &[String]) -> String {
    args.iter().map(|arg| {
        if !arg.is_empty() && !arg.chars().any(|ch| ch.is_whitespace() || ch == '"') { return arg.clone(); }
        let mut result = String::from("\"");
        let mut slashes = 0;
        for ch in arg.chars() {
            if ch == '\\' { slashes += 1; continue; }
            result.extend(std::iter::repeat_n('\\', if ch == '"' { slashes * 2 + 1 } else { slashes }));
            slashes = 0;
            result.push(ch);
        }
        result.extend(std::iter::repeat_n('\\', slashes * 2));
        result.push('"');
        result
    }).collect::<Vec<_>>().join(" ")
}

pub(super) fn constrain_inherited_heap(args: &mut Vec<String>, ram_mb: i64) {
    let initial = args.iter().rev().find(|arg| arg.starts_with("-Xms")).cloned();
    args.retain(|arg| !arg.starts_with("-Xmx") && !arg.starts_with("-Xms"));
    let maximum = ram_mb as u64 * 1024 * 1024;
    if let Some(mut arg) = initial {
        if let Some(value) = arg.strip_prefix("-Xms") {
            let (number, multiplier) = match value.as_bytes().last().map(u8::to_ascii_lowercase) {
                Some(b'k') => (&value[..value.len() - 1], 1024),
                Some(b'm') => (&value[..value.len() - 1], 1024 * 1024),
                Some(b'g') => (&value[..value.len() - 1], 1024 * 1024 * 1024),
                _ => (value, 1),
            };
            if number.parse::<u64>().ok().and_then(|size| size.checked_mul(multiplier)).is_some_and(|size| size > maximum) {
                arg = format!("-Xms{ram_mb}M");
            }
        }
        args.push(arg);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalization_preserves_spaces_backslashes_quotes_and_empty_arguments() {
        let args = vec![r"-Dpath=C:\Game  Data\natives\".into(), r"-Dunc=\\server\Game Data\".into(), "-Dmessage=He said \"hello\"".into(), "-Dfile.encoding=UTF-8".into(), "".into()];
        assert_eq!(parse(&format(&args)).unwrap(), args);
        assert_eq!(parse(r#"-Dpath="C:\Game  Data\natives" -Xmx4G"#).unwrap(), vec![r"-Dpath=C:\Game  Data\natives", "-Xmx4G"]);
        assert!(parse(r#"-Dpath="C:\Game Data"#).is_err());
    }
}
