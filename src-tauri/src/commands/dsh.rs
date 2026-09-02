//! Dsh (DeepSeek Harness) configuration management commands (Tauri wrappers).
//!
//! Core logic lives in `droidgear-core`.

pub use droidgear_core::dsh::{
    DshConfigStatus, DshCredentials, DshCurrentConfig, DshModel, DshProviderConfig,
};

/// Read the current Dsh providers from `~/.dsh/settings.yaml`.
#[tauri::command]
#[specta::specta]
pub async fn read_dsh_current_config() -> Result<DshCurrentConfig, String> {
    droidgear_core::dsh::read_dsh_current_config()
}

/// Insert or update one provider in `llm-pi-ai.providers`.
#[tauri::command]
#[specta::specta]
pub async fn save_dsh_provider(
    provider_id: String,
    config: DshProviderConfig,
) -> Result<(), String> {
    droidgear_core::dsh::save_dsh_provider(&provider_id, &config)
}

/// Remove one provider from `llm-pi-ai.providers`.
#[tauri::command]
#[specta::specta]
pub async fn delete_dsh_provider(provider_id: String) -> Result<(), String> {
    droidgear_core::dsh::delete_dsh_provider(&provider_id)
}

/// Get the status of `~/.dsh/settings.yaml`.
#[tauri::command]
#[specta::specta]
pub async fn get_dsh_config_status() -> Result<DshConfigStatus, String> {
    droidgear_core::dsh::get_dsh_config_status()
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
