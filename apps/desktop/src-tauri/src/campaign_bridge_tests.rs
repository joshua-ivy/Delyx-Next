#[cfg(test)]
mod tests {
    use crate::campaign::CampaignStore;
    use crate::campaign_bridge::{
        campaign_snapshot_from_store, create_campaign_record, CampaignCreateRequest,
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
            player_trait: None,
        }
    }
}
