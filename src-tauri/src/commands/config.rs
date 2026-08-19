//! Factory settings.json management commands (Tauri wrappers).
//!
//! Core logic lives in `droidgear-core`.

pub use droidgear_core::factory_settings::{
    CustomModel, MissionModelSettings, ModelInfo, Provider, SessionDefaultSettings,
};
use std::collections::HashMap;
use tauri::AppHandle;

/// Gets the path to the Factory config file
#[tauri::command]
#[specta::specta]
pub fn get_config_path() -> Result<String, String> {
    droidgear_core::factory_settings::get_config_path()
}

/// Resets the config file to an empty JSON object
#[tauri::command]
#[specta::specta]
pub async fn reset_config_file() -> Result<(), String> {
    droidgear_core::factory_settings::reset_config_file()
}

/// Loads custom models from settings.json
#[tauri::command]
#[specta::specta]
pub async fn load_custom_models() -> Result<Vec<CustomModel>, String> {
    droidgear_core::factory_settings::load_custom_models()
}

/// Saves custom models to settings.json (preserves other fields)
#[tauri::command]
#[specta::specta]
pub async fn save_custom_models(models: Vec<CustomModel>) -> Result<(), String> {
    droidgear_core::factory_settings::save_custom_models(models)
}

/// Loads modelFavorites from settings.json.
#[tauri::command]
#[specta::specta]
pub async fn get_model_favorites() -> Result<Vec<String>, String> {
    droidgear_core::factory_settings::get_model_favorites()
}

/// Saves modelFavorites to settings.json (preserves other fields).
#[tauri::command]
#[specta::specta]
pub async fn save_model_favorites(favorites: Vec<String>) -> Result<(), String> {
    droidgear_core::factory_settings::save_model_favorites(favorites)
}

/// Checks if legacy config.json exists and settings.json has customModels
#[tauri::command]
#[specta::specta]
pub async fn check_legacy_config() -> Result<bool, String> {
    droidgear_core::factory_settings::check_legacy_config()
}

/// Deletes the legacy config.json file
#[tauri::command]
#[specta::specta]
pub async fn delete_legacy_config() -> Result<(), String> {
    droidgear_core::factory_settings::delete_legacy_config()
}

/// Fetches available models from a provider API
#[tauri::command]
#[specta::specta]
pub async fn fetch_models(
    _app: AppHandle,
    provider: Provider,
    base_url: String,
    api_key: String,
) -> Result<Vec<ModelInfo>, String> {
    droidgear_core::factory_settings::fetch_models(provider, &base_url, &api_key).await
}

/// Gets the default model ID from sessionDefaultSettings.model
#[tauri::command]
#[specta::specta]
pub async fn get_default_model() -> Result<Option<String>, String> {
    droidgear_core::factory_settings::get_default_model()
}

/// Saves the default model ID to sessionDefaultSettings.model
#[tauri::command]
#[specta::specta]
pub async fn save_default_model(model_id: String) -> Result<(), String> {
    droidgear_core::factory_settings::save_default_model(&model_id)
}

/// Gets the cloudSessionSync setting from settings.json
/// Returns true by default if not set
#[tauri::command]
#[specta::specta]
pub async fn get_cloud_session_sync() -> Result<bool, String> {
    droidgear_core::factory_settings::get_cloud_session_sync()
}

/// Saves the cloudSessionSync setting to settings.json
#[tauri::command]
#[specta::specta]
pub async fn save_cloud_session_sync(enabled: bool) -> Result<(), String> {
    droidgear_core::factory_settings::save_cloud_session_sync(enabled)
}

/// Gets the reasoningEffort setting from settings.json
/// Returns None if not set (model-dependent default)
#[tauri::command]
#[specta::specta]
pub async fn get_reasoning_effort() -> Result<Option<String>, String> {
    droidgear_core::factory_settings::get_reasoning_effort()
}

/// Saves the reasoningEffort setting to settings.json
#[tauri::command]
#[specta::specta]
pub async fn save_reasoning_effort(value: String) -> Result<(), String> {
    droidgear_core::factory_settings::save_reasoning_effort(&value)
}

/// Gets the diffMode setting from settings.json
/// Returns "github" by default if not set
#[tauri::command]
#[specta::specta]
pub async fn get_diff_mode() -> Result<String, String> {
    droidgear_core::factory_settings::get_diff_mode()
}

/// Saves the diffMode setting to settings.json
#[tauri::command]
#[specta::specta]
pub async fn save_diff_mode(value: String) -> Result<(), String> {
    droidgear_core::factory_settings::save_diff_mode(&value)
}

/// Gets the todoDisplayMode setting from settings.json
/// Returns "pinned" by default if not set
#[tauri::command]
#[specta::specta]
pub async fn get_todo_display_mode() -> Result<String, String> {
    droidgear_core::factory_settings::get_todo_display_mode()
}

/// Saves the todoDisplayMode setting to settings.json
#[tauri::command]
#[specta::specta]
pub async fn save_todo_display_mode(value: String) -> Result<(), String> {
    droidgear_core::factory_settings::save_todo_display_mode(&value)
}

/// Gets the includeCoAuthoredByDroid setting from settings.json
/// Returns true by default if not set
#[tauri::command]
#[specta::specta]
pub async fn get_include_co_authored_by_droid() -> Result<bool, String> {
    droidgear_core::factory_settings::get_include_co_authored_by_droid()
}

/// Saves the includeCoAuthoredByDroid setting to settings.json
#[tauri::command]
#[specta::specta]
pub async fn save_include_co_authored_by_droid(enabled: bool) -> Result<(), String> {
    droidgear_core::factory_settings::save_include_co_authored_by_droid(enabled)
}

/// Gets the showThinkingInMainView setting from settings.json
/// Returns false by default if not set
#[tauri::command]
#[specta::specta]
pub async fn get_show_thinking_in_main_view() -> Result<bool, String> {
    droidgear_core::factory_settings::get_show_thinking_in_main_view()
}

/// Saves the showThinkingInMainView setting to settings.json
#[tauri::command]
#[specta::specta]
pub async fn save_show_thinking_in_main_view(enabled: bool) -> Result<(), String> {
    droidgear_core::factory_settings::save_show_thinking_in_main_view(enabled)
}

/// Gets the missionModelSettings from settings.json
/// Returns empty settings if not set
#[tauri::command]
#[specta::specta]
pub async fn get_mission_model_settings() -> Result<MissionModelSettings, String> {
    droidgear_core::factory_settings::get_mission_model_settings()
}

/// Saves the missionModelSettings to settings.json
#[tauri::command]
#[specta::specta]
pub async fn save_mission_model_settings(settings: MissionModelSettings) -> Result<(), String> {
    droidgear_core::factory_settings::save_mission_model_settings(settings)
}

/// Gets the sessionDefaultSettings from settings.json
/// Returns empty settings if not set
#[tauri::command]
#[specta::specta]
pub async fn get_session_default_settings() -> Result<SessionDefaultSettings, String> {
    droidgear_core::factory_settings::get_session_default_settings()
}

/// Saves the sessionDefaultSettings to settings.json
#[tauri::command]
#[specta::specta]
pub async fn save_session_default_settings(settings: SessionDefaultSettings) -> Result<(), String> {
    droidgear_core::factory_settings::save_session_default_settings(settings)
}

/// Gets the compactionModelMode setting from settings.json
/// Returns "current-model" by default if not set
#[tauri::command]
#[specta::specta]
pub async fn get_compaction_model_mode() -> Result<String, String> {
    droidgear_core::factory_settings::get_compaction_model_mode()
}

/// Saves the compactionModelMode setting to settings.json
#[tauri::command]
#[specta::specta]
pub async fn save_compaction_model_mode(value: String) -> Result<(), String> {
    droidgear_core::factory_settings::save_compaction_model_mode(&value)
}

/// Gets the compactionTokenLimit setting from settings.json
/// Returns 200000 by default if not set
#[tauri::command]
#[specta::specta]
pub async fn get_compaction_token_limit() -> Result<i32, String> {
    droidgear_core::factory_settings::get_compaction_token_limit()
}

/// Saves the compactionTokenLimit setting to settings.json
#[tauri::command]
#[specta::specta]
pub async fn save_compaction_token_limit(value: i32) -> Result<(), String> {
    droidgear_core::factory_settings::save_compaction_token_limit(value)
}

/// Gets the compactionTokenLimitPerModel setting from settings.json
/// Returns empty map by default if not set
#[tauri::command]
#[specta::specta]
pub async fn get_compaction_token_limit_per_model() -> Result<HashMap<String, i32>, String> {
    droidgear_core::factory_settings::get_compaction_token_limit_per_model()
}

/// Saves the compactionTokenLimitPerModel setting to settings.json
#[tauri::command]
#[specta::specta]
pub async fn save_compaction_token_limit_per_model(
    overrides: HashMap<String, i32>,
) -> Result<(), String> {
    droidgear_core::factory_settings::save_compaction_token_limit_per_model(overrides)
}
