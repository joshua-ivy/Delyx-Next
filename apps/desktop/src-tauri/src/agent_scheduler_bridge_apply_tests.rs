#[cfg(test)]
mod tests {
    use crate::agent_scheduler_bridge::{schedule_next_record, AgentScheduleRequest};
    use crate::approval::{ApprovalEngine, ProposalInput, RiskLevel, RiskyAction};
    use crate::patch_bridge::{DiffLineView, PatchBridgeStore, PatchFileView, PatchProposalView};
    use crate::review_bridge::ReviewBridgeStore;
    use crate::test_runner_bridge::TestRunnerBridgeStore;
    use crate::thread_run_bridge::ThreadRunStore;

    #[test]
    fn scheduler_bridge_maps_patch_apply_approval_request_for_ui() {
        let mut thread_store = ThreadRunStore::default();
        let run = thread_store.ledger.create_run("thread-1").unwrap();
        let mut patches = PatchBridgeStore::default();
        patches.records.push(patch(&run.id, "approval-draft"));

        let view = schedule_next_record(
            thread_store.ledger.get_run(&run.id).unwrap(),
            &ApprovalEngine::new(),
            &patches,
            &TestRunnerBridgeStore::default(),
            &ReviewBridgeStore::default(),
            &request(&run.id),
        );

        assert_eq!(view.kind, "request_patch_apply_approval");
        assert_eq!(view.proposal_id.as_deref(), Some("patch-1"));
        assert!(view.message.contains("needs apply approval"));
    }

    #[test]
    fn scheduler_bridge_maps_patch_apply_decision_for_ui() {
        let mut thread_store = ThreadRunStore::default();
        let run = thread_store.ledger.create_run("thread-1").unwrap();
        let mut approvals = ApprovalEngine::new();
        let draft = seed_approval(&mut approvals, &run.id, true);
        let apply = seed_apply_approval(&mut approvals, &run.id, "patch-1");
        let mut patches = PatchBridgeStore::default();
        patches.records.push(patch(&run.id, &draft.id));

        let view = schedule_next_record(
            thread_store.ledger.get_run(&run.id).unwrap(),
            &approvals,
            &patches,
            &TestRunnerBridgeStore::default(),
            &ReviewBridgeStore::default(),
            &AgentScheduleRequest {
                patch_apply_approval_id: Some(apply.id.clone()),
                ..request(&run.id)
            },
        );

        assert_eq!(view.kind, "run_patch_apply");
        assert_eq!(view.approval_ids, vec![apply.id]);
        assert_eq!(view.proposal_id.as_deref(), Some("patch-1"));
        assert!(view.message.contains("has apply approval"));
    }

    fn seed_approval(
        approvals: &mut ApprovalEngine,
        run_id: &str,
        approve: bool,
    ) -> crate::approval::ActionProposal {
        let proposal = approvals.propose(proposal_input(run_id, "scheduler-bridge-node"));
        if approve {
            approvals.approve(&proposal.id, 2, "approved").unwrap();
        }
        proposal
    }

    fn seed_apply_approval(
        approvals: &mut ApprovalEngine,
        run_id: &str,
        patch_id: &str,
    ) -> crate::approval::ActionProposal {
        let node = format!("{run_id}-patch-apply-{patch_id}");
        let proposal = approvals.propose(proposal_input(run_id, &node));
        approvals.approve(&proposal.id, 2, "approved").unwrap();
        proposal
    }

    fn proposal_input(run_id: &str, node_id: &str) -> ProposalInput {
        ProposalInput {
            action: RiskyAction::FileWrite,
            expected_result: "Apply a proposed patch.".to_string(),
            expires_at: 10,
            node_id: node_id.to_string(),
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
}
