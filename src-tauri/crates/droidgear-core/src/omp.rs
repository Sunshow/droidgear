//! OMP (Oh My Pi) configuration management (core).
//!
//! OMP stores configuration across three locations:
//! - `~/.omp/agent/config.yml` — agent behavior (modelRoles, theme, etc.)
//! - `~/.omp/agent/models.db` — SQLite model catalog cache (read-only from DroidGear)
//! - `~/.omp/agent/agent.db` — SQLite credentials (read-only from DroidGear)
//!
//! DroidGear manages model role assignments via `config.yml` and reads the
//! model catalog and credential status from the SQLite databases.

use chrono::Utc;
use serde::{Deserialize, Serialize};
use specta::Type;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::time::Duration;
use uuid::Uuid;

use crate::{paths, storage};

// ============================================================================
// Types — config.yml
// ============================================================================

/// OMP model roles — which model handles which role.
#[derive(Debug, Clone, Default, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct OmpModelRoles {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub smol: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub slow: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub plan: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub commit: Option<String>,
    #[serde(flatten)]
    #[specta(skip)]
    pub extra: HashMap<String, serde_json::Value>,
}

/// OMP theme configuration.
#[derive(Debug, Clone, Default, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct OmpTheme {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dark: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub light: Option<String>,
}

/// OMP agent configuration (from `~/.omp/agent/config.yml`).
#[derive(Debug, Clone, Default, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct OmpAgentConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model_roles: Option<OmpModelRoles>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub theme: Option<OmpTheme>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub steering_mode: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub follow_up_mode: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub interrupt_mode: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub symbol_preset: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub setup_version: Option<u32>,
    #[serde(flatten)]
    #[specta(skip)]
    pub extra: HashMap<String, serde_json::Value>,
}

// ============================================================================
// Types — models.db (read-only)
// ============================================================================

/// OMP model cost information.
#[derive(Debug, Clone, Default, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct OmpModelCost {
    #[serde(default)]
    pub input: f64,
    #[serde(default)]
    pub output: f64,
    #[serde(default)]
    pub cache_read: f64,
    #[serde(default)]
    pub cache_write: f64,
}

/// OMP thinking/reasoning configuration.
#[derive(Debug, Clone, Default, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct OmpThinkingConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mode: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub efforts: Option<Vec<String>>,
}

/// A cached model entry from `models.db` (read-only).
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct OmpCachedModel {
    pub id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub api: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub provider: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub base_url: Option<String>,
    #[serde(default)]
    pub reasoning: bool,
    #[serde(default = "default_input")]
    pub input: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cost: Option<OmpModelCost>,
    #[serde(default = "default_context_window")]
    pub context_window: u32,
    #[serde(default = "default_max_tokens")]
    pub max_tokens: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub thinking: Option<OmpThinkingConfig>,
    #[serde(flatten)]
    #[specta(skip)]
    pub extra: HashMap<String, serde_json::Value>,
}

fn default_input() -> Vec<String> {
    vec!["text".to_string()]
}

fn default_context_window() -> u32 {
    128000
}

fn default_max_tokens() -> u32 {
    16384
}

/// A provider's cached model list from `models.db`.
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct OmpProviderModels {
    pub provider_id: String,
    pub models: Vec<OmpCachedModel>,
}

// ============================================================================
// Types — agent.db (read-only)
// ============================================================================

/// OMP credential status for a provider.
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct OmpCredentialStatus {
    pub provider: String,
    pub credential_type: String,
    pub has_key: bool,
}

// ============================================================================
// Types — combined read result
// ============================================================================

/// Full OMP configuration status.
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct OmpConfigStatus {
    pub config_exists: bool,
    pub config_path: String,
    pub models_db_exists: bool,
    pub agent_db_exists: bool,
}

/// Current OMP configuration (combined from all sources).
#[derive(Debug, Clone, Default, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct OmpCurrentConfig {
    #[serde(default)]
    pub agent_config: OmpAgentConfig,
    #[serde(default)]
    pub provider_models: Vec<OmpProviderModels>,
    #[serde(default)]
    pub credentials: Vec<OmpCredentialStatus>,
}

/// Result of validating an OMP provider through HTTP connectivity testing.
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct OmpProviderTestResult {
    pub success: bool,
    pub provider_id: String,
    pub model_id: String,
    pub latency_ms: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub response_text: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

/// OMP profile (stored in DroidGear) — snapshot of model role assignments.
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct OmpProfile {
    pub id: String,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    pub created_at: String,
    pub updated_at: String,
    /// Model role assignments (what gets written to config.yml on apply).
    #[serde(default)]
    pub model_roles: OmpModelRoles,
}

// ============================================================================
// Path Helpers
// ============================================================================

fn droidgear_omp_dir_for_home(home_dir: &Path) -> PathBuf {
    home_dir.join(".droidgear").join("omp")
}

/// `~/.droidgear/omp/profiles/`
pub fn profiles_dir_for_home(home_dir: &Path) -> Result<PathBuf, String> {
    let dir = droidgear_omp_dir_for_home(home_dir).join("profiles");
    if !dir.exists() {
        std::fs::create_dir_all(&dir)
            .map_err(|e| format!("Failed to create OMP profiles directory: {e}"))?;
    }
    Ok(dir)
}

/// `~/.droidgear/omp/active-profile.txt`
pub fn active_profile_path_for_home(home_dir: &Path) -> Result<PathBuf, String> {
    let dir = droidgear_omp_dir_for_home(home_dir);
    if !dir.exists() {
        std::fs::create_dir_all(&dir)
            .map_err(|e| format!("Failed to create OMP directory: {e}"))?;
    }
    Ok(dir.join("active-profile.txt"))
}

/// `~/.omp/agent/` (OMP home directory)
pub fn omp_config_dir_for_home(home_dir: &Path) -> Result<PathBuf, String> {
    let config_paths = paths::load_config_paths_for_home(home_dir);
    let dir = paths::get_omp_home_for_home(home_dir, &config_paths)?;
    if !dir.exists() {
        std::fs::create_dir_all(&dir)
            .map_err(|e| format!("Failed to create OMP config directory: {e}"))?;
    }
    Ok(dir)
}

/// `~/.omp/agent/config.yml`
pub fn omp_config_yml_path_for_home(home_dir: &Path) -> Result<PathBuf, String> {
    Ok(omp_config_dir_for_home(home_dir)?.join("config.yml"))
}

/// `~/.omp/agent/models.db`
pub fn omp_models_db_path_for_home(home_dir: &Path) -> Result<PathBuf, String> {
    Ok(omp_config_dir_for_home(home_dir)?.join("models.db"))
}

/// `~/.omp/agent/agent.db`
pub fn omp_agent_db_path_for_home(home_dir: &Path) -> Result<PathBuf, String> {
    Ok(omp_config_dir_for_home(home_dir)?.join("agent.db"))
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
// System wrappers
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

pub fn omp_config_dir() -> Result<PathBuf, String> {
    omp_config_dir_for_home(&system_home_dir()?)
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

fn read_profile_file(path: &Path) -> Result<OmpProfile, String> {
    let s = std::fs::read_to_string(path).map_err(|e| format!("Failed to read profile: {e}"))?;
    serde_json::from_str::<OmpProfile>(&s).map_err(|e| format!("Invalid profile JSON: {e}"))
}

fn write_profile_file(home_dir: &Path, profile: &OmpProfile) -> Result<(), String> {
    let path = profile_path_for_home(home_dir, &profile.id)?;
    let s = serde_json::to_string_pretty(profile)
        .map_err(|e| format!("Failed to serialize profile JSON: {e}"))?;
    storage::atomic_write(&path, s.as_bytes())
}

fn load_profile_by_id(home_dir: &Path, id: &str) -> Result<OmpProfile, String> {
    let path = profile_path_for_home(home_dir, id)?;
    read_profile_file(&path)
}

// ============================================================================
// config.yml Read/Write
// ============================================================================

fn read_config_yml(home_dir: &Path) -> Result<OmpAgentConfig, String> {
    let path = omp_config_yml_path_for_home(home_dir)?;
    if !path.exists() {
        return Ok(OmpAgentConfig::default());
    }
    let s =
        std::fs::read_to_string(&path).map_err(|e| format!("Failed to read config.yml: {e}"))?;
    if s.trim().is_empty() {
        return Ok(OmpAgentConfig::default());
    }
    serde_yaml::from_str(&s).map_err(|e| format!("Invalid config.yml YAML: {e}"))
}

fn write_config_yml(home_dir: &Path, config: &OmpAgentConfig) -> Result<(), String> {
    let path = omp_config_yml_path_for_home(home_dir)?;
    let yaml_str = serde_yaml::to_string(config)
        .map_err(|e| format!("Failed to serialize config.yml: {e}"))?;

    let config_dir = omp_config_dir_for_home(home_dir)?;
    if !config_dir.exists() {
        std::fs::create_dir_all(&config_dir)
            .map_err(|e| format!("Failed to create OMP config directory: {e}"))?;
    }

    storage::atomic_write(&path, yaml_str.as_bytes())
}

// ============================================================================
// models.db Read (read-only)
// ============================================================================

#[cfg(test)]
fn read_model_cache_from_db(_path: &Path) -> Result<Vec<OmpProviderModels>, String> {
    // In tests, return empty (SQLite not available in unit tests)
    Ok(vec![])
}

#[cfg(not(test))]
fn read_model_cache_from_db(path: &Path) -> Result<Vec<OmpProviderModels>, String> {
    use rusqlite::Connection;

    if !path.exists() {
        return Ok(vec![]);
    }

    let conn = Connection::open(path).map_err(|e| format!("Failed to open models.db: {e}"))?;

    let mut stmt = conn
        .prepare("SELECT provider_id, models FROM model_cache")
        .map_err(|e| format!("Failed to query model_cache: {e}"))?;

    let rows = stmt
        .query_map([], |row| {
            let provider_id: String = row.get(0)?;
            let models_json: String = row.get(1)?;
            Ok((provider_id, models_json))
        })
        .map_err(|e| format!("Failed to read model_cache rows: {e}"))?;

    let mut result = Vec::new();
    for row in rows {
        let (provider_id, models_json) =
            row.map_err(|e| format!("Failed to read model_cache row: {e}"))?;
        let models: Vec<OmpCachedModel> = serde_json::from_str(&models_json)
            .map_err(|e| format!("Failed to parse models for '{provider_id}': {e}"))?;
        result.push(OmpProviderModels {
            provider_id,
            models,
        });
    }

    Ok(result)
}

// ============================================================================
// agent.db Read (read-only)
// ============================================================================

#[cfg(test)]
fn read_credentials_from_db(_path: &Path) -> Result<Vec<OmpCredentialStatus>, String> {
    Ok(vec![])
}

#[cfg(not(test))]
fn read_credentials_from_db(path: &Path) -> Result<Vec<OmpCredentialStatus>, String> {
    use rusqlite::Connection;

    if !path.exists() {
        return Ok(vec![]);
    }

    let conn = Connection::open(path).map_err(|e| format!("Failed to open agent.db: {e}"))?;

    let mut stmt = conn
        .prepare("SELECT provider, credential_type, data FROM auth_credentials")
        .map_err(|e| format!("Failed to query auth_credentials: {e}"))?;

    let rows = stmt
        .query_map([], |row| {
            let provider: String = row.get(0)?;
            let credential_type: String = row.get(1)?;
            let data: String = row.get(2)?;
            Ok((provider, credential_type, data))
        })
        .map_err(|e| format!("Failed to read auth_credentials rows: {e}"))?;

    let mut result = Vec::new();
    for row in rows {
        let (provider, credential_type, data) =
            row.map_err(|e| format!("Failed to read auth_credentials row: {e}"))?;
        let has_key = !data.trim().is_empty() && data != "null";
        result.push(OmpCredentialStatus {
            provider,
            credential_type,
            has_key,
        });
    }

    Ok(result)
}

// ============================================================================
// CRUD (Profiles)
// ============================================================================

pub fn list_omp_profiles_for_home(home_dir: &Path) -> Result<Vec<OmpProfile>, String> {
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

pub fn get_omp_profile_for_home(home_dir: &Path, id: &str) -> Result<OmpProfile, String> {
    load_profile_by_id(home_dir, id)
}

pub fn save_omp_profile_for_home(home_dir: &Path, mut profile: OmpProfile) -> Result<(), String> {
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

pub fn delete_omp_profile_for_home(home_dir: &Path, id: &str) -> Result<(), String> {
    let path = profile_path_for_home(home_dir, id)?;
    if path.exists() {
        std::fs::remove_file(&path).map_err(|e| format!("Failed to delete profile: {e}"))?;
    }

    if let Ok(active) = get_active_omp_profile_id_for_home(home_dir) {
        if active.as_deref() == Some(id) {
            let active_path = active_profile_path_for_home(home_dir)?;
            let _ = std::fs::remove_file(active_path);
        }
    }
    Ok(())
}

pub fn duplicate_omp_profile_for_home(
    home_dir: &Path,
    id: &str,
    new_name: &str,
) -> Result<OmpProfile, String> {
    let mut profile = load_profile_by_id(home_dir, id)?;
    profile.id = Uuid::new_v4().to_string();
    profile.name = new_name.to_string();
    profile.created_at = now_rfc3339();
    profile.updated_at = profile.created_at.clone();
    write_profile_file(home_dir, &profile)?;
    Ok(profile)
}

pub fn create_default_omp_profile_for_home(home_dir: &Path) -> Result<OmpProfile, String> {
    let profiles = list_omp_profiles_for_home(home_dir)?;
    if !profiles.is_empty() {
        return Err("Profiles already exist".to_string());
    }

    let id = Uuid::new_v4().to_string();
    let now = now_rfc3339();

    let profile = OmpProfile {
        id,
        name: "Default".to_string(),
        description: None,
        created_at: now.clone(),
        updated_at: now,
        model_roles: OmpModelRoles::default(),
    };

    write_profile_file(home_dir, &profile)?;
    Ok(profile)
}

// ============================================================================
// Active profile
// ============================================================================

pub fn get_active_omp_profile_id_for_home(home_dir: &Path) -> Result<Option<String>, String> {
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

pub fn set_active_omp_profile_id_for_home(home_dir: &Path, id: &str) -> Result<(), String> {
    let path = active_profile_path_for_home(home_dir)?;
    storage::atomic_write(&path, id.as_bytes())
}

// ============================================================================
// Apply + Config Status + Read Current Config
// ============================================================================

/// Apply a profile: write model roles to `~/.omp/agent/config.yml`.
pub fn apply_omp_profile_for_home(home_dir: &Path, id: &str) -> Result<(), String> {
    let profile = load_profile_by_id(home_dir, id)?;

    // Read current config.yml, update only modelRoles
    let mut config = read_config_yml(home_dir)?;
    config.model_roles = Some(profile.model_roles);
    write_config_yml(home_dir, &config)?;
    set_active_omp_profile_id_for_home(home_dir, id)?;
    Ok(())
}

/// Get the status of OMP config files.
pub fn get_omp_config_status_for_home(home_dir: &Path) -> Result<OmpConfigStatus, String> {
    let config_path = omp_config_yml_path_for_home(home_dir)?;
    let models_db = omp_models_db_path_for_home(home_dir)?;
    let agent_db = omp_agent_db_path_for_home(home_dir)?;
    Ok(OmpConfigStatus {
        config_exists: config_path.exists(),
        config_path: config_path.to_string_lossy().to_string(),
        models_db_exists: models_db.exists(),
        agent_db_exists: agent_db.exists(),
    })
}

/// Read the current OMP config from all sources.
/// Returns partial results if DB reads fail (e.g. files locked by running OMP).
pub fn read_omp_current_config_for_home(home_dir: &Path) -> Result<OmpCurrentConfig, String> {
    let agent_config = read_config_yml(home_dir)?;

    let models_db_path = omp_models_db_path_for_home(home_dir)?;
    let provider_models = read_model_cache_from_db(&models_db_path).unwrap_or_default();

    let agent_db_path = omp_agent_db_path_for_home(home_dir)?;
    let credentials = read_credentials_from_db(&agent_db_path).unwrap_or_default();

    Ok(OmpCurrentConfig {
        agent_config,
        provider_models,
        credentials,
    })
}

// ============================================================================
// Connection Testing
// ============================================================================

const OMP_TEST_TIMEOUT: Duration = Duration::from_secs(30);

/// Extract the API key for a specific provider from agent.db.
#[cfg(not(test))]
fn read_api_key_for_provider(home_dir: &Path, provider_id: &str) -> Result<String, String> {
    use rusqlite::Connection;

    let path = omp_agent_db_path_for_home(home_dir)?;
    if !path.exists() {
        return Err(format!("agent.db not found at {}", path.display()));
    }

    let conn = Connection::open(&path).map_err(|e| format!("Failed to open agent.db: {e}"))?;

    let mut stmt = conn
        .prepare("SELECT data FROM auth_credentials WHERE provider = ?1")
        .map_err(|e| format!("Failed to query auth_credentials: {e}"))?;

    let mut rows = stmt
        .query([provider_id])
        .map_err(|e| format!("Failed to query auth_credentials for '{provider_id}': {e}"))?;

    if let Some(row) = rows
        .next()
        .map_err(|e| format!("Failed to read auth_credentials: {e}"))?
    {
        let data: String = row
            .get(0)
            .map_err(|e| format!("Failed to read data column: {e}"))?;
        let data = data.trim().to_string();
        if data.is_empty() || data == "null" {
            return Err(format!(
                "No API key configured for provider '{provider_id}'"
            ));
        }
        // Data might be JSON like {"type":"api_key","api_key":"sk-..."} or plain key
        if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(&data) {
            if let Some(key) = parsed.get("api_key").and_then(|v| v.as_str()) {
                return Ok(key.to_string());
            }
        }
        return Ok(data);
    }

    Err(format!(
        "No credentials found for provider '{provider_id}' in agent.db"
    ))
}

#[cfg(test)]
fn read_api_key_for_provider(_home_dir: &Path, _provider_id: &str) -> Result<String, String> {
    Err("API key reading not available in tests".to_string())
}

/// Find the base URL and a model ID for a provider from models.db.
fn find_provider_info(home_dir: &Path, provider_id: &str) -> Result<(String, String), String> {
    let models_db_path = omp_models_db_path_for_home(home_dir)?;
    let provider_models = read_model_cache_from_db(&models_db_path)?;

    let pm = provider_models
        .iter()
        .find(|pm| pm.provider_id == provider_id)
        .ok_or_else(|| format!("Provider '{provider_id}' not found in models.db"))?;

    let model = pm
        .models
        .first()
        .ok_or_else(|| format!("No models found for provider '{provider_id}' in models.db"))?;

    let base_url = model
        .base_url
        .clone()
        .unwrap_or_else(|| format!("https://api.{provider_id}.com"));

    Ok((base_url, model.id.clone()))
}

/// Test an OMP provider by making an HTTP request to its API endpoint.
pub fn test_omp_provider_connection_for_home(
    home_dir: &Path,
    provider_id: &str,
) -> Result<OmpProviderTestResult, String> {
    let provider_id = provider_id.trim().to_string();
    if provider_id.is_empty() {
        return Err("OMP provider ID cannot be empty".to_string());
    }

    let (base_url, model_id) = find_provider_info(home_dir, &provider_id)?;
    let api_key = read_api_key_for_provider(home_dir, &provider_id)?;

    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .map_err(|e| format!("Failed to create tokio runtime: {e}"))?;

    let tester = crate::connectivity::ModelTester::new();
    let base_url_clone = base_url.clone();
    let provider_id_clone = provider_id.clone();
    let model_id_clone = model_id.clone();

    let diagnostics = runtime.block_on(async {
        tokio::time::timeout(
            OMP_TEST_TIMEOUT,
            tester.test_model_direct(
                &provider_id_clone,
                &base_url_clone,
                &api_key,
                &model_id_clone,
            ),
        )
        .await
        .unwrap_or_else(|_| crate::connectivity::ConnectionDiagnostics {
            success: false,
            provider: provider_id_clone.clone(),
            model_id: model_id_clone.clone(),
            latency_ms: OMP_TEST_TIMEOUT.as_millis() as u32,
            error: Some(format!(
                "OMP connection test timed out after {} seconds",
                OMP_TEST_TIMEOUT.as_secs()
            )),
            timestamp: Utc::now().to_rfc3339(),
            test_mode: crate::connectivity::TestMode::Ping,
            response_text: None,
            prompt_used: None,
        })
    });

    Ok(OmpProviderTestResult {
        success: diagnostics.success,
        provider_id,
        model_id: diagnostics.model_id,
        latency_ms: diagnostics.latency_ms as u64,
        response_text: diagnostics.response_text,
        error: diagnostics.error,
    })
}

pub fn test_omp_provider_connection(provider_id: &str) -> Result<OmpProviderTestResult, String> {
    test_omp_provider_connection_for_home(&system_home_dir()?, provider_id)
}

// ============================================================================
// System wrappers (CRUD)
// ============================================================================

pub fn list_omp_profiles() -> Result<Vec<OmpProfile>, String> {
    list_omp_profiles_for_home(&system_home_dir()?)
}

pub fn get_omp_profile(id: &str) -> Result<OmpProfile, String> {
    get_omp_profile_for_home(&system_home_dir()?, id)
}

pub fn save_omp_profile(profile: OmpProfile) -> Result<(), String> {
    save_omp_profile_for_home(&system_home_dir()?, profile)
}

pub fn delete_omp_profile(id: &str) -> Result<(), String> {
    delete_omp_profile_for_home(&system_home_dir()?, id)
}

pub fn duplicate_omp_profile(id: &str, new_name: &str) -> Result<OmpProfile, String> {
    duplicate_omp_profile_for_home(&system_home_dir()?, id, new_name)
}

pub fn create_default_omp_profile() -> Result<OmpProfile, String> {
    create_default_omp_profile_for_home(&system_home_dir()?)
}

pub fn get_active_omp_profile_id() -> Result<Option<String>, String> {
    get_active_omp_profile_id_for_home(&system_home_dir()?)
}

pub fn set_active_omp_profile_id(id: &str) -> Result<(), String> {
    set_active_omp_profile_id_for_home(&system_home_dir()?, id)
}

pub fn apply_omp_profile(id: &str) -> Result<(), String> {
    apply_omp_profile_for_home(&system_home_dir()?, id)
}

pub fn get_omp_config_status() -> Result<OmpConfigStatus, String> {
    get_omp_config_status_for_home(&system_home_dir()?)
}

pub fn read_omp_current_config() -> Result<OmpCurrentConfig, String> {
    read_omp_current_config_for_home(&system_home_dir()?)
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

    fn make_profile(id: &str, name: &str) -> OmpProfile {
        OmpProfile {
            id: id.to_string(),
            name: name.to_string(),
            description: None,
            created_at: "2026-01-01T00:00:00Z".to_string(),
            updated_at: "2026-01-01T00:00:00Z".to_string(),
            model_roles: OmpModelRoles::default(),
        }
    }

    fn make_profile_with_roles(id: &str, name: &str) -> OmpProfile {
        OmpProfile {
            id: id.to_string(),
            name: name.to_string(),
            description: Some("A test profile".to_string()),
            created_at: "2026-01-01T00:00:00Z".to_string(),
            updated_at: "2026-01-01T00:00:00Z".to_string(),
            model_roles: OmpModelRoles {
                default: Some("anthropic/claude-sonnet-4-5".to_string()),
                smol: Some("anthropic/claude-haiku-4-5".to_string()),
                slow: Some("anthropic/claude-opus-4-6".to_string()),
                ..Default::default()
            },
        }
    }

    #[test]
    fn test_path_helpers() {
        let temp = TempDir::new().unwrap();
        let home = temp.path();

        let profiles = profiles_dir_for_home(home).unwrap();
        assert!(profiles.ends_with(".droidgear/omp/profiles"));

        let active = active_profile_path_for_home(home).unwrap();
        assert!(active.ends_with(".droidgear/omp/active-profile.txt"));
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
    fn test_profile_serialization() {
        let profile = make_profile_with_roles("test-id", "Test Profile");
        let json = serde_json::to_string_pretty(&profile).unwrap();
        assert!(json.contains("\"id\": \"test-id\""));
        assert!(json.contains("\"modelRoles\""));
        assert!(json.contains("\"default\": \"anthropic/claude-sonnet-4-5\""));
    }

    #[test]
    fn test_config_yml_roundtrip() {
        let temp = TempDir::new().unwrap();
        let home = temp.path();

        let config = OmpAgentConfig {
            model_roles: Some(OmpModelRoles {
                default: Some("xiaomi-token-plan-cn/mimo-v2.5".to_string()),
                ..Default::default()
            }),
            theme: Some(OmpTheme {
                dark: Some("titanium".to_string()),
                ..Default::default()
            }),
            ..Default::default()
        };

        write_config_yml(home, &config).unwrap();
        let loaded = read_config_yml(home).unwrap();

        assert_eq!(
            loaded.model_roles.as_ref().unwrap().default.as_deref(),
            Some("xiaomi-token-plan-cn/mimo-v2.5")
        );
        assert_eq!(
            loaded.theme.as_ref().unwrap().dark.as_deref(),
            Some("titanium")
        );
    }

    #[test]
    fn test_config_yml_missing_file() {
        let temp = TempDir::new().unwrap();
        let home = temp.path();
        let config = read_config_yml(home).unwrap();
        assert!(config.model_roles.is_none());
    }

    #[test]
    fn test_config_yml_overwrite_model_roles() {
        let temp = TempDir::new().unwrap();
        let home = temp.path();

        // Write initial config with theme
        let initial = OmpAgentConfig {
            model_roles: Some(OmpModelRoles {
                default: Some("old-model".to_string()),
                ..Default::default()
            }),
            theme: Some(OmpTheme {
                dark: Some("catppuccin".to_string()),
                ..Default::default()
            }),
            ..Default::default()
        };
        write_config_yml(home, &initial).unwrap();

        // Overwrite only model roles
        let mut config = read_config_yml(home).unwrap();
        config.model_roles = Some(OmpModelRoles {
            default: Some("new-model".to_string()),
            slow: Some("slow-model".to_string()),
            ..Default::default()
        });
        write_config_yml(home, &config).unwrap();

        let loaded = read_config_yml(home).unwrap();
        assert_eq!(
            loaded.model_roles.as_ref().unwrap().default.as_deref(),
            Some("new-model")
        );
        assert_eq!(
            loaded.model_roles.as_ref().unwrap().slow.as_deref(),
            Some("slow-model")
        );
        // Theme should be preserved
        assert_eq!(
            loaded.theme.as_ref().unwrap().dark.as_deref(),
            Some("catppuccin")
        );
    }

    #[test]
    fn test_create_default_profile() {
        let temp = TempDir::new().unwrap();
        let home = home(&temp);

        let profile = create_default_omp_profile_for_home(home).unwrap();
        assert!(!profile.id.is_empty());
        assert_eq!(profile.name, "Default");
        assert!(profile.model_roles.default.is_none());

        let err = create_default_omp_profile_for_home(home).unwrap_err();
        assert_eq!(err, "Profiles already exist");
    }

    #[test]
    fn test_save_and_get_profile() {
        let temp = TempDir::new().unwrap();
        let home = home(&temp);

        let profile = make_profile("p1", "Profile 1");
        save_omp_profile_for_home(home, profile).unwrap();

        let loaded = get_omp_profile_for_home(home, "p1").unwrap();
        assert_eq!(loaded.id, "p1");
        assert_eq!(loaded.name, "Profile 1");
    }

    #[test]
    fn test_list_profiles_sorted_by_name() {
        let temp = TempDir::new().unwrap();
        let home = home(&temp);

        save_omp_profile_for_home(home, make_profile("p1", "Zebra")).unwrap();
        save_omp_profile_for_home(home, make_profile("p2", "Alpha")).unwrap();
        save_omp_profile_for_home(home, make_profile("p3", "Middle")).unwrap();

        let profiles = list_omp_profiles_for_home(home).unwrap();
        assert_eq!(profiles.len(), 3);
        assert_eq!(profiles[0].name, "Alpha");
        assert_eq!(profiles[1].name, "Middle");
        assert_eq!(profiles[2].name, "Zebra");
    }

    #[test]
    fn test_delete_profile() {
        let temp = TempDir::new().unwrap();
        let home = home(&temp);

        save_omp_profile_for_home(home, make_profile("p1", "Profile 1")).unwrap();
        assert_eq!(list_omp_profiles_for_home(home).unwrap().len(), 1);

        delete_omp_profile_for_home(home, "p1").unwrap();
        assert_eq!(list_omp_profiles_for_home(home).unwrap().len(), 0);
    }

    #[test]
    fn test_active_profile_get_set() {
        let temp = TempDir::new().unwrap();
        let home = home(&temp);

        let active = get_active_omp_profile_id_for_home(home).unwrap();
        assert!(active.is_none());

        set_active_omp_profile_id_for_home(home, "p1").unwrap();
        let active = get_active_omp_profile_id_for_home(home).unwrap();
        assert_eq!(active.as_deref(), Some("p1"));
    }

    #[test]
    fn test_apply_profile_writes_config_yml() {
        let temp = TempDir::new().unwrap();
        let home = home(&temp);

        let profile = make_profile_with_roles("p1", "Test");
        save_omp_profile_for_home(home, profile).unwrap();

        apply_omp_profile_for_home(home, "p1").unwrap();

        let config = read_config_yml(home).unwrap();
        assert_eq!(
            config.model_roles.as_ref().unwrap().default.as_deref(),
            Some("anthropic/claude-sonnet-4-5")
        );
        assert_eq!(
            config.model_roles.as_ref().unwrap().slow.as_deref(),
            Some("anthropic/claude-opus-4-6")
        );

        let active = get_active_omp_profile_id_for_home(home).unwrap();
        assert_eq!(active.as_deref(), Some("p1"));
    }

    #[test]
    fn test_config_status() {
        let temp = TempDir::new().unwrap();
        let home = home(&temp);

        let status = get_omp_config_status_for_home(home).unwrap();
        assert!(!status.config_exists);
        assert!(!status.models_db_exists);
        assert!(!status.agent_db_exists);
    }

    #[test]
    fn test_read_current_config_missing_files() {
        let temp = TempDir::new().unwrap();
        let home = home(&temp);

        let config = read_omp_current_config_for_home(home).unwrap();
        assert!(config.agent_config.model_roles.is_none());
        assert!(config.provider_models.is_empty());
        assert!(config.credentials.is_empty());
    }
}
