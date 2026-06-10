//! Non-text attachment handling: PDF page chunking, image policy, and the
//! archive extraction-path safety gate.
//!
//! PDF text extraction itself (bytes -> page strings) needs a PDF library or a
//! webview-side extractor; this module takes already-extracted page texts and
//! chunks them per page. Archive *extraction* needs an archive reader, but the
//! path-traversal guard — the part that keeps an archive from escaping its
//! extraction root — is implemented and tested here.

use crate::attachment_parser::ParsedChunk;
use std::path::{Component, Path, PathBuf};

/// Chunk extracted PDF page texts into one chunk per non-empty page, with
/// `name#page=N` locators (1-based). Empty pages are skipped.
pub fn chunk_pdf_pages(name: &str, pages: &[String]) -> Vec<ParsedChunk> {
    let mut chunks = Vec::new();
    let mut index = 0;
    for (page_number, page) in pages.iter().enumerate() {
        let text = page.trim();
        if text.is_empty() {
            continue;
        }
        let page_label = page_number + 1;
        chunks.push(ParsedChunk {
            index,
            kind: "pdf_page".to_string(),
            title: format!("{name} page {page_label}"),
            locator: format!("{name}#page={page_label}"),
            text: text.to_string(),
            token_estimate: (text.len() / 4).max(1) as u32,
            content_hash: hash(text),
        });
        index += 1;
    }
    chunks
}

/// Resolve where an archive entry may be extracted, refusing anything that would
/// escape `root` (parent traversal, absolute paths, drive prefixes). Callers MUST
/// use the returned path and never the raw entry name.
pub fn safe_extract_path(root: &Path, entry: &str) -> Result<PathBuf, String> {
    let normalized = entry.replace('\\', "/");
    if normalized.trim().is_empty() {
        return Err("Archive entry has an empty name.".to_string());
    }
    let relative = Path::new(&normalized);
    for component in relative.components() {
        match component {
            Component::Normal(_) | Component::CurDir => {}
            Component::ParentDir => {
                return Err(format!(
                    "Archive entry escapes the extraction root: {entry}"
                ));
            }
            Component::RootDir | Component::Prefix(_) => {
                return Err(format!("Archive entry is an absolute path: {entry}"));
            }
        }
    }
    Ok(root.join(relative))
}

fn hash(text: &str) -> String {
    let mut hash: u64 = 1469598103934665603;
    for byte in text.bytes() {
        hash ^= u64::from(byte);
        hash = hash.wrapping_mul(1099511628211);
    }
    format!("{hash:016x}")
}
