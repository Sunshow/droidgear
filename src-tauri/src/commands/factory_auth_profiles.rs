//! Factory Droid auth profile management commands (Tauri wrappers).
//!
//! Core logic lives in `droidgear_core::factory_auth_profiles`.

pub use droidgear_core::factory_auth_profiles::AuthProfileState;

#[tauri::command]
#[specta::specta]
pub async fn list_factory_auth_profiles() -> Result<AuthProfileState, String> {
    droidgear_core::factory_auth_profiles::list_profiles()
}

#[tauri::command]
#[specta::specta]
pub async fn get_active_factory_auth_profile() -> Result<Option<String>, String> {
    droidgear_core::factory_auth_profiles::get_active_profile()
}

#[tauri::command]
#[specta::specta]
pub async fn switch_factory_auth_profile(name: String) -> Result<(), String> {
    droidgear_core::factory_auth_profiles::switch_profile(&name)
}

#[tauri::command]
#[specta::specta]
pub async fn save_current_factory_auth_profile(name: String, label: String) -> Result<(), String> {
    droidgear_core::factory_auth_profiles::save_current_as_profile(&name, &label)
}

#[tauri::command]
#[specta::specta]
pub async fn delete_factory_auth_profile(name: String) -> Result<(), String> {
    droidgear_core::factory_auth_profiles::delete_profile(&name)
}

#[tauri::command]
#[specta::specta]
pub async fn rename_factory_auth_profile(name: String, label: String) -> Result<(), String> {
    droidgear_core::factory_auth_profiles::rename_profile(&name, &label)
}
