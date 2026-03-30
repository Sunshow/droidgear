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

const OFFICIAL_PROFILE_ID: &str = "official";
const OPENAI_API_KEY_FIELD: &str = "OPENAI_API_KEY";

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

fn is_official_profile_id(id: &str) -> bool {
    id == OFFICIAL_PROFILE_ID
}

fn has_official_auth_for_home(home_dir: &Path) -> Result<bool, String> {
    let auth_path = codex_auth_path_for_home(home_dir)?;
    let auth = json::read_json_object_file(&auth_path).unwrap_or_default();
    Ok(auth.keys().any(|k| k != OPENAI_API_KEY_FIELD))
}

fn ensure_official_profile_for_home(home_dir: &Path) -> Result<(), String> {
    if !has_official_auth_for_home(home_dir)? {
        return Ok(());
    }

    let official_path = profile_path_for_home(home_dir, OFFICIAL_PROFILE_ID)?;
    if official_path.exists() {
        return Ok(());
    }

    // Best-effort: snapshot the current live config as a starting point for the official profile.
    // The official profile intentionally never stores API keys.
    let live = read_codex_current_config_for_home(home_dir).unwrap_or(CodexCurrentConfig {
        providers: HashMap::new(),
        model_provider: "openai".to_string(),
        model: String::new(),
        model_reasoning_effort: None,
        api_key: None,
    });

    let mut providers = HashMap::new();
    // Prefer preserving any explicit openai provider config from config.toml, if present.
    if let Some(mut openai_provider) = live.providers.get("openai").cloned() {
        openai_provider.api_key = None;
        providers.insert("openai".to_string(), openai_provider);
    }

    let now = now_rfc3339();
    let profile = CodexProfile {
        id: OFFICIAL_PROFILE_ID.to_string(),
        name: "Official Login / 官方登录".to_string(),
        description: Some(
            "Uses `codex login` credentials (Apply will clear OPENAI_API_KEY and preserve other fields in auth.json) / 使用 codex login 的官方认证（应用时会清除 OPENAI_API_KEY，且会保留 auth.json 里其它字段）"
                .to_string(),
        ),
        created_at: now.clone(),
        updated_at: now,
        providers,
        model_provider: "openai".to_string(),
        model: live.model,
        model_reasoning_effort: live.model_reasoning_effort,
        api_key: None,
    };

    // Write under ~/.droidgear/codex/profiles/official.json
    write_profile_file(home_dir, &profile)
}

// ============================================================================
// TOML helpers
// ============================================================================

/// Convert CodexProviderConfig to toml::Value
fn provider_config_to_toml(config: &CodexProviderConfig) -> Result<toml::Value, String> {
    let mut table = toml::map::Map::new();

    if let Some(ref name) = config.name {
        table.insert("name".to_string(), toml::Value::String(name.clone()));
    }
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

pub fn list_codex_profiles_for_home(home_dir: &Path) -> Result<Vec<CodexProfile>, String> {
    // Auto-create a system "official" profile if the user has codex login credentials.
    // This keeps GUI/TUI in sync without extra UI logic.
    let _ = ensure_official_profile_for_home(home_dir);

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

    profiles.sort_by(|a, b| {
        match (is_official_profile_id(&a.id), is_official_profile_id(&b.id)) {
            (true, false) => std::cmp::Ordering::Less,
            (false, true) => std::cmp::Ordering::Greater,
            _ => a.name.to_lowercase().cmp(&b.name.to_lowercase()),
        }
    });
    Ok(profiles)
}

pub fn get_codex_profile_for_home(home_dir: &Path, id: &str) -> Result<CodexProfile, String> {
    load_profile_by_id(home_dir, id)
}

pub fn save_codex_profile_for_home(
    home_dir: &Path,
    mut profile: CodexProfile,
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

pub fn delete_codex_profile_for_home(home_dir: &Path, id: &str) -> Result<(), String> {
    if is_official_profile_id(id) {
        return Err("Cannot delete the official profile".to_string());
    }
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
    if profiles.iter().any(|p| !is_official_profile_id(&p.id)) {
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

/// 应用指定 Profile 到 `~/.codex/*`
///
/// 只替换 config.toml 中的模型相关配置（model_provider, model, model_reasoning_effort,
/// [model_providers]），保留其他所有配置（projects, network_access 等）。
pub fn apply_codex_profile_for_home(home_dir: &Path, id: &str) -> Result<(), String> {
    let profile = load_profile_by_id(home_dir, id)?;

    let (effective_provider_id, active_provider) =
        if profile.providers.contains_key(&profile.model_provider) {
            (
                profile.model_provider.clone(),
                profile.providers.get(&profile.model_provider),
            )
        } else if let Some((first_id, first_config)) = profile.providers.iter().next() {
            (first_id.clone(), Some(first_config))
        } else {
            (profile.model_provider.clone(), None)
        };

    let resolved_model = active_provider
        .and_then(|p| p.model.as_deref())
        .filter(|s| !s.is_empty())
        .unwrap_or(&profile.model);
    let resolved_effort = active_provider
        .and_then(|p| p.model_reasoning_effort.clone())
        .or(profile.model_reasoning_effort.clone());
    let resolved_api_key = active_provider
        .and_then(|p| p.api_key.clone())
        .or(profile.api_key.clone());

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

    config.insert(
        "model_provider".to_string(),
        toml::Value::String(effective_provider_id),
    );
    config.insert(
        "model".to_string(),
        toml::Value::String(resolved_model.to_string()),
    );
    if let Some(ref effort) = resolved_effort {
        config.insert(
            "model_reasoning_effort".to_string(),
            toml::Value::String(effort.clone()),
        );
    } else {
        config.remove("model_reasoning_effort");
    }

    config.remove("model_providers");
    if !profile.providers.is_empty() {
        let mut providers_table = toml::map::Map::new();
        for (provider_id, provider_config) in &profile.providers {
            providers_table.insert(
                provider_id.clone(),
                provider_config_to_toml(provider_config)?,
            );
        }
        config.insert(
            "model_providers".to_string(),
            toml::Value::Table(providers_table),
        );
    }

    let toml_str = toml::to_string_pretty(&config)
        .map_err(|e| format!("Failed to serialize config.toml: {e}"))?;
    storage::atomic_write(&config_path, toml_str.as_bytes())?;

    let auth_path = codex_auth_path_for_home(home_dir)?;
    let mut auth = json::read_json_object_file(&auth_path).unwrap_or_default();

    if let Some(ref key) = resolved_api_key {
        if !key.is_empty() {
            auth.insert(OPENAI_API_KEY_FIELD.to_string(), Value::String(key.clone()));
        } else {
            auth.remove(OPENAI_API_KEY_FIELD);
        }
    } else {
        auth.remove(OPENAI_API_KEY_FIELD);
    }

    json::write_json_object_file(&auth_path, &auth)?;

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

pub fn get_codex_config_status() -> Result<CodexConfigStatus, String> {
    get_codex_config_status_for_home(&system_home_dir()?)
}

pub fn read_codex_current_config() -> Result<CodexCurrentConfig, String> {
    read_codex_current_config_for_home(&system_home_dir()?)
}
