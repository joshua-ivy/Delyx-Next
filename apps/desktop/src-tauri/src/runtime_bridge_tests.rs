#[cfg(test)]
mod tests {
    use crate::{
        model_provider::{
            ModelInfo, ModelProvider, ModelRegistry, ModelRole, ProviderHealth, ProviderKind,
            ProviderStatus, SecretPolicy,
        },
        model_provider_persistence::{load_routes_from_path, save_routes_to_path},
        runtime_bridge::{
            runtime_status_from_registry, runtime_status_from_registry_with_version,
            runtime_status_with_provider,
        },
    };
    use std::fs;
    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn runtime_status_from_runtime_defaults_exposes_no_mock_route() {
        let status = runtime_status_from_registry(&ModelRegistry::with_runtime_defaults(10));

        assert_eq!(status.app_name, "Delyx Next");
        assert_eq!(status.app_identifier, "com.geaux.delyxnext");
        assert_eq!(
            status.desktop_shell.reopen_behavior,
            "single_instance_focus_main_window"
        );
        assert_eq!(status.desktop_shell.signing_policy, "unsigned_dev_build");
        assert!(status.coding_route.is_none());
        assert!(!status
            .providers
            .iter()
            .any(|provider| provider.id == "mock-local"));
        assert!(status
            .providers
            .iter()
            .any(|provider| provider.id == "ollama-local"));
    }

    #[test]
    fn runtime_status_maps_ready_ollama_models() {
        let mut registry = ModelRegistry::with_runtime_defaults(10);
        registry.register_provider(ready_ollama_provider());
        let status = runtime_status_from_registry(&registry);
        let ollama = status
            .providers
            .iter()
            .find(|provider| provider.id == "ollama-local")
            .unwrap();

        assert_eq!(ollama.status, "ready");
        assert_eq!(ollama.models, vec!["qwen:latest"]);
    }

    #[test]
    fn runtime_status_maps_optional_ollama_version() {
        let mut registry = ModelRegistry::with_runtime_defaults(10);
        registry.register_provider(ready_ollama_provider());
        let status =
            runtime_status_from_registry_with_version(&registry, Some("0.12.6".to_string()));
        let ollama = status
            .providers
            .iter()
            .find(|provider| provider.id == "ollama-local")
            .unwrap();

        assert_eq!(ollama.version.as_deref(), Some("0.12.6"));
    }

    #[test]
    fn runtime_status_maps_ready_ollama_coding_route() {
        let mut registry = ModelRegistry::with_runtime_defaults(10);
        registry.register_provider(ready_ollama_provider());
        registry
            .save_role_route(ModelRole::Coding, "ollama-local", "qwen:latest")
            .unwrap();
        let status = runtime_status_from_registry(&registry);
        let route = status.coding_route.unwrap();

        assert_eq!(route.provider_id, "ollama-local");
        assert_eq!(route.model_id, "qwen:latest");
    }

    #[test]
    fn runtime_status_prefers_persisted_ready_coding_route() {
        let path = temp_path("runtime-route");
        let mut registry = ModelRegistry::with_runtime_defaults(10);
        registry.register_provider(ready_ollama_provider_with(vec![
            "qwen:latest",
            "llama3:latest",
        ]));
        registry
            .save_role_route(ModelRole::Coding, "ollama-local", "llama3:latest")
            .unwrap();
        save_routes_to_path(&path, registry.routes()).unwrap();

        let status = runtime_status_with_provider(
            &path,
            ready_ollama_provider_with(vec!["qwen:latest", "llama3:latest"]),
        )
        .unwrap();
        let route = status.coding_route.unwrap();

        assert_eq!(route.provider_id, "ollama-local");
        assert_eq!(route.model_id, "llama3:latest");
        let _ = fs::remove_file(path);
    }

    #[test]
    fn runtime_status_saves_first_ready_ollama_route_when_none_persisted() {
        let path = temp_path("runtime-route-auto");

        let status = runtime_status_with_provider(&path, ready_ollama_provider()).unwrap();
        let route = status.coding_route.unwrap();
        let loaded = load_routes_from_path(&path).unwrap();

        assert_eq!(route.model_id, "qwen:latest");
        assert!(loaded
            .iter()
            .any(|item| item.role == ModelRole::Coding && item.model_id == "qwen:latest"));
        let _ = fs::remove_file(path);
    }

    fn ready_ollama_provider() -> ModelProvider {
        ready_ollama_provider_with(vec!["qwen:latest"])
    }

    fn ready_ollama_provider_with(model_ids: Vec<&str>) -> ModelProvider {
        ModelProvider {
            health: ProviderHealth {
                checked_at: 11,
                message: "Ollama ready.".to_string(),
                status: ProviderStatus::Ready,
            },
            id: "ollama-local".to_string(),
            kind: ProviderKind::Ollama,
            label: "Ollama".to_string(),
            models: model_ids.into_iter().map(ollama_model).collect(),
            secret_policy: SecretPolicy::NoSecretRequired,
        }
    }

    fn ollama_model(id: &str) -> ModelInfo {
        ModelInfo {
            context_window: 32768,
            display_name: id.to_string(),
            id: id.to_string(),
            supports_tools: true,
            format: None,
            runtime: None,
            path: None,
        }
    }

    fn temp_path(name: &str) -> PathBuf {
        let stamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        std::env::temp_dir().join(format!("delyx-next-{name}-{stamp}.sqlite3"))
    }
}
