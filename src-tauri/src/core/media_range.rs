pub fn parse_range(value: &str, length: u64) -> Result<(u64, u64), ()> {
    let value = value.trim().strip_prefix("bytes=").ok_or(())?;
    let (start, end) = value.split_once('-').ok_or(())?;
    if length == 0 || value.contains(',') { return Err(()); }
    if start.is_empty() {
        let suffix = end.parse::<u64>().map_err(|_| ())?;
        if suffix == 0 { return Err(()); }
        return Ok((length.saturating_sub(suffix), length - 1));
    }
    let start = start.parse::<u64>().map_err(|_| ())?;
    let end = if end.is_empty() { length - 1 } else { end.parse::<u64>().map_err(|_| ())?.min(length - 1) };
    if start >= length || start > end { return Err(()); }
    Ok((start, end))
}

#[cfg(test)]
mod tests {
    use super::parse_range;
    #[test]
    fn supports_browser_seek_ranges() {
        assert_eq!(parse_range("bytes=0-", 100), Ok((0, 99)));
        assert_eq!(parse_range("bytes=-20", 100), Ok((80, 99)));
        assert_eq!(parse_range("bytes=20-999", 100), Ok((20, 99)));
        for value in ["bytes=100-", "bytes=2-1", "bytes=-0", "bytes=0-1,3-4", "bytes=nope"] {
            assert!(parse_range(value, 100).is_err());
        }
        assert!(parse_range("bytes=0-", 0).is_err());
    }
}
