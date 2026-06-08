#[cfg(test)]
mod tests {
    use crate::external_agent_stream_json::parse_claude_stream_json;

    #[test]
    fn parse_claude_stream_json_extracts_edits() {
        let stdout = concat!(
            r#"{"type":"system","subtype":"init","tools":["Read","Edit"]}"#,
            "\n",
            "this line is not json and must be ignored\n",
            r#"{"type":"assistant","message":{"content":[{"type":"text","text":" I will fix the bug. "},{"type":"tool_use","name":"Read","input":{"file_path":"src/lib.rs"}}]}}"#,
            "\n",
            r#"{"type":"assistant","message":{"content":[{"type":"tool_use","name":"Edit","input":{"file_path":"src/lib.rs","old_string":"a","new_string":"b"}}]}}"#,
            "\n",
            r#"{"type":"assistant","message":{"content":[{"type":"tool_use","name":"Write","input":{"file_path":"src/new.rs","content":"x"}}]}}"#,
            "\n",
            r#"{"type":"assistant","message":{"content":[{"type":"tool_use","name":"MultiEdit","input":{"file_path":"src/lib.rs","edits":[]}}]}}"#,
            "\n",
            r#"{"type":"result","subtype":"success","is_error":false,"result":"Done."}"#,
            "\n",
        );

        let summary = parse_claude_stream_json(stdout);

        assert_eq!(
            summary.assistant_texts,
            vec!["I will fix the bug.".to_string()]
        );
        // Read does not contribute an edited file; Edit/Write/MultiEdit do, deduped in order.
        assert_eq!(
            summary.edited_files,
            vec!["src/lib.rs".to_string(), "src/new.rs".to_string()]
        );
        assert_eq!(summary.tool_uses.len(), 4);
        assert_eq!(summary.tool_uses[0].name, "Read");
        assert_eq!(summary.tool_uses[1].name, "Edit");
        assert_eq!(
            summary.tool_uses[1].file_path,
            Some("src/lib.rs".to_string())
        );
        assert_eq!(summary.result_text, Some("Done.".to_string()));
        assert!(summary.saw_result);
        assert!(!summary.is_error);
    }

    #[test]
    fn parse_claude_stream_json_flags_error_result() {
        let stdout = concat!(
            r#"{"type":"assistant","message":{"content":[{"type":"text","text":"Working"}]}}"#,
            "\n",
            r#"{"type":"result","subtype":"error_max_turns","is_error":true,"result":"Reached max turns"}"#,
            "\n",
        );

        let summary = parse_claude_stream_json(stdout);

        assert!(summary.saw_result);
        assert!(summary.is_error);
        assert_eq!(summary.result_text, Some("Reached max turns".to_string()));
    }

    #[test]
    fn parse_claude_stream_json_handles_empty_and_blank_lines() {
        let summary = parse_claude_stream_json("\n   \n");

        assert!(summary.assistant_texts.is_empty());
        assert!(summary.edited_files.is_empty());
        assert!(!summary.saw_result);
        assert!(!summary.is_error);
    }
}
