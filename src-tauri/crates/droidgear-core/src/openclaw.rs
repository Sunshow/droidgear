//! OpenClaw configuration management (core).
//!
//! Provides Profile CRUD and supports applying profiles to `~/.openclaw/` config files.

use chrono::Utc;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use specta::Type;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use uuid::Uuid;

use crate::{paths, storage};

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

/// OpenClaw SubAgent identity
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct OpenClawSubAgentIdentity {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub emoji: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
}

/// OpenClaw SubAgent tools config
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct OpenClawSubAgentTools {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub profile: Option<String>,
}

/// OpenClaw SubAgent model config
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct OpenClawSubAgentModel {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub primary: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fallbacks: Option<Vec<String>>,
}

/// OpenClaw SubAgent subagents config (for main agent's allowAgents)
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct OpenClawSubAgentSubagentsConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub allow_agents: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_concurrent: Option<u32>,
}

/// OpenClaw SubAgent definition
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct OpenClawSubAgent {
    pub id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub identity: Option<OpenClawSubAgentIdentity>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model: Option<OpenClawSubAgentModel>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tools: Option<OpenClawSubAgentTools>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub workspace: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub subagents: Option<OpenClawSubAgentSubagentsConfig>,
}

// ============================================================================
// Path Helpers
// ============================================================================

fn droidgear_openclaw_dir_for_home(home_dir: &Path) -> PathBuf {
    home_dir.join(".droidgear").join("openclaw")
}

fn profiles_dir_for_home(home_dir: &Path) -> Result<PathBuf, String> {
    let dir = droidgear_openclaw_dir_for_home(home_dir).join("profiles");
    if !dir.exists() {
        std::fs::create_dir_all(&dir)
            .map_err(|e| format!("Failed to create openclaw profiles directory: {e}"))?;
    }
    Ok(dir)
}

fn active_profile_path_for_home(home_dir: &Path) -> Result<PathBuf, String> {
    let dir = droidgear_openclaw_dir_for_home(home_dir);
    if !dir.exists() {
        std::fs::create_dir_all(&dir)
            .map_err(|e| format!("Failed to create openclaw directory: {e}"))?;
    }
    Ok(dir.join("active-profile.txt"))
}

fn openclaw_config_dir_for_home(home_dir: &Path) -> Result<PathBuf, String> {
    let config_paths = paths::load_config_paths_for_home(home_dir);
    let dir = paths::get_openclaw_home_for_home(home_dir, &config_paths)?;
    if !dir.exists() {
        std::fs::create_dir_all(&dir)
            .map_err(|e| format!("Failed to create openclaw config directory: {e}"))?;
    }
    Ok(dir)
}

fn openclaw_config_path_for_home(home_dir: &Path) -> Result<PathBuf, String> {
    Ok(openclaw_config_dir_for_home(home_dir)?.join("openclaw.json"))
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

fn now_rfc3339() -> String {
    Utc::now().to_rfc3339()
}

// ============================================================================
// File Helpers
// ============================================================================

fn read_profile_file(path: &Path) -> Result<OpenClawProfile, String> {
    let s = std::fs::read_to_string(path).map_err(|e| format!("Failed to read profile: {e}"))?;
    serde_json::from_str::<OpenClawProfile>(&s).map_err(|e| format!("Invalid profile JSON: {e}"))
}

fn write_profile_file(home_dir: &Path, profile: &OpenClawProfile) -> Result<(), String> {
    let path = profile_path_for_home(home_dir, &profile.id)?;
    let s = serde_json::to_string_pretty(profile)
        .map_err(|e| format!("Failed to serialize profile JSON: {e}"))?;
    storage::atomic_write(&path, s.as_bytes())
}

fn load_profile_by_id(home_dir: &Path, id: &str) -> Result<OpenClawProfile, String> {
    let path = profile_path_for_home(home_dir, id)?;
    read_profile_file(&path)
}

// ============================================================================
// Config merge helpers
// ============================================================================
//
// DroidGear's job when saving an OpenClaw profile is to update **only** the
// fields it owns and leave everything else (unknown keys, future schema
// additions, user-added settings, key ordering) byte-for-byte intact wherever
// possible.
//
// Concretely, DroidGear owns these fields:
//   - agents.defaults.model.primary
//   - agents.defaults.model.fallbacks
//   - agents.defaults.blockStreamingDefault
//   - agents.defaults.blockStreamingBreak
//   - agents.defaults.blockStreamingChunk.{minChars,maxChars}
//   - agents.defaults.blockStreamingCoalesce.idleMs
//   - channels.telegram.{blockStreaming,chunkMode}
//   - models.providers.<id>.{baseUrl,apiKey,api}
//   - models.providers.<id>.models[].{id,name,reasoning,input,contextWindow,maxTokens}
//   - models.providers (add/remove provider entries)
//   - agents.defaults.models[<owned-ref>] (add/remove ref entries)
//   - agents.list (managed separately by save_openclaw_subagents)
//
// Everything else is preserved as-is.
//
// Key ordering is preserved thanks to `serde_json/preserve_order`.

/// Deep merge of `overlay` into `base`. Objects are merged recursively;
/// non-object values (including arrays) are replaced wholesale. Used by
/// `save_openclaw_subagents_for_home` to merge per-agent overlays.
fn deep_merge(base: &mut Value, overlay: &Value) {
    match (base, overlay) {
        (Value::Object(base_map), Value::Object(overlay_map)) => {
            for (key, overlay_val) in overlay_map {
                match base_map.get_mut(key) {
                    Some(base_val) => deep_merge(base_val, overlay_val),
                    None => {
                        base_map.insert(key.clone(), overlay_val.clone());
                    }
                }
            }
        }
        (base, overlay) => *base = overlay.clone(),
    }
}

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
                if let Some(failover_arr) = model.get("fallbacks").and_then(|v| v.as_array()) {
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

fn read_openclaw_config_raw_for_home(home_dir: &Path) -> Result<Value, String> {
    let config_path = openclaw_config_path_for_home(home_dir)?;
    if !config_path.exists() {
        return Ok(Value::Object(serde_json::Map::new()));
    }
    let s = std::fs::read_to_string(&config_path)
        .map_err(|e| format!("Failed to read config file: {e}"))?;
    serde_json::from_str(&s).map_err(|e| format!("Invalid config JSON: {e}"))
}

fn write_openclaw_config_for_home(
    home_dir: &Path,
    profile: &OpenClawProfile,
) -> Result<(), String> {
    let config_path = openclaw_config_path_for_home(home_dir)?;

    // Surgically update DroidGear-owned fields on top of the existing config.
    // Everything else — unknown keys, key order, user-added settings — is
    // preserved.
    let mut config = read_openclaw_config_raw_for_home(home_dir)?;
    if !config.is_object() {
        config = Value::Object(serde_json::Map::new());
    }
    apply_profile_in_place(&mut config, profile);

    let s = serde_json::to_string_pretty(&config)
        .map_err(|e| format!("Failed to serialize config: {e}"))?;
    storage::atomic_write(&config_path, s.as_bytes())
}

/// Apply every DroidGear-owned field from `profile` into `config` in place.
///
/// Guarantees:
/// - Unknown top-level keys (gateway, channels.*, bindings, plugins, wizard,
///   tools, session, meta, ...) are not touched.
/// - Within objects DroidGear partially owns (e.g. `agents.defaults`,
///   `agents.defaults.model`, `agents.defaults.blockStreamingChunk`,
///   `models.providers.<id>`, model entries inside `models[]`,
///   `channels.telegram`), only owned fields are written; siblings are
///   preserved.
/// - Empty intermediate objects DroidGear created but did not populate are
///   removed at the end.
fn apply_profile_in_place(config: &mut Value, profile: &OpenClawProfile) {
    let owned_provider_ids: std::collections::HashSet<String> =
        profile.providers.keys().cloned().collect();

    let config_obj = match config.as_object_mut() {
        Some(o) => o,
        None => return,
    };

    // -------------------- agents.defaults.{model, blockStreaming*} --------------------
    let needs_agents_block = profile.default_model.is_some()
        || profile.failover_models.is_some()
        || profile.block_streaming_config.is_some()
        || profile.providers.values().any(|p| !p.models.is_empty())
        || agents_defaults_models_has_owned_ref(config_obj, &owned_provider_ids);

    if needs_agents_block {
        let agents_value = config_obj
            .entry("agents".to_string())
            .or_insert_with(|| Value::Object(serde_json::Map::new()));
        if let Value::Object(agents_map) = agents_value {
            let defaults_value = agents_map
                .entry("defaults".to_string())
                .or_insert_with(|| Value::Object(serde_json::Map::new()));
            if let Value::Object(defaults_map) = defaults_value {
                apply_default_model(defaults_map, profile);
                apply_block_streaming_to_defaults(defaults_map, profile);
                apply_agents_defaults_models(defaults_map, profile, &owned_provider_ids);
            }
        }
    }

    // -------------------- channels.telegram --------------------
    if let Some(bs) = profile
        .block_streaming_config
        .as_ref()
        .and_then(|c| c.telegram_channel.as_ref())
    {
        let channels_value = config_obj
            .entry("channels".to_string())
            .or_insert_with(|| Value::Object(serde_json::Map::new()));
        if let Value::Object(channels_map) = channels_value {
            let telegram_value = channels_map
                .entry("telegram".to_string())
                .or_insert_with(|| Value::Object(serde_json::Map::new()));
            if let Value::Object(telegram_map) = telegram_value {
                if let Some(b) = bs.block_streaming {
                    telegram_map.insert("blockStreaming".to_string(), Value::Bool(b));
                }
                if let Some(ref mode) = bs.chunk_mode {
                    telegram_map.insert("chunkMode".to_string(), Value::String(mode.clone()));
                }
            }
            // Don't drop `channels.telegram` if it ended up empty — the user
            // may have placed a placeholder there. We only ever insert into
            // it (above), never delete.
        }
    }

    // -------------------- models.providers --------------------
    apply_models_providers(config_obj, profile, &owned_provider_ids);

    // -------------------- final cleanup of empties we just created --------------------
    cleanup_empty_dg_blocks(config_obj);
}

/// Helper: return true if `config.agents.defaults.models` already has at
/// least one ref whose provider prefix is owned by the profile (i.e. we may
/// need to prune stale entries even if profile has no models[] currently).
fn agents_defaults_models_has_owned_ref(
    config_obj: &serde_json::Map<String, Value>,
    owned: &std::collections::HashSet<String>,
) -> bool {
    config_obj
        .get("agents")
        .and_then(|v| v.get("defaults"))
        .and_then(|v| v.get("models"))
        .and_then(|v| v.as_object())
        .is_some_and(|m| {
            m.keys().any(|key| {
                let prefix = key.split('/').next().unwrap_or("");
                owned.contains(prefix)
            })
        })
}

/// Update `agents.defaults.model` in place. Only `primary` and `fallbacks`
/// are owned by DroidGear; any unknown sibling keys (e.g. user-added routing
/// metadata) are preserved.
fn apply_default_model(
    defaults_map: &mut serde_json::Map<String, Value>,
    profile: &OpenClawProfile,
) {
    if profile.default_model.is_none() && profile.failover_models.is_none() {
        return;
    }

    let model_value = defaults_map
        .entry("model".to_string())
        .or_insert_with(|| Value::Object(serde_json::Map::new()));
    let model_map = match model_value {
        Value::Object(m) => m,
        // If the user (or a future OpenClaw version) made `model` a string,
        // convert it to the documented object form rather than crashing.
        _ => {
            *model_value = Value::Object(serde_json::Map::new());
            match model_value {
                Value::Object(m) => m,
                _ => unreachable!(),
            }
        }
    };

    if let Some(ref primary) = profile.default_model {
        model_map.insert("primary".to_string(), Value::String(primary.clone()));
    }

    match &profile.failover_models {
        Some(fallbacks) if !fallbacks.is_empty() => {
            model_map.insert(
                "fallbacks".to_string(),
                Value::Array(fallbacks.iter().map(|s| Value::String(s.clone())).collect()),
            );
        }
        Some(_) => {
            // Empty list → user explicitly cleared fallbacks.
            model_map.remove("fallbacks");
        }
        None => {
            // Unset → leave whatever was already there alone.
        }
    }
}

/// Update the `agents.defaults.blockStreaming*` keys in place.
fn apply_block_streaming_to_defaults(
    defaults_map: &mut serde_json::Map<String, Value>,
    profile: &OpenClawProfile,
) {
    let bs = match profile.block_streaming_config.as_ref() {
        Some(b) => b,
        None => return,
    };

    if let Some(ref val) = bs.block_streaming_default {
        defaults_map.insert(
            "blockStreamingDefault".to_string(),
            Value::String(val.clone()),
        );
    }
    if let Some(ref val) = bs.block_streaming_break {
        defaults_map.insert(
            "blockStreamingBreak".to_string(),
            Value::String(val.clone()),
        );
    }
    if let Some(ref chunk) = bs.block_streaming_chunk {
        let chunk_value = defaults_map
            .entry("blockStreamingChunk".to_string())
            .or_insert_with(|| Value::Object(serde_json::Map::new()));
        if let Value::Object(chunk_map) = chunk_value {
            if let Some(min) = chunk.min_chars {
                chunk_map.insert("minChars".to_string(), Value::Number(min.into()));
            }
            if let Some(max) = chunk.max_chars {
                chunk_map.insert("maxChars".to_string(), Value::Number(max.into()));
            }
        }
    }
    if let Some(ref coalesce) = bs.block_streaming_coalesce {
        if let Some(idle) = coalesce.idle_ms {
            let coalesce_value = defaults_map
                .entry("blockStreamingCoalesce".to_string())
                .or_insert_with(|| Value::Object(serde_json::Map::new()));
            if let Value::Object(coalesce_map) = coalesce_value {
                coalesce_map.insert("idleMs".to_string(), Value::Number(idle.into()));
            }
        }
    }
}

/// Reconcile `agents.defaults.models` with the refs implied by
/// `profile.providers[*].models[]`. Only refs whose provider prefix is owned
/// by the profile are added/removed; refs for other providers are untouched,
/// and existing entry contents (`alias`, `params`, `streaming`, ...) are
/// preserved.
fn apply_agents_defaults_models(
    defaults_map: &mut serde_json::Map<String, Value>,
    profile: &OpenClawProfile,
    owned: &std::collections::HashSet<String>,
) {
    let target_refs: std::collections::HashSet<String> = profile
        .providers
        .iter()
        .flat_map(|(pid, p)| p.models.iter().map(move |m| format!("{pid}/{}", m.id)))
        .collect();

    let need_models_block = !target_refs.is_empty() || defaults_map.contains_key("models");
    if !need_models_block {
        return;
    }

    let models_value = defaults_map
        .entry("models".to_string())
        .or_insert_with(|| Value::Object(serde_json::Map::new()));
    let Value::Object(models_map) = models_value else {
        return;
    };

    // Remove stale refs whose provider prefix is owned but the model id is gone.
    let to_remove: Vec<String> = models_map
        .keys()
        .filter(|key| {
            let prefix = key.split('/').next().unwrap_or("");
            owned.contains(prefix) && !target_refs.contains(*key)
        })
        .cloned()
        .collect();
    for key in to_remove {
        models_map.remove(&key);
    }

    // Insert refs we own that aren't already present.
    for r in &target_refs {
        models_map
            .entry(r.clone())
            .or_insert_with(|| Value::Object(serde_json::Map::new()));
    }
}

/// Reconcile `models.providers` with `profile.providers`. The profile is the
/// source of truth for provider identity: any provider in the file but not in
/// the profile is removed (treated as user-deleted). For providers the
/// profile owns, only typed fields (`baseUrl`, `apiKey`, `api`, model entries)
/// are written; everything else is preserved.
fn apply_models_providers(
    config_obj: &mut serde_json::Map<String, Value>,
    profile: &OpenClawProfile,
    owned: &std::collections::HashSet<String>,
) {
    // Skip entirely if there's nothing to do.
    let has_existing_providers = config_obj
        .get("models")
        .and_then(|v| v.get("providers"))
        .is_some();
    if profile.providers.is_empty() && !has_existing_providers {
        return;
    }

    let models_value = config_obj
        .entry("models".to_string())
        .or_insert_with(|| Value::Object(serde_json::Map::new()));
    let Value::Object(models_map) = models_value else {
        return;
    };

    // Default `mode` to "merge" if absent (matches the OpenClaw default).
    models_map
        .entry("mode".to_string())
        .or_insert_with(|| Value::String("merge".to_string()));

    let providers_value = models_map
        .entry("providers".to_string())
        .or_insert_with(|| Value::Object(serde_json::Map::new()));
    let Value::Object(providers_map) = providers_value else {
        return;
    };

    // Drop providers no longer in the profile.
    let to_remove: Vec<String> = providers_map
        .keys()
        .filter(|id| !owned.contains(id.as_str()))
        .cloned()
        .collect();
    for id in to_remove {
        providers_map.remove(&id);
    }

    // Update each owned provider in place.
    for (id, provider) in &profile.providers {
        let provider_value = providers_map
            .entry(id.clone())
            .or_insert_with(|| Value::Object(serde_json::Map::new()));
        if let Value::Object(provider_obj) = provider_value {
            update_provider_in_place(provider_obj, provider);
        }
    }
}

/// Final pass: drop top-level blocks DroidGear created but didn't populate.
/// Existing user blocks (even if empty) are left alone.
fn cleanup_empty_dg_blocks(config_obj: &mut serde_json::Map<String, Value>) {
    // agents.defaults.models → agents.defaults → agents
    if let Some(Value::Object(agents_map)) = config_obj.get_mut("agents") {
        if let Some(Value::Object(defaults_map)) = agents_map.get_mut("defaults") {
            if defaults_map
                .get("models")
                .and_then(|v| v.as_object())
                .is_some_and(|m| m.is_empty())
            {
                defaults_map.remove("models");
            }
            if defaults_map
                .get("blockStreamingChunk")
                .and_then(|v| v.as_object())
                .is_some_and(|m| m.is_empty())
            {
                defaults_map.remove("blockStreamingChunk");
            }
            if defaults_map
                .get("blockStreamingCoalesce")
                .and_then(|v| v.as_object())
                .is_some_and(|m| m.is_empty())
            {
                defaults_map.remove("blockStreamingCoalesce");
            }
            if defaults_map
                .get("model")
                .and_then(|v| v.as_object())
                .is_some_and(|m| m.is_empty())
            {
                defaults_map.remove("model");
            }
        }
        if agents_map
            .get("defaults")
            .and_then(|v| v.as_object())
            .is_some_and(|m| m.is_empty())
        {
            agents_map.remove("defaults");
        }
    }
    if config_obj
        .get("agents")
        .and_then(|v| v.as_object())
        .is_some_and(|m| m.is_empty())
    {
        config_obj.remove("agents");
    }
}

/// Update a provider object in place. Sets typed fields from the profile;
/// removes typed fields when the profile clears them; preserves everything
/// else.
fn update_provider_in_place(
    provider_obj: &mut serde_json::Map<String, Value>,
    provider: &OpenClawProviderConfig,
) {
    set_or_remove_str(provider_obj, "baseUrl", provider.base_url.as_deref());
    set_or_remove_str(provider_obj, "apiKey", provider.api_key.as_deref());
    set_or_remove_str(provider_obj, "api", provider.api.as_deref());

    // Update models[] surgically: match existing entries by id, preserve unknown fields.
    let existing_models: Vec<Value> = provider_obj
        .get("models")
        .and_then(|v| v.as_array())
        .cloned()
        .unwrap_or_default();

    // Build an id → existing-entry map (last write wins, mirroring how
    // OpenClaw treats duplicate ids).
    let mut existing_by_id: HashMap<String, Value> = HashMap::new();
    for entry in existing_models {
        if let Some(id) = entry.get("id").and_then(|v| v.as_str()) {
            existing_by_id.insert(id.to_string(), entry);
        }
    }

    let mut new_models: Vec<Value> = Vec::with_capacity(provider.models.len());
    for m in &provider.models {
        let mut entry = existing_by_id
            .remove(&m.id)
            .and_then(|v| match v {
                Value::Object(_) => Some(v),
                _ => None,
            })
            .unwrap_or_else(|| Value::Object(serde_json::Map::new()));

        if let Value::Object(map) = &mut entry {
            map.insert("id".to_string(), Value::String(m.id.clone()));
            map.insert(
                "name".to_string(),
                Value::String(m.name.as_deref().unwrap_or(&m.id).to_string()),
            );
            map.insert("reasoning".to_string(), Value::Bool(m.reasoning));
            if m.input.is_empty() {
                map.remove("input");
            } else {
                map.insert(
                    "input".to_string(),
                    Value::Array(m.input.iter().map(|s| Value::String(s.clone())).collect()),
                );
            }
            match m.context_window {
                Some(cw) => {
                    map.insert("contextWindow".to_string(), Value::Number(cw.into()));
                }
                None => {
                    map.remove("contextWindow");
                }
            }
            match m.max_tokens {
                Some(mt) => {
                    map.insert("maxTokens".to_string(), Value::Number(mt.into()));
                }
                None => {
                    map.remove("maxTokens");
                }
            }
        }
        new_models.push(entry);
    }

    if new_models.is_empty() {
        provider_obj.remove("models");
    } else {
        provider_obj.insert("models".to_string(), Value::Array(new_models));
    }
}

fn set_or_remove_str(map: &mut serde_json::Map<String, Value>, key: &str, value: Option<&str>) {
    match value {
        Some(s) => {
            map.insert(key.to_string(), Value::String(s.to_string()));
        }
        None => {
            map.remove(key);
        }
    }
}

// ============================================================================
// Profile CRUD
// ============================================================================

pub fn list_openclaw_profiles_for_home(home_dir: &Path) -> Result<Vec<OpenClawProfile>, String> {
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

pub fn get_openclaw_profile_for_home(home_dir: &Path, id: &str) -> Result<OpenClawProfile, String> {
    load_profile_by_id(home_dir, id)
}

pub fn save_openclaw_profile_for_home(
    home_dir: &Path,
    mut profile: OpenClawProfile,
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

pub fn delete_openclaw_profile_for_home(home_dir: &Path, id: &str) -> Result<(), String> {
    let path = profile_path_for_home(home_dir, id)?;
    if path.exists() {
        std::fs::remove_file(&path).map_err(|e| format!("Failed to delete profile: {e}"))?;
    }

    if let Ok(active) = get_active_openclaw_profile_id_for_home(home_dir) {
        if active.as_deref() == Some(id) {
            let active_path = active_profile_path_for_home(home_dir)?;
            let _ = std::fs::remove_file(active_path);
        }
    }

    Ok(())
}

pub fn duplicate_openclaw_profile_for_home(
    home_dir: &Path,
    id: &str,
    new_name: &str,
) -> Result<OpenClawProfile, String> {
    let mut profile = load_profile_by_id(home_dir, id)?;
    profile.id = Uuid::new_v4().to_string();
    profile.name = new_name.to_string();
    profile.created_at = now_rfc3339();
    profile.updated_at = profile.created_at.clone();
    write_profile_file(home_dir, &profile)?;
    Ok(profile)
}

/// Create default profile (when no profiles exist)
/// If openclaw.json exists, initialize profile from its content
pub fn create_default_openclaw_profile_for_home(
    home_dir: &Path,
) -> Result<OpenClawProfile, String> {
    let id = Uuid::new_v4().to_string();
    let now = now_rfc3339();

    let config_path = openclaw_config_path_for_home(home_dir)?;
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

    write_profile_file(home_dir, &profile)?;
    Ok(profile)
}

// ============================================================================
// Active + Apply + status
// ============================================================================

pub fn get_active_openclaw_profile_id_for_home(home_dir: &Path) -> Result<Option<String>, String> {
    let path = active_profile_path_for_home(home_dir)?;
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

fn set_active_profile_id_for_home(home_dir: &Path, id: &str) -> Result<(), String> {
    let path = active_profile_path_for_home(home_dir)?;
    storage::atomic_write(&path, id.as_bytes())
}

pub fn apply_openclaw_profile_for_home(
    home_dir: &Path,
    profile: &OpenClawProfile,
) -> Result<(), String> {
    write_openclaw_config_for_home(home_dir, profile)?;
    set_active_profile_id_for_home(home_dir, &profile.id)?;
    Ok(())
}

pub fn get_openclaw_config_status_for_home(
    home_dir: &Path,
) -> Result<OpenClawConfigStatus, String> {
    let config_path = openclaw_config_path_for_home(home_dir)?;
    Ok(OpenClawConfigStatus {
        config_exists: config_path.exists(),
        config_path: config_path.to_string_lossy().to_string(),
    })
}

pub fn read_openclaw_current_config_for_home(
    home_dir: &Path,
) -> Result<OpenClawCurrentConfig, String> {
    let config_path = openclaw_config_path_for_home(home_dir)?;
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

// ============================================================================
// System wrappers
// ============================================================================

fn system_home_dir() -> Result<PathBuf, String> {
    dirs::home_dir().ok_or_else(|| "Failed to get home directory".to_string())
}

pub fn list_openclaw_profiles() -> Result<Vec<OpenClawProfile>, String> {
    list_openclaw_profiles_for_home(&system_home_dir()?)
}

pub fn get_openclaw_profile(id: &str) -> Result<OpenClawProfile, String> {
    get_openclaw_profile_for_home(&system_home_dir()?, id)
}

pub fn save_openclaw_profile(profile: OpenClawProfile) -> Result<(), String> {
    save_openclaw_profile_for_home(&system_home_dir()?, profile)
}

pub fn delete_openclaw_profile(id: &str) -> Result<(), String> {
    delete_openclaw_profile_for_home(&system_home_dir()?, id)
}

pub fn duplicate_openclaw_profile(id: &str, new_name: &str) -> Result<OpenClawProfile, String> {
    duplicate_openclaw_profile_for_home(&system_home_dir()?, id, new_name)
}

pub fn create_default_openclaw_profile() -> Result<OpenClawProfile, String> {
    create_default_openclaw_profile_for_home(&system_home_dir()?)
}

pub fn get_active_openclaw_profile_id() -> Result<Option<String>, String> {
    get_active_openclaw_profile_id_for_home(&system_home_dir()?)
}

pub fn apply_openclaw_profile(profile: &OpenClawProfile) -> Result<(), String> {
    apply_openclaw_profile_for_home(&system_home_dir()?, profile)
}

pub fn get_openclaw_config_status() -> Result<OpenClawConfigStatus, String> {
    get_openclaw_config_status_for_home(&system_home_dir()?)
}

pub fn read_openclaw_current_config() -> Result<OpenClawCurrentConfig, String> {
    read_openclaw_current_config_for_home(&system_home_dir()?)
}

// ============================================================================
// SubAgents
// ============================================================================

pub fn read_openclaw_subagents_for_home(home_dir: &Path) -> Result<Vec<OpenClawSubAgent>, String> {
    let config = read_openclaw_config_raw_for_home(home_dir)?;

    let mut subagents = Vec::new();
    if let Some(agents) = config.get("agents").and_then(|v| v.as_object()) {
        if let Some(list) = agents.get("list").and_then(|v| v.as_array()) {
            for item in list {
                if let Ok(agent) = serde_json::from_value::<OpenClawSubAgent>(item.clone()) {
                    subagents.push(agent);
                }
            }
        }
    }

    Ok(subagents)
}

pub fn save_openclaw_subagents_for_home(
    home_dir: &Path,
    subagents: Vec<OpenClawSubAgent>,
) -> Result<(), String> {
    let config_path = openclaw_config_path_for_home(home_dir)?;
    let mut config = read_openclaw_config_raw_for_home(home_dir)?;

    // Read existing agents.list as raw Values, indexed by id
    let mut existing_map: std::collections::HashMap<String, Value> =
        std::collections::HashMap::new();
    if let Some(agents) = config.get("agents").and_then(|v| v.as_object()) {
        if let Some(list) = agents.get("list").and_then(|v| v.as_array()) {
            for item in list {
                if let Some(id) = item.get("id").and_then(|v| v.as_str()) {
                    existing_map.insert(id.to_string(), item.clone());
                }
            }
        }
    }

    // Collect all non-main subagent IDs for main's allowAgents
    let non_main_ids: Vec<String> = subagents
        .iter()
        .filter(|a| a.id != "main")
        .map(|a| a.id.clone())
        .collect();

    // Build merged list: for each subagent, merge new data into existing entry
    let mut result_list: Vec<Value> = Vec::new();

    for agent in &subagents {
        let new_value = serde_json::to_value(agent)
            .map_err(|e| format!("Failed to serialize subagent: {e}"))?;

        let merged = if let Some(mut existing) = existing_map.remove(&agent.id) {
            // Deep merge new into existing (new fields override, existing fields preserved)
            deep_merge(&mut existing, &new_value);
            existing
        } else {
            new_value
        };

        result_list.push(merged);
    }

    // Ensure main entry exists with subagents.allowAgents
    if !non_main_ids.is_empty() {
        let has_main = subagents.iter().any(|a| a.id == "main");
        if !has_main {
            // Build main entry, merging with existing main if present
            let allow_agents_value = Value::Array(
                non_main_ids
                    .iter()
                    .map(|s| Value::String(s.clone()))
                    .collect(),
            );
            let mut sa_obj = serde_json::Map::new();
            sa_obj.insert("allowAgents".to_string(), allow_agents_value);

            let mut main_overlay = serde_json::Map::new();
            main_overlay.insert("id".to_string(), Value::String("main".to_string()));
            main_overlay.insert("subagents".to_string(), Value::Object(sa_obj));

            let main_entry = if let Some(mut existing_main) = existing_map.remove("main") {
                deep_merge(&mut existing_main, &Value::Object(main_overlay));
                existing_main
            } else {
                Value::Object(main_overlay)
            };
            // Insert main at the beginning
            result_list.insert(0, main_entry);
        }
    }

    // Build overlay with agents.list
    let mut overlay = serde_json::Map::new();
    let mut agents = serde_json::Map::new();
    agents.insert("list".to_string(), Value::Array(result_list));
    overlay.insert("agents".to_string(), Value::Object(agents));

    deep_merge(&mut config, &Value::Object(overlay));

    let s = serde_json::to_string_pretty(&config)
        .map_err(|e| format!("Failed to serialize config: {e}"))?;
    storage::atomic_write(&config_path, s.as_bytes())
}

pub fn read_openclaw_subagents() -> Result<Vec<OpenClawSubAgent>, String> {
    read_openclaw_subagents_for_home(&system_home_dir()?)
}

pub fn save_openclaw_subagents(subagents: Vec<OpenClawSubAgent>) -> Result<(), String> {
    save_openclaw_subagents_for_home(&system_home_dir()?, subagents)
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    fn empty_profile() -> OpenClawProfile {
        OpenClawProfile {
            id: "test-id".to_string(),
            name: "Test".to_string(),
            description: None,
            created_at: "2025-01-01T00:00:00Z".to_string(),
            updated_at: "2025-01-01T00:00:00Z".to_string(),
            default_model: None,
            failover_models: None,
            providers: HashMap::new(),
            block_streaming_config: None,
        }
    }

    /// Helper: build the full config the way `write_openclaw_config_for_home`
    /// does (apply profile to an empty base) for tests that want to inspect
    /// the JSON structure.
    fn build_full_config(profile: &OpenClawProfile) -> Value {
        let mut config = Value::Object(serde_json::Map::new());
        apply_profile_in_place(&mut config, profile);
        config
    }

    /// Helper: run full roundtrip build → parse and verify invariants.
    fn roundtrip(
        profile: &OpenClawProfile,
    ) -> (
        Option<String>,
        Option<Vec<String>>,
        HashMap<String, OpenClawProviderConfig>,
    ) {
        let config = build_full_config(profile);
        parse_openclaw_config(&config)
    }

    // ------------------------------------------------------------------
    // Roundtrip tests (build → parse → verify)
    // ------------------------------------------------------------------

    #[test]
    fn test_roundtrip_empty_profile() {
        let profile = empty_profile();
        let (model, failovers, providers) = roundtrip(&profile);
        assert!(
            model.is_none(),
            "empty profile should produce no default_model"
        );
        assert!(
            failovers.is_none(),
            "empty profile should produce no failover_models"
        );
        assert!(
            providers.is_empty(),
            "empty profile should produce no providers"
        );
    }

    #[test]
    fn test_roundtrip_default_model_only() {
        let mut profile = empty_profile();
        profile.default_model = Some("anthropic/claude-sonnet-4-20250514".to_string());

        let (model, failovers, providers) = roundtrip(&profile);

        assert_eq!(model.as_deref(), Some("anthropic/claude-sonnet-4-20250514"));
        assert!(failovers.is_none(), "no failovers configured");
        assert!(providers.is_empty());
    }

    #[test]
    fn test_roundtrip_default_model_with_fallbacks() {
        let mut profile = empty_profile();
        profile.default_model = Some("anthropic/claude-opus-4-6".to_string());
        profile.failover_models = Some(vec![
            "minimax/MiniMax-M2.7".to_string(),
            "openai/gpt-5.4-mini".to_string(),
        ]);

        let (model, failovers, providers) = roundtrip(&profile);

        assert_eq!(model.as_deref(), Some("anthropic/claude-opus-4-6"));
        let f = failovers.expect("failovers should be present");
        assert_eq!(f, vec!["minimax/MiniMax-M2.7", "openai/gpt-5.4-mini"]);
        assert!(providers.is_empty());
    }

    #[test]
    fn test_roundtrip_empty_failovers_not_written() {
        // When failover list is empty, the key should not appear in config.
        let mut profile = empty_profile();
        profile.default_model = Some("anthropic/claude-opus-4-6".to_string());
        profile.failover_models = Some(vec![]);

        let config = build_full_config(&profile);

        // Verify `agents.defaults.model` has `primary` but no `fallbacks`
        let agents = config.get("agents").and_then(|v| v.as_object()).unwrap();
        let defaults = agents.get("defaults").and_then(|v| v.as_object()).unwrap();
        let model = defaults.get("model").and_then(|v| v.as_object()).unwrap();
        assert!(model.contains_key("primary"));
        assert!(
            !model.contains_key("fallbacks"),
            "empty failovers should not emit fallbacks key"
        );

        // Parse it back
        let (model_out, failovers_out, _) = parse_openclaw_config(&config);
        assert_eq!(model_out.as_deref(), Some("anthropic/claude-opus-4-6"));
        assert!(
            failovers_out.is_none(),
            "empty failovers should parse back as None"
        );
    }

    #[test]
    fn test_roundtrip_providers_with_models() {
        let mut profile = empty_profile();
        profile.default_model = Some("custom/my-model".to_string());

        let mut providers = HashMap::new();
        providers.insert(
            "custom".to_string(),
            OpenClawProviderConfig {
                base_url: Some("http://localhost:4000/v1".to_string()),
                api_key: Some("sk-test-key".to_string()),
                api: Some("openai-completions".to_string()),
                models: vec![
                    OpenClawModel {
                        id: "my-model".to_string(),
                        name: Some("My Model".to_string()),
                        reasoning: false,
                        input: vec!["text".to_string()],
                        context_window: Some(128000),
                        max_tokens: Some(32000),
                    },
                    OpenClawModel {
                        id: "vision-model".to_string(),
                        name: Some("Vision Model".to_string()),
                        reasoning: true,
                        input: vec!["text".to_string(), "image".to_string()],
                        context_window: Some(200000),
                        max_tokens: Some(4096),
                    },
                ],
            },
        );
        profile.providers = providers;

        let (model, failovers, parsed_providers) = roundtrip(&profile);

        assert_eq!(model.as_deref(), Some("custom/my-model"));
        assert!(failovers.is_none());

        let custom = parsed_providers
            .get("custom")
            .expect("custom provider should be present");
        assert_eq!(custom.base_url.as_deref(), Some("http://localhost:4000/v1"));
        assert_eq!(custom.api_key.as_deref(), Some("sk-test-key"));
        assert_eq!(custom.api.as_deref(), Some("openai-completions"));

        assert_eq!(custom.models.len(), 2);

        let m1 = &custom.models[0];
        assert_eq!(m1.id, "my-model");
        assert_eq!(m1.name.as_deref(), Some("My Model"));
        assert!(!m1.reasoning);
        assert_eq!(m1.input, vec!["text"]);
        assert_eq!(m1.context_window, Some(128000));
        assert_eq!(m1.max_tokens, Some(32000));

        let m2 = &custom.models[1];
        assert_eq!(m2.id, "vision-model");
        assert_eq!(m2.name.as_deref(), Some("Vision Model"));
        assert!(m2.reasoning);
        assert_eq!(m2.input, vec!["text", "image"]);
        assert_eq!(m2.context_window, Some(200000));
        assert_eq!(m2.max_tokens, Some(4096));
    }

    #[test]
    fn test_roundtrip_provider_without_optional_fields() {
        // Provider with only required fields — no baseUrl, no apiKey, no api, no models
        let mut profile = empty_profile();
        let mut providers = HashMap::new();
        providers.insert(
            "minimal".to_string(),
            OpenClawProviderConfig {
                base_url: None,
                api_key: None,
                api: None,
                models: vec![],
            },
        );
        profile.providers = providers;

        let config = build_full_config(&profile);

        // Since no models in provider, no `agents.defaults.models` should be written.
        // And since no default_model is set, no `agents` block should appear either.
        let agents = config.get("agents");
        assert!(
            agents.is_none(),
            "no default_model and no model refs → no agents block"
        );

        // A provider entry with all-optional fields empty still gets written as "minimal": {}
        // because the provider id itself is a valid config key.
        let models = config.get("models").and_then(|v| v.as_object()).unwrap();
        let providers_obj = models.get("providers").and_then(|v| v.as_object()).unwrap();
        assert!(
            providers_obj.contains_key("minimal"),
            "provider id should appear as a key"
        );
        let minimal = providers_obj
            .get("minimal")
            .and_then(|v| v.as_object())
            .unwrap();
        assert!(
            minimal.is_empty(),
            "minimal provider object should be empty '{{}}'"
        );

        // Parse back: a provider with no baseUrl/apiKey/models should produce
        // a config entry, but the parsed provider will have all None/empty fields.
        let (_, _, parsed_providers) = roundtrip(&profile);
        let minimal_parsed = parsed_providers.get("minimal").unwrap();
        assert!(minimal_parsed.base_url.is_none());
        assert!(minimal_parsed.api_key.is_none());
        assert!(minimal_parsed.api.is_none());
        assert!(minimal_parsed.models.is_empty());
    }

    #[test]
    fn test_roundtrip_block_streaming_config() {
        let mut profile = empty_profile();
        profile.block_streaming_config = Some(BlockStreamingConfig {
            block_streaming_default: Some("on".to_string()),
            block_streaming_break: Some("message_end".to_string()),
            block_streaming_chunk: Some(BlockStreamingChunk {
                min_chars: Some(800),
                max_chars: Some(1200),
            }),
            block_streaming_coalesce: Some(BlockStreamingCoalesce {
                idle_ms: Some(1000),
            }),
            telegram_channel: Some(TelegramChannelConfig {
                block_streaming: Some(true),
                chunk_mode: Some("message".to_string()),
            }),
        });

        let config = build_full_config(&profile);

        // Verify structure before roundtrip
        let agents = config.get("agents").and_then(|v| v.as_object()).unwrap();
        let defaults = agents.get("defaults").and_then(|v| v.as_object()).unwrap();

        assert_eq!(
            defaults
                .get("blockStreamingDefault")
                .and_then(|v| v.as_str()),
            Some("on")
        );
        assert_eq!(
            defaults.get("blockStreamingBreak").and_then(|v| v.as_str()),
            Some("message_end")
        );

        let chunk = defaults
            .get("blockStreamingChunk")
            .and_then(|v| v.as_object())
            .unwrap();
        assert_eq!(chunk.get("minChars").and_then(|v| v.as_u64()), Some(800));
        assert_eq!(chunk.get("maxChars").and_then(|v| v.as_u64()), Some(1200));

        let coalesce = defaults
            .get("blockStreamingCoalesce")
            .and_then(|v| v.as_object())
            .unwrap();
        assert_eq!(coalesce.get("idleMs").and_then(|v| v.as_u64()), Some(1000));

        // Verify telegram channel config
        let channels = config.get("channels").and_then(|v| v.as_object()).unwrap();
        let telegram = channels
            .get("telegram")
            .and_then(|v| v.as_object())
            .unwrap();
        assert_eq!(
            telegram.get("blockStreaming").and_then(|v| v.as_bool()),
            Some(true)
        );
        assert_eq!(
            telegram.get("chunkMode").and_then(|v| v.as_str()),
            Some("message")
        );

        // Roundtrip: the parsed BlockStreamingConfig is not roundtripped through
        // parse_openclaw_config (it only extracts model+provider). We just verify
        // the config structure is correct above.
        let (_, _, _) = roundtrip(&profile);
        // No panic = success
    }

    #[test]
    fn test_roundtrip_full_profile() {
        // A profile exercising all fields at once
        let mut profile = empty_profile();
        profile.default_model = Some("openai/gpt-5.5".to_string());
        profile.failover_models = Some(vec!["anthropic/claude-opus-4-6".to_string()]);

        let mut providers = HashMap::new();
        providers.insert(
            "openai".to_string(),
            OpenClawProviderConfig {
                base_url: Some("https://api.openai.com/v1".to_string()),
                api_key: None,
                api: Some("openai-completions".to_string()),
                models: vec![OpenClawModel {
                    id: "gpt-5.5".to_string(),
                    name: Some("GPT 5.5".to_string()),
                    reasoning: false,
                    input: vec!["text".to_string()],
                    context_window: Some(1000000),
                    max_tokens: Some(32000),
                }],
            },
        );
        profile.providers = providers;

        let config = build_full_config(&profile);
        let config_str = serde_json::to_string_pretty(&config).unwrap();

        // Verify the JSON output contains expected keys
        assert!(config_str.contains("\"primary\""));
        assert!(config_str.contains("\"fallbacks\""));
        assert!(config_str.contains("\"openai\""));
        assert!(config_str.contains("\"gpt-5.5\""));
        assert!(config_str.contains("\"GPT 5.5\""));

        // Roundtrip parse
        let (model, failovers, parsed_providers) = parse_openclaw_config(&config);
        assert_eq!(model.as_deref(), Some("openai/gpt-5.5"));
        assert_eq!(
            failovers,
            Some(vec!["anthropic/claude-opus-4-6".to_string()])
        );

        let openai = parsed_providers.get("openai").unwrap();
        assert_eq!(
            openai.base_url.as_deref(),
            Some("https://api.openai.com/v1")
        );
        assert!(openai.api_key.is_none());
        assert_eq!(openai.models.len(), 1);
        assert_eq!(openai.models[0].id, "gpt-5.5");
        assert_eq!(openai.models[0].context_window, Some(1000000));
    }

    // ------------------------------------------------------------------
    // Parse documented config format
    // ------------------------------------------------------------------

    #[test]
    fn test_parse_documented_agents_defaults_model() {
        // Exact documented format from https://docs.openclaw.ai/gateway/config-agents
        let json_str = r#"
        {
            "agents": {
                "defaults": {
                    "model": {
                        "primary": "anthropic/claude-opus-4-6",
                        "fallbacks": ["minimax/MiniMax-M2.7"]
                    }
                }
            }
        }
        "#;

        let config: Value = serde_json::from_str(json_str).unwrap();
        let (model, failovers, providers) = parse_openclaw_config(&config);

        assert_eq!(model.as_deref(), Some("anthropic/claude-opus-4-6"));
        assert_eq!(failovers, Some(vec!["minimax/MiniMax-M2.7".to_string()]));
        assert!(providers.is_empty());
    }

    #[test]
    fn test_parse_documented_model_as_string() {
        // Docs say model can also be a plain string
        let json_str = r#"
        {
            "agents": {
                "defaults": {
                    "model": "anthropic/claude-sonnet-4-6"
                }
            }
        }
        "#;

        let config: Value = serde_json::from_str(json_str).unwrap();
        let (model, failovers, providers) = parse_openclaw_config(&config);

        // The parser reads from `model.primary`, so a plain string won't match
        assert!(model.is_none(), "plain string model is not parsed (only object form with 'primary'/'fallbacks' keys is supported)");
        assert!(failovers.is_none());
        assert!(providers.is_empty());
    }

    #[test]
    fn test_parse_documented_custom_providers() {
        // Exact documented format from https://docs.openclaw.ai/gateway/config-tools
        let json_str = r#"
        {
            "models": {
                "mode": "merge",
                "providers": {
                    "custom-proxy": {
                        "baseUrl": "http://localhost:4000/v1",
                        "apiKey": "LITELLM_KEY",
                        "api": "openai-completions",
                        "models": [
                            {
                                "id": "llama-3.1-8b",
                                "name": "Llama 3.1 8B",
                                "reasoning": false,
                                "input": ["text"],
                                "contextWindow": 128000,
                                "maxTokens": 32000
                            }
                        ]
                    }
                }
            }
        }
        "#;

        let config: Value = serde_json::from_str(json_str).unwrap();
        let (_model, _failovers, providers) = parse_openclaw_config(&config);

        let proxy = providers
            .get("custom-proxy")
            .expect("custom-proxy provider should be parsed");
        assert_eq!(proxy.base_url.as_deref(), Some("http://localhost:4000/v1"));
        assert_eq!(proxy.api_key.as_deref(), Some("LITELLM_KEY"));
        assert_eq!(proxy.api.as_deref(), Some("openai-completions"));
        assert_eq!(proxy.models.len(), 1);

        let m = &proxy.models[0];
        assert_eq!(m.id, "llama-3.1-8b");
        assert_eq!(m.name.as_deref(), Some("Llama 3.1 8B"));
        assert!(!m.reasoning);
        assert_eq!(m.input, vec!["text"]);
        assert_eq!(m.context_window, Some(128000));
        assert_eq!(m.max_tokens, Some(32000));
    }

    #[test]
    fn test_parse_documented_provider_with_runtime_cap() {
        // Parse config that has both contextWindow and contextTokens (runtime cap)
        let json_str = r#"
        {
            "models": {
                "providers": {
                    "minimax": {
                        "baseUrl": "https://api.minimax.io/anthropic",
                        "apiKey": "${MINIMAX_API_KEY}",
                        "api": "anthropic-messages",
                        "models": [
                            {
                                "id": "MiniMax-M2.7",
                                "name": "MiniMax M2.7",
                                "reasoning": true,
                                "input": ["text"],
                                "contextWindow": 204800,
                                "maxTokens": 131072
                            }
                        ]
                    }
                }
            }
        }
        "#;

        let config: Value = serde_json::from_str(json_str).unwrap();
        let (_model, _failovers, providers) = parse_openclaw_config(&config);

        let minimax = providers.get("minimax").unwrap();
        assert_eq!(
            minimax.base_url.as_deref(),
            Some("https://api.minimax.io/anthropic")
        );
        assert_eq!(minimax.api.as_deref(), Some("anthropic-messages"));
        assert_eq!(minimax.models.len(), 1);

        let m = &minimax.models[0];
        assert_eq!(m.id, "MiniMax-M2.7");
        assert!(m.reasoning);
        assert_eq!(m.input, vec!["text"]);
        assert_eq!(m.context_window, Some(204800));
        assert_eq!(m.max_tokens, Some(131072));
    }

    #[test]
    fn test_parse_provider_with_vision_model() {
        // Vision model with input: ["text", "image"]
        let json_str = r#"
        {
            "models": {
                "providers": {
                    "moonshot": {
                        "baseUrl": "https://api.moonshot.ai/v1",
                        "api": "openai-completions",
                        "models": [
                            {
                                "id": "kimi-k2.6",
                                "name": "Kimi K2.6",
                                "reasoning": false,
                                "input": ["text", "image"],
                                "contextWindow": 262144,
                                "maxTokens": 262144
                            }
                        ]
                    }
                }
            }
        }
        "#;

        let config: Value = serde_json::from_str(json_str).unwrap();
        let (_model, _failovers, providers) = parse_openclaw_config(&config);

        let moonshot = providers.get("moonshot").unwrap();
        assert_eq!(moonshot.models.len(), 1);
        assert_eq!(moonshot.models[0].input, vec!["text", "image"]);
        assert_eq!(moonshot.models[0].context_window, Some(262144));
    }

    #[test]
    fn test_parse_provider_without_models_array() {
        // Provider with no models array at all
        let json_str = r#"
        {
            "models": {
                "providers": {
                    "empty": {
                        "baseUrl": "http://localhost:9999/v1"
                    }
                }
            }
        }
        "#;

        let config: Value = serde_json::from_str(json_str).unwrap();
        let (_model, _failovers, providers) = parse_openclaw_config(&config);

        let empty = providers.get("empty").unwrap();
        assert_eq!(empty.base_url.as_deref(), Some("http://localhost:9999/v1"));
        assert!(empty.api_key.is_none());
        assert!(empty.api.is_none());
        assert!(empty.models.is_empty(), "no models array → empty vec");
    }

    #[test]
    fn test_parse_provider_without_any_optional_fields() {
        // Provider with just an id and nothing else
        let json_str = r#"
        {
            "models": {
                "providers": {
                    "bare": {}
                }
            }
        }
        "#;

        let config: Value = serde_json::from_str(json_str).unwrap();
        let (_model, _failovers, providers) = parse_openclaw_config(&config);

        let bare = providers.get("bare").unwrap();
        assert!(bare.base_url.is_none());
        assert!(bare.api_key.is_none());
        assert!(bare.api.is_none());
        assert!(bare.models.is_empty());
    }

    #[test]
    fn test_parse_model_without_fallbacks() {
        // agents.defaults.model with only primary
        let json_str = r#"
        {
            "agents": {
                "defaults": {
                    "model": {
                        "primary": "openai/gpt-5.4-mini"
                    }
                }
            }
        }
        "#;

        let config: Value = serde_json::from_str(json_str).unwrap();
        let (model, failovers, _providers) = parse_openclaw_config(&config);

        assert_eq!(model.as_deref(), Some("openai/gpt-5.4-mini"));
        assert!(
            failovers.is_none(),
            "no fallbacks key → None, not empty vec"
        );
    }

    #[test]
    fn test_parse_model_with_empty_fallbacks() {
        // agents.defaults.model with empty fallbacks array
        let json_str = r#"
        {
            "agents": {
                "defaults": {
                    "model": {
                        "primary": "openai/gpt-5.4-mini",
                        "fallbacks": []
                    }
                }
            }
        }
        "#;

        let config: Value = serde_json::from_str(json_str).unwrap();
        let (model, failovers, _providers) = parse_openclaw_config(&config);

        assert_eq!(model.as_deref(), Some("openai/gpt-5.4-mini"));
        assert!(failovers.is_none(), "empty fallbacks array → None");
    }

    #[test]
    fn test_parse_model_with_model_field_only() {
        // agents.defaults with model as plain object but missing primary/fallbacks
        let json_str = r#"
        {
            "agents": {
                "defaults": {
                    "model": {
                        "someUnknownField": "test"
                    }
                }
            }
        }
        "#;

        let config: Value = serde_json::from_str(json_str).unwrap();
        let (model, failovers, _providers) = parse_openclaw_config(&config);

        assert!(
            model.is_none(),
            "model object without 'primary' should not be parsed"
        );
        assert!(failovers.is_none());
    }

    #[test]
    fn test_parse_agents_without_defaults() {
        // agents exists but no defaults
        let json_str = r#"
        {
            "agents": {
                "list": [
                    { "id": "main" }
                ]
            }
        }
        "#;

        let config: Value = serde_json::from_str(json_str).unwrap();
        let (model, failovers, providers) = parse_openclaw_config(&config);

        assert!(model.is_none());
        assert!(failovers.is_none());
        assert!(providers.is_empty());
    }

    #[test]
    fn test_parse_empty_config() {
        // Empty object
        let config: Value = serde_json::from_str("{}").unwrap();
        let (model, failovers, providers) = parse_openclaw_config(&config);

        assert!(model.is_none());
        assert!(failovers.is_none());
        assert!(providers.is_empty());
    }

    #[test]
    fn test_parse_top_level_keys_ignored() {
        // Config with unrelated top-level keys should be ignored
        let json_str = r#"
        {
            "gateway": { "port": 18789 },
            "logging": { "level": "info" },
            "agents": {
                "defaults": {
                    "model": {
                        "primary": "anthropic/claude-opus-4-6"
                    }
                }
            }
        }
        "#;

        let config: Value = serde_json::from_str(json_str).unwrap();
        let (model, failovers, providers) = parse_openclaw_config(&config);

        assert_eq!(model.as_deref(), Some("anthropic/claude-opus-4-6"));
        assert!(failovers.is_none());
        assert!(providers.is_empty());
    }

    // ------------------------------------------------------------------
    // Edge cases for provider model optional fields
    // ------------------------------------------------------------------

    #[test]
    fn test_parse_model_minimal_fields() {
        // Model with only id — all other fields should be defaults
        let json_str = r#"
        {
            "models": {
                "providers": {
                    "test": {
                        "models": [
                            { "id": "minimal-model" }
                        ]
                    }
                }
            }
        }
        "#;

        let config: Value = serde_json::from_str(json_str).unwrap();
        let (_model, _failovers, providers) = parse_openclaw_config(&config);

        let test = providers.get("test").unwrap();
        assert_eq!(test.models.len(), 1);

        let m = &test.models[0];
        assert_eq!(m.id, "minimal-model");
        assert!(m.name.is_none(), "name defaults to None");
        assert!(!m.reasoning, "reasoning defaults to false");
        assert!(m.input.is_empty(), "input defaults to empty vec");
        assert!(m.context_window.is_none(), "contextWindow defaults to None");
        assert!(m.max_tokens.is_none(), "maxTokens defaults to None");
    }

    #[test]
    fn test_parse_empty_providers_block() {
        // models.providers exists but is empty
        let json_str = r#"
        {
            "models": {
                "providers": {}
            }
        }
        "#;

        let config: Value = serde_json::from_str(json_str).unwrap();
        let (_model, _failovers, providers) = parse_openclaw_config(&config);

        assert!(providers.is_empty());
    }

    #[test]
    fn test_parse_models_without_providers() {
        // models block exists but no providers sub-key
        let json_str = r#"
        {
            "models": {
                "mode": "merge"
            }
        }
        "#;

        let config: Value = serde_json::from_str(json_str).unwrap();
        let (_model, _failovers, providers) = parse_openclaw_config(&config);

        assert!(providers.is_empty());
    }

    // ------------------------------------------------------------------
    // Build → JSON structure verification (camelCase field names)
    // ------------------------------------------------------------------

    #[test]
    fn test_build_json_fields_use_camelcase() {
        let mut profile = empty_profile();
        profile.default_model = Some("test/model".to_string());

        let mut providers = HashMap::new();
        providers.insert(
            "test".to_string(),
            OpenClawProviderConfig {
                base_url: Some("http://localhost/v1".to_string()),
                api_key: Some("key".to_string()),
                api: Some("openai-completions".to_string()),
                models: vec![OpenClawModel {
                    id: "test-model".to_string(),
                    name: Some("Test".to_string()),
                    reasoning: true,
                    input: vec!["text".to_string()],
                    context_window: Some(1000),
                    max_tokens: Some(500),
                }],
            },
        );
        profile.providers = providers;

        let config = build_full_config(&profile);
        let json_str = serde_json::to_string_pretty(&config).unwrap();

        // Verify camelCase field names in output
        assert!(
            json_str.contains("\"primary\""),
            "model key should be 'primary'"
        );
        assert!(
            json_str.contains("\"baseUrl\""),
            "provider field should be 'baseUrl'"
        );
        assert!(
            json_str.contains("\"apiKey\""),
            "provider field should be 'apiKey'"
        );
        assert!(
            json_str.contains("\"contextWindow\""),
            "model field should be 'contextWindow'"
        );
        assert!(
            json_str.contains("\"maxTokens\""),
            "model field should be 'maxTokens'"
        );

        // Verify snake_case is NOT in output for these fields
        assert!(
            !json_str.contains("\"base_url\""),
            "should not use snake_case 'base_url'"
        );
        assert!(
            !json_str.contains("\"api_key\""),
            "should not use snake_case 'api_key'"
        );
        assert!(
            !json_str.contains("\"context_window\""),
            "should not use snake_case 'context_window'"
        );
        assert!(
            !json_str.contains("\"max_tokens\""),
            "should not use snake_case 'max_tokens'"
        );
    }

    // ------------------------------------------------------------------
    // create_default_openclaw_profile_for_home parse correctness
    // ------------------------------------------------------------------

    #[test]
    fn test_default_profile_writes_correct_format() {
        // Create a temp directory for the config
        let temp = tempfile::TempDir::new().unwrap();
        let home = temp.path().join("home");
        std::fs::create_dir_all(&home).unwrap();

        // Create default profile (no existing config → fresh defaults)
        let profile = create_default_openclaw_profile_for_home(&home).unwrap();

        assert_eq!(
            profile.default_model.as_deref(),
            Some("anthropic/claude-sonnet-4-20250514")
        );
        assert!(profile.failover_models.is_none());
        assert!(profile.providers.is_empty());

        // Apply the profile to generate openclaw.json
        apply_openclaw_profile_for_home(&home, &profile).unwrap();

        // Read back the config file
        let config = read_openclaw_current_config_for_home(&home).unwrap();
        assert_eq!(
            config.default_model.as_deref(),
            Some("anthropic/claude-sonnet-4-20250514")
        );
        assert!(config.providers.is_empty());

        // Read the raw file and verify JSON structure matches docs format
        let config_path = home.join(".openclaw").join("openclaw.json");
        let raw = std::fs::read_to_string(&config_path).unwrap();
        let parsed: Value = serde_json::from_str(&raw).unwrap();

        // Verify the structure matches documented format
        let agents = parsed.get("agents").and_then(|v| v.as_object()).unwrap();
        let defaults = agents.get("defaults").and_then(|v| v.as_object()).unwrap();
        let model = defaults.get("model").and_then(|v| v.as_object()).unwrap();
        assert_eq!(
            model.get("primary").and_then(|v| v.as_str()),
            Some("anthropic/claude-sonnet-4-20250514")
        );
        // No fallbacks key since there are none
        assert!(model.get("fallbacks").is_none());
    }

    #[test]
    fn test_default_profile_parses_existing_config() {
        // Create a temp directory and write an existing openclaw.json
        let temp = tempfile::TempDir::new().unwrap();
        let home = temp.path().join("home");
        std::fs::create_dir_all(home.join(".openclaw")).unwrap();

        let existing_config = r#"
        {
            "agents": {
                "defaults": {
                    "model": {
                        "primary": "openai/gpt-5.5",
                        "fallbacks": ["anthropic/claude-opus-4-6"]
                    },
                    "models": {
                        "openai/gpt-5.5": { "alias": "gpt" },
                        "anthropic/claude-opus-4-6": { "alias": "opus" }
                    }
                }
            },
            "models": {
                "mode": "merge",
                "providers": {
                    "openai": {
                        "baseUrl": "https://api.openai.com/v1",
                        "api": "openai-completions",
                        "models": [
                            {
                                "id": "gpt-5.5",
                                "name": "GPT 5.5",
                                "input": ["text"],
                                "contextWindow": 1000000,
                                "maxTokens": 32000
                            }
                        ]
                    },
                    "anthropic": {
                        "baseUrl": "https://api.anthropic.com/v1",
                        "api": "anthropic-messages",
                        "models": [
                            {
                                "id": "claude-opus-4-6",
                                "name": "Claude Opus 4",
                                "reasoning": true,
                                "input": ["text", "image"],
                                "contextWindow": 200000,
                                "maxTokens": 4096
                            }
                        ]
                    }
                }
            }
        }
        "#;

        std::fs::write(
            home.join(".openclaw").join("openclaw.json"),
            existing_config,
        )
        .unwrap();

        // Create default profile — should parse existing config
        let profile = create_default_openclaw_profile_for_home(&home).unwrap();

        // Verify it read the existing values
        assert_eq!(profile.default_model.as_deref(), Some("openai/gpt-5.5"));
        assert_eq!(
            profile.failover_models,
            Some(vec!["anthropic/claude-opus-4-6".to_string()])
        );

        // Verify providers were parsed
        let openai = profile.providers.get("openai").unwrap();
        assert_eq!(
            openai.base_url.as_deref(),
            Some("https://api.openai.com/v1")
        );
        assert_eq!(openai.models.len(), 1);
        assert_eq!(openai.models[0].id, "gpt-5.5");
        assert_eq!(openai.models[0].context_window, Some(1000000));

        let anthropic = profile.providers.get("anthropic").unwrap();
        assert_eq!(
            anthropic.base_url.as_deref(),
            Some("https://api.anthropic.com/v1")
        );
        assert_eq!(anthropic.models.len(), 1);
        assert_eq!(anthropic.models[0].id, "claude-opus-4-6");
        assert!(anthropic.models[0].reasoning);
        assert_eq!(anthropic.models[0].input, vec!["text", "image"]);
    }

    // ------------------------------------------------------------------
    // Load + save preservation tests
    //
    // OpenClaw's config supports many provider/model fields DroidGear does
    // not model in `OpenClawProviderConfig`/`OpenClawModel` (timeoutSeconds,
    // headers, params, contextTokens, alias, compat, ...). A load → save
    // roundtrip must preserve those untouched fields, otherwise saving a
    // profile silently corrupts the user's openclaw.json.
    // ------------------------------------------------------------------

    fn run_load_save_roundtrip(existing_config: &str) -> Value {
        let temp = tempfile::TempDir::new().unwrap();
        let home = temp.path().join("home");
        std::fs::create_dir_all(home.join(".openclaw")).unwrap();
        std::fs::write(
            home.join(".openclaw").join("openclaw.json"),
            existing_config,
        )
        .unwrap();

        let profile = create_default_openclaw_profile_for_home(&home).unwrap();
        apply_openclaw_profile_for_home(&home, &profile).unwrap();

        let s = std::fs::read_to_string(home.join(".openclaw").join("openclaw.json")).unwrap();
        serde_json::from_str(&s).unwrap()
    }

    #[test]
    fn test_load_save_preserves_unrelated_top_level_keys() {
        // Top-level keys DroidGear does not own must survive a roundtrip
        // (gateway, channels, plugins, bindings, wizard, tools, session, meta).
        let existing = r#"{
            "agents": {
                "defaults": {
                    "model": {"primary": "wududu/m1"},
                    "workspace": "/home/u/.openclaw/workspace"
                }
            },
            "models": {
                "providers": {
                    "wududu": {
                        "baseUrl": "https://w",
                        "apiKey": "k",
                        "api": "openai-completions",
                        "models": [{"id": "m1", "name": "M1", "reasoning": true, "input": ["text"], "contextWindow": 100000, "maxTokens": 4096}]
                    }
                }
            },
            "gateway": {"port": 18789, "mode": "local"},
            "channels": {"feishu": {"enabled": true}},
            "bindings": [{"agentId": "main"}],
            "plugins": {"entries": {"feishu": {"enabled": true}}},
            "wizard": {"lastRunCommand": "onboard"},
            "tools": {"profile": "coding"},
            "session": {"dmScope": "per-channel-peer"},
            "meta": {"lastTouchedVersion": "2026.5.18"}
        }"#;

        let v = run_load_save_roundtrip(existing);

        for key in [
            "gateway", "channels", "bindings", "plugins", "wizard", "tools", "session", "meta",
        ] {
            assert!(v.get(key).is_some(), "top-level key '{key}' lost");
        }
        assert_eq!(
            v.pointer("/agents/defaults/workspace")
                .and_then(|x| x.as_str()),
            Some("/home/u/.openclaw/workspace"),
            "agents.defaults.workspace lost"
        );
    }

    #[test]
    fn test_load_save_preserves_extra_provider_fields() {
        // Provider-level fields OpenClaw supports but DroidGear doesn't
        // model: auth, timeoutSeconds, headers, params, etc.
        let existing = r#"{
            "models": {
                "providers": {
                    "wududu": {
                        "api": "openai-completions",
                        "apiKey": "secret",
                        "baseUrl": "https://w.example",
                        "auth": "api-key",
                        "timeoutSeconds": 600,
                        "headers": {"X-Custom": "value"},
                        "params": {"temperature": 0.7},
                        "injectNumCtxForOpenAICompat": false,
                        "models": [
                            {"id": "m1", "name": "M1", "reasoning": false, "input": ["text"], "contextWindow": 100000, "maxTokens": 4096}
                        ]
                    }
                }
            }
        }"#;

        let v = run_load_save_roundtrip(existing);

        let prov = v.pointer("/models/providers/wududu").unwrap();
        assert_eq!(prov.get("auth").and_then(|x| x.as_str()), Some("api-key"));
        assert_eq!(
            prov.get("timeoutSeconds").and_then(|x| x.as_u64()),
            Some(600)
        );
        assert_eq!(
            prov.pointer("/headers/X-Custom").and_then(|x| x.as_str()),
            Some("value")
        );
        assert_eq!(
            prov.pointer("/params/temperature").and_then(|x| x.as_f64()),
            Some(0.7)
        );
        assert_eq!(
            prov.get("injectNumCtxForOpenAICompat")
                .and_then(|x| x.as_bool()),
            Some(false)
        );
    }

    #[test]
    fn test_load_save_preserves_extra_model_fields() {
        // Model-level fields OpenClaw supports but DroidGear doesn't model:
        // contextTokens, params, headers, compat, ...
        let existing = r#"{
            "models": {
                "providers": {
                    "wududu": {
                        "baseUrl": "https://w",
                        "apiKey": "k",
                        "api": "openai-completions",
                        "models": [
                            {
                                "id": "m1",
                                "name": "M1",
                                "reasoning": true,
                                "input": ["text"],
                                "contextWindow": 200000,
                                "contextTokens": 150000,
                                "maxTokens": 8192,
                                "params": {"thinking_budget": 8000},
                                "headers": {"X-Trace": "1"},
                                "compat": {"supportsTools": true}
                            }
                        ]
                    }
                }
            }
        }"#;

        let v = run_load_save_roundtrip(existing);

        let model0 = v.pointer("/models/providers/wududu/models/0").unwrap();
        assert_eq!(
            model0.get("contextTokens").and_then(|x| x.as_u64()),
            Some(150000),
            "model.contextTokens lost"
        );
        assert_eq!(
            model0
                .pointer("/params/thinking_budget")
                .and_then(|x| x.as_u64()),
            Some(8000),
            "model.params lost"
        );
        assert_eq!(
            model0.pointer("/headers/X-Trace").and_then(|x| x.as_str()),
            Some("1"),
            "model.headers lost"
        );
        assert_eq!(
            model0
                .pointer("/compat/supportsTools")
                .and_then(|x| x.as_bool()),
            Some(true),
            "model.compat lost"
        );
    }

    #[test]
    fn test_load_save_preserves_agents_defaults_models_alias() {
        // agents.defaults.models[ref] entries can carry alias/params/streaming.
        let existing = r#"{
            "agents": {
                "defaults": {
                    "model": {"primary": "wududu/m1"},
                    "models": {
                        "wududu/m1": {
                            "alias": "fast",
                            "params": {"thinking": "low"},
                            "streaming": true
                        }
                    }
                }
            },
            "models": {
                "providers": {
                    "wududu": {
                        "baseUrl": "https://w",
                        "apiKey": "k",
                        "api": "openai-completions",
                        "models": [{"id": "m1", "name": "M1", "reasoning": true, "input": ["text"], "contextWindow": 100000, "maxTokens": 4096}]
                    }
                }
            }
        }"#;

        let v = run_load_save_roundtrip(existing);

        let entry = v.pointer("/agents/defaults/models/wududu~1m1").unwrap();
        assert_eq!(entry.get("alias").and_then(|x| x.as_str()), Some("fast"));
        assert_eq!(
            entry.pointer("/params/thinking").and_then(|x| x.as_str()),
            Some("low")
        );
        assert_eq!(entry.get("streaming").and_then(|x| x.as_bool()), Some(true));
    }

    #[test]
    fn test_load_save_preserves_unowned_models_defaults_entries() {
        // agents.defaults.models entries whose provider prefix is NOT owned
        // by the profile must be left alone (e.g. a built-in 'openai/' ref a
        // user added by hand alongside the custom 'wududu' provider).
        let existing = r#"{
            "agents": {
                "defaults": {
                    "model": {"primary": "wududu/m1"},
                    "models": {
                        "wududu/m1": {"alias": "fast"},
                        "openai/gpt-5.5": {"alias": "smart"}
                    }
                }
            },
            "models": {
                "providers": {
                    "wududu": {
                        "baseUrl": "https://w",
                        "apiKey": "k",
                        "api": "openai-completions",
                        "models": [{"id": "m1", "name": "M1", "reasoning": true, "input": ["text"], "contextWindow": 100000, "maxTokens": 4096}]
                    }
                }
            }
        }"#;

        let v = run_load_save_roundtrip(existing);

        let entry = v
            .pointer("/agents/defaults/models/openai~1gpt-5.5")
            .unwrap();
        assert_eq!(
            entry.get("alias").and_then(|x| x.as_str()),
            Some("smart"),
            "non-owned models.defaults entry should be preserved"
        );
    }

    #[test]
    fn test_load_save_removes_models_for_models_dropped_from_profile() {
        // If profile no longer references a model id, the typed `models[]`
        // entry under that provider should be removed from the file.
        let existing = r#"{
            "agents": {
                "defaults": {
                    "model": {"primary": "wududu/keep"}
                }
            },
            "models": {
                "providers": {
                    "wududu": {
                        "baseUrl": "https://w",
                        "apiKey": "k",
                        "api": "openai-completions",
                        "models": [
                            {"id": "keep", "name": "Keep", "reasoning": false, "input": ["text"], "contextWindow": 100000, "maxTokens": 4096}
                        ]
                    }
                }
            }
        }"#;

        let temp = tempfile::TempDir::new().unwrap();
        let home = temp.path().join("home");
        std::fs::create_dir_all(home.join(".openclaw")).unwrap();
        std::fs::write(home.join(".openclaw").join("openclaw.json"), existing).unwrap();

        // Load profile, then drop the only model from the only provider.
        let mut profile = create_default_openclaw_profile_for_home(&home).unwrap();
        if let Some(p) = profile.providers.get_mut("wududu") {
            p.models.clear();
        }
        apply_openclaw_profile_for_home(&home, &profile).unwrap();

        let s = std::fs::read_to_string(home.join(".openclaw").join("openclaw.json")).unwrap();
        let v: Value = serde_json::from_str(&s).unwrap();

        // Provider still present (user didn't remove it), but with no models[].
        assert!(v.pointer("/models/providers/wududu").is_some());
        assert!(
            v.pointer("/models/providers/wududu/models").is_none()
                || v.pointer("/models/providers/wududu/models")
                    .and_then(|x| x.as_array())
                    .is_some_and(|a| a.is_empty()),
            "models[] should be empty/absent"
        );
    }

    #[test]
    fn test_load_save_preserves_unknown_keys_inside_agents_defaults_model() {
        // OpenClaw may add new sub-keys to `agents.defaults.model`
        // (e.g. routing strategies, weights). DroidGear only owns `primary`
        // and `fallbacks`; everything else here must survive.
        let existing = r#"{
            "agents": {
                "defaults": {
                    "model": {
                        "primary": "wududu/m1",
                        "fallbacks": ["wududu/m1"],
                        "strategy": "weighted",
                        "weights": {"wududu/m1": 1.0},
                        "futureKey": {"x": 42}
                    }
                }
            },
            "models": {
                "providers": {
                    "wududu": {
                        "baseUrl": "https://w",
                        "apiKey": "k",
                        "api": "openai-completions",
                        "models": [{"id": "m1", "name": "M1", "reasoning": false, "input": ["text"], "contextWindow": 100000, "maxTokens": 4096}]
                    }
                }
            }
        }"#;

        let v = run_load_save_roundtrip(existing);
        let model = v.pointer("/agents/defaults/model").unwrap();
        assert_eq!(
            model.get("strategy").and_then(|x| x.as_str()),
            Some("weighted"),
            "unknown agents.defaults.model.strategy lost"
        );
        assert_eq!(
            model
                .pointer("/weights/wududu~1m1")
                .and_then(|x| x.as_f64()),
            Some(1.0)
        );
        assert_eq!(
            model.pointer("/futureKey/x").and_then(|x| x.as_u64()),
            Some(42)
        );
    }

    #[test]
    fn test_load_save_preserves_unknown_keys_inside_block_streaming_chunk() {
        // Owned: minChars/maxChars. Unknown sibling sub-keys must survive.
        let existing = r#"{
            "agents": {
                "defaults": {
                    "blockStreamingChunk": {
                        "minChars": 200,
                        "maxChars": 600,
                        "smoothingMode": "adaptive",
                        "customTuning": {"weight": 0.5}
                    }
                }
            }
        }"#;

        let v = run_load_save_roundtrip(existing);
        let chunk = v.pointer("/agents/defaults/blockStreamingChunk").unwrap();
        assert_eq!(
            chunk.get("smoothingMode").and_then(|x| x.as_str()),
            Some("adaptive")
        );
        assert_eq!(
            chunk
                .pointer("/customTuning/weight")
                .and_then(|x| x.as_f64()),
            Some(0.5)
        );
        assert_eq!(chunk.get("minChars").and_then(|x| x.as_u64()), Some(200));
        assert_eq!(chunk.get("maxChars").and_then(|x| x.as_u64()), Some(600));
    }

    #[test]
    fn test_load_save_preserves_unknown_keys_inside_channels_telegram() {
        // Owned: blockStreaming, chunkMode. Anything else (botToken, chatIds,
        // proxy, etc.) must survive. Sibling channels (`channels.feishu`)
        // must also survive.
        let existing = r#"{
            "channels": {
                "telegram": {
                    "blockStreaming": true,
                    "chunkMode": "message",
                    "botToken": "xxx:yyy",
                    "chatIds": ["-1001", "-1002"],
                    "proxy": {"url": "socks5://localhost:1080"}
                },
                "feishu": {"webhookUrl": "https://example/hook"}
            }
        }"#;

        let v = run_load_save_roundtrip(existing);
        let tg = v.pointer("/channels/telegram").unwrap();
        assert_eq!(
            tg.get("botToken").and_then(|x| x.as_str()),
            Some("xxx:yyy"),
            "telegram.botToken lost"
        );
        assert_eq!(
            tg.pointer("/chatIds/0").and_then(|x| x.as_str()),
            Some("-1001")
        );
        assert_eq!(
            tg.pointer("/proxy/url").and_then(|x| x.as_str()),
            Some("socks5://localhost:1080")
        );
        assert_eq!(
            v.pointer("/channels/feishu/webhookUrl")
                .and_then(|x| x.as_str()),
            Some("https://example/hook"),
            "sibling channel `feishu` lost"
        );
    }

    #[test]
    fn test_load_save_preserves_top_level_key_order() {
        // With serde_json's preserve_order feature, top-level key order
        // should be stable across a load → save roundtrip.
        let existing = r#"{
            "meta": {"v": 1},
            "agents": {
                "defaults": {
                    "model": {"primary": "wududu/m1"}
                }
            },
            "gateway": {"port": 18789},
            "models": {
                "providers": {
                    "wududu": {
                        "baseUrl": "https://w",
                        "apiKey": "k",
                        "api": "openai-completions",
                        "models": [{"id": "m1", "name": "M1", "reasoning": false, "input": ["text"], "contextWindow": 100000, "maxTokens": 4096}]
                    }
                }
            },
            "plugins": {"entries": {}}
        }"#;

        let temp = tempfile::TempDir::new().unwrap();
        let home = temp.path().join("home");
        std::fs::create_dir_all(home.join(".openclaw")).unwrap();
        std::fs::write(home.join(".openclaw").join("openclaw.json"), existing).unwrap();

        let profile = create_default_openclaw_profile_for_home(&home).unwrap();
        apply_openclaw_profile_for_home(&home, &profile).unwrap();

        let after_text =
            std::fs::read_to_string(home.join(".openclaw").join("openclaw.json")).unwrap();
        let after: Value = serde_json::from_str(&after_text).unwrap();
        let after_obj = after.as_object().unwrap();
        let actual_order: Vec<&str> = after_obj.keys().map(String::as_str).collect();
        assert_eq!(
            actual_order,
            vec!["meta", "agents", "gateway", "models", "plugins"],
            "top-level key order changed: {actual_order:?}"
        );
    }

    #[test]
    fn test_load_save_preserves_provider_key_order_inside_object() {
        // Provider object key order should also survive (preserve_order +
        // surgical updates do not reinsert keys we don't own).
        let existing = r#"{
            "models": {
                "providers": {
                    "wududu": {
                        "timeoutSeconds": 600,
                        "baseUrl": "https://w",
                        "headers": {"X-A": "1"},
                        "apiKey": "k",
                        "api": "openai-completions",
                        "models": [{"id": "m1", "name": "M1", "reasoning": false, "input": ["text"], "contextWindow": 100000, "maxTokens": 4096}]
                    }
                }
            }
        }"#;

        let v = run_load_save_roundtrip(existing);
        let prov = v
            .pointer("/models/providers/wududu")
            .and_then(|x| x.as_object())
            .unwrap();
        let actual_order: Vec<&str> = prov.keys().map(String::as_str).collect();
        assert_eq!(
            actual_order,
            vec![
                "timeoutSeconds",
                "baseUrl",
                "headers",
                "apiKey",
                "api",
                "models"
            ],
            "provider key order changed: {actual_order:?}"
        );
    }

    #[test]
    fn test_load_save_preserves_model_array_order_and_position() {
        // models[] is keyed by id; order must match the profile order
        // (DroidGear-owned semantics). Existing entries' unknown fields
        // are preserved per-model.
        let existing = r#"{
            "models": {
                "providers": {
                    "wududu": {
                        "baseUrl": "https://w",
                        "apiKey": "k",
                        "api": "openai-completions",
                        "models": [
                            {"id": "a", "name": "A", "reasoning": false, "input": ["text"], "contextWindow": 100000, "maxTokens": 4096, "params": {"x": 1}},
                            {"id": "b", "name": "B", "reasoning": false, "input": ["text"], "contextWindow": 100000, "maxTokens": 4096}
                        ]
                    }
                }
            }
        }"#;

        let v = run_load_save_roundtrip(existing);
        let arr = v
            .pointer("/models/providers/wududu/models")
            .and_then(|x| x.as_array())
            .unwrap();
        assert_eq!(arr.len(), 2);
        assert_eq!(arr[0].get("id").and_then(|x| x.as_str()), Some("a"));
        assert_eq!(arr[1].get("id").and_then(|x| x.as_str()), Some("b"));
        // Unknown per-model field preserved.
        assert_eq!(
            arr[0].pointer("/params/x").and_then(|x| x.as_u64()),
            Some(1)
        );
    }

    #[test]
    fn test_load_save_does_not_inject_unowned_keys() {
        // A round-trip on a config that DroidGear has nothing to update
        // should not introduce keys (no `models.mode` materialization, no
        // empty `agents.defaults.models`, etc.).
        let existing = r#"{
            "meta": {"v": 1},
            "gateway": {"port": 18789}
        }"#;

        let v = run_load_save_roundtrip(existing);
        let obj = v.as_object().unwrap();
        assert!(obj.contains_key("meta"));
        assert!(obj.contains_key("gateway"));
        assert!(
            !obj.contains_key("models"),
            "empty profile should not materialize `models` block: {obj:?}"
        );
        assert!(
            !obj.contains_key("agents"),
            "empty profile should not materialize `agents` block: {obj:?}"
        );
    }
}
