#[cfg(test)]
mod tests {
    use crate::command_exec::{
        cap_output, run_command_exec, CommandExecError, CommandExecEventKind, CommandExecRequest,
        CommandExecStatus,
    };
    use std::fs;
    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn command_exec_captures_status_output_duration_and_events() {
        let root = temp_workspace("command-exec-pass");

        let artifact = run_command_exec(request(&root, passing_command())).unwrap();

        assert_eq!(artifact.status, CommandExecStatus::Succeeded);
        assert_eq!(artifact.exit_code, Some(0));
        assert!(artifact.stdout.contains("command exec passed"));
        assert_eq!(artifact.run_id, "run-1");
        assert_eq!(artifact.approval_id, "approval-1");
        assert!(artifact.duration_ms <= 60_000);
        assert!(artifact.events.iter().any(|event| event.kind == CommandExecEventKind::Started));
        assert!(artifact.events.iter().any(|event| event.kind == CommandExecEventKind::Stdout));
        assert!(artifact.events.iter().any(|event| event.kind == CommandExecEventKind::Completed));
    }

    #[test]
    fn command_exec_reports_failed_exit_as_failed_artifact() {
        let root = temp_workspace("command-exec-fail");

        let artifact = run_command_exec(request(&root, failing_command())).unwrap();

        assert_eq!(artifact.status, CommandExecStatus::Failed);
        assert_ne!(artifact.exit_code, Some(0));
        assert!(artifact.stderr.contains("command exec failed"));
        assert!(artifact.events.iter().any(|event| event.kind == CommandExecEventKind::Failed));
    }

    #[test]
    fn command_exec_rejects_empty_command_and_zero_timeout() {
        let root = temp_workspace("command-exec-invalid");
        let mut empty = request(&root, ("".to_string(), Vec::new()));
        assert_eq!(run_command_exec(empty).unwrap_err(), CommandExecError::EmptyCommand);

        empty = request(&root, passing_command());
        empty.timeout_ms = 0;
        assert_eq!(run_command_exec(empty).unwrap_err(), CommandExecError::Timeout);
    }

    #[test]
    fn command_exec_caps_large_output_without_breaking_utf8() {
        let (text, truncated) = cap_output(format!("{}{}", "a".repeat(70_000), "é"));

        assert!(truncated);
        assert!(text.ends_with("...[truncated]"));
        assert!(text.is_char_boundary(text.len()));
    }

    fn request(root: &std::path::Path, command: (String, Vec<String>)) -> CommandExecRequest {
        CommandExecRequest {
            approval_id: "approval-1".to_string(),
            args: command.1,
            prepare_terminal: false,
            program: command.0,
            run_id: "run-1".to_string(),
            started_at_ms: 10,
            timeout_ms: 60_000,
            working_directory: root.to_path_buf(),
        }
    }

    fn passing_command() -> (String, Vec<String>) {
        if cfg!(windows) {
            ("cmd".to_string(), vec!["/C".to_string(), "echo command exec passed".to_string()])
        } else {
            ("sh".to_string(), vec!["-c".to_string(), "echo command exec passed".to_string()])
        }
    }

    fn failing_command() -> (String, Vec<String>) {
        if cfg!(windows) {
            ("cmd".to_string(), vec!["/C".to_string(), "echo command exec failed 1>&2 & exit /B 9".to_string()])
        } else {
            ("sh".to_string(), vec!["-c".to_string(), "echo command exec failed >&2; exit 9".to_string()])
        }
    }

    fn temp_workspace(label: &str) -> PathBuf {
        let stamp = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos();
        let path = std::env::temp_dir().join(format!("delyx-next-{label}-{stamp}"));
        fs::create_dir_all(&path).unwrap();
        path
    }
}

