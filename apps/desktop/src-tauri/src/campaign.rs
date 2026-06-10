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

#[derive(Debug, Default)]
pub struct CampaignStore {
    campaigns: Vec<Campaign>,
    characters: Vec<CampaignCharacter>,
    turns: Vec<CampaignTurn>,
    events: Vec<CampaignEvent>,
    next_campaign_id: usize,
    next_character_id: usize,
    next_event_id: usize,
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

    fn character_mut(
        &mut self,
        character_id: &str,
    ) -> Result<&mut CampaignCharacter, CampaignError> {
        self.characters
            .iter_mut()
            .find(|character| character.id == character_id)
            .ok_or(CampaignError::CharacterNotFound)
    }

    fn campaign_mut(&mut self, campaign_id: &str) -> Result<&mut Campaign, CampaignError> {
        self.campaigns
            .iter_mut()
            .find(|campaign| campaign.id == campaign_id)
            .ok_or(CampaignError::CampaignNotFound)
    }

    fn touch_campaign(&mut self, campaign_id: &str, updated_at: &str) -> Result<(), CampaignError> {
        let campaign = self.campaign_mut(campaign_id)?;
        campaign.updated_at = updated_at.to_string();
        Ok(())
    }
}

fn max_numeric_suffix<'a>(ids: impl Iterator<Item = &'a str>, prefix: &str) -> usize {
    ids.filter_map(|id| id.strip_prefix(prefix)?.parse::<usize>().ok())
        .max()
        .unwrap_or(0)
}
