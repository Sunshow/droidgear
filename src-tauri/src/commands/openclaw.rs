//! OpenClaw configuration management commands (Tauri wrappers).
//!
//! Core logic lives in `droidgear-core`.

pub use droidgear_core::openclaw::{
    OpenClawConfigStatus, OpenClawCurrentConfig, OpenClawProfile, OpenClawSubAgent,
};

/// List all OpenClaw profiles
#[tauri::command]
#[specta::specta]
pub async fn list_openclaw_profiles() -> Result<Vec<OpenClawProfile>, String> {
    droidgear_core::openclaw::list_openclaw_profiles()
}

/// Get a profile by ID
#[tauri::command]
#[specta::specta]
pub async fn get_openclaw_profile(id: String) -> Result<OpenClawProfile, String> {
    droidgear_core::openclaw::get_openclaw_profile(&id)
}

/// Save a profile (create or update)
#[tauri::command]
#[specta::specta]
pub async fn save_openclaw_profile(profile: OpenClawProfile) -> Result<(), String> {
    droidgear_core::openclaw::save_openclaw_profile(profile)
}

/// Delete a profile
#[tauri::command]
#[specta::specta]
pub async fn delete_openclaw_profile(id: String) -> Result<(), String> {
    droidgear_core::openclaw::delete_openclaw_profile(&id)
}

/// Duplicate a profile
#[tauri::command]
#[specta::specta]
pub async fn duplicate_openclaw_profile(
    id: String,
    new_name: String,
) -> Result<OpenClawProfile, String> {
    droidgear_core::openclaw::duplicate_openclaw_profile(&id, &new_name)
}

/// Create default profile (when no profiles exist)
#[tauri::command]
#[specta::specta]
pub async fn create_default_openclaw_profile() -> Result<OpenClawProfile, String> {
    droidgear_core::openclaw::create_default_openclaw_profile()
}

/// Get active profile ID
#[tauri::command]
#[specta::specta]
pub async fn get_active_openclaw_profile_id() -> Result<Option<String>, String> {
    droidgear_core::openclaw::get_active_openclaw_profile_id()
}

/// Apply a profile to `~/.openclaw/openclaw.json`
#[tauri::command]
#[specta::specta]
pub async fn apply_openclaw_profile(profile: OpenClawProfile) -> Result<(), String> {
    droidgear_core::openclaw::apply_openclaw_profile(&profile)
}

/// Get OpenClaw config status
#[tauri::command]
#[specta::specta]
pub async fn get_openclaw_config_status() -> Result<OpenClawConfigStatus, String> {
    droidgear_core::openclaw::get_openclaw_config_status()
}

/// Read current OpenClaw configuration from config file
#[tauri::command]
#[specta::specta]
pub async fn read_openclaw_current_config() -> Result<OpenClawCurrentConfig, String> {
    droidgear_core::openclaw::read_openclaw_current_config()
}

/// Read subagents from OpenClaw config file
#[tauri::command]
#[specta::specta]
pub async fn read_openclaw_subagents() -> Result<Vec<OpenClawSubAgent>, String> {
    droidgear_core::openclaw::read_openclaw_subagents()
}

/// Save subagents to OpenClaw config file
#[tauri::command]
#[specta::specta]
pub async fn save_openclaw_subagents(subagents: Vec<OpenClawSubAgent>) -> Result<(), String> {
    droidgear_core::openclaw::save_openclaw_subagents(subagents)
}
