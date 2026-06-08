use crate::patch::{DiffLineKind, PatchProposal};
use crate::review_helpers::{first_hunk, risk_summary, test_summary};
use crate::test_runner::{TestArtifact, TestStatus};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReviewReport {
    pub id: String,
    pub run_id: String,
    pub mode: ReviewMode,
    pub findings: Vec<ReviewFinding>,
    pub risk_summary: String,
    pub test_summary: String,
    pub evidence_summary: String,
    pub decision: ReviewDecision,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReviewMode {
    ReadOnly,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReviewDecision {
    Pending,
    ReviseRequested,
    Accepted,
    RevertRequested,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReviewFinding {
    pub id: String,
    pub priority: FindingPriority,
    pub title: String,
    pub detail: String,
    pub risk_label: String,
    pub suggested_fix: String,
    pub hunk: DiffHunkRef,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum FindingPriority {
    P0,
    P1,
    P2,
    P3,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DiffHunkRef {
    pub patch_id: String,
    pub file_path: String,
    pub line_index: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RevisionRequest {
    pub report_id: String,
    pub finding_id: String,
    pub decision: ReviewDecision,
    pub next_flow: Vec<&'static str>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReviewCapability {
    ReadDiff,
    ReadTestArtifact,
    ReadEvidence,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ReviewError {
    FindingNotFound,
    ReportNotFound,
}

#[derive(Debug, Default)]
pub struct ReviewAgent {
    reports: Vec<ReviewReport>,
    next_finding_id: usize,
    next_report_id: usize,
}

impl ReviewAgent {
    pub fn new() -> Self {
        Self::default()
    }

    pub(crate) fn with_loaded_counters(next_report_id: usize, next_finding_id: usize) -> Self {
        Self {
            reports: Vec::new(),
            next_finding_id,
            next_report_id,
        }
    }

    pub fn capabilities() -> Vec<ReviewCapability> {
        vec![
            ReviewCapability::ReadDiff,
            ReviewCapability::ReadTestArtifact,
            ReviewCapability::ReadEvidence,
        ]
    }

    pub fn can_write() -> bool {
        false
    }

    pub fn review(
        &mut self,
        run_id: &str,
        patches: &[PatchProposal],
        tests: &[TestArtifact],
    ) -> ReviewReport {
        let mut findings = Vec::new();
        for patch in patches {
            findings.extend(self.find_patch_risks(patch));
        }
        findings.extend(self.find_failed_tests(patches, tests));
        findings.sort_by_key(|finding| finding.priority);

        self.next_report_id += 1;
        let report = ReviewReport {
            id: format!("review-{}", self.next_report_id),
            run_id: run_id.to_string(),
            mode: ReviewMode::ReadOnly,
            risk_summary: risk_summary(&findings),
            test_summary: test_summary(tests),
            evidence_summary: "Evidence review waits for EvidenceRecords.".to_string(),
            decision: ReviewDecision::Pending,
            findings,
        };
        self.reports.push(report.clone());
        report
    }

    pub fn request_revision(
        &mut self,
        report_id: &str,
        finding_id: &str,
    ) -> Result<RevisionRequest, ReviewError> {
        let report = self.report_mut(report_id)?;
        if !report
            .findings
            .iter()
            .any(|finding| finding.id == finding_id)
        {
            return Err(ReviewError::FindingNotFound);
        }
        report.decision = ReviewDecision::ReviseRequested;
        Ok(RevisionRequest {
            report_id: report_id.to_string(),
            finding_id: finding_id.to_string(),
            decision: ReviewDecision::ReviseRequested,
            next_flow: vec!["plan", "build"],
        })
    }

    pub fn report_decision(&self, report_id: &str) -> Result<ReviewDecision, ReviewError> {
        Ok(self.report(report_id)?.decision)
    }

    fn find_patch_risks(&mut self, patch: &PatchProposal) -> Vec<ReviewFinding> {
        let mut findings = Vec::new();
        for file in &patch.files {
            for (index, line) in file.diff.iter().enumerate() {
                if line.kind != DiffLineKind::Added {
                    continue;
                }
                let lower = line.text.to_lowercase();
                if lower.contains("unwrap()") {
                    findings.push(self.finding(
                        FindingPriority::P1,
                        "Added unwrap can panic",
                        "Runtime panic risk in new code.",
                        "panic",
                        "Handle the None/Err case explicitly.",
                        patch,
                        &file.path.display().to_string(),
                        index,
                    ));
                } else if lower.contains("todo!") || lower.contains("todo") {
                    findings.push(self.finding(
                        FindingPriority::P2,
                        "Added unfinished TODO",
                        "New code still contains unfinished work.",
                        "incomplete",
                        "Finish or remove the TODO before accepting.",
                        patch,
                        &file.path.display().to_string(),
                        index,
                    ));
                }
            }
        }
        findings
    }

    fn find_failed_tests(
        &mut self,
        patches: &[PatchProposal],
        tests: &[TestArtifact],
    ) -> Vec<ReviewFinding> {
        tests
            .iter()
            .filter(|artifact| artifact.status == TestStatus::Failed)
            .filter_map(|artifact| first_hunk(patches).map(|hunk| (artifact, hunk)))
            .map(|(artifact, hunk)| {
                self.finding_from_hunk(
                    FindingPriority::P1,
                    "Test artifact failed",
                    artifact
                        .failure_summary
                        .as_deref()
                        .unwrap_or("A test command failed."),
                    "test failure",
                    "Repair the failing test before accepting.",
                    hunk,
                )
            })
            .collect()
    }

    fn finding(
        &mut self,
        priority: FindingPriority,
        title: &str,
        detail: &str,
        risk: &str,
        fix: &str,
        patch: &PatchProposal,
        file_path: &str,
        line_index: usize,
    ) -> ReviewFinding {
        self.finding_from_hunk(
            priority,
            title,
            detail,
            risk,
            fix,
            DiffHunkRef {
                patch_id: patch.id.clone(),
                file_path: file_path.to_string(),
                line_index,
            },
        )
    }

    fn finding_from_hunk(
        &mut self,
        priority: FindingPriority,
        title: &str,
        detail: &str,
        risk: &str,
        fix: &str,
        hunk: DiffHunkRef,
    ) -> ReviewFinding {
        self.next_finding_id += 1;
        ReviewFinding {
            id: format!("finding-{}", self.next_finding_id),
            priority,
            title: title.to_string(),
            detail: detail.to_string(),
            risk_label: risk.to_string(),
            suggested_fix: fix.to_string(),
            hunk,
        }
    }

    fn report(&self, report_id: &str) -> Result<&ReviewReport, ReviewError> {
        self.reports
            .iter()
            .find(|report| report.id == report_id)
            .ok_or(ReviewError::ReportNotFound)
    }

    fn report_mut(&mut self, report_id: &str) -> Result<&mut ReviewReport, ReviewError> {
        self.reports
            .iter_mut()
            .find(|report| report.id == report_id)
            .ok_or(ReviewError::ReportNotFound)
    }
}
