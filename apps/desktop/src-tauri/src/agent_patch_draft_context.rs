use crate::agent_patch_draft_bridge::{
    execute_patch_draft_record, AgentPatchDraftBridgeView, AgentPatchDraftExecuteRequest,
};
use crate::agent_patch_draft_dispatch::{
    verify_scheduler_patch_draft, AgentPatchDraftDispatchRequest,
};
use crate::approval_bridge::{ApprovalBridgeState, PermissionScopeView};
use crate::patch_bridge::PatchBridgeState;
use crate::plan_bridge::{PlanBridgeState, PlanView};
use crate::review_bridge::{ReviewBridgeState, ReviewBridgeStore};
use crate::test_runner_bridge::TestRunnerBridgeState;
use crate::thread_run_bridge::ThreadRunBridgeState;
use crate::workspace_bridge::WorkspaceProjectView;
use serde::Deserialize;

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentPatchDraftContextRequest {
    pub run_id: String,
    pub project_id: String,
    pub approval_id: String,
    pub model: String,
    pub has_supported_test_command: bool,
    #[serde(default)]
    pub test_approval_id: Option<String>,
    pub now_ms: u64,
    pub max_bytes_per_file: Option<usize>,
}

#[tauri::command]
pub fn agent_dispatch_patch_draft_from_context(
    threads: tauri::State<ThreadRunBridgeState>,
    patches: tauri::State<PatchBridgeState>,
    tests: tauri::State<TestRunnerBridgeState>,
    reviews: tauri::State<ReviewBridgeState>,
    approvals: tauri::State<ApprovalBridgeState>,
    plans: tauri::State<PlanBridgeState>,
    request: AgentPatchDraftContextRequest,
) -> Result<AgentPatchDraftBridgeView, String> {
    let execute = context_execute_request(&threads, &reviews, &approvals, &plans, &request)?;
    let dispatch = AgentPatchDraftDispatchRequest {
        execute,
        has_supported_test_command: request.has_supported_test_command,
        now_ms: request.now_ms,
        patch_draft_approval_id: Some(request.approval_id),
        test_approval_id: request.test_approval_id,
    };
    verify_scheduler_patch_draft(&threads, &patches, &tests, &reviews, &approvals, &dispatch)?;
    execute_patch_draft_record(&threads, &patches, &approvals, dispatch.execute)
}

pub(crate) fn context_execute_request(
    threads: &ThreadRunBridgeState,
    reviews: &ReviewBridgeState,
    approvals: &ApprovalBridgeState,
    plans: &PlanBridgeState,
    request: &AgentPatchDraftContextRequest,
) -> Result<AgentPatchDraftExecuteRequest, String> {
    validate_context_request(request)?;
    let (thread_id, thread_goal) = thread_context(threads, &request.run_id, &request.project_id)?;
    let project = crate::workspace_persistence::load_project_by_id(
        plans.database_path(),
        &request.project_id,
    )?
    .ok_or_else(|| "PatchDraft context requires a persisted workspace project.".to_string())?;
    let plan =
        crate::plan_persistence::load_plans_from_path(plans.database_path(), &request.project_id)?
            .into_iter()
            .find(|plan| plan.thread_id == thread_id)
            .ok_or_else(|| {
                "PatchDraft context requires a persisted plan for this thread.".to_string()
            })?;
    let approval_scope = approval_scope(approvals, &request.approval_id)?;
    let draft = draft_context(
        reviews,
        &request.run_id,
        &thread_goal,
        &project,
        &plan,
        &approval_scope,
    )?;
    Ok(AgentPatchDraftExecuteRequest {
        approval_id: request.approval_id.clone(),
        approved_roots: project.approved_roots,
        client_id: format!("patch-{}-{}", request.run_id, request.approval_id),
        created_at_ms: request.now_ms,
        files_likely_involved: draft.files,
        goal: draft.goal,
        max_bytes_per_file: request.max_bytes_per_file,
        model: request.model.clone(),
        plan_steps: draft.steps,
        project_path: project.path,
        run_id: request.run_id.clone(),
        scope_paths: draft.scope_paths,
    })
}

fn validate_context_request(request: &AgentPatchDraftContextRequest) -> Result<(), String> {
    if request.run_id.trim().is_empty()
        || request.project_id.trim().is_empty()
        || request.approval_id.trim().is_empty()
        || request.model.trim().is_empty()
        || request.now_ms == 0
    {
        return Err(
            "PatchDraft context requires run, project, approval, model, and clock.".to_string(),
        );
    }
    Ok(())
}

fn thread_context(
    threads: &ThreadRunBridgeState,
    run_id: &str,
    project_id: &str,
) -> Result<(String, String), String> {
    let store = threads
        .store
        .lock()
        .map_err(|_| "Thread bridge lock failed.".to_string())?;
    let record = store
        .records
        .iter()
        .find(|record| record.run_id == run_id)
        .ok_or_else(|| "PatchDraft context requires a thread/run record.".to_string())?;
    if record.project_id != project_id {
        return Err("PatchDraft context project does not match the run.".to_string());
    }
    let thread = store
        .manager
        .get_thread(&record.thread_id)
        .map_err(|error| format!("{error:?}"))?;
    Ok((record.thread_id.clone(), thread.goal.clone()))
}

fn approval_scope(
    approvals: &ApprovalBridgeState,
    approval_id: &str,
) -> Result<PermissionScopeView, String> {
    let store = approvals
        .store
        .lock()
        .map_err(|_| "Approval bridge lock failed.".to_string())?;
    store
        .records
        .iter()
        .find(|record| record.proposal_id == approval_id)
        .map(|record| record.scope.clone())
        .ok_or_else(|| "PatchDraft context requires a persisted approval scope.".to_string())
}

struct DraftContext {
    files: Vec<String>,
    goal: String,
    scope_paths: Vec<String>,
    steps: Vec<String>,
}

fn draft_context(
    reviews: &ReviewBridgeState,
    run_id: &str,
    thread_goal: &str,
    project: &WorkspaceProjectView,
    plan: &PlanView,
    approval_scope: &PermissionScopeView,
) -> Result<DraftContext, String> {
    let scope_paths = approval_scope.paths.clone().unwrap_or_default();
    if let Some(repair) = repair_context(reviews, run_id, thread_goal, project, &scope_paths)? {
        return Ok(repair);
    }
    if plan.decision != "approved" {
        return Err("PatchDraft context requires an approved persisted plan.".to_string());
    }
    let files = approved_plan_files(plan, project, &scope_paths);
    if files.is_empty() {
        return Err("PatchDraft context found no approved plan files to read.".to_string());
    }
    Ok(DraftContext {
        scope_paths: if scope_paths.is_empty() {
            files.clone()
        } else {
            scope_paths
        },
        files,
        goal: thread_goal.to_string(),
        steps: plan.steps.clone(),
    })
}

fn repair_context(
    reviews: &ReviewBridgeState,
    run_id: &str,
    thread_goal: &str,
    project: &WorkspaceProjectView,
    scope_paths: &[String],
) -> Result<Option<DraftContext>, String> {
    let store = reviews
        .store
        .lock()
        .map_err(|_| "Review bridge lock failed.".to_string())?;
    let Some((path, title, detail, suggested_fix)) = active_repair_finding(&store, run_id, project)
    else {
        return Ok(None);
    };
    if !scope_paths.iter().any(|scope| same_path(scope, &path)) {
        return Ok(None);
    }
    Ok(Some(DraftContext {
        files: vec![path.clone()],
        goal: format!("Repair review finding \"{title}\" for: {thread_goal}"),
        scope_paths: vec![path],
        steps: vec![
            format!("Fix finding: {detail}"),
            format!("Suggested fix: {suggested_fix}"),
        ],
    }))
}

fn active_repair_finding(
    store: &ReviewBridgeStore,
    run_id: &str,
    project: &WorkspaceProjectView,
) -> Option<(String, String, String, String)> {
    let report = store
        .reports
        .iter()
        .rev()
        .find(|report| report.run_id == run_id && report.decision == "revise_requested")?;
    let finding = report.findings.first()?;
    let path = repair_relative_path(&finding.file_path, &project.path)?;
    Some((
        path,
        finding.title.clone(),
        finding.detail.clone(),
        finding.suggested_fix.clone(),
    ))
}

fn approved_plan_files(
    plan: &PlanView,
    project: &WorkspaceProjectView,
    scope_paths: &[String],
) -> Vec<String> {
    plan.files_likely_involved
        .iter()
        .filter(|path| {
            project
                .indexed_files
                .iter()
                .any(|file| same_path(file, path))
        })
        .filter(|path| {
            scope_paths.is_empty() || scope_paths.iter().any(|scope| same_path(scope, path))
        })
        .take(4)
        .cloned()
        .collect()
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
