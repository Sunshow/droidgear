use droidgear_core::{codex, factory_settings, mcp, openclaw, opencode, paths};
use serde_json::Value;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use tempfile::TempDir;

fn write_file(path: &Path, content: &str) {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).unwrap();
    }
    std::fs::write(path, content).unwrap();
}

fn read_to_string(path: &Path) -> String {
    std::fs::read_to_string(path).unwrap()
}

fn home_dir(temp: &TempDir) -> &Path {
    temp.path()
}

fn factory_settings_path(home: &Path) -> PathBuf {
    home.join(".factory").join("settings.json")
}

#[test]
fn codex_apply_preserves_non_model_config() {
    let temp = TempDir::new().unwrap();
    let home = home_dir(&temp);

    // Base config.toml with unrelated fields.
    let base_toml = r#"
model_provider = "openai"
model = "old-model"
model_reasoning_effort = "low"
network_access = "none"

[projects]
foo = "bar"

[model_providers.openai]
name = "OpenAI"
base_url = "https://api.openai.com/v1"
"#;
    write_file(
        &home.join(".codex").join("config.toml"),
        base_toml.trim_start(),
    );

    // Existing auth.json may contain official codex-login credentials; applying a profile must
    // preserve non-OPENAI_API_KEY fields.
    write_file(
        &home.join(".codex").join("auth.json"),
        r#"{
  "session": "official-session-token",
  "user": "alice@example.com"
}"#,
    );

    // Profile file
    let mut providers = HashMap::new();
    providers.insert(
        "custom".to_string(),
        codex::CodexProviderConfig {
            name: Some("Custom Provider".to_string()),
            base_url: Some("https://example.com/v1".to_string()),
            wire_api: Some("responses".to_string()),
            requires_openai_auth: Some(true),
            env_key: None,
            env_key_instructions: None,
            http_headers: None,
            query_params: None,
            model: Some("gpt-5.2".to_string()),
            model_reasoning_effort: Some("high".to_string()),
            api_key: Some("sk-test".to_string()),
        },
    );

    let profile = codex::CodexProfile {
        id: "test_profile".to_string(),
        name: "默认".to_string(),
        description: None,
        created_at: "2026-01-01T00:00:00Z".to_string(),
        updated_at: "2026-01-01T00:00:00Z".to_string(),
        providers,
        model_provider: "custom".to_string(),
        model: "fallback-model".to_string(),
        model_reasoning_effort: Some("medium".to_string()),
        api_key: Some("sk-profile-level".to_string()),
    };
    let profile_json = serde_json::to_string_pretty(&profile).unwrap();
    write_file(
        &home
            .join(".droidgear")
            .join("codex")
            .join("profiles")
            .join("test_profile.json"),
        &profile_json,
    );

    codex::apply_codex_profile_for_home(home, "test_profile").unwrap();

    // config.toml: preserve unrelated keys, update model selection and providers
    let after_toml = read_to_string(&home.join(".codex").join("config.toml"));
    let parsed: toml::Value = toml::from_str(&after_toml).unwrap();

    assert_eq!(
        parsed.get("network_access").and_then(|v| v.as_str()),
        Some("none")
    );
    assert_eq!(
        parsed
            .get("projects")
            .and_then(|v| v.get("foo"))
            .and_then(|v| v.as_str()),
        Some("bar")
    );
    assert_eq!(
        parsed.get("model_provider").and_then(|v| v.as_str()),
        Some("custom")
    );
    assert_eq!(
        parsed.get("model").and_then(|v| v.as_str()),
        Some("gpt-5.2")
    );
    assert_eq!(
        parsed
            .get("model_reasoning_effort")
            .and_then(|v| v.as_str()),
        Some("high")
    );

    let providers = parsed
        .get("model_providers")
        .and_then(|v| v.as_table())
        .unwrap();
    assert!(providers.contains_key("custom"));
    assert!(!providers.contains_key("openai"));

    // auth.json: provider-level api_key wins and is written as OPENAI_API_KEY
    let auth_after = read_to_string(&home.join(".codex").join("auth.json"));
    let auth_json: Value = serde_json::from_str(&auth_after).unwrap();
    assert_eq!(
        auth_json.get("OPENAI_API_KEY").and_then(|v| v.as_str()),
        Some("sk-test")
    );
    assert_eq!(
        auth_json.get("session").and_then(|v| v.as_str()),
        Some("official-session-token")
    );
    assert_eq!(
        auth_json.get("user").and_then(|v| v.as_str()),
        Some("alice@example.com")
    );

    // active profile id
    let active = read_to_string(
        &home
            .join(".droidgear")
            .join("codex")
            .join("active-profile.txt"),
    );
    assert_eq!(active.trim(), "test_profile");
}

#[test]
fn codex_apply_can_remove_openai_api_key_without_destroying_official_auth() {
    let temp = TempDir::new().unwrap();
    let home = home_dir(&temp);

    // Pretend we have codex login credentials + an API key currently set.
    write_file(
        &home.join(".codex").join("auth.json"),
        r#"{
  "session": "official-session-token",
  "OPENAI_API_KEY": "sk-should-be-removed"
}"#,
    );

    // Minimal profile with no api_key configured.
    let profile = codex::CodexProfile {
        id: "no_key".to_string(),
        name: "No Key".to_string(),
        description: None,
        created_at: "2026-01-01T00:00:00Z".to_string(),
        updated_at: "2026-01-01T00:00:00Z".to_string(),
        providers: HashMap::new(),
        model_provider: "openai".to_string(),
        model: "gpt-5.2".to_string(),
        model_reasoning_effort: None,
        api_key: None,
    };
    write_file(
        &home
            .join(".droidgear")
            .join("codex")
            .join("profiles")
            .join("no_key.json"),
        &serde_json::to_string_pretty(&profile).unwrap(),
    );

    codex::apply_codex_profile_for_home(home, "no_key").unwrap();

    let auth_after = read_to_string(&home.join(".codex").join("auth.json"));
    let auth_json: Value = serde_json::from_str(&auth_after).unwrap();
    assert!(auth_json.get("OPENAI_API_KEY").is_none());
    assert_eq!(
        auth_json.get("session").and_then(|v| v.as_str()),
        Some("official-session-token")
    );
}

#[test]
fn codex_auto_creates_official_profile_when_official_auth_exists() {
    let temp = TempDir::new().unwrap();
    let home = home_dir(&temp);

    // Existence of any non-OPENAI_API_KEY field indicates official login credentials exist.
    write_file(
        &home.join(".codex").join("auth.json"),
        r#"{
  "session": "official-session-token"
}"#,
    );

    // Live config present so the profile can snapshot model fields (best-effort).
    write_file(
        &home.join(".codex").join("config.toml"),
        r#"
model_provider = "openai"
model = "gpt-5.2"
"#
        .trim_start(),
    );

    let profiles = codex::list_codex_profiles_for_home(home).unwrap();
    assert!(
        profiles.iter().any(|p| p.id == "official"),
        "expected system official profile to exist"
    );
    assert_eq!(
        profiles[0].id, "official",
        "official profile should be sorted first"
    );

    // Ensure creating the default BYOK profile is still allowed when only official exists.
    let created = codex::create_default_codex_profile_for_home(home).unwrap();
    assert_ne!(created.id, "official");
}

#[test]
fn opencode_apply_prefers_jsonc_and_merges_provider_and_auth() {
    let temp = TempDir::new().unwrap();
    let home = home_dir(&temp);

    // Profile file
    let mut providers = HashMap::new();
    providers.insert(
        "new".to_string(),
        opencode::OpenCodeProviderConfig {
            name: Some("New Provider".to_string()),
            ..Default::default()
        },
    );
    let mut auth = HashMap::new();
    auth.insert(
        "openai".to_string(),
        serde_json::json!({
          "type": "api",
          "key": "sk-opencode"
        }),
    );

    let profile = opencode::OpenCodeProfile {
        id: "p1".to_string(),
        name: "Default".to_string(),
        description: None,
        created_at: "2026-01-01T00:00:00Z".to_string(),
        updated_at: "2026-01-01T00:00:00Z".to_string(),
        providers,
        auth,
    };
    write_file(
        &home
            .join(".droidgear")
            .join("opencode")
            .join("profiles")
            .join("p1.json"),
        &serde_json::to_string_pretty(&profile).unwrap(),
    );

    // Both json and jsonc exist; apply must modify jsonc.
    write_file(
        &home.join(".config").join("opencode").join("opencode.json"),
        r#"{ "provider": { "keep": { "name": "Keep" } } }"#,
    );
    write_file(
        &home.join(".config").join("opencode").join("opencode.jsonc"),
        r#"// comment
{ "provider": { "openai": { "name": "Old" } } }"#,
    );

    // Auth: prefer jsonc when present
    write_file(
        &home
            .join(".local")
            .join("share")
            .join("opencode")
            .join("auth.jsonc"),
        r#"{ "existing": 1 }"#,
    );

    opencode::apply_opencode_profile_for_home(home, "p1").unwrap();

    // opencode.json unchanged
    let json_after = read_to_string(&home.join(".config").join("opencode").join("opencode.json"));
    let json_after_v: Value = serde_json::from_str(&json_after).unwrap();
    assert!(json_after_v
        .get("provider")
        .and_then(|v| v.get("keep"))
        .is_some());

    // opencode.jsonc updated
    let jsonc_after = read_to_string(&home.join(".config").join("opencode").join("opencode.jsonc"));
    let jsonc_after_v: Value = serde_json::from_str(&jsonc_after).unwrap();
    let provider_obj = jsonc_after_v
        .get("provider")
        .and_then(|v| v.as_object())
        .unwrap();
    assert!(provider_obj.contains_key("openai"));
    assert!(provider_obj.contains_key("new"));

    // auth.jsonc updated and merged
    let auth_after = read_to_string(
        &home
            .join(".local")
            .join("share")
            .join("opencode")
            .join("auth.jsonc"),
    );
    let auth_after_v: Value = serde_json::from_str(&auth_after).unwrap();
    assert_eq!(
        auth_after_v.get("existing").and_then(|v| v.as_i64()),
        Some(1)
    );
    assert!(auth_after_v.get("openai").is_some());

    // active profile id
    let active = read_to_string(
        &home
            .join(".droidgear")
            .join("opencode")
            .join("active-profile.txt"),
    );
    assert_eq!(active.trim(), "p1");
}

#[test]
fn openclaw_apply_replaces_models_providers_and_chunk_object() {
    let temp = TempDir::new().unwrap();
    let home = home_dir(&temp);

    // Base config contains a provider that should be replaced, plus unrelated keys.
    let base_config = serde_json::json!({
      "logging": { "level": "info" },
      "agents": {
        "defaults": {
          "model": { "primary": "old", "something": "should-be-removed" },
          "blockStreamingChunk": { "minChars": 1, "maxChars": 2 }
        }
      },
      "models": {
        "mode": "merge",
        "providers": {
          "base": {
            "baseUrl": "https://base.example",
            "models": [
              { "id": "base-model", "name": "Base Model", "reasoning": false }
            ]
          }
        }
      }
    });
    write_file(
        &home.join(".openclaw").join("openclaw.json"),
        &serde_json::to_string_pretty(&base_config).unwrap(),
    );

    // Profile overlay writes a different provider and a partial chunk object (maxChars only).
    let mut providers = HashMap::new();
    providers.insert(
        "new".to_string(),
        openclaw::OpenClawProviderConfig {
            base_url: Some("https://new.example".to_string()),
            api_key: Some("sk-openclaw".to_string()),
            api: None,
            models: vec![openclaw::OpenClawModel {
                id: "m1".to_string(),
                name: Some("M1".to_string()),
                reasoning: true,
                input: vec![],
                context_window: Some(200000),
                max_tokens: Some(8192),
            }],
        },
    );

    let profile = openclaw::OpenClawProfile {
        id: "p1".to_string(),
        name: "Default".to_string(),
        description: None,
        created_at: "2026-01-01T00:00:00Z".to_string(),
        updated_at: "2026-01-01T00:00:00Z".to_string(),
        default_model: Some("new/m1".to_string()),
        failover_models: None,
        providers,
        block_streaming_config: Some(openclaw::BlockStreamingConfig {
            block_streaming_default: None,
            block_streaming_break: None,
            block_streaming_chunk: Some(openclaw::BlockStreamingChunk {
                min_chars: None,
                max_chars: Some(100),
            }),
            block_streaming_coalesce: None,
            telegram_channel: None,
        }),
    };
    write_file(
        &home
            .join(".droidgear")
            .join("openclaw")
            .join("profiles")
            .join("p1.json"),
        &serde_json::to_string_pretty(&profile).unwrap(),
    );

    openclaw::apply_openclaw_profile_for_home(home, "p1").unwrap();

    let after = read_to_string(&home.join(".openclaw").join("openclaw.json"));
    let after_v: Value = serde_json::from_str(&after).unwrap();

    // Unrelated keys preserved.
    assert_eq!(
        after_v
            .get("logging")
            .and_then(|v| v.get("level"))
            .and_then(|v| v.as_str()),
        Some("info")
    );

    // models.providers replaced (base removed, new present)
    let providers_obj = after_v
        .get("models")
        .and_then(|v| v.get("providers"))
        .and_then(|v| v.as_object())
        .unwrap();
    assert!(!providers_obj.contains_key("base"));
    assert!(providers_obj.contains_key("new"));

    // agents.defaults.model replaced (extra key removed)
    let model_obj = after_v
        .get("agents")
        .and_then(|v| v.get("defaults"))
        .and_then(|v| v.get("model"))
        .and_then(|v| v.as_object())
        .unwrap();
    assert_eq!(
        model_obj.get("primary").and_then(|v| v.as_str()),
        Some("new/m1")
    );
    assert!(!model_obj.contains_key("something"));

    // blockStreamingChunk replaced: minChars removed, maxChars updated.
    let chunk_obj = after_v
        .get("agents")
        .and_then(|v| v.get("defaults"))
        .and_then(|v| v.get("blockStreamingChunk"))
        .and_then(|v| v.as_object())
        .unwrap();
    assert_eq!(
        chunk_obj.get("maxChars").and_then(|v| v.as_i64()),
        Some(100)
    );
    assert!(chunk_obj.get("minChars").is_none());

    // active profile id
    let active = read_to_string(
        &home
            .join(".droidgear")
            .join("openclaw")
            .join("active-profile.txt"),
    );
    assert_eq!(active.trim(), "p1");
}

#[test]
fn paths_save_and_reset_roundtrip() {
    let temp = TempDir::new().unwrap();
    let home = home_dir(&temp);

    paths::save_config_path_for_home(home, "codex", "/tmp/codex").unwrap();
    let loaded = paths::load_config_paths_for_home(home);
    assert_eq!(loaded.codex.as_deref(), Some("/tmp/codex"));

    let effective = paths::get_effective_paths_for_home(home).unwrap();
    assert!(!effective.codex.is_default);
    assert_eq!(effective.codex.path, "/tmp/codex");

    paths::reset_config_path_for_home(home, "codex").unwrap();
    let loaded2 = paths::load_config_paths_for_home(home);
    assert!(loaded2.codex.is_none());

    let settings_path = paths::get_droidgear_settings_path_for_home(home);
    let s = read_to_string(&settings_path);
    let v: Value = serde_json::from_str(&s).unwrap();
    assert_eq!(v, serde_json::json!({}));
}

#[test]
fn factory_save_custom_models_preserves_other_fields_and_errors_on_parse() {
    let temp = TempDir::new().unwrap();
    let home = home_dir(&temp);

    // Preserve unrelated keys
    write_file(
        &factory_settings_path(home),
        r#"{ "cloudSessionSync": false, "customModels": [] }"#,
    );

    let model = factory_settings::CustomModel {
        model: "m1".to_string(),
        id: None,
        index: None,
        display_name: Some("M1".to_string()),
        base_url: "https://example.com/v1".to_string(),
        api_key: "sk".to_string(),
        provider: factory_settings::Provider::Openai,
        max_output_tokens: Some(123),
        no_image_support: None,
        extra_args: None,
        extra_headers: None,
    };

    factory_settings::save_custom_models_for_home(home, vec![model]).unwrap();
    let after = read_to_string(&factory_settings_path(home));
    let after_v: Value = serde_json::from_str(&after).unwrap();
    assert_eq!(
        after_v.get("cloudSessionSync").and_then(|v| v.as_bool()),
        Some(false)
    );
    assert_eq!(
        after_v
            .get("customModels")
            .and_then(|v| v.as_array())
            .map(|arr| arr.len()),
        Some(1)
    );

    // Parse error should be surfaced for write operations that require valid JSON
    write_file(&factory_settings_path(home), "{ invalid json");
    let err = factory_settings::save_custom_models_for_home(home, vec![]).unwrap_err();
    assert!(err.starts_with(factory_settings::CONFIG_PARSE_ERROR_PREFIX));
}

#[test]
fn mcp_toggle_missing_server_returns_error() {
    let temp = TempDir::new().unwrap();
    let home = home_dir(&temp);

    let err = mcp::toggle_mcp_server_for_home(home, "missing", true).unwrap_err();
    assert_eq!(err, "Server not found: missing");
}
