//! Hermes Agent 配置管理（core）。
//!
//! 负责 Profile CRUD，并支持将 Profile 应用到 `~/.hermes/config.yaml`。
//! Apply 逻辑采用读取-修改-写入模式，以保留 YAML 文件中的其他非 model 配置节。
//! 逻辑从原 Tauri command 层抽离，以便在 TUI 与桌面端复用。

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

/// Hermes model 配置（对应 config.yaml 中的 model 节）
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct HermesModelConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub provider: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub base_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub api_key: Option<String>,
}

/// Hermes Profile（用于在 DroidGear 内部保存并切换）
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct HermesProfile {
    pub id: String,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    pub created_at: String,
    pub updated_at: String,
    pub model: HermesModelConfig,
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
    pub model: HermesModelConfig,
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
        model: HermesModelConfig {
            default: Some(String::new()),
            provider: Some(String::new()),
            base_url: Some(String::new()),
            api_key: Some(String::new()),
        },
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

/// Internal: write a profile's model config to the given config.yaml path.
///
/// 采用读取-修改-写入模式：只替换 config.yaml 中的 model 节，保留其他所有配置。
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

    // Build the new model section from the profile's model config.
    let mut model_map = serde_yaml::Mapping::new();
    if let Some(ref default) = profile.model.default {
        model_map.insert(
            Value::String("default".to_string()),
            Value::String(default.clone()),
        );
    }
    if let Some(ref provider) = profile.model.provider {
        model_map.insert(
            Value::String("provider".to_string()),
            Value::String(provider.clone()),
        );
    }
    if let Some(ref base_url) = profile.model.base_url {
        model_map.insert(
            Value::String("base_url".to_string()),
            Value::String(base_url.clone()),
        );
    }
    if let Some(ref api_key) = profile.model.api_key {
        model_map.insert(
            Value::String("api_key".to_string()),
            Value::String(api_key.clone()),
        );
    }

    // Replace the model section (preserving all other sections).
    root.insert(
        Value::String("model".to_string()),
        Value::Mapping(model_map),
    );

    let yaml_str = serde_yaml::to_string(&config)
        .map_err(|e| format!("Failed to serialize config.yaml: {e}"))?;
    storage::atomic_write(config_path, yaml_str.as_bytes())
}

/// Internal: read current Hermes config from a specific config.yaml path.
fn read_current_config_from_path(config_path: &Path) -> Result<HermesCurrentConfig, String> {
    let model = if config_path.exists() {
        let s = std::fs::read_to_string(config_path)
            .map_err(|e| format!("Failed to read config.yaml: {e}"))?;
        if s.trim().is_empty() {
            HermesModelConfig {
                default: None,
                provider: None,
                base_url: None,
                api_key: None,
            }
        } else {
            let parsed: Value = serde_yaml::from_str(&s)
                .map_err(|e| format!("Failed to parse config.yaml: {e}"))?;

            let model_section = parsed.get("model");

            let get_str = |key: &str| -> Option<String> {
                model_section
                    .and_then(|m| m.get(key))
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string())
            };

            HermesModelConfig {
                default: get_str("default"),
                provider: get_str("provider"),
                base_url: get_str("base_url"),
                api_key: get_str("api_key"),
            }
        }
    } else {
        HermesModelConfig {
            default: None,
            provider: None,
            base_url: None,
            api_key: None,
        }
    };

    Ok(HermesCurrentConfig { model })
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

    fn make_profile(id: &str, name: &str, default_model: Option<&str>) -> HermesProfile {
        HermesProfile {
            id: id.to_string(),
            name: name.to_string(),
            description: None,
            created_at: "2026-01-01T00:00:00Z".to_string(),
            updated_at: "2026-01-01T00:00:00Z".to_string(),
            model: HermesModelConfig {
                default: default_model.map(|s| s.to_string()),
                provider: Some("openai".to_string()),
                base_url: Some("https://api.openai.com/v1".to_string()),
                api_key: Some("sk-test".to_string()),
            },
        }
    }

    #[test]
    fn test_yaml_serialization() {
        let model = HermesModelConfig {
            default: Some("gpt-4".to_string()),
            provider: Some("openai".to_string()),
            base_url: Some("https://api.openai.com/v1".to_string()),
            api_key: Some("sk-test".to_string()),
        };

        let profile = HermesProfile {
            id: "p1".to_string(),
            name: "Test".to_string(),
            description: None,
            created_at: "2026-01-01T00:00:00Z".to_string(),
            updated_at: "2026-01-01T00:00:00Z".to_string(),
            model,
        };

        // Verify JSON serialization (profiles stored as JSON)
        let json = serde_json::to_string_pretty(&profile).unwrap();
        assert!(json.contains("\"default\":"));
        assert!(json.contains("\"provider\":"));
        assert!(json.contains("\"baseUrl\":"));
        assert!(json.contains("\"apiKey\":"));
        assert!(json.contains("gpt-4"));
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
        assert_eq!(loaded.model.default.as_deref(), Some("gpt-4"));

        // List
        let profiles = list_hermes_profiles_for_home(home).unwrap();
        assert_eq!(profiles.len(), 1);

        // Delete
        delete_hermes_profile_for_home(home, "p1").unwrap();
        let profiles_after = list_hermes_profiles_for_home(home).unwrap();
        assert_eq!(profiles_after.len(), 0);
    }

    #[test]
    fn test_apply_preserves_existing() {
        let temp = TempDir::new().unwrap();
        let home = home(&temp);

        // Existing config.yaml with unrelated sections
        let base_yaml = r#"
other_section:
  key: value
  nested:
    deep: 42
model:
  default: old-model
  provider: old-provider
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

        // Model section updated
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
            Some("openai")
        );
        assert_eq!(
            parsed
                .get("model")
                .and_then(|v| v.get("api_key"))
                .and_then(|v| v.as_str()),
            Some("sk-test")
        );

        // Active profile set
        let active = get_active_hermes_profile_id_for_home(home)
            .unwrap()
            .unwrap();
        assert_eq!(active, "p1");
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
        assert_eq!(dup.model.default.as_deref(), Some("gpt-4"));

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

    #[test]
    fn test_read_current_config_from_yaml() {
        let temp = TempDir::new().unwrap();
        let home = home(&temp);

        let yaml = r#"model:
  default: gpt-4-turbo
  provider: openai
  base_url: https://api.openai.com/v1
  api_key: sk-live
unrelated: data
"#;
        write_file(&home.join(".hermes").join("config.yaml"), yaml);

        let current = read_hermes_current_config_for_home(home).unwrap();
        assert_eq!(current.model.default.as_deref(), Some("gpt-4-turbo"));
        assert_eq!(current.model.provider.as_deref(), Some("openai"));
        assert_eq!(
            current.model.base_url.as_deref(),
            Some("https://api.openai.com/v1")
        );
        assert_eq!(current.model.api_key.as_deref(), Some("sk-live"));
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
}
