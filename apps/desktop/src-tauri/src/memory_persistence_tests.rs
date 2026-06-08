#[cfg(test)]
mod tests {
    use crate::approval::{ApprovalEngine, ProposalInput, RiskLevel, RiskyAction};
    use crate::memory::{
        MemoryCandidateInput, MemoryCandidateStatus, MemoryScope, MemoryStore, SourceRunStatus,
    };
    use crate::memory_persistence::{load_from_path, save_to_path};
    use std::fs;
    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn memory_store_survives_sqlite_reload() {
        let path = temp_path("memory");
        let mut store = MemoryStore::new();
        let promoted = store.propose_candidate(candidate_input("style", "Prefer small files."));
        let mut approvals = ApprovalEngine::new();
        let approval = approvals.propose(memory_save_input(&promoted.id));
        approvals
            .approve(&approval.id, 10, "approved in test")
            .unwrap();
        let record = store
            .promote_approved(
                &promoted.id,
                &approval.id,
                10,
                &approvals,
                SourceRunStatus::Completed,
            )
            .unwrap();
        store.suppress_memory(&record.id).unwrap();
        let suppressed = store.propose_candidate(candidate_input("tone", "Be direct."));
        store.suppress_candidate(&suppressed.id).unwrap();

        save_to_path(&store, &path).unwrap();
        let bytes = fs::read(&path).unwrap();
        assert!(bytes.starts_with(b"SQLite format 3"));

        let mut loaded = load_from_path(&path).unwrap();
        assert_eq!(
            loaded.candidates()[0].status,
            MemoryCandidateStatus::Promoted
        );
        assert_eq!(
            loaded.candidates()[1].status,
            MemoryCandidateStatus::Suppressed
        );
        assert!(loaded.records()[0].suppressed);
        assert_eq!(loaded.records()[0].source_thread_id, "thread-1");

        let next = loaded.propose_candidate(candidate_input("review", "Require test artifacts."));
        assert_eq!(next.id, "memory-candidate-3");
        let mut next_approvals = ApprovalEngine::new();
        let next_approval = next_approvals.propose(memory_save_input(&next.id));
        next_approvals
            .approve(&next_approval.id, 12, "approved in test")
            .unwrap();
        let next_record = loaded
            .promote_approved(
                &next.id,
                &next_approval.id,
                12,
                &next_approvals,
                SourceRunStatus::Completed,
            )
            .unwrap();
        assert_eq!(next_record.id, "memory-2");
        let _ = fs::remove_file(path);
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

    fn temp_path(name: &str) -> PathBuf {
        let stamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        std::env::temp_dir().join(format!("delyx-next-{name}-{stamp}.sqlite3"))
    }
}
