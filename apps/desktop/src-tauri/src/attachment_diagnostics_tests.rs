#[cfg(test)]
mod tests {
    use crate::attachment::{
        AttachmentIndexStatus, AttachmentKind, AttachmentParseStatus, AttachmentRecord,
        AttachmentSourceKind,
    };
    use crate::attachment_diagnostics::{attachment_report, redact};

    fn record(
        name: &str,
        parse: AttachmentParseStatus,
        approval: Option<&str>,
    ) -> AttachmentRecord {
        AttachmentRecord {
            id: "attach-1".to_string(),
            project_id: "p1".to_string(),
            thread_id: Some("t1".to_string()),
            message_id: None,
            run_id: None,
            source_kind: AttachmentSourceKind::LocalFile,
            detected_kind: AttachmentKind::Code,
            display_name: name.to_string(),
            original_locator: "C:/secret/path/with/token=abc/main.rs".to_string(),
            local_reference_path: None,
            content_hash: None,
            bytes: Some(1234),
            parse_status: parse,
            index_status: AttachmentIndexStatus::NotIndexed,
            approval_id: approval.map(|a| a.to_string()),
            created_at: String::new(),
            updated_at: String::new(),
        }
    }

    #[test]
    fn report_is_content_free_and_omits_raw_paths() {
        let records = vec![record(
            "main.rs",
            AttachmentParseStatus::Parsed,
            Some("approval-1"),
        )];
        let report = attachment_report(&records);
        let json = serde_json::to_string(&report).unwrap();
        // The raw original_locator (a path containing a secret-looking token) must
        // never appear in the report.
        assert!(!json.contains("C:/secret"));
        assert!(!json.contains("token=abc"));
        assert_eq!(report[0].has_approval, true);
        assert!(report[0].failure_hint.is_none());
    }

    #[test]
    fn secret_looking_display_names_are_redacted() {
        let report = attachment_report(&vec![record(
            "aws sk-LIVEKEY123 dump.txt",
            AttachmentParseStatus::Parsed,
            None,
        )]);
        assert_eq!(report[0].display_name, "[redacted]");
        let json = serde_json::to_string(&report).unwrap();
        assert!(!json.contains("sk-LIVEKEY123"));
    }

    #[test]
    fn failure_states_get_a_human_hint() {
        let failed = attachment_report(&vec![record("a.rs", AttachmentParseStatus::Failed, None)]);
        assert!(failed[0]
            .failure_hint
            .as_ref()
            .unwrap()
            .contains("Could not read"));
        let unsupported = attachment_report(&vec![record(
            "img.png",
            AttachmentParseStatus::Unsupported,
            None,
        )]);
        assert!(unsupported[0]
            .failure_hint
            .as_ref()
            .unwrap()
            .contains("not parsed"));
    }

    #[test]
    fn redact_passes_clean_values() {
        assert_eq!(redact("notes.md"), "notes.md");
        assert_eq!(redact("api_key dump"), "[redacted]");
    }
}
