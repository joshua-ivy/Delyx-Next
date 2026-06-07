#[cfg(test)]
mod tests {
    use crate::review_bridge::{
        create_review_record, review_snapshot_from_store, DiffLineReviewInput, PatchFileReviewInput,
        PatchReviewInput, ReviewBridgeStore, ReviewCreateRequest, TestReviewInput,
    };

    #[test]
    fn review_bridge_returns_ui_ready_patch_findings() {
        let mut store = ReviewBridgeStore::default();

        let report = create_review_record(&mut store, request(vec![
            added_line("let value = maybe.unwrap();"),
            added_line("// TODO: finish"),
        ], Vec::new()))
        .unwrap();

        assert_eq!(report.mode, "read_only");
        assert_eq!(report.decision, "pending");
        assert_eq!(report.findings.len(), 2);
        assert_eq!(report.findings[0].priority, "p1");
        assert_eq!(report.findings[0].title, "Added unwrap can panic");
        assert_eq!(report.findings[0].file_path, "src/main.rs");
        assert_eq!(report.findings[0].hunk_label, "patch-1:0");
    }

    #[test]
    fn failed_test_artifact_creates_review_finding() {
        let mut store = ReviewBridgeStore::default();

        let report = create_review_record(&mut store, request(
            vec![added_line("let value = 1;")],
            vec![failed_test()],
        ))
        .unwrap();

        assert!(report.test_summary.contains("failed"));
        assert!(report.findings.iter().any(|finding| finding.title == "Test artifact failed"));
    }

    #[test]
    fn review_snapshot_filters_by_run() {
        let mut store = ReviewBridgeStore::default();
        create_review_record(&mut store, request(vec![added_line("let value = 1;")], Vec::new())).unwrap();

        assert_eq!(review_snapshot_from_store(&store, "run-1").len(), 1);
        assert!(review_snapshot_from_store(&store, "run-2").is_empty());
    }

    #[test]
    fn not_run_test_artifacts_are_rejected() {
        let mut store = ReviewBridgeStore::default();
        let mut test = failed_test();
        test.status = Some("not_run".to_string());

        let result = create_review_record(&mut store, request(vec![added_line("let value = 1;")], vec![test]));

        assert!(result.unwrap_err().contains("did not run"));
        assert!(review_snapshot_from_store(&store, "run-1").is_empty());
    }

    #[test]
    fn mismatched_run_artifacts_are_rejected() {
        let mut store = ReviewBridgeStore::default();
        let mut patch_request = request(vec![added_line("let value = 1;")], Vec::new());
        patch_request.patches[0].run_id = "run-2".to_string();

        let patch_result = create_review_record(&mut store, patch_request);

        assert!(patch_result.unwrap_err().contains("requested run"));

        let mut test_request = request(vec![added_line("let value = 1;")], vec![failed_test()]);
        test_request.tests[0].run_id = "run-2".to_string();

        let test_result = create_review_record(&mut store, test_request);

        assert!(test_result.unwrap_err().contains("requested run"));
        assert!(review_snapshot_from_store(&store, "run-1").is_empty());
    }

    fn request(lines: Vec<DiffLineReviewInput>, tests: Vec<TestReviewInput>) -> ReviewCreateRequest {
        ReviewCreateRequest {
            patches: vec![PatchReviewInput {
                approval_id: "prop-1".to_string(),
                files: vec![PatchFileReviewInput {
                    diff: lines,
                    path: "src/main.rs".to_string(),
                }],
                id: "patch-1".to_string(),
                run_id: "run-1".to_string(),
                status: "proposed".to_string(),
            }],
            run_id: "run-1".to_string(),
            tests,
        }
    }

    fn added_line(text: &str) -> DiffLineReviewInput {
        DiffLineReviewInput {
            kind: "added".to_string(),
            text: text.to_string(),
        }
    }

    fn failed_test() -> TestReviewInput {
        TestReviewInput {
            approval_id: Some("prop-2".to_string()),
            command: "cargo test".to_string(),
            cwd: "C:/workspace".to_string(),
            duration_ms: 42,
            exit_code: Some(1),
            failure_summary: Some("assertion failed".to_string()),
            id: "test-artifact-1".to_string(),
            run_id: "run-1".to_string(),
            status: Some("failed".to_string()),
            stderr: "assertion failed".to_string(),
            stdout: String::new(),
        }
    }
}
