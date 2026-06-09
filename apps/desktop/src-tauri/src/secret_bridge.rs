use crate::secret_store::{KeyringSecretStore, SecretStore};
use serde::Serialize;

/// Known cloud providers that accept a pasted API key, with the keyring account
/// name used to store it. The account names are stable storage keys, not secrets.
const PROVIDERS: [(&str, &str, &str); 2] = [
    ("anthropic", "Anthropic", "anthropic_api_key"),
    ("openai", "OpenAI", "openai_api_key"),
];

pub struct SecretBridgeState {
    store: Box<dyn SecretStore>,
}

impl SecretBridgeState {
    pub fn keyring() -> Self {
        Self {
            store: Box::new(KeyringSecretStore),
        }
    }
}

impl Default for SecretBridgeState {
    fn default() -> Self {
        Self::keyring()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SecretStatusView {
    pub providers: Vec<SecretProviderView>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SecretProviderView {
    pub id: String,
    pub label: String,
    pub has_key: bool,
}

#[tauri::command]
pub fn secret_set(
    state: tauri::State<SecretBridgeState>,
    provider_id: String,
    value: String,
) -> Result<SecretStatusView, String> {
    set_secret_record(state.store.as_ref(), &provider_id, &value)
}

#[tauri::command]
pub fn secret_clear(
    state: tauri::State<SecretBridgeState>,
    provider_id: String,
) -> Result<SecretStatusView, String> {
    clear_secret_record(state.store.as_ref(), &provider_id)
}

#[tauri::command]
pub fn secret_status(state: tauri::State<SecretBridgeState>) -> Result<SecretStatusView, String> {
    secret_status_record(state.store.as_ref())
}

pub fn set_secret_record(
    store: &dyn SecretStore,
    provider_id: &str,
    value: &str,
) -> Result<SecretStatusView, String> {
    let account = account_for(provider_id)?;
    let value = value.trim();
    if value.is_empty() {
        return Err("API key cannot be empty.".to_string());
    }
    store.set(account, value)?;
    secret_status_record(store)
}

pub fn clear_secret_record(
    store: &dyn SecretStore,
    provider_id: &str,
) -> Result<SecretStatusView, String> {
    let account = account_for(provider_id)?;
    store.delete(account)?;
    secret_status_record(store)
}

pub fn secret_status_record(store: &dyn SecretStore) -> Result<SecretStatusView, String> {
    let mut providers = Vec::new();
    for (id, label, account) in PROVIDERS {
        providers.push(SecretProviderView {
            has_key: store.get(account)?.is_some(),
            id: id.to_string(),
            label: label.to_string(),
        });
    }
    Ok(SecretStatusView { providers })
}

/// Read a stored key for the model-execution path only. Not exposed as a command.
pub fn read_provider_secret(
    store: &dyn SecretStore,
    provider_id: &str,
) -> Result<Option<String>, String> {
    store.get(account_for(provider_id)?)
}

fn account_for(provider_id: &str) -> Result<&'static str, String> {
    PROVIDERS
        .iter()
        .find(|(id, _, _)| *id == provider_id)
        .map(|(_, _, account)| *account)
        .ok_or_else(|| format!("Unknown provider `{provider_id}`."))
}
