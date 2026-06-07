use crate::approval::{
    ActionProposal, ApprovalDecision, ApprovalDecisionKind, ApprovalEngine, ProposalStatus,
    RiskLevel, RiskyAction,
};
use crate::approval_bridge::{ApprovalBridgeRecord, ApprovalBridgeStore, PermissionScopeView};
use rusqlite::{params, Connection};
use std::path::Path;

pub fn save_to_path(store: &ApprovalBridgeStore, path: &Path) -> Result<(), String> {
    let connection = crate::sqlite_store::open_migrated_database(path).map_err(sql_string)?;
    clear_tables(&connection)?;
    for proposal in store.engine.all_proposals() {
        insert_proposal(&connection, proposal)?;
    }
    for record in &store.records {
        insert_record(&connection, record)?;
    }
    Ok(())
}

pub fn load_from_path(path: &Path) -> Result<ApprovalBridgeStore, String> {
    let connection = crate::sqlite_store::open_migrated_database(path).map_err(sql_string)?;
    let proposals = load_proposals(&connection)?;
    let records = load_records(&connection)?;
    Ok(ApprovalBridgeStore { engine: ApprovalEngine::from_loaded(proposals), records })
}

fn clear_tables(connection: &Connection) -> Result<(), String> {
    connection
        .execute_batch(
            "DELETE FROM approval_bridge_records;
             DELETE FROM action_proposals;",
        )
        .map_err(sql_string)
}

fn insert_proposal(connection: &Connection, proposal: &ActionProposal) -> Result<(), String> {
    let (decision_kind, decision_at, decision_note) = decision_parts(proposal.decision.as_ref());
    connection
        .execute(
            "INSERT INTO action_proposals
             (id, run_id, node_id, action, risk, scope, reason, expected_result, rollback_plan, expires_at, status, decision_kind, decision_at, decision_note)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14)",
            params![
                proposal.id,
                proposal.run_id,
                proposal.node_id,
                action_key(proposal.action),
                risk_key(proposal.risk),
                proposal.scope,
                proposal.reason,
                proposal.expected_result,
                proposal.rollback_plan,
                proposal.expires_at as i64,
                status_key(proposal.status),
                decision_kind,
                decision_at,
                decision_note,
            ],
        )
        .map(|_| ())
        .map_err(sql_string)
}

fn insert_record(connection: &Connection, record: &ApprovalBridgeRecord) -> Result<(), String> {
    let scope_json = serde_json::to_string(&record.scope).map_err(|error| error.to_string())?;
    connection
        .execute(
            "INSERT INTO approval_bridge_records
             (client_id, proposal_id, run_id, action_type, required_permission, expires_at, scope_json)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params![
                record.client_id,
                record.proposal_id,
                record.run_id,
                record.action_type,
                record.required_permission,
                record.expires_at,
                scope_json,
            ],
        )
        .map(|_| ())
        .map_err(sql_string)
}

fn load_proposals(connection: &Connection) -> Result<Vec<ActionProposal>, String> {
    let mut statement = connection
        .prepare(
            "SELECT id, run_id, node_id, action, risk, scope, reason, expected_result,
                    rollback_plan, expires_at, status, decision_kind, decision_at, decision_note
             FROM action_proposals ORDER BY rowid",
        )
        .map_err(sql_string)?;
    let mut rows = statement.query([]).map_err(sql_string)?;
    let mut proposals = Vec::new();
    while let Some(row) = rows.next().map_err(sql_string)? {
        let action_value: String = row.get(3).map_err(sql_string)?;
        let risk_value: String = row.get(4).map_err(sql_string)?;
        let status_value: String = row.get(10).map_err(sql_string)?;
        proposals.push(ActionProposal {
            id: row.get(0).map_err(sql_string)?,
            run_id: row.get(1).map_err(sql_string)?,
            node_id: row.get(2).map_err(sql_string)?,
            action: parse_action(&action_value)?,
            risk: parse_risk(&risk_value)?,
            scope: row.get(5).map_err(sql_string)?,
            reason: row.get(6).map_err(sql_string)?,
            expected_result: row.get(7).map_err(sql_string)?,
            rollback_plan: row.get(8).map_err(sql_string)?,
            expires_at: row.get::<_, i64>(9).map_err(sql_string)? as u64,
            status: parse_status(&status_value)?,
            decision: load_decision(row.get(11).map_err(sql_string)?, row.get(12).map_err(sql_string)?, row.get(13).map_err(sql_string)?)?,
        });
    }
    Ok(proposals)
}

fn load_records(connection: &Connection) -> Result<Vec<ApprovalBridgeRecord>, String> {
    let mut statement = connection
        .prepare(
            "SELECT client_id, proposal_id, run_id, action_type, required_permission, expires_at, scope_json
             FROM approval_bridge_records ORDER BY rowid",
        )
        .map_err(sql_string)?;
    let mut rows = statement.query([]).map_err(sql_string)?;
    let mut records = Vec::new();
    while let Some(row) = rows.next().map_err(sql_string)? {
        let scope_json: String = row.get(6).map_err(sql_string)?;
        records.push(ApprovalBridgeRecord {
            client_id: row.get(0).map_err(sql_string)?,
            proposal_id: row.get(1).map_err(sql_string)?,
            run_id: row.get(2).map_err(sql_string)?,
            action_type: row.get(3).map_err(sql_string)?,
            required_permission: row.get(4).map_err(sql_string)?,
            expires_at: row.get(5).map_err(sql_string)?,
            scope: serde_json::from_str::<PermissionScopeView>(&scope_json).map_err(|error| error.to_string())?,
        });
    }
    Ok(records)
}

fn decision_parts(decision: Option<&ApprovalDecision>) -> (Option<&'static str>, Option<i64>, Option<&str>) {
    match decision {
        Some(decision) => (
            Some(match decision.kind {
                ApprovalDecisionKind::Approve => "approved",
                ApprovalDecisionKind::Deny => "denied",
            }),
            Some(decision.decided_at as i64),
            Some(decision.note.as_str()),
        ),
        None => (None, None, None),
    }
}

fn load_decision(kind: Option<String>, decided_at: Option<i64>, note: Option<String>) -> Result<Option<ApprovalDecision>, String> {
    let Some(kind) = kind else {
        return Ok(None);
    };
    Ok(Some(ApprovalDecision {
        kind: match kind.as_str() {
            "approved" => ApprovalDecisionKind::Approve,
            "denied" => ApprovalDecisionKind::Deny,
            _ => return Err("Unsupported approval decision kind.".to_string()),
        },
        decided_at: decided_at.unwrap_or_default() as u64,
        note: note.unwrap_or_default(),
    }))
}

fn action_key(action: RiskyAction) -> &'static str {
    match action {
        RiskyAction::ConnectorWrite => "connector_write",
        RiskyAction::DependencyInstall => "dependency_install",
        RiskyAction::DurableMemorySave => "durable_memory_save",
        RiskyAction::ExternalAgentExecution => "external_agent",
        RiskyAction::ExternalSend => "external_send",
        RiskyAction::FileWrite => "file_write",
        RiskyAction::ScheduledRiskyAction => "scheduled_risky_action",
        RiskyAction::TerminalCommand => "terminal_command",
    }
}

fn parse_action(value: &str) -> Result<RiskyAction, String> {
    match value {
        "connector_write" => Ok(RiskyAction::ConnectorWrite),
        "dependency_install" => Ok(RiskyAction::DependencyInstall),
        "durable_memory_save" => Ok(RiskyAction::DurableMemorySave),
        "external_agent" => Ok(RiskyAction::ExternalAgentExecution),
        "external_send" => Ok(RiskyAction::ExternalSend),
        "file_write" => Ok(RiskyAction::FileWrite),
        "scheduled_risky_action" => Ok(RiskyAction::ScheduledRiskyAction),
        "terminal_command" => Ok(RiskyAction::TerminalCommand),
        _ => Err("Unsupported persisted approval action.".to_string()),
    }
}

fn risk_key(risk: RiskLevel) -> &'static str {
    match risk {
        RiskLevel::Dangerous => "dangerous",
        RiskLevel::High => "high",
        RiskLevel::Low => "low",
        RiskLevel::Medium => "medium",
    }
}

fn parse_risk(value: &str) -> Result<RiskLevel, String> {
    match value {
        "dangerous" => Ok(RiskLevel::Dangerous),
        "high" => Ok(RiskLevel::High),
        "low" => Ok(RiskLevel::Low),
        "medium" => Ok(RiskLevel::Medium),
        _ => Err("Unsupported persisted approval risk.".to_string()),
    }
}

fn status_key(status: ProposalStatus) -> &'static str {
    match status {
        ProposalStatus::Approved => "approved",
        ProposalStatus::Denied => "denied",
        ProposalStatus::Expired => "expired",
        ProposalStatus::Pending => "pending",
    }
}

fn parse_status(value: &str) -> Result<ProposalStatus, String> {
    match value {
        "approved" => Ok(ProposalStatus::Approved),
        "denied" => Ok(ProposalStatus::Denied),
        "expired" => Ok(ProposalStatus::Expired),
        "pending" => Ok(ProposalStatus::Pending),
        _ => Err("Unsupported persisted approval status.".to_string()),
    }
}

fn sql_string(error: rusqlite::Error) -> String {
    error.to_string()
}
