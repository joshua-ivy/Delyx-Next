//! Layered Game Master prompt assembly. The model narrates; the store owns the
//! truth. Every prompt re-states canon (roster, event ledger, world clock) so
//! the narrator can never drift from recorded facts.

use crate::campaign::{
    CampaignStatus, CampaignStore, CharacterKind, CharacterStatus, ContentRating,
};
use crate::campaign_dice::TurnResolution;
use crate::campaign_packs::{find_pack, find_scenario, EraPack, LoreChunk};
use serde::Serialize;

/// How many recent turns ride along verbatim. Older turns are represented by
/// the canon ledger and the rolling memory summary.
const TURN_WINDOW: usize = 10;

/// How many lore chunks ride along, picked by relevance to the current scene.
const LORE_WINDOW: usize = 2;

const GM_CONTRACT: &str =
    "You are the Game Master of a roleplaying campaign. You narrate the world \
and voice every named character except the player's. Never speak for the player, never decide \
their actions, and never skip ahead past a choice they have not made. Honor every fact in the \
CANON sections exactly: characters listed as dead stay dead, recorded events have happened and \
cannot be undone, and the world date and location are where the story stands right now. Keep \
each scene to 2-4 short paragraphs, grounded in what the player can see, hear, and do, and end \
every scene on a hook, a choice, or a question.";

const RESOLUTION_RULES: &str =
    "When the player's message ends with a RESOLUTION line, the dice have already been rolled \
and that outcome is binding. \"success\" means the attempt works cleanly. \"partial\" means it \
works at a real cost or only partway. \"setback\" means it fails and the situation gets worse. \
Narrate exactly that outcome with concrete consequences; never soften a setback into a win and \
never decide success or failure yourself when no RESOLUTION line is present for a risky act - \
instead narrate up to the brink of the attempt.";

const DELTA_INSTRUCTIONS: &str = "After the scene, end your reply with exactly one fenced code \
block tagged delta containing JSON. It is machine-read and hidden from the player. Record only \
what actually changed in this scene:\n\
```delta\n\
{\"events\":[{\"kind\":\"death|wound|promotion|item|bond|location|historical|note\",\"summary\":\"one settled fact, one line\"}],\n\
 \"characters\":[{\"name\":\"exact roster name\",\"status\":\"active|wounded|missing|dead|departed\",\"notes\":\"new persistent fact about them\"}],\n\
 \"inventory\":{\"add\":[\"item gained\"],\"remove\":[\"item lost\"]},\n\
 \"clock\":{\"date\":\"YYYY-MM-DD\"},\n\
 \"location\":\"new location if the player moved\"}\n\
```\n\
Omit any field that did not change. If nothing changed, the block contains {}. Never revive \
dead characters and never move the clock backwards.";

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GmPromptMessage {
    pub role: String,
    pub content: String,
}

pub fn build_turn_messages(
    store: &CampaignStore,
    packs: &[EraPack],
    campaign_id: &str,
    player_text: &str,
    resolution: Option<&TurnResolution>,
) -> Result<Vec<GmPromptMessage>, String> {
    if player_text.trim().is_empty() {
        return Err("A campaign turn requires player input.".to_string());
    }
    let campaign = store
        .campaign(campaign_id)
        .map_err(|error| format!("{error:?}"))?;
    if campaign.status != CampaignStatus::Active {
        return Err("This campaign is no longer active.".to_string());
    }
    let pack = find_pack(packs, &campaign.era_pack_id)?;

    let mut messages = vec![GmPromptMessage {
        role: "system".to_string(),
        content: system_prompt(store, &pack, campaign_id, player_text)?,
    }];

    let turns = store.turns_for(campaign_id);
    if turns.is_empty() {
        if let Some(scenario_id) = &campaign.scenario_id {
            let scenario = find_scenario(&pack, scenario_id)?;
            messages.push(GmPromptMessage {
                role: "assistant".to_string(),
                content: scenario.opening.clone(),
            });
        }
    }
    for turn in turns.iter().rev().take(TURN_WINDOW).rev() {
        messages.push(GmPromptMessage {
            role: "user".to_string(),
            content: turn.player_text.clone(),
        });
        messages.push(GmPromptMessage {
            role: "assistant".to_string(),
            content: turn.narration.clone(),
        });
    }

    let mut current = player_text.trim().to_string();
    if let Some(resolution) = resolution {
        let resolution_json = serde_json::to_string(resolution)
            .map_err(|error| format!("resolution serialization failed: {error}"))?;
        current.push_str(&format!("\n\nRESOLUTION: {resolution_json}"));
    }
    messages.push(GmPromptMessage {
        role: "user".to_string(),
        content: current,
    });
    Ok(messages)
}

fn system_prompt(
    store: &CampaignStore,
    pack: &EraPack,
    campaign_id: &str,
    player_text: &str,
) -> Result<String, String> {
    let campaign = store
        .campaign(campaign_id)
        .map_err(|error| format!("{error:?}"))?;

    let mut sections = vec![
        format!("[GM CONTRACT]\n{GM_CONTRACT}"),
        format!("[RESOLUTION RULES]\n{RESOLUTION_RULES}"),
        format!("[ERA VOICE]\n{}", pack.gm_style),
        format!(
            "[CONTENT RATING]\n{}",
            rating_overlay(pack, campaign.content_rating)
        ),
    ];

    let mut world = format!(
        "[CANON - WORLD]\nDate: {}\nLocation: {}",
        campaign.world_date, campaign.location
    );
    if let Some(scenario_id) = &campaign.scenario_id {
        let scenario = find_scenario(pack, scenario_id)?;
        if !scenario.timeline_pressure.is_empty() {
            world.push_str(
                "\nTimeline (these events arrive on schedule whether or not the player is ready):",
            );
            for entry in &scenario.timeline_pressure {
                world.push_str(&format!("\n- {entry}"));
            }
        }
    }
    sections.push(world);

    let mut roster = String::from("[CANON - CHARACTERS]");
    for character in store.characters_for(campaign_id) {
        roster.push_str(&format!(
            "\n- {} ({}, {}) - {}",
            character.name,
            character_kind_label(character.kind),
            character.role,
            character_status_label(character.status),
        ));
        if !character.notes.trim().is_empty() {
            roster.push_str(&format!(". {}", character.notes.trim()));
        }
        if character.kind == CharacterKind::Player {
            let items: Vec<String> =
                serde_json::from_str(&character.inventory_json).unwrap_or_default();
            if !items.is_empty() {
                roster.push_str(&format!(". Carrying: {}", items.join(", ")));
            }
        }
    }
    sections.push(roster);

    let events = store.events_for(campaign_id);
    let ledger = if events.is_empty() {
        String::from("[CANON - EVENT LEDGER]\nNo recorded events yet.")
    } else {
        let mut ledger = String::from(
            "[CANON - EVENT LEDGER]\nEverything below has already happened and is settled fact:",
        );
        for event in events {
            ledger.push_str(&format!(
                "\n- (turn {}) {}",
                event.turn_index, event.summary
            ));
        }
        ledger
    };
    sections.push(ledger);

    if !campaign.memory_summary.trim().is_empty() {
        sections.push(format!("[STORY SO FAR]\n{}", campaign.memory_summary));
    }

    let scene_context = format!("{} {}", campaign.location, player_text);
    for chunk in relevant_lore(&pack.lore, &scene_context, LORE_WINDOW) {
        sections.push(format!("[LORE - {}]\n{}", chunk.title, chunk.text));
    }

    sections.push(format!("[OUTPUT FORMAT]\n{DELTA_INSTRUCTIONS}"));

    Ok(sections.join("\n\n"))
}

/// Cheap deterministic relevance: score each lore chunk by how many distinct
/// scene words appear in it. Ties keep authoring order. No model, no index.
pub fn relevant_lore<'a>(
    lore: &'a [LoreChunk],
    scene_context: &str,
    limit: usize,
) -> Vec<&'a LoreChunk> {
    let words: Vec<String> = scene_context
        .to_lowercase()
        .split(|c: char| !c.is_alphanumeric())
        .filter(|word| word.len() >= 3)
        .map(|word| word.to_string())
        .collect();
    let mut scored: Vec<(usize, &LoreChunk)> = lore
        .iter()
        .map(|chunk| {
            let haystack = format!("{} {}", chunk.title, chunk.text).to_lowercase();
            let score = words
                .iter()
                .filter(|word| haystack.contains(word.as_str()))
                .count();
            (score, chunk)
        })
        .filter(|(score, _)| *score > 0)
        .collect();
    scored.sort_by(|a, b| b.0.cmp(&a.0));
    scored
        .into_iter()
        .take(limit)
        .map(|(_, chunk)| chunk)
        .collect()
}

pub fn rating_overlay(pack: &EraPack, rating: ContentRating) -> &str {
    match rating {
        ContentRating::Story => &pack.rating_overlays.story,
        ContentRating::Heroic => &pack.rating_overlays.heroic,
        ContentRating::Historical => &pack.rating_overlays.historical,
    }
}

fn character_kind_label(kind: CharacterKind) -> &'static str {
    match kind {
        CharacterKind::Player => "the player's character",
        CharacterKind::Npc => "NPC",
    }
}

fn character_status_label(status: CharacterStatus) -> &'static str {
    match status {
        CharacterStatus::Active => "active",
        CharacterStatus::Wounded => "wounded",
        CharacterStatus::Missing => "missing",
        CharacterStatus::Dead => "dead",
        CharacterStatus::Departed => "departed",
    }
}
