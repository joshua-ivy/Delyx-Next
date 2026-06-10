use crate::campaign::{
    Campaign, CampaignCharacter, CampaignEvent, CampaignStatus, CampaignStore, CampaignTurn,
    CharacterKind, CharacterStatus, ContentRating, TurnQaqcStatus,
};
use rusqlite::{params, Connection};
use std::path::Path;

pub fn save_to_path(store: &CampaignStore, path: &Path) -> Result<(), String> {
    let mut connection = crate::sqlite_store::open_migrated_database(path).map_err(sql_string)?;
    let transaction = connection.transaction().map_err(sql_string)?;
    clear_tables(&transaction)?;
    for campaign in store.all_campaigns() {
        insert_campaign(&transaction, campaign)?;
    }
    for character in store.all_characters() {
        insert_character(&transaction, character)?;
    }
    for turn in store.all_turns() {
        insert_turn(&transaction, turn)?;
    }
    for event in store.all_events() {
        insert_event(&transaction, event)?;
    }
    transaction.commit().map_err(sql_string)
}

pub fn load_from_path(path: &Path) -> Result<CampaignStore, String> {
    let connection = crate::sqlite_store::open_migrated_database(path).map_err(sql_string)?;
    let campaigns = load_campaigns(&connection)?;
    let characters = load_characters(&connection)?;
    let turns = load_turns(&connection)?;
    let events = load_events(&connection)?;
    Ok(CampaignStore::from_loaded(
        campaigns, characters, turns, events,
    ))
}

fn clear_tables(connection: &Connection) -> Result<(), String> {
    connection
        .execute_batch(
            "DELETE FROM campaign_events;
             DELETE FROM campaign_turns;
             DELETE FROM campaign_characters;
             DELETE FROM campaigns;",
        )
        .map_err(sql_string)
}

fn insert_campaign(connection: &Connection, campaign: &Campaign) -> Result<(), String> {
    connection
        .execute(
            "INSERT INTO campaigns
             (id, project_id, era_pack_id, scenario_id, title, status, content_rating,
              world_date, location, memory_summary, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)",
            params![
                campaign.id,
                campaign.project_id,
                campaign.era_pack_id,
                campaign.scenario_id,
                campaign.title,
                campaign_status_key(campaign.status),
                content_rating_key(campaign.content_rating),
                campaign.world_date,
                campaign.location,
                campaign.memory_summary,
                campaign.created_at,
                campaign.updated_at,
            ],
        )
        .map(|_| ())
        .map_err(sql_string)
}

fn insert_character(connection: &Connection, character: &CampaignCharacter) -> Result<(), String> {
    connection
        .execute(
            "INSERT INTO campaign_characters
             (id, campaign_id, kind, name, role, status, sheet_json, inventory_json,
              bonds_json, notes, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)",
            params![
                character.id,
                character.campaign_id,
                character_kind_key(character.kind),
                character.name,
                character.role,
                character_status_key(character.status),
                character.sheet_json,
                character.inventory_json,
                character.bonds_json,
                character.notes,
                character.created_at,
                character.updated_at,
            ],
        )
        .map(|_| ())
        .map_err(sql_string)
}

fn insert_turn(connection: &Connection, turn: &CampaignTurn) -> Result<(), String> {
    connection
        .execute(
            "INSERT INTO campaign_turns
             (campaign_id, turn_index, player_text, resolution_json, narration,
              state_delta_json, qaqc_status, qaqc_notes, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
            params![
                turn.campaign_id,
                turn.turn_index as i64,
                turn.player_text,
                turn.resolution_json,
                turn.narration,
                turn.state_delta_json,
                qaqc_status_key(turn.qaqc_status),
                turn.qaqc_notes,
                turn.created_at,
            ],
        )
        .map(|_| ())
        .map_err(sql_string)
}

fn insert_event(connection: &Connection, event: &CampaignEvent) -> Result<(), String> {
    connection
        .execute(
            "INSERT INTO campaign_events (id, campaign_id, turn_index, kind, summary, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![
                event.id,
                event.campaign_id,
                event.turn_index as i64,
                event.kind,
                event.summary,
                event.created_at,
            ],
        )
        .map(|_| ())
        .map_err(sql_string)
}

fn load_campaigns(connection: &Connection) -> Result<Vec<Campaign>, String> {
    let mut statement = connection
        .prepare(
            "SELECT id, project_id, era_pack_id, scenario_id, title, status, content_rating,
                    world_date, location, memory_summary, created_at, updated_at
             FROM campaigns ORDER BY rowid",
        )
        .map_err(sql_string)?;
    let mut rows = statement.query([]).map_err(sql_string)?;
    let mut campaigns = Vec::new();
    while let Some(row) = rows.next().map_err(sql_string)? {
        let status: String = row.get(5).map_err(sql_string)?;
        let rating: String = row.get(6).map_err(sql_string)?;
        campaigns.push(Campaign {
            id: row.get(0).map_err(sql_string)?,
            project_id: row.get(1).map_err(sql_string)?,
            era_pack_id: row.get(2).map_err(sql_string)?,
            scenario_id: row.get(3).map_err(sql_string)?,
            title: row.get(4).map_err(sql_string)?,
            status: parse_campaign_status(&status)?,
            content_rating: parse_content_rating(&rating)?,
            world_date: row.get(7).map_err(sql_string)?,
            location: row.get(8).map_err(sql_string)?,
            memory_summary: row.get(9).map_err(sql_string)?,
            created_at: row.get(10).map_err(sql_string)?,
            updated_at: row.get(11).map_err(sql_string)?,
        });
    }
    Ok(campaigns)
}

fn load_characters(connection: &Connection) -> Result<Vec<CampaignCharacter>, String> {
    let mut statement = connection
        .prepare(
            "SELECT id, campaign_id, kind, name, role, status, sheet_json, inventory_json,
                    bonds_json, notes, created_at, updated_at
             FROM campaign_characters ORDER BY rowid",
        )
        .map_err(sql_string)?;
    let mut rows = statement.query([]).map_err(sql_string)?;
    let mut characters = Vec::new();
    while let Some(row) = rows.next().map_err(sql_string)? {
        let kind: String = row.get(2).map_err(sql_string)?;
        let status: String = row.get(5).map_err(sql_string)?;
        characters.push(CampaignCharacter {
            id: row.get(0).map_err(sql_string)?,
            campaign_id: row.get(1).map_err(sql_string)?,
            kind: parse_character_kind(&kind)?,
            name: row.get(3).map_err(sql_string)?,
            role: row.get(4).map_err(sql_string)?,
            status: parse_character_status(&status)?,
            sheet_json: row.get(6).map_err(sql_string)?,
            inventory_json: row.get(7).map_err(sql_string)?,
            bonds_json: row.get(8).map_err(sql_string)?,
            notes: row.get(9).map_err(sql_string)?,
            created_at: row.get(10).map_err(sql_string)?,
            updated_at: row.get(11).map_err(sql_string)?,
        });
    }
    Ok(characters)
}

fn load_turns(connection: &Connection) -> Result<Vec<CampaignTurn>, String> {
    let mut statement = connection
        .prepare(
            "SELECT campaign_id, turn_index, player_text, resolution_json, narration,
                    state_delta_json, qaqc_status, qaqc_notes, created_at
             FROM campaign_turns ORDER BY campaign_id, turn_index",
        )
        .map_err(sql_string)?;
    let mut rows = statement.query([]).map_err(sql_string)?;
    let mut turns = Vec::new();
    while let Some(row) = rows.next().map_err(sql_string)? {
        let qaqc: String = row.get(6).map_err(sql_string)?;
        turns.push(CampaignTurn {
            campaign_id: row.get(0).map_err(sql_string)?,
            turn_index: row.get::<_, i64>(1).map_err(sql_string)? as usize,
            player_text: row.get(2).map_err(sql_string)?,
            resolution_json: row.get(3).map_err(sql_string)?,
            narration: row.get(4).map_err(sql_string)?,
            state_delta_json: row.get(5).map_err(sql_string)?,
            qaqc_status: parse_qaqc_status(&qaqc)?,
            qaqc_notes: row.get(7).map_err(sql_string)?,
            created_at: row.get(8).map_err(sql_string)?,
        });
    }
    Ok(turns)
}

fn load_events(connection: &Connection) -> Result<Vec<CampaignEvent>, String> {
    let mut statement = connection
        .prepare(
            "SELECT id, campaign_id, turn_index, kind, summary, created_at
             FROM campaign_events ORDER BY rowid",
        )
        .map_err(sql_string)?;
    let mut rows = statement.query([]).map_err(sql_string)?;
    let mut events = Vec::new();
    while let Some(row) = rows.next().map_err(sql_string)? {
        events.push(CampaignEvent {
            id: row.get(0).map_err(sql_string)?,
            campaign_id: row.get(1).map_err(sql_string)?,
            turn_index: row.get::<_, i64>(2).map_err(sql_string)? as usize,
            kind: row.get(3).map_err(sql_string)?,
            summary: row.get(4).map_err(sql_string)?,
            created_at: row.get(5).map_err(sql_string)?,
        });
    }
    Ok(events)
}

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

fn sql_string(error: rusqlite::Error) -> String {
    error.to_string()
}
