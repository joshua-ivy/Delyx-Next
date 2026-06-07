#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};

    use crate::explore_plan::{ExploreAgent, ExplorePlanError, PlanAgent, PlanDecision, ToolCapability};
    use crate::workspace::{WorkspaceError, WorkspaceManager};

    #[test]
    fn explore_and_plan_modes_are_read_only() {
        assert_eq!(
            ExploreAgent::capabilities(),
            &[ToolCapability::SearchApprovedFiles, ToolCapability::ReadApprovedFile]
        );
        assert!(!PlanAgent::capabilities().contains(&ToolCapability::EditFile));
        assert!(!PlanAgent::capabilities().contains(&ToolCapability::RunTerminal));
    }

    #[test]
    fn explore_searches_approved_workspace_and_discovers_commands() {
        let fixture = Fixture::new("explore");
        fixture.file("Cargo.toml", "[workspace]\n");
        fixture.file("src/thread_policy.rs", "pub struct ThreadPolicy {}\n");
        let (manager, project_id) = linked_workspace(&fixture);

        let output = ExploreAgent::explore(&manager, &project_id, "thread policy update").unwrap();

        assert_eq!(output.relevant_files, vec!["src/thread_policy.rs"]);
        assert_eq!(output.relevant_symbols, vec!["pub struct ThreadPolicy {}"]);
        assert_eq!(output.project_commands, vec!["cargo test --workspace"]);
        assert!(output.architecture_summary.contains("Rust workspace"));
        assert!(output.suggested_next_steps[0].contains("Create a plan"));
    }

    #[test]
    fn explore_read_outside_workspace_fails() {
        let fixture = Fixture::new("outside");
        fixture.file("inside.txt", "inside");
        let outside = fixture.sibling("outside.txt");
        fs::write(&outside, "outside").unwrap();
        let (manager, project_id) = linked_workspace(&fixture);

        assert_eq!(
            ExploreAgent::read_file(&manager, &project_id, &outside).unwrap_err(),
            ExplorePlanError::Workspace(WorkspaceError::OutsideApprovedRoot)
        );
    }

    #[test]
    fn plan_output_contains_goal_files_tests_and_permissions() {
        let fixture = Fixture::new("plan");
        fixture.file("package.json", "{\"scripts\":{\"test\":\"vitest\"}}\n");
        fixture.file("src/plan_model.ts", "export interface PlanModel {}\n");
        let (manager, project_id) = linked_workspace(&fixture);
        let explore = ExploreAgent::explore(&manager, &project_id, "plan model").unwrap();
        let plan = PlanAgent::create_plan("Add plan model states", &explore).unwrap();

        assert_eq!(plan.goal_understanding, "Add plan model states");
        assert_eq!(plan.files_likely_involved, vec!["src/plan_model.ts"]);
        assert_eq!(plan.tests_to_run, vec!["npm test"]);
        assert!(plan.permissions_needed.iter().any(|item| item.contains("approval required")));
        assert!(plan.rollback_strategy.contains("checkpoint"));
    }

    #[test]
    fn user_can_approve_revise_or_cancel_plan() {
        let explore = crate::explore_plan::ExploreOutput {
            relevant_files: Vec::new(),
            relevant_symbols: Vec::new(),
            architecture_summary: "No dominant stack detected from approved project files.".to_string(),
            project_commands: Vec::new(),
            risks: Vec::new(),
            unknowns: Vec::new(),
            suggested_next_steps: Vec::new(),
        };
        let mut plan = PlanAgent::create_plan("Inspect current project", &explore).unwrap();

        PlanAgent::approve(&mut plan);
        assert_eq!(plan.decision, PlanDecision::Approved);
        PlanAgent::request_revision(&mut plan);
        assert_eq!(plan.decision, PlanDecision::RevisionRequested);
        PlanAgent::cancel(&mut plan);
        assert_eq!(plan.decision, PlanDecision::Cancelled);
    }

    fn linked_workspace(fixture: &Fixture) -> (WorkspaceManager, String) {
        let mut manager = WorkspaceManager::new();
        let project = manager.add_project(fixture.root()).unwrap();
        (manager, project.id)
    }

    struct Fixture {
        root: PathBuf,
    }

    impl Fixture {
        fn new(name: &str) -> Self {
            let stamp = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos();
            let root = std::env::temp_dir().join(format!("delyx-next-{name}-{stamp}"));
            fs::create_dir_all(&root).unwrap();
            Self { root }
        }

        fn root(&self) -> &PathBuf {
            &self.root
        }

        fn sibling(&self, name: &str) -> PathBuf {
            self.root.parent().unwrap().join(format!("{}-{name}", self.root.file_name().unwrap().to_string_lossy()))
        }

        fn file(&self, relative_path: &str, contents: &str) {
            let path = self.root.join(relative_path);
            fs::create_dir_all(path.parent().unwrap()).unwrap();
            fs::write(path, contents).unwrap();
        }
    }

    impl Drop for Fixture {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.root);
        }
    }
}
