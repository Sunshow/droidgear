//! Claude Code temporary-run planning.
//!
//! Builds a runtime settings overlay plus child-process env scrubbing without
//! mutating the live `settings.json`.

use serde::{Deserialize, Serialize};
use specta::Type;
use std::collections::{BTreeSet, HashMap};
use std::path::{Path, PathBuf};
use uuid::Uuid;

use crate::{claude, paths};

const CLAUDE_CONFIG_DIR_ENV: &str = "CLAUDE_CONFIG_DIR";
const CLAUDE_ENV_FILE_ENV: &str = "CLAUDE_ENV_FILE";
const CLAUDE_RUNTIME_DIR: &str = "runtime/claude";
const TEMP_RUNTIME_PREFIX: &str = "temporary-run-";

#[derive(Debug, Clone, Serialize, Deserialize, Type, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ClaudeTemporaryRunPlan {
    pub program: String,
    pub args: Vec<String>,
    pub env: Vec<(String, String)>,
    pub unset_env: Vec<String>,
    pub secret_env_keys: Vec<String>,
    pub warnings: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClaudeTemporaryLaunchPlan {
    pub program: String,
    pub args: Vec<String>,
    pub env: Vec<(String, String)>,
    pub secret_env: Vec<(String, String)>,
    pub unset_env: Vec<String>,
    pub warnings: Vec<String>,
    pub runtime_dir_path: PathBuf,
    pub settings_overlay_path: PathBuf,
    pub copied_env_file_path: Option<PathBuf>,
}

impl From<&ClaudeTemporaryLaunchPlan> for ClaudeTemporaryRunPlan {
    fn from(plan: &ClaudeTemporaryLaunchPlan) -> Self {
        Self {
            program: plan.program.clone(),
            args: plan.args.clone(),
            env: plan.env.clone(),
            unset_env: plan.unset_env.clone(),
            secret_env_keys: plan.secret_env.iter().map(|(key, _)| key.clone()).collect(),
            warnings: plan.warnings.clone(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
struct ClaudeRuntimeSettingsOverlay {
    #[serde(skip_serializing_if = "Option::is_none")]
    always_thinking_enabled: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    env: Option<HashMap<String, String>>,
}

fn runtime_dir_for_home(home_dir: &Path) -> PathBuf {
    paths::droidgear_dir_from_home(home_dir).join(CLAUDE_RUNTIME_DIR)
}

fn next_runtime_dir_path(home_dir: &Path) -> Result<PathBuf, String> {
    let runtime_dir = runtime_dir_for_home(home_dir);
    if !runtime_dir.exists() {
        std::fs::create_dir_all(&runtime_dir)
            .map_err(|e| format!("Failed to create Claude runtime directory: {e}"))?;
    }

    let timestamp = chrono::Utc::now().format("%Y%m%dT%H%M%S%.3fZ");
    Ok(runtime_dir.join(format!(
        "{TEMP_RUNTIME_PREFIX}{timestamp}-{}",
        Uuid::new_v4()
    )))
}

fn write_private_file(path: &Path, bytes: &[u8]) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        if !parent.exists() {
            std::fs::create_dir_all(parent)
                .map_err(|e| format!("Failed to create directory: {e}"))?;
        }
    }

    let temp_path = path.with_extension("tmp");
    std::fs::write(&temp_path, bytes).map_err(|e| format!("Failed to write file: {e}"))?;

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;

        let mut perms = std::fs::metadata(&temp_path)
            .map_err(|e| format!("Failed to read file metadata: {e}"))?
            .permissions();
        perms.set_mode(0o600);
        std::fs::set_permissions(&temp_path, perms)
            .map_err(|e| format!("Failed to set file permissions: {e}"))?;
    }

    std::fs::rename(&temp_path, path).map_err(|e| {
        let _ = std::fs::remove_file(&temp_path);
        format!("Failed to finalize file: {e}")
    })?;
    Ok(())
}

fn normalize_optional_env(value: Option<&String>) -> Option<String> {
    value
        .map(String::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToOwned::to_owned)
}

fn set_env_or_tombstone(env: &mut HashMap<String, String>, key: &str, value: Option<&str>) {
    env.insert(key.to_string(), value.unwrap_or_default().to_string());
}

fn apply_profile_env_settings(
    env: &mut HashMap<String, String>,
    profile: &claude::ClaudeCodeProfile,
) {
    set_env_or_tombstone(
        env,
        claude::CLAUDE_BASE_URL_ENV,
        claude::normalize_optional_string(profile.base_url.as_deref()).as_deref(),
    );
    set_env_or_tombstone(
        env,
        claude::CLAUDE_AUTH_TOKEN_ENV,
        claude::normalize_optional_string(profile.bearer_token.as_deref()).as_deref(),
    );
    set_env_or_tombstone(
        env,
        claude::CLAUDE_MODEL_ENV,
        claude::normalize_optional_string(profile.model.as_deref()).as_deref(),
    );
    let resolved_small_model = claude::resolved_small_model_value(profile);
    set_env_or_tombstone(
        env,
        claude::CLAUDE_SMALL_MODEL_ENV,
        resolved_small_model.as_deref(),
    );
}

fn apply_reasoning_settings(
    env: &mut HashMap<String, String>,
    reasoning_effort: Option<claude::ClaudeReasoningEffort>,
) {
    match reasoning_effort {
        Some(claude::ClaudeReasoningEffort::Low) | Some(claude::ClaudeReasoningEffort::Medium) => {
            env.insert(
                claude::CLAUDE_EFFORT_ENV.to_string(),
                claude::reasoning_effort_to_string(reasoning_effort.unwrap()).to_string(),
            );
            set_env_or_tombstone(env, claude::CLAUDE_DISABLE_ADAPTIVE_ENV, None);
        }
        Some(claude::ClaudeReasoningEffort::High) | Some(claude::ClaudeReasoningEffort::Max) => {
            env.insert(
                claude::CLAUDE_EFFORT_ENV.to_string(),
                claude::reasoning_effort_to_string(reasoning_effort.unwrap()).to_string(),
            );
            env.insert(
                claude::CLAUDE_DISABLE_ADAPTIVE_ENV.to_string(),
                "1".to_string(),
            );
        }
        None => {
            set_env_or_tombstone(env, claude::CLAUDE_EFFORT_ENV, None);
            set_env_or_tombstone(env, claude::CLAUDE_DISABLE_ADAPTIVE_ENV, None);
        }
    }
}

fn build_runtime_settings_overlay(
    profile: &claude::ClaudeCodeProfile,
) -> ClaudeRuntimeSettingsOverlay {
    let mut env = HashMap::new();

    apply_profile_env_settings(&mut env, profile);
    apply_reasoning_settings(&mut env, profile.reasoning_effort);

    match profile.thinking_mode {
        claude::ClaudeThinkingMode::Inherit => ClaudeRuntimeSettingsOverlay {
            always_thinking_enabled: None,
            env: {
                for key in claude::CLAUDE_CONFLICT_ENV_KEYS {
                    env.insert((*key).to_string(), String::new());
                }
                if env.is_empty() {
                    None
                } else {
                    Some(env)
                }
            },
        },
        claude::ClaudeThinkingMode::On => {
            env.insert(
                claude::CLAUDE_DISABLE_THINKING_ENV.to_string(),
                String::new(),
            );
            env.insert(
                claude::CLAUDE_MAX_THINKING_TOKENS_ENV.to_string(),
                String::new(),
            );
            for key in claude::CLAUDE_CONFLICT_ENV_KEYS {
                env.insert((*key).to_string(), String::new());
            }
            ClaudeRuntimeSettingsOverlay {
                always_thinking_enabled: Some(true),
                env: Some(env),
            }
        }
        claude::ClaudeThinkingMode::Off => {
            env.insert(
                claude::CLAUDE_DISABLE_THINKING_ENV.to_string(),
                "1".to_string(),
            );
            env.insert(
                claude::CLAUDE_MAX_THINKING_TOKENS_ENV.to_string(),
                String::new(),
            );
            for key in claude::CLAUDE_CONFLICT_ENV_KEYS {
                env.insert((*key).to_string(), String::new());
            }
            ClaudeRuntimeSettingsOverlay {
                always_thinking_enabled: Some(false),
                env: Some(env),
            }
        }
    }
}

fn overlay_path(runtime_dir_path: &Path) -> PathBuf {
    runtime_dir_path.join("claude-settings-overlay.json")
}

fn env_file_copy_path(runtime_dir_path: &Path) -> PathBuf {
    runtime_dir_path.join("claude.env")
}

fn write_overlay_file(
    runtime_dir_path: &Path,
    overlay: &ClaudeRuntimeSettingsOverlay,
) -> Result<PathBuf, String> {
    let path = overlay_path(runtime_dir_path);
    let bytes = serde_json::to_vec_pretty(overlay)
        .map_err(|e| format!("Failed to serialize Claude runtime overlay: {e}"))?;
    write_private_file(&path, &bytes)?;
    Ok(path)
}

fn copy_env_file_if_needed(
    process_env: &HashMap<String, String>,
    runtime_dir_path: &Path,
    env: &mut Vec<(String, String)>,
    warnings: &mut Vec<String>,
) -> Result<Option<PathBuf>, String> {
    let Some(source) = normalize_optional_env(process_env.get(CLAUDE_ENV_FILE_ENV)) else {
        return Ok(None);
    };

    let source_path = PathBuf::from(&source);
    let bytes = match std::fs::read(&source_path) {
        Ok(bytes) => bytes,
        Err(error) => {
            warnings.push(format!(
                "Failed to copy inherited CLAUDE_ENV_FILE from {}: {error}",
                source_path.display()
            ));
            return Ok(None);
        }
    };

    let dest_path = env_file_copy_path(runtime_dir_path);
    write_private_file(&dest_path, &bytes)?;
    env.push((
        CLAUDE_ENV_FILE_ENV.to_string(),
        dest_path.to_string_lossy().to_string(),
    ));
    Ok(Some(dest_path))
}

fn build_unset_env() -> Vec<String> {
    let mut unset = BTreeSet::from([
        CLAUDE_CONFIG_DIR_ENV.to_string(),
        CLAUDE_ENV_FILE_ENV.to_string(),
        claude::CLAUDE_BASE_URL_ENV.to_string(),
        claude::CLAUDE_AUTH_TOKEN_ENV.to_string(),
        claude::CLAUDE_API_KEY_ENV.to_string(),
        claude::CLAUDE_MODEL_ENV.to_string(),
        claude::CLAUDE_SMALL_MODEL_ENV.to_string(),
        claude::CLAUDE_EFFORT_ENV.to_string(),
        claude::CLAUDE_DISABLE_THINKING_ENV.to_string(),
        claude::CLAUDE_MAX_THINKING_TOKENS_ENV.to_string(),
        claude::CLAUDE_DISABLE_ADAPTIVE_ENV.to_string(),
    ]);

    for key in claude::CLAUDE_CONFLICT_ENV_KEYS {
        unset.insert((*key).to_string());
    }

    unset.into_iter().collect()
}

pub fn build_temporary_run_plan_for_home(
    home_dir: &Path,
    profile: &claude::ClaudeCodeProfile,
) -> Result<ClaudeTemporaryLaunchPlan, String> {
    let process_env: HashMap<String, String> = std::env::vars().collect();
    build_temporary_run_plan_for_home_with_env(home_dir, profile, &process_env)
}

pub fn build_temporary_run_preview_plan_for_home(
    home_dir: &Path,
    profile: &claude::ClaudeCodeProfile,
) -> Result<ClaudeTemporaryRunPlan, String> {
    let process_env: HashMap<String, String> = std::env::vars().collect();
    build_temporary_run_preview_plan_for_home_with_env(home_dir, profile, &process_env)
}

pub fn build_temporary_run_preview_plan(
    profile: &claude::ClaudeCodeProfile,
) -> Result<ClaudeTemporaryRunPlan, String> {
    let home_dir = dirs::home_dir().ok_or_else(|| "Failed to get home directory".to_string())?;
    build_temporary_run_preview_plan_for_home(&home_dir, profile)
}

fn build_temporary_run_preview_plan_for_home_with_env(
    home_dir: &Path,
    profile: &claude::ClaudeCodeProfile,
    process_env: &HashMap<String, String>,
) -> Result<ClaudeTemporaryRunPlan, String> {
    let live_config_dir = claude::claude_config_dir_for_home(home_dir)?;
    let mut env = vec![(
        CLAUDE_CONFIG_DIR_ENV.to_string(),
        live_config_dir.to_string_lossy().to_string(),
    )];
    let mut warnings = Vec::new();

    if let Some(source) = normalize_optional_env(process_env.get(CLAUDE_ENV_FILE_ENV)) {
        let source_path = PathBuf::from(&source);
        match std::fs::read(&source_path) {
            Ok(_) => env.push((
                CLAUDE_ENV_FILE_ENV.to_string(),
                "<runtime copy written at launch>".to_string(),
            )),
            Err(error) => warnings.push(format!(
                "Failed to copy inherited CLAUDE_ENV_FILE from {}: {error}",
                source_path.display()
            )),
        }
    }

    let overlay_notice = if profile.bearer_token.is_some() {
        "<generated at launch; contains managed Claude settings>"
    } else {
        "<generated at launch>"
    };

    Ok(ClaudeTemporaryRunPlan {
        program: "claude".to_string(),
        args: vec!["--settings".to_string(), overlay_notice.to_string()],
        env,
        unset_env: build_unset_env(),
        secret_env_keys: Vec::new(),
        warnings,
    })
}

fn build_temporary_run_plan_for_home_with_env(
    home_dir: &Path,
    profile: &claude::ClaudeCodeProfile,
    process_env: &HashMap<String, String>,
) -> Result<ClaudeTemporaryLaunchPlan, String> {
    let runtime_dir_path = next_runtime_dir_path(home_dir)?;
    std::fs::create_dir_all(&runtime_dir_path)
        .map_err(|e| format!("Failed to create Claude runtime directory: {e}"))?;

    let live_config_dir = claude::claude_config_dir_for_home(home_dir)?;
    let overlay = build_runtime_settings_overlay(profile);
    let settings_overlay_path = write_overlay_file(&runtime_dir_path, &overlay)?;

    let mut env = vec![(
        CLAUDE_CONFIG_DIR_ENV.to_string(),
        live_config_dir.to_string_lossy().to_string(),
    )];
    let mut warnings = Vec::new();
    let copied_env_file_path =
        copy_env_file_if_needed(process_env, &runtime_dir_path, &mut env, &mut warnings)?;

    Ok(ClaudeTemporaryLaunchPlan {
        program: "claude".to_string(),
        args: vec![
            "--settings".to_string(),
            settings_overlay_path.to_string_lossy().to_string(),
        ],
        env,
        secret_env: Vec::new(),
        unset_env: build_unset_env(),
        warnings,
        runtime_dir_path,
        settings_overlay_path,
        copied_env_file_path,
    })
}

pub fn build_temporary_run_plan(
    profile: &claude::ClaudeCodeProfile,
) -> Result<ClaudeTemporaryLaunchPlan, String> {
    let home_dir = dirs::home_dir().ok_or_else(|| "Failed to get home directory".to_string())?;
    build_temporary_run_plan_for_home(&home_dir, profile)
}

pub fn cleanup_stale_runtime_dirs_for_home(home_dir: &Path) -> Result<u32, String> {
    let runtime_dir = runtime_dir_for_home(home_dir);
    if !runtime_dir.exists() {
        return Ok(0);
    }

    let cutoff = std::time::SystemTime::now()
        .checked_sub(std::time::Duration::from_secs(60 * 60 * 24))
        .ok_or_else(|| "Failed to compute Claude runtime cleanup cutoff".to_string())?;

    let mut removed = 0;
    let entries = std::fs::read_dir(&runtime_dir)
        .map_err(|e| format!("Failed to read Claude runtime directory: {e}"))?;

    for entry in entries.flatten() {
        let path = entry.path();
        let Some(name) = path.file_name().and_then(|value| value.to_str()) else {
            continue;
        };

        if !name.starts_with(TEMP_RUNTIME_PREFIX) {
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

        if std::fs::remove_dir_all(&path).is_ok() {
            removed += 1;
        }
    }

    Ok(removed)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    fn make_profile() -> claude::ClaudeCodeProfile {
        claude::ClaudeCodeProfile {
            id: "p1".to_string(),
            name: "Profile 1".to_string(),
            description: None,
            base_url: Some("https://proxy.example.com".to_string()),
            bearer_token: Some("bearer-token".to_string()),
            model: Some("claude-sonnet-4-5".to_string()),
            small_model_uses_main_model: false,
            small_model: Some("claude-haiku-4".to_string()),
            reasoning_effort: Some(claude::ClaudeReasoningEffort::Max),
            thinking_mode: claude::ClaudeThinkingMode::On,
            created_at: "2026-01-01T00:00:00Z".to_string(),
            updated_at: "2026-01-01T00:00:00Z".to_string(),
        }
    }

    fn read_overlay(path: &Path) -> serde_json::Value {
        serde_json::from_str(&fs::read_to_string(path).unwrap()).unwrap()
    }

    #[test]
    fn runtime_overlay_contains_managed_values_and_tombstones() {
        let overlay = build_runtime_settings_overlay(&make_profile());
        let env = overlay.env.unwrap();

        assert_eq!(overlay.always_thinking_enabled, Some(true));
        assert_eq!(
            env.get(claude::CLAUDE_BASE_URL_ENV).map(String::as_str),
            Some("https://proxy.example.com")
        );
        assert_eq!(
            env.get(claude::CLAUDE_AUTH_TOKEN_ENV).map(String::as_str),
            Some("bearer-token")
        );
        assert_eq!(
            env.get(claude::CLAUDE_MODEL_ENV).map(String::as_str),
            Some("claude-sonnet-4-5")
        );
        assert_eq!(
            env.get(claude::CLAUDE_SMALL_MODEL_ENV).map(String::as_str),
            Some("claude-haiku-4")
        );
        assert_eq!(
            env.get(claude::CLAUDE_EFFORT_ENV).map(String::as_str),
            Some("max")
        );
        assert_eq!(
            env.get(claude::CLAUDE_DISABLE_ADAPTIVE_ENV)
                .map(String::as_str),
            Some("1")
        );
        assert_eq!(
            env.get(claude::CLAUDE_DISABLE_THINKING_ENV)
                .map(String::as_str),
            Some("")
        );
        assert_eq!(
            env.get(claude::CLAUDE_MAX_THINKING_TOKENS_ENV)
                .map(String::as_str),
            Some("")
        );
        assert_eq!(
            env.get(claude::CLAUDE_API_KEY_ENV).map(String::as_str),
            Some("")
        );
    }

    #[test]
    fn temporary_run_plan_uses_overlay_file_and_scrubs_inherited_env() {
        let temp = TempDir::new().unwrap();
        let home = temp.path();
        let profile = make_profile();
        let process_env = HashMap::from([
            ("ANTHROPIC_AUTH_TOKEN".to_string(), "stale".to_string()),
            (
                "CLAUDE_ENV_FILE".to_string(),
                "/tmp/should-not-leak".to_string(),
            ),
        ]);

        let plan =
            build_temporary_run_plan_for_home_with_env(home, &profile, &process_env).unwrap();

        assert_eq!(plan.program, "claude");
        assert_eq!(plan.args.len(), 2);
        assert_eq!(plan.args[0], "--settings");
        assert_eq!(plan.secret_env, Vec::<(String, String)>::new());
        assert!(plan
            .unset_env
            .contains(&claude::CLAUDE_AUTH_TOKEN_ENV.to_string()));
        assert!(plan.unset_env.contains(&CLAUDE_ENV_FILE_ENV.to_string()));
        assert!(plan.unset_env.contains(&CLAUDE_CONFIG_DIR_ENV.to_string()));
        assert!(plan
            .env
            .iter()
            .any(|(key, value)| key == CLAUDE_CONFIG_DIR_ENV && value.ends_with(".claude")));
        assert!(!plan.args.join(" ").contains("bearer-token"));
        assert!(!plan.env.iter().any(|(_, value)| value == "bearer-token"));
    }

    #[test]
    fn temporary_run_plan_does_not_mutate_live_settings() {
        let temp = TempDir::new().unwrap();
        let home = temp.path();
        let live_settings_path = claude::claude_settings_path_for_home(home).unwrap();
        fs::write(&live_settings_path, r#"{"env":{"KEEP":"1"}}"#).unwrap();

        let plan =
            build_temporary_run_plan_for_home_with_env(home, &make_profile(), &HashMap::new())
                .unwrap();

        let live = fs::read_to_string(&live_settings_path).unwrap();
        assert_eq!(live, r#"{"env":{"KEEP":"1"}}"#);

        let overlay = read_overlay(&PathBuf::from(&plan.args[1]));
        assert_eq!(
            overlay
                .get("env")
                .and_then(|value| value.get(claude::CLAUDE_AUTH_TOKEN_ENV))
                .and_then(serde_json::Value::as_str),
            Some("bearer-token")
        );
    }

    #[test]
    fn temporary_run_plan_copies_claude_env_file() {
        let temp = TempDir::new().unwrap();
        let home = temp.path();
        let inherited_env_path = home.join("inherited.env");
        fs::write(&inherited_env_path, "export EXAMPLE=1\n").unwrap();

        let process_env = HashMap::from([(
            CLAUDE_ENV_FILE_ENV.to_string(),
            inherited_env_path.to_string_lossy().to_string(),
        )]);

        let plan = build_temporary_run_plan_for_home_with_env(home, &make_profile(), &process_env)
            .unwrap();

        let copied_path = plan.copied_env_file_path.unwrap();
        assert_eq!(
            fs::read_to_string(&copied_path).unwrap(),
            "export EXAMPLE=1\n"
        );
        assert!(plan.env.iter().any(|(key, value)| {
            key == CLAUDE_ENV_FILE_ENV && value == &copied_path.to_string_lossy()
        }));
    }

    #[test]
    fn temporary_run_plan_warns_when_inherited_env_file_cannot_be_copied() {
        let temp = TempDir::new().unwrap();
        let home = temp.path();
        let process_env = HashMap::from([(
            CLAUDE_ENV_FILE_ENV.to_string(),
            home.join("missing.env").to_string_lossy().to_string(),
        )]);

        let plan = build_temporary_run_plan_for_home_with_env(home, &make_profile(), &process_env)
            .unwrap();

        assert!(plan.copied_env_file_path.is_none());
        assert_eq!(plan.env.len(), 1);
        assert_eq!(plan.warnings.len(), 1);
        assert!(plan.warnings[0].contains("Failed to copy inherited CLAUDE_ENV_FILE"));
    }

    #[test]
    fn temporary_run_plan_normalizes_profile_env_values_and_mirrors_main_model() {
        let temp = TempDir::new().unwrap();
        let home = temp.path();
        let mut profile = make_profile();
        profile.base_url = Some("  https://proxy.example.com  ".to_string());
        profile.bearer_token = Some("  bearer-token  ".to_string());
        profile.model = Some("  claude-sonnet-4-5  ".to_string());
        profile.small_model_uses_main_model = true;
        profile.small_model = Some("  ignored-small-model  ".to_string());

        let plan =
            build_temporary_run_plan_for_home_with_env(home, &profile, &HashMap::new()).unwrap();
        let overlay = read_overlay(&PathBuf::from(&plan.args[1]));

        assert_eq!(
            overlay
                .get("env")
                .and_then(|value| value.get(claude::CLAUDE_BASE_URL_ENV))
                .and_then(serde_json::Value::as_str),
            Some("https://proxy.example.com")
        );
        assert_eq!(
            overlay
                .get("env")
                .and_then(|value| value.get(claude::CLAUDE_AUTH_TOKEN_ENV))
                .and_then(serde_json::Value::as_str),
            Some("bearer-token")
        );
        assert_eq!(
            overlay
                .get("env")
                .and_then(|value| value.get(claude::CLAUDE_MODEL_ENV))
                .and_then(serde_json::Value::as_str),
            Some("claude-sonnet-4-5")
        );
        assert_eq!(
            overlay
                .get("env")
                .and_then(|value| value.get(claude::CLAUDE_SMALL_MODEL_ENV))
                .and_then(serde_json::Value::as_str),
            Some("claude-sonnet-4-5")
        );
    }

    #[test]
    fn temporary_run_preview_does_not_materialize_runtime_artifacts() {
        let temp = TempDir::new().unwrap();
        let home = temp.path();
        let inherited_env_path = home.join("inherited.env");
        fs::write(&inherited_env_path, "export EXAMPLE=1\n").unwrap();

        let process_env = HashMap::from([(
            CLAUDE_ENV_FILE_ENV.to_string(),
            inherited_env_path.to_string_lossy().to_string(),
        )]);

        let preview =
            build_temporary_run_preview_plan_for_home_with_env(home, &make_profile(), &process_env)
                .unwrap();

        assert_eq!(preview.program, "claude");
        assert_eq!(preview.args[0], "--settings");
        assert_eq!(
            preview.args[1],
            "<generated at launch; contains managed Claude settings>"
        );
        assert!(preview.env.iter().any(|(key, value)| {
            key == CLAUDE_ENV_FILE_ENV && value == "<runtime copy written at launch>"
        }));
        assert!(!runtime_dir_for_home(home).exists());
    }

    #[test]
    fn overlay_file_is_written_with_private_permissions_on_unix() {
        let temp = TempDir::new().unwrap();
        let home = temp.path();
        let plan =
            build_temporary_run_plan_for_home_with_env(home, &make_profile(), &HashMap::new())
                .unwrap();

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;

            let mode = fs::metadata(&plan.settings_overlay_path)
                .unwrap()
                .permissions()
                .mode()
                & 0o777;
            assert_eq!(mode, 0o600);
        }
    }

    #[test]
    fn cleanup_stale_runtime_dirs_only_removes_old_runtime_dirs() {
        let temp = TempDir::new().unwrap();
        let runtime_dir = runtime_dir_for_home(temp.path());
        fs::create_dir_all(&runtime_dir).unwrap();

        let stale_dir = runtime_dir.join(format!("{TEMP_RUNTIME_PREFIX}stale"));
        let fresh_dir = runtime_dir.join(format!("{TEMP_RUNTIME_PREFIX}fresh"));
        let unrelated_dir = runtime_dir.join("keep-me");
        fs::create_dir_all(&stale_dir).unwrap();
        fs::create_dir_all(&fresh_dir).unwrap();
        fs::create_dir_all(&unrelated_dir).unwrap();

        let two_days_ago = filetime::FileTime::from_system_time(
            std::time::SystemTime::now() - std::time::Duration::from_secs(60 * 60 * 48),
        );
        filetime::set_file_mtime(&stale_dir, two_days_ago).unwrap();

        let removed = cleanup_stale_runtime_dirs_for_home(temp.path()).unwrap();
        assert_eq!(removed, 1);
        assert!(!stale_dir.exists());
        assert!(fresh_dir.exists());
        assert!(unrelated_dir.exists());
    }
}
