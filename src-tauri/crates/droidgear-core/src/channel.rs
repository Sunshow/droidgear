//! API Channel management (core).
//!
//! Handles channel configuration and token management for New API, Sub2API, Ollama, etc.

use serde::{Deserialize, Serialize};
use serde_json::Value;
use specta::Type;
use std::fs;
use std::path::{Path, PathBuf};

use crate::factory_settings::ModelInfo;

// ============================================================================
// Types
// ============================================================================

/// Channel types supported
#[derive(Debug, Clone, Serialize, Deserialize, Type, PartialEq)]
#[serde(rename_all = "kebab-case")]
#[allow(clippy::enum_variant_names)]
pub enum ChannelType {
    NewApi,
    #[serde(rename = "sub-2-api")]
    Sub2Api,
    CliProxyApi,
    Ollama,
    General,
    #[serde(rename = "deep-seek")]
    DeepSeek,
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
    ApiKey { api_key: String },
}

// ============================================================================
// Paths
// ============================================================================

fn droidgear_dir_for_home(home_dir: &Path) -> PathBuf {
    home_dir.join(".droidgear")
}

fn channels_path_for_home(home_dir: &Path) -> Result<PathBuf, String> {
    let dir = droidgear_dir_for_home(home_dir);
    if !dir.exists() {
        fs::create_dir_all(&dir)
            .map_err(|e| format!("Failed to create .droidgear directory: {e}"))?;
    }
    Ok(dir.join("channels.json"))
}

fn auth_dir_for_home(home_dir: &Path) -> PathBuf {
    droidgear_dir_for_home(home_dir).join("auth")
}

fn auth_file_path_for_home(home_dir: &Path, channel_id: &str) -> PathBuf {
    auth_dir_for_home(home_dir).join(format!("{channel_id}.json"))
}

// ============================================================================
// File helpers
// ============================================================================

fn read_channels_from_file(path: &Path) -> Result<Vec<Channel>, String> {
    let content =
        fs::read_to_string(path).map_err(|e| format!("Failed to read channels file: {e}"))?;
    let channels: Vec<Channel> = serde_json::from_str(&content)
        .map_err(|e| format!("Failed to parse channels file: {e}"))?;
    Ok(channels)
}

fn write_channels_to_file(path: &Path, channels: &[Channel]) -> Result<(), String> {
    let content = serde_json::to_string_pretty(channels)
        .map_err(|e| format!("Failed to serialize channels: {e}"))?;
    fs::write(path, content).map_err(|e| format!("Failed to write channels file: {e}"))?;
    Ok(())
}

fn read_channel_auth_for_home(
    home_dir: &Path,
    channel_id: &str,
) -> Result<Option<ChannelAuth>, String> {
    let path = auth_file_path_for_home(home_dir, channel_id);
    if !path.exists() {
        return Ok(None);
    }
    let content =
        fs::read_to_string(&path).map_err(|e| format!("Failed to read auth file: {e}"))?;
    let auth: ChannelAuth =
        serde_json::from_str(&content).map_err(|e| format!("Failed to parse auth file: {e}"))?;
    Ok(Some(auth))
}

fn write_channel_auth_for_home(
    home_dir: &Path,
    channel_id: &str,
    auth: &ChannelAuth,
) -> Result<(), String> {
    let dir = auth_dir_for_home(home_dir);
    fs::create_dir_all(&dir).map_err(|e| format!("Failed to create auth directory: {e}"))?;
    let path = auth_file_path_for_home(home_dir, channel_id);
    let content =
        serde_json::to_string_pretty(auth).map_err(|e| format!("Failed to serialize auth: {e}"))?;
    fs::write(&path, content).map_err(|e| format!("Failed to write auth file: {e}"))?;
    Ok(())
}

fn delete_channel_auth_for_home(home_dir: &Path, channel_id: &str) -> Result<(), String> {
    let path = auth_file_path_for_home(home_dir, channel_id);
    if path.exists() {
        fs::remove_file(&path).map_err(|e| format!("Failed to delete auth file: {e}"))?;
    }
    Ok(())
}

// ============================================================================
// Public API (CRUD)
// ============================================================================

/// Loads all channels from ~/.droidgear/channels.json
/// Falls back to Factory settings.json for migration
pub fn load_channels_for_home(home_dir: &Path) -> Result<Vec<Channel>, String> {
    let droidgear_path = channels_path_for_home(home_dir)?;
    log::info!("Channel: channels file path: {}", droidgear_path.display());

    if droidgear_path.exists() {
        log::info!("Channel: channels file exists, reading...");
        let channels = read_channels_from_file(&droidgear_path)?;
        log::info!("Channel: loaded {} channels", channels.len());
        return Ok(channels);
    }

    log::info!("Channel: channels file not found, attempting migration from Factory settings");

    // Migration: ~/.factory/settings.json -> channels array
    let factory_settings_path = {
        let config_paths = crate::paths::load_config_paths_for_home(home_dir);
        let factory_dir = crate::paths::get_factory_home_for_home(home_dir, &config_paths)?;
        factory_dir.join("settings.json")
    };

    if factory_settings_path.exists() {
        let s = fs::read_to_string(&factory_settings_path)
            .map_err(|e| format!("Failed to read config file: {e}"))?;
        if !s.trim().is_empty() {
            if let Ok(config) = serde_json::from_str::<Value>(&s) {
                if let Some(channels_value) = config.get("channels") {
                    if let Some(arr) = channels_value.as_array() {
                        if !arr.is_empty() {
                            let channels: Vec<Channel> = arr
                                .iter()
                                .filter_map(|v| serde_json::from_value(v.clone()).ok())
                                .collect();

                            if !channels.is_empty() {
                                log::info!(
                                    "Channel: migrated {} channels from Factory settings",
                                    channels.len()
                                );
                                write_channels_to_file(&droidgear_path, &channels)?;
                            }
                            return Ok(channels);
                        }
                    }
                }
            }
        }
    }

    log::info!("Channel: no channels found (empty)");
    Ok(vec![])
}

pub fn load_channels() -> Result<Vec<Channel>, String> {
    let home = crate::paths::get_home_dir()?;
    log::info!("Channel: loading channels from home: {}", home.display());
    load_channels_for_home(&home)
}

pub fn save_channels_for_home(home_dir: &Path, channels: Vec<Channel>) -> Result<(), String> {
    let path = channels_path_for_home(home_dir)?;
    write_channels_to_file(&path, &channels)
}

pub fn save_channels(channels: Vec<Channel>) -> Result<(), String> {
    save_channels_for_home(&crate::paths::get_home_dir()?, channels)
}

pub fn save_channel_credentials_for_home(
    home_dir: &Path,
    channel_id: &str,
    username: &str,
    password: &str,
) -> Result<(), String> {
    let auth = ChannelAuth::Credentials {
        username: username.to_string(),
        password: password.to_string(),
    };
    write_channel_auth_for_home(home_dir, channel_id, &auth)
}

pub fn save_channel_credentials(
    channel_id: &str,
    username: &str,
    password: &str,
) -> Result<(), String> {
    save_channel_credentials_for_home(
        &crate::paths::get_home_dir()?,
        channel_id,
        username,
        password,
    )
}

pub fn get_channel_credentials_for_home(
    home_dir: &Path,
    channel_id: &str,
) -> Result<Option<(String, String)>, String> {
    match read_channel_auth_for_home(home_dir, channel_id)? {
        Some(ChannelAuth::Credentials { username, password }) => Ok(Some((username, password))),
        _ => Ok(None),
    }
}

pub fn get_channel_credentials(channel_id: &str) -> Result<Option<(String, String)>, String> {
    get_channel_credentials_for_home(&crate::paths::get_home_dir()?, channel_id)
}

pub fn save_channel_api_key_for_home(
    home_dir: &Path,
    channel_id: &str,
    api_key: &str,
) -> Result<(), String> {
    let auth = ChannelAuth::ApiKey {
        api_key: api_key.to_string(),
    };
    write_channel_auth_for_home(home_dir, channel_id, &auth)
}

pub fn save_channel_api_key(channel_id: &str, api_key: &str) -> Result<(), String> {
    save_channel_api_key_for_home(&crate::paths::get_home_dir()?, channel_id, api_key)
}

pub fn get_channel_api_key_for_home(
    home_dir: &Path,
    channel_id: &str,
) -> Result<Option<String>, String> {
    match read_channel_auth_for_home(home_dir, channel_id)? {
        Some(ChannelAuth::ApiKey { api_key }) => Ok(Some(api_key)),
        _ => Ok(None),
    }
}

pub fn get_channel_api_key(channel_id: &str) -> Result<Option<String>, String> {
    get_channel_api_key_for_home(&crate::paths::get_home_dir()?, channel_id)
}

pub fn delete_channel_credentials_for_home(
    home_dir: &Path,
    channel_id: &str,
) -> Result<(), String> {
    delete_channel_auth_for_home(home_dir, channel_id)
}

pub fn delete_channel_credentials(channel_id: &str) -> Result<(), String> {
    delete_channel_credentials_for_home(&crate::paths::get_home_dir()?, channel_id)
}

// ============================================================================
// Network features
// ============================================================================

pub async fn detect_channel_type(base_url: &str) -> Result<ChannelType, String> {
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(10))
        .build()
        .map_err(|e| format!("Failed to create HTTP client: {e}"))?;

    let base = base_url.trim_end_matches('/');

    // 1. Ollama: GET / contains "Ollama"
    if let Ok(resp) = client.get(base).send().await {
        if resp.status().is_success() {
            if let Ok(text) = resp.text().await {
                if text.contains("Ollama") || text.contains("ollama") {
                    return Ok(ChannelType::Ollama);
                }
            }
        }
    }

    // 2. Sub2API: GET /health -> {"status":"ok"}
    if let Ok(resp) = client.get(format!("{base}/health")).send().await {
        if resp.status().is_success() {
            if let Ok(data) = resp.json::<Value>().await {
                if data.get("status").and_then(|s| s.as_str()) == Some("ok") {
                    return Ok(ChannelType::Sub2Api);
                }
            }
        }
    }

    // 3. New API: GET /api/status
    if let Ok(resp) = client.get(format!("{base}/api/status")).send().await {
        if resp.status().is_success() {
            return Ok(ChannelType::NewApi);
        }
    }

    // 3.5. DeepSeek: GET /models returns OpenAI format with owned_by field
    if let Ok(resp) = client.get(format!("{base}/models")).send().await {
        if resp.status().is_success() {
            if let Ok(data) = resp.json::<Value>().await {
                if let Some(arr) = data.get("data").and_then(|d| d.as_array()) {
                    if arr.iter().any(|m| {
                        m.get("id").is_some()
                            && m.get("owned_by").and_then(|v| v.as_str()) == Some("deepseek")
                    }) {
                        return Ok(ChannelType::DeepSeek);
                    }
                }
            }
        }
    }

    // 4. CLI Proxy API: GET /v1/models returns OpenAI format
    if let Ok(resp) = client.get(format!("{base}/v1/models")).send().await {
        if resp.status().is_success() {
            if let Ok(data) = resp.json::<Value>().await {
                if let Some(arr) = data.get("data").and_then(|d| d.as_array()) {
                    if arr.iter().any(|m| m.get("id").is_some()) {
                        return Ok(ChannelType::CliProxyApi);
                    }
                }
            }
        }
    }

    Err("Unable to auto-detect channel type".to_string())
}

pub async fn fetch_channel_tokens(
    channel_type: ChannelType,
    base_url: &str,
    username: &str,
    password: &str,
) -> Result<Vec<ChannelToken>, String> {
    match channel_type {
        ChannelType::NewApi => fetch_new_api_keys(base_url, username, password).await,
        ChannelType::Sub2Api => fetch_sub2api_tokens(base_url, username, password).await,
        ChannelType::CliProxyApi
        | ChannelType::Ollama
        | ChannelType::General
        | ChannelType::DeepSeek => Ok(vec![ChannelToken {
            id: 0.0,
            name: "API Key".to_string(),
            key: password.to_string(),
            status: 1,
            remain_quota: 0.0,
            used_quota: 0.0,
            unlimited_quota: true,
            platform: None,
            group_name: None,
        }]),
    }
}

/// Blocking version of `fetch_channel_tokens`.
/// Creates a lightweight tokio runtime and blocks on the async version.
pub fn fetch_channel_tokens_blocking(
    channel_type: ChannelType,
    base_url: &str,
    username: &str,
    password: &str,
) -> Result<Vec<ChannelToken>, String> {
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .map_err(|e| format!("Failed to create tokio runtime: {e}"))?;
    runtime.block_on(fetch_channel_tokens(
        channel_type,
        base_url,
        username,
        password,
    ))
}

async fn fetch_new_api_keys(
    base_url: &str,
    username: &str,
    password: &str,
) -> Result<Vec<ChannelToken>, String> {
    let client = reqwest::Client::builder()
        .cookie_store(true)
        .build()
        .map_err(|e| format!("Failed to create HTTP client: {e}"))?;

    let base = base_url.trim_end_matches('/');

    // Login
    let login_url = format!("{base}/api/user/login");
    let login_response = client
        .post(&login_url)
        .json(&serde_json::json!({ "username": username, "password": password }))
        .send()
        .await
        .map_err(|e| format!("Failed to login: {e}"))?;

    if !login_response.status().is_success() {
        let status = login_response.status();
        let body = login_response.text().await.unwrap_or_default();
        return Err(format!("Login failed {status}: {body}"));
    }

    let login_data: Value = {
        let body = login_response
            .text()
            .await
            .map_err(|e| format!("Failed to read login response body: {e}"))?;
        serde_json::from_str(&body).map_err(|e| {
            let truncated = if body.len() > 500 {
                format!("{}...", &body[..500])
            } else {
                body.clone()
            };
            log::warn!("Channel: failed to parse login response, body={truncated}");
            format!("Failed to parse login response: {e}")
        })?
    };

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
        .and_then(|v| v.as_i64())
        .ok_or("Could not get user id from login response")?;

    // Fetch tokens with pagination
    let keys_url = format!("{base}/api/token");
    let page_size: usize = 100;
    let mut all_keys: Vec<ChannelToken> = Vec::new();
    let mut page: usize = 1;

    loop {
        let response = client
            .get(&keys_url)
            .header("New-Api-User", user_id.to_string())
            .query(&[("p", page.to_string()), ("size", page_size.to_string())])
            .send()
            .await
            .map_err(|e| format!("Failed to fetch keys: {e}"))?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(format!("API error {status}: {body}"));
        }

        let data: Value = {
            let body = response
                .text()
                .await
                .map_err(|e| format!("Failed to read response body: {e}"))?;
            serde_json::from_str(&body).map_err(|e| {
                let truncated = if body.len() > 500 {
                    format!("{}...", &body[..500])
                } else {
                    body.clone()
                };
                log::warn!("Channel: failed to parse keys response, body={truncated}");
                format!("Failed to parse response: {e}")
            })?
        };

        let items: Vec<Value> = data
            .get("data")
            .and_then(|d| d.get("items"))
            .and_then(|v| v.as_array())
            .cloned()
            .unwrap_or_default();

        let count = items.len();

        for t in &items {
            let token_id = match t.get("id").and_then(|v| v.as_i64()) {
                Some(id) => id,
                None => continue,
            };

            // Fetch unmasked key via POST /api/token/{id}/key
            let raw_key = match client
                .post(format!("{base}/api/token/{token_id}/key"))
                .header("New-Api-User", user_id.to_string())
                .send()
                .await
            {
                Ok(resp) if resp.status().is_success() => {
                    resp.json::<Value>().await.ok().and_then(|v| {
                        v.get("data")
                            .and_then(|d| d.get("key"))
                            .and_then(|k| k.as_str())
                            .map(String::from)
                    })
                }
                _ => None,
            };

            let key = match raw_key {
                Some(k) if k.starts_with("sk-") => k,
                Some(k) => format!("sk-{k}"),
                None => {
                    // Fallback to masked key from list
                    match t.get("key").and_then(|v| v.as_str()) {
                        Some(k) if k.starts_with("sk-") => k.to_string(),
                        Some(k) => format!("sk-{k}"),
                        None => continue,
                    }
                }
            };

            if let (Some(name), Some(status)) = (
                t.get("name").and_then(|v| v.as_str()),
                t.get("status").and_then(|v| v.as_i64()),
            ) {
                all_keys.push(ChannelToken {
                    id: token_id as f64,
                    name: name.to_string(),
                    key,
                    status: status as i32,
                    remain_quota: t
                        .get("remain_quota")
                        .and_then(|v| v.as_f64())
                        .unwrap_or(0.0),
                    used_quota: t.get("used_quota").and_then(|v| v.as_f64()).unwrap_or(0.0),
                    unlimited_quota: t
                        .get("unlimited_quota")
                        .and_then(|v| v.as_bool())
                        .unwrap_or(false),
                    platform: None,
                    group_name: None,
                });
            }
        }

        if count < page_size {
            break;
        }
        page += 1;
    }

    Ok(all_keys)
}

async fn fetch_sub2api_tokens(
    base_url: &str,
    email: &str,
    password: &str,
) -> Result<Vec<ChannelToken>, String> {
    let client = reqwest::Client::new();
    let base = base_url.trim_end_matches('/');

    // Login
    let login_url = format!("{base}/api/v1/auth/login");
    let login_response = client
        .post(&login_url)
        .json(&serde_json::json!({ "email": email, "password": password }))
        .send()
        .await
        .map_err(|e| format!("Failed to login: {e}"))?;

    if !login_response.status().is_success() {
        let status = login_response.status();
        let body = login_response.text().await.unwrap_or_default();
        return Err(format!("Login failed {status}: {body}"));
    }

    let login_data: Value = {
        let body = login_response
            .text()
            .await
            .map_err(|e| format!("Failed to read login response body: {e}"))?;
        serde_json::from_str(&body).map_err(|e| {
            let truncated = if body.len() > 500 {
                format!("{}...", &body[..500])
            } else {
                body.clone()
            };
            log::warn!("Channel: failed to parse login response (v1), body={truncated}");
            format!("Failed to parse login response: {e}")
        })?
    };

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

    // Fetch groups for platform info
    let groups_url = format!("{base}/api/v1/groups/available");
    let groups_response = client
        .get(&groups_url)
        .header("Authorization", format!("Bearer {access_token}"))
        .send()
        .await
        .map_err(|e| format!("Failed to fetch groups: {e}"))?;

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
            std::collections::HashMap::new()
        };

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

        let keys_data: Value = {
            let body = keys_response
                .text()
                .await
                .map_err(|e| format!("Failed to read keys response body: {e}"))?;
            serde_json::from_str(&body).map_err(|e| {
                let truncated = if body.len() > 500 {
                    format!("{}...", &body[..500])
                } else {
                    body.clone()
                };
                log::warn!("Channel: failed to parse keys response, body={truncated}");
                format!("Failed to parse keys response: {e}")
            })?
        };

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

    // Fetch usage (optional)
    let usage_url = format!("{base}/api/v1/keys/usage");
    let usage_response = client
        .get(&usage_url)
        .header("Authorization", format!("Bearer {access_token}"))
        .send()
        .await
        .ok();

    let usage_data: Option<Value> = match usage_response {
        Some(resp) if resp.status().is_success() => resp.json::<Value>().await.ok(),
        _ => None,
    };

    let usage_map: std::collections::HashMap<i64, Value> = usage_data
        .as_ref()
        .and_then(|v| v.get("data"))
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|u| Some((u.get("key_id")?.as_i64()?, u.clone())))
                .collect()
        })
        .unwrap_or_default();

    let tokens: Vec<ChannelToken> = all_items
        .iter()
        .filter_map(|k| {
            let id = k.get("id")?.as_f64()?;
            let status = k.get("status").and_then(|v| v.as_i64()).unwrap_or(1) as i32;

            let usage = k
                .get("id")
                .and_then(|v| v.as_i64())
                .and_then(|key_id| usage_map.get(&key_id));

            let (platform, group_name) = k
                .get("group")
                .map(|g| {
                    let platform = g.get("platform").and_then(|p| p.as_str()).map(String::from);
                    let name = g.get("name").and_then(|n| n.as_str()).map(String::from);
                    (platform, name)
                })
                .unwrap_or_else(|| {
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
                remain_quota: 0.0,
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

    Ok(tokens)
}

/// Candidate model-listing URLs for a platform, tried in order.
///
/// DeepSeek 官方在裸路径提供 `/models`，而 sub2api 等网关把同一接口挂在 `/v1` 下，
/// 因此 deepseek 平台两个都尝试。
fn model_endpoint_urls(trimmed_base: &str, platform: Option<&str>) -> Vec<String> {
    match platform {
        Some("gemini") => vec![format!("{trimmed_base}/v1beta/models")],
        Some("openai") => vec![format!("{trimmed_base}/v1/models")],
        Some("deepseek") => vec![
            format!("{trimmed_base}/v1/models"),
            format!("{trimmed_base}/models"),
        ],
        _ => vec![format!("{trimmed_base}/v1/models")],
    }
}

/// Fetch models using an API key (for quick model addition from channels)
pub async fn fetch_models_by_api_key(
    base_url: &str,
    api_key: &str,
    platform: Option<&str>,
) -> Result<Vec<ModelInfo>, String> {
    let trimmed_base = base_url.trim_end_matches('/');
    let client = reqwest::Client::new();

    if platform == Some("antigravity") {
        let claude_url = format!("{trimmed_base}/antigravity/v1/models");
        log::debug!("Channel: fetching models from {claude_url} (antigravity)");

        let response = client
            .get(&claude_url)
            .header("Authorization", format!("Bearer {api_key}"))
            .send()
            .await
            .map_err(|e| format!("Request failed: {e}"))?;

        let status = response.status();
        if !status.is_success() {
            let body = response.text().await.unwrap_or_default();
            log::warn!(
                "Channel: antigravity API error, url={claude_url} status={status} body={body}"
            );
            return Err(format!("API error {status}: {body}"));
        }

        let body = response
            .text()
            .await
            .map_err(|e| format!("Failed to read response body: {e}"))?;
        let data: Value = serde_json::from_str(&body).map_err(|e| {
            let truncated = if body.len() > 500 {
                format!("{}...", &body[..500])
            } else {
                body.clone()
            };
            log::warn!("Channel: failed to parse antigravity response, url={claude_url} status={status} body={truncated}");
            format!("Failed to parse response: {e}")
        })?;

        return Ok(parse_openai_models(&data));
    }

    // Candidate URLs are tried in order; the first successful response wins.
    let urls = model_endpoint_urls(trimmed_base, platform);
    let parser: fn(&Value) -> Vec<ModelInfo> = match platform {
        Some("gemini") => parse_gemini_models,
        _ => parse_openai_models,
    };

    let mut last_error = String::from("No model endpoint available");
    for url in &urls {
        log::debug!("Channel: fetching models from {url} (platform={platform:?})");

        let response = client
            .get(url)
            .header("Authorization", format!("Bearer {api_key}"))
            .send()
            .await
            .map_err(|e| format!("Request failed: {e}"))?;

        let status = response.status();
        if !status.is_success() {
            let body = response.text().await.unwrap_or_default();
            log::warn!("Channel: API error, url={url} status={status} body={body}");
            last_error = format!("API error {status}: {body}");

            // 仅在候选端点不存在时回退（401/403 等说明端点存在但鉴权失败）
            let endpoint_missing = status == reqwest::StatusCode::NOT_FOUND
                || status == reqwest::StatusCode::METHOD_NOT_ALLOWED;
            if !endpoint_missing {
                return Err(last_error);
            }
            continue;
        }

        let body = response
            .text()
            .await
            .map_err(|e| format!("Failed to read response body: {e}"))?;
        let data: Value = serde_json::from_str(&body).map_err(|e| {
            let truncated = if body.len() > 500 {
                format!("{}...", &body[..500])
            } else {
                body.clone()
            };
            log::warn!(
                "Channel: failed to parse response, url={url} status={status} body={truncated}"
            );
            format!("Failed to parse response: {e}")
        })?;

        return Ok(parser(&data));
    }

    Err(last_error)
}

pub fn fetch_models_by_api_key_blocking(
    base_url: &str,
    api_key: &str,
    platform: Option<&str>,
) -> Result<Vec<ModelInfo>, String> {
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .map_err(|e| format!("Failed to create tokio runtime: {e}"))?;
    let base_url = base_url.to_string();
    let api_key = api_key.to_string();
    let platform = platform.map(|s| s.to_string());
    runtime.block_on(async {
        let base_url = &base_url;
        let api_key = &api_key;
        let platform = platform.as_deref();
        fetch_models_by_api_key(base_url, api_key, platform).await
    })
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_model_endpoint_urls_sub2api_deepseek_falls_back_to_bare_path() {
        // sub2api 的 deepseek 分组在 /v1 下提供模型列表
        assert_eq!(
            model_endpoint_urls("https://api.example.com", Some("deepseek")),
            vec![
                "https://api.example.com/v1/models".to_string(),
                "https://api.example.com/models".to_string(),
            ]
        );
    }

    #[test]
    fn test_model_endpoint_urls_per_platform() {
        assert_eq!(
            model_endpoint_urls("https://api.example.com", Some("gemini")),
            vec!["https://api.example.com/v1beta/models".to_string()]
        );
        assert_eq!(
            model_endpoint_urls("https://api.example.com", Some("openai")),
            vec!["https://api.example.com/v1/models".to_string()]
        );
        assert_eq!(
            model_endpoint_urls("https://api.example.com", None),
            vec!["https://api.example.com/v1/models".to_string()]
        );
        assert_eq!(
            model_endpoint_urls("https://api.example.com", Some("unknown")),
            vec!["https://api.example.com/v1/models".to_string()]
        );
    }
}
