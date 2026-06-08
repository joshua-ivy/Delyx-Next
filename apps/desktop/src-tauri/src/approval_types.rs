#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ActionProposal {
    pub id: String,
    pub run_id: String,
    pub node_id: String,
    pub action: RiskyAction,
    pub risk: RiskLevel,
    pub scope: String,
    pub reason: String,
    pub expected_result: String,
    pub rollback_plan: String,
    pub expires_at: u64,
    pub status: ProposalStatus,
    pub decision: Option<ApprovalDecision>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RiskyAction {
    FileWrite,
    TerminalCommand,
    DependencyInstall,
    ConnectorWrite,
    DurableMemorySave,
    ScheduledRiskyAction,
    ExternalAgentExecution,
    ExternalSend,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum RiskLevel {
    Low,
    Medium,
    High,
    Dangerous,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RiskTaxonomyEntry {
    pub action: RiskyAction,
    pub minimum_risk: RiskLevel,
    pub summary: &'static str,
    pub rollback_required: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProposalStatus {
    Pending,
    Approved,
    Denied,
    Expired,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ApprovalDecisionKind {
    Approve,
    Deny,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ApprovalDecision {
    pub kind: ApprovalDecisionKind,
    pub decided_at: u64,
    pub note: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ApprovalGateState {
    WaitingForApproval,
    Ready,
    Blocked,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ApprovalError {
    ActionMismatch {
        expected: RiskyAction,
        actual: RiskyAction,
    },
    NodeMismatch {
        expected: String,
        actual: String,
    },
    RunMismatch {
        expected: String,
        actual: String,
    },
    ProposalNotFound,
    AlreadyDecided,
    Expired,
    NotApproved,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProposalInput {
    pub run_id: String,
    pub node_id: String,
    pub action: RiskyAction,
    pub risk: RiskLevel,
    pub scope: String,
    pub reason: String,
    pub expected_result: String,
    pub rollback_plan: String,
    pub expires_at: u64,
}
