use crate::model_provider::{
    ModelInfo, ModelProvider, ProviderHealth, ProviderKind, ProviderStatus, SecretPolicy,
};
use serde::{Deserialize, Serialize};
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

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub struct OllamaChatMessage {
    pub role: String,
    pub content: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OllamaChatResult {
    pub provider_id: String,
    pub model: String,
    pub text: String,
}

#[derive(Debug, Serialize)]
struct OllamaChatRequestBody<'a> {
    model: &'a str,
    messages: &'a [OllamaChatMessage],
    stream: bool,
}

#[derive(Debug, Deserialize)]
struct OllamaChatHttpResponse {
    message: Option<OllamaChatResponseMessage>,
    response: Option<String>,
}

#[derive(Debug, Deserialize)]
struct OllamaChatResponseMessage {
    content: Option<String>,
}

pub fn detect_local_ollama_provider(checked_at: u64, timeout: Duration) -> ModelProvider {
    provider_from_tags_result(checked_at, fetch_tags(timeout))
}

pub fn send_ollama_chat(
    model: String,
    messages: Vec<OllamaChatMessage>,
    timeout: Duration,
) -> Result<OllamaChatResult, String> {
    let model = validate_model(model)?;
    let messages = validate_messages(messages)?;
    chat_from_http_result(&model, fetch_chat(&model, &messages, timeout))
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
        Err(message) => {
            ollama_provider(checked_at, ProviderStatus::Unreachable, message, Vec::new())
        }
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

pub(crate) fn chat_from_http_result(
    model: &str,
    result: Result<(u16, String), String>,
) -> Result<OllamaChatResult, String> {
    match result {
        Ok((200, body)) => {
            let text = parse_ollama_chat_text(&body)?;
            Ok(OllamaChatResult {
                model: model.to_string(),
                provider_id: OLLAMA_ID.to_string(),
                text,
            })
        }
        Ok((status, body)) => Err(format!(
            "Ollama chat failed with HTTP {status}{}.",
            response_detail(&body)
        )),
        Err(message) => Err(message),
    }
}

fn parse_ollama_chat_text(body: &str) -> Result<String, String> {
    let response = serde_json::from_str::<OllamaChatHttpResponse>(body)
        .map_err(|error| format!("Ollama chat response was not parseable: {error}."))?;
    let text = response
        .message
        .and_then(|message| message.content)
        .or(response.response)
        .unwrap_or_default()
        .trim()
        .to_string();
    if text.is_empty() {
        return Err("Ollama returned an empty response.".to_string());
    }
    Ok(text)
}

fn fetch_tags(timeout: Duration) -> Result<(u16, String), String> {
    let addr: SocketAddr = OLLAMA_ADDR
        .parse()
        .map_err(|_| "Invalid Ollama address.".to_string())?;
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

fn fetch_chat(
    model: &str,
    messages: &[OllamaChatMessage],
    timeout: Duration,
) -> Result<(u16, String), String> {
    let body = serde_json::to_string(&OllamaChatRequestBody {
        model,
        messages,
        stream: false,
    })
    .map_err(|error| format!("Ollama chat request was not serializable: {error}."))?;
    let request = format!(
        "POST /api/chat HTTP/1.1\r\nHost: 127.0.0.1:11434\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
        body.as_bytes().len(),
        body
    );
    let addr: SocketAddr = OLLAMA_ADDR
        .parse()
        .map_err(|_| "Invalid Ollama address.".to_string())?;
    let mut stream = TcpStream::connect_timeout(&addr, timeout)
        .map_err(|_| format!("Ollama is not reachable at {OLLAMA_ADDR}."))?;
    stream.set_read_timeout(Some(timeout)).ok();
    stream.set_write_timeout(Some(timeout)).ok();
    stream
        .write_all(request.as_bytes())
        .map_err(|error| format!("Ollama chat request failed: {error}."))?;
    let mut response = String::new();
    stream
        .read_to_string(&mut response)
        .map_err(|error| format!("Ollama chat response failed: {error}."))?;
    Ok(split_http_response(&response))
}

pub(crate) fn split_http_response(response: &str) -> (u16, String) {
    let status = response
        .lines()
        .next()
        .and_then(|line| line.split_whitespace().nth(1))
        .and_then(|value| value.parse::<u16>().ok())
        .unwrap_or(0);
    let (head, body) = response.split_once("\r\n\r\n").unwrap_or(("", ""));
    // Ollama sends larger replies with `Transfer-Encoding: chunked` (no
    // Content-Length); the raw body then carries hex chunk-size lines that are
    // not JSON. Decode them so the parser sees only the payload.
    let body = if is_chunked(head) {
        dechunk(body)
    } else {
        body.to_string()
    };
    (status, body)
}

fn is_chunked(head: &str) -> bool {
    head.lines().any(|line| {
        line.split_once(':').is_some_and(|(name, value)| {
            name.trim().eq_ignore_ascii_case("transfer-encoding")
                && value.to_ascii_lowercase().contains("chunked")
        })
    })
}

/// Decode an HTTP/1.1 chunked body. Operates on bytes so a multi-byte UTF-8
/// character split across a chunk boundary is reassembled before decoding.
fn dechunk(body: &str) -> String {
    let bytes = body.as_bytes();
    let mut out: Vec<u8> = Vec::with_capacity(bytes.len());
    let mut index = 0;
    while index < bytes.len() {
        let Some(line_end) = find_crlf(bytes, index) else {
            break;
        };
        // The chunk-size line may carry `;ext` extensions after the size.
        let size_field = bytes[index..line_end]
            .split(|&byte| byte == b';')
            .next()
            .unwrap_or(&[]);
        let Ok(size_text) = std::str::from_utf8(size_field) else {
            break;
        };
        let Ok(size) = usize::from_str_radix(size_text.trim(), 16) else {
            break;
        };
        if size == 0 {
            break;
        }
        let data_start = line_end + 2;
        let data_end = data_start.saturating_add(size).min(bytes.len());
        out.extend_from_slice(&bytes[data_start..data_end]);
        index = data_end;
        if bytes.get(index) == Some(&b'\r') && bytes.get(index + 1) == Some(&b'\n') {
            index += 2;
        }
    }
    String::from_utf8_lossy(&out).into_owned()
}

fn find_crlf(bytes: &[u8], from: usize) -> Option<usize> {
    (from..bytes.len().saturating_sub(1)).find(|&i| bytes[i] == b'\r' && bytes[i + 1] == b'\n')
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
        health: ProviderHealth {
            checked_at,
            message,
            status,
        },
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
        format: None,
        runtime: None,
        path: None,
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

fn validate_model(model: String) -> Result<String, String> {
    let model = model.trim().to_string();
    if model.is_empty() {
        return Err("Ollama chat requires a selected model.".to_string());
    }
    Ok(model)
}

fn validate_messages(messages: Vec<OllamaChatMessage>) -> Result<Vec<OllamaChatMessage>, String> {
    if messages.is_empty() {
        return Err("Ollama chat requires at least one message.".to_string());
    }
    messages.into_iter().map(validate_message).collect()
}

fn validate_message(message: OllamaChatMessage) -> Result<OllamaChatMessage, String> {
    let role = message.role.trim().to_string();
    if !matches!(role.as_str(), "assistant" | "system" | "user") {
        return Err(format!(
            "Ollama chat message role `{role}` is not supported."
        ));
    }
    let content = message.content.trim().to_string();
    if content.is_empty() {
        return Err("Ollama chat messages cannot be empty.".to_string());
    }
    Ok(OllamaChatMessage { role, content })
}
