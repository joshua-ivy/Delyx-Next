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

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ModelChatStreamRequest {
    pub provider_id: String,
    pub model: String,
    pub messages: Vec<ModelChatMessage>,
    pub request_id: String,
}

/// Streamed chat for the embedded Delyx Local runtime: deltas arrive as
/// `model-stream` events; the command resolves with the full (or cancelled
/// partial) text. Other providers stay on the non-streaming `model_chat`.
#[tauri::command]
pub async fn model_chat_stream(
    app: tauri::AppHandle,
    runtime: tauri::State<'_, crate::runtime_bridge::RuntimeBridgeState>,
    embedded: tauri::State<'_, EmbeddedRuntimeState>,
    request: ModelChatStreamRequest,
) -> Result<ModelChatResult, String> {
    if request.provider_id != "delyx-local" {
        return Err("Streaming is only available for the Delyx Local provider.".to_string());
    }
    if request.request_id.trim().is_empty() {
        return Err("Streaming requires a request id.".to_string());
    }
    validate_messages(&request.messages)?;
    let database_path = runtime.database_path().to_path_buf();
    let profile = load_profile_from_path(&database_path, &request.model)?;
    crate::model_embedded::stream_embedded_chat(
        &embedded,
        &app,
        &database_path,
        profile,
        request.messages,
        request.request_id,
    )
    .await
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ModelChatToolsRequest {
    pub provider_id: String,
    pub model: String,
    pub messages: Vec<ModelChatMessage>,
    pub request_id: String,
    pub project_root: String,
}

/// Agentic chat for Delyx Local: the model may call read-only project tools
/// (read_file / list_dir / grep) before answering. Tool turns narrate via
/// `tool-loop` events; the final answer streams via `model-stream`.
#[tauri::command]
pub async fn model_chat_tools(
    app: tauri::AppHandle,
    runtime: tauri::State<'_, crate::runtime_bridge::RuntimeBridgeState>,
    embedded: tauri::State<'_, EmbeddedRuntimeState>,
    request: ModelChatToolsRequest,
) -> Result<ModelChatResult, String> {
    if request.provider_id != "delyx-local" {
        return Err("Tool-using chat is only available for the Delyx Local provider.".to_string());
    }
    if request.request_id.trim().is_empty() || request.project_root.trim().is_empty() {
        return Err("Tool-using chat requires a request id and project root.".to_string());
    }
    validate_messages(&request.messages)?;
    let database_path = runtime.database_path().to_path_buf();
    let profile = load_profile_from_path(&database_path, &request.model)?;
    crate::agent_tool_loop::run_tool_loop(
        &embedded,
        &app,
        &database_path,
        profile,
        request.messages,
        request.project_root,
        request.request_id,
    )
    .await
}

#[tauri::command]
pub fn model_chat_cancel(
    embedded: tauri::State<'_, EmbeddedRuntimeState>,
    request_id: String,
) -> Result<(), String> {
    embedded.cancel_stream(&request_id);
    Ok(())
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
