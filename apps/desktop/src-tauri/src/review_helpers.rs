use crate::patch::PatchProposal;
use crate::review::{DiffHunkRef, ReviewFinding};
use crate::test_runner::{TestArtifact, TestStatus};

pub(crate) fn first_hunk(patches: &[PatchProposal]) -> Option<DiffHunkRef> {
    let patch = patches.first()?;
    let file = patch.files.first()?;
    Some(DiffHunkRef {
        patch_id: patch.id.clone(),
        file_path: file.path.display().to_string(),
        line_index: 0,
    })
}

pub(crate) fn risk_summary(findings: &[ReviewFinding]) -> String {
    if findings.is_empty() {
        "No deterministic review findings.".to_string()
    } else {
        format!("{} prioritized finding(s).", findings.len())
    }
}

pub(crate) fn test_summary(tests: &[TestArtifact]) -> String {
    if tests.is_empty() {
        "No test artifacts captured.".to_string()
    } else if tests
        .iter()
        .any(|artifact| artifact.status == TestStatus::Failed)
    {
        "At least one test artifact failed.".to_string()
    } else {
        "All captured test artifacts passed.".to_string()
    }
}
