#[cfg(test)]
mod tests {
    use crate::terminal_command_prep::{prepare_terminal_command, POWERSHELL_UTF8_OUTPUT_PREFIX};

    #[test]
    fn powershell_command_is_prefixed_for_utf8_capture() {
        let prepared = prepare_terminal_command(
            "powershell.exe",
            &["-NoProfile".to_string(), "-Command".to_string(), "Write-Output cafe".to_string()],
        );

        assert_eq!(prepared.program, "powershell.exe");
        assert!(prepared.args[2].starts_with(POWERSHELL_UTF8_OUTPUT_PREFIX));
        assert!(prepared.args[2].ends_with("Write-Output cafe"));
    }

    #[test]
    fn powershell_utf8_prefix_is_not_duplicated() {
        let script = format!("{POWERSHELL_UTF8_OUTPUT_PREFIX}Write-Output cafe");

        let prepared = prepare_terminal_command("pwsh", &["-c".to_string(), script.clone()]);

        assert_eq!(prepared.args, vec!["-c".to_string(), script]);
    }

    #[test]
    fn non_powershell_command_is_left_alone() {
        let args = vec!["/C".to_string(), "echo ok".to_string()];

        let prepared = prepare_terminal_command("cmd.exe", &args);

        assert_eq!(prepared.program, "cmd.exe");
        assert_eq!(prepared.args, args);
    }
}
