#[cfg(test)]
mod tests {
    use crate::patch_bridge::{
        patch_snapshot_from_store, propose_patch_record, PatchBridgeStore, PatchFileRequest,
        PatchProposalRequest,
    };
    use std::fs;
    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn patch_bridge_returns_ui_ready_diff_without_writing() {
        let root = temp_workspace("bridge-proposal");
        let file = root.join("settings.toml");
        fs::write(&file, "network = true\n").unwrap();
        let mut store = PatchBridgeStore::default();

        let proposal = propose_patch_record(
            &mut store,
            request("patch-client-1", &root, &file, "network = false\n"),
        )
        .unwrap();

        assert_eq!(proposal.id, "patch-client-1");
        assert_eq!(proposal.status, "proposed");
        assert_eq!(proposal.files[0].change_kind, "modify");
        assert_eq!(proposal.files[0].before, "network = true\n");
        assert_eq!(proposal.files[0].after, "network = false\n");
        assert!(proposal.checkpoint_files.is_empty());
        assert_eq!(proposal.files.len(), 1);
        assert!(proposal.files[0]
            .diff
            .iter()
            .any(|line| line.kind == "removed"));
        assert!(proposal.files[0]
            .diff
            .iter()
            .any(|line| line.kind == "added"));
        assert_eq!(fs::read_to_string(&file).unwrap(), "network = true\n");
    }

    #[test]
    fn patch_bridge_reports_create_intent_without_writing() {
        let root = temp_workspace("bridge-create-intent");
        let file = root.join("new.txt");
        let mut store = PatchBridgeStore::default();

        let proposal = propose_patch_record(
            &mut store,
            request("patch-client-1", &root, &file, "created\n"),
        )
        .unwrap();

        assert_eq!(proposal.files[0].change_kind, "create");
        assert!(!file.exists());
    }

    #[test]
    fn duplicate_client_id_returns_existing_patch_record() {
        let root = temp_workspace("bridge-duplicate");
        let file = root.join("copy.txt");
        fs::write(&file, "before\n").unwrap();
        let mut store = PatchBridgeStore::default();

        let first = propose_patch_record(
            &mut store,
            request("patch-client-1", &root, &file, "after\n"),
        )
        .unwrap();
        let second = propose_patch_record(
            &mut store,
            request("patch-client-1", &root, &file, "later\n"),
        )
        .unwrap();
        let snapshot = patch_snapshot_from_store(&store, "run-1");

        assert_eq!(first, second);
        assert_eq!(snapshot.len(), 1);
    }

    #[test]
    fn patch_snapshot_filters_by_run_id() {
        let root = temp_workspace("bridge-snapshot");
        let file = root.join("copy.txt");
        fs::write(&file, "before\n").unwrap();
        let mut store = PatchBridgeStore::default();
        propose_patch_record(
            &mut store,
            request("patch-client-1", &root, &file, "after\n"),
        )
        .unwrap();

        assert_eq!(patch_snapshot_from_store(&store, "run-1").len(), 1);
        assert!(patch_snapshot_from_store(&store, "run-2").is_empty());
    }

    #[test]
    fn outside_approved_root_is_rejected_without_storing_patch() {
        let root = temp_workspace("bridge-root");
        let outside = temp_workspace("bridge-outside").join("escape.txt");
        let mut store = PatchBridgeStore::default();

        let result = propose_patch_record(
            &mut store,
            request("patch-client-1", &root, &outside, "nope\n"),
        );

        assert!(result.unwrap_err().contains("OutsideApprovedRoot"));
        assert!(patch_snapshot_from_store(&store, "run-1").is_empty());
    }

    fn request(
        client_id: &str,
        root: &std::path::Path,
        path: &std::path::Path,
        after: &str,
    ) -> PatchProposalRequest {
        PatchProposalRequest {
            approval_id: "prop-1".to_string(),
            approved_roots: vec![root.display().to_string()],
            client_id: client_id.to_string(),
            files: vec![PatchFileRequest {
                after: after.to_string(),
                path: path.display().to_string(),
            }],
            run_id: "run-1".to_string(),
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
