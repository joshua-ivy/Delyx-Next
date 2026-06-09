//! In-process embedded local model runtime (feature `embedded_mistral`).
//!
//! On default builds this is a stub that returns a clear "not compiled" error, so
//! the whole app compiles and ships without the heavy dependency. The real
//! runtime is compiled only with `--features embedded_mistral`.

use crate::model_chat::{ModelChatMessage, ModelChatResult};
#[cfg(feature = "embedded_mistral")]
use crate::model_embedded_persistence::mark_profile_status;
use crate::model_embedded_persistence::LocalModelProfile;
use std::{collections::HashMap, path::Path, sync::Arc};
use tokio::sync::Mutex;

#[derive(Default)]
pub struct EmbeddedRuntimeState {
    loaded: Mutex<HashMap<String, Arc<LoadedLocalModel>>>,
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
        }
    }

    pub async fn unload(&self, profile_id: &str) -> bool {
        self.loaded.lock().await.remove(profile_id).is_some()
    }
}

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
fn describe_chat_error(error: &str, name: &str) -> String {
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
async fn load_or_get_model(
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
fn validate_profile(profile: &LocalModelProfile) -> Result<(), String> {
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
fn chat_request(
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
