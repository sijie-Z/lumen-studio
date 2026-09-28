use crate::AppError;

pub fn max_chars(value: Option<&str>, max: usize, field: &str) -> Result<(), AppError> {
    let length = value.map(str::trim).map(str::chars).map(Iterator::count);
    if length.is_some_and(|length| length > max) {
        return Err(AppError::BadRequest(format!(
            "{field} must be at most {max} characters"
        )));
    }
    Ok(())
}

pub fn escape_like_pattern(value: &str) -> String {
    value
        .replace('\\', "\\\\")
        .replace('%', "\\%")
        .replace('_', "\\_")
}

#[cfg(test)]
mod tests {
    use super::{escape_like_pattern, max_chars};
    use crate::AppError;

    #[test]
    fn max_chars_allows_unicode_within_limit() {
        assert!(max_chars(Some("摄影服务"), 4, "title").is_ok());
        assert!(matches!(
            max_chars(Some("摄影服务"), 3, "title"),
            Err(AppError::BadRequest(_))
        ));
    }

    #[test]
    fn like_pattern_escapes_wildcards_and_backslashes() {
        assert_eq!(escape_like_pattern(r"100%_done\"), r"100\%\_done\\");
    }
}
