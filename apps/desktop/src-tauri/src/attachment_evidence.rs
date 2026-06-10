//! Evidence records derived from a context pack. Every attached-file claim the
//! assistant makes should trace back to one of these, with a stable locator that
//! points at the exact chunk range. Generation is pure; persistence is separate.

use crate::context_pack::ContextPack;
use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use std::path::Path;

const EXCERPT_CHARS: usize = 280;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AttachmentEvidenceRecord {
    pub id: String,
    pub project_id: String,
    pub thread_id: Option<String>,
    pub run_id: Option<String>,
    pub attachment_id: String,
    pub source_kind: String,
    pub title: String,
    pub locator: String,
    pub excerpt: String,
    pub content_hash: Option<String>,
    pub retrieved_at: String,
    pub relevance_score: Option<u32>,
    pub relevance_reason: Option<String>,
}

/// Build one evidence record per pack item that came from an attachment. The
/// evidence locator IS the chunk locator, so a citation points at exact lines.
pub fn evidence_from_pack(pack: &ContextPack, retrieved_at: &str) -> Vec<AttachmentEvidenceRecord> {
    pack.items
        .iter()
        .filter_map(|item| {
            let attachment_id = item.attachment_id.clone()?;
            Some(AttachmentEvidenceRecord {
                id: stable_evidence_id(&pack.id, &item.locator),
                project_id: pack.project_id.clone(),
                thread_id: Some(pack.thread_id.clone()),
                run_id: pack.run_id.clone(),
                attachment_id,
                source_kind: "attachment".to_string(),
                title: item.locator.clone(),
                locator: item.locator.clone(),
                excerpt: excerpt(&item.text),
                content_hash: Some(hash(&item.text)),
                retrieved_at: retrieved_at.to_string(),
                relevance_score: None,
                relevance_reason: Some("Included in the thread context pack.".to_string()),
            })
        })
        .collect()
}

fn excerpt(text: &str) -> String {
    let trimmed = text.trim();
    if trimmed.chars().count() <= EXCERPT_CHARS {
        return trimmed.to_string();
    }
    let truncated: String = trimmed.chars().take(EXCERPT_CHARS).collect();
    format!("{truncated}…")
}

fn stable_evidence_id(pack_id: &str, locator: &str) -> String {
    format!("evid-{}", hash(&format!("{pack_id}::{locator}")))
}

fn hash(text: &str) -> String {
    let mut hash: u64 = 1469598103934665603;
    for byte in text.bytes() {
        hash ^= u64::from(byte);
        hash = hash.wrapping_mul(1099511628211);
    }
    format!("{hash:016x}")
}

// ---- Persistence ----

pub fn save_evidence_to_path(path: &Path, record: &AttachmentEvidenceRecord) -> Result<(), String> {
    let connection = crate::sqlite_store::open_migrated_database(path).map_err(sql_string)?;
    upsert_evidence(&connection, record)
}

pub fn save_evidence_batch_to_path(
    path: &Path,
    records: &[AttachmentEvidenceRecord],
) -> Result<(), String> {
    let connection = crate::sqlite_store::open_migrated_database(path).map_err(sql_string)?;
    for record in records {
        upsert_evidence(&connection, record)?;
    }
    Ok(())
}

pub fn list_evidence_for_thread(
    path: &Path,
    project_id: &str,
    thread_id: &str,
) -> Result<Vec<AttachmentEvidenceRecord>, String> {
    let connection = crate::sqlite_store::open_migrated_database(path).map_err(sql_string)?;
    let mut statement = connection
        .prepare(
            "SELECT id, project_id, thread_id, run_id, attachment_id, source_kind, title, locator,
                    excerpt, content_hash, retrieved_at, relevance_score, relevance_reason
             FROM attachment_evidence_records
             WHERE project_id = ?1 AND thread_id = ?2
             ORDER BY retrieved_at DESC, rowid DESC",
        )
        .map_err(sql_string)?;
    let rows = statement
        .query_map(params![project_id, thread_id], evidence_from_row)
        .map_err(sql_string)?;
    rows.collect::<Result<Vec<_>, _>>().map_err(sql_string)
}

fn upsert_evidence(
    connection: &Connection,
    record: &AttachmentEvidenceRecord,
) -> Result<(), String> {
    connection
        .execute(
            "INSERT INTO attachment_evidence_records (
                id, project_id, thread_id, run_id, attachment_id, source_kind, title, locator,
                excerpt, content_hash, retrieved_at, relevance_score, relevance_reason
             ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13)
             ON CONFLICT(id) DO UPDATE SET
                excerpt = excluded.excerpt,
                content_hash = excluded.content_hash,
                retrieved_at = excluded.retrieved_at,
                relevance_score = excluded.relevance_score,
                relevance_reason = excluded.relevance_reason",
            params![
                record.id,
                record.project_id,
                record.thread_id,
                record.run_id,
                record.attachment_id,
                record.source_kind,
                record.title,
                record.locator,
                record.excerpt,
                record.content_hash,
                record.retrieved_at,
                record.relevance_score.map(|value| value as i64),
                record.relevance_reason,
            ],
        )
        .map(|_| ())
        .map_err(sql_string)
}

fn evidence_from_row(row: &rusqlite::Row) -> rusqlite::Result<AttachmentEvidenceRecord> {
    Ok(AttachmentEvidenceRecord {
        id: row.get(0)?,
        project_id: row.get(1)?,
        thread_id: row.get(2)?,
        run_id: row.get(3)?,
        attachment_id: row.get(4)?,
        source_kind: row.get(5)?,
        title: row.get(6)?,
        locator: row.get(7)?,
        excerpt: row.get(8)?,
        content_hash: row.get(9)?,
        retrieved_at: row.get(10)?,
        relevance_score: row.get::<_, Option<i64>>(11)?.map(|value| value as u32),
        relevance_reason: row.get(12)?,
    })
}

fn sql_string(error: rusqlite::Error) -> String {
    error.to_string()
}
