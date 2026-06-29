//! Codex auth profile management commands (Tauri wrappers).
//!
//! Core logic lives in `droidgear-core`.

pub use droidgear_core::codex_auth_profiles::{CodexAuthConflictInfo, CodexAuthProfileState};

/// List all Codex auth profiles
#[tauri::command]
#[specta::specta]
pub async fn list_codex_auth_profiles() -> Result<CodexAuthProfileState, String> {
    droidgear_core::codex_auth_profiles::list_profiles()
}

/// Check if current auth is official
#[tauri::command]
#[specta::specta]
pub async fn is_codex_official_auth() -> Result<bool, String> {
    droidgear_core::codex_auth_profiles::is_official_auth()
}

/// Save current auth.json as a named profile
#[tauri::command]
#[specta::specta]
pub async fn save_current_codex_auth_profile(name: String, label: String) -> Result<(), String> {
    droidgear_core::codex_auth_profiles::save_current_as_profile(&name, &label)
}

/// Switch to a saved auth profile
#[tauri::command]
#[specta::specta]
pub async fn switch_codex_auth_profile(name: String) -> Result<(), String> {
    droidgear_core::codex_auth_profiles::switch_profile(&name)
}

/// Delete a saved auth profile
#[tauri::command]
#[specta::specta]
pub async fn delete_codex_auth_profile(name: String) -> Result<(), String> {
    droidgear_core::codex_auth_profiles::delete_profile(&name)
}

/// Rename a saved auth profile's label
#[tauri::command]
#[specta::specta]
pub async fn rename_codex_auth_profile(name: String, label: String) -> Result<(), String> {
    droidgear_core::codex_auth_profiles::rename_profile(&name, &label)
}

/// Detect auth mode conflict between current auth and a profile
#[tauri::command]
#[specta::specta]
pub async fn detect_codex_auth_conflict(
    profile_name: String,
) -> Result<CodexAuthConflictInfo, String> {
    droidgear_core::codex_auth_profiles::detect_auth_conflict(&profile_name)
}

/// Detect auth mode conflict when applying a CodexProfile (not an auth profile)
#[tauri::command]
#[specta::specta]
pub async fn detect_codex_apply_auth_conflict(
    codex_profile_id: String,
) -> Result<CodexAuthConflictInfo, String> {
    droidgear_core::codex_auth_profiles::detect_apply_auth_conflict(&codex_profile_id)
}
