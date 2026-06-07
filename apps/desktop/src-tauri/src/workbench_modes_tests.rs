#[cfg(test)]
mod tests {
    use crate::workbench_modes::{
        authorize_build_mode, authorize_tested_claim, BuildModeGate, WorkbenchMode, WorkbenchModeError,
    };

    #[test]
    fn build_mode_requires_approved_plan_or_direct_instruction() {
        let blocked = BuildModeGate { approved_plan: false, direct_user_instruction: false };
        let approved_plan = BuildModeGate { approved_plan: true, direct_user_instruction: false };
        let direct = BuildModeGate { approved_plan: false, direct_user_instruction: true };

        assert_eq!(authorize_build_mode(blocked), Err(WorkbenchModeError::BuildRequiresApprovedPlanOrDirectInstruction));
        assert_eq!(authorize_build_mode(approved_plan), Ok(WorkbenchMode::Build));
        assert_eq!(authorize_build_mode(direct), Ok(WorkbenchMode::Build));
    }

    #[test]
    fn tested_claim_requires_execution_artifact() {
        assert_eq!(authorize_tested_claim(false), Err(WorkbenchModeError::TestClaimMissingExecutionArtifact));
        assert_eq!(authorize_tested_claim(true), Ok(()));
    }
}
