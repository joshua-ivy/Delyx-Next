#[cfg(test)]
mod tests {
    use crate::approval::{ApprovalEngine, ProposalInput, RiskLevel, RiskyAction};
    use crate::patch_apply_bridge::{apply_patch_record, PatchApplyRequest};
    use crate::patch_bridge::{
        propose_patch_record, PatchBridgeStore, PatchFileRequest, PatchProposalRequest,
    };
    use crate::patch_restore_bridge::{restore_patch_record, PatchRestoreRequest};
    use std::fs;
    use std::path::{Path, PathBuf};
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn approved_patch_restore_reverts_file_and_persists_receipt() {
        let root = temp_workspace("restore-approved");
        let db_path = root.join("delyx.sqlite3");
        let file = root.join("settings.toml");
        fs::write(&file, "network = true\n").unwrap();
        let (mut approvals, apply_id) = approved_file_write("run-1");
        let restore_id = approve_another_file_write(&mut approvals, "run-1");
        let mut store = applied_store(&root, &file, &apply_id, &approvals, "network = false\n");

        let restored = restore_patch_record(
            &mut store,
            &approvals,
            restore_request("patch-client-1", &restore_id, &root),
        )
        .unwrap();
        crate::patch_persistence::save_to_path(&store, &db_path).unwrap();
        let reloaded = crate::patch_persistence::load_from_path(&db_path).unwrap();

        assert_eq!(fs::read_to_string(&file).unwrap(), "network = true\n");
        assert_eq!(restored.status, "restored");
        assert_eq!(
            restored.restore_approval_id.as_deref(),
            Some(restore_id.as_str())
        );
        assert_eq!(reloaded.records[0].status, "restored");
        assert_eq!(
            reloaded.records[0].restore_approval_id.as_deref(),
            Some(restore_id.as_str())
        );
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn pending_patch_restore_approval_blocks_without_writing() {
        let root = temp_workspace("restore-pending");
        let file = root.join("settings.toml");
        fs::write(&file, "network = true\n").unwrap();
        let (mut approvals, apply_id) = approved_file_write("run-1");
        let pending_restore_id = pending_another_file_write(&mut approvals, "run-1");
        let mut store = applied_store(&root, &file, &apply_id, &approvals, "network = false\n");

        let error = restore_patch_record(
            &mut store,
            &approvals,
            restore_request("patch-client-1", &pending_restore_id, &root),
        )
        .unwrap_err();

        assert_eq!(error, "Patch restore approval blocked: NotApproved");
        assert_eq!(fs::read_to_string(&file).unwrap(), "network = false\n");
        assert_eq!(store.records[0].status, "applied");
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn stale_patch_restore_blocks_without_overwriting_file() {
        let root = temp_workspace("restore-stale");
        let file = root.join("settings.toml");
        fs::write(&file, "network = true\n").unwrap();
        let (mut approvals, apply_id) = approved_file_write("run-1");
        let restore_id = approve_another_file_write(&mut approvals, "run-1");
        let mut store = applied_store(&root, &file, &apply_id, &approvals, "network = false\n");
        fs::write(&file, "network = maybe\n").unwrap();

        let error = restore_patch_record(
            &mut store,
            &approvals,
            restore_request("patch-client-1", &restore_id, &root),
        )
        .unwrap_err();

        assert_eq!(
            error,
            "Patch restore blocked because a file changed since apply."
        );
        assert_eq!(fs::read_to_string(&file).unwrap(), "network = maybe\n");
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn patch_restore_removes_file_created_by_patch() {
        let root = temp_workspace("restore-new-file");
        let file = root.join("new.txt");
        let (mut approvals, apply_id) = approved_file_write("run-1");
        let restore_id = approve_another_file_write(&mut approvals, "run-1");
        let mut store = applied_store(&root, &file, &apply_id, &approvals, "created\n");

        restore_patch_record(
            &mut store,
            &approvals,
            restore_request("patch-client-1", &restore_id, &root),
        )
        .unwrap();

        assert!(!file.exists());
        assert_eq!(store.records[0].status, "restored");
        let _ = fs::remove_dir_all(root);
    }

    fn applied_store(
        root: &Path,
        file: &Path,
        approval_id: &str,
        approvals: &ApprovalEngine,
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
        apply_patch_record(
            &mut store,
            approvals,
            PatchApplyRequest {
                approval_id: approval_id.to_string(),
                approved_roots: vec![root.display().to_string()],
                created_at_ms: 2,
                proposal_id: "patch-client-1".to_string(),
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

    fn approve_another_file_write(engine: &mut ApprovalEngine, run_id: &str) -> String {
        let proposal = engine.propose(proposal_input(run_id));
        engine.approve(&proposal.id, 1, "restore approval").unwrap();
        proposal.id
    }

    fn pending_another_file_write(engine: &mut ApprovalEngine, run_id: &str) -> String {
        engine.propose(proposal_input(run_id)).id
    }

    fn proposal_input(run_id: &str) -> ProposalInput {
        ProposalInput {
            action: RiskyAction::FileWrite,
            expected_result: "Restore a checkpointed patch.".to_string(),
            expires_at: 10_000,
            node_id: "patch-restore".to_string(),
            reason: "User requested rollback to the captured checkpoint.".to_string(),
            risk: RiskLevel::Medium,
            rollback_plan: "Restore command is itself the rollback action.".to_string(),
            run_id: run_id.to_string(),
            scope: "One checkpointed patch inside the approved root.".to_string(),
        }
    }

    fn restore_request(proposal_id: &str, approval_id: &str, root: &Path) -> PatchRestoreRequest {
        PatchRestoreRequest {
            approval_id: approval_id.to_string(),
            approved_roots: vec![root.display().to_string()],
            created_at_ms: 3,
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
