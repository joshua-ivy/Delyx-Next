#[cfg(test)]
mod tests {
    use crate::cli_review::{build_review_prompt, parse_review_verdict};

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
}
