//! Droid settings file management (core).
//!
//! Manages multiple Droid settings files stored in `~/.droidgear/droid-settings/`.
//! Tracks the active settings file and provides path resolution for read/write operations.
//!
//! Local JSON files outside that directory can be *linked* by reference: the
//! file stays at its original location and is passed to `droid --settings`
//! as-is. Links are tracked in `~/.droidgear/settings.json` under
//! `droidSettingsExternalFiles`.

use serde::{Deserialize, Serialize};
use specta::Type;
use std::path::{Path, PathBuf};

use crate::paths;

const DROID_SETTINGS_DIR: &str = "droid-settings";
const ACTIVE_FILE_KEY: &str = "droidSettingsActiveFile";
const EXTERNAL_FILES_KEY: &str = "droidSettingsExternalFiles";

// ============================================================================
// Types
// ============================================================================

/// A local JSON settings file linked by reference (never copied). The file
/// stays at its original location and is passed to `droid --settings` as-is.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExternalSettingsFile {
    /// Display name (the source filename without extension).
    pub name: String,
    /// Absolute path to the linked file.
    pub path: String,
}

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
    /// Whether this is a linked external file kept at its original location
    pub is_external: bool,
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

pub fn global_settings_path_for_home(home_dir: &Path) -> PathBuf {
    home_dir.join(".factory").join("settings.json")
}

/// Resolves the absolute path to the currently active settings file.
/// Returns the global path if no custom file is set, or if the active file doesn't exist.
pub fn get_active_settings_path_for_home(home_dir: &Path) -> Result<PathBuf, String> {
    let active_name = load_active_file_name_for_home(home_dir)?;
    match active_name {
        Some(name) if !name.is_empty() => {
            // An absolute path is a linked external file kept at its original
            // location; a bare name is a managed file under droid-settings/.
            let active_path = PathBuf::from(&name);
            if active_path.is_absolute() {
                if active_path.exists() {
                    return Ok(active_path);
                }
                return Ok(global_settings_path_for_home(home_dir));
            }

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
// External link registry
// ============================================================================

fn load_external_files_for_home(home_dir: &Path) -> Result<Vec<ExternalSettingsFile>, String> {
    let settings_path = paths::get_droidgear_settings_path_for_home(home_dir);
    let settings = paths::read_droidgear_settings_from_path_internal(&settings_path)?;
    let value = settings
        .get(EXTERNAL_FILES_KEY)
        .cloned()
        .unwrap_or_else(|| serde_json::json!([]));
    serde_json::from_value(value)
        .map_err(|e| format!("Failed to parse external settings file registry: {e}"))
}

fn save_external_files_for_home(
    home_dir: &Path,
    files: &[ExternalSettingsFile],
) -> Result<(), String> {
    let settings_path = paths::get_droidgear_settings_path_for_home(home_dir);
    let mut settings = paths::read_droidgear_settings_from_path_internal(&settings_path)?;

    let value = serde_json::to_value(files)
        .map_err(|e| format!("Failed to serialize external settings file registry: {e}"))?;
    if let Some(obj) = settings.as_object_mut() {
        obj.insert(EXTERNAL_FILES_KEY.to_string(), value);
    }

    paths::write_droidgear_settings_to_path_internal(&settings_path, &settings)?;
    Ok(())
}

fn is_external_path_linked_for_home(home_dir: &Path, path: &Path) -> Result<bool, String> {
    Ok(load_external_files_for_home(home_dir)?
        .iter()
        .any(|f| Path::new(&f.path) == path))
}

fn display_name_in_use_for_home(home_dir: &Path, name: &str) -> Result<bool, String> {
    if name.eq_ignore_ascii_case("global") {
        return Ok(true);
    }
    if load_external_files_for_home(home_dir)?
        .iter()
        .any(|f| f.name == name)
    {
        return Ok(true);
    }
    let managed_path = droid_settings_dir_for_home(home_dir)
        .join(name)
        .with_extension("json");
    Ok(managed_path.exists())
}

/// Picks a free display name for a linked file based on the source file stem.
fn pick_link_display_name_for_home(home_dir: &Path, base: &str) -> Result<String, String> {
    if !display_name_in_use_for_home(home_dir, base)? {
        return Ok(base.to_string());
    }
    for i in 2..1000 {
        let candidate = format!("{base}-{i}");
        if !display_name_in_use_for_home(home_dir, &candidate)? {
            return Ok(candidate);
        }
    }
    Err(format!("Could not find a free display name for '{base}'"))
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
        is_external: false,
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
                        is_external: false,
                    });
                }
            }
        }
    }

    // Linked external files (kept at their original locations)
    for external in load_external_files_for_home(home_dir)? {
        let path = PathBuf::from(&external.path);
        files.push(SettingsFileInfo {
            name: external.name.clone(),
            path: external.path.clone(),
            is_global: false,
            is_active: active_path == path,
            exists: path.exists(),
            is_external: true,
        });
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
/// Pass `None` or empty string to switch to Global. A bare name selects a
/// managed file under `~/.droidgear/droid-settings/`; an absolute path selects
/// a linked external file.
pub fn set_active_settings_file_for_home(
    home_dir: &Path,
    name: Option<String>,
) -> Result<SettingsFileInfo, String> {
    match &name {
        Some(n) if !n.is_empty() => {
            let candidate = PathBuf::from(n);
            if candidate.is_absolute() {
                if !is_external_path_linked_for_home(home_dir, &candidate)? {
                    return Err(format!(
                        "Settings file '{}' is not a linked settings file",
                        n
                    ));
                }
                save_active_file_name_for_home(home_dir, Some(n))?;
            } else {
                let path = droid_settings_dir_for_home(home_dir)
                    .join(n)
                    .with_extension("json");
                if !path.exists() {
                    return Err(format!("Settings file '{n}' does not exist"));
                }
                save_active_file_name_for_home(home_dir, Some(n))?;
            }
        }
        _ => {
            save_active_file_name_for_home(home_dir, None)?;
        }
    }

    let active_path = get_active_settings_path_for_home(home_dir)?;
    let global_path = global_settings_path_for_home(home_dir);
    let is_global = active_path == global_path;

    Ok(SettingsFileInfo {
        name: if is_global {
            "Global".to_string()
        } else {
            active_path
                .file_stem()
                .and_then(|value| value.to_str())
                .unwrap_or("unknown")
                .to_string()
        },
        path: active_path.to_string_lossy().to_string(),
        is_global,
        is_active: true,
        exists: active_path.exists(),
        is_external: is_external_path_linked_for_home(home_dir, &active_path)?,
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

// ============================================================================
// Linking local JSON files
// ============================================================================

/// Links a local JSON file as a settings profile by reference (never copied).
///
/// The file must exist, end with `.json`, and parse as JSON. The link is
/// registered under `~/.droidgear/settings.json` and the file becomes the
/// active settings file, so subsequent launches pass it via `--settings`.
pub fn link_settings_file_for_home(
    home_dir: &Path,
    source_path: &str,
) -> Result<SettingsFileInfo, String> {
    let source = PathBuf::from(source_path).canonicalize().map_err(|e| {
        format!(
            "Settings file '{}' does not exist or cannot be read: {e}",
            source_path
        )
    })?;

    if source.extension().and_then(|s| s.to_str()) != Some("json") {
        return Err("Settings file must have a .json extension".to_string());
    }

    let content = std::fs::read_to_string(&source)
        .map_err(|e| format!("Failed to read settings file '{}': {e}", source.display()))?;
    serde_json::from_str::<serde_json::Value>(&content).map_err(|e| {
        format!(
            "Settings file '{}' is not valid JSON: {e}",
            source.display()
        )
    })?;

    let source_str = source.to_string_lossy().to_string();
    let mut externals = load_external_files_for_home(home_dir)?;

    // Re-linking the same file just activates it.
    if let Some(existing) = externals.iter().find(|f| f.path == source_str) {
        save_active_file_name_for_home(home_dir, Some(&existing.path))?;
        return get_active_settings_file_for_home(home_dir);
    }

    let stem = source
        .file_stem()
        .and_then(|s| s.to_str())
        .filter(|s| !s.is_empty())
        .unwrap_or("settings")
        .to_string();
    let name = pick_link_display_name_for_home(home_dir, &stem)?;

    externals.push(ExternalSettingsFile {
        name,
        path: source_str.clone(),
    });
    save_external_files_for_home(home_dir, &externals)?;
    save_active_file_name_for_home(home_dir, Some(&source_str))?;

    get_active_settings_file_for_home(home_dir)
}

/// Links a local JSON file as a settings profile using the system home
/// directory.
pub fn link_settings_file(source_path: &str) -> Result<SettingsFileInfo, String> {
    link_settings_file_for_home(&system_home_dir()?, source_path)
}

/// Removes the registration of a linked external settings file. The file on
/// disk is never touched. If the unlinked file was active, the active settings
/// file resets to Global.
pub fn unlink_settings_file_for_home(home_dir: &Path, path: &str) -> Result<(), String> {
    let mut externals = load_external_files_for_home(home_dir)?;
    let target = PathBuf::from(path);
    let canonical = target.canonicalize().ok();

    let index = externals.iter().position(|f| {
        Path::new(&f.path) == target.as_path()
            || canonical
                .as_deref()
                .is_some_and(|c| c == Path::new(&f.path))
    });
    let Some(index) = index else {
        return Err(format!("Settings file '{path}' is not linked"));
    };

    let removed = externals.remove(index);
    save_external_files_for_home(home_dir, &externals)?;

    if load_active_file_name_for_home(home_dir)?.as_deref() == Some(removed.path.as_str()) {
        save_active_file_name_for_home(home_dir, None)?;
    }

    Ok(())
}

/// Removes the registration of a linked external settings file using the
/// system home directory.
pub fn unlink_settings_file(path: &str) -> Result<(), String> {
    unlink_settings_file_for_home(&system_home_dir()?, path)
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
        get_active_settings_file_for_home, get_launch_command_for_home,
        get_settings_path_by_name_for_home, link_settings_file_for_home,
        list_settings_files_for_home, set_active_settings_file_for_home,
        unlink_settings_file_for_home,
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

    #[test]
    fn link_settings_file_registers_external_path_and_activates_it() {
        let temp = TempDir::new().unwrap();
        let home_dir = temp.path();
        let external = temp.path().join("team-settings.json");
        write_file(&external, r#"{"sessionDefaultSettings":{"model":"x"}}"#);

        let info = link_settings_file_for_home(home_dir, &external.to_string_lossy()).unwrap();

        assert_eq!(info.name, "team-settings");
        assert!(info.is_external);
        assert!(info.is_active);
        assert_eq!(
            info.path,
            external.canonicalize().unwrap().to_string_lossy()
        );

        // Listed alongside managed files
        let files = list_settings_files_for_home(home_dir).unwrap();
        assert!(files.iter().any(|f| f.is_external && f.is_active));

        // The original file is untouched (not copied, not moved)
        assert!(external.exists());
        assert!(!home_dir
            .join(".droidgear/droid-settings/team-settings.json")
            .exists());

        // The launch command passes the external path natively
        let (command, path) = get_launch_command_for_home(home_dir).unwrap();
        assert_eq!(path, info.path);
        assert!(command.contains("--settings"), "got: {command}");
    }

    #[test]
    fn link_settings_file_keeps_source_untouched_and_is_idempotent() {
        let temp = TempDir::new().unwrap();
        let home_dir = temp.path();
        let external = temp.path().join("team-settings.json");
        write_file(&external, r#"{"customModels":[]}"#);
        let before = std::fs::read_to_string(&external).unwrap();

        let first = link_settings_file_for_home(home_dir, &external.to_string_lossy()).unwrap();
        let second = link_settings_file_for_home(home_dir, &external.to_string_lossy()).unwrap();

        assert_eq!(first.path, second.path);
        assert_eq!(second.name, "team-settings");
        assert_eq!(std::fs::read_to_string(&external).unwrap(), before);
        let files = list_settings_files_for_home(home_dir).unwrap();
        assert_eq!(files.iter().filter(|f| f.is_external).count(), 1);
    }

    #[test]
    fn link_settings_file_uniquifies_display_name_on_collision() {
        let temp = TempDir::new().unwrap();
        let home_dir = temp.path();
        let external = temp.path().join("team-settings.json");
        write_file(&external, "{}");
        write_file(
            &home_dir.join(".droidgear/droid-settings/team-settings.json"),
            "{}",
        );

        let info = link_settings_file_for_home(home_dir, &external.to_string_lossy()).unwrap();

        assert_eq!(info.name, "team-settings-2");
    }

    #[test]
    fn link_settings_file_rejects_missing_non_json_and_invalid_files() {
        let temp = TempDir::new().unwrap();
        let home_dir = temp.path();

        let err = link_settings_file_for_home(home_dir, "/definitely/missing.json").unwrap_err();
        assert!(err.contains("does not exist"));

        let txt = temp.path().join("settings.txt");
        write_file(&txt, "{}");
        let err = link_settings_file_for_home(home_dir, &txt.to_string_lossy()).unwrap_err();
        assert!(err.contains(".json extension"));

        let broken = temp.path().join("broken.json");
        write_file(&broken, "{not json");
        let err = link_settings_file_for_home(home_dir, &broken.to_string_lossy()).unwrap_err();
        assert!(err.contains("not valid JSON"));
    }

    #[test]
    fn unlink_settings_file_removes_registration_without_deleting_source() {
        let temp = TempDir::new().unwrap();
        let home_dir = temp.path();
        let external = temp.path().join("team-settings.json");
        write_file(&external, "{}");
        let info = link_settings_file_for_home(home_dir, &external.to_string_lossy()).unwrap();

        unlink_settings_file_for_home(home_dir, &info.path).unwrap();

        assert!(external.exists(), "the source file must never be deleted");
        let files = list_settings_files_for_home(home_dir).unwrap();
        assert!(!files.iter().any(|f| f.is_external));
        // Active resets to Global after unlinking the active file
        let active = get_active_settings_file_for_home(home_dir).unwrap();
        assert!(active.is_global);
    }

    #[test]
    fn unlink_settings_file_rejects_unknown_paths() {
        let temp = TempDir::new().unwrap();
        let home_dir = temp.path();

        let err = unlink_settings_file_for_home(home_dir, "/never/linked.json").unwrap_err();
        assert!(err.contains("not linked"));
    }

    #[test]
    fn set_active_accepts_linked_external_paths_only() {
        let temp = TempDir::new().unwrap();
        let home_dir = temp.path();
        let external = temp.path().join("team-settings.json");
        write_file(&external, "{}");
        let info = link_settings_file_for_home(home_dir, &external.to_string_lossy()).unwrap();

        // Switch away and back via absolute path
        set_active_settings_file_for_home(home_dir, None).unwrap();
        let active = set_active_settings_file_for_home(home_dir, Some(info.path.clone())).unwrap();
        assert!(active.is_external);
        assert!(active.is_active);

        // An arbitrary absolute path is rejected
        let other = temp.path().join("other.json");
        write_file(&other, "{}");
        let err =
            set_active_settings_file_for_home(home_dir, Some(other.to_string_lossy().to_string()))
                .unwrap_err();
        assert!(err.contains("not a linked settings file"));
    }

    #[test]
    fn active_path_falls_back_to_global_when_linked_file_vanishes() {
        let temp = TempDir::new().unwrap();
        let home_dir = temp.path();
        let external = temp.path().join("team-settings.json");
        write_file(&external, "{}");
        link_settings_file_for_home(home_dir, &external.to_string_lossy()).unwrap();
        std::fs::remove_file(&external).unwrap();

        let active = get_active_settings_file_for_home(home_dir).unwrap();
        assert!(active.is_global);
    }
}
