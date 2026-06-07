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
    fn approved_action_must_match_execution_request() {
        let mut engine = ApprovalEngine::new();
        let proposal = engine.propose(input(RiskyAction::FileWrite, 100));
        engine.approve(&proposal.id, 50, "approved once").unwrap();

        let result = engine.assert_can_execute_action(&proposal.id, 60, RiskyAction::TerminalCommand);

        assert_eq!(
            result.unwrap_err(),
            ApprovalError::ActionMismatch {
                expected: RiskyAction::TerminalCommand,
                actual: RiskyAction::FileWrite,
            }
        );
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

    #[test]
    fn proposal_expires_at_deadline_boundary() {
        let mut engine = ApprovalEngine::new();
        let proposal = engine.propose(input(RiskyAction::FileWrite, 100));

        assert_eq!(engine.gate_state(&proposal.id, 100).unwrap(), ApprovalGateState::Blocked);
        assert_eq!(engine.approve(&proposal.id, 100, "deadline").unwrap_err(), ApprovalError::Expired);
        assert_eq!(engine.list_proposals("run-1")[0].status, ProposalStatus::Expired);
    }

    #[test]
    fn approved_proposal_expires_after_deadline() {
        let mut engine = ApprovalEngine::new();
        let proposal = engine.propose(input(RiskyAction::TerminalCommand, 100));

        engine.approve(&proposal.id, 90, "approved before deadline").unwrap();
        assert_eq!(engine.assert_can_execute(&proposal.id, 99), Ok(()));

        engine.expire_due(100);

        assert_eq!(engine.list_proposals("run-1")[0].status, ProposalStatus::Expired);
        assert_eq!(engine.assert_can_execute(&proposal.id, 100).unwrap_err(), ApprovalError::NotApproved);
    }

    #[test]
    fn action_taxonomy_clamps_downgraded_risk() {
        let mut engine = ApprovalEngine::new();
        let proposal = engine.propose(input_with_risk(RiskyAction::DependencyInstall, RiskLevel::Low, 100));

        assert_eq!(proposal.risk, RiskLevel::High);
    }

    #[test]
    fn action_taxonomy_allows_escalated_dangerous_risk() {
        let mut engine = ApprovalEngine::new();
        let proposal = engine.propose(input_with_risk(RiskyAction::TerminalCommand, RiskLevel::Dangerous, 100));

        assert_eq!(proposal.risk, RiskLevel::Dangerous);
    }

    #[test]
    fn risk_taxonomy_declares_minimums_for_risky_actions() {
        for (action, minimum, rollback_required) in [
            (RiskyAction::FileWrite, RiskLevel::High, true),
            (RiskyAction::TerminalCommand, RiskLevel::Medium, false),
            (RiskyAction::DependencyInstall, RiskLevel::High, true),
            (RiskyAction::ConnectorWrite, RiskLevel::High, true),
            (RiskyAction::DurableMemorySave, RiskLevel::Medium, true),
            (RiskyAction::ScheduledRiskyAction, RiskLevel::Dangerous, true),
            (RiskyAction::ExternalAgentExecution, RiskLevel::High, true),
            (RiskyAction::ExternalSend, RiskLevel::High, false),
        ] {
            let entry = action.taxonomy();
            assert_eq!(entry.action, action);
            assert_eq!(entry.minimum_risk, minimum);
            assert_eq!(entry.rollback_required, rollback_required);
            assert!(!entry.summary.is_empty());
        }
    }

    fn input(action: RiskyAction, expires_at: u64) -> ProposalInput {
        input_with_risk(action, RiskLevel::High, expires_at)
    }

    fn input_with_risk(action: RiskyAction, risk: RiskLevel, expires_at: u64) -> ProposalInput {
        ProposalInput {
            action,
            expected_result: "risky action would execute only after approval".to_string(),
            expires_at,
            node_id: "node-1".to_string(),
            reason: "required by requested local task".to_string(),
            risk,
            rollback_plan: "restore checkpoint or cancel action".to_string(),
            run_id: "run-1".to_string(),
            scope: "approved workspace file".to_string(),
        }
    }
}
