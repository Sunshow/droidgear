//! OpenCode configuration management commands.
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

use super::paths;

// ============================================================================
// Types
// ============================================================================

/// OpenCode Provider options
#[derive(Debug, Clone, Serialize, Deserialize, Type, Default)]
#[serde(rename_all = "camelCase")]
pub struct OpenCodeProviderOptions {
    #[serde(skip_serializing_if = "Option::is_none")]
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
fn get_profiles_dir() -> Result<PathBuf, String> {
    let home = dirs::home_dir().ok_or("Failed to get home directory")?;
    let dir = home.join(".droidgear").join("opencode").join("profiles");
    if !dir.exists() {
        std::fs::create_dir_all(&dir)
            .map_err(|e| format!("Failed to create profiles directory: {e}"))?;
    }
    Ok(dir)
}

/// Gets ~/.droidgear/opencode/active-profile.txt
fn get_active_profile_path() -> Result<PathBuf, String> {
    let home = dirs::home_dir().ok_or("Failed to get home directory")?;
    let dir = home.join(".droidgear").join("opencode");
    if !dir.exists() {
        std::fs::create_dir_all(&dir)
            .map_err(|e| format!("Failed to create opencode directory: {e}"))?;
    }
    Ok(dir.join("active-profile.txt"))
}

/// Gets ~/.config/opencode/ directory
fn get_opencode_config_dir() -> Result<PathBuf, String> {
    let dir = paths::get_opencode_config_dir()?;
    if !dir.exists() {
        std::fs::create_dir_all(&dir)
            .map_err(|e| format!("Failed to create opencode config directory: {e}"))?;
    }
    Ok(dir)
}

/// Gets ~/.local/share/opencode/ directory
fn get_opencode_auth_dir() -> Result<PathBuf, String> {
    let dir = paths::get_opencode_auth_dir()?;
    if !dir.exists() {
        std::fs::create_dir_all(&dir)
            .map_err(|e| format!("Failed to create opencode auth directory: {e}"))?;
    }
    Ok(dir)
}

/// Resolves actual config file path, preferring .jsonc over .json
/// Returns (actual_path, default_path_if_none_exists)
/// If .jsonc exists, returns .jsonc path
/// If only .json exists, returns .json path
/// If neither exists, returns .json path (default for new files)
fn resolve_config_file(dir: &Path, base_name: &str) -> PathBuf {
    let jsonc_path = dir.join(format!("{base_name}.jsonc"));
    let json_path = dir.join(format!("{base_name}.json"));

    if jsonc_path.exists() {
        jsonc_path
    } else {
        json_path
    }
}

/// Gets the actual opencode config file path (prefers .jsonc over .json)
fn get_opencode_config_path() -> Result<PathBuf, String> {
    let dir = get_opencode_config_dir()?;
    Ok(resolve_config_file(&dir, "opencode"))
}

/// Gets the actual opencode auth file path (prefers .jsonc over .json)
fn get_opencode_auth_path() -> Result<PathBuf, String> {
    let dir = get_opencode_auth_dir()?;
    Ok(resolve_config_file(&dir, "auth"))
}

/// Atomic write helper
fn atomic_write(path: &PathBuf, content: &str) -> Result<(), String> {
    let temp_path = path.with_extension("tmp");
    std::fs::write(&temp_path, content).map_err(|e| format!("Failed to write file: {e}"))?;
    std::fs::rename(&temp_path, path).map_err(|e| {
        let _ = std::fs::remove_file(&temp_path);
        format!("Failed to finalize file: {e}")
    })
}

/// Read JSON/JSONC file or return empty object
/// Supports JSONC format (JSON with // and /* */ comments)
fn read_json_file(path: &PathBuf) -> Value {
    if !path.exists() {
        return serde_json::json!({});
    }

    let content = match std::fs::read_to_string(path) {
        Ok(s) => s,
        Err(_) => return serde_json::json!({}),
    };

    // Use StripComments to remove // and /* */ comments before parsing
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

// ============================================================================
// Profile CRUD Commands
// ============================================================================

/// List all profiles
#[tauri::command]
#[specta::specta]
pub async fn list_opencode_profiles() -> Result<Vec<OpenCodeProfile>, String> {
    let dir = get_profiles_dir()?;
    let mut profiles = Vec::new();

    if let Ok(entries) = std::fs::read_dir(&dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().is_some_and(|e| e == "json") {
                if let Ok(content) = std::fs::read_to_string(&path) {
                    if let Ok(profile) = serde_json::from_str::<OpenCodeProfile>(&content) {
                        profiles.push(profile);
                    }
                }
            }
        }
    }

    profiles.sort_by(|a, b| a.name.cmp(&b.name));
    Ok(profiles)
}

/// Get a single profile by ID
#[tauri::command]
#[specta::specta]
pub async fn get_opencode_profile(id: String) -> Result<OpenCodeProfile, String> {
    let dir = get_profiles_dir()?;
    let path = dir.join(format!("{id}.json"));

    if !path.exists() {
        return Err(format!("Profile not found: {id}"));
    }

    let content =
        std::fs::read_to_string(&path).map_err(|e| format!("Failed to read profile: {e}"))?;
    serde_json::from_str(&content).map_err(|e| format!("Failed to parse profile: {e}"))
}

/// Save a profile
#[tauri::command]
#[specta::specta]
pub async fn save_opencode_profile(mut profile: OpenCodeProfile) -> Result<(), String> {
    let dir = get_profiles_dir()?;
    let path = dir.join(format!("{}.json", profile.id));

    profile.updated_at = chrono::Utc::now().to_rfc3339();

    let content = serde_json::to_string_pretty(&profile)
        .map_err(|e| format!("Failed to serialize profile: {e}"))?;
    atomic_write(&path, &content)
}

/// Delete a profile
#[tauri::command]
#[specta::specta]
pub async fn delete_opencode_profile(id: String) -> Result<(), String> {
    let dir = get_profiles_dir()?;
    let path = dir.join(format!("{id}.json"));

    if path.exists() {
        std::fs::remove_file(&path).map_err(|e| format!("Failed to delete profile: {e}"))?;
    }

    // Clear active profile if it was the deleted one
    let active_path = get_active_profile_path()?;
    if let Ok(active_id) = std::fs::read_to_string(&active_path) {
        if active_id.trim() == id {
            let _ = std::fs::remove_file(&active_path);
        }
    }

    Ok(())
}

/// Duplicate a profile
#[tauri::command]
#[specta::specta]
pub async fn duplicate_opencode_profile(
    id: String,
    new_name: String,
) -> Result<OpenCodeProfile, String> {
    let mut profile = get_opencode_profile(id).await?;
    let now = chrono::Utc::now().to_rfc3339();

    profile.id = Uuid::new_v4().to_string();
    profile.name = new_name;
    profile.created_at = now.clone();
    profile.updated_at = now;

    save_opencode_profile(profile.clone()).await?;
    Ok(profile)
}

/// Create default profile if none exists
#[tauri::command]
#[specta::specta]
pub async fn create_default_profile() -> Result<OpenCodeProfile, String> {
    let profiles = list_opencode_profiles().await?;
    if !profiles.is_empty() {
        return Err("Profiles already exist".to_string());
    }

    let now = chrono::Utc::now().to_rfc3339();
    let profile = OpenCodeProfile {
        id: Uuid::new_v4().to_string(),
        name: "Default".to_string(),
        description: None,
        created_at: now.clone(),
        updated_at: now,
        providers: HashMap::new(),
        auth: HashMap::new(),
    };

    save_opencode_profile(profile.clone()).await?;

    // Set as active
    let active_path = get_active_profile_path()?;
    atomic_write(&active_path, &profile.id)?;

    Ok(profile)
}

// ============================================================================
// Profile Apply Commands
// ============================================================================

/// Get active profile ID
#[tauri::command]
#[specta::specta]
pub async fn get_active_opencode_profile_id() -> Result<Option<String>, String> {
    let path = get_active_profile_path()?;
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

/// Apply a profile to OpenCode config files (merge write)
/// Supports both .json and .jsonc files, preferring .jsonc when both exist
#[tauri::command]
#[specta::specta]
pub async fn apply_opencode_profile(id: String) -> Result<(), String> {
    let profile = get_opencode_profile(id.clone()).await?;

    // 1. Merge providers into opencode config (json or jsonc)
    let config_path = get_opencode_config_path()?;
    let mut config = read_json_file(&config_path);

    if !profile.providers.is_empty() {
        let providers_value = serde_json::to_value(&profile.providers)
            .map_err(|e| format!("Failed to serialize providers: {e}"))?;

        if let Some(obj) = config.as_object_mut() {
            // Merge into existing provider object
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
    atomic_write(&config_path, &config_content)?;

    // 2. Merge auth into auth config (json or jsonc)
    let auth_path = get_opencode_auth_path()?;
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
    atomic_write(&auth_path, &auth_content)?;

    // 3. Update active profile
    let active_path = get_active_profile_path()?;
    atomic_write(&active_path, &id)?;

    log::info!("Applied OpenCode profile: {}", profile.name);
    Ok(())
}

// ============================================================================
// Helper Commands
// ============================================================================

/// Get OpenCode config status
/// Returns actual file paths, preferring .jsonc over .json when both exist
#[tauri::command]
#[specta::specta]
pub async fn get_opencode_config_status() -> Result<OpenCodeConfigStatus, String> {
    let config_path = get_opencode_config_path()?;
    let auth_path = get_opencode_auth_path()?;

    Ok(OpenCodeConfigStatus {
        config_exists: config_path.exists(),
        auth_exists: auth_path.exists(),
        config_path: config_path.to_string_lossy().to_string(),
        auth_path: auth_path.to_string_lossy().to_string(),
    })
}

/// Get provider templates
#[tauri::command]
#[specta::specta]
pub async fn get_opencode_provider_templates() -> Result<Vec<ProviderTemplate>, String> {
    Ok(vec![ProviderTemplate {
        id: "anthropic".to_string(),
        name: "Anthropic".to_string(),
        default_base_url: Some("https://api.anthropic.com".to_string()),
        requires_api_key: true,
    }])
}

/// Test provider connection
#[tauri::command]
#[specta::specta]
pub async fn test_opencode_provider_connection(
    provider_id: String,
    base_url: String,
    api_key: String,
) -> Result<bool, String> {
    let client = reqwest::Client::new();
    let url = format!("{}/v1/models", base_url.trim_end_matches('/'));

    let response = match provider_id.as_str() {
        "anthropic" => {
            client
                .get(&url)
                .header("x-api-key", &api_key)
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

/// Read current OpenCode configuration from config files
/// Returns providers from opencode.json/jsonc and auth from auth.json/jsonc
/// Also extracts apiKey from provider.options.apiKey if auth.json doesn't have it
#[tauri::command]
#[specta::specta]
pub async fn read_opencode_current_config() -> Result<OpenCodeCurrentConfig, String> {
    // Read providers from opencode config (as raw JSON to preserve all fields)
    let config_path = get_opencode_config_path()?;
    let config = read_json_file(&config_path);
    let provider_value = config
        .get("provider")
        .cloned()
        .unwrap_or(serde_json::json!({}));

    // Normalize provider options: convert baseURL to baseUrl for consistency
    let normalized_provider = normalize_provider_options(&provider_value);

    let providers: HashMap<String, OpenCodeProviderConfig> =
        serde_json::from_value(normalized_provider.clone()).unwrap_or_default();

    // Read auth from auth config
    let auth_path = get_opencode_auth_path()?;
    let auth_value = read_json_file(&auth_path);
    let mut auth: HashMap<String, Value> = auth_value
        .as_object()
        .map(|obj| obj.iter().map(|(k, v)| (k.clone(), v.clone())).collect())
        .unwrap_or_default();

    // Extract apiKey from provider.options.apiKey if auth doesn't have it
    if let Some(provider_obj) = normalized_provider.as_object() {
        for (provider_id, provider_config) in provider_obj {
            if auth.contains_key(provider_id) {
                continue;
            }
            // Check for apiKey in options
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

    log::info!(
        "Read {} providers and {} auth entries from OpenCode config",
        providers.len(),
        auth.len()
    );

    Ok(OpenCodeCurrentConfig { providers, auth })
}

/// Normalize provider options: convert baseURL to baseUrl for consistency
fn normalize_provider_options(provider_value: &Value) -> Value {
    let mut result = provider_value.clone();
    if let Some(providers) = result.as_object_mut() {
        for (_provider_id, provider_config) in providers.iter_mut() {
            if let Some(options) = provider_config.get_mut("options") {
                if let Some(options_obj) = options.as_object_mut() {
                    // Convert baseURL to baseUrl
                    if let Some(base_url) = options_obj.remove("baseURL") {
                        options_obj.insert("baseUrl".to_string(), base_url);
                    }
                }
            }
        }
    }
    result
}
