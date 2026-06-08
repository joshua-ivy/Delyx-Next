#[cfg(test)]
mod tests {
    use crate::agent_executor::AgentExecutionStatus;
    use crate::agent_patch_restore_executor::execute_patch_restore_node;
    use crate::agent_run::{AgentRunLedger, AgentRunStatus};
    use crate::approval::{ApprovalEngine, ProposalInput, RiskLevel, RiskyAction};
    use crate::patch_apply_bridge::{apply_patch_record, PatchApplyRequest};
    use crate::patch_bridge::{
        propose_patch_record, PatchBridgeStore, PatchFileRequest, PatchProposalRequest,
    };
    use crate::patch_restore_bridge::PatchRestoreRequest;
    use std::fs;
    use std::path::{Path, PathBuf};
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn patch_restore_node_waits_without_writing() {
        let root = temp_workspace("executor-restore-wait");
        let file = root.join("settings.toml");
        fs::write(&file, "network = true\n").unwrap();
        let mut ledger = AgentRunLedger::new();
        let run = ledger.create_run("thread-1").unwrap();
        let (mut approvals, apply_id) = approved_file_write(&run.id);
        let restore_id = pending_file_write(&mut approvals, &run.id);
        let mut patches = applied_store(&run.id, &apply_id, &approvals, &root, &file);

        let result = execute_patch_restore_node(
            &mut ledger,
            &mut patches,
            &approvals,
            restore_request("executor-patch-1", &restore_id, &root, 3),
        )
        .unwrap();

        assert_eq!(result.status, AgentExecutionStatus::WaitingForApproval);
        assert_eq!(
            ledger.get_run(&run.id).unwrap().status,
            AgentRunStatus::WaitingForApproval
        );
        assert_eq!(patches.records[0].status, "applied");
        assert_eq!(fs::read_to_string(&file).unwrap(), "network = false\n");
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn approved_patch_restore_node_reverts_and_records_receipts() {
        let root = temp_workspace("executor-restore");
        let file = root.join("settings.toml");
        fs::write(&file, "network = true\n").unwrap();
        let mut ledger = AgentRunLedger::new();
        let run = ledger.create_run("thread-1").unwrap();
        let (mut approvals, apply_id) = approved_file_write(&run.id);
        let restore_id = approved_more_file_write(&mut approvals, &run.id);
        let mut patches = applied_store(&run.id, &apply_id, &approvals, &root, &file);

        let result = execute_patch_restore_node(
            &mut ledger,
            &mut patches,
            &approvals,
            restore_request("executor-patch-1", &restore_id, &root, 3),
        )
        .unwrap();
        let run = ledger.get_run(&run.id).unwrap();

        assert_eq!(result.status, AgentExecutionStatus::Completed);
        assert_eq!(run.artifacts[0].kind, "patch_restore");
        assert_eq!(
            run.evidence[0].title,
            "Restored patch proposal executor-patch-1"
        );
        assert_eq!(patches.records[0].status, "restored");
        assert_eq!(
            patches.records[0].restore_approval_id.as_deref(),
            Some(restore_id.as_str())
        );
        assert_eq!(fs::read_to_string(&file).unwrap(), "network = true\n");
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn stale_patch_restore_node_fails_without_overwriting() {
        let root = temp_workspace("executor-restore-stale");
        let file = root.join("settings.toml");
        fs::write(&file, "network = true\n").unwrap();
        let mut ledger = AgentRunLedger::new();
        let run = ledger.create_run("thread-1").unwrap();
        let (mut approvals, apply_id) = approved_file_write(&run.id);
        let restore_id = approved_more_file_write(&mut approvals, &run.id);
        let mut patches = applied_store(&run.id, &apply_id, &approvals, &root, &file);
        fs::write(&file, "network = maybe\n").unwrap();

        let result = execute_patch_restore_node(
            &mut ledger,
            &mut patches,
            &approvals,
            restore_request("executor-patch-1", &restore_id, &root, 3),
        )
        .unwrap();

        assert_eq!(result.status, AgentExecutionStatus::Failed);
        assert_eq!(
            ledger.get_run(&run.id).unwrap().status,
            AgentRunStatus::Failed
        );
        assert_eq!(patches.records[0].status, "applied");
        assert_eq!(fs::read_to_string(&file).unwrap(), "network = maybe\n");
        let _ = fs::remove_dir_all(root);
    }

    fn applied_store(
        run_id: &str,
        approval_id: &str,
        approvals: &ApprovalEngine,
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
        apply_patch_record(
            &mut store,
            approvals,
            PatchApplyRequest {
                approved_roots: vec![root.display().to_string()],
                created_at_ms: 2,
                proposal_id: "executor-patch-1".to_string(),
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

    fn approved_more_file_write(engine: &mut ApprovalEngine, run_id: &str) -> String {
        let proposal = engine.propose(proposal_input(run_id));
        engine.approve(&proposal.id, 1, "restore approval").unwrap();
        proposal.id
    }

    fn pending_file_write(engine: &mut ApprovalEngine, run_id: &str) -> String {
        engine.propose(proposal_input(run_id)).id
    }

    fn proposal_input(run_id: &str) -> ProposalInput {
        ProposalInput {
            action: RiskyAction::FileWrite,
            expected_result: "Restore a checkpointed patch.".to_string(),
            expires_at: 10_000,
            node_id: "agent-executor-patch-restore".to_string(),
            reason: "User requested rollback to the captured checkpoint.".to_string(),
            risk: RiskLevel::Medium,
            rollback_plan: "Restore command is itself the rollback action.".to_string(),
            run_id: run_id.to_string(),
            scope: "One checkpointed patch inside the approved root.".to_string(),
        }
    }

    fn restore_request(
        proposal_id: &str,
        approval_id: &str,
        root: &Path,
        created_at_ms: u64,
    ) -> PatchRestoreRequest {
        PatchRestoreRequest {
            approval_id: approval_id.to_string(),
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
