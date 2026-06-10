#[cfg(test)]
mod tests {
    use crate::campaign_dice::{detect_check, resolve, sheet_stat};

    fn checks() -> Vec<String> {
        vec![
            "grit".to_string(),
            "wits".to_string(),
            "aim".to_string(),
            "charm".to_string(),
        ]
    }

    #[test]
    fn risky_actions_map_to_pack_checks() {
        assert_eq!(
            detect_check("I charge across no-man's-land", &checks()),
            Some("grit".to_string())
        );
        assert_eq!(
            detect_check("Take the shot before he sees us", &checks()),
            Some("aim".to_string())
        );
        assert_eq!(
            detect_check("We sneak past the listening post", &checks()),
            Some("wits".to_string())
        );
        assert_eq!(
            detect_check("Try to convince the sergeant to wait", &checks()),
            Some("charm".to_string())
        );
        assert_eq!(
            detect_check("I look at the photographs again", &checks()),
            None
        );
    }

    #[test]
    fn checks_outside_the_pack_are_ignored() {
        let only_charm = vec!["charm".to_string()];
        assert_eq!(detect_check("I shoot the lock off", &only_charm), None);
    }

    #[test]
    fn resolution_is_deterministic_and_banded() {
        let first = resolve("aim", 1, 42);
        let second = resolve("aim", 1, 42);
        assert_eq!(first, second);
        assert_eq!(first.roll.len(), 2);
        assert!(first.roll.iter().all(|die| (1..=6).contains(die)));
        assert_eq!(
            first.total,
            i64::from(first.roll[0]) + i64::from(first.roll[1]) + 1
        );
        let expected = if first.total >= 10 {
            "success"
        } else if first.total >= 7 {
            "partial"
        } else {
            "setback"
        };
        assert_eq!(first.outcome, expected);

        // Different seeds explore the full band space.
        let outcomes: std::collections::HashSet<String> = (0..200u64)
            .map(|seed| resolve("grit", 0, seed).outcome)
            .collect();
        assert!(outcomes.contains("success"));
        assert!(outcomes.contains("partial"));
        assert!(outcomes.contains("setback"));
    }

    #[test]
    fn sheet_stat_reads_the_character_sheet() {
        assert_eq!(sheet_stat("{\"grit\":2,\"aim\":-1}", "grit"), 2);
        assert_eq!(sheet_stat("{\"grit\":2,\"aim\":-1}", "aim"), -1);
        assert_eq!(sheet_stat("{\"grit\":2}", "wits"), 0);
        assert_eq!(sheet_stat("not json", "grit"), 0);
    }
}
