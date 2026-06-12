#[cfg(test)]
mod tests {
    use crate::campaign::{CampaignStore, CharacterStatus};
    use crate::campaign_bridge::{create_campaign_record, CampaignCreateRequest};
    use crate::campaign_delta::{apply_delta, split_narration_and_delta};

    #[test]
    fn split_extracts_trailing_delta_block() {
        let raw = "The trench goes quiet.\n\nA flare hangs overhead.\n\n```delta\n{\"location\":\"Forward sap\"}\n```";
        let (narration, delta) = split_narration_and_delta(raw);
        assert_eq!(
            narration,
            "The trench goes quiet.\n\nA flare hangs overhead."
        );
        assert_eq!(delta.unwrap().location.as_deref(), Some("Forward sap"));
    }

    #[test]
    fn split_survives_missing_or_malformed_blocks() {
        let (narration, delta) = split_narration_and_delta("Just a scene, no block.");
        assert_eq!(narration, "Just a scene, no block.");
        assert!(delta.is_none());

        let (narration, delta) =
            split_narration_and_delta("Scene.\n```delta\nnot json at all\n```");
        assert_eq!(narration, "Scene.");
        assert!(delta.is_none());

        let (narration, delta) = split_narration_and_delta("Scene.\n```delta\n{}\n");
        assert_eq!(narration, "Scene.");
        assert!(delta.unwrap().events.is_empty());
    }

    #[test]
    fn apply_records_events_and_character_changes() {
        let (mut store, campaign_id) = seeded();
        let proposal = serde_json::from_str(
            r#"{"events":[{"kind":"wound","summary":"Joey grazed by shrapnel."}],
                "characters":[{"name":"sgt. calloway","status":"wounded","notes":"shrapnel, left arm"}],
                "inventory":{"add":["German pistol"]},
                "clock":{"date":"1918-03-21"},
                "location":"Forward trench, St. Mihiel"}"#,
        )
        .unwrap();

        let applied = apply_delta(&mut store, &campaign_id, 0, &proposal, "T1").unwrap();
        assert_eq!(applied.events.len(), 1);
        assert_eq!(applied.characters[0].name, "Sgt. Calloway");
        assert_eq!(applied.inventory_added, vec!["German pistol"]);
        assert_eq!(applied.world_date.as_deref(), Some("1918-03-21"));
        assert!(applied.rejected.is_empty());

        let campaign = store.campaign(&campaign_id).unwrap();
        assert_eq!(campaign.world_date, "1918-03-21");
        assert_eq!(campaign.location, "Forward trench, St. Mihiel");
        let calloway = store
            .find_character_by_name(&campaign_id, "Sgt. Calloway")
            .unwrap();
        assert_eq!(calloway.status, CharacterStatus::Wounded);
        assert!(calloway.notes.contains("shrapnel, left arm"));
        let player = store.player_character(&campaign_id).unwrap();
        assert!(player.inventory_json.contains("German pistol"));
        assert_eq!(store.events_for(&campaign_id).len(), 1);
    }

    #[test]
    fn apply_rejects_canon_violations_without_failing() {
        let (mut store, campaign_id) = seeded();
        let mills_id = store
            .find_character_by_name(&campaign_id, "Pvt. Eli Mills")
            .unwrap()
            .id
            .clone();
        store
            .set_character_status(&mills_id, CharacterStatus::Dead, "T2")
            .unwrap();

        let proposal = serde_json::from_str(
            r#"{"characters":[{"name":"Pvt. Eli Mills","status":"active"},
                              {"name":"Colonel Nobody","status":"wounded"}],
                "inventory":{"remove":["map of Berlin"]},
                "clock":{"date":"1918-01-01"}}"#,
        )
        .unwrap();

        let applied = apply_delta(&mut store, &campaign_id, 1, &proposal, "T3").unwrap();
        assert_eq!(applied.rejected.len(), 4);
        assert!(applied
            .rejected
            .iter()
            .any(|reason| reason.contains("dead and stays dead")));
        assert!(applied
            .rejected
            .iter()
            .any(|reason| reason.contains("unknown character: Colonel Nobody")));
        assert!(applied
            .rejected
            .iter()
            .any(|reason| reason.contains("does not have: map of Berlin")));
        assert!(applied
            .rejected
            .iter()
            .any(|reason| reason.contains("cannot move backwards")));

        // Canon untouched.
        assert_eq!(
            store
                .find_character_by_name(&campaign_id, "Pvt. Eli Mills")
                .unwrap()
                .status,
            CharacterStatus::Dead
        );
        assert_eq!(
            store.campaign(&campaign_id).unwrap().world_date,
            "1918-03-15"
        );
    }

    fn seeded() -> (CampaignStore, String) {
        let mut store = CampaignStore::new();
        let snapshot = create_campaign_record(
            &mut store,
            CampaignCreateRequest {
                project_id: "project-1".to_string(),
                era_pack_id: "ww1".to_string(),
                scenario_id: "doughboy-1918".to_string(),
                player_name: "Joey".to_string(),
                player_role: String::new(),
                content_rating: "story".to_string(),
                created_at: "T0".to_string(),
                title: None,
                player_trait: None,
            },
        )
        .unwrap();
        let campaign_id = snapshot.campaigns[0].id.clone();
        (store, campaign_id)
    }
}
