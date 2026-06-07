#[cfg(test)]
mod tests {
    use crate::skills::{SkillError, SkillPermissions, SkillRegistry, SkillStatus, SkillTrust};

    #[test]
    fn third_party_skill_never_auto_activates() {
        let mut registry = SkillRegistry::new();

        let skill = registry.import_skill_file("skills/review/SKILL.md", "name: Code review\n", SkillTrust::ThirdParty);

        assert_eq!(skill.status, SkillStatus::Inactive);
        assert_eq!(skill.trust, SkillTrust::ThirdParty);
    }

    #[test]
    fn skill_can_be_disabled_and_suppressed() {
        let mut registry = SkillRegistry::new();
        let skill = registry.import_skill_file("skills/review/SKILL.md", "name: Code review\n", SkillTrust::Local);

        registry.disable(&skill.id).unwrap();
        assert_eq!(registry.skills()[0].status, SkillStatus::Disabled);
        registry.suppress(&skill.id).unwrap();
        assert_eq!(registry.skills()[0].status, SkillStatus::Suppressed);
    }

    #[test]
    fn skill_permissions_and_source_hash_are_visible() {
        let mut registry = SkillRegistry::new();
        let skill = registry.import_skill_file("skills/review/SKILL.md", "name: Code review\n", SkillTrust::Local);

        assert_eq!(skill.name, "Code review");
        assert_eq!(skill.source, "skills/review/SKILL.md");
        assert!(!skill.source_hash.is_empty());
        assert_eq!(skill.permissions, SkillPermissions::default());
    }

    #[test]
    fn skill_cannot_run_scripts_unless_allowed() {
        let mut registry = SkillRegistry::new();
        let skill = registry.import_skill_file("skills/review/SKILL.md", "name: Code review\n", SkillTrust::Local);

        registry.activate(&skill.id, SkillPermissions { can_run_scripts: false, can_edit_files: false, can_use_network: false }).unwrap();
        assert_eq!(registry.assert_can_run_scripts(&skill.id).unwrap_err(), SkillError::ScriptsNotAllowed);
        registry.activate(&skill.id, SkillPermissions { can_run_scripts: true, can_edit_files: false, can_use_network: false }).unwrap();
        assert_eq!(registry.assert_can_run_scripts(&skill.id), Ok(()));
    }

    #[test]
    fn inactive_skill_cannot_run_scripts() {
        let mut registry = SkillRegistry::new();
        let skill = registry.import_skill_file("skills/review/SKILL.md", "name: Code review\n", SkillTrust::Local);

        assert_eq!(registry.assert_can_run_scripts(&skill.id).unwrap_err(), SkillError::SkillNotActive);
    }
}
