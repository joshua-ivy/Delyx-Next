#[cfg(test)]
mod tests {
    use crate::approval::{ApprovalEngine, ApprovalError, ProposalInput, RiskLevel, RiskyAction};
    use crate::automation::{
        ActiveHours, AutomationEngine, MissionContractInput, MissionStatus, ScheduledRunStatus,
    };

    #[test]
    fn recurring_work_starts_paused_until_approved() {
        let mut engine = AutomationEngine::new();

        let contract = engine.create_contract(contract_input(vec!["read".to_string()]));

        assert_eq!(contract.status, MissionStatus::Paused);
    }

    #[test]
    fn contract_shows_scope_schedule_targets_and_stop_condition() {
        let mut engine = AutomationEngine::new();

        let contract = engine.create_contract(contract_input(vec!["read".to_string()]));

        assert_eq!(contract.scope, "C:/workspace");
        assert_eq!(contract.active_hours.start_hour, 8);
        assert_eq!(contract.timezone, "America/Chicago");
        assert_eq!(contract.delivery_targets, vec!["desktop_notification"]);
        assert_eq!(contract.stop_condition, "Stop after one failed run.");
    }

    #[test]
    fn workspace_drift_blocks_scheduled_work() {
        let mut engine = AutomationEngine::new();
        let mut approvals = ApprovalEngine::new();
        let contract = engine.create_contract(contract_input(vec!["read".to_string()]));
        let approval = approvals.propose(approval_input(&contract.id));
        approvals
            .approve(&approval.id, 10, "approved in test")
            .unwrap();
        engine
            .approve_contract(&contract.id, &approval.id, 10, &approvals)
            .unwrap();

        let run = engine
            .schedule_due_run(&contract.id, "changed", 10, &mut approvals)
            .unwrap();

        assert_eq!(run.status, ScheduledRunStatus::Blocked);
        assert!(run.reason.contains("Workspace drift"));
    }

    #[test]
    fn contract_activation_requires_scheduled_action_approval() {
        let mut engine = AutomationEngine::new();
        let mut approvals = ApprovalEngine::new();
        let contract = engine.create_contract(contract_input(vec!["read".to_string()]));
        let approval = approvals.propose(ProposalInput {
            action: RiskyAction::FileWrite,
            ..approval_input(&contract.id)
        });
        approvals
            .approve(&approval.id, 10, "approved in test")
            .unwrap();

        let result = engine.approve_contract(&contract.id, &approval.id, 10, &approvals);

        assert_eq!(
            result.unwrap_err(),
            crate::automation::AutomationError::Approval(ApprovalError::ActionMismatch {
                expected: RiskyAction::ScheduledRiskyAction,
                actual: RiskyAction::FileWrite,
            })
        );
        assert_eq!(engine.contracts()[0].status, MissionStatus::Paused);
    }

    #[test]
    fn contract_activation_requires_matching_contract_approval() {
        let mut engine = AutomationEngine::new();
        let mut approvals = ApprovalEngine::new();
        let first = engine.create_contract(contract_input(vec!["read".to_string()]));
        let second = engine.create_contract(contract_input(vec!["read".to_string()]));
        let approval = approvals.propose(approval_input(&first.id));
        approvals
            .approve(&approval.id, 10, "approved in test")
            .unwrap();

        let result = engine.approve_contract(&second.id, &approval.id, 10, &approvals);

        assert_eq!(
            result.unwrap_err(),
            crate::automation::AutomationError::Approval(ApprovalError::RunMismatch {
                expected: second.id,
                actual: first.id,
            })
        );
        assert_eq!(engine.contracts()[1].status, MissionStatus::Paused);
    }

    #[test]
    fn risky_scheduled_action_creates_approval_instead_of_executing() {
        let mut engine = AutomationEngine::new();
        let mut approvals = ApprovalEngine::new();
        let contract = engine.create_contract(contract_input(vec!["terminal_command".to_string()]));
        let approval = approvals.propose(approval_input(&contract.id));
        approvals
            .approve(&approval.id, 10, "approved in test")
            .unwrap();
        engine
            .approve_contract(&contract.id, &approval.id, 10, &approvals)
            .unwrap();

        let run = engine
            .schedule_due_run(&contract.id, "fingerprint-1", 10, &mut approvals)
            .unwrap();

        assert_eq!(run.status, ScheduledRunStatus::WaitingForApproval);
        assert!(run.approval_id.is_some());
        assert_eq!(approvals.list_proposals(&contract.id).len(), 2);
        assert!(approvals
            .list_proposals(&contract.id)
            .iter()
            .any(|proposal| Some(proposal.id.as_str()) == run.approval_id.as_deref()));
    }

    #[test]
    fn low_risk_scheduled_work_can_create_run_after_approval() {
        let mut engine = AutomationEngine::new();
        let mut approvals = ApprovalEngine::new();
        let contract = engine.create_contract(contract_input(vec!["read".to_string()]));
        let approval = approvals.propose(approval_input(&contract.id));
        approvals
            .approve(&approval.id, 10, "approved in test")
            .unwrap();
        engine
            .approve_contract(&contract.id, &approval.id, 10, &approvals)
            .unwrap();

        let run = engine
            .schedule_due_run(&contract.id, "fingerprint-1", 10, &mut approvals)
            .unwrap();

        assert_eq!(run.status, ScheduledRunStatus::Created);
    }

    fn contract_input(allowed_tools: Vec<String>) -> MissionContractInput {
        MissionContractInput {
            active_hours: ActiveHours {
                start_hour: 8,
                end_hour: 18,
            },
            allowed_tools,
            delivery_targets: vec!["desktop_notification".to_string()],
            scope: "C:/workspace".to_string(),
            stop_condition: "Stop after one failed run.".to_string(),
            timezone: "America/Chicago".to_string(),
            title: "Morning repo health".to_string(),
            workspace_fingerprint: "fingerprint-1".to_string(),
        }
    }

    fn approval_input(run_id: &str) -> ProposalInput {
        ProposalInput {
            action: RiskyAction::ScheduledRiskyAction,
            expires_at: 30,
            expected_result: "Allow mission contract activation.".to_string(),
            node_id: "node-automation".to_string(),
            reason: "Deterministic automation test.".to_string(),
            risk: RiskLevel::Medium,
            rollback_plan: "Pause the mission contract.".to_string(),
            run_id: run_id.to_string(),
            scope: "Activate one mission contract.".to_string(),
        }
    }
}
