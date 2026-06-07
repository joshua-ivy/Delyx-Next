use crate::approval::{ApprovalEngine, ApprovalError, ProposalInput, RiskLevel, RiskyAction};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MissionContract {
    pub id: String,
    pub title: String,
    pub status: MissionStatus,
    pub scope: String,
    pub allowed_tools: Vec<String>,
    pub active_hours: ActiveHours,
    pub timezone: String,
    pub delivery_targets: Vec<String>,
    pub stop_condition: String,
    pub workspace_fingerprint: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MissionStatus {
    Paused,
    Active,
    Blocked,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ActiveHours {
    pub start_hour: u8,
    pub end_hour: u8,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScheduledRun {
    pub id: String,
    pub contract_id: String,
    pub status: ScheduledRunStatus,
    pub reason: String,
    pub approval_id: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScheduledRunStatus {
    Created,
    WaitingForApproval,
    Blocked,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AutomationError {
    Approval(ApprovalError),
    ContractNotFound,
}

#[derive(Debug, Default)]
pub struct AutomationEngine {
    contracts: Vec<MissionContract>,
    next_contract_id: usize,
    next_run_id: usize,
}

impl AutomationEngine {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn create_contract(&mut self, input: MissionContractInput) -> MissionContract {
        self.next_contract_id += 1;
        let contract = MissionContract {
            id: format!("mission-{}", self.next_contract_id),
            title: input.title,
            status: MissionStatus::Paused,
            scope: input.scope,
            allowed_tools: input.allowed_tools,
            active_hours: input.active_hours,
            timezone: input.timezone,
            delivery_targets: input.delivery_targets,
            stop_condition: input.stop_condition,
            workspace_fingerprint: input.workspace_fingerprint,
        };
        self.contracts.push(contract.clone());
        contract
    }

    pub fn approve_contract(&mut self, contract_id: &str, approval_id: &str, now: u64, approvals: &ApprovalEngine) -> Result<(), AutomationError> {
        approvals
            .assert_can_execute_action(approval_id, now, RiskyAction::ScheduledRiskyAction)
            .map_err(AutomationError::Approval)?;
        self.contract_mut(contract_id)?.status = MissionStatus::Active;
        Ok(())
    }

    pub fn schedule_due_run(
        &mut self,
        contract_id: &str,
        workspace_fingerprint: &str,
        now: u64,
        approvals: &mut ApprovalEngine,
    ) -> Result<ScheduledRun, AutomationError> {
        let contract = self.contract(contract_id)?.clone();
        if contract.status != MissionStatus::Active {
            return Ok(self.run(contract_id, ScheduledRunStatus::Blocked, "Contract is paused or blocked.", None));
        }
        if contract.workspace_fingerprint != workspace_fingerprint {
            return Ok(self.run(contract_id, ScheduledRunStatus::Blocked, "Workspace drift blocks scheduled work.", None));
        }
        if contract.allowed_tools.iter().any(|tool| risky_tool(tool)) {
            let approval = approvals.propose(ProposalInput {
                action: RiskyAction::ScheduledRiskyAction,
                expires_at: now + 900,
                expected_result: "Scheduled risky action may run after approval.".to_string(),
                node_id: format!("automation-node-{}", contract.id),
                reason: format!("Mission contract {} requested a risky scheduled action.", contract.id),
                risk: RiskLevel::High,
                rollback_plan: "Do not run the scheduled action; leave contract paused or revise scope.".to_string(),
                run_id: contract.id.clone(),
                scope: contract.scope.clone(),
            });
            return Ok(self.run(contract_id, ScheduledRunStatus::WaitingForApproval, "Approval required before scheduled execution.", Some(approval.id)));
        }
        Ok(self.run(contract_id, ScheduledRunStatus::Created, "Scheduled run created.", None))
    }

    pub fn contracts(&self) -> &[MissionContract] {
        &self.contracts
    }

    fn contract(&self, contract_id: &str) -> Result<&MissionContract, AutomationError> {
        self.contracts.iter().find(|contract| contract.id == contract_id).ok_or(AutomationError::ContractNotFound)
    }

    fn contract_mut(&mut self, contract_id: &str) -> Result<&mut MissionContract, AutomationError> {
        self.contracts.iter_mut().find(|contract| contract.id == contract_id).ok_or(AutomationError::ContractNotFound)
    }

    fn run(&mut self, contract_id: &str, status: ScheduledRunStatus, reason: &str, approval_id: Option<String>) -> ScheduledRun {
        self.next_run_id += 1;
        ScheduledRun { id: format!("scheduled-run-{}", self.next_run_id), contract_id: contract_id.to_string(), status, reason: reason.to_string(), approval_id }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MissionContractInput {
    pub title: String,
    pub scope: String,
    pub allowed_tools: Vec<String>,
    pub active_hours: ActiveHours,
    pub timezone: String,
    pub delivery_targets: Vec<String>,
    pub stop_condition: String,
    pub workspace_fingerprint: String,
}

fn risky_tool(tool: &str) -> bool {
    matches!(tool, "file_write" | "terminal_command" | "external_agent")
}
