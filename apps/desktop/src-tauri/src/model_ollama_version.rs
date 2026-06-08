use serde::Deserialize;
use std::io::{Read, Write};
use std::net::{SocketAddr, TcpStream};
use std::time::Duration;

const OLLAMA_ADDR: &str = "127.0.0.1:11434";

#[derive(Debug, Deserialize)]
struct OllamaVersionResponse {
    version: Option<String>,
}

pub fn detect_local_ollama_version(timeout: Duration) -> Option<String> {
    version_from_http_result(fetch_version(timeout))
}

pub(crate) fn version_from_http_result(result: Result<(u16, String), String>) -> Option<String> {
    match result {
        Ok((200, body)) => parse_ollama_version(&body).ok().flatten(),
        _ => None,
    }
}

pub(crate) fn parse_ollama_version(body: &str) -> Result<Option<String>, String> {
    let response = serde_json::from_str::<OllamaVersionResponse>(body)
        .map_err(|error| format!("Ollama version response was not parseable: {error}."))?;
    Ok(response
        .version
        .map(|version| version.trim().to_string())
        .filter(|version| !version.is_empty()))
}

fn fetch_version(timeout: Duration) -> Result<(u16, String), String> {
    let addr: SocketAddr = OLLAMA_ADDR
        .parse()
        .map_err(|_| "Invalid Ollama address.".to_string())?;
    let mut stream = TcpStream::connect_timeout(&addr, timeout)
        .map_err(|_| format!("Ollama is not reachable at {OLLAMA_ADDR}."))?;
    stream.set_read_timeout(Some(timeout)).ok();
    stream.set_write_timeout(Some(timeout)).ok();
    stream
        .write_all(
            b"GET /api/version HTTP/1.1\r\nHost: 127.0.0.1:11434\r\nConnection: close\r\n\r\n",
        )
        .map_err(|error| format!("Ollama version request failed: {error}."))?;
    let mut response = String::new();
    stream
        .read_to_string(&mut response)
        .map_err(|error| format!("Ollama version response failed: {error}."))?;
    Ok(split_http_response(&response))
}

fn split_http_response(response: &str) -> (u16, String) {
    let status = response
        .lines()
        .next()
        .and_then(|line| line.split_whitespace().nth(1))
        .and_then(|value| value.parse::<u16>().ok())
        .unwrap_or(0);
    let body = response
        .split_once("\r\n\r\n")
        .map(|(_, body)| body)
        .unwrap_or("")
        .to_string();
    (status, body)
}
