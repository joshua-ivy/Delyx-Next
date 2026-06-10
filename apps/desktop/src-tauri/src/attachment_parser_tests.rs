#[cfg(test)]
mod tests {
    use crate::attachment::AttachmentKind;
    use crate::attachment_parser::{is_text_like, parse_attachment_text, MAX_PARSE_BYTES};

    #[test]
    fn code_is_chunked_into_line_windows_with_locators() {
        let body = (1..=200)
            .map(|n| format!("let x{n} = {n};"))
            .collect::<Vec<_>>()
            .join("\n");
        let out = parse_attachment_text("main.rs", AttachmentKind::Code, &body);
        assert!(!out.partial);
        // 200 lines / 80 per chunk = 3 chunks.
        assert_eq!(out.chunks.len(), 3);
        assert_eq!(out.chunks[0].locator, "main.rs#L1-L80");
        assert_eq!(out.chunks[1].locator, "main.rs#L81-L160");
        assert_eq!(out.chunks[2].locator, "main.rs#L161-L200");
        assert!(out.chunks[0].token_estimate > 0);
    }

    #[test]
    fn markdown_splits_on_headings() {
        let body = "# Title\nintro line\n\n## Section A\nalpha\n\n## Section B\nbeta";
        let out = parse_attachment_text("notes.md", AttachmentKind::Markdown, &body);
        assert_eq!(out.chunks.len(), 3);
        assert!(out.chunks[0].text.contains("# Title"));
        assert!(out.chunks[1].text.contains("Section A"));
        assert!(out.chunks[2].text.contains("Section B"));
    }

    #[test]
    fn oversized_content_is_truncated_and_marked_partial() {
        let big = "a\n".repeat(MAX_PARSE_BYTES); // ~2x the cap
        let out = parse_attachment_text("big.txt", AttachmentKind::Text, &big);
        assert!(out.partial);
        assert!(!out.chunks.is_empty());
    }

    #[test]
    fn empty_content_yields_no_chunks() {
        let out = parse_attachment_text("empty.txt", AttachmentKind::Text, "   \n  \n");
        assert!(out.chunks.is_empty());
        assert!(!out.partial);
    }

    #[test]
    fn only_text_like_kinds_parse() {
        assert!(is_text_like(AttachmentKind::Code));
        assert!(is_text_like(AttachmentKind::Markdown));
        assert!(!is_text_like(AttachmentKind::Pdf));
        assert!(!is_text_like(AttachmentKind::Image));
        assert!(!is_text_like(AttachmentKind::Binary));
    }
}
