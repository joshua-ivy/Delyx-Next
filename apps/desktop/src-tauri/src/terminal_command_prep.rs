use std::path::Path;

pub const POWERSHELL_UTF8_OUTPUT_PREFIX: &str =
    "try { [Console]::OutputEncoding=[System.Text.Encoding]::UTF8 } catch {}\n";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PreparedTerminalCommand {
    pub program: String,
    pub args: Vec<String>,
}

pub fn prepare_terminal_command(program: &str, args: &[String]) -> PreparedTerminalCommand {
    PreparedTerminalCommand {
        program: program.to_string(),
        args: prepare_powershell_args(program, args),
    }
}

fn prepare_powershell_args(program: &str, args: &[String]) -> Vec<String> {
    if !is_powershell_program(program) {
        return args.to_vec();
    }
    let Some(script_index) = powershell_script_index(args) else {
        return args.to_vec();
    };
    let mut prepared = args.to_vec();
    if !prepared[script_index]
        .trim_start()
        .starts_with(POWERSHELL_UTF8_OUTPUT_PREFIX)
    {
        prepared[script_index] = format!("{POWERSHELL_UTF8_OUTPUT_PREFIX}{}", prepared[script_index]);
    }
    prepared
}

fn powershell_script_index(args: &[String]) -> Option<usize> {
    args.iter()
        .position(|arg| arg.eq_ignore_ascii_case("-command") || arg.eq_ignore_ascii_case("-c"))
        .and_then(|index| (index + 1 < args.len()).then_some(index + 1))
}

fn is_powershell_program(program: &str) -> bool {
    let name = Path::new(program)
        .file_name()
        .and_then(|value| value.to_str())
        .unwrap_or(program)
        .to_ascii_lowercase();
    matches!(trim_executable_suffix(&name), "powershell" | "pwsh")
}

fn trim_executable_suffix(name: &str) -> &str {
    name.strip_suffix(".exe").unwrap_or(name)
}
