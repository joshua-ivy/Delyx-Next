#[cfg(test)]
mod tests {
    use crate::attachment::{
        accept_proposal, AttachmentKind, AttachmentParseStatus, AttachmentProposal,
        AttachmentProposalStatus, AttachmentRisk, AttachmentScope, AttachmentSourceKind,
    };
    use crate::attachment_parser::{parse_attachment_text, ParsedChunk};
    use crate::attachment_persistence::{
        list_chunks_from_path, list_proposals_from_path, list_records_from_path,
        load_proposal_from_path, save_chunks_to_path, save_proposal_to_path, save_record_to_path,
        set_proposal_status_to_path,
    };
    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn temp_path(name: &str) -> PathBuf {
        let stamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        std::env::temp_dir().join(format!("delyx-next-{name}-{stamp}.sqlite3"))
    }

    fn proposal(
        id: &str,
        thread: Option<&str>,
        status: AttachmentProposalStatus,
    ) -> AttachmentProposal {
        AttachmentProposal {
            id: id.to_string(),
            project_id: "project-1".to_string(),
            thread_id: thread.map(|value| value.to_string()),
            source_kind: AttachmentSourceKind::LocalFile,
            detected_kind: AttachmentKind::Code,
            display_name: "main.rs".to_string(),
            source_locator: "/code/main.rs".to_string(),
            proposed_scope: AttachmentScope::default(),
            estimated_bytes: Some(1_200),
            estimated_file_count: None,
            requires_approval: false,
            approval_reason: None,
            risk: AttachmentRisk::Low,
            status,
            approval_id: None,
            created_at: String::new(),
            updated_at: String::new(),
        }
    }

    #[test]
    fn proposal_survives_sqlite_reload() {
        let path = temp_path("attach-roundtrip");
        let saved = save_proposal_to_path(
            &path,
            &proposal(
                "attach-1",
                Some("thread-1"),
                AttachmentProposalStatus::NeedsApproval,
            ),
        )
        .unwrap();
        assert!(!saved.created_at.is_empty());

        let loaded = load_proposal_from_path(&path, "attach-1").unwrap().unwrap();
        assert_eq!(loaded.status, AttachmentProposalStatus::NeedsApproval);
        assert_eq!(loaded.detected_kind, AttachmentKind::Code);
        assert_eq!(loaded.estimated_bytes, Some(1_200));

        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn denied_and_expired_proposals_stay_visible_in_snapshot() {
        let path = temp_path("attach-visible");
        save_proposal_to_path(
            &path,
            &proposal(
                "attach-keep",
                Some("thread-1"),
                AttachmentProposalStatus::Pending,
            ),
        )
        .unwrap();
        let denied = save_proposal_to_path(
            &path,
            &proposal(
                "attach-denied",
                Some("thread-1"),
                AttachmentProposalStatus::NeedsApproval,
            ),
        )
        .unwrap();
        set_proposal_status_to_path(&path, &denied.id, AttachmentProposalStatus::Denied, None)
            .unwrap();

        let listed = list_proposals_from_path(&path, "project-1", Some("thread-1")).unwrap();
        assert_eq!(listed.len(), 2);
        assert!(listed
            .iter()
            .any(|p| p.id == "attach-denied" && p.status == AttachmentProposalStatus::Denied));

        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn snapshot_includes_project_wide_proposals_for_a_thread() {
        let path = temp_path("attach-scope");
        // Project-wide proposal (no thread).
        save_proposal_to_path(
            &path,
            &proposal("attach-proj", None, AttachmentProposalStatus::Pending),
        )
        .unwrap();
        // Another thread's proposal must NOT show up.
        save_proposal_to_path(
            &path,
            &proposal(
                "attach-other",
                Some("thread-2"),
                AttachmentProposalStatus::Pending,
            ),
        )
        .unwrap();
        save_proposal_to_path(
            &path,
            &proposal(
                "attach-mine",
                Some("thread-1"),
                AttachmentProposalStatus::Pending,
            ),
        )
        .unwrap();

        let listed = list_proposals_from_path(&path, "project-1", Some("thread-1")).unwrap();
        let ids: Vec<&str> = listed.iter().map(|p| p.id.as_str()).collect();
        assert!(ids.contains(&"attach-proj"));
        assert!(ids.contains(&"attach-mine"));
        assert!(!ids.contains(&"attach-other"));

        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn accepted_proposal_becomes_a_record_in_not_started_state() {
        let path = temp_path("attach-record");
        let source = proposal(
            "attach-1",
            Some("thread-1"),
            AttachmentProposalStatus::NeedsApproval,
        );
        let record = accept_proposal(&source, Some("approval-7")).unwrap();
        let saved = save_record_to_path(&path, &record).unwrap();
        assert!(!saved.created_at.is_empty());
        assert_eq!(saved.parse_status, AttachmentParseStatus::NotStarted);
        assert_eq!(saved.approval_id.as_deref(), Some("approval-7"));

        let listed = list_records_from_path(&path, "project-1", Some("thread-1")).unwrap();
        assert_eq!(listed.len(), 1);
        assert_eq!(listed[0].display_name, "main.rs");

        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn parsed_chunks_persist_and_replace_on_reparse() {
        let path = temp_path("attach-chunks");
        // Chunks FK to a real attachment row, so create one first.
        let record = accept_proposal(
            &proposal(
                "attach-1",
                Some("thread-1"),
                AttachmentProposalStatus::Pending,
            ),
            Some("approval-1"),
        )
        .unwrap();
        save_record_to_path(&path, &record).unwrap();
        let first: Vec<ParsedChunk> =
            parse_attachment_text("main.rs", AttachmentKind::Code, "let a = 1;\nlet b = 2;").chunks;
        save_chunks_to_path(&path, "attach-1", &first).unwrap();
        let listed = list_chunks_from_path(&path, "attach-1").unwrap();
        assert_eq!(listed.len(), 1);
        assert_eq!(listed[0].locator, "main.rs#L1-L2");

        // Re-parse with different content replaces the old chunk set.
        let second = parse_attachment_text("main.rs", AttachmentKind::Code, "x\ny\nz").chunks;
        save_chunks_to_path(&path, "attach-1", &second).unwrap();
        let reloaded = list_chunks_from_path(&path, "attach-1").unwrap();
        assert_eq!(reloaded.len(), 1);
        assert_eq!(reloaded[0].locator, "main.rs#L1-L3");

        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn re_proposing_same_id_updates_in_place() {
        let path = temp_path("attach-upsert");
        save_proposal_to_path(
            &path,
            &proposal(
                "attach-1",
                Some("thread-1"),
                AttachmentProposalStatus::Pending,
            ),
        )
        .unwrap();
        let mut second = proposal(
            "attach-1",
            Some("thread-1"),
            AttachmentProposalStatus::NeedsApproval,
        );
        second.display_name = "renamed.rs".to_string();
        save_proposal_to_path(&path, &second).unwrap();

        let listed = list_proposals_from_path(&path, "project-1", Some("thread-1")).unwrap();
        assert_eq!(listed.len(), 1);
        assert_eq!(listed[0].display_name, "renamed.rs");
        assert_eq!(listed[0].status, AttachmentProposalStatus::NeedsApproval);

        let _ = std::fs::remove_file(path);
    }
}
