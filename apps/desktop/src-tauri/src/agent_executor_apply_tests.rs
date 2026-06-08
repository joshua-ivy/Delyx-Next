#[cfg(test)]
mod tests {
    use crate::agent_executor::{execute_patch_apply_node, AgentExecutionStatus};
    use crate::agent_run::{AgentRunLedger, AgentRunStatus};
    use crate::approval::{ApprovalEngine, ProposalInput, RiskLevel, RiskyAction};
    use crate::patch_apply_bridge::PatchApplyRequest;
    use crate::patch_bridge::{
        propose_patch_record, PatchBridgeStore, PatchFileRequest, PatchProposalRequest,
    };
    use std::fs;
    use std::path::{Path, PathBuf};
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn patch_apply_node_waits_without_writing() {
        let root = temp_workspace("executor-apply-wait");
        let file = root.join("settings.toml");
        fs::write(&file, "network = true\n").unwrap();
        let mut ledger = AgentRunLedger::new();
        let run = ledger.create_run("thread-1").unwrap();
        let (approvals, approval_id) = pending_file_write(&run.id);
        let mut patches = proposed_store(&run.id, &approval_id, &root, &file);

        let result = execute_patch_apply_node(
            &mut ledger,
            &mut patches,
            &approvals,
            apply_request("executor-patch-1", &root, 2),
        )
        .unwrap();

        assert_eq!(result.status, AgentExecutionStatus::WaitingForApproval);
        assert_eq!(
            ledger.get_run(&run.id).unwrap().status,
            AgentRunStatus::WaitingForApproval
        );
        assert_eq!(patches.records[0].status, "proposed");
        assert_eq!(fs::read_to_string(&file).unwrap(), "network = true\n");
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn approved_patch_apply_node_writes_and_records_receipts() {
        let root = temp_workspace("executor-apply");
        let file = root.join("settings.toml");
        fs::write(&file, "network = true\n").unwrap();
        let mut ledger = AgentRunLedger::new();
        let run = ledger.create_run("thread-1").unwrap();
        let (approvals, approval_id) = approved_file_write(&run.id);
        let mut patches = proposed_store(&run.id, &approval_id, &root, &file);

        let result = execute_patch_apply_node(
            &mut ledger,
            &mut patches,
            &approvals,
            apply_request("executor-patch-1", &root, 2),
        )
        .unwrap();
        let run = ledger.get_run(&run.id).unwrap();

        assert_eq!(result.status, AgentExecutionStatus::Completed);
        assert_eq!(run.nodes[0].kind, "tool_execution");
        assert_eq!(run.nodes[0].status, AgentRunStatus::Completed);
        assert_eq!(run.artifacts[0].kind, "patch_apply");
        assert_eq!(
            run.evidence[0].title,
            "Applied patch proposal executor-patch-1"
        );
        assert_eq!(patches.records[0].status, "applied");
        assert_eq!(fs::read_to_string(&file).unwrap(), "network = false\n");
        assert_eq!(
            patches.records[0].checkpoint_files[0].contents.as_deref(),
            Some("network = true\n")
        );
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn stale_patch_apply_node_fails_without_overwriting() {
        let root = temp_workspace("executor-apply-stale");
        let file = root.join("settings.toml");
        fs::write(&file, "network = true\n").unwrap();
        let mut ledger = AgentRunLedger::new();
        let run = ledger.create_run("thread-1").unwrap();
        let (approvals, approval_id) = approved_file_write(&run.id);
        let mut patches = proposed_store(&run.id, &approval_id, &root, &file);
        fs::write(&file, "network = maybe\n").unwrap();

        let result = execute_patch_apply_node(
            &mut ledger,
            &mut patches,
            &approvals,
            apply_request("executor-patch-1", &root, 2),
        )
        .unwrap();

        assert_eq!(result.status, AgentExecutionStatus::Failed);
        assert_eq!(
            ledger.get_run(&run.id).unwrap().status,
            AgentRunStatus::Failed
        );
        assert_eq!(patches.records[0].status, "proposed");
        assert_eq!(fs::read_to_string(&file).unwrap(), "network = maybe\n");
        let _ = fs::remove_dir_all(root);
    }

    fn proposed_store(
        run_id: &str,
        approval_id: &str,
        root: &Path,
        file: &Path,
    ) -> PatchBridgeStore {
        let mut store = PatchBridgeStore::default();
        propose_patch_record(
            &mut store,
            PatchProposalRequest {
                approval_id: approval_id.to_string(),
                approved_roots: vec![root.display().to_string()],
                client_id: "executor-patch-1".to_string(),
                files: vec![PatchFileRequest {
                    after: "network = false\n".to_string(),
                    path: file.display().to_string(),
                }],
                run_id: run_id.to_string(),
            },
        )
        .unwrap();
        store
    }

    fn approved_file_write(run_id: &str) -> (ApprovalEngine, String) {
        let mut engine = ApprovalEngine::new();
        let proposal = engine.propose(proposal_input(run_id));
        engine.approve(&proposal.id, 1, "test approval").unwrap();
        (engine, proposal.id)
    }

    fn pending_file_write(run_id: &str) -> (ApprovalEngine, String) {
        let mut engine = ApprovalEngine::new();
        let proposal = engine.propose(proposal_input(run_id));
        (engine, proposal.id)
    }

    fn proposal_input(run_id: &str) -> ProposalInput {
        ProposalInput {
            action: RiskyAction::FileWrite,
            expected_result: "Apply a proposed patch and capture a checkpoint.".to_string(),
            expires_at: 10_000,
            node_id: "agent-executor-patch-apply".to_string(),
            reason: "Approved plan is ready for patch application.".to_string(),
            risk: RiskLevel::Medium,
            rollback_plan: "Restore the checkpoint if review rejects the diff.".to_string(),
            run_id: run_id.to_string(),
            scope: "One patch proposal inside the approved root.".to_string(),
        }
    }

    fn apply_request(proposal_id: &str, root: &Path, created_at_ms: u64) -> PatchApplyRequest {
        PatchApplyRequest {
            approved_roots: vec![root.display().to_string()],
            created_at_ms,
            proposal_id: proposal_id.to_string(),
        }
    }

    fn temp_workspace(label: &str) -> PathBuf {
        let stamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let path = std::env::temp_dir().join(format!("delyx-next-{label}-{stamp}"));
        fs::create_dir_all(&path).unwrap();
        path
    }
}
