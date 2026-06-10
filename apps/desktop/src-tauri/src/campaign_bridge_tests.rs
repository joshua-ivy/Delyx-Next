#[cfg(test)]
mod tests {
    use crate::campaign::CampaignStore;
    use crate::campaign_bridge::{
        build_memory_prompt_record, build_qaqc_prompt_record, build_turn_prompt_record,
        campaign_snapshot_from_store, campaign_turn_views, commit_memory_record,
        commit_turn_qaqc_record, commit_turn_record, create_campaign_record, set_rating_record,
        CampaignCreateRequest, CampaignMemoryCommitRequest, CampaignQaqcPromptRequest,
        CampaignSetRatingRequest, CampaignTurnCommitRequest, CampaignTurnPromptRequest,
        CampaignTurnQaqcCommitRequest,
    };

    #[test]
    fn create_campaign_seeds_player_and_scenario_squad() {
        let mut store = CampaignStore::new();
        let snapshot = create_campaign_record(&mut store, request("Joey", "story")).unwrap();

        assert_eq!(snapshot.campaigns.len(), 1);
        let campaign = &snapshot.campaigns[0];
        assert_eq!(campaign.era_pack_id, "ww1");
        assert_eq!(campaign.scenario_id.as_deref(), Some("doughboy-1918"));
        assert_eq!(campaign.status, "active");
        assert_eq!(campaign.content_rating, "story");
        assert_eq!(campaign.world_date, "1918-03-15");
        assert_eq!(campaign.turn_count, 0);

        let player = &campaign.characters[0];
        assert_eq!(player.kind, "player");
        assert_eq!(player.name, "Joey");
        assert!(player.sheet_json.contains("\"grit\":0"));
        assert!(campaign.characters.len() > 1);
        assert!(campaign
            .characters
            .iter()
            .skip(1)
            .all(|character| character.kind == "npc"));
    }

    #[test]
    fn create_campaign_validates_inputs() {
        let mut store = CampaignStore::new();
        assert!(create_campaign_record(&mut store, request("  ", "story")).is_err());
        assert!(create_campaign_record(&mut store, request("Joey", "nightmare")).is_err());

        let mut bad_pack = request("Joey", "story");
        bad_pack.era_pack_id = "ww9".to_string();
        assert!(create_campaign_record(&mut store, bad_pack).is_err());

        let mut bad_scenario = request("Joey", "story");
        bad_scenario.scenario_id = "missing".to_string();
        assert!(create_campaign_record(&mut store, bad_scenario).is_err());
        assert!(campaign_snapshot_from_store(&store, "project-1")
            .campaigns
            .is_empty());
    }

    #[test]
    fn snapshot_filters_by_project() {
        let mut store = CampaignStore::new();
        create_campaign_record(&mut store, request("Joey", "story")).unwrap();
        let mut other = request("Sam", "heroic");
        other.project_id = "project-2".to_string();
        create_campaign_record(&mut store, other).unwrap();

        let first = campaign_snapshot_from_store(&store, "project-1");
        assert_eq!(first.campaigns.len(), 1);
        assert_eq!(first.campaigns[0].characters[0].name, "Joey");
        let second = campaign_snapshot_from_store(&store, "project-2");
        assert_eq!(second.campaigns.len(), 1);
        assert_eq!(second.campaigns[0].content_rating, "heroic");
    }

    #[test]
    fn default_title_comes_from_scenario() {
        let mut store = CampaignStore::new();
        let snapshot = create_campaign_record(&mut store, request("Joey", "story")).unwrap();
        assert_eq!(
            snapshot.campaigns[0].title,
            "Over There - AEF Rifleman, Spring 1918"
        );

        let mut titled = request("Sam", "story");
        titled.title = Some("Joey's First Campaign".to_string());
        let snapshot = create_campaign_record(&mut store, titled).unwrap();
        assert!(snapshot
            .campaigns
            .iter()
            .any(|campaign| campaign.title == "Joey's First Campaign"));
    }

    #[test]
    fn turn_prompt_rolls_dice_for_risky_actions_only() {
        let (mut store, campaign_id) = seeded();
        let calm = build_turn_prompt_record(
            &store,
            CampaignTurnPromptRequest {
                campaign_id: campaign_id.clone(),
                player_text: "I write a letter home.".to_string(),
                seed: Some(11),
            },
            11,
        )
        .unwrap();
        assert!(calm.resolution.is_none());

        let risky = build_turn_prompt_record(
            &store,
            CampaignTurnPromptRequest {
                campaign_id: campaign_id.clone(),
                player_text: "I sneak toward the German wire.".to_string(),
                seed: Some(11),
            },
            11,
        )
        .unwrap();
        let resolution = risky.resolution.unwrap();
        assert_eq!(resolution.check, "wits");
        assert!(risky
            .messages
            .last()
            .unwrap()
            .content
            .contains("RESOLUTION: {"));
        let _ = &mut store;
    }

    #[test]
    fn commit_turn_strips_delta_and_applies_it() {
        let (mut store, campaign_id) = seeded();
        let model_text = "The wire party freezes as a flare pops overhead.\n\n```delta\n{\"events\":[{\"kind\":\"wound\",\"summary\":\"Tommy Reyes caught wire barbs across the palm.\"}],\"characters\":[{\"name\":\"Pvt. Tommy Reyes\",\"status\":\"wounded\"}],\"location\":\"No-man's-land, near the listening post\"}\n```";

        let view = commit_turn_record(
            &mut store,
            CampaignTurnCommitRequest {
                campaign_id: campaign_id.clone(),
                player_text: "We push out to repair the wire.".to_string(),
                model_text: model_text.to_string(),
                resolution: None,
                created_at: "T1".to_string(),
            },
        )
        .unwrap();

        assert_eq!(view.turn.turn_index, 0);
        assert!(!view.turn.narration.contains("```delta"));
        assert!(view.turn.state_delta_json.contains("Tommy Reyes"));
        assert_eq!(
            view.campaign.location,
            "No-man's-land, near the listening post"
        );
        assert_eq!(view.campaign.events.len(), 1);
        assert!(
            view.campaign
                .characters
                .iter()
                .any(|character| character.name == "Pvt. Tommy Reyes"
                    && character.status == "wounded")
        );
    }

    #[test]
    fn commit_turn_survives_missing_delta_and_keeps_resolution() {
        let (mut store, campaign_id) = seeded();
        let resolution = crate::campaign_dice::resolve("grit", 0, 99);
        let view = commit_turn_record(
            &mut store,
            CampaignTurnCommitRequest {
                campaign_id: campaign_id.clone(),
                player_text: "I charge the crater.".to_string(),
                model_text: "You make it three steps before the mud takes a boot.".to_string(),
                resolution: Some(resolution.clone()),
                created_at: "T1".to_string(),
            },
        )
        .unwrap();
        assert_eq!(view.turn.state_delta_json, "{}");
        assert!(view.turn.resolution_json.contains(&resolution.outcome));
    }

    #[test]
    fn commit_turn_validates_inputs() {
        let (mut store, campaign_id) = seeded();
        assert!(commit_turn_record(&mut store, commit("  ", "Scene.", &campaign_id)).is_err());
        assert!(commit_turn_record(&mut store, commit("Go.", "   ", &campaign_id)).is_err());
        assert!(commit_turn_record(&mut store, commit("Go.", "Scene.", "campaign-404")).is_err());
        // A reply that is only a delta block has no narration to show.
        assert!(
            commit_turn_record(&mut store, commit("Go.", "```delta\n{}\n```", &campaign_id))
                .is_err()
        );
        assert!(campaign_turn_views(&store, &campaign_id).is_empty());
    }

    #[test]
    fn memory_prompt_and_commit_roll_the_chronicle() {
        let (mut store, campaign_id) = seeded();
        assert!(build_memory_prompt_record(&store, &campaign_id).is_err());

        commit_turn_record(
            &mut store,
            commit("Look around.", "Mud everywhere.", &campaign_id),
        )
        .unwrap();
        let messages = build_memory_prompt_record(&store, &campaign_id).unwrap();
        assert_eq!(messages.len(), 2);
        assert!(messages[0].content.contains("chronicle"));
        assert!(messages[1].content.contains("Mud everywhere."));

        let view = commit_memory_record(
            &mut store,
            CampaignMemoryCommitRequest {
                campaign_id: campaign_id.clone(),
                summary: "The squad reached the line and learned the mud.".to_string(),
                created_at: "T2".to_string(),
            },
        )
        .unwrap();
        assert!(view.memory_summary.contains("learned the mud"));
        assert!(commit_memory_record(
            &mut store,
            CampaignMemoryCommitRequest {
                campaign_id,
                summary: "   ".to_string(),
                created_at: "T3".to_string(),
            },
        )
        .is_err());
    }

    #[test]
    fn qaqc_prompt_carries_canon_and_commit_updates_the_turn() {
        let (mut store, campaign_id) = seeded();
        commit_turn_record(
            &mut store,
            commit(
                "Look around.",
                "An M1 Garand leans against the trench wall.",
                &campaign_id,
            ),
        )
        .unwrap();

        let prompt = build_qaqc_prompt_record(
            &store,
            &CampaignQaqcPromptRequest {
                campaign_id: campaign_id.clone(),
                turn_index: 0,
            },
        )
        .unwrap();
        assert!(prompt.contains("The Great War"));
        assert!(prompt.contains("M1 Garand leans"));
        assert!(prompt.contains("VERDICT: clean"));
        assert!(prompt.contains("Joey"));

        let turn = commit_turn_qaqc_record(
            &mut store,
            CampaignTurnQaqcCommitRequest {
                campaign_id: campaign_id.clone(),
                turn_index: 0,
                qaqc_status: "corrected".to_string(),
                qaqc_notes: Some("- The M1 Garand was not issued until 1936.".to_string()),
            },
        )
        .unwrap();
        assert_eq!(turn.qaqc_status, "corrected");
        assert!(turn.qaqc_notes.unwrap().contains("1936"));

        assert!(build_qaqc_prompt_record(
            &store,
            &CampaignQaqcPromptRequest {
                campaign_id,
                turn_index: 9,
            },
        )
        .is_err());
    }

    #[test]
    fn rating_change_requires_parent_confirmation() {
        let (mut store, campaign_id) = seeded();
        assert!(set_rating_record(
            &mut store,
            CampaignSetRatingRequest {
                campaign_id: campaign_id.clone(),
                content_rating: "historical".to_string(),
                parent_confirmed: false,
                updated_at: "T1".to_string(),
            },
        )
        .is_err());

        let view = set_rating_record(
            &mut store,
            CampaignSetRatingRequest {
                campaign_id,
                content_rating: "historical".to_string(),
                parent_confirmed: true,
                updated_at: "T1".to_string(),
            },
        )
        .unwrap();
        assert_eq!(view.content_rating, "historical");
    }

    fn commit(player_text: &str, model_text: &str, campaign_id: &str) -> CampaignTurnCommitRequest {
        CampaignTurnCommitRequest {
            campaign_id: campaign_id.to_string(),
            player_text: player_text.to_string(),
            model_text: model_text.to_string(),
            resolution: None,
            created_at: "T1".to_string(),
        }
    }

    fn seeded() -> (CampaignStore, String) {
        let mut store = CampaignStore::new();
        let snapshot = create_campaign_record(&mut store, request("Joey", "story")).unwrap();
        let campaign_id = snapshot.campaigns[0].id.clone();
        (store, campaign_id)
    }

    fn request(player_name: &str, rating: &str) -> CampaignCreateRequest {
        CampaignCreateRequest {
            project_id: "project-1".to_string(),
            era_pack_id: "ww1".to_string(),
            scenario_id: "doughboy-1918".to_string(),
            player_name: player_name.to_string(),
            player_role: String::new(),
            content_rating: rating.to_string(),
            created_at: "T0".to_string(),
            title: None,
        }
    }
}
