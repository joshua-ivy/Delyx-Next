use crate::model_ollama::OLLAMA_ID;
use crate::model_provider::{
    ModelInfo, ModelProvider, ProviderHealth, ProviderKind, ProviderStatus, SecretPolicy,
};

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

pub(crate) fn provider_from_models(checked_at: u64, names: Vec<String>) -> ModelProvider {
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

pub(crate) fn ollama_provider(
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
