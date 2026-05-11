//! Claude Code profile management commands (Tauri wrappers).
//!
//! Core logic lives in `droidgear-core`.

pub use droidgear_core::claude::{ClaudeCodeProfile, ClaudeConfigStatus, ClaudeCurrentConfig};
use droidgear_core::claude_runtime::{self, ClaudeTemporaryLaunchPlan, ClaudeTemporaryRunPlan};

use crate::utils::login_shell::run_command_in_login_shell;
use crate::utils::preferences::load_preferences;
use crate::utils::terminal_launch::{launch_in_terminal, LaunchSpec};

fn probe_claude_cli() -> Result<(), String> {
    let version_output = run_command_in_login_shell("claude", &["--version"])?;
    if version_output.status.code() == Some(127) {
        return Err("Failed to execute claude --version: No such file or directory".to_string());
    }
    if !version_output.status.success() {
        return Err("Failed to read Claude CLI version".to_string());
    }

    Ok(())
}

fn current_launcher_program() -> Result<String, String> {
    std::env::current_exe()
        .map(|path| path.to_string_lossy().to_string())
        .map_err(|e| format!("Failed to locate current launcher executable: {e}"))
}

/// List all Claude Code profiles
#[tauri::command]
#[specta::specta]
pub async fn list_claude_profiles() -> Result<Vec<ClaudeCodeProfile>, String> {
    droidgear_core::claude::list_claude_profiles()
}

/// Get a profile by ID
#[tauri::command]
#[specta::specta]
pub async fn get_claude_profile(id: String) -> Result<ClaudeCodeProfile, String> {
    droidgear_core::claude::get_claude_profile(&id)
}

/// Save a profile (create or update)
#[tauri::command]
#[specta::specta]
pub async fn save_claude_profile(profile: ClaudeCodeProfile) -> Result<(), String> {
    droidgear_core::claude::save_claude_profile(profile)
}

/// Delete a profile
#[tauri::command]
#[specta::specta]
pub async fn delete_claude_profile(id: String) -> Result<(), String> {
    droidgear_core::claude::delete_claude_profile(&id)
}

/// Duplicate a profile
#[tauri::command]
#[specta::specta]
pub async fn duplicate_claude_profile(
    id: String,
    new_name: String,
) -> Result<ClaudeCodeProfile, String> {
    droidgear_core::claude::duplicate_claude_profile(&id, &new_name)
}

/// Create default profile (when no profiles exist)
#[tauri::command]
#[specta::specta]
pub async fn create_default_claude_profile() -> Result<ClaudeCodeProfile, String> {
    droidgear_core::claude::create_default_claude_profile()
}

/// Get active profile ID
#[tauri::command]
#[specta::specta]
pub async fn get_active_claude_profile_id() -> Result<Option<String>, String> {
    droidgear_core::claude::get_active_claude_profile_id()
}

/// Set active profile ID
#[tauri::command]
#[specta::specta]
pub async fn set_active_claude_profile_id(id: String) -> Result<(), String> {
    droidgear_core::claude::set_active_claude_profile_id(&id)
}

/// Apply a profile to `~/.claude/settings.json`
#[tauri::command]
#[specta::specta]
pub async fn apply_claude_profile(id: String) -> Result<(), String> {
    droidgear_core::claude::apply_claude_profile(&id)
}

/// Get Claude Code config status
#[tauri::command]
#[specta::specta]
pub async fn get_claude_config_status() -> Result<ClaudeConfigStatus, String> {
    droidgear_core::claude::get_claude_config_status()
}

/// Read current Claude Code configuration from settings.json
#[tauri::command]
#[specta::specta]
pub async fn read_claude_current_config() -> Result<ClaudeCurrentConfig, String> {
    droidgear_core::claude::read_claude_current_config()
}

/// Build the temporary-run launch plan preview for a Claude Code profile.
#[tauri::command]
#[specta::specta]
pub async fn get_claude_temporary_run_plan(id: String) -> Result<ClaudeTemporaryRunPlan, String> {
    let profile = droidgear_core::claude::get_claude_profile(&id)?;
    let launcher_program = current_launcher_program()?;
    let launcher_args = claude_runtime::internal_launcher_args();
    claude_runtime::build_temporary_run_preview_plan(&profile, &launcher_program, &launcher_args)
}

/// Launch Claude Code using a runtime settings overlay instead of mutating live config.
#[tauri::command]
#[specta::specta]
pub async fn launch_claude(id: String, app: tauri::AppHandle) -> Result<(), String> {
    probe_claude_cli()?;

    let home_dir = dirs::home_dir().ok_or_else(|| "Failed to get home directory".to_string())?;
    if let Err(error) = claude_runtime::cleanup_stale_runtime_dirs_for_home(&home_dir) {
        log::warn!("Failed to clean up stale Claude runtime directories: {error}");
    }

    let profile = droidgear_core::claude::get_claude_profile(&id)?;
    let launcher_program = current_launcher_program()?;
    let launcher_args = claude_runtime::internal_launcher_args();
    let plan =
        claude_runtime::build_temporary_run_plan(&profile, &launcher_program, &launcher_args)
            .map_err(|e| format!("Failed to prepare Claude temporary run: {e}"))?;
    let prefs = load_preferences(&app).unwrap_or_default();
    let preferred = prefs.preferred_terminal.unwrap_or_default();

    launch_in_terminal(&build_claude_launch_spec(&plan), &preferred)
}

fn build_claude_launch_spec(plan: &ClaudeTemporaryLaunchPlan) -> LaunchSpec {
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

#[cfg(test)]
mod tests {
    use super::build_claude_launch_spec;
    use droidgear_core::claude_runtime::ClaudeTemporaryLaunchPlan;
    use std::path::PathBuf;

    #[test]
    fn build_claude_launch_spec_preserves_args_and_runtime_support_dir() {
        let spec = build_claude_launch_spec(&ClaudeTemporaryLaunchPlan {
            program: "/tmp/droidgear-launcher".to_string(),
            args: vec![
                "__droidgear_internal".to_string(),
                "claude-launcher".to_string(),
            ],
            env: vec![(
                "CLAUDE_CONFIG_DIR".to_string(),
                "/tmp/live-claude".to_string(),
            )],
            secret_env: vec![(
                "DROIDGEAR_INTERNAL_CLAUDE_RUNTIME_JSON".to_string(),
                "{\"runtimeDirPath\":\"/tmp/runtime-claude\"}".to_string(),
            )],
            unset_env: vec!["ANTHROPIC_AUTH_TOKEN".to_string()],
            warnings: vec!["warning".to_string()],
            runtime_dir_path: PathBuf::from("/tmp/runtime-claude"),
        });

        assert_eq!(spec.program, "/tmp/droidgear-launcher");
        assert_eq!(
            spec.args,
            vec![
                "__droidgear_internal".to_string(),
                "claude-launcher".to_string(),
            ]
        );
        assert_eq!(
            spec.env,
            vec![(
                "CLAUDE_CONFIG_DIR".to_string(),
                "/tmp/live-claude".to_string(),
            )]
        );
        assert_eq!(spec.secret_env.len(), 1);
        assert_eq!(spec.unset_env, vec!["ANTHROPIC_AUTH_TOKEN".to_string()]);
        assert_eq!(spec.support_dir, Some(PathBuf::from("/tmp/runtime-claude")));
    }
}
