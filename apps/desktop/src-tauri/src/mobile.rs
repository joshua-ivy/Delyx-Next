use crate::approval::{ActionProposal, ProposalStatus, RiskLevel, RiskyAction};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MobilePolicy {
    pub allow_low_risk_approval: bool,
    pub max_approval_risk: RiskLevel,
    pub can_access_files: bool,
    pub can_access_terminal: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MobileCompanionView {
    pub threads: Vec<MobileThreadView>,
    pub pending_approvals: Vec<MobileApprovalView>,
    pub runs: Vec<MobileRunView>,
    pub policy: MobilePolicy,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MobileThreadView {
    pub id: String,
    pub title: String,
    pub status: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MobileApprovalView {
    pub id: String,
    pub run_id: String,
    pub risk: RiskLevel,
    pub scope: String,
    pub reason: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MobileRunView {
    pub id: String,
    pub status: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MobileDecisionKind {
    Approve,
    Deny,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MobileDecision {
    pub proposal_id: String,
    pub kind: MobileDecisionKind,
    pub scope: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MobileError {
    BroadFileAccessDenied,
    BroadTerminalAccessDenied,
    Disabled,
    RiskExceedsDesktopPolicy,
    RiskTooHigh,
}

pub fn mobile_status_view(
    threads: Vec<MobileThreadView>,
    proposals: Vec<&ActionProposal>,
    runs: Vec<MobileRunView>,
    policy: MobilePolicy,
) -> MobileCompanionView {
    MobileCompanionView {
        pending_approvals: proposals
            .into_iter()
            .filter(|proposal| proposal.status == ProposalStatus::Pending)
            .map(approval_view)
            .collect(),
        threads,
        runs,
        policy,
    }
}

pub fn decide_mobile_approval(
    proposal: &ActionProposal,
    kind: MobileDecisionKind,
    policy: &MobilePolicy,
    desktop_max_risk: RiskLevel,
) -> Result<MobileDecision, MobileError> {
    if !policy.allow_low_risk_approval {
        return Err(MobileError::Disabled);
    }
    if risk_rank(policy.max_approval_risk) > risk_rank(desktop_max_risk) {
        return Err(MobileError::RiskExceedsDesktopPolicy);
    }
    if risk_rank(proposal.risk) > risk_rank(policy.max_approval_risk) {
        return Err(MobileError::RiskTooHigh);
    }
    match proposal.action {
        RiskyAction::FileWrite if !policy.can_access_files => Err(MobileError::BroadFileAccessDenied),
        RiskyAction::TerminalCommand if !policy.can_access_terminal => Err(MobileError::BroadTerminalAccessDenied),
        _ => Ok(MobileDecision { proposal_id: proposal.id.clone(), kind, scope: proposal.scope.clone() }),
    }
}

pub fn default_mobile_policy() -> MobilePolicy {
    MobilePolicy {
        allow_low_risk_approval: false,
        max_approval_risk: RiskLevel::Low,
        can_access_files: false,
        can_access_terminal: false,
    }
}

fn approval_view(proposal: &ActionProposal) -> MobileApprovalView {
    MobileApprovalView {
        id: proposal.id.clone(),
        run_id: proposal.run_id.clone(),
        risk: proposal.risk,
        scope: proposal.scope.clone(),
        reason: proposal.reason.clone(),
    }
}

fn risk_rank(risk: RiskLevel) -> u8 {
    match risk {
        RiskLevel::Low => 0,
        RiskLevel::Medium => 1,
        RiskLevel::High => 2,
        RiskLevel::Dangerous => 3,
    }
}
