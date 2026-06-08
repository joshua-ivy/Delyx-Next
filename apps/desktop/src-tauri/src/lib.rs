pub mod agent_run;
pub mod agent_run_commands;
mod agent_run_evidence_persistence;
mod agent_run_ids;
mod agent_run_outcome;
pub mod agent_run_persistence;
mod agent_run_tests;
mod agent_run_types;
pub mod approval;
pub mod approval_bridge;
mod approval_bridge_keys;
pub mod approval_bridge_taxonomy;
mod approval_bridge_tests;
pub mod approval_persistence;
mod approval_persistence_tests;
mod approval_policy;
mod approval_tests;
mod approval_types;
pub mod automation;
pub mod automation_bridge;
pub mod automation_persistence;
mod automation_persistence_tests;
mod automation_tests;
pub mod command_exec;
mod command_exec_tests;
pub mod explore_plan;
mod explore_plan_tests;
pub mod external_agent;
mod external_agent_adapters;
pub mod external_agent_command_contracts;
mod external_agent_command_contracts_tests;
pub mod external_agent_contract_bridge;
mod external_agent_contract_bridge_tests;
mod external_agent_guard_tests;
pub mod external_agent_run_bridge;
mod external_agent_run_bridge_keys;
mod external_agent_run_bridge_tests;
pub mod external_agent_run_persistence;
mod external_agent_run_persistence_tests;
mod external_agent_scope;
pub mod external_agent_status_bridge;
mod external_agent_status_bridge_tests;
pub mod external_agent_terminal;
mod external_agent_terminal_tests;
mod external_agent_tests;
mod external_agent_types;
pub mod local_store_bridge;
mod local_store_bridge_tests;
pub mod memory;
pub mod memory_bridge;
mod memory_bridge_tests;
mod memory_bridge_views;
pub mod memory_persistence;
mod memory_persistence_tests;
mod memory_tests;
pub mod mobile;
mod mobile_tests;
mod model_ollama;
mod model_ollama_tests;
pub mod model_provider;
pub mod model_provider_persistence;
mod model_provider_persistence_tests;
mod model_provider_tests;
pub mod patch;
pub mod patch_bridge;
mod patch_bridge_tests;
pub mod patch_persistence;
mod patch_persistence_tests;
mod patch_tests;
pub mod release;
pub mod release_bridge;
pub mod release_persistence;
mod release_persistence_tests;
mod release_tests;
pub mod research;
pub mod research_persistence;
mod research_persistence_tests;
mod research_tests;
pub mod review;
pub mod review_bridge;
mod review_bridge_keys;
mod review_bridge_tests;
mod review_helpers;
pub mod review_persistence;
mod review_persistence_tests;
mod review_tests;
pub mod runtime_bridge;
mod runtime_bridge_tests;
pub mod skills;
pub mod skills_bridge;
pub mod skills_persistence;
mod skills_persistence_tests;
mod skills_tests;
pub mod sqlite_store;
mod sqlite_store_tests;
pub mod terminal_command_prep;
mod terminal_command_prep_tests;
pub mod test_runner;
pub mod test_runner_bridge;
mod test_runner_bridge_tests;
pub mod test_runner_persistence;
mod test_runner_persistence_tests;
mod test_runner_tests;
pub mod thread_run_bridge;
mod thread_run_bridge_parse;
mod thread_run_bridge_state;
mod thread_run_bridge_tests;
pub mod thread_run_bridge_views;
pub mod thread_run_persistence;
mod thread_run_persistence_tests;
pub mod threads;
mod threads_tests;
pub mod workbench_modes;
mod workbench_modes_tests;
pub mod workspace;
pub mod workspace_bridge;
mod workspace_bridge_tests;
mod workspace_git;
mod workspace_git_tests;
pub mod workspace_persistence;
mod workspace_persistence_tests;
mod workspace_tests;

pub const APP_NAME: &str = "Delyx Next";
pub const APP_IDENTIFIER: &str = "com.geaux.delyxnext";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DesktopShellInfo {
    pub name: &'static str,
    pub identifier: &'static str,
    pub milestone: &'static str,
}

pub fn desktop_shell_info() -> DesktopShellInfo {
    DesktopShellInfo {
        name: APP_NAME,
        identifier: APP_IDENTIFIER,
        milestone: "PR 18 Packaging and release",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn exposes_separate_app_identity() {
        let info = desktop_shell_info();

        assert_eq!(info.name, "Delyx Next");
        assert_eq!(info.identifier, "com.geaux.delyxnext");
    }
}
