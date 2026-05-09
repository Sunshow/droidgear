//! Droid temporary run planning.
//!
//! Builds a temporary settings file plus runtime env policy without mutating
//! the live Factory settings file.

use serde::{Deserialize, Serialize};
use specta::Type;
use std::path::{Path, PathBuf};
use uuid::Uuid;

use crate::{droid_settings_files, storage};

const DROID_RUNTIME_DIR: &str = "runtime/droid";
const TEMP_SETTINGS_PREFIX: &str = "temporary-run-";
const TEMP_SETTINGS_EXTENSION: &str = "json";

#[derive(Debug, Clone, Serialize, Deserialize, Type, PartialEq, Eq, Default)]
#[serde(rename_all = "camelCase")]
pub struct DroidRunPreferences {
    #[serde(default)]
    pub disable_auto_update_env: Option<bool>,
    #[serde(default)]
    pub unset_anthropic_auth_token: Option<bool>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DroidTemporaryRunPlan {
    pub program: String,
    pub args: Vec<String>,
    pub env: Vec<(String, String)>,
    pub unset_env: Vec<String>,
    pub temp_settings_path: PathBuf,
}

fn runtime_dir_for_home(home_dir: &Path) -> PathBuf {
    crate::paths::droidgear_dir_from_home(home_dir).join(DROID_RUNTIME_DIR)
}

fn next_temp_settings_path(home_dir: &Path) -> Result<PathBuf, String> {
    let runtime_dir = runtime_dir_for_home(home_dir);
    if !runtime_dir.exists() {
        std::fs::create_dir_all(&runtime_dir)
            .map_err(|e| format!("Failed to create Droid runtime directory: {e}"))?;
    }

    let timestamp = chrono::Utc::now().format("%Y%m%dT%H%M%S%.3fZ");
    Ok(runtime_dir.join(format!(
        "{TEMP_SETTINGS_PREFIX}{timestamp}-{}.{}",
        Uuid::new_v4(),
        TEMP_SETTINGS_EXTENSION
    )))
}

fn copy_settings_to_temp(source: &Path, destination: &Path) -> Result<(), String> {
    if source.exists() {
        let contents = std::fs::read(source)
            .map_err(|e| format!("Failed to read Droid settings file: {e}"))?;
        storage::atomic_write(destination, &contents)?;
    } else {
        storage::atomic_write(destination, b"{}")?;
    }

    Ok(())
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

pub fn cleanup_stale_temp_settings_for_home(home_dir: &Path) -> Result<u32, String> {
    let runtime_dir = runtime_dir_for_home(home_dir);
    if !runtime_dir.exists() {
        return Ok(0);
    }

    let cutoff = std::time::SystemTime::now()
        .checked_sub(std::time::Duration::from_secs(60 * 60 * 24))
        .ok_or_else(|| "Failed to compute Droid runtime cleanup cutoff".to_string())?;

    let mut removed = 0;

    let entries = std::fs::read_dir(&runtime_dir)
        .map_err(|e| format!("Failed to read Droid runtime directory: {e}"))?;

    for entry in entries.flatten() {
        let path = entry.path();
        let Some(name) = path.file_name().and_then(|value| value.to_str()) else {
            continue;
        };

        if !name.starts_with(TEMP_SETTINGS_PREFIX) {
            continue;
        }

        let Ok(metadata) = entry.metadata() else {
            continue;
        };
        let Ok(modified) = metadata.modified() else {
            continue;
        };

        if modified >= cutoff {
            continue;
        }

        if std::fs::remove_file(&path).is_ok() {
            removed += 1;
        }
    }

    Ok(removed)
}

pub fn build_temporary_run_plan_for_home(
    home_dir: &Path,
    prefs: &DroidRunPreferences,
) -> Result<DroidTemporaryRunPlan, String> {
    let temp_settings_path = next_temp_settings_path(home_dir)?;
    let source = droid_settings_files::get_active_settings_path_for_home(home_dir)?;
    copy_settings_to_temp(&source, &temp_settings_path)?;

    let (env, unset_env) = build_env_overrides(prefs);

    Ok(DroidTemporaryRunPlan {
        program: "droid".to_string(),
        args: vec![
            "--settings".to_string(),
            temp_settings_path.to_string_lossy().to_string(),
        ],
        env,
        unset_env,
        temp_settings_path,
    })
}

pub fn build_temporary_run_plan_from_settings_path_for_home(
    home_dir: &Path,
    settings_path: &Path,
    prefs: &DroidRunPreferences,
) -> Result<DroidTemporaryRunPlan, String> {
    let temp_settings_path = next_temp_settings_path(home_dir)?;
    copy_settings_to_temp(settings_path, &temp_settings_path)?;

    let (env, unset_env) = build_env_overrides(prefs);

    Ok(DroidTemporaryRunPlan {
        program: "droid".to_string(),
        args: vec![
            "--settings".to_string(),
            temp_settings_path.to_string_lossy().to_string(),
        ],
        env,
        unset_env,
        temp_settings_path,
    })
}

pub fn build_temporary_run_plan(
    prefs: &DroidRunPreferences,
) -> Result<DroidTemporaryRunPlan, String> {
    let home_dir = dirs::home_dir().ok_or_else(|| "Failed to get home directory".to_string())?;
    build_temporary_run_plan_for_home(&home_dir, prefs)
}

#[cfg(test)]
mod tests {
    use super::{
        build_temporary_run_plan_for_home, build_temporary_run_plan_from_settings_path_for_home,
        cleanup_stale_temp_settings_for_home, DroidRunPreferences,
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
    fn temporary_run_plan_copies_global_settings_and_applies_default_env_policy() {
        let temp = TempDir::new().unwrap();
        let global_path = home(&temp).join(".factory/settings.json");
        write_file(&global_path, r#"{"customModels":[{"id":"demo"}]}"#);

        let plan = build_temporary_run_plan_for_home(home(&temp), &DroidRunPreferences::default())
            .unwrap();

        assert_eq!(plan.program, "droid");
        assert_eq!(plan.args[0], "--settings");
        assert_eq!(plan.args[1], plan.temp_settings_path.to_string_lossy());
        assert_eq!(
            std::fs::read_to_string(&plan.temp_settings_path).unwrap(),
            r#"{"customModels":[{"id":"demo"}]}"#
        );
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
    fn temporary_run_plan_uses_active_custom_settings_file_as_base() {
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

        let plan = build_temporary_run_plan_for_home(home(&temp), &DroidRunPreferences::default())
            .unwrap();

        assert_eq!(
            std::fs::read_to_string(&plan.temp_settings_path).unwrap(),
            r#"{"sessionDefaultSettings":{"model":"x"}}"#
        );
    }

    #[test]
    fn temporary_run_plan_can_use_an_explicit_settings_path_without_switching_active_file() {
        let temp = TempDir::new().unwrap();
        let explicit_settings_path = home(&temp).join(".droidgear/droid-settings/profile-b.json");
        write_file(
            &explicit_settings_path,
            r#"{"sessionDefaultSettings":{"model":"y"}}"#,
        );

        let plan = build_temporary_run_plan_from_settings_path_for_home(
            home(&temp),
            &explicit_settings_path,
            &DroidRunPreferences::default(),
        )
        .unwrap();

        assert_eq!(
            std::fs::read_to_string(&plan.temp_settings_path).unwrap(),
            r#"{"sessionDefaultSettings":{"model":"y"}}"#
        );
    }

    #[test]
    fn temporary_run_plan_respects_explicit_run_policy_overrides() {
        let temp = TempDir::new().unwrap();
        let global_path = home(&temp).join(".factory/settings.json");
        write_file(&global_path, "{}");

        let plan = build_temporary_run_plan_for_home(
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

    #[test]
    fn temporary_run_plan_creates_empty_settings_when_base_file_is_missing() {
        let temp = TempDir::new().unwrap();

        let plan = build_temporary_run_plan_for_home(home(&temp), &DroidRunPreferences::default())
            .unwrap();

        assert_eq!(
            std::fs::read_to_string(&plan.temp_settings_path).unwrap(),
            "{}"
        );
    }

    #[test]
    fn cleanup_stale_temp_settings_only_removes_old_runtime_files() {
        let temp = TempDir::new().unwrap();
        let runtime_dir = home(&temp).join(".droidgear/runtime/droid");
        std::fs::create_dir_all(&runtime_dir).unwrap();

        let stale_file = runtime_dir.join("temporary-run-20000101T000000.000Z.json");
        let fresh_file = runtime_dir.join("temporary-run-keep.json");
        let other_file = runtime_dir.join("notes.txt");

        write_file(&stale_file, "{}");
        write_file(&fresh_file, "{}");
        write_file(&other_file, "{}");

        let stale_time = filetime::FileTime::from_unix_time(0, 0);
        filetime::set_file_mtime(&stale_file, stale_time).unwrap();

        let removed = cleanup_stale_temp_settings_for_home(home(&temp)).unwrap();

        assert_eq!(removed, 1);
        assert!(!stale_file.exists());
        assert!(fresh_file.exists());
        assert!(other_file.exists());
    }
}
