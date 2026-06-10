use crate::campaign::CampaignStore;
use crate::campaign_bridge_requests::{CampaignQaqcPromptRequest, CampaignTurnPromptRequest};
use crate::campaign_bridge_views::CampaignTurnPromptView;
use crate::campaign_dice::{detect_check, resolve, sheet_stat};
use crate::campaign_packs::find_pack;
use crate::campaign_packs_user::available_packs;
use crate::campaign_persistence::{character_status_key, content_rating_key};
use crate::campaign_prompt::{build_turn_messages, rating_overlay, GmPromptMessage};

pub fn build_turn_prompt_record(
    store: &CampaignStore,
    request: CampaignTurnPromptRequest,
    seed: u64,
) -> Result<CampaignTurnPromptView, String> {
    let packs = available_packs()?;
    let campaign = store
        .campaign(&request.campaign_id)
        .map_err(|error| format!("{error:?}"))?;
    let pack = find_pack(&packs, &campaign.era_pack_id)?;
    let resolution = detect_check(&request.player_text, &pack.checks).map(|check| {
        let stat = store
            .player_character(&request.campaign_id)
            .map(|player| sheet_stat(&player.sheet_json, &check))
            .unwrap_or(0);
        resolve(&check, stat, seed)
    });
    let messages = build_turn_messages(
        store,
        &packs,
        &request.campaign_id,
        &request.player_text,
        resolution.as_ref(),
    )?;
    Ok(CampaignTurnPromptView {
        messages,
        resolution,
    })
}

pub fn build_memory_prompt_record(
    store: &CampaignStore,
    campaign_id: &str,
) -> Result<Vec<GmPromptMessage>, String> {
    let campaign = store
        .campaign(campaign_id)
        .map_err(|error| format!("{error:?}"))?;
    let turns = store.turns_for(campaign_id);
    if turns.is_empty() {
        return Err("There is nothing to summarize yet.".to_string());
    }
    let mut chronicle = String::new();
    if !campaign.memory_summary.trim().is_empty() {
        chronicle.push_str(&format!(
            "Previous chronicle:\n{}\n\n",
            campaign.memory_summary
        ));
    }
    chronicle.push_str("Turns to fold in:\n");
    for turn in &turns {
        chronicle.push_str(&format!(
            "Turn {} - Player: {}\nGM: {}\n",
            turn.turn_index, turn.player_text, turn.narration
        ));
    }
    Ok(vec![
        GmPromptMessage {
            role: "system".to_string(),
            content: "You maintain the chronicle of a roleplaying campaign. Rewrite it as a \
single 150-250 word recap in past tense: the arc so far, promises and grudges, unresolved \
threads, and the emotional state of the squad. Keep every concrete fact (names, wounds, \
deaths, items, places). Output only the recap text."
                .to_string(),
        },
        GmPromptMessage {
            role: "user".to_string(),
            content: chronicle,
        },
    ])
}

pub fn build_qaqc_prompt_record(
    store: &CampaignStore,
    request: &CampaignQaqcPromptRequest,
) -> Result<String, String> {
    let campaign = store
        .campaign(&request.campaign_id)
        .map_err(|error| format!("{error:?}"))?;
    let turn = store
        .turns_for(&request.campaign_id)
        .into_iter()
        .find(|turn| turn.turn_index == request.turn_index)
        .cloned()
        .ok_or_else(|| "Turn not found for QA/QC review.".to_string())?;
    let packs = available_packs()?;
    let pack = find_pack(&packs, &campaign.era_pack_id)?;

    let mut prompt = format!(
        "You are the continuity and historical-accuracy reviewer for a \"{}\" roleplaying \
campaign. Review ONE Game Master scene against canon. Be strict about anachronisms (wrong-era \
weapons, technology, slang), contradictions of recorded facts, and content-rating violations. \
Ignore style.\n\nContent rating \"{}\": {}\n\nWorld date: {} | Location: {}\n\nRoster:\n",
        pack.title,
        content_rating_key(campaign.content_rating),
        rating_overlay(&pack, campaign.content_rating),
        campaign.world_date,
        campaign.location,
    );
    for character in store.characters_for(&request.campaign_id) {
        prompt.push_str(&format!(
            "- {} ({}) - {}\n",
            character.name,
            character.role,
            character_status_key(character.status)
        ));
    }
    prompt.push_str("\nSettled facts:\n");
    let events = store.events_for(&request.campaign_id);
    if events.is_empty() {
        prompt.push_str("- none recorded yet\n");
    }
    for event in events {
        prompt.push_str(&format!(
            "- (turn {}) {}\n",
            event.turn_index, event.summary
        ));
    }
    prompt.push_str(&format!(
        "\nScene to review (turn {}):\n{}\n\nReply with exactly one line \"VERDICT: clean\" if \
the scene honors canon, era, and rating, or \"VERDICT: issues\" followed by one bullet per \
problem, each a single line starting with \"- \".",
        turn.turn_index, turn.narration
    ));
    Ok(prompt)
}
