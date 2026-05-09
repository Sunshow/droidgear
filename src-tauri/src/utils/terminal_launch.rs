use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

static LAUNCH_ARTIFACT_COUNTER: AtomicU64 = AtomicU64::new(0);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LaunchSpec {
    pub program: String,
    pub args: Vec<String>,
    pub env: Vec<(String, String)>,
    pub secret_env: Vec<(String, String)>,
    pub unset_env: Vec<String>,
    pub cwd: Option<PathBuf>,
    pub support_dir: Option<PathBuf>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PreparedCommand {
    pub command: String,
    pub keep_open_command: String,
}

pub fn launch_in_terminal(spec: &LaunchSpec, preferred: &str) -> Result<(), String> {
    #[cfg(target_os = "macos")]
    {
        launch_macos(spec, preferred)
    }
    #[cfg(target_os = "linux")]
    {
        launch_linux(spec, preferred)
    }
    #[cfg(target_os = "windows")]
    {
        launch_windows(spec, preferred)
    }
    #[cfg(not(any(target_os = "macos", target_os = "linux", target_os = "windows")))]
    {
        let _ = spec;
        let _ = preferred;
        Err("Unsupported platform".to_string())
    }
}

fn quote_posix(value: &str) -> String {
    format!("'{}'", value.replace('\'', r"'\''"))
}

#[cfg(target_os = "windows")]
fn quote_powershell(value: &str) -> String {
    format!("'{}'", value.replace('\'', "''"))
}

#[cfg(target_os = "windows")]
fn escape_cmd_value(value: &str) -> String {
    value
        .replace('^', "^^")
        .replace('"', "^\"")
        .replace('%', "%%")
}

#[cfg(target_os = "windows")]
fn quote_cmd(value: &str) -> String {
    format!("\"{}\"", escape_cmd_value(value))
}

#[cfg(any(target_os = "macos", target_os = "windows"))]
fn startup_message(program: &str) -> String {
    let display_name = Path::new(program)
        .file_name()
        .and_then(|name| name.to_str())
        .filter(|name| !name.is_empty())
        .unwrap_or(program);

    format!("Starting {display_name}...")
}

#[cfg(target_os = "macos")]
fn render_posix_banner(program: &str) -> String {
    format!("printf '%s\\n' {}", quote_posix(&startup_message(program)))
}

#[cfg(target_os = "windows")]
fn render_powershell_banner(program: &str) -> String {
    format!("Write-Host {}", quote_powershell(&startup_message(program)))
}

fn render_posix_env(env: &[(String, String)], unset_env: &[String]) -> String {
    let mut parts = Vec::new();

    for key in unset_env {
        parts.push(format!("unset {key}"));
    }

    for (key, value) in env {
        parts.push(format!("export {key}={}", quote_posix(value)));
    }

    parts.join("; ")
}

#[cfg(target_os = "windows")]
fn render_powershell_env(env: &[(String, String)], unset_env: &[String]) -> String {
    let mut parts = Vec::new();

    for key in unset_env {
        parts.push(format!(
            "Remove-Item Env:{key} -ErrorAction SilentlyContinue"
        ));
    }

    for (key, value) in env {
        parts.push(format!("$env:{key} = {}", quote_powershell(value)));
    }

    parts.join("; ")
}

#[cfg(target_os = "windows")]
fn render_cmd_env(env: &[(String, String)], unset_env: &[String]) -> String {
    let mut parts = Vec::new();

    for key in unset_env {
        parts.push(format!("set {key}="));
    }

    for (key, value) in env {
        parts.push(format!("set \"{key}={}\"", escape_cmd_value(value)));
    }

    parts.join(" && ")
}

fn render_program_command(spec: &LaunchSpec, quote_fn: fn(&str) -> String) -> String {
    let mut parts = Vec::with_capacity(spec.args.len() + 1);
    parts.push(quote_fn(&spec.program));
    parts.extend(spec.args.iter().map(|arg| quote_fn(arg)));
    parts.join(" ")
}

fn render_posix_cd(cwd: Option<&PathBuf>) -> Option<String> {
    cwd.map(|path| format!("cd {}", quote_posix(&path.to_string_lossy())))
}

#[cfg(target_os = "windows")]
fn render_powershell_cd(cwd: Option<&PathBuf>) -> Option<String> {
    cwd.map(|path| format!("Set-Location {}", quote_powershell(&path.to_string_lossy())))
}

#[cfg(target_os = "windows")]
fn render_cmd_cd(cwd: Option<&PathBuf>) -> Option<String> {
    cwd.map(|path| format!("cd /d {}", quote_cmd(&path.to_string_lossy())))
}

fn join_non_empty(parts: impl IntoIterator<Item = Option<String>>, separator: &str) -> String {
    parts
        .into_iter()
        .flatten()
        .filter(|part| !part.is_empty())
        .collect::<Vec<_>>()
        .join(separator)
}

fn needs_secure_wrapper(spec: &LaunchSpec) -> bool {
    !spec.secret_env.is_empty()
}

fn launch_artifact_path(spec: &LaunchSpec, filename: &str) -> PathBuf {
    if let Some(dir) = spec.support_dir.as_ref() {
        return dir.join(filename);
    }

    let timestamp = chrono::Utc::now().format("%Y%m%dT%H%M%S%.3fZ");
    let pid = std::process::id();
    let counter = LAUNCH_ARTIFACT_COUNTER.fetch_add(1, Ordering::Relaxed);
    std::env::temp_dir().join(format!("{timestamp}-{pid}-{counter}-{filename}"))
}

fn write_launch_script(path: &Path, contents: &str) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|e| format!("Failed to create launch script directory: {e}"))?;
    }

    std::fs::write(path, contents).map_err(|e| format!("Failed to write launch script: {e}"))?;

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;

        let mut perms = std::fs::metadata(path)
            .map_err(|e| format!("Failed to read launch script metadata: {e}"))?
            .permissions();
        perms.set_mode(0o700);
        std::fs::set_permissions(path, perms)
            .map_err(|e| format!("Failed to set launch script permissions: {e}"))?;
    }

    Ok(())
}

fn render_posix_child_command(spec: &LaunchSpec) -> String {
    let command = render_program_command(spec, quote_posix);
    let mut env_parts = Vec::new();

    for key in &spec.unset_env {
        env_parts.push(format!("-u {key}"));
    }

    for (key, value) in spec.env.iter().chain(spec.secret_env.iter()) {
        env_parts.push(format!("{key}={}", quote_posix(value)));
    }

    if env_parts.is_empty() {
        command
    } else {
        format!("env {} {command}", env_parts.join(" "))
    }
}

fn prepare_secure_posix_command(script_path: &Path) -> PreparedCommand {
    let command = format!("bash {}", quote_posix(&script_path.to_string_lossy()));

    PreparedCommand {
        command: command.clone(),
        keep_open_command: format!("{command}; exec bash"),
    }
}

#[cfg(any(target_os = "macos", target_os = "linux"))]
fn write_secure_posix_wrapper(spec: &LaunchSpec) -> Result<PathBuf, String> {
    let path = launch_artifact_path(spec, "terminal-launch.sh");
    let child_command = render_posix_child_command(spec);
    let cd = render_posix_cd(spec.cwd.as_ref());

    let script = join_non_empty(
        [
            Some("#!/bin/bash".to_string()),
            Some("rm -f -- \"$0\"".to_string()),
            cd,
            Some(child_command),
        ],
        "\n",
    );
    let script = format!("{script}\n");
    write_launch_script(&path, &script)?;
    Ok(path)
}

#[cfg(target_os = "windows")]
fn write_secure_cmd_wrapper(spec: &LaunchSpec) -> Result<PathBuf, String> {
    let path = launch_artifact_path(spec, "terminal-launch.cmd");
    let mut lines = vec![
        "@echo off".to_string(),
        "setlocal".to_string(),
        "cls".to_string(),
        format!("echo {}", startup_message(&spec.program)),
    ];

    for key in &spec.unset_env {
        lines.push(format!("set {key}="));
    }
    for (key, value) in spec.env.iter().chain(spec.secret_env.iter()) {
        lines.push(format!("set \"{key}={}\"", escape_cmd_value(value)));
    }
    if let Some(cd) = render_cmd_cd(spec.cwd.as_ref()) {
        lines.push(cd);
    }
    lines.push(render_program_command(spec, quote_cmd));
    lines.push("set \"DROIDGEAR_EXIT_CODE=%ERRORLEVEL%\"".to_string());
    lines.push("start \"\" /b cmd /c del /f /q \"%~f0\" >nul 2>nul".to_string());
    lines.push("endlocal & exit /b %DROIDGEAR_EXIT_CODE%".to_string());

    let script = format!("{}\r\n", lines.join("\r\n"));
    write_launch_script(&path, &script)?;
    Ok(path)
}

pub fn prepare_posix_command(spec: &LaunchSpec) -> PreparedCommand {
    let env_setup = render_posix_env(&spec.env, &spec.unset_env);
    let cd = render_posix_cd(spec.cwd.as_ref());
    let command = render_program_command(spec, quote_posix);

    let base = join_non_empty(
        [
            (!env_setup.is_empty()).then_some(env_setup),
            cd,
            Some(command),
        ],
        "; ",
    );

    PreparedCommand {
        command: base.clone(),
        keep_open_command: format!("{base}; exec bash"),
    }
}

#[cfg(target_os = "windows")]
pub fn prepare_powershell_command(spec: &LaunchSpec) -> PreparedCommand {
    let env_setup = render_powershell_env(&spec.env, &spec.unset_env);
    let cd = render_powershell_cd(spec.cwd.as_ref());
    let command = format!("& {}", render_program_command(spec, quote_powershell));
    let banner = render_powershell_banner(&spec.program);

    let base = join_non_empty(
        [
            (!env_setup.is_empty()).then_some(env_setup),
            cd,
            Some(command),
        ],
        "; ",
    );

    PreparedCommand {
        command: base.clone(),
        keep_open_command: join_non_empty([Some(banner), Some(base)], "; "),
    }
}

#[cfg(target_os = "windows")]
pub fn prepare_cmd_command(spec: &LaunchSpec) -> PreparedCommand {
    let env_setup = render_cmd_env(&spec.env, &spec.unset_env);
    let cd = render_cmd_cd(spec.cwd.as_ref());
    let command = render_program_command(spec, quote_cmd);

    let base = join_non_empty(
        [
            (!env_setup.is_empty()).then_some(env_setup),
            cd,
            Some(command),
        ],
        " && ",
    );

    PreparedCommand {
        command: base.clone(),
        keep_open_command: base,
    }
}

#[cfg(target_os = "macos")]
fn launch_macos(spec: &LaunchSpec, preferred: &str) -> Result<(), String> {
    if needs_secure_wrapper(spec) {
        let script_path = write_secure_posix_wrapper(spec)?;
        let prepared = prepare_secure_posix_command(&script_path);
        let banner = render_posix_banner(&spec.program);
        return match preferred {
            "iterm2" => launch_iterm2(&prepared.command, &prepared.keep_open_command, &banner),
            "terminal" => launch_terminal_app(&prepared.command, &banner),
            _ => launch_system_default_macos(spec, &prepared.command, &banner),
        };
    }

    let prepared = prepare_posix_command(spec);
    let banner = render_posix_banner(&spec.program);
    match preferred {
        "iterm2" => launch_iterm2(&prepared.command, &prepared.keep_open_command, &banner),
        "terminal" => launch_terminal_app(&prepared.command, &banner),
        _ => launch_system_default_macos(spec, &prepared.command, &banner),
    }
}

#[cfg(target_os = "macos")]
fn launch_iterm2(command: &str, keep_open_command: &str, banner: &str) -> Result<(), String> {
    let escaped = command.replace('\\', "\\\\").replace('"', "\\\"");
    let escaped_banner = banner.replace('\\', "\\\\").replace('"', "\\\"");
    let script = format!(
        r#"tell application "iTerm2"
    if (count of windows) = 0 then
        create window with default profile
    end if
    tell current window
        create tab with default profile
        tell current session
            write text "clear; {}; {}; exit"
        end tell
    end tell
    activate
end tell"#,
        escaped_banner, escaped
    );

    let status = std::process::Command::new("osascript")
        .arg("-e")
        .arg(&script)
        .status()
        .map_err(|e| format!("Failed to launch iTerm2: {e}"))?;

    if !status.success() {
        let status2 = std::process::Command::new("open")
            .args(["-a", "iTerm", "--args", "bash", "-c", keep_open_command])
            .status()
            .map_err(|e| format!("Failed to launch iTerm2: {e}"))?;
        if !status2.success() {
            return Err("Failed to launch iTerm2".to_string());
        }
    }
    Ok(())
}

#[cfg(target_os = "macos")]
fn launch_terminal_app(command: &str, banner: &str) -> Result<(), String> {
    let escaped = command.replace('\\', "\\\\").replace('"', "\\\"");
    let escaped_banner = banner.replace('\\', "\\\\").replace('"', "\\\"");
    let script = format!(
        r#"tell application "Terminal"
    activate
    do script "clear; {}; {}; exit"
end tell"#,
        escaped_banner, escaped
    );

    let status = std::process::Command::new("osascript")
        .arg("-e")
        .arg(&script)
        .status()
        .map_err(|e| format!("Failed to launch Terminal: {e}"))?;

    if !status.success() {
        return Err("Failed to open Terminal".to_string());
    }
    Ok(())
}

#[cfg(target_os = "macos")]
fn launch_system_default_macos(
    spec: &LaunchSpec,
    command: &str,
    banner: &str,
) -> Result<(), String> {
    let file_path = launch_artifact_path(spec, "terminal-launch.command");
    let script_content =
        format!("#!/bin/bash\nrm -f -- \"$0\"\nclear\n{banner}\n{command}\nexit\n");
    write_launch_script(&file_path, &script_content)?;

    let status = std::process::Command::new("open")
        .arg(&file_path)
        .status()
        .map_err(|e| format!("Failed to open terminal: {e}"))?;

    if !status.success() {
        return Err("Failed to open terminal".to_string());
    }
    Ok(())
}

#[cfg(target_os = "linux")]
fn launch_linux(spec: &LaunchSpec, preferred: &str) -> Result<(), String> {
    let prepared = if needs_secure_wrapper(spec) {
        let script_path = write_secure_posix_wrapper(spec)?;
        prepare_secure_posix_command(&script_path)
    } else {
        prepare_posix_command(spec)
    };
    let terminals: Vec<(&str, Vec<String>)> = match preferred {
        "gnome-terminal" => vec![(
            "gnome-terminal",
            vec![
                "--tab".to_string(),
                "--".to_string(),
                "bash".to_string(),
                "-c".to_string(),
                prepared.keep_open_command.clone(),
            ],
        )],
        "konsole" => vec![(
            "konsole",
            vec![
                "--new-tab".to_string(),
                "-e".to_string(),
                "bash".to_string(),
                "-c".to_string(),
                prepared.keep_open_command.clone(),
            ],
        )],
        "xfce4-terminal" => vec![(
            "xfce4-terminal",
            vec![
                "--tab".to_string(),
                "-e".to_string(),
                format!("bash -c {}", quote_posix(&prepared.keep_open_command)),
            ],
        )],
        "x-terminal-emulator" => vec![(
            "x-terminal-emulator",
            vec![
                "-e".to_string(),
                "bash".to_string(),
                "-c".to_string(),
                prepared.keep_open_command.clone(),
            ],
        )],
        _ => vec![
            (
                "gnome-terminal",
                vec![
                    "--tab".to_string(),
                    "--".to_string(),
                    "bash".to_string(),
                    "-c".to_string(),
                    prepared.keep_open_command.clone(),
                ],
            ),
            (
                "konsole",
                vec![
                    "--new-tab".to_string(),
                    "-e".to_string(),
                    "bash".to_string(),
                    "-c".to_string(),
                    prepared.keep_open_command.clone(),
                ],
            ),
            (
                "xfce4-terminal",
                vec![
                    "--tab".to_string(),
                    "-e".to_string(),
                    format!("bash -c {}", quote_posix(&prepared.keep_open_command)),
                ],
            ),
            (
                "x-terminal-emulator",
                vec![
                    "-e".to_string(),
                    "bash".to_string(),
                    "-c".to_string(),
                    prepared.keep_open_command.clone(),
                ],
            ),
            (
                "xterm",
                vec![
                    "-e".to_string(),
                    "bash".to_string(),
                    "-c".to_string(),
                    prepared.keep_open_command,
                ],
            ),
        ],
    };

    let mut last_err = String::new();
    for (term, args) in terminals {
        match std::process::Command::new(term).args(&args).spawn() {
            Ok(_) => return Ok(()),
            Err(e) => {
                last_err = format!("{term}: {e}");
            }
        }
    }

    Err(format!(
        "Could not find a terminal emulator. Tried: {last_err}"
    ))
}

#[cfg(target_os = "windows")]
fn launch_windows(spec: &LaunchSpec, preferred: &str) -> Result<(), String> {
    if needs_secure_wrapper(spec) {
        let wrapper_path = write_secure_cmd_wrapper(spec)?;
        let wrapper_cmd = wrapper_path.to_string_lossy().to_string();

        return match preferred {
            "cmd" => {
                std::process::Command::new("cmd")
                    .args(["/c", "start", "cmd", "/k", &wrapper_cmd])
                    .spawn()
                    .map_err(|e| format!("Failed to launch cmd: {e}"))?;
                Ok(())
            }
            "powershell" => {
                let command = format!("& {}", quote_powershell(&wrapper_cmd));
                std::process::Command::new("powershell")
                    .args(["-NoExit", "-Command", &command])
                    .spawn()
                    .map_err(|e| format!("Failed to launch PowerShell: {e}"))?;
                Ok(())
            }
            _ => {
                let wt_status = std::process::Command::new("wt")
                    .args(["-w", "0", "new-tab", "cmd", "/k", &wrapper_cmd])
                    .spawn();

                if wt_status.is_ok() {
                    return Ok(());
                }

                let wt_status2 = std::process::Command::new("wt")
                    .args(["cmd", "/k", &wrapper_cmd])
                    .spawn();

                if wt_status2.is_ok() {
                    return Ok(());
                }

                std::process::Command::new("cmd")
                    .args(["/c", "start", "cmd", "/k", &wrapper_cmd])
                    .spawn()
                    .map_err(|e| format!("Failed to launch command prompt: {e}"))?;
                Ok(())
            }
        };
    }

    let prepared_cmd = prepare_cmd_command(spec);
    let prepared_ps = prepare_powershell_command(spec);

    match preferred {
        "cmd" => {
            std::process::Command::new("cmd")
                .args(["/c", "start", "cmd", "/k", &prepared_cmd.keep_open_command])
                .spawn()
                .map_err(|e| format!("Failed to launch cmd: {e}"))?;
            Ok(())
        }
        "powershell" => {
            std::process::Command::new("powershell")
                .args(["-NoExit", "-Command", &prepared_ps.keep_open_command])
                .spawn()
                .map_err(|e| format!("Failed to launch PowerShell: {e}"))?;
            Ok(())
        }
        _ => {
            let wt_status = std::process::Command::new("wt")
                .args([
                    "-w",
                    "0",
                    "new-tab",
                    "cmd",
                    "/k",
                    &prepared_cmd.keep_open_command,
                ])
                .spawn();

            if wt_status.is_ok() {
                return Ok(());
            }

            let wt_status2 = std::process::Command::new("wt")
                .args(["cmd", "/k", &prepared_cmd.keep_open_command])
                .spawn();

            if wt_status2.is_ok() {
                return Ok(());
            }

            std::process::Command::new("cmd")
                .args(["/c", "start", "cmd", "/k", &prepared_cmd.keep_open_command])
                .spawn()
                .map_err(|e| format!("Failed to launch command prompt: {e}"))?;
            Ok(())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{
        launch_artifact_path, prepare_posix_command, prepare_secure_posix_command,
        render_posix_child_command, LaunchSpec,
    };
    use std::path::PathBuf;

    fn sample_spec() -> LaunchSpec {
        LaunchSpec {
            program: "droid".to_string(),
            args: vec![
                "--settings".to_string(),
                "/tmp/demo settings.json".to_string(),
            ],
            env: vec![("FOO".to_string(), "bar baz".to_string())],
            secret_env: vec![],
            unset_env: vec!["ANTHROPIC_AUTH_TOKEN".to_string()],
            cwd: Some(PathBuf::from("/work tree")),
            support_dir: None,
        }
    }

    #[test]
    fn prepare_posix_command_renders_env_unset_and_cwd() {
        let prepared = prepare_posix_command(&sample_spec());

        assert!(
            prepared.command.contains("unset ANTHROPIC_AUTH_TOKEN"),
            "expected unset env in command: {}",
            prepared.command
        );
        assert!(
            prepared.command.contains("export FOO='bar baz'"),
            "expected env export in command: {}",
            prepared.command
        );
        assert!(
            prepared.command.contains("cd '/work tree'"),
            "expected cwd change in command: {}",
            prepared.command
        );
        assert!(
            prepared
                .command
                .contains("'droid' '--settings' '/tmp/demo settings.json'"),
            "expected quoted program and args in command: {}",
            prepared.command
        );
        assert!(
            prepared.keep_open_command.ends_with("; exec bash"),
            "expected keep-open command to exec bash: {}",
            prepared.keep_open_command
        );
    }

    #[test]
    #[cfg(target_os = "windows")]
    #[test]
    fn prepare_powershell_command_renders_env_unset_and_cwd() {
        use super::prepare_powershell_command;

        let prepared = prepare_powershell_command(&sample_spec());

        assert!(
            prepared
                .command
                .contains("Remove-Item Env:ANTHROPIC_AUTH_TOKEN -ErrorAction SilentlyContinue"),
            "expected unset env in command: {}",
            prepared.command
        );
        assert!(
            prepared.command.contains("$env:FOO = 'bar baz'"),
            "expected env assignment in command: {}",
            prepared.command
        );
        assert!(
            prepared.command.contains("Set-Location '/work tree'"),
            "expected cwd change in command: {}",
            prepared.command
        );
        assert!(
            prepared
                .keep_open_command
                .starts_with("Write-Host 'Starting droid...'; "),
            "expected keep-open prefix in command: {}",
            prepared.keep_open_command
        );
    }

    #[test]
    #[cfg(target_os = "windows")]
    #[test]
    fn prepare_cmd_command_renders_env_unset_and_cwd() {
        use super::prepare_cmd_command;

        let prepared = prepare_cmd_command(&sample_spec());

        assert!(
            prepared.command.contains("set ANTHROPIC_AUTH_TOKEN="),
            "expected unset env in command: {}",
            prepared.command
        );
        assert!(
            prepared.command.contains("set \"FOO=bar baz\""),
            "expected env assignment in command: {}",
            prepared.command
        );
        assert!(
            prepared.command.contains("cd /d \"/work tree\""),
            "expected cwd change in command: {}",
            prepared.command
        );
        assert!(
            prepared
                .command
                .contains("\"droid\" \"--settings\" \"/tmp/demo settings.json\""),
            "expected quoted program and args in command: {}",
            prepared.command
        );
        assert_eq!(prepared.command, prepared.keep_open_command);
    }

    #[test]
    fn prepare_commands_skip_empty_env_sections() {
        let spec = LaunchSpec {
            program: "droid".to_string(),
            args: vec![],
            env: vec![],
            secret_env: vec![],
            unset_env: vec![],
            cwd: None,
            support_dir: None,
        };

        let posix = prepare_posix_command(&spec);
        assert_eq!(posix.command, "'droid'");
    }

    #[cfg(target_os = "windows")]
    #[test]
    fn prepare_windows_commands_skip_empty_env_sections() {
        use super::{prepare_cmd_command, prepare_powershell_command};

        let spec = LaunchSpec {
            program: "droid".to_string(),
            args: vec![],
            env: vec![],
            secret_env: vec![],
            unset_env: vec![],
            cwd: None,
            support_dir: None,
        };

        let powershell = prepare_powershell_command(&spec);
        let cmd = prepare_cmd_command(&spec);

        assert_eq!(powershell.command, "& 'droid'");
        assert_eq!(cmd.command, "\"droid\"");
    }

    #[test]
    fn prepare_posix_command_escapes_embedded_single_quotes() {
        let spec = LaunchSpec {
            program: "droid".to_string(),
            args: vec!["O'Brien".to_string()],
            env: vec![("MESSAGE".to_string(), "it's live".to_string())],
            secret_env: vec![],
            unset_env: vec![],
            cwd: None,
            support_dir: None,
        };

        let prepared = prepare_posix_command(&spec);

        assert!(
            prepared.command.contains("export MESSAGE='it'\\''s live'"),
            "expected escaped single quote in env value: {}",
            prepared.command
        );
        assert!(
            prepared.command.contains("'O'\\''Brien'"),
            "expected escaped single quote in argument: {}",
            prepared.command
        );
    }

    #[test]
    #[cfg(target_os = "windows")]
    #[test]
    fn prepare_cmd_command_escapes_percent_quotes_and_carets() {
        use super::prepare_cmd_command;

        let spec = LaunchSpec {
            program: "codex".to_string(),
            args: vec!["100% ready".to_string(), "^caret\"quote".to_string()],
            env: vec![("PROMPT".to_string(), "100%^\" ready".to_string())],
            secret_env: vec![],
            unset_env: vec![],
            cwd: Some(PathBuf::from(r"C:\Users\%USERNAME%\A^B")),
            support_dir: None,
        };

        let prepared = prepare_cmd_command(&spec);

        assert!(
            prepared.command.contains("set \"PROMPT=100%%^^^\" ready\""),
            "expected escaped cmd env assignment: {}",
            prepared.command
        );
        assert!(
            prepared
                .command
                .contains("cd /d \"C:\\Users\\%%USERNAME%%\\A^^B\""),
            "expected escaped cmd cwd: {}",
            prepared.command
        );
        assert!(
            prepared.command.contains("\"100%% ready\""),
            "expected escaped percent in cmd argument: {}",
            prepared.command
        );
        assert!(
            prepared.command.contains("\"^^caret^\"quote\""),
            "expected escaped caret and quote in cmd argument: {}",
            prepared.command
        );
    }

    #[test]
    fn secure_posix_command_uses_wrapper_path_without_exposing_secret_values() {
        let spec = LaunchSpec {
            program: "codex".to_string(),
            args: vec!["--help".to_string()],
            env: vec![("CODEX_HOME".to_string(), "/tmp/runtime-codex".to_string())],
            secret_env: vec![("EXAMPLE_API_KEY".to_string(), "sk-secret".to_string())],
            unset_env: vec!["OPENAI_API_KEY".to_string()],
            cwd: Some(PathBuf::from("/workspace")),
            support_dir: Some(PathBuf::from("/tmp/runtime-codex")),
        };

        let child_command = render_posix_child_command(&spec);
        assert!(child_command.contains("env -u OPENAI_API_KEY"));
        assert!(child_command.contains("CODEX_HOME='/tmp/runtime-codex'"));
        assert!(child_command.contains("EXAMPLE_API_KEY='sk-secret'"));

        let prepared = prepare_secure_posix_command(
            PathBuf::from("/tmp/runtime-codex/terminal-launch.sh").as_path(),
        );
        assert_eq!(
            prepared.command,
            "bash '/tmp/runtime-codex/terminal-launch.sh'"
        );
        assert!(prepared.keep_open_command.contains("exec bash"));
        assert!(!prepared.command.contains("sk-secret"));
        assert!(!prepared.keep_open_command.contains("sk-secret"));
    }

    #[test]
    fn launch_artifact_path_is_unique_without_support_dir() {
        let spec = sample_spec();

        let first = launch_artifact_path(&spec, "terminal-launch.command");
        let second = launch_artifact_path(&spec, "terminal-launch.command");

        assert_ne!(first, second);
        assert!(first
            .file_name()
            .and_then(|value| value.to_str())
            .is_some_and(|value| value.ends_with("terminal-launch.command")));
        assert!(second
            .file_name()
            .and_then(|value| value.to_str())
            .is_some_and(|value| value.ends_with("terminal-launch.command")));
    }
}
