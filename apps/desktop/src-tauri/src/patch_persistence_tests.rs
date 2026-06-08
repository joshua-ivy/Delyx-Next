#[cfg(test)]
mod tests {
    use crate::patch_bridge::{patch_snapshot_from_store, propose_patch_record, PatchBridgeStore};
    use crate::patch_bridge::{PatchFileRequest, PatchProposalRequest};
    use crate::patch_persistence::{load_from_path, save_to_path};
    use std::fs;
    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn patch_bridge_store_survives_sqlite_reload() {
        let root = temp_workspace("patch-persistence");
        let db_path = root.join("delyx.sqlite3");
        let file = root.join("settings.toml");
        fs::write(&file, "network = true\n").unwrap();
        let mut store = PatchBridgeStore::default();

        let first =
            propose_patch_record(&mut store, request("", &root, &file, "network = false\n"))
                .unwrap();
        save_to_path(&store, &db_path).unwrap();
        let mut loaded = load_from_path(&db_path).unwrap();
        let loaded_snapshot = patch_snapshot_from_store(&loaded, "run-1");

        assert!(fs::read(&db_path).unwrap().starts_with(b"SQLite format 3"));
        assert_eq!(loaded_snapshot, vec![first.clone()]);
        assert_eq!(first.id, "patch-1");
        assert_eq!(first.files[0].before, "network = true\n");
        assert_eq!(first.files[0].after, "network = false\n");
        assert!(first.files[0].diff.iter().any(|line| line.kind == "added"));
        assert!(first.files[0]
            .diff
            .iter()
            .any(|line| line.kind == "removed"));

        let second =
            propose_patch_record(&mut loaded, request("", &root, &file, "network = maybe\n"))
                .unwrap();
        save_to_path(&loaded, &db_path).unwrap();
        let reloaded = load_from_path(&db_path).unwrap();

        assert_eq!(second.id, "patch-2");
        assert_eq!(
            patch_snapshot_from_store(&reloaded, "run-1"),
            vec![first, second]
        );
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
