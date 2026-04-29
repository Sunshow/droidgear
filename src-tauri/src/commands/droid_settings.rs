//! Droid settings file management commands (Tauri wrappers).
//!
//! Core logic lives in `droidgear_core::droid_settings_files`.

pub use droidgear_core::droid_settings_files::SettingsFileInfo;

use droidgear_core::droid_settings_files;
use tauri::Manager;

/// Lists all available Droid settings files (global + custom)
#[tauri::command]
#[specta::specta]
pub async fn list_droid_settings_files() -> Result<Vec<SettingsFileInfo>, String> {
    droid_settings_files::list_settings_files()
}

/// Gets the currently active settings file info
#[tauri::command]
#[specta::specta]
pub async fn get_active_droid_settings_file() -> Result<SettingsFileInfo, String> {
    droid_settings_files::get_active_settings_file()
}

/// Sets the active settings file. Pass null or empty to switch to Global.
#[tauri::command]
#[specta::specta]
pub async fn set_active_droid_settings_file(
    name: Option<String>,
) -> Result<SettingsFileInfo, String> {
    droid_settings_files::set_active_settings_file(name)
}

/// Creates a new settings file. If copy_from_active is true, copies from the active file.
#[tauri::command]
#[specta::specta]
pub async fn create_droid_settings_file(
    name: String,
    copy_from_active: bool,
) -> Result<SettingsFileInfo, String> {
    droid_settings_files::create_settings_file(name, copy_from_active)
}

/// Deletes a custom settings file. Cannot delete the global file.
#[tauri::command]
#[specta::specta]
pub async fn delete_droid_settings_file(name: String) -> Result<(), String> {
    droid_settings_files::delete_settings_file(name)
}

/// Gets the launch command for Droid with the active settings file.
/// Returns [command_string, settings_path].
#[tauri::command]
#[specta::specta]
pub async fn get_droid_launch_command() -> Result<(String, String), String> {
    droid_settings_files::get_launch_command()
}

/// Launches Droid CLI in a terminal with the active settings file.
/// Respects the user's preferredTerminal preference.
#[tauri::command]
#[specta::specta]
pub async fn launch_droid(app: tauri::AppHandle) -> Result<(), String> {
    let (command, _path) = droid_settings_files::get_launch_command()?;

    // Read preferred terminal from preferences
    let preferred = load_preferred_terminal(&app).unwrap_or_default();

    launch_droid_in_terminal(&command, &preferred)
}

fn load_preferred_terminal(app: &tauri::AppHandle) -> Result<String, String> {
    let prefs_path = {
        let app_data_dir = app
            .path()
            .app_data_dir()
            .map_err(|e| format!("Failed to get app data dir: {e}"))?;
        app_data_dir.join("preferences.json")
    };

    if !prefs_path.exists() {
        return Ok(String::new());
    }

    let contents = std::fs::read_to_string(&prefs_path)
        .map_err(|e| format!("Failed to read preferences: {e}"))?;
    let prefs: serde_json::Value = serde_json::from_str(&contents).map_err(|_e| String::new())?;

    Ok(prefs
        .get("preferred_terminal")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string())
}

fn launch_droid_in_terminal(command: &str, preferred: &str) -> Result<(), String> {
    #[cfg(target_os = "macos")]
    {
        launch_macos(command, preferred)
    }
    #[cfg(target_os = "linux")]
    {
        launch_linux(command, preferred)
    }
    #[cfg(target_os = "windows")]
    {
        launch_windows(command, preferred)
    }
    #[cfg(not(any(target_os = "macos", target_os = "linux", target_os = "windows")))]
    {
        Err("Unsupported platform".to_string())
    }
}

// ============================================================================
// macOS
// ============================================================================

#[cfg(target_os = "macos")]
fn launch_macos(command: &str, preferred: &str) -> Result<(), String> {
    match preferred {
        "iterm2" => launch_iterm2(command),
        "terminal" => launch_terminal_app(command),
        _ => launch_system_default_macos(command), // "system-default" or empty
    }
}

#[cfg(target_os = "macos")]
fn launch_iterm2(command: &str) -> Result<(), String> {
    let escaped = command.replace('\\', "\\\\").replace('"', "\\\"");
    // Try to create a new tab in the current window; if iTerm2 isn't running, it will launch
    let script = format!(
        r#"tell application "iTerm2"
    if (count of windows) = 0 then
        create window with default profile
    end if
    tell current window
        create tab with default profile
        tell current session
            write text "clear; echo 'Starting Droid...'; {}; exit"
        end tell
    end tell
    activate
end tell"#,
        escaped
    );

    let status = std::process::Command::new("osascript")
        .arg("-e")
        .arg(&script)
        .status()
        .map_err(|e| format!("Failed to launch iTerm2: {e}"))?;

    if !status.success() {
        // Fallback: try launching iTerm2 directly
        let status2 = std::process::Command::new("open")
            .args([
                "-a",
                "iTerm",
                "--args",
                "bash",
                "-c",
                &format!("{}; exec bash", command),
            ])
            .status()
            .map_err(|e| format!("Failed to launch iTerm2: {e}"))?;
        if !status2.success() {
            return Err("Failed to launch iTerm2".to_string());
        }
    }
    Ok(())
}

#[cfg(target_os = "macos")]
fn launch_terminal_app(command: &str) -> Result<(), String> {
    let escaped = command.replace('\\', "\\\\").replace('"', "\\\"");
    let script = format!(
        r#"tell application "Terminal"
    activate
    do script "clear; echo 'Starting Droid...'; {}; exit"
end tell"#,
        escaped
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
fn launch_system_default_macos(command: &str) -> Result<(), String> {
    // Create a temporary .command file and open it with the system default handler
    let tmp_dir = std::env::temp_dir();
    let file_path = tmp_dir.join("droid-launch.command");
    let script_content = format!(
        "#!/bin/bash\nclear\necho 'Starting Droid...'\n{}\nexit\n",
        command
    );
    std::fs::write(&file_path, script_content)
        .map_err(|e| format!("Failed to create launch script: {e}"))?;

    // Make it executable
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = std::fs::metadata(&file_path)
            .map_err(|e| format!("Failed to read metadata: {e}"))?
            .permissions();
        perms.set_mode(0o755);
        std::fs::set_permissions(&file_path, perms)
            .map_err(|e| format!("Failed to set permissions: {e}"))?;
    }

    let status = std::process::Command::new("open")
        .arg(&file_path)
        .status()
        .map_err(|e| format!("Failed to open terminal: {e}"))?;

    if !status.success() {
        return Err("Failed to open terminal".to_string());
    }
    Ok(())
}

// ============================================================================
// Linux
// ============================================================================

#[cfg(target_os = "linux")]
fn launch_linux(command: &str, preferred: &str) -> Result<(), String> {
    let terminals: Vec<(&str, Vec<&str>)> = match preferred {
        "gnome-terminal" => vec![(
            "gnome-terminal",
            vec![
                "--tab",
                "--",
                "bash",
                "-c",
                &format!("{}; exec bash", command),
            ],
        )],
        "konsole" => vec![(
            "konsole",
            vec![
                "--new-tab",
                "-e",
                "bash",
                "-c",
                &format!("{}; exec bash", command),
            ],
        )],
        "xfce4-terminal" => vec![(
            "xfce4-terminal",
            vec!["--tab", "-e", &format!("bash -c '{}; exec bash'", command)],
        )],
        "x-terminal-emulator" => vec![(
            "x-terminal-emulator",
            vec!["-e", "bash", "-c", &format!("{}; exec bash", command)],
        )],
        _ => {
            // auto-detect or empty — try common terminals in order
            vec![
                (
                    "gnome-terminal",
                    vec![
                        "--tab",
                        "--",
                        "bash",
                        "-c",
                        &format!("{}; exec bash", command),
                    ],
                ),
                (
                    "konsole",
                    vec![
                        "--new-tab",
                        "-e",
                        "bash",
                        "-c",
                        &format!("{}; exec bash", command),
                    ],
                ),
                (
                    "xfce4-terminal",
                    vec!["--tab", "-e", &format!("bash -c '{}; exec bash'", command)],
                ),
                (
                    "x-terminal-emulator",
                    vec!["-e", "bash", "-c", &format!("{}; exec bash", command)],
                ),
                (
                    "xterm",
                    vec!["-e", "bash", "-c", &format!("{}; exec bash", command)],
                ),
            ]
        }
    };

    let mut last_err = String::new();
    for (term, args) in &terminals {
        let args: Vec<&str> = args.iter().map(|s| *s).collect();
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

// ============================================================================
// Windows
// ============================================================================

#[cfg(target_os = "windows")]
fn launch_windows(command: &str, preferred: &str) -> Result<(), String> {
    match preferred {
        "cmd" => {
            std::process::Command::new("cmd")
                .args(["/c", "start", "cmd", "/k", command])
                .spawn()
                .map_err(|e| format!("Failed to launch cmd: {e}"))?;
            Ok(())
        }
        "powershell" => {
            std::process::Command::new("powershell")
                .args([
                    "-NoExit",
                    "-Command",
                    &format!("Write-Host 'Starting Droid...'; {}", command),
                ])
                .spawn()
                .map_err(|e| format!("Failed to launch PowerShell: {e}"))?;
            Ok(())
        }
        _ => {
            // "windows-terminal" or empty — try Windows Terminal first, fall back to cmd
            let wt_status = std::process::Command::new("wt")
                .args(["-w", "0", "new-tab", "cmd", "/k", command])
                .spawn();

            if wt_status.is_ok() {
                return Ok(());
            }

            // Fallback: try launching Windows Terminal without -w flag (first launch)
            let wt_status2 = std::process::Command::new("wt")
                .args(["cmd", "/k", command])
                .spawn();

            if wt_status2.is_ok() {
                return Ok(());
            }

            std::process::Command::new("cmd")
                .args(["/c", "start", "cmd", "/k", command])
                .spawn()
                .map_err(|e| format!("Failed to launch command prompt: {e}"))?;
            Ok(())
        }
    }
}
