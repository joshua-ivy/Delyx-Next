#[cfg(test)]
mod tests {
    use crate::approval::{ApprovalEngine, ProposalInput, RiskLevel, RiskyAction};
    use crate::patch::{DiffLineKind, PatchEngine, PatchError, PatchFileInput, PatchInput, PatchStatus};
    use std::fs;
    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn patch_can_be_proposed_without_applying() {
        let root = temp_workspace("proposal-only");
        let file = root.join("settings.toml");
        fs::write(&file, "network = true\n").unwrap();
        let mut engine = PatchEngine::new(vec![root]).unwrap();

        let proposal = engine.propose_patch(patch_input("prop-1", &file, "network = false\n")).unwrap();

        assert_eq!(proposal.status, PatchStatus::Proposed);
        assert_eq!(fs::read_to_string(&file).unwrap(), "network = true\n");
        assert!(proposal.files[0].diff.iter().any(|line| line.kind == DiffLineKind::Removed));
        assert!(proposal.files[0].diff.iter().any(|line| line.kind == DiffLineKind::Added));
    }

    #[test]
    fn patch_cannot_be_applied_without_approval() {
        let root = temp_workspace("pending-approval");
        let file = root.join("main.rs");
        fs::write(&file, "fn main() {}\n").unwrap();
        let mut approvals = ApprovalEngine::new();
        let approval = approvals.propose(file_write_input(30));
        let mut engine = PatchEngine::new(vec![root]).unwrap();
        let patch = engine.propose_patch(patch_input(&approval.id, &file, "fn main() { println!(\"ok\"); }\n")).unwrap();

        let result = engine.apply_approved_patch(&patch.id, 10, &approvals);

        assert_eq!(result.unwrap_err(), PatchError::Approval(crate::approval::ApprovalError::NotApproved));
        assert_eq!(fs::read_to_string(&file).unwrap(), "fn main() {}\n");
    }

    #[test]
    fn approved_patch_creates_checkpoint_and_writes() {
        let root = temp_workspace("approved-write");
        let file = root.join("plan.md");
        fs::write(&file, "before\n").unwrap();
        let mut approvals = ApprovalEngine::new();
        let approval = approvals.propose(file_write_input(30));
        approvals.approve(&approval.id, 10, "approved in test").unwrap();
        let mut engine = PatchEngine::new(vec![root]).unwrap();
        let patch = engine.propose_patch(patch_input(&approval.id, &file, "after\n")).unwrap();

        let checkpoint = engine.apply_approved_patch(&patch.id, 10, &approvals).unwrap();

        assert_eq!(checkpoint.files[0].contents.as_deref(), Some("before\n"));
        assert_eq!(fs::read_to_string(&file).unwrap(), "after\n");
        assert_eq!(engine.list_proposals("run-1")[0].status, PatchStatus::Applied);
    }

    #[test]
    fn restore_checkpoint_restores_original_contents() {
        let root = temp_workspace("restore");
        let file = root.join("copy.txt");
        fs::write(&file, "original\n").unwrap();
        let mut approvals = ApprovalEngine::new();
        let approval = approvals.propose(file_write_input(30));
        approvals.approve(&approval.id, 10, "approved in test").unwrap();
        let mut engine = PatchEngine::new(vec![root]).unwrap();
        let patch = engine.propose_patch(patch_input(&approval.id, &file, "changed\n")).unwrap();
        let checkpoint = engine.apply_approved_patch(&patch.id, 10, &approvals).unwrap();

        engine.restore_checkpoint(&checkpoint.id, 10, &approvals).unwrap();

        assert_eq!(fs::read_to_string(&file).unwrap(), "original\n");
        assert_eq!(engine.list_proposals("run-1")[0].status, PatchStatus::Restored);
    }

    #[test]
    fn outside_approved_root_is_rejected() {
        let root = temp_workspace("inside-root");
        let outside = temp_workspace("outside-root").join("escape.txt");
        let mut engine = PatchEngine::new(vec![root]).unwrap();

        let result = engine.propose_patch(patch_input("prop-1", &outside, "nope\n"));

        assert_eq!(result.unwrap_err(), PatchError::OutsideApprovedRoot);
    }

    fn patch_input(approval_id: &str, path: &std::path::Path, after: &str) -> PatchInput {
        PatchInput {
            run_id: "run-1".to_string(),
            approval_id: approval_id.to_string(),
            files: vec![PatchFileInput { path: path.to_path_buf(), after: after.to_string() }],
        }
    }

    fn file_write_input(expires_at: u64) -> ProposalInput {
        ProposalInput {
            action: RiskyAction::FileWrite,
            expires_at,
            expected_result: "Patch writes requested file contents.".to_string(),
            node_id: "node-patch".to_string(),
            reason: "Deterministic patch test.".to_string(),
            risk: RiskLevel::High,
            rollback_plan: "Restore the checkpoint created before writing.".to_string(),
            run_id: "run-1".to_string(),
            scope: "Write one approved-root file.".to_string(),
        }
    }

    fn temp_workspace(label: &str) -> PathBuf {
        let stamp = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos();
        let path = std::env::temp_dir().join(format!("delyx-next-{label}-{stamp}"));
        fs::create_dir_all(&path).unwrap();
        path
    }
}
