#[cfg(test)]
mod tests {
    use crate::approval::{ApprovalEngine, ProposalInput, RiskLevel, RiskyAction};
    use crate::automation::{ActiveHours, AutomationEngine, MissionContractInput, ScheduledRunStatus};
    use crate::automation_bridge::automation_snapshot_from_path;
    use crate::memory::{MemoryCandidateInput, MemoryScope, MemoryStore, SourceRunStatus};
    use crate::memory_bridge::memory_snapshot_from_path;
    use crate::release::{default_release_profile, export_support_bundle};
    use crate::release_bridge::release_snapshot_from_path;
    use crate::skills::{SkillPermissions, SkillRegistry, SkillStatus, SkillTrust};
    use crate::skills_bridge::skill_snapshot_from_path;
    use std::fs;
    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn persisted_governance_snapshots_are_ui_ready() {
        let path = temp_path("local-bridges");
        save_memory(&path);
        save_skills(&path);
        save_automation(&path);
        save_release(&path);

        let memory = memory_snapshot_from_path(&path).unwrap();
        let skills = skill_snapshot_from_path(&path).unwrap();
        let automation = automation_snapshot_from_path(&path).unwrap();
        let release = release_snapshot_from_path(&path).unwrap();

        assert_eq!(memory.candidates[0].status, "promoted");
        assert_eq!(memory.records[0].scope, "project");
        assert_eq!(skills.skills[0].status, "active");
        assert_eq!(skills.skills[0].permissions.can_run_scripts, true);
        assert_eq!(automation.contracts[0].active_hours, "08:00-18:00");
        assert_eq!(automation.scheduled_runs[0].status, "waiting_for_approval");
        assert_eq!(release.support_bundle.export_status, "available");
        assert_eq!(release.update_metadata.status, "published");
        let _ = fs::remove_file(path);
    }

    fn save_memory(path: &PathBuf) {
        let mut store = MemoryStore::new();
        let candidate = store.propose_candidate(MemoryCandidateInput {
            key: "style".to_string(),
            scope: MemoryScope::Project,
            source_run_id: "run-1".to_string(),
            source_thread_id: "thread-1".to_string(),
            value: "Prefer small files.".to_string(),
        });
        let mut approvals = ApprovalEngine::new();
        let approval = approvals.propose(proposal_input(RiskyAction::DurableMemorySave, &candidate.id, "run-1"));
        approvals.approve(&approval.id, 10, "approved in test").unwrap();
        store.promote_approved(&candidate.id, &approval.id, 10, &approvals, SourceRunStatus::Completed).unwrap();
        crate::memory_persistence::save_to_path(&store, path).unwrap();
    }

    fn save_skills(path: &PathBuf) {
        let mut registry = SkillRegistry::new();
        let skill = registry.import_skill_file("skills/review/SKILL.md", "name: Code review\n", SkillTrust::Local);
        registry
            .activate(&skill.id, SkillPermissions { can_run_scripts: true, can_edit_files: false, can_use_network: false })
            .unwrap();
        assert_eq!(registry.skills()[0].status, SkillStatus::Active);
        crate::skills_persistence::save_to_path(&registry, path).unwrap();
    }

    fn save_automation(path: &PathBuf) {
        let mut engine = AutomationEngine::new();
        let mut approvals = ApprovalEngine::new();
        let contract = engine.create_contract(MissionContractInput {
            active_hours: ActiveHours { start_hour: 8, end_hour: 18 },
            allowed_tools: vec!["terminal_command".to_string()],
            delivery_targets: vec!["desktop_notification".to_string()],
            scope: "C:/workspace".to_string(),
            stop_condition: "Stop after one failed run.".to_string(),
            timezone: "America/Chicago".to_string(),
            title: "Morning repo health".to_string(),
            workspace_fingerprint: "fingerprint-1".to_string(),
        });
        let approval = approvals.propose(proposal_input(RiskyAction::ScheduledRiskyAction, "node-automation", &contract.id));
        approvals.approve(&approval.id, 10, "approved in test").unwrap();
        engine.approve_contract(&contract.id, &approval.id, 10, &approvals).unwrap();
        let run = engine.schedule_due_run(&contract.id, "fingerprint-1", 10, &mut approvals).unwrap();
        assert_eq!(run.status, ScheduledRunStatus::WaitingForApproval);
        crate::automation_persistence::save_to_path(&engine, path).unwrap();
    }

    fn save_release(path: &PathBuf) {
        let mut profile = default_release_profile();
        profile.update_metadata.published = true;
        let bundle = export_support_bundle(&profile, vec![("workspace", "C:/work")], vec![("runtime", "ok")], 42);
        crate::release_persistence::save_profile_to_path(&profile, path).unwrap();
        crate::release_persistence::save_support_bundle_to_path(&bundle, path).unwrap();
    }

    fn proposal_input(action: RiskyAction, node_id: &str, run_id: &str) -> ProposalInput {
        ProposalInput {
            action,
            expires_at: 30,
            expected_result: "Allow tested snapshot fixture.".to_string(),
            node_id: node_id.to_string(),
            reason: "Bridge snapshot test.".to_string(),
            risk: RiskLevel::Medium,
            rollback_plan: "Discard fixture state.".to_string(),
            run_id: run_id.to_string(),
            scope: "Test fixture.".to_string(),
        }
    }

    fn temp_path(name: &str) -> PathBuf {
        let stamp = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos();
        std::env::temp_dir().join(format!("delyx-next-{name}-{stamp}.sqlite3"))
    }
}
