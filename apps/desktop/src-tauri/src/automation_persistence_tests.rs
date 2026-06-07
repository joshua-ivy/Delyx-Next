#[cfg(test)]
mod tests {
    use crate::approval::{ApprovalEngine, ProposalInput, RiskLevel, RiskyAction};
    use crate::automation::{ActiveHours, AutomationEngine, MissionContractInput, MissionStatus, ScheduledRunStatus};
    use crate::automation_persistence::{load_from_path, save_to_path};
    use std::fs;
    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn automation_engine_survives_sqlite_reload() {
        let path = temp_path("automation");
        let mut engine = AutomationEngine::new();
        let mut approvals = ApprovalEngine::new();
        let contract = engine.create_contract(contract_input(vec!["terminal_command".to_string()]));
        let approval = approvals.propose(approval_input(&contract.id));
        approvals.approve(&approval.id, 10, "approved in test").unwrap();
        engine.approve_contract(&contract.id, &approval.id, 10, &approvals).unwrap();
        let waiting = engine.schedule_due_run(&contract.id, "fingerprint-1", 10, &mut approvals).unwrap();
        let blocked = engine.schedule_due_run(&contract.id, "changed", 11, &mut approvals).unwrap();

        save_to_path(&engine, &path).unwrap();
        let bytes = fs::read(&path).unwrap();
        assert!(bytes.starts_with(b"SQLite format 3"));

        let mut loaded = load_from_path(&path).unwrap();
        assert_eq!(loaded.contracts()[0].status, MissionStatus::Active);
        assert_eq!(loaded.contracts()[0].allowed_tools, vec!["terminal_command"]);
        assert_eq!(loaded.scheduled_runs()[0].id, waiting.id);
        assert_eq!(loaded.scheduled_runs()[0].status, ScheduledRunStatus::WaitingForApproval);
        assert!(loaded.scheduled_runs()[0].approval_id.is_some());
        assert_eq!(loaded.scheduled_runs()[1].id, blocked.id);
        assert_eq!(loaded.scheduled_runs()[1].status, ScheduledRunStatus::Blocked);

        let next_contract = loaded.create_contract(contract_input(vec!["read".to_string()]));
        assert_eq!(next_contract.id, "mission-2");
        let next_run = loaded.schedule_due_run(&contract.id, "changed", 12, &mut approvals).unwrap();
        assert_eq!(next_run.id, "scheduled-run-3");
        let _ = fs::remove_file(path);
    }

    fn contract_input(allowed_tools: Vec<String>) -> MissionContractInput {
        MissionContractInput {
            active_hours: ActiveHours { start_hour: 8, end_hour: 18 },
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
            reason: "Deterministic automation persistence test.".to_string(),
            risk: RiskLevel::Medium,
            rollback_plan: "Pause the mission contract.".to_string(),
            run_id: run_id.to_string(),
            scope: "Activate one mission contract.".to_string(),
        }
    }

    fn temp_path(name: &str) -> PathBuf {
        let stamp = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos();
        std::env::temp_dir().join(format!("delyx-next-{name}-{stamp}.sqlite3"))
    }
}
