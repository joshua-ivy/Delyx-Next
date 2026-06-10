#[cfg(test)]
mod tests {
    use crate::campaign::{
        CampaignError, CampaignInput, CampaignStatus, CampaignStore, CharacterInput, CharacterKind,
        CharacterStatus, ContentRating, TurnInput, TurnQaqcStatus,
    };

    #[test]
    fn create_campaign_assigns_ids_and_characters() {
        let mut store = CampaignStore::new();
        let campaign = store
            .create_campaign(
                campaign_input("Over There"),
                vec![player("Joey"), npc("Sgt. Calloway")],
            )
            .unwrap();

        assert_eq!(campaign.id, "campaign-1");
        assert_eq!(campaign.status, CampaignStatus::Active);
        assert_eq!(campaign.world_date, "1918-03-15");

        let characters = store.characters_for(&campaign.id);
        assert_eq!(characters.len(), 2);
        assert_eq!(characters[0].id, "character-1");
        assert_eq!(characters[0].kind, CharacterKind::Player);
        assert_eq!(characters[1].kind, CharacterKind::Npc);
        assert!(characters
            .iter()
            .all(|character| character.status == CharacterStatus::Active));
    }

    #[test]
    fn create_campaign_rejects_blank_title_and_names() {
        let mut store = CampaignStore::new();
        assert_eq!(
            store.create_campaign(campaign_input("  "), vec![player("Joey")]),
            Err(CampaignError::EmptyTitle)
        );
        assert_eq!(
            store.create_campaign(campaign_input("Over There"), vec![player("  ")]),
            Err(CampaignError::EmptyCharacterName)
        );
    }

    #[test]
    fn append_turn_orders_indexes_and_touches_campaign() {
        let mut store = CampaignStore::new();
        let campaign = store
            .create_campaign(campaign_input("Over There"), vec![player("Joey")])
            .unwrap();

        let first = store
            .append_turn(&campaign.id, turn_input("Look around", "T1"))
            .unwrap();
        let second = store
            .append_turn(&campaign.id, turn_input("Talk to Mills", "T2"))
            .unwrap();

        assert_eq!(first.turn_index, 0);
        assert_eq!(second.turn_index, 1);
        assert_eq!(first.qaqc_status, TurnQaqcStatus::Pending);
        assert_eq!(store.campaign(&campaign.id).unwrap().updated_at, "T2");
    }

    #[test]
    fn append_turn_requires_active_campaign() {
        let mut store = CampaignStore::new();
        let campaign = store
            .create_campaign(campaign_input("Over There"), vec![player("Joey")])
            .unwrap();
        store
            .set_campaign_status(&campaign.id, CampaignStatus::Completed, "T9")
            .unwrap();

        assert_eq!(
            store.append_turn(&campaign.id, turn_input("One more", "T10")),
            Err(CampaignError::CampaignNotActive)
        );
        assert_eq!(
            store.append_turn("campaign-404", turn_input("Ghost", "T10")),
            Err(CampaignError::CampaignNotFound)
        );
    }

    #[test]
    fn record_event_builds_canon_ledger() {
        let mut store = CampaignStore::new();
        let campaign = store
            .create_campaign(campaign_input("Over There"), vec![player("Joey")])
            .unwrap();

        let event = store
            .record_event(&campaign.id, 0, "wound", "Joey grazed by shrapnel.", "T1")
            .unwrap();
        assert_eq!(event.id, "campaign-event-1");
        assert_eq!(store.events_for(&campaign.id).len(), 1);
        assert_eq!(
            store.record_event(&campaign.id, 0, "wound", "   ", "T1"),
            Err(CampaignError::EmptyEventSummary)
        );
    }

    #[test]
    fn character_status_updates_are_persistent_facts() {
        let mut store = CampaignStore::new();
        let campaign = store
            .create_campaign(
                campaign_input("Over There"),
                vec![player("Joey"), npc("Pvt. Mills")],
            )
            .unwrap();
        let mills_id = store.characters_for(&campaign.id)[1].id.clone();

        store
            .set_character_status(&mills_id, CharacterStatus::Dead, "T5")
            .unwrap();
        assert_eq!(
            store.characters_for(&campaign.id)[1].status,
            CharacterStatus::Dead
        );
        assert_eq!(
            store.set_character_status("character-404", CharacterStatus::Dead, "T5"),
            Err(CampaignError::CharacterNotFound)
        );
    }

    #[test]
    fn from_loaded_continues_id_sequences() {
        let mut store = CampaignStore::new();
        let campaign = store
            .create_campaign(campaign_input("Over There"), vec![player("Joey")])
            .unwrap();
        store
            .record_event(&campaign.id, 0, "location", "Moved up to the line.", "T1")
            .unwrap();

        let mut reloaded = CampaignStore::from_loaded(
            store.all_campaigns().to_vec(),
            store.all_characters().to_vec(),
            store.all_turns().to_vec(),
            store.all_events().to_vec(),
        );
        let next = reloaded
            .create_campaign(campaign_input("Second Tour"), vec![player("Sam")])
            .unwrap();
        assert_eq!(next.id, "campaign-2");
        assert_eq!(reloaded.characters_for(&next.id)[0].id, "character-2");
        let event = reloaded
            .record_event(&next.id, 0, "location", "Back to Chaumont.", "T2")
            .unwrap();
        assert_eq!(event.id, "campaign-event-2");
    }

    fn campaign_input(title: &str) -> CampaignInput {
        CampaignInput {
            project_id: "project-1".to_string(),
            era_pack_id: "ww1".to_string(),
            scenario_id: Some("doughboy-1918".to_string()),
            title: title.to_string(),
            content_rating: ContentRating::Story,
            world_date: "1918-03-15".to_string(),
            location: "Chaumont, France".to_string(),
            created_at: "T0".to_string(),
        }
    }

    fn player(name: &str) -> CharacterInput {
        CharacterInput {
            kind: CharacterKind::Player,
            name: name.to_string(),
            role: "rifleman".to_string(),
            sheet_json: "{}".to_string(),
            notes: String::new(),
        }
    }

    fn npc(name: &str) -> CharacterInput {
        CharacterInput {
            kind: CharacterKind::Npc,
            name: name.to_string(),
            role: "squad leader".to_string(),
            sheet_json: "{}".to_string(),
            notes: "by-the-book".to_string(),
        }
    }

    fn turn_input(player_text: &str, created_at: &str) -> TurnInput {
        TurnInput {
            player_text: player_text.to_string(),
            resolution_json: "{}".to_string(),
            narration: "The trench waits.".to_string(),
            state_delta_json: "{}".to_string(),
            created_at: created_at.to_string(),
        }
    }
}
