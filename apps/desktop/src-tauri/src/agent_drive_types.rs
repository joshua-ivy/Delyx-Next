use crate::agent_drive::AgentDriveContext;
use serde::Serialize;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentDriveOutcomeView {
    pub run_id: String,
    pub steps: Vec<AgentDriveStepView>,
    pub stopped_because: AgentDriveStopView,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentDriveStepView {
    pub decision: String,
    pub status: String,
    pub message: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentDriveStopView {
    pub kind: String,
    pub message: String,
    pub approval_ids: Vec<String>,
    pub proposal_id: Option<String>,
    pub review_report_id: Option<String>,
    pub finding_id: Option<String>,
    pub status: Option<String>,
}

pub(crate) fn step(
    decision: impl Into<String>,
    status: impl Into<String>,
    message: impl Into<String>,
) -> AgentDriveStepView {
    AgentDriveStepView {
        decision: decision.into(),
        status: status.into(),
        message: message.into(),
    }
}

pub(crate) fn outcome(
    context: &AgentDriveContext<'_>,
    steps: Vec<AgentDriveStepView>,
    stopped_because: AgentDriveStopView,
) -> AgentDriveOutcomeView {
    AgentDriveOutcomeView {
        run_id: context.run_id.clone(),
        steps,
        stopped_because,
    }
}

pub(crate) fn stop(kind: &str, message: impl Into<String>) -> AgentDriveStopView {
    AgentDriveStopView {
        approval_ids: Vec::new(),
        finding_id: None,
        kind: kind.to_string(),
        message: message.into(),
        proposal_id: None,
        review_report_id: None,
        status: None,
    }
}

pub(crate) fn stop_with_approvals(
    kind: &str,
    message: impl Into<String>,
    approval_ids: Vec<String>,
) -> AgentDriveStopView {
    AgentDriveStopView {
        approval_ids,
        ..stop(kind, message)
    }
}

pub(crate) fn stop_with_proposal(
    kind: &str,
    message: impl Into<String>,
    proposal_id: Option<String>,
) -> AgentDriveStopView {
    AgentDriveStopView {
        proposal_id,
        ..stop(kind, message)
    }
}

pub(crate) fn stop_with_review(
    kind: &str,
    message: impl Into<String>,
    review_report_id: Option<String>,
    finding_id: Option<String>,
) -> AgentDriveStopView {
    AgentDriveStopView {
        finding_id,
        review_report_id,
        ..stop(kind, message)
    }
}

pub(crate) fn stop_with_status(
    kind: &str,
    message: impl Into<String>,
    status: Option<String>,
) -> AgentDriveStopView {
    AgentDriveStopView {
        status,
        ..stop(kind, message)
    }
}
