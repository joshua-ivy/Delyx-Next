#[cfg(test)]
mod tests {
    use crate::attachment_media::{chunk_pdf_pages, safe_extract_path};
    use std::path::Path;

    #[test]
    fn pdf_pages_become_page_chunks_with_locators() {
        let pages = vec![
            "Title page".to_string(),
            "   ".to_string(), // blank page skipped
            "Body of page three".to_string(),
        ];
        let chunks = chunk_pdf_pages("spec.pdf", &pages);
        assert_eq!(chunks.len(), 2);
        assert_eq!(chunks[0].locator, "spec.pdf#page=1");
        // Page 2 was blank, so the next chunk is page 3 (locator keeps the page number).
        assert_eq!(chunks[1].locator, "spec.pdf#page=3");
        assert!(chunks[1].text.contains("page three"));
    }

    #[test]
    fn no_extractable_text_yields_no_chunks() {
        let chunks = chunk_pdf_pages("scan.pdf", &["".to_string(), "  ".to_string()]);
        assert!(chunks.is_empty());
    }

    #[test]
    fn archive_entries_cannot_escape_the_extraction_root() {
        let root = Path::new("/tmp/extract");
        // Safe entries resolve under the root.
        let ok = safe_extract_path(root, "docs/readme.md").unwrap();
        assert!(ok.ends_with("docs/readme.md"));
        assert!(ok.starts_with(root));

        // Traversal, absolute, and drive-prefixed entries are rejected.
        assert!(safe_extract_path(root, "../escape.txt").is_err());
        assert!(safe_extract_path(root, "a/../../escape.txt").is_err());
        assert!(safe_extract_path(root, "/etc/passwd").is_err());
        assert!(safe_extract_path(root, "..\\windows\\win.ini").is_err());
        assert!(safe_extract_path(root, "  ").is_err());
    }
}
