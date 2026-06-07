#[cfg(test)]
mod tests {
    use crate::approval_bridge::{
        approval_snapshot_from_store, decide_approval_record, propose_approval_record,
        ApprovalBridgeStore, ApprovalDecisionRequest, ApprovalProposalRequest, PermissionScopeView,
    };

    #[test]
    fn proposal_request_returns_ui_ready_approval_from_rust_gate() {
        let mut store = ApprovalBridgeStore::default();

        let proposal = propose_approval_record(&mut store, request("approval-plan-build")).unwrap();
        let snapshot = approval_snapshot_from_store(&store, "run-1");

        assert_eq!(proposal.id, "prop-1");
        assert_eq!(proposal.action_type, "edit_file");
        assert_eq!(proposal.risk_label, "high");
        assert_eq!(proposal.status, "pending");
        assert_eq!(proposal.scope.paths, Some(vec!["src/main.ts".to_string()]));
        assert_eq!(snapshot, vec![proposal]);
    }

    #[test]
    fn duplicate_client_proposal_returns_existing_record() {
        let mut store = ApprovalBridgeStore::default();

        let first = propose_approval_record(&mut store, request("approval-plan-build")).unwrap();
        let second = propose_approval_record(&mut store, request("approval-plan-build")).unwrap();
        let snapshot = approval_snapshot_from_store(&store, "run-1");

        assert_eq!(second.id, first.id);
        assert_eq!(snapshot.len(), 1);
    }

    #[test]
    fn approval_decision_updates_rust_gate_status() {
        let mut store = ApprovalBridgeStore::default();
        let proposal = propose_approval_record(&mut store, request("approval-plan-build")).unwrap();

        let decided = decide_approval_record(&mut store, decision(&proposal.id, "approved", 50)).unwrap();

        assert_eq!(decided.status, "approved");
        assert_eq!(decided.id, proposal.id);
    }

    #[test]
    fn expired_approval_decision_returns_visible_expired_state() {
        let mut store = ApprovalBridgeStore::default();
        let proposal = propose_approval_record(&mut store, request("approval-plan-build")).unwrap();

        let decided = decide_approval_record(&mut store, decision(&proposal.id, "approved", 100)).unwrap();

        assert_eq!(decided.status, "expired");
    }

    #[test]
    fn read_file_is_rejected_as_non_risky_bridge_action() {
        let mut store = ApprovalBridgeStore::default();
        let mut request = request("read-only");
        request.action_type = "read_file".to_string();

        let result = propose_approval_record(&mut store, request);

        assert!(result.is_err());
        assert!(approval_snapshot_from_store(&store, "run-1").is_empty());
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
}
