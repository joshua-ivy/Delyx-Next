#[cfg(test)]
mod tests {
    use crate::campaign_delta::{split_narration_and_delta, DeltaProposal};
    use crate::campaign_delta_repair::{
        append_delta_block, delta_extraction_messages, delta_json_schema,
    };

    #[test]
    fn schema_conforming_deltas_parse_into_delta_proposal() {
        // Every shape the constrained sampler is allowed to emit must parse —
        // the schema and the serde struct cannot drift apart.
        let full = r#"{
            "events": [{ "kind": "wound", "summary": "Sgt. Calloway hit by shrapnel" }],
            "characters": [{ "name": "Sgt. Calloway", "status": "wounded", "notes": "left arm" }],
            "inventory": { "add": ["German pistol"], "remove": ["last ration tin"] },
            "clock": { "date": "1918-03-22" },
            "location": "Shell crater, no-man's-land"
        }"#;
        let parsed: DeltaProposal = serde_json::from_str(full).expect("full delta parses");
        assert_eq!(parsed.events.len(), 1);
        assert_eq!(parsed.characters.len(), 1);
        assert_eq!(parsed.clock.unwrap().date.as_deref(), Some("1918-03-22"));

        let empty: DeltaProposal = serde_json::from_str("{}").expect("empty delta parses");
        assert!(empty.events.is_empty());
        assert!(empty.location.is_none());
    }

    #[test]
    fn schema_lists_exactly_the_delta_proposal_keys() {
        let schema = delta_json_schema();
        let properties = schema["properties"].as_object().expect("properties");
        let mut keys: Vec<&str> = properties.keys().map(String::as_str).collect();
        keys.sort_unstable();
        assert_eq!(
            keys,
            vec!["characters", "clock", "events", "inventory", "location"]
        );
        assert_eq!(schema["additionalProperties"], serde_json::json!(false));
    }

    #[test]
    fn appended_block_round_trips_through_the_commit_split() {
        let scene = "The whistle blows. You go over the top.";
        let delta = r#"{"events":[{"kind":"historical","summary":"Spring Offensive begins"}]}"#;
        let repaired = append_delta_block(scene, delta);
        let (narration, parsed) = split_narration_and_delta(&repaired);
        assert_eq!(narration, scene);
        let parsed = parsed.expect("appended delta must parse");
        assert_eq!(parsed.events.len(), 1);
        assert_eq!(parsed.events[0].kind, "historical");
    }

    #[test]
    fn extraction_messages_carry_the_scene_and_the_contract() {
        let messages = delta_extraction_messages("You crawl through the wire.");
        assert_eq!(messages.len(), 2);
        assert_eq!(messages[0].role, "system");
        assert!(messages[0].content.contains("ONLY one JSON object"));
        assert_eq!(messages[1].role, "user");
        assert!(messages[1].content.contains("You crawl through the wire."));
    }

    #[test]
    fn scenes_with_a_valid_delta_already_split_clean() {
        // The command's no-op guard relies on this: a parseable block means no
        // repair pass, so an already-good scene never pays a second model call.
        let raw = "Scene.\n```delta\n{\"location\":\"Forward trench\"}\n```";
        let (narration, delta) = split_narration_and_delta(raw);
        assert_eq!(narration, "Scene.");
        assert_eq!(
            delta.expect("delta parses").location.as_deref(),
            Some("Forward trench")
        );
    }
}
