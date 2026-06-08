use crate::agent_run::{
    AgentRun, AgentRunError, AgentRunLedger, Artifact, EvidenceRecordInput, EvidenceRelevance,
};
use crate::approval::{ActionProposal, ApprovalEngine, ProposalStatus, RiskyAction};
use crate::test_runner_bridge::TestArtifactView;

#[derive(Debug, Clone, Default)]
pub(crate) struct FinalSupportInput {
    pub(crate) approval_records: Vec<ApprovalSupportRecord>,
    pub(crate) test_artifacts: Vec<TestArtifactView>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct FinalSupportLinks {
    pub(crate) evidence_record_ids: Vec<String>,
    pub(crate) test_artifact_ids: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ApprovalSupportRecord {
    pub(crate) id: String,
    pub(crate) action_type: String,
    pub(crate) scope: String,
    pub(crate) status: String,
}

pub(crate) fn approval_support_records(
    engine: &ApprovalEngine,
    run_id: &str,
) -> Vec<ApprovalSupportRecord> {
    engine
        .list_proposals(run_id)
        .into_iter()
        .map(approval_support_record)
        .collect()
}

pub(crate) fn synthesize_final_support(
    ledger: &mut AgentRunLedger,
    run_id: &str,
    input: FinalSupportInput,
) -> Result<FinalSupportLinks, String> {
    let run = ledger.get_run(run_id).map_err(agent_error)?.clone();
    let mut evidence_ids = unique_non_empty(run.evidence.iter().map(|item| item.id.clone()));
    let mut evidence_keys = run
        .evidence
        .iter()
        .map(|item| evidence_key(&item.source_kind, &item.source_id))
        .collect::<Vec<_>>();

    for candidate in support_candidates(&run, &input.approval_records, &input.test_artifacts) {
        if candidate.source_id.trim().is_empty() {
            continue;
        }
        let key = evidence_key(&candidate.source_kind, &candidate.source_id);
        if evidence_keys.contains(&key) {
            continue;
        }
        let record = ledger
            .record_evidence_detail(run_id, candidate.into_input())
            .map_err(agent_error)?;
        evidence_keys.push(key);
        evidence_ids.push(record.id);
    }

    Ok(FinalSupportLinks {
        evidence_record_ids: evidence_ids,
        test_artifact_ids: unique_non_empty(input.test_artifacts.into_iter().map(|item| item.id)),
    })
}

fn support_candidates(
    run: &AgentRun,
    approvals: &[ApprovalSupportRecord],
    tests: &[TestArtifactView],
) -> Vec<SupportCandidate> {
    let mut candidates = run
        .artifacts
        .iter()
        .filter_map(artifact_candidate)
        .collect::<Vec<_>>();
    candidates.extend(approvals.iter().map(approval_candidate));
    candidates.extend(tests.iter().map(test_command_candidate));
    candidates
}

fn artifact_candidate(artifact: &Artifact) -> Option<SupportCandidate> {
    let (source_kind, relationship, reason, title) = match artifact.kind.as_str() {
        "model_response" => (
            "model_call",
            "model-generated",
            "Model response artifact was recorded before final support.",
            artifact.label.clone(),
        ),
        "patch_apply" => (
            "diff",
            "direct_implementation",
            "Applied diff artifact was recorded before final support.",
            format!("Applied patch {}", artifact.label),
        ),
        "patch_proposal" => (
            "diff",
            "direct_implementation",
            "Proposed diff artifact was recorded before final support.",
            format!("Patch proposal {}", artifact.label),
        ),
        "patch_restore" => (
            "diff",
            "direct_implementation",
            "Patch restore artifact was recorded before final support.",
            format!("Restored patch {}", artifact.label),
        ),
        "review_report" | "review_revision" => (
            "review",
            "review",
            "Review artifact was recorded before final support.",
            artifact.label.clone(),
        ),
        "test" => (
            "test",
            "test",
            "Test artifact was recorded before final support.",
            format!("Test artifact {}", artifact.label),
        ),
        _ => return None,
    };
    Some(SupportCandidate {
        quote: Some(artifact.label.clone()),
        reason: reason.to_string(),
        relationship: relationship.to_string(),
        source_id: artifact.label.clone(),
        source_kind: source_kind.to_string(),
        title,
    })
}

fn approval_candidate(approval: &ApprovalSupportRecord) -> SupportCandidate {
    SupportCandidate {
        quote: Some(format!(
            "{} / {} / {}",
            approval.status, approval.action_type, approval.scope
        )),
        reason: "Approval record was linked before final support.".to_string(),
        relationship: "approval".to_string(),
        source_id: approval.id.clone(),
        source_kind: "approval".to_string(),
        title: format!("Approval {} ({})", approval.id, approval.status),
    }
}

fn approval_support_record(proposal: &ActionProposal) -> ApprovalSupportRecord {
    ApprovalSupportRecord {
        action_type: action_key(proposal.action).to_string(),
        id: proposal.id.clone(),
        scope: proposal.scope.clone(),
        status: status_key(proposal.status).to_string(),
    }
}

fn action_key(action: RiskyAction) -> &'static str {
    match action {
        RiskyAction::ConnectorWrite => "use_connector",
        RiskyAction::DependencyInstall => "install_dependency",
        RiskyAction::DurableMemorySave => "save_memory",
        RiskyAction::ExternalAgentExecution => "external_agent",
        RiskyAction::ExternalSend => "external_send",
        RiskyAction::FileWrite => "edit_file",
        RiskyAction::ScheduledRiskyAction => "schedule_work",
        RiskyAction::TerminalCommand => "run_terminal",
    }
}

fn status_key(status: ProposalStatus) -> &'static str {
    match status {
        ProposalStatus::Approved => "approved",
        ProposalStatus::Denied => "denied",
        ProposalStatus::Expired => "expired",
        ProposalStatus::Pending => "pending",
    }
}

fn test_command_candidate(test: &TestArtifactView) -> SupportCandidate {
    SupportCandidate {
        quote: Some(format!(
            "{} / {} / exit {}",
            test.cwd,
            test.command,
            test.exit_code
                .map(|code| code.to_string())
                .unwrap_or_else(|| "unknown".to_string())
        )),
        reason: "Approved test command receipt was linked before final support.".to_string(),
        relationship: "test".to_string(),
        source_id: test.id.clone(),
        source_kind: "terminal".to_string(),
        title: format!("Command: {}", test.command),
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct SupportCandidate {
    source_kind: String,
    source_id: String,
    title: String,
    quote: Option<String>,
    relationship: String,
    reason: String,
}

impl SupportCandidate {
    fn into_input(self) -> EvidenceRecordInput {
        EvidenceRecordInput {
            hash: None,
            quote: self.quote,
            relevance: Some(EvidenceRelevance {
                reason: self.reason,
                relationship: self.relationship,
                score: 10,
            }),
            retrieved_at: String::new(),
            source_id: self.source_id,
            source_kind: self.source_kind,
            title: self.title,
            uri: None,
        }
    }
}

fn evidence_key(source_kind: &str, source_id: &str) -> String {
    format!("{source_kind}:{source_id}")
}

fn unique_non_empty(items: impl IntoIterator<Item = String>) -> Vec<String> {
    items.into_iter().fold(Vec::new(), |mut acc, item| {
        if !item.trim().is_empty() && !acc.contains(&item) {
            acc.push(item);
        }
        acc
    })
}

fn agent_error(error: AgentRunError) -> String {
    format!("{error:?}")
}
