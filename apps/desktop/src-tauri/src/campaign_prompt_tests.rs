#[cfg(test)]
mod tests {
    use crate::campaign::{CampaignStatus, CampaignStore, CharacterStatus, TurnInput};
    use crate::campaign_bridge::{create_campaign_record, CampaignCreateRequest};
    use crate::campaign_dice::resolve;
    use crate::campaign_packs::builtin_packs;
    use crate::campaign_prompt::{build_turn_messages, relevant_lore};

    #[test]
    fn first_turn_layers_canon_and_opens_with_the_scenario() {
        let (store, campaign_id) = seeded_store("story");
        let packs = builtin_packs().unwrap();
        let messages =
            build_turn_messages(&store, &packs, &campaign_id, "Look around the camp.", None)
                .unwrap();

        assert_eq!(messages.len(), 3);
        let system = &messages[0];
        assert_eq!(system.role, "system");
        assert!(system.content.starts_with("[GM CONTRACT]"));
        let contract_index = system.content.find("[GM CONTRACT]").unwrap();
        let resolution_index = system.content.find("[RESOLUTION RULES]").unwrap();
        let voice_index = system.content.find("[ERA VOICE]").unwrap();
        let rating_index = system.content.find("[CONTENT RATING]").unwrap();
        let world_index = system.content.find("[CANON - WORLD]").unwrap();
        let roster_index = system.content.find("[CANON - CHARACTERS]").unwrap();
        let ledger_index = system.content.find("[CANON - EVENT LEDGER]").unwrap();
        let output_index = system.content.find("[OUTPUT FORMAT]").unwrap();
        assert!(contract_index < resolution_index);
        assert!(resolution_index < voice_index);
        assert!(voice_index < rating_index);
        assert!(rating_index < world_index);
        assert!(world_index < roster_index);
        assert!(roster_index < ledger_index);
        assert!(ledger_index < output_index);

        assert!(system.content.contains("Date: 1918-03-15"));
        assert!(system.content.contains("Sgt. Calloway"));
        assert!(system.content.contains("adventure novel"));
        assert!(system.content.contains("No recorded events yet."));
        assert!(system.content.contains("German Spring Offensive"));
        assert!(system.content.contains("```delta"));

        assert_eq!(messages[1].role, "assistant");
        assert!(messages[1].content.contains("Sergeant Calloway"));
        assert_eq!(messages[2].role, "user");
        assert_eq!(messages[2].content, "Look around the camp.");
    }

    #[test]
    fn rating_overlay_matches_campaign_setting() {
        let (store, campaign_id) = seeded_store("historical");
        let packs = builtin_packs().unwrap();
        let messages =
            build_turn_messages(&store, &packs, &campaign_id, "Press on.", None).unwrap();
        assert!(messages[0].content.contains("Honest to the era"));
        assert!(!messages[0].content.contains("adventure novel"));
    }

    #[test]
    fn canon_reflects_deaths_and_recorded_events() {
        let (mut store, campaign_id) = seeded_store("story");
        let mills_id = store
            .characters_for(&campaign_id)
            .iter()
            .find(|character| character.name == "Pvt. Eli Mills")
            .unwrap()
            .id
            .clone();
        store
            .set_character_status(&mills_id, CharacterStatus::Dead, "T3")
            .unwrap();
        store
            .record_event(
                &campaign_id,
                2,
                "death",
                "Pvt. Eli Mills killed by a sniper.",
                "T3",
            )
            .unwrap();

        let packs = builtin_packs().unwrap();
        let messages =
            build_turn_messages(&store, &packs, &campaign_id, "Check on the squad.", None).unwrap();
        let system = &messages[0].content;
        assert!(system.contains("Pvt. Eli Mills (NPC, rifleman) - dead"));
        assert!(system.contains("(turn 2) Pvt. Eli Mills killed by a sniper."));
        assert!(!system.contains("No recorded events yet."));
    }

    #[test]
    fn resolution_line_rides_with_the_player_message() {
        let (store, campaign_id) = seeded_store("story");
        let packs = builtin_packs().unwrap();
        let resolution = resolve("aim", 1, 7);
        let messages = build_turn_messages(
            &store,
            &packs,
            &campaign_id,
            "Take the shot.",
            Some(&resolution),
        )
        .unwrap();
        let last = messages.last().unwrap();
        assert!(last.content.starts_with("Take the shot."));
        assert!(last.content.contains("RESOLUTION: {"));
        assert!(last
            .content
            .contains(&format!("\"outcome\":\"{}\"", resolution.outcome)));
    }

    #[test]
    fn relevant_lore_is_injected_for_matching_scenes() {
        let (store, campaign_id) = seeded_store("story");
        let packs = builtin_packs().unwrap();
        let messages = build_turn_messages(
            &store,
            &packs,
            &campaign_id,
            "I check my Chauchat and count the magazines before the attack.",
            None,
        )
        .unwrap();
        assert!(messages[0]
            .content
            .contains("[LORE - Weapons and Equipment]"));
    }

    #[test]
    fn relevant_lore_scores_by_word_overlap() {
        let packs = builtin_packs().unwrap();
        let lore = &packs.iter().find(|pack| pack.id == "ww1").unwrap().lore;
        assert!(lore.len() >= 4);
        let picked = relevant_lore(lore, "Mustard gas drifts toward our trench", 2);
        assert!(!picked.is_empty());
        assert_eq!(picked[0].title, "Gas");
        assert!(relevant_lore(lore, "zzz qqq", 2).is_empty());
    }

    #[test]
    fn turn_window_keeps_the_last_ten_turns_and_drops_the_opening() {
        let (mut store, campaign_id) = seeded_store("story");
        for index in 0..12 {
            store
                .append_turn(
                    &campaign_id,
                    TurnInput {
                        player_text: format!("Action {index}"),
                        resolution_json: "{}".to_string(),
                        narration: format!("Scene {index}"),
                        state_delta_json: "{}".to_string(),
                        created_at: format!("T{index}"),
                    },
                )
                .unwrap();
        }

        let packs = builtin_packs().unwrap();
        let messages =
            build_turn_messages(&store, &packs, &campaign_id, "Next move.", None).unwrap();
        // system + 10 turns (user+assistant each) + current user input
        assert_eq!(messages.len(), 1 + 10 * 2 + 1);
        assert_eq!(messages[1].content, "Action 2");
        assert!(!messages
            .iter()
            .any(|message| message.content == "Action 0" || message.content == "Action 1"));
        assert_eq!(messages.last().unwrap().content, "Next move.");
    }

    #[test]
    fn prompt_rejects_blank_input_and_inactive_campaigns() {
        let (mut store, campaign_id) = seeded_store("story");
        let packs = builtin_packs().unwrap();
        assert!(build_turn_messages(&store, &packs, &campaign_id, "   ", None).is_err());
        assert!(build_turn_messages(&store, &packs, "campaign-404", "Go.", None).is_err());
        store
            .set_campaign_status(&campaign_id, CampaignStatus::Completed, "T9")
            .unwrap();
        assert!(build_turn_messages(&store, &packs, &campaign_id, "Go.", None).is_err());
    }

    fn seeded_store(rating: &str) -> (CampaignStore, String) {
        let mut store = CampaignStore::new();
        let snapshot = create_campaign_record(
            &mut store,
            CampaignCreateRequest {
                project_id: "project-1".to_string(),
                era_pack_id: "ww1".to_string(),
                scenario_id: "doughboy-1918".to_string(),
                player_name: "Joey".to_string(),
                player_role: String::new(),
                content_rating: rating.to_string(),
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
