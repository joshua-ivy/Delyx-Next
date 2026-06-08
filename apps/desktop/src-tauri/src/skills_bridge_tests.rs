#[cfg(test)]
mod tests {
    use crate::skills::{SkillRegistry, SkillStatus};
    use crate::skills_bridge::{
        activate_skill_record, disable_skill_record, import_skill_record, skill_snapshot_from_path,
        suppress_skill_record, SkillActionRequest, SkillActivateRequest, SkillImportRequest,
        SkillPermissionsRequest,
    };
    use std::fs;
    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn skill_bridge_imports_activates_and_survives_reload() {
        let path = temp_path("skill-bridge-activate");
        let mut registry = SkillRegistry::new();
        let imported =
            import_skill_record(&mut registry, import_request("review", "local")).unwrap();
        let skill_id = imported.skills[0].id.clone();

        let active = activate_skill_record(
            &mut registry,
            SkillActivateRequest {
                permissions: permissions(true, false, false),
                skill_id,
            },
        )
        .unwrap();

        assert_eq!(active.skills[0].status, "active");
        assert!(active.skills[0].permissions.can_run_scripts);
        crate::skills_persistence::save_to_path(&registry, &path).unwrap();
        let reloaded = skill_snapshot_from_path(&path).unwrap();
        assert_eq!(reloaded.skills[0].status, "active");
        assert!(reloaded.skills[0].permissions.can_run_scripts);
        let _ = fs::remove_file(path);
    }

    #[test]
    fn skill_bridge_preserves_third_party_inactive_import() {
        let mut registry = SkillRegistry::new();

        let view =
            import_skill_record(&mut registry, import_request("remote", "third_party")).unwrap();

        assert_eq!(view.skills[0].trust, "third_party");
        assert_eq!(view.skills[0].status, "inactive");
    }

    #[test]
    fn skill_bridge_disables_and_suppresses_visible_state() {
        let mut registry = SkillRegistry::new();
        let imported =
            import_skill_record(&mut registry, import_request("review", "local")).unwrap();
        let skill_id = imported.skills[0].id.clone();

        let disabled = disable_skill_record(
            &mut registry,
            SkillActionRequest {
                skill_id: skill_id.clone(),
            },
        )
        .unwrap();
        let suppressed =
            suppress_skill_record(&mut registry, SkillActionRequest { skill_id }).unwrap();

        assert_eq!(disabled.skills[0].status, "disabled");
        assert_eq!(suppressed.skills[0].status, "suppressed");
        assert_eq!(registry.skills()[0].status, SkillStatus::Suppressed);
    }

    #[test]
    fn skill_bridge_rejects_empty_imports_and_unknown_ids() {
        let mut registry = SkillRegistry::new();

        assert_eq!(
            import_skill_record(
                &mut registry,
                SkillImportRequest {
                    contents: String::new(),
                    source: String::new(),
                    trust: "local".to_string(),
                },
            )
            .unwrap_err(),
            "Skill import requires source and contents."
        );
        assert!(activate_skill_record(
            &mut registry,
            SkillActivateRequest {
                permissions: permissions(false, false, false),
                skill_id: "missing".to_string(),
            },
        )
        .unwrap_err()
        .contains("MissingSkill"));
    }

    fn import_request(name: &str, trust: &str) -> SkillImportRequest {
        SkillImportRequest {
            contents: format!("name: {name}\n"),
            source: format!("skills/{name}/SKILL.md"),
            trust: trust.to_string(),
        }
    }

    fn permissions(
        can_run_scripts: bool,
        can_edit_files: bool,
        can_use_network: bool,
    ) -> SkillPermissionsRequest {
        SkillPermissionsRequest {
            can_edit_files,
            can_run_scripts,
            can_use_network,
        }
    }

    fn temp_path(name: &str) -> PathBuf {
        let stamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        std::env::temp_dir().join(format!("delyx-next-{name}-{stamp}.sqlite3"))
    }
}
