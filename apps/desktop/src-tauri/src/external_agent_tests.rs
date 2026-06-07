#[cfg(test)]
mod tests {
    use crate::approval::{ApprovalEngine, ApprovalError, ProposalInput, RiskLevel, RiskyAction};
    use crate::external_agent::{
        tests_are_trusted, AdapterStatus, ExternalAgentBridge, ExternalAgentCapturePlan, ExternalAgentError,
        ExternalAgentEventKind, ExternalAgentReviewDecision, ExternalAgentRunRequest, ExternalAgentScope,
        ExternalAgentTaskPolicy,
    };
    use std::fs;
    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn detects_placeholder_adapters() {
        let root = temp_workspace("adapter-detect");
        let bridge = ExternalAgentBridge::new(vec![root]).unwrap();
        let adapters = bridge.detect_adapters();

        assert!(adapters.iter().any(|adapter| adapter.adapter_id == "codex-cli" && adapter.status == AdapterStatus::Missing));
        assert!(adapters.iter().any(|adapter| adapter.adapter_id == "claude-code" && adapter.status == AdapterStatus::Missing));
        assert!(adapters.iter().any(|adapter| adapter.adapter_id == "generic-terminal" && adapter.status == AdapterStatus::Available));
    }

    #[test]
    fn external_agent_cannot_run_without_approval() {
        let root = temp_workspace("pending-external");
        let mut approvals = ApprovalEngine::new();
        let approval = approvals.propose(external_agent_input());
        let mut bridge = ExternalAgentBridge::new(vec![root.clone()]).unwrap();

        let result = bridge.run_approved_worker(run_request(&approval.id, &root, true, vec![]), 10, &approvals);

        assert_eq!(result.unwrap_err(), ExternalAgentError::Approval(ApprovalError::NotApproved));
    }

    #[test]
    fn external_agent_scope_must_be_inside_approved_root() {
        let root = temp_workspace("approved-external");
        let outside = temp_workspace("outside-external");
        let mut approvals = ApprovalEngine::new();
        let approval = approvals.propose(external_agent_input());
        approvals.approve(&approval.id, 10, "approved in test").unwrap();
        let mut bridge = ExternalAgentBridge::new(vec![root]).unwrap();

        let result = bridge.run_approved_worker(run_request(&approval.id, &outside, true, vec![]), 10, &approvals);

        assert_eq!(result.unwrap_err(), ExternalAgentError::OutsideApprovedRoot);
    }

    #[test]
    fn approved_generic_worker_captures_transcript_output_and_diff() {
        let root = temp_workspace("approved-worker");
        let mut approvals = ApprovalEngine::new();
        let approval = approvals.propose(external_agent_input());
        approvals.approve(&approval.id, 10, "approved in test").unwrap();
        let mut bridge = ExternalAgentBridge::new(vec![root.clone()]).unwrap();

        let artifact = bridge.run_approved_worker(run_request(&approval.id, &root, true, vec![]), 10, &approvals).unwrap();

        assert_eq!(artifact.adapter_id, "generic-terminal");
        assert!(artifact.transcript.iter().any(|event| event.kind == ExternalAgentEventKind::Stdout));
        assert!(artifact.transcript.iter().any(|event| event.kind == ExternalAgentEventKind::Command));
        assert!(artifact.transcript.iter().any(|event| event.kind == ExternalAgentEventKind::FileChanged));
        assert!(artifact.transcript.iter().any(|event| event.kind == ExternalAgentEventKind::Completed));
        assert!(artifact.transcript.iter().all(|event| event.timestamp == 10));
        assert!(artifact.terminal_output.contains("prototype external agent bridge"));
        assert!(artifact.diff_summary.unwrap().contains("Delyx review"));
        assert!(artifact.review_required);
    }

    #[test]
    fn external_agent_cannot_request_tools_outside_task_policy() {
        let root = temp_workspace("tool-authority");
        let mut approvals = ApprovalEngine::new();
        let approval = approvals.propose(external_agent_input());
        approvals.approve(&approval.id, 10, "approved in test").unwrap();
        let mut bridge = ExternalAgentBridge::new(vec![root.clone()]).unwrap();
        let mut request = run_request(&approval.id, &root, true, vec![]);
        request.allowed_tools.push("terminal".to_string());

        let result = bridge.run_approved_worker(request, 10, &approvals);

        assert_eq!(result.unwrap_err(), ExternalAgentError::ToolNotAllowed("terminal".to_string()));
    }

    #[test]
    fn external_agent_requires_checkpoint_or_worktree_isolation() {
        let root = temp_workspace("missing-isolation");
        let mut approvals = ApprovalEngine::new();
        let approval = approvals.propose(external_agent_input());
        approvals.approve(&approval.id, 10, "approved in test").unwrap();
        let mut bridge = ExternalAgentBridge::new(vec![root.clone()]).unwrap();
        let mut request = run_request(&approval.id, &root, true, vec![]);
        request.scope.checkpoint_id = None;
        request.scope.worktree_id = None;

        let result = bridge.run_approved_worker(request, 10, &approvals);

        assert_eq!(result.unwrap_err(), ExternalAgentError::MissingIsolation);
    }

    #[test]
    fn user_can_accept_or_revert_external_agent_artifact() {
        let root = temp_workspace("review-decision");
        let mut approvals = ApprovalEngine::new();
        let approval = approvals.propose(external_agent_input());
        approvals.approve(&approval.id, 10, "approved in test").unwrap();
        let mut bridge = ExternalAgentBridge::new(vec![root.clone()]).unwrap();
        let artifact = bridge.run_approved_worker(run_request(&approval.id, &root, true, vec![]), 10, &approvals).unwrap();

        let accepted = bridge.record_review_decision(&artifact.id, ExternalAgentReviewDecision::Accept, 20).unwrap();
        let reverted = bridge.record_review_decision(&artifact.id, ExternalAgentReviewDecision::Revert, 30).unwrap();

        assert_eq!(accepted.status, crate::external_agent::ExternalAgentRunStatus::Accepted);
        assert_eq!(reverted.status, crate::external_agent::ExternalAgentRunStatus::Reverted);
        assert!(reverted.transcript.iter().any(|event| event.kind == ExternalAgentEventKind::ReviewDecision && event.timestamp == 30));
    }

    #[test]
    fn tests_are_not_trusted_without_captured_artifacts() {
        let root = temp_workspace("test-trust");
        let mut approvals = ApprovalEngine::new();
        let approval = approvals.propose(external_agent_input());
        approvals.approve(&approval.id, 10, "approved in test").unwrap();
        let mut bridge = ExternalAgentBridge::new(vec![root.clone()]).unwrap();

        let no_tests = bridge.run_approved_worker(run_request(&approval.id, &root, false, vec![]), 10, &approvals).unwrap();
        let with_tests = bridge
            .run_approved_worker(run_request(&approval.id, &root, false, vec!["test-artifact-1".to_string()]), 10, &approvals)
            .unwrap();

        assert!(!tests_are_trusted(&no_tests));
        assert!(tests_are_trusted(&with_tests));
    }

    fn external_agent_input() -> ProposalInput {
        ProposalInput {
            action: RiskyAction::ExternalAgentExecution,
            expires_at: 30,
            expected_result: "External worker runs inside approved scope.".to_string(),
            node_id: "node-external-agent".to_string(),
            reason: "Deterministic external bridge test.".to_string(),
            risk: RiskLevel::High,
            rollback_plan: "Discard transcript and restore checkpoint if changes are rejected.".to_string(),
            run_id: "run-1".to_string(),
            scope: "Launch generic terminal agent in one approved project root.".to_string(),
        }
    }

    fn run_request(approval_id: &str, root: &std::path::Path, capture_diff: bool, test_artifact_ids: Vec<String>) -> ExternalAgentRunRequest {
        ExternalAgentRunRequest {
            adapter_id: "generic-terminal".to_string(),
            allowed_tools: vec!["read".to_string(), "patch_proposal".to_string()],
            approval_id: approval_id.to_string(),
            capture_plan: ExternalAgentCapturePlan {
                capture_diff,
                changed_files: vec![root.join("src").join("main.rs")],
                commands: vec!["echo prototype".to_string()],
                test_artifact_ids,
            },
            run_id: "run-1".to_string(),
            scope: ExternalAgentScope {
                allowed_paths: vec![root.to_path_buf()],
                checkpoint_id: Some("checkpoint-1".to_string()),
                project_root: root.to_path_buf(),
                worktree_id: None,
            },
            task: "Inspect and propose a patch.".to_string(),
            task_policy: ExternalAgentTaskPolicy {
                allowed_tools: vec!["read".to_string(), "patch_proposal".to_string()],
                approval_scope: "Launch generic terminal agent in one approved project root.".to_string(),
            },
            timeout_ms: 60_000,
        }
    }

    fn temp_workspace(label: &str) -> PathBuf {
        let stamp = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos();
        let path = std::env::temp_dir().join(format!("delyx-next-{label}-{stamp}"));
        fs::create_dir_all(&path).unwrap();
        path
    }
}
