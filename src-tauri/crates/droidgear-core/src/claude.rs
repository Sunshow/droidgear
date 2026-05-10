//! Claude Code profile management (core).
//!
//! Provides Claude Code profile CRUD and active profile persistence for
//! DroidGear-managed profile storage under `~/.droidgear/claude/`.

use chrono::Utc;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use specta::Type;
use std::path::{Path, PathBuf};
use uuid::Uuid;

use crate::{paths, storage};

// ============================================================================
// Types
// ============================================================================

#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize, Type, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum ClaudeReasoningEffort {
    #[default]
    Low,
    Medium,
    High,
    Max,
}

#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize, Type, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum ClaudeThinkingMode {
    #[default]
    Inherit,
    On,
    Off,
}

/// Claude Code profile stored in DroidGear.
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct ClaudeCodeProfile {
    pub id: String,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub base_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bearer_token: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,
    #[serde(default = "default_small_model_uses_main_model")]
    pub small_model_uses_main_model: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub small_model: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reasoning_effort: Option<ClaudeReasoningEffort>,
    #[serde(default)]
    pub thinking_mode: ClaudeThinkingMode,
    pub created_at: String,
    pub updated_at: String,
}

/// Claude Code live config status.
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct ClaudeConfigStatus {
    pub settings_exists: bool,
    pub settings_path: String,
    pub config_dir: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parse_error: Option<String>,
}

/// Current Claude Code configuration read back from live settings.
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct ClaudeCurrentConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub base_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bearer_token: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,
    #[serde(default = "default_small_model_uses_main_model")]
    pub small_model_uses_main_model: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub small_model: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reasoning_effort: Option<ClaudeReasoningEffort>,
    #[serde(default)]
    pub thinking_mode: ClaudeThinkingMode,
}

fn default_small_model_uses_main_model() -> bool {
    true
}

pub(crate) const CLAUDE_BASE_URL_ENV: &str = "ANTHROPIC_BASE_URL";
pub(crate) const CLAUDE_AUTH_TOKEN_ENV: &str = "ANTHROPIC_AUTH_TOKEN";
pub(crate) const CLAUDE_API_KEY_ENV: &str = "ANTHROPIC_API_KEY";
pub(crate) const CLAUDE_MODEL_ENV: &str = "ANTHROPIC_MODEL";
pub(crate) const CLAUDE_SMALL_MODEL_ENV: &str = "ANTHROPIC_DEFAULT_HAIKU_MODEL";
pub(crate) const CLAUDE_EFFORT_ENV: &str = "CLAUDE_CODE_EFFORT_LEVEL";
pub(crate) const CLAUDE_DISABLE_THINKING_ENV: &str = "CLAUDE_CODE_DISABLE_THINKING";
pub(crate) const CLAUDE_MAX_THINKING_TOKENS_ENV: &str = "MAX_THINKING_TOKENS";
pub(crate) const CLAUDE_DISABLE_ADAPTIVE_ENV: &str = "CLAUDE_CODE_DISABLE_ADAPTIVE_THINKING";

pub(crate) const CLAUDE_CONFLICT_ENV_KEYS: &[&str] = &[
    CLAUDE_API_KEY_ENV,
    "CLAUDE_CODE_USE_BEDROCK",
    "CLAUDE_CODE_USE_VERTEX",
    "CLAUDE_CODE_USE_FOUNDRY",
    "ANTHROPIC_BEDROCK_BASE_URL",
    "ANTHROPIC_BEDROCK_MANTLE_BASE_URL",
    "ANTHROPIC_VERTEX_BASE_URL",
    "ANTHROPIC_VERTEX_PROJECT_ID",
    "ANTHROPIC_FOUNDRY_BASE_URL",
    "ANTHROPIC_FOUNDRY_RESOURCE",
    "ANTHROPIC_FOUNDRY_API_KEY",
    "CLAUDE_CODE_PROVIDER_MANAGED_BY_HOST",
];

// ============================================================================
// Path Helpers
// ============================================================================

fn droidgear_claude_dir_for_home(home_dir: &Path) -> PathBuf {
    home_dir.join(".droidgear").join("claude")
}

/// `~/.droidgear/claude/profiles/`
pub fn profiles_dir_for_home(home_dir: &Path) -> Result<PathBuf, String> {
    let dir = droidgear_claude_dir_for_home(home_dir).join("profiles");
    if !dir.exists() {
        std::fs::create_dir_all(&dir)
            .map_err(|e| format!("Failed to create claude profiles directory: {e}"))?;
    }
    Ok(dir)
}

/// `~/.droidgear/claude/active-profile.txt`
pub fn active_profile_path_for_home(home_dir: &Path) -> Result<PathBuf, String> {
    let dir = droidgear_claude_dir_for_home(home_dir);
    if !dir.exists() {
        std::fs::create_dir_all(&dir)
            .map_err(|e| format!("Failed to create claude directory: {e}"))?;
    }
    Ok(dir.join("active-profile.txt"))
}

/// `~/.claude/` (or custom path)
pub fn claude_config_dir_for_home(home_dir: &Path) -> Result<PathBuf, String> {
    let config_paths = paths::load_config_paths_for_home(home_dir);
    let dir = paths::get_claude_home_for_home(home_dir, &config_paths)?;
    if !dir.exists() {
        std::fs::create_dir_all(&dir)
            .map_err(|e| format!("Failed to create claude config directory: {e}"))?;
    }
    Ok(dir)
}

/// `~/.claude/settings.json`
pub fn claude_settings_path_for_home(home_dir: &Path) -> Result<PathBuf, String> {
    Ok(claude_config_dir_for_home(home_dir)?.join("settings.json"))
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

pub fn profile_path_for_home(home_dir: &Path, id: &str) -> Result<PathBuf, String> {
    validate_profile_id(id)?;
    Ok(profiles_dir_for_home(home_dir)?.join(format!("{id}.json")))
}

// ============================================================================
// System wrappers (use system home dir)
// ============================================================================

fn system_home_dir() -> Result<PathBuf, String> {
    dirs::home_dir().ok_or_else(|| "Failed to get home directory".to_string())
}

pub fn profiles_dir() -> Result<PathBuf, String> {
    profiles_dir_for_home(&system_home_dir()?)
}

pub fn active_profile_path() -> Result<PathBuf, String> {
    active_profile_path_for_home(&system_home_dir()?)
}

pub fn claude_config_dir() -> Result<PathBuf, String> {
    claude_config_dir_for_home(&system_home_dir()?)
}

pub fn claude_settings_path() -> Result<PathBuf, String> {
    claude_settings_path_for_home(&system_home_dir()?)
}

pub fn profile_path(id: &str) -> Result<PathBuf, String> {
    profile_path_for_home(&system_home_dir()?, id)
}

// ============================================================================
// CRUD Helpers
// ============================================================================

fn now_rfc3339() -> String {
    Utc::now().to_rfc3339()
}

fn read_profile_file(path: &Path) -> Result<ClaudeCodeProfile, String> {
    let content =
        std::fs::read_to_string(path).map_err(|e| format!("Failed to read profile: {e}"))?;
    serde_json::from_str::<ClaudeCodeProfile>(&content)
        .map_err(|e| format!("Invalid profile JSON: {e}"))
}

fn write_profile_file(home_dir: &Path, profile: &ClaudeCodeProfile) -> Result<(), String> {
    let path = profile_path_for_home(home_dir, &profile.id)?;
    let content = serde_json::to_string_pretty(profile)
        .map_err(|e| format!("Failed to serialize profile JSON: {e}"))?;
    storage::atomic_write(&path, content.as_bytes())
}

fn load_profile_by_id(home_dir: &Path, id: &str) -> Result<ClaudeCodeProfile, String> {
    let path = profile_path_for_home(home_dir, id)?;
    read_profile_file(&path)
}

fn resolve_profile_by_name<'a>(
    profiles: &'a [ClaudeCodeProfile],
    selector: &str,
) -> Result<Option<&'a ClaudeCodeProfile>, String> {
    let exact_matches = profiles
        .iter()
        .filter(|profile| profile.name == selector)
        .collect::<Vec<_>>();
    match exact_matches.as_slice() {
        [] => {}
        [profile] => return Ok(Some(profile)),
        _ => {
            return Err(format!(
                "Multiple Claude profiles share the name '{selector}'. Use the profile index or id instead."
            ));
        }
    }

    let folded_selector = selector.to_lowercase();
    let folded_matches = profiles
        .iter()
        .filter(|profile| profile.name.to_lowercase() == folded_selector)
        .collect::<Vec<_>>();
    match folded_matches.as_slice() {
        [] => Ok(None),
        [profile] => Ok(Some(profile)),
        _ => Err(format!(
            "Multiple Claude profiles share the name '{selector}'. Use the profile index or id instead."
        )),
    }
}

pub(crate) fn normalize_optional_string(value: Option<&str>) -> Option<String> {
    value
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToOwned::to_owned)
}

fn normalize_model_id(value: &str) -> String {
    value.to_ascii_lowercase().replace(['-', '_'], ".")
}

pub fn is_recognized_claude_model_id(value: &str) -> bool {
    let Some(value) = normalize_optional_string(Some(value)) else {
        return false;
    };

    normalize_model_id(&value).starts_with("claude.")
}

pub fn has_opaque_claude_model_id(value: Option<&str>) -> bool {
    let Some(value) = normalize_optional_string(value) else {
        return false;
    };

    !is_recognized_claude_model_id(&value)
}

pub(crate) fn resolved_small_model_value(profile: &ClaudeCodeProfile) -> Option<String> {
    if profile.small_model_uses_main_model {
        normalize_optional_string(profile.model.as_deref())
    } else {
        normalize_optional_string(profile.small_model.as_deref())
    }
}

fn read_settings_object_from_path(path: &Path) -> Result<serde_json::Map<String, Value>, String> {
    if !path.exists() {
        return Ok(serde_json::Map::new());
    }

    let content =
        std::fs::read_to_string(path).map_err(|e| format!("Failed to read settings.json: {e}"))?;
    if content.trim().is_empty() {
        return Ok(serde_json::Map::new());
    }

    let value: Value = serde_json::from_str(&content)
        .map_err(|e| format!("Failed to parse settings.json: {e}"))?;
    match value {
        Value::Object(map) => Ok(map),
        _ => Err("settings.json root must be a JSON object".to_string()),
    }
}

fn get_env_object(
    root: &serde_json::Map<String, Value>,
) -> Result<Option<&serde_json::Map<String, Value>>, String> {
    match root.get("env") {
        Some(Value::Object(env)) => Ok(Some(env)),
        Some(_) => Err("settings.json env must be a JSON object".to_string()),
        None => Ok(None),
    }
}

fn remove_env_key(env: &mut serde_json::Map<String, Value>, key: &str) {
    env.remove(key);
}

fn set_env_string(env: &mut serde_json::Map<String, Value>, key: &str, value: Option<&str>) {
    if let Some(value) = normalize_optional_string(value) {
        env.insert(key.to_string(), Value::String(value));
    } else {
        remove_env_key(env, key);
    }
}

fn get_string_from_map(
    map: &serde_json::Map<String, Value>,
    key: &str,
) -> Result<Option<String>, String> {
    match map.get(key) {
        Some(Value::String(value)) => Ok(normalize_optional_string(Some(value))),
        Some(Value::Null) => Ok(None),
        Some(_) => Err(format!("settings.json {key} must be a string")),
        None => Ok(None),
    }
}

fn get_bool_from_map(
    map: &serde_json::Map<String, Value>,
    key: &str,
) -> Result<Option<bool>, String> {
    match map.get(key) {
        Some(Value::Bool(value)) => Ok(Some(*value)),
        Some(Value::Null) => Ok(None),
        Some(_) => Err(format!("settings.json {key} must be a boolean")),
        None => Ok(None),
    }
}

fn parse_reasoning_effort(value: &str) -> Option<ClaudeReasoningEffort> {
    match value.trim().to_ascii_lowercase().as_str() {
        "low" => Some(ClaudeReasoningEffort::Low),
        "medium" => Some(ClaudeReasoningEffort::Medium),
        "high" => Some(ClaudeReasoningEffort::High),
        "max" => Some(ClaudeReasoningEffort::Max),
        _ => None,
    }
}

pub(crate) fn reasoning_effort_to_string(effort: ClaudeReasoningEffort) -> &'static str {
    match effort {
        ClaudeReasoningEffort::Low => "low",
        ClaudeReasoningEffort::Medium => "medium",
        ClaudeReasoningEffort::High => "high",
        ClaudeReasoningEffort::Max => "max",
    }
}

fn env_value_is_truthy(value: Option<&Value>) -> bool {
    match value {
        Some(Value::Bool(value)) => *value,
        Some(Value::String(value)) => matches!(
            value.trim().to_ascii_lowercase().as_str(),
            "1" | "true" | "yes" | "on"
        ),
        Some(Value::Number(value)) => value.as_i64().is_some_and(|value| value != 0),
        _ => false,
    }
}

fn remove_conflicting_env_keys(env: &mut serde_json::Map<String, Value>) {
    for key in CLAUDE_CONFLICT_ENV_KEYS {
        remove_env_key(env, key);
    }
}

fn apply_reasoning_env(
    env: &mut serde_json::Map<String, Value>,
    reasoning_effort: Option<ClaudeReasoningEffort>,
) {
    match reasoning_effort {
        Some(effort @ (ClaudeReasoningEffort::Low | ClaudeReasoningEffort::Medium)) => {
            env.insert(
                CLAUDE_EFFORT_ENV.to_string(),
                Value::String(reasoning_effort_to_string(effort).to_string()),
            );
            remove_env_key(env, CLAUDE_DISABLE_ADAPTIVE_ENV);
        }
        Some(effort @ (ClaudeReasoningEffort::High | ClaudeReasoningEffort::Max)) => {
            env.insert(
                CLAUDE_EFFORT_ENV.to_string(),
                Value::String(reasoning_effort_to_string(effort).to_string()),
            );
            env.insert(
                CLAUDE_DISABLE_ADAPTIVE_ENV.to_string(),
                Value::String("1".to_string()),
            );
        }
        None => {
            remove_env_key(env, CLAUDE_EFFORT_ENV);
            remove_env_key(env, CLAUDE_DISABLE_ADAPTIVE_ENV);
        }
    }
}

fn apply_thinking_settings(
    root: &mut serde_json::Map<String, Value>,
    env: &mut serde_json::Map<String, Value>,
    thinking_mode: ClaudeThinkingMode,
) {
    match thinking_mode {
        ClaudeThinkingMode::Inherit => {
            root.remove("alwaysThinkingEnabled");
            remove_env_key(env, CLAUDE_DISABLE_THINKING_ENV);
        }
        ClaudeThinkingMode::On => {
            root.insert("alwaysThinkingEnabled".to_string(), Value::Bool(true));
            remove_env_key(env, CLAUDE_DISABLE_THINKING_ENV);
            remove_env_key(env, CLAUDE_MAX_THINKING_TOKENS_ENV);
        }
        ClaudeThinkingMode::Off => {
            root.insert("alwaysThinkingEnabled".to_string(), Value::Bool(false));
            env.insert(
                CLAUDE_DISABLE_THINKING_ENV.to_string(),
                Value::String("1".to_string()),
            );
            remove_env_key(env, CLAUDE_MAX_THINKING_TOKENS_ENV);
        }
    }
}

fn cleanup_env_if_empty(root: &mut serde_json::Map<String, Value>) {
    let remove_env = root
        .get("env")
        .and_then(Value::as_object)
        .is_some_and(serde_json::Map::is_empty);
    if remove_env {
        root.remove("env");
    }
}

fn build_current_config_from_settings(
    root: &serde_json::Map<String, Value>,
) -> Result<ClaudeCurrentConfig, String> {
    let env = get_env_object(root)?;

    let get_env_string = |key: &str| -> Result<Option<String>, String> {
        match env {
            Some(env) => get_string_from_map(env, key),
            None => Ok(None),
        }
    };

    let model = match get_env_string(CLAUDE_MODEL_ENV)? {
        Some(model) => Some(model),
        None => get_string_from_map(root, "model")?,
    };

    let reasoning_effort = match get_env_string(CLAUDE_EFFORT_ENV)? {
        Some(value) => parse_reasoning_effort(&value),
        None => get_string_from_map(root, "effortLevel")?
            .and_then(|value| parse_reasoning_effort(&value)),
    };

    let small_model = get_env_string(CLAUDE_SMALL_MODEL_ENV)?;

    let thinking_mode =
        if env_value_is_truthy(env.and_then(|env| env.get(CLAUDE_DISABLE_THINKING_ENV))) {
            ClaudeThinkingMode::Off
        } else {
            match get_bool_from_map(root, "alwaysThinkingEnabled")? {
                Some(true) => ClaudeThinkingMode::On,
                Some(false) => ClaudeThinkingMode::Off,
                None => ClaudeThinkingMode::Inherit,
            }
        };

    Ok(ClaudeCurrentConfig {
        base_url: get_env_string(CLAUDE_BASE_URL_ENV)?,
        bearer_token: get_env_string(CLAUDE_AUTH_TOKEN_ENV)?,
        model,
        small_model_uses_main_model: small_model.is_none(),
        small_model,
        reasoning_effort,
        thinking_mode,
    })
}

fn apply_profile_to_settings_path(
    profile: &ClaudeCodeProfile,
    settings_path: &Path,
) -> Result<(), String> {
    let mut root = read_settings_object_from_path(settings_path)?;
    let mut env = get_env_object(&root)?.cloned().unwrap_or_default();

    set_env_string(&mut env, CLAUDE_BASE_URL_ENV, profile.base_url.as_deref());
    set_env_string(
        &mut env,
        CLAUDE_AUTH_TOKEN_ENV,
        profile.bearer_token.as_deref(),
    );
    set_env_string(&mut env, CLAUDE_MODEL_ENV, profile.model.as_deref());
    let resolved_small_model = resolved_small_model_value(profile);
    set_env_string(
        &mut env,
        CLAUDE_SMALL_MODEL_ENV,
        resolved_small_model.as_deref(),
    );

    apply_reasoning_env(&mut env, profile.reasoning_effort);
    apply_thinking_settings(&mut root, &mut env, profile.thinking_mode);
    remove_conflicting_env_keys(&mut env);

    if env.is_empty() {
        root.remove("env");
    } else {
        root.insert("env".to_string(), Value::Object(env));
    }
    cleanup_env_if_empty(&mut root);

    let content = serde_json::to_string_pretty(&Value::Object(root))
        .map_err(|e| format!("Failed to serialize settings.json: {e}"))?;
    storage::atomic_write(settings_path, content.as_bytes())
}

fn read_current_config_from_path(settings_path: &Path) -> Result<ClaudeCurrentConfig, String> {
    let root = read_settings_object_from_path(settings_path)?;
    build_current_config_from_settings(&root)
}

// ============================================================================
// CRUD (Profiles)
// ============================================================================

pub fn list_claude_profiles_for_home(home_dir: &Path) -> Result<Vec<ClaudeCodeProfile>, String> {
    let dir = profiles_dir_for_home(home_dir)?;
    if !dir.exists() {
        return Ok(vec![]);
    }

    let mut profiles = Vec::new();
    for entry in std::fs::read_dir(&dir).map_err(|e| format!("Failed to read profiles dir: {e}"))? {
        let entry = match entry {
            Ok(entry) => entry,
            Err(_) => continue,
        };
        let path = entry.path();
        if path.extension().and_then(|s| s.to_str()) != Some("json") {
            continue;
        }
        if let Ok(profile) = read_profile_file(&path) {
            profiles.push(profile);
        }
    }

    profiles.sort_by_key(|profile| profile.name.to_lowercase());
    Ok(profiles)
}

pub fn get_claude_profile_for_home(home_dir: &Path, id: &str) -> Result<ClaudeCodeProfile, String> {
    load_profile_by_id(home_dir, id)
}

pub fn resolve_claude_profile_selector_for_home(
    home_dir: &Path,
    selector: &str,
) -> Result<ClaudeCodeProfile, String> {
    let selector = selector.trim();
    if selector.is_empty() {
        return Err("Claude profile selector cannot be empty".to_string());
    }

    let profiles = list_claude_profiles_for_home(home_dir)?;

    if let Some(profile) = profiles.iter().find(|profile| profile.id == selector) {
        return Ok(profile.clone());
    }

    if let Some(profile) = resolve_profile_by_name(&profiles, selector)? {
        return Ok(profile.clone());
    }

    if let Ok(index) = selector.parse::<usize>() {
        if let Some(profile) = index
            .checked_sub(1)
            .and_then(|zero_based_index| profiles.get(zero_based_index))
        {
            return Ok(profile.clone());
        }
    }

    Err(format!(
        "No Claude profile matches '{selector}'. Use `droidgear-tui run claude --list` to inspect available profiles."
    ))
}

pub fn save_claude_profile_for_home(
    home_dir: &Path,
    mut profile: ClaudeCodeProfile,
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

pub fn delete_claude_profile_for_home(home_dir: &Path, id: &str) -> Result<(), String> {
    let path = profile_path_for_home(home_dir, id)?;
    if path.exists() {
        std::fs::remove_file(&path).map_err(|e| format!("Failed to delete profile: {e}"))?;
    }

    if let Ok(active) = get_active_claude_profile_id_for_home(home_dir) {
        if active.as_deref() == Some(id) {
            let active_path = active_profile_path_for_home(home_dir)?;
            let _ = std::fs::remove_file(active_path);
        }
    }
    Ok(())
}

pub fn duplicate_claude_profile_for_home(
    home_dir: &Path,
    id: &str,
    new_name: &str,
) -> Result<ClaudeCodeProfile, String> {
    let mut profile = load_profile_by_id(home_dir, id)?;
    profile.id = Uuid::new_v4().to_string();
    profile.name = new_name.to_string();
    profile.created_at = now_rfc3339();
    profile.updated_at = profile.created_at.clone();
    write_profile_file(home_dir, &profile)?;
    Ok(profile)
}

pub fn create_default_claude_profile_for_home(
    home_dir: &Path,
) -> Result<ClaudeCodeProfile, String> {
    let profiles = list_claude_profiles_for_home(home_dir)?;
    if !profiles.is_empty() {
        return Err("Profiles already exist".to_string());
    }

    let now = now_rfc3339();
    let profile = ClaudeCodeProfile {
        id: Uuid::new_v4().to_string(),
        name: "Default".to_string(),
        description: None,
        base_url: None,
        bearer_token: None,
        model: None,
        small_model_uses_main_model: true,
        small_model: None,
        reasoning_effort: None,
        thinking_mode: ClaudeThinkingMode::Inherit,
        created_at: now.clone(),
        updated_at: now,
    };

    write_profile_file(home_dir, &profile)?;
    Ok(profile)
}

// ============================================================================
// Active profile
// ============================================================================

pub fn get_active_claude_profile_id_for_home(home_dir: &Path) -> Result<Option<String>, String> {
    let path = active_profile_path_for_home(home_dir)?;
    if !path.exists() {
        return Ok(None);
    }
    let content = std::fs::read_to_string(&path)
        .map_err(|e| format!("Failed to read active profile id: {e}"))?;
    let id = content.trim().to_string();
    if id.is_empty() {
        Ok(None)
    } else {
        Ok(Some(id))
    }
}

pub fn set_active_claude_profile_id_for_home(home_dir: &Path, id: &str) -> Result<(), String> {
    let path = active_profile_path_for_home(home_dir)?;
    let trimmed = id.trim();
    if trimmed.is_empty() {
        if path.exists() {
            std::fs::remove_file(&path)
                .map_err(|e| format!("Failed to clear active profile id: {e}"))?;
        }
        return Ok(());
    }

    validate_profile_id(trimmed)?;
    storage::atomic_write(&path, trimmed.as_bytes())
}

// ============================================================================
// Apply + status + read current config
// ============================================================================

pub fn apply_claude_profile_for_home(home_dir: &Path, id: &str) -> Result<(), String> {
    let profile = load_profile_by_id(home_dir, id)?;
    let settings_path = claude_settings_path_for_home(home_dir)?;
    apply_profile_to_settings_path(&profile, &settings_path)?;
    set_active_claude_profile_id_for_home(home_dir, id)?;
    Ok(())
}

pub fn get_claude_config_status_for_home(home_dir: &Path) -> Result<ClaudeConfigStatus, String> {
    let config_dir = claude_config_dir_for_home(home_dir)?;
    let settings_path = claude_settings_path_for_home(home_dir)?;
    let parse_error = if settings_path.exists() {
        read_settings_object_from_path(&settings_path).err()
    } else {
        None
    };

    Ok(ClaudeConfigStatus {
        settings_exists: settings_path.exists(),
        settings_path: settings_path.to_string_lossy().to_string(),
        config_dir: config_dir.to_string_lossy().to_string(),
        parse_error,
    })
}

pub fn read_claude_current_config_for_home(home_dir: &Path) -> Result<ClaudeCurrentConfig, String> {
    let settings_path = claude_settings_path_for_home(home_dir)?;
    read_current_config_from_path(&settings_path)
}

// ============================================================================
// System wrappers (CRUD)
// ============================================================================

pub fn list_claude_profiles() -> Result<Vec<ClaudeCodeProfile>, String> {
    list_claude_profiles_for_home(&system_home_dir()?)
}

pub fn get_claude_profile(id: &str) -> Result<ClaudeCodeProfile, String> {
    get_claude_profile_for_home(&system_home_dir()?, id)
}

pub fn resolve_claude_profile_selector(selector: &str) -> Result<ClaudeCodeProfile, String> {
    resolve_claude_profile_selector_for_home(&system_home_dir()?, selector)
}

pub fn save_claude_profile(profile: ClaudeCodeProfile) -> Result<(), String> {
    save_claude_profile_for_home(&system_home_dir()?, profile)
}

pub fn delete_claude_profile(id: &str) -> Result<(), String> {
    delete_claude_profile_for_home(&system_home_dir()?, id)
}

pub fn duplicate_claude_profile(id: &str, new_name: &str) -> Result<ClaudeCodeProfile, String> {
    duplicate_claude_profile_for_home(&system_home_dir()?, id, new_name)
}

pub fn create_default_claude_profile() -> Result<ClaudeCodeProfile, String> {
    create_default_claude_profile_for_home(&system_home_dir()?)
}

pub fn get_active_claude_profile_id() -> Result<Option<String>, String> {
    get_active_claude_profile_id_for_home(&system_home_dir()?)
}

pub fn set_active_claude_profile_id(id: &str) -> Result<(), String> {
    set_active_claude_profile_id_for_home(&system_home_dir()?, id)
}

pub fn apply_claude_profile(id: &str) -> Result<(), String> {
    apply_claude_profile_for_home(&system_home_dir()?, id)
}

pub fn get_claude_config_status() -> Result<ClaudeConfigStatus, String> {
    get_claude_config_status_for_home(&system_home_dir()?)
}

pub fn read_claude_current_config() -> Result<ClaudeCurrentConfig, String> {
    read_claude_current_config_for_home(&system_home_dir()?)
}

// ============================================================================
// Unit Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn home(temp: &TempDir) -> &Path {
        temp.path()
    }

    fn make_profile(id: &str, name: &str) -> ClaudeCodeProfile {
        ClaudeCodeProfile {
            id: id.to_string(),
            name: name.to_string(),
            description: Some("A test profile".to_string()),
            base_url: Some("https://example.com/v1".to_string()),
            bearer_token: Some("secret-token".to_string()),
            model: Some("claude-sonnet-4-5".to_string()),
            small_model_uses_main_model: true,
            small_model: None,
            reasoning_effort: Some(ClaudeReasoningEffort::High),
            thinking_mode: ClaudeThinkingMode::On,
            created_at: "2026-01-01T00:00:00Z".to_string(),
            updated_at: "2026-01-01T00:00:00Z".to_string(),
        }
    }

    #[test]
    fn test_reasoning_effort_serialization() {
        let json = serde_json::to_string(&ClaudeReasoningEffort::Max).unwrap();
        assert_eq!(json, "\"max\"");
    }

    #[test]
    fn test_profile_deserialization_applies_defaults() {
        let json = r#"{
            "id": "test-id",
            "name": "Test Profile",
            "createdAt": "2026-01-01T00:00:00Z",
            "updatedAt": "2026-01-01T00:00:00Z"
        }"#;

        let profile: ClaudeCodeProfile = serde_json::from_str(json).unwrap();
        assert!(profile.small_model_uses_main_model);
        assert_eq!(profile.thinking_mode, ClaudeThinkingMode::Inherit);
        assert_eq!(profile.reasoning_effort, None);
    }

    #[test]
    fn test_validate_profile_id() {
        assert!(validate_profile_id("valid-id").is_ok());
        assert!(validate_profile_id("valid_id").is_ok());
        assert!(validate_profile_id("abc123").is_ok());
        assert!(validate_profile_id("").is_err());
        assert!(validate_profile_id("has spaces").is_err());
        assert!(validate_profile_id("has/slash").is_err());
    }

    #[test]
    fn test_path_helpers() {
        let temp = TempDir::new().unwrap();
        let home = home(&temp);

        let profiles = profiles_dir_for_home(home).unwrap();
        assert!(profiles.ends_with(".droidgear/claude/profiles"));

        let active = active_profile_path_for_home(home).unwrap();
        assert!(active.ends_with(".droidgear/claude/active-profile.txt"));

        let settings = claude_settings_path_for_home(home).unwrap();
        assert!(settings.ends_with(".claude/settings.json"));
    }

    #[test]
    fn test_claude_config_dir_honors_path_override() {
        let temp = TempDir::new().unwrap();
        let home = home(&temp);

        let default_dir = claude_config_dir_for_home(home).unwrap();
        assert!(default_dir.ends_with(".claude"));

        let custom_dir = home.join("custom-claude");
        paths::save_config_path_for_home(home, "claude", &custom_dir.to_string_lossy()).unwrap();

        let resolved = claude_config_dir_for_home(home).unwrap();
        assert_eq!(resolved, custom_dir);
        assert!(resolved.exists());
    }

    #[test]
    fn test_create_default_profile() {
        let temp = TempDir::new().unwrap();
        let home = home(&temp);

        let profile = create_default_claude_profile_for_home(home).unwrap();
        assert!(!profile.id.is_empty());
        assert_eq!(profile.name, "Default");
        assert!(profile.small_model_uses_main_model);
        assert_eq!(profile.thinking_mode, ClaudeThinkingMode::Inherit);
        assert_eq!(profile.reasoning_effort, None);

        let err = create_default_claude_profile_for_home(home).unwrap_err();
        assert_eq!(err, "Profiles already exist");
    }

    #[test]
    fn test_save_and_get_profile() {
        let temp = TempDir::new().unwrap();
        let home = home(&temp);

        let profile = make_profile("p1", "Profile 1");
        save_claude_profile_for_home(home, profile).unwrap();

        let loaded = get_claude_profile_for_home(home, "p1").unwrap();
        assert_eq!(loaded.id, "p1");
        assert_eq!(loaded.name, "Profile 1");
        assert_eq!(loaded.model.as_deref(), Some("claude-sonnet-4-5"));
    }

    #[test]
    fn test_save_profile_generates_id_when_empty() {
        let temp = TempDir::new().unwrap();
        let home = home(&temp);

        let profile = ClaudeCodeProfile {
            id: String::new(),
            name: "New Profile".to_string(),
            description: None,
            base_url: None,
            bearer_token: None,
            model: None,
            small_model_uses_main_model: true,
            small_model: None,
            reasoning_effort: None,
            thinking_mode: ClaudeThinkingMode::Inherit,
            created_at: String::new(),
            updated_at: String::new(),
        };

        save_claude_profile_for_home(home, profile).unwrap();

        let profiles = list_claude_profiles_for_home(home).unwrap();
        assert_eq!(profiles.len(), 1);
        assert!(!profiles[0].id.is_empty());
        assert!(!profiles[0].created_at.is_empty());
        assert!(!profiles[0].updated_at.is_empty());
        assert_eq!(profiles[0].name, "New Profile");
    }

    #[test]
    fn test_save_profile_updates_timestamp() {
        let temp = TempDir::new().unwrap();
        let home = home(&temp);

        save_claude_profile_for_home(home, make_profile("p1", "Profile 1")).unwrap();

        let loaded = get_claude_profile_for_home(home, "p1").unwrap();
        assert_ne!(loaded.updated_at, "2026-01-01T00:00:00Z");
        let first_updated_at = loaded.updated_at.clone();

        let mut updated = loaded.clone();
        updated.name = "Updated Name".to_string();
        save_claude_profile_for_home(home, updated).unwrap();

        let reloaded = get_claude_profile_for_home(home, "p1").unwrap();
        assert_eq!(reloaded.name, "Updated Name");
        assert_ne!(reloaded.updated_at, first_updated_at);
        assert_eq!(reloaded.created_at, "2026-01-01T00:00:00Z");
    }

    #[test]
    fn test_list_profiles_sorted_by_name() {
        let temp = TempDir::new().unwrap();
        let home = home(&temp);

        save_claude_profile_for_home(home, make_profile("p1", "Zebra")).unwrap();
        save_claude_profile_for_home(home, make_profile("p2", "Alpha")).unwrap();
        save_claude_profile_for_home(home, make_profile("p3", "Middle")).unwrap();

        let profiles = list_claude_profiles_for_home(home).unwrap();
        assert_eq!(profiles.len(), 3);
        assert_eq!(profiles[0].name, "Alpha");
        assert_eq!(profiles[1].name, "Middle");
        assert_eq!(profiles[2].name, "Zebra");
    }

    #[test]
    fn test_duplicate_profile() {
        let temp = TempDir::new().unwrap();
        let home = home(&temp);

        save_claude_profile_for_home(home, make_profile("p1", "Original")).unwrap();

        let duplicate = duplicate_claude_profile_for_home(home, "p1", "Copy").unwrap();
        assert_ne!(duplicate.id, "p1");
        assert_eq!(duplicate.name, "Copy");
        assert_eq!(
            duplicate.base_url.as_deref(),
            Some("https://example.com/v1")
        );

        let profiles = list_claude_profiles_for_home(home).unwrap();
        assert_eq!(profiles.len(), 2);
    }

    #[test]
    fn test_delete_profile_clears_active_profile() {
        let temp = TempDir::new().unwrap();
        let home = home(&temp);

        save_claude_profile_for_home(home, make_profile("p1", "Profile 1")).unwrap();
        set_active_claude_profile_id_for_home(home, "p1").unwrap();

        delete_claude_profile_for_home(home, "p1").unwrap();
        assert!(list_claude_profiles_for_home(home).unwrap().is_empty());
        assert_eq!(get_active_claude_profile_id_for_home(home).unwrap(), None);
    }

    #[test]
    fn test_active_profile_roundtrip_and_clear() {
        let temp = TempDir::new().unwrap();
        let home = home(&temp);

        assert_eq!(get_active_claude_profile_id_for_home(home).unwrap(), None);

        set_active_claude_profile_id_for_home(home, "profile_1").unwrap();
        assert_eq!(
            get_active_claude_profile_id_for_home(home).unwrap(),
            Some("profile_1".to_string())
        );

        set_active_claude_profile_id_for_home(home, "").unwrap();
        assert_eq!(get_active_claude_profile_id_for_home(home).unwrap(), None);
    }

    #[test]
    fn test_set_active_profile_rejects_invalid_id() {
        let temp = TempDir::new().unwrap();
        let home = home(&temp);

        let err = set_active_claude_profile_id_for_home(home, "bad id").unwrap_err();
        assert_eq!(err, "Invalid profile id");
    }

    #[test]
    fn test_resolve_claude_profile_selector_accepts_id_name_and_index() {
        let temp = TempDir::new().unwrap();

        save_claude_profile_for_home(temp.path(), make_profile("profile-a", "First Profile"))
            .unwrap();
        save_claude_profile_for_home(temp.path(), make_profile("profile-b", "Second Profile"))
            .unwrap();

        let by_id = resolve_claude_profile_selector_for_home(temp.path(), "profile-a").unwrap();
        let by_name =
            resolve_claude_profile_selector_for_home(temp.path(), "second profile").unwrap();
        let by_index = resolve_claude_profile_selector_for_home(temp.path(), "2").unwrap();

        assert_eq!(by_id.id, "profile-a");
        assert_eq!(by_name.id, "profile-b");
        assert_eq!(by_index.id, "profile-b");
    }

    #[test]
    fn test_resolve_claude_profile_selector_rejects_ambiguous_names() {
        let temp = TempDir::new().unwrap();

        save_claude_profile_for_home(temp.path(), make_profile("profile-a", "Shared")).unwrap();
        save_claude_profile_for_home(temp.path(), make_profile("profile-b", "Shared")).unwrap();

        let error = resolve_claude_profile_selector_for_home(temp.path(), "Shared").unwrap_err();
        assert!(error.contains("Multiple Claude profiles share the name"));
    }

    #[test]
    fn test_recognized_claude_model_id_matches_official_patterns() {
        assert!(is_recognized_claude_model_id("claude-sonnet-4-5"));
        assert!(is_recognized_claude_model_id(" claude_opus_4_7 "));
        assert!(!is_recognized_claude_model_id("gateway-prod-model"));
        assert!(!is_recognized_claude_model_id(""));
    }

    #[test]
    fn test_has_opaque_claude_model_id_flags_custom_ids() {
        assert!(has_opaque_claude_model_id(Some("gateway-prod-model")));
        assert!(has_opaque_claude_model_id(Some(
            "anthropic/claude-sonnet-4-5"
        )));
        assert!(!has_opaque_claude_model_id(Some("claude-sonnet-4-5")));
        assert!(!has_opaque_claude_model_id(None));
    }

    fn write_file(path: &Path, content: &str) {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).unwrap();
        }
        std::fs::write(path, content).unwrap();
    }

    fn read_json(path: &Path) -> Value {
        serde_json::from_str(&std::fs::read_to_string(path).unwrap()).unwrap()
    }

    #[test]
    fn test_apply_creates_settings_file_when_missing() {
        let temp = TempDir::new().unwrap();
        let home = home(&temp);

        save_claude_profile_for_home(home, make_profile("p1", "Profile 1")).unwrap();
        apply_claude_profile_for_home(home, "p1").unwrap();

        let settings_path = claude_settings_path_for_home(home).unwrap();
        let settings = read_json(&settings_path);
        let env = settings.get("env").and_then(Value::as_object).unwrap();

        assert_eq!(
            env.get(CLAUDE_BASE_URL_ENV).and_then(Value::as_str),
            Some("https://example.com/v1")
        );
        assert_eq!(
            env.get(CLAUDE_AUTH_TOKEN_ENV).and_then(Value::as_str),
            Some("secret-token")
        );
        assert_eq!(
            env.get(CLAUDE_MODEL_ENV).and_then(Value::as_str),
            Some("claude-sonnet-4-5")
        );
        assert_eq!(
            env.get(CLAUDE_EFFORT_ENV).and_then(Value::as_str),
            Some("high")
        );
        assert_eq!(
            env.get(CLAUDE_DISABLE_ADAPTIVE_ENV).and_then(Value::as_str),
            Some("1")
        );
        assert_eq!(
            settings
                .get("alwaysThinkingEnabled")
                .and_then(Value::as_bool),
            Some(true)
        );
        assert_eq!(
            get_active_claude_profile_id_for_home(home).unwrap(),
            Some("p1".to_string())
        );
    }

    #[test]
    fn test_apply_preserves_unrelated_settings_and_cleans_conflicts() {
        let temp = TempDir::new().unwrap();
        let home = home(&temp);
        let settings_path = claude_settings_path_for_home(home).unwrap();

        write_file(
            &settings_path,
            r#"{
              "theme": "dark",
              "model": "top-level-model",
              "env": {
                "KEEP_ME": "1",
                "ANTHROPIC_API_KEY": "bad",
                "CLAUDE_CODE_USE_BEDROCK": "1",
                "CLAUDE_CODE_DISABLE_THINKING": "1",
                "MAX_THINKING_TOKENS": "4096"
              }
            }"#,
        );

        let mut profile = make_profile("p1", "Profile 1");
        profile.reasoning_effort = Some(ClaudeReasoningEffort::Medium);
        profile.thinking_mode = ClaudeThinkingMode::On;
        save_claude_profile_for_home(home, profile).unwrap();

        apply_claude_profile_for_home(home, "p1").unwrap();

        let settings = read_json(&settings_path);
        assert_eq!(settings.get("theme").and_then(Value::as_str), Some("dark"));
        assert_eq!(
            settings.get("model").and_then(Value::as_str),
            Some("top-level-model")
        );
        assert_eq!(
            settings
                .get("alwaysThinkingEnabled")
                .and_then(Value::as_bool),
            Some(true)
        );

        let env = settings.get("env").and_then(Value::as_object).unwrap();
        assert_eq!(env.get("KEEP_ME").and_then(Value::as_str), Some("1"));
        assert!(!env.contains_key(CLAUDE_API_KEY_ENV));
        assert!(!env.contains_key("CLAUDE_CODE_USE_BEDROCK"));
        assert!(!env.contains_key(CLAUDE_DISABLE_THINKING_ENV));
        assert!(!env.contains_key(CLAUDE_MAX_THINKING_TOKENS_ENV));
        assert_eq!(
            env.get(CLAUDE_EFFORT_ENV).and_then(Value::as_str),
            Some("medium")
        );
        assert!(!env.contains_key(CLAUDE_DISABLE_ADAPTIVE_ENV));
    }

    #[test]
    fn test_apply_small_model_uses_main_model_sets_small_model_to_main_model() {
        let temp = TempDir::new().unwrap();
        let home = home(&temp);
        let settings_path = claude_settings_path_for_home(home).unwrap();

        write_file(
            &settings_path,
            r#"{"env":{"ANTHROPIC_DEFAULT_HAIKU_MODEL":"claude-haiku-4"}}"#,
        );

        let mut profile = make_profile("p1", "Profile 1");
        profile.small_model_uses_main_model = true;
        profile.small_model = Some("claude-haiku-4".to_string());
        save_claude_profile_for_home(home, profile).unwrap();

        apply_claude_profile_for_home(home, "p1").unwrap();

        let settings = read_json(&settings_path);
        let env = settings.get("env").and_then(Value::as_object).unwrap();
        assert_eq!(
            env.get(CLAUDE_SMALL_MODEL_ENV).and_then(Value::as_str),
            Some("claude-sonnet-4-5")
        );
    }

    #[test]
    fn test_apply_thinking_modes_manage_expected_keys() {
        let temp = TempDir::new().unwrap();
        let home = home(&temp);
        let settings_path = claude_settings_path_for_home(home).unwrap();

        write_file(
            &settings_path,
            r#"{
              "alwaysThinkingEnabled": false,
              "env": {
                "CLAUDE_CODE_DISABLE_THINKING": "1",
                "MAX_THINKING_TOKENS": "1024"
              }
            }"#,
        );

        let mut profile = make_profile("p1", "Profile 1");
        profile.reasoning_effort = None;
        profile.thinking_mode = ClaudeThinkingMode::Inherit;
        save_claude_profile_for_home(home, profile.clone()).unwrap();
        apply_claude_profile_for_home(home, "p1").unwrap();

        let settings = read_json(&settings_path);
        assert!(settings.get("alwaysThinkingEnabled").is_none());
        let env = settings.get("env").and_then(Value::as_object).unwrap();
        assert!(!env.contains_key(CLAUDE_DISABLE_THINKING_ENV));
        assert_eq!(
            env.get(CLAUDE_MAX_THINKING_TOKENS_ENV)
                .and_then(Value::as_str),
            Some("1024")
        );

        let mut profile_on = profile.clone();
        profile_on.thinking_mode = ClaudeThinkingMode::On;
        save_claude_profile_for_home(home, profile_on).unwrap();
        apply_claude_profile_for_home(home, "p1").unwrap();

        let settings_on = read_json(&settings_path);
        let env_on = settings_on.get("env").and_then(Value::as_object).unwrap();
        assert_eq!(
            settings_on
                .get("alwaysThinkingEnabled")
                .and_then(Value::as_bool),
            Some(true)
        );
        assert!(!env_on.contains_key(CLAUDE_DISABLE_THINKING_ENV));
        assert!(!env_on.contains_key(CLAUDE_MAX_THINKING_TOKENS_ENV));

        let mut profile_off = profile.clone();
        profile_off.thinking_mode = ClaudeThinkingMode::Off;
        save_claude_profile_for_home(home, profile_off).unwrap();
        apply_claude_profile_for_home(home, "p1").unwrap();

        let settings_off = read_json(&settings_path);
        let env_off = settings_off.get("env").and_then(Value::as_object).unwrap();
        assert_eq!(
            settings_off
                .get("alwaysThinkingEnabled")
                .and_then(Value::as_bool),
            Some(false)
        );
        assert_eq!(
            env_off
                .get(CLAUDE_DISABLE_THINKING_ENV)
                .and_then(Value::as_str),
            Some("1")
        );
        assert!(!env_off.contains_key(CLAUDE_MAX_THINKING_TOKENS_ENV));
    }

    #[test]
    fn test_apply_reasoning_effort_writes_and_clears_expected_keys() {
        let temp = TempDir::new().unwrap();
        let home = home(&temp);
        let settings_path = claude_settings_path_for_home(home).unwrap();

        write_file(
            &settings_path,
            r#"{"env":{"CLAUDE_CODE_EFFORT_LEVEL":"high","CLAUDE_CODE_DISABLE_ADAPTIVE_THINKING":"1"}}"#,
        );

        let mut profile = make_profile("p1", "Profile 1");
        profile.thinking_mode = ClaudeThinkingMode::Inherit;
        profile.reasoning_effort = Some(ClaudeReasoningEffort::Low);
        save_claude_profile_for_home(home, profile.clone()).unwrap();
        apply_claude_profile_for_home(home, "p1").unwrap();

        let settings_low = read_json(&settings_path);
        let env_low = settings_low.get("env").and_then(Value::as_object).unwrap();
        assert_eq!(
            env_low.get(CLAUDE_EFFORT_ENV).and_then(Value::as_str),
            Some("low")
        );
        assert!(!env_low.contains_key(CLAUDE_DISABLE_ADAPTIVE_ENV));

        let mut profile_max = profile.clone();
        profile_max.reasoning_effort = Some(ClaudeReasoningEffort::Max);
        save_claude_profile_for_home(home, profile_max).unwrap();
        apply_claude_profile_for_home(home, "p1").unwrap();

        let settings_max = read_json(&settings_path);
        let env_max = settings_max.get("env").and_then(Value::as_object).unwrap();
        assert_eq!(
            env_max.get(CLAUDE_EFFORT_ENV).and_then(Value::as_str),
            Some("max")
        );
        assert_eq!(
            env_max
                .get(CLAUDE_DISABLE_ADAPTIVE_ENV)
                .and_then(Value::as_str),
            Some("1")
        );

        let mut profile_none = profile;
        profile_none.reasoning_effort = None;
        save_claude_profile_for_home(home, profile_none).unwrap();
        apply_claude_profile_for_home(home, "p1").unwrap();

        let settings_none = read_json(&settings_path);
        let env_none = settings_none.get("env").and_then(Value::as_object).unwrap();
        assert!(!env_none.contains_key(CLAUDE_EFFORT_ENV));
        assert!(!env_none.contains_key(CLAUDE_DISABLE_ADAPTIVE_ENV));
    }

    #[test]
    fn test_apply_returns_error_for_malformed_settings_json() {
        let temp = TempDir::new().unwrap();
        let home = home(&temp);
        let settings_path = claude_settings_path_for_home(home).unwrap();

        write_file(&settings_path, "{not-json");
        save_claude_profile_for_home(home, make_profile("p1", "Profile 1")).unwrap();

        let err = apply_claude_profile_for_home(home, "p1").unwrap_err();
        assert!(err.contains("Failed to parse settings.json"));
    }

    #[test]
    fn test_config_status_reports_parse_errors() {
        let temp = TempDir::new().unwrap();
        let home = home(&temp);
        let settings_path = claude_settings_path_for_home(home).unwrap();

        write_file(&settings_path, "{not-json");

        let status = get_claude_config_status_for_home(home).unwrap();
        assert!(status.settings_exists);
        assert!(status.parse_error.is_some());
        assert!(status.settings_path.ends_with(".claude/settings.json"));
        assert!(status.config_dir.ends_with(".claude"));
    }

    #[test]
    fn test_read_current_config_prefers_managed_env_and_falls_back_to_top_level() {
        let temp = TempDir::new().unwrap();
        let home = home(&temp);
        let settings_path = claude_settings_path_for_home(home).unwrap();

        write_file(
            &settings_path,
            r#"{
              "model": "top-level-model",
              "effortLevel": "max",
              "alwaysThinkingEnabled": true,
              "env": {
                "ANTHROPIC_BASE_URL": "https://proxy.example.com",
                "ANTHROPIC_AUTH_TOKEN": "bearer-123",
                "ANTHROPIC_MODEL": "env-model",
                "ANTHROPIC_DEFAULT_HAIKU_MODEL": "env-small",
                "CLAUDE_CODE_EFFORT_LEVEL": "medium"
              }
            }"#,
        );

        let current = read_claude_current_config_for_home(home).unwrap();
        assert_eq!(
            current.base_url.as_deref(),
            Some("https://proxy.example.com")
        );
        assert_eq!(current.bearer_token.as_deref(), Some("bearer-123"));
        assert_eq!(current.model.as_deref(), Some("env-model"));
        assert!(!current.small_model_uses_main_model);
        assert_eq!(current.small_model.as_deref(), Some("env-small"));
        assert_eq!(
            current.reasoning_effort,
            Some(ClaudeReasoningEffort::Medium)
        );
        assert_eq!(current.thinking_mode, ClaudeThinkingMode::On);

        write_file(
            &settings_path,
            r#"{
              "model": "top-level-model",
              "effortLevel": "high",
              "alwaysThinkingEnabled": false
            }"#,
        );

        let fallback = read_claude_current_config_for_home(home).unwrap();
        assert_eq!(fallback.model.as_deref(), Some("top-level-model"));
        assert_eq!(fallback.reasoning_effort, Some(ClaudeReasoningEffort::High));
        assert!(fallback.small_model_uses_main_model);
        assert_eq!(fallback.small_model, None);
        assert_eq!(fallback.thinking_mode, ClaudeThinkingMode::Off);
    }
}
