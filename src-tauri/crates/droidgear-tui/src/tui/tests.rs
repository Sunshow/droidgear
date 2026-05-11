use super::*;
use crate::tui::utils::{
    format_claude_temporary_run_preview, load_droid_run_preferences_from_path,
    preview_codex_temporary_run, preview_droid_temporary_run,
};
use crossterm::event::KeyCode;
use std::collections::HashMap;
use std::path::Path;
use tempfile::TempDir;

fn write_file(path: &Path, contents: &str) {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).unwrap();
    }
    std::fs::write(path, contents).unwrap();
}

fn make_claude_profile(id: &str, name: &str) -> droidgear_core::claude::ClaudeCodeProfile {
    droidgear_core::claude::ClaudeCodeProfile {
        id: id.to_string(),
        name: name.to_string(),
        description: None,
        base_url: None,
        bearer_token: None,
        model: None,
        small_model_uses_main_model: true,
        small_model: None,
        reasoning_effort: None,
        thinking_mode: droidgear_core::claude::ClaudeThinkingMode::Inherit,
        created_at: "2026-01-01T00:00:00Z".to_string(),
        updated_at: "2026-01-01T00:00:00Z".to_string(),
    }
}

#[test]
fn normalize_factory_models_sets_index_and_id() {
    let mut models = vec![
        droidgear_core::factory_settings::CustomModel {
            model: "m1".to_string(),
            id: None,
            index: None,
            display_name: Some("My Model".to_string()),
            base_url: "https://api.example.test".to_string(),
            api_key: "sk-test".to_string(),
            provider: droidgear_core::factory_settings::Provider::Openai,
            max_output_tokens: None,
            no_image_support: None,
            extra_args: None,
            extra_headers: None,
        },
        droidgear_core::factory_settings::CustomModel {
            model: "m2".to_string(),
            id: None,
            index: None,
            display_name: None,
            base_url: "https://api.example.test".to_string(),
            api_key: "sk-test".to_string(),
            provider: droidgear_core::factory_settings::Provider::Openai,
            max_output_tokens: None,
            no_image_support: None,
            extra_args: None,
            extra_headers: None,
        },
    ];

    normalize_factory_models(&mut models);

    assert_eq!(models[0].index, Some(0));
    assert_eq!(models[0].id.as_deref(), Some("custom:My Model-0"));
    assert_eq!(models[1].index, Some(1));
    assert_eq!(models[1].id.as_deref(), Some("custom:m2-1"));
}

#[test]
fn hermes_screens_are_included_in_nav_items() {
    let nav = app::App::nav_items();
    let has_hermes = nav
        .iter()
        .any(|(label, screen)| *label == "Hermes" && *screen == app::Screen::Hermes);
    assert!(has_hermes, "nav_items() should include Hermes entry");
}

#[test]
fn hermes_app_state_initializes_correctly() {
    use std::path::PathBuf;
    let app = app::App::new(PathBuf::from("/tmp/test-home"));
    assert!(app.hermes_profiles.is_empty());
    assert!(app.hermes_active_id.is_none());
    assert_eq!(app.hermes_index, 0);
    assert!(app.hermes_detail_id.is_none());
    assert!(app.hermes_detail.is_none());
    assert_eq!(app.hermes_detail_field_index, 0);
    assert_eq!(app.hermes_provider_field_index, 0);
}

#[test]
fn hermes_clamp_indices_does_not_panic_on_empty_profiles() {
    use std::path::PathBuf;
    let mut app = app::App::new(PathBuf::from("/tmp/test-home"));
    // Should not panic when hermes_profiles is empty
    app.clamp_indices();
    assert_eq!(app.hermes_index, 0);
}

#[test]
fn hermes_screen_variants_exist() {
    // Validates M2-TUI-APP-001: Screen enum includes Hermes, HermesProfile, HermesProvider
    let _hermes = app::Screen::Hermes;
    let _hermes_profile = app::Screen::HermesProfile;
    let _hermes_provider = app::Screen::HermesProvider;
}

#[test]
fn hermes_confirm_action_variants_exist() {
    // Validates M2-TUI-APP-004: ConfirmAction includes Hermes variants
    let _apply = app::ConfirmAction::HermesApply {
        id: "test".to_string(),
    };
    let _delete = app::ConfirmAction::HermesDelete {
        id: "test".to_string(),
    };
}

#[test]
fn hermes_input_action_variants_exist() {
    // Validates M2-TUI-APP-005: InputAction includes Hermes-specific variants
    let _create = app::InputAction::HermesCreateProfile;
    let _dup = app::InputAction::HermesDuplicate {
        id: "x".to_string(),
    };
    let _name = app::InputAction::HermesSetProfileName {
        id: "x".to_string(),
    };
    let _desc = app::InputAction::HermesSetProfileDescription {
        id: "x".to_string(),
    };
    let _model = app::InputAction::HermesSetProfileDefaultModel {
        id: "x".to_string(),
    };
    let _prov = app::InputAction::HermesSetProfileProvider {
        id: "x".to_string(),
    };
    let _url = app::InputAction::HermesSetProfileBaseUrl {
        id: "x".to_string(),
    };
    let _key = app::InputAction::HermesSetProfileApiKey {
        id: "x".to_string(),
    };
    let _import_key = app::InputAction::HermesImportSetApiKey {
        id: "x".to_string(),
    };
    let _import_channel = app::SelectAction::HermesImportFromChannel {
        profile_id: "x".to_string(),
    };
}

#[test]
fn claude_screens_are_included_in_nav_items() {
    let nav = app::App::nav_items();
    let has_claude = nav
        .iter()
        .any(|(label, screen)| *label == "Claude" && *screen == app::Screen::Claude);
    assert!(has_claude, "nav_items() should include Claude entry");
}

#[test]
fn claude_app_state_initializes_correctly() {
    use std::path::PathBuf;
    let app = app::App::new(PathBuf::from("/tmp/test-home"));
    assert!(app.claude_profiles.is_empty());
    assert!(app.claude_active_id.is_none());
    assert_eq!(app.claude_index, 0);
    assert!(app.claude_detail_id.is_none());
    assert!(app.claude_detail.is_none());
    assert_eq!(app.claude_detail_field_index, 0);
}

#[test]
fn claude_clamp_indices_does_not_panic_on_empty_profiles() {
    use std::path::PathBuf;
    let mut app = app::App::new(PathBuf::from("/tmp/test-home"));
    app.clamp_indices();
    assert_eq!(app.claude_index, 0);
}

#[test]
fn claude_screen_variants_exist() {
    let _claude = app::Screen::Claude;
    let _claude_profile = app::Screen::ClaudeProfile;
}

#[test]
fn claude_confirm_action_variants_exist() {
    let _apply = app::ConfirmAction::ClaudeApply {
        id: "test".to_string(),
    };
    let _delete = app::ConfirmAction::ClaudeDelete {
        id: "test".to_string(),
    };
}

#[test]
fn claude_input_action_variants_exist() {
    let _create = app::InputAction::ClaudeCreateProfile;
    let _dup = app::InputAction::ClaudeDuplicate {
        id: "x".to_string(),
    };
    let _name = app::InputAction::ClaudeSetProfileName {
        id: "x".to_string(),
    };
    let _desc = app::InputAction::ClaudeSetProfileDescription {
        id: "x".to_string(),
    };
    let _base = app::InputAction::ClaudeSetProfileBaseUrl {
        id: "x".to_string(),
    };
    let _token = app::InputAction::ClaudeSetProfileBearerToken {
        id: "x".to_string(),
    };
    let _model = app::InputAction::ClaudeSetProfileModel {
        id: "x".to_string(),
    };
    let _small = app::InputAction::ClaudeSetProfileSmallModel {
        id: "x".to_string(),
    };
    let _reasoning = app::SelectAction::ClaudeSetProfileReasoningEffort {
        id: "x".to_string(),
    };
    let _thinking = app::SelectAction::ClaudeSetProfileThinkingMode {
        id: "x".to_string(),
    };
}

#[test]
fn claude_run_action_variant_exists() {
    let action = super::Action::RunClaudeRun {
        id: "profile-a".to_string(),
    };

    match action {
        super::Action::RunClaudeRun { id } => assert_eq!(id, "profile-a"),
        _ => panic!("expected RunClaudeRun action"),
    }
}

#[test]
fn claude_list_x_key_routes_through_run_action() {
    use std::path::PathBuf;

    let mut app = app::App::new(PathBuf::from("/tmp/test-home"));
    app.claude_profiles = vec![make_claude_profile("profile-a", "Alpha")];

    let action = super::keys_claude::handle_claude_key(&mut app, KeyCode::Char('x'));

    match action {
        Some(super::Action::RunClaudeRun { id }) => assert_eq!(id, "profile-a"),
        other => panic!("expected RunClaudeRun action, got {other:?}"),
    }
}

#[test]
fn claude_detail_x_key_routes_through_run_action() {
    use std::path::PathBuf;

    let mut app = app::App::new(PathBuf::from("/tmp/test-home"));
    app.claude_detail_id = Some("profile-a".to_string());
    app.claude_detail = Some(make_claude_profile("profile-a", "Alpha"));

    let action = super::keys_claude::handle_claude_profile_key(&mut app, KeyCode::Char('x'));

    match action {
        Some(super::Action::RunClaudeRun { id }) => assert_eq!(id, "profile-a"),
        other => panic!("expected RunClaudeRun action, got {other:?}"),
    }
}

#[test]
fn sanitize_terminal_for_direct_exec_is_callable() {
    super::utils::sanitize_terminal_for_direct_exec().unwrap();
}

#[test]
fn pi_screen_variants_exist() {
    let _pi = app::Screen::Pi;
    let _pi_profile = app::Screen::PiProfile;
    let _pi_provider = app::Screen::PiProvider;
    let _pi_model = app::Screen::PiModel;
}

#[test]
fn pi_is_in_nav_items() {
    let nav = app::App::nav_items();
    let has_pi = nav
        .iter()
        .any(|(label, screen)| *label == "Pi" && *screen == app::Screen::Pi);
    assert!(has_pi, "nav_items() should include Pi entry");
}

#[test]
fn pi_app_state_initializes_correctly() {
    use std::path::PathBuf;
    let app = app::App::new(PathBuf::from("/tmp/test-home"));
    assert!(app.pi_profiles.is_empty());
    assert!(app.pi_active_id.is_none());
    assert_eq!(app.pi_index, 0);
    assert!(app.pi_detail_id.is_none());
    assert!(app.pi_detail.is_none());
    assert_eq!(app.pi_detail_field_index, 0);
    assert_eq!(app.pi_provider_index, 0);
    assert_eq!(app.pi_provider_field_index, 0);
    assert_eq!(app.pi_model_index, 0);
    assert_eq!(app.pi_model_field_index, 0);
}

#[test]
fn pi_clamp_indices_does_not_panic_on_empty_profiles() {
    use std::path::PathBuf;
    let mut app = app::App::new(PathBuf::from("/tmp/test-home"));
    app.clamp_indices();
    assert_eq!(app.pi_index, 0);
}

#[test]
fn pi_confirm_action_variants_exist() {
    let _apply = app::ConfirmAction::PiApply {
        id: "test".to_string(),
    };
    let _delete = app::ConfirmAction::PiDelete {
        id: "test".to_string(),
    };
    let _del_prov = app::ConfirmAction::PiDeleteProvider {
        profile_id: "p".to_string(),
        provider_id: "prov".to_string(),
    };
    let _del_model = app::ConfirmAction::PiDeleteModel {
        profile_id: "p".to_string(),
        provider_id: "prov".to_string(),
        model_index: 0,
    };
}

#[test]
fn pi_input_action_variants_exist() {
    let _create = app::InputAction::PiCreateProfile;
    let _dup = app::InputAction::PiDuplicate {
        id: "x".to_string(),
    };
    let _name = app::InputAction::PiSetProfileName {
        id: "x".to_string(),
    };
    let _desc = app::InputAction::PiSetProfileDescription {
        id: "x".to_string(),
    };
    let _add_prov = app::InputAction::PiAddProvider {
        profile_id: "x".to_string(),
    };
    let _base_url = app::InputAction::PiSetProviderBaseUrl {
        profile_id: "x".to_string(),
        provider_id: "y".to_string(),
    };
    let _api_key = app::InputAction::PiSetProviderApiKey {
        profile_id: "x".to_string(),
        provider_id: "y".to_string(),
    };
    let _add_model = app::InputAction::PiAddModel {
        profile_id: "x".to_string(),
        provider_id: "y".to_string(),
    };
    let _set_id = app::InputAction::PiSetModelId {
        profile_id: "x".to_string(),
        provider_id: "y".to_string(),
        model_index: 0,
    };
    let _set_name = app::InputAction::PiSetModelName {
        profile_id: "x".to_string(),
        provider_id: "y".to_string(),
        model_index: 0,
    };
    let _set_ctx = app::InputAction::PiSetModelContextWindow {
        profile_id: "x".to_string(),
        provider_id: "y".to_string(),
        model_index: 0,
    };
    let _set_max = app::InputAction::PiSetModelMaxTokens {
        profile_id: "x".to_string(),
        provider_id: "y".to_string(),
        model_index: 0,
    };
    let _set_cost = app::InputAction::PiSetModelCost {
        profile_id: "x".to_string(),
        provider_id: "y".to_string(),
        model_index: 0,
    };
    let _sel_api = app::SelectAction::PiSetProviderApi {
        profile_id: "x".to_string(),
        provider_id: "y".to_string(),
    };
}

#[test]
fn load_droid_run_preferences_from_path_reads_nested_policy() {
    let temp = TempDir::new().unwrap();
    let prefs_path = temp.path().join("preferences.json");
    write_file(
        &prefs_path,
        r#"{
  "theme": "system",
  "droid_run": {
"disableAutoUpdateEnv": false,
"unsetAnthropicAuthToken": true
  }
}"#,
    );

    let prefs = load_droid_run_preferences_from_path(&prefs_path).unwrap();
    assert_eq!(
        prefs,
        droidgear_core::droid_runtime::DroidRunPreferences {
            disable_auto_update_env: Some(false),
            unset_anthropic_auth_token: Some(true),
        }
    );
}

#[test]
fn preview_droid_temporary_run_uses_selected_settings_path_without_dumping_contents() {
    let temp = TempDir::new().unwrap();
    let settings_path = temp.path().join(".droidgear/droid-settings/profile-a.json");
    write_file(
        &settings_path,
        r#"{"apiKey":"sk-droid-secret","model":"demo"}"#,
    );

    let preview = preview_droid_temporary_run(temp.path(), &settings_path).unwrap();

    assert!(preview.contains("Droid temporary run preview"));
    assert!(preview.contains(settings_path.to_string_lossy().as_ref()));
    assert!(preview.contains("FACTORY_DROID_AUTO_UPDATE_ENABLED=0"));
    assert!(preview.contains("ANTHROPIC_AUTH_TOKEN"));
    assert!(!preview.contains("sk-droid-secret"));
}

#[test]
fn list_droid_temporary_run_targets_lists_global_and_custom_names() {
    let temp = TempDir::new().unwrap();
    write_file(&temp.path().join(".factory/settings.json"), "{}");
    write_file(
        &temp.path().join(".droidgear/droid-settings/profile-a.json"),
        "{}",
    );
    droidgear_core::droid_settings_files::set_active_settings_file_for_home(
        temp.path(),
        Some("profile-a".to_string()),
    )
    .unwrap();

    let output = list_droid_temporary_run_targets(temp.path()).unwrap();

    assert!(output.contains("Available Droid run targets:"));
    assert!(output.contains(" global"));
    assert!(output.contains("* profile-a"));
    assert!(output.contains("run droid <settings-name>"));
}

#[test]
fn list_codex_temporary_run_targets_lists_index_name_and_id() {
    let temp = TempDir::new().unwrap();
    droidgear_core::codex::save_codex_profile_for_home(
        temp.path(),
        droidgear_core::codex::CodexProfile {
            id: "profile-a".to_string(),
            name: "Alpha".to_string(),
            description: None,
            created_at: "2026-01-01T00:00:00Z".to_string(),
            updated_at: "2026-01-01T00:00:00Z".to_string(),
            providers: HashMap::new(),
            model_provider: "openai".to_string(),
            model: "gpt-5".to_string(),
            model_reasoning_effort: None,
            api_key: None,
        },
    )
    .unwrap();
    droidgear_core::codex::save_codex_profile_for_home(
        temp.path(),
        droidgear_core::codex::CodexProfile {
            id: "profile-b".to_string(),
            name: "Beta".to_string(),
            description: None,
            created_at: "2026-01-01T00:00:00Z".to_string(),
            updated_at: "2026-01-01T00:00:00Z".to_string(),
            providers: HashMap::new(),
            model_provider: "openai".to_string(),
            model: "gpt-5".to_string(),
            model_reasoning_effort: None,
            api_key: None,
        },
    )
    .unwrap();
    droidgear_core::codex::apply_codex_profile_for_home(temp.path(), "profile-b").unwrap();

    let output = list_codex_temporary_run_targets(temp.path()).unwrap();

    assert!(output.contains("Available Codex run targets:"));
    assert!(output.contains("1. Alpha [id: profile-a]"));
    assert!(output.contains("* 2. Beta [id: profile-b]"));
    assert!(output.contains("run codex <index|name|id>"));
}

#[test]
fn preview_codex_temporary_run_lists_secret_keys_without_secret_values() {
    let temp = TempDir::new().unwrap();
    droidgear_core::codex::save_codex_profile_for_home(
        temp.path(),
        droidgear_core::codex::CodexProfile {
            id: "profile-a".to_string(),
            name: "Alpha".to_string(),
            description: None,
            created_at: "2026-01-01T00:00:00Z".to_string(),
            updated_at: "2026-01-01T00:00:00Z".to_string(),
            providers: HashMap::new(),
            model_provider: "openai".to_string(),
            model: "gpt-5".to_string(),
            model_reasoning_effort: None,
            api_key: Some("sk-secret".to_string()),
        },
    )
    .unwrap();

    let output = preview_codex_temporary_run(temp.path(), "profile-a").unwrap();

    assert!(output.contains("Codex temporary run preview"));
    assert!(output.contains("Runtime CODEX_HOME:"));
    assert!(output.contains("Secret environment keys:"));
    assert!(output.contains("OPENAI_API_KEY"));
    assert!(!output.contains("sk-secret"));
}

#[test]
fn list_claude_temporary_run_targets_lists_index_name_and_id() {
    let temp = TempDir::new().unwrap();
    droidgear_core::claude::save_claude_profile_for_home(
        temp.path(),
        droidgear_core::claude::ClaudeCodeProfile {
            id: "profile-a".to_string(),
            name: "Alpha".to_string(),
            description: None,
            base_url: Some("https://proxy.example.com".to_string()),
            bearer_token: Some("token-a".to_string()),
            model: Some("claude-sonnet-4-5".to_string()),
            small_model_uses_main_model: true,
            small_model: None,
            reasoning_effort: None,
            thinking_mode: droidgear_core::claude::ClaudeThinkingMode::Inherit,
            created_at: "2026-01-01T00:00:00Z".to_string(),
            updated_at: "2026-01-01T00:00:00Z".to_string(),
        },
    )
    .unwrap();
    droidgear_core::claude::save_claude_profile_for_home(
        temp.path(),
        droidgear_core::claude::ClaudeCodeProfile {
            id: "profile-b".to_string(),
            name: "Beta".to_string(),
            description: None,
            base_url: Some("https://proxy.example.com".to_string()),
            bearer_token: Some("token-b".to_string()),
            model: Some("claude-sonnet-4-5".to_string()),
            small_model_uses_main_model: true,
            small_model: None,
            reasoning_effort: None,
            thinking_mode: droidgear_core::claude::ClaudeThinkingMode::Inherit,
            created_at: "2026-01-01T00:00:00Z".to_string(),
            updated_at: "2026-01-01T00:00:00Z".to_string(),
        },
    )
    .unwrap();
    droidgear_core::claude::set_active_claude_profile_id_for_home(temp.path(), "profile-b")
        .unwrap();

    let output = list_claude_temporary_run_targets(temp.path()).unwrap();

    assert!(output.contains("Available Claude run targets:"));
    assert!(output.contains("1. Alpha [id: profile-a]"));
    assert!(output.contains("* 2. Beta [id: profile-b]"));
    assert!(output.contains("run claude <index|name|id>"));
    assert!(output.contains("run claude --preview <index|name|id>"));
}

#[test]
fn format_claude_temporary_run_preview_includes_overlay_and_sensitive_notice() {
    let preview = droidgear_core::claude_runtime::ClaudeTemporaryRunDebugPreview {
        profile_id: "profile-a".to_string(),
        profile_name: "Alpha".to_string(),
        program: "/tmp/droidgear-launcher".to_string(),
        args: vec![
            "__droidgear_internal".to_string(),
            "claude-launcher".to_string(),
        ],
        child_program: "claude".to_string(),
        child_args: Vec::new(),
        live_config_dir: "/tmp/demo-home/.claude".to_string(),
        inherited_env_file_source: Some("/tmp/demo-home/inherited.env".to_string()),
        env: vec![
            (
                "CLAUDE_CONFIG_DIR".to_string(),
                "/tmp/demo-home/.claude".to_string(),
            ),
            (
                "CLAUDE_ENV_FILE".to_string(),
                "<runtime copy written at launch>".to_string(),
            ),
        ],
        unset_env: vec![
            "ANTHROPIC_AUTH_TOKEN".to_string(),
            "ANTHROPIC_MODEL".to_string(),
        ],
        secret_env_keys: vec!["DROIDGEAR_INTERNAL_CLAUDE_RUNTIME_JSON".to_string()],
        warnings: vec!["example warning".to_string()],
        settings_overlay_json: "{\n  \"env\": {\n    \"ANTHROPIC_AUTH_TOKEN\": \"token-a\"\n  }\n}"
            .to_string(),
    };

    let output = format_claude_temporary_run_preview(&preview);

    assert!(output.contains("Claude temporary run preview"));
    assert!(output.contains("Sensitive preview:"));
    assert!(output.contains("Alpha [id: profile-a]"));
    assert!(output.contains("/tmp/demo-home/inherited.env"));
    assert!(output.contains("claude-launcher"));
    assert!(output.contains("DROIDGEAR_INTERNAL_CLAUDE_RUNTIME_JSON"));
    assert!(output.contains("ANTHROPIC_AUTH_TOKEN"));
    assert!(output.contains("token-a"));
    assert!(output.contains("example warning"));
}
