//! OpenClaw configuration management commands.
//!
//! Provides Profile CRUD and supports applying profiles to `~/.openclaw/` config files.

use chrono::Utc;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use specta::Type;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use uuid::Uuid;

use super::paths;

// ============================================================================
// Types
// ============================================================================

/// OpenClaw Model definition
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct OpenClawModel {
    pub id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(default)]
    pub reasoning: bool,
    #[serde(default)]
    pub input: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub context_window: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_tokens: Option<u32>,
}

/// OpenClaw Provider configuration
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct OpenClawProviderConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub base_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub api_key: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub api: Option<String>,
    #[serde(default)]
    pub models: Vec<OpenClawModel>,
}

/// Block streaming chunk configuration
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct BlockStreamingChunk {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub min_chars: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_chars: Option<u32>,
}

/// Block streaming coalesce configuration
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct BlockStreamingCoalesce {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub idle_ms: Option<u32>,
}

/// Telegram channel configuration
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct TelegramChannelConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub block_streaming: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub chunk_mode: Option<String>,
}

/// Block streaming configuration
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct BlockStreamingConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub block_streaming_default: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub block_streaming_break: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub block_streaming_chunk: Option<BlockStreamingChunk>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub block_streaming_coalesce: Option<BlockStreamingCoalesce>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub telegram_channel: Option<TelegramChannelConfig>,
}

/// OpenClaw Profile (stored in DroidGear)
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct OpenClawProfile {
    pub id: String,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    pub created_at: String,
    pub updated_at: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default_model: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub failover_models: Option<Vec<String>>,
    #[serde(default)]
    pub providers: HashMap<String, OpenClawProviderConfig>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub block_streaming_config: Option<BlockStreamingConfig>,
}

/// OpenClaw config status
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct OpenClawConfigStatus {
    pub config_exists: bool,
    pub config_path: String,
}

/// Current OpenClaw configuration (from config files)
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct OpenClawCurrentConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default_model: Option<String>,
    #[serde(default)]
    pub providers: HashMap<String, OpenClawProviderConfig>,
}

// ============================================================================
// Path Helpers
// ============================================================================

fn get_home_dir() -> Result<PathBuf, String> {
    dirs::home_dir().ok_or("Failed to get home directory".to_string())
}

fn get_droidgear_openclaw_dir() -> Result<PathBuf, String> {
    Ok(get_home_dir()?.join(".droidgear").join("openclaw"))
}

fn get_profiles_dir() -> Result<PathBuf, String> {
    let dir = get_droidgear_openclaw_dir()?.join("profiles");
    if !dir.exists() {
        std::fs::create_dir_all(&dir)
            .map_err(|e| format!("Failed to create openclaw profiles directory: {e}"))?;
    }
    Ok(dir)
}

fn get_active_profile_path() -> Result<PathBuf, String> {
    let dir = get_droidgear_openclaw_dir()?;
    if !dir.exists() {
        std::fs::create_dir_all(&dir)
            .map_err(|e| format!("Failed to create openclaw directory: {e}"))?;
    }
    Ok(dir.join("active-profile.txt"))
}

fn get_openclaw_config_dir() -> Result<PathBuf, String> {
    let dir = paths::get_openclaw_home()?;
    if !dir.exists() {
        std::fs::create_dir_all(&dir)
            .map_err(|e| format!("Failed to create openclaw config directory: {e}"))?;
    }
    Ok(dir)
}

fn get_openclaw_config_path() -> Result<PathBuf, String> {
    Ok(get_openclaw_config_dir()?.join("openclaw.json"))
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

fn get_profile_path(id: &str) -> Result<PathBuf, String> {
    validate_profile_id(id)?;
    Ok(get_profiles_dir()?.join(format!("{id}.json")))
}

// ============================================================================
// File Helpers
// ============================================================================

fn atomic_write(path: &Path, bytes: &[u8]) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        if !parent.exists() {
            std::fs::create_dir_all(parent)
                .map_err(|e| format!("Failed to create directory: {e}"))?;
        }
    }

    let temp_path = path.with_extension("tmp");
    std::fs::write(&temp_path, bytes).map_err(|e| format!("Failed to write file: {e}"))?;
    std::fs::rename(&temp_path, path).map_err(|e| {
        let _ = std::fs::remove_file(&temp_path);
        format!("Failed to finalize file: {e}")
    })?;
    Ok(())
}

fn now_rfc3339() -> String {
    Utc::now().to_rfc3339()
}

fn read_profile_file(path: &Path) -> Result<OpenClawProfile, String> {
    let s = std::fs::read_to_string(path).map_err(|e| format!("Failed to read profile: {e}"))?;
    serde_json::from_str::<OpenClawProfile>(&s).map_err(|e| format!("Invalid profile JSON: {e}"))
}

fn write_profile_file(profile: &OpenClawProfile) -> Result<(), String> {
    let path = get_profile_path(&profile.id)?;
    let s = serde_json::to_string_pretty(profile)
        .map_err(|e| format!("Failed to serialize profile JSON: {e}"))?;
    atomic_write(&path, s.as_bytes())
}

fn load_profile_by_id(id: &str) -> Result<OpenClawProfile, String> {
    let path = get_profile_path(id)?;
    read_profile_file(&path)
}

// ============================================================================
// Config File Helpers
// ============================================================================

/// Paths that should be replaced instead of deep merged
const REPLACE_PATHS: &[&[&str]] = &[
    &["models", "providers"],
    &["agents", "defaults", "models"],
    &["agents", "defaults", "blockStreamingDefault"],
    &["agents", "defaults", "blockStreamingBreak"],
    &["agents", "defaults", "blockStreamingChunk"],
    &["agents", "defaults", "blockStreamingCoalesce"],
];

/// Deep merge with path-based replacement strategy.
/// Paths in REPLACE_PATHS are replaced entirely instead of deep merged.
fn deep_merge_with_replace(base: &mut Value, overlay: &Value, current_path: &[String]) {
    // Check if current path should be replaced
    let should_replace = REPLACE_PATHS.iter().any(|replace_path| {
        replace_path.len() == current_path.len()
            && replace_path
                .iter()
                .zip(current_path.iter())
                .all(|(a, b)| *a == b)
    });

    if should_replace {
        *base = overlay.clone();
        return;
    }

    match (base, overlay) {
        (Value::Object(base_map), Value::Object(overlay_map)) => {
            for (key, overlay_val) in overlay_map {
                let mut new_path = current_path.to_vec();
                new_path.push(key.clone());

                match base_map.get_mut(key) {
                    Some(base_val) => deep_merge_with_replace(base_val, overlay_val, &new_path),
                    None => {
                        base_map.insert(key.clone(), overlay_val.clone());
                    }
                }
            }
        }
        (base, overlay) => *base = overlay.clone(),
    }
}

/// Build OpenClaw config JSON structure from profile
fn build_openclaw_config(profile: &OpenClawProfile) -> Value {
    let mut config = serde_json::Map::new();

    // Collect all model refs from providers for agents.defaults.models
    let mut all_model_refs: Vec<String> = Vec::new();
    for (provider_id, provider) in &profile.providers {
        for model in &provider.models {
            all_model_refs.push(format!("{provider_id}/{}", model.id));
        }
    }

    // agents.defaults.model.primary and agents.defaults.models
    if profile.default_model.is_some() || !all_model_refs.is_empty() {
        let mut agents = serde_json::Map::new();
        let mut defaults = serde_json::Map::new();

        if let Some(ref model) = profile.default_model {
            let mut model_obj = serde_json::Map::new();
            model_obj.insert("primary".to_string(), Value::String(model.clone()));
            // Write failover list if present and non-empty
            if let Some(ref failover) = profile.failover_models {
                if !failover.is_empty() {
                    model_obj.insert(
                        "failover".to_string(),
                        Value::Array(failover.iter().map(|s| Value::String(s.clone())).collect()),
                    );
                }
            }
            defaults.insert("model".to_string(), Value::Object(model_obj));
        }

        if !all_model_refs.is_empty() {
            let mut models_map = serde_json::Map::new();
            for model_ref in all_model_refs {
                models_map.insert(model_ref, Value::Object(serde_json::Map::new()));
            }
            defaults.insert("models".to_string(), Value::Object(models_map));
        }

        agents.insert("defaults".to_string(), Value::Object(defaults));
        config.insert("agents".to_string(), Value::Object(agents));
    }

    // models.providers (only if there are custom providers)
    if !profile.providers.is_empty() {
        let mut models = serde_json::Map::new();
        models.insert("mode".to_string(), Value::String("merge".to_string()));

        let mut providers = serde_json::Map::new();
        for (id, provider) in &profile.providers {
            let mut provider_obj = serde_json::Map::new();

            if let Some(ref base_url) = provider.base_url {
                provider_obj.insert("baseUrl".to_string(), Value::String(base_url.clone()));
            }
            if let Some(ref api_key) = provider.api_key {
                provider_obj.insert("apiKey".to_string(), Value::String(api_key.clone()));
            }
            if let Some(ref api) = provider.api {
                provider_obj.insert("api".to_string(), Value::String(api.clone()));
            }

            if !provider.models.is_empty() {
                let models_arr: Vec<Value> = provider
                    .models
                    .iter()
                    .map(|m| {
                        let mut model_obj = serde_json::Map::new();
                        model_obj.insert("id".to_string(), Value::String(m.id.clone()));
                        if let Some(ref name) = m.name {
                            model_obj.insert("name".to_string(), Value::String(name.clone()));
                        }
                        model_obj.insert("reasoning".to_string(), Value::Bool(m.reasoning));
                        if !m.input.is_empty() {
                            model_obj.insert(
                                "input".to_string(),
                                Value::Array(
                                    m.input.iter().map(|s| Value::String(s.clone())).collect(),
                                ),
                            );
                        }
                        if let Some(cw) = m.context_window {
                            model_obj.insert("contextWindow".to_string(), Value::Number(cw.into()));
                        }
                        if let Some(mt) = m.max_tokens {
                            model_obj.insert("maxTokens".to_string(), Value::Number(mt.into()));
                        }
                        Value::Object(model_obj)
                    })
                    .collect();
                provider_obj.insert("models".to_string(), Value::Array(models_arr));
            }

            providers.insert(id.clone(), Value::Object(provider_obj));
        }

        models.insert("providers".to_string(), Value::Object(providers));
        config.insert("models".to_string(), Value::Object(models));
    }

    // Block streaming config (agents.defaults block streaming settings)
    if let Some(ref bs_config) = profile.block_streaming_config {
        let agents = config
            .entry("agents".to_string())
            .or_insert_with(|| Value::Object(serde_json::Map::new()));
        if let Value::Object(agents_map) = agents {
            let defaults = agents_map
                .entry("defaults".to_string())
                .or_insert_with(|| Value::Object(serde_json::Map::new()));
            if let Value::Object(defaults_map) = defaults {
                if let Some(ref val) = bs_config.block_streaming_default {
                    defaults_map.insert(
                        "blockStreamingDefault".to_string(),
                        Value::String(val.clone()),
                    );
                }
                if let Some(ref val) = bs_config.block_streaming_break {
                    defaults_map.insert(
                        "blockStreamingBreak".to_string(),
                        Value::String(val.clone()),
                    );
                }
                if let Some(ref chunk) = bs_config.block_streaming_chunk {
                    let mut chunk_obj = serde_json::Map::new();
                    if let Some(min) = chunk.min_chars {
                        chunk_obj.insert("minChars".to_string(), Value::Number(min.into()));
                    }
                    if let Some(max) = chunk.max_chars {
                        chunk_obj.insert("maxChars".to_string(), Value::Number(max.into()));
                    }
                    if !chunk_obj.is_empty() {
                        defaults_map
                            .insert("blockStreamingChunk".to_string(), Value::Object(chunk_obj));
                    }
                }
                if let Some(ref coalesce) = bs_config.block_streaming_coalesce {
                    if let Some(idle) = coalesce.idle_ms {
                        let mut coalesce_obj = serde_json::Map::new();
                        coalesce_obj.insert("idleMs".to_string(), Value::Number(idle.into()));
                        defaults_map.insert(
                            "blockStreamingCoalesce".to_string(),
                            Value::Object(coalesce_obj),
                        );
                    }
                }
            }
        }

        // Telegram channel config (channels.telegram)
        if let Some(ref telegram) = bs_config.telegram_channel {
            let channels = config
                .entry("channels".to_string())
                .or_insert_with(|| Value::Object(serde_json::Map::new()));
            if let Value::Object(channels_map) = channels {
                let telegram_obj = channels_map
                    .entry("telegram".to_string())
                    .or_insert_with(|| Value::Object(serde_json::Map::new()));
                if let Value::Object(telegram_map) = telegram_obj {
                    if let Some(bs) = telegram.block_streaming {
                        telegram_map.insert("blockStreaming".to_string(), Value::Bool(bs));
                    }
                    if let Some(ref mode) = telegram.chunk_mode {
                        telegram_map.insert("chunkMode".to_string(), Value::String(mode.clone()));
                    }
                }
            }
        }
    }

    Value::Object(config)
}

/// Parse OpenClaw config JSON into profile fields
fn parse_openclaw_config(
    config: &Value,
) -> (
    Option<String>,
    Option<Vec<String>>,
    HashMap<String, OpenClawProviderConfig>,
) {
    let mut default_model = None;
    let mut failover_models = None;
    let mut providers = HashMap::new();

    if let Some(agents) = config.get("agents").and_then(|v| v.as_object()) {
        if let Some(defaults) = agents.get("defaults").and_then(|v| v.as_object()) {
            if let Some(model) = defaults.get("model").and_then(|v| v.as_object()) {
                if let Some(primary) = model.get("primary").and_then(|v| v.as_str()) {
                    default_model = Some(primary.to_string());
                }
                if let Some(failover_arr) = model.get("failover").and_then(|v| v.as_array()) {
                    let list: Vec<String> = failover_arr
                        .iter()
                        .filter_map(|v| v.as_str().map(|s| s.to_string()))
                        .collect();
                    if !list.is_empty() {
                        failover_models = Some(list);
                    }
                }
            }
        }
    }

    if let Some(models) = config.get("models").and_then(|v| v.as_object()) {
        if let Some(providers_obj) = models.get("providers").and_then(|v| v.as_object()) {
            for (id, provider_val) in providers_obj {
                if let Some(provider_obj) = provider_val.as_object() {
                    let mut provider_config = OpenClawProviderConfig {
                        base_url: provider_obj
                            .get("baseUrl")
                            .and_then(|v| v.as_str())
                            .map(|s| s.to_string()),
                        api_key: provider_obj
                            .get("apiKey")
                            .and_then(|v| v.as_str())
                            .map(|s| s.to_string()),
                        api: provider_obj
                            .get("api")
                            .and_then(|v| v.as_str())
                            .map(|s| s.to_string()),
                        models: Vec::new(),
                    };

                    if let Some(models_arr) = provider_obj.get("models").and_then(|v| v.as_array())
                    {
                        for model_val in models_arr {
                            if let Some(model_obj) = model_val.as_object() {
                                let model = OpenClawModel {
                                    id: model_obj
                                        .get("id")
                                        .and_then(|v| v.as_str())
                                        .unwrap_or("")
                                        .to_string(),
                                    name: model_obj
                                        .get("name")
                                        .and_then(|v| v.as_str())
                                        .map(|s| s.to_string()),
                                    reasoning: model_obj
                                        .get("reasoning")
                                        .and_then(|v| v.as_bool())
                                        .unwrap_or(false),
                                    input: model_obj
                                        .get("input")
                                        .and_then(|v| v.as_array())
                                        .map(|arr| {
                                            arr.iter()
                                                .filter_map(|v| v.as_str().map(|s| s.to_string()))
                                                .collect()
                                        })
                                        .unwrap_or_default(),
                                    context_window: model_obj
                                        .get("contextWindow")
                                        .and_then(|v| v.as_u64())
                                        .map(|n| n as u32),
                                    max_tokens: model_obj
                                        .get("maxTokens")
                                        .and_then(|v| v.as_u64())
                                        .map(|n| n as u32),
                                };
                                provider_config.models.push(model);
                            }
                        }
                    }

                    providers.insert(id.clone(), provider_config);
                }
            }
        }
    }

    (default_model, failover_models, providers)
}

/// Read existing openclaw.json config file as JSON Value
fn read_openclaw_config_raw() -> Result<Value, String> {
    let config_path = get_openclaw_config_path()?;
    if !config_path.exists() {
        return Ok(Value::Object(serde_json::Map::new()));
    }
    let s = std::fs::read_to_string(&config_path)
        .map_err(|e| format!("Failed to read config file: {e}"))?;
    serde_json::from_str(&s).map_err(|e| format!("Invalid config JSON: {e}"))
}

fn write_openclaw_config(profile: &OpenClawProfile) -> Result<(), String> {
    let config_path = get_openclaw_config_path()?;

    // Read existing config and merge with replace strategy for model configs
    let mut base_config = read_openclaw_config_raw()?;
    let overlay_config = build_openclaw_config(profile);
    deep_merge_with_replace(&mut base_config, &overlay_config, &[]);

    let s = serde_json::to_string_pretty(&base_config)
        .map_err(|e| format!("Failed to serialize config: {e}"))?;
    atomic_write(&config_path, s.as_bytes())
}

// ============================================================================
// Tauri Commands
// ============================================================================

/// List all OpenClaw Profiles
#[tauri::command]
#[specta::specta]
pub async fn list_openclaw_profiles() -> Result<Vec<OpenClawProfile>, String> {
    let dir = get_profiles_dir()?;
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

    profiles.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
    Ok(profiles)
}

/// Get a single profile by ID
#[tauri::command]
#[specta::specta]
pub async fn get_openclaw_profile(id: String) -> Result<OpenClawProfile, String> {
    load_profile_by_id(&id)
}

/// Save a profile (create or update)
#[tauri::command]
#[specta::specta]
pub async fn save_openclaw_profile(mut profile: OpenClawProfile) -> Result<(), String> {
    if profile.id.trim().is_empty() {
        profile.id = Uuid::new_v4().to_string();
        profile.created_at = now_rfc3339();
    } else if get_profile_path(&profile.id)?.exists() {
        if let Ok(old) = load_profile_by_id(&profile.id) {
            profile.created_at = old.created_at;
        }
    } else if profile.created_at.trim().is_empty() {
        profile.created_at = now_rfc3339();
    }

    profile.updated_at = now_rfc3339();
    write_profile_file(&profile)
}

/// Delete a profile
#[tauri::command]
#[specta::specta]
pub async fn delete_openclaw_profile(id: String) -> Result<(), String> {
    let path = get_profile_path(&id)?;
    if path.exists() {
        std::fs::remove_file(&path).map_err(|e| format!("Failed to delete profile: {e}"))?;
    }

    if let Ok(active) = get_active_profile_id_internal() {
        if active.as_deref() == Some(id.as_str()) {
            let active_path = get_active_profile_path()?;
            let _ = std::fs::remove_file(active_path);
        }
    }

    Ok(())
}

/// Duplicate a profile
#[tauri::command]
#[specta::specta]
pub async fn duplicate_openclaw_profile(
    id: String,
    new_name: String,
) -> Result<OpenClawProfile, String> {
    let mut profile = load_profile_by_id(&id)?;
    profile.id = Uuid::new_v4().to_string();
    profile.name = new_name;
    profile.created_at = now_rfc3339();
    profile.updated_at = profile.created_at.clone();
    write_profile_file(&profile)?;
    Ok(profile)
}

/// Create default profile (when no profiles exist)
/// If openclaw.json exists, initialize profile from its content
#[tauri::command]
#[specta::specta]
pub async fn create_default_openclaw_profile() -> Result<OpenClawProfile, String> {
    let id = Uuid::new_v4().to_string();
    let now = now_rfc3339();

    // Try to read existing openclaw.json config
    let config_path = get_openclaw_config_path()?;
    let (default_model, failover_models, providers) = if config_path.exists() {
        let s = std::fs::read_to_string(&config_path)
            .map_err(|e| format!("Failed to read config file: {e}"))?;
        let config: Value =
            serde_json::from_str(&s).map_err(|e| format!("Invalid config JSON: {e}"))?;
        parse_openclaw_config(&config)
    } else {
        (
            Some("anthropic/claude-sonnet-4-20250514".to_string()),
            None,
            HashMap::new(),
        )
    };

    let profile = OpenClawProfile {
        id,
        name: "Default".to_string(),
        description: Some("Default OpenClaw profile".to_string()),
        created_at: now.clone(),
        updated_at: now,
        default_model,
        failover_models,
        providers,
        block_streaming_config: None,
    };

    write_profile_file(&profile)?;
    Ok(profile)
}

fn get_active_profile_id_internal() -> Result<Option<String>, String> {
    let path = get_active_profile_path()?;
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

/// Get active profile ID
#[tauri::command]
#[specta::specta]
pub async fn get_active_openclaw_profile_id() -> Result<Option<String>, String> {
    get_active_profile_id_internal()
}

fn set_active_profile_id(id: &str) -> Result<(), String> {
    let path = get_active_profile_path()?;
    atomic_write(&path, id.as_bytes())
}

/// Apply a profile to `~/.openclaw/openclaw.json`
#[tauri::command]
#[specta::specta]
pub async fn apply_openclaw_profile(id: String) -> Result<(), String> {
    let profile = load_profile_by_id(&id)?;
    write_openclaw_config(&profile)?;
    set_active_profile_id(&id)?;
    Ok(())
}

/// Get OpenClaw config status
#[tauri::command]
#[specta::specta]
pub async fn get_openclaw_config_status() -> Result<OpenClawConfigStatus, String> {
    let config_path = get_openclaw_config_path()?;
    Ok(OpenClawConfigStatus {
        config_exists: config_path.exists(),
        config_path: config_path.to_string_lossy().to_string(),
    })
}

/// Read current OpenClaw configuration from config file
#[tauri::command]
#[specta::specta]
pub async fn read_openclaw_current_config() -> Result<OpenClawCurrentConfig, String> {
    let config_path = get_openclaw_config_path()?;

    if !config_path.exists() {
        return Ok(OpenClawCurrentConfig {
            default_model: None,
            providers: HashMap::new(),
        });
    }

    let s = std::fs::read_to_string(&config_path)
        .map_err(|e| format!("Failed to read config file: {e}"))?;
    let config: Value =
        serde_json::from_str(&s).map_err(|e| format!("Invalid config JSON: {e}"))?;

    let (default_model, _failover_models, providers) = parse_openclaw_config(&config);

    Ok(OpenClawCurrentConfig {
        default_model,
        providers,
    })
}
