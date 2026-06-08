use crate::external_agent::{AdapterStatus, ExternalAgentAvailability, ExternalAgentKind};
use crate::external_agent_adapters::default_adapters;
use serde::Serialize;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ExternalAgentStatusView {
    pub adapters: Vec<ExternalAgentAdapterStatusView>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ExternalAgentAdapterStatusView {
    pub id: String,
    pub kind: String,
    pub label: String,
    pub status: String,
    pub detail: String,
}

#[tauri::command]
pub fn external_agent_status() -> ExternalAgentStatusView {
    external_agent_status_from_adapters(default_adapters())
}

pub fn external_agent_status_from_adapters(
    adapters: Vec<ExternalAgentAvailability>,
) -> ExternalAgentStatusView {
    ExternalAgentStatusView {
        adapters: adapters.into_iter().map(adapter_status).collect(),
    }
}

fn adapter_status(adapter: ExternalAgentAvailability) -> ExternalAgentAdapterStatusView {
    ExternalAgentAdapterStatusView {
        detail: adapter_detail(adapter.kind, adapter.detail),
        id: adapter.adapter_id,
        kind: kind_key(adapter.kind).to_string(),
        label: adapter.display_name,
        status: status_key(adapter.status).to_string(),
    }
}

fn adapter_detail(kind: ExternalAgentKind, detail: String) -> String {
    match kind {
        ExternalAgentKind::ClaudeCode => {
            format!("{detail} Approval-gated read and write launch is available; every run still requires external-agent and terminal approvals.")
        }
        _ => detail,
    }
}

fn kind_key(kind: ExternalAgentKind) -> &'static str {
    match kind {
        ExternalAgentKind::ClaudeCode => "claude_code",
        ExternalAgentKind::CodexCli => "codex_cli",
        ExternalAgentKind::GenericTerminal => "generic_terminal",
    }
}

fn status_key(status: AdapterStatus) -> &'static str {
    match status {
        AdapterStatus::Available => "available",
        AdapterStatus::Missing => "missing",
        AdapterStatus::NotChecked => "not_checked",
    }
}
