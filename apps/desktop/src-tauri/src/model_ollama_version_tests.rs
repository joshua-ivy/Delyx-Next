#[cfg(test)]
mod tests {
    use crate::model_ollama_version::{parse_ollama_version, version_from_http_result};

    #[test]
    fn parses_trimmed_ollama_version() {
        assert_eq!(
            parse_ollama_version(r#"{"version":" 0.12.6 "}"#).unwrap(),
            Some("0.12.6".to_string())
        );
    }

    #[test]
    fn ignores_missing_or_failed_version() {
        assert_eq!(parse_ollama_version(r#"{"version":""}"#).unwrap(), None);
        assert_eq!(version_from_http_result(Ok((500, "{}".to_string()))), None);
        assert_eq!(version_from_http_result(Err("offline".to_string())), None);
    }
}
