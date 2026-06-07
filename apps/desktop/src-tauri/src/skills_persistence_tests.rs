#[cfg(test)]
mod tests {
    use crate::skills::{SkillPermissions, SkillRegistry, SkillStatus, SkillTrust};
    use crate::skills_persistence::{load_from_path, save_to_path};
    use std::fs;
    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn skill_registry_survives_sqlite_reload() {
        let path = temp_path("skills");
        let mut registry = SkillRegistry::new();
        let local = registry.import_skill_file("skills/review/SKILL.md", "name: Code review\n", SkillTrust::Local);
        let third_party = registry.import_skill_file("skills/remote/SKILL.md", "name: Remote helper\n", SkillTrust::ThirdParty);
        registry
            .activate(&local.id, SkillPermissions { can_run_scripts: true, can_edit_files: false, can_use_network: false })
            .unwrap();
        registry.suppress(&third_party.id).unwrap();

        save_to_path(&registry, &path).unwrap();
        let bytes = fs::read(&path).unwrap();
        assert!(bytes.starts_with(b"SQLite format 3"));

        let mut loaded = load_from_path(&path).unwrap();
        assert_eq!(loaded.skills()[0].status, SkillStatus::Active);
        assert!(loaded.skills()[0].permissions.can_run_scripts);
        assert_eq!(loaded.skills()[1].trust, SkillTrust::ThirdParty);
        assert_eq!(loaded.skills()[1].status, SkillStatus::Suppressed);
        assert_eq!(loaded.assert_can_run_scripts(&local.id), Ok(()));

        let next = loaded.import_skill_file("skills/next/SKILL.md", "name: Next\n", SkillTrust::Local);
        assert_eq!(next.id, "skill-3");
        let _ = fs::remove_file(path);
    }

    fn temp_path(name: &str) -> PathBuf {
        let stamp = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos();
        std::env::temp_dir().join(format!("delyx-next-{name}-{stamp}.sqlite3"))
    }
}
