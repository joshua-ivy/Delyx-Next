use crate::{
    desktop_shell_info,
    model_ollama::{detect_local_ollama_provider, send_ollama_chat, OllamaChatMessage, OllamaChatResult},
    model_provider::{ModelProvider, ModelRegistry, ModelRole, ProviderKind, ProviderStatus},
};
use serde::Serialize;
use std::time::Duration;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RuntimeStatusView {
    pub app_name: String,
    pub app_identifier: String,
    pub milestone: String,
    pub providers: Vec<ModelProviderStatusView>,
    pub coding_route: Option<RoleRouteStatusView>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ModelProviderStatusView {
    pub id: String,
    pub kind: String,
    pub label: String,
    pub status: String,
    pub message: String,
    pub models: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RoleRouteStatusView {
    pub provider_id: String,
    pub model_id: String,
}

#[tauri::command]
pub fn runtime_status() -> RuntimeStatusView {
    let mut registry = ModelRegistry::with_runtime_defaults(0);
    let ollama = detect_local_ollama_provider(0, Duration::from_millis(750));
    let coding_model = ollama.models.first().map(|model| model.id.clone());
    registry.register_provider(ollama);
    if let Some(model_id) = coding_model {
        let _ = registry.save_role_route(ModelRole::Coding, "ollama-local", &model_id);
    }
    runtime_status_from_registry(&registry)
}

#[tauri::command]
pub fn ollama_chat(
    model: String,
    messages: Vec<OllamaChatMessage>,
) -> Result<OllamaChatResult, String> {
    send_ollama_chat(model, messages, Duration::from_secs(120))
}

pub fn runtime_status_from_registry(registry: &ModelRegistry) -> RuntimeStatusView {
    let shell = desktop_shell_info();
    RuntimeStatusView {
        app_identifier: shell.identifier.to_string(),
        app_name: shell.name.to_string(),
        coding_route: registry.route_for(ModelRole::Coding).map(|route| RoleRouteStatusView {
            model_id: route.model_id.clone(),
            provider_id: route.provider_id.clone(),
        }),
        milestone: shell.milestone.to_string(),
        providers: registry.list_providers().iter().map(provider_status).collect(),
    }
}

fn provider_status(provider: &ModelProvider) -> ModelProviderStatusView {
    ModelProviderStatusView {
        id: provider.id.clone(),
        kind: provider_kind(provider.kind).to_string(),
        label: provider.label.clone(),
        message: provider.health.message.clone(),
        models: provider.models.iter().map(|model| model.id.clone()).collect(),
        status: provider_status_label(provider.health.status).to_string(),
    }
}

fn provider_kind(kind: ProviderKind) -> &'static str {
    match kind {
        ProviderKind::Mock => "mock",
        ProviderKind::Ollama => "ollama",
        ProviderKind::OpenAiCompatible => "openai_compatible",
    }
}

fn provider_status_label(status: ProviderStatus) -> &'static str {
    match status {
        ProviderStatus::MissingApiKey => "missing_key",
        ProviderStatus::NotConfigured => "not_configured",
        ProviderStatus::Ready => "ready",
        ProviderStatus::Unreachable => "unreachable",
    }
}
