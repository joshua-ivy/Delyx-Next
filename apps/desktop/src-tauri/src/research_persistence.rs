use crate::research::{EvidenceRecord, EvidenceSourceKind, EvidenceStance, EvidenceStore};
use rusqlite::{params, Connection};
use std::path::Path;

pub fn save_to_path(store: &EvidenceStore, path: &Path) -> Result<(), String> {
    let connection = crate::sqlite_store::open_migrated_database(path).map_err(sql_string)?;
    clear_records(&connection)?;
    for record in store.all_records() {
        connection
            .execute(
                "INSERT INTO research_evidence_records
                 (id, run_id, source_kind, title, locator, excerpt, stance, claim_key)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
                params![
                    record.id,
                    record.run_id,
                    source_kind_key(record.source_kind),
                    record.title,
                    record.locator,
                    record.excerpt,
                    stance_key(record.stance),
                    record.claim_key
                ],
            )
            .map_err(sql_string)?;
    }
    Ok(())
}

pub fn load_from_path(path: &Path) -> Result<EvidenceStore, String> {
    let connection = crate::sqlite_store::open_migrated_database(path).map_err(sql_string)?;
    let mut statement = connection
        .prepare(
            "SELECT id, run_id, source_kind, title, locator, excerpt, stance, claim_key
             FROM research_evidence_records
             ORDER BY rowid",
        )
        .map_err(sql_string)?;
    let mut rows = statement.query([]).map_err(sql_string)?;
    let mut records = Vec::new();
    while let Some(row) = rows.next().map_err(sql_string)? {
        let source_kind: String = row.get(2).map_err(sql_string)?;
        let stance: String = row.get(6).map_err(sql_string)?;
        records.push(EvidenceRecord {
            id: row.get(0).map_err(sql_string)?,
            run_id: row.get(1).map_err(sql_string)?,
            source_kind: parse_source_kind(&source_kind)?,
            title: row.get(3).map_err(sql_string)?,
            locator: row.get(4).map_err(sql_string)?,
            excerpt: row.get(5).map_err(sql_string)?,
            stance: parse_stance(&stance)?,
            claim_key: row.get(7).map_err(sql_string)?,
        });
    }
    Ok(EvidenceStore::from_loaded(records))
}

fn clear_records(connection: &Connection) -> Result<(), String> {
    connection
        .execute("DELETE FROM research_evidence_records", [])
        .map_err(sql_string)?;
    Ok(())
}

fn source_kind_key(kind: EvidenceSourceKind) -> &'static str {
    match kind {
        EvidenceSourceKind::Diff => "diff",
        EvidenceSourceKind::ExternalAgent => "external_agent",
        EvidenceSourceKind::LocalFile => "local_file",
        EvidenceSourceKind::Memory => "memory",
        EvidenceSourceKind::ModelCall => "model_call",
        EvidenceSourceKind::RepoSymbol => "repo_symbol",
        EvidenceSourceKind::Terminal => "terminal",
        EvidenceSourceKind::Test => "test",
        EvidenceSourceKind::Web => "web",
    }
}

fn parse_source_kind(value: &str) -> Result<EvidenceSourceKind, String> {
    match value {
        "diff" => Ok(EvidenceSourceKind::Diff),
        "external_agent" => Ok(EvidenceSourceKind::ExternalAgent),
        "local_file" => Ok(EvidenceSourceKind::LocalFile),
        "memory" => Ok(EvidenceSourceKind::Memory),
        "model_call" => Ok(EvidenceSourceKind::ModelCall),
        "repo_symbol" => Ok(EvidenceSourceKind::RepoSymbol),
        "terminal" => Ok(EvidenceSourceKind::Terminal),
        "test" => Ok(EvidenceSourceKind::Test),
        "web" => Ok(EvidenceSourceKind::Web),
        _ => Err(format!("Unsupported evidence source kind: {value}")),
    }
}

fn stance_key(stance: EvidenceStance) -> &'static str {
    match stance {
        EvidenceStance::Contradicts => "contradicts",
        EvidenceStance::Supports => "supports",
    }
}

fn parse_stance(value: &str) -> Result<EvidenceStance, String> {
    match value {
        "contradicts" => Ok(EvidenceStance::Contradicts),
        "supports" => Ok(EvidenceStance::Supports),
        _ => Err(format!("Unsupported evidence stance: {value}")),
    }
}

fn sql_string(error: rusqlite::Error) -> String {
    error.to_string()
}
