#[cfg(test)]
mod tests {
    use crate::{
        model_provider::{ModelInfo, ModelProvider, ModelRegistry, ProviderHealth, ProviderKind, ProviderStatus, SecretPolicy},
        runtime_bridge::runtime_status_from_registry,
    };

    #[test]
    fn runtime_status_from_runtime_defaults_exposes_no_mock_route() {
        let status = runtime_status_from_registry(&ModelRegistry::with_runtime_defaults(10));

        assert_eq!(status.app_name, "Delyx Next");
        assert_eq!(status.app_identifier, "com.geaux.delyxnext");
        assert!(status.coding_route.is_none());
        assert!(!status.providers.iter().any(|provider| provider.id == "mock-local"));
        assert!(status.providers.iter().any(|provider| provider.id == "ollama-local"));
    }

    #[test]
    fn runtime_status_maps_ready_ollama_models() {
        let mut registry = ModelRegistry::with_runtime_defaults(10);
        registry.register_provider(ready_ollama_provider());
        let status = runtime_status_from_registry(&registry);
        let ollama = status.providers.iter().find(|provider| provider.id == "ollama-local").unwrap();

        assert_eq!(ollama.status, "ready");
        assert_eq!(ollama.models, vec!["qwen:latest"]);
    }

    #[test]
    fn runtime_status_maps_ready_ollama_coding_route() {
        let mut registry = ModelRegistry::with_runtime_defaults(10);
        registry.register_provider(ready_ollama_provider());
        registry
            .save_role_route(crate::model_provider::ModelRole::Coding, "ollama-local", "qwen:latest")
            .unwrap();
        let status = runtime_status_from_registry(&registry);
        let route = status.coding_route.unwrap();

        assert_eq!(route.provider_id, "ollama-local");
        assert_eq!(route.model_id, "qwen:latest");
    }

    fn ready_ollama_provider() -> ModelProvider {
        ModelProvider {
            health: ProviderHealth { checked_at: 11, message: "Ollama ready.".to_string(), status: ProviderStatus::Ready },
            id: "ollama-local".to_string(),
            kind: ProviderKind::Ollama,
            label: "Ollama".to_string(),
            models: vec![ModelInfo {
                context_window: 32768,
                display_name: "Qwen".to_string(),
                id: "qwen:latest".to_string(),
                supports_tools: true,
            }],
            secret_policy: SecretPolicy::NoSecretRequired,
        }
    }
}
