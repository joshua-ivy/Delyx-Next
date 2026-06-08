use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug)]
pub struct PlanBridgeState {
    database_path: PathBuf,
}

impl PlanBridgeState {
    pub fn persistent(database_path: PathBuf) -> Self {
        Self { database_path }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ExploreView {
    pub relevant_files: Vec<String>,
    pub relevant_symbols: Vec<String>,
    pub architecture_summary: String,
    pub project_commands: Vec<String>,
    pub risks: Vec<String>,
    pub unknowns: Vec<String>,
    pub suggested_next_steps: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PlanView {
    pub thread_id: String,
    pub goal_understanding: String,
    pub files_likely_involved: Vec<String>,
    pub steps: Vec<String>,
    pub risks: Vec<String>,
    pub tests_to_run: Vec<String>,
    pub permissions_needed: Vec<String>,
    pub rollback_strategy: String,
    pub decision: String,
    pub explore: ExploreView,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PlanSaveRequest {
    pub project_id: String,
    pub plan: PlanView,
}

#[tauri::command]
pub fn plan_save(
    state: tauri::State<PlanBridgeState>,
    request: PlanSaveRequest,
) -> Result<PlanView, String> {
    crate::plan_persistence::save_plan_to_path(
        &state.database_path,
        &request.project_id,
        &request.plan,
    )?;
    Ok(request.plan)
}

#[tauri::command]
pub fn plan_snapshot(
    state: tauri::State<PlanBridgeState>,
    project_id: String,
) -> Result<Vec<PlanView>, String> {
    crate::plan_persistence::load_plans_from_path(&state.database_path, &project_id)
}
