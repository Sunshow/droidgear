//! Droid settings file management (core).
//!
//! Manages multiple Droid settings files stored in `~/.droidgear/droid-settings/`.
//! Tracks the active settings file and provides path resolution for read/write operations.

use serde::{Deserialize, Serialize};
use specta::Type;
use std::path::PathBuf;

use crate::paths;

const DROID_SETTINGS_DIR: &str = "droid-settings";
const ACTIVE_FILE_KEY: &str = "droidSettingsActiveFile";

// ============================================================================
// Types
// ============================================================================

/// Information about a single settings file
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct SettingsFileInfo {
    /// Display name ("Global" or the filename without extension)
    pub name: String,
    /// Full path to the settings file
    pub path: String,
    /// Whether this is the global `~/.factory/settings.json`
    pub is_global: bool,
    /// Whether this file is currently active for editing
    pub is_active: bool,
    /// Whether the file exists on disk
    pub exists: bool,
}

// ============================================================================
// Path resolution
// ============================================================================

fn system_home_dir() -> Result<PathBuf, String> {
    dirs::home_dir().ok_or_else(|| "Failed to get home directory".to_string())
}

fn droidgear_dir() -> Result<PathBuf, String> {
    Ok(paths::droidgear_dir_from_home(&system_home_dir()?))
}

fn droid_settings_dir() -> Result<PathBuf, String> {
    Ok(droidgear_dir()?.join(DROID_SETTINGS_DIR))
}

fn global_settings_path() -> Result<PathBuf, String> {
    let home = system_home_dir()?;
    Ok(home.join(".factory").join("settings.json"))
}

/// Resolves the absolute path to the currently active settings file.
/// Returns the global path if no custom file is set, or if the active file doesn't exist.
pub fn get_active_settings_path() -> Result<PathBuf, String> {
    let active_name = load_active_file_name()?;
    match active_name {
        Some(name) if !name.is_empty() => {
            let custom_path = droid_settings_dir()?.join(&name).with_extension("json");
            if custom_path.exists() {
                Ok(custom_path)
            } else {
                // Fall back to global if custom file doesn't exist
                global_settings_path()
            }
        }
        _ => global_settings_path(),
    }
}

// ============================================================================
// Active file tracking
// ============================================================================

fn load_active_file_name() -> Result<Option<String>, String> {
    let settings_path = paths::get_droidgear_settings_path()?;
    let settings = paths::read_droidgear_settings_from_path_internal(&settings_path)?;
    Ok(settings
        .get(ACTIVE_FILE_KEY)
        .and_then(|v| v.as_str())
        .map(String::from))
}

fn save_active_file_name(name: Option<&str>) -> Result<(), String> {
    let settings_path = paths::get_droidgear_settings_path()?;
    let mut settings = paths::read_droidgear_settings_from_path_internal(&settings_path)?;

    if let Some(obj) = settings.as_object_mut() {
        match name {
            Some(n) if !n.is_empty() => {
                obj.insert(ACTIVE_FILE_KEY.to_string(), serde_json::json!(n));
            }
            _ => {
                obj.remove(ACTIVE_FILE_KEY);
            }
        }
    }

    paths::write_droidgear_settings_to_path_internal(&settings_path, &settings)?;
    Ok(())
}

// ============================================================================
// Public API
// ============================================================================

/// List all available settings files (global + custom files)
pub fn list_settings_files() -> Result<Vec<SettingsFileInfo>, String> {
    let active_name = load_active_file_name().unwrap_or(None);
    let global_path = global_settings_path()?;
    let mut files = Vec::new();

    // Global file
    files.push(SettingsFileInfo {
        name: "Global".to_string(),
        path: global_path.to_string_lossy().to_string(),
        is_global: true,
        is_active: active_name.is_none(),
        exists: global_path.exists(),
    });

    // Custom files from ~/.droidgear/droid-settings/
    let dir = droid_settings_dir()?;
    if dir.exists() {
        if let Ok(entries) = std::fs::read_dir(&dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.extension().is_some_and(|ext| ext == "json") {
                    let name = path
                        .file_stem()
                        .and_then(|s| s.to_str())
                        .unwrap_or("unknown")
                        .to_string();
                    let is_active = active_name.as_deref() == Some(&name);
                    files.push(SettingsFileInfo {
                        name: name.clone(),
                        path: path.to_string_lossy().to_string(),
                        is_global: false,
                        is_active,
                        exists: true,
                    });
                }
            }
        }
    }

    Ok(files)
}

/// Get the currently active settings file info
pub fn get_active_settings_file() -> Result<SettingsFileInfo, String> {
    let files = list_settings_files()?;
    files
        .into_iter()
        .find(|f| f.is_active)
        .ok_or_else(|| "No active settings file found".to_string())
}

/// Set the active settings file.
/// Pass `None` or empty string to switch to Global.
pub fn set_active_settings_file(name: Option<String>) -> Result<SettingsFileInfo, String> {
    match &name {
        Some(n) if !n.is_empty() => {
            let path = droid_settings_dir()?.join(n).with_extension("json");
            if !path.exists() {
                return Err(format!("Settings file '{}' does not exist", n));
            }
            save_active_file_name(Some(n))?;
        }
        _ => {
            save_active_file_name(None)?;
        }
    }

    get_active_settings_file()
}

/// Create a new settings file.
/// If `copy_from_active` is true, copies the current active file's content.
pub fn create_settings_file(
    name: String,
    copy_from_active: bool,
) -> Result<SettingsFileInfo, String> {
    if name.is_empty() {
        return Err("File name cannot be empty".to_string());
    }
    if name.eq_ignore_ascii_case("global") {
        return Err("Cannot use 'Global' as a custom file name".to_string());
    }

    let dir = droid_settings_dir()?;
    if !dir.exists() {
        std::fs::create_dir_all(&dir)
            .map_err(|e| format!("Failed to create droid-settings directory: {e}"))?;
    }

    let path = dir.join(&name).with_extension("json");
    if path.exists() {
        return Err(format!("Settings file '{}' already exists", name));
    }

    if copy_from_active {
        let active_path = get_active_settings_path()?;
        if active_path.exists() {
            std::fs::copy(&active_path, &path)
                .map_err(|e| format!("Failed to copy settings: {e}"))?;
        } else {
            std::fs::write(&path, "{}")
                .map_err(|e| format!("Failed to create settings file: {e}"))?;
        }
    } else {
        std::fs::write(&path, "{}").map_err(|e| format!("Failed to create settings file: {e}"))?;
    }

    // Auto-switch to the new file
    save_active_file_name(Some(&name))?;

    get_active_settings_file()
}

/// Delete a custom settings file. Cannot delete the global file.
pub fn delete_settings_file(name: String) -> Result<(), String> {
    if name.eq_ignore_ascii_case("global") {
        return Err("Cannot delete the global settings file".to_string());
    }

    let path = droid_settings_dir()?.join(&name).with_extension("json");
    if !path.exists() {
        return Err(format!("Settings file '{}' does not exist", name));
    }

    std::fs::remove_file(&path).map_err(|e| format!("Failed to delete settings file: {e}"))?;

    // If the deleted file was active, switch back to Global
    let active_name = load_active_file_name().unwrap_or(None);
    if active_name.as_deref() == Some(&name) {
        save_active_file_name(None)?;
    }

    Ok(())
}

/// Get the launch command for Droid with the active settings file.
/// Returns the command string and the settings path used.
pub fn get_launch_command() -> Result<(String, String), String> {
    let active_path = get_active_settings_path()?;
    let path_str = active_path.to_string_lossy().to_string();

    let is_global = {
        let global = global_settings_path()?;
        active_path == global
    };

    let command = if is_global {
        "droid".to_string()
    } else {
        format!("droid --settings \"{}\"", path_str)
    };

    Ok((command, path_str))
}
