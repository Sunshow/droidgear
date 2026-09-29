//! GitHub Copilot CLI profile and BYOK launch commands.
//!
//! Profile persistence is implemented in `droidgear-core`; this module keeps
//! the Tauri surface small and uses the same terminal launcher as Codex and
//! Claude so secrets never have to be placed in a visible command line.

use droidgear_core::copilot::CopilotChannelSelection;
pub use droidgear_core::copilot::{CopilotConfigStatus, CopilotCurrentConfig, CopilotProfile};

use droidgear_core::copilot_runtime::{self, CopilotRunPlan};

use crate::utils::preferences::load_preferences;
use crate::utils::terminal_launch::{launch_in_terminal, LaunchSpec};

#[tauri::command]
#[specta::specta]
pub async fn list_copilot_profiles() -> Result<Vec<CopilotProfile>, String> {
    droidgear_core::copilot::list_copilot_profiles()
}

#[tauri::command]
#[specta::specta]
pub async fn get_copilot_profile(id: String) -> Result<CopilotProfile, String> {
    droidgear_core::copilot::get_copilot_profile(&id)
}

#[tauri::command]
#[specta::specta]
pub async fn save_copilot_profile(profile: CopilotProfile) -> Result<(), String> {
    droidgear_core::copilot::save_copilot_profile(profile)
}

#[tauri::command]
#[specta::specta]
pub async fn prepare_copilot_channel_import(
    profile: CopilotProfile,
    selection: CopilotChannelSelection,
) -> Result<CopilotProfile, String> {
    droidgear_core::copilot::prepare_channel_import(profile, selection)
}

#[tauri::command]
#[specta::specta]
pub async fn delete_copilot_profile(id: String) -> Result<(), String> {
    droidgear_core::copilot::delete_copilot_profile(&id)
}

#[tauri::command]
#[specta::specta]
pub async fn duplicate_copilot_profile(
    id: String,
    new_name: String,
) -> Result<CopilotProfile, String> {
    droidgear_core::copilot::duplicate_copilot_profile(&id, &new_name)
}

#[tauri::command]
#[specta::specta]
pub async fn create_default_copilot_profile() -> Result<CopilotProfile, String> {
    droidgear_core::copilot::create_default_copilot_profile()
}

#[tauri::command]
#[specta::specta]
pub async fn get_active_copilot_profile_id() -> Result<Option<String>, String> {
    droidgear_core::copilot::get_active_copilot_profile_id()
}

#[tauri::command]
#[specta::specta]
pub async fn apply_copilot_profile(id: String) -> Result<(), String> {
    droidgear_core::copilot::apply_copilot_profile(&id)
}

#[tauri::command]
#[specta::specta]
pub async fn get_copilot_config_status() -> Result<CopilotConfigStatus, String> {
    droidgear_core::copilot::get_copilot_config_status()
}

#[tauri::command]
#[specta::specta]
pub async fn read_copilot_current_config() -> Result<CopilotCurrentConfig, String> {
    droidgear_core::copilot::read_copilot_current_config()
}

/// Launch Copilot with the selected profile's environment overlay.
/// The live profile file is not changed by this command.  The terminal
/// launcher writes the API key into a short-lived secure wrapper when needed,
/// then removes that wrapper after Copilot exits.
#[tauri::command]
#[specta::specta]
pub async fn launch_copilot(
    id: String,
    app: tauri::AppHandle,
    cwd: Option<String>,
) -> Result<(), String> {
    let profile = droidgear_core::copilot::get_copilot_profile(&id)?;
    let plan = copilot_runtime::build_run_plan(&profile)?;
    let prefs = load_preferences(&app).unwrap_or_default();
    let preferred = prefs.preferred_terminal.unwrap_or_default();

    let mut spec = build_copilot_launch_spec(&plan);
    spec.cwd = cwd.map(std::path::PathBuf::from);
    launch_in_terminal(&spec, &preferred)
}

fn build_copilot_launch_spec(plan: &CopilotRunPlan) -> LaunchSpec {
    LaunchSpec {
        program: plan.program.clone(),
        args: plan.args.clone(),
        env: plan.env.clone(),
        secret_env: plan.secret_env.clone(),
        unset_env: plan.unset_env.clone(),
        cwd: None,
        support_dir: None,
        no_keep_open: false,
    }
}

#[cfg(test)]
mod tests {
    use super::build_copilot_launch_spec;
    use droidgear_core::copilot_runtime::CopilotRunPlan;

    #[test]
    fn build_copilot_launch_spec_keeps_secret_env_separate() {
        let spec = build_copilot_launch_spec(&CopilotRunPlan {
            program: "copilot".to_string(),
            args: vec!["--model".to_string(), "selected-model".to_string()],
            env: vec![("COPILOT_OFFLINE".to_string(), "true".to_string())],
            secret_env: vec![("COPILOT_PROVIDER_API_KEY".to_string(), "secret".to_string())],
            unset_env: vec!["COPILOT_AUTH_TOKEN".to_string()],
        });

        assert_eq!(spec.program, "copilot");
        assert_eq!(spec.args, vec!["--model", "selected-model"]);
        assert_eq!(spec.env.len(), 1);
        assert_eq!(spec.secret_env.len(), 1);
        assert_eq!(spec.unset_env, vec!["COPILOT_AUTH_TOKEN"]);
        assert!(spec.support_dir.is_none());
    }
}
