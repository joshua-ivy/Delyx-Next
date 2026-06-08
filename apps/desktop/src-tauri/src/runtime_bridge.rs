use crate::{
    desktop_shell_info,
    model_ollama::{
        detect_local_ollama_provider, send_ollama_chat, OllamaChatMessage, OllamaChatResult,
    },
    model_provider::{
        ModelInfo, ModelProvider, ModelRegistry, ModelRole, ProviderKind, ProviderStatus,
    },
};
use serde::Serialize;
use std::{
    path::{Path, PathBuf},
    time::Duration,
};

#[derive(Debug)]
pub struct RuntimeBridgeState {
    database_path: PathBuf,
}

impl RuntimeBridgeState {
    pub fn persistent(database_path: PathBuf) -> Self {
        Self { database_path }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RuntimeStatusView {
    pub app_name: String,
    pub app_identifier: String,
    pub desktop_shell: DesktopShellStatusView,
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

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DesktopShellStatusView {
    pub main_window_label: String,
    pub native_menu_policy: String,
    pub reopen_behavior: String,
    pub signing_policy: String,
    pub startup_behavior: String,
}

#[tauri::command]
pub fn runtime_status(
    state: tauri::State<RuntimeBridgeState>,
) -> Result<RuntimeStatusView, String> {
    let ollama = detect_local_ollama_provider(0, Duration::from_millis(750));
    runtime_status_with_provider(&state.database_path, ollama)
}

pub fn runtime_status_with_provider(
    database_path: &Path,
    ollama: ModelProvider,
) -> Result<RuntimeStatusView, String> {
    let mut registry = ModelRegistry::with_runtime_defaults(0);
    registry.register_provider(ollama);
    for route in crate::model_provider_persistence::load_routes_from_path(database_path)? {
        let _ = registry.save_role_route(route.role, &route.provider_id, &route.model_id);
    }
    if registry.route_for(ModelRole::Coding).is_none() {
        save_detected_coding_route(&mut registry, database_path)?;
    }
    Ok(runtime_status_from_registry(&registry))
}

fn save_detected_coding_route(
    registry: &mut ModelRegistry,
    database_path: &Path,
) -> Result<(), String> {
    if let Some(model_id) = first_ready_ollama_model(registry).map(|model| model.id.clone()) {
        let _ = registry.save_role_route(ModelRole::Coding, "ollama-local", &model_id);
        crate::model_provider_persistence::save_routes_to_path(database_path, registry.routes())?;
    }
    Ok(())
}

fn first_ready_ollama_model(registry: &ModelRegistry) -> Option<&ModelInfo> {
    registry
        .list_providers()
        .iter()
        .find(|provider| {
            provider.id == "ollama-local" && provider.health.status == ProviderStatus::Ready
        })
        .and_then(|provider| provider.models.first())
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
        coding_route: registry
            .route_for(ModelRole::Coding)
            .map(|route| RoleRouteStatusView {
                model_id: route.model_id.clone(),
                provider_id: route.provider_id.clone(),
            }),
        desktop_shell: DesktopShellStatusView {
            main_window_label: shell.main_window_label.to_string(),
            native_menu_policy: shell.native_menu_policy.to_string(),
            reopen_behavior: shell.reopen_behavior.to_string(),
            signing_policy: shell.signing_policy.to_string(),
            startup_behavior: shell.startup_behavior.to_string(),
        },
        milestone: shell.milestone.to_string(),
        providers: registry
            .list_providers()
            .iter()
            .map(provider_status)
            .collect(),
    }
}

fn provider_status(provider: &ModelProvider) -> ModelProviderStatusView {
    ModelProviderStatusView {
        id: provider.id.clone(),
        kind: provider_kind(provider.kind).to_string(),
        label: provider.label.clone(),
        message: provider.health.message.clone(),
        models: provider
            .models
            .iter()
            .map(|model| model.id.clone())
            .collect(),
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
