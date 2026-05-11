//! Codex temporary-run planning.
//!
//! Builds a runtime `CODEX_HOME` snapshot plus child-process-only env without
//! mutating the live `config.toml` / `auth.json`.

use serde::{Deserialize, Serialize};
use specta::Type;
use std::collections::{BTreeSet, HashMap};
use std::path::{Path, PathBuf};
use uuid::Uuid;

use crate::{codex, json, paths, storage};

const CODEX_CONFIG_SUPPORT_MIN_VERSION: &str = "0.128.0";
const CODEX_RUNTIME_DIR: &str = "runtime/codex";
const TEMP_RUNTIME_PREFIX: &str = "temporary-run-";
const OFFICIAL_PROFILE_ID: &str = "official";
const TOKENS_FIELD: &str = "tokens";
const AUTH_MODE_FIELD: &str = "auth_mode";
const LAST_REFRESH_FIELD: &str = "last_refresh";
const AGENT_IDENTITY_FIELD: &str = "agent_identity";
const MANAGED_RUNTIME_FILES: [&str; 2] = ["config.toml", "auth.json"];

#[derive(Debug, Clone, Serialize, Deserialize, Type, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct CodexCliCapability {
    pub version: String,
    pub supports_config_override: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct CodexTemporaryRunPlan {
    pub program: String,
    pub args: Vec<String>,
    pub env: Vec<(String, String)>,
    pub unset_env: Vec<String>,
    pub secret_env_keys: Vec<String>,
    pub warnings: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CodexTemporaryLaunchPlan {
    pub program: String,
    pub args: Vec<String>,
    pub env: Vec<(String, String)>,
    pub secret_env: Vec<(String, String)>,
    pub unset_env: Vec<String>,
    pub warnings: Vec<String>,
    pub runtime_home_path: PathBuf,
}

impl From<&CodexTemporaryLaunchPlan> for CodexTemporaryRunPlan {
    fn from(plan: &CodexTemporaryLaunchPlan) -> Self {
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

fn quote_toml_string(value: &str) -> String {
    let escaped = value.replace('\\', "\\\\").replace('"', "\\\"");
    format!("\"{escaped}\"")
}

fn validate_provider_id(provider_id: &str) -> Result<(), String> {
    if provider_id.is_empty() {
        return Err("Codex provider id cannot be empty".to_string());
    }

    let valid = provider_id
        .chars()
        .all(|ch| ch.is_ascii_alphanumeric() || ch == '_' || ch == '-');
    if valid {
        Ok(())
    } else {
        Err(format!(
            "Invalid Codex provider id '{provider_id}': only alphanumeric characters, '_' and '-' are allowed"
        ))
    }
}

fn provider_overrides(provider_id: &str, provider: &codex::CodexProviderConfig) -> Vec<String> {
    let mut overrides = Vec::new();

    if let Some(name) = provider.name.as_deref() {
        overrides.push(format!(
            "model_providers.{provider_id}.name={}",
            quote_toml_string(name)
        ));
    }
    if let Some(base_url) = provider.base_url.as_deref() {
        overrides.push(format!(
            "model_providers.{provider_id}.base_url={}",
            quote_toml_string(base_url)
        ));
    }
    if let Some(wire_api) = provider.wire_api.as_deref() {
        overrides.push(format!(
            "model_providers.{provider_id}.wire_api={}",
            quote_toml_string(wire_api)
        ));
    }
    if let Some(requires_openai_auth) = provider.requires_openai_auth {
        overrides.push(format!(
            "model_providers.{provider_id}.requires_openai_auth={requires_openai_auth}"
        ));
    }
    if let Some(env_key) = provider.env_key.as_deref() {
        overrides.push(format!(
            "model_providers.{provider_id}.env_key={}",
            quote_toml_string(env_key)
        ));
    }
    if let Some(env_key_instructions) = provider.env_key_instructions.as_deref() {
        overrides.push(format!(
            "model_providers.{provider_id}.env_key_instructions={}",
            quote_toml_string(env_key_instructions)
        ));
    }
    if let Some(http_headers) = provider.http_headers.as_ref() {
        for (key, value) in http_headers {
            overrides.push(format!(
                "model_providers.{provider_id}.http_headers.{key}={}",
                quote_toml_string(value)
            ));
        }
    }
    if let Some(query_params) = provider.query_params.as_ref() {
        for (key, value) in query_params {
            overrides.push(format!(
                "model_providers.{provider_id}.query_params.{key}={}",
                quote_toml_string(value)
            ));
        }
    }

    overrides
}

pub fn build_cli_overrides(profile: &codex::CodexProfile) -> Result<Vec<String>, String> {
    let (provider_id, provider) = codex::resolve_active_provider(profile);
    validate_provider_id(&provider_id)?;

    let mut overrides = vec![
        format!("model_provider={}", quote_toml_string(&provider_id)),
        format!(
            "model={}",
            quote_toml_string(&codex::resolved_model(profile, provider))
        ),
    ];

    if let Some(effort) = codex::resolved_reasoning_effort(profile, provider) {
        overrides.push(format!(
            "model_reasoning_effort={}",
            quote_toml_string(&effort)
        ));
    }

    if let Some(provider) = provider {
        overrides.extend(provider_overrides(&provider_id, provider));
    }

    Ok(overrides)
}

fn runtime_dir_for_home(home_dir: &Path) -> PathBuf {
    crate::paths::droidgear_dir_from_home(home_dir).join(CODEX_RUNTIME_DIR)
}

fn next_runtime_home_path(home_dir: &Path) -> Result<PathBuf, String> {
    let runtime_dir = runtime_dir_for_home(home_dir);
    if !runtime_dir.exists() {
        std::fs::create_dir_all(&runtime_dir)
            .map_err(|e| format!("Failed to create Codex runtime directory: {e}"))?;
    }

    let timestamp = chrono::Utc::now().format("%Y%m%dT%H%M%S%.3fZ");
    Ok(runtime_dir.join(format!(
        "{TEMP_RUNTIME_PREFIX}{timestamp}-{}",
        Uuid::new_v4()
    )))
}

fn read_config_template(path: &Path) -> Result<toml::map::Map<String, toml::Value>, String> {
    if !path.exists() {
        return Ok(toml::map::Map::new());
    }

    let contents =
        std::fs::read_to_string(path).map_err(|e| format!("Failed to read config.toml: {e}"))?;
    if contents.trim().is_empty() {
        Ok(toml::map::Map::new())
    } else {
        toml::from_str::<toml::map::Map<String, toml::Value>>(&contents)
            .map_err(|e| format!("Failed to parse config.toml: {e}"))
    }
}

fn write_config_snapshot(
    runtime_home_path: &Path,
    config: &toml::map::Map<String, toml::Value>,
) -> Result<(), String> {
    let config_path = runtime_home_path.join("config.toml");
    let toml_str = toml::to_string_pretty(config)
        .map_err(|e| format!("Failed to serialize config.toml: {e}"))?;
    storage::atomic_write(&config_path, toml_str.as_bytes())
}

fn read_auth_template(path: &Path) -> HashMap<String, serde_json::Value> {
    json::read_json_object_file(path).unwrap_or_default()
}

fn write_auth_snapshot(
    runtime_home_path: &Path,
    auth: &HashMap<String, serde_json::Value>,
) -> Result<(), String> {
    json::write_json_object_file(&runtime_home_path.join("auth.json"), auth)
}

fn create_shared_entry(target: &Path, link_path: &Path) -> Result<(), String> {
    #[cfg(unix)]
    {
        std::os::unix::fs::symlink(target, link_path).map_err(|e| {
            format!(
                "Failed to create shared Codex runtime entry {:?} -> {:?}: {e}",
                link_path, target
            )
        })
    }

    #[cfg(windows)]
    {
        let metadata = std::fs::metadata(target).map_err(|e| {
            format!(
                "Failed to inspect shared Codex runtime target {:?}: {e}",
                target
            )
        })?;
        let result = if metadata.is_dir() {
            std::os::windows::fs::symlink_dir(target, link_path)
        } else {
            std::os::windows::fs::symlink_file(target, link_path)
        };
        result.map_err(|e| {
            format!(
                "Failed to create shared Codex runtime entry {:?} -> {:?}: {e}",
                link_path, target
            )
        })
    }
}

fn populate_runtime_shared_entries(
    live_codex_home: &Path,
    runtime_home_path: &Path,
) -> Result<(), String> {
    if !live_codex_home.exists() {
        return Ok(());
    }

    let entries = std::fs::read_dir(live_codex_home)
        .map_err(|e| format!("Failed to read live CODEX_HOME: {e}"))?;
    for entry in entries {
        let entry = entry.map_err(|e| format!("Failed to read live CODEX_HOME entry: {e}"))?;
        let file_name = entry.file_name();
        let Some(file_name_str) = file_name.to_str() else {
            continue;
        };
        if MANAGED_RUNTIME_FILES.contains(&file_name_str) {
            continue;
        }

        let link_path = runtime_home_path.join(&file_name);
        if link_path.exists() {
            continue;
        }

        create_shared_entry(&entry.path(), &link_path)?;
    }

    Ok(())
}

fn has_portable_managed_auth(auth: &HashMap<String, serde_json::Value>) -> bool {
    auth.contains_key(TOKENS_FIELD)
        || auth.contains_key(AUTH_MODE_FIELD)
        || auth.contains_key(LAST_REFRESH_FIELD)
        || auth.contains_key(AGENT_IDENTITY_FIELD)
}

fn build_runtime_auth_snapshot(
    profile: &codex::CodexProfile,
    provider: Option<&codex::CodexProviderConfig>,
    mut live_auth: HashMap<String, serde_json::Value>,
) -> Result<Option<HashMap<String, serde_json::Value>>, String> {
    if let Some(api_key) = codex::resolved_api_key(profile, provider) {
        let mut auth = HashMap::new();
        auth.insert(
            codex::OPENAI_API_KEY_FIELD.to_string(),
            serde_json::Value::String(api_key),
        );
        return Ok(Some(auth));
    }

    codex::apply_api_key_to_auth_map(&mut live_auth, None);

    if has_portable_managed_auth(&live_auth) {
        return Ok(Some(live_auth));
    }

    if profile.id == OFFICIAL_PROFILE_ID {
        return Err(
            "Codex temporary run could not snapshot official auth from live CODEX_HOME. Keyring-backed official auth is not supported yet; use a file-backed Codex auth store or an API-key profile.".to_string(),
        );
    }

    if provider.and_then(|config| config.requires_openai_auth) == Some(true) {
        return Err(
            "Codex temporary run requires live Codex auth for this profile, but no portable auth snapshot was available. Use an API-key profile or switch Codex CLI auth storage to file.".to_string(),
        );
    }

    Ok(None)
}

fn build_secret_env(
    profile: &codex::CodexProfile,
    provider: Option<&codex::CodexProviderConfig>,
) -> Vec<(String, String)> {
    let Some(api_key) = codex::resolved_api_key(profile, provider) else {
        return Vec::new();
    };

    let env_key = provider
        .and_then(|config| config.env_key.as_deref())
        .filter(|key| !key.is_empty())
        .unwrap_or(codex::OPENAI_API_KEY_FIELD);

    vec![(env_key.to_string(), api_key)]
}

fn build_unset_env(profile: &codex::CodexProfile) -> Vec<String> {
    let mut unset_env = BTreeSet::from([codex::OPENAI_API_KEY_FIELD.to_string()]);

    for provider in profile.providers.values() {
        if let Some(env_key) = provider.env_key.as_deref().filter(|key| !key.is_empty()) {
            unset_env.insert(env_key.to_string());
        }
    }

    unset_env.into_iter().collect()
}

fn build_runtime_home_snapshot(
    home_dir: &Path,
    profile: &codex::CodexProfile,
) -> Result<PathBuf, String> {
    let config_paths = paths::load_config_paths_for_home(home_dir);
    let live_codex_home = paths::get_codex_home_for_home(home_dir, &config_paths)?;
    let runtime_home_path = next_runtime_home_path(home_dir)?;
    std::fs::create_dir_all(&runtime_home_path)
        .map_err(|e| format!("Failed to create Codex runtime home: {e}"))?;
    populate_runtime_shared_entries(&live_codex_home, &runtime_home_path)?;

    let (provider_id, provider) = codex::resolve_active_provider(profile);
    validate_provider_id(&provider_id)?;

    let mut config = read_config_template(&live_codex_home.join("config.toml"))?;
    codex::apply_profile_to_config_map(&mut config, profile)?;
    write_config_snapshot(&runtime_home_path, &config)?;

    let live_auth = read_auth_template(&live_codex_home.join("auth.json"));
    if let Some(auth) = build_runtime_auth_snapshot(profile, provider, live_auth)? {
        write_auth_snapshot(&runtime_home_path, &auth)?;
    }

    Ok(runtime_home_path)
}

pub fn cleanup_stale_runtime_homes_for_home(home_dir: &Path) -> Result<u32, String> {
    let runtime_dir = runtime_dir_for_home(home_dir);
    if !runtime_dir.exists() {
        return Ok(0);
    }

    let cutoff = std::time::SystemTime::now()
        .checked_sub(std::time::Duration::from_secs(60 * 60 * 24))
        .ok_or_else(|| "Failed to compute Codex runtime cleanup cutoff".to_string())?;

    let mut removed = 0;
    let entries = std::fs::read_dir(&runtime_dir)
        .map_err(|e| format!("Failed to read Codex runtime directory: {e}"))?;

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

fn parse_version(version: &str) -> Vec<u64> {
    version
        .trim()
        .trim_start_matches("codex-cli")
        .trim()
        .split('.')
        .map(|part| part.parse::<u64>().unwrap_or(0))
        .collect()
}

fn version_is_at_least(found: &str, minimum: &str) -> bool {
    let found_parts = parse_version(found);
    let minimum_parts = parse_version(minimum);
    let len = found_parts.len().max(minimum_parts.len());

    for index in 0..len {
        let found_part = *found_parts.get(index).unwrap_or(&0);
        let minimum_part = *minimum_parts.get(index).unwrap_or(&0);
        match found_part.cmp(&minimum_part) {
            std::cmp::Ordering::Greater => return true,
            std::cmp::Ordering::Less => return false,
            std::cmp::Ordering::Equal => {}
        }
    }

    true
}

pub fn parse_codex_cli_capability(help_output: &str, version_output: &str) -> CodexCliCapability {
    CodexCliCapability {
        version: version_output.trim().to_string(),
        supports_config_override: help_output.contains("--config <key=value>")
            && version_is_at_least(version_output, CODEX_CONFIG_SUPPORT_MIN_VERSION),
    }
}

pub fn validate_cli_capability(capability: &CodexCliCapability) -> Result<(), String> {
    if capability.supports_config_override {
        Ok(())
    } else {
        Err(format!(
            "Installed Codex CLI ({}) does not support --config overrides required for temporary run",
            capability.version
        ))
    }
}

pub fn build_temporary_run_plan_for_home(
    home_dir: &Path,
    profile: &codex::CodexProfile,
) -> Result<CodexTemporaryLaunchPlan, String> {
    let (provider_id, provider) = codex::resolve_active_provider(profile);
    validate_provider_id(&provider_id)?;

    let runtime_home_path = build_runtime_home_snapshot(home_dir, profile)?;
    let secret_env = build_secret_env(profile, provider);

    let mut warnings = Vec::new();
    if provider
        .and_then(|config| config.env_key.as_deref())
        .filter(|key| !key.is_empty())
        .is_none()
        && !secret_env.is_empty()
    {
        warnings.push(
            "Codex provider did not declare env_key; temporary run will inject OPENAI_API_KEY"
                .to_string(),
        );
    }

    Ok(CodexTemporaryLaunchPlan {
        program: "codex".to_string(),
        args: Vec::new(),
        env: vec![(
            "CODEX_HOME".to_string(),
            runtime_home_path.to_string_lossy().to_string(),
        )],
        secret_env,
        unset_env: build_unset_env(profile),
        warnings,
        runtime_home_path,
    })
}

pub fn build_temporary_run_plan(
    profile: &codex::CodexProfile,
) -> Result<CodexTemporaryLaunchPlan, String> {
    let home_dir = dirs::home_dir().ok_or_else(|| "Failed to get home directory".to_string())?;
    build_temporary_run_plan_for_home(&home_dir, profile)
}

#[cfg(test)]
mod tests {
    use super::{
        build_cli_overrides, build_temporary_run_plan_for_home,
        cleanup_stale_runtime_homes_for_home, parse_codex_cli_capability, validate_cli_capability,
        version_is_at_least,
    };
    use crate::codex::{CodexProfile, CodexProviderConfig};
    use std::collections::HashMap;
    use std::path::Path;
    use tempfile::TempDir;

    fn sample_profile() -> CodexProfile {
        let mut providers = HashMap::new();
        providers.insert(
            "custom".to_string(),
            CodexProviderConfig {
                name: Some("Custom Provider".to_string()),
                base_url: Some("https://example.com/v1".to_string()),
                wire_api: Some("responses".to_string()),
                requires_openai_auth: Some(false),
                env_key: Some("EXAMPLE_API_KEY".to_string()),
                env_key_instructions: Some("Set EXAMPLE_API_KEY".to_string()),
                http_headers: Some(HashMap::from([("X-Test".to_string(), "abc".to_string())])),
                query_params: Some(HashMap::from([(
                    "api-version".to_string(),
                    "2026-01-01".to_string(),
                )])),
                model: Some("gpt-5.5".to_string()),
                model_reasoning_effort: Some("high".to_string()),
                api_key: Some("sk-provider".to_string()),
            },
        );

        CodexProfile {
            id: "demo".to_string(),
            name: "Demo".to_string(),
            description: None,
            created_at: "2026-01-01T00:00:00Z".to_string(),
            updated_at: "2026-01-01T00:00:00Z".to_string(),
            providers,
            model_provider: "custom".to_string(),
            model: "fallback-model".to_string(),
            model_reasoning_effort: Some("medium".to_string()),
            api_key: Some("sk-profile".to_string()),
        }
    }

    fn write_file(path: &Path, contents: &str) {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).unwrap();
        }
        std::fs::write(path, contents).unwrap();
    }

    #[test]
    fn cli_overrides_include_model_provider_provider_fields_and_reasoning_effort() {
        let overrides = build_cli_overrides(&sample_profile()).unwrap();

        assert_eq!(overrides[0], r#"model_provider="custom""#);
        assert_eq!(overrides[1], r#"model="gpt-5.5""#);
        assert!(overrides.contains(&r#"model_reasoning_effort="high""#.to_string()));
        assert!(overrides
            .contains(&r#"model_providers.custom.base_url="https://example.com/v1""#.to_string()));
        assert!(overrides.contains(&r#"model_providers.custom.wire_api="responses""#.to_string()));
        assert!(
            overrides.contains(&r#"model_providers.custom.env_key="EXAMPLE_API_KEY""#.to_string())
        );
        assert!(overrides.contains(
            &r#"model_providers.custom.env_key_instructions="Set EXAMPLE_API_KEY""#.to_string()
        ));
        assert!(
            overrides.contains(&r#"model_providers.custom.http_headers.X-Test="abc""#.to_string())
        );
        assert!(overrides.contains(
            &r#"model_providers.custom.query_params.api-version="2026-01-01""#.to_string()
        ));
    }

    #[test]
    fn temporary_run_plan_uses_runtime_codex_home_and_provider_specific_secret_env() {
        let temp = TempDir::new().unwrap();
        let live_home = temp.path().join("live-codex-home");
        crate::paths::save_config_path_for_home(
            temp.path(),
            "codex",
            live_home.to_string_lossy().as_ref(),
        )
        .unwrap();

        write_file(
            &live_home.join("config.toml"),
            "network_access = \"restricted\"\n",
        );

        let plan = build_temporary_run_plan_for_home(temp.path(), &sample_profile()).unwrap();

        assert_eq!(plan.program, "codex");
        assert!(plan.args.is_empty());
        assert_eq!(plan.env.len(), 1);
        assert_eq!(plan.env[0].0, "CODEX_HOME");
        assert!(
            plan.env[0]
                .1
                .contains(".droidgear/runtime/codex/temporary-run-"),
            "expected runtime CODEX_HOME, got {:?}",
            plan.env
        );
        assert_eq!(
            plan.secret_env,
            vec![("EXAMPLE_API_KEY".to_string(), "sk-provider".to_string())]
        );
        assert_eq!(
            plan.unset_env,
            vec!["EXAMPLE_API_KEY".to_string(), "OPENAI_API_KEY".to_string(),]
        );

        let runtime_config =
            std::fs::read_to_string(plan.runtime_home_path.join("config.toml")).unwrap();
        let config: toml::Value = toml::from_str(&runtime_config).unwrap();
        assert_eq!(
            config.get("network_access").and_then(|v| v.as_str()),
            Some("restricted")
        );
        assert_eq!(
            config.get("model_provider").and_then(|v| v.as_str()),
            Some("custom")
        );
        assert_eq!(
            config.get("model").and_then(|v| v.as_str()),
            Some("gpt-5.5")
        );
    }

    #[test]
    fn temporary_run_plan_shares_existing_live_codex_state_entries() {
        let temp = TempDir::new().unwrap();
        let live_home = temp.path().join("live-codex-home");
        crate::paths::save_config_path_for_home(
            temp.path(),
            "codex",
            live_home.to_string_lossy().as_ref(),
        )
        .unwrap();

        write_file(&live_home.join("config.toml"), "model = \"live-model\"\n");
        write_file(&live_home.join("auth.json"), "{}\n");
        write_file(
            &live_home.join("session_index.jsonl"),
            "{\"id\":\"session-a\"}\n",
        );
        write_file(&live_home.join("history.jsonl"), "{\"prompt\":\"hello\"}\n");
        std::fs::create_dir_all(live_home.join("sessions/2026/05")).unwrap();
        write_file(
            &live_home.join("sessions/2026/05/session-a.jsonl"),
            "{\"type\":\"message\"}\n",
        );
        write_file(&live_home.join("threads.db"), "sqlite-bytes");
        write_file(&live_home.join("state_5.sqlite"), "sqlite-state");

        let plan = build_temporary_run_plan_for_home(temp.path(), &sample_profile()).unwrap();

        let runtime_session_index = plan.runtime_home_path.join("session_index.jsonl");
        let runtime_history = plan.runtime_home_path.join("history.jsonl");
        let runtime_sessions = plan.runtime_home_path.join("sessions");
        let runtime_threads = plan.runtime_home_path.join("threads.db");
        let runtime_state = plan.runtime_home_path.join("state_5.sqlite");

        assert!(runtime_session_index.exists());
        assert!(runtime_history.exists());
        assert!(runtime_sessions.exists());
        assert!(runtime_threads.exists());
        assert!(runtime_state.exists());

        #[cfg(unix)]
        {
            assert!(std::fs::symlink_metadata(&runtime_session_index)
                .unwrap()
                .file_type()
                .is_symlink());
            assert!(std::fs::symlink_metadata(&runtime_history)
                .unwrap()
                .file_type()
                .is_symlink());
            assert!(std::fs::symlink_metadata(&runtime_sessions)
                .unwrap()
                .file_type()
                .is_symlink());
            assert!(std::fs::symlink_metadata(&runtime_threads)
                .unwrap()
                .file_type()
                .is_symlink());
            assert!(std::fs::symlink_metadata(&runtime_state)
                .unwrap()
                .file_type()
                .is_symlink());
        }
    }

    #[test]
    fn temporary_run_plan_clears_openai_api_key_from_runtime_auth_snapshot() {
        let temp = TempDir::new().unwrap();
        let live_home = temp.path().join("live-codex-home");
        crate::paths::save_config_path_for_home(
            temp.path(),
            "codex",
            live_home.to_string_lossy().as_ref(),
        )
        .unwrap();

        write_file(
            &live_home.join("auth.json"),
            r#"{
  "auth_mode": "chatgpt",
  "tokens": {
    "access_token": "access-token",
    "refresh_token": "refresh-token"
  },
  "OPENAI_API_KEY": "sk-live"
}"#,
        );

        let plan = build_temporary_run_plan_for_home(temp.path(), &sample_profile()).unwrap();
        let runtime_auth =
            std::fs::read_to_string(plan.runtime_home_path.join("auth.json")).unwrap();
        let auth: serde_json::Value = serde_json::from_str(&runtime_auth).unwrap();

        assert_eq!(
            auth.get("OPENAI_API_KEY").and_then(|value| value.as_str()),
            Some("sk-provider")
        );
        assert!(auth.get("tokens").is_none());
        assert!(auth.get("auth_mode").is_none());
    }

    #[test]
    fn temporary_run_plan_resets_optional_values_in_runtime_config_snapshot() {
        let temp = TempDir::new().unwrap();
        let live_home = temp.path().join("live-codex-home");
        crate::paths::save_config_path_for_home(
            temp.path(),
            "codex",
            live_home.to_string_lossy().as_ref(),
        )
        .unwrap();

        write_file(
            &live_home.join("config.toml"),
            r#"
model_provider = "custom"
model = "old-model"
model_reasoning_effort = "high"

[model_providers.custom]
name = "Old Provider"
base_url = "https://old.example.com/v1"
wire_api = "responses"
env_key = "OLD_API_KEY"
"#
            .trim_start(),
        );

        let mut profile = sample_profile();
        profile.model_reasoning_effort = None;
        let provider = profile.providers.get_mut("custom").unwrap();
        provider.base_url = None;
        provider.wire_api = None;
        provider.env_key = None;
        provider.env_key_instructions = None;
        provider.model_reasoning_effort = None;

        let plan = build_temporary_run_plan_for_home(temp.path(), &profile).unwrap();
        let runtime_config =
            std::fs::read_to_string(plan.runtime_home_path.join("config.toml")).unwrap();
        let config: toml::Value = toml::from_str(&runtime_config).unwrap();

        assert!(config.get("model_reasoning_effort").is_none());
        let provider = config
            .get("model_providers")
            .and_then(|value| value.get("custom"))
            .unwrap();
        assert!(provider.get("base_url").is_none());
        assert!(provider.get("wire_api").is_none());
        assert!(provider.get("env_key").is_none());
    }

    #[test]
    fn temporary_run_plan_falls_back_to_openai_api_key_when_env_key_is_missing() {
        let temp = TempDir::new().unwrap();
        let mut profile = sample_profile();
        let provider = profile.providers.get_mut("custom").unwrap();
        provider.env_key = None;
        provider.env_key_instructions = None;

        let plan = build_temporary_run_plan_for_home(temp.path(), &profile).unwrap();

        assert_eq!(
            plan.secret_env,
            vec![("OPENAI_API_KEY".to_string(), "sk-provider".to_string())]
        );
        assert_eq!(plan.unset_env, vec!["OPENAI_API_KEY".to_string()]);
        assert!(plan.warnings.contains(
            &"Codex provider did not declare env_key; temporary run will inject OPENAI_API_KEY"
                .to_string()
        ));
    }

    #[test]
    fn temporary_run_plan_does_not_inject_auth_env_when_profile_has_no_api_key() {
        let temp = TempDir::new().unwrap();
        let mut profile = sample_profile();
        profile.api_key = None;
        let provider = profile.providers.get_mut("custom").unwrap();
        provider.api_key = None;

        let plan = build_temporary_run_plan_for_home(temp.path(), &profile).unwrap();

        assert!(plan.secret_env.is_empty());
        assert_eq!(
            plan.unset_env,
            vec!["EXAMPLE_API_KEY".to_string(), "OPENAI_API_KEY".to_string(),]
        );
    }

    #[test]
    fn temporary_run_plan_skips_empty_runtime_auth_snapshot_when_no_auth_is_needed() {
        let temp = TempDir::new().unwrap();
        let live_home = temp.path().join("live-codex-home");
        crate::paths::save_config_path_for_home(
            temp.path(),
            "codex",
            live_home.to_string_lossy().as_ref(),
        )
        .unwrap();

        let mut profile = sample_profile();
        profile.api_key = None;
        let provider = profile.providers.get_mut("custom").unwrap();
        provider.api_key = None;
        provider.requires_openai_auth = Some(false);

        let plan = build_temporary_run_plan_for_home(temp.path(), &profile).unwrap();

        assert!(!plan.runtime_home_path.join("auth.json").exists());
    }

    #[test]
    fn temporary_run_plan_errors_when_official_auth_cannot_be_portably_snapshotted() {
        let temp = TempDir::new().unwrap();
        let profile = CodexProfile {
            id: "official".to_string(),
            name: "Official".to_string(),
            description: None,
            created_at: "2026-01-01T00:00:00Z".to_string(),
            updated_at: "2026-01-01T00:00:00Z".to_string(),
            providers: HashMap::new(),
            model_provider: "openai".to_string(),
            model: "gpt-5".to_string(),
            model_reasoning_effort: None,
            api_key: None,
        };

        let error = build_temporary_run_plan_for_home(temp.path(), &profile).unwrap_err();

        assert!(error.contains("could not snapshot official auth"));
    }

    #[test]
    fn cli_capability_parsing_requires_config_override_support_and_min_version() {
        let capability = parse_codex_cli_capability(
            "Usage: codex\n  -c, --config <key=value>\n",
            "codex-cli 0.128.0\n",
        );
        assert!(capability.supports_config_override);
        validate_cli_capability(&capability).unwrap();

        let too_old = parse_codex_cli_capability(
            "Usage: codex\n  -c, --config <key=value>\n",
            "codex-cli 0.127.9\n",
        );
        assert!(!too_old.supports_config_override);
        assert!(validate_cli_capability(&too_old).is_err());
    }

    #[test]
    fn version_comparison_handles_multi_segment_versions() {
        assert!(version_is_at_least("codex-cli 0.128.0", "0.128.0"));
        assert!(version_is_at_least("codex-cli 0.128.1", "0.128.0"));
        assert!(version_is_at_least("codex-cli 0.129.0", "0.128.0"));
        assert!(!version_is_at_least("codex-cli 0.127.9", "0.128.0"));
    }

    #[test]
    fn cleanup_stale_runtime_homes_only_removes_old_runtime_directories() {
        let temp = TempDir::new().unwrap();
        let runtime_dir = temp.path().join(".droidgear/runtime/codex");
        std::fs::create_dir_all(&runtime_dir).unwrap();

        let stale_dir = runtime_dir.join("temporary-run-20000101T000000.000Z-stale");
        let fresh_dir = runtime_dir.join("temporary-run-keep");
        let other_dir = runtime_dir.join("notes");
        std::fs::create_dir_all(&stale_dir).unwrap();
        std::fs::create_dir_all(&fresh_dir).unwrap();
        std::fs::create_dir_all(&other_dir).unwrap();

        let old = filetime::FileTime::from_unix_time(946684800, 0);
        filetime::set_file_mtime(&stale_dir, old).unwrap();

        let removed = cleanup_stale_runtime_homes_for_home(temp.path()).unwrap();

        assert_eq!(removed, 1);
        assert!(!stale_dir.exists());
        assert!(fresh_dir.exists());
        assert!(other_dir.exists());
    }
}
