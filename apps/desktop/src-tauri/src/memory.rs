use crate::approval::{ApprovalEngine, ApprovalError, RiskyAction};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MemoryCandidate {
    pub id: String,
    pub scope: MemoryScope,
    pub key: String,
    pub value: String,
    pub source_run_id: String,
    pub source_thread_id: String,
    pub status: MemoryCandidateStatus,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MemoryRecord {
    pub id: String,
    pub scope: MemoryScope,
    pub key: String,
    pub value: String,
    pub source_run_id: String,
    pub source_thread_id: String,
    pub supersedes: Option<String>,
    pub suppressed: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MemoryScope {
    Project,
    User,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MemoryCandidateStatus {
    Pending,
    Promoted,
    Suppressed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SourceRunStatus {
    Completed,
    Failed,
    Running,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MemoryError {
    Approval(ApprovalError),
    CandidateNotFound,
    FailedRunCannotPromote,
    NotPending,
    RecordNotFound,
}

#[derive(Debug, Default)]
pub struct MemoryStore {
    candidates: Vec<MemoryCandidate>,
    next_candidate_id: usize,
    next_record_id: usize,
    records: Vec<MemoryRecord>,
}

impl MemoryStore {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn propose_candidate(&mut self, input: MemoryCandidateInput) -> MemoryCandidate {
        self.next_candidate_id += 1;
        let candidate = MemoryCandidate {
            id: format!("memory-candidate-{}", self.next_candidate_id),
            scope: input.scope,
            key: input.key,
            value: input.value,
            source_run_id: input.source_run_id,
            source_thread_id: input.source_thread_id,
            status: MemoryCandidateStatus::Pending,
        };
        self.candidates.push(candidate.clone());
        candidate
    }

    pub fn promote_approved(
        &mut self,
        candidate_id: &str,
        approval_id: &str,
        now: u64,
        approvals: &ApprovalEngine,
        source_status: SourceRunStatus,
    ) -> Result<MemoryRecord, MemoryError> {
        approvals
            .assert_can_execute_action(approval_id, now, RiskyAction::DurableMemorySave)
            .map_err(MemoryError::Approval)?;
        if source_status != SourceRunStatus::Completed {
            return Err(MemoryError::FailedRunCannotPromote);
        }
        let index = self.candidate_index(candidate_id)?;
        if self.candidates[index].status != MemoryCandidateStatus::Pending {
            return Err(MemoryError::NotPending);
        }

        let candidate = self.candidates[index].clone();
        let supersedes = self.suppress_matching_record(candidate.scope, &candidate.key);
        self.next_record_id += 1;
        let record = MemoryRecord {
            id: format!("memory-{}", self.next_record_id),
            scope: candidate.scope,
            key: candidate.key,
            value: candidate.value,
            source_run_id: candidate.source_run_id,
            source_thread_id: candidate.source_thread_id,
            supersedes,
            suppressed: false,
        };
        self.records.push(record.clone());
        self.candidates[index].status = MemoryCandidateStatus::Promoted;
        Ok(record)
    }

    pub fn suppress_candidate(&mut self, candidate_id: &str) -> Result<(), MemoryError> {
        let index = self.candidate_index(candidate_id)?;
        if self.candidates[index].status != MemoryCandidateStatus::Pending {
            return Err(MemoryError::NotPending);
        }
        self.candidates[index].status = MemoryCandidateStatus::Suppressed;
        Ok(())
    }

    pub fn suppress_memory(&mut self, record_id: &str) -> Result<(), MemoryError> {
        let record = self.records.iter_mut().find(|item| item.id == record_id).ok_or(MemoryError::RecordNotFound)?;
        record.suppressed = true;
        Ok(())
    }

    pub fn candidates(&self) -> &[MemoryCandidate] {
        &self.candidates
    }

    pub fn records(&self) -> &[MemoryRecord] {
        &self.records
    }

    fn candidate_index(&self, candidate_id: &str) -> Result<usize, MemoryError> {
        self.candidates.iter().position(|candidate| candidate.id == candidate_id).ok_or(MemoryError::CandidateNotFound)
    }

    fn suppress_matching_record(&mut self, scope: MemoryScope, key: &str) -> Option<String> {
        let record = self.records.iter_mut().find(|item| item.scope == scope && item.key == key && !item.suppressed)?;
        record.suppressed = true;
        Some(record.id.clone())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MemoryCandidateInput {
    pub scope: MemoryScope,
    pub key: String,
    pub value: String,
    pub source_run_id: String,
    pub source_thread_id: String,
}
