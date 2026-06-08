#[cfg(test)]
mod tests {
    use crate::agent_run::AgentRunLedger;
    use crate::agent_scheduler::{schedule_next, AgentScheduleDecision, AgentSchedulerContext};
    use crate::approval::{ApprovalEngine, ProposalInput, RiskLevel, RiskyAction};
    use crate::patch_bridge::PatchBridgeStore;
    use crate::review_bridge::{ReviewBridgeStore, ReviewFindingView, ReviewReportView};
    use crate::test_runner_bridge::TestRunnerBridgeStore;

    #[test]
    fn scheduler_blocks_pending_review_findings_before_final_support() {
        let mut ledger = AgentRunLedger::new();
        let run = ledger.create_run("thread-1").unwrap();
        let mut reviews = ReviewBridgeStore::default();
        let stores = Stores::default();
        reviews.reports.push(review_report(&run.id, "pending"));

        let decision = schedule_next(stores.context(&run, &reviews));

        assert!(matches!(decision, AgentScheduleDecision::Blocked { .. }));
    }

    #[test]
    fn scheduler_reports_requested_repair_from_review_finding() {
        let mut ledger = AgentRunLedger::new();
        let run = ledger.create_run("thread-1").unwrap();
        let mut reviews = ReviewBridgeStore::default();
        let stores = Stores::default();
        reviews
            .reports
            .push(review_report(&run.id, "revise_requested"));

        let decision = schedule_next(stores.context(&run, &reviews));

        assert_eq!(
            decision,
            AgentScheduleDecision::RepairRequested {
                finding_id: "finding-1".to_string(),
                review_report_id: "review-1".to_string()
            }
        );
    }

    #[test]
    fn scheduler_runs_patch_draft_for_approved_repair_finding() {
        let mut ledger = AgentRunLedger::new();
        let run = ledger.create_run("thread-1").unwrap();
        let mut reviews = ReviewBridgeStore::default();
        let mut stores = Stores::default();
        let proposal = stores.approvals.propose(proposal_input(&run.id));
        stores
            .approvals
            .approve(&proposal.id, 2, "approved")
            .unwrap();
        reviews
            .reports
            .push(review_report(&run.id, "revise_requested"));

        let decision = schedule_next(AgentSchedulerContext {
            patch_draft_approval_id: Some(&proposal.id),
            ..stores.context(&run, &reviews)
        });

        assert_eq!(
            decision,
            AgentScheduleDecision::RunPatchDraft {
                approval_id: proposal.id
            }
        );
    }

    #[derive(Default)]
    struct Stores {
        approvals: ApprovalEngine,
        patches: PatchBridgeStore,
        tests: TestRunnerBridgeStore,
    }

    impl Stores {
        fn context<'a>(
            &'a self,
            run: &'a crate::agent_run::AgentRun,
            reviews: &'a ReviewBridgeStore,
        ) -> AgentSchedulerContext<'a> {
            AgentSchedulerContext {
                approvals: &self.approvals,
                has_supported_test_command: false,
                now_ms: 3,
                patch_apply_approval_id: None,
                patch_draft_approval_id: None,
                patches: &self.patches,
                reviews,
                run,
                test_approval_id: None,
                tests: &self.tests,
            }
        }
    }

    fn review_report(run_id: &str, decision: &str) -> ReviewReportView {
        ReviewReportView {
            decision: decision.to_string(),
            evidence_summary: "Stored review receipt.".to_string(),
            findings: vec![ReviewFindingView {
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

    fn proposal_input(run_id: &str) -> ProposalInput {
        ProposalInput {
            action: RiskyAction::FileWrite,
            expected_result: "Draft a repair patch proposal.".to_string(),
            expires_at: 10,
            node_id: "repair-node".to_string(),
            reason: "Repair review finding.".to_string(),
            risk: RiskLevel::High,
            rollback_plan: "PatchDraft does not write files.".to_string(),
            run_id: run_id.to_string(),
            scope: "Repair one file.".to_string(),
        }
    }
}
