//! Constrained delta repair: when a Game Master scene arrives without a
//! parseable trailing ```delta block (the 30B forgot or mangled it), a second
//! non-streaming extraction pass re-reads the scene with the sampler locked to
//! the delta JSON schema (llguidance via mistral.rs), so the regenerated delta
//! is valid by construction. Best-effort: the narration is never held hostage
//! — repair failures leave the turn exactly as it was (QA/QC still backstops).

use crate::campaign_delta::{split_narration_and_delta, DeltaProposal};
use crate::model_embedded::EmbeddedRuntimeState;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CampaignDeltaRepairRequest {
    pub provider_id: String,
    pub model: String,
    pub raw_text: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CampaignDeltaRepairView {
    pub raw_text: String,
    pub repaired: bool,
}

/// JSON Schema mirroring `DeltaProposal`'s serde shape (camelCase), applied as
/// a decoding constraint so an extracted delta always parses. Keep in sync
/// with `DeltaProposal`.
pub fn delta_json_schema() -> serde_json::Value {
    serde_json::json!({
        "type": "object",
        "properties": {
            "events": { "type": "array", "items": { "type": "object", "properties": {
                "kind": { "type": "string" }, "summary": { "type": "string" }
            }, "required": ["kind", "summary"], "additionalProperties": false } },
            "characters": { "type": "array", "items": { "type": "object", "properties": {
                "name": { "type": "string" }, "status": { "type": "string" }, "notes": { "type": "string" }
            }, "required": ["name"], "additionalProperties": false } },
            "inventory": { "type": "object", "properties": {
                "add": { "type": "array", "items": { "type": "string" } },
                "remove": { "type": "array", "items": { "type": "string" } }
            }, "additionalProperties": false },
            "clock": { "type": "object", "properties": {
                "date": { "type": "string" }
            }, "additionalProperties": false },
            "location": { "type": "string" }
        },
        "additionalProperties": false
    })
}

/// The extraction contract for the schema-locked second pass. Pure so tests
/// can pin its shape without a model.
pub fn delta_extraction_messages(narration: &str) -> Vec<crate::model_chat::ModelChatMessage> {
    vec![
        crate::model_chat::ModelChatMessage {
            role: "system".to_string(),
            content: "You extract structured state changes from a Game Master scene. Reply with \
                      ONLY one JSON object — no code fence, no prose. Optional keys: events \
                      (list of {kind, summary}), characters (list of {name, status, notes}), \
                      inventory ({add, remove}), clock ({date}), location. Include only changes \
                      the scene clearly states. If nothing changed, reply {}."
                .to_string(),
        },
        crate::model_chat::ModelChatMessage {
            role: "user".to_string(),
            content: format!("SCENE:\n{narration}"),
        },
    ]
}

/// Append a repaired delta block to the raw scene text so the normal commit
/// path (`split_narration_and_delta` → validate → apply) handles it untouched.
pub fn append_delta_block(raw: &str, delta_json: &str) -> String {
    format!("{}\n\n```delta\n{}\n```", raw.trim_end(), delta_json.trim())
}

/// Repair a scene that is missing a parseable delta. No-op when the delta is
/// already valid, when the provider is not the embedded runtime (the sampler
/// is only ours to constrain locally), or when extraction fails.
#[tauri::command]
pub async fn campaign_delta_repair(
    runtime: tauri::State<'_, crate::runtime_bridge::RuntimeBridgeState>,
    embedded: tauri::State<'_, EmbeddedRuntimeState>,
    request: CampaignDeltaRepairRequest,
) -> Result<CampaignDeltaRepairView, String> {
    let unchanged = CampaignDeltaRepairView {
        raw_text: request.raw_text.clone(),
        repaired: false,
    };
    let (narration, delta) = split_narration_and_delta(&request.raw_text);
    if delta.is_some() || narration.trim().is_empty() || request.provider_id != "delyx-local" {
        return Ok(unchanged);
    }
    let database_path = runtime.database_path().to_path_buf();
    match extract_delta_constrained(&embedded, &database_path, &request.model, &narration).await {
        Some(delta_json) => Ok(CampaignDeltaRepairView {
            raw_text: append_delta_block(&request.raw_text, &delta_json),
            repaired: true,
        }),
        None => Ok(unchanged),
    }
}

#[cfg(feature = "embedded_mistral")]
async fn extract_delta_constrained(
    state: &EmbeddedRuntimeState,
    database_path: &std::path::Path,
    model: &str,
    narration: &str,
) -> Option<String> {
    use crate::model_embedded::{chat_request, load_or_get_model, validate_profile};
    use crate::model_embedded_persistence::load_profile_from_path;

    let profile = load_profile_from_path(database_path, model).ok()?;
    validate_profile(&profile).ok()?;
    let loaded = load_or_get_model(state, database_path, &profile)
        .await
        .ok()?;
    let request = chat_request(delta_extraction_messages(narration), &profile)
        .ok()?
        .set_constraint(mistralrs::Constraint::JsonSchema(delta_json_schema()));
    let response = loaded.model.send_chat_request(request).await.ok()?;
    let text = response
        .choices
        .first()
        .and_then(|choice| choice.message.content.as_ref())
        .map(|content| content.trim().to_string())
        .filter(|content| !content.is_empty())?;
    // The constraint should guarantee this; the parse check pins it anyway so a
    // drifting schema can never push an unparseable delta into the commit path.
    serde_json::from_str::<DeltaProposal>(&text).ok()?;
    Some(text)
}

#[cfg(not(feature = "embedded_mistral"))]
async fn extract_delta_constrained(
    _state: &EmbeddedRuntimeState,
    _database_path: &std::path::Path,
    _model: &str,
    _narration: &str,
) -> Option<String> {
    None
}
