pub use crate::approval_types::{
    ActionProposal, ApprovalDecision, ApprovalDecisionKind, ApprovalError, ApprovalGateState,
    ProposalInput, ProposalStatus, RiskLevel, RiskTaxonomyEntry, RiskyAction,
};

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
        let risk = input.action.normalize_risk(input.risk);
        let proposal = ActionProposal {
            id: format!("prop-{}", self.next_id),
            run_id: input.run_id,
            node_id: input.node_id,
            action: input.action,
            risk,
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
        self.proposals
            .iter()
            .filter(|proposal| proposal.run_id == run_id)
            .collect()
    }

    pub(crate) fn all_proposals(&self) -> &[ActionProposal] {
        &self.proposals
    }

    pub(crate) fn from_loaded(proposals: Vec<ActionProposal>) -> Self {
        let next_id = proposals
            .iter()
            .filter_map(|proposal| proposal.id.strip_prefix("prop-")?.parse::<usize>().ok())
            .max()
            .unwrap_or(proposals.len());
        Self { proposals, next_id }
    }

    pub fn approve(
        &mut self,
        proposal_id: &str,
        now: u64,
        note: &str,
    ) -> Result<(), ApprovalError> {
        let proposal = self.proposal_mut(proposal_id)?;
        ensure_pending(proposal)?;
        if now >= proposal.expires_at {
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
            if matches!(
                proposal.status,
                ProposalStatus::Pending | ProposalStatus::Approved
            ) && now >= proposal.expires_at
            {
                proposal.status = ProposalStatus::Expired;
            }
        }
    }

    pub fn gate_state(
        &self,
        proposal_id: &str,
        now: u64,
    ) -> Result<ApprovalGateState, ApprovalError> {
        let proposal = self.proposal(proposal_id)?;
        Ok(match proposal.status {
            ProposalStatus::Pending if now < proposal.expires_at => {
                ApprovalGateState::WaitingForApproval
            }
            ProposalStatus::Approved if now < proposal.expires_at => ApprovalGateState::Ready,
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

    pub fn assert_can_execute_action(
        &self,
        proposal_id: &str,
        now: u64,
        expected: RiskyAction,
    ) -> Result<(), ApprovalError> {
        let proposal = self.proposal(proposal_id)?;
        if proposal.action != expected {
            return Err(ApprovalError::ActionMismatch {
                expected,
                actual: proposal.action,
            });
        }
        self.assert_can_execute(proposal_id, now)
    }

    pub fn assert_can_execute_action_for_run(
        &self,
        proposal_id: &str,
        now: u64,
        expected: RiskyAction,
        run_id: &str,
    ) -> Result<(), ApprovalError> {
        let proposal = self.proposal(proposal_id)?;
        if proposal.run_id != run_id {
            return Err(ApprovalError::RunMismatch {
                expected: run_id.to_string(),
                actual: proposal.run_id.clone(),
            });
        }
        if proposal.action != expected {
            return Err(ApprovalError::ActionMismatch {
                expected,
                actual: proposal.action,
            });
        }
        self.assert_can_execute(proposal_id, now)
    }

    pub fn assert_can_execute_action_for_run_node(
        &self,
        proposal_id: &str,
        now: u64,
        expected: RiskyAction,
        run_id: &str,
        node_id: &str,
    ) -> Result<(), ApprovalError> {
        let proposal = self.proposal(proposal_id)?;
        if proposal.run_id != run_id {
            return Err(ApprovalError::RunMismatch {
                expected: run_id.to_string(),
                actual: proposal.run_id.clone(),
            });
        }
        if proposal.node_id != node_id {
            return Err(ApprovalError::NodeMismatch {
                expected: node_id.to_string(),
                actual: proposal.node_id.clone(),
            });
        }
        if proposal.action != expected {
            return Err(ApprovalError::ActionMismatch {
                expected,
                actual: proposal.action,
            });
        }
        self.assert_can_execute(proposal_id, now)
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

fn ensure_pending(proposal: &ActionProposal) -> Result<(), ApprovalError> {
    (proposal.status == ProposalStatus::Pending)
        .then_some(())
        .ok_or(ApprovalError::AlreadyDecided)
}

fn decision(kind: ApprovalDecisionKind, decided_at: u64, note: &str) -> ApprovalDecision {
    ApprovalDecision {
        kind,
        decided_at,
        note: note.to_string(),
    }
}
