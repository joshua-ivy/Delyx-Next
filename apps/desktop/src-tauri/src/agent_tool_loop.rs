//! Bounded agentic tool loop for the embedded local model: the model may call
//! read-only project tools (read_file / list_dir / grep) before answering. Tool
//! turns are consumed silently (a `tool-loop` event narrates them); the final
//! answer streams to the UI token-by-token through the existing `model-stream`
//! event. Writes and commands are NOT exposed here — they stay approval-gated.

use crate::model_chat::{ModelChatMessage, ModelChatResult};
use crate::model_embedded::EmbeddedRuntimeState;
use crate::model_embedded_persistence::LocalModelProfile;
use serde::Serialize;
use std::path::Path;

pub const MAX_TOOL_STEPS: usize = 6;
pub const TOOL_LOOP_EVENT: &str = "tool-loop";

/// Narration event for each tool call so the UI can show live activity.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ToolLoopEvent {
    pub request_id: String,
    /// "tool" while calling, "tool_result" when done.
    pub kind: String,
    pub summary: String,
}

#[cfg(not(feature = "embedded_mistral"))]
#[allow(clippy::too_many_arguments)]
pub async fn run_tool_loop(
    _state: &EmbeddedRuntimeState,
    _app: &tauri::AppHandle,
    _database_path: &Path,
    _profile: LocalModelProfile,
    _messages: Vec<ModelChatMessage>,
    _project_root: String,
    _request_id: String,
) -> Result<ModelChatResult, String> {
    Err(
        "Delyx embedded runtime was not compiled. Build with --features embedded_mistral."
            .to_string(),
    )
}

#[cfg(feature = "embedded_mistral")]
#[allow(clippy::too_many_arguments)]
pub async fn run_tool_loop(
    state: &EmbeddedRuntimeState,
    app: &tauri::AppHandle,
    database_path: &Path,
    profile: LocalModelProfile,
    mut messages: Vec<ModelChatMessage>,
    project_root: String,
    request_id: String,
) -> Result<ModelChatResult, String> {
    use crate::agent_tools::{execute_tool, parse_tool_call, tool_protocol_prompt};
    use crate::model_embedded::{
        chat_request, describe_chat_error, load_or_get_model, validate_profile, ModelStreamEvent,
        MODEL_STREAM_EVENT,
    };
    use tauri::Emitter;

    validate_profile(&profile)?;
    state.take_cancel(&request_id);
    let root = Path::new(&project_root).to_path_buf();
    // Teach the protocol by extending the system message (or adding one).
    if let Some(system) = messages.iter_mut().find(|message| message.role == "system") {
        system.content = format!("{}\n\n{}", system.content, tool_protocol_prompt());
    } else {
        messages.insert(
            0,
            ModelChatMessage {
                role: "system".to_string(),
                content: tool_protocol_prompt().to_string(),
            },
        );
    }

    let loaded = load_or_get_model(state, database_path, &profile).await?;
    let mut final_text = String::new();
    let mut cancelled = false;

    'turns: for _step in 0..=MAX_TOOL_STEPS {
        let request = chat_request(messages.clone(), &profile)?;
        let mut stream = loaded
            .model
            .stream_chat_request(request)
            .await
            .map_err(|error| describe_chat_error(&error.to_string(), &profile.display_name))?;

        // Peek the first non-whitespace character: '{' means a silent tool turn;
        // anything else is the final answer and streams straight to the UI.
        let mut turn_text = String::new();
        let mut decided_tool: Option<bool> = None;
        while let Some(response) = stream.next().await {
            match response {
                mistralrs::Response::Chunk(chunk) => {
                    let delta = chunk
                        .choices
                        .first()
                        .and_then(|choice| choice.delta.content.clone())
                        .unwrap_or_default();
                    if !delta.is_empty() {
                        turn_text.push_str(&delta);
                        if decided_tool.is_none() {
                            if let Some(first) = turn_text.trim_start().chars().next() {
                                decided_tool =
                                    Some(first == '{' || turn_text.trim_start().starts_with("```"));
                            }
                        }
                        if decided_tool == Some(false) {
                            let _ = app.emit(
                                MODEL_STREAM_EVENT,
                                ModelStreamEvent {
                                    request_id: request_id.clone(),
                                    kind: "token".to_string(),
                                    text: delta,
                                },
                            );
                        }
                    }
                    let finished = chunk
                        .choices
                        .first()
                        .map(|choice| choice.finish_reason.is_some())
                        .unwrap_or(false);
                    if finished {
                        break;
                    }
                }
                mistralrs::Response::ModelError(message, _) => {
                    return Err(describe_chat_error(&message, &profile.display_name));
                }
                _ => {}
            }
            if state.take_cancel(&request_id) {
                cancelled = true;
                break;
            }
        }
        drop(stream);
        if cancelled {
            final_text = turn_text;
            break 'turns;
        }

        match parse_tool_call(&turn_text) {
            Some(call) => {
                let summary = call.summary();
                let _ = app.emit(
                    TOOL_LOOP_EVENT,
                    ToolLoopEvent {
                        request_id: request_id.clone(),
                        kind: "tool".to_string(),
                        summary: summary.clone(),
                    },
                );
                let result = execute_tool(&root, &call);
                let _ = app.emit(
                    TOOL_LOOP_EVENT,
                    ToolLoopEvent {
                        request_id: request_id.clone(),
                        kind: "tool_result".to_string(),
                        summary,
                    },
                );
                messages.push(ModelChatMessage {
                    role: "assistant".to_string(),
                    content: turn_text.trim().to_string(),
                });
                messages.push(ModelChatMessage {
                    role: "user".to_string(),
                    content: format!("Tool result:\n{result}\n\nContinue. Call another tool only if needed, otherwise give your final answer."),
                });
            }
            None => {
                final_text = turn_text;
                break 'turns;
            }
        }
    }

    let _ = app.emit(
        MODEL_STREAM_EVENT,
        ModelStreamEvent {
            request_id,
            kind: if cancelled { "cancelled" } else { "done" }.to_string(),
            text: final_text.clone(),
        },
    );
    let text = final_text.trim().to_string();
    if text.is_empty() && !cancelled {
        return Err("Local model returned no final answer within the tool budget.".to_string());
    }
    Ok(ModelChatResult {
        provider_id: "delyx-local".to_string(),
        model: profile.id,
        text,
    })
}
