//! API Channel management commands.
//!
//! Handles channel configuration and token management for New API and similar services.

use serde::{Deserialize, Serialize};
use serde_json::Value;
use specta::Type;
use std::fs;
use std::path::PathBuf;

use super::config::{read_config_file, ConfigReadResult, ModelInfo};

// ============================================================================
// Types
// ============================================================================

/// Channel types supported
#[derive(Debug, Clone, Serialize, Deserialize, Type, PartialEq)]
#[serde(rename_all = "kebab-case")]
pub enum ChannelType {
    NewApi,
    #[serde(rename = "sub-2-api")]
    Sub2Api,
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
    /// Platform type (openai, anthropic, gemini, etc.) - from Sub2API
    pub platform: Option<String>,
    /// Group name - from Sub2API
    pub group_name: Option<String>,
}

/// Channel authentication data (stored in ~/.droidgear/auth/)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ChannelAuth {
    Credentials { username: String, password: String },
}

// ============================================================================
// DroidGear config helpers
// ============================================================================

fn get_droidgear_dir() -> Result<PathBuf, String> {
    let home = dirs::home_dir().ok_or("Could not find home directory")?;
    Ok(home.join(".droidgear"))
}

fn get_droidgear_channels_path() -> Result<PathBuf, String> {
    let dir = get_droidgear_dir()?;
    if !dir.exists() {
        fs::create_dir_all(&dir)
            .map_err(|e| format!("Failed to create .droidgear directory: {e}"))?;
    }
    Ok(dir.join("channels.json"))
}

fn read_channels_from_file(path: &PathBuf) -> Result<Vec<Channel>, String> {
    let content =
        fs::read_to_string(path).map_err(|e| format!("Failed to read channels file: {e}"))?;
    let channels: Vec<Channel> = serde_json::from_str(&content)
        .map_err(|e| format!("Failed to parse channels file: {e}"))?;
    Ok(channels)
}

fn write_channels_to_file(path: &PathBuf, channels: &[Channel]) -> Result<(), String> {
    let content = serde_json::to_string_pretty(channels)
        .map_err(|e| format!("Failed to serialize channels: {e}"))?;
    fs::write(path, content).map_err(|e| format!("Failed to write channels file: {e}"))?;
    Ok(())
}

// ============================================================================
// Auth file helpers
// ============================================================================

fn get_auth_dir() -> Result<PathBuf, String> {
    Ok(get_droidgear_dir()?.join("auth"))
}

fn get_auth_file_path(channel_id: &str) -> Result<PathBuf, String> {
    Ok(get_auth_dir()?.join(format!("{channel_id}.json")))
}

fn read_channel_auth(channel_id: &str) -> Result<Option<ChannelAuth>, String> {
    let path = get_auth_file_path(channel_id)?;
    if !path.exists() {
        return Ok(None);
    }
    let content =
        fs::read_to_string(&path).map_err(|e| format!("Failed to read auth file: {e}"))?;
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

/// Loads all channels from ~/.droidgear/channels.json
/// Falls back to ~/.factory/settings.json for migration
#[tauri::command]
#[specta::specta]
pub async fn load_channels() -> Result<Vec<Channel>, String> {
    log::debug!("Loading channels");

    let droidgear_path = get_droidgear_channels_path()?;

    // Priority 1: Read from ~/.droidgear/channels.json
    if droidgear_path.exists() {
        log::debug!("Reading channels from ~/.droidgear/channels.json");
        return read_channels_from_file(&droidgear_path);
    }

    // Priority 2: Migrate from ~/.factory/settings.json if channels exist there
    if let ConfigReadResult::Ok(config) = read_config_file() {
        if let Some(channels_value) = config.get("channels") {
            if let Some(arr) = channels_value.as_array() {
                if !arr.is_empty() {
                    log::info!("Migrating channels from ~/.factory/settings.json");
                    let channels: Vec<Channel> = arr
                        .iter()
                        .filter_map(|v| serde_json::from_value(v.clone()).ok())
                        .collect();

                    // Save to new location
                    if !channels.is_empty() {
                        write_channels_to_file(&droidgear_path, &channels)?;
                        log::info!("Migrated {} channels to ~/.droidgear/", channels.len());
                    }

                    return Ok(channels);
                }
            }
        }
    }

    log::debug!("No channels found");
    Ok(vec![])
}

/// Saves all channels to ~/.droidgear/channels.json
#[tauri::command]
#[specta::specta]
pub async fn save_channels(channels: Vec<Channel>) -> Result<(), String> {
    log::debug!("Saving {} channels", channels.len());

    let path = get_droidgear_channels_path()?;
    write_channels_to_file(&path, &channels)?;

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

/// Fetches tokens from a channel (dispatches based on channel type)
#[tauri::command]
#[specta::specta]
pub async fn fetch_channel_tokens(
    channel_type: ChannelType,
    base_url: String,
    username: String,
    password: String,
) -> Result<Vec<ChannelToken>, String> {
    match channel_type {
        ChannelType::NewApi => fetch_new_api_tokens(&base_url, &username, &password).await,
        ChannelType::Sub2Api => fetch_sub2api_tokens(&base_url, &username, &password).await,
    }
}

/// Fetches tokens from a New API channel
async fn fetch_new_api_tokens(
    base_url: &str,
    username: &str,
    password: &str,
) -> Result<Vec<ChannelToken>, String> {
    log::debug!("Fetching tokens from New API: {base_url}");

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

    // Fetch tokens with pagination
    let url = format!("{base}/api/token/");
    let page_size: usize = 100;
    let mut all_tokens: Vec<ChannelToken> = Vec::new();
    let mut page: usize = 0;

    loop {
        let response = client
            .get(&url)
            .header("New-Api-User", user_id.to_string())
            .query(&[
                ("p", page.to_string()),
                ("page_size", page_size.to_string()),
            ])
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
                            platform: None, // New API doesn't provide platform info
                            group_name: None,
                        })
                    })
                    .collect()
            })
            .unwrap_or_default();

        let count = tokens.len();
        all_tokens.extend(tokens);

        if count < page_size {
            break;
        }
        page += 1;
    }

    log::info!("Fetched {} tokens", all_tokens.len());
    Ok(all_tokens)
}

/// Fetches tokens from a Sub2API channel
async fn fetch_sub2api_tokens(
    base_url: &str,
    email: &str,
    password: &str,
) -> Result<Vec<ChannelToken>, String> {
    log::debug!("Fetching tokens from Sub2API: {base_url}");

    let client = reqwest::Client::new();
    let base = base_url.trim_end_matches('/');

    // Login to get JWT access token
    let login_url = format!("{base}/api/v1/auth/login");
    let login_response = client
        .post(&login_url)
        .json(&serde_json::json!({
            "email": email,
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

    if login_data.get("code").and_then(|v| v.as_i64()) != Some(0) {
        let msg = login_data
            .get("message")
            .and_then(|v| v.as_str())
            .unwrap_or("Unknown error");
        return Err(format!("Login failed: {msg}"));
    }

    let access_token = login_data
        .get("data")
        .and_then(|d| d.get("access_token"))
        .and_then(|t| t.as_str())
        .ok_or("Could not get access_token from login response")?;

    log::debug!("Got Sub2API access token");

    // Fetch available groups to get platform info
    let groups_url = format!("{base}/api/v1/groups/available");
    let groups_response = client
        .get(&groups_url)
        .header("Authorization", format!("Bearer {access_token}"))
        .send()
        .await
        .map_err(|e| format!("Failed to fetch groups: {e}"))?;

    // Build group_id -> (platform, name) map
    let group_info: std::collections::HashMap<i64, (String, String)> =
        if groups_response.status().is_success() {
            let groups_data: Value = groups_response.json().await.unwrap_or_default();
            groups_data
                .get("data")
                .and_then(|d| d.as_array())
                .map(|arr| {
                    arr.iter()
                        .filter_map(|g| {
                            let id = g.get("id")?.as_i64()?;
                            let platform = g.get("platform")?.as_str()?.to_string();
                            let name = g.get("name")?.as_str()?.to_string();
                            Some((id, (platform, name)))
                        })
                        .collect()
                })
                .unwrap_or_default()
        } else {
            log::warn!("Failed to fetch groups, platform info will be unavailable");
            std::collections::HashMap::new()
        };

    log::debug!("Fetched {} groups with platform info", group_info.len());

    // Fetch keys list with pagination
    let keys_url = format!("{base}/api/v1/keys");
    let page_size: usize = 100;
    let mut all_items: Vec<Value> = Vec::new();
    let mut page: usize = 1;

    loop {
        let keys_response = client
            .get(&keys_url)
            .header("Authorization", format!("Bearer {access_token}"))
            .query(&[
                ("page", page.to_string()),
                ("page_size", page_size.to_string()),
            ])
            .send()
            .await
            .map_err(|e| format!("Failed to fetch keys: {e}"))?;

        if !keys_response.status().is_success() {
            let status = keys_response.status();
            let body = keys_response.text().await.unwrap_or_default();
            return Err(format!("API error {status}: {body}"));
        }

        let keys_data: Value = keys_response
            .json()
            .await
            .map_err(|e| format!("Failed to parse keys response: {e}"))?;

        let items: Vec<Value> = keys_data
            .get("data")
            .and_then(|d| d.get("items"))
            .and_then(|v| v.as_array())
            .cloned()
            .unwrap_or_default();

        let count = items.len();
        all_items.extend(items);

        if count < page_size {
            break;
        }
        page += 1;
    }

    // Extract key IDs for usage query
    let key_ids: Vec<i64> = all_items
        .iter()
        .filter_map(|k| k.get("id").and_then(|id| id.as_i64()))
        .collect();

    // Fetch usage stats
    let usage_url = format!("{base}/api/v1/usage/dashboard/api-keys-usage");
    let usage_response = client
        .post(&usage_url)
        .header("Authorization", format!("Bearer {access_token}"))
        .json(&serde_json::json!({ "api_key_ids": key_ids }))
        .send()
        .await
        .map_err(|e| format!("Failed to fetch usage: {e}"))?;

    let usage_stats: std::collections::HashMap<String, Value> =
        if usage_response.status().is_success() {
            let usage_data: Value = usage_response.json().await.unwrap_or_default();
            usage_data
                .get("data")
                .and_then(|d| d.get("stats"))
                .and_then(|s| serde_json::from_value(s.clone()).ok())
                .unwrap_or_default()
        } else {
            std::collections::HashMap::new()
        };

    // Build tokens list
    let tokens: Vec<ChannelToken> = all_items
        .iter()
        .filter_map(|k| {
            let id = k.get("id")?.as_f64()?;
            let id_str = (id as i64).to_string();
            let usage = usage_stats.get(&id_str);

            let status_str = k
                .get("status")
                .and_then(|s| s.as_str())
                .unwrap_or("unknown");
            let status = match status_str {
                "active" => 1,
                "inactive" => 2,
                _ => 0,
            };

            // Get platform and group_name from embedded group object or fallback to group_info map
            let (platform, group_name) = k
                .get("group")
                .map(|g| {
                    let platform = g.get("platform").and_then(|p| p.as_str()).map(String::from);
                    let name = g.get("name").and_then(|n| n.as_str()).map(String::from);
                    (platform, name)
                })
                .unwrap_or_else(|| {
                    // Fallback to group_info map if group object not embedded
                    k.get("group_id")
                        .and_then(|g| g.as_i64())
                        .and_then(|group_id| group_info.get(&group_id).cloned())
                        .map(|(platform, name)| (Some(platform), Some(name)))
                        .unwrap_or((None, None))
                });

            Some(ChannelToken {
                id,
                name: k.get("name")?.as_str()?.to_string(),
                key: k.get("key")?.as_str()?.to_string(),
                status,
                remain_quota: 0.0, // Sub2API doesn't have quota concept
                used_quota: usage
                    .and_then(|u| u.get("total_actual_cost"))
                    .and_then(|v| v.as_f64())
                    .unwrap_or(0.0),
                unlimited_quota: true,
                platform,
                group_name,
            })
        })
        .collect();

    log::info!("Fetched {} tokens from Sub2API", tokens.len());
    Ok(tokens)
}

/// Fetches models using an API key (for quick model addition from channels)
#[tauri::command]
#[specta::specta]
pub async fn fetch_models_by_api_key(
    base_url: String,
    api_key: String,
    platform: Option<String>,
) -> Result<Vec<ModelInfo>, String> {
    log::debug!(
        "Fetching models from {base_url} for platform {:?}",
        platform
    );

    let trimmed_base = base_url.trim_end_matches('/');
    let client = reqwest::Client::new();

    // Handle antigravity platform - fetch Claude models only
    if platform.as_deref() == Some("antigravity") {
        let claude_url = format!("{trimmed_base}/antigravity/v1/models");

        let response = client
            .get(&claude_url)
            .header("Authorization", format!("Bearer {api_key}"))
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

        let models = parse_openai_models(&data);
        log::info!("Fetched {} models from antigravity", models.len());
        return Ok(models);
    }

    let (url, parser): (String, fn(&Value) -> Vec<ModelInfo>) = match platform.as_deref() {
        Some("gemini") => (format!("{trimmed_base}/v1beta/models"), parse_gemini_models),
        Some("openai") => (format!("{trimmed_base}/v1/models"), parse_openai_models),
        _ => (format!("{trimmed_base}/v1/models"), parse_openai_models),
    };

    log::debug!("Requesting models from {url}");

    let response = client
        .get(&url)
        .header("Authorization", format!("Bearer {api_key}"))
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

    let models = parser(&data);
    log::info!("Fetched {} models", models.len());
    Ok(models)
}

fn parse_openai_models(data: &Value) -> Vec<ModelInfo> {
    data.get("data")
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|m| {
                    let id = m.get("id")?.as_str()?.to_string();
                    Some(ModelInfo { id, name: None })
                })
                .collect()
        })
        .unwrap_or_default()
}

fn parse_gemini_models(data: &Value) -> Vec<ModelInfo> {
    data.get("models")
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|m| {
                    let raw_id = m.get("name")?.as_str()?;
                    // Strip "models/" prefix if present
                    let id = raw_id.strip_prefix("models/").unwrap_or(raw_id).to_string();
                    let display_name = m
                        .get("displayName")
                        .and_then(|n| n.as_str())
                        .map(String::from);
                    Some(ModelInfo {
                        id,
                        name: display_name,
                    })
                })
                .collect()
        })
        .unwrap_or_default()
}
