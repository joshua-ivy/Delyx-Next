#[cfg(test)]
mod tests {
    use crate::agent_run::AgentRunStatus;
    use crate::agent_scheduler_bridge::{
        resume_waiting_run_record, schedule_next_record, AgentScheduleRequest,
    };
    use crate::approval::{ApprovalEngine, ProposalInput, RiskLevel, RiskyAction};
    use crate::patch_bridge::{DiffLineView, PatchBridgeStore, PatchFileView, PatchProposalView};
    use crate::review_bridge::ReviewBridgeStore;
    use crate::test_runner_bridge::TestRunnerBridgeStore;
    use crate::thread_run_bridge::ThreadRunStore;

    #[test]
    fn scheduler_bridge_resumes_and_returns_visible_decision() {
        let mut thread_store = ThreadRunStore::default();
        let run = thread_store.ledger.create_run("thread-1").unwrap();
        let mut approvals = ApprovalEngine::new();
        let proposal = seed_approval(&mut approvals, &run.id, true, RiskyAction::FileWrite);
        thread_store
            .ledger
            .wait_for_approval(&run.id, &proposal.id)
            .unwrap();

        let view = resume_waiting_run_record(
            &mut thread_store,
            &approvals,
            &PatchBridgeStore::default(),
            &TestRunnerBridgeStore::default(),
            &ReviewBridgeStore::default(),
            &request(&run.id),
        )
        .unwrap();
        let run = thread_store.ledger.get_run(&run.id).unwrap();

        assert_eq!(view.kind, "resume_after_approval");
        assert_eq!(view.approval_ids, vec![proposal.id]);
        assert_eq!(run.status, AgentRunStatus::Running);
    }

    #[test]
    fn scheduler_bridge_returns_post_resume_test_decision_when_ready() {
        let mut thread_store = ThreadRunStore::default();
        let run = thread_store.ledger.create_run("thread-1").unwrap();
        let mut approvals = ApprovalEngine::new();
        let proposal = seed_approval(&mut approvals, &run.id, true, RiskyAction::FileWrite);
        thread_store
            .ledger
            .wait_for_approval(&run.id, &proposal.id)
            .unwrap();
        let mut patches = PatchBridgeStore::default();
        patches.records.push(applied_patch(&run.id));

        let view = resume_waiting_run_record(
            &mut thread_store,
            &approvals,
            &patches,
            &TestRunnerBridgeStore::default(),
            &ReviewBridgeStore::default(),
            &request(&run.id),
        )
        .unwrap();
        let run = thread_store.ledger.get_run(&run.id).unwrap();

        assert_eq!(view.kind, "run_tests");
        assert!(view.message.contains("supported test command"));
        assert_eq!(run.status, AgentRunStatus::Running);
    }

    #[test]
    fn scheduler_bridge_maps_patch_draft_decision_for_ui() {
        let mut thread_store = ThreadRunStore::default();
        let run = thread_store.ledger.create_run("thread-1").unwrap();
        let mut approvals = ApprovalEngine::new();
        let proposal = seed_approval(&mut approvals, &run.id, true, RiskyAction::FileWrite);

        let view = schedule_next_record(
            thread_store.ledger.get_run(&run.id).unwrap(),
            &approvals,
            &PatchBridgeStore::default(),
            &TestRunnerBridgeStore::default(),
            &ReviewBridgeStore::default(),
            &AgentScheduleRequest {
                patch_draft_approval_id: Some(proposal.id.clone()),
                ..request(&run.id)
            },
        );

        assert_eq!(view.kind, "run_patch_draft");
        assert_eq!(view.approval_ids, vec![proposal.id]);
        assert!(view.message.contains("PatchDraftAgent"));
    }

    #[test]
    fn scheduler_bridge_attaches_verified_test_approval_for_ui() {
        let mut thread_store = ThreadRunStore::default();
        let run = thread_store.ledger.create_run("thread-1").unwrap();
        let mut approvals = ApprovalEngine::new();
        let proposal = seed_approval(&mut approvals, &run.id, true, RiskyAction::TerminalCommand);
        let mut patches = PatchBridgeStore::default();
        patches.records.push(applied_patch(&run.id));

        let view = schedule_next_record(
            thread_store.ledger.get_run(&run.id).unwrap(),
            &approvals,
            &patches,
            &TestRunnerBridgeStore::default(),
            &ReviewBridgeStore::default(),
            &AgentScheduleRequest {
                test_approval_id: Some(proposal.id.clone()),
                ..request(&run.id)
            },
        );

        assert_eq!(view.kind, "run_tests");
        assert_eq!(view.approval_ids, vec![proposal.id]);
        assert!(view.message.contains("Approved test command"));
    }

    #[test]
    fn scheduler_bridge_maps_repair_requested_decision_for_ui() {
        let mut thread_store = ThreadRunStore::default();
        let run = thread_store.ledger.create_run("thread-1").unwrap();
        let mut reviews = ReviewBridgeStore::default();
        reviews.reports.push(repair_review(&run.id));

        let view = schedule_next_record(
            thread_store.ledger.get_run(&run.id).unwrap(),
            &ApprovalEngine::new(),
            &PatchBridgeStore::default(),
            &TestRunnerBridgeStore::default(),
            &reviews,
            &request(&run.id),
        );

        assert_eq!(view.kind, "repair_requested");
        assert_eq!(view.review_report_id.as_deref(), Some("review-1"));
        assert_eq!(view.finding_id.as_deref(), Some("finding-1"));
    }

    #[test]
    fn scheduler_bridge_schedules_patch_draft_for_running_repair_approval() {
        let mut thread_store = ThreadRunStore::default();
        let run = thread_store.ledger.create_run("thread-1").unwrap();
        let mut approvals = ApprovalEngine::new();
        let proposal = seed_approval(&mut approvals, &run.id, true, RiskyAction::FileWrite);
        let mut reviews = ReviewBridgeStore::default();
        reviews.reports.push(repair_review(&run.id));

        let view = resume_waiting_run_record(
            &mut thread_store,
            &approvals,
            &PatchBridgeStore::default(),
            &TestRunnerBridgeStore::default(),
            &reviews,
            &AgentScheduleRequest {
                patch_draft_approval_id: Some(proposal.id.clone()),
                ..request(&run.id)
            },
        )
        .unwrap();

        assert_eq!(view.kind, "run_patch_draft");
        assert_eq!(view.approval_ids, vec![proposal.id]);
    }

    fn seed_approval(
        approvals: &mut ApprovalEngine,
        run_id: &str,
        approve: bool,
        action: RiskyAction,
    ) -> crate::approval::ActionProposal {
        let proposal = approvals.propose(proposal_input(run_id, action));
        if approve {
            approvals.approve(&proposal.id, 2, "approved").unwrap();
        }
        proposal
    }

    fn proposal_input(run_id: &str, action: RiskyAction) -> ProposalInput {
        ProposalInput {
            action,
            expected_result: "Apply a proposed patch.".to_string(),
            expires_at: 10,
            node_id: "scheduler-bridge-node".to_string(),
            reason: "Scheduler bridge test.".to_string(),
            risk: RiskLevel::Medium,
            rollback_plan: "Use patch checkpoint receipts.".to_string(),
            run_id: run_id.to_string(),
            scope: "One patch.".to_string(),
        }
    }

    fn request(run_id: &str) -> AgentScheduleRequest {
        AgentScheduleRequest {
            has_supported_test_command: true,
            now_ms: 3,
            patch_apply_approval_id: None,
            patch_draft_approval_id: None,
            run_id: run_id.to_string(),
            test_approval_id: None,
        }
    }

    fn patch(run_id: &str, approval_id: &str) -> PatchProposalView {
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
            status: "proposed".to_string(),
        }
    }

    fn applied_patch(run_id: &str) -> PatchProposalView {
        PatchProposalView {
            status: "applied".to_string(),
            ..patch(run_id, "approval-applied")
        }
    }

    fn repair_review(run_id: &str) -> crate::review_bridge::ReviewReportView {
        crate::review_bridge::ReviewReportView {
            decision: "revise_requested".to_string(),
            evidence_summary: "Stored review receipt.".to_string(),
            findings: vec![crate::review_bridge::ReviewFindingView {
                detail: "Runtime panic risk in new code.".to_string(),
                file_path: "src/main.rs".to_string(),
                hunk_label: "patch-1:0".to_string(),
                id: "finding-1".to_string(),
                priority: "p1".to_string(),
                risk_label: "panic".to_string(),
                suggested_fix: "Handle the None/Err case explicitly.".to_string(),
                title: "Added unwrap can panic".to_string(),
            }],
            id: "review-1".to_string(),
            mode: "read_only".to_string(),
            risk_summary: "1 prioritized finding.".to_string(),
            run_id: run_id.to_string(),
            test_summary: "Tests passed.".to_string(),
        }
    }
}
