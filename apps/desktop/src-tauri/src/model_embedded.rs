//! In-process embedded local model runtime (feature `embedded_mistral`).
//!
//! On default builds this is a stub that returns a clear "not compiled" error, so
//! the whole app compiles and ships without the heavy dependency. The real
//! runtime is compiled only with `--features embedded_mistral`.

use crate::model_chat::{ModelChatMessage, ModelChatResult};
#[cfg(feature = "embedded_mistral")]
use crate::model_embedded_persistence::mark_profile_status;
use crate::model_embedded_persistence::LocalModelProfile;
use std::collections::HashSet;
use std::{collections::HashMap, path::Path, sync::Arc};
use tokio::sync::Mutex;

#[derive(Default)]
pub struct EmbeddedRuntimeState {
    loaded: Mutex<HashMap<String, Arc<LoadedLocalModel>>>,
    /// Stream request ids the user asked to stop; checked between chunks.
    cancelled: std::sync::Mutex<HashSet<String>>,
}

pub struct LoadedLocalModel {
    #[allow(dead_code)]
    pub profile_id: String,
    #[cfg(feature = "embedded_mistral")]
    pub model: mistralrs::Model,
}

impl EmbeddedRuntimeState {
    pub fn new() -> Self {
        Self {
            loaded: Mutex::new(HashMap::new()),
            cancelled: std::sync::Mutex::new(HashSet::new()),
        }
    }

    pub async fn unload(&self, profile_id: &str) -> bool {
        self.loaded.lock().await.remove(profile_id).is_some()
    }

    /// Ask a running stream to stop after the current chunk.
    pub fn cancel_stream(&self, request_id: &str) {
        if let Ok(mut cancelled) = self.cancelled.lock() {
            cancelled.insert(request_id.to_string());
        }
    }

    /// Consume a pending cancel for this request, returning whether it was set.
    pub fn take_cancel(&self, request_id: &str) -> bool {
        self.cancelled
            .lock()
            .map(|mut cancelled| cancelled.remove(request_id))
            .unwrap_or(false)
    }
}

/// Token event emitted to the webview while a local model streams.
#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ModelStreamEvent {
    pub request_id: String,
    /// "token", "done", or "cancelled".
    pub kind: String,
    pub text: String,
}

pub const MODEL_STREAM_EVENT: &str = "model-stream";

#[cfg(not(feature = "embedded_mistral"))]
pub async fn send_embedded_chat(
    _state: &EmbeddedRuntimeState,
    _database_path: &Path,
    _profile: LocalModelProfile,
    _messages: Vec<ModelChatMessage>,
) -> Result<ModelChatResult, String> {
    Err(
        "Delyx embedded runtime was not compiled. Build with --features embedded_mistral."
            .to_string(),
    )
}

#[cfg(not(feature = "embedded_mistral"))]
pub async fn stream_embedded_chat(
    _state: &EmbeddedRuntimeState,
    _app: &tauri::AppHandle,
    _database_path: &Path,
    _profile: LocalModelProfile,
    _messages: Vec<ModelChatMessage>,
    _request_id: String,
) -> Result<ModelChatResult, String> {
    Err(
        "Delyx embedded runtime was not compiled. Build with --features embedded_mistral."
            .to_string(),
    )
}

/// Stream a chat reply token-by-token: each delta is emitted as a `model-stream`
/// event, and the full text is returned (and persisted by the caller) once the
/// stream finishes or the user cancels. Cancel keeps the partial text.
#[cfg(feature = "embedded_mistral")]
pub async fn stream_embedded_chat(
    state: &EmbeddedRuntimeState,
    app: &tauri::AppHandle,
    database_path: &Path,
    profile: LocalModelProfile,
    messages: Vec<ModelChatMessage>,
    request_id: String,
) -> Result<ModelChatResult, String> {
    use tauri::Emitter;
    validate_profile(&profile)?;
    // Clear any stale cancel from a previous run with the same id.
    state.take_cancel(&request_id);
    let loaded = load_or_get_model(state, database_path, &profile).await?;
    let request = chat_request(messages, &profile)?;
    let mut stream = loaded
        .model
        .stream_chat_request(request)
        .await
        .map_err(|error| {
            let message = describe_chat_error(&error.to_string(), &profile.display_name);
            let _ = mark_profile_status(database_path, &profile.id, "failed", Some(&message));
            message
        })?;

    let mut text = String::new();
    let mut cancelled = false;
    while let Some(response) = stream.next().await {
        match response {
            mistralrs::Response::Chunk(chunk) => {
                let delta = chunk
                    .choices
                    .first()
                    .and_then(|choice| choice.delta.content.clone())
                    .unwrap_or_default();
                if !delta.is_empty() {
                    text.push_str(&delta);
                    let _ = app.emit(
                        MODEL_STREAM_EVENT,
                        ModelStreamEvent {
                            request_id: request_id.clone(),
                            kind: "token".to_string(),
                            text: delta,
                        },
                    );
                }
                let finished = chunk
                    .choices
                    .first()
                    .map(|choice| choice.finish_reason.is_some())
                    .unwrap_or(false);
                if finished {
                    break;
                }
            }
            mistralrs::Response::Done(done) => {
                // Some engines close with a full response; prefer it when longer.
                if let Some(choice) = done.choices.first() {
                    if let Some(content) = &choice.message.content {
                        if content.len() > text.len() {
                            text = content.clone();
                        }
                    }
                }
                break;
            }
            mistralrs::Response::ModelError(message, _) => {
                let message = describe_chat_error(&message, &profile.display_name);
                let _ = mark_profile_status(database_path, &profile.id, "failed", Some(&message));
                return Err(message);
            }
            _ => {}
        }
        if state.take_cancel(&request_id) {
            cancelled = true;
            break;
        }
    }
    // Dropping the stream tells the engine to stop generating.
    drop(stream);

    let _ = app.emit(
        MODEL_STREAM_EVENT,
        ModelStreamEvent {
            request_id: request_id.clone(),
            kind: if cancelled { "cancelled" } else { "done" }.to_string(),
            text: text.clone(),
        },
    );
    let text = text.trim().to_string();
    if text.is_empty() && !cancelled {
        return Err("Local model returned an empty response.".to_string());
    }
    Ok(ModelChatResult {
        provider_id: "delyx-local".to_string(),
        model: profile.id,
        text,
    })
}

#[cfg(feature = "embedded_mistral")]
pub async fn send_embedded_chat(
    state: &EmbeddedRuntimeState,
    database_path: &Path,
    profile: LocalModelProfile,
    messages: Vec<ModelChatMessage>,
) -> Result<ModelChatResult, String> {
    validate_profile(&profile)?;
    let loaded = load_or_get_model(state, database_path, &profile).await?;
    let request = chat_request(messages, &profile)?;
    let response = match loaded.model.send_chat_request(request).await {
        Ok(response) => response,
        Err(error) => {
            // A crashed engine closes its channel and stays broken; drop the
            // cached model so the next attempt reloads from scratch.
            state.loaded.lock().await.remove(&profile.id);
            let message = describe_chat_error(&error.to_string(), &profile.display_name);
            let _ = mark_profile_status(database_path, &profile.id, "failed", Some(&message));
            return Err(message);
        }
    };
    let text = response
        .choices
        .first()
        .and_then(|choice| choice.message.content.as_ref())
        .cloned()
        .unwrap_or_default()
        .trim()
        .to_string();
    if text.is_empty() {
        return Err("Local model returned an empty response.".to_string());
    }
    Ok(ModelChatResult {
        provider_id: "delyx-local".to_string(),
        model: profile.id,
        text,
    })
}

#[cfg(feature = "embedded_mistral")]
pub(crate) fn describe_chat_error(error: &str, name: &str) -> String {
    if error.contains("channel closed") || error.to_lowercase().contains("out of memory") {
        format!(
            "Local model `{name}` engine stopped mid-response — usually not enough memory for a \
             model this large on CPU. Try a smaller quantization or model (a 7B/14B Q4 fits 16-32GB \
             RAM), or use a GPU build."
        )
    } else {
        format!("Local model chat failed: {error}")
    }
}

#[cfg(feature = "embedded_mistral")]
pub(crate) async fn load_or_get_model(
    state: &EmbeddedRuntimeState,
    database_path: &Path,
    profile: &LocalModelProfile,
) -> Result<Arc<LoadedLocalModel>, String> {
    if let Some(existing) = state.loaded.lock().await.get(&profile.id).cloned() {
        return Ok(existing);
    }
    mark_profile_status(database_path, &profile.id, "loading", None)?;

    let model_path = Path::new(&profile.model_path);
    let model_dir = model_path
        .parent()
        .ok_or_else(|| "Model path has no parent directory.".to_string())?
        .to_string_lossy()
        .to_string();
    let model_file = model_path
        .file_name()
        .and_then(|value| value.to_str())
        .ok_or_else(|| "Model path has no filename.".to_string())?
        .to_string();

    let mut builder = mistralrs::GgufModelBuilder::new(model_dir, vec![model_file]).with_logging();
    if let Some(template) = profile
        .chat_template_path
        .as_deref()
        .filter(|value| !value.trim().is_empty())
    {
        builder = builder.with_chat_template(template);
    }
    if let Some(tokenizer) = profile
        .tokenizer_path
        .as_deref()
        .filter(|value| !value.trim().is_empty())
    {
        builder = builder.with_tok_model_id(tokenizer.to_string());
    }

    let model = builder.build().await.map_err(|error| {
        let message = format!(
            "Failed to load local model `{}`: {error}",
            profile.display_name
        );
        let _ = mark_profile_status(database_path, &profile.id, "failed", Some(&message));
        message
    })?;

    let loaded = Arc::new(LoadedLocalModel {
        profile_id: profile.id.clone(),
        model,
    });
    state
        .loaded
        .lock()
        .await
        .insert(profile.id.clone(), loaded.clone());
    mark_profile_status(database_path, &profile.id, "loaded", None)?;
    Ok(loaded)
}

#[cfg(feature = "embedded_mistral")]
pub(crate) fn validate_profile(profile: &LocalModelProfile) -> Result<(), String> {
    if profile.runtime != "mistralrs" {
        return Err(format!("Unsupported local runtime `{}`.", profile.runtime));
    }
    if profile.format != "gguf" {
        return Err(format!(
            "Unsupported local model format `{}`.",
            profile.format
        ));
    }
    if !Path::new(&profile.model_path).is_file() {
        return Err(format!("Model file is missing: {}", profile.model_path));
    }
    Ok(())
}

#[cfg(feature = "embedded_mistral")]
pub(crate) fn chat_request(
    messages: Vec<ModelChatMessage>,
    profile: &LocalModelProfile,
) -> Result<mistralrs::RequestBuilder, String> {
    let mut builder = mistralrs::RequestBuilder::new();
    for message in messages {
        let role = match message.role.as_str() {
            "assistant" => mistralrs::TextMessageRole::Assistant,
            "system" => mistralrs::TextMessageRole::System,
            "user" => mistralrs::TextMessageRole::User,
            other => return Err(format!("Unsupported message role `{other}`.")),
        };
        let content = message.content.trim();
        if content.is_empty() {
            return Err("Chat messages cannot be empty.".to_string());
        }
        builder = builder.add_message(role, content);
    }
    // Apply the profile's tuned sampling params (the reason for the embedded runtime).
    Ok(builder.set_sampling(mistralrs::SamplingParams {
        temperature: profile.temperature,
        top_k: profile.top_k.map(|value| value as usize),
        top_p: profile.top_p,
        min_p: None,
        top_n_logprobs: 0,
        frequency_penalty: None,
        presence_penalty: None,
        repetition_penalty: profile.repeat_penalty,
        stop_toks: None,
        max_len: profile.max_tokens.map(|value| value as usize),
        logits_bias: None,
        n_choices: 1,
        dry_params: None,
    }))
}
