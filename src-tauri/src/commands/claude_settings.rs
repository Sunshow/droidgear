//! Claude Code settings file management commands (Tauri wrappers).
//!
//! Core logic lives in `droidgear_core::claude_settings_files` and
//! `droidgear_core::claude_runtime`. These commands expose the settings file
//! manager (Global + custom) and a dual-mode launch entry point (normal vs.
//! `--dangerously-skip-permissions`).

pub use droidgear_core::claude_settings_files::ClaudeSettingsFileInfo;

use droidgear_core::{claude_runtime, claude_settings_files};

use crate::utils::preferences::load_preferences;
use crate::utils::terminal_launch::{launch_in_terminal, LaunchSpec};

fn current_launcher_program() -> Result<String, String> {
    std::env::current_exe()
        .map(|path| path.to_string_lossy().to_string())
        .map_err(|e| format!("Failed to locate current launcher executable: {e}"))
}

/// Lists every Claude settings file (global + custom).
#[tauri::command]
#[specta::specta]
pub async fn list_claude_settings_files() -> Result<Vec<ClaudeSettingsFileInfo>, String> {
    claude_settings_files::list_settings_files()
}

/// Gets the currently active settings file info.
#[tauri::command]
#[specta::specta]
pub async fn get_active_claude_settings_file() -> Result<ClaudeSettingsFileInfo, String> {
    claude_settings_files::get_active_settings_file()
}

/// Sets the active settings file. Pass null or empty string to switch to Global.
#[tauri::command]
#[specta::specta]
pub async fn set_active_claude_settings_file(
    name: Option<String>,
) -> Result<ClaudeSettingsFileInfo, String> {
    claude_settings_files::set_active_settings_file(name)
}

/// Creates a new custom settings file.
#[tauri::command]
#[specta::specta]
pub async fn create_claude_settings_file(
    name: String,
    copy_from_active: bool,
) -> Result<ClaudeSettingsFileInfo, String> {
    claude_settings_files::create_settings_file(name, copy_from_active)
}

/// Deletes a custom settings file. The Global file cannot be deleted.
#[tauri::command]
#[specta::specta]
pub async fn delete_claude_settings_file(name: String) -> Result<(), String> {
    claude_settings_files::delete_settings_file(name)
}

/// Reads the raw JSON object stored in a settings file (by display name).
#[tauri::command]
#[specta::specta]
pub async fn read_claude_settings_file(name: String) -> Result<serde_json::Value, String> {
    claude_settings_files::read_settings_file(&name)
}

/// Persists the given JSON object as the named settings file (by display name).
#[tauri::command]
#[specta::specta]
pub async fn save_claude_settings_file(
    name: String,
    contents: serde_json::Value,
) -> Result<(), String> {
    claude_settings_files::save_settings_file(&name, &contents)
}

/// Returns a shell command string preview for launching Claude with the
/// active settings file. Useful for the "copy command" fallback.
#[tauri::command]
#[specta::specta]
pub async fn get_claude_settings_launch_command(
    skip_dangerous: bool,
) -> Result<(String, String), String> {
    let active = claude_settings_files::get_active_settings_path()?;
    let path = active.to_string_lossy().to_string();
    let mut command = format!("claude --settings \"{path}\"");
    if skip_dangerous {
        command.push_str(" --dangerously-skip-permissions");
    }
    Ok((command, path))
}

/// Launches Claude Code in a terminal using the active settings file. The
/// settings file is copied into a runtime-private directory so the live
/// configuration is never mutated. When `skip_dangerous` is true the
/// `--dangerously-skip-permissions` flag is appended.
#[tauri::command]
#[specta::specta]
pub async fn launch_claude_with_settings(
    app: tauri::AppHandle,
    cwd: Option<String>,
    skip_dangerous: bool,
) -> Result<(), String> {
    let home_dir = dirs::home_dir().ok_or_else(|| "Failed to get home directory".to_string())?;
    if let Err(error) = claude_runtime::cleanup_stale_runtime_dirs_for_home(&home_dir) {
        log::warn!("Failed to clean up stale Claude runtime directories: {error}");
    }

    let settings_path = claude_settings_files::get_active_settings_path_for_home(&home_dir)?;
    let launcher_program = current_launcher_program()?;
    let launcher_args = claude_runtime::internal_settings_launcher_args();
    let plan = claude_runtime::build_settings_launch_plan_for_home(
        &home_dir,
        &settings_path,
        skip_dangerous,
        &launcher_program,
        &launcher_args,
    )?;

    let prefs = load_preferences(&app).unwrap_or_default();
    let preferred = prefs.preferred_terminal.unwrap_or_default();

    let mut spec = build_settings_launch_spec(&plan);
    spec.cwd = cwd.map(std::path::PathBuf::from);

    launch_in_terminal(&spec, &preferred)
}

fn build_settings_launch_spec(plan: &claude_runtime::ClaudeSettingsLaunchPlan) -> LaunchSpec {
    LaunchSpec {
        program: plan.program.clone(),
        args: plan.args.clone(),
        env: plan.env.clone(),
        secret_env: plan.secret_env.clone(),
        unset_env: plan.unset_env.clone(),
        cwd: None,
        support_dir: Some(plan.runtime_dir_path.clone()),
    }
}
