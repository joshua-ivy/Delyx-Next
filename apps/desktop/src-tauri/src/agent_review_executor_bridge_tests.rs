#[cfg(test)]
mod tests {
    use crate::agent_review_executor_bridge::{execute_review_record, AgentReviewExecuteRequest};
    use crate::patch_bridge::{
        propose_patch_record, PatchBridgeStore, PatchFileRequest, PatchProposalRequest,
    };
    use crate::review_bridge::ReviewBridgeStore;
    use crate::test_runner_bridge::TestRunnerBridgeStore;
    use crate::thread_run_bridge::{
        create_thread_run_record, ThreadRunCreateRequest, ThreadRunStore,
    };
    use std::fs;
    use std::path::{Path, PathBuf};
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn bridge_reviews_persisted_patch_artifacts_for_run() {
        let root = temp_workspace("review-executor-bridge");
        let file = root.join("src.rs");
        fs::write(&file, "let value = 1;\n").unwrap();
        let mut thread_store = ThreadRunStore::default();
        let record = create_thread_run_record(&mut thread_store, create_request()).unwrap();
        let mut patch_store = PatchBridgeStore::default();
        let test_store = TestRunnerBridgeStore::default();
        let mut review_store = ReviewBridgeStore::default();
        propose_patch_record(
            &mut patch_store,
            patch_request(&record.run.id, &root, &file),
        )
        .unwrap();

        let view = execute_review_record(
            &mut thread_store,
            &patch_store,
            &test_store,
            &mut review_store,
            AgentReviewExecuteRequest {
                run_id: record.run.id.clone(),
            },
        )
        .unwrap();

        assert_eq!(view.status, "completed");
        assert_eq!(
            review_store.reports[0].findings[0].title,
            "Added unwrap can panic"
        );
        let run = thread_store.ledger.get_run(&record.run.id).unwrap();
        assert!(run
            .artifacts
            .iter()
            .any(|artifact| artifact.kind == "review_report"));
        let _ = fs::remove_dir_all(root);
    }

    fn create_request() -> ThreadRunCreateRequest {
        ThreadRunCreateRequest {
            created_at: "2026-06-08T00:00:00.000Z".to_string(),
            goal: "Review through executor bridge".to_string(),
            project_id: "proj-1".to_string(),
        }
    }

    fn patch_request(run_id: &str, root: &Path, file: &Path) -> PatchProposalRequest {
        PatchProposalRequest {
            approval_id: "approval-1".to_string(),
            approved_roots: vec![root.display().to_string()],
            client_id: "review-patch-1".to_string(),
            files: vec![PatchFileRequest {
                after: "let value = maybe.unwrap();\n".to_string(),
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
