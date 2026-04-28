//! Pi Coding Agent configuration management (core).
//!
//! Provides type definitions and path helpers for Pi's `~/.pi/agent/models.json`
//! configuration. Pi uses a provider-model hierarchy similar to OpenClaw.

use chrono::Utc;
use serde::{Deserialize, Serialize};
use specta::Type;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use uuid::Uuid;

use crate::{paths, storage};

// ============================================================================
// Types
// ============================================================================

/// Pi model cost configuration
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct PiModelCost {
    #[serde(default)]
    pub input: f64,
    #[serde(default)]
    pub output: f64,
    #[serde(default)]
    pub cache_read: f64,
    #[serde(default)]
    pub cache_write: f64,
}

/// Pi compatibility configuration
#[derive(Debug, Clone, Default, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct PiCompatConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub supports_store: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub supports_developer_role: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub supports_reasoning_effort: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reasoning_effort_map: Option<HashMap<String, serde_json::Value>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub supports_usage_in_streaming: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_tokens_field: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub requires_tool_result_name: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub requires_assistant_after_tool_result: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub requires_thinking_as_text: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub requires_reasoning_content_on_assistant_messages: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub thinking_format: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cache_control_format: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub supports_strict_mode: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub supports_long_cache_retention: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub supports_eager_tool_input_streaming: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub open_router_routing: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vercel_gateway_routing: Option<serde_json::Value>,
}

/// Pi model definition
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct PiModel {
    pub id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub api: Option<String>,
    #[serde(default)]
    pub reasoning: bool,
    #[serde(default = "default_input")]
    pub input: Vec<String>,
    #[serde(default = "default_context_window")]
    pub context_window: u32,
    #[serde(default = "default_max_tokens")]
    pub max_tokens: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cost: Option<PiModelCost>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub compat: Option<PiCompatConfig>,
}

/// Pi model override (subset of PiModel fields for overriding built-in models)
#[derive(Debug, Clone, Default, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct PiModelOverride {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub api: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reasoning: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub input: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub context_window: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_tokens: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cost: Option<PiModelCost>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub compat: Option<PiCompatConfig>,
}

/// Pi provider configuration
#[derive(Debug, Clone, Default, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct PiProviderConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub base_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub api: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub api_key: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub headers: Option<HashMap<String, String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub auth_header: Option<bool>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub models: Vec<PiModel>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model_overrides: Option<HashMap<String, PiModelOverride>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub compat: Option<PiCompatConfig>,
}

/// Pi profile (stored in DroidGear)
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct PiProfile {
    pub id: String,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    pub created_at: String,
    pub updated_at: String,
    #[serde(default)]
    pub providers: HashMap<String, PiProviderConfig>,
}

/// Pi config status
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct PiConfigStatus {
    pub config_exists: bool,
    pub config_path: String,
}

/// Current Pi configuration (from `~/.pi/agent/models.json`)
#[derive(Debug, Clone, Default, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct PiCurrentConfig {
    #[serde(default)]
    pub providers: HashMap<String, PiProviderConfig>,
}

// ============================================================================
// Default value functions
// ============================================================================

fn default_input() -> Vec<String> {
    vec!["text".to_string()]
}

fn default_context_window() -> u32 {
    128000
}

fn default_max_tokens() -> u32 {
    16384
}

// ============================================================================
// Path Helpers
// ============================================================================

fn droidgear_pi_dir_for_home(home_dir: &Path) -> PathBuf {
    home_dir.join(".droidgear").join("pi")
}

/// `~/.droidgear/pi/profiles/`
pub fn profiles_dir_for_home(home_dir: &Path) -> Result<PathBuf, String> {
    let dir = droidgear_pi_dir_for_home(home_dir).join("profiles");
    if !dir.exists() {
        std::fs::create_dir_all(&dir)
            .map_err(|e| format!("Failed to create pi profiles directory: {e}"))?;
    }
    Ok(dir)
}

/// `~/.droidgear/pi/active-profile.txt`
pub fn active_profile_path_for_home(home_dir: &Path) -> Result<PathBuf, String> {
    let dir = droidgear_pi_dir_for_home(home_dir);
    if !dir.exists() {
        std::fs::create_dir_all(&dir).map_err(|e| format!("Failed to create pi directory: {e}"))?;
    }
    Ok(dir.join("active-profile.txt"))
}

/// `~/.pi/agent/` (Pi home directory)
pub fn pi_config_dir_for_home(home_dir: &Path) -> Result<PathBuf, String> {
    let config_paths = paths::load_config_paths_for_home(home_dir);
    let dir = paths::get_pi_home_for_home(home_dir, &config_paths)?;
    if !dir.exists() {
        std::fs::create_dir_all(&dir)
            .map_err(|e| format!("Failed to create pi config directory: {e}"))?;
    }
    Ok(dir)
}

/// `~/.pi/agent/models.json`
pub fn pi_config_path_for_home(home_dir: &Path) -> Result<PathBuf, String> {
    Ok(pi_config_dir_for_home(home_dir)?.join("models.json"))
}

fn validate_profile_id(id: &str) -> Result<(), String> {
    let ok = id
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_');
    if ok && !id.is_empty() {
        Ok(())
    } else {
        Err("Invalid profile id".to_string())
    }
}

pub fn profile_path_for_home(home_dir: &Path, id: &str) -> Result<PathBuf, String> {
    validate_profile_id(id)?;
    Ok(profiles_dir_for_home(home_dir)?.join(format!("{id}.json")))
}

// ============================================================================
// System wrappers (use system home dir)
// ============================================================================

fn system_home_dir() -> Result<PathBuf, String> {
    dirs::home_dir().ok_or_else(|| "Failed to get home directory".to_string())
}

pub fn profiles_dir() -> Result<PathBuf, String> {
    profiles_dir_for_home(&system_home_dir()?)
}

pub fn active_profile_path() -> Result<PathBuf, String> {
    active_profile_path_for_home(&system_home_dir()?)
}

pub fn pi_config_dir() -> Result<PathBuf, String> {
    pi_config_dir_for_home(&system_home_dir()?)
}

pub fn pi_config_path() -> Result<PathBuf, String> {
    pi_config_path_for_home(&system_home_dir()?)
}

pub fn profile_path(id: &str) -> Result<PathBuf, String> {
    profile_path_for_home(&system_home_dir()?, id)
}

// ============================================================================
// CRUD Helpers
// ============================================================================

fn now_rfc3339() -> String {
    Utc::now().to_rfc3339()
}

fn read_profile_file(path: &Path) -> Result<PiProfile, String> {
    let s = std::fs::read_to_string(path).map_err(|e| format!("Failed to read profile: {e}"))?;
    serde_json::from_str::<PiProfile>(&s).map_err(|e| format!("Invalid profile JSON: {e}"))
}

fn write_profile_file(home_dir: &Path, profile: &PiProfile) -> Result<(), String> {
    let path = profile_path_for_home(home_dir, &profile.id)?;
    let s = serde_json::to_string_pretty(profile)
        .map_err(|e| format!("Failed to serialize profile JSON: {e}"))?;
    storage::atomic_write(&path, s.as_bytes())
}

fn load_profile_by_id(home_dir: &Path, id: &str) -> Result<PiProfile, String> {
    let path = profile_path_for_home(home_dir, id)?;
    read_profile_file(&path)
}

// ============================================================================
// CRUD (Profiles)
// ============================================================================

pub fn list_pi_profiles_for_home(home_dir: &Path) -> Result<Vec<PiProfile>, String> {
    let dir = profiles_dir_for_home(home_dir)?;
    if !dir.exists() {
        return Ok(vec![]);
    }

    let mut profiles = Vec::new();
    for entry in std::fs::read_dir(&dir).map_err(|e| format!("Failed to read profiles dir: {e}"))? {
        let entry = match entry {
            Ok(e) => e,
            Err(_) => continue,
        };
        let path = entry.path();
        if path.extension().and_then(|s| s.to_str()) != Some("json") {
            continue;
        }
        if let Ok(profile) = read_profile_file(&path) {
            profiles.push(profile);
        }
    }

    profiles.sort_by_key(|a| a.name.to_lowercase());
    Ok(profiles)
}

pub fn get_pi_profile_for_home(home_dir: &Path, id: &str) -> Result<PiProfile, String> {
    load_profile_by_id(home_dir, id)
}

pub fn save_pi_profile_for_home(home_dir: &Path, mut profile: PiProfile) -> Result<(), String> {
    if profile.id.trim().is_empty() {
        profile.id = Uuid::new_v4().to_string();
        profile.created_at = now_rfc3339();
    } else if profile_path_for_home(home_dir, &profile.id)?.exists() {
        if let Ok(old) = load_profile_by_id(home_dir, &profile.id) {
            profile.created_at = old.created_at;
        }
    } else if profile.created_at.trim().is_empty() {
        profile.created_at = now_rfc3339();
    }

    profile.updated_at = now_rfc3339();
    write_profile_file(home_dir, &profile)
}

pub fn delete_pi_profile_for_home(home_dir: &Path, id: &str) -> Result<(), String> {
    let path = profile_path_for_home(home_dir, id)?;
    if path.exists() {
        std::fs::remove_file(&path).map_err(|e| format!("Failed to delete profile: {e}"))?;
    }

    if let Ok(active) = get_active_pi_profile_id_for_home(home_dir) {
        if active.as_deref() == Some(id) {
            let active_path = active_profile_path_for_home(home_dir)?;
            let _ = std::fs::remove_file(active_path);
        }
    }
    Ok(())
}

pub fn duplicate_pi_profile_for_home(
    home_dir: &Path,
    id: &str,
    new_name: &str,
) -> Result<PiProfile, String> {
    let mut profile = load_profile_by_id(home_dir, id)?;
    profile.id = Uuid::new_v4().to_string();
    profile.name = new_name.to_string();
    profile.created_at = now_rfc3339();
    profile.updated_at = profile.created_at.clone();
    write_profile_file(home_dir, &profile)?;
    Ok(profile)
}

pub fn create_default_pi_profile_for_home(home_dir: &Path) -> Result<PiProfile, String> {
    let profiles = list_pi_profiles_for_home(home_dir)?;
    if !profiles.is_empty() {
        return Err("Profiles already exist".to_string());
    }

    let id = Uuid::new_v4().to_string();
    let now = now_rfc3339();

    let profile = PiProfile {
        id,
        name: "Default".to_string(),
        description: None,
        created_at: now.clone(),
        updated_at: now,
        providers: HashMap::new(),
    };

    write_profile_file(home_dir, &profile)?;
    Ok(profile)
}

// ============================================================================
// Active profile
// ============================================================================

pub fn get_active_pi_profile_id_for_home(home_dir: &Path) -> Result<Option<String>, String> {
    let path = active_profile_path_for_home(home_dir)?;
    if !path.exists() {
        return Ok(None);
    }
    let s = std::fs::read_to_string(&path)
        .map_err(|e| format!("Failed to read active profile id: {e}"))?;
    let id = s.trim().to_string();
    if id.is_empty() {
        Ok(None)
    } else {
        Ok(Some(id))
    }
}

pub fn set_active_pi_profile_id_for_home(home_dir: &Path, id: &str) -> Result<(), String> {
    let path = active_profile_path_for_home(home_dir)?;
    storage::atomic_write(&path, id.as_bytes())
}

// ============================================================================
// Apply + Config Status + Read Current Config
// ============================================================================

/// Apply a profile to `~/.pi/agent/models.json`.
///
/// Reads the profile, extracts the providers map, and writes it as
/// `{ "providers": {...} }` to Pi's models.json. Also sets the active profile
/// ID to the applied profile.
pub fn apply_pi_profile_for_home(home_dir: &Path, id: &str) -> Result<(), String> {
    let profile = load_profile_by_id(home_dir, id)?;
    let config_path = pi_config_path_for_home(home_dir)?;

    let current = PiCurrentConfig {
        providers: profile.providers,
    };
    let s = serde_json::to_string_pretty(&current)
        .map_err(|e| format!("Failed to serialize Pi config: {e}"))?;
    storage::atomic_write(&config_path, s.as_bytes())?;
    set_active_pi_profile_id_for_home(home_dir, id)?;
    Ok(())
}

/// Get the status of `~/.pi/agent/models.json`.
pub fn get_pi_config_status_for_home(home_dir: &Path) -> Result<PiConfigStatus, String> {
    let config_path = pi_config_path_for_home(home_dir)?;
    Ok(PiConfigStatus {
        config_exists: config_path.exists(),
        config_path: config_path.to_string_lossy().to_string(),
    })
}

/// Read the current Pi config from `~/.pi/agent/models.json`.
///
/// Returns the parsed config. If the file does not exist, returns an empty
/// config (no providers). If the file contains malformed JSON, returns an error.
pub fn read_pi_current_config_for_home(home_dir: &Path) -> Result<PiCurrentConfig, String> {
    let config_path = pi_config_path_for_home(home_dir)?;
    if !config_path.exists() {
        return Ok(PiCurrentConfig {
            providers: HashMap::new(),
        });
    }
    let s = std::fs::read_to_string(&config_path)
        .map_err(|e| format!("Failed to read Pi config file: {e}"))?;
    let config: PiCurrentConfig =
        serde_json::from_str(&s).map_err(|e| format!("Invalid Pi config JSON: {e}"))?;
    Ok(config)
}

// ============================================================================
// System wrappers (CRUD)
// ============================================================================

pub fn list_pi_profiles() -> Result<Vec<PiProfile>, String> {
    list_pi_profiles_for_home(&system_home_dir()?)
}

pub fn get_pi_profile(id: &str) -> Result<PiProfile, String> {
    get_pi_profile_for_home(&system_home_dir()?, id)
}

pub fn save_pi_profile(profile: PiProfile) -> Result<(), String> {
    save_pi_profile_for_home(&system_home_dir()?, profile)
}

pub fn delete_pi_profile(id: &str) -> Result<(), String> {
    delete_pi_profile_for_home(&system_home_dir()?, id)
}

pub fn duplicate_pi_profile(id: &str, new_name: &str) -> Result<PiProfile, String> {
    duplicate_pi_profile_for_home(&system_home_dir()?, id, new_name)
}

pub fn create_default_pi_profile() -> Result<PiProfile, String> {
    create_default_pi_profile_for_home(&system_home_dir()?)
}

pub fn get_active_pi_profile_id() -> Result<Option<String>, String> {
    get_active_pi_profile_id_for_home(&system_home_dir()?)
}

pub fn set_active_pi_profile_id(id: &str) -> Result<(), String> {
    set_active_pi_profile_id_for_home(&system_home_dir()?, id)
}

pub fn apply_pi_profile(id: &str) -> Result<(), String> {
    apply_pi_profile_for_home(&system_home_dir()?, id)
}

pub fn get_pi_config_status() -> Result<PiConfigStatus, String> {
    get_pi_config_status_for_home(&system_home_dir()?)
}

pub fn read_pi_current_config() -> Result<PiCurrentConfig, String> {
    read_pi_current_config_for_home(&system_home_dir()?)
}

// ============================================================================
// Unit Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn home(temp: &TempDir) -> &Path {
        temp.path()
    }

    fn make_profile(id: &str, name: &str) -> PiProfile {
        PiProfile {
            id: id.to_string(),
            name: name.to_string(),
            description: None,
            created_at: "2026-01-01T00:00:00Z".to_string(),
            updated_at: "2026-01-01T00:00:00Z".to_string(),
            providers: HashMap::new(),
        }
    }

    fn make_profile_with_provider(id: &str, name: &str, provider_key: &str) -> PiProfile {
        let mut providers = HashMap::new();
        providers.insert(
            provider_key.to_string(),
            PiProviderConfig {
                base_url: Some("http://localhost:11434/v1".to_string()),
                api: Some("openai-completions".to_string()),
                api_key: Some("ollama".to_string()),
                headers: None,
                auth_header: Some(false),
                models: vec![PiModel {
                    id: "llama3.1:8b".to_string(),
                    name: Some("Llama 3.1 8B".to_string()),
                    api: Some("openai-completions".to_string()),
                    reasoning: false,
                    input: vec!["text".to_string()],
                    context_window: 128000,
                    max_tokens: 16384,
                    cost: None,
                    compat: None,
                }],
                model_overrides: None,
                compat: None,
            },
        );
        PiProfile {
            id: id.to_string(),
            name: name.to_string(),
            description: Some("A test profile".to_string()),
            created_at: "2026-01-01T00:00:00Z".to_string(),
            updated_at: "2026-01-01T00:00:00Z".to_string(),
            providers,
        }
    }

    #[test]
    fn test_type_serialization() {
        let model = PiModel {
            id: "llama3.1:8b".to_string(),
            name: Some("Llama 3.1 8B".to_string()),
            api: Some("openai-completions".to_string()),
            reasoning: false,
            input: vec!["text".to_string()],
            context_window: 128000,
            max_tokens: 16384,
            cost: Some(PiModelCost {
                input: 0.0,
                output: 0.0,
                cache_read: 0.0,
                cache_write: 0.0,
            }),
            compat: None,
        };

        let json = serde_json::to_string_pretty(&model).unwrap();
        assert!(json.contains("\"id\": \"llama3.1:8b\""));
        assert!(json.contains("\"contextWindow\": 128000"));
        assert!(json.contains("\"maxTokens\": 16384"));
    }

    #[test]
    fn test_provider_serialization() {
        let provider = PiProviderConfig {
            base_url: Some("http://localhost:11434/v1".to_string()),
            api: Some("openai-completions".to_string()),
            api_key: Some("ollama".to_string()),
            headers: None,
            auth_header: Some(false),
            models: vec![],
            model_overrides: None,
            compat: None,
        };

        let json = serde_json::to_string_pretty(&provider).unwrap();
        assert!(json.contains("\"baseUrl\": \"http://localhost:11434/v1\""));
        assert!(json.contains("\"apiKey\": \"ollama\""));
        assert!(json.contains("\"authHeader\": false"));
    }

    #[test]
    fn test_profile_serialization() {
        let profile = PiProfile {
            id: "test-id".to_string(),
            name: "Test Profile".to_string(),
            description: Some("A test profile".to_string()),
            created_at: "2026-01-01T00:00:00Z".to_string(),
            updated_at: "2026-01-01T00:00:00Z".to_string(),
            providers: HashMap::new(),
        };

        let json = serde_json::to_string_pretty(&profile).unwrap();
        assert!(json.contains("\"id\": \"test-id\""));
        assert!(json.contains("\"createdAt\":"));
        assert!(json.contains("\"updatedAt\":"));
    }

    #[test]
    fn test_deserialize_full_config() {
        let json = r#"{
            "providers": {
                "ollama": {
                    "baseUrl": "http://localhost:11434/v1",
                    "api": "openai-completions",
                    "apiKey": "ollama",
                    "headers": {},
                    "authHeader": false,
                    "models": [
                        {
                            "id": "llama3.1:8b",
                            "name": "Llama 3.1 8B",
                            "api": "openai-completions",
                            "reasoning": false,
                            "input": ["text"],
                            "contextWindow": 128000,
                            "maxTokens": 16384,
                            "cost": { "input": 0, "output": 0, "cacheRead": 0, "cacheWrite": 0 },
                            "compat": {}
                        }
                    ],
                    "modelOverrides": {},
                    "compat": {}
                }
            }
        }"#;

        let config: PiCurrentConfig = serde_json::from_str(json).unwrap();
        assert!(config.providers.contains_key("ollama"));
        let provider = &config.providers["ollama"];
        assert_eq!(provider.models.len(), 1);
        assert_eq!(provider.models[0].id, "llama3.1:8b");
        assert_eq!(provider.models[0].context_window, 128000);
    }

    #[test]
    fn test_path_helpers() {
        let temp = TempDir::new().unwrap();
        let home = temp.path();

        let profiles = profiles_dir_for_home(home).unwrap();
        assert!(profiles.ends_with(".droidgear/pi/profiles"));

        let active = active_profile_path_for_home(home).unwrap();
        assert!(active.ends_with(".droidgear/pi/active-profile.txt"));
    }

    #[test]
    fn test_compat_config_fields() {
        let compat = PiCompatConfig {
            supports_developer_role: Some(true),
            thinking_format: Some("openai".to_string()),
            ..Default::default()
        };

        let json = serde_json::to_string_pretty(&compat).unwrap();
        assert!(json.contains("\"supportsDeveloperRole\": true"));
        assert!(json.contains("\"thinkingFormat\": \"openai\""));
    }

    #[test]
    fn test_model_defaults() {
        let json = r#"{"id": "test-model"}"#;
        let model: PiModel = serde_json::from_str(json).unwrap();
        assert_eq!(model.id, "test-model");
        assert!(!model.reasoning);
        assert_eq!(model.input, vec!["text".to_string()]);
        assert_eq!(model.context_window, 128000);
        assert_eq!(model.max_tokens, 16384);
    }

    #[test]
    fn test_model_override() {
        let json = r#"{
            "name": "Custom Name",
            "reasoning": true,
            "contextWindow": 64000
        }"#;
        let override_config: PiModelOverride = serde_json::from_str(json).unwrap();
        assert_eq!(override_config.name.as_deref(), Some("Custom Name"));
        assert_eq!(override_config.reasoning, Some(true));
        assert_eq!(override_config.context_window, Some(64000));
    }

    #[test]
    fn test_model_cost_defaults() {
        let json = r#"{}"#;
        let cost: PiModelCost = serde_json::from_str(json).unwrap();
        assert_eq!(cost.input, 0.0);
        assert_eq!(cost.output, 0.0);
        assert_eq!(cost.cache_read, 0.0);
        assert_eq!(cost.cache_write, 0.0);
    }

    #[test]
    fn test_validate_profile_id() {
        assert!(validate_profile_id("valid-id").is_ok());
        assert!(validate_profile_id("valid_id").is_ok());
        assert!(validate_profile_id("abc123").is_ok());
        assert!(validate_profile_id("").is_err());
        assert!(validate_profile_id("has spaces").is_err());
        assert!(validate_profile_id("has/slash").is_err());
    }

    // =========================================================================
    // CRUD Tests
    // =========================================================================

    #[test]
    fn test_create_default_profile() {
        let temp = TempDir::new().unwrap();
        let home = home(&temp);

        let profile = create_default_pi_profile_for_home(home).unwrap();
        assert!(!profile.id.is_empty());
        assert_eq!(profile.name, "Default");
        assert!(profile.providers.is_empty());

        // Should fail when profiles already exist
        let err = create_default_pi_profile_for_home(home).unwrap_err();
        assert_eq!(err, "Profiles already exist");
    }

    #[test]
    fn test_save_and_get_profile() {
        let temp = TempDir::new().unwrap();
        let home = home(&temp);

        let profile = make_profile("p1", "Profile 1");
        save_pi_profile_for_home(home, profile).unwrap();

        let loaded = get_pi_profile_for_home(home, "p1").unwrap();
        assert_eq!(loaded.id, "p1");
        assert_eq!(loaded.name, "Profile 1");
    }

    #[test]
    fn test_save_profile_generates_id_when_empty() {
        let temp = TempDir::new().unwrap();
        let home = home(&temp);

        let profile = PiProfile {
            id: "".to_string(),
            name: "New Profile".to_string(),
            description: None,
            created_at: "".to_string(),
            updated_at: "".to_string(),
            providers: HashMap::new(),
        };

        save_pi_profile_for_home(home, profile).unwrap();

        let profiles = list_pi_profiles_for_home(home).unwrap();
        assert_eq!(profiles.len(), 1);
        assert!(!profiles[0].id.is_empty());
        assert!(!profiles[0].created_at.is_empty());
        assert!(!profiles[0].updated_at.is_empty());
        assert_eq!(profiles[0].name, "New Profile");
    }

    #[test]
    fn test_save_profile_updates_timestamp() {
        let temp = TempDir::new().unwrap();
        let home = home(&temp);

        let profile = make_profile("p1", "Profile 1");
        save_pi_profile_for_home(home, profile).unwrap();

        let loaded = get_pi_profile_for_home(home, "p1").unwrap();
        // updated_at gets overwritten by now_rfc3339() on save
        assert_ne!(loaded.updated_at, "2026-01-01T00:00:00Z");
        let first_updated_at = loaded.updated_at.clone();

        // Update profile
        let mut updated = loaded.clone();
        updated.name = "Updated Name".to_string();
        save_pi_profile_for_home(home, updated).unwrap();

        let reloaded = get_pi_profile_for_home(home, "p1").unwrap();
        assert_eq!(reloaded.name, "Updated Name");
        assert_ne!(reloaded.updated_at, first_updated_at);
        assert_eq!(reloaded.created_at, "2026-01-01T00:00:00Z");
    }

    #[test]
    fn test_list_profiles_sorted_by_name() {
        let temp = TempDir::new().unwrap();
        let home = home(&temp);

        save_pi_profile_for_home(home, make_profile("p1", "Zebra")).unwrap();
        save_pi_profile_for_home(home, make_profile("p2", "Alpha")).unwrap();
        save_pi_profile_for_home(home, make_profile("p3", "Middle")).unwrap();

        let profiles = list_pi_profiles_for_home(home).unwrap();
        assert_eq!(profiles.len(), 3);
        assert_eq!(profiles[0].name, "Alpha");
        assert_eq!(profiles[1].name, "Middle");
        assert_eq!(profiles[2].name, "Zebra");
    }

    #[test]
    fn test_list_profiles_empty() {
        let temp = TempDir::new().unwrap();
        let home = home(&temp);

        let profiles = list_pi_profiles_for_home(home).unwrap();
        assert!(profiles.is_empty());
    }

    #[test]
    fn test_delete_profile() {
        let temp = TempDir::new().unwrap();
        let home = home(&temp);

        save_pi_profile_for_home(home, make_profile("p1", "Profile 1")).unwrap();
        assert_eq!(list_pi_profiles_for_home(home).unwrap().len(), 1);

        delete_pi_profile_for_home(home, "p1").unwrap();
        assert_eq!(list_pi_profiles_for_home(home).unwrap().len(), 0);
    }

    #[test]
    fn test_delete_profile_clears_active() {
        let temp = TempDir::new().unwrap();
        let home = home(&temp);

        save_pi_profile_for_home(home, make_profile("p1", "Profile 1")).unwrap();
        set_active_pi_profile_id_for_home(home, "p1").unwrap();

        let active = get_active_pi_profile_id_for_home(home).unwrap();
        assert_eq!(active.as_deref(), Some("p1"));

        delete_pi_profile_for_home(home, "p1").unwrap();

        let active_after = get_active_pi_profile_id_for_home(home).unwrap();
        assert!(active_after.is_none());
    }

    #[test]
    fn test_duplicate_profile() {
        let temp = TempDir::new().unwrap();
        let home = home(&temp);

        let profile = make_profile_with_provider("p1", "Original", "ollama");
        save_pi_profile_for_home(home, profile).unwrap();

        let dup = duplicate_pi_profile_for_home(home, "p1", "Copy").unwrap();
        assert_ne!(dup.id, "p1");
        assert_eq!(dup.name, "Copy");
        assert!(dup.providers.contains_key("ollama"));

        // Both should exist
        let profiles = list_pi_profiles_for_home(home).unwrap();
        assert_eq!(profiles.len(), 2);

        // Editing duplicate should not affect original
        let mut dup_updated = dup.clone();
        dup_updated.name = "Modified Copy".to_string();
        save_pi_profile_for_home(home, dup_updated).unwrap();

        let original = get_pi_profile_for_home(home, "p1").unwrap();
        assert_eq!(original.name, "Original");
    }

    #[test]
    fn test_active_profile_get_set() {
        let temp = TempDir::new().unwrap();
        let home = home(&temp);

        // Initially no active profile
        let active = get_active_pi_profile_id_for_home(home).unwrap();
        assert!(active.is_none());

        // Set active
        set_active_pi_profile_id_for_home(home, "p1").unwrap();
        let active = get_active_pi_profile_id_for_home(home).unwrap();
        assert_eq!(active.as_deref(), Some("p1"));

        // Change active
        set_active_pi_profile_id_for_home(home, "p2").unwrap();
        let active = get_active_pi_profile_id_for_home(home).unwrap();
        assert_eq!(active.as_deref(), Some("p2"));
    }

    #[test]
    fn test_crud_roundtrip_with_providers() {
        let temp = TempDir::new().unwrap();
        let home = home(&temp);

        // Create profile with providers
        let profile = make_profile_with_provider("p1", "Test", "ollama");
        save_pi_profile_for_home(home, profile).unwrap();

        // Read and verify providers
        let loaded = get_pi_profile_for_home(home, "p1").unwrap();
        assert_eq!(loaded.providers.len(), 1);
        let provider = &loaded.providers["ollama"];
        assert_eq!(
            provider.base_url.as_deref(),
            Some("http://localhost:11434/v1")
        );
        assert_eq!(provider.models.len(), 1);
        assert_eq!(provider.models[0].id, "llama3.1:8b");

        // Update providers
        let mut updated = loaded.clone();
        updated.providers.insert(
            "openai".to_string(),
            PiProviderConfig {
                base_url: Some("https://api.openai.com/v1".to_string()),
                api: Some("openai-completions".to_string()),
                api_key: Some("sk-test".to_string()),
                headers: None,
                auth_header: None,
                models: vec![],
                model_overrides: None,
                compat: None,
            },
        );
        save_pi_profile_for_home(home, updated).unwrap();

        let reloaded = get_pi_profile_for_home(home, "p1").unwrap();
        assert_eq!(reloaded.providers.len(), 2);
        assert!(reloaded.providers.contains_key("ollama"));
        assert!(reloaded.providers.contains_key("openai"));
    }

    // =========================================================================
    // Apply + Read Current Config Tests
    // =========================================================================

    #[test]
    fn test_apply_writes_models_json() {
        let temp = TempDir::new().unwrap();
        let home = home(&temp);

        let profile = make_profile_with_provider("p1", "Test", "ollama");
        save_pi_profile_for_home(home, profile).unwrap();

        apply_pi_profile_for_home(home, "p1").unwrap();

        let config_path = pi_config_path_for_home(home).unwrap();
        assert!(config_path.exists());

        let raw = std::fs::read_to_string(&config_path).unwrap();
        let parsed: PiCurrentConfig = serde_json::from_str(&raw).unwrap();
        assert!(parsed.providers.contains_key("ollama"));
        assert_eq!(parsed.providers["ollama"].models.len(), 1);
        assert_eq!(parsed.providers["ollama"].models[0].id, "llama3.1:8b");
    }

    #[test]
    fn test_apply_creates_parent_directory() {
        let temp = TempDir::new().unwrap();
        let home = home(&temp);

        // ~/.pi/agent/ does not exist yet
        let profile = make_profile("p1", "Minimal");
        save_pi_profile_for_home(home, profile).unwrap();

        apply_pi_profile_for_home(home, "p1").unwrap();

        let config_path = pi_config_path_for_home(home).unwrap();
        assert!(config_path.exists());
        assert!(config_path.parent().unwrap().exists());
    }

    #[test]
    fn test_apply_sets_active_profile() {
        let temp = TempDir::new().unwrap();
        let home = home(&temp);

        let profile = make_profile("p1", "Test");
        save_pi_profile_for_home(home, profile).unwrap();

        apply_pi_profile_for_home(home, "p1").unwrap();

        let active = get_active_pi_profile_id_for_home(home).unwrap();
        assert_eq!(active.as_deref(), Some("p1"));
    }

    #[test]
    fn test_read_current_config_empty_when_missing() {
        let temp = TempDir::new().unwrap();
        let home = home(&temp);

        let config = read_pi_current_config_for_home(home).unwrap();
        assert!(config.providers.is_empty());
    }

    #[test]
    fn test_read_current_config_parses_file() {
        let temp = TempDir::new().unwrap();
        let home = home(&temp);

        let profile = make_profile_with_provider("p1", "Test", "ollama");
        save_pi_profile_for_home(home, profile).unwrap();
        apply_pi_profile_for_home(home, "p1").unwrap();

        let config = read_pi_current_config_for_home(home).unwrap();
        assert!(config.providers.contains_key("ollama"));
        assert_eq!(config.providers["ollama"].models.len(), 1);
        assert_eq!(config.providers["ollama"].models[0].id, "llama3.1:8b");
    }

    #[test]
    fn test_read_current_config_handles_malformed_json() {
        let temp = TempDir::new().unwrap();
        let home = home(&temp);

        // Write invalid JSON
        let config_path = pi_config_path_for_home(home).unwrap();
        std::fs::create_dir_all(config_path.parent().unwrap()).unwrap();
        std::fs::write(&config_path, "{ invalid json }").unwrap();

        let result = read_pi_current_config_for_home(home);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Invalid Pi config JSON"));
    }

    #[test]
    fn test_apply_read_roundtrip() {
        let temp = TempDir::new().unwrap();
        let home = home(&temp);

        // Create a profile with two providers
        let mut providers = HashMap::new();
        providers.insert(
            "ollama".to_string(),
            PiProviderConfig {
                base_url: Some("http://localhost:11434/v1".to_string()),
                api: Some("openai-completions".to_string()),
                api_key: Some("ollama".to_string()),
                headers: None,
                auth_header: Some(false),
                models: vec![PiModel {
                    id: "llama3.1:8b".to_string(),
                    name: Some("Llama 3.1 8B".to_string()),
                    api: None,
                    reasoning: false,
                    input: vec!["text".to_string()],
                    context_window: 128000,
                    max_tokens: 16384,
                    cost: None,
                    compat: None,
                }],
                model_overrides: None,
                compat: None,
            },
        );
        providers.insert(
            "openai".to_string(),
            PiProviderConfig {
                base_url: Some("https://api.openai.com/v1".to_string()),
                api: Some("openai-completions".to_string()),
                api_key: Some("sk-test".to_string()),
                headers: None,
                auth_header: None,
                models: vec![],
                model_overrides: None,
                compat: None,
            },
        );

        let profile = PiProfile {
            id: "p1".to_string(),
            name: "Roundtrip Test".to_string(),
            description: None,
            created_at: "2026-01-01T00:00:00Z".to_string(),
            updated_at: "2026-01-01T00:00:00Z".to_string(),
            providers,
        };
        save_pi_profile_for_home(home, profile).unwrap();

        // Apply
        apply_pi_profile_for_home(home, "p1").unwrap();

        // Read back
        let config = read_pi_current_config_for_home(home).unwrap();
        assert_eq!(config.providers.len(), 2);

        let ollama = &config.providers["ollama"];
        assert_eq!(
            ollama.base_url.as_deref(),
            Some("http://localhost:11434/v1")
        );
        assert_eq!(ollama.api.as_deref(), Some("openai-completions"));
        assert_eq!(ollama.api_key.as_deref(), Some("ollama"));
        assert_eq!(ollama.models.len(), 1);
        assert_eq!(ollama.models[0].id, "llama3.1:8b");

        let openai = &config.providers["openai"];
        assert_eq!(
            openai.base_url.as_deref(),
            Some("https://api.openai.com/v1")
        );
        assert!(openai.models.is_empty());

        // Verify active profile is set
        let active = get_active_pi_profile_id_for_home(home).unwrap();
        assert_eq!(active.as_deref(), Some("p1"));
    }

    #[test]
    fn test_apply_overwrites_previous_config() {
        let temp = TempDir::new().unwrap();
        let home = home(&temp);

        // Create and apply first profile
        save_pi_profile_for_home(home, make_profile_with_provider("p1", "First", "ollama"))
            .unwrap();
        apply_pi_profile_for_home(home, "p1").unwrap();

        let config = read_pi_current_config_for_home(home).unwrap();
        assert_eq!(config.providers.len(), 1);
        assert!(config.providers.contains_key("ollama"));

        // Create and apply second profile with different provider
        save_pi_profile_for_home(home, make_profile_with_provider("p2", "Second", "openai"))
            .unwrap();
        apply_pi_profile_for_home(home, "p2").unwrap();

        let config = read_pi_current_config_for_home(home).unwrap();
        assert_eq!(config.providers.len(), 1);
        assert!(config.providers.contains_key("openai"));
        assert!(!config.providers.contains_key("ollama"));

        // Active should now be p2
        let active = get_active_pi_profile_id_for_home(home).unwrap();
        assert_eq!(active.as_deref(), Some("p2"));
    }

    #[test]
    fn test_config_status_when_missing() {
        let temp = TempDir::new().unwrap();
        let home = home(&temp);

        let status = get_pi_config_status_for_home(home).unwrap();
        assert!(!status.config_exists);
        assert!(status.config_path.ends_with("models.json"));
    }

    #[test]
    fn test_config_status_when_exists() {
        let temp = TempDir::new().unwrap();
        let home = home(&temp);

        save_pi_profile_for_home(home, make_profile("p1", "Test")).unwrap();
        apply_pi_profile_for_home(home, "p1").unwrap();

        let status = get_pi_config_status_for_home(home).unwrap();
        assert!(status.config_exists);
        assert!(status.config_path.ends_with("models.json"));
    }

    #[test]
    fn test_read_current_config_empty_providers_object() {
        let temp = TempDir::new().unwrap();
        let home = home(&temp);

        // Write empty providers JSON
        let config_path = pi_config_path_for_home(home).unwrap();
        std::fs::create_dir_all(config_path.parent().unwrap()).unwrap();
        std::fs::write(&config_path, r#"{"providers": {}}"#).unwrap();

        let config = read_pi_current_config_for_home(home).unwrap();
        assert!(config.providers.is_empty());
    }

    #[test]
    fn test_read_current_config_missing_providers_key() {
        let temp = TempDir::new().unwrap();
        let home = home(&temp);

        // Write JSON without providers key (should default to empty)
        let config_path = pi_config_path_for_home(home).unwrap();
        std::fs::create_dir_all(config_path.parent().unwrap()).unwrap();
        std::fs::write(&config_path, r#"{"other": "data"}"#).unwrap();

        let config = read_pi_current_config_for_home(home).unwrap();
        assert!(config.providers.is_empty());
    }
}
