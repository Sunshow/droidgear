//! Droid run planning (core).
//!
//! Builds the native `droid` launch command plus runtime env policy. Profile
//! settings files are passed directly to Droid's native `--settings` flag —
//! DroidGear does not create temporary settings snapshots.

use serde::{Deserialize, Serialize};
use specta::Type;
use std::path::{Path, PathBuf};

use crate::droid_settings_files;

#[derive(Debug, Clone, Serialize, Deserialize, Type, PartialEq, Eq, Default)]
#[serde(rename_all = "camelCase")]
pub struct DroidRunPreferences {
    #[serde(default)]
    pub disable_auto_update_env: Option<bool>,
    #[serde(default)]
    pub unset_anthropic_auth_token: Option<bool>,
}

/// A planned `droid` invocation using native CLI support.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DroidRunPlan {
    pub program: String,
    pub args: Vec<String>,
    pub env: Vec<(String, String)>,
    pub unset_env: Vec<String>,
    /// Settings path passed natively via `--settings`; `None` for the global
    /// settings file, which Droid reads by default without a flag.
    pub settings_path: Option<PathBuf>,
}

fn should_disable_auto_update_env(prefs: &DroidRunPreferences) -> bool {
    prefs.disable_auto_update_env.unwrap_or(true)
}

fn should_unset_anthropic_auth_token(prefs: &DroidRunPreferences) -> bool {
    prefs.unset_anthropic_auth_token.unwrap_or(true)
}

fn build_env_overrides(prefs: &DroidRunPreferences) -> (Vec<(String, String)>, Vec<String>) {
    let mut env = Vec::new();
    let mut unset_env = Vec::new();

    if should_disable_auto_update_env(prefs) {
        env.push((
            "FACTORY_DROID_AUTO_UPDATE_ENABLED".to_string(),
            "0".to_string(),
        ));
    }

    if should_unset_anthropic_auth_token(prefs) {
        unset_env.push("ANTHROPIC_AUTH_TOKEN".to_string());
    }

    (env, unset_env)
}

/// Builds a launch plan for the currently active settings file.
/// Global settings run as plain `droid`; custom profiles are passed directly
/// to Droid's native `--settings` flag.
pub fn build_run_plan_for_home(
    home_dir: &Path,
    prefs: &DroidRunPreferences,
) -> Result<DroidRunPlan, String> {
    let settings_path = droid_settings_files::get_active_settings_path_for_home(home_dir)?;
    build_run_plan_from_settings_path_for_home(home_dir, &settings_path, prefs)
}

/// Builds a launch plan for an explicit settings path without switching the
/// active file. The global Factory settings file runs as plain `droid`;
/// anything else is passed natively via `--settings`.
pub fn build_run_plan_from_settings_path_for_home(
    home_dir: &Path,
    settings_path: &Path,
    prefs: &DroidRunPreferences,
) -> Result<DroidRunPlan, String> {
    let (env, unset_env) = build_env_overrides(prefs);

    let is_global = settings_path == droid_settings_files::global_settings_path_for_home(home_dir);
    let (args, settings_path) = if is_global {
        (Vec::new(), None)
    } else {
        (
            vec![
                "--settings".to_string(),
                settings_path.to_string_lossy().to_string(),
            ],
            Some(settings_path.to_path_buf()),
        )
    };

    Ok(DroidRunPlan {
        program: "droid".to_string(),
        args,
        env,
        unset_env,
        settings_path,
    })
}

/// Builds a launch plan for the currently active settings file using the
/// system home directory.
pub fn build_run_plan(prefs: &DroidRunPreferences) -> Result<DroidRunPlan, String> {
    let home_dir = dirs::home_dir().ok_or_else(|| "Failed to get home directory".to_string())?;
    build_run_plan_for_home(&home_dir, prefs)
}

#[cfg(test)]
mod tests {
    use super::{
        build_run_plan_for_home, build_run_plan_from_settings_path_for_home, DroidRunPreferences,
    };
    use crate::droid_settings_files;
    use std::path::Path;
    use tempfile::TempDir;

    fn home(temp: &TempDir) -> &Path {
        temp.path()
    }

    fn write_file(path: &Path, contents: &str) {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).unwrap();
        }
        std::fs::write(path, contents).unwrap();
    }

    #[test]
    fn run_plan_passes_active_custom_profile_directly_to_settings_flag() {
        let temp = TempDir::new().unwrap();
        let active_settings_path = home(&temp).join(".droidgear/droid-settings/profile-a.json");
        write_file(
            &active_settings_path,
            r#"{"sessionDefaultSettings":{"model":"x"}}"#,
        );

        droid_settings_files::set_active_settings_file_for_home(
            home(&temp),
            Some("profile-a".to_string()),
        )
        .unwrap();

        let before = std::fs::read_to_string(&active_settings_path).unwrap();
        let plan = build_run_plan_for_home(home(&temp), &DroidRunPreferences::default()).unwrap();

        assert_eq!(plan.program, "droid");
        assert_eq!(
            plan.args,
            vec![
                "--settings".to_string(),
                active_settings_path.to_string_lossy().to_string()
            ]
        );
        assert_eq!(plan.settings_path, Some(active_settings_path.clone()));
        assert_eq!(
            plan.env,
            vec![(
                "FACTORY_DROID_AUTO_UPDATE_ENABLED".to_string(),
                "0".to_string()
            )]
        );
        assert_eq!(plan.unset_env, vec!["ANTHROPIC_AUTH_TOKEN".to_string()]);

        // The profile file is passed through untouched and no runtime
        // snapshot directory is created.
        assert_eq!(
            std::fs::read_to_string(&active_settings_path).unwrap(),
            before
        );
        assert!(!home(&temp).join(".droidgear/runtime").exists());
    }

    #[test]
    fn run_plan_runs_plain_droid_when_global_settings_are_active() {
        let temp = TempDir::new().unwrap();
        write_file(&home(&temp).join(".factory/settings.json"), "{}");

        let plan = build_run_plan_for_home(home(&temp), &DroidRunPreferences::default()).unwrap();

        assert_eq!(plan.program, "droid");
        assert!(plan.args.is_empty());
        assert!(plan.settings_path.is_none());
        assert_eq!(
            plan.env,
            vec![(
                "FACTORY_DROID_AUTO_UPDATE_ENABLED".to_string(),
                "0".to_string()
            )]
        );
        assert_eq!(plan.unset_env, vec!["ANTHROPIC_AUTH_TOKEN".to_string()]);
    }

    #[test]
    fn run_plan_can_use_an_explicit_settings_path_without_switching_active_file() {
        let temp = TempDir::new().unwrap();
        let explicit_settings_path = home(&temp).join(".droidgear/droid-settings/profile-b.json");
        write_file(
            &explicit_settings_path,
            r#"{"sessionDefaultSettings":{"model":"y"}}"#,
        );

        let plan = build_run_plan_from_settings_path_for_home(
            home(&temp),
            &explicit_settings_path,
            &DroidRunPreferences::default(),
        )
        .unwrap();

        assert_eq!(plan.program, "droid");
        assert_eq!(
            plan.args,
            vec![
                "--settings".to_string(),
                explicit_settings_path.to_string_lossy().to_string()
            ]
        );
        assert_eq!(plan.settings_path, Some(explicit_settings_path));
    }

    #[test]
    fn run_plan_treats_explicit_global_settings_path_as_plain_run() {
        let temp = TempDir::new().unwrap();
        let global_path = home(&temp).join(".factory/settings.json");
        write_file(&global_path, "{}");

        let plan = build_run_plan_from_settings_path_for_home(
            home(&temp),
            &global_path,
            &DroidRunPreferences::default(),
        )
        .unwrap();

        assert_eq!(plan.program, "droid");
        assert!(plan.args.is_empty());
        assert!(plan.settings_path.is_none());
    }

    #[test]
    fn run_plan_respects_explicit_run_policy_overrides() {
        let temp = TempDir::new().unwrap();
        let global_path = home(&temp).join(".factory/settings.json");
        write_file(&global_path, "{}");

        let plan = build_run_plan_for_home(
            home(&temp),
            &DroidRunPreferences {
                disable_auto_update_env: Some(false),
                unset_anthropic_auth_token: Some(false),
            },
        )
        .unwrap();

        assert!(plan.env.is_empty());
        assert!(plan.unset_env.is_empty());
    }
}
