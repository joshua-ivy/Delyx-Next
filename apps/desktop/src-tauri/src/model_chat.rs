//! Provider-aware chat dispatch. Routes a (provider, model) chat request to the
//! Delyx embedded runtime or the Ollama adapter behind one command.

use crate::model_embedded::EmbeddedRuntimeState;
use crate::model_embedded_persistence::load_profile_from_path;
use crate::model_ollama::{send_ollama_chat, OllamaChatMessage};
use serde::{Deserialize, Serialize};
use std::{path::Path, time::Duration};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ModelChatMessage {
    pub role: String,
    pub content: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ModelChatResult {
    pub provider_id: String,
    pub model: String,
    pub text: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ModelChatRequest {
    pub provider_id: String,
    pub model: String,
    pub messages: Vec<ModelChatMessage>,
}

#[tauri::command]
pub async fn model_chat(
    runtime: tauri::State<'_, crate::runtime_bridge::RuntimeBridgeState>,
    embedded: tauri::State<'_, EmbeddedRuntimeState>,
    request: ModelChatRequest,
) -> Result<ModelChatResult, String> {
    let database_path = runtime.database_path().to_path_buf();
    send_model_chat(
        &database_path,
        &embedded,
        request.provider_id,
        request.model,
        request.messages,
    )
    .await
}

pub async fn send_model_chat(
    database_path: &Path,
    embedded: &EmbeddedRuntimeState,
    provider_id: String,
    model: String,
    messages: Vec<ModelChatMessage>,
) -> Result<ModelChatResult, String> {
    validate_messages(&messages)?;
    match provider_id.as_str() {
        "delyx-local" => {
            let profile = load_profile_from_path(database_path, &model)?;
            crate::model_embedded::send_embedded_chat(embedded, database_path, profile, messages)
                .await
        }
        "ollama-local" => {
            let ollama_messages = messages
                .into_iter()
                .map(|message| OllamaChatMessage {
                    role: message.role,
                    content: message.content,
                })
                .collect();
            let response = send_ollama_chat(model, ollama_messages, Duration::from_secs(120))?;
            Ok(ModelChatResult {
                provider_id: response.provider_id,
                model: response.model,
                text: response.text,
            })
        }
        other => Err(format!(
            "Provider `{other}` is not supported for model chat."
        )),
    }
}

fn validate_messages(messages: &[ModelChatMessage]) -> Result<(), String> {
    if messages.is_empty() {
        return Err("Model chat requires at least one message.".to_string());
    }
    for message in messages {
        if !matches!(message.role.as_str(), "assistant" | "system" | "user") {
            return Err(format!("Unsupported message role `{}`.", message.role));
        }
        if message.content.trim().is_empty() {
            return Err("Model chat messages cannot be empty.".to_string());
        }
    }
    Ok(())
}
