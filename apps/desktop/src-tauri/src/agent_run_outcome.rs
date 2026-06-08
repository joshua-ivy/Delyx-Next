use crate::agent_run::{
    ensure_running, AgentOutcome, AgentRunError, AgentRunLedger, AgentRunStatus,
};

impl AgentRunLedger {
    pub fn complete_run(&mut self, run_id: &str, summary: &str) -> Result<(), AgentRunError> {
        self.complete_run_with_support(run_id, summary, Vec::new(), Vec::new())
    }

    pub fn complete_run_with_support(
        &mut self,
        run_id: &str,
        summary: &str,
        evidence_record_ids: Vec<String>,
        test_artifact_ids: Vec<String>,
    ) -> Result<(), AgentRunError> {
        self.finish_run(
            run_id,
            AgentRunStatus::Completed,
            summary,
            evidence_record_ids,
            test_artifact_ids,
        )
    }

    pub fn fail_run(&mut self, run_id: &str, summary: &str) -> Result<(), AgentRunError> {
        self.finish_run(
            run_id,
            AgentRunStatus::Failed,
            summary,
            Vec::new(),
            Vec::new(),
        )
    }

    fn finish_run(
        &mut self,
        run_id: &str,
        status: AgentRunStatus,
        summary: &str,
        evidence_record_ids: Vec<String>,
        test_artifact_ids: Vec<String>,
    ) -> Result<(), AgentRunError> {
        let run = self.run_mut(run_id)?;
        ensure_running(run)?;
        run.status = status;
        run.outcome = Some(AgentOutcome {
            evidence_record_ids,
            status,
            summary: summary.to_string(),
            test_artifact_ids,
        });
        Ok(())
    }
}
