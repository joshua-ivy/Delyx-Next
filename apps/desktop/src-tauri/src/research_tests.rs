#[cfg(test)]
mod tests {
    use crate::research::{
        ClaimStatus, EvidenceInput, EvidenceSourceKind, EvidenceStance, ResearchAgent,
    };

    #[test]
    fn research_answer_can_show_evidence_receipts() {
        let mut agent = ResearchAgent::new();
        agent.add_evidence(evidence(
            "Delyx Next is local-first.",
            EvidenceSourceKind::LocalFile,
            EvidenceStance::Supports,
        ));

        let answer = agent.answer(
            "run-1",
            "What is Delyx Next?",
            vec!["Delyx Next is local-first."],
        );

        assert_eq!(answer.summary, "Evidence supports the audited claims.");
        assert_eq!(answer.receipts.len(), 1);
        assert_eq!(answer.audits[0].status, ClaimStatus::Supported);
    }

    #[test]
    fn missing_evidence_produces_insufficient_evidence() {
        let mut agent = ResearchAgent::new();

        let answer = agent.answer("run-1", "What changed?", vec!["The patch updates routing."]);

        assert_eq!(answer.summary, "insufficient evidence.");
        assert_eq!(answer.audits[0].status, ClaimStatus::InsufficientEvidence);
    }

    #[test]
    fn numeric_and_date_claims_require_support() {
        let mut agent = ResearchAgent::new();

        let answer = agent.answer(
            "run-1",
            "When?",
            vec![
                "There are 3 provider routes.",
                "The release date is June 7.",
            ],
        );

        assert!(answer.audits.iter().all(|audit| audit.requires_support));
        assert!(answer
            .audits
            .iter()
            .all(|audit| audit.status == ClaimStatus::InsufficientEvidence));
    }

    #[test]
    fn conflicting_evidence_is_shown_clearly() {
        let mut agent = ResearchAgent::new();
        agent.add_evidence(evidence(
            "The command passed.",
            EvidenceSourceKind::Test,
            EvidenceStance::Supports,
        ));
        agent.add_evidence(evidence(
            "The command passed.",
            EvidenceSourceKind::Terminal,
            EvidenceStance::Contradicts,
        ));

        let answer = agent.answer("run-1", "Did it pass?", vec!["The command passed."]);

        assert_eq!(answer.audits[0].status, ClaimStatus::Contradicted);
        assert_eq!(answer.contradictions.len(), 1);
        assert!(answer.contradictions[0]
            .message
            .contains("Conflicting evidence"));
    }

    #[test]
    fn local_and_repo_evidence_sort_before_web_sources() {
        let mut agent = ResearchAgent::new();
        agent.add_evidence(evidence(
            "The project uses AGENTS.md.",
            EvidenceSourceKind::Web,
            EvidenceStance::Supports,
        ));
        agent.add_evidence(evidence(
            "The project uses AGENTS.md.",
            EvidenceSourceKind::LocalFile,
            EvidenceStance::Supports,
        ));

        let answer = agent.answer(
            "run-1",
            "What rules exist?",
            vec!["The project uses AGENTS.md."],
        );

        assert_eq!(
            answer.receipts[0].source_kind,
            EvidenceSourceKind::LocalFile
        );
        assert_eq!(answer.receipts[1].source_kind, EvidenceSourceKind::Web);
    }

    fn evidence(
        claim: &str,
        source_kind: EvidenceSourceKind,
        stance: EvidenceStance,
    ) -> EvidenceInput {
        EvidenceInput {
            claim: claim.to_string(),
            excerpt: format!("Excerpt for {claim}"),
            locator: "docs/source.md:1".to_string(),
            run_id: "run-1".to_string(),
            source_kind,
            stance,
            title: "Source receipt".to_string(),
        }
    }
}
