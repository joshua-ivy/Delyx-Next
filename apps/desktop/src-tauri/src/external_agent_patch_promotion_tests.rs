#[cfg(test)]
mod tests {
    use crate::external_agent_diff::{
        external_diff_file_changes, snapshot_external_agent_diff, ExternalDiffFileChange,
    };
    use crate::external_agent_patch_promotion::promote_worker_diff_to_patch;
    use crate::patch_bridge::PatchBridgeStore;
    use std::fs;
    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn snapshot_extracts_modify_create_and_delete_changes() {
        let root = temp_workspace("promo-changes");
        let modified = root.join("modified.txt");
        let deleted = root.join("deleted.txt");
        let created = root.join("created.txt");
        fs::write(&modified, "before\n").unwrap();
        fs::write(&deleted, "old\n").unwrap();
        let snapshot =
            snapshot_external_agent_diff(&[modified.clone(), deleted.clone(), created.clone()]);

        // Simulate the worker's edits.
        fs::write(&modified, "after\n").unwrap();
        fs::remove_file(&deleted).unwrap();
        fs::write(&created, "new\n").unwrap();

        let changes = external_diff_file_changes(&snapshot);
        assert_eq!(changes.len(), 3);
        let kinds: Vec<&str> = changes.iter().map(|c| c.change_kind).collect();
        assert_eq!(kinds, vec!["modify", "delete", "create"]);
        assert_eq!(changes[0].before, "before\n");
        assert_eq!(changes[0].after, "after\n");
        assert_eq!(changes[2].before, "");
        assert_eq!(changes[2].after, "new\n");

        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn unchanged_files_produce_no_changes() {
        let root = temp_workspace("promo-unchanged");
        let stable = root.join("stable.txt");
        fs::write(&stable, "same\n").unwrap();
        let snapshot = snapshot_external_agent_diff(&[stable.clone()]);
        assert!(external_diff_file_changes(&snapshot).is_empty());
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn promotion_creates_an_applied_restorable_patch_record() {
        let mut store = PatchBridgeStore::default();
        let changes = vec![
            ExternalDiffFileChange {
                path: "C:/app/src/lib.rs".to_string(),
                before: "old\n".to_string(),
                after: "new\n".to_string(),
                change_kind: "modify",
            },
            ExternalDiffFileChange {
                path: "C:/app/src/new.rs".to_string(),
                before: String::new(),
                after: "fresh\n".to_string(),
                change_kind: "create",
            },
        ];

        let view = promote_worker_diff_to_patch(&mut store, "run-1", "approval-ext", &changes)
            .expect("changes should promote");

        assert_eq!(store.records.len(), 1);
        assert_eq!(view.status, "applied");
        assert_eq!(view.run_id, "run-1");
        assert_eq!(view.approval_id, "approval-ext");
        assert!(view
            .checkpoint_id
            .as_deref()
            .unwrap_or_default()
            .contains("worker-checkpoint"));
        // Checkpoint receipts: pre-run contents for modified, None for created
        // (restore removes the file).
        assert_eq!(view.checkpoint_files.len(), 2);
        assert_eq!(view.checkpoint_files[0].contents.as_deref(), Some("old\n"));
        assert_eq!(view.checkpoint_files[1].contents, None);
        // Diff lines exist for the review UI.
        assert!(view.files[0].diff.iter().any(|line| line.kind == "added"));
        assert!(view.files[0].diff.iter().any(|line| line.kind == "removed"));
    }

    #[test]
    fn empty_changes_promote_nothing() {
        let mut store = PatchBridgeStore::default();
        assert!(promote_worker_diff_to_patch(&mut store, "run-1", "a", &[]).is_none());
        assert!(store.records.is_empty());
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
