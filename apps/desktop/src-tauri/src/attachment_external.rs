//! URL/connector snapshot ingestion. The webview performs the actual fetch (it
//! can reach HTTPS), then hands the fetched text here; the backend turns it into
//! chunks on the existing native attachment record, preserving the source
//! locator and the retrieval time inside the chunk locator. An empty snapshot
//! means the fetch failed and is reported as such (no fake content).

use crate::attachment_parser::{ParsedChunk, MAX_PARSE_BYTES};

/// Build snapshot chunks for fetched external content. Returns empty when there
/// is nothing to store (treated as a fetch failure by the caller). The locator
/// embeds both the source locator and the retrieval time so a citation is
/// reproducible: `https://x/y#retrieved=<ms>`.
pub fn external_snapshot_chunks(
    locator_base: &str,
    content: &str,
    retrieved_at: &str,
) -> Vec<ParsedChunk> {
    let trimmed = content.trim();
    if trimmed.is_empty() {
        return Vec::new();
    }
    let capped: String = trimmed.chars().take(MAX_PARSE_BYTES).collect();
    let locator = format!("{locator_base}#retrieved={retrieved_at}");
    vec![ParsedChunk {
        index: 0,
        kind: "snapshot".to_string(),
        title: format!("{locator_base} (retrieved {retrieved_at})"),
        locator,
        token_estimate: (capped.len() / 4).max(1) as u32,
        content_hash: hash(&capped),
        text: capped,
    }]
}

fn hash(text: &str) -> String {
    let mut hash: u64 = 1469598103934665603;
    for byte in text.bytes() {
        hash ^= u64::from(byte);
        hash = hash.wrapping_mul(1099511628211);
    }
    format!("{hash:016x}")
}
