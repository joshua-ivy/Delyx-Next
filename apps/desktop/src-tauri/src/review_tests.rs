#[cfg(test)]
mod tests {
    use crate::patch::{
        DiffLine, DiffLineKind, PatchFile, PatchFileChangeKind, PatchProposal, PatchStatus,
    };
    use crate::review::{
        FindingPriority, ReviewAgent, ReviewCapability, ReviewDecision, ReviewError,
    };
    use crate::test_runner::{TestArtifact, TestStatus};
    use std::path::PathBuf;

    #[test]
    fn review_agent_is_read_only() {
        assert!(!ReviewAgent::can_write());
        assert_eq!(
            ReviewAgent::capabilities(),
            vec![
                ReviewCapability::ReadDiff,
                ReviewCapability::ReadTestArtifact,
                ReviewCapability::ReadEvidence
            ],
        );
    }

    #[test]
    fn findings_link_to_diff_hunks_and_are_prioritized() {
        let mut agent = ReviewAgent::new();
        let report = agent.review(
            "run-1",
            &[patch_with_added("let value = maybe.unwrap();")],
            &[],
        );

        assert_eq!(report.mode, crate::review::ReviewMode::ReadOnly);
        assert_eq!(report.findings[0].priority, FindingPriority::P1);
        assert_eq!(report.findings[0].title, "Added unwrap can panic");
        assert_eq!(report.findings[0].hunk.patch_id, "patch-1");
        assert_eq!(report.findings[0].hunk.file_path, "src/main.rs");
        assert_eq!(report.findings[0].hunk.line_index, 0);
    }

    #[test]
    fn todo_findings_are_lower_priority_than_panic_risks() {
        let mut agent = ReviewAgent::new();
        let patch = patch_with_lines(vec!["// TODO: finish", "let value = maybe.unwrap();"]);

        let report = agent.review("run-1", &[patch], &[]);

        assert_eq!(report.findings[0].priority, FindingPriority::P1);
        assert_eq!(report.findings[1].priority, FindingPriority::P2);
    }

    #[test]
    fn failed_tests_create_prioritized_findings() {
        let mut agent = ReviewAgent::new();
        let report = agent.review(
            "run-1",
            &[patch_with_added("let value = 1;")],
            &[failed_artifact()],
        );

        assert_eq!(report.findings[0].priority, FindingPriority::P1);
        assert_eq!(report.findings[0].title, "Test artifact failed");
        assert_eq!(report.findings[0].detail, "assertion failed");
        assert_eq!(report.test_summary, "At least one test artifact failed.");
    }

    #[test]
    fn revision_request_creates_plan_build_flow_marker() {
        let mut agent = ReviewAgent::new();
        let report = agent.review(
            "run-1",
            &[patch_with_added("let value = maybe.unwrap();")],
            &[],
        );
        let finding_id = report.findings[0].id.clone();

        let request = agent.request_revision(&report.id, &finding_id).unwrap();

        assert_eq!(request.decision, ReviewDecision::ReviseRequested);
        assert_eq!(request.next_flow, vec!["plan", "build"]);
        assert_eq!(
            agent.report_decision(&report.id).unwrap(),
            ReviewDecision::ReviseRequested
        );
    }

    #[test]
    fn revision_requires_existing_finding() {
        let mut agent = ReviewAgent::new();
        let report = agent.review("run-1", &[patch_with_added("let value = 1;")], &[]);

        let result = agent.request_revision(&report.id, "missing");

        assert_eq!(result.unwrap_err(), ReviewError::FindingNotFound);
    }

    fn patch_with_added(line: &str) -> PatchProposal {
        patch_with_lines(vec![line])
    }

    fn patch_with_lines(lines: Vec<&str>) -> PatchProposal {
        PatchProposal {
            approval_id: "prop-1".to_string(),
            checkpoint_id: None,
            files: vec![PatchFile {
                after: lines.join("\n"),
                before: String::new(),
                change_kind: PatchFileChangeKind::Create,
                diff: lines
                    .into_iter()
                    .map(|text| DiffLine {
                        kind: DiffLineKind::Added,
                        text: text.to_string(),
                    })
                    .collect(),
                path: PathBuf::from("src/main.rs"),
            }],
            id: "patch-1".to_string(),
            run_id: "run-1".to_string(),
            status: PatchStatus::Proposed,
        }
    }

    fn failed_artifact() -> TestArtifact {
        TestArtifact {
            approval_id: "prop-1".to_string(),
            command: "cargo test".to_string(),
            created_at: 10,
            duration_ms: 12,
            exit_code: Some(1),
            failure_summary: Some("assertion failed".to_string()),
            id: "test-artifact-1".to_string(),
            exec_events: Vec::new(),
            output_truncated: false,
            run_id: "run-1".to_string(),
            status: TestStatus::Failed,
            stderr: "assertion failed".to_string(),
            stdout: String::new(),
            working_directory: PathBuf::from("C:/workspace"),
        }
    }
}
