#[cfg(test)]
mod tests {
    use crate::secret_bridge::{
        clear_secret_record, read_provider_secret, secret_status_record, set_secret_record,
    };
    use crate::secret_store::MemorySecretStore;

    #[test]
    fn status_starts_empty_for_known_providers() {
        let store = MemorySecretStore::default();

        let status = secret_status_record(&store).unwrap();

        let ids: Vec<&str> = status.providers.iter().map(|p| p.id.as_str()).collect();
        assert_eq!(ids, vec!["anthropic", "openai"]);
        assert!(status.providers.iter().all(|p| !p.has_key));
    }

    #[test]
    fn set_marks_key_present_without_returning_the_secret() {
        let store = MemorySecretStore::default();

        let status = set_secret_record(&store, "anthropic", "  sk-secret-123  ").unwrap();

        let anthropic = status
            .providers
            .iter()
            .find(|p| p.id == "anthropic")
            .unwrap();
        assert!(anthropic.has_key);
        let openai = status.providers.iter().find(|p| p.id == "openai").unwrap();
        assert!(!openai.has_key);
        // The status view exposes only booleans; the secret itself is trimmed and stored.
        assert_eq!(
            read_provider_secret(&store, "anthropic").unwrap(),
            Some("sk-secret-123".to_string())
        );
    }

    #[test]
    fn clear_removes_the_key() {
        let store = MemorySecretStore::default();
        set_secret_record(&store, "openai", "sk-openai").unwrap();

        let status = clear_secret_record(&store, "openai").unwrap();

        let openai = status.providers.iter().find(|p| p.id == "openai").unwrap();
        assert!(!openai.has_key);
        assert_eq!(read_provider_secret(&store, "openai").unwrap(), None);
    }

    #[test]
    fn empty_key_and_unknown_provider_are_rejected() {
        let store = MemorySecretStore::default();

        assert!(set_secret_record(&store, "anthropic", "   ")
            .unwrap_err()
            .contains("empty"));
        assert!(set_secret_record(&store, "gemini", "key")
            .unwrap_err()
            .contains("Unknown provider"));
    }
}
