#[cfg(test)]
mod tests {
    use crate::campaign_packs::{builtin_packs, find_pack, find_scenario};

    #[test]
    fn builtin_packs_parse_and_include_ww1() {
        let packs = builtin_packs().unwrap();
        let ww1 = find_pack(&packs, "ww1").unwrap();
        assert_eq!(ww1.checks, vec!["grit", "wits", "aim", "charm"]);
        assert!(!ww1.gm_style.trim().is_empty());
        assert!(!ww1.rating_overlays.story.trim().is_empty());
        assert!(!ww1.rating_overlays.heroic.trim().is_empty());
        assert!(!ww1.rating_overlays.historical.trim().is_empty());
        assert!(!ww1.scenarios.is_empty());
    }

    #[test]
    fn every_era_pack_is_complete() {
        let packs = builtin_packs().unwrap();
        let ids: Vec<&str> = packs.iter().map(|pack| pack.id.as_str()).collect();
        assert_eq!(
            ids,
            vec![
                "revolutionary-war",
                "civil-war",
                "ww1",
                "ww2",
                "korea",
                "vietnam",
                "star-wars"
            ]
        );
        for pack in &packs {
            assert!(!pack.title.trim().is_empty(), "{} title", pack.id);
            assert!(!pack.gm_style.trim().is_empty(), "{} gm style", pack.id);
            assert!(!pack.checks.is_empty(), "{} checks", pack.id);
            assert!(
                !pack.rating_overlays.story.trim().is_empty()
                    && !pack.rating_overlays.heroic.trim().is_empty()
                    && !pack.rating_overlays.historical.trim().is_empty(),
                "{} overlays",
                pack.id
            );
            assert!(!pack.scenarios.is_empty(), "{} scenarios", pack.id);
            for scenario in &pack.scenarios {
                assert!(
                    !scenario.opening.trim().is_empty(),
                    "{} opening",
                    scenario.id
                );
                assert!(scenario.squad.len() >= 4, "{} squad", scenario.id);
                assert!(
                    !scenario.timeline_pressure.is_empty(),
                    "{} timeline",
                    scenario.id
                );
            }
            assert!(pack.lore.len() >= 3, "{} lore chunks", pack.id);
        }
        // Two-sided Civil War: the player can pick Union or Confederate.
        let civil_war = find_pack(&packs, "civil-war").unwrap();
        assert_eq!(civil_war.scenarios.len(), 2);
    }

    #[test]
    fn ww1_doughboy_scenario_seeds_a_full_squad() {
        let packs = builtin_packs().unwrap();
        let ww1 = find_pack(&packs, "ww1").unwrap();
        let scenario = find_scenario(&ww1, "doughboy-1918").unwrap();
        assert_eq!(scenario.start_date, "1918-03-15");
        assert!(scenario.squad.len() >= 4);
        assert!(scenario
            .squad
            .iter()
            .all(|member| !member.name.trim().is_empty() && !member.trait_label.trim().is_empty()));
        assert!(!scenario.timeline_pressure.is_empty());
        assert!(!scenario.opening.trim().is_empty());
    }

    #[test]
    fn unknown_pack_and_scenario_are_clear_errors() {
        let packs = builtin_packs().unwrap();
        assert!(find_pack(&packs, "ww9").is_err());
        let ww1 = find_pack(&packs, "ww1").unwrap();
        assert!(find_scenario(&ww1, "missing").is_err());
    }
}
