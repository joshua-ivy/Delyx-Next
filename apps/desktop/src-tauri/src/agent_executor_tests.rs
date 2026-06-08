#[cfg(test)]
mod tests {
    use crate::agent_executor::{execute_patch_proposal_node, AgentExecutionStatus};
    use crate::agent_run::{AgentRunLedger, AgentRunStatus};
    use crate::approval::{ApprovalEngine, ProposalInput, RiskLevel, RiskyAction};
    use crate::patch_bridge::{
        patch_snapshot_from_store, PatchBridgeStore, PatchFileRequest, PatchProposalRequest,
    };
    use std::fs;
    use std::path::{Path, PathBuf};
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn patch_proposal_node_waits_without_approval_or_file_write() {
        let root = temp_workspace("executor-wait");
        let file = root.join("settings.toml");
        fs::write(&file, "network = true\n").unwrap();
        let mut ledger = AgentRunLedger::new();
        let run = ledger.create_run("thread-1").unwrap();
        let mut patches = PatchBridgeStore::default();
        let (approvals, approval_id) = pending_file_write(&run.id);

        let result = execute_patch_proposal_node(
            &mut ledger,
            &mut patches,
            &approvals,
            request(&run.id, &approval_id, &root, &file, "network = false\n"),
            2,
        )
        .unwrap();

        assert_eq!(result.status, AgentExecutionStatus::WaitingForApproval);
        assert_eq!(
            ledger.get_run(&run.id).unwrap().status,
            AgentRunStatus::WaitingForApproval
        );
        assert!(patch_snapshot_from_store(&patches, &run.id).is_empty());
        assert_eq!(fs::read_to_string(&file).unwrap(), "network = true\n");
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn approved_patch_proposal_node_records_diff_receipts_without_writing() {
        let root = temp_workspace("executor-propose");
        let file = root.join("settings.toml");
        fs::write(&file, "network = true\n").unwrap();
        let mut ledger = AgentRunLedger::new();
        let run = ledger.create_run("thread-1").unwrap();
        let mut patches = PatchBridgeStore::default();
        let (approvals, approval_id) = approved_file_write(&run.id);

        let result = execute_patch_proposal_node(
            &mut ledger,
            &mut patches,
            &approvals,
            request(&run.id, &approval_id, &root, &file, "network = false\n"),
            2,
        )
        .unwrap();
        let run = ledger.get_run(&run.id).unwrap();

        assert_eq!(result.status, AgentExecutionStatus::Completed);
        assert_eq!(run.nodes[0].kind, "patch_proposal");
        assert_eq!(run.nodes[0].status, AgentRunStatus::Completed);
        assert_eq!(run.artifacts[0].kind, "patch_proposal");
        assert_eq!(run.evidence[0].source_kind, "diff");
        assert_eq!(patch_snapshot_from_store(&patches, &run.id).len(), 1);
        assert_eq!(fs::read_to_string(&file).unwrap(), "network = true\n");
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn patch_proposal_executor_can_resume_after_approval() {
        let root = temp_workspace("executor-resume");
        let file = root.join("settings.toml");
        fs::write(&file, "network = true\n").unwrap();
        let mut ledger = AgentRunLedger::new();
        let run = ledger.create_run("thread-1").unwrap();
        let mut patches = PatchBridgeStore::default();
        let mut approvals = ApprovalEngine::new();
        let proposal = approvals.propose(proposal_input(&run.id));

        execute_patch_proposal_node(
            &mut ledger,
            &mut patches,
            &approvals,
            request(&run.id, &proposal.id, &root, &file, "network = false\n"),
            2,
        )
        .unwrap();
        approvals
            .approve(&proposal.id, 3, "approved after wait")
            .unwrap();
        ledger.resume_after_approval(&run.id, &proposal.id).unwrap();
        let result = execute_patch_proposal_node(
            &mut ledger,
            &mut patches,
            &approvals,
            request(&run.id, &proposal.id, &root, &file, "network = false\n"),
            4,
        )
        .unwrap();

        assert_eq!(result.status, AgentExecutionStatus::Completed);
        assert_eq!(patch_snapshot_from_store(&patches, &run.id).len(), 1);
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn patch_proposal_executor_fails_run_on_invalid_scope() {
        let root = temp_workspace("executor-root");
        let outside = temp_workspace("executor-outside").join("escape.txt");
        let mut ledger = AgentRunLedger::new();
        let run = ledger.create_run("thread-1").unwrap();
        let mut patches = PatchBridgeStore::default();
        let (approvals, approval_id) = approved_file_write(&run.id);

        let result = execute_patch_proposal_node(
            &mut ledger,
            &mut patches,
            &approvals,
            request(&run.id, &approval_id, &root, &outside, "nope\n"),
            2,
        )
        .unwrap();

        assert_eq!(result.status, AgentExecutionStatus::Failed);
        assert_eq!(
            ledger.get_run(&run.id).unwrap().status,
            AgentRunStatus::Failed
        );
        assert!(patch_snapshot_from_store(&patches, &run.id).is_empty());
        let _ = fs::remove_dir_all(root);
        let _ = fs::remove_dir_all(outside.parent().unwrap());
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
            expected_result: "Create a patch proposal for review.".to_string(),
            expires_at: 10_000,
            node_id: "agent-executor-patch-proposal".to_string(),
            reason: "Approved plan is ready for a patch proposal.".to_string(),
            risk: RiskLevel::Medium,
            rollback_plan: "Patch proposal does not write files.".to_string(),
            run_id: run_id.to_string(),
            scope: "One patch proposal inside the approved root.".to_string(),
        }
    }

    fn request(
        run_id: &str,
        approval_id: &str,
        root: &Path,
        file: &Path,
        after: &str,
    ) -> PatchProposalRequest {
        PatchProposalRequest {
            approval_id: approval_id.to_string(),
            approved_roots: vec![root.display().to_string()],
            client_id: "executor-patch-1".to_string(),
            files: vec![PatchFileRequest {
                after: after.to_string(),
                path: file.display().to_string(),
            }],
            run_id: run_id.to_string(),
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
