#[cfg(test)]
mod tests {
    use crate::{
        model_ollama::{
            chat_from_http_result, parse_ollama_model_names, provider_from_tags_result,
            send_ollama_chat, OllamaChatMessage,
        },
        model_provider::{ProviderKind, ProviderStatus, SecretPolicy},
    };
    use std::time::Duration;

    #[test]
    fn ollama_tags_parser_reads_name_and_model_fields() {
        let body = r#"{
            "models": [
                { "name": "qwen3-coder:30b" },
                { "model": "llama3.2:latest" },
                { "name": "qwen3-coder:30b" }
            ]
        }"#;

        assert_eq!(
            parse_ollama_model_names(body).unwrap(),
            vec!["qwen3-coder:30b".to_string(), "llama3.2:latest".to_string()]
        );
    }

    #[test]
    fn provider_from_tags_result_maps_ready_models() {
        let provider = provider_from_tags_result(
            12,
            Ok((
                200,
                r#"{"models":[{"name":"qwen3-coder:30b"}]}"#.to_string(),
            )),
        );

        assert_eq!(provider.id, "ollama-local");
        assert_eq!(provider.kind, ProviderKind::Ollama);
        assert_eq!(provider.health.status, ProviderStatus::Ready);
        assert_eq!(provider.health.checked_at, 12);
        assert_eq!(provider.secret_policy, SecretPolicy::NoSecretRequired);
        assert_eq!(provider.models[0].id, "qwen3-coder:30b");
    }

    #[test]
    fn provider_from_tags_result_maps_empty_models_to_not_configured() {
        let provider = provider_from_tags_result(12, Ok((200, r#"{"models":[]}"#.to_string())));

        assert_eq!(provider.health.status, ProviderStatus::NotConfigured);
        assert!(provider.models.is_empty());
        assert!(provider.health.message.contains("no local models"));
    }

    #[test]
    fn provider_from_tags_result_maps_http_error_to_unreachable() {
        let provider = provider_from_tags_result(12, Ok((500, r#"{"error":"boom"}"#.to_string())));

        assert_eq!(provider.health.status, ProviderStatus::Unreachable);
        assert!(provider.health.message.contains("HTTP 500"));
    }

    #[test]
    fn provider_from_tags_result_maps_invalid_json_to_unreachable() {
        let provider = provider_from_tags_result(12, Ok((200, "not json".to_string())));

        assert_eq!(provider.health.status, ProviderStatus::Unreachable);
        assert!(provider.health.message.contains("not parseable"));
    }

    #[test]
    fn provider_from_tags_result_maps_connection_error_to_unreachable() {
        let provider = provider_from_tags_result(
            12,
            Err("Ollama is not reachable at 127.0.0.1:11434.".to_string()),
        );

        assert_eq!(provider.health.status, ProviderStatus::Unreachable);
        assert!(provider.health.message.contains("not reachable"));
    }

    #[test]
    fn chat_from_http_result_reads_message_content() {
        let result = chat_from_http_result(
            "qwen3-coder:30b",
            Ok((
                200,
                r#"{"message":{"content":"  Ready to work.  "}}"#.to_string(),
            )),
        )
        .unwrap();

        assert_eq!(result.provider_id, "ollama-local");
        assert_eq!(result.model, "qwen3-coder:30b");
        assert_eq!(result.text, "Ready to work.");
    }

    #[test]
    fn chat_from_http_result_reads_generate_response_fallback() {
        let result = chat_from_http_result(
            "llama3.2:latest",
            Ok((200, r#"{"response":"Fallback text"}"#.to_string())),
        )
        .unwrap();

        assert_eq!(result.text, "Fallback text");
    }

    #[test]
    fn chat_from_http_result_maps_http_error() {
        let error = chat_from_http_result(
            "qwen3-coder:30b",
            Ok((500, r#"{"error":"boom"}"#.to_string())),
        )
        .unwrap_err();

        assert!(error.contains("HTTP 500"));
    }

    #[test]
    fn send_ollama_chat_rejects_empty_model_before_network() {
        let error = send_ollama_chat(" ".to_string(), vec![message("user", "hi")], Duration::ZERO)
            .unwrap_err();

        assert!(error.contains("selected model"));
    }

    #[test]
    fn send_ollama_chat_rejects_unsupported_message_role_before_network() {
        let error = send_ollama_chat(
            "qwen3-coder:30b".to_string(),
            vec![message("tool", "hi")],
            Duration::ZERO,
        )
        .unwrap_err();

        assert!(error.contains("not supported"));
    }

    fn message(role: &str, content: &str) -> OllamaChatMessage {
        OllamaChatMessage {
            content: content.to_string(),
            role: role.to_string(),
        }
    }
}
