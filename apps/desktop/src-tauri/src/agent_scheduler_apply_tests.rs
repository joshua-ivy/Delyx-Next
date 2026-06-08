#[cfg(test)]
mod tests {
    use crate::agent_run::AgentRunLedger;
    use crate::agent_scheduler::{schedule_next, AgentScheduleDecision, AgentSchedulerContext};
    use crate::approval::{ApprovalEngine, ProposalInput, RiskLevel, RiskyAction};
    use crate::patch_bridge::{DiffLineView, PatchBridgeStore, PatchFileView, PatchProposalView};
    use crate::review_bridge::ReviewBridgeStore;
    use crate::test_runner_bridge::TestRunnerBridgeStore;

    #[test]
    fn scheduler_requests_apply_approval_before_patch_apply() {
        let mut ledger = AgentRunLedger::new();
        let run = ledger.create_run("thread-1").unwrap();
        let approvals = ApprovalEngine::new();
        let mut patches = PatchBridgeStore::default();
        let reviews = ReviewBridgeStore::default();
        let tests = TestRunnerBridgeStore::default();
        patches.records.push(patch(&run.id));

        let decision = schedule_next(context(&run, &approvals, &patches, &reviews, &tests, None));

        assert_eq!(
            decision,
            AgentScheduleDecision::RequestPatchApplyApproval {
                proposal_id: "patch-1".to_string()
            }
        );
    }

    #[test]
    fn scheduler_runs_patch_apply_only_with_apply_approval() {
        let mut ledger = AgentRunLedger::new();
        let run = ledger.create_run("thread-1").unwrap();
        let mut approvals = ApprovalEngine::new();
        let approval = approvals.propose(proposal_input(&run.id));
        approvals.approve(&approval.id, 2, "approved").unwrap();
        let mut patches = PatchBridgeStore::default();
        let reviews = ReviewBridgeStore::default();
        let tests = TestRunnerBridgeStore::default();
        patches.records.push(patch(&run.id));

        let decision = schedule_next(context(
            &run,
            &approvals,
            &patches,
            &reviews,
            &tests,
            Some(&approval.id),
        ));

        assert_eq!(
            decision,
            AgentScheduleDecision::RunPatchApply {
                approval_id: approval.id,
                proposal_id: "patch-1".to_string()
            }
        );
    }

    fn context<'a>(
        run: &'a crate::agent_run::AgentRun,
        approvals: &'a ApprovalEngine,
        patches: &'a PatchBridgeStore,
        reviews: &'a ReviewBridgeStore,
        tests: &'a TestRunnerBridgeStore,
        patch_apply_approval_id: Option<&'a str>,
    ) -> AgentSchedulerContext<'a> {
        AgentSchedulerContext {
            approvals,
            has_supported_test_command: false,
            now_ms: 3,
            patch_apply_approval_id,
            patch_draft_approval_id: None,
            patches,
            reviews,
            run,
            test_approval_id: None,
            tests,
        }
    }

    fn proposal_input(run_id: &str) -> ProposalInput {
        ProposalInput {
            action: RiskyAction::FileWrite,
            expected_result: "Apply an approved patch proposal.".to_string(),
            expires_at: 10,
            node_id: "run-1-patch-apply-patch-1".to_string(),
            reason: "Apply the proposed patch.".to_string(),
            risk: RiskLevel::High,
            rollback_plan: "Use checkpoint receipts to restore files.".to_string(),
            run_id: run_id.to_string(),
            scope: "Apply patch-1.".to_string(),
        }
    }

    fn patch(run_id: &str) -> PatchProposalView {
        PatchProposalView {
            approval_id: "approval-draft".to_string(),
            checkpoint_files: Vec::new(),
            checkpoint_id: None,
            files: vec![PatchFileView {
                after: "after\n".to_string(),
                before: "before\n".to_string(),
                change_kind: "modify".to_string(),
                diff: vec![DiffLineView {
                    kind: "added".to_string(),
                    text: "after".to_string(),
                }],
                path: "src/lib.rs".to_string(),
            }],
            id: "patch-1".to_string(),
            restore_approval_id: None,
            run_id: run_id.to_string(),
            status: "proposed".to_string(),
        }
    }
}
