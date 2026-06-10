//! State-delta extraction and application. The GM ends every scene with a
//! fenced ```delta JSON block proposing world changes; the app — not the
//! model — validates and applies them. Invalid entries are dropped with a
//! reason instead of failing the turn: the narration always survives.

use crate::campaign::{CampaignStore, CharacterStatus};
use crate::campaign_persistence::parse_character_status;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Default, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct DeltaProposal {
    pub events: Vec<DeltaEvent>,
    pub characters: Vec<DeltaCharacter>,
    pub inventory: Option<DeltaInventory>,
    pub clock: Option<DeltaClock>,
    pub location: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct DeltaEvent {
    pub kind: String,
    pub summary: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Default, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct DeltaCharacter {
    pub name: String,
    pub status: Option<String>,
    pub notes: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct DeltaInventory {
    pub add: Vec<String>,
    pub remove: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct DeltaClock {
    pub date: Option<String>,
}

/// What actually got applied, persisted as the turn's `state_delta_json`.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AppliedDelta {
    pub events: Vec<AppliedEvent>,
    pub characters: Vec<AppliedCharacter>,
    pub inventory_added: Vec<String>,
    pub inventory_removed: Vec<String>,
    pub world_date: Option<String>,
    pub location: Option<String>,
    pub rejected: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AppliedEvent {
    pub kind: String,
    pub summary: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AppliedCharacter {
    pub name: String,
    pub status: Option<String>,
    pub note: Option<String>,
}

impl AppliedDelta {
    pub fn is_empty(&self) -> bool {
        self.events.is_empty()
            && self.characters.is_empty()
            && self.inventory_added.is_empty()
            && self.inventory_removed.is_empty()
            && self.world_date.is_none()
            && self.location.is_none()
    }
}

/// Split the raw model output into the player-facing narration and the
/// trailing ```delta block, if present. Tolerates a missing or malformed
/// block — the narration is never held hostage by the delta.
pub fn split_narration_and_delta(raw: &str) -> (String, Option<DeltaProposal>) {
    let Some(fence_start) = raw.rfind("```delta") else {
        return (raw.trim().to_string(), None);
    };
    let after_fence = &raw[fence_start + "```delta".len()..];
    let body = match after_fence.find("```") {
        Some(end) => &after_fence[..end],
        None => after_fence,
    };
    let narration = raw[..fence_start].trim().to_string();
    match serde_json::from_str::<DeltaProposal>(body.trim()) {
        Ok(delta) => (narration, Some(delta)),
        Err(_) => (narration, None),
    }
}

/// Validate and apply a proposed delta. Each entry is applied independently;
/// entries that contradict canon are recorded in `rejected` and skipped.
pub fn apply_delta(
    store: &mut CampaignStore,
    campaign_id: &str,
    turn_index: usize,
    proposal: &DeltaProposal,
    created_at: &str,
) -> Result<AppliedDelta, String> {
    let mut applied = AppliedDelta::default();

    for event in &proposal.events {
        if event.summary.trim().is_empty() {
            applied
                .rejected
                .push("event with empty summary".to_string());
            continue;
        }
        let kind = if event.kind.trim().is_empty() {
            "note"
        } else {
            event.kind.trim()
        };
        store
            .record_event(campaign_id, turn_index, kind, &event.summary, created_at)
            .map_err(|error| format!("{error:?}"))?;
        applied.events.push(AppliedEvent {
            kind: kind.to_string(),
            summary: event.summary.trim().to_string(),
        });
    }

    for change in &proposal.characters {
        let Some(character) = store.find_character_by_name(campaign_id, &change.name) else {
            applied
                .rejected
                .push(format!("unknown character: {}", change.name));
            continue;
        };
        let character_id = character.id.clone();
        let character_name = character.name.clone();
        let currently_dead = character.status == CharacterStatus::Dead;

        let mut applied_status = None;
        if let Some(status_text) = &change.status {
            match parse_character_status(status_text.trim()) {
                Ok(status) => {
                    if currently_dead && status != CharacterStatus::Dead {
                        applied
                            .rejected
                            .push(format!("{character_name} is dead and stays dead"));
                    } else {
                        store
                            .set_character_status(&character_id, status, created_at)
                            .map_err(|error| format!("{error:?}"))?;
                        applied_status = Some(status_text.trim().to_string());
                    }
                }
                Err(_) => {
                    applied.rejected.push(format!(
                        "unknown status for {character_name}: {status_text}"
                    ));
                }
            }
        }
        let mut applied_note = None;
        if let Some(note) = &change.notes {
            if !note.trim().is_empty() {
                store
                    .append_character_note(&character_id, note, created_at)
                    .map_err(|error| format!("{error:?}"))?;
                applied_note = Some(note.trim().to_string());
            }
        }
        if applied_status.is_some() || applied_note.is_some() {
            applied.characters.push(AppliedCharacter {
                name: character_name,
                status: applied_status,
                note: applied_note,
            });
        }
    }

    if let Some(inventory) = &proposal.inventory {
        apply_inventory(store, campaign_id, inventory, created_at, &mut applied)?;
    }

    let mut new_date = None;
    if let Some(clock) = &proposal.clock {
        if let Some(date) = &clock.date {
            let campaign = store
                .campaign(campaign_id)
                .map_err(|error| format!("{error:?}"))?;
            // ISO dates compare correctly as strings; the clock never runs backwards.
            if date.trim() >= campaign.world_date.as_str() {
                new_date = Some(date.trim().to_string());
            } else {
                applied
                    .rejected
                    .push(format!("clock cannot move backwards to {date}"));
            }
        }
    }
    let new_location = proposal
        .location
        .as_ref()
        .map(|location| location.trim().to_string())
        .filter(|location| !location.is_empty());
    if new_date.is_some() || new_location.is_some() {
        store
            .set_world(
                campaign_id,
                new_date.as_deref(),
                new_location.as_deref(),
                created_at,
            )
            .map_err(|error| format!("{error:?}"))?;
        applied.world_date = new_date;
        applied.location = new_location;
    }

    Ok(applied)
}

fn apply_inventory(
    store: &mut CampaignStore,
    campaign_id: &str,
    inventory: &DeltaInventory,
    created_at: &str,
    applied: &mut AppliedDelta,
) -> Result<(), String> {
    let Some(player) = store.player_character(campaign_id) else {
        applied
            .rejected
            .push("inventory change with no player character".to_string());
        return Ok(());
    };
    let player_id = player.id.clone();
    let mut items: Vec<String> = serde_json::from_str(&player.inventory_json).unwrap_or_default();

    for item in &inventory.add {
        let item = item.trim();
        if item.is_empty() {
            continue;
        }
        if !items.iter().any(|existing| existing == item) {
            items.push(item.to_string());
            applied.inventory_added.push(item.to_string());
        }
    }
    for item in &inventory.remove {
        let item = item.trim();
        if let Some(position) = items.iter().position(|existing| existing == item) {
            items.remove(position);
            applied.inventory_removed.push(item.to_string());
        } else if !item.is_empty() {
            applied.rejected.push(format!(
                "cannot remove item the player does not have: {item}"
            ));
        }
    }

    if !applied.inventory_added.is_empty() || !applied.inventory_removed.is_empty() {
        let serialized = serde_json::to_string(&items)
            .map_err(|error| format!("inventory serialization failed: {error}"))?;
        store
            .set_character_inventory(&player_id, &serialized, created_at)
            .map_err(|error| format!("{error:?}"))?;
    }
    Ok(())
}
