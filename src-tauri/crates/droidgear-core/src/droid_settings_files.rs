//! Droid settings file management (core).
//!
//! Manages multiple Droid settings files stored in `~/.droidgear/droid-settings/`.
//! Tracks the active settings file and provides path resolution for read/write operations.

use serde::{Deserialize, Serialize};
use specta::Type;
use std::path::{Path, PathBuf};

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

fn droidgear_dir_for_home(home_dir: &Path) -> PathBuf {
    paths::droidgear_dir_from_home(home_dir)
}

fn droidgear_dir() -> Result<PathBuf, String> {
    Ok(paths::droidgear_dir_from_home(&system_home_dir()?))
}

fn droid_settings_dir_for_home(home_dir: &Path) -> PathBuf {
    droidgear_dir_for_home(home_dir).join(DROID_SETTINGS_DIR)
}

fn droid_settings_dir() -> Result<PathBuf, String> {
    Ok(droidgear_dir()?.join(DROID_SETTINGS_DIR))
}

fn global_settings_path_for_home(home_dir: &Path) -> PathBuf {
    home_dir.join(".factory").join("settings.json")
}

/// Resolves the absolute path to the currently active settings file.
/// Returns the global path if no custom file is set, or if the active file doesn't exist.
pub fn get_active_settings_path_for_home(home_dir: &Path) -> Result<PathBuf, String> {
    let active_name = load_active_file_name_for_home(home_dir)?;
    match active_name {
        Some(name) if !name.is_empty() => {
            let custom_path = droid_settings_dir_for_home(home_dir)
                .join(&name)
                .with_extension("json");
            if custom_path.exists() {
                Ok(custom_path)
            } else {
                Ok(global_settings_path_for_home(home_dir))
            }
        }
        _ => Ok(global_settings_path_for_home(home_dir)),
    }
}

/// Resolves the absolute path to the currently active settings file.
/// Returns the global path if no custom file is set, or if the active file doesn't exist.
pub fn get_active_settings_path() -> Result<PathBuf, String> {
    get_active_settings_path_for_home(&system_home_dir()?)
}

// ============================================================================
// Active file tracking
// ============================================================================

fn load_active_file_name_for_home(home_dir: &Path) -> Result<Option<String>, String> {
    let settings_path = paths::get_droidgear_settings_path_for_home(home_dir);
    let settings = paths::read_droidgear_settings_from_path_internal(&settings_path)?;
    Ok(settings
        .get(ACTIVE_FILE_KEY)
        .and_then(|v| v.as_str())
        .map(String::from))
}

fn load_active_file_name() -> Result<Option<String>, String> {
    load_active_file_name_for_home(&system_home_dir()?)
}

fn save_active_file_name_for_home(home_dir: &Path, name: Option<&str>) -> Result<(), String> {
    let settings_path = paths::get_droidgear_settings_path_for_home(home_dir);
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

fn save_active_file_name(name: Option<&str>) -> Result<(), String> {
    save_active_file_name_for_home(&system_home_dir()?, name)
}

// ============================================================================
// Public API
// ============================================================================

/// List all available settings files (global + custom files)
pub fn list_settings_files() -> Result<Vec<SettingsFileInfo>, String> {
    list_settings_files_for_home(&system_home_dir()?)
}

/// List all available settings files (global + custom files) for a specific home directory.
pub fn list_settings_files_for_home(home_dir: &Path) -> Result<Vec<SettingsFileInfo>, String> {
    let active_path = get_active_settings_path_for_home(home_dir)?;
    let global_path = global_settings_path_for_home(home_dir);
    let mut files = Vec::new();

    // Global file
    files.push(SettingsFileInfo {
        name: "Global".to_string(),
        path: global_path.to_string_lossy().to_string(),
        is_global: true,
        is_active: active_path == global_path,
        exists: global_path.exists(),
    });

    // Custom files from ~/.droidgear/droid-settings/
    let dir = droid_settings_dir_for_home(home_dir);
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
                    let is_active = active_path == path;
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
    get_active_settings_file_for_home(&system_home_dir()?)
}

/// Get the currently active settings file info for a specific home directory.
pub fn get_active_settings_file_for_home(home_dir: &Path) -> Result<SettingsFileInfo, String> {
    let files = list_settings_files_for_home(home_dir)?;
    files
        .into_iter()
        .find(|f| f.is_active)
        .ok_or_else(|| "No active settings file found".to_string())
}

/// Resolve a settings file path by display name for a specific home directory.
/// Accepts `Global` (case-insensitive) for the global Factory settings file.
pub fn get_settings_path_by_name_for_home(home_dir: &Path, name: &str) -> Result<PathBuf, String> {
    if name.eq_ignore_ascii_case("global") {
        return Ok(global_settings_path_for_home(home_dir));
    }

    let path = droid_settings_dir_for_home(home_dir)
        .join(name)
        .with_extension("json");
    if path.exists() {
        Ok(path)
    } else {
        Err(format!("Settings file '{name}' does not exist"))
    }
}

/// Set the active settings file.
/// Pass `None` or empty string to switch to Global.
pub fn set_active_settings_file_for_home(
    home_dir: &Path,
    name: Option<String>,
) -> Result<SettingsFileInfo, String> {
    match &name {
        Some(n) if !n.is_empty() => {
            let path = droid_settings_dir_for_home(home_dir)
                .join(n)
                .with_extension("json");
            if !path.exists() {
                return Err(format!("Settings file '{}' does not exist", n));
            }
            save_active_file_name_for_home(home_dir, Some(n))?;
        }
        _ => {
            save_active_file_name_for_home(home_dir, None)?;
        }
    }

    let active_path = get_active_settings_path_for_home(home_dir)?;
    let global_path = global_settings_path_for_home(home_dir);

    Ok(SettingsFileInfo {
        name: if active_path == global_path {
            "Global".to_string()
        } else {
            active_path
                .file_stem()
                .and_then(|value| value.to_str())
                .unwrap_or("unknown")
                .to_string()
        },
        path: active_path.to_string_lossy().to_string(),
        is_global: active_path == global_path,
        is_active: true,
        exists: active_path.exists(),
    })
}

/// Set the active settings file.
/// Pass `None` or empty string to switch to Global.
pub fn set_active_settings_file(name: Option<String>) -> Result<SettingsFileInfo, String> {
    let _ = droid_settings_dir()?;
    set_active_settings_file_for_home(&system_home_dir()?, name)
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
pub fn get_launch_command_for_home(home_dir: &Path) -> Result<(String, String), String> {
    let active_path = get_active_settings_path_for_home(home_dir)?;
    let path_str = active_path.to_string_lossy().to_string();

    let is_global = active_path == global_settings_path_for_home(home_dir);

    let command = if is_global {
        "droid".to_string()
    } else {
        format!("droid --settings \"{path_str}\"")
    };

    Ok((command, path_str))
}

/// Get the launch command for Droid with the active settings file.
/// Returns the command string and the settings path used.
pub fn get_launch_command() -> Result<(String, String), String> {
    get_launch_command_for_home(&system_home_dir()?)
}

#[cfg(test)]
mod tests {
    use super::{
        get_active_settings_file_for_home, get_settings_path_by_name_for_home,
        list_settings_files_for_home, set_active_settings_file_for_home,
    };
    use std::path::Path;
    use tempfile::TempDir;

    fn write_file(path: &Path, contents: &str) {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).unwrap();
        }
        std::fs::write(path, contents).unwrap();
    }

    #[test]
    fn list_settings_files_for_home_uses_the_provided_home_directory() {
        let temp = TempDir::new().unwrap();
        let home_dir = temp.path();

        write_file(&home_dir.join(".factory/settings.json"), "{}");
        write_file(
            &home_dir.join(".droidgear/droid-settings/profile-a.json"),
            "{}",
        );
        set_active_settings_file_for_home(home_dir, Some("profile-a".to_string())).unwrap();

        let files = list_settings_files_for_home(home_dir).unwrap();

        assert_eq!(files.len(), 2);
        assert!(files
            .iter()
            .any(|file| file.name == "Global" && file.is_global));
        assert!(files
            .iter()
            .any(|file| file.name == "profile-a" && file.is_active));

        let active = get_active_settings_file_for_home(home_dir).unwrap();
        assert_eq!(active.name, "profile-a");
        assert!(!active.is_global);
    }

    #[test]
    fn get_settings_path_by_name_for_home_resolves_global_and_custom_files() {
        let temp = TempDir::new().unwrap();
        let home_dir = temp.path();

        let global_path = home_dir.join(".factory/settings.json");
        let custom_path = home_dir.join(".droidgear/droid-settings/profile-b.json");
        write_file(&global_path, "{}");
        write_file(&custom_path, "{}");

        assert_eq!(
            get_settings_path_by_name_for_home(home_dir, "global").unwrap(),
            global_path
        );
        assert_eq!(
            get_settings_path_by_name_for_home(home_dir, "profile-b").unwrap(),
            custom_path
        );
        assert!(get_settings_path_by_name_for_home(home_dir, "missing").is_err());
    }
}
