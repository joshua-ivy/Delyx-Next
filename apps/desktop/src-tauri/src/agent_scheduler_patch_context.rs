use crate::agent_scheduler_bridge::AgentScheduleRequest;
use crate::approval::RiskyAction;
use crate::approval_bridge::{ApprovalBridgeRecord, ApprovalBridgeStore};
use crate::patch_bridge::PatchBridgeStore;
use crate::plan_bridge::PlanView;
use crate::review_bridge::ReviewBridgeStore;
use crate::thread_run_bridge::ThreadRunStore;
use crate::workspace_bridge::WorkspaceProjectView;
use std::path::Path;

pub(crate) fn hydrate_patch_draft_request(
    threads: &ThreadRunStore,
    approvals: &ApprovalBridgeStore,
    patches: &PatchBridgeStore,
    reviews: &ReviewBridgeStore,
    plan_db: &Path,
    request: AgentScheduleRequest,
) -> Result<AgentScheduleRequest, String> {
    let Some(context) = run_context(threads, plan_db, &request.run_id)? else {
        return Ok(request);
    };
    Ok(AgentScheduleRequest {
        patch_draft_approval_id: ready_repair_approval(
            approvals,
            reviews,
            &context.project,
            &context.run_id,
            request.now_ms,
        )
        .or_else(|| {
            ready_plan_approval(
                approvals,
                patches,
                &context,
                &request.run_id,
                request.now_ms,
            )
        }),
        ..request
    })
}

struct RunContext {
    plan: PlanView,
    project: WorkspaceProjectView,
    run_id: String,
}

fn run_context(
    threads: &ThreadRunStore,
    plan_db: &Path,
    run_id: &str,
) -> Result<Option<RunContext>, String> {
    let Some(record) = threads.records.iter().find(|item| item.run_id == run_id) else {
        return Ok(None);
    };
    let Some(plan) = crate::plan_persistence::load_plans_from_path(plan_db, &record.project_id)?
        .into_iter()
        .find(|plan| plan.thread_id == record.thread_id)
    else {
        return Ok(None);
    };
    let Some(project) =
        crate::workspace_persistence::load_project_by_id(plan_db, &record.project_id)?
    else {
        return Ok(None);
    };
    Ok(Some(RunContext {
        plan,
        project,
        run_id: record.run_id.clone(),
    }))
}

fn ready_plan_approval(
    approvals: &ApprovalBridgeStore,
    patches: &PatchBridgeStore,
    context: &RunContext,
    run_id: &str,
    now_ms: u64,
) -> Option<String> {
    if context.plan.decision != "approved"
        || patches.records.iter().any(|patch| patch.run_id == run_id)
        || approved_plan_files(&context.plan, &context.project).is_empty()
    {
        return None;
    }
    ready_file_write_approval(
        approvals,
        run_id,
        &format!("{run_id}-plan-approval"),
        now_ms,
    )
    .filter(|record| plan_scope_matches(record, &context.plan, &context.project))
    .map(|record| record.proposal_id.clone())
}

fn ready_repair_approval(
    approvals: &ApprovalBridgeStore,
    reviews: &ReviewBridgeStore,
    project: &WorkspaceProjectView,
    run_id: &str,
    now_ms: u64,
) -> Option<String> {
    let (node_id, finding_path) = active_repair(reviews, project, run_id)?;
    ready_file_write_approval(approvals, run_id, &node_id, now_ms)
        .filter(|record| scope_includes(record.scope.paths.as_deref(), &finding_path))
        .map(|record| record.proposal_id.clone())
}

fn ready_file_write_approval<'a>(
    approvals: &'a ApprovalBridgeStore,
    run_id: &str,
    node_id: &str,
    now_ms: u64,
) -> Option<&'a ApprovalBridgeRecord> {
    approvals.records.iter().find(|record| {
        record.run_id == run_id
            && matches!(record.action_type.as_str(), "edit_file" | "write_file")
            && approvals
                .engine
                .all_proposals()
                .iter()
                .any(|proposal| proposal.id == record.proposal_id && proposal.node_id == node_id)
            && approvals
                .engine
                .assert_can_execute_action_for_run(
                    &record.proposal_id,
                    now_ms,
                    RiskyAction::FileWrite,
                    run_id,
                )
                .is_ok()
    })
}

fn plan_scope_matches(
    record: &ApprovalBridgeRecord,
    plan: &PlanView,
    project: &WorkspaceProjectView,
) -> bool {
    let files = approved_plan_files(plan, project);
    !files.is_empty()
        && files
            .iter()
            .any(|path| scope_includes(record.scope.paths.as_deref(), path))
}

fn approved_plan_files(plan: &PlanView, project: &WorkspaceProjectView) -> Vec<String> {
    plan.files_likely_involved
        .iter()
        .filter(|path| {
            project
                .indexed_files
                .iter()
                .any(|file| same_path(file, path))
        })
        .take(4)
        .cloned()
        .collect()
}

fn active_repair(
    reviews: &ReviewBridgeStore,
    project: &WorkspaceProjectView,
    run_id: &str,
) -> Option<(String, String)> {
    let report = reviews
        .reports
        .iter()
        .rev()
        .find(|report| report.run_id == run_id && report.decision == "revise_requested")?;
    let finding = report.findings.first()?;
    Some((
        format!("{run_id}-repair-{}-{}", report.id, finding.id),
        repair_relative_path(&finding.file_path, &project.path)?,
    ))
}

fn scope_includes(paths: Option<&[String]>, target: &str) -> bool {
    paths
        .map(|paths| paths.iter().any(|path| same_path(path, target)))
        .unwrap_or(false)
}

fn repair_relative_path(file_path: &str, project_path: &str) -> Option<String> {
    let normalized = file_path.replace('\\', "/");
    let root = project_path
        .replace('\\', "/")
        .trim_end_matches('/')
        .to_string();
    let absolute = normalized.starts_with('/') || normalized.as_bytes().get(1) == Some(&b':');
    let relative = if absolute
        && normalized
            .to_lowercase()
            .starts_with(&format!("{}/", root.to_lowercase()))
    {
        normalized[root.len() + 1..].to_string()
    } else if absolute {
        return None;
    } else {
        normalized.trim_start_matches("./").to_string()
    };
    let parts = relative
        .split('/')
        .filter(|part| !part.is_empty())
        .collect::<Vec<_>>();
    (!parts.is_empty() && parts.iter().all(|part| *part != "." && *part != ".."))
        .then(|| parts.join("/"))
}

fn same_path(left: &str, right: &str) -> bool {
    let left = left.replace('\\', "/").trim_start_matches("./").to_string();
    let right = right
        .replace('\\', "/")
        .trim_start_matches("./")
        .to_string();
    left.eq_ignore_ascii_case(&right)
}
