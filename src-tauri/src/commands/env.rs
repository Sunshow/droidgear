//! Environment variable commands.

use std::fs::OpenOptions;
use std::io::Write;
use std::path::PathBuf;

/// Gets the value of an environment variable.
/// Returns None if the variable is not set.
#[tauri::command]
#[specta::specta]
pub fn get_env_var(name: &str) -> Option<String> {
    std::env::var(name).ok()
}

/// Sets an environment variable for the current process.
/// Note: This only affects the current process, not the system or shell.
#[tauri::command]
#[specta::specta]
pub fn set_env_var(name: &str, value: &str) {
    std::env::set_var(name, value);
}

/// Removes an environment variable from the current process.
/// Note: This only affects the current process, not the system or shell.
#[tauri::command]
#[specta::specta]
pub fn remove_env_var(name: &str) {
    std::env::remove_var(name);
}

/// Sets up an environment variable in the user's shell configuration file.
/// Returns the path of the file that was modified on success.
#[tauri::command]
#[specta::specta]
pub fn setup_env_in_shell_config(key: &str, value: &str) -> Result<String, String> {
    #[cfg(target_os = "windows")]
    {
        setup_env_in_shell_config_windows(key, value)
    }

    #[cfg(not(target_os = "windows"))]
    {
        setup_env_in_shell_config_unix(key, value)
    }
}

#[cfg(target_os = "windows")]
fn setup_env_in_shell_config_windows(key: &str, value: &str) -> Result<String, String> {
    let userprofile =
        std::env::var("USERPROFILE").map_err(|_| "Cannot determine user profile directory")?;
    let userprofile_path = PathBuf::from(&userprofile);

    // PowerShell profile path: Documents\PowerShell\Microsoft.PowerShell_profile.ps1
    // For Windows PowerShell (5.x): Documents\WindowsPowerShell\Microsoft.PowerShell_profile.ps1
    // We'll use the newer PowerShell 7+ path first, fall back to WindowsPowerShell
    let ps_core_dir = userprofile_path.join("Documents").join("PowerShell");
    let ps_legacy_dir = userprofile_path.join("Documents").join("WindowsPowerShell");

    let (profile_dir, config_file) = if ps_core_dir.exists() {
        (
            ps_core_dir.clone(),
            ps_core_dir.join("Microsoft.PowerShell_profile.ps1"),
        )
    } else {
        (
            ps_legacy_dir.clone(),
            ps_legacy_dir.join("Microsoft.PowerShell_profile.ps1"),
        )
    };

    // Create directory if it doesn't exist
    if !profile_dir.exists() {
        std::fs::create_dir_all(&profile_dir)
            .map_err(|e| format!("Failed to create directory {}: {e}", profile_dir.display()))?;
    }

    // PowerShell syntax: $env:KEY = "value"
    let export_line = format!("\n$env:{key} = \"{value}\"\n");

    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(&config_file)
        .map_err(|e| format!("Failed to open {}: {e}", config_file.display()))?;

    file.write_all(export_line.as_bytes())
        .map_err(|e| format!("Failed to write to {}: {e}", config_file.display()))?;

    Ok(config_file.display().to_string())
}

#[cfg(not(target_os = "windows"))]
fn setup_env_in_shell_config_unix(key: &str, value: &str) -> Result<String, String> {
    let shell = std::env::var("SHELL").unwrap_or_default();
    let home = std::env::var("HOME").map_err(|_| "Cannot determine home directory")?;
    let home_path = PathBuf::from(&home);

    let config_file = if shell.contains("zsh") {
        home_path.join(".zshrc")
    } else if shell.contains("bash") {
        // macOS uses .bash_profile, Linux uses .bashrc
        if cfg!(target_os = "macos") {
            home_path.join(".bash_profile")
        } else {
            home_path.join(".bashrc")
        }
    } else {
        return Err(format!(
            "Unknown shell: {}. Please set the environment variable manually.",
            if shell.is_empty() {
                "not detected"
            } else {
                &shell
            }
        ));
    };

    let export_line = format!("\nexport {key}=\"{value}\"\n");

    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(&config_file)
        .map_err(|e| format!("Failed to open {}: {e}", config_file.display()))?;

    file.write_all(export_line.as_bytes())
        .map_err(|e| format!("Failed to write to {}: {e}", config_file.display()))?;

    Ok(config_file.display().to_string())
}
