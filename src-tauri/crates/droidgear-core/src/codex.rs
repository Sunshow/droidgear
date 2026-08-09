//! Codex CLI 配置管理（core）。
//!
//! 负责 Profile CRUD，并支持将 Profile 应用到 `~/.codex/auth.json` 与 `~/.codex/config.toml`。
//! 逻辑从原 Tauri command 层抽离，以便在 TUI 与桌面端复用。

use chrono::Utc;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use specta::Type;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use uuid::Uuid;

use crate::{json, paths, storage};

// ============================================================================
// Types
// ============================================================================

pub(crate) const OPENAI_API_KEY_FIELD: &str = "OPENAI_API_KEY";

/// DeepSeek V4 models that require the Codex model catalog file
/// (`model_catalog_json` in config.toml). Content mirrors the official
/// DeepSeek setup script (codex-deepseek-setup.sh).
const DEEPSEEK_V4_MODELS: [&str; 2] = ["deepseek-v4-flash", "deepseek-v4-pro"];

/// MiMo models that require the Codex model catalog file
/// (`model_catalog_json` in config.toml).
const MIMO_MODELS: [&str; 2] = ["mimo-v2.5", "mimo-v2.5-pro"];

/// Model catalog content for the DeepSeek V4 models, extracted verbatim
/// from the official setup script.
const DEEPSEEK_MODELS_JSON: &str = include_str!("../res/codex-models.json");

/// Model catalog content for the MiMo models, extracted verbatim from the
/// official MiMo Codex docs.
const MIMO_MODELS_JSON: &str = include_str!("../res/codex-mimo-models.json");

/// Codex Provider 配置（对应 config.toml 中的 [model_providers.<id>]）
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct CodexProviderConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub base_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub wire_api: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub requires_openai_auth: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub env_key: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub env_key_instructions: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub http_headers: Option<HashMap<String, String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub query_params: Option<HashMap<String, String>>,
    // DroidGear-only 字段（不写入 config.toml 的 [model_providers] 中）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model_reasoning_effort: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub api_key: Option<String>,
}

/// Codex Profile（用于在 DroidGear 内部保存并切换）
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct CodexProfile {
    pub id: String,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    pub created_at: String,
    pub updated_at: String,
    #[serde(default)]
    pub providers: HashMap<String, CodexProviderConfig>,
    pub model_provider: String,
    pub model: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model_reasoning_effort: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub api_key: Option<String>,
    /// Saved Codex auth profile name to restore on apply (openai mode only).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub auth_profile_name: Option<String>,
}

/// Codex Live 配置状态
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct CodexConfigStatus {
    pub auth_exists: bool,
    pub config_exists: bool,
    pub auth_path: String,
    pub config_path: String,
}

/// 当前 Codex Live 配置（从 `~/.codex/*` 读取）
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct CodexCurrentConfig {
    #[serde(default)]
    pub providers: HashMap<String, CodexProviderConfig>,
    pub model_provider: String,
    pub model: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model_reasoning_effort: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub api_key: Option<String>,
}

// ============================================================================
// Path Helpers
// ============================================================================

fn droidgear_codex_dir_for_home(home_dir: &Path) -> PathBuf {
    home_dir.join(".droidgear").join("codex")
}

/// `~/.droidgear/codex/profiles/`
fn profiles_dir_for_home(home_dir: &Path) -> Result<PathBuf, String> {
    let dir = droidgear_codex_dir_for_home(home_dir).join("profiles");
    if !dir.exists() {
        std::fs::create_dir_all(&dir)
            .map_err(|e| format!("Failed to create codex profiles directory: {e}"))?;
    }
    Ok(dir)
}

/// `~/.droidgear/codex/active-profile.txt`
fn active_profile_path_for_home(home_dir: &Path) -> Result<PathBuf, String> {
    let dir = droidgear_codex_dir_for_home(home_dir);
    if !dir.exists() {
        std::fs::create_dir_all(&dir)
            .map_err(|e| format!("Failed to create codex directory: {e}"))?;
    }
    Ok(dir.join("active-profile.txt"))
}

/// `~/.codex/` (or custom path)
fn codex_config_dir_for_home(home_dir: &Path) -> Result<PathBuf, String> {
    let config_paths = paths::load_config_paths_for_home(home_dir);
    let dir = paths::get_codex_home_for_home(home_dir, &config_paths)?;
    if !dir.exists() {
        std::fs::create_dir_all(&dir)
            .map_err(|e| format!("Failed to create codex config directory: {e}"))?;
    }
    Ok(dir)
}

fn codex_auth_path_for_home(home_dir: &Path) -> Result<PathBuf, String> {
    Ok(codex_config_dir_for_home(home_dir)?.join("auth.json"))
}

fn codex_config_path_for_home(home_dir: &Path) -> Result<PathBuf, String> {
    Ok(codex_config_dir_for_home(home_dir)?.join("config.toml"))
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
// TOML helpers
// ============================================================================

/// Model family that needs a Codex model catalog file
/// (`model_catalog_json` in config.toml). Each family keeps its own catalog
/// under `~/.codex/model-catalogs/` so Codex only lists models that the
/// configured endpoint actually serves.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ModelCatalog {
    DeepSeek,
    Mimo,
}

impl ModelCatalog {
    /// Catalog file name under `~/.codex/model-catalogs/`.
    fn file_name(self) -> &'static str {
        match self {
            ModelCatalog::DeepSeek => "deepseek.json",
            ModelCatalog::Mimo => "mimo.json",
        }
    }

    /// Catalog content for this family.
    fn content(self) -> &'static str {
        match self {
            ModelCatalog::DeepSeek => DEEPSEEK_MODELS_JSON,
            ModelCatalog::Mimo => MIMO_MODELS_JSON,
        }
    }
}

/// Whether `model` belongs to a family that needs the Codex model catalog,
/// and which one.
pub(crate) fn catalog_for_model(model: &str) -> Option<ModelCatalog> {
    let model = model.trim();
    if DEEPSEEK_V4_MODELS.contains(&model) {
        Some(ModelCatalog::DeepSeek)
    } else if MIMO_MODELS.contains(&model) {
        Some(ModelCatalog::Mimo)
    } else {
        None
    }
}

/// Value for `model_catalog_json` in config.toml: `~/.codex/model-catalogs/<family>.json`
/// when the codex home is the default `~/.codex`, otherwise the absolute
/// path so a custom codex home does not dangle.
fn model_catalog_json_value_for_home(
    home_dir: &Path,
    catalog: ModelCatalog,
) -> Result<String, String> {
    let config_paths = paths::load_config_paths_for_home(home_dir);
    let codex_home = paths::get_codex_home_for_home(home_dir, &config_paths)?;
    let rel = format!("model-catalogs/{}", catalog.file_name());
    if codex_home == home_dir.join(".codex") {
        Ok(format!("~/.codex/{rel}"))
    } else {
        Ok(codex_home.join(rel).to_string_lossy().into_owned())
    }
}

/// Write the model catalog for the active model's family under
/// `~/.codex/model-catalogs/`, and remove the legacy single-file catalog
/// (`~/.codex/models.json`) written by older releases. Codex only reads a
/// catalog via the `model_catalog_json` config key, so an orphaned catalog
/// would be dead weight — and a dangling reference would break startup.
pub fn sync_models_json_for_home(home_dir: &Path, model: &str) -> Result<(), String> {
    let codex_dir = codex_config_dir_for_home(home_dir)?;

    let legacy_models_path = codex_dir.join("models.json");
    if legacy_models_path.exists() {
        std::fs::remove_file(&legacy_models_path)
            .map_err(|e| format!("Failed to remove legacy codex models.json: {e}"))?;
    }

    if let Some(catalog) = catalog_for_model(model) {
        let catalog_path = codex_dir.join("model-catalogs").join(catalog.file_name());
        storage::atomic_write(&catalog_path, catalog.content().as_bytes())
            .map_err(|e| format!("Failed to write codex model catalog: {e}"))?;
    }
    Ok(())
}

/// Convert CodexProviderConfig to toml::Value. Codex rejects providers with
/// an empty name (`model_providers.<id>: provider name must not be empty`),
/// so a missing or blank name falls back to the provider id.
pub(crate) fn provider_config_to_toml(
    provider_id: &str,
    config: &CodexProviderConfig,
) -> Result<toml::Value, String> {
    let mut table = toml::map::Map::new();

    let name = config
        .name
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .unwrap_or(provider_id);
    table.insert("name".to_string(), toml::Value::String(name.to_string()));
    if let Some(ref base_url) = config.base_url {
        table.insert(
            "base_url".to_string(),
            toml::Value::String(base_url.clone()),
        );
    }
    if let Some(ref wire_api) = config.wire_api {
        table.insert(
            "wire_api".to_string(),
            toml::Value::String(wire_api.clone()),
        );
    }
    if let Some(requires_openai_auth) = config.requires_openai_auth {
        table.insert(
            "requires_openai_auth".to_string(),
            toml::Value::Boolean(requires_openai_auth),
        );
    }
    if let Some(ref env_key) = config.env_key {
        table.insert("env_key".to_string(), toml::Value::String(env_key.clone()));
    }
    if let Some(ref env_key_instructions) = config.env_key_instructions {
        table.insert(
            "env_key_instructions".to_string(),
            toml::Value::String(env_key_instructions.clone()),
        );
    }
    if let Some(ref http_headers) = config.http_headers {
        let mut headers_table = toml::map::Map::new();
        for (k, v) in http_headers {
            headers_table.insert(k.clone(), toml::Value::String(v.clone()));
        }
        table.insert(
            "http_headers".to_string(),
            toml::Value::Table(headers_table),
        );
    }
    if let Some(ref query_params) = config.query_params {
        let mut params_table = toml::map::Map::new();
        for (k, v) in query_params {
            params_table.insert(k.clone(), toml::Value::String(v.clone()));
        }
        table.insert("query_params".to_string(), toml::Value::Table(params_table));
    }

    Ok(toml::Value::Table(table))
}

pub(crate) fn resolve_active_provider(
    profile: &CodexProfile,
) -> (String, Option<&CodexProviderConfig>) {
    // Built-in OpenAI provider must never fall back to a custom provider id.
    if profile.model_provider == "openai" {
        return ("openai".to_string(), profile.providers.get("openai"));
    }
    if profile.providers.contains_key(&profile.model_provider) {
        (
            profile.model_provider.clone(),
            profile.providers.get(&profile.model_provider),
        )
    } else if let Some((first_id, first_config)) = profile.providers.iter().next() {
        (first_id.clone(), Some(first_config))
    } else {
        (profile.model_provider.clone(), None)
    }
}

pub(crate) fn resolved_model(
    profile: &CodexProfile,
    provider: Option<&CodexProviderConfig>,
) -> String {
    provider
        .and_then(|p| p.model.as_deref())
        .filter(|s| !s.is_empty())
        .unwrap_or(&profile.model)
        .to_string()
}

pub(crate) fn resolved_reasoning_effort(
    profile: &CodexProfile,
    provider: Option<&CodexProviderConfig>,
) -> Option<String> {
    provider
        .and_then(|p| p.model_reasoning_effort.clone())
        .or(profile.model_reasoning_effort.clone())
        .filter(|value| !value.is_empty())
}

pub(crate) fn resolved_api_key(
    profile: &CodexProfile,
    provider: Option<&CodexProviderConfig>,
) -> Option<String> {
    provider
        .and_then(|p| p.api_key.clone())
        .or(profile.api_key.clone())
        .filter(|value| !value.is_empty())
}

pub(crate) fn apply_profile_to_config_map(
    config: &mut toml::map::Map<String, toml::Value>,
    profile: &CodexProfile,
    home_dir: &Path,
) -> Result<(), String> {
    let (effective_provider_id, active_provider) = resolve_active_provider(profile);
    let resolved_model = resolved_model(profile, active_provider);
    let resolved_effort = resolved_reasoning_effort(profile, active_provider);
    let is_openai_provider = effective_provider_id == "openai";

    config.insert(
        "model_provider".to_string(),
        toml::Value::String(effective_provider_id),
    );
    config.insert(
        "model".to_string(),
        toml::Value::String(resolved_model.clone()),
    );

    if let Some(ref effort) = resolved_effort {
        config.insert(
            "model_reasoning_effort".to_string(),
            toml::Value::String(effort.clone()),
        );
    } else {
        config.remove("model_reasoning_effort");
    }

    // Official OpenAI mode should not inject custom model_providers into live config.
    config.remove("model_providers");
    if !is_openai_provider && !profile.providers.is_empty() {
        let mut providers_table = toml::map::Map::new();
        for (provider_id, provider_config) in &profile.providers {
            providers_table.insert(
                provider_id.clone(),
                provider_config_to_toml(provider_id, provider_config)?,
            );
        }
        config.insert(
            "model_providers".to_string(),
            toml::Value::Table(providers_table),
        );
    }

    // Model families that ship a catalog (DeepSeek V4, MiMo) point
    // model_catalog_json at their per-family catalog under model-catalogs/;
    // other models must not reference it, or Codex looks for a missing file.
    match catalog_for_model(&resolved_model) {
        Some(catalog) => {
            config.insert(
                "model_catalog_json".to_string(),
                toml::Value::String(model_catalog_json_value_for_home(home_dir, catalog)?),
            );
        }
        None => {
            config.remove("model_catalog_json");
        }
    }

    // MiMo 当前不支持 web search，写入 config.toml 关闭。
    // 等 MiMo 网关支持后删除这段兼容。
    if matches!(catalog_for_model(&resolved_model), Some(ModelCatalog::Mimo)) {
        config.insert(
            "web_search".to_string(),
            toml::Value::String("disabled".to_string()),
        );
    } else {
        config.remove("web_search");
    }

    Ok(())
}

pub(crate) fn apply_api_key_to_auth_map(
    auth: &mut HashMap<String, Value>,
    resolved_api_key: Option<&str>,
) {
    if let Some(key) = resolved_api_key {
        if !key.is_empty() {
            auth.insert(
                OPENAI_API_KEY_FIELD.to_string(),
                Value::String(key.to_string()),
            );
        } else {
            auth.remove(OPENAI_API_KEY_FIELD);
        }
    } else {
        auth.remove(OPENAI_API_KEY_FIELD);
    }
}

/// Parse CodexProviderConfig from toml::Value
fn toml_to_provider_config(value: &toml::Value) -> Result<CodexProviderConfig, String> {
    let table = value.as_table().ok_or("Provider config must be a table")?;

    let name = table
        .get("name")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());
    let base_url = table
        .get("base_url")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());
    let wire_api = table
        .get("wire_api")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());
    let requires_openai_auth = table.get("requires_openai_auth").and_then(|v| v.as_bool());
    let env_key = table
        .get("env_key")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());
    let env_key_instructions = table
        .get("env_key_instructions")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());

    let http_headers = table
        .get("http_headers")
        .and_then(|v| v.as_table())
        .map(|t| {
            t.iter()
                .filter_map(|(k, v)| v.as_str().map(|s| (k.clone(), s.to_string())))
                .collect::<HashMap<_, _>>()
        });

    let query_params = table
        .get("query_params")
        .and_then(|v| v.as_table())
        .map(|t| {
            t.iter()
                .filter_map(|(k, v)| v.as_str().map(|s| (k.clone(), s.to_string())))
                .collect::<HashMap<_, _>>()
        });

    Ok(CodexProviderConfig {
        name,
        base_url,
        wire_api,
        requires_openai_auth,
        env_key,
        env_key_instructions,
        http_headers,
        query_params,
        model: None,
        model_reasoning_effort: None,
        api_key: None,
    })
}

// ============================================================================
// CRUD (Profiles)
// ============================================================================

fn read_profile_file(path: &Path) -> Result<CodexProfile, String> {
    let s = std::fs::read_to_string(path).map_err(|e| format!("Failed to read profile: {e}"))?;
    serde_json::from_str::<CodexProfile>(&s).map_err(|e| format!("Invalid profile JSON: {e}"))
}

fn write_profile_file(home_dir: &Path, profile: &CodexProfile) -> Result<(), String> {
    let path = profile_path_for_home(home_dir, &profile.id)?;
    let s = serde_json::to_string_pretty(profile)
        .map_err(|e| format!("Failed to serialize profile JSON: {e}"))?;
    storage::atomic_write(&path, s.as_bytes())
}

fn load_profile_by_id(home_dir: &Path, id: &str) -> Result<CodexProfile, String> {
    let path = profile_path_for_home(home_dir, id)?;
    read_profile_file(&path)
}

fn resolve_profile_by_name<'a>(
    profiles: &'a [CodexProfile],
    selector: &str,
) -> Result<Option<&'a CodexProfile>, String> {
    let exact_matches = profiles
        .iter()
        .filter(|profile| profile.name == selector)
        .collect::<Vec<_>>();
    match exact_matches.as_slice() {
        [] => {}
        [profile] => return Ok(Some(profile)),
        _ => {
            return Err(format!(
                "Multiple Codex profiles share the name '{selector}'. Use the profile index or id instead."
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
            "Multiple Codex profiles share the name '{selector}'. Use the profile index or id instead."
        )),
    }
}

pub fn list_codex_profiles_for_home(home_dir: &Path) -> Result<Vec<CodexProfile>, String> {
    let dir = profiles_dir_for_home(home_dir)?;
    if !dir.exists() {
        return Ok(vec![]);
    }

    let mut profiles = Vec::new();
    for entry in std::fs::read_dir(&dir).map_err(|e| format!("Failed to read profiles dir: {e}"))? {
        let entry = match entry {
            Ok(e) => e,
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

    profiles.sort_by_key(|a| a.name.to_lowercase());
    Ok(profiles)
}

pub fn get_codex_profile_for_home(home_dir: &Path, id: &str) -> Result<CodexProfile, String> {
    load_profile_by_id(home_dir, id)
}

pub fn resolve_codex_profile_selector_for_home(
    home_dir: &Path,
    selector: &str,
) -> Result<CodexProfile, String> {
    let selector = selector.trim();
    if selector.is_empty() {
        return Err("Codex profile selector cannot be empty".to_string());
    }

    let profiles = list_codex_profiles_for_home(home_dir)?;

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
        "No Codex profile matches '{selector}'. Use `droidgear-tui run codex --list` to inspect available profiles."
    ))
}

pub fn save_codex_profile_for_home(
    home_dir: &Path,
    mut profile: CodexProfile,
) -> Result<(), String> {
    for key in profile.providers.keys() {
        if key.eq_ignore_ascii_case("openai") {
            return Err("Provider name 'OpenAI' is reserved".to_string());
        }
    }

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

/// Save a profile and, when it is the currently applied profile (recorded in
/// `active-profile.txt`), immediately apply it to `~/.codex/*` so edits take
/// effect right away. Non-active profiles are only saved; they take effect
/// when explicitly applied.
pub fn save_codex_profile_for_home_and_apply_if_active(
    home_dir: &Path,
    profile: CodexProfile,
) -> Result<(), String> {
    let profile_id = profile.id.clone();
    save_codex_profile_for_home(home_dir, profile)?;
    if get_active_codex_profile_id_for_home(home_dir)? == Some(profile_id.clone()) {
        apply_codex_profile_for_home(home_dir, &profile_id)?;
    }
    Ok(())
}

pub fn delete_codex_profile_for_home(home_dir: &Path, id: &str) -> Result<(), String> {
    let path = profile_path_for_home(home_dir, id)?;
    if path.exists() {
        std::fs::remove_file(&path).map_err(|e| format!("Failed to delete profile: {e}"))?;
    }

    if let Ok(active) = get_active_codex_profile_id_for_home(home_dir) {
        if active.as_deref() == Some(id) {
            let active_path = active_profile_path_for_home(home_dir)?;
            let _ = std::fs::remove_file(active_path);
        }
    }
    Ok(())
}

pub fn duplicate_codex_profile_for_home(
    home_dir: &Path,
    id: &str,
    new_name: &str,
) -> Result<CodexProfile, String> {
    let mut profile = load_profile_by_id(home_dir, id)?;
    profile.id = Uuid::new_v4().to_string();
    profile.name = new_name.to_string();
    profile.created_at = now_rfc3339();
    profile.updated_at = profile.created_at.clone();
    write_profile_file(home_dir, &profile)?;
    Ok(profile)
}

pub fn create_default_codex_profile_for_home(home_dir: &Path) -> Result<CodexProfile, String> {
    let profiles = list_codex_profiles_for_home(home_dir)?;
    if !profiles.is_empty() {
        return Err("Profiles already exist".to_string());
    }

    let id = Uuid::new_v4().to_string();
    let now = now_rfc3339();

    let mut providers = HashMap::new();
    providers.insert(
        "custom".to_string(),
        CodexProviderConfig {
            name: Some("Custom Provider".to_string()),
            base_url: None,
            wire_api: Some("responses".to_string()),
            requires_openai_auth: Some(true),
            env_key: None,
            env_key_instructions: None,
            http_headers: None,
            query_params: None,
            model: Some("gpt-5.2".to_string()),
            model_reasoning_effort: Some("high".to_string()),
            api_key: Some(String::new()),
        },
    );

    let profile = CodexProfile {
        id,
        name: "默认".to_string(),
        description: None,
        created_at: now.clone(),
        updated_at: now,
        providers,
        model_provider: "custom".to_string(),
        model: "gpt-5.2".to_string(),
        model_reasoning_effort: Some("high".to_string()),
        api_key: Some(String::new()),
        auth_profile_name: None,
    };

    write_profile_file(home_dir, &profile)?;
    Ok(profile)
}

// ============================================================================
// Active profile
// ============================================================================

pub fn get_active_codex_profile_id_for_home(home_dir: &Path) -> Result<Option<String>, String> {
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

// ============================================================================
// Apply + status
// ============================================================================

fn uses_openai_subscription_auth(profile: &CodexProfile) -> bool {
    profile.model_provider == "openai"
}

fn write_auth_or_delete_if_empty(
    auth_path: &Path,
    auth: &HashMap<String, Value>,
) -> Result<(), String> {
    if auth.is_empty() {
        if auth_path.exists() {
            std::fs::remove_file(auth_path)
                .map_err(|e| format!("Failed to delete auth.json: {e}"))?;
        }
        return Ok(());
    }
    json::write_json_object_file(auth_path, auth)
}

/// Apply a CodexProfile's auth to auth.json.
///
/// When `model_provider == "openai"`, treat as official subscription mode:
/// never write OPENAI_API_KEY; if no non-key fields remain, delete auth.json
/// (empty `{}` is not accepted by Codex). Otherwise merge/replace BYOK key.
pub(crate) fn apply_auth_for_profile(
    home_dir: &Path,
    profile: &CodexProfile,
    resolved_api_key: Option<&str>,
) -> Result<(), String> {
    let auth_path = codex_auth_path_for_home(home_dir)?;
    let current_auth = json::read_json_object_file(&auth_path).unwrap_or_default();

    if uses_openai_subscription_auth(profile) {
        let mut auth = current_auth;
        auth.remove(OPENAI_API_KEY_FIELD);
        write_auth_or_delete_if_empty(&auth_path, &auth)?;
        // Auth no longer matches a BYOK saved profile marker.
        let _ = crate::codex_auth_profiles::clear_active_for_home(home_dir);
        return Ok(());
    }

    // BYOK mode (custom model_provider)
    let current_is_official = current_auth.contains_key("auth_mode");

    if current_is_official {
        // Full replacement: official OAuth and BYOK are incompatible
        let mut auth: HashMap<String, Value> = HashMap::new();
        if let Some(key) = resolved_api_key {
            if !key.is_empty() {
                auth.insert(
                    OPENAI_API_KEY_FIELD.to_string(),
                    Value::String(key.to_string()),
                );
            }
        }
        write_auth_or_delete_if_empty(&auth_path, &auth)?;
        let _ = crate::codex_auth_profiles::clear_active_for_home(home_dir);
    } else {
        // Same mode: merge only OPENAI_API_KEY (preserve other fields)
        let mut auth = current_auth;
        apply_api_key_to_auth_map(&mut auth, resolved_api_key);
        write_auth_or_delete_if_empty(&auth_path, &auth)?;
    }
    Ok(())
}

/// Apply a CodexProfile to config.toml only (no auth.json changes).
/// Used after an auth profile switch to restore the associated provider config
/// without overwriting the auth.json that was just restored.
pub fn apply_codex_profile_config_only_for_home(home_dir: &Path, id: &str) -> Result<(), String> {
    let profile = load_profile_by_id(home_dir, id)?;
    let (_, active_provider) = resolve_active_provider(&profile);
    let resolved_model = resolved_model(&profile, active_provider);

    let config_path = codex_config_path_for_home(home_dir)?;
    let mut config = if config_path.exists() {
        let s = std::fs::read_to_string(&config_path)
            .map_err(|e| format!("Failed to read config.toml: {e}"))?;
        if s.trim().is_empty() {
            toml::map::Map::new()
        } else {
            toml::from_str::<toml::map::Map<String, toml::Value>>(&s)
                .map_err(|e| format!("Failed to parse config.toml: {e}"))?
        }
    } else {
        toml::map::Map::new()
    };

    apply_profile_to_config_map(&mut config, &profile, home_dir)?;

    let toml_str = toml::to_string_pretty(&config)
        .map_err(|e| format!("Failed to serialize config.toml: {e}"))?;
    storage::atomic_write(&config_path, toml_str.as_bytes())?;

    sync_models_json_for_home(home_dir, &resolved_model)?;

    set_active_profile_id_for_home(home_dir, id)?;
    Ok(())
}

/// 应用指定 Profile 到 `~/.codex/*`
///
/// 只替换 config.toml 中的模型相关配置（model_provider, model, model_reasoning_effort,
/// [model_providers]），保留其他所有配置（projects, network_access 等）。
/// Auth.json is updated based on model_provider (openai subscription vs BYOK).
pub fn apply_codex_profile_for_home(home_dir: &Path, id: &str) -> Result<(), String> {
    let profile = load_profile_by_id(home_dir, id)?;
    let (_, active_provider) = resolve_active_provider(&profile);
    let resolved_model = resolved_model(&profile, active_provider);
    let resolved_api_key = if uses_openai_subscription_auth(&profile) {
        None
    } else {
        resolved_api_key(&profile, active_provider)
    };

    let config_path = codex_config_path_for_home(home_dir)?;
    let mut config = if config_path.exists() {
        let s = std::fs::read_to_string(&config_path)
            .map_err(|e| format!("Failed to read config.toml: {e}"))?;
        if s.trim().is_empty() {
            toml::map::Map::new()
        } else {
            toml::from_str::<toml::map::Map<String, toml::Value>>(&s)
                .map_err(|e| format!("Failed to parse config.toml: {e}"))?
        }
    } else {
        toml::map::Map::new()
    };

    apply_profile_to_config_map(&mut config, &profile, home_dir)?;

    let toml_str = toml::to_string_pretty(&config)
        .map_err(|e| format!("Failed to serialize config.toml: {e}"))?;
    storage::atomic_write(&config_path, toml_str.as_bytes())?;

    sync_models_json_for_home(home_dir, &resolved_model)?;

    if uses_openai_subscription_auth(&profile) {
        if let Some(auth_name) = profile
            .auth_profile_name
            .as_deref()
            .map(str::trim)
            .filter(|name| !name.is_empty())
        {
            crate::codex_auth_profiles::restore_auth_file_for_home(home_dir, auth_name)?;
            // Official subscription should not keep a residual API key.
            let auth_path = codex_auth_path_for_home(home_dir)?;
            let mut auth = json::read_json_object_file(&auth_path).unwrap_or_default();
            auth.remove(OPENAI_API_KEY_FIELD);
            write_auth_or_delete_if_empty(&auth_path, &auth)?;
        } else {
            apply_auth_for_profile(home_dir, &profile, None)?;
        }
    } else {
        apply_auth_for_profile(home_dir, &profile, resolved_api_key.as_deref())?;
    }

    set_active_profile_id_for_home(home_dir, id)?;
    Ok(())
}

pub fn get_codex_config_status_for_home(home_dir: &Path) -> Result<CodexConfigStatus, String> {
    let auth_path = codex_auth_path_for_home(home_dir)?;
    let config_path = codex_config_path_for_home(home_dir)?;
    Ok(CodexConfigStatus {
        auth_exists: auth_path.exists(),
        config_exists: config_path.exists(),
        auth_path: auth_path.to_string_lossy().to_string(),
        config_path: config_path.to_string_lossy().to_string(),
    })
}

pub fn read_codex_current_config_for_home(home_dir: &Path) -> Result<CodexCurrentConfig, String> {
    let config_path = codex_config_path_for_home(home_dir)?;
    let auth_path = codex_auth_path_for_home(home_dir)?;

    let (providers, model_provider, model, model_reasoning_effort) = if config_path.exists() {
        let s = std::fs::read_to_string(&config_path)
            .map_err(|e| format!("Failed to read config.toml: {e}"))?;
        if s.trim().is_empty() {
            (HashMap::new(), "openai".to_string(), String::new(), None)
        } else {
            let config: toml::map::Map<String, toml::Value> =
                toml::from_str(&s).map_err(|e| format!("Failed to parse config.toml: {e}"))?;

            let providers = config
                .get("model_providers")
                .and_then(|v| v.as_table())
                .map(|table| {
                    table
                        .iter()
                        .filter_map(|(k, v)| {
                            toml_to_provider_config(v).ok().map(|c| (k.clone(), c))
                        })
                        .collect::<HashMap<_, _>>()
                })
                .unwrap_or_default();

            let model_provider = config
                .get("model_provider")
                .and_then(|v| v.as_str())
                .unwrap_or("openai")
                .to_string();

            let model = config
                .get("model")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();

            let model_reasoning_effort = config
                .get("model_reasoning_effort")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string());

            (providers, model_provider, model, model_reasoning_effort)
        }
    } else {
        (HashMap::new(), "openai".to_string(), String::new(), None)
    };

    let mut providers = providers;
    if let Some(provider) = providers.get_mut(&model_provider) {
        if provider.model.is_none() {
            provider.model = Some(model.clone());
        }
        if provider.model_reasoning_effort.is_none() {
            provider.model_reasoning_effort = model_reasoning_effort.clone();
        }
    }

    let api_key = if auth_path.exists() {
        let auth = json::read_json_object_file(&auth_path)?;
        auth.get("OPENAI_API_KEY")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string())
    } else {
        None
    };

    if let Some(provider) = providers.get_mut(&model_provider) {
        if provider.api_key.is_none() {
            provider.api_key = api_key.clone();
        }
    }

    Ok(CodexCurrentConfig {
        providers,
        model_provider,
        model,
        model_reasoning_effort,
        api_key,
    })
}

// ============================================================================
// System wrappers (use system home dir)
// ============================================================================

fn system_home_dir() -> Result<PathBuf, String> {
    dirs::home_dir().ok_or_else(|| "Failed to get home directory".to_string())
}

pub fn list_codex_profiles() -> Result<Vec<CodexProfile>, String> {
    list_codex_profiles_for_home(&system_home_dir()?)
}

pub fn get_codex_profile(id: &str) -> Result<CodexProfile, String> {
    get_codex_profile_for_home(&system_home_dir()?, id)
}

pub fn resolve_codex_profile_selector(selector: &str) -> Result<CodexProfile, String> {
    resolve_codex_profile_selector_for_home(&system_home_dir()?, selector)
}

pub fn save_codex_profile(profile: CodexProfile) -> Result<(), String> {
    save_codex_profile_for_home(&system_home_dir()?, profile)
}

pub fn delete_codex_profile(id: &str) -> Result<(), String> {
    delete_codex_profile_for_home(&system_home_dir()?, id)
}

pub fn duplicate_codex_profile(id: &str, new_name: &str) -> Result<CodexProfile, String> {
    duplicate_codex_profile_for_home(&system_home_dir()?, id, new_name)
}

pub fn create_default_codex_profile() -> Result<CodexProfile, String> {
    create_default_codex_profile_for_home(&system_home_dir()?)
}

pub fn get_active_codex_profile_id() -> Result<Option<String>, String> {
    get_active_codex_profile_id_for_home(&system_home_dir()?)
}

pub fn apply_codex_profile(id: &str) -> Result<(), String> {
    apply_codex_profile_for_home(&system_home_dir()?, id)
}

pub fn apply_codex_profile_config_only(id: &str) -> Result<(), String> {
    apply_codex_profile_config_only_for_home(&system_home_dir()?, id)
}

pub fn get_codex_config_status() -> Result<CodexConfigStatus, String> {
    get_codex_config_status_for_home(&system_home_dir()?)
}

pub fn read_codex_current_config() -> Result<CodexCurrentConfig, String> {
    read_codex_current_config_for_home(&system_home_dir()?)
}

#[cfg(test)]
mod tests {
    use super::{
        apply_codex_profile_for_home, apply_profile_to_config_map, catalog_for_model,
        provider_config_to_toml, resolve_active_provider, resolve_codex_profile_selector_for_home,
        save_codex_profile_for_home, save_codex_profile_for_home_and_apply_if_active,
        sync_models_json_for_home, CodexProfile, CodexProviderConfig, ModelCatalog,
    };
    use std::collections::HashMap;
    use tempfile::TempDir;

    fn sample_profile(id: &str, name: &str) -> CodexProfile {
        CodexProfile {
            id: id.to_string(),
            name: name.to_string(),
            description: None,
            created_at: String::new(),
            updated_at: String::new(),
            providers: HashMap::new(),
            model_provider: "openai".to_string(),
            model: "gpt-5".to_string(),
            model_reasoning_effort: None,
            api_key: None,
            auth_profile_name: None,
        }
    }

    #[test]
    fn resolve_codex_profile_selector_for_home_accepts_id_name_and_index() {
        let temp = TempDir::new().unwrap();
        save_codex_profile_for_home(temp.path(), sample_profile("profile-a", "Alpha")).unwrap();
        save_codex_profile_for_home(temp.path(), sample_profile("profile-b", "Second Profile"))
            .unwrap();

        let by_id = resolve_codex_profile_selector_for_home(temp.path(), "profile-a").unwrap();
        let by_name =
            resolve_codex_profile_selector_for_home(temp.path(), "second profile").unwrap();
        let by_index = resolve_codex_profile_selector_for_home(temp.path(), "2").unwrap();

        assert_eq!(by_id.id, "profile-a");
        assert_eq!(by_name.id, "profile-b");
        assert_eq!(by_index.id, "profile-b");
    }

    #[test]
    fn resolve_codex_profile_selector_for_home_rejects_ambiguous_names() {
        let temp = TempDir::new().unwrap();
        save_codex_profile_for_home(temp.path(), sample_profile("profile-a", "Shared")).unwrap();
        save_codex_profile_for_home(temp.path(), sample_profile("profile-b", "Shared")).unwrap();

        let error = resolve_codex_profile_selector_for_home(temp.path(), "Shared").unwrap_err();

        assert!(error.contains("Multiple Codex profiles share the name 'Shared'"));
    }

    #[test]
    fn resolve_active_provider_keeps_openai_even_with_custom_providers() {
        let mut providers = HashMap::new();
        providers.insert(
            "custom".to_string(),
            CodexProviderConfig {
                name: Some("Custom".to_string()),
                base_url: Some("https://example.com".to_string()),
                wire_api: Some("responses".to_string()),
                requires_openai_auth: Some(true),
                env_key: None,
                env_key_instructions: None,
                http_headers: None,
                query_params: None,
                model: Some("gpt-custom".to_string()),
                model_reasoning_effort: None,
                api_key: None,
            },
        );
        let profile = CodexProfile {
            id: "p1".to_string(),
            name: "P1".to_string(),
            description: None,
            created_at: String::new(),
            updated_at: String::new(),
            providers,
            model_provider: "openai".to_string(),
            model: "gpt-5".to_string(),
            model_reasoning_effort: None,
            api_key: None,
            auth_profile_name: None,
        };

        let (id, config) = resolve_active_provider(&profile);
        assert_eq!(id, "openai");
        assert!(config.is_none());
    }

    #[test]
    fn apply_profile_to_config_map_skips_model_providers_for_openai() {
        let mut providers = HashMap::new();
        providers.insert(
            "custom".to_string(),
            CodexProviderConfig {
                name: Some("Custom".to_string()),
                base_url: Some("https://example.com".to_string()),
                wire_api: Some("responses".to_string()),
                requires_openai_auth: Some(true),
                env_key: None,
                env_key_instructions: None,
                http_headers: None,
                query_params: None,
                model: Some("gpt-custom".to_string()),
                model_reasoning_effort: None,
                api_key: None,
            },
        );
        let profile = CodexProfile {
            id: "p1".to_string(),
            name: "P1".to_string(),
            description: None,
            created_at: String::new(),
            updated_at: String::new(),
            providers,
            model_provider: "openai".to_string(),
            model: "gpt-5.4".to_string(),
            model_reasoning_effort: Some("high".to_string()),
            api_key: None,
            auth_profile_name: None,
        };

        let temp = TempDir::new().unwrap();
        let mut config = toml::map::Map::new();
        config.insert(
            "model_providers".to_string(),
            toml::Value::Table(toml::map::Map::new()),
        );
        apply_profile_to_config_map(&mut config, &profile, temp.path()).unwrap();

        assert_eq!(
            config.get("model_provider").and_then(|v| v.as_str()),
            Some("openai")
        );
        assert_eq!(
            config.get("model").and_then(|v| v.as_str()),
            Some("gpt-5.4")
        );
        assert!(!config.contains_key("model_providers"));
    }

    #[test]
    fn provider_config_to_toml_defaults_empty_name_to_provider_id() {
        // Codex rejects providers whose name is empty; a missing or blank
        // display name must fall back to the provider id.
        for name in [None, Some("".to_string()), Some("   ".to_string())] {
            let config = CodexProviderConfig {
                name,
                base_url: Some("https://example.com".to_string()),
                wire_api: None,
                requires_openai_auth: None,
                env_key: None,
                env_key_instructions: None,
                http_headers: None,
                query_params: None,
                model: None,
                model_reasoning_effort: None,
                api_key: None,
            };
            let table = provider_config_to_toml("custom", &config).unwrap();
            assert_eq!(
                table.get("name").and_then(|v| v.as_str()),
                Some("custom"),
                "name should fall back to the provider id"
            );
        }
    }

    #[test]
    fn provider_config_to_toml_keeps_non_empty_name() {
        let config = CodexProviderConfig {
            name: Some("My Provider".to_string()),
            base_url: Some("https://example.com".to_string()),
            wire_api: None,
            requires_openai_auth: None,
            env_key: None,
            env_key_instructions: None,
            http_headers: None,
            query_params: None,
            model: None,
            model_reasoning_effort: None,
            api_key: None,
        };
        let table = provider_config_to_toml("custom", &config).unwrap();
        assert_eq!(
            table.get("name").and_then(|v| v.as_str()),
            Some("My Provider")
        );
    }

    #[test]
    fn catalog_for_model_matches_deepseek_and_mimo_families() {
        assert_eq!(
            catalog_for_model("deepseek-v4-flash"),
            Some(ModelCatalog::DeepSeek)
        );
        assert_eq!(
            catalog_for_model("deepseek-v4-pro"),
            Some(ModelCatalog::DeepSeek)
        );
        assert_eq!(
            catalog_for_model("  deepseek-v4-flash  "),
            Some(ModelCatalog::DeepSeek)
        );
        assert_eq!(catalog_for_model("mimo-v2.5-pro"), Some(ModelCatalog::Mimo));
        assert_eq!(catalog_for_model("mimo-v2.5"), Some(ModelCatalog::Mimo));
        assert_eq!(catalog_for_model("  mimo-v2.5  "), Some(ModelCatalog::Mimo));
        assert_eq!(catalog_for_model("gpt-5"), None);
        assert_eq!(catalog_for_model("deepseek-chat"), None);
        assert_eq!(catalog_for_model(""), None);
    }

    #[test]
    fn sync_models_json_writes_per_family_catalog_and_cleans_legacy() {
        let temp = TempDir::new().unwrap();
        let home = temp.path();

        // DeepSeek V4 writes the deepseek family catalog under model-catalogs/.
        sync_models_json_for_home(home, "deepseek-v4-flash").unwrap();
        let deepseek_path = home
            .join(".codex")
            .join("model-catalogs")
            .join("deepseek.json");
        assert!(deepseek_path.exists());
        let content = std::fs::read_to_string(&deepseek_path).unwrap();
        assert!(content.contains("deepseek-v4-flash"));
        assert!(content.contains("deepseek-v4-pro"));
        assert!(!home
            .join(".codex")
            .join("model-catalogs")
            .join("mimo.json")
            .exists());

        // MiMo writes its own family catalog; the deepseek one stays put.
        sync_models_json_for_home(home, "mimo-v2.5-pro").unwrap();
        let mimo_path = home.join(".codex").join("model-catalogs").join("mimo.json");
        assert!(mimo_path.exists());
        let mimo_content = std::fs::read_to_string(&mimo_path).unwrap();
        assert!(mimo_content.contains("mimo-v2.5-pro"));
        assert!(mimo_content.contains("mimo-v2.5"));
        assert!(deepseek_path.exists(), "other family catalogs are kept");

        // Non-catalog models do not touch family catalogs.
        sync_models_json_for_home(home, "gpt-5").unwrap();
        assert!(deepseek_path.exists());
        assert!(mimo_path.exists());
    }

    #[test]
    fn sync_models_json_removes_legacy_single_file_catalog() {
        let temp = TempDir::new().unwrap();
        let home = temp.path();

        // Simulate a catalog written by an older release at ~/.codex/models.json.
        let legacy_path = home.join(".codex").join("models.json");
        std::fs::create_dir_all(legacy_path.parent().unwrap()).unwrap();
        std::fs::write(&legacy_path, b"{\"models\":[]}").unwrap();

        sync_models_json_for_home(home, "deepseek-v4-flash").unwrap();
        assert!(
            !legacy_path.exists(),
            "legacy models.json must be cleaned up"
        );
        assert!(home
            .join(".codex")
            .join("model-catalogs")
            .join("deepseek.json")
            .exists());
    }

    fn sample_profile_with_model(model: &str) -> CodexProfile {
        let mut providers = HashMap::new();
        providers.insert(
            "dsv4".to_string(),
            CodexProviderConfig {
                name: Some("dsv4".to_string()),
                base_url: Some("https://api.deepseek.com/".to_string()),
                wire_api: Some("responses".to_string()),
                requires_openai_auth: None,
                env_key: None,
                env_key_instructions: None,
                http_headers: None,
                query_params: None,
                model: Some(model.to_string()),
                model_reasoning_effort: None,
                api_key: None,
            },
        );
        CodexProfile {
            id: "p1".to_string(),
            name: "P1".to_string(),
            description: None,
            created_at: String::new(),
            updated_at: String::new(),
            providers,
            model_provider: "dsv4".to_string(),
            model: "gpt-5".to_string(),
            model_reasoning_effort: None,
            api_key: None,
            auth_profile_name: None,
        }
    }

    #[test]
    fn apply_profile_to_config_map_sets_catalog_per_family() {
        let temp = TempDir::new().unwrap();
        let home = temp.path();

        let mut config = toml::map::Map::new();
        apply_profile_to_config_map(
            &mut config,
            &sample_profile_with_model("deepseek-v4-flash"),
            home,
        )
        .unwrap();
        assert_eq!(
            config.get("model_catalog_json").and_then(|v| v.as_str()),
            Some("~/.codex/model-catalogs/deepseek.json")
        );

        let mut config = toml::map::Map::new();
        apply_profile_to_config_map(&mut config, &sample_profile_with_model("mimo-v2.5"), home)
            .unwrap();
        assert_eq!(
            config.get("model_catalog_json").and_then(|v| v.as_str()),
            Some("~/.codex/model-catalogs/mimo.json")
        );

        let mut config = toml::map::Map::new();
        config.insert(
            "model_catalog_json".to_string(),
            toml::Value::String("~/.codex/model-catalogs/deepseek.json".to_string()),
        );
        apply_profile_to_config_map(&mut config, &sample_profile_with_model("gpt-5"), home)
            .unwrap();
        assert!(
            !config.contains_key("model_catalog_json"),
            "models without a catalog must not reference one"
        );
    }
    #[test]
    fn apply_profile_to_config_map_disables_web_search_for_mimo() {
        let temp = TempDir::new().unwrap();
        let home = temp.path();

        // MiMo models should have web_search disabled
        let mut config = toml::map::Map::new();
        apply_profile_to_config_map(&mut config, &sample_profile_with_model("mimo-v2.5"), home)
            .unwrap();
        assert_eq!(
            config.get("web_search").and_then(|v| v.as_str()),
            Some("disabled"),
            "MiMo models must have web_search disabled"
        );

        // DeepSeek models should not have web_search
        let mut config = toml::map::Map::new();
        apply_profile_to_config_map(
            &mut config,
            &sample_profile_with_model("deepseek-v4-flash"),
            home,
        )
        .unwrap();
        assert!(
            !config.contains_key("web_search"),
            "DeepSeek models must not have web_search setting"
        );

        // Non-catalog models should not have web_search
        let mut config = toml::map::Map::new();
        apply_profile_to_config_map(&mut config, &sample_profile_with_model("gpt-5"), home)
            .unwrap();
        assert!(
            !config.contains_key("web_search"),
            "Non-catalog models must not have web_search setting"
        );
    }

    #[test]
    fn save_active_profile_applies_immediately() {
        let temp = TempDir::new().unwrap();
        let home = temp.path();

        // Create a profile and apply it so it becomes the active one.
        let profile = sample_profile_with_model("deepseek-v4-flash");
        save_codex_profile_for_home(home, profile.clone()).unwrap();
        apply_codex_profile_for_home(home, &profile.id).unwrap();

        // Mutate the active profile (provider model — resolved_model prefers
        // the provider's model over the profile-level one) and save via the
        // new helper.
        let mut updated = profile;
        if let Some(provider) = updated.providers.get_mut("dsv4") {
            provider.model = Some("gpt-5".to_string());
        }
        save_codex_profile_for_home_and_apply_if_active(home, updated.clone()).unwrap();

        // The live config.toml must reflect the change immediately.
        let config_path = home.join(".codex").join("config.toml");
        let config = std::fs::read_to_string(&config_path).unwrap();
        assert!(
            config.contains("model = \"gpt-5\""),
            "active profile edits should be applied right away"
        );

        // Saving a non-active profile must not touch config.toml.
        let other = sample_profile_with_model("deepseek-v4-pro");
        let mut other_updated = other;
        other_updated.id = "other".to_string();
        other_updated.model = "gpt-5.2".to_string();
        save_codex_profile_for_home(home, other_updated.clone()).unwrap();
        save_codex_profile_for_home_and_apply_if_active(home, other_updated).unwrap();
        let config = std::fs::read_to_string(&config_path).unwrap();
        assert!(
            !config.contains("gpt-5.2"),
            "non-active profile saves must not rewrite config.toml"
        );
    }
}
