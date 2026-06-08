#[cfg(test)]
mod tests {
    use crate::agent_patch_draft_bridge::{
        execute_patch_draft_from_model_text, AgentPatchDraftExecuteRequest,
    };
    use crate::agent_patch_draft_parser::patch_request_from_draft_text;
    use crate::approval::{ApprovalEngine, ProposalInput, RiskLevel, RiskyAction};
    use crate::patch_bridge::PatchBridgeStore;
    use crate::thread_run_bridge::{
        create_thread_run_record, ThreadRunCreateRequest, ThreadRunStore,
    };
    use crate::workspace_bridge::WorkspaceFileReadView;
    use std::fs;
    use std::path::{Path, PathBuf};
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn patch_draft_executes_model_text_into_agent_patch_proposal() {
        let root = temp_workspace("patch-draft-exec");
        let file = root.join("src/main.ts");
        fs::create_dir_all(file.parent().unwrap()).unwrap();
        fs::write(&file, "export const value = 1;\n").unwrap();
        let mut threads = ThreadRunStore::default();
        let record = create_thread_run_record(&mut threads, create_request()).unwrap();
        let (approvals, approval_id) = approved_file_write(&record.run.id);
        let request = draft_request(&root, &record.run.id, &approval_id);
        let files = read_files();
        let mut patches = PatchBridgeStore::default();

        let view = execute_patch_draft_from_model_text(
            &mut threads,
            &mut patches,
            &approvals,
            &request,
            &files,
            "{\"files\":[{\"path\":\"src/main.ts\",\"after\":\"export const value = 2;\\n\"}]}",
        )
        .unwrap();

        let run = threads.ledger.get_run(&record.run.id).unwrap();
        assert_eq!(view.status, "completed");
        assert_eq!(patches.records.len(), 1);
        assert_eq!(
            patches.records[0].files[0].before,
            "export const value = 1;\n"
        );
        assert!(run
            .events
            .iter()
            .any(|event| event.kind == "model_call.completed"));
        assert!(run
            .artifacts
            .iter()
            .any(|artifact| artifact.kind == "model_response"));
        assert!(run
            .artifacts
            .iter()
            .any(|artifact| artifact.kind == "patch_proposal"));
        assert!(run
            .evidence
            .iter()
            .any(|item| item.source_kind == "local_file" && item.source_id == "src/main.ts"));
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn patch_draft_rejects_unapproved_or_unchanged_output() {
        let root = temp_workspace("patch-draft-reject");
        let request = draft_request(&root, "run-1", "approval-1");
        let files = read_files();

        let outside = patch_request_from_draft_text(
            &request,
            &files,
            "{\"files\":[{\"path\":\"src/secret.ts\",\"after\":\"x\\n\"}]}",
        )
        .unwrap_err();
        let unchanged = patch_request_from_draft_text(
            &request,
            &files,
            "{\"files\":[{\"path\":\"src/main.ts\",\"after\":\"export const value = 1;\\n\"}]}",
        )
        .unwrap_err();

        assert!(outside.contains("unapproved file"));
        assert!(unchanged.contains("unchanged"));
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn patch_draft_rejects_truncated_file_inputs() {
        let root = temp_workspace("patch-draft-truncated");
        let request = draft_request(&root, "run-1", "approval-1");
        let mut files = read_files();
        files[0].truncated = true;

        let error = patch_request_from_draft_text(
            &request,
            &files,
            "{\"files\":[{\"path\":\"src/main.ts\",\"after\":\"after\\n\"}]}",
        )
        .unwrap_err();

        assert!(error.contains("truncated"));
        let _ = fs::remove_dir_all(root);
    }

    fn create_request() -> ThreadRunCreateRequest {
        ThreadRunCreateRequest {
            created_at: "2026-06-08T00:00:00.000Z".to_string(),
            goal: "Draft patch in Rust".to_string(),
            project_id: "project-1".to_string(),
        }
    }

    fn draft_request(
        root: &Path,
        run_id: &str,
        approval_id: &str,
    ) -> AgentPatchDraftExecuteRequest {
        AgentPatchDraftExecuteRequest {
            approval_id: approval_id.to_string(),
            approved_roots: vec![root.display().to_string()],
            client_id: "patch-draft-1".to_string(),
            created_at_ms: 2,
            files_likely_involved: vec!["src/main.ts".to_string()],
            goal: "Update value.".to_string(),
            max_bytes_per_file: Some(20_000),
            model: "qwen3-coder:30b".to_string(),
            plan_steps: vec!["Update src/main.ts".to_string()],
            project_path: root.display().to_string(),
            run_id: run_id.to_string(),
            scope_paths: vec!["src/main.ts".to_string()],
        }
    }

    fn read_files() -> Vec<WorkspaceFileReadView> {
        vec![WorkspaceFileReadView {
            contents: "export const value = 1;\n".to_string(),
            path: "src/main.ts".to_string(),
            truncated: false,
        }]
    }

    fn approved_file_write(run_id: &str) -> (ApprovalEngine, String) {
        let mut engine = ApprovalEngine::new();
        let proposal = engine.propose(ProposalInput {
            action: RiskyAction::FileWrite,
            expected_result: "Propose a patch from approved plan files.".to_string(),
            expires_at: 10_000,
            node_id: "patch-draft".to_string(),
            reason: "Approved plan needs a generated patch.".to_string(),
            risk: RiskLevel::High,
            rollback_plan: "No file write happens during proposal.".to_string(),
            run_id: run_id.to_string(),
            scope: "One approved plan file.".to_string(),
        });
        engine.approve(&proposal.id, 1, "draft approval").unwrap();
        (engine, proposal.id)
    }

    fn temp_workspace(label: &str) -> PathBuf {
        let stamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let path = std::env::temp_dir().join(format!("delyx-next-{label}-{stamp}"));
        fs::create_dir_all(&path).unwrap();
        path
    }
}
