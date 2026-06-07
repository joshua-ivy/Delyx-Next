pub mod agent_run;
pub mod agent_run_persistence;
pub mod approval;
pub mod automation;
pub mod external_agent;
pub mod external_agent_terminal;
pub mod explore_plan;
pub mod memory;
pub mod mobile;
pub mod model_provider;
pub mod patch;
pub mod release;
pub mod research;
pub mod review;
pub mod skills;
pub mod test_runner;
pub mod threads;
pub mod workspace;
pub mod workbench_modes;
mod agent_run_tests;
mod approval_tests;
mod automation_tests;
mod external_agent_tests;
mod explore_plan_tests;
mod memory_tests;
mod mobile_tests;
mod model_provider_tests;
mod patch_tests;
mod release_tests;
mod research_tests;
mod review_tests;
mod skills_tests;
mod test_runner_tests;
mod threads_tests;
mod workspace_tests;
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
