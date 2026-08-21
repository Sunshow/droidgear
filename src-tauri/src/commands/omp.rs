//! OMP (Oh My Pi) configuration management commands (Tauri wrappers).
//!
//! Core logic lives in `droidgear-core`.

pub use droidgear_core::omp::{
    OmpConfigStatus, OmpCurrentConfig, OmpProfile, OmpProviderTestResult,
};

/// List all OMP profiles
#[tauri::command]
#[specta::specta]
pub async fn list_omp_profiles() -> Result<Vec<OmpProfile>, String> {
    droidgear_core::omp::list_omp_profiles()
}

/// Get a profile by ID
#[tauri::command]
#[specta::specta]
pub async fn get_omp_profile(id: String) -> Result<OmpProfile, String> {
    droidgear_core::omp::get_omp_profile(&id)
}

/// Save a profile (create or update)
#[tauri::command]
#[specta::specta]
pub async fn save_omp_profile(profile: OmpProfile) -> Result<(), String> {
    droidgear_core::omp::save_omp_profile(profile)
}

/// Delete a profile
#[tauri::command]
#[specta::specta]
pub async fn delete_omp_profile(id: String) -> Result<(), String> {
    droidgear_core::omp::delete_omp_profile(&id)
}

/// Duplicate a profile
#[tauri::command]
#[specta::specta]
pub async fn duplicate_omp_profile(id: String, new_name: String) -> Result<OmpProfile, String> {
    droidgear_core::omp::duplicate_omp_profile(&id, &new_name)
}

/// Create default profile (when no profiles exist)
#[tauri::command]
#[specta::specta]
pub async fn create_default_omp_profile() -> Result<OmpProfile, String> {
    droidgear_core::omp::create_default_omp_profile()
}

/// Get active profile ID
#[tauri::command]
#[specta::specta]
pub async fn get_active_omp_profile_id() -> Result<Option<String>, String> {
    droidgear_core::omp::get_active_omp_profile_id()
}

/// Set active profile ID
#[tauri::command]
#[specta::specta]
pub async fn set_active_omp_profile_id(id: String) -> Result<(), String> {
    droidgear_core::omp::set_active_omp_profile_id(&id)
}

/// Apply a profile to `~/.omp/agent/models.yml`
#[tauri::command]
#[specta::specta]
pub async fn apply_omp_profile(id: String) -> Result<(), String> {
    droidgear_core::omp::apply_omp_profile(&id)
}

/// Get OMP config status
#[tauri::command]
#[specta::specta]
pub async fn get_omp_config_status() -> Result<OmpConfigStatus, String> {
    droidgear_core::omp::get_omp_config_status()
}

/// Read current OMP configuration from config files
#[tauri::command]
#[specta::specta]
pub async fn read_omp_current_config() -> Result<OmpCurrentConfig, String> {
    droidgear_core::omp::read_omp_current_config()
}

/// Test an OMP provider connection via HTTP
#[tauri::command]
#[specta::specta]
pub async fn test_omp_provider_connection(
    provider_id: String,
) -> Result<OmpProviderTestResult, String> {
    tauri::async_runtime::spawn_blocking(move || {
        droidgear_core::omp::test_omp_provider_connection(&provider_id)
    })
    .await
    .map_err(|e| format!("Task join error: {e}"))?
}
