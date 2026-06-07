#[cfg(test)]
mod tests {
    use crate::approval::{
        ApprovalDecisionKind, ApprovalEngine, ApprovalError, ApprovalGateState, ProposalInput, ProposalStatus,
        RiskLevel, RiskyAction,
    };

    #[test]
    fn risky_action_creates_pending_approval_proposal() {
        let mut engine = ApprovalEngine::new();
        let proposal = engine.propose(input(RiskyAction::FileWrite, 100));

        assert_eq!(proposal.id, "prop-1");
        assert_eq!(proposal.status, ProposalStatus::Pending);
        assert_eq!(proposal.risk, RiskLevel::High);
        assert_eq!(proposal.scope, "approved workspace file");
        assert_eq!(proposal.node_id, "node-1");
        assert_eq!(engine.list_proposals("run-1").len(), 1);
    }

    #[test]
    fn pending_proposal_waits_and_cannot_execute() {
        let mut engine = ApprovalEngine::new();
        let proposal = engine.propose(input(RiskyAction::TerminalCommand, 100));

        assert_eq!(engine.gate_state(&proposal.id, 50).unwrap(), ApprovalGateState::WaitingForApproval);
        assert_eq!(engine.assert_can_execute(&proposal.id, 50).unwrap_err(), ApprovalError::NotApproved);
    }

    #[test]
    fn approval_changes_status_and_allows_execution() {
        let mut engine = ApprovalEngine::new();
        let proposal = engine.propose(input(RiskyAction::DependencyInstall, 100));

        engine.approve(&proposal.id, 50, "approved once").unwrap();

        let proposal = engine.list_proposals("run-1")[0];
        assert_eq!(proposal.status, ProposalStatus::Approved);
        assert_eq!(proposal.decision.as_ref().unwrap().kind, ApprovalDecisionKind::Approve);
        assert_eq!(engine.assert_can_execute(&proposal.id, 60), Ok(()));
    }

    #[test]
    fn denial_blocks_execution() {
        let mut engine = ApprovalEngine::new();
        let proposal = engine.propose(input(RiskyAction::ExternalAgentExecution, 100));

        engine.deny(&proposal.id, 40, "scope too broad").unwrap();

        assert_eq!(engine.gate_state(&proposal.id, 50).unwrap(), ApprovalGateState::Blocked);
        assert_eq!(engine.assert_can_execute(&proposal.id, 50).unwrap_err(), ApprovalError::NotApproved);
    }

    #[test]
    fn expired_proposal_blocks_execution() {
        let mut engine = ApprovalEngine::new();
        let proposal = engine.propose(input(RiskyAction::DurableMemorySave, 100));

        engine.expire_due(101);

        assert_eq!(engine.list_proposals("run-1")[0].status, ProposalStatus::Expired);
        assert_eq!(engine.gate_state(&proposal.id, 101).unwrap(), ApprovalGateState::Blocked);
    }

    #[test]
    fn expired_proposal_cannot_be_approved_late() {
        let mut engine = ApprovalEngine::new();
        let proposal = engine.propose(input(RiskyAction::ConnectorWrite, 100));

        assert_eq!(engine.approve(&proposal.id, 101, "late").unwrap_err(), ApprovalError::Expired);
        assert_eq!(engine.list_proposals("run-1")[0].status, ProposalStatus::Expired);
    }

    fn input(action: RiskyAction, expires_at: u64) -> ProposalInput {
        ProposalInput {
            action,
            expected_result: "risky action would execute only after approval".to_string(),
            expires_at,
            node_id: "node-1".to_string(),
            reason: "required by requested local task".to_string(),
            risk: RiskLevel::High,
            rollback_plan: "restore checkpoint or cancel action".to_string(),
            run_id: "run-1".to_string(),
            scope: "approved workspace file".to_string(),
        }
    }
}
