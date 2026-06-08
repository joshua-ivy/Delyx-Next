#[cfg(test)]
mod tests {
    use crate::research::{EvidenceInput, EvidenceSourceKind, EvidenceStance, EvidenceStore};
    use crate::research_persistence::{load_from_path, save_to_path};
    use std::fs;
    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn evidence_store_survives_sqlite_reload() {
        let path = temp_path("research-evidence");
        let mut store = EvidenceStore::new();
        let first = store.add(evidence(
            "run-1",
            EvidenceSourceKind::LocalFile,
            EvidenceStance::Supports,
            "src/research.rs",
            "claim is persisted",
        ));
        store.add(evidence(
            "run-1",
            EvidenceSourceKind::Web,
            EvidenceStance::Contradicts,
            "https://example.invalid/evidence",
            "claim is persisted",
        ));
        store.add(evidence(
            "run-2",
            EvidenceSourceKind::Terminal,
            EvidenceStance::Supports,
            "cargo test research",
            "separate run receipt",
        ));

        save_to_path(&store, &path).unwrap();
        assert!(fs::read(&path).unwrap().starts_with(b"SQLite format 3"));

        let mut loaded = load_from_path(&path).unwrap();
        let run_records = loaded.for_run("run-1");
        assert_eq!(run_records.len(), 2);
        assert_eq!(run_records[0], first);
        assert_eq!(run_records[0].locator, "src/research.rs");
        assert_eq!(
            run_records[0].excerpt,
            "Evidence quote from src/research.rs."
        );
        assert_eq!(run_records[0].claim_key, "claim is persisted");
        assert_eq!(
            loaded.for_run("run-2")[0].source_kind,
            EvidenceSourceKind::Terminal
        );

        let next = loaded.add(evidence(
            "run-3",
            EvidenceSourceKind::ModelCall,
            EvidenceStance::Supports,
            "ollama-local",
            "new receipt after reload",
        ));
        assert_eq!(next.id, "evidence-4");
        let _ = fs::remove_file(path);
    }

    fn evidence(
        run_id: &str,
        source_kind: EvidenceSourceKind,
        stance: EvidenceStance,
        locator: &str,
        claim: &str,
    ) -> EvidenceInput {
        EvidenceInput {
            run_id: run_id.to_string(),
            source_kind,
            title: format!("Evidence from {locator}"),
            locator: locator.to_string(),
            excerpt: format!("Evidence quote from {locator}."),
            stance,
            claim: claim.to_string(),
        }
    }

    fn temp_path(name: &str) -> PathBuf {
        let stamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        std::env::temp_dir().join(format!("delyx-next-{name}-{stamp}.sqlite3"))
    }
}
