#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};

    use crate::agent_run::{
        append_agent_event, create_agent_run, get_agent_run, list_agent_runs, AgentRunError,
        AgentRunLedger, AgentRunStatus, EvidenceRecordInput, EvidenceRelevance,
    };

    const THREAD_ID: &str = "thread-local-real";

    #[test]
    fn creates_and_lists_runs_for_a_thread() {
        let mut ledger = AgentRunLedger::new();
        let run = ledger.create_run(THREAD_ID).unwrap();

        assert_eq!(run.status, AgentRunStatus::Running);
        assert_eq!(ledger.list_runs(THREAD_ID).len(), 1);
        assert!(ledger.list_runs("other-thread").is_empty());
    }

    #[test]
    fn appends_nodes_events_artifacts_and_evidence() {
        let mut ledger = AgentRunLedger::new();
        let run = ledger.create_run(THREAD_ID).unwrap();

        ledger
            .append_node(&run.id, "explore", "Read approved files")
            .unwrap();
        ledger
            .append_event(&run.id, "explore.started", "Read-only search started")
            .unwrap();
        ledger
            .record_artifact(&run.id, "timeline", "run timeline")
            .unwrap();
        ledger
            .record_evidence(&run.id, "local_file", "AGENTS.md")
            .unwrap();

        let run = ledger.get_run(&run.id).unwrap();
        assert_eq!(run.nodes[0].kind, "explore");
        assert_eq!(run.metrics.event_count, 1);
        assert_eq!(run.metrics.artifact_count, 1);
        assert_eq!(run.metrics.evidence_count, 1);
    }

    #[test]
    fn completes_and_fails_runs_as_terminal_states() {
        let mut ledger = AgentRunLedger::new();
        let completed = ledger.create_run(THREAD_ID).unwrap();
        let failed = ledger.create_run(THREAD_ID).unwrap();

        ledger
            .complete_run(&completed.id, "Finished without executing commands")
            .unwrap();
        ledger
            .fail_run(&failed.id, "Planner failed before approval")
            .unwrap();

        assert_eq!(
            ledger.get_run(&completed.id).unwrap().status,
            AgentRunStatus::Completed
        );
        assert_eq!(
            ledger
                .append_event(&completed.id, "late", "should fail")
                .unwrap_err(),
            AgentRunError::TerminalRun
        );
        assert_eq!(
            ledger.get_run(&failed.id).unwrap().status,
            AgentRunStatus::Failed
        );
    }

    #[test]
    fn run_can_resume_after_approval() {
        let mut ledger = AgentRunLedger::new();
        let run = ledger.create_run(THREAD_ID).unwrap();

        let waiting = ledger.wait_for_approval(&run.id, "prop-1").unwrap();
        assert_eq!(waiting.kind, "approval.waiting");
        assert_eq!(
            ledger.get_run(&run.id).unwrap().status,
            AgentRunStatus::WaitingForApproval
        );
        assert_eq!(
            ledger.complete_run(&run.id, "too soon").unwrap_err(),
            AgentRunError::TerminalRun
        );

        let resumed = ledger.resume_after_approval(&run.id, "prop-1").unwrap();
        assert_eq!(resumed.kind, "approval.resumed");
        assert_eq!(
            ledger.get_run(&run.id).unwrap().status,
            AgentRunStatus::Running
        );
    }

    #[test]
    fn persists_and_reloads_run_events() {
        let path = temp_path("ledger");
        let mut ledger = AgentRunLedger::new();
        let run = ledger.create_run(THREAD_ID).unwrap();
        let second = ledger.create_run(THREAD_ID).unwrap();
        ledger
            .append_node(&run.id, "explore", "Loaded from SQLite")
            .unwrap();
        ledger
            .append_event(&run.id, "created", "real ledger event")
            .unwrap();
        ledger
            .record_artifact(&run.id, "timeline", "primary artifact")
            .unwrap();
        ledger
            .record_artifact(&second.id, "timeline", "second run artifact")
            .unwrap();
        ledger
            .record_evidence_detail(&run.id, detailed_evidence("AGENTS.md"))
            .unwrap();
        ledger.complete_run(&run.id, "complete").unwrap();

        ledger.save_to_path(&path).unwrap();
        let bytes = fs::read(&path).unwrap();
        assert!(bytes.starts_with(b"SQLite format 3"));
        let mut loaded = AgentRunLedger::load_from_path(&path).unwrap();
        let loaded_run = loaded.get_run(&run.id).unwrap();

        assert_eq!(loaded_run.status, AgentRunStatus::Completed);
        assert_eq!(loaded_run.nodes[0].label, "Loaded from SQLite");
        assert_eq!(loaded_run.events[0].message, "real ledger event");
        assert_eq!(loaded_run.artifacts[0].label, "primary artifact");
        assert_eq!(loaded_run.evidence[0].title, "AGENTS.md");
        assert_eq!(loaded_run.evidence[0].source_id, "file://AGENTS.md");
        assert_eq!(
            loaded_run.evidence[0].quote.as_deref(),
            Some("Project rules")
        );
        assert_eq!(
            loaded_run.evidence[0]
                .relevance
                .as_ref()
                .unwrap()
                .relationship,
            "doc"
        );
        assert_eq!(loaded_run.outcome.as_ref().unwrap().summary, "complete");
        assert_eq!(
            loaded.get_run(&second.id).unwrap().artifacts[0].id,
            "artifact-1"
        );
        assert_eq!(
            loaded
                .append_event(&second.id, "after.reload", "counter advanced")
                .unwrap()
                .id,
            "event-2"
        );
        let _ = fs::remove_file(path);
    }

    #[test]
    fn command_shaped_api_wraps_ledger_operations() {
        let mut ledger = AgentRunLedger::new();
        let run = create_agent_run(&mut ledger, THREAD_ID).unwrap();
        let event = append_agent_event(&mut ledger, &run.id, "created", "command facade").unwrap();

        assert_eq!(event.kind, "created");
        assert_eq!(list_agent_runs(&ledger, THREAD_ID).len(), 1);
        assert_eq!(get_agent_run(&ledger, &run.id).unwrap().events.len(), 1);
    }

    #[test]
    fn rejects_empty_thread_ids() {
        let mut ledger = AgentRunLedger::new();

        assert_eq!(
            ledger.create_run(" ").unwrap_err(),
            AgentRunError::EmptyThread
        );
    }

    #[test]
    fn resume_requires_waiting_for_approval_state() {
        let mut ledger = AgentRunLedger::new();
        let run = ledger.create_run(THREAD_ID).unwrap();

        assert_eq!(
            ledger.resume_after_approval(&run.id, "prop-1").unwrap_err(),
            AgentRunError::InvalidTransition
        );
    }

    fn temp_path(name: &str) -> PathBuf {
        let stamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        std::env::temp_dir().join(format!("delyx-next-{name}-{stamp}.ledger"))
    }

    fn detailed_evidence(title: &str) -> EvidenceRecordInput {
        EvidenceRecordInput {
            hash: Some("sha256:rules".to_string()),
            quote: Some("Project rules".to_string()),
            relevance: Some(EvidenceRelevance {
                reason: "Rules file supports the answer constraints.".to_string(),
                relationship: "doc".to_string(),
                score: 92,
            }),
            retrieved_at: "2026-06-07T00:04:00.000Z".to_string(),
            source_id: format!("file://{title}"),
            source_kind: "local_file".to_string(),
            title: title.to_string(),
            uri: Some(format!("file:///{title}")),
        }
    }
}
