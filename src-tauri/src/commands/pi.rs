//! Pi Coding Agent 配置管理命令（Tauri wrappers）。
//!
//! Core logic lives in `droidgear-core`.

pub use droidgear_core::pi::{PiConfigStatus, PiCurrentConfig, PiProfile};

/// List all Pi profiles
#[tauri::command]
#[specta::specta]
pub async fn list_pi_profiles() -> Result<Vec<PiProfile>, String> {
    droidgear_core::pi::list_pi_profiles()
}

/// Get a profile by ID
#[tauri::command]
#[specta::specta]
pub async fn get_pi_profile(id: String) -> Result<PiProfile, String> {
    droidgear_core::pi::get_pi_profile(&id)
}

/// Save a profile (create or update)
#[tauri::command]
#[specta::specta]
pub async fn save_pi_profile(profile: PiProfile) -> Result<(), String> {
    droidgear_core::pi::save_pi_profile(profile)
}

/// Delete a profile
#[tauri::command]
#[specta::specta]
pub async fn delete_pi_profile(id: String) -> Result<(), String> {
    droidgear_core::pi::delete_pi_profile(&id)
}

/// Duplicate a profile
#[tauri::command]
#[specta::specta]
pub async fn duplicate_pi_profile(id: String, new_name: String) -> Result<PiProfile, String> {
    droidgear_core::pi::duplicate_pi_profile(&id, &new_name)
}

/// Create default profile (when no profiles exist)
#[tauri::command]
#[specta::specta]
pub async fn create_default_pi_profile() -> Result<PiProfile, String> {
    droidgear_core::pi::create_default_pi_profile()
}

/// Get active profile ID
#[tauri::command]
#[specta::specta]
pub async fn get_active_pi_profile_id() -> Result<Option<String>, String> {
    droidgear_core::pi::get_active_pi_profile_id()
}

/// Set active profile ID
#[tauri::command]
#[specta::specta]
pub async fn set_active_pi_profile_id(id: String) -> Result<(), String> {
    droidgear_core::pi::set_active_pi_profile_id(&id)
}

/// Apply a profile to `~/.pi/agent/models.json`
#[tauri::command]
#[specta::specta]
pub async fn apply_pi_profile(id: String) -> Result<(), String> {
    droidgear_core::pi::apply_pi_profile(&id)
}

/// Get Pi config status
#[tauri::command]
#[specta::specta]
pub async fn get_pi_config_status() -> Result<PiConfigStatus, String> {
    droidgear_core::pi::get_pi_config_status()
}

/// Read current Pi configuration from config files
#[tauri::command]
#[specta::specta]
pub async fn read_pi_current_config() -> Result<PiCurrentConfig, String> {
    droidgear_core::pi::read_pi_current_config()
}
