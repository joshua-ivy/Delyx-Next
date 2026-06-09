#[cfg(test)]
mod tests {
    use crate::model_chat::{send_model_chat, ModelChatMessage};
    use crate::model_embedded::EmbeddedRuntimeState;
    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn msg(role: &str, content: &str) -> ModelChatMessage {
        ModelChatMessage {
            role: role.to_string(),
            content: content.to_string(),
        }
    }

    fn temp_db(label: &str) -> PathBuf {
        let stamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        std::env::temp_dir().join(format!("delyx-model-chat-{label}-{stamp}.sqlite3"))
    }

    #[tokio::test]
    async fn rejects_unknown_provider() {
        let embedded = EmbeddedRuntimeState::new();
        let err = send_model_chat(
            &temp_db("unknown"),
            &embedded,
            "openai".to_string(),
            "x".to_string(),
            vec![msg("user", "hi")],
        )
        .await
        .unwrap_err();
        assert!(err.contains("not supported"));
    }

    #[tokio::test]
    async fn delyx_local_without_profile_reports_missing_profile() {
        let embedded = EmbeddedRuntimeState::new();
        let err = send_model_chat(
            &temp_db("missing-profile"),
            &embedded,
            "delyx-local".to_string(),
            "missing".to_string(),
            vec![msg("user", "hi")],
        )
        .await
        .unwrap_err();
        assert!(err.contains("was not found"));
    }

    #[tokio::test]
    async fn rejects_empty_and_bad_messages() {
        let embedded = EmbeddedRuntimeState::new();
        let empty = send_model_chat(
            &temp_db("empty"),
            &embedded,
            "ollama-local".to_string(),
            "m".to_string(),
            vec![],
        )
        .await
        .unwrap_err();
        assert!(empty.contains("at least one"));

        let bad_role = send_model_chat(
            &temp_db("bad-role"),
            &embedded,
            "ollama-local".to_string(),
            "m".to_string(),
            vec![msg("boss", "hi")],
        )
        .await
        .unwrap_err();
        assert!(bad_role.contains("Unsupported message role"));
    }
}
