use super::*;
use crate::tui::utils::{
    format_claude_temporary_run_preview, load_droid_run_preferences_from_path,
    preview_codex_temporary_run, preview_droid_temporary_run,
};
use crossterm::event::KeyCode;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use tempfile::TempDir;

fn write_file(path: &Path, contents: &str) {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).unwrap();
    }
    std::fs::write(path, contents).unwrap();
}

#[test]
fn claude_screen_is_in_claude_nav_group() {
    let group = app::App::group_of_screen(app::Screen::ClaudeSettings)
        .expect("ClaudeSettings should be a nav item");
    assert_eq!(app::App::nav_groups()[group].label, "Claude");
    assert!(app::App::nav_targets()
        .iter()
        .any(|(label, screen)| label == "claude" && *screen == app::Screen::ClaudeSettings));
}

#[test]
fn claude_app_state_initializes_correctly() {
    use std::path::PathBuf;
    let app = app::App::new(PathBuf::from("/tmp/test-home"));
    assert!(app.claude_files.is_empty());
    assert_eq!(app.claude_index, 0);
    assert!(app.claude_detail_name.is_none());
    assert!(app.claude_detail_json.is_none());
    assert_eq!(app.claude_detail_field_index, 0);
}

#[test]
fn claude_clamp_indices_does_not_panic_on_empty_files() {
    use std::path::PathBuf;
    let mut app = app::App::new(PathBuf::from("/tmp/test-home"));
    app.clamp_indices();
    assert_eq!(app.claude_index, 0);
}

#[test]
fn claude_screen_variants_exist() {
    let _claude = app::Screen::ClaudeSettings;
    let _claude_detail = app::Screen::ClaudeSettingsDetail;
}

#[test]
fn claude_confirm_action_variants_exist() {
    let _apply = app::ConfirmAction::ClaudeSettingsApply {
        name: "test".to_string(),
    };
    let _delete = app::ConfirmAction::ClaudeSettingsDelete {
        name: "test".to_string(),
    };
}

#[test]
fn claude_input_action_variants_exist() {
    let _create = app::InputAction::ClaudeSettingsCreateFile;
    let _dup = app::InputAction::ClaudeSettingsDuplicate {
        name: "x".to_string(),
    };
    let _edit = app::InputAction::ClaudeSettingsEditField { field_index: 0 };
    let _reasoning = app::SelectAction::ClaudeSettingsSetReasoningEffort;
    let _thinking = app::SelectAction::ClaudeSettingsSetThinkingMode;
    let _perm = app::SelectAction::ClaudeSettingsSetPermissionsDefaultMode;
    let _bypass = app::SelectAction::ClaudeSettingsSetDisableBypass;
}

#[test]
fn claude_run_action_variant_exists() {
    let action = super::Action::RunClaudeRun {
        name: "my-settings".to_string(),
        skip_dangerous: false,
    };

    match action {
        super::Action::RunClaudeRun {
            name,
            skip_dangerous,
        } => {
            assert_eq!(name, "my-settings");
            assert!(!skip_dangerous);
        }
        _ => panic!("expected RunClaudeRun action"),
    }
}

fn claude_file(name: &str) -> droidgear_core::claude_settings_files::ClaudeSettingsFileInfo {
    use droidgear_core::claude_settings_files::ClaudeSettingsFileInfo;
    ClaudeSettingsFileInfo {
        name: name.to_string(),
        path: format!("/tmp/{name}.json"),
        is_global: false,
        is_active: true,
        exists: true,
    }
}

#[test]
fn claude_list_t_key_routes_through_run_action() {
    use std::path::PathBuf;

    let mut app = app::App::new(PathBuf::from("/tmp/test-home"));
    app.claude_files = vec![claude_file("my-settings")];

    let action = super::keys_claude::handle_claude_key(&mut app, KeyCode::Char('t'));

    match action {
        Some(super::Action::RunClaudeRun {
            name,
            skip_dangerous,
        }) => {
            assert_eq!(name, "my-settings");
            assert!(!skip_dangerous, "t should run without skip permissions");
        }
        other => panic!("expected RunClaudeRun action, got {other:?}"),
    }
}

#[test]
fn claude_list_uppercase_t_key_routes_through_skip_run() {
    use std::path::PathBuf;

    let mut app = app::App::new(PathBuf::from("/tmp/test-home"));
    app.claude_files = vec![claude_file("my-settings")];

    let action = super::keys_claude::handle_claude_key(&mut app, KeyCode::Char('T'));

    match action {
        Some(super::Action::RunClaudeRun {
            name,
            skip_dangerous,
        }) => {
            assert_eq!(name, "my-settings");
            assert!(skip_dangerous, "T should run with skip permissions");
        }
        other => panic!("expected RunClaudeRun action, got {other:?}"),
    }
}

#[test]
fn claude_list_t_key_is_blocked_when_file_disables_skip_permissions() {
    let home = TempDir::new().unwrap();
    let mut app = app::App::new(home.path().to_path_buf());
    app.claude_files = vec![claude_file("no-skip")];
    // The guard reads the file from disk; write one that disables bypass.
    write_file(
        &home.path().join(".droidgear/claude-settings/no-skip.json"),
        r#"{"disableBypassPermissionsMode":"disable"}"#,
    );

    let action = super::keys_claude::handle_claude_key(&mut app, KeyCode::Char('T'));
    assert!(action.is_none(), "T should be blocked by the disable flag");
    assert!(
        app.toast_message().contains("disabled"),
        "expected a toast explaining the block, got {:?}",
        app.toast_message()
    );
}

#[test]
fn claude_list_s_key_sets_active_custom_file() {
    use std::path::PathBuf;

    let mut app = app::App::new(PathBuf::from("/tmp/test-home"));
    app.claude_files = vec![claude_file("work"), claude_file("personal")];

    let action = super::keys_claude::handle_claude_key(&mut app, KeyCode::Char('s'));

    match action {
        Some(super::Action::SetActiveClaudeSettingsFile { name }) => {
            assert_eq!(name.as_deref(), Some("work"));
        }
        other => panic!("expected SetActiveClaudeSettingsFile action, got {other:?}"),
    }
}

#[test]
fn claude_detail_escape_with_dirty_edits_asks_for_confirmation() {
    use std::path::PathBuf;

    let mut app = app::App::new(PathBuf::from("/tmp/test-home"));
    app.claude_detail_name = Some("work".to_string());
    app.claude_detail_json = Some(serde_json::json!({"env": {}}));
    app.claude_detail_dirty = true;
    app.screen = app::Screen::ClaudeSettingsDetail;

    super::keys_claude::handle_claude_settings_detail_key(&mut app, KeyCode::Esc);

    match app.modal {
        Some(app::Modal::Confirm {
            action: app::ConfirmAction::ClaudeSettingsDiscardDetail,
            ..
        }) => {}
        other => panic!("expected discard-confirm modal, got {other:?}"),
    }
    // Still on the detail screen until the user confirms.
    assert_eq!(app.screen, app::Screen::ClaudeSettingsDetail);
}

#[test]
fn claude_detail_escape_without_dirty_edits_exits() {
    use std::path::PathBuf;

    let mut app = app::App::new(PathBuf::from("/tmp/test-home"));
    app.claude_detail_name = Some("work".to_string());
    app.claude_detail_json = Some(serde_json::json!({"env": {}}));
    app.claude_detail_dirty = false;
    app.screen = app::Screen::ClaudeSettingsDetail;

    super::keys_claude::handle_claude_settings_detail_key(&mut app, KeyCode::Esc);

    assert_eq!(app.screen, app::Screen::ClaudeSettings);
    assert!(app.claude_detail_name.is_none());
}

#[test]
fn claude_detail_i_key_opens_channel_import_select() {
    let home = TempDir::new().unwrap();
    // The `i` key refreshes channels from disk, so write a real channels.json.
    write_file(
        &home.path().join(".droidgear/channels.json"),
        r#"[{"id":"c1","name":"My Proxy","type":"general","baseUrl":"https://proxy.example.com","enabled":true,"createdAt":0}]"#,
    );
    let mut app = app::App::new(home.path().to_path_buf());
    app.claude_detail_name = Some("work".to_string());
    app.claude_detail_json = Some(serde_json::json!({"env": {}}));

    super::keys_claude::handle_claude_settings_detail_key(&mut app, KeyCode::Char('i'));

    match app.modal {
        Some(app::Modal::Select {
            action: app::SelectAction::ClaudeSettingsImportChannel,
            options,
            ..
        }) => {
            assert!(options.iter().any(|o| o.contains("My Proxy")));
        }
        other => panic!("expected channel-import select modal, got {other:?}"),
    }
}

#[test]
fn claude_detail_i_key_with_no_channels_shows_toast() {
    use std::path::PathBuf;

    let mut app = app::App::new(PathBuf::from("/tmp/test-home"));
    app.claude_detail_name = Some("work".to_string());
    app.claude_detail_json = Some(serde_json::json!({"env": {}}));

    super::keys_claude::handle_claude_settings_detail_key(&mut app, KeyCode::Char('i'));

    assert!(app.modal.is_none(), "no channels should not open a modal");
    assert!(
        app.toast_message().contains("No enabled channels"),
        "expected a toast, got {:?}",
        app.toast_message()
    );
}

#[test]
fn claude_detail_t_key_auto_saves_dirty_edits() {
    let home = TempDir::new().unwrap();
    write_file(&home.path().join(".claude/settings.json"), "{}");
    write_file(
        &home.path().join(".droidgear/claude-settings/work.json"),
        r#"{"env":{"ANTHROPIC_MODEL":"claude-sonnet-4-5"}}"#,
    );
    let mut app = app::App::new(home.path().to_path_buf());
    app.claude_detail_name = Some("work".to_string());
    app.claude_detail_json = Some(serde_json::json!({
        "env": {"ANTHROPIC_MODEL": "claude-sonnet-4-5", "ANTHROPIC_BASE_URL": "https://x"}
    }));
    app.claude_detail_dirty = true;

    let action =
        super::keys_claude::handle_claude_settings_detail_key(&mut app, KeyCode::Char('t'));

    match action {
        Some(super::Action::RunClaudeRun { name, .. }) => assert_eq!(name, "work"),
        other => panic!("expected RunClaudeRun action, got {other:?}"),
    }
    assert!(!app.claude_detail_dirty, "auto-save should clear dirty");
    let saved =
        droidgear_core::claude_settings_files::read_settings_file_for_home(home.path(), "work")
            .unwrap();
    assert_eq!(
        saved["env"]["ANTHROPIC_BASE_URL"],
        serde_json::Value::String("https://x".to_string())
    );
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
fn hermes_screen_is_in_hermes_nav_group() {
    let group =
        app::App::group_of_screen(app::Screen::Hermes).expect("Hermes should be a nav item");
    assert_eq!(app::App::nav_groups()[group].label, "Hermes");
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
fn pi_is_in_pi_nav_group() {
    let group = app::App::group_of_screen(app::Screen::Pi).expect("Pi should be a nav item");
    assert_eq!(app::App::nav_groups()[group].label, "Pi");
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
fn pi_provider_t_key_routes_through_test_action() {
    let mut app = app::App::new(PathBuf::from("/tmp/test-home"));
    let provider = droidgear_core::pi::PiProviderConfig {
        api_key: Some("sk-test".to_string()),
        models: vec![droidgear_core::pi::PiModel {
            id: "test-model".to_string(),
            ..Default::default()
        }],
        ..Default::default()
    };
    app.pi_detail_id = Some("profile".to_string());
    app.pi_detail = Some(droidgear_core::pi::PiProfile {
        id: "profile".to_string(),
        name: "Profile".to_string(),
        description: None,
        created_at: "2026-01-01T00:00:00Z".to_string(),
        updated_at: "2026-01-01T00:00:00Z".to_string(),
        providers: HashMap::from([("test-provider".to_string(), provider)]),
    });

    let action = super::keys_pi::handle_pi_provider_key(&mut app, KeyCode::Char('t'));

    match action {
        Some(super::Action::TestPiProvider {
            provider_id,
            config,
        }) => {
            assert_eq!(provider_id, "test-provider");
            assert_eq!(config.models[0].id, "test-model");
        }
        other => panic!("expected TestPiProvider action, got {other:?}"),
    }
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
fn omp_screen_variants_exist() {
    let _omp = app::Screen::Omp;
    let _omp_profile = app::Screen::OmpProfile;
}

#[test]
fn omp_is_in_omp_nav_group() {
    let group = app::App::group_of_screen(app::Screen::Omp).expect("Omp should be a nav item");
    assert_eq!(app::App::nav_groups()[group].label, "OMP");
}

#[test]
fn omp_app_state_initializes_correctly() {
    use std::path::PathBuf;
    let app = app::App::new(PathBuf::from("/tmp/test-home"));
    assert!(app.omp_profiles.is_empty());
    assert!(app.omp_active_id.is_none());
    assert_eq!(app.omp_index, 0);
    assert!(app.omp_detail_id.is_none());
    assert!(app.omp_detail.is_none());
    assert_eq!(app.omp_detail_field_index, 0);
}

#[test]
fn omp_clamp_indices_does_not_panic_on_empty_profiles() {
    use std::path::PathBuf;
    let mut app = app::App::new(PathBuf::from("/tmp/test-home"));
    app.clamp_indices();
    assert_eq!(app.omp_index, 0);
}

#[test]
fn omp_confirm_action_variants_exist() {
    let _apply = app::ConfirmAction::OmpApply {
        id: "test".to_string(),
    };
    let _delete = app::ConfirmAction::OmpDelete {
        id: "test".to_string(),
    };
    let _test_all = app::ConfirmAction::OmpTestAll;
}

#[test]
fn omp_input_action_variants_exist() {
    let _create = app::InputAction::OmpCreateProfile;
    let _dup = app::InputAction::OmpDuplicate {
        id: "x".to_string(),
    };
    let _update_name = app::InputAction::OmpUpdateProfileName;
    let _update_desc = app::InputAction::OmpUpdateProfileDescription;
    let _update_role = app::InputAction::OmpUpdateModelRole {
        role: "default".to_string(),
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
            auth_profile_name: None,
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
            auth_profile_name: None,
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
            model_provider: "custom".to_string(),
            model: "gpt-5".to_string(),
            model_reasoning_effort: None,
            api_key: Some("sk-secret".to_string()),
            auth_profile_name: None,
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
    let settings_dir = temp.path().join(".droidgear").join("claude-settings");
    std::fs::create_dir_all(&settings_dir).unwrap();

    // Create two settings files
    let alpha_path = settings_dir.join("Alpha.json");
    std::fs::write(&alpha_path, "{}").unwrap();
    let beta_path = settings_dir.join("Beta.json");
    std::fs::write(&beta_path, "{}").unwrap();

    // Set Beta as active
    droidgear_core::claude_settings_files::set_active_settings_file_for_home(
        temp.path(),
        Some("Beta".to_string()),
    )
    .unwrap();

    let output = list_claude_temporary_run_targets(temp.path()).unwrap();

    assert!(output.contains("Available Claude settings files:"));
    assert!(output.contains("Alpha"));
    assert!(output.contains("Beta"));
    assert!(output.contains("(global)"));
    assert!(output.contains("run claude -n <name>"));
    assert!(output.contains("run claude --preview -n <name>"));
}

#[test]
fn pi_import_from_channel_action_variants_exist() {
    let _import = app::SelectAction::PiImportFromChannel {
        profile_id: "x".to_string(),
        provider_id: "y".to_string(),
    };
    let _set_key = app::InputAction::PiImportSetApiKey {
        profile_id: "x".to_string(),
        provider_id: "y".to_string(),
    };
}

#[test]
fn pi_add_provider_from_channel_action_variants_exist() {
    let _select = app::SelectAction::PiAddProviderFromChannel {
        profile_id: "x".to_string(),
        provider_id: "y".to_string(),
    };
    let _input = app::InputAction::PiAddProviderFromChannel {
        profile_id: "x".to_string(),
    };
}

#[test]
fn pi_import_set_token_action_exists() {
    let _set_token = app::SelectAction::PiImportSetToken {
        profile_id: "x".to_string(),
        provider_id: "y".to_string(),
    };
    let _toggle = app::SelectAction::PiImportToggleModel {
        profile_id: "x".to_string(),
        provider_id: "y".to_string(),
    };
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

// --- Navigation grouping ---

#[test]
fn nav_groups_cover_all_screens_exactly_once() {
    let screens = [
        app::Screen::Paths,
        app::Screen::DroidSettingsFiles,
        app::Screen::TrustedFolders,
        app::Screen::Factory,
        app::Screen::Mcp,
        app::Screen::ClaudeSettings,
        app::Screen::Codex,
        app::Screen::OpenCode,
        app::Screen::OpenClaw,
        app::Screen::Pi,
        app::Screen::Hermes,
        app::Screen::Sessions,
        app::Screen::Specs,
        app::Screen::Channels,
        app::Screen::Missions,
        app::Screen::FactoryAuth,
        app::Screen::CodexAuth,
        app::Screen::OpenClawSubagents,
        app::Screen::OpenClawHelpers,
    ];
    for screen in screens {
        let group = app::App::group_of_screen(screen)
            .unwrap_or_else(|| panic!("{screen:?} should be a nav item"));
        let occurrences = app::App::nav_groups()[group]
            .items
            .iter()
            .filter(|(_, s)| *s == screen)
            .count();
        assert_eq!(occurrences, 1, "{screen:?} should appear exactly once");
    }
}

#[test]
fn trusted_folders_state_initializes_and_is_navigable() {
    let mut app = app::App::new(PathBuf::from("/tmp/test-home"));
    assert!(app.trusted_folders.is_empty());
    assert_eq!(app.trusted_folders_index, 0);

    let group = app::App::group_of_screen(app::Screen::TrustedFolders)
        .expect("TrustedFolders should be a nav item");
    assert_eq!(app::App::nav_groups()[group].label, "Droid");

    app.screen = app::Screen::TrustedFolders;
    super::keys_trusted_folders::handle_trusted_folders_key(&mut app, KeyCode::Char('a'));
    assert!(matches!(
        app.modal.as_ref(),
        Some(app::Modal::Input {
            action: app::InputAction::TrustedFolderAdd,
            ..
        })
    ));
}

#[test]
fn trusted_folders_support_marking_and_select_all_for_batch_delete() {
    let mut app = app::App::new(PathBuf::from("/tmp/test-home"));
    app.screen = app::Screen::TrustedFolders;
    app.trusted_folders = vec![
        droidgear_core::trusted_folders::TrustedFolder {
            path: "/tmp/a".to_string(),
            trusted_at: "a".to_string(),
        },
        droidgear_core::trusted_folders::TrustedFolder {
            path: "/tmp/b".to_string(),
            trusted_at: "b".to_string(),
        },
    ];

    super::keys_trusted_folders::handle_trusted_folders_key(&mut app, KeyCode::Char('A'));
    assert_eq!(app.trusted_folders_selected.len(), 2);
    super::keys_trusted_folders::handle_trusted_folders_key(&mut app, KeyCode::Char('d'));

    assert!(matches!(
        app.modal.as_ref(),
        Some(app::Modal::Confirm {
            action: app::ConfirmAction::TrustedFoldersDelete { paths },
            ..
        }) if paths == &vec!["/tmp/a".to_string(), "/tmp/b".to_string()]
    ));

    app.modal = None;
    super::keys_trusted_folders::handle_trusted_folders_key(&mut app, KeyCode::Char('A'));
    assert!(app.trusted_folders_selected.is_empty());
}

#[test]
fn go_back_from_multi_item_group_feature_returns_to_feature_list() {
    let mut app = app::App::new(PathBuf::from("/tmp/test-home"));
    app.screen = app::Screen::Factory;
    app.go_back();
    assert_eq!(app.screen, app::Screen::FeatureList);
    assert_eq!(
        app.nav_index,
        app::App::group_of_screen(app::Screen::Factory).unwrap()
    );
}

#[test]
fn go_back_from_feature_list_returns_to_main() {
    let mut app = app::App::new(PathBuf::from("/tmp/test-home"));
    app.screen = app::Screen::FeatureList;
    app.go_back();
    assert_eq!(app.screen, app::Screen::Main);
}

#[test]
fn go_back_from_single_item_group_returns_to_main() {
    let mut app = app::App::new(PathBuf::from("/tmp/test-home"));
    app.screen = app::Screen::Channels;
    app.go_back();
    assert_eq!(app.screen, app::Screen::Main);
}

#[test]
fn go_back_from_sub_screen_returns_to_parent() {
    let mut app = app::App::new(PathBuf::from("/tmp/test-home"));
    app.screen = app::Screen::McpServer;
    app.go_back();
    assert_eq!(app.screen, app::Screen::Mcp);
    app.go_back();
    assert_eq!(app.screen, app::Screen::FeatureList);
    app.go_back();
    assert_eq!(app.screen, app::Screen::Main);
}

#[test]
fn go_back_from_openclaw_helpers_returns_to_openclaw_feature_list() {
    let mut app = app::App::new(PathBuf::from("/tmp/test-home"));
    app.screen = app::Screen::OpenClawHelpers;
    app.go_back();
    assert_eq!(app.screen, app::Screen::FeatureList);
    assert_eq!(
        app.nav_index,
        app::App::group_of_screen(app::Screen::OpenClawHelpers).unwrap()
    );
}

#[test]
fn main_enter_on_multi_item_group_opens_feature_list() {
    let home = TempDir::new().unwrap();
    let mut app = app::App::new(home.path().to_path_buf());
    app.nav_index = app::App::group_of_screen(app::Screen::Factory).unwrap();
    super::keys_main::handle_main_key(&mut app, KeyCode::Enter);
    assert_eq!(app.screen, app::Screen::FeatureList);
    assert_eq!(app.feature_index, 0);
}

#[test]
fn main_enter_on_single_item_group_opens_screen_directly() {
    let home = TempDir::new().unwrap();
    let mut app = app::App::new(home.path().to_path_buf());
    app.nav_index = app::App::group_of_screen(app::Screen::Channels).unwrap();
    super::keys_main::handle_main_key(&mut app, KeyCode::Enter);
    assert_eq!(app.screen, app::Screen::Channels);
}

#[test]
fn feature_list_enter_opens_selected_feature() {
    let home = TempDir::new().unwrap();
    let mut app = app::App::new(home.path().to_path_buf());
    app.screen = app::Screen::FeatureList;
    app.nav_index = app::App::group_of_screen(app::Screen::Factory).unwrap();
    app.feature_index = 3; // Auth Profiles
    super::keys_main::handle_feature_list_key(&mut app, KeyCode::Enter);
    assert_eq!(app.screen, app::Screen::FactoryAuth);
}

#[test]
fn feature_list_escape_returns_to_main() {
    let mut app = app::App::new(PathBuf::from("/tmp/test-home"));
    app.screen = app::Screen::FeatureList;
    super::keys_main::handle_feature_list_key(&mut app, KeyCode::Esc);
    assert_eq!(app.screen, app::Screen::Main);
}

#[test]
fn nav_targets_use_group_prefixed_labels() {
    let targets = app::App::nav_targets();
    assert!(targets
        .iter()
        .any(|(label, screen)| label == "droid: models" && *screen == app::Screen::Factory));
    assert!(targets.iter().any(|(label, screen)| {
        label == "codex: auth profiles" && *screen == app::Screen::CodexAuth
    }));
    assert!(targets
        .iter()
        .any(|(label, screen)| label == "channels" && *screen == app::Screen::Channels));
    // Labels must be unique so the picker can resolve by label.
    let mut labels: Vec<&String> = targets.iter().map(|(label, _)| label).collect();
    labels.sort();
    labels.dedup();
    assert_eq!(labels.len(), targets.len());
}

#[test]
fn nav_picker_filter_narrows_options_and_enter_resolves_by_label() {
    let home = TempDir::new().unwrap();
    let mut app = app::App::new(home.path().to_path_buf());
    super::keys_main::handle_main_key(&mut app, KeyCode::Char('s'));
    let Some(app::Modal::Select {
        options,
        index,
        action: app::SelectAction::GoToNav,
        ..
    }) = app.modal.clone()
    else {
        panic!("expected GoToNav select modal");
    };
    assert_eq!(options.len(), app::App::nav_targets().len());
    assert_eq!(index, 0);

    // Type "dro" to filter down to the Droid group's features.
    for c in ['d', 'r', 'o'] {
        super::keys_main::handle_key(&mut app, KeyCode::Char(c));
    }
    let Some(app::Modal::Select { options, .. }) = app.modal.clone() else {
        panic!("select modal should still be open");
    };
    assert_eq!(options.len(), 8);
    assert!(options.iter().all(|o| o.starts_with("droid:")));

    // Enter picks the first filtered option ("droid: models" -> Factory).
    super::keys_main::handle_key(&mut app, KeyCode::Enter);
    assert_eq!(app.screen, app::Screen::Factory);
    assert!(app.modal.is_none());
    assert!(app.modal_filter.is_empty());
}
