use crate::agent_scheduler::AgentSchedulerContext;
use crate::approval::RiskyAction;
use crate::patch_bridge::PatchProposalView;

pub(crate) fn ready_patch_apply_approval(
    context: &AgentSchedulerContext<'_>,
    proposal: &PatchProposalView,
) -> Option<String> {
    let node_id = apply_node_id(context, proposal);
    context
        .patch_apply_approval_id
        .and_then(|id| ready_apply_node(context, id, &node_id).then(|| id.to_string()))
        .or_else(|| discover_apply_approval(context, &node_id))
}

fn discover_apply_approval(context: &AgentSchedulerContext<'_>, node_id: &str) -> Option<String> {
    context
        .approvals
        .list_proposals(&context.run.id)
        .into_iter()
        .find(|approval| ready_apply_node(context, &approval.id, node_id))
        .map(|approval| approval.id.clone())
}

fn ready_apply_node(context: &AgentSchedulerContext<'_>, approval_id: &str, node_id: &str) -> bool {
    !approval_id.trim().is_empty()
        && context
            .approvals
            .assert_can_execute_action_for_run_node(
                approval_id,
                context.now_ms,
                RiskyAction::FileWrite,
                &context.run.id,
                node_id,
            )
            .is_ok()
}

fn apply_node_id(context: &AgentSchedulerContext<'_>, proposal: &PatchProposalView) -> String {
    format!("{}-patch-apply-{}", context.run.id, proposal.id)
}
