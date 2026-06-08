#[cfg(test)]
mod tests {
    use crate::approval::{ApprovalEngine, ProposalInput, RiskLevel, RiskyAction};
    use crate::release::export_support_bundle;
    use crate::release_bridge::ReleaseBridgeStore;
    use crate::release_support_export::{
        export_support_bundle_file_record, SupportBundleFileExportRequest,
    };
    use std::fs;
    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn approved_support_bundle_file_export_writes_json_and_receipt() {
        let root = temp_dir("support-export-root");
        fs::create_dir_all(&root).unwrap();
        let output = root.join("support-bundle.json");
        let mut store = store_with_bundle();
        let (approvals, approval_id) = approved_file_write("run-1");

        let view = export_support_bundle_file_record(
            &mut store,
            &approvals,
            request("run-1", &approval_id, &output, vec![root.clone()]),
        )
        .unwrap();

        let written = fs::read_to_string(&output).unwrap();
        assert!(written.contains("\"secret_policy\""));
        assert!(!written.contains("sk-test"));
        assert_eq!(view.support_bundle.file_export.status, "exported");
        assert_eq!(
            view.support_bundle.file_export.approval_id.as_deref(),
            Some(approval_id.as_str())
        );
        assert!(
            store
                .support_bundle_file_export
                .as_ref()
                .unwrap()
                .bytes_written
                > 0
        );
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn support_bundle_file_export_requires_approved_write() {
        let root = temp_dir("support-export-pending");
        fs::create_dir_all(&root).unwrap();
        let output = root.join("support-bundle.json");
        let mut store = store_with_bundle();
        let (approvals, approval_id) = pending_file_write("run-1");

        let error = export_support_bundle_file_record(
            &mut store,
            &approvals,
            request("run-1", &approval_id, &output, vec![root.clone()]),
        )
        .unwrap_err();

        assert_eq!(
            error,
            "Support bundle file export approval blocked: NotApproved"
        );
        assert!(!output.exists());
        assert!(store.support_bundle_file_export.is_none());
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn support_bundle_file_export_stays_inside_approved_roots() {
        let root = temp_dir("support-export-approved");
        let outside = temp_dir("support-export-outside");
        fs::create_dir_all(&root).unwrap();
        fs::create_dir_all(&outside).unwrap();
        let output = outside.join("support-bundle.json");
        let mut store = store_with_bundle();
        let (approvals, approval_id) = approved_file_write("run-1");

        let error = export_support_bundle_file_record(
            &mut store,
            &approvals,
            request("run-1", &approval_id, &output, vec![root.clone()]),
        )
        .unwrap_err();

        assert_eq!(
            error,
            "Support bundle file export path must stay inside an approved root."
        );
        assert!(!output.exists());
        let _ = fs::remove_dir_all(root);
        let _ = fs::remove_dir_all(outside);
    }

    #[test]
    fn support_bundle_file_export_receipt_survives_sqlite_reload() {
        let root = temp_dir("support-export-persist-root");
        fs::create_dir_all(&root).unwrap();
        let output = root.join("support-bundle.json");
        let db = temp_file("support-export");
        let mut store = store_with_bundle();
        let (approvals, approval_id) = approved_file_write("run-1");

        export_support_bundle_file_record(
            &mut store,
            &approvals,
            request("run-1", &approval_id, &output, vec![root.clone()]),
        )
        .unwrap();
        crate::release_file_export_persistence::save_file_export_to_path(
            store.support_bundle_file_export.as_ref().unwrap(),
            &db,
        )
        .unwrap();

        let loaded = crate::release_file_export_persistence::load_file_export_from_path(&db)
            .unwrap()
            .unwrap();
        assert_eq!(loaded.approval_id, approval_id);
        assert_eq!(loaded.path, output.display().to_string());
        assert!(loaded.bytes_written > 0);
        let _ = fs::remove_file(db);
        let _ = fs::remove_dir_all(root);
    }

    fn store_with_bundle() -> ReleaseBridgeStore {
        let profile = crate::release::default_release_profile();
        ReleaseBridgeStore {
            smoke: None,
            support_bundle: Some(export_support_bundle(
                &profile,
                vec![("OPENAI_API_KEY", "sk-test"), ("workspace", "C:/work")],
                vec![("runtime", "ready")],
                42,
            )),
            support_bundle_file_export: None,
            profile,
        }
    }

    fn approved_file_write(run_id: &str) -> (ApprovalEngine, String) {
        let mut engine = ApprovalEngine::new();
        let proposal = engine.propose(proposal_input(run_id));
        engine.approve(&proposal.id, 1, "test approval").unwrap();
        (engine, proposal.id)
    }

    fn pending_file_write(run_id: &str) -> (ApprovalEngine, String) {
        let mut engine = ApprovalEngine::new();
        let proposal = engine.propose(proposal_input(run_id));
        (engine, proposal.id)
    }

    fn proposal_input(run_id: &str) -> ProposalInput {
        ProposalInput {
            action: RiskyAction::FileWrite,
            expected_result: "Write a redacted support bundle JSON file.".to_string(),
            expires_at: 10_000,
            node_id: "release-support-file-export".to_string(),
            reason: "User requested a support bundle file export.".to_string(),
            risk: RiskLevel::Medium,
            rollback_plan: "Delete the exported support bundle file.".to_string(),
            run_id: run_id.to_string(),
            scope: "One support bundle JSON path inside the approved root.".to_string(),
        }
    }

    fn request(
        run_id: &str,
        approval_id: &str,
        output: &std::path::Path,
        roots: Vec<PathBuf>,
    ) -> SupportBundleFileExportRequest {
        SupportBundleFileExportRequest {
            approval_id: approval_id.to_string(),
            approved_roots: roots
                .into_iter()
                .map(|root| root.display().to_string())
                .collect(),
            created_at_ms: 2,
            exported_at: "2026-06-08T00:00:00.000Z".to_string(),
            output_path: output.display().to_string(),
            run_id: run_id.to_string(),
        }
    }

    fn temp_dir(name: &str) -> PathBuf {
        std::env::temp_dir().join(format!("delyx-next-{name}-{}", stamp()))
    }

    fn temp_file(name: &str) -> PathBuf {
        std::env::temp_dir().join(format!("delyx-next-{name}-{}.sqlite3", stamp()))
    }

    fn stamp() -> u128 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    }
}
