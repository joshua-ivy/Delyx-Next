use crate::approval::{ProposalStatus, RiskLevel, RiskyAction};

pub(crate) fn parse_action(action: &str) -> Result<RiskyAction, String> {
    match action {
        "edit_file" | "write_file" => Ok(RiskyAction::FileWrite),
        "external_agent" => Ok(RiskyAction::ExternalAgentExecution),
        "external_send" => Ok(RiskyAction::ExternalSend),
        "install_dependency" => Ok(RiskyAction::DependencyInstall),
        "run_terminal" => Ok(RiskyAction::TerminalCommand),
        "save_memory" => Ok(RiskyAction::DurableMemorySave),
        "schedule_work" => Ok(RiskyAction::ScheduledRiskyAction),
        "use_connector" => Ok(RiskyAction::ConnectorWrite),
        _ => Err("Unsupported risky action for approval bridge.".to_string()),
    }
}

pub(crate) fn parse_risk(risk: &str) -> Result<RiskLevel, String> {
    match risk {
        "dangerous" => Ok(RiskLevel::Dangerous),
        "high" => Ok(RiskLevel::High),
        "low" => Ok(RiskLevel::Low),
        "medium" => Ok(RiskLevel::Medium),
        _ => Err("Unsupported risk label.".to_string()),
    }
}

pub(crate) fn risk_key(risk: RiskLevel) -> &'static str {
    match risk {
        RiskLevel::Dangerous => "dangerous",
        RiskLevel::High => "high",
        RiskLevel::Low => "low",
        RiskLevel::Medium => "medium",
    }
}

pub(crate) fn status_key(status: ProposalStatus) -> &'static str {
    match status {
        ProposalStatus::Approved => "approved",
        ProposalStatus::Denied => "denied",
        ProposalStatus::Expired => "expired",
        ProposalStatus::Pending => "pending",
    }
}
