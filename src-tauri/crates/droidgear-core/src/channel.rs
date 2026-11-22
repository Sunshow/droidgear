//! API Channel management (core).
//!
//! Handles channel configuration and token management for New API, Sub2API, Ollama, etc.

use serde::{Deserialize, Serialize};
use serde_json::Value;
use specta::Type;
use std::collections::HashMap;
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

fn auth_file_path_for_home(home_dir: &Path, channel_id: &str) -> Result<PathBuf, String> {
    // The id becomes a file name, so it must be a single path component.
    // Without this, `save_channel_api_key`/`delete_channel_credentials` could
    // write or delete any `*.json` file under the user's home directory.
    crate::storage::validate_child_name(channel_id)
        .map_err(|_| "Invalid channel id".to_string())?;
    Ok(auth_dir_for_home(home_dir).join(format!("{channel_id}.json")))
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
    crate::storage::atomic_write(path, content.as_bytes())
        .map_err(|e| format!("Failed to write channels file: {e}"))
}

fn read_channel_auth_for_home(
    home_dir: &Path,
    channel_id: &str,
) -> Result<Option<ChannelAuth>, String> {
    let path = auth_file_path_for_home(home_dir, channel_id)?;
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
    let path = auth_file_path_for_home(home_dir, channel_id)?;
    let content =
        serde_json::to_string_pretty(auth).map_err(|e| format!("Failed to serialize auth: {e}"))?;
    // Channel auth holds API keys: keep it owner-only on Unix.
    crate::storage::atomic_write_private(&path, content.as_bytes())
        .map_err(|e| format!("Failed to write auth file: {e}"))
}

fn delete_channel_auth_for_home(home_dir: &Path, channel_id: &str) -> Result<(), String> {
    let path = auth_file_path_for_home(home_dir, channel_id)?;
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

/// Truncates a response body for logs and error messages.
///
/// Cuts on a character boundary: New API answers with localized text, and byte
/// slicing would panic in the middle of a multi-byte character.
///
/// Credentials are redacted first — the body may be a token list or a login
/// response, and these snippets end up in the system log and in the UI.
fn body_snippet(body: &str) -> String {
    const MAX_CHARS: usize = 300;
    let redacted = redact_secrets(body);
    if redacted.chars().count() <= MAX_CHARS {
        return redacted;
    }
    let mut snippet: String = redacted.chars().take(MAX_CHARS).collect();
    snippet.push_str("...");
    snippet
}

/// Masks bearer tokens, `sk-…` style keys and JSON credential fields so a
/// response body can be logged for troubleshooting without leaking secrets.
fn redact_secrets(input: &str) -> String {
    use std::sync::LazyLock;

    static REDACTORS: LazyLock<Vec<regex::Regex>> = LazyLock::new(|| {
        vec![
            // Authorization: Bearer <token> / "access_token": "<token>"
            regex::Regex::new(r"(?i)(bearer\s+)[A-Za-z0-9._~+/=-]{8,}").expect("bearer regex"),
            // Known provider key shapes (sk-…, ghp_…, glpat-…, xoxb-…).
            regex::Regex::new(r"(?i)\b(?:sk|pk|rk|ghp|gho|ghu|ghs|ghr|glpat|xox[abp])[-_][A-Za-z0-9_-]{8,}")
                .expect("key regex"),
            // "api_key": "…", "password": "…", token: … (unquoted values too)
            regex::Regex::new(
                r#"(?i)("(?:api[_-]?key|key|access[_-]?token|refresh[_-]?token|token|password|secret)"\s*:\s*")[^"]*(")"#,
            )
            .expect("json field regex"),
            regex::Regex::new(
                r#"(?i)\b((?:api[_-]?key|access[_-]?token|refresh[_-]?token|token|password|secret)\s*[:=]\s*)([^\s,;}&"']{4,})"#,
            )
            .expect("bare field regex"),
        ]
    });

    let mut out = input.to_string();
    out = REDACTORS[0].replace_all(&out, "${1}[REDACTED]").to_string();
    out = REDACTORS[1].replace_all(&out, "[REDACTED]").to_string();
    out = REDACTORS[2]
        .replace_all(&out, "${1}[REDACTED]${2}")
        .to_string();
    out = REDACTORS[3].replace_all(&out, "${1}[REDACTED]").to_string();
    out
}

/// Reads a response body as JSON, logging a truncated copy when parsing fails.
async fn read_json_body(response: reqwest::Response, what: &str) -> Result<Value, String> {
    let status = response.status();
    let body = response
        .text()
        .await
        .map_err(|e| format!("Failed to read {what} response body: {e}"))?;
    serde_json::from_str(&body).map_err(|e| {
        log::warn!(
            "Channel: failed to parse {what} response, status={status}, body={}",
            body_snippet(&body)
        );
        format!("Failed to parse {what} response: {e}")
    })
}

/// Dashboard credentials for a New API server.
///
/// New API v1.0 replaced the dashboard session cookie plus the `New-Api-User`
/// header with a short lived access token sent as `Authorization: Bearer`. Both
/// are carried when the response provides them, so the same requests keep
/// working against either generation of the server.
struct NewApiAuth {
    access_token: Option<String>,
    user_id: Option<i64>,
}

impl NewApiAuth {
    fn apply(&self, request: reqwest::RequestBuilder) -> reqwest::RequestBuilder {
        let mut request = request;
        if let Some(access_token) = self.access_token.as_deref() {
            request = request.header(
                reqwest::header::AUTHORIZATION,
                format!("Bearer {access_token}"),
            );
        }
        if let Some(user_id) = self.user_id {
            request = request.header("New-Api-User", user_id.to_string());
        }
        request
    }
}

/// Extracts the dashboard credentials from a successful login response.
///
/// New API v1.0 answers with an auth bundle (`data.access_token` plus
/// `data.user`), older releases returned the user object directly as `data`, and
/// accounts with second-step verification get a challenge instead.
fn new_api_auth_from_login(login_data: &Value) -> Result<NewApiAuth, String> {
    let data = login_data.get("data").filter(|data| !data.is_null());

    let access_token = data
        .and_then(|data| data.get("access_token"))
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|token| !token.is_empty())
        .map(str::to_string);

    // The user object sits directly under `data` before the auth rewrite and
    // under `data.user` afterwards.
    let user_id = data
        .and_then(|data| data.get("id"))
        .and_then(Value::as_i64)
        .or_else(|| {
            data.and_then(|data| data.get("user"))
                .and_then(|user| user.get("id"))
                .and_then(Value::as_i64)
        });

    if access_token.is_some() || user_id.is_some() {
        return Ok(NewApiAuth {
            access_token,
            user_id,
        });
    }

    // `require_2fa` is the pre-rewrite challenge field, `require_verification`
    // the current one.
    let needs_verification = ["require_2fa", "require_verification"].iter().any(|field| {
        data.and_then(|data| data.get(*field))
            .and_then(Value::as_bool)
            == Some(true)
    });
    if needs_verification {
        return Err(
            "Login requires a second verification step (2FA or email code), which automatic key loading cannot complete; use a New API account without extra verification"
                .to_string(),
        );
    }

    let response_shape = data.map(Value::to_string).unwrap_or_else(|| "null".into());
    Err(format!(
        "Could not get user id from login response: {}",
        body_snippet(&response_shape)
    ))
}

/// Whether the site refuses plain-text passwords (`PASSWORD_LOGIN_ENCRYPTION_ENABLED`).
///
/// The rejection itself is a generic "Invalid parameters", so ask the dedicated
/// endpoint when login fails and surface a usable reason instead.
async fn new_api_password_encryption_required(client: &reqwest::Client, base: &str) -> bool {
    let Ok(response) = client
        .get(format!("{base}/api/user/login/encryption-key"))
        .send()
        .await
    else {
        return false;
    };
    if !response.status().is_success() {
        return false;
    }
    response.json::<Value>().await.ok().and_then(|data| {
        data.get("data")
            .and_then(|data| data.get("enabled"))
            .and_then(Value::as_bool)
    }) == Some(true)
}

/// Logs into the New API dashboard and returns the credentials it granted.
async fn new_api_login(
    client: &reqwest::Client,
    base: &str,
    username: &str,
    password: &str,
) -> Result<NewApiAuth, String> {
    let response = client
        .post(format!("{base}/api/user/login"))
        .json(&serde_json::json!({ "username": username, "password": password }))
        .send()
        .await
        .map_err(|e| format!("Failed to login: {e}"))?;

    let status = response.status();
    let body = response
        .text()
        .await
        .map_err(|e| format!("Failed to read login response body: {e}"))?;

    if !status.is_success() {
        log::warn!(
            "Channel: New API login failed, status={status}, body={}",
            body_snippet(&body)
        );
        return Err(format!("Login failed {status}: {}", body_snippet(&body)));
    }

    let login_data: Value = serde_json::from_str(&body).map_err(|e| {
        log::warn!(
            "Channel: failed to parse login response, body={}",
            body_snippet(&body)
        );
        format!("Failed to parse login response: {e}")
    })?;

    if login_data.get("success").and_then(Value::as_bool) != Some(true) {
        let message = login_data
            .get("message")
            .and_then(Value::as_str)
            .unwrap_or("Unknown error");
        if new_api_password_encryption_required(client, base).await {
            log::warn!("Channel: New API login rejected: password encryption is enabled");
            return Err(
                "Login failed: this New API site requires encrypted passwords (PASSWORD_LOGIN_ENCRYPTION_ENABLED), which is not supported"
                    .to_string(),
            );
        }
        log::warn!("Channel: New API login rejected: {message}");
        return Err(format!("Login failed: {message}"));
    }

    let auth = new_api_auth_from_login(&login_data);
    if let Err(err) = &auth {
        log::warn!("Channel: unusable New API login response: {err}");
    }
    auth
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
    let auth = new_api_login(&client, base, username, password).await?;

    // Fetch tokens with pagination. New API masks the key of every entry here,
    // so the usable value is resolved separately below.
    let keys_url = format!("{base}/api/token");
    let page_size: usize = 100;
    let mut all_keys: Vec<ChannelToken> = Vec::new();
    let mut masked_keys: Vec<(usize, i64)> = Vec::new();
    let mut page: usize = 1;

    loop {
        let response = auth
            .apply(client.get(&keys_url))
            .query(&[("p", page.to_string()), ("size", page_size.to_string())])
            .send()
            .await
            .map_err(|e| format!("Failed to fetch keys: {e}"))?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(format!("API error {status}: {}", body_snippet(&body)));
        }

        let data = read_json_body(response, "keys").await?;

        let items: Vec<Value> = data
            .get("data")
            .and_then(|d| d.get("items"))
            .and_then(|v| v.as_array())
            .cloned()
            .unwrap_or_default();

        let count = items.len();

        for token in &items {
            let token_id = match token.get("id").and_then(Value::as_i64) {
                Some(id) => id,
                None => continue,
            };

            let (Some(name), Some(status)) = (
                token.get("name").and_then(Value::as_str),
                token.get("status").and_then(Value::as_i64),
            ) else {
                continue;
            };

            // Releases before the key split returned the full key here; newer
            // ones return a mask such as `abcd**********wxyz`.
            let list_key = token
                .get("key")
                .and_then(Value::as_str)
                .map(str::trim)
                .filter(|key| is_revealed_new_api_key(key));

            all_keys.push(ChannelToken {
                id: token_id as f64,
                name: name.to_string(),
                key: list_key.map(new_api_key_with_sk_prefix).unwrap_or_default(),
                status: status as i32,
                remain_quota: token
                    .get("remain_quota")
                    .and_then(Value::as_f64)
                    .unwrap_or(0.0),
                used_quota: token
                    .get("used_quota")
                    .and_then(Value::as_f64)
                    .unwrap_or(0.0),
                unlimited_quota: token
                    .get("unlimited_quota")
                    .and_then(Value::as_bool)
                    .unwrap_or(false),
                platform: None,
                group_name: None,
            });

            if list_key.is_none() {
                masked_keys.push((all_keys.len() - 1, token_id));
            }
        }

        if count < page_size {
            break;
        }
        page += 1;
    }

    if !masked_keys.is_empty() {
        let token_ids: Vec<i64> = masked_keys.iter().map(|(_, token_id)| *token_id).collect();
        let revealed = reveal_new_api_token_keys(&client, base, &auth, &token_ids).await;

        for (index, token_id) in &masked_keys {
            if let Some(key) = revealed.get(token_id) {
                all_keys[*index].key = new_api_key_with_sk_prefix(key);
            }
        }

        // A token left without a usable key would be copied as a mask.
        let unresolved = all_keys.iter().filter(|token| token.key.is_empty()).count();
        all_keys.retain(|token| !token.key.is_empty());

        if all_keys.is_empty() {
            return Err(format!(
                "Failed to read API keys: New API rejected all {} key request(s)",
                masked_keys.len()
            ));
        }
        if unresolved > 0 {
            log::warn!("Channel: dropped {unresolved} New API token(s) whose key was not revealed");
        }
    }

    Ok(all_keys)
}

/// Reveals the full API keys of tokens whose list entry is masked.
///
/// New API exposes `POST /api/token/batch/keys` (at most 100 ids per call) on
/// the releases that mask keys, and `POST /api/token/:id/key` on all of them.
/// The single-key route sits behind the strict critical rate limit, so the batch
/// route is tried first and the single route only fills the gaps.
async fn reveal_new_api_token_keys(
    client: &reqwest::Client,
    base: &str,
    auth: &NewApiAuth,
    token_ids: &[i64],
) -> HashMap<i64, String> {
    let mut revealed = reveal_new_api_token_keys_batch(client, base, auth, token_ids).await;

    for token_id in token_ids {
        if revealed.contains_key(token_id) {
            continue;
        }
        if let Some(key) = reveal_new_api_token_key(client, base, auth, *token_id).await {
            revealed.insert(*token_id, key);
        }
    }

    revealed
}

async fn reveal_new_api_token_keys_batch(
    client: &reqwest::Client,
    base: &str,
    auth: &NewApiAuth,
    token_ids: &[i64],
) -> HashMap<i64, String> {
    let mut revealed = HashMap::new();

    for chunk in token_ids.chunks(100) {
        let response = auth
            .apply(client.post(format!("{base}/api/token/batch/keys")))
            .json(&serde_json::json!({ "ids": chunk }))
            .send()
            .await;

        let response = match response {
            Ok(response) => response,
            Err(err) => {
                log::warn!("Channel: batch key request failed: {err}");
                break;
            }
        };

        if !response.status().is_success() {
            // Deployments that predate the endpoint answer 404; the caller falls
            // back to the single-key route.
            log::debug!(
                "Channel: batch key endpoint unavailable (status={})",
                response.status()
            );
            break;
        }

        let data = match response.json::<Value>().await {
            Ok(data) => data,
            Err(err) => {
                log::warn!("Channel: failed to parse batch key response: {err}");
                break;
            }
        };

        if data.get("success").and_then(Value::as_bool) != Some(true) {
            log::warn!("Channel: batch key request rejected by New API");
            break;
        }

        revealed.extend(parse_new_api_batch_keys(&data));
    }

    revealed
}

/// Pulls `id -> key` out of a `POST /api/token/batch/keys` payload.
fn parse_new_api_batch_keys(data: &Value) -> HashMap<i64, String> {
    let Some(entries) = data
        .get("data")
        .and_then(|data| data.get("keys"))
        .and_then(Value::as_object)
    else {
        return HashMap::new();
    };

    entries
        .iter()
        .filter_map(|(id, key)| {
            let id = id.parse::<i64>().ok()?;
            let key = key.as_str()?.trim();
            is_revealed_new_api_key(key).then(|| (id, key.to_string()))
        })
        .collect()
}

async fn reveal_new_api_token_key(
    client: &reqwest::Client,
    base: &str,
    auth: &NewApiAuth,
    token_id: i64,
) -> Option<String> {
    let response = match auth
        .apply(client.post(format!("{base}/api/token/{token_id}/key")))
        .send()
        .await
    {
        Ok(response) => response,
        Err(err) => {
            log::warn!("Channel: key request for token {token_id} failed: {err}");
            return None;
        }
    };

    if !response.status().is_success() {
        log::warn!(
            "Channel: key request for token {token_id} returned {}",
            response.status()
        );
        return None;
    }

    let data = response.json::<Value>().await.ok()?;
    parse_new_api_token_key(&data)
}

/// Pulls the full key out of a `POST /api/token/:id/key` payload. Older builds
/// returned it directly as `data`, current ones wrap it as `data.key`.
fn parse_new_api_token_key(data: &Value) -> Option<String> {
    let key = match data.get("data") {
        Some(Value::String(key)) => Some(key.as_str()),
        Some(data) => data.get("key").and_then(Value::as_str),
        None => None,
    };
    key.map(str::trim)
        .filter(|key| is_revealed_new_api_key(key))
        .map(str::to_string)
}

/// Whether a key taken from the token list is the real value. New API masks keys
/// once it moves them behind the reveal endpoints (`abcd**********wxyz`).
fn is_revealed_new_api_key(key: &str) -> bool {
    !key.is_empty() && !key.contains('*')
}

/// New API stores token keys without the `sk-` prefix; clients expect it.
fn new_api_key_with_sk_prefix(key: &str) -> String {
    if key.starts_with("sk-") {
        key.to_string()
    } else {
        format!("sk-{key}")
    }
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
        return Err(format!("Login failed {status}: {}", body_snippet(&body)));
    }

    let login_data: Value = {
        let body = login_response
            .text()
            .await
            .map_err(|e| format!("Failed to read login response body: {e}"))?;
        serde_json::from_str(&body).map_err(|e| {
            let truncated = body_snippet(&body);
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
            return Err(format!("API error {status}: {}", body_snippet(&body)));
        }

        let keys_data: Value = {
            let body = keys_response
                .text()
                .await
                .map_err(|e| format!("Failed to read keys response body: {e}"))?;
            serde_json::from_str(&body).map_err(|e| {
                let truncated = body_snippet(&body);
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
    let base_without_version =
        |suffix: &str| trimmed_base.strip_suffix(suffix).unwrap_or(trimmed_base);

    match platform {
        Some("gemini") => vec![format!("{}/v1beta/models", base_without_version("/v1beta"))],
        Some("openai") => vec![format!("{}/v1/models", base_without_version("/v1"))],
        Some("deepseek") => vec![
            format!("{}/v1/models", base_without_version("/v1")),
            format!("{}/models", base_without_version("/v1")),
        ],
        _ => vec![format!("{}/v1/models", base_without_version("/v1"))],
    }
}

/// Fetch models using an API key (for quick model addition from channels)
pub async fn fetch_models_by_api_key(
    base_url: &str,
    api_key: &str,
    platform: Option<&str>,
) -> Result<Vec<ModelInfo>, String> {
    let trimmed_base = base_url.trim_end_matches('/');
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .build()
        .map_err(|e| format!("Failed to create HTTP client: {e}"))?;

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
                "Channel: antigravity API error, url={claude_url} status={status} body={}",
                body_snippet(&body)
            );
            return Err(format!("API error {status}: {}", body_snippet(&body)));
        }

        let body = response
            .text()
            .await
            .map_err(|e| format!("Failed to read response body: {e}"))?;
        let data: Value = serde_json::from_str(&body).map_err(|e| {
            let truncated = body_snippet(&body);
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

        let mut request = client
            .get(url)
            .header("Authorization", format!("Bearer {api_key}"));
        if platform == Some("anthropic") {
            // Anthropic's native model endpoint uses x-api-key. Keep the
            // Bearer header as well so Anthropic-compatible gateways that
            // expose an OpenAI-style /v1/models endpoint continue to work.
            request = request
                .header("x-api-key", api_key)
                .header("anthropic-version", "2023-06-01");
        }
        let response = request
            .send()
            .await
            .map_err(|e| format!("Request failed: {e}"))?;

        let status = response.status();
        if !status.is_success() {
            let body = response.text().await.unwrap_or_default();
            log::warn!(
                "Channel: API error, url={url} status={status} body={}",
                body_snippet(&body)
            );
            last_error = format!("API error {status}: {}", body_snippet(&body));

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
            let truncated = body_snippet(&body);
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
                    let name = m
                        .get("display_name")
                        .or_else(|| m.get("name"))
                        .and_then(|name| name.as_str())
                        .map(String::from);
                    Some(ModelInfo { id, name })
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
    fn body_snippet_redacts_json_credentials_but_keeps_diagnostics() {
        let body = r#"{"success":false,"access_token":"eyJhbGciOiJIUzI1NiJ9.body.sig","data":{"key":"sk-live-1234567890abcdef","password":"hunter2"}}"#;
        let snippet = body_snippet(body);

        assert!(snippet.contains("\"success\":false"));
        assert!(snippet.contains("[REDACTED]"));
        assert!(!snippet.contains("eyJhbGciOiJIUzI1NiJ9"));
        assert!(!snippet.contains("sk-live-1234567890abcdef"));
        assert!(!snippet.contains("hunter2"));
    }

    #[test]
    fn body_snippet_redacts_bearer_tokens_and_bare_keys() {
        let snippet =
            body_snippet("Authorization: Bearer abc.def.ghi\napi_key=sk-proj-abcdefghijkl");

        assert!(!snippet.contains("abc.def.ghi"));
        assert!(!snippet.contains("abcdefghijkl"));
        assert!(snippet.contains("Bearer [REDACTED]"));
    }

    #[test]
    fn redact_secrets_leaves_ordinary_text_alone() {
        assert_eq!(redact_secrets("rate limit exceeded"), "rate limit exceeded");
    }

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
        assert_eq!(
            model_endpoint_urls("https://api.example.com/v1", Some("openai")),
            vec!["https://api.example.com/v1/models".to_string()]
        );
        assert_eq!(
            model_endpoint_urls("https://api.example.com/v1", Some("deepseek")),
            vec![
                "https://api.example.com/v1/models".to_string(),
                "https://api.example.com/models".to_string(),
            ]
        );
        assert_eq!(
            model_endpoint_urls("https://api.example.com/v1beta", Some("gemini")),
            vec!["https://api.example.com/v1beta/models".to_string()]
        );
    }

    /// Expects the login response to be rejected and returns the message.
    fn new_api_login_error(login: &Value) -> String {
        match new_api_auth_from_login(login) {
            Ok(_) => panic!("expected the login response to be rejected"),
            Err(err) => err,
        }
    }

    fn new_api_auth_headers(auth: &NewApiAuth) -> reqwest::header::HeaderMap {
        auth.apply(reqwest::Client::new().get("http://localhost/api/token"))
            .build()
            .unwrap()
            .headers()
            .clone()
    }

    #[test]
    fn test_new_api_login_legacy_response_yields_user_id() {
        let login = serde_json::json!({
            "success": true,
            "message": "",
            "data": { "id": 42, "username": "u", "role": 1, "status": 1, "group": "default" }
        });

        let auth = new_api_auth_from_login(&login).unwrap();
        assert_eq!(auth.user_id, Some(42));
        assert!(auth.access_token.is_none());
    }

    #[test]
    fn test_new_api_login_auth_bundle_yields_access_token() {
        let login = serde_json::json!({
            "success": true,
            "message": "",
            "data": {
                "access_token": "jwt-token",
                "token_type": "Bearer",
                "access_expires_at": 1_730_000_000,
                "session": { "sid": "sid-1" },
                "user": { "id": 7, "username": "u" }
            }
        });

        let auth = new_api_auth_from_login(&login).unwrap();
        assert_eq!(auth.access_token.as_deref(), Some("jwt-token"));
        assert_eq!(auth.user_id, Some(7));
    }

    #[test]
    fn test_new_api_login_two_factor_challenge_is_reported() {
        // `require_2fa` is the pre-rewrite challenge field, `require_verification`
        // the current one.
        for field in ["require_2fa", "require_verification"] {
            let login = serde_json::json!({
                "success": true,
                "message": "need 2fa",
                "data": { field: true, "flow_token": "flow-1", "expires_at": 1_730_000_000 }
            });

            let err = new_api_login_error(&login);
            assert!(
                err.contains("second verification step"),
                "unexpected error: {err}"
            );
        }
    }

    #[test]
    fn test_new_api_login_unknown_shape_names_the_cause() {
        let login = serde_json::json!({ "success": true, "data": null });

        let err = new_api_login_error(&login);
        assert!(
            err.contains("Could not get user id"),
            "unexpected error: {err}"
        );
    }

    #[test]
    fn test_new_api_auth_headers_cover_both_generations() {
        let legacy = NewApiAuth {
            access_token: None,
            user_id: Some(9),
        };
        let headers = new_api_auth_headers(&legacy);
        assert_eq!(headers.get("New-Api-User").unwrap(), "9");
        assert!(headers.get(reqwest::header::AUTHORIZATION).is_none());

        let current = NewApiAuth {
            access_token: Some("jwt".to_string()),
            user_id: Some(9),
        };
        let headers = new_api_auth_headers(&current);
        assert_eq!(
            headers.get(reqwest::header::AUTHORIZATION).unwrap(),
            "Bearer jwt"
        );
        assert_eq!(headers.get("New-Api-User").unwrap(), "9");
    }

    #[test]
    fn test_new_api_masked_key_is_not_reused() {
        assert!(!is_revealed_new_api_key("abcd**********wxyz"));
        assert!(!is_revealed_new_api_key(""));
        assert!(is_revealed_new_api_key("A1b2C3d4"));
    }

    #[test]
    fn test_new_api_key_adds_missing_sk_prefix() {
        assert_eq!(new_api_key_with_sk_prefix("abc123"), "sk-abc123");
        assert_eq!(new_api_key_with_sk_prefix("sk-abc123"), "sk-abc123");
    }

    #[test]
    fn test_new_api_token_key_response_shapes() {
        let wrapped = serde_json::json!({ "success": true, "data": { "key": "abc123" } });
        assert_eq!(parse_new_api_token_key(&wrapped).as_deref(), Some("abc123"));

        // Older builds return the key as `data` itself.
        let flat = serde_json::json!({ "success": true, "data": "abc123" });
        assert_eq!(parse_new_api_token_key(&flat).as_deref(), Some("abc123"));

        let masked = serde_json::json!({ "success": true, "data": { "key": "abcd****wxyz" } });
        assert_eq!(parse_new_api_token_key(&masked), None);
    }

    #[test]
    fn test_new_api_batch_key_response_shapes() {
        let payload = serde_json::json!({
            "success": true,
            "data": { "keys": { "3": "key3", "5": "key5", "7": "" } }
        });

        let keys = parse_new_api_batch_keys(&payload);
        assert_eq!(keys.get(&3).map(String::as_str), Some("key3"));
        assert_eq!(keys.get(&5).map(String::as_str), Some("key5"));
        assert_eq!(keys.get(&7), None);
    }

    #[test]
    fn test_body_snippet_truncates_on_char_boundary() {
        let body = "错".repeat(400);
        let snippet = body_snippet(&body);
        assert!(snippet.ends_with("..."));
        assert_eq!(snippet.chars().count(), 303);
    }
}
