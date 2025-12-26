//! API Channel management commands.
//!
//! Handles channel configuration and token management for New API and similar services.

use serde::{Deserialize, Serialize};
use serde_json::Value;
use specta::Type;
use std::fs;
use std::path::PathBuf;

use super::config::{read_config_file, write_config_file, ModelInfo};

// ============================================================================
// Types
// ============================================================================

/// Channel types supported
#[derive(Debug, Clone, Serialize, Deserialize, Type, PartialEq)]
#[serde(rename_all = "kebab-case")]
pub enum ChannelType {
    NewApi,
    OneApi,
}

/// Channel configuration
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct Channel {
    /// Unique identifier (UUID)
    pub id: String,
    /// User-defined name
    pub name: String,
    /// Channel type
    #[serde(rename = "type")]
    pub channel_type: ChannelType,
    /// API base URL
    pub base_url: String,
    /// Whether the channel is enabled
    pub enabled: bool,
    /// Creation timestamp (milliseconds) - use f64 for JS compatibility
    pub created_at: f64,
}

/// Token from channel API
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct ChannelToken {
    /// Token ID from API
    pub id: f64,
    /// Token name
    pub name: String,
    /// Token key (sk-xxx)
    pub key: String,
    /// Status (1=enabled, 2=disabled, etc.)
    pub status: i32,
    /// Remaining quota
    pub remain_quota: f64,
    /// Used quota
    pub used_quota: f64,
    /// Unlimited quota flag
    pub unlimited_quota: bool,
}

/// Channel authentication data (stored in ~/.droidgear/auth/)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ChannelAuth {
    Credentials { username: String, password: String },
}

// ============================================================================
// Auth file helpers
// ============================================================================

fn get_auth_dir() -> Result<PathBuf, String> {
    let home = dirs::home_dir().ok_or("Could not find home directory")?;
    Ok(home.join(".droidgear").join("auth"))
}

fn get_auth_file_path(channel_id: &str) -> Result<PathBuf, String> {
    Ok(get_auth_dir()?.join(format!("{channel_id}.json")))
}

fn read_channel_auth(channel_id: &str) -> Result<Option<ChannelAuth>, String> {
    let path = get_auth_file_path(channel_id)?;
    if !path.exists() {
        return Ok(None);
    }
    let content = fs::read_to_string(&path).map_err(|e| format!("Failed to read auth file: {e}"))?;
    let auth: ChannelAuth =
        serde_json::from_str(&content).map_err(|e| format!("Failed to parse auth file: {e}"))?;
    Ok(Some(auth))
}

fn write_channel_auth(channel_id: &str, auth: &ChannelAuth) -> Result<(), String> {
    let dir = get_auth_dir()?;
    fs::create_dir_all(&dir).map_err(|e| format!("Failed to create auth directory: {e}"))?;
    let path = get_auth_file_path(channel_id)?;
    let content =
        serde_json::to_string_pretty(auth).map_err(|e| format!("Failed to serialize auth: {e}"))?;
    fs::write(&path, content).map_err(|e| format!("Failed to write auth file: {e}"))?;
    Ok(())
}

fn delete_channel_auth(channel_id: &str) -> Result<(), String> {
    let path = get_auth_file_path(channel_id)?;
    if path.exists() {
        fs::remove_file(&path).map_err(|e| format!("Failed to delete auth file: {e}"))?;
    }
    Ok(())
}

// ============================================================================
// Tauri Commands
// ============================================================================

/// Loads all channels from settings.json
#[tauri::command]
#[specta::specta]
pub async fn load_channels() -> Result<Vec<Channel>, String> {
    log::debug!("Loading channels from settings");

    let config = read_config_file()?;

    let channels: Vec<Channel> = config
        .get("channels")
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|v| serde_json::from_value(v.clone()).ok())
                .collect()
        })
        .unwrap_or_default();

    log::info!("Loaded {} channels", channels.len());
    Ok(channels)
}

/// Saves all channels to settings.json
#[tauri::command]
#[specta::specta]
pub async fn save_channels(channels: Vec<Channel>) -> Result<(), String> {
    log::debug!("Saving {} channels to settings", channels.len());

    let mut config = read_config_file()?;

    let channels_value = serde_json::to_value(&channels)
        .map_err(|e| format!("Failed to serialize channels: {e}"))?;

    if let Some(obj) = config.as_object_mut() {
        obj.insert("channels".to_string(), channels_value);
    } else {
        config = serde_json::json!({ "channels": channels_value });
    }

    write_config_file(&config)?;

    log::info!("Successfully saved {} channels", channels.len());
    Ok(())
}

/// Saves a channel's credentials to ~/.droidgear/auth/
#[tauri::command]
#[specta::specta]
pub async fn save_channel_credentials(
    channel_id: String,
    username: String,
    password: String,
) -> Result<(), String> {
    log::debug!("Saving credentials for channel {channel_id}");

    let auth = ChannelAuth::Credentials { username, password };
    write_channel_auth(&channel_id, &auth)?;

    log::info!("Credentials saved for channel {channel_id}");
    Ok(())
}

/// Gets a channel's credentials from ~/.droidgear/auth/
#[tauri::command]
#[specta::specta]
pub async fn get_channel_credentials(
    channel_id: String,
) -> Result<Option<(String, String)>, String> {
    log::debug!("Getting credentials for channel {channel_id}");

    match read_channel_auth(&channel_id)? {
        Some(ChannelAuth::Credentials { username, password }) => Ok(Some((username, password))),
        None => Ok(None),
    }
}

/// Deletes a channel's credentials from ~/.droidgear/auth/
#[tauri::command]
#[specta::specta]
pub async fn delete_channel_credentials(channel_id: String) -> Result<(), String> {
    log::debug!("Deleting credentials for channel {channel_id}");

    delete_channel_auth(&channel_id)?;
    log::info!("Credentials deleted for channel {channel_id}");
    Ok(())
}

/// Fetches tokens from a New API channel
#[tauri::command]
#[specta::specta]
pub async fn fetch_channel_tokens(
    base_url: String,
    username: String,
    password: String,
) -> Result<Vec<ChannelToken>, String> {
    log::debug!("Fetching tokens from {base_url}");

    // Create client with cookie store for session management
    let client = reqwest::Client::builder()
        .cookie_store(true)
        .build()
        .map_err(|e| format!("Failed to create HTTP client: {e}"))?;

    let base = base_url.trim_end_matches('/');

    // First, login to get session cookie and user ID
    let login_url = format!("{base}/api/user/login");
    let login_response = client
        .post(&login_url)
        .json(&serde_json::json!({
            "username": username,
            "password": password
        }))
        .send()
        .await
        .map_err(|e| format!("Failed to login: {e}"))?;

    if !login_response.status().is_success() {
        let status = login_response.status();
        let body = login_response.text().await.unwrap_or_default();
        return Err(format!("Login failed {status}: {body}"));
    }

    let login_data: Value = login_response
        .json()
        .await
        .map_err(|e| format!("Failed to parse login response: {e}"))?;

    if login_data.get("success").and_then(|v| v.as_bool()) != Some(true) {
        let msg = login_data
            .get("message")
            .and_then(|v| v.as_str())
            .unwrap_or("Unknown error");
        return Err(format!("Login failed: {msg}"));
    }

    let user_id = login_data
        .get("data")
        .and_then(|d| d.get("id"))
        .and_then(|id| id.as_i64())
        .ok_or("Could not get user ID from login response")?;

    log::debug!("Logged in as user ID: {user_id}");

    // Now fetch tokens with session cookie and New-Api-User header
    let url = format!("{base}/api/token/");
    let response = client
        .get(&url)
        .header("New-Api-User", user_id.to_string())
        .query(&[("p", "0"), ("page_size", "100")])
        .send()
        .await
        .map_err(|e| format!("Request failed: {e}"))?;

    if !response.status().is_success() {
        let status = response.status();
        let body = response.text().await.unwrap_or_default();
        return Err(format!("API error {status}: {body}"));
    }

    let data: Value = response
        .json()
        .await
        .map_err(|e| format!("Failed to parse response: {e}"))?;

    let tokens: Vec<ChannelToken> = data
        .get("data")
        .and_then(|d| d.get("items"))
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|t| {
                    Some(ChannelToken {
                        id: t.get("id")?.as_f64()?,
                        name: t.get("name")?.as_str()?.to_string(),
                        key: t.get("key")?.as_str()?.to_string(),
                        status: t.get("status")?.as_i64()? as i32,
                        remain_quota: t
                            .get("remain_quota")
                            .and_then(|v| v.as_f64())
                            .unwrap_or(0.0),
                        used_quota: t.get("used_quota").and_then(|v| v.as_f64()).unwrap_or(0.0),
                        unlimited_quota: t
                            .get("unlimited_quota")
                            .and_then(|v| v.as_bool())
                            .unwrap_or(false),
                    })
                })
                .collect()
        })
        .unwrap_or_default();

    log::info!("Fetched {} tokens", tokens.len());
    Ok(tokens)
}

/// Fetches models using a token key (for quick model addition)
#[tauri::command]
#[specta::specta]
pub async fn fetch_models_by_token(
    base_url: String,
    token_key: String,
) -> Result<Vec<ModelInfo>, String> {
    log::debug!("Fetching models from {base_url} using token");

    let client = reqwest::Client::new();
    let url = format!("{}/v1/models", base_url.trim_end_matches('/'));

    let response = client
        .get(&url)
        .header("Authorization", format!("Bearer {token_key}"))
        .send()
        .await
        .map_err(|e| format!("Request failed: {e}"))?;

    if !response.status().is_success() {
        let status = response.status();
        let body = response.text().await.unwrap_or_default();
        return Err(format!("API error {status}: {body}"));
    }

    let data: Value = response
        .json()
        .await
        .map_err(|e| format!("Failed to parse response: {e}"))?;

    let models: Vec<ModelInfo> = data
        .get("data")
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|m| {
                    let id = m.get("id")?.as_str()?.to_string();
                    Some(ModelInfo { id, name: None })
                })
                .collect()
        })
        .unwrap_or_default();

    log::info!("Fetched {} models", models.len());
    Ok(models)
}
