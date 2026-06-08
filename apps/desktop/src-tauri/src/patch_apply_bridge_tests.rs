#[cfg(test)]
mod tests {
    use crate::approval::{ApprovalEngine, ProposalInput, RiskLevel, RiskyAction};
    use crate::patch_apply_bridge::{apply_patch_record, PatchApplyRequest};
    use crate::patch_bridge::{
        propose_patch_record, PatchBridgeStore, PatchFileRequest, PatchProposalRequest,
    };
    use std::fs;
    use std::path::{Path, PathBuf};
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn approved_patch_apply_writes_file_and_records_checkpoint() {
        let root = temp_workspace("apply-approved");
        let file = root.join("settings.toml");
        fs::write(&file, "network = true\n").unwrap();
        let (approvals, approval_id) = approved_file_write("run-1");
        let mut store = proposed_store(&root, &file, &approval_id, "network = false\n");

        let applied = apply_patch_record(
            &mut store,
            &approvals,
            apply_request("patch-client-1", &approval_id, &root),
        )
        .unwrap();

        assert_eq!(fs::read_to_string(&file).unwrap(), "network = false\n");
        assert_eq!(applied.status, "applied");
        assert_eq!(
            applied.checkpoint_files[0].contents.as_deref(),
            Some("network = true\n")
        );
        assert_eq!(store.records[0].status, "applied");
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn approved_patch_apply_uses_separate_apply_approval() {
        let root = temp_workspace("apply-separate");
        let file = root.join("settings.toml");
        fs::write(&file, "network = true\n").unwrap();
        let mut approvals = ApprovalEngine::new();
        let proposal_id = approvals.propose(proposal_input("run-1")).id;
        let apply = approvals.propose(proposal_input("run-1"));
        approvals.approve(&apply.id, 1, "apply approval").unwrap();
        let mut store = proposed_store(&root, &file, &proposal_id, "network = false\n");

        let applied = apply_patch_record(
            &mut store,
            &approvals,
            apply_request("patch-client-1", &apply.id, &root),
        )
        .unwrap();

        assert_eq!(fs::read_to_string(&file).unwrap(), "network = false\n");
        assert_eq!(applied.status, "applied");
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn pending_patch_apply_approval_blocks_without_writing() {
        let root = temp_workspace("apply-pending");
        let file = root.join("settings.toml");
        fs::write(&file, "network = true\n").unwrap();
        let mut approvals = ApprovalEngine::new();
        let proposal_id = approvals.propose(proposal_input("run-1")).id;
        let apply_id = approvals.propose(proposal_input("run-1")).id;
        let mut store = proposed_store(&root, &file, &proposal_id, "network = false\n");

        let error = apply_patch_record(
            &mut store,
            &approvals,
            apply_request("patch-client-1", &apply_id, &root),
        )
        .unwrap_err();

        assert_eq!(error, "Patch apply approval blocked: NotApproved");
        assert_eq!(fs::read_to_string(&file).unwrap(), "network = true\n");
        assert_eq!(store.records[0].status, "proposed");
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn stale_patch_apply_blocks_without_overwriting_file() {
        let root = temp_workspace("apply-stale");
        let file = root.join("settings.toml");
        fs::write(&file, "network = true\n").unwrap();
        let (approvals, approval_id) = approved_file_write("run-1");
        let mut store = proposed_store(&root, &file, &approval_id, "network = false\n");
        fs::write(&file, "network = maybe\n").unwrap();

        let error = apply_patch_record(
            &mut store,
            &approvals,
            apply_request("patch-client-1", &approval_id, &root),
        )
        .unwrap_err();

        assert_eq!(
            error,
            "Patch apply blocked because a file changed since proposal."
        );
        assert_eq!(fs::read_to_string(&file).unwrap(), "network = maybe\n");
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn applied_patch_checkpoint_survives_sqlite_reload() {
        let root = temp_workspace("apply-persist");
        let db_path = root.join("delyx.sqlite3");
        let file = root.join("settings.toml");
        fs::write(&file, "network = true\n").unwrap();
        let (approvals, approval_id) = approved_file_write("run-1");
        let mut store = proposed_store(&root, &file, &approval_id, "network = false\n");

        apply_patch_record(
            &mut store,
            &approvals,
            apply_request("patch-client-1", &approval_id, &root),
        )
        .unwrap();
        crate::patch_persistence::save_to_path(&store, &db_path).unwrap();
        let reloaded = crate::patch_persistence::load_from_path(&db_path).unwrap();

        assert_eq!(reloaded.records[0].status, "applied");
        assert_eq!(reloaded.records[0].files[0].before, "network = true\n");
        assert_eq!(
            reloaded.records[0].checkpoint_files[0].contents.as_deref(),
            Some("network = true\n")
        );
        let _ = fs::remove_dir_all(root);
    }

    fn proposed_store(
        root: &Path,
        file: &Path,
        approval_id: &str,
        after: &str,
    ) -> PatchBridgeStore {
        let mut store = PatchBridgeStore::default();
        propose_patch_record(
            &mut store,
            PatchProposalRequest {
                approval_id: approval_id.to_string(),
                approved_roots: vec![root.display().to_string()],
                client_id: "patch-client-1".to_string(),
                files: vec![PatchFileRequest {
                    after: after.to_string(),
                    path: file.display().to_string(),
                }],
                run_id: "run-1".to_string(),
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

    fn proposal_input(run_id: &str) -> ProposalInput {
        ProposalInput {
            action: RiskyAction::FileWrite,
            expected_result: "Apply a proposed patch and capture a checkpoint.".to_string(),
            expires_at: 10_000,
            node_id: "patch-apply".to_string(),
            reason: "Approved plan requires a file patch.".to_string(),
            risk: RiskLevel::Medium,
            rollback_plan: "Restore the checkpoint if review rejects the diff.".to_string(),
            run_id: run_id.to_string(),
            scope: "One patch proposal inside the approved root.".to_string(),
        }
    }

    fn apply_request(proposal_id: &str, approval_id: &str, root: &Path) -> PatchApplyRequest {
        PatchApplyRequest {
            approval_id: approval_id.to_string(),
            approved_roots: vec![root.display().to_string()],
            created_at_ms: 2,
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
