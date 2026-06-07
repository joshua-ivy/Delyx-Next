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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RiskLevel {
    Low,
    Medium,
    High,
    Dangerous,
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
    ProposalNotFound,
    AlreadyDecided,
    Expired,
    NotApproved,
}

#[derive(Debug, Default)]
pub struct ApprovalEngine {
    proposals: Vec<ActionProposal>,
    next_id: usize,
}

impl ApprovalEngine {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn propose(&mut self, input: ProposalInput) -> ActionProposal {
        self.next_id += 1;
        let proposal = ActionProposal {
            id: format!("prop-{}", self.next_id),
            run_id: input.run_id,
            node_id: input.node_id,
            action: input.action,
            risk: input.risk,
            scope: input.scope,
            reason: input.reason,
            expected_result: input.expected_result,
            rollback_plan: input.rollback_plan,
            expires_at: input.expires_at,
            status: ProposalStatus::Pending,
            decision: None,
        };
        self.proposals.push(proposal.clone());
        proposal
    }

    pub fn list_proposals(&self, run_id: &str) -> Vec<&ActionProposal> {
        self.proposals.iter().filter(|proposal| proposal.run_id == run_id).collect()
    }

    pub fn approve(&mut self, proposal_id: &str, now: u64, note: &str) -> Result<(), ApprovalError> {
        let proposal = self.proposal_mut(proposal_id)?;
        ensure_pending(proposal)?;
        if now > proposal.expires_at {
            proposal.status = ProposalStatus::Expired;
            return Err(ApprovalError::Expired);
        }
        proposal.status = ProposalStatus::Approved;
        proposal.decision = Some(decision(ApprovalDecisionKind::Approve, now, note));
        Ok(())
    }

    pub fn deny(&mut self, proposal_id: &str, now: u64, note: &str) -> Result<(), ApprovalError> {
        let proposal = self.proposal_mut(proposal_id)?;
        ensure_pending(proposal)?;
        proposal.status = ProposalStatus::Denied;
        proposal.decision = Some(decision(ApprovalDecisionKind::Deny, now, note));
        Ok(())
    }

    pub fn expire_due(&mut self, now: u64) {
        for proposal in &mut self.proposals {
            if proposal.status == ProposalStatus::Pending && now > proposal.expires_at {
                proposal.status = ProposalStatus::Expired;
            }
        }
    }

    pub fn gate_state(&self, proposal_id: &str, now: u64) -> Result<ApprovalGateState, ApprovalError> {
        let proposal = self.proposal(proposal_id)?;
        Ok(match proposal.status {
            ProposalStatus::Pending if now <= proposal.expires_at => ApprovalGateState::WaitingForApproval,
            ProposalStatus::Approved if now <= proposal.expires_at => ApprovalGateState::Ready,
            _ => ApprovalGateState::Blocked,
        })
    }

    pub fn assert_can_execute(&self, proposal_id: &str, now: u64) -> Result<(), ApprovalError> {
        match self.gate_state(proposal_id, now)? {
            ApprovalGateState::Ready => Ok(()),
            ApprovalGateState::WaitingForApproval => Err(ApprovalError::NotApproved),
            ApprovalGateState::Blocked => Err(ApprovalError::NotApproved),
        }
    }

    fn proposal(&self, proposal_id: &str) -> Result<&ActionProposal, ApprovalError> {
        self.proposals
            .iter()
            .find(|proposal| proposal.id == proposal_id)
            .ok_or(ApprovalError::ProposalNotFound)
    }

    fn proposal_mut(&mut self, proposal_id: &str) -> Result<&mut ActionProposal, ApprovalError> {
        self.proposals
            .iter_mut()
            .find(|proposal| proposal.id == proposal_id)
            .ok_or(ApprovalError::ProposalNotFound)
    }
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

fn ensure_pending(proposal: &ActionProposal) -> Result<(), ApprovalError> {
    (proposal.status == ProposalStatus::Pending).then_some(()).ok_or(ApprovalError::AlreadyDecided)
}

fn decision(kind: ApprovalDecisionKind, decided_at: u64, note: &str) -> ApprovalDecision {
    ApprovalDecision { kind, decided_at, note: note.to_string() }
}
