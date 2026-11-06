//! Startup model selection for Pi (`~/.pi/agent/settings.json`).
//!
//! pi keeps two documents in its agent directory: `models.json` (providers and
//! their models — what a DroidGear profile carries) and `settings.json` (user
//! settings, including which model to start with). A profile only owns
//! `models.json`, so the startup model lives outside profiles and is written
//! straight into the live `settings.json`, exactly like pi reads it.
//!
//! Upstream reference (pi docs/settings.md § Model and thinking):
//! `defaultProvider`, `defaultModel`, `defaultThinkingLevel`.
//!
//! Every other root key (`retry`, `theme`, `compaction`, `$schema` …) is
//! preserved verbatim: saves are a read-modify-write over the raw JSON object.

use serde::{Deserialize, Serialize};
use specta::Type;
use std::path::{Path, PathBuf};

use crate::pi::pi_config_dir_for_home;
use crate::storage;

/// The upstream `defaultThinkingLevel` enum, in display order.
pub const THINKING_LEVELS: [&str; 7] = ["off", "minimal", "low", "medium", "high", "xhigh", "max"];

const SETTING_KEYS: [&str; 3] = ["defaultProvider", "defaultModel", "defaultThinkingLevel"];

/// The startup model selection DroidGear edits.
///
/// An empty field means "let pi pick automatically" and is written as a
/// removed key, because upstream validates both `""` and `null`.
#[derive(Debug, Clone, Default, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct PiSettings {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default_provider: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default_model: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default_thinking_level: Option<String>,
}

/// [`PiSettings`] plus the document's location and state, so the GUI and the
/// TUI render one shape.
#[derive(Debug, Clone, Default, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct PiSettingsStatus {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default_provider: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default_model: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default_thinking_level: Option<String>,
    pub settings_exists: bool,
    pub settings_path: String,
}

impl PiSettingsStatus {
    /// The editable selection, without the file metadata.
    pub fn settings(&self) -> PiSettings {
        PiSettings {
            default_provider: self.default_provider.clone(),
            default_model: self.default_model.clone(),
            default_thinking_level: self.default_thinking_level.clone(),
        }
    }

    /// Build a status from a selection plus file metadata.
    pub fn from_settings(settings: &PiSettings, exists: bool, path: &Path) -> Self {
        Self {
            default_provider: settings.default_provider.clone(),
            default_model: settings.default_model.clone(),
            default_thinking_level: settings.default_thinking_level.clone(),
            settings_exists: exists,
            settings_path: path.to_string_lossy().to_string(),
        }
    }
}

/// `~/.pi/agent/settings.json`
pub fn pi_settings_path_for_home(home_dir: &Path) -> Result<PathBuf, String> {
    Ok(pi_config_dir_for_home(home_dir)?.join("settings.json"))
}

/// Trim a value and treat an empty string as unset.
pub fn trim_setting(value: Option<&str>) -> Option<String> {
    let value = value?.trim();
    if value.is_empty() {
        None
    } else {
        Some(value.to_string())
    }
}

/// Normalize a thinking level: trim, treat `""` as unset, and reject values
/// outside the upstream enum (a typo here silently disables the setting, which
/// is worse than failing the write).
pub fn normalize_thinking_level(value: Option<&str>) -> Result<Option<String>, String> {
    let Some(trimmed) = trim_setting(value) else {
        return Ok(None);
    };
    if THINKING_LEVELS.contains(&trimmed.as_str()) {
        Ok(Some(trimmed))
    } else {
        Err(format!(
            "Invalid Pi thinking level \"{trimmed}\" (want {})",
            THINKING_LEVELS.join(" | ")
        ))
    }
}

/// Parse `~/.pi/agent/settings.json` (a missing file is not an error — it means
/// "nothing pinned yet").
pub fn read_pi_settings_for_home(home_dir: &Path) -> Result<PiSettingsStatus, String> {
    let path = pi_settings_path_for_home(home_dir)?;
    match std::fs::read_to_string(&path) {
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(PiSettingsStatus::from_settings(
            &PiSettings::default(),
            false,
            &path,
        )),
        Err(e) => Err(format!("Failed to read Pi settings file: {e}")),
        Ok(text) => {
            let settings: PiSettings = serde_json::from_str(&text)
                .map_err(|e| format!("Invalid Pi settings JSON: {e}"))?;
            Ok(PiSettingsStatus::from_settings(&settings, true, &path))
        }
    }
}

/// Write the startup model selection into the live settings.json, keeping every
/// other root key. The file is re-serialized, so key order and formatting are
/// normalized the same way pi's own writes are.
pub fn save_pi_settings_for_home(home_dir: &Path, settings: &PiSettings) -> Result<(), String> {
    let thinking = normalize_thinking_level(settings.default_thinking_level.as_deref())?;
    let path = pi_settings_path_for_home(home_dir)?;

    let mut root: serde_json::Map<String, serde_json::Value> = match std::fs::read_to_string(&path)
    {
        Ok(text) if text.trim().is_empty() => serde_json::Map::new(),
        Ok(text) => {
            serde_json::from_str(&text).map_err(|e| format!("Invalid Pi settings JSON: {e}"))?
        }
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => serde_json::Map::new(),
        Err(e) => return Err(format!("Failed to read Pi settings file: {e}")),
    };

    let values: [(&str, Option<String>); 3] = [
        (
            SETTING_KEYS[0],
            trim_setting(settings.default_provider.as_deref()),
        ),
        (
            SETTING_KEYS[1],
            trim_setting(settings.default_model.as_deref()),
        ),
        (SETTING_KEYS[2], thinking),
    ];
    for (key, value) in values {
        match value {
            // Unset = pi picks automatically, so the key is removed rather than
            // written as "" or null (upstream validates both).
            None => {
                root.remove(key);
            }
            Some(value) => {
                root.insert(key.to_string(), serde_json::Value::String(value));
            }
        }
    }

    let mut out = serde_json::to_string_pretty(&serde_json::Value::Object(root))
        .map_err(|e| format!("Failed to serialize Pi settings: {e}"))?;
    out.push('\n');
    storage::atomic_write(&path, out.as_bytes())
}

/// Remove the startup model selection while keeping the rest of settings.json
/// intact.
pub fn clear_pi_settings_for_home(home_dir: &Path) -> Result<(), String> {
    save_pi_settings_for_home(home_dir, &PiSettings::default())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_home(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!(
            "droidgear-pi-settings-{name}-{}",
            uuid::Uuid::new_v4()
        ));
        std::fs::create_dir_all(&dir).expect("temp home");
        dir
    }

    #[test]
    fn read_reports_missing_file_without_error() {
        let home = temp_home("missing");
        let status = read_pi_settings_for_home(&home).expect("read");
        assert!(!status.settings_exists);
        assert!(status.default_provider.is_none());
    }

    #[test]
    fn save_keeps_foreign_keys_and_removes_cleared_ones() {
        let home = temp_home("save");
        let path = pi_settings_path_for_home(&home).expect("path");
        std::fs::write(
            &path,
            r#"{"theme":"dark","retry":{"enabled":false},"defaultProvider":"stale"}"#,
        )
        .expect("seed");

        save_pi_settings_for_home(
            &home,
            &PiSettings {
                default_provider: Some("acme".to_string()),
                default_model: Some("m1".to_string()),
                default_thinking_level: Some("high".to_string()),
            },
        )
        .expect("save");

        let text = std::fs::read_to_string(&path).expect("read");
        let value: serde_json::Value = serde_json::from_str(&text).expect("json");
        assert_eq!(value["defaultProvider"], "acme");
        assert_eq!(value["defaultModel"], "m1");
        assert_eq!(value["defaultThinkingLevel"], "high");
        assert_eq!(value["theme"], "dark");
        assert_eq!(value["retry"]["enabled"], false);

        clear_pi_settings_for_home(&home).expect("clear");
        let text = std::fs::read_to_string(&path).expect("read");
        let value: serde_json::Value = serde_json::from_str(&text).expect("json");
        assert!(value.get("defaultProvider").is_none());
        assert!(value.get("defaultModel").is_none());
        assert!(value.get("defaultThinkingLevel").is_none());
        assert_eq!(value["theme"], "dark");
    }

    #[test]
    fn save_rejects_unknown_thinking_level() {
        let home = temp_home("level");
        let err = save_pi_settings_for_home(
            &home,
            &PiSettings {
                default_thinking_level: Some("turbo".to_string()),
                ..Default::default()
            },
        )
        .expect_err("invalid level");
        assert!(err.contains("Invalid Pi thinking level"));
    }
}
