//! Promote a write-capable worker run's captured file changes into a real,
//! reviewable Delyx patch record. The promoted patch lands as `applied` with
//! checkpoint receipts (the pre-run contents), so the existing diff UI renders
//! it and the existing approval-gated `patch_restore_approved` path can roll it
//! back — worker edits get the same trust treatment as Delyx's own patches.

use crate::external_agent_diff::ExternalDiffFileChange;
use crate::patch_bridge::{
    diff_view, PatchBridgeStore, PatchCheckpointFileView, PatchFileView, PatchProposalView,
};
use crate::patch_diff::build_diff;

pub fn promote_worker_diff_to_patch(
    store: &mut PatchBridgeStore,
    run_id: &str,
    approval_id: &str,
    changes: &[ExternalDiffFileChange],
) -> Option<PatchProposalView> {
    if changes.is_empty() {
        return None;
    }
    store.next_patch_id += 1;
    let view = PatchProposalView {
        id: format!("patch-{}", store.next_patch_id),
        run_id: run_id.to_string(),
        approval_id: approval_id.to_string(),
        status: "applied".to_string(),
        checkpoint_id: Some(format!(
            "worker-checkpoint-{run_id}-{}",
            store.next_patch_id
        )),
        restore_approval_id: None,
        checkpoint_files: changes
            .iter()
            .map(|change| PatchCheckpointFileView {
                path: change.path.clone(),
                // `None` means the file did not exist before the run, so restore
                // removes it instead of writing empty contents.
                contents: (change.change_kind != "create").then(|| change.before.clone()),
            })
            .collect(),
        files: changes
            .iter()
            .map(|change| PatchFileView {
                path: change.path.clone(),
                before: change.before.clone(),
                after: change.after.clone(),
                change_kind: change.change_kind.to_string(),
                diff: build_diff(&change.before, &change.after)
                    .iter()
                    .map(diff_view)
                    .collect(),
            })
            .collect(),
    };
    store.records.push(view.clone());
    Some(view)
}
