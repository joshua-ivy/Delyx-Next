#[cfg(test)]
mod tests {
    use crate::cli_review::{
        build_review_prompt, cli_review_command, parse_review_verdict, split_review_text,
    };

    #[test]
    fn prompt_includes_task_content_and_verdict_instruction() {
        let prompt = build_review_prompt("Add retry logic", "fn retry() {}");
        assert!(prompt.contains("Add retry logic"));
        assert!(prompt.contains("fn retry() {}"));
        assert!(prompt.contains("VERDICT: PASS"));
        assert!(prompt.contains("VERDICT: FAIL"));
    }

    #[test]
    fn prompt_handles_empty_task() {
        let prompt = build_review_prompt("   ", "code");
        assert!(prompt.contains("no explicit task"));
    }

    #[test]
    fn verdict_reads_the_explicit_line() {
        assert_eq!(
            parse_review_verdict("VERDICT: FAIL\n- off-by-one in loop"),
            "fail"
        );
        assert_eq!(parse_review_verdict("VERDICT: PASS\nlooks correct"), "pass");
    }

    #[test]
    fn ambiguous_review_is_unclear_not_a_silent_pass() {
        assert_eq!(
            parse_review_verdict("I think this is mostly fine"),
            "unclear"
        );
        assert_eq!(parse_review_verdict(""), "unclear");
    }

    #[test]
    fn lone_keyword_is_respected() {
        assert_eq!(
            parse_review_verdict("This will FAIL on empty input"),
            "fail"
        );
    }

    #[test]
    fn claude_review_uses_cheap_readonly_flags_by_default() {
        let (program, args) = cli_review_command("claude-code", None, "review this").unwrap();
        assert_eq!(program, "claude");
        // Cheap model + safe-mode (keeps OAuth, skips CLAUDE.md/MCP) + no tools.
        assert_eq!(args[0], "-p");
        assert_eq!(args[1], "review this");
        let model_pos = args.iter().position(|a| a == "--model").unwrap();
        assert_eq!(args[model_pos + 1], "haiku");
        assert!(args.iter().any(|a| a == "--safe-mode"));
        // `--tools ""` must be last so the variadic flag cannot swallow the prompt.
        assert_eq!(args[args.len() - 2], "--tools");
        assert_eq!(args[args.len() - 1], "");
    }

    #[test]
    fn claude_review_honors_model_override() {
        let (_, args) = cli_review_command("claude-code", Some("sonnet"), "x").unwrap();
        let model_pos = args.iter().position(|a| a == "--model").unwrap();
        assert_eq!(args[model_pos + 1], "sonnet");
    }

    #[test]
    fn codex_review_is_read_only_with_cheap_default_model() {
        let (program, args) = cli_review_command("codex-cli", None, "review this").unwrap();
        assert_eq!(program, "codex");
        assert_eq!(args[0], "exec");
        assert!(args.windows(2).any(|w| w == ["--sandbox", "read-only"]));
        let model_pos = args.iter().position(|a| a == "-m").unwrap();
        assert_eq!(args[model_pos + 1], "gpt-5.4-mini");
        assert_eq!(args.last().unwrap(), "review this");
    }

    #[test]
    fn codex_review_passes_model_when_set() {
        let (_, args) = cli_review_command("codex-cli", Some("gpt-5-codex"), "x").unwrap();
        let model_pos = args.iter().position(|a| a == "-m").unwrap();
        assert_eq!(args[model_pos + 1], "gpt-5-codex");
    }

    #[test]
    fn unsupported_adapter_is_rejected() {
        assert!(cli_review_command("gemini", None, "x").is_err());
    }

    #[test]
    fn split_review_text_separates_findings_from_corrected_code() {
        let raw = "VERDICT: FAIL\n- off-by-one in the loop\n\n```python\ndef f(n):\n    return n + 1\n```";
        let (findings, fix) = split_review_text(raw);
        assert_eq!(findings, "VERDICT: FAIL\n- off-by-one in the loop");
        assert_eq!(fix.as_deref(), Some("def f(n):\n    return n + 1"));
    }

    #[test]
    fn split_review_text_returns_no_fix_for_a_clean_pass() {
        let raw = "VERDICT: PASS\n- looks correct";
        let (findings, fix) = split_review_text(raw);
        assert_eq!(findings, "VERDICT: PASS\n- looks correct");
        assert!(fix.is_none());
    }

    #[test]
    fn split_review_text_ignores_an_empty_code_fence() {
        let raw = "VERDICT: FAIL\n- no fix provided\n```\n```";
        let (_, fix) = split_review_text(raw);
        assert!(fix.is_none());
    }
}
