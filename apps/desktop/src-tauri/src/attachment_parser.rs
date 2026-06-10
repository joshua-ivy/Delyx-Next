//! Pure text/code/markdown chunking for attachments. Reads nothing from disk —
//! the bridge supplies content; this just splits it into chunks with stable
//! line-range locators so evidence can cite exact ranges later.

use crate::attachment::AttachmentKind;

/// Cap content per file so a giant paste/file can't blow up memory or context.
/// Anything beyond this is dropped and the result is marked `partial`.
pub const MAX_PARSE_BYTES: usize = 256_000;

/// Code/text files are chunked into windows of this many lines.
const LINES_PER_CHUNK: usize = 80;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedChunk {
    pub index: u32,
    pub kind: String,
    pub title: String,
    pub locator: String,
    pub text: String,
    pub token_estimate: u32,
    pub content_hash: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParseOutput {
    pub chunks: Vec<ParsedChunk>,
    /// True when the content was truncated at the byte cap.
    pub partial: bool,
}

pub fn is_text_like(kind: AttachmentKind) -> bool {
    matches!(
        kind,
        AttachmentKind::Text | AttachmentKind::Code | AttachmentKind::Markdown
    )
}

/// Chunk `raw` for `display_name`. Markdown splits on headings; code/text split
/// into fixed line windows. Locators are `name#Lstart-Lend` (1-based, inclusive).
pub fn parse_attachment_text(display_name: &str, kind: AttachmentKind, raw: &str) -> ParseOutput {
    let (content, partial) = cap_bytes(raw, MAX_PARSE_BYTES);
    let normalized = content.replace("\r\n", "\n").replace('\r', "\n");
    let chunks = if matches!(kind, AttachmentKind::Markdown) {
        chunk_markdown(display_name, &normalized)
    } else {
        chunk_by_lines(display_name, &normalized)
    };
    ParseOutput { chunks, partial }
}

fn cap_bytes(raw: &str, cap: usize) -> (String, bool) {
    if raw.len() <= cap {
        return (raw.to_string(), false);
    }
    // Truncate on a char boundary at or below the cap.
    let mut end = cap;
    while end > 0 && !raw.is_char_boundary(end) {
        end -= 1;
    }
    (raw[..end].to_string(), true)
}

fn chunk_by_lines(name: &str, content: &str) -> Vec<ParsedChunk> {
    let lines: Vec<&str> = content.lines().collect();
    if lines.is_empty() {
        return Vec::new();
    }
    let mut chunks = Vec::new();
    let mut start = 0;
    let mut index = 0;
    while start < lines.len() {
        let end = (start + LINES_PER_CHUNK).min(lines.len());
        let text = lines[start..end].join("\n");
        if !text.trim().is_empty() {
            chunks.push(make_chunk(index, "lines", name, start + 1, end, text));
            index += 1;
        }
        start = end;
    }
    chunks
}

fn chunk_markdown(name: &str, content: &str) -> Vec<ParsedChunk> {
    let lines: Vec<&str> = content.lines().collect();
    let mut chunks = Vec::new();
    let mut index = 0;
    let mut section_start = 0;
    let mut title = String::new();
    let flush = |chunks: &mut Vec<ParsedChunk>,
                 index: &mut u32,
                 start: usize,
                 end: usize,
                 title: &str,
                 lines: &[&str]| {
        if end <= start {
            return;
        }
        let text = lines[start..end].join("\n");
        if text.trim().is_empty() {
            return;
        }
        let heading = if title.is_empty() { "section" } else { title };
        chunks.push(make_chunk(*index, heading, name, start + 1, end, text));
        *index += 1;
    };
    for (line_index, line) in lines.iter().enumerate() {
        if line.trim_start().starts_with('#') {
            // Close the previous section before starting a new heading.
            flush(
                &mut chunks,
                &mut index,
                section_start,
                line_index,
                &title,
                &lines,
            );
            section_start = line_index;
            title = line.trim_start_matches('#').trim().to_string();
        }
    }
    flush(
        &mut chunks,
        &mut index,
        section_start,
        lines.len(),
        &title,
        &lines,
    );
    chunks
}

fn make_chunk(
    index: u32,
    kind: &str,
    name: &str,
    start_line: usize,
    end_line: usize,
    text: String,
) -> ParsedChunk {
    let token_estimate = (text.len() / 4).max(1) as u32;
    let content_hash = hash(&text);
    ParsedChunk {
        index,
        kind: kind.to_string(),
        title: format!("{name} L{start_line}-L{end_line}"),
        locator: format!("{name}#L{start_line}-L{end_line}"),
        text,
        token_estimate,
        content_hash,
    }
}

fn hash(text: &str) -> String {
    let mut hash: u64 = 1469598103934665603;
    for byte in text.bytes() {
        hash ^= u64::from(byte);
        hash = hash.wrapping_mul(1099511628211);
    }
    format!("{hash:016x}")
}
