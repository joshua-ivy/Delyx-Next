use crate::agent_scheduler_bridge::AgentScheduleRequest;
use crate::approval::RiskyAction;
use crate::approval_bridge::ApprovalBridgeStore;
use crate::plan_bridge::PlanView;
use crate::thread_run_bridge::ThreadRunStore;
use std::path::Path;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct RunnableTestCommand {
    pub(crate) args: Vec<String>,
    pub(crate) label: String,
    pub(crate) program: String,
}

pub(crate) fn hydrate_schedule_request(
    threads: &ThreadRunStore,
    approvals: &ApprovalBridgeStore,
    plan_db: &Path,
    request: AgentScheduleRequest,
) -> Result<AgentScheduleRequest, String> {
    let Some(command) = test_command_for_run(threads, plan_db, &request.run_id)? else {
        return Ok(AgentScheduleRequest {
            has_supported_test_command: false,
            test_approval_id: None,
            ..request
        });
    };
    Ok(AgentScheduleRequest {
        has_supported_test_command: true,
        test_approval_id: ready_test_approval(approvals, &request.run_id, &command, request.now_ms),
        ..request
    })
}

pub(crate) fn test_command_for_run(
    threads: &ThreadRunStore,
    plan_db: &Path,
    run_id: &str,
) -> Result<Option<RunnableTestCommand>, String> {
    let Some(plan) = plan_for_run(threads, plan_db, run_id)? else {
        return Ok(None);
    };
    Ok(first_runnable_test_command(&plan.tests_to_run))
}

pub(crate) fn plan_for_run(
    threads: &ThreadRunStore,
    plan_db: &Path,
    run_id: &str,
) -> Result<Option<PlanView>, String> {
    let Some(record) = threads.records.iter().find(|item| item.run_id == run_id) else {
        return Ok(None);
    };
    Ok(
        crate::plan_persistence::load_plans_from_path(plan_db, &record.project_id)?
            .into_iter()
            .find(|plan| plan.thread_id == record.thread_id),
    )
}

fn ready_test_approval(
    approvals: &ApprovalBridgeStore,
    run_id: &str,
    command: &RunnableTestCommand,
    now_ms: u64,
) -> Option<String> {
    approvals
        .records
        .iter()
        .find(|record| {
            record.run_id == run_id
                && record.action_type == "run_terminal"
                && command_in_scope(record.scope.commands.as_deref(), &command.label)
                && approvals
                    .engine
                    .assert_can_execute_action_for_run(
                        &record.proposal_id,
                        now_ms,
                        RiskyAction::TerminalCommand,
                        run_id,
                    )
                    .is_ok()
        })
        .map(|record| record.proposal_id.clone())
}

fn first_runnable_test_command(commands: &[String]) -> Option<RunnableTestCommand> {
    commands
        .iter()
        .filter_map(|command| parse_runnable_test_command(command))
        .next()
}

fn parse_runnable_test_command(command: &str) -> Option<RunnableTestCommand> {
    let label = command.trim();
    if label.is_empty() || has_shell_control(label) {
        return None;
    }
    let mut parts = split_command(label);
    if parts.is_empty() {
        return None;
    }
    let program = parts.remove(0);
    crate::test_runner::is_test_command(&program, &parts).then(|| RunnableTestCommand {
        args: parts,
        label: label.to_string(),
        program,
    })
}

fn command_in_scope(commands: Option<&[String]>, label: &str) -> bool {
    commands
        .map(|commands| commands.iter().any(|command| command == label))
        .unwrap_or(false)
}

fn has_shell_control(command: &str) -> bool {
    ["&&", "||", ";", "|", "<", ">", "`"]
        .iter()
        .any(|token| command.contains(token))
}

fn split_command(command: &str) -> Vec<String> {
    let mut parts = Vec::new();
    let mut current = String::new();
    let mut quote = None;
    for ch in command.chars() {
        if matches!(ch, '"' | '\'') && quote.is_none() {
            quote = Some(ch);
        } else if Some(ch) == quote {
            quote = None;
        } else if ch.is_whitespace() && quote.is_none() {
            push_part(&mut parts, &mut current);
        } else {
            current.push(ch);
        }
    }
    if quote.is_some() {
        return Vec::new();
    }
    push_part(&mut parts, &mut current);
    parts
}

fn push_part(parts: &mut Vec<String>, current: &mut String) {
    if !current.trim().is_empty() {
        parts.push(current.trim().to_string());
    }
    current.clear();
}
