#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Campaign {
    pub id: String,
    pub project_id: String,
    pub era_pack_id: String,
    pub scenario_id: Option<String>,
    pub title: String,
    pub status: CampaignStatus,
    pub content_rating: ContentRating,
    pub world_date: String,
    pub location: String,
    pub memory_summary: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CampaignStatus {
    Active,
    Completed,
    Abandoned,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ContentRating {
    Story,
    Heroic,
    Historical,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CampaignCharacter {
    pub id: String,
    pub campaign_id: String,
    pub kind: CharacterKind,
    pub name: String,
    pub role: String,
    pub status: CharacterStatus,
    pub sheet_json: String,
    pub inventory_json: String,
    pub bonds_json: String,
    pub notes: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CharacterKind {
    Player,
    Npc,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CharacterStatus {
    Active,
    Wounded,
    Missing,
    Dead,
    Departed,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CampaignTurn {
    pub campaign_id: String,
    pub turn_index: usize,
    pub player_text: String,
    pub resolution_json: String,
    pub narration: String,
    pub state_delta_json: String,
    pub qaqc_status: TurnQaqcStatus,
    pub qaqc_notes: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TurnQaqcStatus {
    Pending,
    Clean,
    Corrected,
    Skipped,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CampaignEvent {
    pub id: String,
    pub campaign_id: String,
    pub turn_index: usize,
    pub kind: String,
    pub summary: String,
    pub created_at: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CampaignError {
    EmptyTitle,
    EmptyCharacterName,
    CampaignNotFound,
    CampaignNotActive,
    CharacterNotFound,
    EmptyEventSummary,
    TurnNotFound,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CampaignInput {
    pub project_id: String,
    pub era_pack_id: String,
    pub scenario_id: Option<String>,
    pub title: String,
    pub content_rating: ContentRating,
    pub world_date: String,
    pub location: String,
    pub created_at: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CharacterInput {
    pub kind: CharacterKind,
    pub name: String,
    pub role: String,
    pub sheet_json: String,
    pub notes: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TurnInput {
    pub player_text: String,
    pub resolution_json: String,
    pub narration: String,
    pub state_delta_json: String,
    pub created_at: String,
}
