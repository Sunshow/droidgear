//! Helpers for running commands in a login-shell context.

use std::process::{Command, Output};

fn format_command_label(program: &str, args: &[&str]) -> String {
    let mut parts = Vec::with_capacity(args.len() + 1);
    parts.push(program.to_string());
    parts.extend(args.iter().map(|arg| (*arg).to_string()));
    parts.join(" ")
}

#[cfg(not(target_os = "windows"))]
fn quote_posix(value: &str) -> String {
    format!("'{}'", value.replace('\'', r"'\''"))
}

#[cfg(not(target_os = "windows"))]
fn build_login_shell_exec_command(program: &str, args: &[&str]) -> String {
    let mut parts = Vec::with_capacity(args.len() + 1);
    parts.push(quote_posix(program));
    parts.extend(args.iter().map(|arg| quote_posix(arg)));
    format!("exec {}", parts.join(" "))
}

pub fn run_command_in_login_shell(program: &str, args: &[&str]) -> Result<Output, String> {
    let label = format_command_label(program, args);

    #[cfg(target_os = "windows")]
    {
        Command::new(program)
            .args(args)
            .output()
            .map_err(|e| format!("Failed to execute {label}: {e}"))
    }

    #[cfg(not(target_os = "windows"))]
    {
        let shell = std::env::var("SHELL")
            .ok()
            .filter(|value| !value.trim().is_empty())
            .unwrap_or_else(|| "/bin/zsh".to_string());
        let shell_command = build_login_shell_exec_command(program, args);

        Command::new(&shell)
            .args(["-l", "-c", &shell_command])
            .output()
            .map_err(|e| format!("Failed to execute {label}: {e}"))
    }
}

#[cfg(test)]
mod tests {
    use super::format_command_label;

    #[test]
    fn format_command_label_joins_program_and_args() {
        assert_eq!(
            format_command_label("claude", &["--version"]),
            "claude --version"
        );
        assert_eq!(
            format_command_label("codex", &["--config", "model=gpt-5"]),
            "codex --config model=gpt-5"
        );
    }

    #[cfg(not(target_os = "windows"))]
    #[test]
    fn build_login_shell_exec_command_quotes_arguments() {
        assert_eq!(
            super::build_login_shell_exec_command("claude", &["O'Brien", "two words"]),
            "exec 'claude' 'O'\\''Brien' 'two words'"
        );
    }
}
