//! Shared Playground plumbing.
//!
//! The Playground pages (video, image) talk to the same channel store and the
//! same New API compatible gateways, so credential resolution and the small
//! HTTP helpers live here instead of being copied into every protocol module.
//!
//! Each page keeps its own model/task/secrets files under
//! `~/.droidgear/playground/`; only the layout helpers are shared.

use serde::Serialize;
use serde_json::Value;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Mutex, OnceLock};
use std::time::{Duration, Instant};

use crate::channel::{self, Channel, ChannelToken, ChannelType};

const TOKEN_CACHE_TTL: Duration = Duration::from_secs(300);
const MAX_ERROR_BODY_CHARS: usize = 500;

// ============================================================================
// Paths and files
// ============================================================================

pub(crate) fn playground_dir(home: &Path) -> PathBuf {
    crate::paths::droidgear_dir_from_home(home).join("playground")
}

/// Directory the pages use for media they download or decode. The Tauri asset
/// protocol is scoped here.
pub(crate) fn media_dir(home: &Path) -> PathBuf {
    playground_dir(home).join("media")
}

/// Reads a JSON string map used to keep custom-endpoint secrets out of the
/// model/task files. Missing and corrupt files both read as empty: a broken
/// secrets file must not make the rest of the Playground unusable.
pub(crate) fn read_secrets(path: &Path, label: &str) -> HashMap<String, String> {
    if !path.exists() {
        return HashMap::new();
    }
    match fs::read_to_string(path) {
        Ok(content) => match serde_json::from_str::<HashMap<String, String>>(&content) {
            Ok(secrets) => secrets,
            Err(error) => {
                log::warn!("{label}: failed to parse secrets file: {error}");
                HashMap::new()
            }
        },
        Err(error) => {
            log::warn!("{label}: failed to read secrets file: {error}");
            HashMap::new()
        }
    }
}

/// Writes a JSON string map owner-only. An empty map removes the file, so
/// deleting the last custom model also removes its secrets.
pub(crate) fn write_secrets(
    path: &Path,
    secrets: &HashMap<String, String>,
    label: &str,
) -> Result<(), String> {
    if secrets.is_empty() {
        if path.exists() {
            fs::remove_file(path)
                .map_err(|error| format!("Failed to remove {label} secrets file: {error}"))?;
        }
        return Ok(());
    }
    let content = serde_json::to_string_pretty(secrets)
        .map_err(|error| format!("Failed to serialize {label} secrets: {error}"))?;
    crate::storage::atomic_write_private(path, content.as_bytes())
        .map_err(|error| format!("Failed to write {label} secrets: {error}"))
}

/// Writes any serializable value as pretty JSON through the atomic writer.
pub(crate) fn write_json<T: Serialize>(path: &Path, value: &T, label: &str) -> Result<(), String> {
    let content = serde_json::to_string_pretty(value)
        .map_err(|error| format!("Failed to serialize {label}: {error}"))?;
    crate::storage::atomic_write(path, content.as_bytes())
        .map_err(|error| format!("Failed to write {label}: {error}"))
}

// ============================================================================
// Channel credential resolution
// ============================================================================

struct CachedTokenKey {
    key: String,
    fetched_at: Instant,
}

static TOKEN_CACHE: OnceLock<Mutex<HashMap<String, CachedTokenKey>>> = OnceLock::new();

/// Endpoint and credentials a protocol client needs for one request.
pub(crate) struct ResolvedSource {
    pub base_url: String,
    pub api_key: String,
    pub channel_type: Option<ChannelType>,
}

fn channel_uses_api_key(channel_type: &ChannelType) -> bool {
    matches!(
        channel_type,
        ChannelType::CliProxyApi
            | ChannelType::Ollama
            | ChannelType::General
            | ChannelType::DeepSeek
    )
}

pub(crate) fn select_active_token(tokens: &[ChannelToken]) -> Option<&ChannelToken> {
    tokens
        .iter()
        .find(|token| token.status == 1 && !token.key.trim().is_empty())
        .or_else(|| tokens.iter().find(|token| !token.key.trim().is_empty()))
}

fn find_token_by_id(tokens: &[ChannelToken], id: f64) -> Option<&ChannelToken> {
    tokens.iter().find(|token| (token.id - id).abs() < 0.5)
}

fn token_cache_get(key: &str) -> Option<String> {
    let cache = TOKEN_CACHE.get_or_init(|| Mutex::new(HashMap::new()));
    let mut cache = cache.lock().ok()?;
    if let Some(entry) = cache.get(key) {
        if entry.fetched_at.elapsed() < TOKEN_CACHE_TTL {
            return Some(entry.key.clone());
        }
        cache.remove(key);
    }
    None
}

fn token_cache_put(key: String, token_key: String) {
    if let Ok(mut cache) = TOKEN_CACHE
        .get_or_init(|| Mutex::new(HashMap::new()))
        .lock()
    {
        cache.insert(
            key,
            CachedTokenKey {
                key: token_key,
                fetched_at: Instant::now(),
            },
        );
    }
}

async fn resolve_channel_api_key(
    home: &Path,
    channel: &Channel,
    token_id: Option<f64>,
) -> Result<String, String> {
    if channel_uses_api_key(&channel.channel_type) {
        let api_key = channel::get_channel_api_key_for_home(home, &channel.id)?.unwrap_or_default();
        if api_key.trim().is_empty() {
            return Err(format!(
                "Channel '{}' has no API key. Add one under Channels.",
                channel.name
            ));
        }
        return Ok(api_key);
    }

    let cache_key = format!("{}:{token_id:?}", channel.id);
    if let Some(api_key) = token_cache_get(&cache_key) {
        return Ok(api_key);
    }

    let (username, password) = channel::get_channel_credentials_for_home(home, &channel.id)?
        .ok_or_else(|| {
            format!(
                "Channel '{}' has no saved credentials. Add them under Channels.",
                channel.name
            )
        })?;
    let tokens = channel::fetch_channel_tokens(
        channel.channel_type.clone(),
        &channel.base_url,
        &username,
        &password,
    )
    .await?;

    let token = match token_id {
        Some(id) => find_token_by_id(&tokens, id).ok_or_else(|| {
            "The token selected for this model no longer exists. Re-import the model or choose another token."
                .to_string()
        })?,
        None => select_active_token(&tokens)
            .ok_or_else(|| format!("Channel '{}' has no active token.", channel.name))?,
    };
    let api_key = token.key.trim().to_string();
    if api_key.is_empty() {
        return Err(format!(
            "Channel '{}' returned an empty token key.",
            channel.name
        ));
    }
    token_cache_put(cache_key, api_key.clone());
    Ok(api_key)
}

/// Resolves an existing channel plus an optional token id from the channel
/// store. The channel is read at request time, so rotating a token or editing
/// the base URL needs no re-import.
pub(crate) async fn resolve_channel_source(
    channel_id: &str,
    token_id: Option<f64>,
) -> Result<ResolvedSource, String> {
    let home = crate::paths::get_home_dir()?;
    let channels = channel::load_channels_for_home(&home)?;
    let channel = channels
        .iter()
        .find(|item| item.id == channel_id)
        .ok_or_else(|| {
            "The channel used by this model no longer exists. Re-create the model.".to_string()
        })?;
    if !channel.enabled {
        return Err(format!(
            "Channel '{}' is disabled. Enable it or choose another channel.",
            channel.name
        ));
    }
    let api_key = resolve_channel_api_key(&home, channel, token_id).await?;
    Ok(ResolvedSource {
        base_url: channel.base_url.clone(),
        api_key,
        channel_type: Some(channel.channel_type.clone()),
    })
}

/// Resolves a standalone endpoint that keeps its own API key.
pub(crate) fn resolve_custom_source(
    base_url: &str,
    api_key: &str,
) -> Result<ResolvedSource, String> {
    let base_url = base_url.trim();
    if base_url.is_empty() {
        return Err("This model has no base URL. Edit it in Manage models.".to_string());
    }
    Ok(ResolvedSource {
        base_url: base_url.to_string(),
        api_key: api_key.trim().to_string(),
        channel_type: None,
    })
}

// ============================================================================
// HTTP helpers
// ============================================================================

pub(crate) fn build_client(timeout: Duration) -> Result<reqwest::Client, String> {
    reqwest::Client::builder()
        .timeout(timeout)
        .build()
        .map_err(|error| format!("Failed to create HTTP client: {error}"))
}

pub(crate) fn authorized(
    request: reqwest::RequestBuilder,
    api_key: &str,
) -> reqwest::RequestBuilder {
    if api_key.trim().is_empty() {
        request
    } else {
        request.bearer_auth(api_key)
    }
}

pub(crate) fn snippet(body: &str) -> String {
    let trimmed = body.trim();
    if trimmed.is_empty() {
        return "(empty response)".to_string();
    }
    let mut chars = trimmed.chars();
    let prefix: String = chars.by_ref().take(MAX_ERROR_BODY_CHARS).collect();
    if chars.next().is_some() {
        format!("{prefix}…")
    } else {
        prefix
    }
}

pub(crate) fn api_error_message(body: &str) -> Option<String> {
    let value: Value = serde_json::from_str(body).ok()?;
    if let Some(object) = value.as_object() {
        for key in ["error", "message", "msg", "detail"] {
            if let Some(message) = object.get(key).and_then(value_message) {
                return Some(message);
            }
        }
    }
    value_message(&value)
}

pub(crate) fn value_message(value: &Value) -> Option<String> {
    match value {
        Value::String(text) => {
            let text = text.trim();
            (!text.is_empty()).then(|| text.to_string())
        }
        Value::Object(object) => {
            let message = object
                .get("message")
                .and_then(Value::as_str)
                .map(str::trim)
                .filter(|text| !text.is_empty());
            let code = object
                .get("code")
                .and_then(value_to_string)
                .filter(|code| code != "0");
            match (message, code) {
                (Some(message), Some(code)) => Some(format!("{message} ({code})")),
                (Some(message), None) => Some(message.to_string()),
                (None, Some(code)) => Some(code),
                (None, None) => None,
            }
        }
        _ => None,
    }
}

fn value_to_string(value: &Value) -> Option<String> {
    match value {
        Value::String(text) => {
            let text = text.trim();
            (!text.is_empty()).then(|| text.to_string())
        }
        Value::Number(number) => Some(number.to_string()),
        _ => None,
    }
}

pub(crate) fn format_api_error(status: reqwest::StatusCode, body: &str) -> String {
    let message = api_error_message(body).unwrap_or_else(|| snippet(body));
    format!("HTTP {}: {message}", status.as_u16())
}

pub(crate) async fn send_json(request: reqwest::RequestBuilder) -> Result<Value, String> {
    let response = request
        .send()
        .await
        .map_err(|error| format!("Request failed: {error}"))?;
    let status = response.status();
    let body = response
        .text()
        .await
        .map_err(|error| format!("Failed to read response body: {error}"))?;
    if !status.is_success() {
        return Err(format_api_error(status, &body));
    }
    serde_json::from_str(&body).map_err(|error| {
        format!(
            "Failed to parse response as JSON: {error} (body: {})",
            snippet(&body)
        )
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn api_error_message_reads_nested_openai_and_gemini_errors() {
        assert_eq!(
            api_error_message(r#"{"error":{"message":"bad model","code":"invalid"}}"#),
            Some("bad model (invalid)".to_string())
        );
        assert_eq!(
            api_error_message(r#"{"error":{"code":400,"message":"API key not valid"}}"#),
            Some("API key not valid (400)".to_string())
        );
        assert_eq!(
            api_error_message(r#"{"message":"plain"}"#),
            Some("plain".to_string())
        );
        assert_eq!(api_error_message("not json"), None);
    }

    #[test]
    fn snippet_truncates_long_bodies() {
        let body = "x".repeat(MAX_ERROR_BODY_CHARS + 10);
        let truncated = snippet(&body);
        assert_eq!(truncated.chars().count(), MAX_ERROR_BODY_CHARS + 1);
        assert!(truncated.ends_with('…'));
        assert_eq!(snippet("  "), "(empty response)");
    }

    #[test]
    fn media_dir_lives_under_the_playground_directory() {
        let home = Path::new("/home/tester");
        let dir = media_dir(home);
        assert!(dir.ends_with("playground/media"));
        assert_eq!(dir, playground_dir(home).join("media"));
    }

    fn token(id: f64, key: &str, status: i32) -> ChannelToken {
        ChannelToken {
            id,
            name: format!("token-{id}"),
            key: key.to_string(),
            status,
            remain_quota: 0.0,
            used_quota: 0.0,
            unlimited_quota: true,
            platform: None,
            group_name: None,
        }
    }

    #[test]
    fn token_selection_prefers_an_enabled_token() {
        let tokens = vec![token(1.0, "sk-disabled", 2), token(2.0, "sk-active", 1)];
        assert_eq!(
            select_active_token(&tokens).map(|item| item.key.as_str()),
            Some("sk-active")
        );
        assert_eq!(
            find_token_by_id(&tokens, 2.0).map(|item| item.key.as_str()),
            Some("sk-active")
        );
        assert!(find_token_by_id(&tokens, 99.0).is_none());
    }

    #[test]
    fn token_selection_falls_back_to_any_non_empty_key() {
        let tokens = vec![token(1.0, "sk-idle", 2), token(2.0, "   ", 1)];
        assert_eq!(
            select_active_token(&tokens).map(|item| item.key.as_str()),
            Some("sk-idle")
        );
    }
}
