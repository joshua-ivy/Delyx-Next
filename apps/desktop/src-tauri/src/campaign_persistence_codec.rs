use crate::campaign::{
    CampaignStatus, CharacterKind, CharacterStatus, ContentRating, TurnQaqcStatus,
};

pub fn campaign_status_key(status: CampaignStatus) -> &'static str {
    match status {
        CampaignStatus::Active => "active",
        CampaignStatus::Completed => "completed",
        CampaignStatus::Abandoned => "abandoned",
    }
}

pub fn parse_campaign_status(value: &str) -> Result<CampaignStatus, String> {
    match value {
        "active" => Ok(CampaignStatus::Active),
        "completed" => Ok(CampaignStatus::Completed),
        "abandoned" => Ok(CampaignStatus::Abandoned),
        _ => Err("Unsupported persisted campaign status.".to_string()),
    }
}

pub fn content_rating_key(rating: ContentRating) -> &'static str {
    match rating {
        ContentRating::Story => "story",
        ContentRating::Heroic => "heroic",
        ContentRating::Historical => "historical",
    }
}

pub fn parse_content_rating(value: &str) -> Result<ContentRating, String> {
    match value {
        "story" => Ok(ContentRating::Story),
        "heroic" => Ok(ContentRating::Heroic),
        "historical" => Ok(ContentRating::Historical),
        _ => Err("Unsupported campaign content rating.".to_string()),
    }
}

pub fn character_kind_key(kind: CharacterKind) -> &'static str {
    match kind {
        CharacterKind::Player => "player",
        CharacterKind::Npc => "npc",
    }
}

pub fn parse_character_kind(value: &str) -> Result<CharacterKind, String> {
    match value {
        "player" => Ok(CharacterKind::Player),
        "npc" => Ok(CharacterKind::Npc),
        _ => Err("Unsupported persisted character kind.".to_string()),
    }
}

pub fn character_status_key(status: CharacterStatus) -> &'static str {
    match status {
        CharacterStatus::Active => "active",
        CharacterStatus::Wounded => "wounded",
        CharacterStatus::Missing => "missing",
        CharacterStatus::Dead => "dead",
        CharacterStatus::Departed => "departed",
    }
}

pub fn parse_character_status(value: &str) -> Result<CharacterStatus, String> {
    match value {
        "active" => Ok(CharacterStatus::Active),
        "wounded" => Ok(CharacterStatus::Wounded),
        "missing" => Ok(CharacterStatus::Missing),
        "dead" => Ok(CharacterStatus::Dead),
        "departed" => Ok(CharacterStatus::Departed),
        _ => Err("Unsupported persisted character status.".to_string()),
    }
}

pub fn qaqc_status_key(status: TurnQaqcStatus) -> &'static str {
    match status {
        TurnQaqcStatus::Pending => "pending",
        TurnQaqcStatus::Clean => "clean",
        TurnQaqcStatus::Corrected => "corrected",
        TurnQaqcStatus::Skipped => "skipped",
    }
}

pub fn parse_qaqc_status(value: &str) -> Result<TurnQaqcStatus, String> {
    match value {
        "pending" => Ok(TurnQaqcStatus::Pending),
        "clean" => Ok(TurnQaqcStatus::Clean),
        "corrected" => Ok(TurnQaqcStatus::Corrected),
        "skipped" => Ok(TurnQaqcStatus::Skipped),
        _ => Err("Unsupported persisted turn QA/QC status.".to_string()),
    }
}
