//! Claude Code settings file management (core).
//!
//! Manages multiple Claude settings files stored in `~/.droidgear/claude-settings/`
//! and tracks the active one. Each file follows the official Claude Code
//! `settings.json` schema (top-level `env`, `permissions`, `autoUpdate`, etc.).
//!
//! The Global option points at `~/.claude/settings.json` (or the path overridden
//! by configuration). Custom files live under `~/.droidgear/claude-settings/`.
//!
//! Legacy DroidGear-managed Claude profiles stored under
//! `~/.droidgear/claude/profiles/` are migrated to custom settings files on
//! first list call.

use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use specta::Type;
use std::path::{Path, PathBuf};

use crate::{claude, paths, storage};

const CLAUDE_SETTINGS_DIR: &str = "claude-settings";
const ACTIVE_FILE_KEY: &str = "claudeSettingsActiveFile";
const LEGACY_MIGRATED_KEY: &str = "claudeLegacyProfilesMigrated";

// ============================================================================
// Types
// ============================================================================

/// Information about a single Claude settings file.
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct ClaudeSettingsFileInfo {
    /// Display name (`Global` or the filename without extension).
    pub name: String,
    /// Full path to the settings file.
    pub path: String,
    /// Whether this is the global `~/.claude/settings.json`.
    pub is_global: bool,
    /// Whether this file is currently active for editing.
    pub is_active: bool,
    /// Whether the file exists on disk.
    pub exists: bool,
}

// ============================================================================
// Path resolution
// ============================================================================

fn system_home_dir() -> Result<PathBuf, String> {
    dirs::home_dir().ok_or_else(|| "Failed to get home directory".to_string())
}

fn claude_settings_dir_for_home(home_dir: &Path) -> PathBuf {
    paths::droidgear_dir_from_home(home_dir).join(CLAUDE_SETTINGS_DIR)
}

fn claude_settings_dir() -> Result<PathBuf, String> {
    Ok(claude_settings_dir_for_home(&system_home_dir()?))
}

fn global_settings_path_for_home(home_dir: &Path) -> Result<PathBuf, String> {
    let config_paths = paths::load_config_paths_for_home(home_dir);
    Ok(paths::get_claude_home_for_home(home_dir, &config_paths)?.join("settings.json"))
}

/// Resolves the absolute path to the currently active settings file.
/// Falls back to the global path when the active file is missing.
pub fn get_active_settings_path_for_home(home_dir: &Path) -> Result<PathBuf, String> {
    let active_name = load_active_file_name_for_home(home_dir)?;
    match active_name {
        Some(name) if !name.is_empty() => {
            let custom_path = claude_settings_dir_for_home(home_dir)
                .join(&name)
                .with_extension("json");
            if custom_path.exists() {
                Ok(custom_path)
            } else {
                global_settings_path_for_home(home_dir)
            }
        }
        _ => global_settings_path_for_home(home_dir),
    }
}

pub fn get_active_settings_path() -> Result<PathBuf, String> {
    get_active_settings_path_for_home(&system_home_dir()?)
}

// ============================================================================
// Active file tracking
// ============================================================================

fn load_active_file_name_for_home(home_dir: &Path) -> Result<Option<String>, String> {
    let settings_path = paths::get_droidgear_settings_path_for_home(home_dir);
    let settings = paths::read_droidgear_settings_from_path_internal(&settings_path)?;
    Ok(settings
        .get(ACTIVE_FILE_KEY)
        .and_then(|v| v.as_str())
        .map(String::from))
}

fn save_active_file_name_for_home(home_dir: &Path, name: Option<&str>) -> Result<(), String> {
    let settings_path = paths::get_droidgear_settings_path_for_home(home_dir);
    let mut settings = paths::read_droidgear_settings_from_path_internal(&settings_path)?;

    if let Some(obj) = settings.as_object_mut() {
        match name {
            Some(value) if !value.is_empty() => {
                obj.insert(ACTIVE_FILE_KEY.to_string(), serde_json::json!(value));
            }
            _ => {
                obj.remove(ACTIVE_FILE_KEY);
            }
        }
    }

    paths::write_droidgear_settings_to_path_internal(&settings_path, &settings)
}

// ============================================================================
// File-name validation
// ============================================================================

fn validate_custom_file_name(name: &str) -> Result<(), String> {
    let trimmed = name.trim();
    if trimmed.is_empty() {
        return Err("File name cannot be empty".to_string());
    }
    if trimmed.eq_ignore_ascii_case("global") {
        return Err("Cannot use 'Global' as a custom file name".to_string());
    }
    if trimmed.chars().count() > 100 {
        return Err("File name too long (max 100 characters)".to_string());
    }
    let ok = trimmed
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_' || c == '.');
    if !ok {
        return Err(
            "Invalid file name: only letters, digits, dashes, underscores, and dots are allowed"
                .to_string(),
        );
    }
    if trimmed.contains("..") || trimmed.starts_with('.') {
        return Err("File name cannot start with '.' or contain '..'".to_string());
    }
    Ok(())
}

// ============================================================================
// Legacy profile migration (one-time)
// ============================================================================

fn legacy_already_migrated(home_dir: &Path) -> Result<bool, String> {
    let settings_path = paths::get_droidgear_settings_path_for_home(home_dir);
    let settings = paths::read_droidgear_settings_from_path_internal(&settings_path)?;
    Ok(settings
        .get(LEGACY_MIGRATED_KEY)
        .and_then(Value::as_bool)
        .unwrap_or(false))
}

fn mark_legacy_migrated(home_dir: &Path) -> Result<(), String> {
    let settings_path = paths::get_droidgear_settings_path_for_home(home_dir);
    let mut settings = paths::read_droidgear_settings_from_path_internal(&settings_path)?;
    if let Some(obj) = settings.as_object_mut() {
        obj.insert(LEGACY_MIGRATED_KEY.to_string(), Value::Bool(true));
    }
    paths::write_droidgear_settings_to_path_internal(&settings_path, &settings)
}

fn slugify_profile_name(name: &str) -> String {
    let mut slug = String::with_capacity(name.len());
    let mut last_was_dash = false;
    for ch in name.chars() {
        if ch.is_ascii_alphanumeric() {
            slug.push(ch.to_ascii_lowercase());
            last_was_dash = false;
        } else if matches!(ch, '-' | '_' | '.') {
            slug.push(ch);
            last_was_dash = false;
        } else if !last_was_dash {
            slug.push('-');
            last_was_dash = true;
        }
    }
    let trimmed = slug.trim_matches(|c| c == '-' || c == '.').to_string();
    if trimmed.is_empty() {
        "profile".to_string()
    } else {
        trimmed
    }
}

fn pick_available_file_name(dir: &Path, base: &str) -> String {
    let candidate = dir.join(base).with_extension("json");
    if !candidate.exists() {
        return base.to_string();
    }
    for i in 2..1000 {
        let next = format!("{base}-{i}");
        if !dir.join(&next).with_extension("json").exists() {
            return next;
        }
    }
    format!("{base}-{}", uuid::Uuid::new_v4())
}

fn build_settings_from_profile(profile: &claude::ClaudeCodeProfile) -> Value {
    let mut root = Map::new();
    let mut env = Map::new();

    if let Some(value) = claude::normalize_optional_string(profile.base_url.as_deref()) {
        env.insert(
            claude::CLAUDE_BASE_URL_ENV.to_string(),
            Value::String(value),
        );
    }
    if let Some(value) = claude::normalize_optional_string(profile.bearer_token.as_deref()) {
        env.insert(
            claude::CLAUDE_AUTH_TOKEN_ENV.to_string(),
            Value::String(value),
        );
    }
    if let Some(value) = claude::normalize_optional_string(profile.model.as_deref()) {
        env.insert(claude::CLAUDE_MODEL_ENV.to_string(), Value::String(value));
    }
    if let Some(value) = claude::resolved_small_model_value(profile) {
        env.insert(
            claude::CLAUDE_SMALL_MODEL_ENV.to_string(),
            Value::String(value),
        );
    }
    if let Some(effort) = profile.reasoning_effort {
        env.insert(
            claude::CLAUDE_EFFORT_ENV.to_string(),
            Value::String(claude::reasoning_effort_to_string(effort).to_string()),
        );
        if matches!(
            effort,
            claude::ClaudeReasoningEffort::High | claude::ClaudeReasoningEffort::Max
        ) {
            env.insert(
                claude::CLAUDE_DISABLE_ADAPTIVE_ENV.to_string(),
                Value::String("1".to_string()),
            );
        }
    }

    match profile.thinking_mode {
        claude::ClaudeThinkingMode::On => {
            root.insert("alwaysThinkingEnabled".to_string(), Value::Bool(true));
        }
        claude::ClaudeThinkingMode::Off => {
            root.insert("alwaysThinkingEnabled".to_string(), Value::Bool(false));
            env.insert(
                claude::CLAUDE_DISABLE_THINKING_ENV.to_string(),
                Value::String("1".to_string()),
            );
        }
        claude::ClaudeThinkingMode::Inherit => {}
    }

    if !env.is_empty() {
        root.insert("env".to_string(), Value::Object(env));
    }

    Value::Object(root)
}

fn legacy_profiles_dir(home_dir: &Path) -> PathBuf {
    home_dir.join(".droidgear").join("claude").join("profiles")
}

/// Migrates any DroidGear-managed legacy profiles to settings files.
/// Runs at most once per machine (tracked in `~/.droidgear/settings.json`).
pub fn migrate_legacy_profiles_for_home(home_dir: &Path) -> Result<u32, String> {
    if legacy_already_migrated(home_dir)? {
        return Ok(0);
    }

    let legacy_dir = legacy_profiles_dir(home_dir);
    if !legacy_dir.exists() {
        let _ = mark_legacy_migrated(home_dir);
        return Ok(0);
    }

    let target_dir = claude_settings_dir_for_home(home_dir);
    if !target_dir.exists() {
        std::fs::create_dir_all(&target_dir)
            .map_err(|e| format!("Failed to create claude-settings directory: {e}"))?;
    }

    let entries = match std::fs::read_dir(&legacy_dir) {
        Ok(value) => value,
        Err(_) => {
            let _ = mark_legacy_migrated(home_dir);
            return Ok(0);
        }
    };

    let mut migrated = 0u32;
    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().and_then(|s| s.to_str()) != Some("json") {
            continue;
        }
        let Ok(content) = std::fs::read_to_string(&path) else {
            continue;
        };
        let Ok(profile) = serde_json::from_str::<claude::ClaudeCodeProfile>(&content) else {
            continue;
        };

        let base_slug = slugify_profile_name(&profile.name);
        let chosen = pick_available_file_name(&target_dir, &base_slug);
        let target_path = target_dir.join(&chosen).with_extension("json");
        let payload = build_settings_from_profile(&profile);
        let bytes = serde_json::to_vec_pretty(&payload)
            .map_err(|e| format!("Failed to serialize migrated settings: {e}"))?;
        if storage::atomic_write(&target_path, &bytes).is_ok() {
            migrated += 1;
        }
    }

    let _ = mark_legacy_migrated(home_dir);
    Ok(migrated)
}

// ============================================================================
// Public API
// ============================================================================

/// Lists every settings file (Global + custom files in `~/.droidgear/claude-settings/`).
pub fn list_settings_files() -> Result<Vec<ClaudeSettingsFileInfo>, String> {
    list_settings_files_for_home(&system_home_dir()?)
}

pub fn list_settings_files_for_home(
    home_dir: &Path,
) -> Result<Vec<ClaudeSettingsFileInfo>, String> {
    let _ = migrate_legacy_profiles_for_home(home_dir);

    let active_path = get_active_settings_path_for_home(home_dir)?;
    let global_path = global_settings_path_for_home(home_dir)?;
    let mut files = Vec::new();

    files.push(ClaudeSettingsFileInfo {
        name: "Global".to_string(),
        path: global_path.to_string_lossy().to_string(),
        is_global: true,
        is_active: active_path == global_path,
        exists: global_path.exists(),
    });

    let dir = claude_settings_dir_for_home(home_dir);
    if dir.exists() {
        if let Ok(entries) = std::fs::read_dir(&dir) {
            let mut customs = Vec::new();
            for entry in entries.flatten() {
                let path = entry.path();
                if path.extension().is_some_and(|ext| ext == "json") {
                    let name = path
                        .file_stem()
                        .and_then(|s| s.to_str())
                        .unwrap_or("unknown")
                        .to_string();
                    let is_active = active_path == path;
                    customs.push(ClaudeSettingsFileInfo {
                        name: name.clone(),
                        path: path.to_string_lossy().to_string(),
                        is_global: false,
                        is_active,
                        exists: true,
                    });
                }
            }
            customs.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
            files.extend(customs);
        }
    }

    Ok(files)
}

pub fn get_active_settings_file() -> Result<ClaudeSettingsFileInfo, String> {
    get_active_settings_file_for_home(&system_home_dir()?)
}

pub fn get_active_settings_file_for_home(
    home_dir: &Path,
) -> Result<ClaudeSettingsFileInfo, String> {
    let files = list_settings_files_for_home(home_dir)?;
    files
        .into_iter()
        .find(|f| f.is_active)
        .ok_or_else(|| "No active Claude settings file found".to_string())
}

/// Resolves a settings file path by display name for a specific home directory.
/// Accepts `Global` (case-insensitive) for the global path.
pub fn get_settings_path_by_name_for_home(home_dir: &Path, name: &str) -> Result<PathBuf, String> {
    if name.eq_ignore_ascii_case("global") {
        return global_settings_path_for_home(home_dir);
    }

    let path = claude_settings_dir_for_home(home_dir)
        .join(name)
        .with_extension("json");
    if path.exists() {
        Ok(path)
    } else {
        Err(format!("Settings file '{name}' does not exist"))
    }
}

pub fn set_active_settings_file(name: Option<String>) -> Result<ClaudeSettingsFileInfo, String> {
    let _ = claude_settings_dir()?;
    set_active_settings_file_for_home(&system_home_dir()?, name)
}

pub fn set_active_settings_file_for_home(
    home_dir: &Path,
    name: Option<String>,
) -> Result<ClaudeSettingsFileInfo, String> {
    match &name {
        Some(value) if !value.is_empty() && !value.eq_ignore_ascii_case("global") => {
            let path = claude_settings_dir_for_home(home_dir)
                .join(value)
                .with_extension("json");
            if !path.exists() {
                return Err(format!("Settings file '{value}' does not exist"));
            }
            save_active_file_name_for_home(home_dir, Some(value))?;
        }
        _ => {
            save_active_file_name_for_home(home_dir, None)?;
        }
    }

    let active_path = get_active_settings_path_for_home(home_dir)?;
    let global_path = global_settings_path_for_home(home_dir)?;
    let is_global = active_path == global_path;

    Ok(ClaudeSettingsFileInfo {
        name: if is_global {
            "Global".to_string()
        } else {
            active_path
                .file_stem()
                .and_then(|value| value.to_str())
                .unwrap_or("unknown")
                .to_string()
        },
        path: active_path.to_string_lossy().to_string(),
        is_global,
        is_active: true,
        exists: active_path.exists(),
    })
}

pub fn create_settings_file(
    name: String,
    copy_from_active: bool,
) -> Result<ClaudeSettingsFileInfo, String> {
    create_settings_file_for_home(&system_home_dir()?, name, copy_from_active)
}

pub fn create_settings_file_for_home(
    home_dir: &Path,
    name: String,
    copy_from_active: bool,
) -> Result<ClaudeSettingsFileInfo, String> {
    validate_custom_file_name(&name)?;

    let dir = claude_settings_dir_for_home(home_dir);
    if !dir.exists() {
        std::fs::create_dir_all(&dir)
            .map_err(|e| format!("Failed to create claude-settings directory: {e}"))?;
    }

    let path = dir.join(&name).with_extension("json");
    if path.exists() {
        return Err(format!("Settings file '{name}' already exists"));
    }

    if copy_from_active {
        let active_path = get_active_settings_path_for_home(home_dir)?;
        if active_path.exists() {
            std::fs::copy(&active_path, &path)
                .map_err(|e| format!("Failed to copy settings: {e}"))?;
        } else {
            storage::atomic_write(&path, b"{}\n")?;
        }
    } else {
        storage::atomic_write(&path, b"{}\n")?;
    }

    save_active_file_name_for_home(home_dir, Some(&name))?;
    get_active_settings_file_for_home(home_dir)
}

pub fn delete_settings_file(name: String) -> Result<(), String> {
    delete_settings_file_for_home(&system_home_dir()?, name)
}

pub fn delete_settings_file_for_home(home_dir: &Path, name: String) -> Result<(), String> {
    if name.eq_ignore_ascii_case("global") {
        return Err("Cannot delete the global settings file".to_string());
    }
    validate_custom_file_name(&name)?;

    let path = claude_settings_dir_for_home(home_dir)
        .join(&name)
        .with_extension("json");
    if !path.exists() {
        return Err(format!("Settings file '{name}' does not exist"));
    }

    std::fs::remove_file(&path).map_err(|e| format!("Failed to delete settings file: {e}"))?;

    if load_active_file_name_for_home(home_dir)?.as_deref() == Some(name.as_str()) {
        save_active_file_name_for_home(home_dir, None)?;
    }

    Ok(())
}

/// Reads the raw JSON contents of a settings file.
/// Returns `{}` when the file does not exist (e.g. fresh global path).
pub fn read_settings_file_for_home(home_dir: &Path, name: &str) -> Result<Value, String> {
    let path = get_settings_path_by_name_for_home(home_dir, name).or_else(|err| {
        if name.eq_ignore_ascii_case("global") {
            global_settings_path_for_home(home_dir)
        } else {
            Err(err)
        }
    })?;
    if !path.exists() {
        return Ok(serde_json::json!({}));
    }
    let content =
        std::fs::read_to_string(&path).map_err(|e| format!("Failed to read settings file: {e}"))?;
    if content.trim().is_empty() {
        return Ok(serde_json::json!({}));
    }
    serde_json::from_str(&content).map_err(|e| format!("Failed to parse settings file: {e}"))
}

pub fn read_settings_file(name: &str) -> Result<Value, String> {
    read_settings_file_for_home(&system_home_dir()?, name)
}

/// Persists the given JSON object as the named settings file. The root value
/// must be a JSON object; otherwise an error is returned.
pub fn save_settings_file_for_home(
    home_dir: &Path,
    name: &str,
    value: &Value,
) -> Result<(), String> {
    if !matches!(value, Value::Object(_)) {
        return Err("Settings file root must be a JSON object".to_string());
    }

    let path = if name.eq_ignore_ascii_case("global") {
        let path = global_settings_path_for_home(home_dir)?;
        if let Some(parent) = path.parent() {
            if !parent.exists() {
                std::fs::create_dir_all(parent)
                    .map_err(|e| format!("Failed to create claude config dir: {e}"))?;
            }
        }
        path
    } else {
        validate_custom_file_name(name)?;
        let dir = claude_settings_dir_for_home(home_dir);
        if !dir.exists() {
            std::fs::create_dir_all(&dir)
                .map_err(|e| format!("Failed to create claude-settings directory: {e}"))?;
        }
        dir.join(name).with_extension("json")
    };

    let bytes = serde_json::to_vec_pretty(value)
        .map_err(|e| format!("Failed to serialize settings file: {e}"))?;
    storage::atomic_write(&path, &bytes)
}

pub fn save_settings_file(name: &str, value: &Value) -> Result<(), String> {
    save_settings_file_for_home(&system_home_dir()?, name, value)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn write_file(path: &Path, contents: &str) {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).unwrap();
        }
        std::fs::write(path, contents).unwrap();
    }

    #[test]
    fn list_includes_global_and_custom_files() {
        let temp = TempDir::new().unwrap();
        let home_dir = temp.path();

        write_file(&home_dir.join(".claude/settings.json"), "{}");
        write_file(&home_dir.join(".droidgear/claude-settings/work.json"), "{}");
        set_active_settings_file_for_home(home_dir, Some("work".to_string())).unwrap();

        let files = list_settings_files_for_home(home_dir).unwrap();
        assert_eq!(files.len(), 2);
        assert!(files.iter().any(|f| f.name == "Global" && f.is_global));
        assert!(files.iter().any(|f| f.name == "work" && f.is_active));

        let active = get_active_settings_file_for_home(home_dir).unwrap();
        assert_eq!(active.name, "work");
        assert!(!active.is_global);
    }

    #[test]
    fn create_file_switches_to_new_file_and_copies_active() {
        let temp = TempDir::new().unwrap();
        let home_dir = temp.path();
        write_file(
            &home_dir.join(".claude/settings.json"),
            r#"{"autoUpdate":false,"env":{"FOO":"bar"}}"#,
        );

        let info = create_settings_file_for_home(home_dir, "personal".to_string(), true).unwrap();
        assert_eq!(info.name, "personal");
        assert!(info.is_active);

        let value = read_settings_file_for_home(home_dir, "personal").unwrap();
        assert_eq!(value["autoUpdate"], Value::Bool(false));
        assert_eq!(value["env"]["FOO"], Value::String("bar".to_string()));
    }

    #[test]
    fn delete_file_resets_active_to_global_when_active_was_removed() {
        let temp = TempDir::new().unwrap();
        let home_dir = temp.path();
        write_file(&home_dir.join(".claude/settings.json"), "{}");
        create_settings_file_for_home(home_dir, "drop".to_string(), false).unwrap();

        delete_settings_file_for_home(home_dir, "drop".to_string()).unwrap();
        let active = get_active_settings_file_for_home(home_dir).unwrap();
        assert!(active.is_global);
    }

    #[test]
    fn create_rejects_reserved_global_name() {
        let temp = TempDir::new().unwrap();
        let err =
            create_settings_file_for_home(temp.path(), "global".to_string(), false).unwrap_err();
        assert!(err.contains("Cannot use 'Global'"));
    }

    #[test]
    fn save_round_trips_through_read() {
        let temp = TempDir::new().unwrap();
        let home_dir = temp.path();
        write_file(&home_dir.join(".claude/settings.json"), "{}");
        create_settings_file_for_home(home_dir, "round".to_string(), false).unwrap();

        let new_value = serde_json::json!({
            "env": {"DISABLE_TELEMETRY": "0"},
            "permissions": {"defaultMode": "acceptEdits"}
        });
        save_settings_file_for_home(home_dir, "round", &new_value).unwrap();

        let loaded = read_settings_file_for_home(home_dir, "round").unwrap();
        assert_eq!(loaded, new_value);
    }

    #[test]
    fn save_rejects_non_object_root() {
        let temp = TempDir::new().unwrap();
        let home_dir = temp.path();
        write_file(&home_dir.join(".claude/settings.json"), "{}");
        create_settings_file_for_home(home_dir, "rejects".to_string(), false).unwrap();
        let err =
            save_settings_file_for_home(home_dir, "rejects", &serde_json::json!([])).unwrap_err();
        assert!(err.contains("must be a JSON object"));
    }

    #[test]
    fn legacy_profiles_are_migrated_into_settings_files() {
        let temp = TempDir::new().unwrap();
        let home_dir = temp.path();
        write_file(
            &home_dir.join(".droidgear/claude/profiles/abc.json"),
            r#"{
              "id":"abc",
              "name":"My Profile",
              "baseUrl":"https://proxy.example.com",
              "bearerToken":"token-1",
              "model":"claude-sonnet-4-5",
              "smallModelUsesMainModel":true,
              "reasoningEffort":"high",
              "thinkingMode":"on",
              "createdAt":"2026-01-01T00:00:00Z",
              "updatedAt":"2026-01-01T00:00:00Z"
            }"#,
        );

        let migrated = migrate_legacy_profiles_for_home(home_dir).unwrap();
        assert_eq!(migrated, 1);

        let files = list_settings_files_for_home(home_dir).unwrap();
        let custom: Vec<_> = files.iter().filter(|f| !f.is_global).collect();
        assert_eq!(custom.len(), 1);
        assert_eq!(custom[0].name, "my-profile");

        let value = read_settings_file_for_home(home_dir, &custom[0].name).unwrap();
        assert_eq!(
            value["env"]["ANTHROPIC_AUTH_TOKEN"],
            Value::String("token-1".to_string())
        );
        assert_eq!(value["alwaysThinkingEnabled"], Value::Bool(true));

        // Idempotent
        let second = migrate_legacy_profiles_for_home(home_dir).unwrap();
        assert_eq!(second, 0);
    }
}
