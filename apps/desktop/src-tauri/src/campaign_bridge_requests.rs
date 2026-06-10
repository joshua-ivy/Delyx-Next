use crate::campaign_dice::TurnResolution;
use serde::Deserialize;

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CampaignCreateRequest {
    pub project_id: String,
    pub era_pack_id: String,
    pub scenario_id: String,
    pub player_name: String,
    pub player_role: String,
    pub content_rating: String,
    pub created_at: String,
    #[serde(default)]
    pub title: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CampaignTurnPromptRequest {
    pub campaign_id: String,
    pub player_text: String,
    #[serde(default)]
    pub seed: Option<u64>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CampaignTurnCommitRequest {
    pub campaign_id: String,
    pub player_text: String,
    pub model_text: String,
    #[serde(default)]
    pub resolution: Option<TurnResolution>,
    pub created_at: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CampaignMemoryCommitRequest {
    pub campaign_id: String,
    pub summary: String,
    pub created_at: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CampaignQaqcPromptRequest {
    pub campaign_id: String,
    pub turn_index: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CampaignTurnQaqcCommitRequest {
    pub campaign_id: String,
    pub turn_index: usize,
    pub qaqc_status: String,
    #[serde(default)]
    pub qaqc_notes: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CampaignSetRatingRequest {
    pub campaign_id: String,
    pub content_rating: String,
    pub parent_confirmed: bool,
    pub updated_at: String,
}
