#[cfg(test)]
mod tests {
    use crate::context_pack::{
        load_context_pack_from_path, save_context_pack_to_path, select_context, ChunkCandidate,
        ContextPack,
    };
    use std::collections::HashSet;
    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn candidate(locator: &str, tokens: u32) -> ChunkCandidate {
        ChunkCandidate {
            attachment_id: "attach-1".to_string(),
            locator: locator.to_string(),
            text: format!("text for {locator}"),
            token_estimate: tokens,
        }
    }

    #[test]
    fn budget_excludes_chunks_that_do_not_fit() {
        let candidates = vec![
            candidate("f#L1-L80", 60),
            candidate("f#L81-L160", 60),
            candidate("f#L161-L240", 60),
        ];
        let selection = select_context(candidates, 130, &HashSet::new());
        assert_eq!(selection.items.len(), 2);
        assert_eq!(selection.used_tokens, 120);
        assert_eq!(selection.excluded_count, 1);
        assert_eq!(selection.status, "partial");
        assert_eq!(selection.strategy, "direct_excerpt");
        assert!(selection
            .items
            .iter()
            .all(|i| i.inclusion_reason == "within budget"));
    }

    #[test]
    fn pinned_chunks_are_always_included_even_past_budget() {
        let candidates = vec![candidate("pin#L1-L80", 500), candidate("other#L1-L80", 10)];
        let mut pinned = HashSet::new();
        pinned.insert("pin#L1-L80".to_string());
        let selection = select_context(candidates, 100, &pinned);
        // Pinned (500) is in despite exceeding the 100 budget; the small one is excluded.
        assert!(selection
            .items
            .iter()
            .any(|i| i.locator == "pin#L1-L80" && i.inclusion_reason == "pinned"));
        assert_eq!(selection.strategy, "manual_pin");
        assert_eq!(selection.excluded_count, 1);
    }

    #[test]
    fn everything_fitting_is_ready() {
        let selection = select_context(
            vec![candidate("a", 10), candidate("b", 10)],
            1000,
            &HashSet::new(),
        );
        assert_eq!(selection.status, "ready");
        assert_eq!(selection.excluded_count, 0);
    }

    #[test]
    fn context_pack_survives_sqlite_reload() {
        let path = temp_path("context-pack");
        let selection = select_context(vec![candidate("a", 10)], 1000, &HashSet::new());
        let pack = ContextPack {
            id: "pack-1".to_string(),
            project_id: "p1".to_string(),
            thread_id: "t1".to_string(),
            run_id: None,
            strategy: selection.strategy,
            budget_tokens: 1000,
            used_tokens: selection.used_tokens,
            status: selection.status,
            items: selection.items,
            created_at: String::new(),
            excluded_count: selection.excluded_count,
        };
        save_context_pack_to_path(&path, &pack).unwrap();
        let loaded = load_context_pack_from_path(&path, "pack-1")
            .unwrap()
            .unwrap();
        assert_eq!(loaded.items.len(), 1);
        assert_eq!(loaded.items[0].locator, "a");
        assert!(!loaded.created_at.is_empty());

        let _ = std::fs::remove_file(path);
    }

    fn temp_path(name: &str) -> PathBuf {
        let stamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        std::env::temp_dir().join(format!("delyx-next-{name}-{stamp}.sqlite3"))
    }
}
