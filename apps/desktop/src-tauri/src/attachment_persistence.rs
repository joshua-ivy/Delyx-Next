//! SQLite persistence for attachment proposals.
//!
//! Proposals are durable so denied/expired/failed states stay visible in the UI
//! rather than vanishing. Attachment *records* (post-approval) land in PR4.

use crate::attachment::{
    AttachmentIndexStatus, AttachmentKind, AttachmentParseStatus, AttachmentProposal,
    AttachmentProposalStatus, AttachmentRecord, AttachmentRisk, AttachmentSourceKind,
};
use rusqlite::{params, Connection, OptionalExtension};
use std::path::Path;

pub fn save_proposal_to_path(
    path: &Path,
    proposal: &AttachmentProposal,
) -> Result<AttachmentProposal, String> {
    let connection = crate::sqlite_store::open_migrated_database(path).map_err(sql_string)?;
    upsert_proposal(&connection, proposal)?;
    load_proposal(&connection, &proposal.id)?
        .ok_or_else(|| "Proposal disappeared immediately after saving.".to_string())
}

pub fn load_proposal_from_path(
    path: &Path,
    id: &str,
) -> Result<Option<AttachmentProposal>, String> {
    let connection = crate::sqlite_store::open_migrated_database(path).map_err(sql_string)?;
    load_proposal(&connection, id)
}

pub fn list_proposals_from_path(
    path: &Path,
    project_id: &str,
    thread_id: Option<&str>,
) -> Result<Vec<AttachmentProposal>, String> {
    let connection = crate::sqlite_store::open_migrated_database(path).map_err(sql_string)?;
    list_proposals(&connection, project_id, thread_id)
}

/// Update a proposal's lifecycle status (e.g. to denied/expired). Returns the
/// refreshed proposal, or an error if it no longer exists.
pub fn set_proposal_status_to_path(
    path: &Path,
    id: &str,
    status: AttachmentProposalStatus,
    approval_id: Option<&str>,
) -> Result<AttachmentProposal, String> {
    let connection = crate::sqlite_store::open_migrated_database(path).map_err(sql_string)?;
    let changed = connection
        .execute(
            "UPDATE attachment_proposals
             SET status = ?2, approval_id = COALESCE(?3, approval_id), updated_at = CURRENT_TIMESTAMP
             WHERE id = ?1",
            params![id.trim(), status.as_str(), approval_id],
        )
        .map_err(sql_string)?;
    if changed == 0 {
        return Err(format!("Attachment proposal `{id}` was not found."));
    }
    load_proposal(&connection, id)?
        .ok_or_else(|| format!("Attachment proposal `{id}` was not found."))
}

fn upsert_proposal(connection: &Connection, proposal: &AttachmentProposal) -> Result<(), String> {
    let scope_json = serde_json::to_string(&proposal.proposed_scope).map_err(|e| e.to_string())?;
    connection
        .execute(
            "INSERT INTO attachment_proposals (
                id, project_id, thread_id, source_kind, detected_kind, display_name,
                source_locator, proposed_scope_json, estimated_bytes, estimated_file_count,
                requires_approval, approval_reason, risk, status, approval_id
             ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15)
             ON CONFLICT(id) DO UPDATE SET
                thread_id = excluded.thread_id,
                source_kind = excluded.source_kind,
                detected_kind = excluded.detected_kind,
                display_name = excluded.display_name,
                source_locator = excluded.source_locator,
                proposed_scope_json = excluded.proposed_scope_json,
                estimated_bytes = excluded.estimated_bytes,
                estimated_file_count = excluded.estimated_file_count,
                requires_approval = excluded.requires_approval,
                approval_reason = excluded.approval_reason,
                risk = excluded.risk,
                status = excluded.status,
                approval_id = excluded.approval_id,
                updated_at = CURRENT_TIMESTAMP",
            params![
                proposal.id,
                proposal.project_id,
                proposal.thread_id,
                proposal.source_kind.as_str(),
                proposal.detected_kind.as_str(),
                proposal.display_name,
                proposal.source_locator,
                scope_json,
                proposal.estimated_bytes.map(|value| value as i64),
                proposal.estimated_file_count.map(|value| value as i64),
                proposal.requires_approval as i64,
                proposal.approval_reason,
                proposal.risk.as_str(),
                proposal.status.as_str(),
                proposal.approval_id,
            ],
        )
        .map(|_| ())
        .map_err(sql_string)
}

fn load_proposal(connection: &Connection, id: &str) -> Result<Option<AttachmentProposal>, String> {
    connection
        .query_row(
            &format!("SELECT {COLUMNS} FROM attachment_proposals WHERE id = ?1"),
            params![id.trim()],
            proposal_from_row,
        )
        .optional()
        .map_err(sql_string)
}

fn list_proposals(
    connection: &Connection,
    project_id: &str,
    thread_id: Option<&str>,
) -> Result<Vec<AttachmentProposal>, String> {
    // Thread-scoped query also returns project-wide proposals (thread_id NULL).
    let (sql, bind_thread) = match thread_id {
        Some(_) => (
            format!(
                "SELECT {COLUMNS} FROM attachment_proposals
                 WHERE project_id = ?1 AND (thread_id = ?2 OR thread_id IS NULL)
                 ORDER BY created_at DESC, rowid DESC"
            ),
            true,
        ),
        None => (
            format!(
                "SELECT {COLUMNS} FROM attachment_proposals
                 WHERE project_id = ?1 ORDER BY created_at DESC, rowid DESC"
            ),
            false,
        ),
    };
    let mut statement = connection.prepare(&sql).map_err(sql_string)?;
    let rows = if bind_thread {
        statement.query_map(params![project_id, thread_id], proposal_from_row)
    } else {
        statement.query_map(params![project_id], proposal_from_row)
    }
    .map_err(sql_string)?;
    rows.collect::<Result<Vec<_>, _>>().map_err(sql_string)
}

const COLUMNS: &str = "id, project_id, thread_id, source_kind, detected_kind, display_name, \
     source_locator, proposed_scope_json, estimated_bytes, estimated_file_count, \
     requires_approval, approval_reason, risk, status, approval_id, created_at, updated_at";

fn proposal_from_row(row: &rusqlite::Row) -> rusqlite::Result<AttachmentProposal> {
    let scope_json: String = row.get(7)?;
    Ok(AttachmentProposal {
        id: row.get(0)?,
        project_id: row.get(1)?,
        thread_id: row.get(2)?,
        source_kind: AttachmentSourceKind::from_str(&row.get::<_, String>(3)?)
            .unwrap_or(AttachmentSourceKind::LocalFile),
        detected_kind: AttachmentKind::from_str(&row.get::<_, String>(4)?),
        display_name: row.get(5)?,
        source_locator: row.get(6)?,
        proposed_scope: serde_json::from_str(&scope_json).unwrap_or_default(),
        estimated_bytes: row.get::<_, Option<i64>>(8)?.map(|value| value as u64),
        estimated_file_count: row.get::<_, Option<i64>>(9)?.map(|value| value as u32),
        requires_approval: row.get::<_, i64>(10)? != 0,
        approval_reason: row.get(11)?,
        risk: AttachmentRisk::from_str(&row.get::<_, String>(12)?),
        status: AttachmentProposalStatus::from_str(&row.get::<_, String>(13)?),
        approval_id: row.get(14)?,
        created_at: row.get(15)?,
        updated_at: row.get(16)?,
    })
}

// ---- Attachment records (durable, post-approval) ----

pub fn save_record_to_path(
    path: &Path,
    record: &AttachmentRecord,
) -> Result<AttachmentRecord, String> {
    let connection = crate::sqlite_store::open_migrated_database(path).map_err(sql_string)?;
    connection
        .execute(
            "INSERT INTO attachments (
                id, project_id, thread_id, message_id, run_id, source_kind, detected_kind,
                display_name, original_locator, local_reference_path, content_hash, bytes,
                parse_status, index_status, approval_id
             ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15)
             ON CONFLICT(id) DO UPDATE SET
                thread_id = excluded.thread_id,
                message_id = excluded.message_id,
                run_id = excluded.run_id,
                source_kind = excluded.source_kind,
                detected_kind = excluded.detected_kind,
                display_name = excluded.display_name,
                original_locator = excluded.original_locator,
                local_reference_path = excluded.local_reference_path,
                content_hash = excluded.content_hash,
                bytes = excluded.bytes,
                parse_status = excluded.parse_status,
                index_status = excluded.index_status,
                approval_id = excluded.approval_id,
                updated_at = CURRENT_TIMESTAMP",
            params![
                record.id,
                record.project_id,
                record.thread_id,
                record.message_id,
                record.run_id,
                record.source_kind.as_str(),
                record.detected_kind.as_str(),
                record.display_name,
                record.original_locator,
                record.local_reference_path,
                record.content_hash,
                record.bytes.map(|value| value as i64),
                record.parse_status.as_str(),
                record.index_status.as_str(),
                record.approval_id,
            ],
        )
        .map_err(sql_string)?;
    load_record(&connection, &record.id)?
        .ok_or_else(|| "Attachment record disappeared immediately after saving.".to_string())
}

pub fn list_records_from_path(
    path: &Path,
    project_id: &str,
    thread_id: Option<&str>,
) -> Result<Vec<AttachmentRecord>, String> {
    let connection = crate::sqlite_store::open_migrated_database(path).map_err(sql_string)?;
    let (sql, bind_thread) = match thread_id {
        Some(_) => (
            format!(
                "SELECT {RECORD_COLUMNS} FROM attachments
                 WHERE project_id = ?1 AND (thread_id = ?2 OR thread_id IS NULL)
                 ORDER BY created_at DESC, rowid DESC"
            ),
            true,
        ),
        None => (
            format!(
                "SELECT {RECORD_COLUMNS} FROM attachments
                 WHERE project_id = ?1 ORDER BY created_at DESC, rowid DESC"
            ),
            false,
        ),
    };
    let mut statement = connection.prepare(&sql).map_err(sql_string)?;
    let rows = if bind_thread {
        statement.query_map(params![project_id, thread_id], record_from_row)
    } else {
        statement.query_map(params![project_id], record_from_row)
    }
    .map_err(sql_string)?;
    rows.collect::<Result<Vec<_>, _>>().map_err(sql_string)
}

const RECORD_COLUMNS: &str = "id, project_id, thread_id, message_id, run_id, source_kind, \
     detected_kind, display_name, original_locator, local_reference_path, content_hash, bytes, \
     parse_status, index_status, approval_id, created_at, updated_at";

pub fn load_record_from_path(path: &Path, id: &str) -> Result<Option<AttachmentRecord>, String> {
    let connection = crate::sqlite_store::open_migrated_database(path).map_err(sql_string)?;
    load_record(&connection, id)
}

pub fn set_record_parse_status_to_path(
    path: &Path,
    id: &str,
    status: AttachmentParseStatus,
) -> Result<AttachmentRecord, String> {
    let connection = crate::sqlite_store::open_migrated_database(path).map_err(sql_string)?;
    connection
        .execute(
            "UPDATE attachments SET parse_status = ?2, updated_at = CURRENT_TIMESTAMP WHERE id = ?1",
            params![id.trim(), status.as_str()],
        )
        .map_err(sql_string)?;
    load_record(&connection, id)?.ok_or_else(|| format!("Attachment record `{id}` was not found."))
}

fn load_record(connection: &Connection, id: &str) -> Result<Option<AttachmentRecord>, String> {
    connection
        .query_row(
            &format!("SELECT {RECORD_COLUMNS} FROM attachments WHERE id = ?1"),
            params![id.trim()],
            record_from_row,
        )
        .optional()
        .map_err(sql_string)
}

// ---- Parsed chunks ----

pub fn save_chunks_to_path(
    path: &Path,
    attachment_id: &str,
    chunks: &[crate::attachment_parser::ParsedChunk],
) -> Result<(), String> {
    let connection = crate::sqlite_store::open_migrated_database(path).map_err(sql_string)?;
    connection
        .execute(
            "DELETE FROM attachment_chunks WHERE attachment_id = ?1",
            params![attachment_id],
        )
        .map_err(sql_string)?;
    for chunk in chunks {
        connection
            .execute(
                "INSERT INTO attachment_chunks (
                    attachment_id, chunk_index, kind, title, locator, text, token_estimate, content_hash
                 ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
                params![
                    attachment_id,
                    chunk.index as i64,
                    chunk.kind,
                    chunk.title,
                    chunk.locator,
                    chunk.text,
                    chunk.token_estimate as i64,
                    chunk.content_hash,
                ],
            )
            .map_err(sql_string)?;
    }
    Ok(())
}

pub fn list_chunks_from_path(
    path: &Path,
    attachment_id: &str,
) -> Result<Vec<crate::attachment_parser::ParsedChunk>, String> {
    let connection = crate::sqlite_store::open_migrated_database(path).map_err(sql_string)?;
    let mut statement = connection
        .prepare(
            "SELECT chunk_index, kind, title, locator, text, token_estimate, content_hash
             FROM attachment_chunks WHERE attachment_id = ?1 ORDER BY chunk_index",
        )
        .map_err(sql_string)?;
    let rows = statement
        .query_map(params![attachment_id], |row| {
            Ok(crate::attachment_parser::ParsedChunk {
                index: row.get::<_, i64>(0)? as u32,
                kind: row.get(1)?,
                title: row.get(2)?,
                locator: row.get(3)?,
                text: row.get(4)?,
                token_estimate: row.get::<_, i64>(5)? as u32,
                content_hash: row.get(6)?,
            })
        })
        .map_err(sql_string)?;
    rows.collect::<Result<Vec<_>, _>>().map_err(sql_string)
}

fn record_from_row(row: &rusqlite::Row) -> rusqlite::Result<AttachmentRecord> {
    Ok(AttachmentRecord {
        id: row.get(0)?,
        project_id: row.get(1)?,
        thread_id: row.get(2)?,
        message_id: row.get(3)?,
        run_id: row.get(4)?,
        source_kind: AttachmentSourceKind::from_str(&row.get::<_, String>(5)?)
            .unwrap_or(AttachmentSourceKind::LocalFile),
        detected_kind: AttachmentKind::from_str(&row.get::<_, String>(6)?),
        display_name: row.get(7)?,
        original_locator: row.get(8)?,
        local_reference_path: row.get(9)?,
        content_hash: row.get(10)?,
        bytes: row.get::<_, Option<i64>>(11)?.map(|value| value as u64),
        parse_status: AttachmentParseStatus::from_str(&row.get::<_, String>(12)?),
        index_status: AttachmentIndexStatus::from_str(&row.get::<_, String>(13)?),
        approval_id: row.get(14)?,
        created_at: row.get(15)?,
        updated_at: row.get(16)?,
    })
}

fn sql_string(error: rusqlite::Error) -> String {
    error.to_string()
}
