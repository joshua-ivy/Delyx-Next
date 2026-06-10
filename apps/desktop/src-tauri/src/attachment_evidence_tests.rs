#[cfg(test)]
mod tests {
    use crate::attachment::{
        accept_proposal, AttachmentKind, AttachmentProposal, AttachmentProposalStatus,
        AttachmentRisk, AttachmentScope, AttachmentSourceKind,
    };
    use crate::attachment_evidence::{
        evidence_from_pack, list_evidence_for_thread, save_evidence_batch_to_path,
    };
    use crate::attachment_persistence::save_record_to_path;
    use crate::context_pack::{ContextPack, ContextPackItem};
    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn pack() -> ContextPack {
        ContextPack {
            id: "pack-1".to_string(),
            project_id: "p1".to_string(),
            thread_id: "t1".to_string(),
            run_id: Some("run-1".to_string()),
            strategy: "direct_excerpt".to_string(),
            budget_tokens: 1000,
            used_tokens: 20,
            status: "ready".to_string(),
            items: vec![
                ContextPackItem {
                    attachment_id: Some("attach-1".to_string()),
                    evidence_record_id: None,
                    locator: "main.rs#L1-L80".to_string(),
                    text: "fn main() {}".to_string(),
                    token_estimate: 10,
                    inclusion_reason: "within budget".to_string(),
                },
                ContextPackItem {
                    attachment_id: None, // not from an attachment → no evidence
                    evidence_record_id: None,
                    locator: "manual".to_string(),
                    text: "note".to_string(),
                    token_estimate: 10,
                    inclusion_reason: "pinned".to_string(),
                },
            ],
            created_at: String::new(),
            excluded_count: 0,
        }
    }

    #[test]
    fn evidence_locator_matches_chunk_and_skips_non_attachment_items() {
        let records = evidence_from_pack(&pack(), "1700000000000");
        assert_eq!(records.len(), 1);
        let evidence = &records[0];
        assert_eq!(evidence.locator, "main.rs#L1-L80");
        assert_eq!(evidence.attachment_id, "attach-1");
        assert_eq!(evidence.run_id.as_deref(), Some("run-1"));
        assert!(evidence.content_hash.is_some());
        assert!(evidence.excerpt.contains("fn main"));
    }

    #[test]
    fn evidence_persists_and_lists_by_thread() {
        let path = temp_path("attach-evidence");
        // Evidence FK to a real attachment row.
        let mut source = AttachmentProposal {
            id: "attach-1".to_string(),
            project_id: "p1".to_string(),
            thread_id: Some("t1".to_string()),
            source_kind: AttachmentSourceKind::LocalFile,
            detected_kind: AttachmentKind::Code,
            display_name: "main.rs".to_string(),
            source_locator: "main.rs".to_string(),
            proposed_scope: AttachmentScope::default(),
            estimated_bytes: Some(12),
            estimated_file_count: None,
            requires_approval: false,
            approval_reason: None,
            risk: AttachmentRisk::Low,
            status: AttachmentProposalStatus::Pending,
            approval_id: None,
            created_at: String::new(),
            updated_at: String::new(),
        };
        source.status = AttachmentProposalStatus::Pending;
        let record = accept_proposal(&source, None).unwrap();
        save_record_to_path(&path, &record).unwrap();

        let records = evidence_from_pack(&pack(), "1700000000000");
        save_evidence_batch_to_path(&path, &records).unwrap();
        let listed = list_evidence_for_thread(&path, "p1", "t1").unwrap();
        assert_eq!(listed.len(), 1);
        assert_eq!(listed[0].locator, "main.rs#L1-L80");
        let _ = std::fs::remove_file(path);
    }

    fn temp_path(name: &str) -> PathBuf {
        let stamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        std::env::temp_dir().join(format!("delyx-next-{name}-{stamp}.sqlite3"))
    }
}
