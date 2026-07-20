//! Codex CLI 配置管理命令（Tauri wrappers）。
//!
//! Core logic lives in `droidgear-core`.

pub use droidgear_core::codex::{
    CodexConfigStatus, CodexCurrentConfig, CodexProfile, CodexTempRun,
};

/// List all Codex profiles
#[tauri::command]
#[specta::specta]
pub async fn list_codex_profiles() -> Result<Vec<CodexProfile>, String> {
    droidgear_core::codex::list_codex_profiles()
}

/// Get a profile by ID
#[tauri::command]
#[specta::specta]
pub async fn get_codex_profile(id: String) -> Result<CodexProfile, String> {
    droidgear_core::codex::get_codex_profile(&id)
}

/// Save a profile (create or update)
#[tauri::command]
#[specta::specta]
pub async fn save_codex_profile(profile: CodexProfile) -> Result<(), String> {
    droidgear_core::codex::save_codex_profile(profile)
}

/// Delete a profile
#[tauri::command]
#[specta::specta]
pub async fn delete_codex_profile(id: String) -> Result<(), String> {
    droidgear_core::codex::delete_codex_profile(&id)
}

/// Duplicate a profile
#[tauri::command]
#[specta::specta]
pub async fn duplicate_codex_profile(id: String, new_name: String) -> Result<CodexProfile, String> {
    droidgear_core::codex::duplicate_codex_profile(&id, &new_name)
}

/// Create default profile (when no profiles exist)
#[tauri::command]
#[specta::specta]
pub async fn create_default_codex_profile() -> Result<CodexProfile, String> {
    droidgear_core::codex::create_default_codex_profile()
}

/// Get active profile ID
#[tauri::command]
#[specta::specta]
pub async fn get_active_codex_profile_id() -> Result<Option<String>, String> {
    droidgear_core::codex::get_active_codex_profile_id()
}

/// Apply a profile to `~/.codex/*`
#[tauri::command]
#[specta::specta]
pub async fn apply_codex_profile(id: String) -> Result<(), String> {
    droidgear_core::codex::apply_codex_profile(&id)
}

/// Get Codex config status
#[tauri::command]
#[specta::specta]
pub async fn get_codex_config_status() -> Result<CodexConfigStatus, String> {
    droidgear_core::codex::get_codex_config_status()
}

/// Read current Codex configuration from config files
#[tauri::command]
#[specta::specta]
pub async fn read_codex_current_config() -> Result<CodexCurrentConfig, String> {
    droidgear_core::codex::read_codex_current_config()
}

/// Prepare a temporary run plan (codex -c overrides + env) without writing any files
#[tauri::command]
#[specta::specta]
pub async fn prepare_codex_temp_run(id: String) -> Result<CodexTempRun, String> {
    droidgear_core::codex::prepare_codex_temp_run(&id)
}
