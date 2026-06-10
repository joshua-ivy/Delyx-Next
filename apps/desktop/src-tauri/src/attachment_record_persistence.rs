//! SQLite persistence for durable attachment records and their parsed chunks.

use crate::attachment::{
    AttachmentIndexStatus, AttachmentKind, AttachmentParseStatus, AttachmentRecord,
    AttachmentSourceKind,
};
use crate::attachment_persistence::sql_string;
use rusqlite::{params, Connection, OptionalExtension};
use std::path::Path;

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
