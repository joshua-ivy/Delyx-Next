use crate::patch_bridge::{
    DiffLineView, PatchBridgeStore, PatchCheckpointFileView, PatchFileView, PatchProposalView,
};
use rusqlite::{params, Connection};
use std::path::Path;

pub fn save_to_path(store: &PatchBridgeStore, path: &Path) -> Result<(), String> {
    let mut connection = crate::sqlite_store::open_migrated_database(path).map_err(sql_string)?;
    let transaction = connection.transaction().map_err(sql_string)?;
    clear_tables(&transaction)?;
    for proposal in &store.records {
        insert_proposal(&transaction, proposal)?;
        for (file_index, file) in proposal.files.iter().enumerate() {
            insert_file(&transaction, &proposal.id, file_index, file)?;
            for (diff_index, line) in file.diff.iter().enumerate() {
                insert_diff_line(&transaction, &proposal.id, file_index, diff_index, line)?;
            }
        }
        for (file_index, file) in proposal.checkpoint_files.iter().enumerate() {
            insert_checkpoint_file(&transaction, &proposal.id, file_index, file)?;
        }
    }
    transaction.commit().map_err(sql_string)
}

pub fn load_from_path(path: &Path) -> Result<PatchBridgeStore, String> {
    let connection = crate::sqlite_store::open_migrated_database(path).map_err(sql_string)?;
    let mut records = load_proposals(&connection)?;
    for proposal in &mut records {
        proposal.files = load_files(&connection, &proposal.id)?;
        proposal.checkpoint_files = load_checkpoint_files(&connection, &proposal.id)?;
    }
    Ok(PatchBridgeStore {
        next_patch_id: next_patch_id(&records),
        records,
    })
}

fn clear_tables(connection: &Connection) -> Result<(), String> {
    connection
        .execute_batch(
            "DELETE FROM patch_diff_lines;
             DELETE FROM patch_checkpoint_files;
             DELETE FROM patch_proposal_files;
             DELETE FROM patch_proposal_records;",
        )
        .map_err(sql_string)
}

fn insert_proposal(connection: &Connection, proposal: &PatchProposalView) -> Result<(), String> {
    connection
        .execute(
            "INSERT INTO patch_proposal_records
             (id, run_id, approval_id, status, checkpoint_id)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            params![
                proposal.id,
                proposal.run_id,
                proposal.approval_id,
                proposal.status,
                proposal.checkpoint_id,
            ],
        )
        .map(|_| ())
        .map_err(sql_string)
}

fn insert_file(
    connection: &Connection,
    proposal_id: &str,
    file_index: usize,
    file: &PatchFileView,
) -> Result<(), String> {
    connection
        .execute(
            "INSERT INTO patch_proposal_files
             (proposal_id, file_index, path, before_text, after_text)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            params![
                proposal_id,
                file_index as i64,
                file.path,
                file.before,
                file.after,
            ],
        )
        .map(|_| ())
        .map_err(sql_string)
}

fn insert_checkpoint_file(
    connection: &Connection,
    proposal_id: &str,
    file_index: usize,
    file: &PatchCheckpointFileView,
) -> Result<(), String> {
    connection
        .execute(
            "INSERT INTO patch_checkpoint_files (proposal_id, file_index, path, contents)
             VALUES (?1, ?2, ?3, ?4)",
            params![proposal_id, file_index as i64, file.path, file.contents],
        )
        .map(|_| ())
        .map_err(sql_string)
}

fn insert_diff_line(
    connection: &Connection,
    proposal_id: &str,
    file_index: usize,
    diff_index: usize,
    line: &DiffLineView,
) -> Result<(), String> {
    connection
        .execute(
            "INSERT INTO patch_diff_lines
             (proposal_id, file_index, diff_index, kind, text)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            params![
                proposal_id,
                file_index as i64,
                diff_index as i64,
                line.kind,
                line.text,
            ],
        )
        .map(|_| ())
        .map_err(sql_string)
}

fn load_proposals(connection: &Connection) -> Result<Vec<PatchProposalView>, String> {
    let mut statement = connection
        .prepare(
            "SELECT id, run_id, approval_id, status, checkpoint_id
             FROM patch_proposal_records ORDER BY rowid",
        )
        .map_err(sql_string)?;
    let rows = statement
        .query_map([], |row| {
            Ok(PatchProposalView {
                id: row.get(0)?,
                run_id: row.get(1)?,
                approval_id: row.get(2)?,
                status: row.get(3)?,
                checkpoint_id: row.get(4)?,
                checkpoint_files: Vec::new(),
                files: Vec::new(),
            })
        })
        .map_err(sql_string)?;
    rows.collect::<Result<Vec<_>, _>>().map_err(sql_string)
}

fn load_files(connection: &Connection, proposal_id: &str) -> Result<Vec<PatchFileView>, String> {
    let mut statement = connection
        .prepare(
            "SELECT file_index, path, before_text, after_text FROM patch_proposal_files
             WHERE proposal_id = ?1 ORDER BY file_index",
        )
        .map_err(sql_string)?;
    let rows = statement
        .query_map(params![proposal_id], |row| {
            Ok((
                row.get::<_, i64>(0)? as usize,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, String>(3)?,
            ))
        })
        .map_err(sql_string)?;
    let files = rows.collect::<Result<Vec<_>, _>>().map_err(sql_string)?;
    files
        .into_iter()
        .map(|(file_index, path, before, after)| {
            Ok(PatchFileView {
                after,
                before,
                path,
                diff: load_diff_lines(connection, proposal_id, file_index)?,
            })
        })
        .collect()
}

fn load_checkpoint_files(
    connection: &Connection,
    proposal_id: &str,
) -> Result<Vec<PatchCheckpointFileView>, String> {
    let mut statement = connection
        .prepare(
            "SELECT path, contents FROM patch_checkpoint_files
             WHERE proposal_id = ?1 ORDER BY file_index",
        )
        .map_err(sql_string)?;
    let rows = statement
        .query_map(params![proposal_id], |row| {
            Ok(PatchCheckpointFileView {
                path: row.get(0)?,
                contents: row.get(1)?,
            })
        })
        .map_err(sql_string)?;
    rows.collect::<Result<Vec<_>, _>>().map_err(sql_string)
}

fn load_diff_lines(
    connection: &Connection,
    proposal_id: &str,
    file_index: usize,
) -> Result<Vec<DiffLineView>, String> {
    let mut statement = connection
        .prepare(
            "SELECT kind, text FROM patch_diff_lines
             WHERE proposal_id = ?1 AND file_index = ?2 ORDER BY diff_index",
        )
        .map_err(sql_string)?;
    let rows = statement
        .query_map(params![proposal_id, file_index as i64], |row| {
            Ok(DiffLineView {
                kind: row.get(0)?,
                text: row.get(1)?,
            })
        })
        .map_err(sql_string)?;
    rows.collect::<Result<Vec<_>, _>>().map_err(sql_string)
}

fn next_patch_id(records: &[PatchProposalView]) -> usize {
    records
        .iter()
        .filter_map(|record| record.id.strip_prefix("patch-")?.parse::<usize>().ok())
        .max()
        .unwrap_or(records.len())
}

fn sql_string(error: rusqlite::Error) -> String {
    error.to_string()
}
