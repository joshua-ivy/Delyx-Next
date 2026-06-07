#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SkillManifest {
    pub id: String,
    pub name: String,
    pub source: String,
    pub source_hash: String,
    pub trust: SkillTrust,
    pub status: SkillStatus,
    pub permissions: SkillPermissions,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SkillTrust {
    Local,
    ThirdParty,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SkillStatus {
    Inactive,
    Active,
    Disabled,
    Suppressed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct SkillPermissions {
    pub can_run_scripts: bool,
    pub can_edit_files: bool,
    pub can_use_network: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SkillError {
    MissingSkill,
    ScriptsNotAllowed,
    SkillNotActive,
}

#[derive(Debug, Default)]
pub struct SkillRegistry {
    next_id: usize,
    skills: Vec<SkillManifest>,
}

impl SkillRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn import_skill_file(&mut self, source: &str, contents: &str, trust: SkillTrust) -> SkillManifest {
        self.next_id += 1;
        let skill = SkillManifest {
            id: format!("skill-{}", self.next_id),
            name: scan_name(contents).unwrap_or_else(|| "Unnamed skill".to_string()),
            source: source.to_string(),
            source_hash: stable_hash(contents),
            trust,
            status: SkillStatus::Inactive,
            permissions: SkillPermissions::default(),
        };
        self.skills.push(skill.clone());
        skill
    }

    pub fn activate(&mut self, skill_id: &str, permissions: SkillPermissions) -> Result<(), SkillError> {
        let skill = self.skill_mut(skill_id)?;
        skill.permissions = permissions;
        skill.status = SkillStatus::Active;
        Ok(())
    }

    pub fn disable(&mut self, skill_id: &str) -> Result<(), SkillError> {
        self.skill_mut(skill_id)?.status = SkillStatus::Disabled;
        Ok(())
    }

    pub fn suppress(&mut self, skill_id: &str) -> Result<(), SkillError> {
        self.skill_mut(skill_id)?.status = SkillStatus::Suppressed;
        Ok(())
    }

    pub fn assert_can_run_scripts(&self, skill_id: &str) -> Result<(), SkillError> {
        let skill = self.skill(skill_id)?;
        if skill.status != SkillStatus::Active {
            return Err(SkillError::SkillNotActive);
        }
        skill.permissions.can_run_scripts.then_some(()).ok_or(SkillError::ScriptsNotAllowed)
    }

    pub fn skills(&self) -> &[SkillManifest] {
        &self.skills
    }

    fn skill(&self, skill_id: &str) -> Result<&SkillManifest, SkillError> {
        self.skills.iter().find(|skill| skill.id == skill_id).ok_or(SkillError::MissingSkill)
    }

    fn skill_mut(&mut self, skill_id: &str) -> Result<&mut SkillManifest, SkillError> {
        self.skills.iter_mut().find(|skill| skill.id == skill_id).ok_or(SkillError::MissingSkill)
    }
}

fn scan_name(contents: &str) -> Option<String> {
    contents
        .lines()
        .find_map(|line| line.strip_prefix("name:").map(|value| value.trim().to_string()))
        .filter(|value| !value.is_empty())
}

fn stable_hash(contents: &str) -> String {
    let hash = contents.bytes().fold(2_166_136_261_u32, |value, byte| {
        value.wrapping_mul(16_777_619) ^ byte as u32
    });
    format!("{hash:08x}")
}
