//! Context packs: the scoped, budgeted, model-ready subset of attachment chunks
//! for one thread. Chat sees only the pack — never every chunk of every file.
//! Selection is a pure function (pinned first, then fill the token budget) so it
//! is unit-testable; persistence lives alongside it.

use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::path::Path;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ContextPackItem {
    pub attachment_id: Option<String>,
    pub evidence_record_id: Option<String>,
    pub locator: String,
    pub text: String,
    pub token_estimate: u32,
    pub inclusion_reason: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ContextPack {
    pub id: String,
    pub project_id: String,
    pub thread_id: String,
    pub run_id: Option<String>,
    pub strategy: String,
    pub budget_tokens: u32,
    pub used_tokens: u32,
    /// "ready" when everything fit, "partial" when budget excluded chunks.
    pub status: String,
    pub items: Vec<ContextPackItem>,
    pub created_at: String,
    /// How many available chunks were left out for budget (UI: what was excluded).
    pub excluded_count: u32,
}

/// One candidate chunk for inclusion in a pack.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChunkCandidate {
    pub attachment_id: String,
    pub locator: String,
    pub text: String,
    pub token_estimate: u32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Selection {
    pub items: Vec<ContextPackItem>,
    pub used_tokens: u32,
    pub strategy: String,
    pub status: String,
    pub excluded_count: u32,
}

/// Pick chunks for a pack: pinned/manual locators are always included (even past
/// budget), then remaining chunks fill the token budget in order. Excluded chunks
/// are counted so the UI can show what didn't fit.
pub fn select_context(
    candidates: Vec<ChunkCandidate>,
    budget_tokens: u32,
    pinned: &HashSet<String>,
) -> Selection {
    let mut items = Vec::new();
    let mut used = 0u32;
    let mut excluded = 0u32;
    let mut had_pinned = false;
    let mut had_budgeted = false;

    // Pinned first — always in.
    for candidate in candidates.iter().filter(|c| pinned.contains(&c.locator)) {
        used = used.saturating_add(candidate.token_estimate);
        had_pinned = true;
        items.push(item(candidate, "pinned"));
    }
    // Then budgeted fill.
    for candidate in candidates.iter().filter(|c| !pinned.contains(&c.locator)) {
        if used.saturating_add(candidate.token_estimate) <= budget_tokens {
            used = used.saturating_add(candidate.token_estimate);
            had_budgeted = true;
            items.push(item(candidate, "within budget"));
        } else {
            excluded += 1;
        }
    }

    let strategy = match (had_pinned, had_budgeted) {
        (true, true) => "mixed",
        (true, false) => "manual_pin",
        _ => "direct_excerpt",
    }
    .to_string();
    let status = if excluded > 0 { "partial" } else { "ready" }.to_string();
    Selection {
        items,
        used_tokens: used,
        strategy,
        status,
        excluded_count: excluded,
    }
}

fn item(candidate: &ChunkCandidate, reason: &str) -> ContextPackItem {
    ContextPackItem {
        attachment_id: Some(candidate.attachment_id.clone()),
        evidence_record_id: None,
        locator: candidate.locator.clone(),
        text: candidate.text.clone(),
        token_estimate: candidate.token_estimate,
        inclusion_reason: reason.to_string(),
    }
}

// ---- Persistence ----

pub fn save_context_pack_to_path(path: &Path, pack: &ContextPack) -> Result<ContextPack, String> {
    let connection = crate::sqlite_store::open_migrated_database(path).map_err(sql_string)?;
    connection
        .execute(
            "INSERT INTO context_packs (
                id, project_id, thread_id, run_id, strategy, budget_tokens, used_tokens, status
             ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)
             ON CONFLICT(id) DO UPDATE SET
                strategy = excluded.strategy,
                budget_tokens = excluded.budget_tokens,
                used_tokens = excluded.used_tokens,
                status = excluded.status",
            params![
                pack.id,
                pack.project_id,
                pack.thread_id,
                pack.run_id,
                pack.strategy,
                pack.budget_tokens as i64,
                pack.used_tokens as i64,
                pack.status,
            ],
        )
        .map_err(sql_string)?;
    connection
        .execute(
            "DELETE FROM context_pack_items WHERE context_pack_id = ?1",
            params![pack.id],
        )
        .map_err(sql_string)?;
    for (index, item) in pack.items.iter().enumerate() {
        connection
            .execute(
                "INSERT INTO context_pack_items (
                    context_pack_id, item_index, attachment_id, evidence_record_id,
                    locator, text, token_estimate, inclusion_reason
                 ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
                params![
                    pack.id,
                    index as i64,
                    item.attachment_id,
                    item.evidence_record_id,
                    item.locator,
                    item.text,
                    item.token_estimate as i64,
                    item.inclusion_reason,
                ],
            )
            .map_err(sql_string)?;
    }
    load_context_pack_from_path(path, &pack.id)?
        .ok_or_else(|| "Context pack disappeared immediately after saving.".to_string())
}

pub fn load_context_pack_from_path(path: &Path, id: &str) -> Result<Option<ContextPack>, String> {
    let connection = crate::sqlite_store::open_migrated_database(path).map_err(sql_string)?;
    let pack = connection
        .query_row(
            "SELECT id, project_id, thread_id, run_id, strategy, budget_tokens, used_tokens,
                    status, created_at
             FROM context_packs WHERE id = ?1",
            params![id.trim()],
            |row| {
                Ok(ContextPack {
                    id: row.get(0)?,
                    project_id: row.get(1)?,
                    thread_id: row.get(2)?,
                    run_id: row.get(3)?,
                    strategy: row.get(4)?,
                    budget_tokens: row.get::<_, i64>(5)? as u32,
                    used_tokens: row.get::<_, i64>(6)? as u32,
                    status: row.get(7)?,
                    items: Vec::new(),
                    created_at: row.get(8)?,
                    excluded_count: 0,
                })
            },
        )
        .ok();
    let Some(mut pack) = pack else {
        return Ok(None);
    };
    pack.items = load_items(&connection, &pack.id)?;
    Ok(Some(pack))
}

fn load_items(connection: &Connection, pack_id: &str) -> Result<Vec<ContextPackItem>, String> {
    let mut statement = connection
        .prepare(
            "SELECT attachment_id, evidence_record_id, locator, text, token_estimate, inclusion_reason
             FROM context_pack_items WHERE context_pack_id = ?1 ORDER BY item_index",
        )
        .map_err(sql_string)?;
    let rows = statement
        .query_map(params![pack_id], |row| {
            Ok(ContextPackItem {
                attachment_id: row.get(0)?,
                evidence_record_id: row.get(1)?,
                locator: row.get(2)?,
                text: row.get(3)?,
                token_estimate: row.get::<_, i64>(4)? as u32,
                inclusion_reason: row.get(5)?,
            })
        })
        .map_err(sql_string)?;
    rows.collect::<Result<Vec<_>, _>>().map_err(sql_string)
}

fn sql_string(error: rusqlite::Error) -> String {
    error.to_string()
}
