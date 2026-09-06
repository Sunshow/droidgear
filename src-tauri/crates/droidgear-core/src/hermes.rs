//! Hermes Agent 配置管理（core）。
//!
//! 负责 Profile CRUD，并支持将 Profile 应用到 `~/.hermes/config.yaml`。
//! Apply 逻辑采用读取-修改-写入模式，以保留 YAML 文件中的其他非 model 配置节。
//! 逻辑从原 Tauri command 层抽离，以便在 TUI 与桌面端复用。
//!
//! 与 Hermes 官方文档（[AI Providers](https://hermes-agent.nousresearch.com/integrations/providers)）
//! 保持一致的 config.yaml 语义：
//! - `model.default`（或 `model.model`）是 Hermes 实际使用的模型 ID；
//! - `model.provider` 选择供应商，命名自定义供应商写作 `custom:<name>`，
//!   裸自定义端点写作 `custom`（此时 `model.base_url` / `model.api_key` 生效）；
//! - `custom_providers` 是命名自定义供应商列表（name / base_url / api_key / model）。
//!
//! Profile 内的 `models` 是模型配置列表，其中 `is_default` 为 true 的条目
//! 在 apply 时写入 `model.default` + `model.provider`。

use chrono::Utc;
use serde::{Deserialize, Serialize};
use serde_yaml::Value;
use specta::Type;
use std::path::{Path, PathBuf};
use uuid::Uuid;

use crate::{paths, storage};

// ============================================================================
// Types
// ============================================================================

/// 单条 Hermes model 配置。
///
/// apply 时：
/// - `is_default` 的条目 → 写入 config.yaml 的 `model.default` / `model.provider`；
/// - 解析出 `custom:<name>` 且带 base_url 的条目 → 写入 `custom_providers` 列表；
/// - 裸 `custom`（无 name）→ base_url/api_key 直接写入 `model` 节。
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct HermesModelConfig {
    /// 配置名称（对应 `custom_providers[].name`，也是 `custom:<name>` 的引用名）。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    /// 模型 ID（写入 `model.default` 或 `custom_providers[].model`）。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default: Option<String>,
    /// 显式 provider（如 openrouter / deepseek / custom）。留空时按 name/base_url 推导。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub provider: Option<String>,
    /// OpenAI 兼容端点地址（自定义供应商使用）。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub base_url: Option<String>,
    /// API 密钥（自定义供应商使用）。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub api_key: Option<String>,
    /// 是否为默认配置（apply 时写入 `model.default` + `model.provider`）。
    #[serde(default, skip_serializing_if = "is_false")]
    pub is_default: bool,
}

fn is_false(v: &bool) -> bool {
    !*v
}

/// Hermes Profile（用于在 DroidGear 内部保存并切换）。
///
/// 旧版 Profile 只有单条 `model` 对象；反序列化时会自动迁移为单元素的
/// `models` 列表（并标记 is_default）。
#[derive(Debug, Clone, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct HermesProfile {
    pub id: String,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    pub created_at: String,
    pub updated_at: String,
    /// 模型配置列表（至少一条应标记 is_default）。
    pub models: Vec<HermesModelConfig>,
    /// 推理努力程度（对应 config.yaml 中的 agent.reasoning_effort）
    /// 选项：none, minimal, low, medium, high, xhigh, max, ultra
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reasoning_effort: Option<String>,
}

impl<'de> Deserialize<'de> for HermesProfile {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        #[derive(Deserialize)]
        #[serde(rename_all = "camelCase")]
        struct Helper {
            id: String,
            name: String,
            #[serde(default)]
            description: Option<String>,
            created_at: String,
            updated_at: String,
            /// 旧版单条 model 配置（仅用于迁移）。
            #[serde(default)]
            model: Option<HermesModelConfig>,
            #[serde(default)]
            models: Vec<HermesModelConfig>,
            #[serde(default)]
            reasoning_effort: Option<String>,
        }

        let h = Helper::deserialize(deserializer)?;

        let mut models = h.models;
        if models.is_empty() {
            if let Some(mut legacy) = h.model {
                legacy.is_default = true;
                models.push(legacy);
            }
        }
        let models = normalize_models(models);

        Ok(HermesProfile {
            id: h.id,
            name: h.name,
            description: h.description,
            created_at: h.created_at,
            updated_at: h.updated_at,
            models,
            reasoning_effort: h.reasoning_effort,
        })
    }
}

/// Hermes Live 配置状态
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct HermesConfigStatus {
    pub config_exists: bool,
    pub config_path: String,
}

/// 当前 Hermes Live 配置（从 `~/.hermes/config.yaml` 读取）
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct HermesCurrentConfig {
    /// 模型配置列表：live 的 `model` 节合并 `custom_providers` 列表；
    /// `is_default` 标记当前生效（model.default/model.provider）的条目。
    pub models: Vec<HermesModelConfig>,
    /// 推理努力程度（对应 config.yaml 中的 agent.reasoning_effort）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reasoning_effort: Option<String>,
}

// ============================================================================
// Helpers
// ============================================================================

/// Trim 后的非空字符串引用。
fn non_empty(value: &Option<String>) -> Option<&str> {
    value.as_deref().map(str::trim).filter(|s| !s.is_empty())
}

/// 保证至多一条 is_default；若无任何条目标记，则把第一条设为默认。
fn normalize_models(mut models: Vec<HermesModelConfig>) -> Vec<HermesModelConfig> {
    if models.is_empty() {
        return models;
    }
    let mut seen_default = false;
    for entry in &mut models {
        if seen_default {
            entry.is_default = false;
        } else if entry.is_default {
            seen_default = true;
        }
    }
    if !seen_default {
        if let Some(first) = models.first_mut() {
            first.is_default = true;
        }
    }
    models
}

/// 计算某条目 apply 后应写入 `model.provider` 的值。
///
/// 规则（与 Hermes 文档一致）：
/// - 显式 provider 优先；`custom` 且有 name 时升格为 `custom:<name>`；
/// - 无 provider 但有 name → `custom:<name>`；
/// - 无 provider 无 name 但有 base_url → `custom`（裸自定义端点）；
/// - 都没有 → None（不写 provider）。
fn resolve_provider(entry: &HermesModelConfig) -> Option<String> {
    let provider = non_empty(&entry.provider);
    let name = non_empty(&entry.name);
    match provider {
        Some("custom") => match name {
            Some(n) => Some(format!("custom:{n}")),
            None => Some("custom".to_string()),
        },
        Some(p) => Some(p.to_string()),
        None => match name {
            Some(n) => Some(format!("custom:{n}")),
            None => non_empty(&entry.base_url).map(|_| "custom".to_string()),
        },
    }
}

// ============================================================================
// Path Helpers
// ============================================================================

fn droidgear_hermes_dir_for_home(home_dir: &Path) -> PathBuf {
    home_dir.join(".droidgear").join("hermes")
}

/// `~/.droidgear/hermes/profiles/`
fn profiles_dir_for_home(home_dir: &Path) -> Result<PathBuf, String> {
    let dir = droidgear_hermes_dir_for_home(home_dir).join("profiles");
    if !dir.exists() {
        std::fs::create_dir_all(&dir)
            .map_err(|e| format!("Failed to create hermes profiles directory: {e}"))?;
    }
    Ok(dir)
}

/// `~/.droidgear/hermes/active-profile.txt`
fn active_profile_path_for_home(home_dir: &Path) -> Result<PathBuf, String> {
    let dir = droidgear_hermes_dir_for_home(home_dir);
    if !dir.exists() {
        std::fs::create_dir_all(&dir)
            .map_err(|e| format!("Failed to create hermes directory: {e}"))?;
    }
    Ok(dir.join("active-profile.txt"))
}

/// `~/.hermes/` (or custom path) — NOT WSL-aware; used by `_for_home` variants
/// and tests that pass a temp directory.
fn hermes_config_dir_for_home(home_dir: &Path) -> Result<PathBuf, String> {
    // Check AppData/Local/hermes first (Windows user config)
    // Only when home_dir is the system home (not a custom/test path)
    #[cfg(target_os = "windows")]
    {
        if let Some(system_home) = dirs::home_dir() {
            if home_dir == system_home {
                if let Some(local_app_data) = std::env::var_os("LOCALAPPDATA") {
                    let user_path = std::path::PathBuf::from(local_app_data).join("hermes");
                    if user_path.join("config.yaml").exists() {
                        return Ok(user_path);
                    }
                }
            }
        }
    }
    // Fallback to main config path
    let config_paths = paths::load_config_paths_for_home(home_dir);
    let dir = paths::get_hermes_home_for_home(home_dir, &config_paths)?;
    if !dir.exists() {
        std::fs::create_dir_all(&dir)
            .map_err(|e| format!("Failed to create hermes config directory: {e}"))?;
    }
    Ok(dir)
}

fn hermes_config_path_for_home(home_dir: &Path) -> Result<PathBuf, String> {
    Ok(hermes_config_dir_for_home(home_dir)?.join("config.yaml"))
}

/// WSL-aware hermes config dir — uses `paths::get_hermes_home()` which
/// resolves to the WSL path on Windows when WSL is available.
fn hermes_config_dir() -> Result<PathBuf, String> {
    let dir = paths::get_hermes_home()?;
    if !dir.exists() {
        std::fs::create_dir_all(&dir)
            .map_err(|e| format!("Failed to create hermes config directory: {e}"))?;
    }
    Ok(dir)
}

/// WSL-aware hermes config.yaml path (system wrapper).
fn hermes_config_path() -> Result<PathBuf, String> {
    Ok(hermes_config_dir()?.join("config.yaml"))
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
// CRUD (Profiles)
// ============================================================================

fn read_profile_file(path: &Path) -> Result<HermesProfile, String> {
    let s = std::fs::read_to_string(path).map_err(|e| format!("Failed to read profile: {e}"))?;
    serde_json::from_str::<HermesProfile>(&s).map_err(|e| format!("Invalid profile JSON: {e}"))
}

fn write_profile_file(home_dir: &Path, profile: &HermesProfile) -> Result<(), String> {
    let path = profile_path_for_home(home_dir, &profile.id)?;
    let s = serde_json::to_string_pretty(profile)
        .map_err(|e| format!("Failed to serialize profile JSON: {e}"))?;
    storage::atomic_write(&path, s.as_bytes())
}

fn load_profile_by_id(home_dir: &Path, id: &str) -> Result<HermesProfile, String> {
    let path = profile_path_for_home(home_dir, id)?;
    read_profile_file(&path)
}

pub fn list_hermes_profiles_for_home(home_dir: &Path) -> Result<Vec<HermesProfile>, String> {
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

pub fn get_hermes_profile_for_home(home_dir: &Path, id: &str) -> Result<HermesProfile, String> {
    load_profile_by_id(home_dir, id)
}

pub fn save_hermes_profile_for_home(
    home_dir: &Path,
    mut profile: HermesProfile,
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
    profile.models = normalize_models(profile.models);
    write_profile_file(home_dir, &profile)
}

pub fn delete_hermes_profile_for_home(home_dir: &Path, id: &str) -> Result<(), String> {
    let path = profile_path_for_home(home_dir, id)?;
    if path.exists() {
        std::fs::remove_file(&path).map_err(|e| format!("Failed to delete profile: {e}"))?;
    }

    if let Ok(active) = get_active_hermes_profile_id_for_home(home_dir) {
        if active.as_deref() == Some(id) {
            let active_path = active_profile_path_for_home(home_dir)?;
            let _ = std::fs::remove_file(active_path);
        }
    }
    Ok(())
}

pub fn duplicate_hermes_profile_for_home(
    home_dir: &Path,
    id: &str,
    new_name: &str,
) -> Result<HermesProfile, String> {
    let mut profile = load_profile_by_id(home_dir, id)?;
    profile.id = Uuid::new_v4().to_string();
    profile.name = new_name.to_string();
    profile.created_at = now_rfc3339();
    profile.updated_at = profile.created_at.clone();
    write_profile_file(home_dir, &profile)?;
    Ok(profile)
}

pub fn create_default_hermes_profile_for_home(home_dir: &Path) -> Result<HermesProfile, String> {
    let profiles = list_hermes_profiles_for_home(home_dir)?;
    if !profiles.is_empty() {
        return Err("Profiles already exist".to_string());
    }

    let id = Uuid::new_v4().to_string();
    let now = now_rfc3339();

    let profile = HermesProfile {
        id,
        name: "默认".to_string(),
        description: None,
        created_at: now.clone(),
        updated_at: now,
        models: vec![HermesModelConfig {
            name: None,
            default: None,
            provider: None,
            base_url: None,
            api_key: None,
            is_default: true,
        }],
        reasoning_effort: None,
    };

    write_profile_file(home_dir, &profile)?;
    Ok(profile)
}

// ============================================================================
// Active profile
// ============================================================================

pub fn get_active_hermes_profile_id_for_home(home_dir: &Path) -> Result<Option<String>, String> {
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

/// 将一条命名自定义供应商条目 upsert 进 `custom_providers` 列表。
/// 按 name 优先、base_url 其次匹配已有条目；未设置的字段保留原值。
fn upsert_custom_provider(
    providers_list: &mut Vec<Value>,
    name: &str,
    entry: &HermesModelConfig,
) -> Result<(), String> {
    let existing_idx = providers_list.iter().position(|p| {
        p.get("name")
            .and_then(|v| v.as_str())
            .map(|n| n == name)
            .unwrap_or(false)
            || (non_empty(&entry.base_url).is_some()
                && p.get("base_url")
                    .and_then(|v| v.as_str())
                    .map(|url| url == non_empty(&entry.base_url).unwrap())
                    .unwrap_or(false))
    });

    if let Some(idx) = existing_idx {
        let existing = providers_list
            .get_mut(idx)
            .and_then(|v| v.as_mapping_mut())
            .ok_or("custom_providers entry must be a YAML mapping")?;
        existing.insert(
            Value::String("name".to_string()),
            Value::String(name.to_string()),
        );
        if let Some(base_url) = non_empty(&entry.base_url) {
            existing.insert(
                Value::String("base_url".to_string()),
                Value::String(base_url.to_string()),
            );
        }
        if let Some(api_key) = non_empty(&entry.api_key) {
            existing.insert(
                Value::String("api_key".to_string()),
                Value::String(api_key.to_string()),
            );
        }
        if let Some(model) = non_empty(&entry.default) {
            existing.insert(
                Value::String("model".to_string()),
                Value::String(model.to_string()),
            );
        }
    } else {
        let mut new_provider = serde_yaml::Mapping::new();
        new_provider.insert(
            Value::String("name".to_string()),
            Value::String(name.to_string()),
        );
        if let Some(base_url) = non_empty(&entry.base_url) {
            new_provider.insert(
                Value::String("base_url".to_string()),
                Value::String(base_url.to_string()),
            );
        }
        if let Some(api_key) = non_empty(&entry.api_key) {
            new_provider.insert(
                Value::String("api_key".to_string()),
                Value::String(api_key.to_string()),
            );
        }
        if let Some(model) = non_empty(&entry.default) {
            new_provider.insert(
                Value::String("model".to_string()),
                Value::String(model.to_string()),
            );
        }
        providers_list.push(Value::Mapping(new_provider));
    }
    Ok(())
}

/// Internal: write a profile's model config to the given config.yaml path.
///
/// 采用读取-修改-写入模式：只替换 config.yaml 中与模型相关的节
/// （`model` + `custom_providers` + `agent.reasoning_effort`），保留其他所有配置。
fn apply_profile_to_config_path(profile: &HermesProfile, config_path: &Path) -> Result<(), String> {
    // Read existing YAML as a generic Value to preserve all non-model sections.
    let mut config: Value = if config_path.exists() {
        let s = std::fs::read_to_string(config_path)
            .map_err(|e| format!("Failed to read config.yaml: {e}"))?;
        if s.trim().is_empty() {
            Value::Mapping(serde_yaml::Mapping::new())
        } else {
            serde_yaml::from_str(&s).map_err(|e| format!("Failed to parse config.yaml: {e}"))?
        }
    } else {
        Value::Mapping(serde_yaml::Mapping::new())
    };

    // Ensure root is a mapping.
    let root = config
        .as_mapping_mut()
        .ok_or("config.yaml root must be a YAML mapping")?;

    // 1. Upsert 命名自定义供应商（custom:<name>）到 custom_providers 列表。
    for entry in &profile.models {
        let Some(resolved) = resolve_provider(entry) else {
            continue;
        };
        let Some(name) = resolved.strip_prefix("custom:") else {
            continue;
        };
        if name.is_empty() || non_empty(&entry.base_url).is_none() {
            continue;
        }
        let custom_providers = root
            .entry(Value::String("custom_providers".to_string()))
            .or_insert_with(|| Value::Sequence(Vec::new()));
        let providers_list = custom_providers
            .as_sequence_mut()
            .ok_or("custom_providers must be a YAML sequence")?;
        upsert_custom_provider(providers_list, name, entry)?;
    }

    // 2. 写入 model 节：default + provider（Hermes 实际生效的配置）。
    let default_entry = profile
        .models
        .iter()
        .find(|m| m.is_default)
        .or_else(|| profile.models.first());
    if let Some(default_entry) = default_entry {
        let resolved_provider = resolve_provider(default_entry);
        let has_model = non_empty(&default_entry.default).is_some();
        let has_provider = resolved_provider.is_some();
        if has_model || has_provider {
            let model_section = root
                .entry(Value::String("model".to_string()))
                .or_insert_with(|| Value::Mapping(serde_yaml::Mapping::new()));
            let model_map = model_section
                .as_mapping_mut()
                .ok_or("model section must be a YAML mapping")?;

            if let Some(model) = non_empty(&default_entry.default) {
                model_map.insert(
                    Value::String("default".to_string()),
                    Value::String(model.to_string()),
                );
            }

            if let Some(provider) = resolved_provider {
                model_map.insert(
                    Value::String("provider".to_string()),
                    Value::String(provider.clone()),
                );
                if provider == "custom" {
                    // 裸自定义端点：base_url / api_key 写在 model 节。
                    match non_empty(&default_entry.base_url) {
                        Some(url) => {
                            model_map.insert(
                                Value::String("base_url".to_string()),
                                Value::String(url.to_string()),
                            );
                        }
                        None => {
                            model_map.remove(Value::String("base_url".to_string()));
                        }
                    }
                    match non_empty(&default_entry.api_key) {
                        Some(key) => {
                            model_map.insert(
                                Value::String("api_key".to_string()),
                                Value::String(key.to_string()),
                            );
                        }
                        None => {
                            model_map.remove(Value::String("api_key".to_string()));
                        }
                    }
                } else if provider.starts_with("custom:") {
                    // 命名自定义供应商：密钥/端点由 custom_providers 提供，
                    // 清除 model 节中残留的 base_url/api_key 以免混淆。
                    model_map.remove(Value::String("base_url".to_string()));
                    model_map.remove(Value::String("api_key".to_string()));
                }
                // 其他一等供应商：保留用户已有的 model.base_url/api_key 不动。
            }
        }
    }

    // 3. Update agent.reasoning_effort if set in profile.
    if let Some(ref effort) = profile.reasoning_effort {
        let agent_section = root
            .entry(Value::String("agent".to_string()))
            .or_insert_with(|| Value::Mapping(serde_yaml::Mapping::new()));
        let agent_map = agent_section
            .as_mapping_mut()
            .ok_or("agent section must be a YAML mapping")?;
        agent_map.insert(
            Value::String("reasoning_effort".to_string()),
            Value::String(effort.clone()),
        );
    }

    let yaml_str = serde_yaml::to_string(&config)
        .map_err(|e| format!("Failed to serialize config.yaml: {e}"))?;
    storage::atomic_write(config_path, yaml_str.as_bytes())
}

/// Internal: read current Hermes config from a specific config.yaml path.
///
/// 以 live 的 `model` 节为准（`model.default` / `model.provider` 是 Hermes
/// 实际使用的值），合并 `custom_providers` 列表为模型配置列表，并标记
/// 当前生效条目为 is_default。
fn read_current_config_from_path(config_path: &Path) -> Result<HermesCurrentConfig, String> {
    let empty = || HermesCurrentConfig {
        models: Vec::new(),
        reasoning_effort: None,
    };

    if !config_path.exists() {
        return Ok(empty());
    }

    let s = std::fs::read_to_string(config_path)
        .map_err(|e| format!("Failed to read config.yaml: {e}"))?;
    if s.trim().is_empty() {
        return Ok(empty());
    }

    let parsed: Value =
        serde_yaml::from_str(&s).map_err(|e| format!("Failed to parse config.yaml: {e}"))?;

    // custom_providers → 列表条目（先不标记默认）
    let mut models: Vec<HermesModelConfig> = Vec::new();
    if let Some(seq) = parsed.get("custom_providers").and_then(|v| v.as_sequence()) {
        for item in seq.iter().filter_map(|v| v.as_mapping()) {
            let get_str = |key: &str| -> Option<String> {
                item.get(Value::String(key.to_string()))
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string())
            };
            models.push(HermesModelConfig {
                name: get_str("name"),
                default: get_str("model"),
                provider: None,
                base_url: get_str("base_url"),
                api_key: get_str("api_key"),
                is_default: false,
            });
        }
    }

    // live model 节（Hermes 实际生效的配置）
    let model_section = parsed.get("model").and_then(|v| v.as_mapping());
    let get_str = |key: &str| -> Option<String> {
        model_section
            .and_then(|m| m.get(Value::String(key.to_string())))
            .and_then(|v| v.as_str())
            .map(|s| s.to_string())
    };
    let live_default = get_str("default").or_else(|| get_str("model"));
    let live_provider = get_str("provider");
    let live_base_url = get_str("base_url");
    let live_api_key = get_str("api_key");

    match live_provider.as_deref() {
        Some(p) if p.starts_with("custom:") && p.len() > "custom:".len() => {
            let name = &p["custom:".len()..];
            if let Some(entry) = models.iter_mut().find(|m| m.name.as_deref() == Some(name)) {
                entry.is_default = true;
                if entry.default.is_none() {
                    entry.default = live_default;
                }
            } else {
                models.push(HermesModelConfig {
                    name: Some(name.to_string()),
                    default: live_default,
                    provider: None,
                    base_url: None,
                    api_key: None,
                    is_default: true,
                });
            }
        }
        Some("custom") => {
            // 裸自定义端点：base_url/api_key 在 model 节。
            models.push(HermesModelConfig {
                name: None,
                default: live_default,
                provider: Some("custom".to_string()),
                base_url: live_base_url,
                api_key: live_api_key,
                is_default: true,
            });
        }
        Some(p) => {
            // 一等供应商（openrouter / deepseek / ...）
            models.push(HermesModelConfig {
                name: None,
                default: live_default,
                provider: Some(p.to_string()),
                base_url: live_base_url,
                api_key: live_api_key,
                is_default: true,
            });
        }
        None => {
            if live_default.is_some() || live_base_url.is_some() || live_api_key.is_some() {
                models.push(HermesModelConfig {
                    name: None,
                    default: live_default,
                    provider: None,
                    base_url: live_base_url,
                    api_key: live_api_key,
                    is_default: true,
                });
            }
        }
    }

    // 有条目但没有任何默认（如只有 custom_providers、没有 model 节）时，
    // 把第一条视为默认，便于「从当前配置加载」。
    if !models.is_empty() && !models.iter().any(|m| m.is_default) {
        models[0].is_default = true;
    }

    // Read agent.reasoning_effort
    let reasoning_effort = parsed
        .get("agent")
        .and_then(|a| a.get("reasoning_effort"))
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());

    Ok(HermesCurrentConfig {
        models,
        reasoning_effort,
    })
}

/// 应用指定 Profile 到 `~/.hermes/config.yaml`（for_home variant, NOT WSL-aware）
pub fn apply_hermes_profile_for_home(home_dir: &Path, id: &str) -> Result<(), String> {
    let profile = load_profile_by_id(home_dir, id)?;
    let config_path = hermes_config_path_for_home(home_dir)?;
    apply_profile_to_config_path(&profile, &config_path)?;
    set_active_profile_id_for_home(home_dir, id)?;
    Ok(())
}

pub fn get_hermes_config_status_for_home(home_dir: &Path) -> Result<HermesConfigStatus, String> {
    let config_path = hermes_config_path_for_home(home_dir)?;
    Ok(HermesConfigStatus {
        config_exists: config_path.exists(),
        config_path: config_path.to_string_lossy().to_string(),
    })
}

pub fn read_hermes_current_config_for_home(home_dir: &Path) -> Result<HermesCurrentConfig, String> {
    let config_path = hermes_config_path_for_home(home_dir)?;
    read_current_config_from_path(&config_path)
}

// ============================================================================
// System wrappers (use system home dir)
// ============================================================================

fn system_home_dir() -> Result<PathBuf, String> {
    dirs::home_dir().ok_or_else(|| "Failed to get home directory".to_string())
}

pub fn list_hermes_profiles() -> Result<Vec<HermesProfile>, String> {
    list_hermes_profiles_for_home(&system_home_dir()?)
}

pub fn get_hermes_profile(id: &str) -> Result<HermesProfile, String> {
    get_hermes_profile_for_home(&system_home_dir()?, id)
}

pub fn save_hermes_profile(profile: HermesProfile) -> Result<(), String> {
    save_hermes_profile_for_home(&system_home_dir()?, profile)
}

pub fn delete_hermes_profile(id: &str) -> Result<(), String> {
    delete_hermes_profile_for_home(&system_home_dir()?, id)
}

pub fn duplicate_hermes_profile(id: &str, new_name: &str) -> Result<HermesProfile, String> {
    duplicate_hermes_profile_for_home(&system_home_dir()?, id, new_name)
}

pub fn create_default_hermes_profile() -> Result<HermesProfile, String> {
    create_default_hermes_profile_for_home(&system_home_dir()?)
}

pub fn get_active_hermes_profile_id() -> Result<Option<String>, String> {
    get_active_hermes_profile_id_for_home(&system_home_dir()?)
}

pub fn apply_hermes_profile(id: &str) -> Result<(), String> {
    let home = system_home_dir()?;
    let profile = load_profile_by_id(&home, id)?;
    let config_path = hermes_config_path()?;
    apply_profile_to_config_path(&profile, &config_path)?;
    set_active_profile_id_for_home(&home, id)?;
    Ok(())
}

pub fn get_hermes_config_status() -> Result<HermesConfigStatus, String> {
    let config_path = hermes_config_path()?;
    Ok(HermesConfigStatus {
        config_exists: config_path.exists(),
        config_path: config_path.to_string_lossy().to_string(),
    })
}

pub fn read_hermes_current_config() -> Result<HermesCurrentConfig, String> {
    let config_path = hermes_config_path()?;
    read_current_config_from_path(&config_path)
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

    fn write_file(path: &Path, content: &str) {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).unwrap();
        }
        std::fs::write(path, content).unwrap();
    }

    fn make_entry(name: &str, default_model: Option<&str>) -> HermesModelConfig {
        HermesModelConfig {
            name: Some(name.to_string()),
            default: default_model.map(|s| s.to_string()),
            provider: None,
            base_url: Some("https://api.openai.com/v1".to_string()),
            api_key: Some("sk-test".to_string()),
            is_default: true,
        }
    }

    fn make_profile(id: &str, name: &str, default_model: Option<&str>) -> HermesProfile {
        HermesProfile {
            id: id.to_string(),
            name: name.to_string(),
            description: None,
            created_at: "2026-01-01T00:00:00Z".to_string(),
            updated_at: "2026-01-01T00:00:00Z".to_string(),
            models: vec![make_entry("openai", default_model)],
            reasoning_effort: None,
        }
    }

    #[test]
    fn test_yaml_serialization() {
        let model = HermesModelConfig {
            name: Some("openai".to_string()),
            default: Some("gpt-4".to_string()),
            provider: Some("openai".to_string()),
            base_url: Some("https://api.openai.com/v1".to_string()),
            api_key: Some("sk-test".to_string()),
            is_default: true,
        };

        let profile = HermesProfile {
            id: "p1".to_string(),
            name: "Test".to_string(),
            description: None,
            created_at: "2026-01-01T00:00:00Z".to_string(),
            updated_at: "2026-01-01T00:00:00Z".to_string(),
            models: vec![model],
            reasoning_effort: Some("high".to_string()),
        };

        // Verify JSON serialization (profiles stored as JSON)
        let json = serde_json::to_string_pretty(&profile).unwrap();
        assert!(json.contains("\"models\":"));
        assert!(json.contains("\"default\":"));
        assert!(json.contains("\"provider\":"));
        assert!(json.contains("\"baseUrl\":"));
        assert!(json.contains("\"apiKey\":"));
        assert!(json.contains("\"isDefault\": true"));
        assert!(json.contains("\"reasoningEffort\":"));
        assert!(json.contains("gpt-4"));
        assert!(json.contains("high"));
    }

    #[test]
    fn test_yaml_deserialization() {
        let yaml = r#"
model:
  default: gpt-4
  provider: openai
  base_url: https://api.openai.com/v1
  api_key: sk-test
other_section:
  key: value
"#;
        let parsed: Value = serde_yaml::from_str(yaml).unwrap();
        let model_section = parsed.get("model").unwrap();

        assert_eq!(
            model_section.get("default").and_then(|v| v.as_str()),
            Some("gpt-4")
        );
        assert_eq!(
            model_section.get("provider").and_then(|v| v.as_str()),
            Some("openai")
        );
        assert_eq!(
            model_section.get("base_url").and_then(|v| v.as_str()),
            Some("https://api.openai.com/v1")
        );
        assert_eq!(
            model_section.get("api_key").and_then(|v| v.as_str()),
            Some("sk-test")
        );
    }

    #[test]
    fn test_profile_crud() {
        let temp = TempDir::new().unwrap();
        let home = home(&temp);

        // Create
        let profile = make_profile("p1", "Profile 1", Some("gpt-4"));
        write_file(
            &home
                .join(".droidgear")
                .join("hermes")
                .join("profiles")
                .join("p1.json"),
            &serde_json::to_string_pretty(&profile).unwrap(),
        );

        // Read
        let loaded = get_hermes_profile_for_home(home, "p1").unwrap();
        assert_eq!(loaded.id, "p1");
        assert_eq!(loaded.name, "Profile 1");
        assert_eq!(loaded.models.len(), 1);
        assert_eq!(loaded.models[0].default.as_deref(), Some("gpt-4"));
        assert!(loaded.models[0].is_default);

        // List
        let profiles = list_hermes_profiles_for_home(home).unwrap();
        assert_eq!(profiles.len(), 1);

        // Delete
        delete_hermes_profile_for_home(home, "p1").unwrap();
        let profiles_after = list_hermes_profiles_for_home(home).unwrap();
        assert_eq!(profiles_after.len(), 0);
    }

    /// 回归：apply 必须写入 model.default / model.provider（微信 bot 等
    /// 外部工具读取 model.default，只写 custom_providers 不生效）。
    #[test]
    fn test_apply_writes_default_model_and_provider() {
        let temp = TempDir::new().unwrap();
        let home = home(&temp);

        // 模拟用户被旧版 droidgear 改坏的 config.yaml：
        // astra 只写在 custom_providers 里，model.default 仍是 sol。
        let base_yaml = r#"
model:
  default: gpt-5.6-sol
  provider: custom:wududu-codex-pro
custom_providers:
- name: custom
  base_url: https://sub.wududu.com/v1
  model: gpt-6-astra
"#;
        write_file(
            &home.join(".hermes").join("config.yaml"),
            base_yaml.trim_start(),
        );

        let profile = HermesProfile {
            id: "p1".to_string(),
            name: "astra".to_string(),
            description: None,
            created_at: "2026-01-01T00:00:00Z".to_string(),
            updated_at: "2026-01-01T00:00:00Z".to_string(),
            models: vec![HermesModelConfig {
                name: Some("wududu-codex-pro".to_string()),
                default: Some("gpt-6-astra".to_string()),
                provider: None,
                base_url: Some("https://sub.wududu.com/v1".to_string()),
                api_key: Some("sk-astra".to_string()),
                is_default: true,
            }],
            reasoning_effort: None,
        };
        write_file(
            &home
                .join(".droidgear")
                .join("hermes")
                .join("profiles")
                .join("p1.json"),
            &serde_json::to_string_pretty(&profile).unwrap(),
        );

        apply_hermes_profile_for_home(home, "p1").unwrap();

        let after = std::fs::read_to_string(home.join(".hermes").join("config.yaml")).unwrap();
        let parsed: Value = serde_yaml::from_str(&after).unwrap();

        // model.default / model.provider 必须是新模型（核心修复）
        let model_section = parsed.get("model").unwrap();
        assert_eq!(
            model_section.get("default").and_then(|v| v.as_str()),
            Some("gpt-6-astra")
        );
        assert_eq!(
            model_section.get("provider").and_then(|v| v.as_str()),
            Some("custom:wududu-codex-pro")
        );
        // 命名自定义供应商时 model 节不应残留 base_url/api_key
        assert!(model_section.get("base_url").is_none());
        assert!(model_section.get("api_key").is_none());

        // custom_providers 里应有对应的命名条目
        let providers = parsed
            .get("custom_providers")
            .and_then(|v| v.as_sequence())
            .unwrap();
        let ours = providers
            .iter()
            .find(|p| p.get("name").and_then(|v| v.as_str()) == Some("wududu-codex-pro"))
            .expect("wududu-codex-pro entry should exist");
        assert_eq!(
            ours.get("model").and_then(|v| v.as_str()),
            Some("gpt-6-astra")
        );
        assert_eq!(
            ours.get("api_key").and_then(|v| v.as_str()),
            Some("sk-astra")
        );
    }

    /// 命名自定义供应商：apply 应更新已有同名条目而不是重复添加。
    #[test]
    fn test_apply_updates_existing_named_provider() {
        let temp = TempDir::new().unwrap();
        let home = home(&temp);

        let base_yaml = r#"
custom_providers:
- name: mine
  base_url: https://old.example.com/v1
  model: old-model
"#;
        write_file(
            &home.join(".hermes").join("config.yaml"),
            base_yaml.trim_start(),
        );

        let mut profile = make_profile("p1", "Profile 1", Some("gpt-4"));
        profile.models[0] = HermesModelConfig {
            name: Some("mine".to_string()),
            default: Some("gpt-4".to_string()),
            provider: None,
            base_url: Some("https://new.example.com/v1".to_string()),
            api_key: Some("sk-new".to_string()),
            is_default: true,
        };
        write_file(
            &home
                .join(".droidgear")
                .join("hermes")
                .join("profiles")
                .join("p1.json"),
            &serde_json::to_string_pretty(&profile).unwrap(),
        );

        apply_hermes_profile_for_home(home, "p1").unwrap();

        let after = std::fs::read_to_string(home.join(".hermes").join("config.yaml")).unwrap();
        let parsed: Value = serde_yaml::from_str(&after).unwrap();

        let providers = parsed
            .get("custom_providers")
            .and_then(|v| v.as_sequence())
            .unwrap();
        assert_eq!(providers.len(), 1, "同名条目应被更新而不是新增");
        assert_eq!(
            providers[0].get("base_url").and_then(|v| v.as_str()),
            Some("https://new.example.com/v1")
        );
        assert_eq!(
            providers[0].get("model").and_then(|v| v.as_str()),
            Some("gpt-4")
        );

        // model 节引用命名供应商
        assert_eq!(
            parsed
                .get("model")
                .and_then(|v| v.get("provider"))
                .and_then(|v| v.as_str()),
            Some("custom:mine")
        );
    }

    /// 裸 custom（无 name）：base_url/api_key 写入 model 节。
    #[test]
    fn test_apply_bare_custom_writes_model_base_url() {
        let temp = TempDir::new().unwrap();
        let home = home(&temp);

        let base_yaml = r#"
model:
  default: old-model
  provider: openrouter
  base_url: https://stale.example.com/v1
"#;
        write_file(
            &home.join(".hermes").join("config.yaml"),
            base_yaml.trim_start(),
        );

        let mut profile = make_profile("p1", "Profile 1", Some("gpt-4"));
        profile.models[0] = HermesModelConfig {
            name: None,
            default: Some("gpt-4".to_string()),
            provider: Some("custom".to_string()),
            base_url: Some("https://api.openai.com/v1".to_string()),
            api_key: Some("sk-test".to_string()),
            is_default: true,
        };
        write_file(
            &home
                .join(".droidgear")
                .join("hermes")
                .join("profiles")
                .join("p1.json"),
            &serde_json::to_string_pretty(&profile).unwrap(),
        );

        apply_hermes_profile_for_home(home, "p1").unwrap();

        let after = std::fs::read_to_string(home.join(".hermes").join("config.yaml")).unwrap();
        let parsed: Value = serde_yaml::from_str(&after).unwrap();

        let model_section = parsed.get("model").unwrap();
        assert_eq!(
            model_section.get("provider").and_then(|v| v.as_str()),
            Some("custom")
        );
        assert_eq!(
            model_section.get("base_url").and_then(|v| v.as_str()),
            Some("https://api.openai.com/v1")
        );
        assert_eq!(
            model_section.get("api_key").and_then(|v| v.as_str()),
            Some("sk-test")
        );
    }

    #[test]
    fn test_apply_preserves_existing() {
        let temp = TempDir::new().unwrap();
        let home = home(&temp);

        // Existing config.yaml with unrelated sections and custom_providers
        let base_yaml = r#"
other_section:
  key: value
  nested:
    deep: 42
custom_providers:
- name: old-provider
  base_url: https://old-api.com/v1
  model: old-model
"#;
        write_file(
            &home.join(".hermes").join("config.yaml"),
            base_yaml.trim_start(),
        );

        // Save and apply a profile
        let profile = make_profile("p1", "Profile 1", Some("gpt-4"));
        write_file(
            &home
                .join(".droidgear")
                .join("hermes")
                .join("profiles")
                .join("p1.json"),
            &serde_json::to_string_pretty(&profile).unwrap(),
        );

        apply_hermes_profile_for_home(home, "p1").unwrap();

        let after = std::fs::read_to_string(home.join(".hermes").join("config.yaml")).unwrap();
        let parsed: Value = serde_yaml::from_str(&after).unwrap();

        // Unrelated section preserved
        assert_eq!(
            parsed
                .get("other_section")
                .and_then(|v| v.get("key"))
                .and_then(|v| v.as_str()),
            Some("value")
        );
        assert_eq!(
            parsed
                .get("other_section")
                .and_then(|v| v.get("nested"))
                .and_then(|v| v.get("deep"))
                .and_then(|v| v.as_i64()),
            Some(42)
        );

        // custom_providers: old one preserved (no match), new one added
        let providers = parsed
            .get("custom_providers")
            .and_then(|v| v.as_sequence())
            .unwrap();
        assert_eq!(providers.len(), 2);
        assert_eq!(
            providers[0].get("name").and_then(|v| v.as_str()),
            Some("old-provider")
        );
        assert_eq!(
            providers[0].get("model").and_then(|v| v.as_str()),
            Some("old-model")
        );
        // New provider added
        assert_eq!(
            providers[1].get("name").and_then(|v| v.as_str()),
            Some("openai")
        );
        assert_eq!(
            providers[1].get("model").and_then(|v| v.as_str()),
            Some("gpt-4")
        );
        assert_eq!(
            providers[1].get("api_key").and_then(|v| v.as_str()),
            Some("sk-test")
        );

        // model 节写入默认模型 + custom:<name>
        assert_eq!(
            parsed
                .get("model")
                .and_then(|v| v.get("default"))
                .and_then(|v| v.as_str()),
            Some("gpt-4")
        );
        assert_eq!(
            parsed
                .get("model")
                .and_then(|v| v.get("provider"))
                .and_then(|v| v.as_str()),
            Some("custom:openai")
        );

        // Active profile set
        let active = get_active_hermes_profile_id_for_home(home)
            .unwrap()
            .unwrap();
        assert_eq!(active, "p1");
    }

    #[test]
    fn test_apply_writes_reasoning_effort() {
        let temp = TempDir::new().unwrap();
        let home = home(&temp);

        // Create a profile with reasoning_effort
        let mut profile = make_profile("p1", "Profile 1", Some("gpt-4"));
        profile.reasoning_effort = Some("high".to_string());
        write_file(
            &home
                .join(".droidgear")
                .join("hermes")
                .join("profiles")
                .join("p1.json"),
            &serde_json::to_string_pretty(&profile).unwrap(),
        );

        apply_hermes_profile_for_home(home, "p1").unwrap();

        let after = std::fs::read_to_string(home.join(".hermes").join("config.yaml")).unwrap();
        let parsed: Value = serde_yaml::from_str(&after).unwrap();

        // reasoning_effort should be written under agent section
        assert_eq!(
            parsed
                .get("agent")
                .and_then(|v| v.get("reasoning_effort"))
                .and_then(|v| v.as_str()),
            Some("high")
        );
    }

    /// 空 profile（只有一条空默认条目）apply 不应破坏现有 model 节。
    #[test]
    fn test_apply_empty_profile_leaves_model_section() {
        let temp = TempDir::new().unwrap();
        let home = home(&temp);

        let base_yaml = r#"
model:
  default: keep-me
  provider: openrouter
"#;
        write_file(
            &home.join(".hermes").join("config.yaml"),
            base_yaml.trim_start(),
        );

        let profile = create_default_hermes_profile_for_home(home).unwrap();
        apply_hermes_profile_for_home(home, &profile.id).unwrap();

        let after = std::fs::read_to_string(home.join(".hermes").join("config.yaml")).unwrap();
        let parsed: Value = serde_yaml::from_str(&after).unwrap();
        assert_eq!(
            parsed
                .get("model")
                .and_then(|v| v.get("default"))
                .and_then(|v| v.as_str()),
            Some("keep-me")
        );
        assert_eq!(
            parsed
                .get("model")
                .and_then(|v| v.get("provider"))
                .and_then(|v| v.as_str()),
            Some("openrouter")
        );
    }

    #[test]
    fn test_read_current_config_with_reasoning_effort() {
        let temp = TempDir::new().unwrap();
        let home = home(&temp);

        let yaml = r#"
model:
  default: gpt-4
  provider: custom:openai
custom_providers:
- name: openai
  base_url: https://api.openai.com/v1
  api_key: sk-test
  model: gpt-4
agent:
  reasoning_effort: xhigh
"#;
        write_file(&home.join(".hermes").join("config.yaml"), yaml.trim_start());

        let config = read_hermes_current_config_for_home(home).unwrap();
        assert_eq!(config.reasoning_effort.as_deref(), Some("xhigh"));
        // live model 节 + custom_providers 合并为一条默认条目
        assert_eq!(config.models.len(), 1);
        assert_eq!(config.models[0].default.as_deref(), Some("gpt-4"));
        assert_eq!(config.models[0].name.as_deref(), Some("openai"));
        assert!(config.models[0].is_default);
    }

    #[test]
    fn test_read_current_config_without_reasoning_effort() {
        let temp = TempDir::new().unwrap();
        let home = home(&temp);

        let yaml = r#"
custom_providers:
- name: openai
  model: gpt-4
"#;
        write_file(&home.join(".hermes").join("config.yaml"), yaml.trim_start());

        let config = read_hermes_current_config_for_home(home).unwrap();
        assert_eq!(config.reasoning_effort, None);
        // 只有 custom_providers 时第一条视为默认
        assert_eq!(config.models.len(), 1);
        assert_eq!(config.models[0].default.as_deref(), Some("gpt-4"));
        assert!(config.models[0].is_default);
    }

    #[test]
    fn test_read_legacy_model_section() {
        let temp = TempDir::new().unwrap();
        let home = home(&temp);

        // Only model section, no custom_providers
        let yaml = r#"
model:
  default: gpt-4-turbo
  provider: openai
  base_url: https://api.openai.com/v1
  api_key: sk-live
"#;
        write_file(&home.join(".hermes").join("config.yaml"), yaml.trim_start());

        let config = read_hermes_current_config_for_home(home).unwrap();
        assert_eq!(config.models.len(), 1);
        assert_eq!(config.models[0].default.as_deref(), Some("gpt-4-turbo"));
        assert_eq!(config.models[0].provider.as_deref(), Some("openai"));
        assert_eq!(
            config.models[0].base_url.as_deref(),
            Some("https://api.openai.com/v1")
        );
        assert_eq!(config.models[0].api_key.as_deref(), Some("sk-live"));
        assert!(config.models[0].is_default);
    }

    /// live model 节优先：provider 指向的命名条目是默认，其余条目保留在列表中。
    #[test]
    fn test_read_model_section_takes_precedence() {
        let temp = TempDir::new().unwrap();
        let home = home(&temp);

        // 用户反馈的坏状态：provider 引用 wududu-codex-pro，
        // 但 custom_providers 里只有 name: custom 的旧条目。
        let yaml = r#"
model:
  default: gpt-5.6-sol
  provider: custom:wududu-codex-pro
custom_providers:
- name: custom
  base_url: https://sub.wududu.com/v1
  model: gpt-6-astra
"#;
        write_file(&home.join(".hermes").join("config.yaml"), yaml.trim_start());

        let config = read_hermes_current_config_for_home(home).unwrap();

        // 列表应包含 custom_providers 里的条目 + live 引用条目
        assert_eq!(config.models.len(), 2);
        let live = config
            .models
            .iter()
            .find(|m| m.is_default)
            .expect("live entry should be default");
        assert_eq!(live.name.as_deref(), Some("wududu-codex-pro"));
        assert_eq!(live.default.as_deref(), Some("gpt-5.6-sol"));
    }

    #[test]
    fn test_read_bare_custom_roundtrip() {
        let temp = TempDir::new().unwrap();
        let home = home(&temp);

        let yaml = r#"
model:
  default: gpt-4
  provider: custom
  base_url: https://api.openai.com/v1
  api_key: sk-test
"#;
        write_file(&home.join(".hermes").join("config.yaml"), yaml.trim_start());

        let config = read_hermes_current_config_for_home(home).unwrap();
        assert_eq!(config.models.len(), 1);
        let entry = &config.models[0];
        assert!(entry.is_default);
        assert_eq!(entry.provider.as_deref(), Some("custom"));
        assert_eq!(entry.base_url.as_deref(), Some("https://api.openai.com/v1"));
        assert_eq!(entry.api_key.as_deref(), Some("sk-test"));
    }

    #[test]
    fn test_config_status() {
        let temp = TempDir::new().unwrap();
        let home = home(&temp);

        // No config yet
        let status = get_hermes_config_status_for_home(home).unwrap();
        assert!(!status.config_exists);
        assert!(status.config_path.contains("config.yaml"));

        // Create config
        write_file(&home.join(".hermes").join("config.yaml"), "model: {}\n");
        let status = get_hermes_config_status_for_home(home).unwrap();
        assert!(status.config_exists);
    }

    #[test]
    fn test_default_profile_creation() {
        let temp = TempDir::new().unwrap();
        let home = home(&temp);

        // Should succeed when no profiles exist
        let profile = create_default_hermes_profile_for_home(home).unwrap();
        assert!(!profile.id.is_empty());
        assert_eq!(profile.name, "默认");
        assert_eq!(profile.models.len(), 1);
        assert!(profile.models[0].is_default);

        // Should fail when profiles already exist
        let err = create_default_hermes_profile_for_home(home).unwrap_err();
        assert_eq!(err, "Profiles already exist");
    }

    #[test]
    fn test_duplicate_profile() {
        let temp = TempDir::new().unwrap();
        let home = home(&temp);

        let profile = make_profile("orig", "Original", Some("gpt-4"));
        write_file(
            &home
                .join(".droidgear")
                .join("hermes")
                .join("profiles")
                .join("orig.json"),
            &serde_json::to_string_pretty(&profile).unwrap(),
        );

        let dup = duplicate_hermes_profile_for_home(home, "orig", "Copy").unwrap();
        assert_ne!(dup.id, "orig");
        assert_eq!(dup.name, "Copy");
        assert_eq!(dup.models[0].default.as_deref(), Some("gpt-4"));

        // Both should exist
        let profiles = list_hermes_profiles_for_home(home).unwrap();
        assert_eq!(profiles.len(), 2);
    }

    #[test]
    fn test_active_profile() {
        let temp = TempDir::new().unwrap();
        let home = home(&temp);

        // Initially no active profile
        let active = get_active_hermes_profile_id_for_home(home).unwrap();
        assert!(active.is_none());

        // Create and apply profile
        let profile = make_profile("p1", "Profile 1", Some("gpt-4"));
        write_file(
            &home
                .join(".droidgear")
                .join("hermes")
                .join("profiles")
                .join("p1.json"),
            &serde_json::to_string_pretty(&profile).unwrap(),
        );

        apply_hermes_profile_for_home(home, "p1").unwrap();
        let active = get_active_hermes_profile_id_for_home(home)
            .unwrap()
            .unwrap();
        assert_eq!(active, "p1");

        // Delete profile should clear active
        delete_hermes_profile_for_home(home, "p1").unwrap();
        let active_after = get_active_hermes_profile_id_for_home(home).unwrap();
        assert!(active_after.is_none());
    }

    /// 旧版单条 model 的 profile JSON 应自动迁移为 models 列表。
    #[test]
    fn test_legacy_profile_migration() {
        let temp = TempDir::new().unwrap();
        let home = home(&temp);

        // 旧版格式：profile.model 是单个对象
        let legacy_json = r#"{
  "id": "legacy1",
  "name": "Legacy",
  "description": null,
  "createdAt": "2026-01-01T00:00:00Z",
  "updatedAt": "2026-01-01T00:00:00Z",
  "model": {
    "default": "gpt-5.6-sol",
    "provider": "custom:wududu-codex-pro",
    "baseUrl": "https://sub.wududu.com/v1",
    "apiKey": "sk-old"
  }
}"#;
        write_file(
            &home
                .join(".droidgear")
                .join("hermes")
                .join("profiles")
                .join("legacy1.json"),
            legacy_json,
        );

        let loaded = get_hermes_profile_for_home(home, "legacy1").unwrap();
        assert_eq!(loaded.models.len(), 1);
        let entry = &loaded.models[0];
        assert_eq!(entry.default.as_deref(), Some("gpt-5.6-sol"));
        assert_eq!(entry.provider.as_deref(), Some("custom:wududu-codex-pro"));
        assert_eq!(entry.base_url.as_deref(), Some("https://sub.wududu.com/v1"));
        assert_eq!(entry.api_key.as_deref(), Some("sk-old"));
        assert!(entry.is_default);
    }

    /// 多条 is_default 的 profile 保存时应归一化为一条默认。
    #[test]
    fn test_normalize_models_on_save() {
        let temp = TempDir::new().unwrap();
        let home = home(&temp);

        let profile = HermesProfile {
            id: "p1".to_string(),
            name: "multi".to_string(),
            description: None,
            created_at: "2026-01-01T00:00:00Z".to_string(),
            updated_at: "2026-01-01T00:00:00Z".to_string(),
            models: vec![
                HermesModelConfig {
                    name: Some("a".to_string()),
                    default: Some("model-a".to_string()),
                    provider: None,
                    base_url: Some("https://a.example.com/v1".to_string()),
                    api_key: None,
                    is_default: true,
                },
                HermesModelConfig {
                    name: Some("b".to_string()),
                    default: Some("model-b".to_string()),
                    provider: None,
                    base_url: Some("https://b.example.com/v1".to_string()),
                    api_key: None,
                    is_default: true,
                },
            ],
            reasoning_effort: None,
        };

        save_hermes_profile_for_home(home, profile.clone()).unwrap();
        let loaded = get_hermes_profile_for_home(home, "p1").unwrap();
        assert_eq!(loaded.models.iter().filter(|m| m.is_default).count(), 1);
        assert!(loaded.models[0].is_default);
    }

    #[test]
    fn test_read_current_config_from_yaml() {
        let temp = TempDir::new().unwrap();
        let home = home(&temp);

        let yaml = r#"custom_providers:
- name: openai
  base_url: https://api.openai.com/v1
  api_key: sk-live
  model: gpt-4-turbo
unrelated: data
"#;
        write_file(&home.join(".hermes").join("config.yaml"), yaml);

        let current = read_hermes_current_config_for_home(home).unwrap();
        assert_eq!(current.models.len(), 1);
        assert_eq!(current.models[0].default.as_deref(), Some("gpt-4-turbo"));
        assert_eq!(current.models[0].name.as_deref(), Some("openai"));
        assert_eq!(
            current.models[0].base_url.as_deref(),
            Some("https://api.openai.com/v1")
        );
        assert_eq!(current.models[0].api_key.as_deref(), Some("sk-live"));
        assert!(current.models[0].is_default);
    }

    #[test]
    fn test_save_profile_preserves_created_at() {
        let temp = TempDir::new().unwrap();
        let home = home(&temp);

        let mut profile = make_profile("p1", "Profile 1", Some("gpt-4"));
        profile.created_at = "2024-06-01T00:00:00Z".to_string();

        // Save new profile
        save_hermes_profile_for_home(home, profile.clone()).unwrap();
        let loaded = get_hermes_profile_for_home(home, "p1").unwrap();
        assert_eq!(loaded.created_at, "2024-06-01T00:00:00Z");

        // Update profile: created_at must be preserved
        let mut updated = loaded.clone();
        updated.name = "Updated Name".to_string();
        updated.created_at = "2024-06-01T00:00:00Z".to_string(); // explicit
        save_hermes_profile_for_home(home, updated).unwrap();

        let reloaded = get_hermes_profile_for_home(home, "p1").unwrap();
        assert_eq!(reloaded.created_at, "2024-06-01T00:00:00Z");
        assert_eq!(reloaded.name, "Updated Name");
    }

    #[test]
    fn test_system_wrapper_reads_appdata_config() {
        // Only run on Windows with actual config
        #[cfg(target_os = "windows")]
        {
            if let Some(local_app_data) = std::env::var_os("LOCALAPPDATA") {
                let config_path = std::path::PathBuf::from(local_app_data)
                    .join("hermes")
                    .join("config.yaml");
                if config_path.exists() {
                    let config = read_hermes_current_config().unwrap();
                    assert!(
                        config.reasoning_effort.is_some(),
                        "reasoning_effort should be Some, got None. Config: {:?}",
                        config_path
                    );
                }
            }
        }
    }
}
