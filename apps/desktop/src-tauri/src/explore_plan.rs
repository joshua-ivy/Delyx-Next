use std::path::Path;

use crate::workspace::{FileIndexEntry, WorkspaceError, WorkspaceManager};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ToolCapability {
    SearchApprovedFiles,
    ReadApprovedFile,
    EditFile,
    RunTerminal,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExploreOutput {
    pub relevant_files: Vec<String>,
    pub relevant_symbols: Vec<String>,
    pub architecture_summary: String,
    pub project_commands: Vec<String>,
    pub risks: Vec<String>,
    pub unknowns: Vec<String>,
    pub suggested_next_steps: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlanOutput {
    pub goal_understanding: String,
    pub files_likely_involved: Vec<String>,
    pub steps: Vec<String>,
    pub risks: Vec<String>,
    pub tests_to_run: Vec<String>,
    pub permissions_needed: Vec<String>,
    pub rollback_strategy: String,
    pub decision: PlanDecision,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlanDecision {
    Pending,
    Approved,
    RevisionRequested,
    Cancelled,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ExplorePlanError {
    EmptyGoal,
    Workspace(WorkspaceError),
}

impl From<WorkspaceError> for ExplorePlanError {
    fn from(value: WorkspaceError) -> Self {
        Self::Workspace(value)
    }
}

pub struct ExploreAgent;
pub struct PlanAgent;

impl ExploreAgent {
    pub fn capabilities() -> &'static [ToolCapability] {
        &[
            ToolCapability::SearchApprovedFiles,
            ToolCapability::ReadApprovedFile,
        ]
    }

    pub fn search(
        manager: &WorkspaceManager,
        project_id: &str,
        query: &str,
    ) -> Result<Vec<FileIndexEntry>, ExplorePlanError> {
        Ok(manager.search_files(project_id, query)?)
    }

    pub fn read_file(
        manager: &WorkspaceManager,
        project_id: &str,
        path: &Path,
    ) -> Result<String, ExplorePlanError> {
        Ok(manager.read_file(project_id, path)?)
    }

    pub fn explore(
        manager: &WorkspaceManager,
        project_id: &str,
        goal: &str,
    ) -> Result<ExploreOutput, ExplorePlanError> {
        let goal = goal.trim();
        if goal.is_empty() {
            return Err(ExplorePlanError::EmptyGoal);
        }

        let files = relevant_files(manager, project_id, goal)?;
        let commands = discover_commands(manager, project_id)?;
        let symbols = discover_symbols(manager, project_id, &files)?;
        let unknowns = if files.is_empty() {
            vec!["No matching approved files found.".to_string()]
        } else {
            Vec::new()
        };
        let suggested_next_steps = suggested_next_steps(&files);

        Ok(ExploreOutput {
            relevant_files: files,
            relevant_symbols: symbols,
            architecture_summary: architecture_summary(&commands),
            project_commands: commands,
            risks: vec!["Explore mode is read-only; edits require a later approval.".to_string()],
            unknowns,
            suggested_next_steps,
        })
    }
}

impl PlanAgent {
    pub fn capabilities() -> &'static [ToolCapability] {
        &[
            ToolCapability::SearchApprovedFiles,
            ToolCapability::ReadApprovedFile,
        ]
    }

    pub fn create_plan(
        goal: &str,
        explore: &ExploreOutput,
    ) -> Result<PlanOutput, ExplorePlanError> {
        let goal = goal.trim();
        if goal.is_empty() {
            return Err(ExplorePlanError::EmptyGoal);
        }

        Ok(PlanOutput {
            goal_understanding: goal.to_string(),
            files_likely_involved: explore.relevant_files.clone(),
            steps: plan_steps(explore),
            risks: explore.risks.iter().chain(explore.unknowns.iter()).cloned().collect(),
            tests_to_run: tests_from_commands(&explore.project_commands),
            permissions_needed: vec![
                "read approved workspace files".to_string(),
                "approval required before file edits".to_string(),
                "approval required before terminal commands".to_string(),
            ],
            rollback_strategy: "Create or reuse a checkpoint before edits, then restore that checkpoint if review rejects the diff.".to_string(),
            decision: PlanDecision::Pending,
        })
    }

    pub fn approve(plan: &mut PlanOutput) {
        plan.decision = PlanDecision::Approved;
    }

    pub fn request_revision(plan: &mut PlanOutput) {
        plan.decision = PlanDecision::RevisionRequested;
    }

    pub fn cancel(plan: &mut PlanOutput) {
        plan.decision = PlanDecision::Cancelled;
    }
}

fn relevant_files(
    manager: &WorkspaceManager,
    project_id: &str,
    goal: &str,
) -> Result<Vec<String>, ExplorePlanError> {
    let mut files = Vec::new();
    for term in search_terms(goal) {
        for entry in manager.search_files(project_id, &term)? {
            if !files.contains(&entry.relative_path) {
                files.push(entry.relative_path);
            }
            if files.len() >= 6 {
                return Ok(files);
            }
        }
    }
    Ok(files)
}

fn discover_commands(
    manager: &WorkspaceManager,
    project_id: &str,
) -> Result<Vec<String>, ExplorePlanError> {
    let files = manager.index_files(project_id, 200)?;
    let mut commands = Vec::new();
    if files.iter().any(|file| file.relative_path == "Cargo.toml") {
        commands.push("cargo test --workspace".to_string());
    }
    if files
        .iter()
        .any(|file| file.relative_path == "package.json")
    {
        commands.push("npm test".to_string());
    }
    Ok(commands)
}

fn discover_symbols(
    manager: &WorkspaceManager,
    project_id: &str,
    files: &[String],
) -> Result<Vec<String>, ExplorePlanError> {
    let index = manager.index_files(project_id, 500)?;
    let mut symbols = Vec::new();
    for file in files.iter().take(4) {
        if let Some(entry) = index.iter().find(|entry| &entry.relative_path == file) {
            let contents = manager.read_file(project_id, &entry.path)?;
            symbols.extend(symbol_lines(&contents).into_iter().take(4));
        }
    }
    Ok(symbols)
}

fn symbol_lines(contents: &str) -> Vec<String> {
    contents
        .lines()
        .map(str::trim)
        .filter(|line| {
            line.starts_with("pub struct ")
                || line.starts_with("pub enum ")
                || line.starts_with("export interface ")
                || line.starts_with("export function ")
        })
        .map(ToString::to_string)
        .collect()
}

fn search_terms(goal: &str) -> Vec<String> {
    goal.split(|value: char| !value.is_ascii_alphanumeric())
        .filter(|term| term.len() >= 4)
        .map(|term| term.to_ascii_lowercase())
        .collect()
}

fn plan_steps(explore: &ExploreOutput) -> Vec<String> {
    let first = if explore.relevant_files.is_empty() {
        "Review approved workspace index for a better file target."
    } else {
        "Review the relevant approved files listed by Explore."
    };
    vec![
        first.to_string(),
        "Draft a narrow change proposal without editing files.".to_string(),
        "Request explicit approval before any file write or command.".to_string(),
    ]
}

fn tests_from_commands(commands: &[String]) -> Vec<String> {
    if commands.is_empty() {
        return vec!["No project test command discovered yet.".to_string()];
    }

    commands.to_vec()
}

fn architecture_summary(commands: &[String]) -> String {
    if commands.iter().any(|command| command.contains("cargo")) {
        return "Rust workspace detected from approved project files.".to_string();
    }
    if commands.iter().any(|command| command.contains("npm")) {
        return "TypeScript or JavaScript workspace detected from approved project files."
            .to_string();
    }
    "No dominant stack detected from approved project files.".to_string()
}

fn suggested_next_steps(files: &[String]) -> Vec<String> {
    if files.is_empty() {
        return vec!["Refine the goal or approve a narrower file search.".to_string()];
    }
    vec!["Create a plan from the relevant files before requesting edits.".to_string()]
}
