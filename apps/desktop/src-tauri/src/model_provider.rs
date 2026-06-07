#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModelProvider {
    pub id: String,
    pub kind: ProviderKind,
    pub label: String,
    pub health: ProviderHealth,
    pub models: Vec<ModelInfo>,
    pub secret_policy: SecretPolicy,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProviderKind {
    Mock,
    Ollama,
    OpenAiCompatible,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProviderStatus {
    Ready,
    MissingApiKey,
    NotConfigured,
    Unreachable,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProviderHealth {
    pub status: ProviderStatus,
    pub message: String,
    pub checked_at: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModelInfo {
    pub id: String,
    pub display_name: String,
    pub context_window: u32,
    pub supports_tools: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SecretPolicy {
    NoSecretRequired,
    ExternalSecretOnly,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ModelRole {
    Answer,
    Helper,
    DeepResearch,
    MaxReasoning,
    Coding,
    Embedding,
    Scoring,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RoleRoute {
    pub role: ModelRole,
    pub provider_id: String,
    pub model_id: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModelResponse {
    pub provider_id: String,
    pub model_id: String,
    pub text: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ModelProviderError {
    ModelNotFound,
    ProviderNotFound,
    ProviderUnavailable,
}

#[derive(Debug, Default)]
pub struct ModelRegistry {
    providers: Vec<ModelProvider>,
    routes: Vec<RoleRoute>,
}

impl ModelRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_default_local(checked_at: u64) -> Self {
        let mut registry = Self::new();
        registry.providers.push(mock_provider(checked_at));
        registry.providers.push(ollama_provider(checked_at));
        registry.providers.push(openai_compatible_missing_key(checked_at));
        let route_result = registry.save_role_route(ModelRole::Coding, "mock-local", "delyx-mock-coder");
        debug_assert!(route_result.is_ok(), "default mock route is valid");
        registry
    }

    pub fn with_runtime_defaults(checked_at: u64) -> Self {
        let mut registry = Self::new();
        registry.providers.push(ollama_provider(checked_at));
        registry.providers.push(openai_compatible_missing_key(checked_at));
        registry
    }

    pub fn list_providers(&self) -> &[ModelProvider] {
        &self.providers
    }

    pub fn register_provider(&mut self, provider: ModelProvider) {
        self.providers.retain(|current| current.id != provider.id);
        self.providers.push(provider);
    }

    pub fn health(&self, provider_id: &str) -> Result<&ProviderHealth, ModelProviderError> {
        Ok(&self.provider(provider_id)?.health)
    }

    pub fn save_role_route(&mut self, role: ModelRole, provider_id: &str, model_id: &str) -> Result<(), ModelProviderError> {
        let provider = self.provider(provider_id)?;
        if provider.health.status != ProviderStatus::Ready {
            return Err(ModelProviderError::ProviderUnavailable);
        }
        if !provider.models.iter().any(|model| model.id == model_id) {
            return Err(ModelProviderError::ModelNotFound);
        }
        self.routes.retain(|route| route.role != role);
        self.routes.push(RoleRoute { role, provider_id: provider_id.to_string(), model_id: model_id.to_string() });
        Ok(())
    }

    pub fn route_for(&self, role: ModelRole) -> Option<&RoleRoute> {
        self.routes.iter().find(|route| route.role == role)
    }

    pub fn mock_complete(&self, role: ModelRole, prompt: &str) -> Result<ModelResponse, ModelProviderError> {
        let route = self.route_for(role).ok_or(ModelProviderError::ModelNotFound)?;
        let provider = self.provider(&route.provider_id)?;
        if provider.kind != ProviderKind::Mock {
            return Err(ModelProviderError::ProviderNotFound);
        }
        Ok(ModelResponse {
            provider_id: route.provider_id.clone(),
            model_id: route.model_id.clone(),
            text: format!("mock:{}:{}", route.model_id, stable_prompt_score(prompt)),
        })
    }

    fn provider(&self, provider_id: &str) -> Result<&ModelProvider, ModelProviderError> {
        self.providers.iter().find(|provider| provider.id == provider_id).ok_or(ModelProviderError::ProviderNotFound)
    }
}

fn mock_provider(checked_at: u64) -> ModelProvider {
    ModelProvider {
        id: "mock-local".to_string(),
        kind: ProviderKind::Mock,
        label: "Local deterministic".to_string(),
        health: ProviderHealth { status: ProviderStatus::Ready, message: "Deterministic local mock provider is ready.".to_string(), checked_at },
        models: vec![model("delyx-mock-coder", "Delyx Mock Coder", true), model("delyx-mock-reasoner", "Delyx Mock Reasoner", false)],
        secret_policy: SecretPolicy::NoSecretRequired,
    }
}

fn ollama_provider(checked_at: u64) -> ModelProvider {
    ModelProvider {
        id: "ollama-local".to_string(),
        kind: ProviderKind::Ollama,
        label: "Ollama".to_string(),
        health: ProviderHealth { status: ProviderStatus::NotConfigured, message: "No local Ollama endpoint configured yet.".to_string(), checked_at },
        models: Vec::new(),
        secret_policy: SecretPolicy::NoSecretRequired,
    }
}

fn openai_compatible_missing_key(checked_at: u64) -> ModelProvider {
    ModelProvider {
        id: "openai-compatible".to_string(),
        kind: ProviderKind::OpenAiCompatible,
        label: "OpenAI-compatible".to_string(),
        health: ProviderHealth { status: ProviderStatus::MissingApiKey, message: "API key is missing; secrets must stay outside the repo.".to_string(), checked_at },
        models: Vec::new(),
        secret_policy: SecretPolicy::ExternalSecretOnly,
    }
}

fn model(id: &str, display_name: &str, supports_tools: bool) -> ModelInfo {
    ModelInfo { id: id.to_string(), display_name: display_name.to_string(), context_window: 8192, supports_tools }
}

fn stable_prompt_score(prompt: &str) -> u32 {
    prompt.bytes().fold(0_u32, |score, byte| score.wrapping_mul(31).wrapping_add(byte as u32))
}
