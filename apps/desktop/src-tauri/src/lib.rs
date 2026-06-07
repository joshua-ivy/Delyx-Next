pub mod agent_run;
pub mod agent_run_persistence;
pub mod approval;
pub mod approval_bridge;
pub mod automation;
pub mod external_agent;
mod external_agent_adapters;
pub mod external_agent_contract_bridge;
pub mod external_agent_command_contracts;
mod external_agent_scope;
pub mod external_agent_status_bridge;
pub mod external_agent_terminal;
pub mod explore_plan;
pub mod memory;
pub mod mobile;
mod model_ollama;
pub mod model_provider;
pub mod patch;
pub mod patch_bridge;
pub mod release;
pub mod research;
pub mod review;
pub mod runtime_bridge;
pub mod skills;
pub mod test_runner;
pub mod test_runner_bridge;
pub mod thread_run_bridge;
pub mod thread_run_bridge_views;
pub mod threads;
pub mod workspace;
pub mod workspace_bridge;
mod workspace_git;
pub mod workbench_modes;
mod agent_run_tests;
mod approval_bridge_tests;
mod approval_tests;
mod automation_tests;
mod external_agent_command_contracts_tests;
mod external_agent_contract_bridge_tests;
mod external_agent_status_bridge_tests;
mod external_agent_tests;
mod external_agent_terminal_tests;
mod explore_plan_tests;
mod memory_tests;
mod mobile_tests;
mod model_ollama_tests;
mod model_provider_tests;
mod patch_tests;
mod patch_bridge_tests;
mod release_tests;
mod research_tests;
mod review_tests;
mod runtime_bridge_tests;
mod skills_tests;
mod test_runner_tests;
mod test_runner_bridge_tests;
mod thread_run_bridge_tests;
mod threads_tests;
mod workspace_tests;
mod workspace_bridge_tests;
mod workspace_git_tests;
mod workbench_modes_tests;

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
