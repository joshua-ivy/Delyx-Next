#[cfg(test)]
mod tests {
    use std::time::{SystemTime, UNIX_EPOCH};

    use crate::review_bridge::{
        create_review_record, review_snapshot_from_store, DiffLineReviewInput,
        PatchFileReviewInput, PatchReviewInput, ReviewBridgeStore, ReviewCreateRequest,
        TestReviewInput,
    };
    use crate::review_persistence::{load_from_path, save_to_path};

    #[test]
    fn review_bridge_store_survives_sqlite_reload() {
        let path = temp_db_path("review-reload");
        let mut store = ReviewBridgeStore::default();
        let report = create_review_record(
            &mut store,
            request(
                "run-1",
                vec![
                    added_line("let value = maybe.unwrap();"),
                    added_line("// TODO: finish"),
                ],
                vec![failed_test()],
            ),
        )
        .unwrap();

        save_to_path(&store, &path).unwrap();
        let mut loaded = load_from_path(&path).unwrap();
        let snapshot = review_snapshot_from_store(&loaded, "run-1");

        assert_eq!(snapshot, vec![report.clone()]);
        assert_eq!(snapshot[0].findings.len(), 3);
        assert!(snapshot[0]
            .findings
            .iter()
            .any(|finding| finding.title == "Added unwrap can panic"));
        assert!(snapshot[0]
            .findings
            .iter()
            .any(|finding| finding.title == "Test artifact failed"));
        assert!(review_snapshot_from_store(&loaded, "run-2").is_empty());

        let next = create_review_record(
            &mut loaded,
            request(
                "run-1",
                vec![added_line("// TODO: second pass")],
                Vec::new(),
            ),
        )
        .unwrap();

        assert_eq!(next.id, "review-2");
        assert_eq!(next.findings[0].id, "finding-4");

        std::fs::remove_file(path).ok();
    }

    fn request(
        run_id: &str,
        lines: Vec<DiffLineReviewInput>,
        tests: Vec<TestReviewInput>,
    ) -> ReviewCreateRequest {
        ReviewCreateRequest {
            patches: vec![PatchReviewInput {
                approval_id: "prop-1".to_string(),
                files: vec![PatchFileReviewInput {
                    diff: lines,
                    path: "src/main.rs".to_string(),
                }],
                id: "patch-1".to_string(),
                run_id: run_id.to_string(),
                status: "proposed".to_string(),
            }],
            run_id: run_id.to_string(),
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

    fn temp_db_path(label: &str) -> std::path::PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        std::env::temp_dir().join(format!("delyx-next-{label}-{nanos}.sqlite"))
    }
}
