#[cfg(test)]
mod tests {
    use crate::{
        model_provider::{ModelInfo, ModelProvider, ModelRegistry, ProviderHealth, ProviderKind, ProviderStatus, SecretPolicy},
        runtime_bridge::runtime_status_from_registry,
    };

    #[test]
    fn runtime_status_exposes_app_identity_and_default_route() {
        let status = runtime_status_from_registry(&ModelRegistry::with_default_local(10));

        assert_eq!(status.app_name, "Delyx Next");
        assert_eq!(status.app_identifier, "com.geaux.delyxnext");
        assert_eq!(status.coding_route.unwrap().provider_id, "mock-local");
        assert!(status.providers.iter().any(|provider| provider.id == "ollama-local"));
    }

    #[test]
    fn runtime_status_maps_ready_ollama_models() {
        let mut registry = ModelRegistry::with_default_local(10);
        registry.register_provider(ModelProvider {
            health: ProviderHealth { checked_at: 11, message: "Ollama ready.".to_string(), status: ProviderStatus::Ready },
            id: "ollama-local".to_string(),
            kind: ProviderKind::Ollama,
            label: "Ollama".to_string(),
            models: vec![ModelInfo { context_window: 32768, display_name: "Qwen".to_string(), id: "qwen:latest".to_string(), supports_tools: true }],
            secret_policy: SecretPolicy::NoSecretRequired,
        });
        let status = runtime_status_from_registry(&registry);
        let ollama = status.providers.iter().find(|provider| provider.id == "ollama-local").unwrap();

        assert_eq!(ollama.status, "ready");
        assert_eq!(ollama.models, vec!["qwen:latest"]);
    }
}
