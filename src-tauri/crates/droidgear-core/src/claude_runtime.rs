//! Claude Code temporary-run planning and internal launcher support.
//!
//! Temporary runs keep sharing the live Claude config dir, but freeze managed
//! profile settings into a wrapper-private payload. The internal launcher
//! materializes the runtime `--settings` overlay immediately before exec.

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
const CLAUDE_RUNTIME_PAYLOAD_ENV: &str = "DROIDGEAR_INTERNAL_CLAUDE_RUNTIME_JSON";
const INTERNAL_LAUNCH_MARKER: &str = "__droidgear_internal";
const INTERNAL_LAUNCH_CLAUDE_RUNNER: &str = "claude-launcher";

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

#[derive(Debug, Clone, Serialize, Deserialize, Type, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ClaudeTemporaryRunDebugPreview {
    pub profile_id: String,
    pub profile_name: String,
    pub program: String,
    pub args: Vec<String>,
    pub child_program: String,
    pub child_args: Vec<String>,
    pub live_config_dir: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub inherited_env_file_source: Option<String>,
    pub env: Vec<(String, String)>,
    pub unset_env: Vec<String>,
    pub secret_env_keys: Vec<String>,
    pub warnings: Vec<String>,
    pub settings_overlay_json: String,
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

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
struct ClaudeInternalLauncherPayload {
    runtime_dir_path: String,
    live_config_dir: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    config_dir_env_override: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    inherited_env_file_source: Option<String>,
    child_program: String,
    child_args: Vec<String>,
    settings_overlay: ClaudeRuntimeSettingsOverlay,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ClaudeMaterializedChildLaunch {
    program: String,
    args: Vec<String>,
    env: Vec<(String, String)>,
    unset_env: Vec<String>,
    warnings: Vec<String>,
    settings_overlay_path: PathBuf,
    copied_env_file_path: Option<PathBuf>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ClaudeConfigDirBinding {
    effective_dir: String,
    env_override: Option<String>,
}

pub fn internal_launcher_args() -> Vec<String> {
    vec![
        INTERNAL_LAUNCH_MARKER.to_string(),
        INTERNAL_LAUNCH_CLAUDE_RUNNER.to_string(),
    ]
}

pub fn matches_internal_launcher_args(args: &[String]) -> bool {
    matches!(
        args,
        [marker, command, ..]
            if marker == INTERNAL_LAUNCH_MARKER && command == INTERNAL_LAUNCH_CLAUDE_RUNNER
    )
}

fn runtime_dir_for_home(home_dir: &Path) -> PathBuf {
    paths::droidgear_dir_from_home(home_dir).join(CLAUDE_RUNTIME_DIR)
}

fn next_runtime_dir_path(home_dir: &Path) -> Result<PathBuf, String> {
    let timestamp = chrono::Utc::now().format("%Y%m%dT%H%M%S%.3fZ");
    Ok(runtime_dir_for_home(home_dir).join(format!(
        "{TEMP_RUNTIME_PREFIX}{timestamp}-{}",
        Uuid::new_v4()
    )))
}

fn overlay_path(runtime_dir_path: &Path) -> PathBuf {
    runtime_dir_path.join("claude-settings-overlay.json")
}

fn env_file_copy_path(runtime_dir_path: &Path) -> PathBuf {
    runtime_dir_path.join("claude.env")
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

fn resolve_live_config_dir_for_home(home_dir: &Path) -> Result<PathBuf, String> {
    let config_paths = paths::load_config_paths_for_home(home_dir);
    paths::get_claude_home_for_home(home_dir, &config_paths)
}

fn resolve_live_config_dir_binding_for_home_with_env(
    home_dir: &Path,
    process_env: &HashMap<String, String>,
) -> Result<ClaudeConfigDirBinding, String> {
    let config_paths = paths::load_config_paths_for_home(home_dir);

    if let Some(configured_override) = config_paths
        .claude
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
    {
        return Ok(ClaudeConfigDirBinding {
            effective_dir: configured_override.to_string(),
            env_override: Some(configured_override.to_string()),
        });
    }

    if let Some(inherited_override) = normalize_optional_env(process_env.get(CLAUDE_CONFIG_DIR_ENV))
    {
        return Ok(ClaudeConfigDirBinding {
            effective_dir: inherited_override.clone(),
            env_override: Some(inherited_override),
        });
    }

    Ok(ClaudeConfigDirBinding {
        effective_dir: resolve_live_config_dir_for_home(home_dir)?
            .to_string_lossy()
            .to_string(),
        env_override: None,
    })
}

fn format_env_file_copy_warning(source_path: &Path, error: &std::io::Error) -> String {
    format!(
        "Failed to copy inherited CLAUDE_ENV_FILE from {}: {error}. Claude temporary run will continue without inheriting that runtime env file.",
        source_path.display()
    )
}

fn check_inherited_env_file_source(
    source: Option<String>,
    warnings: &mut Vec<String>,
) -> Option<String> {
    let source = source?;

    let source_path = PathBuf::from(&source);
    if let Err(error) = std::fs::read(&source_path) {
        warnings.push(format_env_file_copy_warning(&source_path, &error));
    }

    Some(source)
}

fn build_internal_launcher_payload_for_home_with_env(
    home_dir: &Path,
    profile: &claude::ClaudeCodeProfile,
    process_env: &HashMap<String, String>,
) -> Result<(ClaudeInternalLauncherPayload, Vec<String>), String> {
    let mut warnings = Vec::new();
    let config_dir_binding =
        resolve_live_config_dir_binding_for_home_with_env(home_dir, process_env)?;
    let inherited_env_file_source = check_inherited_env_file_source(
        normalize_optional_env(process_env.get(CLAUDE_ENV_FILE_ENV)),
        &mut warnings,
    );

    Ok((
        ClaudeInternalLauncherPayload {
            runtime_dir_path: next_runtime_dir_path(home_dir)?
                .to_string_lossy()
                .to_string(),
            live_config_dir: config_dir_binding.effective_dir,
            config_dir_env_override: config_dir_binding.env_override,
            inherited_env_file_source,
            child_program: "claude".to_string(),
            child_args: Vec::new(),
            settings_overlay: build_runtime_settings_overlay(profile),
        },
        warnings,
    ))
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

fn build_visible_env(config_dir_env_override: Option<&str>) -> Vec<(String, String)> {
    config_dir_env_override
        .map(|value| vec![(CLAUDE_CONFIG_DIR_ENV.to_string(), value.to_string())])
        .unwrap_or_default()
}

fn build_secret_env(
    payload: &ClaudeInternalLauncherPayload,
) -> Result<Vec<(String, String)>, String> {
    let raw = serde_json::to_string(payload)
        .map_err(|e| format!("Failed to serialize Claude launcher payload: {e}"))?;
    Ok(vec![(CLAUDE_RUNTIME_PAYLOAD_ENV.to_string(), raw)])
}

fn copy_env_file_if_needed(
    source: Option<&str>,
    runtime_dir_path: &Path,
    env: &mut Vec<(String, String)>,
    warnings: &mut Vec<String>,
) -> Result<Option<PathBuf>, String> {
    let Some(source) = source else {
        return Ok(None);
    };

    let source_path = PathBuf::from(source);
    let bytes = match std::fs::read(&source_path) {
        Ok(bytes) => bytes,
        Err(error) => {
            warnings.push(format_env_file_copy_warning(&source_path, &error));
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

fn materialize_child_launch_from_payload(
    payload: &ClaudeInternalLauncherPayload,
) -> Result<ClaudeMaterializedChildLaunch, String> {
    let runtime_dir_path = PathBuf::from(&payload.runtime_dir_path);
    std::fs::create_dir_all(&runtime_dir_path)
        .map_err(|e| format!("Failed to create Claude runtime directory: {e}"))?;

    let settings_overlay_path = write_overlay_file(&runtime_dir_path, &payload.settings_overlay)?;
    let mut env = build_visible_env(payload.config_dir_env_override.as_deref());
    let mut warnings = Vec::new();
    let copied_env_file_path = copy_env_file_if_needed(
        payload.inherited_env_file_source.as_deref(),
        &runtime_dir_path,
        &mut env,
        &mut warnings,
    )?;

    let mut args = payload.child_args.clone();
    args.push("--settings".to_string());
    args.push(settings_overlay_path.to_string_lossy().to_string());

    Ok(ClaudeMaterializedChildLaunch {
        program: payload.child_program.clone(),
        args,
        env,
        unset_env: vec![CLAUDE_RUNTIME_PAYLOAD_ENV.to_string()],
        warnings,
        settings_overlay_path,
        copied_env_file_path,
    })
}

fn sanitize_terminal_for_internal_claude_exec() -> Result<(), String> {
    use std::io::{self, IsTerminal, Write};

    if !io::stdin().is_terminal() || !io::stdout().is_terminal() {
        return Ok(());
    }

    let mut stdout = io::stdout();
    stdout
        .write_all(
            b"\x1b[0m\x1b[?25h\x1b[?1000l\x1b[?1002l\x1b[?1003l\x1b[?1004l\x1b[?1006l\x1b[?2004l",
        )
        .map_err(|e| format!("Failed to write Claude terminal reset sequence: {e}"))?;
    stdout
        .flush()
        .map_err(|e| format!("Failed to flush Claude terminal reset sequence: {e}"))?;

    #[cfg(unix)]
    unsafe {
        if libc::tcflush(libc::STDIN_FILENO, libc::TCIFLUSH) != 0 {
            return Err(format!(
                "Failed to flush Claude terminal input queue: {}",
                std::io::Error::last_os_error()
            ));
        }
    }

    Ok(())
}

pub fn build_temporary_run_plan_for_home(
    home_dir: &Path,
    profile: &claude::ClaudeCodeProfile,
    launcher_program: &str,
    launcher_args: &[String],
) -> Result<ClaudeTemporaryLaunchPlan, String> {
    let process_env: HashMap<String, String> = std::env::vars().collect();
    build_temporary_run_plan_for_home_with_env(
        home_dir,
        profile,
        &process_env,
        launcher_program,
        launcher_args,
    )
}

pub fn build_temporary_run_plan(
    profile: &claude::ClaudeCodeProfile,
    launcher_program: &str,
    launcher_args: &[String],
) -> Result<ClaudeTemporaryLaunchPlan, String> {
    let home_dir = dirs::home_dir().ok_or_else(|| "Failed to get home directory".to_string())?;
    build_temporary_run_plan_for_home(&home_dir, profile, launcher_program, launcher_args)
}

fn build_temporary_run_plan_for_home_with_env(
    home_dir: &Path,
    profile: &claude::ClaudeCodeProfile,
    process_env: &HashMap<String, String>,
    launcher_program: &str,
    launcher_args: &[String],
) -> Result<ClaudeTemporaryLaunchPlan, String> {
    let (payload, warnings) =
        build_internal_launcher_payload_for_home_with_env(home_dir, profile, process_env)?;

    Ok(ClaudeTemporaryLaunchPlan {
        program: launcher_program.to_string(),
        args: launcher_args.to_vec(),
        env: build_visible_env(payload.config_dir_env_override.as_deref()),
        secret_env: build_secret_env(&payload)?,
        unset_env: build_unset_env(),
        warnings,
        runtime_dir_path: PathBuf::from(&payload.runtime_dir_path),
    })
}

pub fn build_temporary_run_preview_plan_for_home(
    home_dir: &Path,
    profile: &claude::ClaudeCodeProfile,
    launcher_program: &str,
    launcher_args: &[String],
) -> Result<ClaudeTemporaryRunPlan, String> {
    let process_env: HashMap<String, String> = std::env::vars().collect();
    build_temporary_run_preview_plan_for_home_with_env(
        home_dir,
        profile,
        &process_env,
        launcher_program,
        launcher_args,
    )
}

pub fn build_temporary_run_preview_plan(
    profile: &claude::ClaudeCodeProfile,
    launcher_program: &str,
    launcher_args: &[String],
) -> Result<ClaudeTemporaryRunPlan, String> {
    let home_dir = dirs::home_dir().ok_or_else(|| "Failed to get home directory".to_string())?;
    build_temporary_run_preview_plan_for_home(&home_dir, profile, launcher_program, launcher_args)
}

fn build_temporary_run_preview_plan_for_home_with_env(
    home_dir: &Path,
    profile: &claude::ClaudeCodeProfile,
    process_env: &HashMap<String, String>,
    launcher_program: &str,
    launcher_args: &[String],
) -> Result<ClaudeTemporaryRunPlan, String> {
    let preview = build_temporary_run_debug_preview_for_home_with_env(
        home_dir,
        profile,
        process_env,
        launcher_program,
        launcher_args,
    )?;
    Ok(ClaudeTemporaryRunPlan {
        program: preview.program,
        args: preview.args,
        env: preview.env,
        unset_env: preview.unset_env,
        secret_env_keys: preview.secret_env_keys,
        warnings: preview.warnings,
    })
}

pub fn build_temporary_run_debug_preview_for_home(
    home_dir: &Path,
    profile: &claude::ClaudeCodeProfile,
    launcher_program: &str,
    launcher_args: &[String],
) -> Result<ClaudeTemporaryRunDebugPreview, String> {
    let process_env: HashMap<String, String> = std::env::vars().collect();
    build_temporary_run_debug_preview_for_home_with_env(
        home_dir,
        profile,
        &process_env,
        launcher_program,
        launcher_args,
    )
}

fn build_temporary_run_debug_preview_for_home_with_env(
    home_dir: &Path,
    profile: &claude::ClaudeCodeProfile,
    process_env: &HashMap<String, String>,
    launcher_program: &str,
    launcher_args: &[String],
) -> Result<ClaudeTemporaryRunDebugPreview, String> {
    let (payload, warnings) =
        build_internal_launcher_payload_for_home_with_env(home_dir, profile, process_env)?;
    let settings_overlay_json = serde_json::to_string_pretty(&payload.settings_overlay)
        .map_err(|e| format!("Failed to serialize Claude runtime overlay: {e}"))?;

    Ok(ClaudeTemporaryRunDebugPreview {
        profile_id: profile.id.clone(),
        profile_name: profile.name.clone(),
        program: launcher_program.to_string(),
        args: launcher_args.to_vec(),
        child_program: payload.child_program.clone(),
        child_args: payload.child_args.clone(),
        live_config_dir: payload.live_config_dir.clone(),
        inherited_env_file_source: payload.inherited_env_file_source.clone(),
        env: build_visible_env(payload.config_dir_env_override.as_deref()),
        unset_env: build_unset_env(),
        secret_env_keys: vec![CLAUDE_RUNTIME_PAYLOAD_ENV.to_string()],
        warnings,
        settings_overlay_json,
    })
}

pub fn run_internal_launcher_from_env() -> Result<(), String> {
    let raw = std::env::var(CLAUDE_RUNTIME_PAYLOAD_ENV)
        .map_err(|_| "Missing Claude temporary-run payload".to_string())?;
    let payload: ClaudeInternalLauncherPayload = serde_json::from_str(&raw)
        .map_err(|e| format!("Failed to decode Claude temporary-run payload: {e}"))?;
    let child = materialize_child_launch_from_payload(&payload)?;

    for warning in &child.warnings {
        eprintln!("Warning: {warning}");
    }

    sanitize_terminal_for_internal_claude_exec()?;

    let mut command = std::process::Command::new(&child.program);
    command.args(&child.args);

    for key in &child.unset_env {
        command.env_remove(key);
    }

    for (key, value) in &child.env {
        command.env(key, value);
    }

    #[cfg(unix)]
    {
        use std::os::unix::process::CommandExt;

        let error = command.exec();
        Err(format!("Failed to exec {}: {error}", child.program))
    }

    #[cfg(not(unix))]
    {
        let status = command
            .status()
            .map_err(|e| format!("Failed to launch {}: {e}", child.program))?;
        if status.success() {
            Ok(())
        } else {
            Err(format!(
                "Claude exited with status {}",
                status
                    .code()
                    .map_or_else(|| "unknown".to_string(), |code| code.to_string())
            ))
        }
    }
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

    fn test_launcher_program() -> String {
        "/tmp/droidgear-launcher".to_string()
    }

    fn test_launcher_args() -> Vec<String> {
        internal_launcher_args()
    }

    fn read_overlay(path: &Path) -> serde_json::Value {
        serde_json::from_str(&fs::read_to_string(path).unwrap()).unwrap()
    }

    fn write_claude_path_override(home: &Path, path: &Path) {
        let settings_path = home.join(".droidgear").join("settings.json");
        fs::create_dir_all(settings_path.parent().unwrap()).unwrap();
        fs::write(
            settings_path,
            format!(
                r#"{{
  "configPaths": {{
    "claude": "{}"
  }}
}}"#,
                path.display()
            ),
        )
        .unwrap();
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
    fn internal_launcher_args_match_expected_marker() {
        let args = internal_launcher_args();
        assert!(matches_internal_launcher_args(&args));
        assert!(!matches_internal_launcher_args(&["claude".to_string()]));
    }

    #[test]
    fn temporary_run_plan_uses_internal_launcher_and_secret_payload() {
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

        let plan = build_temporary_run_plan_for_home_with_env(
            home,
            &profile,
            &process_env,
            &test_launcher_program(),
            &test_launcher_args(),
        )
        .unwrap();

        assert_eq!(plan.program, test_launcher_program());
        assert_eq!(plan.args, test_launcher_args());
        assert_eq!(plan.secret_env.len(), 1);
        assert_eq!(plan.secret_env[0].0, CLAUDE_RUNTIME_PAYLOAD_ENV);
        assert!(plan
            .unset_env
            .contains(&claude::CLAUDE_AUTH_TOKEN_ENV.to_string()));
        assert!(plan.unset_env.contains(&CLAUDE_ENV_FILE_ENV.to_string()));
        assert!(plan.unset_env.contains(&CLAUDE_CONFIG_DIR_ENV.to_string()));
        assert!(plan.env.is_empty());
        assert!(!plan.args.join(" ").contains("bearer-token"));
        assert!(!plan.env.iter().any(|(_, value)| value == "bearer-token"));
    }

    #[test]
    fn temporary_run_plan_preserves_explicit_claude_config_dir_override() {
        let temp = TempDir::new().unwrap();
        let home = temp.path();
        let custom_claude_dir = home.join("custom-claude");
        write_claude_path_override(home, &custom_claude_dir);

        let plan = build_temporary_run_plan_for_home_with_env(
            home,
            &make_profile(),
            &HashMap::new(),
            &test_launcher_program(),
            &test_launcher_args(),
        )
        .unwrap();

        assert_eq!(
            plan.env,
            vec![(
                CLAUDE_CONFIG_DIR_ENV.to_string(),
                custom_claude_dir.display().to_string(),
            )]
        );
    }

    #[test]
    fn temporary_run_plan_preserves_inherited_claude_config_dir_when_no_path_override_exists() {
        let temp = TempDir::new().unwrap();
        let home = temp.path();
        let inherited_claude_dir = home.join("shell-claude");

        let plan = build_temporary_run_plan_for_home_with_env(
            home,
            &make_profile(),
            &HashMap::from([(
                CLAUDE_CONFIG_DIR_ENV.to_string(),
                inherited_claude_dir.display().to_string(),
            )]),
            &test_launcher_program(),
            &test_launcher_args(),
        )
        .unwrap();

        assert_eq!(
            plan.env,
            vec![(
                CLAUDE_CONFIG_DIR_ENV.to_string(),
                inherited_claude_dir.display().to_string(),
            )]
        );
    }

    #[test]
    fn temporary_run_plan_does_not_mutate_live_settings_or_materialize_runtime_files() {
        let temp = TempDir::new().unwrap();
        let home = temp.path();
        let live_settings_path = home.join(".claude").join("settings.json");
        fs::create_dir_all(live_settings_path.parent().unwrap()).unwrap();
        fs::write(&live_settings_path, r#"{"env":{"KEEP":"1"}}"#).unwrap();

        let plan = build_temporary_run_plan_for_home_with_env(
            home,
            &make_profile(),
            &HashMap::new(),
            &test_launcher_program(),
            &test_launcher_args(),
        )
        .unwrap();

        let live = fs::read_to_string(&live_settings_path).unwrap();
        assert_eq!(live, r#"{"env":{"KEEP":"1"}}"#);
        assert!(!overlay_path(&plan.runtime_dir_path).exists());
        assert!(plan.secret_env[0].1.contains("bearer-token"));
    }

    #[test]
    fn materialized_child_launch_writes_overlay_and_copies_env_file() {
        let temp = TempDir::new().unwrap();
        let home = temp.path();
        let inherited_env_path = home.join("inherited.env");
        fs::write(&inherited_env_path, "export EXAMPLE=1\n").unwrap();

        let (payload, warnings) = build_internal_launcher_payload_for_home_with_env(
            home,
            &make_profile(),
            &HashMap::from([(
                CLAUDE_ENV_FILE_ENV.to_string(),
                inherited_env_path.to_string_lossy().to_string(),
            )]),
        )
        .unwrap();
        assert!(warnings.is_empty());

        let child = materialize_child_launch_from_payload(&payload).unwrap();
        let overlay = read_overlay(&child.settings_overlay_path);

        assert_eq!(child.program, "claude");
        assert_eq!(child.args[0], "--settings");
        assert_eq!(child.env.len(), 1);
        assert_eq!(
            overlay
                .get("env")
                .and_then(|value| value.get(claude::CLAUDE_AUTH_TOKEN_ENV))
                .and_then(serde_json::Value::as_str),
            Some("bearer-token")
        );
        let copied_path = child.copied_env_file_path.unwrap();
        assert_eq!(
            fs::read_to_string(&copied_path).unwrap(),
            "export EXAMPLE=1\n"
        );
        assert!(child.env.iter().any(|(key, value)| {
            key == CLAUDE_ENV_FILE_ENV && value == &copied_path.to_string_lossy()
        }));
        assert_eq!(
            child.unset_env,
            vec![CLAUDE_RUNTIME_PAYLOAD_ENV.to_string()]
        );
    }

    #[test]
    fn materialized_child_launch_warns_when_inherited_env_file_cannot_be_copied() {
        let temp = TempDir::new().unwrap();
        let home = temp.path();

        let (payload, initial_warnings) = build_internal_launcher_payload_for_home_with_env(
            home,
            &make_profile(),
            &HashMap::from([(
                CLAUDE_ENV_FILE_ENV.to_string(),
                home.join("missing.env").to_string_lossy().to_string(),
            )]),
        )
        .unwrap();

        assert_eq!(initial_warnings.len(), 1);
        let child = materialize_child_launch_from_payload(&payload).unwrap();

        assert!(child.copied_env_file_path.is_none());
        assert_eq!(child.warnings.len(), 1);
        assert!(child.warnings[0].contains("Failed to copy inherited CLAUDE_ENV_FILE"));
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

        let (payload, _) =
            build_internal_launcher_payload_for_home_with_env(home, &profile, &HashMap::new())
                .unwrap();
        let overlay = payload.settings_overlay;

        assert_eq!(
            overlay
                .env
                .as_ref()
                .and_then(|value| value.get(claude::CLAUDE_BASE_URL_ENV))
                .map(String::as_str),
            Some("https://proxy.example.com")
        );
        assert_eq!(
            overlay
                .env
                .as_ref()
                .and_then(|value| value.get(claude::CLAUDE_AUTH_TOKEN_ENV))
                .map(String::as_str),
            Some("bearer-token")
        );
        assert_eq!(
            overlay
                .env
                .as_ref()
                .and_then(|value| value.get(claude::CLAUDE_MODEL_ENV))
                .map(String::as_str),
            Some("claude-sonnet-4-5")
        );
        assert_eq!(
            overlay
                .env
                .as_ref()
                .and_then(|value| value.get(claude::CLAUDE_SMALL_MODEL_ENV))
                .map(String::as_str),
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

        let preview = build_temporary_run_preview_plan_for_home_with_env(
            home,
            &make_profile(),
            &process_env,
            &test_launcher_program(),
            &test_launcher_args(),
        )
        .unwrap();

        assert_eq!(preview.program, test_launcher_program());
        assert_eq!(preview.args, test_launcher_args());
        assert_eq!(
            preview.secret_env_keys,
            vec![CLAUDE_RUNTIME_PAYLOAD_ENV.to_string()]
        );
        assert!(!runtime_dir_for_home(home).exists());
        assert!(!home.join(".claude").exists());
    }

    #[test]
    fn temporary_run_debug_preview_exposes_overlay_without_creating_runtime_files() {
        let temp = TempDir::new().unwrap();
        let home = temp.path();
        let inherited_env_path = home.join("inherited.env");
        fs::write(&inherited_env_path, "export EXAMPLE=1\n").unwrap();

        let process_env = HashMap::from([(
            CLAUDE_ENV_FILE_ENV.to_string(),
            inherited_env_path.to_string_lossy().to_string(),
        )]);

        let preview = build_temporary_run_debug_preview_for_home_with_env(
            home,
            &make_profile(),
            &process_env,
            &test_launcher_program(),
            &test_launcher_args(),
        )
        .unwrap();

        assert_eq!(preview.profile_id, "p1");
        assert_eq!(preview.profile_name, "Profile 1");
        assert_eq!(preview.program, test_launcher_program());
        assert_eq!(preview.args, test_launcher_args());
        assert_eq!(preview.child_program, "claude");
        assert!(preview.child_args.is_empty());
        assert!(preview.env.is_empty());
        assert_eq!(
            preview.inherited_env_file_source.as_deref(),
            Some(inherited_env_path.to_string_lossy().as_ref())
        );
        assert_eq!(
            preview.live_config_dir,
            home.join(".claude").display().to_string()
        );
        assert!(preview
            .settings_overlay_json
            .contains(claude::CLAUDE_AUTH_TOKEN_ENV));
        assert!(preview.settings_overlay_json.contains("bearer-token"));
        assert_eq!(
            preview.secret_env_keys,
            vec![CLAUDE_RUNTIME_PAYLOAD_ENV.to_string()]
        );
        assert!(!runtime_dir_for_home(home).exists());
        assert!(!home.join(".claude").exists());
    }

    #[test]
    fn launcher_payload_round_trips_and_stays_secret_env_only() {
        let temp = TempDir::new().unwrap();
        let home = temp.path();
        let (payload, _) = build_internal_launcher_payload_for_home_with_env(
            home,
            &make_profile(),
            &HashMap::new(),
        )
        .unwrap();
        let secret_env = build_secret_env(&payload).unwrap();

        assert_eq!(secret_env.len(), 1);
        assert_eq!(secret_env[0].0, CLAUDE_RUNTIME_PAYLOAD_ENV);

        let decoded: ClaudeInternalLauncherPayload =
            serde_json::from_str(&secret_env[0].1).unwrap();
        assert_eq!(decoded, payload);
    }

    #[test]
    fn overlay_file_is_written_with_private_permissions_on_unix() {
        let temp = TempDir::new().unwrap();
        let home = temp.path();
        let (payload, _) = build_internal_launcher_payload_for_home_with_env(
            home,
            &make_profile(),
            &HashMap::new(),
        )
        .unwrap();
        let child = materialize_child_launch_from_payload(&payload).unwrap();

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;

            let mode = fs::metadata(&child.settings_overlay_path)
                .unwrap()
                .permissions()
                .mode()
                & 0o777;
            assert_eq!(mode, 0o600);
        }
    }

    #[test]
    fn sanitize_terminal_for_internal_claude_exec_is_callable() {
        sanitize_terminal_for_internal_claude_exec().unwrap();
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
