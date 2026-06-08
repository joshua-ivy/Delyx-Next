#[cfg(test)]
mod tests {
    use crate::approval_bridge::{
        propose_approval_record, ApprovalBridgeStore, ApprovalProposalRequest, PermissionScopeView,
    };
    use crate::patch_bridge::{DiffLineView, PatchBridgeStore, PatchFileView, PatchProposalView};
    use crate::thread_run_bridge::{
        create_thread_run_record, ThreadRunCreateRequest, ThreadRunStore,
    };
    use crate::{approval_persistence, patch_persistence, thread_run_persistence};
    use std::fs;
    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};

    /// The real desktop app points every bridge at one shared
    /// `delyx-next.sqlite3` file, and each persistence module clears its own
    /// tables on save. This proves they coexist in one migrated database: a later
    /// module's save does not clobber an earlier module's rows, and all survive a
    /// fresh reload from the same on-disk file.
    #[test]
    fn all_bridges_share_one_migrated_database_and_survive_reload() {
        let db = temp_db("d1-shared-store");

        let mut threads = ThreadRunStore::default();
        let record = create_thread_run_record(&mut threads, thread_request()).unwrap();
        thread_run_persistence::save_to_path(&threads, &db).unwrap();

        let mut approvals = ApprovalBridgeStore::default();
        propose_approval_record(&mut approvals, approval_request(&record.run.id)).unwrap();
        approval_persistence::save_to_path(&approvals, &db).unwrap();

        let mut patches = PatchBridgeStore::default();
        patches.records.push(patch(&record.run.id));
        patch_persistence::save_to_path(&patches, &db).unwrap();

        // Reload each store from the single shared file.
        let threads_back = thread_run_persistence::load_from_path(&db).unwrap();
        let approvals_back = approval_persistence::load_from_path(&db).unwrap();
        let patches_back = patch_persistence::load_from_path(&db).unwrap();

        // The thread/run rows written first were not wiped by the later saves.
        assert_eq!(threads_back.records.len(), 1);
        assert_eq!(threads_back.records[0].run_id, record.run.id);
        assert_eq!(approvals_back.records.len(), 1);
        assert_eq!(patches_back.records.len(), 1);
        assert_eq!(patches_back.records[0].run_id, record.run.id);

        // A real SQLite file exists on disk and reopens cleanly.
        assert!(db.exists());
        assert!(thread_run_persistence::load_from_path(&db).is_ok());

        let _ = fs::remove_file(db);
    }

    fn thread_request() -> ThreadRunCreateRequest {
        ThreadRunCreateRequest {
            created_at: "2026-06-08T00:00:00.000Z".to_string(),
            goal: "Shared store round trip.".to_string(),
            project_id: "project-1".to_string(),
        }
    }

    fn approval_request(run_id: &str) -> ApprovalProposalRequest {
        ApprovalProposalRequest {
            action_type: "edit_file".to_string(),
            client_id: "shared-approval".to_string(),
            expected_result: "Apply one patch.".to_string(),
            expires_at: "2999-01-01T00:00:00.000Z".to_string(),
            expires_at_ms: 32_503_680_000_000,
            node_id: format!("{run_id}-patch-apply-patch-1"),
            rationale: "Allow apply.".to_string(),
            required_permission: "write_file".to_string(),
            risk_label: "high".to_string(),
            rollback_plan: Some("Use checkpoint receipts.".to_string()),
            run_id: run_id.to_string(),
            scope: PermissionScopeView {
                commands: None,
                connector_id: None,
                kind: "file".to_string(),
                paths: Some(vec!["src/main.rs".to_string()]),
                project_id: Some("project-1".to_string()),
                root: None,
                summary: "Apply one patch.".to_string(),
            },
        }
    }

    fn patch(run_id: &str) -> PatchProposalView {
        PatchProposalView {
            approval_id: "approval-draft".to_string(),
            checkpoint_files: Vec::new(),
            checkpoint_id: None,
            files: vec![PatchFileView {
                after: "let value = 1;\n".to_string(),
                before: String::new(),
                change_kind: "create".to_string(),
                diff: vec![DiffLineView {
                    kind: "added".to_string(),
                    text: "let value = 1;".to_string(),
                }],
                path: "src/main.rs".to_string(),
            }],
            id: "patch-1".to_string(),
            restore_approval_id: None,
            run_id: run_id.to_string(),
            status: "proposed".to_string(),
        }
    }

    fn temp_db(label: &str) -> PathBuf {
        let stamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        std::env::temp_dir().join(format!("delyx-next-{label}-{stamp}.sqlite3"))
    }
}
