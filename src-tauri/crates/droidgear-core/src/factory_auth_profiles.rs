//! Factory Droid auth profile management (core).
//!
//! Manages multiple Droid authentication profiles stored in
//! `~/.droidgear/auth-profiles/droid/`. Supports switching between accounts.

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
pub struct AuthProfile {
    pub name: String,
    pub label: String,
    pub created_at: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ProfilesManifest {
    #[serde(default)]
    active: Option<String>,
    #[serde(default)]
    profiles: Vec<AuthProfile>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct AuthProfileState {
    pub active: Option<String>,
    pub profiles: Vec<AuthProfile>,
}

// ============================================================================
// Path Helpers
// ============================================================================

fn auth_profiles_dir_for_home(home_dir: &Path) -> PathBuf {
    paths::droidgear_dir_from_home(home_dir)
        .join("auth-profiles")
        .join("droid")
}

fn manifest_path_for_home(home_dir: &Path) -> PathBuf {
    auth_profiles_dir_for_home(home_dir).join("profiles.json")
}

fn profile_dir_for_home(home_dir: &Path, name: &str) -> PathBuf {
    auth_profiles_dir_for_home(home_dir).join(name)
}

fn factory_home_for(home_dir: &Path) -> Result<PathBuf, String> {
    let config_paths = paths::load_config_paths_for_home(home_dir);
    paths::get_factory_home_for_home(home_dir, &config_paths)
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
// Auth File Operations
// ============================================================================

const AUTH_KEY_FILE: &str = "auth.v2.key";
const AUTH_TOKEN_FILE: &str = "auth.v2.file";

fn copy_auth_files(src_dir: &Path, dst_dir: &Path) -> Result<(), String> {
    ensure_dir(dst_dir)?;

    for filename in [AUTH_KEY_FILE, AUTH_TOKEN_FILE] {
        let src = src_dir.join(filename);
        let dst = dst_dir.join(filename);
        if src.exists() {
            std::fs::copy(&src, &dst).map_err(|e| {
                format!(
                    "Failed to copy {} from {} to {}: {e}",
                    filename,
                    src_dir.display(),
                    dst_dir.display()
                )
            })?;
        }
    }
    Ok(())
}

fn auth_files_exist(dir: &Path) -> bool {
    dir.join(AUTH_KEY_FILE).exists() && dir.join(AUTH_TOKEN_FILE).exists()
}

// ============================================================================
// Public API (_for_home variants)
// ============================================================================

pub fn list_profiles_for_home(home_dir: &Path) -> Result<AuthProfileState, String> {
    let manifest = read_manifest(home_dir);
    Ok(AuthProfileState {
        active: manifest.active,
        profiles: manifest.profiles,
    })
}

pub fn get_active_profile_for_home(home_dir: &Path) -> Result<Option<String>, String> {
    Ok(read_manifest(home_dir).active)
}

pub fn save_current_as_profile_for_home(
    home_dir: &Path,
    name: &str,
    label: &str,
) -> Result<(), String> {
    validate_profile_name(name)?;

    let factory_home = factory_home_for(home_dir)?;
    if !auth_files_exist(&factory_home) {
        return Err("No active auth files found in Factory home directory".to_string());
    }

    let profile_dir = profile_dir_for_home(home_dir, name);
    copy_auth_files(&factory_home, &profile_dir)?;

    let mut manifest = read_manifest(home_dir);

    if !manifest.profiles.iter().any(|p| p.name == name) {
        manifest.profiles.push(AuthProfile {
            name: name.to_string(),
            label: label.to_string(),
            created_at: Utc::now().to_rfc3339(),
        });
    } else if let Some(p) = manifest.profiles.iter_mut().find(|p| p.name == name) {
        p.label = label.to_string();
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

    let target_dir = profile_dir_for_home(home_dir, name);
    if !auth_files_exist(&target_dir) {
        return Err(format!("Auth files missing for profile '{name}'"));
    }

    let factory_home = factory_home_for(home_dir)?;

    // Backup current auth files to the currently active profile (if any)
    if let Some(ref current_active) = manifest.active {
        if current_active != name && auth_files_exist(&factory_home) {
            let current_dir = profile_dir_for_home(home_dir, current_active);
            copy_auth_files(&factory_home, &current_dir)?;
        }
    }

    // Copy target profile auth files to factory home
    copy_auth_files(&target_dir, &factory_home)?;

    // Update manifest
    let mut manifest = manifest;
    manifest.active = Some(name.to_string());
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

// ============================================================================
// Public API (system home wrappers)
// ============================================================================

fn system_home_dir() -> Result<PathBuf, String> {
    dirs::home_dir().ok_or_else(|| "Failed to get home directory".to_string())
}

pub fn list_profiles() -> Result<AuthProfileState, String> {
    list_profiles_for_home(&system_home_dir()?)
}

pub fn get_active_profile() -> Result<Option<String>, String> {
    get_active_profile_for_home(&system_home_dir()?)
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

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn setup_factory_home(home: &Path) {
        let factory_dir = home.join(".factory");
        std::fs::create_dir_all(&factory_dir).unwrap();
        std::fs::write(factory_dir.join(AUTH_KEY_FILE), "test-key-data").unwrap();
        std::fs::write(factory_dir.join(AUTH_TOKEN_FILE), "test-token-data").unwrap();
    }

    #[test]
    fn list_profiles_empty_initially() {
        let tmp = TempDir::new().unwrap();
        let state = list_profiles_for_home(tmp.path()).unwrap();
        assert!(state.profiles.is_empty());
        assert!(state.active.is_none());
    }

    #[test]
    fn save_and_list_profile() {
        let tmp = TempDir::new().unwrap();
        setup_factory_home(tmp.path());

        save_current_as_profile_for_home(tmp.path(), "acct-1", "Account 1").unwrap();

        let state = list_profiles_for_home(tmp.path()).unwrap();
        assert_eq!(state.profiles.len(), 1);
        assert_eq!(state.profiles[0].name, "acct-1");
        assert_eq!(state.profiles[0].label, "Account 1");
        assert_eq!(state.active, Some("acct-1".to_string()));

        // Verify files were copied
        let profile_dir = profile_dir_for_home(tmp.path(), "acct-1");
        assert!(profile_dir.join(AUTH_KEY_FILE).exists());
        assert!(profile_dir.join(AUTH_TOKEN_FILE).exists());
    }

    #[test]
    fn switch_profile_copies_files() {
        let tmp = TempDir::new().unwrap();
        setup_factory_home(tmp.path());

        save_current_as_profile_for_home(tmp.path(), "acct-1", "Account 1").unwrap();

        // Create a second profile with different content
        let factory_dir = tmp.path().join(".factory");
        std::fs::write(factory_dir.join(AUTH_KEY_FILE), "key-2").unwrap();
        std::fs::write(factory_dir.join(AUTH_TOKEN_FILE), "token-2").unwrap();
        save_current_as_profile_for_home(tmp.path(), "acct-2", "Account 2").unwrap();

        // Switch to acct-1
        switch_profile_for_home(tmp.path(), "acct-1").unwrap();

        let key = std::fs::read_to_string(factory_dir.join(AUTH_KEY_FILE)).unwrap();
        assert_eq!(key, "test-key-data");

        let state = list_profiles_for_home(tmp.path()).unwrap();
        assert_eq!(state.active, Some("acct-1".to_string()));
    }

    #[test]
    fn delete_profile() {
        let tmp = TempDir::new().unwrap();
        setup_factory_home(tmp.path());

        save_current_as_profile_for_home(tmp.path(), "acct-1", "Account 1").unwrap();

        let factory_dir = tmp.path().join(".factory");
        std::fs::write(factory_dir.join(AUTH_KEY_FILE), "key-2").unwrap();
        std::fs::write(factory_dir.join(AUTH_TOKEN_FILE), "token-2").unwrap();
        save_current_as_profile_for_home(tmp.path(), "acct-2", "Account 2").unwrap();

        // Cannot delete active
        switch_profile_for_home(tmp.path(), "acct-1").unwrap();
        let err = delete_profile_for_home(tmp.path(), "acct-1").unwrap_err();
        assert!(err.contains("active"));

        // Can delete non-active
        delete_profile_for_home(tmp.path(), "acct-2").unwrap();
        let state = list_profiles_for_home(tmp.path()).unwrap();
        assert_eq!(state.profiles.len(), 1);
    }

    #[test]
    fn rename_profile_updates_label() {
        let tmp = TempDir::new().unwrap();
        setup_factory_home(tmp.path());

        save_current_as_profile_for_home(tmp.path(), "acct-1", "Old Label").unwrap();
        rename_profile_for_home(tmp.path(), "acct-1", "New Label").unwrap();

        let state = list_profiles_for_home(tmp.path()).unwrap();
        assert_eq!(state.profiles[0].label, "New Label");
    }

    #[test]
    fn invalid_profile_name_rejected() {
        let tmp = TempDir::new().unwrap();
        setup_factory_home(tmp.path());

        let err = save_current_as_profile_for_home(tmp.path(), "bad name!", "Label").unwrap_err();
        assert!(err.contains("alphanumeric"));

        let err = save_current_as_profile_for_home(tmp.path(), "", "Label").unwrap_err();
        assert!(err.contains("empty"));
    }
}
