use crate::approval::{RiskLevel, RiskyAction};
use serde::Serialize;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RiskTaxonomyBridgeView {
    pub action_type: String,
    pub minimum_risk: String,
    pub summary: String,
    pub rollback_required: bool,
}

#[tauri::command]
pub fn approval_taxonomy() -> Vec<RiskTaxonomyBridgeView> {
    approval_taxonomy_records()
}

pub fn approval_taxonomy_records() -> Vec<RiskTaxonomyBridgeView> {
    [
        ("edit_file", RiskyAction::FileWrite),
        ("write_file", RiskyAction::FileWrite),
        ("external_agent", RiskyAction::ExternalAgentExecution),
        ("external_send", RiskyAction::ExternalSend),
        ("install_dependency", RiskyAction::DependencyInstall),
        ("run_terminal", RiskyAction::TerminalCommand),
        ("save_memory", RiskyAction::DurableMemorySave),
        ("schedule_work", RiskyAction::ScheduledRiskyAction),
        ("use_connector", RiskyAction::ConnectorWrite),
    ]
    .into_iter()
    .map(taxonomy_entry)
    .collect()
}

fn taxonomy_entry((action_type, action): (&str, RiskyAction)) -> RiskTaxonomyBridgeView {
    let taxonomy = action.taxonomy();
    RiskTaxonomyBridgeView {
        action_type: action_type.to_string(),
        minimum_risk: risk_key(taxonomy.minimum_risk).to_string(),
        rollback_required: taxonomy.rollback_required,
        summary: taxonomy.summary.to_string(),
    }
}

fn risk_key(risk: RiskLevel) -> &'static str {
    match risk {
        RiskLevel::Dangerous => "dangerous",
        RiskLevel::High => "high",
        RiskLevel::Low => "low",
        RiskLevel::Medium => "medium",
    }
}
