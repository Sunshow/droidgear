//! Factory settings.json management (core).
//!
//! Handles reading/writing settings under Factory home (`~/.factory/settings.json` by default).

use serde::{Deserialize, Serialize};
use serde_json::Value;
use specta::Type;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

use crate::paths;

// ============================================================================
// Config Read Result
// ============================================================================

/// Result of reading the config file
pub enum ConfigReadResult {
    /// Successfully parsed config
    Ok(Value),
    /// File does not exist
    NotFound,
    /// Failed to parse JSON (contains error message)
    ParseError(String),
}

/// Error type for config operations that require valid JSON
pub const CONFIG_PARSE_ERROR_PREFIX: &str = "CONFIG_PARSE_ERROR:";

// ============================================================================
// Types
// ============================================================================

/// Provider types supported by Factory BYOK
#[derive(Debug, Clone, Serialize, Deserialize, Type, PartialEq)]
#[serde(rename_all = "kebab-case")]
pub enum Provider {
    Anthropic,
    Openai,
    GenericChatCompletionApi,
}

/// Custom model configuration
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct CustomModel {
    /// Model identifier sent via API
    pub model: String,
    /// Unique identifier for the model (e.g., "custom:ModelName-0")
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    /// Index of the model in the list
    #[serde(skip_serializing_if = "Option::is_none")]
    pub index: Option<u32>,
    /// Human-friendly name shown in model selector
    #[serde(skip_serializing_if = "Option::is_none")]
    pub display_name: Option<String>,
    /// API endpoint base URL
    pub base_url: String,
    /// API key for the provider
    pub api_key: String,
    /// Provider type
    pub provider: Provider,
    /// Maximum output tokens
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_output_tokens: Option<u32>,
    /// Whether to disable image support for this model (default: images supported)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub no_image_support: Option<bool>,
    /// Additional provider-specific arguments
    #[serde(skip_serializing_if = "Option::is_none")]
    pub extra_args: Option<HashMap<String, Value>>,
    /// Additional HTTP headers
    #[serde(skip_serializing_if = "Option::is_none")]
    pub extra_headers: Option<HashMap<String, String>>,
}

/// Model info returned from API
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct ModelInfo {
    pub id: String,
    pub name: Option<String>,
}

// ============================================================================
// Mission Model Settings
// ============================================================================

/// Mission model settings for Mission mode workers
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct MissionModelSettings {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub worker_model: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub worker_reasoning_effort: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub validation_worker_model: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub validation_worker_reasoning_effort: Option<String>,
}

/// Session default settings for mixed models configuration
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct SessionDefaultSettings {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reasoning_effort: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub spec_mode_model: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub spec_mode_reasoning_effort: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub autonomy_mode: Option<String>,
}

// ============================================================================
// Helpers
// ============================================================================

fn factory_config_path_for_home(home_dir: &Path) -> Result<PathBuf, String> {
    let config_paths = paths::load_config_paths_for_home(home_dir);
    let factory_dir = paths::get_factory_home_for_home(home_dir, &config_paths)?;

    if !factory_dir.exists() {
        std::fs::create_dir_all(&factory_dir)
            .map_err(|e| format!("Failed to create .factory directory: {e}"))?;
    }

    Ok(factory_dir.join("settings.json"))
}

fn read_config_file_for_home(home_dir: &Path) -> ConfigReadResult {
    let config_path = match factory_config_path_for_home(home_dir) {
        Ok(path) => path,
        Err(_) => return ConfigReadResult::NotFound,
    };

    if !config_path.exists() {
        return ConfigReadResult::NotFound;
    }

    let contents = match std::fs::read_to_string(&config_path) {
        Ok(c) => c,
        Err(e) => return ConfigReadResult::ParseError(format!("Failed to read config file: {e}")),
    };

    if contents.trim().is_empty() {
        return ConfigReadResult::NotFound;
    }

    match serde_json::from_str(&contents) {
        Ok(value) => ConfigReadResult::Ok(value),
        Err(e) => ConfigReadResult::ParseError(format!("Failed to parse config JSON: {e}")),
    }
}

fn write_config_file_for_home(home_dir: &Path, config: &Value) -> Result<(), String> {
    let config_path = factory_config_path_for_home(home_dir)?;

    // Resolve symlink to get the actual file path
    let actual_path = if config_path.is_symlink() {
        std::fs::canonicalize(&config_path)
            .map_err(|e| format!("Failed to resolve symlink: {e}"))?
    } else {
        config_path
    };

    let temp_path = actual_path.with_extension("tmp");
    let json_content = serde_json::to_string_pretty(config)
        .map_err(|e| format!("Failed to serialize config: {e}"))?;

    std::fs::write(&temp_path, json_content)
        .map_err(|e| format!("Failed to write config file: {e}"))?;

    std::fs::rename(&temp_path, &actual_path).map_err(|e| {
        let _ = std::fs::remove_file(&temp_path);
        format!("Failed to finalize config file: {e}")
    })?;

    Ok(())
}

// ============================================================================
// Active file helpers — route through droid_settings_files
// ============================================================================

/// Reads the config from the active settings file (respects user's file selection)
fn read_active_config_file() -> ConfigReadResult {
    let config_path = match crate::droid_settings_files::get_active_settings_path() {
        Ok(path) => path,
        Err(_) => return ConfigReadResult::NotFound,
    };

    if !config_path.exists() {
        return ConfigReadResult::NotFound;
    }

    let contents = match std::fs::read_to_string(&config_path) {
        Ok(c) => c,
        Err(e) => return ConfigReadResult::ParseError(format!("Failed to read config file: {e}")),
    };

    if contents.trim().is_empty() {
        return ConfigReadResult::NotFound;
    }

    match serde_json::from_str(&contents) {
        Ok(value) => ConfigReadResult::Ok(value),
        Err(e) => ConfigReadResult::ParseError(format!("Failed to parse config JSON: {e}")),
    }
}

/// Writes the config to the active settings file
fn write_active_config_file(config: &Value) -> Result<(), String> {
    let config_path = crate::droid_settings_files::get_active_settings_path()?;

    // Ensure parent directory exists
    if let Some(parent) = config_path.parent() {
        if !parent.exists() {
            std::fs::create_dir_all(parent)
                .map_err(|e| format!("Failed to create directory: {e}"))?;
        }
    }

    let actual_path = if config_path.is_symlink() {
        std::fs::canonicalize(&config_path)
            .map_err(|e| format!("Failed to resolve symlink: {e}"))?
    } else {
        config_path
    };

    let temp_path = actual_path.with_extension("tmp");
    let json_content = serde_json::to_string_pretty(config)
        .map_err(|e| format!("Failed to serialize config: {e}"))?;

    std::fs::write(&temp_path, json_content)
        .map_err(|e| format!("Failed to write config file: {e}"))?;

    std::fs::rename(&temp_path, &actual_path).map_err(|e| {
        let _ = std::fs::remove_file(&temp_path);
        format!("Failed to finalize config file: {e}")
    })?;

    Ok(())
}

fn system_home_dir() -> Result<PathBuf, String> {
    dirs::home_dir().ok_or_else(|| "Failed to get home directory".to_string())
}

// ============================================================================
// Public API (for Tauri + TUI)
// ============================================================================

pub fn get_config_path_for_home(home_dir: &Path) -> Result<String, String> {
    Ok(factory_config_path_for_home(home_dir)?
        .to_string_lossy()
        .to_string())
}

pub fn get_config_path() -> Result<String, String> {
    crate::droid_settings_files::get_active_settings_path().map(|p| p.to_string_lossy().to_string())
}

pub fn reset_config_file_for_home(home_dir: &Path) -> Result<(), String> {
    write_config_file_for_home(home_dir, &serde_json::json!({}))
}

pub fn reset_config_file() -> Result<(), String> {
    write_active_config_file(&serde_json::json!({}))
}

pub fn load_custom_models_for_home(home_dir: &Path) -> Result<Vec<CustomModel>, String> {
    let config = match read_config_file_for_home(home_dir) {
        ConfigReadResult::Ok(value) => value,
        ConfigReadResult::NotFound => return Ok(vec![]),
        ConfigReadResult::ParseError(_) => return Ok(vec![]),
    };

    let models: Vec<CustomModel> = config
        .get("customModels")
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|v| serde_json::from_value(v.clone()).ok())
                .collect()
        })
        .unwrap_or_default();

    Ok(models)
}

pub fn load_custom_models() -> Result<Vec<CustomModel>, String> {
    let config = match read_active_config_file() {
        ConfigReadResult::Ok(value) => value,
        ConfigReadResult::NotFound => return Ok(vec![]),
        ConfigReadResult::ParseError(_) => return Ok(vec![]),
    };

    let models: Vec<CustomModel> = config
        .get("customModels")
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|v| serde_json::from_value(v.clone()).ok())
                .collect()
        })
        .unwrap_or_default();

    Ok(models)
}

pub fn save_custom_models_for_home(
    home_dir: &Path,
    models: Vec<CustomModel>,
) -> Result<(), String> {
    let mut config = match read_config_file_for_home(home_dir) {
        ConfigReadResult::Ok(value) => value,
        ConfigReadResult::NotFound => serde_json::json!({}),
        ConfigReadResult::ParseError(e) => {
            return Err(format!("{CONFIG_PARSE_ERROR_PREFIX} {e}"));
        }
    };

    let models_value =
        serde_json::to_value(&models).map_err(|e| format!("Failed to serialize models: {e}"))?;

    if let Some(obj) = config.as_object_mut() {
        obj.insert("customModels".to_string(), models_value);
    } else {
        config = serde_json::json!({ "customModels": models_value });
    }

    write_config_file_for_home(home_dir, &config)
}

pub fn save_custom_models(models: Vec<CustomModel>) -> Result<(), String> {
    let mut config = match read_active_config_file() {
        ConfigReadResult::Ok(value) => value,
        ConfigReadResult::NotFound => serde_json::json!({}),
        ConfigReadResult::ParseError(e) => {
            return Err(format!("{CONFIG_PARSE_ERROR_PREFIX} {e}"));
        }
    };

    let models_value =
        serde_json::to_value(&models).map_err(|e| format!("Failed to serialize models: {e}"))?;

    if let Some(obj) = config.as_object_mut() {
        obj.insert("customModels".to_string(), models_value);
    } else {
        config = serde_json::json!({ "customModels": models_value });
    }

    write_active_config_file(&config)
}

pub fn check_legacy_config_for_home(home_dir: &Path) -> Result<bool, String> {
    let config_paths = paths::load_config_paths_for_home(home_dir);
    let factory_dir = paths::get_factory_home_for_home(home_dir, &config_paths)?;
    let legacy_path = factory_dir.join("config.json");

    if !legacy_path.exists() {
        return Ok(false);
    }

    match read_config_file_for_home(home_dir) {
        ConfigReadResult::Ok(value) => {
            let has_custom_models = value
                .get("customModels")
                .and_then(|v| v.as_array())
                .map(|arr| !arr.is_empty())
                .unwrap_or(false);
            Ok(has_custom_models)
        }
        _ => Ok(false),
    }
}

pub fn check_legacy_config() -> Result<bool, String> {
    check_legacy_config_for_home(&system_home_dir()?)
}

pub fn delete_legacy_config_for_home(home_dir: &Path) -> Result<(), String> {
    let config_paths = paths::load_config_paths_for_home(home_dir);
    let factory_dir = paths::get_factory_home_for_home(home_dir, &config_paths)?;
    let legacy_path = factory_dir.join("config.json");

    if legacy_path.exists() {
        std::fs::remove_file(&legacy_path)
            .map_err(|e| format!("Failed to delete legacy config: {e}"))?;
    }
    Ok(())
}

pub fn delete_legacy_config() -> Result<(), String> {
    delete_legacy_config_for_home(&system_home_dir()?)
}

pub async fn fetch_models(
    provider: Provider,
    base_url: &str,
    api_key: &str,
) -> Result<Vec<ModelInfo>, String> {
    let client = reqwest::Client::new();

    let models = match provider {
        Provider::Anthropic => fetch_anthropic_models(&client, base_url, api_key).await?,
        Provider::Openai | Provider::GenericChatCompletionApi => {
            fetch_openai_models(&client, base_url, api_key).await?
        }
    };

    Ok(models)
}

async fn fetch_anthropic_models(
    client: &reqwest::Client,
    base_url: &str,
    api_key: &str,
) -> Result<Vec<ModelInfo>, String> {
    let base = base_url.trim_end_matches('/').trim_end_matches("/v1");
    let url = format!("{base}/v1/models");
    log::debug!("FetchModels: requesting Anthropic models from {url}");

    let response = client
        .get(&url)
        .header("x-api-key", api_key)
        .header("anthropic-version", "2023-06-01")
        .send()
        .await
        .map_err(|e| format!("Request failed: {e}"))?;

    let response = if !response.status().is_success() {
        log::debug!(
            "FetchModels: Anthropic x-api-key auth failed ({}), retrying with Bearer token",
            response.status()
        );
        client
            .get(&url)
            .header("Authorization", format!("Bearer {api_key}"))
            .send()
            .await
            .map_err(|e| format!("Request failed: {e}"))?
    } else {
        response
    };

    let status = response.status();
    if !status.is_success() {
        let body = response.text().await.unwrap_or_default();
        log::warn!("FetchModels: Anthropic API error, url={url} status={status} body={body}");
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
        log::warn!("FetchModels: failed to parse Anthropic response, url={url} status={status} body={truncated}");
        format!("Failed to parse response: {e}")
    })?;

    let models = data
        .get("data")
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|m| {
                    let id = m.get("id")?.as_str()?.to_string();
                    let name = m
                        .get("display_name")
                        .and_then(|n| n.as_str())
                        .map(String::from);
                    Some(ModelInfo { id, name })
                })
                .collect()
        })
        .unwrap_or_default();

    Ok(models)
}

async fn fetch_openai_models(
    client: &reqwest::Client,
    base_url: &str,
    api_key: &str,
) -> Result<Vec<ModelInfo>, String> {
    let base = base_url.trim_end_matches('/').trim_end_matches("/v1");
    let url = format!("{base}/v1/models");
    log::debug!("FetchModels: requesting OpenAI models from {url}");

    let response = client
        .get(&url)
        .header("Authorization", format!("Bearer {api_key}"))
        .send()
        .await
        .map_err(|e| format!("Request failed: {e}"))?;

    let status = response.status();
    if !status.is_success() {
        let body = response.text().await.unwrap_or_default();
        log::warn!("FetchModels: OpenAI API error, url={url} status={status} body={body}");
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
        log::warn!("FetchModels: failed to parse OpenAI response, url={url} status={status} body={truncated}");
        format!("Failed to parse response: {e}")
    })?;

    let models = data
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

    Ok(models)
}

pub fn get_default_model_for_home(home_dir: &Path) -> Result<Option<String>, String> {
    let config = match read_config_file_for_home(home_dir) {
        ConfigReadResult::Ok(value) => value,
        ConfigReadResult::NotFound => return Ok(None),
        ConfigReadResult::ParseError(_) => return Ok(None),
    };

    let model_id = config
        .get("sessionDefaultSettings")
        .and_then(|s| s.get("model"))
        .and_then(|m| m.as_str())
        .map(String::from);

    Ok(model_id)
}

pub fn get_default_model() -> Result<Option<String>, String> {
    let config = match read_active_config_file() {
        ConfigReadResult::Ok(value) => value,
        ConfigReadResult::NotFound => return Ok(None),
        ConfigReadResult::ParseError(_) => return Ok(None),
    };

    let model_id = config
        .get("sessionDefaultSettings")
        .and_then(|s| s.get("model"))
        .and_then(|m| m.as_str())
        .map(String::from);

    Ok(model_id)
}

pub fn save_default_model_for_home(home_dir: &Path, model_id: &str) -> Result<(), String> {
    let mut config = match read_config_file_for_home(home_dir) {
        ConfigReadResult::Ok(value) => value,
        ConfigReadResult::NotFound => serde_json::json!({}),
        ConfigReadResult::ParseError(e) => {
            return Err(format!("{CONFIG_PARSE_ERROR_PREFIX} {e}"));
        }
    };

    if let Some(obj) = config.as_object_mut() {
        let session_settings = obj
            .entry("sessionDefaultSettings")
            .or_insert_with(|| serde_json::json!({}));

        if let Some(session_obj) = session_settings.as_object_mut() {
            session_obj.insert("model".to_string(), serde_json::json!(model_id));
        }
    }

    write_config_file_for_home(home_dir, &config)
}

pub fn save_default_model(model_id: &str) -> Result<(), String> {
    let mut config = match read_active_config_file() {
        ConfigReadResult::Ok(value) => value,
        ConfigReadResult::NotFound => serde_json::json!({}),
        ConfigReadResult::ParseError(e) => {
            return Err(format!("{CONFIG_PARSE_ERROR_PREFIX} {e}"));
        }
    };

    if let Some(obj) = config.as_object_mut() {
        let session_settings = obj
            .entry("sessionDefaultSettings")
            .or_insert_with(|| serde_json::json!({}));
        if let Some(session_obj) = session_settings.as_object_mut() {
            session_obj.insert("model".to_string(), serde_json::json!(model_id));
        }
    }

    write_active_config_file(&config)
}

pub fn get_cloud_session_sync_for_home(home_dir: &Path) -> Result<bool, String> {
    let config = match read_config_file_for_home(home_dir) {
        ConfigReadResult::Ok(value) => value,
        ConfigReadResult::NotFound => return Ok(true),
        ConfigReadResult::ParseError(_) => return Ok(true),
    };

    let enabled = config
        .get("cloudSessionSync")
        .and_then(|v| v.as_bool())
        .unwrap_or(true);

    Ok(enabled)
}

pub fn get_cloud_session_sync() -> Result<bool, String> {
    let config = match read_active_config_file() {
        ConfigReadResult::Ok(value) => value,
        ConfigReadResult::NotFound => return Ok(true),
        ConfigReadResult::ParseError(_) => return Ok(true),
    };
    let enabled = config
        .get("cloudSessionSync")
        .and_then(|v| v.as_bool())
        .unwrap_or(true);
    Ok(enabled)
}

pub fn save_cloud_session_sync_for_home(home_dir: &Path, enabled: bool) -> Result<(), String> {
    let mut config = match read_config_file_for_home(home_dir) {
        ConfigReadResult::Ok(value) => value,
        ConfigReadResult::NotFound => serde_json::json!({}),
        ConfigReadResult::ParseError(e) => {
            return Err(format!("{CONFIG_PARSE_ERROR_PREFIX} {e}"));
        }
    };

    if let Some(obj) = config.as_object_mut() {
        obj.insert("cloudSessionSync".to_string(), serde_json::json!(enabled));
    }

    write_config_file_for_home(home_dir, &config)
}

pub fn save_cloud_session_sync(enabled: bool) -> Result<(), String> {
    let mut config = match read_active_config_file() {
        ConfigReadResult::Ok(value) => value,
        ConfigReadResult::NotFound => serde_json::json!({}),
        ConfigReadResult::ParseError(e) => {
            return Err(format!("{CONFIG_PARSE_ERROR_PREFIX} {e}"));
        }
    };
    if let Some(obj) = config.as_object_mut() {
        obj.insert("cloudSessionSync".to_string(), serde_json::json!(enabled));
    }
    write_active_config_file(&config)
}

pub fn get_reasoning_effort_for_home(home_dir: &Path) -> Result<Option<String>, String> {
    let config = match read_config_file_for_home(home_dir) {
        ConfigReadResult::Ok(value) => value,
        ConfigReadResult::NotFound => return Ok(None),
        ConfigReadResult::ParseError(_) => return Ok(None),
    };

    let value = config
        .get("reasoningEffort")
        .and_then(|v| v.as_str())
        .map(String::from);

    Ok(value)
}

pub fn get_reasoning_effort() -> Result<Option<String>, String> {
    let config = match read_active_config_file() {
        ConfigReadResult::Ok(value) => value,
        ConfigReadResult::NotFound => return Ok(None),
        ConfigReadResult::ParseError(_) => return Ok(None),
    };
    let value = config
        .get("reasoningEffort")
        .and_then(|v| v.as_str())
        .map(String::from);
    Ok(value)
}

pub fn save_reasoning_effort_for_home(home_dir: &Path, value: &str) -> Result<(), String> {
    let mut config = match read_config_file_for_home(home_dir) {
        ConfigReadResult::Ok(value) => value,
        ConfigReadResult::NotFound => serde_json::json!({}),
        ConfigReadResult::ParseError(e) => {
            return Err(format!("{CONFIG_PARSE_ERROR_PREFIX} {e}"));
        }
    };

    if let Some(obj) = config.as_object_mut() {
        obj.insert("reasoningEffort".to_string(), serde_json::json!(value));
    }

    write_config_file_for_home(home_dir, &config)
}

pub fn save_reasoning_effort(value: &str) -> Result<(), String> {
    let mut config = match read_active_config_file() {
        ConfigReadResult::Ok(value) => value,
        ConfigReadResult::NotFound => serde_json::json!({}),
        ConfigReadResult::ParseError(e) => {
            return Err(format!("{CONFIG_PARSE_ERROR_PREFIX} {e}"));
        }
    };
    if let Some(obj) = config.as_object_mut() {
        obj.insert("reasoningEffort".to_string(), serde_json::json!(value));
    }
    write_active_config_file(&config)
}

pub fn get_diff_mode_for_home(home_dir: &Path) -> Result<String, String> {
    let config = match read_config_file_for_home(home_dir) {
        ConfigReadResult::Ok(value) => value,
        ConfigReadResult::NotFound => return Ok("github".to_string()),
        ConfigReadResult::ParseError(_) => return Ok("github".to_string()),
    };

    let value = config
        .get("diffMode")
        .and_then(|v| v.as_str())
        .map(String::from)
        .unwrap_or_else(|| "github".to_string());

    Ok(value)
}

pub fn get_diff_mode() -> Result<String, String> {
    let config = match read_active_config_file() {
        ConfigReadResult::Ok(value) => value,
        ConfigReadResult::NotFound => return Ok("github".to_string()),
        ConfigReadResult::ParseError(_) => return Ok("github".to_string()),
    };
    let value = config
        .get("diffMode")
        .and_then(|v| v.as_str())
        .map(String::from)
        .unwrap_or_else(|| "github".to_string());
    Ok(value)
}

pub fn save_diff_mode_for_home(home_dir: &Path, value: &str) -> Result<(), String> {
    let mut config = match read_config_file_for_home(home_dir) {
        ConfigReadResult::Ok(value) => value,
        ConfigReadResult::NotFound => serde_json::json!({}),
        ConfigReadResult::ParseError(e) => {
            return Err(format!("{CONFIG_PARSE_ERROR_PREFIX} {e}"));
        }
    };

    if let Some(obj) = config.as_object_mut() {
        obj.insert("diffMode".to_string(), serde_json::json!(value));
    }

    write_config_file_for_home(home_dir, &config)
}

pub fn save_diff_mode(value: &str) -> Result<(), String> {
    let mut config = match read_active_config_file() {
        ConfigReadResult::Ok(value) => value,
        ConfigReadResult::NotFound => serde_json::json!({}),
        ConfigReadResult::ParseError(e) => {
            return Err(format!("{CONFIG_PARSE_ERROR_PREFIX} {e}"));
        }
    };
    if let Some(obj) = config.as_object_mut() {
        obj.insert("diffMode".to_string(), serde_json::json!(value));
    }
    write_active_config_file(&config)
}

pub fn get_todo_display_mode_for_home(home_dir: &Path) -> Result<String, String> {
    let config = match read_config_file_for_home(home_dir) {
        ConfigReadResult::Ok(value) => value,
        ConfigReadResult::NotFound => return Ok("pinned".to_string()),
        ConfigReadResult::ParseError(_) => return Ok("pinned".to_string()),
    };

    let value = config
        .get("todoDisplayMode")
        .and_then(|v| v.as_str())
        .map(String::from)
        .unwrap_or_else(|| "pinned".to_string());

    Ok(value)
}

pub fn get_todo_display_mode() -> Result<String, String> {
    let config = match read_active_config_file() {
        ConfigReadResult::Ok(value) => value,
        ConfigReadResult::NotFound => return Ok("pinned".to_string()),
        ConfigReadResult::ParseError(_) => return Ok("pinned".to_string()),
    };
    let value = config
        .get("todoDisplayMode")
        .and_then(|v| v.as_str())
        .map(String::from)
        .unwrap_or_else(|| "pinned".to_string());
    Ok(value)
}

pub fn save_todo_display_mode_for_home(home_dir: &Path, value: &str) -> Result<(), String> {
    let mut config = match read_config_file_for_home(home_dir) {
        ConfigReadResult::Ok(value) => value,
        ConfigReadResult::NotFound => serde_json::json!({}),
        ConfigReadResult::ParseError(e) => {
            return Err(format!("{CONFIG_PARSE_ERROR_PREFIX} {e}"));
        }
    };

    if let Some(obj) = config.as_object_mut() {
        obj.insert("todoDisplayMode".to_string(), serde_json::json!(value));
    }

    write_config_file_for_home(home_dir, &config)
}

pub fn save_todo_display_mode(value: &str) -> Result<(), String> {
    let mut config = match read_active_config_file() {
        ConfigReadResult::Ok(value) => value,
        ConfigReadResult::NotFound => serde_json::json!({}),
        ConfigReadResult::ParseError(e) => {
            return Err(format!("{CONFIG_PARSE_ERROR_PREFIX} {e}"));
        }
    };
    if let Some(obj) = config.as_object_mut() {
        obj.insert("todoDisplayMode".to_string(), serde_json::json!(value));
    }
    write_active_config_file(&config)
}

pub fn get_include_co_authored_by_droid_for_home(home_dir: &Path) -> Result<bool, String> {
    let config = match read_config_file_for_home(home_dir) {
        ConfigReadResult::Ok(value) => value,
        ConfigReadResult::NotFound => return Ok(true),
        ConfigReadResult::ParseError(_) => return Ok(true),
    };

    let value = config
        .get("includeCoAuthoredByDroid")
        .and_then(|v| v.as_bool())
        .unwrap_or(true);

    Ok(value)
}

pub fn get_include_co_authored_by_droid() -> Result<bool, String> {
    let config = match read_active_config_file() {
        ConfigReadResult::Ok(value) => value,
        ConfigReadResult::NotFound => return Ok(true),
        ConfigReadResult::ParseError(_) => return Ok(true),
    };
    let value = config
        .get("includeCoAuthoredByDroid")
        .and_then(|v| v.as_bool())
        .unwrap_or(true);
    Ok(value)
}

pub fn save_include_co_authored_by_droid_for_home(
    home_dir: &Path,
    enabled: bool,
) -> Result<(), String> {
    let mut config = match read_config_file_for_home(home_dir) {
        ConfigReadResult::Ok(value) => value,
        ConfigReadResult::NotFound => serde_json::json!({}),
        ConfigReadResult::ParseError(e) => {
            return Err(format!("{CONFIG_PARSE_ERROR_PREFIX} {e}"));
        }
    };

    if let Some(obj) = config.as_object_mut() {
        obj.insert(
            "includeCoAuthoredByDroid".to_string(),
            serde_json::json!(enabled),
        );
    }

    write_config_file_for_home(home_dir, &config)
}

pub fn save_include_co_authored_by_droid(enabled: bool) -> Result<(), String> {
    let mut config = match read_active_config_file() {
        ConfigReadResult::Ok(value) => value,
        ConfigReadResult::NotFound => serde_json::json!({}),
        ConfigReadResult::ParseError(e) => {
            return Err(format!("{CONFIG_PARSE_ERROR_PREFIX} {e}"));
        }
    };
    if let Some(obj) = config.as_object_mut() {
        obj.insert(
            "includeCoAuthoredByDroid".to_string(),
            serde_json::json!(enabled),
        );
    }
    write_active_config_file(&config)
}

pub fn get_show_thinking_in_main_view_for_home(home_dir: &Path) -> Result<bool, String> {
    let config = match read_config_file_for_home(home_dir) {
        ConfigReadResult::Ok(value) => value,
        ConfigReadResult::NotFound => return Ok(false),
        ConfigReadResult::ParseError(_) => return Ok(false),
    };

    let value = config
        .get("showThinkingInMainView")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);

    Ok(value)
}

pub fn get_show_thinking_in_main_view() -> Result<bool, String> {
    let config = match read_active_config_file() {
        ConfigReadResult::Ok(value) => value,
        ConfigReadResult::NotFound => return Ok(false),
        ConfigReadResult::ParseError(_) => return Ok(false),
    };
    let value = config
        .get("showThinkingInMainView")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);
    Ok(value)
}

pub fn save_show_thinking_in_main_view_for_home(
    home_dir: &Path,
    enabled: bool,
) -> Result<(), String> {
    let mut config = match read_config_file_for_home(home_dir) {
        ConfigReadResult::Ok(value) => value,
        ConfigReadResult::NotFound => serde_json::json!({}),
        ConfigReadResult::ParseError(e) => {
            return Err(format!("{CONFIG_PARSE_ERROR_PREFIX} {e}"));
        }
    };

    if let Some(obj) = config.as_object_mut() {
        obj.insert(
            "showThinkingInMainView".to_string(),
            serde_json::json!(enabled),
        );
    }

    write_config_file_for_home(home_dir, &config)
}

pub fn save_show_thinking_in_main_view(enabled: bool) -> Result<(), String> {
    let mut config = match read_active_config_file() {
        ConfigReadResult::Ok(value) => value,
        ConfigReadResult::NotFound => serde_json::json!({}),
        ConfigReadResult::ParseError(e) => {
            return Err(format!("{CONFIG_PARSE_ERROR_PREFIX} {e}"));
        }
    };
    if let Some(obj) = config.as_object_mut() {
        obj.insert(
            "showThinkingInMainView".to_string(),
            serde_json::json!(enabled),
        );
    }
    write_active_config_file(&config)
}

pub fn get_mission_model_settings_for_home(
    home_dir: &Path,
) -> Result<MissionModelSettings, String> {
    let config = match read_config_file_for_home(home_dir) {
        ConfigReadResult::Ok(value) => value,
        ConfigReadResult::NotFound => {
            return Ok(MissionModelSettings {
                worker_model: None,
                worker_reasoning_effort: None,
                validation_worker_model: None,
                validation_worker_reasoning_effort: None,
            });
        }
        ConfigReadResult::ParseError(_) => {
            return Ok(MissionModelSettings {
                worker_model: None,
                worker_reasoning_effort: None,
                validation_worker_model: None,
                validation_worker_reasoning_effort: None,
            });
        }
    };

    let settings = config
        .get("missionModelSettings")
        .and_then(|v| serde_json::from_value(v.clone()).ok())
        .unwrap_or(MissionModelSettings {
            worker_model: None,
            worker_reasoning_effort: None,
            validation_worker_model: None,
            validation_worker_reasoning_effort: None,
        });

    Ok(settings)
}

pub fn get_mission_model_settings() -> Result<MissionModelSettings, String> {
    let config = match read_active_config_file() {
        ConfigReadResult::Ok(value) => value,
        ConfigReadResult::NotFound => {
            return Ok(MissionModelSettings {
                worker_model: None,
                worker_reasoning_effort: None,
                validation_worker_model: None,
                validation_worker_reasoning_effort: None,
            });
        }
        ConfigReadResult::ParseError(_) => {
            return Ok(MissionModelSettings {
                worker_model: None,
                worker_reasoning_effort: None,
                validation_worker_model: None,
                validation_worker_reasoning_effort: None,
            });
        }
    };
    let settings = config
        .get("missionModelSettings")
        .and_then(|v| serde_json::from_value(v.clone()).ok())
        .unwrap_or(MissionModelSettings {
            worker_model: None,
            worker_reasoning_effort: None,
            validation_worker_model: None,
            validation_worker_reasoning_effort: None,
        });
    Ok(settings)
}

pub fn save_mission_model_settings_for_home(
    home_dir: &Path,
    settings: MissionModelSettings,
) -> Result<(), String> {
    let mut config = match read_config_file_for_home(home_dir) {
        ConfigReadResult::Ok(value) => value,
        ConfigReadResult::NotFound => serde_json::json!({}),
        ConfigReadResult::ParseError(e) => {
            return Err(format!("{CONFIG_PARSE_ERROR_PREFIX} {e}"));
        }
    };

    let settings_value = serde_json::to_value(&settings)
        .map_err(|e| format!("Failed to serialize mission model settings: {e}"))?;

    if let Some(obj) = config.as_object_mut() {
        obj.insert("missionModelSettings".to_string(), settings_value);
    }

    write_config_file_for_home(home_dir, &config)
}

pub fn save_mission_model_settings(settings: MissionModelSettings) -> Result<(), String> {
    let mut config = match read_active_config_file() {
        ConfigReadResult::Ok(value) => value,
        ConfigReadResult::NotFound => serde_json::json!({}),
        ConfigReadResult::ParseError(e) => {
            return Err(format!("{CONFIG_PARSE_ERROR_PREFIX} {e}"));
        }
    };
    let settings_value = serde_json::to_value(&settings)
        .map_err(|e| format!("Failed to serialize mission model settings: {e}"))?;
    if let Some(obj) = config.as_object_mut() {
        obj.insert("missionModelSettings".to_string(), settings_value);
    }
    write_active_config_file(&config)
}

pub fn get_session_default_settings_for_home(
    home_dir: &Path,
) -> Result<SessionDefaultSettings, String> {
    let config = match read_config_file_for_home(home_dir) {
        ConfigReadResult::Ok(value) => value,
        ConfigReadResult::NotFound => {
            return Ok(SessionDefaultSettings {
                model: None,
                reasoning_effort: None,
                spec_mode_model: None,
                spec_mode_reasoning_effort: None,
                autonomy_mode: None,
            });
        }
        ConfigReadResult::ParseError(_) => {
            return Ok(SessionDefaultSettings {
                model: None,
                reasoning_effort: None,
                spec_mode_model: None,
                spec_mode_reasoning_effort: None,
                autonomy_mode: None,
            });
        }
    };

    let settings = config
        .get("sessionDefaultSettings")
        .and_then(|v| serde_json::from_value(v.clone()).ok())
        .unwrap_or(SessionDefaultSettings {
            model: None,
            reasoning_effort: None,
            spec_mode_model: None,
            spec_mode_reasoning_effort: None,
            autonomy_mode: None,
        });

    Ok(settings)
}

pub fn get_session_default_settings() -> Result<SessionDefaultSettings, String> {
    let config = match read_active_config_file() {
        ConfigReadResult::Ok(value) => value,
        ConfigReadResult::NotFound => {
            return Ok(SessionDefaultSettings {
                model: None,
                reasoning_effort: None,
                spec_mode_model: None,
                spec_mode_reasoning_effort: None,
                autonomy_mode: None,
            });
        }
        ConfigReadResult::ParseError(_) => {
            return Ok(SessionDefaultSettings {
                model: None,
                reasoning_effort: None,
                spec_mode_model: None,
                spec_mode_reasoning_effort: None,
                autonomy_mode: None,
            });
        }
    };
    let settings = config
        .get("sessionDefaultSettings")
        .and_then(|v| serde_json::from_value(v.clone()).ok())
        .unwrap_or(SessionDefaultSettings {
            model: None,
            reasoning_effort: None,
            spec_mode_model: None,
            spec_mode_reasoning_effort: None,
            autonomy_mode: None,
        });
    Ok(settings)
}

pub fn save_session_default_settings_for_home(
    home_dir: &Path,
    settings: SessionDefaultSettings,
) -> Result<(), String> {
    let mut config = match read_config_file_for_home(home_dir) {
        ConfigReadResult::Ok(value) => value,
        ConfigReadResult::NotFound => serde_json::json!({}),
        ConfigReadResult::ParseError(e) => {
            return Err(format!("{CONFIG_PARSE_ERROR_PREFIX} {e}"));
        }
    };

    let settings_value = serde_json::to_value(&settings)
        .map_err(|e| format!("Failed to serialize session default settings: {e}"))?;

    if let Some(obj) = config.as_object_mut() {
        obj.insert("sessionDefaultSettings".to_string(), settings_value);
    }

    write_config_file_for_home(home_dir, &config)
}

pub fn save_session_default_settings(settings: SessionDefaultSettings) -> Result<(), String> {
    let mut config = match read_active_config_file() {
        ConfigReadResult::Ok(value) => value,
        ConfigReadResult::NotFound => serde_json::json!({}),
        ConfigReadResult::ParseError(e) => {
            return Err(format!("{CONFIG_PARSE_ERROR_PREFIX} {e}"));
        }
    };
    let settings_value = serde_json::to_value(&settings)
        .map_err(|e| format!("Failed to serialize session default settings: {e}"))?;
    if let Some(obj) = config.as_object_mut() {
        obj.insert("sessionDefaultSettings".to_string(), settings_value);
    }
    write_active_config_file(&config)
}

// ============================================================================
// Compaction Settings
// ============================================================================

pub fn get_compaction_model_mode_for_home(home_dir: &Path) -> Result<String, String> {
    let config = match read_config_file_for_home(home_dir) {
        ConfigReadResult::Ok(value) => value,
        ConfigReadResult::NotFound => return Ok("current-model".to_string()),
        ConfigReadResult::ParseError(_) => return Ok("current-model".to_string()),
    };

    let value = config
        .get("compactionModelMode")
        .and_then(|v| v.as_str())
        .map(String::from)
        .unwrap_or_else(|| "current-model".to_string());

    Ok(value)
}

pub fn get_compaction_model_mode() -> Result<String, String> {
    let config = match read_active_config_file() {
        ConfigReadResult::Ok(value) => value,
        ConfigReadResult::NotFound => return Ok("current-model".to_string()),
        ConfigReadResult::ParseError(_) => return Ok("current-model".to_string()),
    };
    let value = config
        .get("compactionModelMode")
        .and_then(|v| v.as_str())
        .map(String::from)
        .unwrap_or_else(|| "current-model".to_string());
    Ok(value)
}

pub fn save_compaction_model_mode_for_home(home_dir: &Path, value: &str) -> Result<(), String> {
    let mut config = match read_config_file_for_home(home_dir) {
        ConfigReadResult::Ok(value) => value,
        ConfigReadResult::NotFound => serde_json::json!({}),
        ConfigReadResult::ParseError(e) => {
            return Err(format!("{CONFIG_PARSE_ERROR_PREFIX} {e}"));
        }
    };

    if let Some(obj) = config.as_object_mut() {
        obj.insert("compactionModelMode".to_string(), serde_json::json!(value));
    }

    write_config_file_for_home(home_dir, &config)
}

pub fn save_compaction_model_mode(value: &str) -> Result<(), String> {
    let mut config = match read_active_config_file() {
        ConfigReadResult::Ok(value) => value,
        ConfigReadResult::NotFound => serde_json::json!({}),
        ConfigReadResult::ParseError(e) => {
            return Err(format!("{CONFIG_PARSE_ERROR_PREFIX} {e}"));
        }
    };
    if let Some(obj) = config.as_object_mut() {
        obj.insert("compactionModelMode".to_string(), serde_json::json!(value));
    }
    write_active_config_file(&config)
}

pub fn get_compaction_token_limit_for_home(home_dir: &Path) -> Result<i32, String> {
    let config = match read_config_file_for_home(home_dir) {
        ConfigReadResult::Ok(value) => value,
        ConfigReadResult::NotFound => return Ok(200_000),
        ConfigReadResult::ParseError(_) => return Ok(200_000),
    };

    let value = config
        .get("compactionTokenLimit")
        .and_then(|v| v.as_i64())
        .map(|v| v as i32)
        .unwrap_or(200_000);

    Ok(value)
}

pub fn get_compaction_token_limit() -> Result<i32, String> {
    let config = match read_active_config_file() {
        ConfigReadResult::Ok(value) => value,
        ConfigReadResult::NotFound => return Ok(200_000),
        ConfigReadResult::ParseError(_) => return Ok(200_000),
    };
    let value = config
        .get("compactionTokenLimit")
        .and_then(|v| v.as_i64())
        .map(|v| v as i32)
        .unwrap_or(200_000);
    Ok(value)
}

pub fn save_compaction_token_limit_for_home(home_dir: &Path, value: i32) -> Result<(), String> {
    let mut config = match read_config_file_for_home(home_dir) {
        ConfigReadResult::Ok(value) => value,
        ConfigReadResult::NotFound => serde_json::json!({}),
        ConfigReadResult::ParseError(e) => {
            return Err(format!("{CONFIG_PARSE_ERROR_PREFIX} {e}"));
        }
    };

    if let Some(obj) = config.as_object_mut() {
        obj.insert("compactionTokenLimit".to_string(), serde_json::json!(value));
    }

    write_config_file_for_home(home_dir, &config)
}

pub fn save_compaction_token_limit(value: i32) -> Result<(), String> {
    let mut config = match read_active_config_file() {
        ConfigReadResult::Ok(value) => value,
        ConfigReadResult::NotFound => serde_json::json!({}),
        ConfigReadResult::ParseError(e) => {
            return Err(format!("{CONFIG_PARSE_ERROR_PREFIX} {e}"));
        }
    };
    if let Some(obj) = config.as_object_mut() {
        obj.insert("compactionTokenLimit".to_string(), serde_json::json!(value));
    }
    write_active_config_file(&config)
}

pub fn get_compaction_token_limit_per_model_for_home(
    home_dir: &Path,
) -> Result<HashMap<String, i32>, String> {
    let config = match read_config_file_for_home(home_dir) {
        ConfigReadResult::Ok(value) => value,
        ConfigReadResult::NotFound => return Ok(HashMap::new()),
        ConfigReadResult::ParseError(_) => return Ok(HashMap::new()),
    };

    let overrides = config
        .get("compactionTokenLimitPerModel")
        .and_then(|v| serde_json::from_value::<HashMap<String, i32>>(v.clone()).ok())
        .unwrap_or_default();

    Ok(overrides)
}

pub fn get_compaction_token_limit_per_model() -> Result<HashMap<String, i32>, String> {
    let config = match read_active_config_file() {
        ConfigReadResult::Ok(value) => value,
        ConfigReadResult::NotFound => return Ok(HashMap::new()),
        ConfigReadResult::ParseError(_) => return Ok(HashMap::new()),
    };
    let overrides = config
        .get("compactionTokenLimitPerModel")
        .and_then(|v| serde_json::from_value::<HashMap<String, i32>>(v.clone()).ok())
        .unwrap_or_default();
    Ok(overrides)
}

pub fn save_compaction_token_limit_per_model_for_home(
    home_dir: &Path,
    overrides: HashMap<String, i32>,
) -> Result<(), String> {
    let mut config = match read_config_file_for_home(home_dir) {
        ConfigReadResult::Ok(value) => value,
        ConfigReadResult::NotFound => serde_json::json!({}),
        ConfigReadResult::ParseError(e) => {
            return Err(format!("{CONFIG_PARSE_ERROR_PREFIX} {e}"));
        }
    };

    let overrides_value = serde_json::to_value(&overrides)
        .map_err(|e| format!("Failed to serialize compaction token limit per model: {e}"))?;

    if let Some(obj) = config.as_object_mut() {
        obj.insert("compactionTokenLimitPerModel".to_string(), overrides_value);
    }

    write_config_file_for_home(home_dir, &config)
}

pub fn save_compaction_token_limit_per_model(
    overrides: HashMap<String, i32>,
) -> Result<(), String> {
    let mut config = match read_active_config_file() {
        ConfigReadResult::Ok(value) => value,
        ConfigReadResult::NotFound => serde_json::json!({}),
        ConfigReadResult::ParseError(e) => {
            return Err(format!("{CONFIG_PARSE_ERROR_PREFIX} {e}"));
        }
    };
    let overrides_value = serde_json::to_value(&overrides)
        .map_err(|e| format!("Failed to serialize compaction token limit per model: {e}"))?;
    if let Some(obj) = config.as_object_mut() {
        obj.insert("compactionTokenLimitPerModel".to_string(), overrides_value);
    }
    write_active_config_file(&config)
}
