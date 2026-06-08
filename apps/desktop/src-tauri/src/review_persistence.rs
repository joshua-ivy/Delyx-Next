use crate::review::ReviewAgent;
use crate::review_bridge::{ReviewBridgeStore, ReviewFindingView, ReviewReportView};
use rusqlite::{params, Connection};
use std::path::Path;

pub fn save_to_path(store: &ReviewBridgeStore, path: &Path) -> Result<(), String> {
    let mut connection = crate::sqlite_store::open_migrated_database(path).map_err(sql_string)?;
    let transaction = connection.transaction().map_err(sql_string)?;
    clear_tables(&transaction)?;
    for report in &store.reports {
        insert_report(&transaction, report)?;
        for (index, finding) in report.findings.iter().enumerate() {
            insert_finding(&transaction, &report.id, index, finding)?;
        }
    }
    transaction.commit().map_err(sql_string)
}

pub fn load_from_path(path: &Path) -> Result<ReviewBridgeStore, String> {
    let connection = crate::sqlite_store::open_migrated_database(path).map_err(sql_string)?;
    let mut reports = load_reports(&connection)?;
    for report in &mut reports {
        report.findings = load_findings(&connection, &report.id)?;
    }
    Ok(ReviewBridgeStore {
        agent: ReviewAgent::with_loaded_counters(
            next_report_id(&reports),
            next_finding_id(&reports),
        ),
        reports,
    })
}

fn clear_tables(connection: &Connection) -> Result<(), String> {
    connection
        .execute_batch(
            "DELETE FROM review_findings;
             DELETE FROM review_report_records;",
        )
        .map_err(sql_string)
}

fn insert_report(connection: &Connection, report: &ReviewReportView) -> Result<(), String> {
    connection
        .execute(
            "INSERT INTO review_report_records
             (id, run_id, mode, decision, risk_summary, test_summary, evidence_summary)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params![
                report.id,
                report.run_id,
                report.mode,
                report.decision,
                report.risk_summary,
                report.test_summary,
                report.evidence_summary,
            ],
        )
        .map(|_| ())
        .map_err(sql_string)
}

fn insert_finding(
    connection: &Connection,
    report_id: &str,
    index: usize,
    finding: &ReviewFindingView,
) -> Result<(), String> {
    connection
        .execute(
            "INSERT INTO review_findings
             (report_id, finding_index, id, priority, title, detail, risk_label,
              suggested_fix, file_path, hunk_label)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
            params![
                report_id,
                index as i64,
                finding.id,
                finding.priority,
                finding.title,
                finding.detail,
                finding.risk_label,
                finding.suggested_fix,
                finding.file_path,
                finding.hunk_label,
            ],
        )
        .map(|_| ())
        .map_err(sql_string)
}

fn load_reports(connection: &Connection) -> Result<Vec<ReviewReportView>, String> {
    let mut statement = connection
        .prepare(
            "SELECT id, run_id, mode, decision, risk_summary, test_summary, evidence_summary
             FROM review_report_records ORDER BY rowid",
        )
        .map_err(sql_string)?;
    let rows = statement
        .query_map([], |row| {
            Ok(ReviewReportView {
                id: row.get(0)?,
                run_id: row.get(1)?,
                mode: row.get(2)?,
                decision: row.get(3)?,
                risk_summary: row.get(4)?,
                test_summary: row.get(5)?,
                evidence_summary: row.get(6)?,
                findings: Vec::new(),
            })
        })
        .map_err(sql_string)?;
    rows.collect::<Result<Vec<_>, _>>().map_err(sql_string)
}

fn load_findings(
    connection: &Connection,
    report_id: &str,
) -> Result<Vec<ReviewFindingView>, String> {
    let mut statement = connection
        .prepare(
            "SELECT id, priority, title, detail, risk_label, suggested_fix, file_path, hunk_label
             FROM review_findings WHERE report_id = ?1 ORDER BY finding_index",
        )
        .map_err(sql_string)?;
    let rows = statement
        .query_map(params![report_id], |row| {
            Ok(ReviewFindingView {
                id: row.get(0)?,
                priority: row.get(1)?,
                title: row.get(2)?,
                detail: row.get(3)?,
                risk_label: row.get(4)?,
                suggested_fix: row.get(5)?,
                file_path: row.get(6)?,
                hunk_label: row.get(7)?,
            })
        })
        .map_err(sql_string)?;
    rows.collect::<Result<Vec<_>, _>>().map_err(sql_string)
}

fn next_report_id(reports: &[ReviewReportView]) -> usize {
    reports
        .iter()
        .filter_map(|report| report.id.strip_prefix("review-")?.parse::<usize>().ok())
        .max()
        .unwrap_or(reports.len())
}

fn next_finding_id(reports: &[ReviewReportView]) -> usize {
    reports
        .iter()
        .flat_map(|report| report.findings.iter())
        .filter_map(|finding| finding.id.strip_prefix("finding-")?.parse::<usize>().ok())
        .max()
        .unwrap_or(0)
}

fn sql_string(error: rusqlite::Error) -> String {
    error.to_string()
}
