#[cfg(test)]
mod tests {
    use crate::attachment::{
        accept_proposal, classify, infer_kind, stable_proposal_id, AttachmentKind,
        AttachmentProposal, AttachmentProposalStatus, AttachmentRisk, AttachmentScope,
        AttachmentSourceKind, ClassifyInput,
    };
    use crate::project::ApprovalPolicyRecord;

    fn proposal(requires_approval: bool, status: AttachmentProposalStatus) -> AttachmentProposal {
        AttachmentProposal {
            id: "attach-1".to_string(),
            project_id: "p1".to_string(),
            thread_id: Some("t1".to_string()),
            source_kind: AttachmentSourceKind::LocalFile,
            detected_kind: AttachmentKind::Code,
            display_name: "main.rs".to_string(),
            source_locator: "/code/main.rs".to_string(),
            proposed_scope: AttachmentScope::default(),
            estimated_bytes: Some(1_000),
            estimated_file_count: None,
            requires_approval,
            approval_reason: None,
            risk: AttachmentRisk::Low,
            status,
            approval_id: None,
            created_at: String::new(),
            updated_at: String::new(),
        }
    }

    fn policy() -> ApprovalPolicyRecord {
        ApprovalPolicyRecord::default()
    }

    fn input<'a>(
        source_kind: AttachmentSourceKind,
        detected_kind: AttachmentKind,
        policy: &'a ApprovalPolicyRecord,
    ) -> ClassifyInput<'a> {
        ClassifyInput {
            source_kind,
            detected_kind,
            estimated_bytes: None,
            estimated_file_count: None,
            in_read_scope: None,
            policy,
        }
    }

    #[test]
    fn small_in_scope_local_file_needs_no_approval() {
        let policy = policy();
        let mut request = input(
            AttachmentSourceKind::LocalFile,
            AttachmentKind::Code,
            &policy,
        );
        request.estimated_bytes = Some(4_000);
        request.in_read_scope = Some(true);
        let result = classify(request);
        assert!(!result.requires_approval);
        assert_eq!(result.risk, AttachmentRisk::Low);
    }

    #[test]
    fn folder_import_always_requires_approval() {
        let policy = policy();
        let result = classify(input(
            AttachmentSourceKind::LocalFolder,
            AttachmentKind::Folder,
            &policy,
        ));
        assert!(result.requires_approval);
        assert_eq!(result.risk, AttachmentRisk::High);
    }

    #[test]
    fn archive_requires_approval() {
        let policy = policy();
        let result = classify(input(
            AttachmentSourceKind::LocalFile,
            AttachmentKind::Archive,
            &policy,
        ));
        assert!(result.requires_approval);
        assert!(result.reason.unwrap().to_lowercase().contains("archive"));
    }

    #[test]
    fn external_sources_require_approval() {
        let policy = policy();
        for source in [
            AttachmentSourceKind::Url,
            AttachmentSourceKind::Connector,
            AttachmentSourceKind::McpResource,
        ] {
            let result = classify(input(source, AttachmentKind::Unknown, &policy));
            assert!(
                result.requires_approval,
                "{source:?} should require approval"
            );
        }
    }

    #[test]
    fn large_file_or_many_files_requires_approval() {
        let policy = policy();
        let mut big = input(
            AttachmentSourceKind::LocalFile,
            AttachmentKind::Text,
            &policy,
        );
        big.estimated_bytes = Some(policy.large_file_bytes + 1);
        big.in_read_scope = Some(true);
        assert!(classify(big).requires_approval);

        let mut many = input(
            AttachmentSourceKind::LocalFile,
            AttachmentKind::Text,
            &policy,
        );
        many.estimated_file_count = Some(policy.folder_file_count + 1);
        many.in_read_scope = Some(true);
        assert!(classify(many).requires_approval);
    }

    #[test]
    fn local_path_outside_read_scope_requires_approval() {
        let policy = policy();
        let mut request = input(
            AttachmentSourceKind::LocalFile,
            AttachmentKind::Code,
            &policy,
        );
        request.estimated_bytes = Some(1_000);
        request.in_read_scope = Some(false);
        let result = classify(request);
        assert!(result.requires_approval);
        assert!(result.reason.unwrap().to_lowercase().contains("scope"));
    }

    #[test]
    fn infer_kind_reads_extensions() {
        assert_eq!(infer_kind("a/b/main.rs"), AttachmentKind::Code);
        assert_eq!(infer_kind("notes.md"), AttachmentKind::Markdown);
        assert_eq!(infer_kind("spec.pdf"), AttachmentKind::Pdf);
        assert_eq!(infer_kind("photo.PNG"), AttachmentKind::Image);
        assert_eq!(infer_kind("bundle.zip"), AttachmentKind::Archive);
        assert_eq!(infer_kind("mystery"), AttachmentKind::Unknown);
    }

    #[test]
    fn safe_proposal_is_accepted_without_approval() {
        let record =
            accept_proposal(&proposal(false, AttachmentProposalStatus::Pending), None).unwrap();
        assert_eq!(record.id, "attach-1");
        assert!(record.approval_id.is_none());
    }

    #[test]
    fn risky_proposal_needs_an_approval_id() {
        let risky = proposal(true, AttachmentProposalStatus::NeedsApproval);
        assert!(accept_proposal(&risky, None).is_err());
        assert!(accept_proposal(&risky, Some("  ")).is_err());
        let record = accept_proposal(&risky, Some("approval-9")).unwrap();
        assert_eq!(record.approval_id.as_deref(), Some("approval-9"));
    }

    #[test]
    fn denied_or_expired_proposals_cannot_be_accepted() {
        assert!(accept_proposal(
            &proposal(false, AttachmentProposalStatus::Denied),
            Some("a")
        )
        .is_err());
        assert!(accept_proposal(
            &proposal(false, AttachmentProposalStatus::Expired),
            Some("a")
        )
        .is_err());
    }

    #[test]
    fn proposal_id_is_stable_and_distinguishes_inputs() {
        let a = stable_proposal_id("proj", Some("thread-1"), "/a/b.rs");
        let b = stable_proposal_id("proj", Some("thread-1"), "/a/b.rs");
        assert_eq!(a, b);
        assert_ne!(a, stable_proposal_id("proj", Some("thread-2"), "/a/b.rs"));
        assert_ne!(a, stable_proposal_id("proj", Some("thread-1"), "/a/c.rs"));
    }
}
