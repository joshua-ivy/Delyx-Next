use rusqlite::{params, Connection};

use crate::agent_run::{AgentRunError, AgentRunLedger, EvidenceRecord, EvidenceRelevance};

pub fn insert_evidence(
    connection: &Connection,
    run_id: &str,
    evidence: &EvidenceRecord,
) -> Result<(), AgentRunError> {
    let relevance = evidence.relevance.as_ref();
    connection
        .execute(
            "INSERT INTO evidence_records
             (id, run_id, source_kind, source_id, title, uri, quote, hash, retrieved_at,
              relevance_relationship, relevance_score, relevance_reason)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)",
            params![
                evidence.id,
                run_id,
                evidence.source_kind,
                evidence.source_id,
                evidence.title,
                evidence.uri,
                evidence.quote,
                evidence.hash,
                evidence.retrieved_at,
                relevance.map(|value| value.relationship.as_str()),
                relevance.map(|value| value.score),
                relevance.map(|value| value.reason.as_str()),
            ],
        )
        .map(|_| ())
        .map_err(sql_error)
}

pub fn load_evidence(
    connection: &Connection,
    ledger: &mut AgentRunLedger,
) -> Result<(), AgentRunError> {
    let mut statement = connection
        .prepare(
            "SELECT run_id, id, source_kind, source_id, title, uri, quote, hash, retrieved_at,
                    relevance_relationship, relevance_score, relevance_reason
             FROM evidence_records ORDER BY rowid",
        )
        .map_err(sql_error)?;
    let rows = statement
        .query_map([], |row| {
            Ok((
                row.get::<_, String>(0)?,
                EvidenceRecord {
                    hash: row.get(7)?,
                    id: row.get(1)?,
                    quote: row.get(6)?,
                    relevance: relevance_from_row(row.get(9)?, row.get(10)?, row.get(11)?),
                    retrieved_at: row.get(8)?,
                    source_id: row.get(3)?,
                    source_kind: row.get(2)?,
                    title: row.get(4)?,
                    uri: row.get(5)?,
                },
            ))
        })
        .map_err(sql_error)?;
    for row in rows {
        let (run_id, evidence) = row.map_err(sql_error)?;
        let run = ledger.run_mut(&run_id)?;
        run.evidence.push(evidence);
        run.metrics.evidence_count = run.evidence.len();
    }
    Ok(())
}

fn relevance_from_row(
    relationship: Option<String>,
    score: Option<i32>,
    reason: Option<String>,
) -> Option<EvidenceRelevance> {
    if relationship.is_none() && score.is_none() && reason.is_none() {
        return None;
    }
    Some(EvidenceRelevance {
        reason: reason.unwrap_or_default(),
        relationship: relationship.unwrap_or_else(|| "unknown".to_string()),
        score: score.unwrap_or_default(),
    })
}

fn sql_error(error: rusqlite::Error) -> AgentRunError {
    AgentRunError::Io(error.to_string())
}
