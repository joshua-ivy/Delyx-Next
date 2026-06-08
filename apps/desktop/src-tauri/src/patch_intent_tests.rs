#[cfg(test)]
mod tests {
    use crate::patch::{PatchEngine, PatchError, PatchFileChangeKind, PatchFileInput, PatchInput};
    use std::fs;
    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn new_file_patch_is_classified_before_apply() {
        let root = temp_workspace("create-intent");
        let file = root.join("new.txt");
        let mut engine = PatchEngine::new(vec![root]).unwrap();

        let proposal = engine
            .propose_patch(patch_input("prop-1", &file, "created\n"))
            .unwrap();

        assert_eq!(proposal.files[0].change_kind, PatchFileChangeKind::Create);
        assert_eq!(proposal.files[0].before, "");
        assert!(!file.exists());
    }

    #[test]
    fn noop_file_patch_is_rejected_before_approval_or_apply() {
        let root = temp_workspace("noop-intent");
        let file = root.join("same.txt");
        fs::write(&file, "same\n").unwrap();
        let mut engine = PatchEngine::new(vec![root]).unwrap();

        let result = engine.propose_patch(patch_input("prop-1", &file, "same\n"));

        assert_eq!(result.unwrap_err(), PatchError::NoFileChanges);
        assert!(engine.list_proposals("run-1").is_empty());
    }

    fn patch_input(approval_id: &str, path: &std::path::Path, after: &str) -> PatchInput {
        PatchInput {
            approval_id: approval_id.to_string(),
            files: vec![PatchFileInput {
                after: after.to_string(),
                path: path.to_path_buf(),
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
