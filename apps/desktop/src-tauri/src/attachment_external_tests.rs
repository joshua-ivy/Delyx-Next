#[cfg(test)]
mod tests {
    use crate::attachment_external::external_snapshot_chunks;

    #[test]
    fn snapshot_preserves_source_locator_and_retrieval_time() {
        let chunks = external_snapshot_chunks(
            "url:https://example.com/spec",
            "Spec body text.",
            "1700000000000",
        );
        assert_eq!(chunks.len(), 1);
        assert_eq!(
            chunks[0].locator,
            "url:https://example.com/spec#retrieved=1700000000000"
        );
        assert!(chunks[0].text.contains("Spec body"));
        assert!(chunks[0].content_hash.len() == 16);
    }

    #[test]
    fn failed_fetch_yields_no_chunks() {
        assert!(external_snapshot_chunks("url:https://x", "", "1").is_empty());
        assert!(external_snapshot_chunks("url:https://x", "   \n  ", "1").is_empty());
    }
}
