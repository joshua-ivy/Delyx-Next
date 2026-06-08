#[cfg(test)]
mod tests {
    use crate::agent_review_executor::{execute_review_node, AgentReviewExecutionStatus};
    use crate::agent_run::{AgentRunLedger, AgentRunStatus};
    use crate::patch_bridge::{DiffLineView, PatchFileView, PatchProposalView};
    use crate::review_bridge::ReviewBridgeStore;

    #[test]
    fn review_node_records_report_from_patch_artifacts() {
        let mut ledger = AgentRunLedger::new();
        let run = ledger.create_run("thread-1").unwrap();
        let mut reviews = ReviewBridgeStore::default();

        let result = execute_review_node(
            &mut ledger,
            &mut reviews,
            run.id.clone(),
            vec![patch(&run.id, "let value = maybe.unwrap();")],
            Vec::new(),
        )
        .unwrap();
        let run = ledger.get_run(&run.id).unwrap();

        assert_eq!(result.status, AgentReviewExecutionStatus::Completed);
        assert_eq!(
            reviews.reports[0].findings[0].title,
            "Added unwrap can panic"
        );
        assert_eq!(run.nodes[0].kind, "diff_review");
        assert_eq!(run.nodes[0].status, AgentRunStatus::Completed);
        assert_eq!(run.artifacts[0].kind, "review_report");
    }

    #[test]
    fn review_node_fails_on_mismatched_patch_run() {
        let mut ledger = AgentRunLedger::new();
        let run = ledger.create_run("thread-1").unwrap();
        let mut reviews = ReviewBridgeStore::default();

        let result = execute_review_node(
            &mut ledger,
            &mut reviews,
            run.id.clone(),
            vec![patch("other-run", "let value = maybe.unwrap();")],
            Vec::new(),
        )
        .unwrap();

        assert_eq!(result.status, AgentReviewExecutionStatus::Failed);
        assert_eq!(
            ledger.get_run(&run.id).unwrap().status,
            AgentRunStatus::Failed
        );
        assert!(reviews.reports.is_empty());
    }

    fn patch(run_id: &str, added: &str) -> PatchProposalView {
        PatchProposalView {
            approval_id: "approval-1".to_string(),
            checkpoint_files: Vec::new(),
            checkpoint_id: None,
            files: vec![PatchFileView {
                after: added.to_string(),
                before: String::new(),
                change_kind: "create".to_string(),
                diff: vec![DiffLineView {
                    kind: "added".to_string(),
                    text: added.to_string(),
                }],
                path: "src/main.rs".to_string(),
            }],
            id: "patch-1".to_string(),
            restore_approval_id: None,
            run_id: run_id.to_string(),
            status: "proposed".to_string(),
        }
    }
}
