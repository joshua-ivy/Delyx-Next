#[cfg(test)]
mod tests {
    use crate::campaign::{
        CampaignInput, CampaignStore, CharacterInput, CharacterKind, CharacterStatus,
        ContentRating, TurnInput, TurnQaqcStatus,
    };
    use crate::campaign_persistence::{load_from_path, save_to_path};
    use std::fs;
    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn campaign_store_survives_sqlite_reload() {
        let path = temp_path("campaign");
        let mut store = CampaignStore::new();
        let campaign = store
            .create_campaign(
                CampaignInput {
                    project_id: "project-1".to_string(),
                    era_pack_id: "ww1".to_string(),
                    scenario_id: Some("doughboy-1918".to_string()),
                    title: "Over There".to_string(),
                    content_rating: ContentRating::Story,
                    world_date: "1918-03-15".to_string(),
                    location: "Chaumont, France".to_string(),
                    created_at: "T0".to_string(),
                },
                vec![
                    CharacterInput {
                        kind: CharacterKind::Player,
                        name: "Joey".to_string(),
                        role: "rifleman".to_string(),
                        sheet_json: "{\"grit\":0}".to_string(),
                        notes: String::new(),
                    },
                    CharacterInput {
                        kind: CharacterKind::Npc,
                        name: "Pvt. Mills".to_string(),
                        role: "rifleman".to_string(),
                        sheet_json: "{}".to_string(),
                        notes: "writes everything down".to_string(),
                    },
                ],
            )
            .unwrap();
        store
            .append_turn(
                &campaign.id,
                TurnInput {
                    player_text: "Look around the trench.".to_string(),
                    resolution_json: "{}".to_string(),
                    narration: "Mud, wire, and a kettle that never boils.".to_string(),
                    state_delta_json: "{\"clock\":{\"advance\":\"1h\"}}".to_string(),
                    created_at: "T1".to_string(),
                },
            )
            .unwrap();
        store
            .record_event(
                &campaign.id,
                0,
                "location",
                "Relieved the French battalion.",
                "T1",
            )
            .unwrap();
        let mills_id = store.characters_for(&campaign.id)[1].id.clone();
        store
            .set_character_status(&mills_id, CharacterStatus::Wounded, "T1")
            .unwrap();

        save_to_path(&store, &path).unwrap();
        let bytes = fs::read(&path).unwrap();
        assert!(bytes.starts_with(b"SQLite format 3"));

        let mut loaded = load_from_path(&path).unwrap();
        let reloaded = loaded.campaign(&campaign.id).unwrap().clone();
        assert_eq!(reloaded.title, "Over There");
        assert_eq!(reloaded.content_rating, ContentRating::Story);
        assert_eq!(reloaded.updated_at, "T1");
        assert_eq!(reloaded.scenario_id.as_deref(), Some("doughboy-1918"));

        let characters = loaded.characters_for(&campaign.id);
        assert_eq!(characters.len(), 2);
        assert_eq!(characters[1].status, CharacterStatus::Wounded);
        assert_eq!(characters[0].sheet_json, "{\"grit\":0}");

        let turns = loaded.turns_for(&campaign.id);
        assert_eq!(turns.len(), 1);
        assert_eq!(turns[0].qaqc_status, TurnQaqcStatus::Pending);
        assert_eq!(
            turns[0].state_delta_json,
            "{\"clock\":{\"advance\":\"1h\"}}"
        );

        let events = loaded.events_for(&campaign.id);
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].summary, "Relieved the French battalion.");

        let next = loaded
            .create_campaign(
                CampaignInput {
                    project_id: "project-1".to_string(),
                    era_pack_id: "ww1".to_string(),
                    scenario_id: None,
                    title: "Second Tour".to_string(),
                    content_rating: ContentRating::Heroic,
                    world_date: "1918-06-01".to_string(),
                    location: "Belleau Wood".to_string(),
                    created_at: "T2".to_string(),
                },
                vec![],
            )
            .unwrap();
        assert_eq!(next.id, "campaign-2");
        let _ = fs::remove_file(path);
    }

    fn temp_path(name: &str) -> PathBuf {
        let stamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        std::env::temp_dir().join(format!("delyx-next-{name}-{stamp}.sqlite3"))
    }
}
