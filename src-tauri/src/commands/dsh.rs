//! Dsh (DeepSeek Harness) configuration management commands (Tauri wrappers).
//!
//! Core logic lives in `droidgear-core`.

pub use droidgear_core::dsh::{
    DshConfigStatus, DshCredentials, DshCurrentConfig, DshModel, DshProfile, DshProviderConfig,
};

/// List the Dsh profiles under `~/.dsh/profiles/`.
#[tauri::command]
#[specta::specta]
pub async fn list_dsh_profiles() -> Result<Vec<DshProfile>, String> {
    droidgear_core::dsh::list_dsh_profiles()
}

/// Read the effective Dsh providers. Without a profile the default
/// resolution is used (official `desktop` > `web` > legacy
/// `settings.yaml`).
#[tauri::command]
#[specta::specta]
pub async fn read_dsh_current_config(profile: Option<String>) -> Result<DshCurrentConfig, String> {
    droidgear_core::dsh::read_dsh_current_config_with_profile(profile)
}

/// Insert or update one provider in the target profile's `llm-pi-ai`
/// patch entry (`cordis.patch.yml`), or in the legacy `settings.yaml` when
/// no profile exists.
#[tauri::command]
#[specta::specta]
pub async fn save_dsh_provider(
    profile: Option<String>,
    provider_id: String,
    config: DshProviderConfig,
) -> Result<(), String> {
    droidgear_core::dsh::save_dsh_provider_with_profile(profile, &provider_id, &config)
}

/// Remove one provider from the target profile's `llm-pi-ai` patch entry
/// (`cordis.patch.yml`), or from the legacy `settings.yaml` when no profile
/// exists.
#[tauri::command]
#[specta::specta]
pub async fn delete_dsh_provider(
    profile: Option<String>,
    provider_id: String,
) -> Result<(), String> {
    droidgear_core::dsh::delete_dsh_provider_with_profile(profile, &provider_id)
}

/// Get the Dsh configuration file status for a profile (or the legacy
/// `settings.yaml` layout when no profile exists).
#[tauri::command]
#[specta::specta]
pub async fn get_dsh_config_status(profile: Option<String>) -> Result<DshConfigStatus, String> {
    droidgear_core::dsh::get_dsh_config_status_with_profile(profile)
}

/// Read env-var → API key refs from `~/.dsh/.credentials.yaml`.
#[tauri::command]
#[specta::specta]
pub async fn read_dsh_credentials() -> Result<DshCredentials, String> {
    droidgear_core::dsh::read_dsh_credentials()
}

/// Insert or update one credential ref (env var name → value) in
/// `~/.dsh/.credentials.yaml`. An empty value removes the entry.
#[tauri::command]
#[specta::specta]
pub async fn save_dsh_credential_ref(name: String, value: String) -> Result<(), String> {
    droidgear_core::dsh::save_dsh_credential_ref(&name, &value)
}

/// Remove one credential ref from `~/.dsh/.credentials.yaml`.
#[tauri::command]
#[specta::specta]
pub async fn delete_dsh_credential_ref(name: String) -> Result<(), String> {
    droidgear_core::dsh::delete_dsh_credential_ref(&name)
}

/// Fetch the model list from a provider's `/{baseURL}/models` endpoint using
/// the given API key, with registry metadata enrichment (reasoningEfforts,
/// contextWindow, maxTokens, name).
#[tauri::command]
#[specta::specta]
pub async fn fetch_dsh_models(
    base_url: String,
    api_key: String,
    api: Option<String>,
) -> Result<Vec<DshModel>, String> {
    droidgear_core::dsh::fetch_dsh_models(&base_url, &api_key, api.as_deref()).await
}
