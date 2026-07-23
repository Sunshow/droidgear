use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

static LAUNCH_ARTIFACT_COUNTER: AtomicU64 = AtomicU64::new(0);

/// Hide helper/launcher console processes (PATH probes, `cmd /c start` outer hop).
#[cfg(target_os = "windows")]
const CREATE_NO_WINDOW: u32 = 0x0800_0000;
/// Open a dedicated console for a user-visible PowerShell window launched from the GUI.
#[cfg(target_os = "windows")]
const CREATE_NEW_CONSOLE: u32 = 0x0000_0010;

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

#[cfg(not(target_os = "windows"))]
fn quote_posix(value: &str) -> String {
    format!("'{0}'", value.replace('\'', r"'\''"))
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

#[cfg(not(target_os = "windows"))]
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

#[cfg(not(target_os = "windows"))]
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

#[cfg(not(target_os = "windows"))]
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

#[cfg(not(target_os = "windows"))]
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
    // No `cls`: clearing the console causes an extra visible flash on launch.
    let mut lines = vec![
        "@echo off".to_string(),
        "setlocal".to_string(),
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

/// Self-deleting PowerShell launcher with Machine+User PATH refresh, env, secrets, cwd.
/// Used only when `secret_env` is present so secrets stay out of process command lines.
#[cfg(target_os = "windows")]
fn write_secure_ps_wrapper(spec: &LaunchSpec) -> Result<PathBuf, String> {
    let path = launch_artifact_path(spec, "terminal-launch.ps1");
    // No Clear-Host: wiping the console adds a visible flash before the CLI starts.
    let mut lines = vec![
        "$ErrorActionPreference = 'Continue'".to_string(),
        "try {".to_string(),
        format!("  {}", render_powershell_path_refresh()),
        format!("  {}", render_powershell_banner(&spec.program)),
    ];

    for key in &spec.unset_env {
        lines.push(format!(
            "  Remove-Item Env:{key} -ErrorAction SilentlyContinue"
        ));
    }
    for (key, value) in spec.env.iter().chain(spec.secret_env.iter()) {
        lines.push(format!("  $env:{key} = {}", quote_powershell(value)));
    }
    if let Some(cd) = render_powershell_cd(spec.cwd.as_ref()) {
        lines.push(format!("  {cd}"));
    }
    lines.push(format!(
        "  & {}",
        render_program_command(spec, quote_powershell)
    ));
    lines.push("} finally {".to_string());
    lines.push(
        "  Remove-Item -LiteralPath $PSCommandPath -Force -ErrorAction SilentlyContinue"
            .to_string(),
    );
    lines.push("}".to_string());

    let script = format!("{}\r\n", lines.join("\r\n"));
    write_launch_script(&path, &script)?;
    Ok(path)
}

/// Merge Machine + User Path so tools installed after the GUI app started are visible.
#[cfg(target_os = "windows")]
fn render_powershell_path_refresh() -> String {
    "$machinePath = [Environment]::GetEnvironmentVariable('Path','Machine'); $userPath = [Environment]::GetEnvironmentVariable('Path','User'); if ($machinePath -or $userPath) { $env:Path = @($machinePath, $userPath, $env:Path | Where-Object { $_ }) -join ';' }".to_string()
}

#[cfg(not(target_os = "windows"))]
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
    let path_refresh = render_powershell_path_refresh();
    let env_setup = render_powershell_env(
        &spec
            .env
            .iter()
            .chain(spec.secret_env.iter())
            .cloned()
            .collect::<Vec<_>>(),
        &spec.unset_env,
    );
    let cd = render_powershell_cd(spec.cwd.as_ref());
    let command = format!("& {}", render_program_command(spec, quote_powershell));
    let banner = render_powershell_banner(&spec.program);

    let base = join_non_empty(
        [
            Some(path_refresh),
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
            "iterm2" => launch_iterm2(
                spec,
                &prepared.command,
                &prepared.keep_open_command,
                &banner,
            ),
            "ghostty" => launch_ghostty(
                spec,
                &prepared.command,
                &prepared.keep_open_command,
                &banner,
            ),
            "terminal" => launch_terminal_app(&prepared.command, &banner),
            _ => launch_system_default_macos(spec, &prepared.command, &banner),
        };
    }

    let prepared = prepare_posix_command(spec);
    let banner = render_posix_banner(&spec.program);
    match preferred {
        "iterm2" => launch_iterm2(
            spec,
            &prepared.command,
            &prepared.keep_open_command,
            &banner,
        ),
        "ghostty" => launch_ghostty(
            spec,
            &prepared.command,
            &prepared.keep_open_command,
            &banner,
        ),
        "terminal" => launch_terminal_app(&prepared.command, &banner),
        _ => launch_system_default_macos(spec, &prepared.command, &banner),
    }
}

#[cfg(target_os = "macos")]
fn launch_iterm2(
    spec: &LaunchSpec,
    command: &str,
    keep_open_command: &str,
    banner: &str,
) -> Result<(), String> {
    let escaped_keep_open = keep_open_command.replace('\\', "\\\\").replace('"', "\\\"");
    let escaped_banner = banner.replace('\\', "\\\\").replace('"', "\\\"");
    let script = format!(
        r#"tell application "iTerm2"
    activate
    delay 0.5
    if (count of windows) = 0 then
        create window with default profile
        delay 0.3
    end if
    tell current window
        create tab with default profile
        tell current session
            write text "clear; {}; {}"
        end tell
    end tell
end tell"#,
        escaped_banner, escaped_keep_open
    );

    let status = std::process::Command::new("osascript")
        .arg("-e")
        .arg(&script)
        .status()
        .map_err(|e| format!("Failed to launch iTerm2: {e}"))?;

    if !status.success() {
        let file_path = launch_artifact_path(spec, "terminal-launch.command");
        let script_content =
            format!("#!/bin/bash\nrm -f -- \"$0\"\nclear\n{banner}\n{command}\nexit\n");
        write_launch_script(&file_path, &script_content)?;

        let file_arg = file_path.to_string_lossy().to_string();
        let status2 = std::process::Command::new("open")
            .args(["-a", "iTerm", &file_arg])
            .status()
            .map_err(|e| format!("Failed to launch iTerm2: {e}"))?;
        if !status2.success() {
            return Err("Failed to launch iTerm2".to_string());
        }
    }
    Ok(())
}

#[cfg(target_os = "macos")]
fn launch_ghostty(
    spec: &LaunchSpec,
    _command: &str,
    keep_open_command: &str,
    banner: &str,
) -> Result<(), String> {
    let file_path = launch_artifact_path(spec, "terminal-launch.command");
    let script_content =
        format!("#!/bin/bash\nrm -f -- \"$0\"\nclear\n{banner}\n{keep_open_command}\nexit\n");
    write_launch_script(&file_path, &script_content)?;

    let file_arg = file_path.to_string_lossy().to_string();
    let status = std::process::Command::new("open")
        .args(["-a", "Ghostty", &file_arg])
        .status()
        .map_err(|e| format!("Failed to launch Ghostty: {e}"))?;

    if !status.success() {
        return Err("Failed to launch Ghostty".to_string());
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

/// Build `cmd /c start "" [/D cwd] cmd /k <payload>` args (explicit cmd preference only).
///
/// `start` detaches the user-visible console from the GUI process tree (survives app
/// exit / job cleanup). The empty title (`""`) is required because `start` treats the
/// first quoted argument as the window title. `/D` sets the starting directory.
/// Pair with `CREATE_NO_WINDOW` on the outer `cmd` so the launcher hop itself never flashes.
#[cfg(target_os = "windows")]
fn windows_cmd_start_args(payload: &str, cwd: Option<&Path>) -> Vec<String> {
    let mut args = vec!["/c".to_string(), "start".to_string(), String::new()];
    if let Some(dir) = cwd {
        args.push("/D".to_string());
        args.push(dir.to_string_lossy().into_owned());
    }
    args.push("cmd".to_string());
    args.push("/k".to_string());
    args.push(payload.to_string());
    args
}

/// Build Windows Terminal + PowerShell args with optional `-d <cwd>`.
///
/// Default product path is a new WT window running PowerShell (not cmd).
/// Avoid `-w` / `new-tab` — they caused CLI parse errors on some WT versions.
/// `shell` is typically the resolved `pwsh` or `powershell` executable path/name.
/// `-NoLogo -NoProfile` cuts the PowerShell copyright banner flash on cold start.
#[cfg(target_os = "windows")]
fn windows_wt_ps_args(shell: &str, command: &str, cwd: Option<&Path>) -> Vec<String> {
    let mut args = Vec::new();
    if let Some(dir) = cwd {
        args.push("-d".to_string());
        args.push(dir.to_string_lossy().into_owned());
    }
    args.push(shell.to_string());
    args.push("-NoLogo".to_string());
    args.push("-NoProfile".to_string());
    args.push("-NoExit".to_string());
    args.push("-Command".to_string());
    args.push(command.to_string());
    args
}

/// Build PowerShell launch args with optional `-WorkingDirectory`.
#[cfg(target_os = "windows")]
fn windows_powershell_args(command: &str, cwd: Option<&Path>) -> Vec<String> {
    let mut args = Vec::new();
    if let Some(dir) = cwd {
        args.push("-WorkingDirectory".to_string());
        args.push(dir.to_string_lossy().into_owned());
    }
    args.push("-NoLogo".to_string());
    args.push("-NoProfile".to_string());
    args.push("-NoExit".to_string());
    args.push("-Command".to_string());
    args.push(command.to_string());
    args
}

/// Prefer PowerShell 7+ (`pwsh`) when installed; otherwise Windows PowerShell 5.1.
#[cfg(target_os = "windows")]
fn resolve_windows_powershell_shell() -> String {
    if let Some(path) = find_windows_shell_executable("pwsh") {
        return path;
    }
    if let Some(path) = find_windows_shell_executable("powershell") {
        return path;
    }
    "powershell".to_string()
}

#[cfg(target_os = "windows")]
fn find_windows_shell_executable(name: &str) -> Option<String> {
    let resolved = resolve_windows_program(name);
    if Path::new(&resolved).is_file() {
        return Some(resolved);
    }

    // Common install locations when PATH is stale (GUI apps often miss user installs).
    if name.eq_ignore_ascii_case("pwsh") {
        let mut candidates = Vec::new();
        if let Ok(program_files) = std::env::var("ProgramFiles") {
            candidates.push(PathBuf::from(&program_files).join(r"PowerShell\7\pwsh.exe"));
            candidates.push(PathBuf::from(&program_files).join(r"PowerShell\7-preview\pwsh.exe"));
        }
        if let Ok(local_app_data) = std::env::var("LOCALAPPDATA") {
            candidates.push(PathBuf::from(local_app_data).join(r"Microsoft\powershell\pwsh.exe"));
        }
        for candidate in candidates {
            if candidate.is_file() {
                return Some(candidate.to_string_lossy().into_owned());
            }
        }
    }

    None
}

/// Resolve a bare program name against process PATH first, then Machine/User PATH.
/// Prefer process PATH so common installs avoid spawning helper PowerShell probes.
#[cfg(target_os = "windows")]
fn resolve_windows_program(program: &str) -> String {
    let path = Path::new(program);
    if path.is_absolute() {
        return program.to_string();
    }
    if program.contains('\\') || program.contains('/') {
        if path.exists() {
            return path.to_string_lossy().into_owned();
        }
        return program.to_string();
    }

    let candidates = if program.to_ascii_lowercase().ends_with(".exe") {
        vec![program.to_string()]
    } else {
        vec![format!("{program}.exe"), program.to_string()]
    };

    if let Ok(process_path) = std::env::var("PATH") {
        if let Some(found) = search_program_in_path_value(&process_path, &candidates) {
            return found;
        }
    }

    // Fallback: registry-backed Machine/User PATH (GUI apps often miss user installs).
    let mut search_dirs: Vec<PathBuf> = Vec::new();
    for scope in ["Machine", "User"] {
        if let Some(value) = windows_environment_path(scope) {
            for part in value.split(';') {
                let part = part.trim();
                if !part.is_empty() {
                    search_dirs.push(PathBuf::from(part));
                }
            }
        }
    }

    for dir in search_dirs {
        for name in &candidates {
            let candidate = dir.join(name);
            if candidate.is_file() {
                return candidate.to_string_lossy().into_owned();
            }
        }
    }

    program.to_string()
}

#[cfg(target_os = "windows")]
fn search_program_in_path_value(path_value: &str, candidates: &[String]) -> Option<String> {
    for part in path_value.split(';') {
        let part = part.trim();
        if part.is_empty() {
            continue;
        }
        let dir = PathBuf::from(part);
        for name in candidates {
            let candidate = dir.join(name);
            if candidate.is_file() {
                return Some(candidate.to_string_lossy().into_owned());
            }
        }
    }
    None
}

#[cfg(target_os = "windows")]
fn windows_environment_path(scope: &str) -> Option<String> {
    use std::os::windows::process::CommandExt;

    let output = std::process::Command::new("powershell")
        .args([
            "-NoLogo",
            "-NoProfile",
            "-Command",
            &format!("[Environment]::GetEnvironmentVariable('Path','{scope}')"),
        ])
        // Helper probe only — never flash a console from the GUI process.
        .creation_flags(CREATE_NO_WINDOW)
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    let value = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if value.is_empty() {
        None
    } else {
        Some(value)
    }
}

#[cfg(target_os = "windows")]
fn launch_windows_cmd_payload(payload: &str, cwd: Option<&Path>) -> Result<(), String> {
    use std::os::windows::process::CommandExt;

    // Keep `start` so the terminal detaches from the GUI process tree, but hide the
    // outer `cmd /c start` launcher console to avoid the intermediate black flash.
    std::process::Command::new("cmd")
        .args(windows_cmd_start_args(payload, cwd))
        .creation_flags(CREATE_NO_WINDOW)
        .spawn()
        .map_err(|e| format!("Failed to launch cmd: {e}"))?;
    Ok(())
}

#[cfg(target_os = "windows")]
fn launch_windows_powershell_payload(
    shell: &str,
    command: &str,
    cwd: Option<&Path>,
) -> Result<(), String> {
    use std::os::windows::process::CommandExt;

    let mut process = std::process::Command::new(shell);
    process.args(windows_powershell_args(command, cwd));
    // Explicit new console when the parent is a GUI app.
    process.creation_flags(CREATE_NEW_CONSOLE);
    process
        .spawn()
        .map_err(|e| format!("Failed to launch PowerShell: {e}"))?;
    Ok(())
}

/// Default path: Windows Terminal + PowerShell; fallback to PowerShell window.
#[cfg(target_os = "windows")]
fn launch_windows_wt_or_ps_payload(
    shell: &str,
    command: &str,
    cwd: Option<&Path>,
) -> Result<(), String> {
    if std::process::Command::new("wt")
        .args(windows_wt_ps_args(shell, command, cwd))
        .spawn()
        .is_ok()
    {
        return Ok(());
    }

    launch_windows_powershell_payload(shell, command, cwd)
}

/// Build the PowerShell -Command payload for non-cmd launches.
///
/// Temp wrappers are only required when secrets must stay out of argv.
/// Non-secret env/unset/cwd go inline (and cwd is also set via `-d` / `-WorkingDirectory`).
#[cfg(target_os = "windows")]
fn windows_ps_launch_command(spec: &LaunchSpec) -> Result<String, String> {
    if needs_secure_wrapper(spec) {
        let wrapper_path = write_secure_ps_wrapper(spec)?;
        Ok(format!(
            "& {}",
            quote_powershell(&wrapper_path.to_string_lossy())
        ))
    } else {
        Ok(prepare_powershell_command(spec).keep_open_command)
    }
}

#[cfg(target_os = "windows")]
fn launch_windows(spec: &LaunchSpec, preferred: &str) -> Result<(), String> {
    let mut resolved = spec.clone();
    resolved.program = resolve_windows_program(&resolved.program);
    let cwd = resolved.cwd.as_deref();
    let shell = resolve_windows_powershell_shell();

    match preferred {
        "cmd" => {
            if needs_secure_wrapper(&resolved) {
                let wrapper_path = write_secure_cmd_wrapper(&resolved)?;
                let wrapper_cmd = wrapper_path.to_string_lossy().to_string();
                launch_windows_cmd_payload(&wrapper_cmd, cwd)
            } else {
                let prepared = prepare_cmd_command(&resolved);
                launch_windows_cmd_payload(&prepared.keep_open_command, cwd)
            }
        }
        "powershell" => {
            let command = windows_ps_launch_command(&resolved)?;
            launch_windows_powershell_payload(&shell, &command, cwd)
        }
        _ => {
            // Default: Windows Terminal + PowerShell/pwsh (never bare cmd).
            let command = windows_ps_launch_command(&resolved)?;
            launch_windows_wt_or_ps_payload(&shell, &command, cwd)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{launch_artifact_path, LaunchSpec};
    use std::path::PathBuf;

    #[cfg(not(target_os = "windows"))]
    use super::{prepare_posix_command, prepare_secure_posix_command, render_posix_child_command};

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

    #[cfg(not(target_os = "windows"))]
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

    #[cfg(target_os = "windows")]
    #[test]
    fn prepare_powershell_command_renders_env_unset_and_cwd() {
        use super::prepare_powershell_command;

        let prepared = prepare_powershell_command(&sample_spec());

        assert!(
            prepared
                .command
                .contains("GetEnvironmentVariable('Path','Machine')"),
            "expected PATH refresh in command: {}",
            prepared.command
        );
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

    #[cfg(target_os = "windows")]
    #[test]
    fn windows_cmd_start_args_include_empty_title_and_optional_d() {
        use super::windows_cmd_start_args;
        use std::path::Path;

        let without_cwd = windows_cmd_start_args(r"C:\temp\run.cmd", None);
        assert_eq!(
            without_cwd,
            vec![
                "/c".to_string(),
                "start".to_string(),
                String::new(),
                "cmd".to_string(),
                "/k".to_string(),
                r"C:\temp\run.cmd".to_string(),
            ]
        );

        let with_cwd = windows_cmd_start_args(r"C:\temp\run.cmd", Some(Path::new(r"D:\work tree")));
        assert_eq!(
            with_cwd,
            vec![
                "/c".to_string(),
                "start".to_string(),
                String::new(),
                "/D".to_string(),
                r"D:\work tree".to_string(),
                "cmd".to_string(),
                "/k".to_string(),
                r"C:\temp\run.cmd".to_string(),
            ]
        );
    }

    #[cfg(target_os = "windows")]
    #[test]
    fn windows_wt_and_powershell_args_include_start_directory_and_quiet_flags() {
        use super::{windows_powershell_args, windows_wt_ps_args};
        use std::path::Path;

        let wt_with_cwd =
            windows_wt_ps_args("pwsh", "& 'C:\\temp\\run.ps1'", Some(Path::new(r"D:\proj")));
        assert_eq!(
            wt_with_cwd,
            vec![
                "-d".to_string(),
                r"D:\proj".to_string(),
                "pwsh".to_string(),
                "-NoLogo".to_string(),
                "-NoProfile".to_string(),
                "-NoExit".to_string(),
                "-Command".to_string(),
                "& 'C:\\temp\\run.ps1'".to_string(),
            ]
        );

        let wt_plain = windows_wt_ps_args("powershell", "& 'C:\\temp\\run.ps1'", None);
        assert_eq!(
            wt_plain,
            vec![
                "powershell".to_string(),
                "-NoLogo".to_string(),
                "-NoProfile".to_string(),
                "-NoExit".to_string(),
                "-Command".to_string(),
                "& 'C:\\temp\\run.ps1'".to_string(),
            ]
        );

        let ps = windows_powershell_args(r"& 'C:\temp\run.ps1'", Some(Path::new(r"D:\work")));
        assert_eq!(
            ps,
            vec![
                "-WorkingDirectory".to_string(),
                r"D:\work".to_string(),
                "-NoLogo".to_string(),
                "-NoProfile".to_string(),
                "-NoExit".to_string(),
                "-Command".to_string(),
                r"& 'C:\temp\run.ps1'".to_string(),
            ]
        );
    }

    #[cfg(target_os = "windows")]
    #[test]
    fn windows_ps_launch_command_inlines_non_secret_env_without_wrapper() {
        use super::windows_ps_launch_command;

        let command = windows_ps_launch_command(&sample_spec()).unwrap();
        assert!(
            command.contains("$env:FOO = 'bar baz'"),
            "expected inline env: {command}"
        );
        assert!(
            command.contains("Remove-Item Env:ANTHROPIC_AUTH_TOKEN"),
            "expected inline unset: {command}"
        );
        assert!(
            command.contains("& 'droid'"),
            "expected program: {command}"
        );
        assert!(
            !command.contains("terminal-launch.ps1"),
            "non-secret launches should not write a temp wrapper: {command}"
        );
    }

    #[cfg(target_os = "windows")]
    #[test]
    fn windows_ps_launch_command_uses_wrapper_only_for_secrets() {
        use super::windows_ps_launch_command;

        let support_dir = std::env::temp_dir().join(format!(
            "droidgear-ps-launch-secret-{}",
            std::process::id()
        ));
        std::fs::create_dir_all(&support_dir).unwrap();

        let mut spec = sample_spec();
        spec.secret_env = vec![("EXAMPLE_API_KEY".to_string(), "sk-secret".to_string())];
        spec.support_dir = Some(support_dir.clone());

        let command = windows_ps_launch_command(&spec).unwrap();
        let _ = std::fs::remove_dir_all(&support_dir);

        assert!(
            command.contains("terminal-launch.ps1"),
            "secret launches must use a temp wrapper: {command}"
        );
        assert!(
            !command.contains("sk-secret"),
            "secret value must not appear in the outer command: {command}"
        );
    }

    #[cfg(target_os = "windows")]
    #[test]
    fn find_windows_shell_executable_prefers_existing_file() {
        use super::find_windows_shell_executable;

        let support_dir =
            std::env::temp_dir().join(format!("droidgear-shell-resolve-{}", std::process::id()));
        std::fs::create_dir_all(&support_dir).unwrap();
        let fake_pwsh = support_dir.join("pwsh.exe");
        std::fs::write(&fake_pwsh, b"").unwrap();

        // Inject into process PATH so resolve_windows_program can find it.
        let original_path = std::env::var("PATH").unwrap_or_default();
        let mutated = format!("{};{}", support_dir.to_string_lossy(), original_path);
        // SAFETY: test-only mutation of process PATH for resolution probe.
        unsafe {
            std::env::set_var("PATH", &mutated);
        }

        let found = find_windows_shell_executable("pwsh");

        unsafe {
            std::env::set_var("PATH", original_path);
        }
        let _ = std::fs::remove_dir_all(&support_dir);

        let found = found.expect("pwsh should be resolved from temp PATH entry");
        assert!(
            found.to_ascii_lowercase().ends_with("pwsh.exe"),
            "unexpected shell path: {found}"
        );
    }

    #[cfg(target_os = "windows")]
    #[test]
    fn secure_ps_wrapper_includes_path_refresh_env_and_program() {
        use super::write_secure_ps_wrapper;

        let support_dir =
            std::env::temp_dir().join(format!("droidgear-ps-wrapper-test-{}", std::process::id()));
        std::fs::create_dir_all(&support_dir).unwrap();

        let spec = LaunchSpec {
            program: "codex".to_string(),
            args: vec![],
            env: vec![("CODEX_HOME".to_string(), r"C:\tmp\runtime".to_string())],
            secret_env: vec![("EXAMPLE_API_KEY".to_string(), "sk-secret".to_string())],
            unset_env: vec!["OPENAI_API_KEY".to_string()],
            cwd: Some(PathBuf::from(r"D:\work tree")),
            support_dir: Some(support_dir.clone()),
        };

        let path = write_secure_ps_wrapper(&spec).unwrap();
        let contents = std::fs::read_to_string(&path).unwrap();
        let _ = std::fs::remove_dir_all(&support_dir);

        assert!(
            contents.contains("GetEnvironmentVariable('Path','Machine')"),
            "expected PATH refresh: {contents}"
        );
        assert!(
            contents.contains("Remove-Item Env:OPENAI_API_KEY"),
            "expected unset env: {contents}"
        );
        assert!(
            contents.contains("$env:CODEX_HOME = 'C:\\tmp\\runtime'"),
            "expected CODEX_HOME: {contents}"
        );
        assert!(
            contents.contains("$env:EXAMPLE_API_KEY = 'sk-secret'"),
            "expected secret env: {contents}"
        );
        assert!(
            contents.contains("Set-Location 'D:\\work tree'"),
            "expected cwd: {contents}"
        );
        assert!(
            contents.contains("& 'codex'"),
            "expected program: {contents}"
        );
        assert!(
            contents.contains("Remove-Item -LiteralPath $PSCommandPath"),
            "expected self-delete: {contents}"
        );
    }

    #[cfg(not(target_os = "windows"))]
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

        assert!(
            powershell
                .command
                .contains("GetEnvironmentVariable('Path','Machine')"),
            "expected PATH refresh even without env: {}",
            powershell.command
        );
        assert!(
            powershell.command.contains("& 'droid'"),
            "expected program invocation: {}",
            powershell.command
        );
        assert_eq!(cmd.command, "\"droid\"");
    }

    #[cfg(not(target_os = "windows"))]
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

    #[cfg(not(target_os = "windows"))]
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
