//! Droid settings file management commands (Tauri wrappers).
//!
//! Core logic lives in `droidgear_core::droid_settings_files`.

pub use droidgear_core::droid_settings_files::SettingsFileInfo;

use droidgear_core::{droid_runtime, droid_settings_files};
use tauri::Manager;

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
    droid_settings_files::get_launch_command()
}

/// Launches Droid CLI in a terminal using a temporary settings snapshot.
#[tauri::command]
#[specta::specta]
pub async fn launch_droid(app: tauri::AppHandle) -> Result<(), String> {
    let preferred = load_preferred_terminal(&app).unwrap_or_default();
    let home_dir = dirs::home_dir().ok_or_else(|| "Failed to get home directory".to_string())?;

    if let Err(error) = droid_runtime::cleanup_stale_temp_settings_for_home(&home_dir) {
        log::warn!("Failed to clean up stale Droid temporary settings files: {error}");
    }

    let plan = droid_runtime::build_temporary_run_plan_for_home(&home_dir)?;
    let spec = build_droid_launch_spec(&plan);

    launch_in_terminal(&spec, &preferred)
}

fn build_droid_launch_spec(plan: &droid_runtime::DroidTemporaryRunPlan) -> LaunchSpec {
    LaunchSpec {
        program: plan.program.clone(),
        args: plan.args.clone(),
        env: plan.env.clone(),
        unset_env: plan.unset_env.clone(),
        cwd: None,
    }
}

fn load_preferred_terminal_from_path(prefs_path: &std::path::Path) -> Result<String, String> {
    if !prefs_path.exists() {
        return Ok(String::new());
    }

    let contents = std::fs::read_to_string(prefs_path)
        .map_err(|e| format!("Failed to read preferences: {e}"))?;
    let prefs: serde_json::Value = serde_json::from_str(&contents).map_err(|_e| String::new())?;

    Ok(prefs
        .get("preferred_terminal")
        .and_then(|value| value.as_str())
        .unwrap_or("")
        .to_string())
}

fn load_preferred_terminal(app: &tauri::AppHandle) -> Result<String, String> {
    let prefs_path = {
        let app_data_dir = app
            .path()
            .app_data_dir()
            .map_err(|e| format!("Failed to get app data dir: {e}"))?;
        app_data_dir.join("preferences.json")
    };

    load_preferred_terminal_from_path(&prefs_path)
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::{build_droid_launch_spec, load_preferred_terminal_from_path};
    use droidgear_core::droid_runtime::DroidTemporaryRunPlan;

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
        assert_eq!(spec.unset_env, vec!["ANTHROPIC_AUTH_TOKEN".to_string()]);
    }

    #[test]
    fn load_preferred_terminal_returns_empty_when_file_is_missing() {
        let path = std::env::temp_dir().join(format!(
            "droidgear-missing-prefs-{}.json",
            std::process::id()
        ));
        let _ = std::fs::remove_file(&path);

        let preferred = load_preferred_terminal_from_path(&path).unwrap();

        assert!(preferred.is_empty());
    }

    #[test]
    fn load_preferred_terminal_reads_existing_json_payload() {
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
              "preferred_terminal": "terminal"
            }"#,
        )
        .unwrap();

        let preferred = load_preferred_terminal_from_path(&path).unwrap();
        let _ = std::fs::remove_file(&path);

        assert_eq!(preferred, "terminal");
    }
}
