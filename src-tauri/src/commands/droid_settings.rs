//! Droid settings file management commands (Tauri wrappers).
//!
//! Core logic lives in `droidgear_core::droid_settings_files`.

pub use droidgear_core::droid_settings_files::SettingsFileInfo;

use droidgear_core::{droid_runtime, droid_settings_files};

use crate::utils::preferences::load_preferences;
use crate::utils::terminal_launch::{launch_in_terminal, LaunchSpec};

/// Lists all available Droid settings files (global + custom)
#[tauri::command]
#[specta::specta]
pub async fn list_droid_settings_files() -> Result<Vec<SettingsFileInfo>, String> {
    droid_settings_files::list_settings_files()
}

/// Gets the currently active settings file info
#[tauri::command]
#[specta::specta]
pub async fn get_active_droid_settings_file() -> Result<SettingsFileInfo, String> {
    droid_settings_files::get_active_settings_file()
}

/// Sets the active settings file. Pass null or empty to switch to Global.
#[tauri::command]
#[specta::specta]
pub async fn set_active_droid_settings_file(
    name: Option<String>,
) -> Result<SettingsFileInfo, String> {
    droid_settings_files::set_active_settings_file(name)
}

/// Creates a new settings file. If copy_from_active is true, copies from the active file.
#[tauri::command]
#[specta::specta]
pub async fn create_droid_settings_file(
    name: String,
    copy_from_active: bool,
) -> Result<SettingsFileInfo, String> {
    droid_settings_files::create_settings_file(name, copy_from_active)
}

/// Deletes a custom settings file. Cannot delete the global file.
#[tauri::command]
#[specta::specta]
pub async fn delete_droid_settings_file(name: String) -> Result<(), String> {
    droid_settings_files::delete_settings_file(name)
}

/// Gets the launch command for Droid with the active settings file.
/// Returns [command_string, settings_path].
#[tauri::command]
#[specta::specta]
pub async fn get_droid_launch_command() -> Result<(String, String), String> {
    let (command, path) = droid_settings_files::get_launch_command()?;
    Ok((command, path))
}

/// Launches Droid CLI in a terminal with the active settings file.
/// Respects the user's preferredTerminal preference.
#[tauri::command]
#[specta::specta]
pub async fn launch_droid(app: tauri::AppHandle) -> Result<(), String> {
    // Read preferred terminal from preferences
    let prefs = load_preferences(&app).unwrap_or_default();
    let preferred = prefs.preferred_terminal.unwrap_or_default();
    let droid_run = prefs.droid_run.unwrap_or_default();

    let home_dir = dirs::home_dir().ok_or_else(|| "Failed to get home directory".to_string())?;
    if let Err(error) = droid_runtime::cleanup_stale_temp_settings_for_home(&home_dir) {
        log::warn!("Failed to clean up stale Droid temporary settings files: {error}");
    }
    let plan = droid_runtime::build_temporary_run_plan_for_home(&home_dir, &droid_run)?;
    let spec = build_droid_launch_spec(&plan);

    launch_in_terminal(&spec, &preferred)
}

fn build_droid_launch_spec(plan: &droid_runtime::DroidTemporaryRunPlan) -> LaunchSpec {
    LaunchSpec {
        program: plan.program.clone(),
        args: plan.args.clone(),
        env: plan.env.clone(),
        secret_env: Vec::new(),
        unset_env: plan.unset_env.clone(),
        cwd: None,
        support_dir: None,
    }
}

#[cfg(test)]
mod tests {
    use super::build_droid_launch_spec;
    use crate::utils::preferences::load_preferences_from_path;
    use droidgear_core::droid_runtime::{DroidRunPreferences, DroidTemporaryRunPlan};
    use std::path::PathBuf;

    #[test]
    fn build_droid_launch_spec_preserves_temp_run_args_and_env() {
        let spec = build_droid_launch_spec(&DroidTemporaryRunPlan {
            program: "droid".to_string(),
            args: vec![
                "--settings".to_string(),
                "/tmp/runtime/droid/temporary-run.json".to_string(),
            ],
            env: vec![(
                "FACTORY_DROID_AUTO_UPDATE_ENABLED".to_string(),
                "0".to_string(),
            )],
            unset_env: vec!["ANTHROPIC_AUTH_TOKEN".to_string()],
            temp_settings_path: PathBuf::from("/tmp/runtime/droid/temporary-run.json"),
        });

        assert_eq!(spec.program, "droid");
        assert_eq!(
            spec.args,
            vec![
                "--settings".to_string(),
                "/tmp/runtime/droid/temporary-run.json".to_string()
            ]
        );
        assert_eq!(
            spec.env,
            vec![(
                "FACTORY_DROID_AUTO_UPDATE_ENABLED".to_string(),
                "0".to_string()
            )]
        );
        assert!(spec.secret_env.is_empty());
        assert_eq!(spec.unset_env, vec!["ANTHROPIC_AUTH_TOKEN".to_string()]);
        assert!(spec.support_dir.is_none());
    }

    #[test]
    fn droid_run_preferences_default_to_unconfigured_policy() {
        let prefs = DroidRunPreferences::default();

        assert!(prefs.disable_auto_update_env.is_none());
        assert!(prefs.unset_anthropic_auth_token.is_none());
    }

    #[test]
    fn load_preferences_returns_default_when_file_is_missing() {
        let path = std::env::temp_dir().join(format!(
            "droidgear-missing-prefs-{}.json",
            std::process::id()
        ));
        let _ = std::fs::remove_file(&path);
        let prefs = load_preferences_from_path(&path).unwrap();

        assert!(prefs.preferred_terminal.is_none());
        assert!(prefs.droid_run.is_none());
    }

    #[test]
    fn load_preferences_reads_droid_run_policy_from_disk() {
        let path = std::env::temp_dir().join(format!(
            "droidgear-test-preferences-{}.json",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        std::fs::write(
            &path,
            r#"{
              "theme": "system",
              "droid_run": {
                "disableAutoUpdateEnv": false,
                "unsetAnthropicAuthToken": true
              },
              "preferred_terminal": "terminal"
            }"#,
        )
        .unwrap();

        let prefs = load_preferences_from_path(&path).unwrap();
        let _ = std::fs::remove_file(&path);

        assert_eq!(prefs.preferred_terminal.as_deref(), Some("terminal"));
        assert_eq!(
            prefs.droid_run,
            Some(DroidRunPreferences {
                disable_auto_update_env: Some(false),
                unset_anthropic_auth_token: Some(true),
            })
        );
    }
}
