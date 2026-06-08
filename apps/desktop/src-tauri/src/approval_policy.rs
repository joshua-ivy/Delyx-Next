use crate::approval::{RiskLevel, RiskTaxonomyEntry, RiskyAction};

impl RiskyAction {
    pub fn taxonomy(self) -> RiskTaxonomyEntry {
        match self {
            RiskyAction::FileWrite => entry(
                self,
                RiskLevel::High,
                "file writes require checkpoint scope",
                true,
            ),
            RiskyAction::TerminalCommand => entry(
                self,
                RiskLevel::Medium,
                "terminal commands require captured artifacts",
                false,
            ),
            RiskyAction::DependencyInstall => entry(
                self,
                RiskLevel::High,
                "dependency installs mutate the project",
                true,
            ),
            RiskyAction::ConnectorWrite => entry(
                self,
                RiskLevel::High,
                "connector writes leave the local trust boundary",
                true,
            ),
            RiskyAction::DurableMemorySave => entry(
                self,
                RiskLevel::Medium,
                "durable memory changes future runs",
                true,
            ),
            RiskyAction::ScheduledRiskyAction => entry(
                self,
                RiskLevel::Dangerous,
                "scheduled risky actions can run later without attention",
                true,
            ),
            RiskyAction::ExternalAgentExecution => entry(
                self,
                RiskLevel::High,
                "external agents run inside bounded scope only",
                true,
            ),
            RiskyAction::ExternalSend => entry(
                self,
                RiskLevel::High,
                "external sends disclose data outside the workspace",
                false,
            ),
        }
    }

    pub fn minimum_risk(self) -> RiskLevel {
        self.taxonomy().minimum_risk
    }

    pub fn normalize_risk(self, requested: RiskLevel) -> RiskLevel {
        requested.max(self.minimum_risk())
    }
}

fn entry(
    action: RiskyAction,
    minimum_risk: RiskLevel,
    summary: &'static str,
    rollback_required: bool,
) -> RiskTaxonomyEntry {
    RiskTaxonomyEntry {
        action,
        minimum_risk,
        summary,
        rollback_required,
    }
}
