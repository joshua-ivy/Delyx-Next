use crate::patch::PatchStatus;
use crate::review::{FindingPriority, ReviewDecision, ReviewMode};
use crate::test_runner::TestStatus;

pub(crate) fn patch_status(status: &str) -> Result<PatchStatus, String> {
    match status {
        "applied" => Ok(PatchStatus::Applied),
        "proposed" => Ok(PatchStatus::Proposed),
        "restored" => Ok(PatchStatus::Restored),
        _ => Err("Unsupported patch status.".to_string()),
    }
}

pub(crate) fn test_status(
    status: Option<&str>,
    exit_code: Option<i32>,
) -> Result<TestStatus, String> {
    match status {
        Some("failed") => Ok(TestStatus::Failed),
        Some("passed") => Ok(TestStatus::Passed),
        Some("not_run") => Err("Cannot review a test that did not run.".to_string()),
        Some(_) => Err("Unsupported test status.".to_string()),
        None => Ok(if exit_code == Some(0) {
            TestStatus::Passed
        } else {
            TestStatus::Failed
        }),
    }
}

pub(crate) fn mode_key(mode: ReviewMode) -> &'static str {
    match mode {
        ReviewMode::ReadOnly => "read_only",
    }
}

pub(crate) fn decision_key(decision: ReviewDecision) -> &'static str {
    match decision {
        ReviewDecision::Accepted => "accepted",
        ReviewDecision::Pending => "pending",
        ReviewDecision::RevertRequested => "revert_requested",
        ReviewDecision::ReviseRequested => "revise_requested",
    }
}

pub(crate) fn priority_key(priority: FindingPriority) -> &'static str {
    match priority {
        FindingPriority::P0 => "p0",
        FindingPriority::P1 => "p1",
        FindingPriority::P2 => "p2",
        FindingPriority::P3 => "p3",
    }
}
