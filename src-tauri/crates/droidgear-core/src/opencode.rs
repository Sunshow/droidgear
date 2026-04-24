//! OpenCode configuration management (core).
//!
//! Handles Profile CRUD and applying profiles to OpenCode config files.

use json_comments::StripComments;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use specta::Type;
use std::collections::HashMap;
use std::io::Read;
use std::path::{Path, PathBuf};
use uuid::Uuid;

use crate::{paths, storage};

// ============================================================================
// Types
// ============================================================================

/// OpenCode Provider options
#[derive(Debug, Clone, Serialize, Deserialize, Type, Default)]
#[serde(rename_all = "camelCase")]
pub struct OpenCodeProviderOptions {
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "baseURL")]
    pub base_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub api_key: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timeout: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub headers: Option<HashMap<String, String>>,
}

/// OpenCode Model limit configuration
#[derive(Debug, Clone, Serialize, Deserialize, Type, Default)]
#[serde(rename_all = "camelCase")]
pub struct OpenCodeModelLimit {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub context: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub output: Option<u32>,
}

/// OpenCode Model configuration
#[derive(Debug, Clone, Serialize, Deserialize, Type, Default)]
#[serde(rename_all = "camelCase")]
pub struct OpenCodeModelConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<OpenCodeModelLimit>,
}

/// OpenCode Provider configuration
#[derive(Debug, Clone, Serialize, Deserialize, Type, Default)]
#[serde(rename_all = "camelCase")]
pub struct OpenCodeProviderConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub npm: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub options: Option<OpenCodeProviderOptions>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub models: Option<HashMap<String, OpenCodeModelConfig>>,
}

/// OpenCode Profile
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct OpenCodeProfile {
    pub id: String,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    pub created_at: String,
    pub updated_at: String,
    pub providers: HashMap<String, OpenCodeProviderConfig>,
    pub auth: HashMap<String, Value>,
}

/// Configuration status
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct OpenCodeConfigStatus {
    pub config_exists: bool,
    pub auth_exists: bool,
    pub config_path: String,
    pub auth_path: String,
}

/// Provider template for quick setup
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct ProviderTemplate {
    pub id: String,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub npm: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default_base_url: Option<String>,
    pub requires_api_key: bool,
}

/// Current OpenCode configuration (providers and auth from config files)
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct OpenCodeCurrentConfig {
    pub providers: HashMap<String, OpenCodeProviderConfig>,
    pub auth: HashMap<String, Value>,
}

// ============================================================================
// Path Helpers
// ============================================================================

/// Gets ~/.droidgear/opencode/profiles/
fn profiles_dir_for_home(home_dir: &Path) -> Result<PathBuf, String> {
    let dir = home_dir
        .join(".droidgear")
        .join("opencode")
        .join("profiles");
    if !dir.exists() {
        std::fs::create_dir_all(&dir)
            .map_err(|e| format!("Failed to create profiles directory: {e}"))?;
    }
    Ok(dir)
}

/// Gets ~/.droidgear/opencode/active-profile.txt
fn active_profile_path_for_home(home_dir: &Path) -> Result<PathBuf, String> {
    let dir = home_dir.join(".droidgear").join("opencode");
    if !dir.exists() {
        std::fs::create_dir_all(&dir)
            .map_err(|e| format!("Failed to create opencode directory: {e}"))?;
    }
    Ok(dir.join("active-profile.txt"))
}

/// Gets ~/.config/opencode/ directory (or override)
fn opencode_config_dir_for_home(home_dir: &Path) -> Result<PathBuf, String> {
    let config_paths = paths::load_config_paths_for_home(home_dir);
    let dir = paths::get_opencode_config_dir_for_home(home_dir, &config_paths)?;
    if !dir.exists() {
        std::fs::create_dir_all(&dir)
            .map_err(|e| format!("Failed to create opencode config directory: {e}"))?;
    }
    Ok(dir)
}

/// Gets ~/.local/share/opencode/ directory (or override)
fn opencode_auth_dir_for_home(home_dir: &Path) -> Result<PathBuf, String> {
    let config_paths = paths::load_config_paths_for_home(home_dir);
    let dir = paths::get_opencode_auth_dir_for_home(home_dir, &config_paths)?;
    if !dir.exists() {
        std::fs::create_dir_all(&dir)
            .map_err(|e| format!("Failed to create opencode auth directory: {e}"))?;
    }
    Ok(dir)
}

/// Resolves actual config file path, preferring .jsonc over .json
fn resolve_config_file(dir: &Path, base_name: &str) -> PathBuf {
    let jsonc_path = dir.join(format!("{base_name}.jsonc"));
    let json_path = dir.join(format!("{base_name}.json"));

    if jsonc_path.exists() {
        jsonc_path
    } else {
        json_path
    }
}

fn opencode_config_path_for_home(home_dir: &Path) -> Result<PathBuf, String> {
    let dir = opencode_config_dir_for_home(home_dir)?;
    Ok(resolve_config_file(&dir, "opencode"))
}

fn opencode_auth_path_for_home(home_dir: &Path) -> Result<PathBuf, String> {
    let dir = opencode_auth_dir_for_home(home_dir)?;
    Ok(resolve_config_file(&dir, "auth"))
}

// ============================================================================
// File helpers
// ============================================================================

fn read_json_file(path: &Path) -> Value {
    if !path.exists() {
        return serde_json::json!({});
    }

    let content = match std::fs::read_to_string(path) {
        Ok(s) => s,
        Err(_) => return serde_json::json!({}),
    };

    let stripped = StripComments::new(content.as_bytes());
    let mut buf = String::new();
    if std::io::BufReader::new(stripped)
        .read_to_string(&mut buf)
        .is_err()
    {
        return serde_json::json!({});
    }

    serde_json::from_str(&buf).unwrap_or(serde_json::json!({}))
}

fn now_rfc3339() -> String {
    chrono::Utc::now().to_rfc3339()
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

fn profile_path_for_home(home_dir: &Path, id: &str) -> Result<PathBuf, String> {
    validate_profile_id(id)?;
    Ok(profiles_dir_for_home(home_dir)?.join(format!("{id}.json")))
}

fn read_profile_file(path: &Path) -> Result<OpenCodeProfile, String> {
    let s = std::fs::read_to_string(path).map_err(|e| format!("Failed to read profile: {e}"))?;
    serde_json::from_str::<OpenCodeProfile>(&s).map_err(|e| format!("Invalid profile JSON: {e}"))
}

fn write_profile_file(home_dir: &Path, profile: &OpenCodeProfile) -> Result<(), String> {
    let path = profile_path_for_home(home_dir, &profile.id)?;
    let s = serde_json::to_string_pretty(profile)
        .map_err(|e| format!("Failed to serialize profile JSON: {e}"))?;
    storage::atomic_write(&path, s.as_bytes())
}

fn load_profile_by_id(home_dir: &Path, id: &str) -> Result<OpenCodeProfile, String> {
    let path = profile_path_for_home(home_dir, id)?;
    read_profile_file(&path)
}

// ============================================================================
// Profile CRUD
// ============================================================================

pub fn list_opencode_profiles_for_home(home_dir: &Path) -> Result<Vec<OpenCodeProfile>, String> {
    let dir = profiles_dir_for_home(home_dir)?;
    let mut profiles = Vec::new();

    for entry in std::fs::read_dir(&dir).map_err(|e| format!("Failed to read profiles dir: {e}"))? {
        let entry = entry.map_err(|e| format!("Failed to read dir entry: {e}"))?;
        let path = entry.path();
        if path.extension().and_then(|s| s.to_str()) != Some("json") {
            continue;
        }
        if let Ok(p) = read_profile_file(&path) {
            profiles.push(p);
        }
    }

    profiles.sort_by_key(|a| a.name.to_lowercase());
    Ok(profiles)
}

pub fn get_opencode_profile_for_home(home_dir: &Path, id: &str) -> Result<OpenCodeProfile, String> {
    load_profile_by_id(home_dir, id)
}

pub fn save_opencode_profile_for_home(
    home_dir: &Path,
    mut profile: OpenCodeProfile,
) -> Result<(), String> {
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

pub fn delete_opencode_profile_for_home(home_dir: &Path, id: &str) -> Result<(), String> {
    let path = profile_path_for_home(home_dir, id)?;
    if path.exists() {
        std::fs::remove_file(&path).map_err(|e| format!("Failed to delete profile: {e}"))?;
    }

    if let Ok(active) = get_active_opencode_profile_id_for_home(home_dir) {
        if active.as_deref() == Some(id) {
            let active_path = active_profile_path_for_home(home_dir)?;
            let _ = std::fs::remove_file(active_path);
        }
    }

    Ok(())
}

pub fn duplicate_opencode_profile_for_home(
    home_dir: &Path,
    id: &str,
    new_name: &str,
) -> Result<OpenCodeProfile, String> {
    let mut profile = load_profile_by_id(home_dir, id)?;
    profile.id = Uuid::new_v4().to_string();
    profile.name = new_name.to_string();
    profile.created_at = now_rfc3339();
    profile.updated_at = profile.created_at.clone();
    write_profile_file(home_dir, &profile)?;
    Ok(profile)
}

pub fn create_default_profile_for_home(home_dir: &Path) -> Result<OpenCodeProfile, String> {
    let profiles = list_opencode_profiles_for_home(home_dir)?;
    if !profiles.is_empty() {
        return Err("Profiles already exist".to_string());
    }

    let now = now_rfc3339();
    let profile = OpenCodeProfile {
        id: Uuid::new_v4().to_string(),
        name: "Default".to_string(),
        description: None,
        created_at: now.clone(),
        updated_at: now,
        providers: HashMap::new(),
        auth: HashMap::new(),
    };

    save_opencode_profile_for_home(home_dir, profile.clone())?;

    let active_path = active_profile_path_for_home(home_dir)?;
    storage::atomic_write(&active_path, profile.id.as_bytes())?;

    Ok(profile)
}

// ============================================================================
// Apply
// ============================================================================

pub fn get_active_opencode_profile_id_for_home(home_dir: &Path) -> Result<Option<String>, String> {
    let path = active_profile_path_for_home(home_dir)?;
    if !path.exists() {
        return Ok(None);
    }
    let id = std::fs::read_to_string(&path)
        .map_err(|e| format!("Failed to read active profile: {e}"))?;
    let id = id.trim().to_string();
    if id.is_empty() {
        Ok(None)
    } else {
        Ok(Some(id))
    }
}

pub fn apply_opencode_profile_for_home(home_dir: &Path, id: &str) -> Result<(), String> {
    let profile = get_opencode_profile_for_home(home_dir, id)?;

    let config_path = opencode_config_path_for_home(home_dir)?;
    let mut config = read_json_file(&config_path);

    if !profile.providers.is_empty() {
        let providers_value = serde_json::to_value(&profile.providers)
            .map_err(|e| format!("Failed to serialize providers: {e}"))?;

        if let Some(obj) = config.as_object_mut() {
            let existing = obj
                .entry("provider")
                .or_insert_with(|| serde_json::json!({}));
            if let (Some(existing_obj), Some(new_obj)) =
                (existing.as_object_mut(), providers_value.as_object())
            {
                for (k, v) in new_obj {
                    existing_obj.insert(k.clone(), v.clone());
                }
            }
        }
    }

    let config_content = serde_json::to_string_pretty(&config)
        .map_err(|e| format!("Failed to serialize config: {e}"))?;
    storage::atomic_write(&config_path, config_content.as_bytes())?;

    let auth_path = opencode_auth_path_for_home(home_dir)?;
    let mut auth = read_json_file(&auth_path);

    if !profile.auth.is_empty() {
        if let Some(obj) = auth.as_object_mut() {
            for (k, v) in &profile.auth {
                obj.insert(k.clone(), v.clone());
            }
        }
    }

    let auth_content = serde_json::to_string_pretty(&auth)
        .map_err(|e| format!("Failed to serialize auth: {e}"))?;
    storage::atomic_write(&auth_path, auth_content.as_bytes())?;

    let active_path = active_profile_path_for_home(home_dir)?;
    storage::atomic_write(&active_path, id.as_bytes())?;

    Ok(())
}

pub fn get_opencode_config_status_for_home(
    home_dir: &Path,
) -> Result<OpenCodeConfigStatus, String> {
    let config_path = opencode_config_path_for_home(home_dir)?;
    let auth_path = opencode_auth_path_for_home(home_dir)?;

    Ok(OpenCodeConfigStatus {
        config_exists: config_path.exists(),
        auth_exists: auth_path.exists(),
        config_path: config_path.to_string_lossy().to_string(),
        auth_path: auth_path.to_string_lossy().to_string(),
    })
}

pub fn get_opencode_provider_templates() -> Vec<ProviderTemplate> {
    vec![
        ProviderTemplate {
            id: "anthropic".to_string(),
            name: "Anthropic".to_string(),
            npm: Some("@ai-sdk/anthropic".to_string()),
            default_base_url: Some("https://api.anthropic.com/v1".to_string()),
            requires_api_key: true,
        },
        ProviderTemplate {
            id: "openai".to_string(),
            name: "OpenAI".to_string(),
            npm: Some("@ai-sdk/openai".to_string()),
            default_base_url: Some("https://api.openai.com/v1".to_string()),
            requires_api_key: true,
        },
        ProviderTemplate {
            id: "gemini".to_string(),
            name: "Gemini".to_string(),
            npm: Some("@ai-sdk/google".to_string()),
            default_base_url: Some("https://generativelanguage.googleapis.com/v1beta".to_string()),
            requires_api_key: true,
        },
    ]
}

pub async fn test_opencode_provider_connection(
    provider_id: &str,
    base_url: &str,
    api_key: &str,
) -> Result<bool, String> {
    let client = reqwest::Client::new();
    let url = format!("{}/v1/models", base_url.trim_end_matches('/'));

    let response = match provider_id {
        "anthropic" => {
            client
                .get(&url)
                .header("x-api-key", api_key)
                .header("anthropic-version", "2023-06-01")
                .send()
                .await
        }
        _ => {
            client
                .get(&url)
                .header("Authorization", format!("Bearer {api_key}"))
                .send()
                .await
        }
    };

    match response {
        Ok(resp) => Ok(resp.status().is_success()),
        Err(e) => Err(format!("Connection failed: {e}")),
    }
}

fn normalize_provider_options(provider_value: &Value) -> Value {
    let mut result = provider_value.clone();
    if let Some(providers) = result.as_object_mut() {
        for (_provider_id, provider_config) in providers.iter_mut() {
            if let Some(options) = provider_config.get_mut("options") {
                if let Some(options_obj) = options.as_object_mut() {
                    if options_obj.contains_key("baseUrl") && !options_obj.contains_key("baseURL") {
                        if let Some(base_url) = options_obj.get("baseUrl").cloned() {
                            options_obj.insert("baseURL".to_string(), base_url);
                        }
                    }
                }
            }
        }
    }
    result
}

pub fn read_opencode_current_config_for_home(
    home_dir: &Path,
) -> Result<OpenCodeCurrentConfig, String> {
    let config_path = opencode_config_path_for_home(home_dir)?;
    let config = read_json_file(&config_path);
    let provider_value = config
        .get("provider")
        .cloned()
        .unwrap_or(serde_json::json!({}));

    let normalized_provider = normalize_provider_options(&provider_value);

    let providers: HashMap<String, OpenCodeProviderConfig> =
        serde_json::from_value(normalized_provider.clone()).unwrap_or_default();

    let auth_path = opencode_auth_path_for_home(home_dir)?;
    let auth_value = read_json_file(&auth_path);
    let mut auth: HashMap<String, Value> = auth_value
        .as_object()
        .map(|obj| obj.iter().map(|(k, v)| (k.clone(), v.clone())).collect())
        .unwrap_or_default();

    if let Some(provider_obj) = normalized_provider.as_object() {
        for (provider_id, provider_config) in provider_obj {
            if auth.contains_key(provider_id) {
                continue;
            }
            if let Some(api_key) = provider_config
                .get("options")
                .and_then(|opts| opts.get("apiKey"))
                .and_then(|k| k.as_str())
            {
                if !api_key.is_empty() {
                    auth.insert(
                        provider_id.clone(),
                        serde_json::json!({
                          "type": "api",
                          "key": api_key
                        }),
                    );
                }
            }
        }
    }

    Ok(OpenCodeCurrentConfig { providers, auth })
}

// ============================================================================
// System wrappers
// ============================================================================

fn system_home_dir() -> Result<PathBuf, String> {
    dirs::home_dir().ok_or_else(|| "Failed to get home directory".to_string())
}

pub fn list_opencode_profiles() -> Result<Vec<OpenCodeProfile>, String> {
    list_opencode_profiles_for_home(&system_home_dir()?)
}

pub fn get_opencode_profile(id: &str) -> Result<OpenCodeProfile, String> {
    get_opencode_profile_for_home(&system_home_dir()?, id)
}

pub fn save_opencode_profile(profile: OpenCodeProfile) -> Result<(), String> {
    save_opencode_profile_for_home(&system_home_dir()?, profile)
}

pub fn delete_opencode_profile(id: &str) -> Result<(), String> {
    delete_opencode_profile_for_home(&system_home_dir()?, id)
}

pub fn duplicate_opencode_profile(id: &str, new_name: &str) -> Result<OpenCodeProfile, String> {
    duplicate_opencode_profile_for_home(&system_home_dir()?, id, new_name)
}

pub fn create_default_profile() -> Result<OpenCodeProfile, String> {
    create_default_profile_for_home(&system_home_dir()?)
}

pub fn get_active_opencode_profile_id() -> Result<Option<String>, String> {
    get_active_opencode_profile_id_for_home(&system_home_dir()?)
}

pub fn apply_opencode_profile(id: &str) -> Result<(), String> {
    apply_opencode_profile_for_home(&system_home_dir()?, id)
}

pub fn get_opencode_config_status() -> Result<OpenCodeConfigStatus, String> {
    get_opencode_config_status_for_home(&system_home_dir()?)
}

pub fn read_opencode_current_config() -> Result<OpenCodeCurrentConfig, String> {
    read_opencode_current_config_for_home(&system_home_dir()?)
}
