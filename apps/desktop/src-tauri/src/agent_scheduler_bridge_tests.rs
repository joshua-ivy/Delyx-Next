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
    fn scheduler_bridge_maps_patch_apply_decision_for_ui() {
        let mut thread_store = ThreadRunStore::default();
        let run = thread_store.ledger.create_run("thread-1").unwrap();
        let mut approvals = ApprovalEngine::new();
        let proposal = seed_approval(&mut approvals, &run.id, true);
        let mut patches = PatchBridgeStore::default();
        patches.records.push(patch(&run.id, &proposal.id));

        let view = schedule_next_record(
            thread_store.ledger.get_run(&run.id).unwrap(),
            &approvals,
            &patches,
            &TestRunnerBridgeStore::default(),
            &ReviewBridgeStore::default(),
            &request(&run.id),
        );

        assert_eq!(view.kind, "run_patch_apply");
        assert_eq!(view.proposal_id.as_deref(), Some("patch-1"));
        assert!(view.message.contains("ready to apply"));
    }

    #[test]
    fn scheduler_bridge_resumes_and_returns_visible_decision() {
        let mut thread_store = ThreadRunStore::default();
        let run = thread_store.ledger.create_run("thread-1").unwrap();
        let mut approvals = ApprovalEngine::new();
        let proposal = seed_approval(&mut approvals, &run.id, true);
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
        let proposal = seed_approval(&mut approvals, &run.id, true);
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

    fn seed_approval(
        approvals: &mut ApprovalEngine,
        run_id: &str,
        approve: bool,
    ) -> crate::approval::ActionProposal {
        let proposal = approvals.propose(ProposalInput {
            action: RiskyAction::FileWrite,
            expected_result: "Apply a proposed patch.".to_string(),
            expires_at: 10,
            node_id: "scheduler-bridge-node".to_string(),
            reason: "Scheduler bridge test.".to_string(),
            risk: RiskLevel::Medium,
            rollback_plan: "Use patch checkpoint receipts.".to_string(),
            run_id: run_id.to_string(),
            scope: "One patch.".to_string(),
        });
        if approve {
            approvals.approve(&proposal.id, 2, "approved").unwrap();
        }
        proposal
    }

    fn request(run_id: &str) -> AgentScheduleRequest {
        AgentScheduleRequest {
            has_supported_test_command: true,
            now_ms: 3,
            run_id: run_id.to_string(),
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
}
