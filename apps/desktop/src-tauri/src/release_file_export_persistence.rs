use crate::release::SupportBundleFileExport;
use rusqlite::{params, OptionalExtension};
use std::path::Path;

const FILE_EXPORT_ID: &str = "latest";

pub fn save_file_export_to_path(
    export: &SupportBundleFileExport,
    path: &Path,
) -> Result<(), String> {
    let connection = crate::sqlite_store::open_migrated_database(path).map_err(sql_string)?;
    connection
        .execute(
            "INSERT INTO support_bundle_file_exports
             (id, run_id, approval_id, path, exported_at, bytes_written)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)
             ON CONFLICT(id) DO UPDATE SET
               run_id = excluded.run_id,
               approval_id = excluded.approval_id,
               path = excluded.path,
               exported_at = excluded.exported_at,
               bytes_written = excluded.bytes_written",
            params![
                FILE_EXPORT_ID,
                &export.run_id,
                &export.approval_id,
                &export.path,
                &export.exported_at,
                export.bytes_written as i64,
            ],
        )
        .map(|_| ())
        .map_err(sql_string)
}

pub fn load_file_export_from_path(path: &Path) -> Result<Option<SupportBundleFileExport>, String> {
    let connection = crate::sqlite_store::open_migrated_database(path).map_err(sql_string)?;
    connection
        .query_row(
            "SELECT run_id, approval_id, path, exported_at, bytes_written
             FROM support_bundle_file_exports WHERE id = ?1",
            [FILE_EXPORT_ID],
            |row| {
                Ok(SupportBundleFileExport {
                    run_id: row.get(0)?,
                    approval_id: row.get(1)?,
                    path: row.get(2)?,
                    exported_at: row.get(3)?,
                    bytes_written: row.get::<_, i64>(4)? as u64,
                })
            },
        )
        .optional()
        .map_err(sql_string)
}

fn sql_string(error: rusqlite::Error) -> String {
    error.to_string()
}
