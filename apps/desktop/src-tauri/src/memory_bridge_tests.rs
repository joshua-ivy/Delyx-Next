#[cfg(test)]
mod tests {
    use crate::approval::{ApprovalEngine, ProposalInput, RiskLevel, RiskyAction};
    use crate::memory::MemoryStore;
    use crate::memory_bridge::{
        memory_snapshot_from_path, promote_memory_record, propose_memory_candidate_record,
        suppress_memory_candidate_record, suppress_memory_record, MemoryCandidateActionRequest,
        MemoryCandidateRequest, MemoryPromoteRequest, MemoryRecordActionRequest,
    };
    use std::fs;
    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn memory_bridge_promotes_approved_candidate_and_survives_reload() {
        let path = temp_path("memory-bridge-promote");
        let mut store = MemoryStore::new();
        let proposed =
            propose_memory_candidate_record(&mut store, candidate_request("style")).unwrap();
        let candidate_id = proposed.candidates[0].id.clone();
        let mut approvals = ApprovalEngine::new();
        let approval = approvals.propose(memory_save_input(&candidate_id));
        approvals
            .approve(&approval.id, 10, "approved in bridge test")
            .unwrap();

        let promoted = promote_memory_record(
            &mut store,
            &approvals,
            MemoryPromoteRequest {
                approval_id: approval.id,
                approved_at_ms: 10,
                candidate_id,
                source_run_status: "succeeded".to_string(),
            },
        )
        .unwrap();

        assert_eq!(promoted.candidates[0].status, "promoted");
        assert_eq!(promoted.records[0].key, "style");
        crate::memory_persistence::save_to_path(&store, &path).unwrap();
        let reloaded = memory_snapshot_from_path(&path).unwrap();
        assert_eq!(reloaded.candidates[0].status, "promoted");
        assert_eq!(reloaded.records[0].source_thread_id, "thread-1");
        let _ = fs::remove_file(path);
    }

    #[test]
    fn memory_bridge_rejects_unapproved_promotion_without_recording_memory() {
        let mut store = MemoryStore::new();
        let proposed =
            propose_memory_candidate_record(&mut store, candidate_request("style")).unwrap();
        let candidate_id = proposed.candidates[0].id.clone();
        let approvals = ApprovalEngine::new();

        let result = promote_memory_record(
            &mut store,
            &approvals,
            MemoryPromoteRequest {
                approval_id: "approval-1".to_string(),
                approved_at_ms: 10,
                candidate_id,
                source_run_status: "succeeded".to_string(),
            },
        );

        assert!(result.unwrap_err().contains("ProposalNotFound"));
        assert_eq!(
            store.candidates()[0].status,
            crate::memory::MemoryCandidateStatus::Pending
        );
        assert!(store.records().is_empty());
    }

    #[test]
    fn memory_bridge_suppresses_candidates_and_records() {
        let mut store = MemoryStore::new();
        let proposed = propose_memory_candidate_record(&mut store, candidate_request("tone"))
            .unwrap()
            .candidates[0]
            .clone();

        let suppressed = suppress_memory_candidate_record(
            &mut store,
            MemoryCandidateActionRequest {
                candidate_id: proposed.id,
            },
        )
        .unwrap();

        assert_eq!(suppressed.candidates[0].status, "suppressed");
        let promoted = promote_test_memory(&mut store, "style");
        let record_id = promoted.records[0].id.clone();
        let view =
            suppress_memory_record(&mut store, MemoryRecordActionRequest { record_id }).unwrap();
        assert!(view.records[0].suppressed);
    }

    fn promote_test_memory(
        store: &mut MemoryStore,
        key: &str,
    ) -> crate::memory_bridge::MemoryStateView {
        let proposed = propose_memory_candidate_record(store, candidate_request(key)).unwrap();
        let candidate_id = proposed
            .candidates
            .iter()
            .find(|candidate| candidate.key == key)
            .unwrap()
            .id
            .clone();
        let mut approvals = ApprovalEngine::new();
        let approval = approvals.propose(memory_save_input(&candidate_id));
        approvals
            .approve(&approval.id, 10, "approved in bridge test")
            .unwrap();
        promote_memory_record(
            store,
            &approvals,
            MemoryPromoteRequest {
                approval_id: approval.id,
                approved_at_ms: 10,
                candidate_id,
                source_run_status: "completed".to_string(),
            },
        )
        .unwrap()
    }

    fn candidate_request(key: &str) -> MemoryCandidateRequest {
        MemoryCandidateRequest {
            key: key.to_string(),
            scope: "project".to_string(),
            source_run_id: "run-1".to_string(),
            source_thread_id: "thread-1".to_string(),
            value: format!("Remember {key}."),
        }
    }

    fn memory_save_input(node_id: &str) -> ProposalInput {
        ProposalInput {
            action: RiskyAction::DurableMemorySave,
            expires_at: 30,
            expected_result: "Persist selected memory after review.".to_string(),
            node_id: node_id.to_string(),
            reason: "Deterministic memory bridge test.".to_string(),
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
