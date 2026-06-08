#[cfg(test)]
mod tests {
    use crate::approval::{ApprovalEngine, ProposalInput, RiskLevel, RiskyAction};
    use crate::approval_bridge::{approval_snapshot_from_store, ApprovalBridgeStore};
    use crate::automation::AutomationEngine;
    use crate::automation_bridge::{
        approve_contract_record, automation_snapshot_from_path, create_contract_record,
        pause_contract_record, schedule_due_run_record, MissionActionRequest,
        MissionApproveRequest, MissionContractRequest, ScheduledRunRequest,
    };
    use std::fs;
    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn automation_bridge_creates_approves_pauses_and_survives_reload() {
        let path = temp_path("automation-bridge-contract");
        let mut engine = AutomationEngine::new();
        let created = create_contract_record(&mut engine, contract_request(vec!["read"])).unwrap();
        let contract_id = created.contracts[0].id.clone();
        let mut approvals = ApprovalEngine::new();
        let approval = approvals.propose(approval_input(&contract_id));
        approvals
            .approve(&approval.id, 10, "approved in bridge test")
            .unwrap();

        let active = approve_contract_record(
            &mut engine,
            &approvals,
            MissionApproveRequest {
                approval_id: approval.id,
                approved_at_ms: 10,
                contract_id: contract_id.clone(),
            },
        )
        .unwrap();
        let paused =
            pause_contract_record(&mut engine, MissionActionRequest { contract_id }).unwrap();

        assert_eq!(active.contracts[0].status, "active");
        assert_eq!(paused.contracts[0].status, "paused");
        crate::automation_persistence::save_to_path(&engine, &path).unwrap();
        let reloaded = automation_snapshot_from_path(&path).unwrap();
        assert_eq!(reloaded.contracts[0].status, "paused");
        assert_eq!(reloaded.contracts[0].active_hours, "08:00-18:00");
        let _ = fs::remove_file(path);
    }

    #[test]
    fn automation_bridge_rejects_unapproved_activation() {
        let mut engine = AutomationEngine::new();
        let created = create_contract_record(&mut engine, contract_request(vec!["read"])).unwrap();
        let approvals = ApprovalEngine::new();

        let result = approve_contract_record(
            &mut engine,
            &approvals,
            MissionApproveRequest {
                approval_id: "approval-1".to_string(),
                approved_at_ms: 10,
                contract_id: created.contracts[0].id.clone(),
            },
        );

        assert!(result.unwrap_err().contains("ProposalNotFound"));
        assert_eq!(
            engine.contracts()[0].status,
            crate::automation::MissionStatus::Paused
        );
    }

    #[test]
    fn automation_bridge_validates_contract_shape() {
        let mut engine = AutomationEngine::new();

        let result = create_contract_record(
            &mut engine,
            MissionContractRequest {
                active_end_hour: 8,
                active_start_hour: 18,
                allowed_tools: Vec::new(),
                delivery_targets: Vec::new(),
                scope: String::new(),
                stop_condition: String::new(),
                timezone: String::new(),
                title: String::new(),
                workspace_fingerprint: String::new(),
            },
        );

        assert_eq!(
            result.unwrap_err(),
            "Mission contract requires title, scope, timezone, stop condition, and workspace fingerprint."
        );
        assert!(engine.contracts().is_empty());
    }

    #[test]
    fn automation_bridge_scheduled_run_persists_generated_approval_card() {
        let path = temp_path("automation-bridge-schedule");
        let mut engine = AutomationEngine::new();
        let mut approvals = ApprovalBridgeStore::default();
        let created =
            create_contract_record(&mut engine, contract_request(vec!["terminal_command"]))
                .unwrap();
        let contract_id = created.contracts[0].id.clone();
        let activation = approvals.engine.propose(approval_input(&contract_id));
        approvals
            .engine
            .approve(&activation.id, 10, "approved in bridge test")
            .unwrap();
        approve_contract_record(
            &mut engine,
            &approvals.engine,
            MissionApproveRequest {
                approval_id: activation.id,
                approved_at_ms: 10,
                contract_id: contract_id.clone(),
            },
        )
        .unwrap();

        let scheduled = schedule_due_run_record(
            &mut engine,
            &mut approvals,
            ScheduledRunRequest {
                contract_id: contract_id.clone(),
                requested_at_ms: 11,
                workspace_fingerprint: "fingerprint-1".to_string(),
            },
        )
        .unwrap();

        assert_eq!(scheduled.scheduled_runs[0].status, "waiting_for_approval");
        let approval_snapshot = approval_snapshot_from_store(&approvals, &contract_id);
        assert_eq!(approval_snapshot.len(), 1);
        assert_eq!(approval_snapshot[0].action_type, "schedule_work");
        assert_eq!(
            approval_snapshot[0].scope.commands,
            Some(vec!["terminal_command".to_string()])
        );
        crate::automation_persistence::save_to_path(&engine, &path).unwrap();
        crate::approval_persistence::save_to_path(&approvals, &path).unwrap();
        let reloaded_automation = automation_snapshot_from_path(&path).unwrap();
        let reloaded_approvals = crate::approval_persistence::load_from_path(&path).unwrap();
        assert_eq!(
            reloaded_automation.scheduled_runs[0].status,
            "waiting_for_approval"
        );
        assert_eq!(
            approval_snapshot_from_store(&reloaded_approvals, &contract_id)[0].status,
            "pending"
        );
        let _ = fs::remove_file(path);
    }

    fn contract_request(allowed_tools: Vec<&str>) -> MissionContractRequest {
        MissionContractRequest {
            active_end_hour: 18,
            active_start_hour: 8,
            allowed_tools: allowed_tools.into_iter().map(str::to_string).collect(),
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
            reason: "Deterministic automation bridge test.".to_string(),
            risk: RiskLevel::Medium,
            rollback_plan: "Pause the mission contract.".to_string(),
            run_id: run_id.to_string(),
            scope: "Activate one mission contract.".to_string(),
        }
    }

    fn temp_path(name: &str) -> PathBuf {
        let stamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        std::env::temp_dir().join(format!("delyx-next-{name}-{stamp}.sqlite3"))
    }
}
