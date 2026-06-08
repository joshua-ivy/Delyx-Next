use crate::skills::{SkillManifest, SkillPermissions, SkillRegistry, SkillStatus, SkillTrust};
use rusqlite::{params, Connection};
use std::path::Path;

pub fn save_to_path(registry: &SkillRegistry, path: &Path) -> Result<(), String> {
    let connection = crate::sqlite_store::open_migrated_database(path).map_err(sql_string)?;
    connection
        .execute("DELETE FROM skill_manifests", [])
        .map_err(sql_string)?;
    for skill in registry.skills() {
        insert_skill(&connection, skill)?;
    }
    Ok(())
}

pub fn load_from_path(path: &Path) -> Result<SkillRegistry, String> {
    let connection = crate::sqlite_store::open_migrated_database(path).map_err(sql_string)?;
    let mut statement = connection
        .prepare(
            "SELECT id, name, source, source_hash, trust, status, can_run_scripts, can_edit_files, can_use_network
             FROM skill_manifests ORDER BY rowid",
        )
        .map_err(sql_string)?;
    let mut rows = statement.query([]).map_err(sql_string)?;
    let mut skills = Vec::new();
    while let Some(row) = rows.next().map_err(sql_string)? {
        let trust: String = row.get(4).map_err(sql_string)?;
        let status: String = row.get(5).map_err(sql_string)?;
        skills.push(SkillManifest {
            id: row.get(0).map_err(sql_string)?,
            name: row.get(1).map_err(sql_string)?,
            source: row.get(2).map_err(sql_string)?,
            source_hash: row.get(3).map_err(sql_string)?,
            trust: parse_trust(&trust)?,
            status: parse_status(&status)?,
            permissions: SkillPermissions {
                can_run_scripts: row.get::<_, i64>(6).map_err(sql_string)? != 0,
                can_edit_files: row.get::<_, i64>(7).map_err(sql_string)? != 0,
                can_use_network: row.get::<_, i64>(8).map_err(sql_string)? != 0,
            },
        });
    }
    Ok(SkillRegistry::from_loaded(skills))
}

fn insert_skill(connection: &Connection, skill: &SkillManifest) -> Result<(), String> {
    connection
        .execute(
            "INSERT INTO skill_manifests
             (id, name, source, source_hash, trust, status, can_run_scripts, can_edit_files, can_use_network)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
            params![
                skill.id,
                skill.name,
                skill.source,
                skill.source_hash,
                trust_key(skill.trust),
                status_key(skill.status),
                skill.permissions.can_run_scripts as i64,
                skill.permissions.can_edit_files as i64,
                skill.permissions.can_use_network as i64,
            ],
        )
        .map(|_| ())
        .map_err(sql_string)
}

fn trust_key(trust: SkillTrust) -> &'static str {
    match trust {
        SkillTrust::Local => "local",
        SkillTrust::ThirdParty => "third_party",
    }
}

fn parse_trust(value: &str) -> Result<SkillTrust, String> {
    match value {
        "local" => Ok(SkillTrust::Local),
        "third_party" => Ok(SkillTrust::ThirdParty),
        _ => Err("Unsupported persisted skill trust.".to_string()),
    }
}

fn status_key(status: SkillStatus) -> &'static str {
    match status {
        SkillStatus::Active => "active",
        SkillStatus::Disabled => "disabled",
        SkillStatus::Inactive => "inactive",
        SkillStatus::Suppressed => "suppressed",
    }
}

fn parse_status(value: &str) -> Result<SkillStatus, String> {
    match value {
        "active" => Ok(SkillStatus::Active),
        "disabled" => Ok(SkillStatus::Disabled),
        "inactive" => Ok(SkillStatus::Inactive),
        "suppressed" => Ok(SkillStatus::Suppressed),
        _ => Err("Unsupported persisted skill status.".to_string()),
    }
}

fn sql_string(error: rusqlite::Error) -> String {
    error.to_string()
}
