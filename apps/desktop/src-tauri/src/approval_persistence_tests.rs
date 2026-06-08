#[cfg(test)]
mod tests {
    use crate::approval::RiskyAction;
    use crate::approval_bridge::{
        approval_snapshot_from_store, decide_approval_record, propose_approval_record,
        ApprovalBridgeStore, ApprovalDecisionRequest, ApprovalProposalRequest, PermissionScopeView,
    };
    use crate::approval_persistence::{load_from_path, save_to_path};
    use std::fs;
    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn approval_bridge_state_survives_sqlite_reload() {
        let path = temp_path("approval");
        let mut store = ApprovalBridgeStore::default();
        let proposal = propose_approval_record(&mut store, request("approval-plan-build")).unwrap();
        decide_approval_record(&mut store, decision(&proposal.id, "approved", 50)).unwrap();

        save_to_path(&store, &path).unwrap();
        let bytes = fs::read(&path).unwrap();
        assert!(bytes.starts_with(b"SQLite format 3"));

        let mut loaded = load_from_path(&path).unwrap();
        let snapshot = approval_snapshot_from_store(&loaded, "run-1");
        assert_eq!(snapshot[0].id, proposal.id);
        assert_eq!(snapshot[0].status, "approved");
        assert_eq!(
            snapshot[0].scope.paths,
            Some(vec!["src/main.ts".to_string()])
        );
        loaded
            .engine
            .assert_can_execute_action_for_run(&proposal.id, 50, RiskyAction::FileWrite, "run-1")
            .unwrap();

        let next = propose_approval_record(&mut loaded, request("approval-second")).unwrap();
        assert_eq!(next.id, "prop-2");
        let _ = fs::remove_file(path);
    }

    fn request(client_id: &str) -> ApprovalProposalRequest {
        ApprovalProposalRequest {
            action_type: "edit_file".to_string(),
            client_id: client_id.to_string(),
            expected_result: "Allow Delyx to propose a patch without applying it.".to_string(),
            expires_at: "2026-06-07T00:30:00.000Z".to_string(),
            expires_at_ms: 100,
            node_id: "node-1".to_string(),
            rationale: "Approved plan requires a patch proposal.".to_string(),
            required_permission: "edit_file".to_string(),
            risk_label: "low".to_string(),
            rollback_plan: Some("Restore checkpoint before applying.".to_string()),
            run_id: "run-1".to_string(),
            scope: PermissionScopeView {
                commands: None,
                connector_id: None,
                kind: "file".to_string(),
                paths: Some(vec!["src/main.ts".to_string()]),
                project_id: Some("project-1".to_string()),
                root: Some("C:/work/project".to_string()),
                summary: "Files likely involved in the approved plan".to_string(),
            },
        }
    }

    fn decision(proposal_id: &str, decision: &str, decided_at_ms: u64) -> ApprovalDecisionRequest {
        ApprovalDecisionRequest {
            decided_at_ms,
            decision: decision.to_string(),
            note: Some("user decision".to_string()),
            proposal_id: proposal_id.to_string(),
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
