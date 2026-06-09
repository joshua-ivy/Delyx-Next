//! Persistence for Delyx-managed local model profiles.
//!
//! Stores profile metadata and the on-disk path to the model weights — never the
//! weights themselves. Removing a profile never deletes the model file.

use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LocalModelProfile {
    pub id: String,
    pub display_name: String,
    pub runtime: String,
    pub format: String,
    pub model_path: String,
    pub chat_template_path: Option<String>,
    pub tokenizer_path: Option<String>,
    pub context_window: u32,
    pub supports_tools: bool,
    pub sha256: Option<String>,
    pub size_bytes: Option<u64>,
    pub load_status: String,
    pub last_error: Option<String>,
    pub temperature: Option<f64>,
    pub top_p: Option<f64>,
    pub top_k: Option<u32>,
    pub repeat_penalty: Option<f32>,
    pub max_tokens: Option<u32>,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ModelSamplingRequest {
    pub id: String,
    pub temperature: Option<f64>,
    pub top_p: Option<f64>,
    pub top_k: Option<u32>,
    pub repeat_penalty: Option<f32>,
    pub max_tokens: Option<u32>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ImportLocalModelRequest {
    pub model_path: String,
    pub display_name: Option<String>,
    pub chat_template_path: Option<String>,
    pub tokenizer_path: Option<String>,
    pub context_window: Option<u32>,
}

pub fn import_profile_to_path(
    path: &Path,
    request: ImportLocalModelRequest,
) -> Result<LocalModelProfile, String> {
    let profile = profile_from_request(request)?;
    let connection = crate::sqlite_store::open_migrated_database(path).map_err(sql_string)?;
    upsert_profile(&connection, &profile)?;
    Ok(profile)
}

pub fn list_profiles_from_path(path: &Path) -> Result<Vec<LocalModelProfile>, String> {
    let connection = crate::sqlite_store::open_migrated_database(path).map_err(sql_string)?;
    list_profiles(&connection)
}

pub fn load_profile_from_path(path: &Path, id: &str) -> Result<LocalModelProfile, String> {
    list_profiles_from_path(path)?
        .into_iter()
        .find(|profile| profile.id == id)
        .ok_or_else(|| format!("Local model profile `{id}` was not found."))
}

pub fn delete_profile_from_path(path: &Path, id: &str) -> Result<(), String> {
    let connection = crate::sqlite_store::open_migrated_database(path).map_err(sql_string)?;
    connection
        .execute(
            "DELETE FROM local_model_profiles WHERE id = ?1",
            params![id.trim()],
        )
        .map(|_| ())
        .map_err(sql_string)
}

pub fn mark_profile_status(
    path: &Path,
    id: &str,
    status: &str,
    error: Option<&str>,
) -> Result<(), String> {
    let connection = crate::sqlite_store::open_migrated_database(path).map_err(sql_string)?;
    connection
        .execute(
            "UPDATE local_model_profiles
             SET load_status = ?2, last_error = ?3, updated_at = CURRENT_TIMESTAMP
             WHERE id = ?1",
            params![id, status, error],
        )
        .map(|_| ())
        .map_err(sql_string)
}

fn profile_from_request(request: ImportLocalModelRequest) -> Result<LocalModelProfile, String> {
    let model_path = PathBuf::from(request.model_path.trim());
    if !model_path.is_file() {
        return Err(format!(
            "Model file does not exist: {}",
            model_path.display()
        ));
    }
    let extension = model_path
        .extension()
        .and_then(|value| value.to_str())
        .unwrap_or_default()
        .to_ascii_lowercase();
    if extension != "gguf" {
        return Err("First embedded runtime milestone only supports .gguf files.".to_string());
    }
    let file_name = model_path
        .file_name()
        .and_then(|value| value.to_str())
        .ok_or_else(|| "Model path has no filename.".to_string())?;
    let id = stable_profile_id(file_name);
    let metadata = std::fs::metadata(&model_path).map_err(|error| error.to_string())?;
    Ok(LocalModelProfile {
        id,
        display_name: request
            .display_name
            .unwrap_or_else(|| file_name.to_string()),
        runtime: "mistralrs".to_string(),
        format: "gguf".to_string(),
        model_path: model_path.display().to_string(),
        chat_template_path: non_empty(request.chat_template_path),
        tokenizer_path: non_empty(request.tokenizer_path),
        context_window: request.context_window.unwrap_or(8192),
        supports_tools: false,
        sha256: None,
        size_bytes: Some(metadata.len()),
        load_status: "unloaded".to_string(),
        last_error: None,
        temperature: None,
        top_p: None,
        top_k: None,
        repeat_penalty: None,
        max_tokens: None,
    })
}

pub fn set_sampling_to_path(path: &Path, request: ModelSamplingRequest) -> Result<(), String> {
    let connection = crate::sqlite_store::open_migrated_database(path).map_err(sql_string)?;
    let changed = connection
        .execute(
            "UPDATE local_model_profiles
             SET temperature = ?2, top_p = ?3, top_k = ?4, repeat_penalty = ?5, max_tokens = ?6,
                 updated_at = CURRENT_TIMESTAMP
             WHERE id = ?1",
            params![
                request.id.trim(),
                request.temperature,
                request.top_p,
                request.top_k,
                request.repeat_penalty.map(|value| value as f64),
                request.max_tokens,
            ],
        )
        .map_err(sql_string)?;
    if changed == 0 {
        return Err(format!(
            "Local model profile `{}` was not found.",
            request.id
        ));
    }
    Ok(())
}

fn stable_profile_id(file_name: &str) -> String {
    let mut id = file_name
        .trim()
        .trim_end_matches(".gguf")
        .chars()
        .map(|ch| {
            if ch.is_ascii_alphanumeric() {
                ch.to_ascii_lowercase()
            } else {
                '-'
            }
        })
        .collect::<String>();
    while id.contains("--") {
        id = id.replace("--", "-");
    }
    id.trim_matches('-').to_string()
}

fn non_empty(value: Option<String>) -> Option<String> {
    value
        .map(|item| item.trim().to_string())
        .filter(|item| !item.is_empty())
}

fn upsert_profile(connection: &Connection, profile: &LocalModelProfile) -> Result<(), String> {
    connection
        .execute(
            "INSERT INTO local_model_profiles (
                id, display_name, runtime, format, model_path, chat_template_path, tokenizer_path,
                context_window, supports_tools, sha256, size_bytes, load_status, last_error,
                temperature, top_p, top_k, repeat_penalty, max_tokens
             ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17, ?18)
             ON CONFLICT(id) DO UPDATE SET
                display_name = excluded.display_name,
                runtime = excluded.runtime,
                format = excluded.format,
                model_path = excluded.model_path,
                chat_template_path = excluded.chat_template_path,
                tokenizer_path = excluded.tokenizer_path,
                context_window = excluded.context_window,
                supports_tools = excluded.supports_tools,
                sha256 = excluded.sha256,
                size_bytes = excluded.size_bytes,
                load_status = excluded.load_status,
                last_error = excluded.last_error,
                updated_at = CURRENT_TIMESTAMP",
            params![
                profile.id,
                profile.display_name,
                profile.runtime,
                profile.format,
                profile.model_path,
                profile.chat_template_path,
                profile.tokenizer_path,
                profile.context_window,
                profile.supports_tools as i64,
                profile.sha256,
                profile.size_bytes.map(|value| value as i64),
                profile.load_status,
                profile.last_error,
                profile.temperature,
                profile.top_p,
                profile.top_k,
                profile.repeat_penalty.map(|value| value as f64),
                profile.max_tokens,
            ],
        )
        .map(|_| ())
        .map_err(sql_string)
}

fn list_profiles(connection: &Connection) -> Result<Vec<LocalModelProfile>, String> {
    let mut statement = connection
        .prepare(
            "SELECT id, display_name, runtime, format, model_path, chat_template_path, tokenizer_path,
                    context_window, supports_tools, sha256, size_bytes, load_status, last_error,
                    temperature, top_p, top_k, repeat_penalty, max_tokens
             FROM local_model_profiles
             ORDER BY display_name",
        )
        .map_err(sql_string)?;
    let rows = statement
        .query_map([], |row| {
            Ok(LocalModelProfile {
                id: row.get(0)?,
                display_name: row.get(1)?,
                runtime: row.get(2)?,
                format: row.get(3)?,
                model_path: row.get(4)?,
                chat_template_path: row.get(5)?,
                tokenizer_path: row.get(6)?,
                context_window: row.get::<_, i64>(7)? as u32,
                supports_tools: row.get::<_, i64>(8)? != 0,
                sha256: row.get(9)?,
                size_bytes: row.get::<_, Option<i64>>(10)?.map(|value| value as u64),
                load_status: row.get(11)?,
                last_error: row.get(12)?,
                temperature: row.get(13)?,
                top_p: row.get(14)?,
                top_k: row.get::<_, Option<i64>>(15)?.map(|value| value as u32),
                repeat_penalty: row.get::<_, Option<f64>>(16)?.map(|value| value as f32),
                max_tokens: row.get::<_, Option<i64>>(17)?.map(|value| value as u32),
            })
        })
        .map_err(sql_string)?;
    rows.collect::<Result<Vec<_>, _>>().map_err(sql_string)
}

fn sql_string(error: rusqlite::Error) -> String {
    error.to_string()
}
