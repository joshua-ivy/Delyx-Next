#[cfg(test)]
mod tests {
    use crate::plan_bridge::{ExploreView, PlanView};
    use crate::plan_persistence::{load_plans_from_path, save_plan_to_path};
    use std::fs;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn plan_records_survive_reload_and_filter_by_project() {
        let path = temp_db("plan-records");
        let mut plan = sample_plan("thread-1", "pending");
        save_plan_to_path(&path, "project-1", &plan).unwrap();
        save_plan_to_path(&path, "project-2", &sample_plan("thread-2", "approved")).unwrap();

        plan.decision = "approved".to_string();
        plan.steps.push("Run the focused tests.".to_string());
        save_plan_to_path(&path, "project-1", &plan).unwrap();

        let loaded = load_plans_from_path(&path, "project-1").unwrap();
        assert_eq!(loaded.len(), 1);
        assert_eq!(loaded[0].thread_id, "thread-1");
        assert_eq!(loaded[0].decision, "approved");
        assert!(loaded[0].steps.iter().any(|step| step.contains("focused")));

        let other = load_plans_from_path(&path, "project-2").unwrap();
        assert_eq!(other.len(), 1);
        assert_eq!(other[0].thread_id, "thread-2");
        let _ = fs::remove_file(path);
    }

    fn sample_plan(thread_id: &str, decision: &str) -> PlanView {
        PlanView {
            decision: decision.to_string(),
            explore: ExploreView {
                architecture_summary: "Typed local workbench.".to_string(),
                project_commands: vec!["npm test".to_string()],
                relevant_files: vec!["src/main.ts".to_string()],
                relevant_symbols: Vec::new(),
                risks: vec!["Needs approval before writes.".to_string()],
                suggested_next_steps: vec!["Review the plan.".to_string()],
                unknowns: Vec::new(),
            },
            files_likely_involved: vec!["src/main.ts".to_string()],
            goal_understanding: "Persist approved plan state.".to_string(),
            permissions_needed: vec!["edit_file".to_string()],
            rollback_strategy: "Do not apply without patch approval.".to_string(),
            risks: Vec::new(),
            steps: vec!["Persist the plan record.".to_string()],
            tests_to_run: vec!["npm test".to_string()],
            thread_id: thread_id.to_string(),
        }
    }

    fn temp_db(name: &str) -> std::path::PathBuf {
        let stamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        std::env::temp_dir().join(format!("delyx-next-{name}-{stamp}.sqlite3"))
    }
}
