//! Factory settings.json management commands.
//!
//! Handles reading and writing customModels in ~/.factory/settings.json

use serde::{Deserialize, Serialize};
use serde_json::Value;
use specta::Type;
use std::collections::HashMap;
use std::path::PathBuf;
use tauri::AppHandle;

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
    /// Whether the model supports image inputs
    #[serde(skip_serializing_if = "Option::is_none")]
    pub supports_images: Option<bool>,
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
// Helper Functions
// ============================================================================

/// Gets the path to ~/.factory/settings.json
fn get_factory_config_path() -> Result<PathBuf, String> {
    let home_dir = dirs::home_dir().ok_or("Failed to get home directory")?;
    let factory_dir = home_dir.join(".factory");

    // Ensure .factory directory exists
    if !factory_dir.exists() {
        std::fs::create_dir_all(&factory_dir)
            .map_err(|e| format!("Failed to create .factory directory: {e}"))?;
    }

    Ok(factory_dir.join("settings.json"))
}

/// Reads the entire config.json file
fn read_config_file() -> Result<Value, String> {
    let config_path = get_factory_config_path()?;

    if !config_path.exists() {
        // Return empty object if file doesn't exist
        return Ok(serde_json::json!({}));
    }

    let contents = std::fs::read_to_string(&config_path)
        .map_err(|e| format!("Failed to read config file: {e}"))?;

    serde_json::from_str(&contents).map_err(|e| format!("Failed to parse config JSON: {e}"))
}

/// Writes the entire config.json file (atomic write)
fn write_config_file(config: &Value) -> Result<(), String> {
    let config_path = get_factory_config_path()?;
    let temp_path = config_path.with_extension("tmp");

    let json_content = serde_json::to_string_pretty(config)
        .map_err(|e| format!("Failed to serialize config: {e}"))?;

    std::fs::write(&temp_path, json_content)
        .map_err(|e| format!("Failed to write config file: {e}"))?;

    std::fs::rename(&temp_path, &config_path).map_err(|e| {
        let _ = std::fs::remove_file(&temp_path);
        format!("Failed to finalize config file: {e}")
    })?;

    Ok(())
}

// ============================================================================
// Tauri Commands
// ============================================================================

/// Gets the path to the Factory config file
#[tauri::command]
#[specta::specta]
pub fn get_config_path() -> Result<String, String> {
    let path = get_factory_config_path()?;
    Ok(path.to_string_lossy().to_string())
}

/// Loads custom models from settings.json
#[tauri::command]
#[specta::specta]
pub async fn load_custom_models() -> Result<Vec<CustomModel>, String> {
    log::debug!("Loading custom models from settings");

    let config = read_config_file()?;

    let models: Vec<CustomModel> = config
        .get("customModels")
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|v| serde_json::from_value(v.clone()).ok())
                .collect()
        })
        .unwrap_or_default();

    log::info!("Loaded {} custom models", models.len());
    Ok(models)
}

/// Saves custom models to settings.json (preserves other fields)
#[tauri::command]
#[specta::specta]
pub async fn save_custom_models(models: Vec<CustomModel>) -> Result<(), String> {
    log::debug!("Saving {} custom models to settings", models.len());

    let mut config = read_config_file()?;

    // Convert models to JSON value
    let models_value =
        serde_json::to_value(&models).map_err(|e| format!("Failed to serialize models: {e}"))?;

    // Update only the customModels field
    if let Some(obj) = config.as_object_mut() {
        obj.insert("customModels".to_string(), models_value);
    } else {
        config = serde_json::json!({ "customModels": models_value });
    }

    write_config_file(&config)?;

    log::info!("Successfully saved {} custom models", models.len());
    Ok(())
}

/// Fetches available models from a provider API
#[tauri::command]
#[specta::specta]
pub async fn fetch_models(
    _app: AppHandle,
    provider: Provider,
    base_url: String,
    api_key: String,
) -> Result<Vec<ModelInfo>, String> {
    log::debug!(
        "Fetching models from {base_url} for provider {:?}",
        provider
    );

    let client = reqwest::Client::new();

    let models = match provider {
        Provider::Anthropic => fetch_anthropic_models(&client, &base_url, &api_key).await?,
        Provider::Openai | Provider::GenericChatCompletionApi => {
            fetch_openai_models(&client, &base_url, &api_key).await?
        }
    };

    log::info!("Fetched {} models", models.len());
    Ok(models)
}

/// Fetches models from Anthropic API
async fn fetch_anthropic_models(
    client: &reqwest::Client,
    base_url: &str,
    api_key: &str,
) -> Result<Vec<ModelInfo>, String> {
    let url = format!("{}/v1/models", base_url.trim_end_matches('/'));

    let response = client
        .get(&url)
        .header("x-api-key", api_key)
        .header("anthropic-version", "2023-06-01")
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

/// Fetches models from OpenAI-compatible API
async fn fetch_openai_models(
    client: &reqwest::Client,
    base_url: &str,
    api_key: &str,
) -> Result<Vec<ModelInfo>, String> {
    let url = format!("{}/v1/models", base_url.trim_end_matches('/'));

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
