#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EvidenceRecord {
    pub id: String,
    pub run_id: String,
    pub source_kind: EvidenceSourceKind,
    pub title: String,
    pub locator: String,
    pub excerpt: String,
    pub stance: EvidenceStance,
    pub claim_key: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum EvidenceSourceKind {
    LocalFile,
    RepoSymbol,
    Diff,
    Test,
    Terminal,
    ExternalAgent,
    Web,
    Memory,
    ModelCall,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EvidenceStance {
    Supports,
    Contradicts,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClaimAudit {
    pub id: String,
    pub text: String,
    pub status: ClaimStatus,
    pub requires_support: bool,
    pub evidence_ids: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ClaimStatus {
    Supported,
    InsufficientEvidence,
    Contradicted,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Contradiction {
    pub claim_id: String,
    pub supporting_evidence_id: String,
    pub contradicting_evidence_id: String,
    pub message: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResearchAnswer {
    pub run_id: String,
    pub question: String,
    pub summary: String,
    pub receipts: Vec<EvidenceRecord>,
    pub audits: Vec<ClaimAudit>,
    pub contradictions: Vec<Contradiction>,
}

#[derive(Debug, Default)]
pub struct EvidenceStore {
    records: Vec<EvidenceRecord>,
    next_id: usize,
}

impl EvidenceStore {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add(&mut self, input: EvidenceInput) -> EvidenceRecord {
        self.next_id += 1;
        let record = EvidenceRecord {
            id: format!("evidence-{}", self.next_id),
            run_id: input.run_id,
            source_kind: input.source_kind,
            title: input.title,
            locator: input.locator,
            excerpt: input.excerpt,
            stance: input.stance,
            claim_key: normalize_claim(&input.claim),
        };
        self.records.push(record.clone());
        record
    }

    pub fn for_run(&self, run_id: &str) -> Vec<EvidenceRecord> {
        let mut records: Vec<_> = self
            .records
            .iter()
            .filter(|record| record.run_id == run_id)
            .cloned()
            .collect();
        records.sort_by_key(|record| record.source_kind);
        records
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EvidenceInput {
    pub run_id: String,
    pub source_kind: EvidenceSourceKind,
    pub title: String,
    pub locator: String,
    pub excerpt: String,
    pub stance: EvidenceStance,
    pub claim: String,
}

#[derive(Debug, Default)]
pub struct ResearchAgent {
    store: EvidenceStore,
    next_claim_id: usize,
}

impl ResearchAgent {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_evidence(&mut self, input: EvidenceInput) -> EvidenceRecord {
        self.store.add(input)
    }

    pub fn answer(&mut self, run_id: &str, question: &str, claims: Vec<&str>) -> ResearchAnswer {
        let receipts = self.store.for_run(run_id);
        let mut audits = Vec::new();
        let mut contradictions = Vec::new();
        for claim in claims {
            let audit = self.audit_claim(claim, &receipts);
            contradictions.extend(contradictions_for(&audit, &receipts));
            audits.push(audit);
        }
        let summary = if audits
            .iter()
            .all(|audit| audit.status == ClaimStatus::Supported)
        {
            "Evidence supports the audited claims.".to_string()
        } else {
            "insufficient evidence.".to_string()
        };
        ResearchAnswer {
            run_id: run_id.to_string(),
            question: question.to_string(),
            summary,
            receipts,
            audits,
            contradictions,
        }
    }

    fn audit_claim(&mut self, claim: &str, receipts: &[EvidenceRecord]) -> ClaimAudit {
        self.next_claim_id += 1;
        let key = normalize_claim(claim);
        let matching: Vec<_> = receipts
            .iter()
            .filter(|record| record.claim_key == key)
            .collect();
        let supports = matching
            .iter()
            .filter(|record| record.stance == EvidenceStance::Supports)
            .count();
        let contradicts = matching
            .iter()
            .filter(|record| record.stance == EvidenceStance::Contradicts)
            .count();
        let status = match (supports, contradicts) {
            (0, _) => ClaimStatus::InsufficientEvidence,
            (_, 0) => ClaimStatus::Supported,
            _ => ClaimStatus::Contradicted,
        };
        ClaimAudit {
            id: format!("claim-{}", self.next_claim_id),
            text: claim.to_string(),
            status,
            requires_support: requires_support(claim),
            evidence_ids: matching.iter().map(|record| record.id.clone()).collect(),
        }
    }
}

fn contradictions_for(audit: &ClaimAudit, receipts: &[EvidenceRecord]) -> Vec<Contradiction> {
    let supporting = audit
        .evidence_ids
        .iter()
        .find(|id| evidence_stance(id, receipts) == Some(EvidenceStance::Supports));
    let contradicting = audit
        .evidence_ids
        .iter()
        .find(|id| evidence_stance(id, receipts) == Some(EvidenceStance::Contradicts));
    match (supporting, contradicting) {
        (Some(left), Some(right)) => vec![Contradiction {
            claim_id: audit.id.clone(),
            supporting_evidence_id: left.to_string(),
            contradicting_evidence_id: right.to_string(),
            message: "Conflicting evidence supports and contradicts this claim.".to_string(),
        }],
        _ => Vec::new(),
    }
}

fn evidence_stance(id: &str, receipts: &[EvidenceRecord]) -> Option<EvidenceStance> {
    receipts
        .iter()
        .find(|record| record.id == id)
        .map(|record| record.stance)
}

fn normalize_claim(value: &str) -> String {
    value.trim().to_lowercase()
}

fn requires_support(claim: &str) -> bool {
    claim.chars().any(|character| character.is_ascii_digit()) || contains_date_word(claim)
}

fn contains_date_word(claim: &str) -> bool {
    let lower = claim.to_lowercase();
    [
        "january",
        "february",
        "march",
        "april",
        "may",
        "june",
        "july",
        "august",
        "september",
        "october",
        "november",
        "december",
    ]
    .iter()
    .any(|month| lower.contains(month))
}
