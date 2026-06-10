#[cfg(test)]
mod tests {
    use crate::agent_tools::{execute_tool, parse_tool_call, ToolCall};
    use std::fs;
    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn parses_bare_and_fenced_tool_calls() {
        assert_eq!(
            parse_tool_call(r#"{"tool":"read_file","path":"src/main.rs"}"#),
            Some(ToolCall::ReadFile {
                path: "src/main.rs".to_string(),
                start_line: None,
                end_line: None
            }),
        );
        assert_eq!(
            parse_tool_call("```json\n{\"tool\":\"grep\",\"query\":\"fn main\"}\n```"),
            Some(ToolCall::Grep {
                query: "fn main".to_string()
            }),
        );
        assert_eq!(
            parse_tool_call(r#"{"tool":"list_dir"}"#),
            Some(ToolCall::ListDir { path: None }),
        );
    }

    #[test]
    fn normal_answers_are_not_tool_calls() {
        assert_eq!(parse_tool_call("Here is the fix: use a checked add."), None);
        assert_eq!(parse_tool_call("{ not json at all"), None);
        assert_eq!(parse_tool_call(r#"{"tool":"unknown_tool"}"#), None);
    }

    #[test]
    fn read_file_returns_numbered_lines_inside_root() {
        let root = workspace("tools-read");
        fs::write(root.join("a.txt"), "alpha\nbeta\ngamma\n").unwrap();
        let result = execute_tool(
            &root,
            &ToolCall::ReadFile {
                path: "a.txt".to_string(),
                start_line: Some(2),
                end_line: Some(3),
            },
        );
        assert!(result.contains("2: beta"));
        assert!(result.contains("3: gamma"));
        assert!(!result.contains("1: alpha"));
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn traversal_and_absolute_paths_are_refused() {
        let root = workspace("tools-scope");
        let escape = execute_tool(
            &root,
            &ToolCall::ReadFile {
                path: "../secret.txt".to_string(),
                start_line: None,
                end_line: None,
            },
        );
        assert!(escape.contains("Tool error"));
        let absolute = execute_tool(
            &root,
            &ToolCall::ReadFile {
                path: "C:/windows/win.ini".to_string(),
                start_line: None,
                end_line: None,
            },
        );
        assert!(absolute.contains("Tool error"));
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn grep_finds_matches_and_skips_heavy_dirs() {
        let root = workspace("tools-grep");
        fs::create_dir_all(root.join("src")).unwrap();
        fs::create_dir_all(root.join("node_modules")).unwrap();
        fs::write(
            root.join("src/lib.rs"),
            "fn main() {}\nlet needle_here = 1;\n",
        )
        .unwrap();
        fs::write(root.join("node_modules/x.js"), "needle_here\n").unwrap();
        let result = execute_tool(
            &root,
            &ToolCall::Grep {
                query: "needle_here".to_string(),
            },
        );
        assert!(result.contains("lib.rs:2"));
        assert!(!result.contains("node_modules"));
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn list_dir_marks_directories_and_skips_noise() {
        let root = workspace("tools-list");
        fs::create_dir_all(root.join("src")).unwrap();
        fs::create_dir_all(root.join(".git")).unwrap();
        fs::write(root.join("README.md"), "x").unwrap();
        let result = execute_tool(&root, &ToolCall::ListDir { path: None });
        assert!(result.contains("src/"));
        assert!(result.contains("README.md"));
        assert!(!result.contains(".git"));
        let _ = fs::remove_dir_all(root);
    }

    fn workspace(label: &str) -> PathBuf {
        let stamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let path = std::env::temp_dir().join(format!("delyx-next-{label}-{stamp}"));
        fs::create_dir_all(&path).unwrap();
        path
    }
}
