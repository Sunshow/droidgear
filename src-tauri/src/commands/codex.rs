//! Codex CLI 配置管理命令。
//!
//! 提供 Profile CRUD，并支持将 Profile 应用到 `~/.codex/auth.json` 与 `~/.codex/config.toml`。
//! 采用结构化 Provider 管理，支持多 Provider 配置，apply 时只替换模型相关配置。

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

fn get_home_dir() -> Result<PathBuf, String> {
    dirs::home_dir().ok_or("Failed to get home directory".to_string())
}

fn get_droidgear_codex_dir() -> Result<PathBuf, String> {
    Ok(get_home_dir()?.join(".droidgear").join("codex"))
}

/// `~/.droidgear/codex/profiles/`
fn get_profiles_dir() -> Result<PathBuf, String> {
    let dir = get_droidgear_codex_dir()?.join("profiles");
    if !dir.exists() {
        std::fs::create_dir_all(&dir)
            .map_err(|e| format!("Failed to create codex profiles directory: {e}"))?;
    }
    Ok(dir)
}

/// `~/.droidgear/codex/active-profile.txt`
fn get_active_profile_path() -> Result<PathBuf, String> {
    let dir = get_droidgear_codex_dir()?;
    if !dir.exists() {
        std::fs::create_dir_all(&dir)
            .map_err(|e| format!("Failed to create codex directory: {e}"))?;
    }
    Ok(dir.join("active-profile.txt"))
}

/// `~/.codex/` (or custom path)
fn get_codex_config_dir() -> Result<PathBuf, String> {
    let dir = paths::get_codex_home()?;
    if !dir.exists() {
        std::fs::create_dir_all(&dir)
            .map_err(|e| format!("Failed to create codex config directory: {e}"))?;
    }
    Ok(dir)
}

fn get_codex_auth_path() -> Result<PathBuf, String> {
    Ok(get_codex_config_dir()?.join("auth.json"))
}

fn get_codex_config_path() -> Result<PathBuf, String> {
    Ok(get_codex_config_dir()?.join("config.toml"))
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

fn read_json_object_file(path: &Path) -> Result<HashMap<String, Value>, String> {
    if !path.exists() {
        return Ok(HashMap::new());
    }
    let s = std::fs::read_to_string(path).map_err(|e| format!("Failed to read file: {e}"))?;
    if s.trim().is_empty() {
        return Ok(HashMap::new());
    }
    let v: Value = serde_json::from_str(&s).map_err(|e| format!("Invalid JSON: {e}"))?;
    match v {
        Value::Object(map) => Ok(map.into_iter().collect()),
        _ => Err("Invalid JSON: expected object".to_string()),
    }
}

fn write_json_object_file(path: &Path, obj: &HashMap<String, Value>) -> Result<(), String> {
    let v = Value::Object(obj.clone().into_iter().collect());
    let s =
        serde_json::to_string_pretty(&v).map_err(|e| format!("Failed to serialize JSON: {e}"))?;
    atomic_write(path, s.as_bytes())
}

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
                .collect()
        });

    let query_params = table
        .get("query_params")
        .and_then(|v| v.as_table())
        .map(|t| {
            t.iter()
                .filter_map(|(k, v)| v.as_str().map(|s| (k.clone(), s.to_string())))
                .collect()
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
// Profile Helpers
// ============================================================================

fn now_rfc3339() -> String {
    Utc::now().to_rfc3339()
}

fn read_profile_file(path: &Path) -> Result<CodexProfile, String> {
    let s = std::fs::read_to_string(path).map_err(|e| format!("Failed to read profile: {e}"))?;
    serde_json::from_str::<CodexProfile>(&s).map_err(|e| format!("Invalid profile JSON: {e}"))
}

fn write_profile_file(profile: &CodexProfile) -> Result<(), String> {
    let path = get_profile_path(&profile.id)?;
    let s = serde_json::to_string_pretty(profile)
        .map_err(|e| format!("Failed to serialize profile JSON: {e}"))?;
    atomic_write(&path, s.as_bytes())
}

fn load_profile_by_id(id: &str) -> Result<CodexProfile, String> {
    let path = get_profile_path(id)?;
    read_profile_file(&path)
}

// ============================================================================
// Tauri Commands
// ============================================================================

/// 列出所有 Codex Profiles
#[tauri::command]
#[specta::specta]
pub async fn list_codex_profiles() -> Result<Vec<CodexProfile>, String> {
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

/// 获取指定 Profile
#[tauri::command]
#[specta::specta]
pub async fn get_codex_profile(id: String) -> Result<CodexProfile, String> {
    load_profile_by_id(&id)
}

/// 保存 Profile（新建或更新）
#[tauri::command]
#[specta::specta]
pub async fn save_codex_profile(mut profile: CodexProfile) -> Result<(), String> {
    if profile.id.trim().is_empty() {
        profile.id = Uuid::new_v4().to_string();
        profile.created_at = now_rfc3339();
    } else if get_profile_path(&profile.id)?.exists() {
        // 保留 created_at
        if let Ok(old) = load_profile_by_id(&profile.id) {
            profile.created_at = old.created_at;
        }
    } else if profile.created_at.trim().is_empty() {
        profile.created_at = now_rfc3339();
    }

    profile.updated_at = now_rfc3339();
    write_profile_file(&profile)
}

/// 删除 Profile
#[tauri::command]
#[specta::specta]
pub async fn delete_codex_profile(id: String) -> Result<(), String> {
    let path = get_profile_path(&id)?;
    if path.exists() {
        std::fs::remove_file(&path).map_err(|e| format!("Failed to delete profile: {e}"))?;
    }

    // 如果删除的是 active profile，则清空
    if let Ok(active) = get_active_profile_id_internal() {
        if active.as_deref() == Some(id.as_str()) {
            let active_path = get_active_profile_path()?;
            let _ = std::fs::remove_file(active_path);
        }
    }

    Ok(())
}

/// 复制 Profile
#[tauri::command]
#[specta::specta]
pub async fn duplicate_codex_profile(id: String, new_name: String) -> Result<CodexProfile, String> {
    let mut profile = load_profile_by_id(&id)?;
    profile.id = Uuid::new_v4().to_string();
    profile.name = new_name;
    profile.created_at = now_rfc3339();
    profile.updated_at = profile.created_at.clone();
    write_profile_file(&profile)?;
    Ok(profile)
}

/// 创建默认 Profile（当无 Profile 时调用）
#[tauri::command]
#[specta::specta]
pub async fn create_default_codex_profile() -> Result<CodexProfile, String> {
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

/// 读取 active Profile ID
#[tauri::command]
#[specta::specta]
pub async fn get_active_codex_profile_id() -> Result<Option<String>, String> {
    get_active_profile_id_internal()
}

fn set_active_profile_id(id: &str) -> Result<(), String> {
    let path = get_active_profile_path()?;
    atomic_write(&path, id.as_bytes())
}

/// 应用指定 Profile 到 `~/.codex/*`
///
/// 只替换 config.toml 中的模型相关配置（model_provider, model, model_reasoning_effort,
/// [model_providers]），保留其他所有配置（projects, network_access 等）。
#[tauri::command]
#[specta::specta]
pub async fn apply_codex_profile(id: String) -> Result<(), String> {
    let profile = load_profile_by_id(&id)?;

    // If model_provider points to a non-existent provider, fall back to first available
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

    // Resolve model/effort/apiKey from active provider (fallback to profile-level for compat)
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

    // 1. 读取现有 config.toml 为 toml::Table
    let config_path = get_codex_config_path()?;
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

    // 2. 替换顶层模型配置
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

    // 3. 替换整个 [model_providers] section
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

    // 4. 写回 config.toml
    let toml_str = toml::to_string_pretty(&config)
        .map_err(|e| format!("Failed to serialize config.toml: {e}"))?;
    atomic_write(&config_path, toml_str.as_bytes())?;

    // 5. 写 auth.json
    let auth_path = get_codex_auth_path()?;
    let mut auth = HashMap::new();
    if let Some(ref key) = resolved_api_key {
        if !key.is_empty() {
            auth.insert("OPENAI_API_KEY".to_string(), Value::String(key.clone()));
        }
    }
    write_json_object_file(&auth_path, &auth)?;

    // 6. 更新 active profile
    set_active_profile_id(&id)?;
    Ok(())
}

/// 获取 Codex Live 配置状态（文件是否存在及路径）
#[tauri::command]
#[specta::specta]
pub async fn get_codex_config_status() -> Result<CodexConfigStatus, String> {
    let auth_path = get_codex_auth_path()?;
    let config_path = get_codex_config_path()?;
    Ok(CodexConfigStatus {
        auth_exists: auth_path.exists(),
        config_exists: config_path.exists(),
        auth_path: auth_path.to_string_lossy().to_string(),
        config_path: config_path.to_string_lossy().to_string(),
    })
}

/// 读取当前 `~/.codex/*` 配置（解析 providers 和 model selection）
#[tauri::command]
#[specta::specta]
pub async fn read_codex_current_config() -> Result<CodexCurrentConfig, String> {
    let config_path = get_codex_config_path()?;
    let auth_path = get_codex_auth_path()?;

    // 解析 config.toml
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

    // Populate active provider's DroidGear-only fields from top-level config
    let mut providers = providers;
    if let Some(provider) = providers.get_mut(&model_provider) {
        if provider.model.is_none() {
            provider.model = Some(model.clone());
        }
        if provider.model_reasoning_effort.is_none() {
            provider.model_reasoning_effort = model_reasoning_effort.clone();
        }
    }

    // 读取 auth.json
    let api_key = if auth_path.exists() {
        let auth = read_json_object_file(&auth_path)?;
        auth.get("OPENAI_API_KEY")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string())
    } else {
        None
    };

    // Populate active provider's api_key from auth.json
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
