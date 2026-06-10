use crate::campaign::{
    Campaign, CampaignCharacter, CampaignError, CampaignEvent, CampaignStatus, CampaignStore,
    CampaignTurn, CharacterKind, TurnQaqcStatus,
};

impl CampaignStore {
    pub fn find_character_by_name(
        &self,
        campaign_id: &str,
        name: &str,
    ) -> Option<&CampaignCharacter> {
        let needle = name.trim().to_lowercase();
        self.characters.iter().find(|character| {
            character.campaign_id == campaign_id && character.name.to_lowercase() == needle
        })
    }

    pub fn player_character(&self, campaign_id: &str) -> Option<&CampaignCharacter> {
        self.characters.iter().find(|character| {
            character.campaign_id == campaign_id && character.kind == CharacterKind::Player
        })
    }

    pub fn append_character_note(
        &mut self,
        character_id: &str,
        note: &str,
        updated_at: &str,
    ) -> Result<(), CampaignError> {
        let note = note.trim();
        if note.is_empty() {
            return Ok(());
        }
        let character = self.character_mut(character_id)?;
        if character.notes.trim().is_empty() {
            character.notes = note.to_string();
        } else {
            character.notes = format!("{}; {}", character.notes.trim(), note);
        }
        character.updated_at = updated_at.to_string();
        Ok(())
    }

    pub fn set_character_inventory(
        &mut self,
        character_id: &str,
        inventory_json: &str,
        updated_at: &str,
    ) -> Result<(), CampaignError> {
        let character = self.character_mut(character_id)?;
        character.inventory_json = inventory_json.to_string();
        character.updated_at = updated_at.to_string();
        Ok(())
    }

    pub fn set_turn_qaqc(
        &mut self,
        campaign_id: &str,
        turn_index: usize,
        status: TurnQaqcStatus,
        notes: Option<String>,
    ) -> Result<(), CampaignError> {
        let turn = self
            .turns
            .iter_mut()
            .find(|turn| turn.campaign_id == campaign_id && turn.turn_index == turn_index)
            .ok_or(CampaignError::TurnNotFound)?;
        turn.qaqc_status = status;
        turn.qaqc_notes = notes;
        Ok(())
    }

    pub fn set_campaign_status(
        &mut self,
        campaign_id: &str,
        status: CampaignStatus,
        updated_at: &str,
    ) -> Result<(), CampaignError> {
        let campaign = self.campaign_mut(campaign_id)?;
        campaign.status = status;
        campaign.updated_at = updated_at.to_string();
        Ok(())
    }

    pub fn campaign(&self, campaign_id: &str) -> Result<&Campaign, CampaignError> {
        self.campaigns
            .iter()
            .find(|campaign| campaign.id == campaign_id)
            .ok_or(CampaignError::CampaignNotFound)
    }

    pub fn campaigns_for_project(&self, project_id: &str) -> Vec<&Campaign> {
        self.campaigns
            .iter()
            .filter(|campaign| campaign.project_id == project_id)
            .collect()
    }

    pub fn characters_for(&self, campaign_id: &str) -> Vec<&CampaignCharacter> {
        self.characters
            .iter()
            .filter(|character| character.campaign_id == campaign_id)
            .collect()
    }

    pub fn turns_for(&self, campaign_id: &str) -> Vec<&CampaignTurn> {
        self.turns
            .iter()
            .filter(|turn| turn.campaign_id == campaign_id)
            .collect()
    }

    pub fn events_for(&self, campaign_id: &str) -> Vec<&CampaignEvent> {
        self.events
            .iter()
            .filter(|event| event.campaign_id == campaign_id)
            .collect()
    }

    pub(crate) fn all_campaigns(&self) -> &[Campaign] {
        &self.campaigns
    }

    pub(crate) fn all_characters(&self) -> &[CampaignCharacter] {
        &self.characters
    }

    pub(crate) fn all_turns(&self) -> &[CampaignTurn] {
        &self.turns
    }

    pub(crate) fn all_events(&self) -> &[CampaignEvent] {
        &self.events
    }

    pub(crate) fn from_loaded(
        campaigns: Vec<Campaign>,
        characters: Vec<CampaignCharacter>,
        turns: Vec<CampaignTurn>,
        events: Vec<CampaignEvent>,
    ) -> Self {
        let next_campaign_id =
            max_numeric_suffix(campaigns.iter().map(|c| c.id.as_str()), "campaign-");
        let next_character_id =
            max_numeric_suffix(characters.iter().map(|c| c.id.as_str()), "character-");
        let next_event_id =
            max_numeric_suffix(events.iter().map(|e| e.id.as_str()), "campaign-event-");
        Self {
            campaigns,
            characters,
            turns,
            events,
            next_campaign_id,
            next_character_id,
            next_event_id,
        }
    }
}

fn max_numeric_suffix<'a>(ids: impl Iterator<Item = &'a str>, prefix: &str) -> usize {
    ids.filter_map(|id| id.strip_prefix(prefix)?.parse::<usize>().ok())
        .max()
        .unwrap_or(0)
}
