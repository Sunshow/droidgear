//! Codex auth profile management (core).
//!
//! Manages multiple Codex authentication profiles stored in
//! `~/.droidgear/auth-profiles/codex/`. Supports switching between
//! official codex login credentials and BYOK API keys.
//!
//! Official auth is detected by the presence of an `auth_mode` field
//! in `auth.json` (e.g., `"chatgpt"`). BYOK auth only contains
//! `OPENAI_API_KEY`.

use chrono::Utc;
use serde::{Deserialize, Serialize};
use specta::Type;
use std::path::{Path, PathBuf};

use crate::{paths, storage};

// ============================================================================
// Types
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct CodexAuthProfile {
    pub name: String,
    pub label: String,
    pub created_at: String,
    /// Whether this profile contains official auth (has auth_mode field)
    pub is_official: bool,
    /// The CodexProfile ID that was active when this auth profile was saved.
    /// When restoring, this profile will also be applied (config.toml only).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub codex_profile_id: Option<String>,
    /// Live config.toml `model` snapshotted when this auth was saved.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,
    /// Live config.toml `model_reasoning_effort` snapshotted when this auth was saved.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub model_reasoning_effort: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ProfilesManifest {
    #[serde(default)]
    active: Option<String>,
    #[serde(default)]
    profiles: Vec<CodexAuthProfile>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct CodexAuthProfileState {
    pub active: Option<String>,
    pub profiles: Vec<CodexAuthProfile>,
    pub is_current_official: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct CodexAuthConflictInfo {
    pub has_conflict: bool,
    pub current_is_official: bool,
    pub target_is_official: bool,
}

// ============================================================================
// Path Helpers
// ============================================================================

fn auth_profiles_dir_for_home(home_dir: &Path) -> PathBuf {
    paths::droidgear_dir_from_home(home_dir)
        .join("auth-profiles")
        .join("codex")
}

fn manifest_path_for_home(home_dir: &Path) -> PathBuf {
    auth_profiles_dir_for_home(home_dir).join("profiles.json")
}

fn profile_dir_for_home(home_dir: &Path, name: &str) -> PathBuf {
    auth_profiles_dir_for_home(home_dir).join(name)
}

fn auth_json_in_profile_dir_for_home(home_dir: &Path, name: &str) -> PathBuf {
    profile_dir_for_home(home_dir, name).join("auth.json")
}

fn codex_auth_path_for_home(home_dir: &Path) -> Result<PathBuf, String> {
    let config_paths = paths::load_config_paths_for_home(home_dir);
    let codex_home = paths::get_codex_home_for_home(home_dir, &config_paths)?;
    Ok(codex_home.join("auth.json"))
}

fn codex_config_path_for_home(home_dir: &Path) -> Result<PathBuf, String> {
    let config_paths = paths::load_config_paths_for_home(home_dir);
    let codex_home = paths::get_codex_home_for_home(home_dir, &config_paths)?;
    Ok(codex_home.join("config.toml"))
}

fn read_live_model_snapshot(home_dir: &Path) -> (Option<String>, Option<String>) {
    let Ok(config_path) = codex_config_path_for_home(home_dir) else {
        return (None, None);
    };
    if !config_path.exists() {
        return (None, None);
    }
    let Ok(s) = std::fs::read_to_string(&config_path) else {
        return (None, None);
    };
    if s.trim().is_empty() {
        return (None, None);
    }
    let Ok(config) = toml::from_str::<toml::map::Map<String, toml::Value>>(&s) else {
        return (None, None);
    };

    let model = config
        .get("model")
        .and_then(|v| v.as_str())
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(|s| s.to_string());
    let model_reasoning_effort = config
        .get("model_reasoning_effort")
        .and_then(|v| v.as_str())
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(|s| s.to_string());
    (model, model_reasoning_effort)
}

fn apply_auth_profile_model_to_live_config(
    home_dir: &Path,
    profile: &CodexAuthProfile,
) -> Result<(), String> {
    let model = profile
        .model
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty());
    let effort = profile
        .model_reasoning_effort
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty());

    if model.is_none() && effort.is_none() {
        return Ok(());
    }

    let config_path = codex_config_path_for_home(home_dir)?;
    if let Some(parent) = config_path.parent() {
        ensure_dir(parent)?;
    }

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

    if let Some(model) = model {
        config.insert("model".to_string(), toml::Value::String(model.to_string()));
    }
    if let Some(effort) = effort {
        config.insert(
            "model_reasoning_effort".to_string(),
            toml::Value::String(effort.to_string()),
        );
    }

    let toml_str = toml::to_string_pretty(&config)
        .map_err(|e| format!("Failed to serialize config.toml: {e}"))?;
    storage::atomic_write(&config_path, toml_str.as_bytes())
}

fn ensure_dir(dir: &Path) -> Result<(), String> {
    if !dir.exists() {
        std::fs::create_dir_all(dir)
            .map_err(|e| format!("Failed to create directory {}: {e}", dir.display()))?;
    }
    Ok(())
}

// ============================================================================
// Manifest I/O
// ============================================================================

fn read_manifest(home_dir: &Path) -> ProfilesManifest {
    let path = manifest_path_for_home(home_dir);
    if !path.exists() {
        return ProfilesManifest::default();
    }
    std::fs::read_to_string(&path)
        .ok()
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or_default()
}

fn write_manifest(home_dir: &Path, manifest: &ProfilesManifest) -> Result<(), String> {
    let path = manifest_path_for_home(home_dir);
    ensure_dir(path.parent().unwrap())?;
    let bytes = serde_json::to_string_pretty(manifest)
        .map_err(|e| format!("Failed to serialize profiles manifest: {e}"))?;
    storage::atomic_write(&path, bytes.as_bytes())
}

// ============================================================================
// Validation
// ============================================================================

fn validate_profile_name(name: &str) -> Result<(), String> {
    if name.is_empty() {
        return Err("Profile name cannot be empty".to_string());
    }
    let ok = name
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_');
    if !ok {
        return Err(
            "Profile name can only contain alphanumeric characters, hyphens, and underscores"
                .to_string(),
        );
    }
    Ok(())
}

// ============================================================================
// Auth Detection
// ============================================================================

fn is_auth_official(auth: &serde_json::Map<String, serde_json::Value>) -> bool {
    auth.contains_key("auth_mode")
}

fn read_live_auth(home_dir: &Path) -> Result<serde_json::Map<String, serde_json::Value>, String> {
    let auth_path = codex_auth_path_for_home(home_dir)?;
    if !auth_path.exists() {
        return Ok(serde_json::Map::new());
    }
    let s = std::fs::read_to_string(&auth_path)
        .map_err(|e| format!("Failed to read auth.json: {e}"))?;
    if s.trim().is_empty() {
        return Ok(serde_json::Map::new());
    }
    let v: serde_json::Value =
        serde_json::from_str(&s).map_err(|e| format!("Invalid auth.json: {e}"))?;
    match v {
        serde_json::Value::Object(map) => Ok(map),
        _ => Ok(serde_json::Map::new()),
    }
}

fn read_profile_auth(
    home_dir: &Path,
    name: &str,
) -> Result<serde_json::Map<String, serde_json::Value>, String> {
    let path = auth_json_in_profile_dir_for_home(home_dir, name);
    if !path.exists() {
        return Ok(serde_json::Map::new());
    }
    let s = std::fs::read_to_string(&path)
        .map_err(|e| format!("Failed to read profile auth.json: {e}"))?;
    if s.trim().is_empty() {
        return Ok(serde_json::Map::new());
    }
    let v: serde_json::Value =
        serde_json::from_str(&s).map_err(|e| format!("Invalid profile auth.json: {e}"))?;
    match v {
        serde_json::Value::Object(map) => Ok(map),
        _ => Ok(serde_json::Map::new()),
    }
}

fn auth_json_exists(home_dir: &Path) -> bool {
    codex_auth_path_for_home(home_dir)
        .map(|p| p.exists())
        .unwrap_or(false)
}

fn droidgear_codex_dir_for_home(home_dir: &Path) -> PathBuf {
    home_dir.join(".droidgear").join("codex")
}

fn active_codex_profile_path_for_home(home_dir: &Path) -> PathBuf {
    droidgear_codex_dir_for_home(home_dir).join("active-profile.txt")
}

fn read_active_codex_profile_id(home_dir: &Path) -> Option<String> {
    let path = active_codex_profile_path_for_home(home_dir);
    if !path.exists() {
        return None;
    }
    std::fs::read_to_string(&path)
        .ok()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
}

// ============================================================================
// Public API (_for_home variants)
// ============================================================================

pub fn list_profiles_for_home(home_dir: &Path) -> Result<CodexAuthProfileState, String> {
    let manifest = read_manifest(home_dir);

    let is_current_official = if auth_json_exists(home_dir) {
        read_live_auth(home_dir)
            .map(|auth| is_auth_official(&auth))
            .unwrap_or(false)
    } else {
        false
    };

    Ok(CodexAuthProfileState {
        active: manifest.active,
        profiles: manifest.profiles,
        is_current_official,
    })
}

pub fn is_official_auth_for_home(home_dir: &Path) -> Result<bool, String> {
    if !auth_json_exists(home_dir) {
        return Ok(false);
    }
    let auth = read_live_auth(home_dir)?;
    Ok(is_auth_official(&auth))
}

pub fn save_current_as_profile_for_home(
    home_dir: &Path,
    name: &str,
    label: &str,
) -> Result<(), String> {
    validate_profile_name(name)?;

    if !is_official_auth_for_home(home_dir)? {
        return Err(
            "Current auth is not an official login configuration. Only official login (codex login) credentials can be saved as auth profiles."
                .to_string(),
        );
    }

    let auth_path = codex_auth_path_for_home(home_dir)?;
    if !auth_path.exists() {
        return Err("No auth.json found".to_string());
    }

    let profile_dir = profile_dir_for_home(home_dir, name);
    ensure_dir(&profile_dir)?;

    let profile_auth_path = auth_json_in_profile_dir_for_home(home_dir, name);
    std::fs::copy(&auth_path, &profile_auth_path)
        .map_err(|e| format!("Failed to copy auth.json to profile: {e}"))?;

    let auth = read_live_auth(home_dir)?;
    let is_official = is_auth_official(&auth);
    let active_codex_id = read_active_codex_profile_id(home_dir);
    let (model, model_reasoning_effort) = read_live_model_snapshot(home_dir);

    let mut manifest = read_manifest(home_dir);

    if !manifest.profiles.iter().any(|p| p.name == name) {
        manifest.profiles.push(CodexAuthProfile {
            name: name.to_string(),
            label: label.to_string(),
            created_at: Utc::now().to_rfc3339(),
            is_official,
            codex_profile_id: active_codex_id,
            model,
            model_reasoning_effort,
        });
    } else if let Some(p) = manifest.profiles.iter_mut().find(|p| p.name == name) {
        p.label = label.to_string();
        p.is_official = is_official;
        p.codex_profile_id = active_codex_id;
        p.model = model;
        p.model_reasoning_effort = model_reasoning_effort;
    }

    if manifest.active.is_none() {
        manifest.active = Some(name.to_string());
    }

    write_manifest(home_dir, &manifest)
}

pub fn switch_profile_for_home(home_dir: &Path, name: &str) -> Result<(), String> {
    validate_profile_name(name)?;

    let manifest = read_manifest(home_dir);
    if !manifest.profiles.iter().any(|p| p.name == name) {
        return Err(format!("Profile '{name}' not found"));
    }

    let target_auth_path = auth_json_in_profile_dir_for_home(home_dir, name);
    if !target_auth_path.exists() {
        return Err(format!("Auth file missing for profile '{name}'"));
    }

    let live_auth_path = codex_auth_path_for_home(home_dir)?;

    // Backup current auth to the currently active profile (if any)
    if let Some(ref current_active) = manifest.active {
        if current_active != name && live_auth_path.exists() {
            let current_profile_dir = profile_dir_for_home(home_dir, current_active);
            ensure_dir(&current_profile_dir)?;
            let current_profile_auth = current_profile_dir.join("auth.json");
            std::fs::copy(&live_auth_path, &current_profile_auth).map_err(|e| {
                format!(
                    "Failed to backup current auth to profile '{}': {e}",
                    current_active
                )
            })?;

            // Update flags/snapshot for the backed-up profile
            if let Ok(auth) = read_live_auth(home_dir) {
                let is_official = is_auth_official(&auth);
                let (model, model_reasoning_effort) = read_live_model_snapshot(home_dir);
                if let Some(p) = manifest
                    .profiles
                    .iter()
                    .position(|p| p.name == *current_active)
                {
                    let mut updated_manifest = manifest.clone();
                    updated_manifest.profiles[p].is_official = is_official;
                    updated_manifest.profiles[p].model = model;
                    updated_manifest.profiles[p].model_reasoning_effort = model_reasoning_effort;
                    write_manifest(home_dir, &updated_manifest)?;
                }
            }
        }
    }

    // Copy target profile auth to live location
    let codex_dir = live_auth_path.parent().unwrap();
    ensure_dir(codex_dir)?;
    std::fs::copy(&target_auth_path, &live_auth_path)
        .map_err(|e| format!("Failed to copy profile auth to live location: {e}"))?;

    let mut manifest = read_manifest(home_dir);
    manifest.active = Some(name.to_string());

    // Apply the associated CodexProfile (config.toml only) if one was stored
    let auth_profile = manifest.profiles.iter().find(|p| p.name == name).cloned();
    let codex_profile_id = auth_profile
        .as_ref()
        .and_then(|p| p.codex_profile_id.clone());
    write_manifest(home_dir, &manifest)?;

    if let Some(profile_id) = codex_profile_id {
        // Apply config.toml only; don't touch auth.json since we just restored it
        if let Err(e) =
            crate::codex::apply_codex_profile_config_only_for_home(home_dir, &profile_id)
        {
            log::warn!(
                "Failed to apply associated CodexProfile '{}' after auth switch: {}",
                profile_id,
                e
            );
        }
    }

    // Auth-profile model snapshot wins over possibly-empty CodexProfile.model.
    if let Some(profile) = auth_profile {
        apply_auth_profile_model_to_live_config(home_dir, &profile)?;
    }

    Ok(())
}

/// Restore a saved auth profile's auth.json into live CODEX_HOME without
/// re-applying any associated CodexProfile config.toml.
/// Used when applying a Codex profile with `model_provider == "openai"` and a
/// selected `auth_profile_name`.
pub fn restore_auth_file_for_home(home_dir: &Path, name: &str) -> Result<(), String> {
    validate_profile_name(name)?;

    let manifest = read_manifest(home_dir);
    let profile = manifest
        .profiles
        .iter()
        .find(|p| p.name == name)
        .ok_or_else(|| format!("Auth profile '{name}' not found"))?;

    if !profile.is_official {
        return Err(format!(
            "Auth profile '{name}' is not an official login backup"
        ));
    }

    let target_auth_path = auth_json_in_profile_dir_for_home(home_dir, name);
    if !target_auth_path.exists() {
        return Err(format!("Auth file missing for profile '{name}'"));
    }

    let live_auth_path = codex_auth_path_for_home(home_dir)?;
    if let Some(codex_dir) = live_auth_path.parent() {
        ensure_dir(codex_dir)?;
    }
    std::fs::copy(&target_auth_path, &live_auth_path)
        .map_err(|e| format!("Failed to restore auth profile '{name}': {e}"))?;

    let mut manifest = read_manifest(home_dir);
    manifest.active = Some(name.to_string());
    write_manifest(home_dir, &manifest)?;

    // Overlay snapshotted model after auth restore.
    apply_auth_profile_model_to_live_config(home_dir, profile)?;
    Ok(())
}

/// Clear the active auth profile marker. Used when auth.json is replaced
/// externally (e.g., by CodexProfile apply with auth mode conflict), making
/// no saved profile match the current live auth.
pub fn clear_active_for_home(home_dir: &Path) -> Result<(), String> {
    let mut manifest = read_manifest(home_dir);
    manifest.active = None;
    write_manifest(home_dir, &manifest)
}

pub fn delete_profile_for_home(home_dir: &Path, name: &str) -> Result<(), String> {
    validate_profile_name(name)?;

    let mut manifest = read_manifest(home_dir);
    if !manifest.profiles.iter().any(|p| p.name == name) {
        return Err(format!("Profile '{name}' not found"));
    }

    if manifest.active.as_deref() == Some(name) {
        return Err("Cannot delete the currently active profile".to_string());
    }

    let profile_dir = profile_dir_for_home(home_dir, name);
    if profile_dir.exists() {
        std::fs::remove_dir_all(&profile_dir)
            .map_err(|e| format!("Failed to delete profile directory: {e}"))?;
    }

    manifest.profiles.retain(|p| p.name != name);
    write_manifest(home_dir, &manifest)
}

pub fn rename_profile_for_home(home_dir: &Path, name: &str, new_label: &str) -> Result<(), String> {
    validate_profile_name(name)?;

    let mut manifest = read_manifest(home_dir);
    let profile = manifest
        .profiles
        .iter_mut()
        .find(|p| p.name == name)
        .ok_or_else(|| format!("Profile '{name}' not found"))?;

    profile.label = new_label.to_string();
    write_manifest(home_dir, &manifest)
}

pub fn detect_auth_conflict_for_home(
    home_dir: &Path,
    profile_name: &str,
) -> Result<CodexAuthConflictInfo, String> {
    let current_is_official = if auth_json_exists(home_dir) {
        read_live_auth(home_dir)
            .map(|auth| is_auth_official(&auth))
            .unwrap_or(false)
    } else {
        false
    };

    let target_auth = read_profile_auth(home_dir, profile_name)?;
    let target_is_official = is_auth_official(&target_auth);

    Ok(CodexAuthConflictInfo {
        has_conflict: current_is_official != target_is_official,
        current_is_official,
        target_is_official,
    })
}

// ============================================================================
// Public API (system home wrappers)
// ============================================================================

fn system_home_dir() -> Result<PathBuf, String> {
    dirs::home_dir().ok_or_else(|| "Failed to get home directory".to_string())
}

pub fn list_profiles() -> Result<CodexAuthProfileState, String> {
    list_profiles_for_home(&system_home_dir()?)
}

pub fn is_official_auth() -> Result<bool, String> {
    is_official_auth_for_home(&system_home_dir()?)
}

pub fn save_current_as_profile(name: &str, label: &str) -> Result<(), String> {
    save_current_as_profile_for_home(&system_home_dir()?, name, label)
}

pub fn switch_profile(name: &str) -> Result<(), String> {
    switch_profile_for_home(&system_home_dir()?, name)
}

pub fn delete_profile(name: &str) -> Result<(), String> {
    delete_profile_for_home(&system_home_dir()?, name)
}

pub fn rename_profile(name: &str, new_label: &str) -> Result<(), String> {
    rename_profile_for_home(&system_home_dir()?, name, new_label)
}

pub fn detect_auth_conflict(profile_name: &str) -> Result<CodexAuthConflictInfo, String> {
    detect_auth_conflict_for_home(&system_home_dir()?, profile_name)
}

/// Check if applying a CodexProfile would cause an auth mode conflict.
/// Target official subscription mode is determined by `model_provider == "openai"`.
pub fn detect_apply_auth_conflict_for_home(
    home_dir: &Path,
    codex_profile_id: &str,
) -> Result<CodexAuthConflictInfo, String> {
    let current_is_official = if auth_json_exists(home_dir) {
        read_live_auth(home_dir)
            .map(|auth| is_auth_official(&auth))
            .unwrap_or(false)
    } else {
        false
    };

    let profile = crate::codex::get_codex_profile_for_home(home_dir, codex_profile_id)?;
    let target_is_official = profile.model_provider == "openai";

    Ok(CodexAuthConflictInfo {
        has_conflict: current_is_official != target_is_official,
        current_is_official,
        target_is_official,
    })
}

pub fn detect_apply_auth_conflict(codex_profile_id: &str) -> Result<CodexAuthConflictInfo, String> {
    detect_apply_auth_conflict_for_home(&system_home_dir()?, codex_profile_id)
}

pub fn clear_active() -> Result<(), String> {
    clear_active_for_home(&system_home_dir()?)
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn setup_official_auth(home: &Path) {
        let codex_dir = home.join(".codex");
        std::fs::create_dir_all(&codex_dir).unwrap();
        let auth = serde_json::json!({
            "auth_mode": "chatgpt",
            "OPENAI_API_KEY": "sk-test-key",
            "tokens": {
                "id_token": "test-id-token",
                "access_token": "test-access-token",
                "refresh_token": "test-refresh-token",
                "account_id": "test-account-id"
            },
            "last_refresh": "2026-06-29T00:00:00Z"
        });
        std::fs::write(
            codex_dir.join("auth.json"),
            serde_json::to_string_pretty(&auth).unwrap(),
        )
        .unwrap();
    }

    fn setup_byok_auth(home: &Path) {
        let codex_dir = home.join(".codex");
        std::fs::create_dir_all(&codex_dir).unwrap();
        let auth = serde_json::json!({
            "OPENAI_API_KEY": "sk-byok-key"
        });
        std::fs::write(
            codex_dir.join("auth.json"),
            serde_json::to_string_pretty(&auth).unwrap(),
        )
        .unwrap();
    }

    #[test]
    fn list_profiles_empty_initially() {
        let tmp = TempDir::new().unwrap();
        let state = list_profiles_for_home(tmp.path()).unwrap();
        assert!(state.profiles.is_empty());
        assert!(state.active.is_none());
        assert!(!state.is_current_official);
    }

    #[test]
    fn is_official_auth_detects_official() {
        let tmp = TempDir::new().unwrap();
        setup_official_auth(tmp.path());
        assert!(is_official_auth_for_home(tmp.path()).unwrap());
    }

    #[test]
    fn is_official_auth_rejects_byok() {
        let tmp = TempDir::new().unwrap();
        setup_byok_auth(tmp.path());
        assert!(!is_official_auth_for_home(tmp.path()).unwrap());
    }

    #[test]
    fn save_current_rejects_byok() {
        let tmp = TempDir::new().unwrap();
        setup_byok_auth(tmp.path());

        let err = save_current_as_profile_for_home(tmp.path(), "test", "Test").unwrap_err();
        assert!(err.contains("official login"));
    }

    #[test]
    fn save_and_list_official_profile() {
        let tmp = TempDir::new().unwrap();
        setup_official_auth(tmp.path());

        save_current_as_profile_for_home(tmp.path(), "acct-1", "Account 1").unwrap();

        let state = list_profiles_for_home(tmp.path()).unwrap();
        assert_eq!(state.profiles.len(), 1);
        assert_eq!(state.profiles[0].name, "acct-1");
        assert_eq!(state.profiles[0].label, "Account 1");
        assert!(state.profiles[0].is_official);
        assert_eq!(state.active, Some("acct-1".to_string()));
        assert!(state.is_current_official);
    }

    #[test]
    fn save_official_profile_snapshots_live_model() {
        let tmp = TempDir::new().unwrap();
        setup_official_auth(tmp.path());

        let codex_dir = tmp.path().join(".codex");
        std::fs::write(
            codex_dir.join("config.toml"),
            r#"
model_provider = "openai"
model = "gpt-5.4"
model_reasoning_effort = "high"
"#
            .trim_start(),
        )
        .unwrap();

        save_current_as_profile_for_home(tmp.path(), "acct-1", "Account 1").unwrap();

        let state = list_profiles_for_home(tmp.path()).unwrap();
        assert_eq!(state.profiles[0].model.as_deref(), Some("gpt-5.4"));
        assert_eq!(
            state.profiles[0].model_reasoning_effort.as_deref(),
            Some("high")
        );

        // Wipe live model then restore via switch
        std::fs::write(
            codex_dir.join("config.toml"),
            r#"
model_provider = "openai"
model = ""
"#
            .trim_start(),
        )
        .unwrap();
        switch_profile_for_home(tmp.path(), "acct-1").unwrap();

        let config = std::fs::read_to_string(codex_dir.join("config.toml")).unwrap();
        let parsed: toml::Value = toml::from_str(&config).unwrap();
        assert_eq!(
            parsed.get("model").and_then(|v| v.as_str()),
            Some("gpt-5.4")
        );
        assert_eq!(
            parsed
                .get("model_reasoning_effort")
                .and_then(|v| v.as_str()),
            Some("high")
        );
    }

    #[test]
    fn switch_profile_copies_auth() {
        let tmp = TempDir::new().unwrap();
        setup_official_auth(tmp.path());

        save_current_as_profile_for_home(tmp.path(), "acct-1", "Account 1").unwrap();

        // Modify the live auth to create a second profile
        let auth = serde_json::json!({
            "auth_mode": "chatgpt",
            "OPENAI_API_KEY": "sk-account-2",
            "tokens": {
                "id_token": "id-token-2",
                "access_token": "access-token-2",
                "refresh_token": "refresh-token-2",
                "account_id": "account-2"
            },
            "last_refresh": "2026-06-29T00:00:00Z"
        });
        let auth_path = codex_auth_path_for_home(tmp.path()).unwrap();
        std::fs::write(&auth_path, serde_json::to_string_pretty(&auth).unwrap()).unwrap();
        save_current_as_profile_for_home(tmp.path(), "acct-2", "Account 2").unwrap();

        // Switch to acct-1
        switch_profile_for_home(tmp.path(), "acct-1").unwrap();

        let live_auth = read_live_auth(tmp.path()).unwrap();
        assert_eq!(
            live_auth.get("OPENAI_API_KEY").and_then(|v| v.as_str()),
            Some("sk-test-key")
        );

        let state = list_profiles_for_home(tmp.path()).unwrap();
        assert_eq!(state.active, Some("acct-1".to_string()));
    }

    #[test]
    fn delete_profile() {
        let tmp = TempDir::new().unwrap();
        setup_official_auth(tmp.path());

        save_current_as_profile_for_home(tmp.path(), "acct-1", "Account 1").unwrap();

        let auth = serde_json::json!({
            "auth_mode": "chatgpt",
            "OPENAI_API_KEY": "sk-2",
            "tokens": { "id_token": "x", "access_token": "y", "refresh_token": "z", "account_id": "a" },
            "last_refresh": "2026-06-29T00:00:00Z"
        });
        let auth_path = codex_auth_path_for_home(tmp.path()).unwrap();
        std::fs::write(&auth_path, serde_json::to_string_pretty(&auth).unwrap()).unwrap();
        save_current_as_profile_for_home(tmp.path(), "acct-2", "Account 2").unwrap();

        switch_profile_for_home(tmp.path(), "acct-1").unwrap();
        let err = delete_profile_for_home(tmp.path(), "acct-1").unwrap_err();
        assert!(err.contains("active"));

        delete_profile_for_home(tmp.path(), "acct-2").unwrap();
        let state = list_profiles_for_home(tmp.path()).unwrap();
        assert_eq!(state.profiles.len(), 1);
    }

    #[test]
    fn rename_profile_updates_label() {
        let tmp = TempDir::new().unwrap();
        setup_official_auth(tmp.path());

        save_current_as_profile_for_home(tmp.path(), "acct-1", "Old Label").unwrap();
        rename_profile_for_home(tmp.path(), "acct-1", "New Label").unwrap();

        let state = list_profiles_for_home(tmp.path()).unwrap();
        assert_eq!(state.profiles[0].label, "New Label");
    }

    #[test]
    fn detect_conflict_between_official_and_byok() {
        let tmp = TempDir::new().unwrap();

        // Set up a BYOK live auth
        setup_byok_auth(tmp.path());

        // Create a profile dir with official auth manually
        let profile_dir = profile_dir_for_home(tmp.path(), "official-prof");
        std::fs::create_dir_all(&profile_dir).unwrap();
        let auth = serde_json::json!({
            "auth_mode": "chatgpt",
            "OPENAI_API_KEY": "sk-official",
            "tokens": { "id_token": "x", "access_token": "y", "refresh_token": "z", "account_id": "a" },
            "last_refresh": "2026-06-29T00:00:00Z"
        });
        std::fs::write(
            profile_dir.join("auth.json"),
            serde_json::to_string_pretty(&auth).unwrap(),
        )
        .unwrap();

        // Add to manifest
        let manifest = ProfilesManifest {
            active: None,
            profiles: vec![CodexAuthProfile {
                name: "official-prof".to_string(),
                label: "Official".to_string(),
                created_at: Utc::now().to_rfc3339(),
                is_official: true,
                codex_profile_id: None,
                model: None,
                model_reasoning_effort: None,
            }],
        };
        write_manifest(tmp.path(), &manifest).unwrap();

        let conflict = detect_auth_conflict_for_home(tmp.path(), "official-prof").unwrap();
        assert!(conflict.has_conflict);
        assert!(!conflict.current_is_official);
        assert!(conflict.target_is_official);
    }

    #[test]
    fn no_conflict_when_same_mode() {
        let tmp = TempDir::new().unwrap();
        setup_official_auth(tmp.path());

        save_current_as_profile_for_home(tmp.path(), "acct-1", "Account 1").unwrap();

        let conflict = detect_auth_conflict_for_home(tmp.path(), "acct-1").unwrap();
        assert!(!conflict.has_conflict);
    }
}
