//! Configuration paths management.
//!
//! Provides centralized path management for Droid/Factory, OpenCode, Codex, and OpenClaw configurations.
//! Supports custom path overrides stored in ~/.droidgear/settings.json.

use serde::{Deserialize, Serialize};
use serde_json::Value;
use specta::Type;
use std::path::PathBuf;

// ============================================================================
// Types
// ============================================================================

/// User-defined configuration paths (only stores explicitly set paths)
#[derive(Debug, Clone, Default, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct ConfigPaths {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub factory: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub opencode: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub opencode_auth: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub codex: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub openclaw: Option<String>,
}

/// Effective path info with default indicator
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct EffectivePath {
    pub key: String,
    pub path: String,
    pub is_default: bool,
}

/// All effective paths
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct EffectivePaths {
    pub factory: EffectivePath,
    pub opencode: EffectivePath,
    pub opencode_auth: EffectivePath,
    pub codex: EffectivePath,
    pub openclaw: EffectivePath,
}

// ============================================================================
// Path Constants
// ============================================================================

const SETTINGS_FILE: &str = "settings.json";

// ============================================================================
// Internal Helpers
// ============================================================================

fn get_home_dir() -> Result<PathBuf, String> {
    dirs::home_dir().ok_or_else(|| "Failed to get home directory".to_string())
}

fn get_droidgear_dir() -> Result<PathBuf, String> {
    let home = get_home_dir()?;
    let dir = home.join(".droidgear");
    if !dir.exists() {
        std::fs::create_dir_all(&dir)
            .map_err(|e| format!("Failed to create .droidgear directory: {e}"))?;
    }
    Ok(dir)
}

fn get_droidgear_settings_path() -> Result<PathBuf, String> {
    Ok(get_droidgear_dir()?.join(SETTINGS_FILE))
}

fn read_droidgear_settings() -> Result<Value, String> {
    let path = get_droidgear_settings_path()?;
    if !path.exists() {
        return Ok(serde_json::json!({}));
    }
    let content =
        std::fs::read_to_string(&path).map_err(|e| format!("Failed to read settings: {e}"))?;
    if content.trim().is_empty() {
        return Ok(serde_json::json!({}));
    }
    serde_json::from_str(&content).map_err(|e| format!("Failed to parse settings: {e}"))
}

fn write_droidgear_settings(settings: &Value) -> Result<(), String> {
    let path = get_droidgear_settings_path()?;
    let temp_path = path.with_extension("tmp");
    let content = serde_json::to_string_pretty(settings)
        .map_err(|e| format!("Failed to serialize settings: {e}"))?;
    std::fs::write(&temp_path, content).map_err(|e| format!("Failed to write settings: {e}"))?;
    std::fs::rename(&temp_path, &path).map_err(|e| {
        let _ = std::fs::remove_file(&temp_path);
        format!("Failed to finalize settings: {e}")
    })?;
    Ok(())
}

fn load_config_paths_internal() -> ConfigPaths {
    match read_droidgear_settings() {
        Ok(settings) => settings
            .get("configPaths")
            .and_then(|v| serde_json::from_value(v.clone()).ok())
            .unwrap_or_default(),
        Err(_) => ConfigPaths::default(),
    }
}

// ============================================================================
// Default Path Getters
// ============================================================================

fn default_factory_home() -> Result<PathBuf, String> {
    Ok(get_home_dir()?.join(".factory"))
}

fn default_opencode_config_dir() -> Result<PathBuf, String> {
    Ok(get_home_dir()?.join(".config").join("opencode"))
}

fn default_opencode_auth_dir() -> Result<PathBuf, String> {
    Ok(get_home_dir()?
        .join(".local")
        .join("share")
        .join("opencode"))
}

fn default_codex_home() -> Result<PathBuf, String> {
    Ok(get_home_dir()?.join(".codex"))
}

fn default_openclaw_home() -> Result<PathBuf, String> {
    Ok(get_home_dir()?.join(".openclaw"))
}

// ============================================================================
// Public Path Getters (used by other modules)
// ============================================================================

/// Gets the Factory home directory (~/.factory or custom path)
pub fn get_factory_home() -> Result<PathBuf, String> {
    let config = load_config_paths_internal();
    match config.factory {
        Some(custom) => Ok(PathBuf::from(custom)),
        None => default_factory_home(),
    }
}

/// Gets the OpenCode config directory (~/.config/opencode or custom path)
pub fn get_opencode_config_dir() -> Result<PathBuf, String> {
    let config = load_config_paths_internal();
    match config.opencode {
        Some(custom) => Ok(PathBuf::from(custom)),
        None => default_opencode_config_dir(),
    }
}

/// Gets the OpenCode auth directory (~/.local/share/opencode or custom path)
pub fn get_opencode_auth_dir() -> Result<PathBuf, String> {
    let config = load_config_paths_internal();
    match config.opencode_auth {
        Some(custom) => Ok(PathBuf::from(custom)),
        None => default_opencode_auth_dir(),
    }
}

/// Gets the Codex home directory (~/.codex or custom path)
pub fn get_codex_home() -> Result<PathBuf, String> {
    let config = load_config_paths_internal();
    match config.codex {
        Some(custom) => Ok(PathBuf::from(custom)),
        None => default_codex_home(),
    }
}

/// Gets the OpenClaw home directory (~/.openclaw or custom path)
pub fn get_openclaw_home() -> Result<PathBuf, String> {
    let config = load_config_paths_internal();
    match config.openclaw {
        Some(custom) => Ok(PathBuf::from(custom)),
        None => default_openclaw_home(),
    }
}

// ============================================================================
// Tauri Commands
// ============================================================================

/// Gets the current configuration paths (custom values only)
#[tauri::command]
#[specta::specta]
pub async fn get_config_paths() -> Result<ConfigPaths, String> {
    Ok(load_config_paths_internal())
}

/// Gets all effective paths with default indicators
#[tauri::command]
#[specta::specta]
pub async fn get_effective_paths() -> Result<EffectivePaths, String> {
    let config = load_config_paths_internal();

    let factory_path = get_factory_home()?;
    let opencode_path = get_opencode_config_dir()?;
    let opencode_auth_path = get_opencode_auth_dir()?;
    let codex_path = get_codex_home()?;
    let openclaw_path = get_openclaw_home()?;

    Ok(EffectivePaths {
        factory: EffectivePath {
            key: "factory".to_string(),
            path: factory_path.to_string_lossy().to_string(),
            is_default: config.factory.is_none(),
        },
        opencode: EffectivePath {
            key: "opencode".to_string(),
            path: opencode_path.to_string_lossy().to_string(),
            is_default: config.opencode.is_none(),
        },
        opencode_auth: EffectivePath {
            key: "opencodeAuth".to_string(),
            path: opencode_auth_path.to_string_lossy().to_string(),
            is_default: config.opencode_auth.is_none(),
        },
        codex: EffectivePath {
            key: "codex".to_string(),
            path: codex_path.to_string_lossy().to_string(),
            is_default: config.codex.is_none(),
        },
        openclaw: EffectivePath {
            key: "openclaw".to_string(),
            path: openclaw_path.to_string_lossy().to_string(),
            is_default: config.openclaw.is_none(),
        },
    })
}

/// Saves a single configuration path
#[tauri::command]
#[specta::specta]
pub async fn save_config_path(key: String, path: String) -> Result<(), String> {
    log::info!("Setting config path: {} = {}", key, path);

    // Validate path is not empty
    if path.trim().is_empty() {
        return Err("Path cannot be empty".to_string());
    }

    let mut settings = read_droidgear_settings()?;
    let config_paths = settings
        .as_object_mut()
        .ok_or("Invalid settings format")?
        .entry("configPaths")
        .or_insert_with(|| serde_json::json!({}));

    let obj = config_paths
        .as_object_mut()
        .ok_or("Invalid configPaths format")?;

    // Map camelCase key to snake_case for internal storage
    let storage_key = match key.as_str() {
        "factory" => "factory",
        "opencode" => "opencode",
        "opencodeAuth" => "opencodeAuth",
        "codex" => "codex",
        "openclaw" => "openclaw",
        _ => return Err(format!("Unknown config path key: {key}")),
    };

    obj.insert(storage_key.to_string(), serde_json::json!(path));
    write_droidgear_settings(&settings)?;

    log::info!("Config path saved: {} = {}", key, path);
    Ok(())
}

/// Resets a single configuration path to default
#[tauri::command]
#[specta::specta]
pub async fn reset_config_path(key: String) -> Result<(), String> {
    log::info!("Resetting config path: {}", key);

    let mut settings = read_droidgear_settings()?;

    if let Some(obj) = settings.as_object_mut() {
        if let Some(config_paths) = obj.get_mut("configPaths") {
            if let Some(paths_obj) = config_paths.as_object_mut() {
                // Map camelCase key
                let storage_key = match key.as_str() {
                    "factory" => "factory",
                    "opencode" => "opencode",
                    "opencodeAuth" => "opencodeAuth",
                    "codex" => "codex",
                    "openclaw" => "openclaw",
                    _ => return Err(format!("Unknown config path key: {key}")),
                };
                paths_obj.remove(storage_key);

                // Remove configPaths if empty
                if paths_obj.is_empty() {
                    obj.remove("configPaths");
                }
            }
        }
    }

    write_droidgear_settings(&settings)?;
    log::info!("Config path reset: {}", key);
    Ok(())
}

/// Gets the default paths (for UI display)
#[tauri::command]
#[specta::specta]
pub async fn get_default_paths() -> Result<EffectivePaths, String> {
    Ok(EffectivePaths {
        factory: EffectivePath {
            key: "factory".to_string(),
            path: default_factory_home()?.to_string_lossy().to_string(),
            is_default: true,
        },
        opencode: EffectivePath {
            key: "opencode".to_string(),
            path: default_opencode_config_dir()?.to_string_lossy().to_string(),
            is_default: true,
        },
        opencode_auth: EffectivePath {
            key: "opencodeAuth".to_string(),
            path: default_opencode_auth_dir()?.to_string_lossy().to_string(),
            is_default: true,
        },
        codex: EffectivePath {
            key: "codex".to_string(),
            path: default_codex_home()?.to_string_lossy().to_string(),
            is_default: true,
        },
        openclaw: EffectivePath {
            key: "openclaw".to_string(),
            path: default_openclaw_home()?.to_string_lossy().to_string(),
            is_default: true,
        },
    })
}
