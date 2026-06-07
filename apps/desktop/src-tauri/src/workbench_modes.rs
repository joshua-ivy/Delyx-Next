#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WorkbenchMode {
    Explore,
    Plan,
    Build,
    Test,
    Review,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BuildModeGate {
    pub approved_plan: bool,
    pub direct_user_instruction: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WorkbenchModeError {
    BuildRequiresApprovedPlanOrDirectInstruction,
    TestClaimMissingExecutionArtifact,
}

pub fn authorize_build_mode(gate: BuildModeGate) -> Result<WorkbenchMode, WorkbenchModeError> {
    if gate.approved_plan || gate.direct_user_instruction {
        return Ok(WorkbenchMode::Build);
    }
    Err(WorkbenchModeError::BuildRequiresApprovedPlanOrDirectInstruction)
}

pub fn authorize_tested_claim(has_execution_artifact: bool) -> Result<(), WorkbenchModeError> {
    if has_execution_artifact {
        return Ok(());
    }
    Err(WorkbenchModeError::TestClaimMissingExecutionArtifact)
}
