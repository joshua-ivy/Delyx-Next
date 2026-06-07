#[cfg(test)]
mod tests {
    use crate::approval::{ActionProposal, RiskLevel, RiskyAction, ProposalStatus};
    use crate::mobile::{
        decide_mobile_approval, default_mobile_policy, mobile_status_view, MobileDecisionKind, MobileError,
        MobilePolicy, MobileRunView, MobileThreadView,
    };

    #[test]
    fn mobile_cannot_grant_broader_permissions_than_desktop_policy() {
        let proposal = proposal(RiskLevel::Medium, RiskyAction::ExternalSend);
        let policy = MobilePolicy { allow_low_risk_approval: true, max_approval_risk: RiskLevel::Medium, can_access_files: false, can_access_terminal: false };

        let result = decide_mobile_approval(&proposal, MobileDecisionKind::Approve, &policy, RiskLevel::Low);

        assert_eq!(result.unwrap_err(), MobileError::RiskExceedsDesktopPolicy);
    }

    #[test]
    fn mobile_approval_scope_is_visible() {
        let proposal = proposal(RiskLevel::Low, RiskyAction::ExternalSend);
        let view = mobile_status_view(Vec::new(), vec![&proposal], Vec::new(), default_mobile_policy());

        assert_eq!(view.pending_approvals[0].scope, "Send one external status message.");
        assert_eq!(view.pending_approvals[0].reason, "Low-risk mobile approval test.");
    }

    #[test]
    fn mobile_view_only_lists_pending_approvals() {
        let pending = proposal(RiskLevel::Low, RiskyAction::ExternalSend);
        let mut approved = proposal(RiskLevel::Low, RiskyAction::ExternalSend);
        approved.id = "prop-approved".to_string();
        approved.status = ProposalStatus::Approved;

        let view = mobile_status_view(Vec::new(), vec![&pending, &approved], Vec::new(), default_mobile_policy());

        assert_eq!(view.pending_approvals.len(), 1);
        assert_eq!(view.pending_approvals[0].id, "prop-1");
    }

    #[test]
    fn mobile_can_review_status_without_full_runtime_access() {
        let view = mobile_status_view(
            vec![MobileThreadView { id: "thread-1".to_string(), title: "Review patch".to_string(), status: "active".to_string() }],
            Vec::new(),
            vec![MobileRunView { id: "run-1".to_string(), status: "waiting_for_approval".to_string() }],
            default_mobile_policy(),
        );

        assert_eq!(view.threads[0].status, "active");
        assert_eq!(view.runs[0].status, "waiting_for_approval");
        assert!(!view.policy.can_access_files);
        assert!(!view.policy.can_access_terminal);
    }

    #[test]
    fn mobile_can_approve_low_risk_if_configured() {
        let proposal = proposal(RiskLevel::Low, RiskyAction::ExternalSend);
        let policy = MobilePolicy { allow_low_risk_approval: true, max_approval_risk: RiskLevel::Low, can_access_files: false, can_access_terminal: false };

        let decision = decide_mobile_approval(&proposal, MobileDecisionKind::Approve, &policy, RiskLevel::Low).unwrap();

        assert_eq!(decision.proposal_id, "prop-1");
        assert_eq!(decision.scope, "Send one external status message.");
    }

    #[test]
    fn mobile_has_no_broad_file_or_terminal_access_by_default() {
        let file_proposal = proposal(RiskLevel::Low, RiskyAction::FileWrite);
        let terminal_proposal = proposal(RiskLevel::Low, RiskyAction::TerminalCommand);
        let policy = MobilePolicy { allow_low_risk_approval: true, max_approval_risk: RiskLevel::Low, ..default_mobile_policy() };

        assert_eq!(decide_mobile_approval(&file_proposal, MobileDecisionKind::Approve, &policy, RiskLevel::Low).unwrap_err(), MobileError::BroadFileAccessDenied);
        assert_eq!(decide_mobile_approval(&terminal_proposal, MobileDecisionKind::Deny, &policy, RiskLevel::Low).unwrap_err(), MobileError::BroadTerminalAccessDenied);
    }

    fn proposal(risk: RiskLevel, action: RiskyAction) -> ActionProposal {
        ActionProposal {
            action,
            decision: None,
            expected_result: "Mobile decision is recorded.".to_string(),
            expires_at: 30,
            id: "prop-1".to_string(),
            node_id: "node-mobile".to_string(),
            reason: "Low-risk mobile approval test.".to_string(),
            risk,
            rollback_plan: "No durable change from mobile.".to_string(),
            run_id: "run-1".to_string(),
            scope: "Send one external status message.".to_string(),
            status: ProposalStatus::Pending,
        }
    }
}
