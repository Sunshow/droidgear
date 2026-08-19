//! Factory Droid trusted-folder management.
//!
//! Trusted folders are stored in the global Factory `settings.json` file. They
//! intentionally do not follow DroidGear's model/settings-file profiles: a
//! folder trust decision is a machine-level permission, not a model choice.

use chrono::SecondsFormat;
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use specta::Type;
use std::path::{Path, PathBuf};

use crate::{factory_settings, storage};

const TRUSTED_FOLDERS_KEY: &str = "trustedFolders";
const TRUSTED_AT_KEY: &str = "trustedAt";

/// A folder trusted by Factory Droid.
#[derive(Debug, Clone, Serialize, Deserialize, Type, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct TrustedFolder {
    pub path: String,
    pub trusted_at: String,
}

fn config_path_for_home(home_dir: &Path) -> Result<PathBuf, String> {
    factory_settings::get_config_path_for_home(home_dir).map(PathBuf::from)
}

fn read_config(path: &Path) -> Result<Value, String> {
    if !path.exists() {
        return Ok(Value::Object(Map::new()));
    }

    let contents = std::fs::read_to_string(path)
        .map_err(|error| format!("Failed to read Factory settings: {error}"))?;
    if contents.trim().is_empty() {
        return Ok(Value::Object(Map::new()));
    }

    serde_json::from_str(&contents).map_err(|error| {
        format!(
            "{} Failed to parse config JSON: {error}",
            factory_settings::CONFIG_PARSE_ERROR_PREFIX
        )
    })
}

fn write_config(path: &Path, config: &Value) -> Result<(), String> {
    let actual_path = if path.is_symlink() {
        std::fs::canonicalize(path)
            .map_err(|error| format!("Failed to resolve Factory settings symlink: {error}"))?
    } else {
        path.to_path_buf()
    };
    let contents = serde_json::to_vec_pretty(config)
        .map_err(|error| format!("Failed to serialize Factory settings: {error}"))?;
    storage::atomic_write(&actual_path, &contents)
}

fn config_object_mut(config: &mut Value) -> Result<&mut Map<String, Value>, String> {
    config
        .as_object_mut()
        .ok_or_else(|| "Factory settings must contain a JSON object".to_string())
}

fn trusted_folders_object(config: &Value) -> Result<Option<&Map<String, Value>>, String> {
    let Some(value) = config.get(TRUSTED_FOLDERS_KEY) else {
        return Ok(None);
    };
    if value.is_null() {
        return Ok(None);
    }
    value
        .as_object()
        .map(Some)
        .ok_or_else(|| "Factory settings trustedFolders must be a JSON object".to_string())
}

fn entry_trusted_at(value: &Value) -> Result<String, String> {
    // Droid currently writes `{ trustedAt: string }`. Accepting a bare string
    // here keeps the manager compatible with older experimental builds.
    if let Some(timestamp) = value.as_str() {
        return Ok(timestamp.to_string());
    }
    let Some(object) = value.as_object() else {
        return Err("Factory settings trustedFolders contains an invalid entry".to_string());
    };
    Ok(object
        .get(TRUSTED_AT_KEY)
        .and_then(Value::as_str)
        .unwrap_or_default()
        .to_string())
}

fn normalize_input_path(path: &str) -> Result<String, String> {
    let trimmed = path.trim();
    if trimmed.is_empty() {
        return Err("Trusted folder path cannot be empty".to_string());
    }

    let path = Path::new(trimmed);
    if !path.is_absolute() {
        return Err("Trusted folder path must be absolute".to_string());
    }
    let metadata = std::fs::metadata(path)
        .map_err(|error| format!("Trusted folder does not exist: {error}"))?;
    if !metadata.is_dir() {
        return Err("Trusted folder path must point to a directory".to_string());
    }

    Ok(trimmed.to_string())
}

fn now_timestamp() -> String {
    chrono::Utc::now().to_rfc3339_opts(SecondsFormat::Millis, true)
}

/// Lists trusted folders from the global Factory settings file.
pub fn list_trusted_folders_for_home(home_dir: &Path) -> Result<Vec<TrustedFolder>, String> {
    let config_path = config_path_for_home(home_dir)?;
    let config = read_config(&config_path)?;
    let Some(entries) = trusted_folders_object(&config)? else {
        return Ok(Vec::new());
    };

    let mut folders = entries
        .iter()
        .map(|(path, value)| {
            Ok(TrustedFolder {
                path: path.clone(),
                trusted_at: entry_trusted_at(value)?,
            })
        })
        .collect::<Result<Vec<_>, String>>()?;
    folders.sort_by(|left, right| left.path.cmp(&right.path));
    Ok(folders)
}

/// Lists trusted folders using the current user's home directory.
pub fn list_trusted_folders() -> Result<Vec<TrustedFolder>, String> {
    let home_dir = dirs::home_dir().ok_or_else(|| "Failed to get home directory".to_string())?;
    list_trusted_folders_for_home(&home_dir)
}

/// Adds a trusted folder to the global Factory settings file.
pub fn add_trusted_folder_for_home(home_dir: &Path, path: &str) -> Result<TrustedFolder, String> {
    let normalized_path = normalize_input_path(path)?;
    let config_path = config_path_for_home(home_dir)?;
    let mut config = read_config(&config_path)?;
    let object = config_object_mut(&mut config)?;
    let folders = object
        .entry(TRUSTED_FOLDERS_KEY.to_string())
        .or_insert_with(|| Value::Object(Map::new()));
    let folders_object = folders
        .as_object_mut()
        .ok_or_else(|| "Factory settings trustedFolders must be a JSON object".to_string())?;

    let trusted_at = match folders_object.get(&normalized_path) {
        Some(value) => {
            let existing = entry_trusted_at(value)?;
            if existing.is_empty() {
                let timestamp = now_timestamp();
                folders_object.insert(
                    normalized_path.clone(),
                    serde_json::json!({ TRUSTED_AT_KEY: timestamp }),
                );
                timestamp
            } else {
                existing
            }
        }
        None => {
            let timestamp = now_timestamp();
            folders_object.insert(
                normalized_path.clone(),
                serde_json::json!({ TRUSTED_AT_KEY: timestamp }),
            );
            timestamp
        }
    };

    write_config(&config_path, &config)?;
    Ok(TrustedFolder {
        path: normalized_path,
        trusted_at,
    })
}

/// Adds a trusted folder using the current user's home directory.
pub fn add_trusted_folder(path: &str) -> Result<TrustedFolder, String> {
    let home_dir = dirs::home_dir().ok_or_else(|| "Failed to get home directory".to_string())?;
    add_trusted_folder_for_home(&home_dir, path)
}

/// Removes a trusted folder from the global Factory settings file.
pub fn remove_trusted_folder_for_home(home_dir: &Path, path: &str) -> Result<(), String> {
    let paths = [path.to_string()];
    remove_trusted_folders_for_home(home_dir, &paths)
}

/// Removes multiple trusted folders in one read-modify-write operation.
pub fn remove_trusted_folders_for_home(home_dir: &Path, paths: &[String]) -> Result<(), String> {
    let normalized_paths = paths
        .iter()
        .map(|path| {
            let trimmed = path.trim();
            if trimmed.is_empty() {
                return Err("Trusted folder path cannot be empty".to_string());
            }
            Ok(trimmed.to_string())
        })
        .collect::<Result<Vec<_>, String>>()?;
    if normalized_paths.is_empty() {
        return Ok(());
    }

    let config_path = config_path_for_home(home_dir)?;
    let mut config = read_config(&config_path)?;
    let Some(folders) = config
        .as_object_mut()
        .and_then(|object| object.get_mut(TRUSTED_FOLDERS_KEY))
    else {
        return Ok(());
    };
    let Some(folders_object) = folders.as_object_mut() else {
        return Err("Factory settings trustedFolders must be a JSON object".to_string());
    };
    for path in normalized_paths {
        folders_object.remove(&path);
    }
    write_config(&config_path, &config)
}

/// Removes a trusted folder using the current user's home directory.
pub fn remove_trusted_folder(path: &str) -> Result<(), String> {
    let home_dir = dirs::home_dir().ok_or_else(|| "Failed to get home directory".to_string())?;
    remove_trusted_folder_for_home(&home_dir, path)
}

/// Removes multiple trusted folders using the current user's home directory.
pub fn remove_trusted_folders(paths: Vec<String>) -> Result<(), String> {
    let home_dir = dirs::home_dir().ok_or_else(|| "Failed to get home directory".to_string())?;
    remove_trusted_folders_for_home(&home_dir, &paths)
}

#[cfg(test)]
mod tests {
    use super::{
        add_trusted_folder_for_home, list_trusted_folders_for_home, remove_trusted_folder_for_home,
        remove_trusted_folders_for_home,
    };
    use serde_json::Value;
    use std::path::Path;
    use tempfile::TempDir;

    fn config_path(home: &Path) -> std::path::PathBuf {
        home.join(".factory/settings.json")
    }

    fn write_config(home: &Path, contents: &str) {
        let path = config_path(home);
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(path, contents).unwrap();
    }

    #[test]
    fn lists_entries_sorted_and_preserves_timestamps() {
        let temp = TempDir::new().unwrap();
        write_config(
            temp.path(),
            r#"{
              "customModels": [],
              "trustedFolders": {
                "/tmp/z": {"trustedAt": "2026-01-02T00:00:00.000Z"},
                "/tmp/a": {"trustedAt": "2026-01-01T00:00:00.000Z"}
              }
            }"#,
        );

        let folders = list_trusted_folders_for_home(temp.path()).unwrap();
        assert_eq!(folders[0].path, "/tmp/a");
        assert_eq!(folders[0].trusted_at, "2026-01-01T00:00:00.000Z");
        assert_eq!(folders[1].path, "/tmp/z");
    }

    #[test]
    fn add_preserves_other_settings_and_is_idempotent() {
        let temp = TempDir::new().unwrap();
        let folder = temp.path().join("project");
        std::fs::create_dir_all(&folder).unwrap();
        write_config(
            temp.path(),
            r#"{"customModels":[{"id":"keep"}],"trustedFolders":{}}"#,
        );

        let added = add_trusted_folder_for_home(temp.path(), folder.to_str().unwrap()).unwrap();
        let added_again =
            add_trusted_folder_for_home(temp.path(), folder.to_str().unwrap()).unwrap();
        assert_eq!(added, added_again);

        let value: Value =
            serde_json::from_str(&std::fs::read_to_string(config_path(temp.path())).unwrap())
                .unwrap();
        assert_eq!(value["customModels"][0]["id"], "keep");
        assert_eq!(
            value["trustedFolders"][folder.to_str().unwrap()]["trustedAt"],
            added.trusted_at
        );
    }

    #[test]
    fn remove_only_deletes_the_requested_entry() {
        let temp = TempDir::new().unwrap();
        write_config(
            temp.path(),
            r#"{"customModels":[{"id":"keep"}],"trustedFolders":{"/tmp/a":{"trustedAt":"a"},"/tmp/b":{"trustedAt":"b"}}}"#,
        );

        remove_trusted_folder_for_home(temp.path(), "/tmp/a").unwrap();
        let value: Value =
            serde_json::from_str(&std::fs::read_to_string(config_path(temp.path())).unwrap())
                .unwrap();
        assert!(value["trustedFolders"].get("/tmp/a").is_none());
        assert_eq!(value["trustedFolders"]["/tmp/b"]["trustedAt"], "b");
        assert_eq!(value["customModels"][0]["id"], "keep");
    }

    #[test]
    fn remove_many_deletes_selected_entries_in_one_operation() {
        let temp = TempDir::new().unwrap();
        write_config(
            temp.path(),
            r#"{"trustedFolders":{"/tmp/a":{"trustedAt":"a"},"/tmp/b":{"trustedAt":"b"},"/tmp/c":{"trustedAt":"c"}}}"#,
        );

        remove_trusted_folders_for_home(
            temp.path(),
            &[" /tmp/a ".to_string(), "/tmp/c".to_string()],
        )
        .unwrap();

        let value: Value =
            serde_json::from_str(&std::fs::read_to_string(config_path(temp.path())).unwrap())
                .unwrap();
        assert!(value["trustedFolders"].get("/tmp/a").is_none());
        assert!(value["trustedFolders"].get("/tmp/c").is_none());
        assert_eq!(value["trustedFolders"]["/tmp/b"]["trustedAt"], "b");
    }

    #[test]
    fn add_rejects_relative_and_non_directory_paths() {
        let temp = TempDir::new().unwrap();
        write_config(temp.path(), "{}");
        assert!(add_trusted_folder_for_home(temp.path(), "relative").is_err());
        let file = temp.path().join("file");
        std::fs::write(&file, "").unwrap();
        assert!(add_trusted_folder_for_home(temp.path(), file.to_str().unwrap()).is_err());
    }
}
