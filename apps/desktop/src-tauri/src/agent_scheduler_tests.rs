#[cfg(test)]
mod tests {
    use crate::agent_run::{AgentRunLedger, AgentRunStatus};
    use crate::agent_scheduler::{
        resume_waiting_run, schedule_next, AgentScheduleDecision, AgentSchedulerContext,
    };
    use crate::approval::{ApprovalEngine, ProposalInput, RiskLevel, RiskyAction};
    use crate::patch_bridge::{DiffLineView, PatchBridgeStore, PatchFileView, PatchProposalView};
    use crate::review_bridge::{ReviewBridgeStore, ReviewReportView};
    use crate::test_runner_bridge::{TestArtifactView, TestRunnerBridgeStore};

    #[test]
    fn scheduler_waits_for_pending_approval() {
        let mut ledger = AgentRunLedger::new();
        let run = ledger.create_run("thread-1").unwrap();
        let mut approvals = ApprovalEngine::new();
        let proposal = approvals.propose(proposal_input(&run.id, RiskyAction::FileWrite));
        ledger.wait_for_approval(&run.id, &proposal.id).unwrap();
        let decision = resume_waiting_run(&mut ledger, &approvals, &run.id, 2).unwrap();
        assert_eq!(
            decision,
            AgentScheduleDecision::WaitForApproval {
                approval_ids: vec![proposal.id]
            }
        );
        assert_eq!(
            ledger.get_run(&run.id).unwrap().status,
            AgentRunStatus::WaitingForApproval
        );
    }

    #[test]
    fn scheduler_resumes_single_ready_approval() {
        let mut ledger = AgentRunLedger::new();
        let run = ledger.create_run("thread-1").unwrap();
        let mut approvals = ApprovalEngine::new();
        let proposal = approvals.propose(proposal_input(&run.id, RiskyAction::FileWrite));
        ledger.wait_for_approval(&run.id, &proposal.id).unwrap();
        approvals.approve(&proposal.id, 2, "approved").unwrap();
        let decision = resume_waiting_run(&mut ledger, &approvals, &run.id, 3).unwrap();
        let run = ledger.get_run(&run.id).unwrap();
        assert_eq!(
            decision,
            AgentScheduleDecision::ResumeAfterApproval {
                approval_id: proposal.id
            }
        );
        assert_eq!(run.status, AgentRunStatus::Running);
        assert_eq!(run.events.last().unwrap().kind, "approval.resumed");
    }

    #[test]
    fn scheduler_selects_approved_patch_apply() {
        let mut ledger = AgentRunLedger::new();
        let run = ledger.create_run("thread-1").unwrap();
        let mut approvals = ApprovalEngine::new();
        let proposal = approvals.propose(proposal_input(&run.id, RiskyAction::FileWrite));
        approvals.approve(&proposal.id, 2, "approved").unwrap();
        let mut patches = PatchBridgeStore::default();
        patches
            .records
            .push(patch(&run.id, &proposal.id, "proposed"));

        let decision = schedule_next(context(
            &run,
            &approvals,
            &patches,
            &TestRunnerBridgeStore::default(),
            &ReviewBridgeStore::default(),
            false,
        ));

        assert_eq!(
            decision,
            AgentScheduleDecision::RunPatchApply {
                proposal_id: "patch-1".to_string()
            }
        );
    }

    #[test]
    fn scheduler_selects_patch_draft_from_approved_plan_hint() {
        let mut ledger = AgentRunLedger::new();
        let run = ledger.create_run("thread-1").unwrap();
        let mut approvals = ApprovalEngine::new();
        let proposal = approvals.propose(proposal_input(&run.id, RiskyAction::FileWrite));
        approvals.approve(&proposal.id, 2, "approved").unwrap();

        let decision = schedule_next(AgentSchedulerContext {
            patch_draft_approval_id: Some(&proposal.id),
            ..context(
                &run,
                &approvals,
                &PatchBridgeStore::default(),
                &TestRunnerBridgeStore::default(),
                &ReviewBridgeStore::default(),
                false,
            )
        });

        assert_eq!(
            decision,
            AgentScheduleDecision::RunPatchDraft {
                approval_id: proposal.id
            }
        );
    }

    #[test]
    fn scheduler_blocks_unbacked_patch_draft_hint() {
        let mut ledger = AgentRunLedger::new();
        let run = ledger.create_run("thread-1").unwrap();

        let decision = schedule_next(AgentSchedulerContext {
            patch_draft_approval_id: Some("missing-approval"),
            ..context(
                &run,
                &ApprovalEngine::new(),
                &PatchBridgeStore::default(),
                &TestRunnerBridgeStore::default(),
                &ReviewBridgeStore::default(),
                false,
            )
        });

        assert!(matches!(decision, AgentScheduleDecision::Blocked { .. }));
    }

    #[test]
    fn scheduler_requires_test_command_after_applied_patch() {
        let mut ledger = AgentRunLedger::new();
        let run = ledger.create_run("thread-1").unwrap();
        let approvals = ApprovalEngine::new();
        let mut patches = PatchBridgeStore::default();
        patches.records.push(patch(&run.id, "prop-1", "applied"));

        let missing = schedule_next(context(
            &run,
            &approvals,
            &patches,
            &TestRunnerBridgeStore::default(),
            &ReviewBridgeStore::default(),
            false,
        ));
        let ready = schedule_next(context(
            &run,
            &approvals,
            &patches,
            &TestRunnerBridgeStore::default(),
            &ReviewBridgeStore::default(),
            true,
        ));

        assert!(matches!(missing, AgentScheduleDecision::Blocked { .. }));
        assert!(matches!(ready, AgentScheduleDecision::RunTests { .. }));
    }

    #[test]
    fn scheduler_selects_review_from_persisted_artifacts() {
        let mut ledger = AgentRunLedger::new();
        let run = ledger.create_run("thread-1").unwrap();
        let mut patches = PatchBridgeStore::default();
        let mut tests = TestRunnerBridgeStore::default();
        patches.records.push(patch(&run.id, "prop-1", "applied"));
        tests.artifacts.push(test_artifact(&run.id));

        let decision = schedule_next(context(
            &run,
            &ApprovalEngine::new(),
            &patches,
            &tests,
            &ReviewBridgeStore::default(),
            true,
        ));

        assert_eq!(
            decision,
            AgentScheduleDecision::RunReview {
                patch_count: 1,
                test_count: 1
            }
        );
    }

    #[test]
    fn scheduler_reports_ready_for_final_support_after_review() {
        let mut ledger = AgentRunLedger::new();
        let run = ledger.create_run("thread-1").unwrap();
        let mut reviews = ReviewBridgeStore::default();
        reviews.reports.push(review_report(&run.id));

        let decision = schedule_next(context(
            &run,
            &ApprovalEngine::new(),
            &PatchBridgeStore::default(),
            &TestRunnerBridgeStore::default(),
            &reviews,
            false,
        ));

        assert_eq!(
            decision,
            AgentScheduleDecision::ReadyForFinalSupport {
                review_report_id: "review-1".to_string()
            }
        );
    }

    fn context<'a>(
        run: &'a crate::agent_run::AgentRun,
        approvals: &'a ApprovalEngine,
        patches: &'a PatchBridgeStore,
        tests: &'a TestRunnerBridgeStore,
        reviews: &'a ReviewBridgeStore,
        has_supported_test_command: bool,
    ) -> AgentSchedulerContext<'a> {
        AgentSchedulerContext {
            approvals,
            has_supported_test_command,
            now_ms: 3,
            patch_draft_approval_id: None,
            patches,
            reviews,
            run,
            tests,
        }
    }

    fn proposal_input(run_id: &str, action: RiskyAction) -> ProposalInput {
        ProposalInput {
            action,
            expected_result: "Execute next scheduled runtime step.".to_string(),
            expires_at: 10,
            node_id: "scheduler-node".to_string(),
            reason: "Scheduler test approval.".to_string(),
            risk: RiskLevel::Medium,
            rollback_plan: "No rollback needed for scheduler decision.".to_string(),
            run_id: run_id.to_string(),
            scope: "Scheduler test scope.".to_string(),
        }
    }

    fn patch(run_id: &str, approval_id: &str, status: &str) -> PatchProposalView {
        PatchProposalView {
            approval_id: approval_id.to_string(),
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
            status: status.to_string(),
        }
    }

    fn test_artifact(run_id: &str) -> TestArtifactView {
        TestArtifactView {
            approval_id: Some("prop-test".to_string()),
            command: "npm test".to_string(),
            completed_at: "2026-06-08T00:00:01Z".to_string(),
            cwd: "C:/project".to_string(),
            duration_ms: 1,
            exec_events: Vec::new(),
            exit_code: Some(0),
            failure_summary: None,
            id: "test-artifact-1".to_string(),
            output_truncated: false,
            parsed_failures: None,
            run_id: run_id.to_string(),
            started_at: "2026-06-08T00:00:00Z".to_string(),
            status: "passed".to_string(),
            stderr: String::new(),
            stdout: "ok".to_string(),
        }
    }

    fn review_report(run_id: &str) -> ReviewReportView {
        ReviewReportView {
            decision: "accept".to_string(),
            evidence_summary: "1 stored review receipt.".to_string(),
            findings: Vec::new(),
            id: "review-1".to_string(),
            mode: "summary".to_string(),
            risk_summary: "No findings.".to_string(),
            run_id: run_id.to_string(),
            test_summary: "Tests passed.".to_string(),
        }
    }
}
