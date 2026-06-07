#[cfg(test)]
mod tests {
    use crate::model_provider::{
        ModelProviderError, ModelRegistry, ModelRole, ProviderStatus, SecretPolicy,
    };

    #[test]
    fn mock_provider_works_deterministically() {
        let registry = ModelRegistry::with_default_local(10);

        let first = registry.mock_complete(ModelRole::Coding, "review this patch").unwrap();
        let second = registry.mock_complete(ModelRole::Coding, "review this patch").unwrap();

        assert_eq!(first, second);
        assert_eq!(first.provider_id, "mock-local");
        assert_eq!(first.model_id, "delyx-mock-coder");
        assert!(first.text.starts_with("mock:delyx-mock-coder:"));
    }

    #[test]
    fn provider_health_surfaces_missing_api_key() {
        let registry = ModelRegistry::with_default_local(10);

        let health = registry.health("openai-compatible").unwrap();

        assert_eq!(health.status, ProviderStatus::MissingApiKey);
        assert!(health.message.contains("API key is missing"));
    }

    #[test]
    fn role_routing_can_be_saved() {
        let mut registry = ModelRegistry::with_default_local(10);

        registry.save_role_route(ModelRole::Answer, "mock-local", "delyx-mock-reasoner").unwrap();
        let route = registry.route_for(ModelRole::Answer).unwrap();

        assert_eq!(route.provider_id, "mock-local");
        assert_eq!(route.model_id, "delyx-mock-reasoner");
    }

    #[test]
    fn unknown_model_route_is_rejected() {
        let mut registry = ModelRegistry::with_default_local(10);

        let result = registry.save_role_route(ModelRole::Coding, "mock-local", "missing-model");

        assert_eq!(result.unwrap_err(), ModelProviderError::ModelNotFound);
    }

    #[test]
    fn secrets_are_not_stored_in_provider_config() {
        let registry = ModelRegistry::with_default_local(10);

        let provider = registry
            .list_providers()
            .iter()
            .find(|provider| provider.id == "openai-compatible")
            .unwrap();

        assert_eq!(provider.secret_policy, SecretPolicy::ExternalSecretOnly);
        assert!(provider.health.message.contains("outside the repo"));
    }
}
