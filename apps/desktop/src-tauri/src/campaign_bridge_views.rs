use crate::campaign::{Campaign, CampaignStore};
use crate::campaign_dice::TurnResolution;
use crate::campaign_packs::EraPack;
use crate::campaign_persistence::{
    campaign_status_key, character_kind_key, character_status_key, content_rating_key,
    qaqc_status_key,
};
use crate::campaign_prompt::GmPromptMessage;
use serde::Serialize;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EraPackView {
    pub id: String,
    pub title: String,
    pub gm_style: String,
    pub checks: Vec<String>,
    pub scenarios: Vec<EraScenarioView>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EraScenarioView {
    pub id: String,
    pub title: String,
    pub start_date: String,
    pub start_location: String,
    pub opening: String,
    pub squad_names: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CampaignView {
    pub id: String,
    pub project_id: String,
    pub era_pack_id: String,
    pub scenario_id: Option<String>,
    pub title: String,
    pub status: String,
    pub content_rating: String,
    pub world_date: String,
    pub location: String,
    pub memory_summary: String,
    pub created_at: String,
    pub updated_at: String,
    pub characters: Vec<CampaignCharacterView>,
    pub turn_count: usize,
    pub events: Vec<CampaignEventView>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CampaignCharacterView {
    pub id: String,
    pub kind: String,
    pub name: String,
    pub role: String,
    pub status: String,
    pub sheet_json: String,
    pub inventory_json: String,
    pub bonds_json: String,
    pub notes: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CampaignTurnView {
    pub turn_index: usize,
    pub player_text: String,
    pub resolution_json: String,
    pub narration: String,
    pub state_delta_json: String,
    pub qaqc_status: String,
    pub qaqc_notes: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CampaignEventView {
    pub id: String,
    pub turn_index: usize,
    pub kind: String,
    pub summary: String,
    pub created_at: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CampaignSnapshotView {
    pub campaigns: Vec<CampaignView>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CampaignTurnPromptView {
    pub messages: Vec<GmPromptMessage>,
    pub resolution: Option<TurnResolution>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CampaignTurnCommitView {
    pub turn: CampaignTurnView,
    pub campaign: CampaignView,
}

pub fn campaign_snapshot_from_store(
    store: &CampaignStore,
    project_id: &str,
) -> CampaignSnapshotView {
    CampaignSnapshotView {
        campaigns: store
            .campaigns_for_project(project_id)
            .into_iter()
            .map(|campaign| campaign_view(store, campaign))
            .collect(),
    }
}

pub fn campaign_turn_views(store: &CampaignStore, campaign_id: &str) -> Vec<CampaignTurnView> {
    store
        .turns_for(campaign_id)
        .into_iter()
        .map(turn_view)
        .collect()
}

pub(crate) fn turn_view(turn: &crate::campaign::CampaignTurn) -> CampaignTurnView {
    CampaignTurnView {
        turn_index: turn.turn_index,
        player_text: turn.player_text.clone(),
        resolution_json: turn.resolution_json.clone(),
        narration: turn.narration.clone(),
        state_delta_json: turn.state_delta_json.clone(),
        qaqc_status: qaqc_status_key(turn.qaqc_status).to_string(),
        qaqc_notes: turn.qaqc_notes.clone(),
        created_at: turn.created_at.clone(),
    }
}

pub(crate) fn campaign_view(store: &CampaignStore, campaign: &Campaign) -> CampaignView {
    CampaignView {
        id: campaign.id.clone(),
        project_id: campaign.project_id.clone(),
        era_pack_id: campaign.era_pack_id.clone(),
        scenario_id: campaign.scenario_id.clone(),
        title: campaign.title.clone(),
        status: campaign_status_key(campaign.status).to_string(),
        content_rating: content_rating_key(campaign.content_rating).to_string(),
        world_date: campaign.world_date.clone(),
        location: campaign.location.clone(),
        memory_summary: campaign.memory_summary.clone(),
        created_at: campaign.created_at.clone(),
        updated_at: campaign.updated_at.clone(),
        characters: store
            .characters_for(&campaign.id)
            .into_iter()
            .map(|character| CampaignCharacterView {
                id: character.id.clone(),
                kind: character_kind_key(character.kind).to_string(),
                name: character.name.clone(),
                role: character.role.clone(),
                status: character_status_key(character.status).to_string(),
                sheet_json: character.sheet_json.clone(),
                inventory_json: character.inventory_json.clone(),
                bonds_json: character.bonds_json.clone(),
                notes: character.notes.clone(),
            })
            .collect(),
        turn_count: store.turns_for(&campaign.id).len(),
        events: store
            .events_for(&campaign.id)
            .into_iter()
            .map(|event| CampaignEventView {
                id: event.id.clone(),
                turn_index: event.turn_index,
                kind: event.kind.clone(),
                summary: event.summary.clone(),
                created_at: event.created_at.clone(),
            })
            .collect(),
    }
}

pub(crate) fn pack_view(pack: &EraPack) -> EraPackView {
    EraPackView {
        id: pack.id.clone(),
        title: pack.title.clone(),
        gm_style: pack.gm_style.clone(),
        checks: pack.checks.clone(),
        scenarios: pack
            .scenarios
            .iter()
            .map(|scenario| EraScenarioView {
                id: scenario.id.clone(),
                title: scenario.title.clone(),
                start_date: scenario.start_date.clone(),
                start_location: scenario.start_location.clone(),
                opening: scenario.opening.clone(),
                squad_names: scenario
                    .squad
                    .iter()
                    .map(|member| member.name.clone())
                    .collect(),
            })
            .collect(),
    }
}
