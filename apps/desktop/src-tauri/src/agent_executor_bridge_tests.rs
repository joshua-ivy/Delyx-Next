#[cfg(test)]
mod tests {
    use crate::agent_executor_bridge::{
        execute_patch_proposal_record, AgentPatchProposalExecuteRequest,
    };
    use crate::approval::{ApprovalEngine, ProposalInput, RiskLevel, RiskyAction};
    use crate::patch_bridge::{patch_snapshot_from_store, PatchBridgeStore, PatchFileRequest};
    use crate::thread_run_bridge::{
        create_thread_run_record, ThreadRunCreateRequest, ThreadRunStore,
    };
    use std::fs;
    use std::path::{Path, PathBuf};
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn bridge_executes_patch_proposal_against_thread_run_store() {
        let root = temp_workspace("executor-bridge");
        let file = root.join("settings.toml");
        fs::write(&file, "network = true\n").unwrap();
        let mut thread_store = ThreadRunStore::default();
        let record = create_thread_run_record(&mut thread_store, create_request()).unwrap();
        let mut patch_store = PatchBridgeStore::default();
        let (approvals, approval_id) = approved_file_write(&record.run.id);

        let view = execute_patch_proposal_record(
            &mut thread_store,
            &mut patch_store,
            &approvals,
            request(&record.run.id, &approval_id, &root, &file),
        )
        .unwrap();

        assert_eq!(view.status, "completed");
        assert_eq!(
            patch_snapshot_from_store(&patch_store, &record.run.id).len(),
            1
        );
        let run = thread_store.ledger.get_run(&record.run.id).unwrap();
        assert_eq!(run.artifacts[0].kind, "patch_proposal");
        let _ = fs::remove_dir_all(root);
    }

    fn create_request() -> ThreadRunCreateRequest {
        ThreadRunCreateRequest {
            created_at: "2026-06-08T00:00:00.000Z".to_string(),
            goal: "Patch through executor bridge".to_string(),
            project_id: "proj-1".to_string(),
        }
    }

    fn approved_file_write(run_id: &str) -> (ApprovalEngine, String) {
        let mut engine = ApprovalEngine::new();
        let proposal = engine.propose(ProposalInput {
            action: RiskyAction::FileWrite,
            expected_result: "Create a patch proposal.".to_string(),
            expires_at: 10_000,
            node_id: "agent-executor-bridge".to_string(),
            reason: "Approved build flow needs a patch proposal.".to_string(),
            risk: RiskLevel::Medium,
            rollback_plan: "Patch proposal does not write files.".to_string(),
            run_id: run_id.to_string(),
            scope: "One patch proposal inside the approved root.".to_string(),
        });
        engine.approve(&proposal.id, 1, "test approval").unwrap();
        (engine, proposal.id)
    }

    fn request(
        run_id: &str,
        approval_id: &str,
        root: &Path,
        file: &Path,
    ) -> AgentPatchProposalExecuteRequest {
        AgentPatchProposalExecuteRequest {
            approval_id: approval_id.to_string(),
            approved_roots: vec![root.display().to_string()],
            client_id: "executor-bridge-patch-1".to_string(),
            created_at_ms: 2,
            files: vec![PatchFileRequest {
                after: "network = false\n".to_string(),
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
