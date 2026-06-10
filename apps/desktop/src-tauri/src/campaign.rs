pub use crate::campaign_model::{
    Campaign, CampaignCharacter, CampaignError, CampaignEvent, CampaignInput, CampaignStatus,
    CampaignTurn, CharacterInput, CharacterKind, CharacterStatus, ContentRating, TurnInput,
    TurnQaqcStatus,
};

#[derive(Debug, Default)]
pub struct CampaignStore {
    pub(crate) campaigns: Vec<Campaign>,
    pub(crate) characters: Vec<CampaignCharacter>,
    pub(crate) turns: Vec<CampaignTurn>,
    pub(crate) events: Vec<CampaignEvent>,
    pub(crate) next_campaign_id: usize,
    pub(crate) next_character_id: usize,
    pub(crate) next_event_id: usize,
}

impl CampaignStore {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn create_campaign(
        &mut self,
        input: CampaignInput,
        characters: Vec<CharacterInput>,
    ) -> Result<Campaign, CampaignError> {
        if input.title.trim().is_empty() {
            return Err(CampaignError::EmptyTitle);
        }
        if characters
            .iter()
            .any(|character| character.name.trim().is_empty())
        {
            return Err(CampaignError::EmptyCharacterName);
        }

        self.next_campaign_id += 1;
        let campaign = Campaign {
            id: format!("campaign-{}", self.next_campaign_id),
            project_id: input.project_id,
            era_pack_id: input.era_pack_id,
            scenario_id: input.scenario_id,
            title: input.title.trim().to_string(),
            status: CampaignStatus::Active,
            content_rating: input.content_rating,
            world_date: input.world_date,
            location: input.location,
            memory_summary: String::new(),
            created_at: input.created_at.clone(),
            updated_at: input.created_at.clone(),
        };
        self.campaigns.push(campaign.clone());
        for character in characters {
            self.add_character(&campaign.id, character, &input.created_at)?;
        }
        Ok(campaign)
    }

    pub fn add_character(
        &mut self,
        campaign_id: &str,
        input: CharacterInput,
        created_at: &str,
    ) -> Result<CampaignCharacter, CampaignError> {
        if input.name.trim().is_empty() {
            return Err(CampaignError::EmptyCharacterName);
        }
        self.campaign(campaign_id)?;
        self.next_character_id += 1;
        let character = CampaignCharacter {
            id: format!("character-{}", self.next_character_id),
            campaign_id: campaign_id.to_string(),
            kind: input.kind,
            name: input.name.trim().to_string(),
            role: input.role,
            status: CharacterStatus::Active,
            sheet_json: input.sheet_json,
            inventory_json: "[]".to_string(),
            bonds_json: "[]".to_string(),
            notes: input.notes,
            created_at: created_at.to_string(),
            updated_at: created_at.to_string(),
        };
        self.characters.push(character.clone());
        Ok(character)
    }

    pub fn append_turn(
        &mut self,
        campaign_id: &str,
        input: TurnInput,
    ) -> Result<CampaignTurn, CampaignError> {
        let campaign = self.campaign(campaign_id)?;
        if campaign.status != CampaignStatus::Active {
            return Err(CampaignError::CampaignNotActive);
        }
        let turn_index = self.turns_for(campaign_id).len();
        let turn = CampaignTurn {
            campaign_id: campaign_id.to_string(),
            turn_index,
            player_text: input.player_text,
            resolution_json: input.resolution_json,
            narration: input.narration,
            state_delta_json: input.state_delta_json,
            qaqc_status: TurnQaqcStatus::Pending,
            qaqc_notes: None,
            created_at: input.created_at.clone(),
        };
        self.turns.push(turn.clone());
        self.touch_campaign(campaign_id, &input.created_at)?;
        Ok(turn)
    }

    pub fn record_event(
        &mut self,
        campaign_id: &str,
        turn_index: usize,
        kind: &str,
        summary: &str,
        created_at: &str,
    ) -> Result<CampaignEvent, CampaignError> {
        if summary.trim().is_empty() {
            return Err(CampaignError::EmptyEventSummary);
        }
        self.campaign(campaign_id)?;
        self.next_event_id += 1;
        let event = CampaignEvent {
            id: format!("campaign-event-{}", self.next_event_id),
            campaign_id: campaign_id.to_string(),
            turn_index,
            kind: kind.to_string(),
            summary: summary.trim().to_string(),
            created_at: created_at.to_string(),
        };
        self.events.push(event.clone());
        Ok(event)
    }

    pub fn set_character_status(
        &mut self,
        character_id: &str,
        status: CharacterStatus,
        updated_at: &str,
    ) -> Result<(), CampaignError> {
        let character = self
            .characters
            .iter_mut()
            .find(|character| character.id == character_id)
            .ok_or(CampaignError::CharacterNotFound)?;
        character.status = status;
        character.updated_at = updated_at.to_string();
        Ok(())
    }

    pub fn set_world(
        &mut self,
        campaign_id: &str,
        world_date: Option<&str>,
        location: Option<&str>,
        updated_at: &str,
    ) -> Result<(), CampaignError> {
        let campaign = self.campaign_mut(campaign_id)?;
        if let Some(date) = world_date {
            if !date.trim().is_empty() {
                campaign.world_date = date.trim().to_string();
            }
        }
        if let Some(location) = location {
            if !location.trim().is_empty() {
                campaign.location = location.trim().to_string();
            }
        }
        campaign.updated_at = updated_at.to_string();
        Ok(())
    }

    pub fn set_memory_summary(
        &mut self,
        campaign_id: &str,
        summary: &str,
        updated_at: &str,
    ) -> Result<(), CampaignError> {
        let campaign = self.campaign_mut(campaign_id)?;
        campaign.memory_summary = summary.trim().to_string();
        campaign.updated_at = updated_at.to_string();
        Ok(())
    }

    pub fn set_content_rating(
        &mut self,
        campaign_id: &str,
        rating: ContentRating,
        updated_at: &str,
    ) -> Result<(), CampaignError> {
        let campaign = self.campaign_mut(campaign_id)?;
        campaign.content_rating = rating;
        campaign.updated_at = updated_at.to_string();
        Ok(())
    }

    pub(crate) fn character_mut(
        &mut self,
        character_id: &str,
    ) -> Result<&mut CampaignCharacter, CampaignError> {
        self.characters
            .iter_mut()
            .find(|character| character.id == character_id)
            .ok_or(CampaignError::CharacterNotFound)
    }

    pub(crate) fn campaign_mut(
        &mut self,
        campaign_id: &str,
    ) -> Result<&mut Campaign, CampaignError> {
        self.campaigns
            .iter_mut()
            .find(|campaign| campaign.id == campaign_id)
            .ok_or(CampaignError::CampaignNotFound)
    }

    pub(crate) fn touch_campaign(
        &mut self,
        campaign_id: &str,
        updated_at: &str,
    ) -> Result<(), CampaignError> {
        let campaign = self.campaign_mut(campaign_id)?;
        campaign.updated_at = updated_at.to_string();
        Ok(())
    }
}
