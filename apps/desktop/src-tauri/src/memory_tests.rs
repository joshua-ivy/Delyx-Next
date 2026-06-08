#[cfg(test)]
mod tests {
    use crate::approval::{ApprovalEngine, ApprovalError, ProposalInput, RiskLevel, RiskyAction};
    use crate::memory::{
        MemoryCandidateInput, MemoryCandidateStatus, MemoryError, MemoryScope, MemoryStore,
        SourceRunStatus,
    };

    #[test]
    fn memory_candidate_requires_approval_before_promotion() {
        let mut store = MemoryStore::new();
        let candidate = store.propose_candidate(candidate_input("style", "Prefer small files."));
        let mut approvals = ApprovalEngine::new();
        let approval = approvals.propose(memory_save_input(&candidate.id));

        let result = store.promote_approved(
            &candidate.id,
            &approval.id,
            10,
            &approvals,
            SourceRunStatus::Completed,
        );

        assert_eq!(
            result.unwrap_err(),
            MemoryError::Approval(ApprovalError::NotApproved)
        );
    }

    #[test]
    fn failed_run_cannot_auto_promote_memory() {
        let mut store = MemoryStore::new();
        let candidate = store.propose_candidate(candidate_input("style", "Prefer small files."));
        let mut approvals = ApprovalEngine::new();
        let approval = approvals.propose(memory_save_input(&candidate.id));
        approvals
            .approve(&approval.id, 10, "approved in test")
            .unwrap();

        let result = store.promote_approved(
            &candidate.id,
            &approval.id,
            10,
            &approvals,
            SourceRunStatus::Failed,
        );

        assert_eq!(result.unwrap_err(), MemoryError::FailedRunCannotPromote);
    }

    #[test]
    fn memory_promotion_requires_memory_save_approval_action() {
        let mut store = MemoryStore::new();
        let candidate = store.propose_candidate(candidate_input("style", "Prefer small files."));
        let mut approvals = ApprovalEngine::new();
        let approval = approvals.propose(ProposalInput {
            action: RiskyAction::FileWrite,
            ..memory_save_input(&candidate.id)
        });
        approvals
            .approve(&approval.id, 10, "approved in test")
            .unwrap();

        let result = store.promote_approved(
            &candidate.id,
            &approval.id,
            10,
            &approvals,
            SourceRunStatus::Completed,
        );

        assert_eq!(
            result.unwrap_err(),
            MemoryError::Approval(ApprovalError::ActionMismatch {
                expected: RiskyAction::DurableMemorySave,
                actual: RiskyAction::FileWrite,
            })
        );
        assert!(store.records().is_empty());
    }

    #[test]
    fn memory_promotion_requires_same_run_approval() {
        let mut store = MemoryStore::new();
        let candidate = store.propose_candidate(candidate_input("style", "Prefer small files."));
        let mut approvals = ApprovalEngine::new();
        let approval = approvals.propose(ProposalInput {
            run_id: "run-2".to_string(),
            ..memory_save_input(&candidate.id)
        });
        approvals
            .approve(&approval.id, 10, "approved in test")
            .unwrap();

        let result = store.promote_approved(
            &candidate.id,
            &approval.id,
            10,
            &approvals,
            SourceRunStatus::Completed,
        );

        assert_eq!(
            result.unwrap_err(),
            MemoryError::Approval(ApprovalError::RunMismatch {
                expected: "run-1".to_string(),
                actual: "run-2".to_string(),
            })
        );
        assert!(store.records().is_empty());
    }

    #[test]
    fn memory_promotion_requires_matching_candidate_approval() {
        let mut store = MemoryStore::new();
        let first = store.propose_candidate(candidate_input("style", "Prefer small files."));
        let second =
            store.propose_candidate(candidate_input("style", "Prefer files under 300 lines."));
        let mut approvals = ApprovalEngine::new();
        let approval = approvals.propose(memory_save_input(&first.id));
        approvals
            .approve(&approval.id, 10, "approved in test")
            .unwrap();

        let result = store.promote_approved(
            &second.id,
            &approval.id,
            10,
            &approvals,
            SourceRunStatus::Completed,
        );

        assert_eq!(
            result.unwrap_err(),
            MemoryError::Approval(ApprovalError::NodeMismatch {
                expected: second.id,
                actual: first.id,
            })
        );
        assert!(store.records().is_empty());
    }

    #[test]
    fn user_can_suppress_memory_candidate() {
        let mut store = MemoryStore::new();
        let candidate = store.propose_candidate(candidate_input("style", "Prefer small files."));

        store.suppress_candidate(&candidate.id).unwrap();

        assert_eq!(
            store.candidates()[0].status,
            MemoryCandidateStatus::Suppressed
        );
    }

    #[test]
    fn promoted_candidate_cannot_be_suppressed_as_pending() {
        let mut store = MemoryStore::new();
        let candidate = store.propose_candidate(candidate_input("style", "Prefer small files."));
        let mut approvals = ApprovalEngine::new();
        let approval = approvals.propose(memory_save_input(&candidate.id));
        approvals
            .approve(&approval.id, 10, "approved in test")
            .unwrap();

        store
            .promote_approved(
                &candidate.id,
                &approval.id,
                10,
                &approvals,
                SourceRunStatus::Completed,
            )
            .unwrap();

        assert_eq!(
            store.suppress_candidate(&candidate.id).unwrap_err(),
            MemoryError::NotPending
        );
        assert_eq!(
            store.candidates()[0].status,
            MemoryCandidateStatus::Promoted
        );
    }

    #[test]
    fn promoted_memory_shows_source_run_and_thread() {
        let mut store = MemoryStore::new();
        let candidate = store.propose_candidate(candidate_input("style", "Prefer small files."));
        let mut approvals = ApprovalEngine::new();
        let approval = approvals.propose(memory_save_input(&candidate.id));
        approvals
            .approve(&approval.id, 10, "approved in test")
            .unwrap();

        let record = store
            .promote_approved(
                &candidate.id,
                &approval.id,
                10,
                &approvals,
                SourceRunStatus::Completed,
            )
            .unwrap();

        assert_eq!(record.source_run_id, "run-1");
        assert_eq!(record.source_thread_id, "thread-1");
        assert_eq!(
            store.candidates()[0].status,
            MemoryCandidateStatus::Promoted
        );
    }

    #[test]
    fn superseding_memory_suppresses_previous_record() {
        let mut store = MemoryStore::new();
        let first = store.propose_candidate(candidate_input("style", "Prefer small files."));
        let mut approvals = ApprovalEngine::new();
        let first_approval = approvals.propose(memory_save_input(&first.id));
        approvals
            .approve(&first_approval.id, 10, "approved in test")
            .unwrap();
        let first_record = store
            .promote_approved(
                &first.id,
                &first_approval.id,
                10,
                &approvals,
                SourceRunStatus::Completed,
            )
            .unwrap();
        let second =
            store.propose_candidate(candidate_input("style", "Prefer files under 300 lines."));
        let second_approval = approvals.propose(memory_save_input(&second.id));
        approvals
            .approve(&second_approval.id, 11, "approved in test")
            .unwrap();

        let second_record = store
            .promote_approved(
                &second.id,
                &second_approval.id,
                11,
                &approvals,
                SourceRunStatus::Completed,
            )
            .unwrap();

        assert_eq!(
            second_record.supersedes.as_deref(),
            Some(first_record.id.as_str())
        );
        assert!(store.records()[0].suppressed);
    }

    fn candidate_input(key: &str, value: &str) -> MemoryCandidateInput {
        MemoryCandidateInput {
            key: key.to_string(),
            scope: MemoryScope::Project,
            source_run_id: "run-1".to_string(),
            source_thread_id: "thread-1".to_string(),
            value: value.to_string(),
        }
    }

    fn memory_save_input(node_id: &str) -> ProposalInput {
        ProposalInput {
            action: RiskyAction::DurableMemorySave,
            expires_at: 30,
            expected_result: "Persist selected memory after review.".to_string(),
            node_id: node_id.to_string(),
            reason: "Deterministic memory governance test.".to_string(),
            risk: RiskLevel::Medium,
            rollback_plan: "Suppress or supersede the memory record.".to_string(),
            run_id: "run-1".to_string(),
            scope: "Save one reviewed project memory item.".to_string(),
        }
    }
}
