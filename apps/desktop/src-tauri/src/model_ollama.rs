use crate::model_provider::{
    ModelInfo, ModelProvider, ProviderHealth, ProviderKind, ProviderStatus, SecretPolicy,
};
use serde::Deserialize;
use std::collections::HashSet;
use std::io::{Read, Write};
use std::net::{SocketAddr, TcpStream};
use std::time::Duration;

const OLLAMA_ADDR: &str = "127.0.0.1:11434";
const OLLAMA_ID: &str = "ollama-local";

#[derive(Debug, Deserialize)]
struct OllamaTagsResponse {
    #[serde(default)]
    models: Vec<OllamaTagsModel>,
}

#[derive(Debug, Deserialize)]
struct OllamaTagsModel {
    model: Option<String>,
    name: Option<String>,
}

pub fn detect_local_ollama_provider(checked_at: u64, timeout: Duration) -> ModelProvider {
    provider_from_tags_result(checked_at, fetch_tags(timeout))
}

pub(crate) fn provider_from_tags_result(
    checked_at: u64,
    result: Result<(u16, String), String>,
) -> ModelProvider {
    match result {
        Ok((200, body)) => match parse_ollama_model_names(&body) {
            Ok(names) => provider_from_models(checked_at, names),
            Err(message) => {
                ollama_provider(checked_at, ProviderStatus::Unreachable, message, Vec::new())
            }
        },
        Ok((status, body)) => ollama_provider(
            checked_at,
            ProviderStatus::Unreachable,
            format!("Ollama returned HTTP {status}{}.", response_detail(&body)),
            Vec::new(),
        ),
        Err(message) => ollama_provider(checked_at, ProviderStatus::Unreachable, message, Vec::new()),
    }
}

pub(crate) fn parse_ollama_model_names(body: &str) -> Result<Vec<String>, String> {
    let response = serde_json::from_str::<OllamaTagsResponse>(body)
        .map_err(|error| format!("Ollama tags response was not parseable: {error}."))?;
    let mut names = Vec::new();
    let mut seen = HashSet::new();
    for model in response.models {
        for value in [model.name, model.model].into_iter().flatten() {
            let value = value.trim().to_string();
            if !value.is_empty() && seen.insert(value.clone()) {
                names.push(value);
            }
        }
    }
    Ok(names)
}

fn fetch_tags(timeout: Duration) -> Result<(u16, String), String> {
    let addr: SocketAddr = OLLAMA_ADDR.parse().map_err(|_| "Invalid Ollama address.".to_string())?;
    let mut stream = TcpStream::connect_timeout(&addr, timeout)
        .map_err(|_| format!("Ollama is not reachable at {OLLAMA_ADDR}."))?;
    stream.set_read_timeout(Some(timeout)).ok();
    stream.set_write_timeout(Some(timeout)).ok();
    stream
        .write_all(b"GET /api/tags HTTP/1.1\r\nHost: 127.0.0.1:11434\r\nConnection: close\r\n\r\n")
        .map_err(|error| format!("Ollama health request failed: {error}."))?;
    let mut response = String::new();
    stream
        .read_to_string(&mut response)
        .map_err(|error| format!("Ollama health response failed: {error}."))?;
    Ok(split_http_response(&response))
}

fn split_http_response(response: &str) -> (u16, String) {
    let status = response
        .lines()
        .next()
        .and_then(|line| line.split_whitespace().nth(1))
        .and_then(|value| value.parse::<u16>().ok())
        .unwrap_or(0);
    let body = response.split_once("\r\n\r\n").map(|(_, body)| body).unwrap_or("").to_string();
    (status, body)
}

fn provider_from_models(checked_at: u64, names: Vec<String>) -> ModelProvider {
    if names.is_empty() {
        return ollama_provider(
            checked_at,
            ProviderStatus::NotConfigured,
            "Ollama is running, but no local models are installed.".to_string(),
            Vec::new(),
        );
    }
    let models = names.into_iter().map(ollama_model).collect::<Vec<_>>();
    ollama_provider(
        checked_at,
        ProviderStatus::Ready,
        format!("{} local model(s) available.", models.len()),
        models,
    )
}

fn ollama_provider(
    checked_at: u64,
    status: ProviderStatus,
    message: String,
    models: Vec<ModelInfo>,
) -> ModelProvider {
    ModelProvider {
        health: ProviderHealth { checked_at, message, status },
        id: OLLAMA_ID.to_string(),
        kind: ProviderKind::Ollama,
        label: "Ollama".to_string(),
        models,
        secret_policy: SecretPolicy::NoSecretRequired,
    }
}

fn ollama_model(id: String) -> ModelInfo {
    ModelInfo {
        context_window: 0,
        display_name: id.clone(),
        id,
        supports_tools: false,
    }
}

fn response_detail(body: &str) -> String {
    let body = body.trim();
    if body.is_empty() {
        String::new()
    } else {
        format!(": {}", body.chars().take(180).collect::<String>())
    }
}
