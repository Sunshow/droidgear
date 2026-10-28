use super::*;
use crate::tui::utils::{
    format_claude_temporary_run_preview, load_droid_run_preferences_from_path,
    preview_codex_temporary_run, preview_droid_run,
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
fn claude_detail_field_cursor_can_reach_auto_compact_window() {
    use std::path::PathBuf;

    let mut app = app::App::new(PathBuf::from("/tmp/test-home"));
    app.claude_detail_field_index = app::CLAUDE_DETAIL_FIELD_COUNT - 1;
    app.clamp_indices();
    assert_eq!(
        app.claude_detail_field_index,
        app::CLAUDE_DETAIL_FIELD_COUNT - 1,
        "the last Claude detail row must stay selectable"
    );
}

#[test]
fn claude_detail_enter_on_auto_compact_window_prefills_the_stored_value() {
    use std::path::PathBuf;

    let mut app = app::App::new(PathBuf::from("/tmp/test-home"));
    app.claude_detail_name = Some("work".to_string());
    app.claude_detail_json = Some(serde_json::json!({"autoCompactWindow": 500000}));
    app.claude_detail_field_index = 14;

    super::keys_claude::handle_claude_settings_detail_key(&mut app, KeyCode::Enter);

    match app.modal {
        Some(app::Modal::Input { value, action, .. }) => {
            assert_eq!(value, "500000");
            assert!(matches!(
                action,
                app::InputAction::ClaudeSettingsEditField { field_index: 14 }
            ));
        }
        other => panic!("expected auto-compact input modal, got {other:?}"),
    }
}

#[test]
fn claude_auto_compact_window_edit_sets_clears_and_validates() {
    use std::path::PathBuf;

    let mut app = app::App::new(PathBuf::from("/tmp/test-home"));
    app.claude_detail_json = Some(serde_json::json!({"env": {}}));
    let action = || app::InputAction::ClaudeSettingsEditField { field_index: 14 };

    super::modal::run_input_action(&mut app, action(), "500000".to_string()).unwrap();
    assert_eq!(
        app.claude_detail_json.as_ref().unwrap()["autoCompactWindow"],
        serde_json::json!(500000)
    );
    assert!(app.claude_detail_dirty);

    // Out-of-range and non-numeric values are rejected without touching JSON.
    assert!(super::modal::run_input_action(&mut app, action(), "200".to_string()).is_err());
    assert!(super::modal::run_input_action(&mut app, action(), "500k".to_string()).is_err());
    assert_eq!(
        app.claude_detail_json.as_ref().unwrap()["autoCompactWindow"],
        serde_json::json!(500000)
    );

    // An empty value removes the key so Claude Code uses the model default.
    super::modal::run_input_action(&mut app, action(), String::new()).unwrap();
    assert!(app
        .claude_detail_json
        .as_ref()
        .unwrap()
        .get("autoCompactWindow")
        .is_none());
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
    let _delete_model = app::ConfirmAction::HermesDeleteModel {
        id: "test".to_string(),
        model_index: 0,
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
    let _model_name = app::InputAction::HermesSetModelName {
        id: "x".to_string(),
        model_index: 0,
    };
    let _model = app::InputAction::HermesSetModelDefault {
        id: "x".to_string(),
        model_index: 0,
    };
    let _prov = app::InputAction::HermesSetModelProvider {
        id: "x".to_string(),
        model_index: 0,
    };
    let _url = app::InputAction::HermesSetModelBaseUrl {
        id: "x".to_string(),
        model_index: 0,
    };
    let _key = app::InputAction::HermesSetModelApiKey {
        id: "x".to_string(),
        model_index: 0,
    };
    let _import_key = app::InputAction::HermesImportSetApiKey {
        id: "x".to_string(),
        model_index: 0,
    };
    let _import_channel = app::SelectAction::HermesImportFromChannel {
        profile_id: "x".to_string(),
        model_index: 0,
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
    let _pi_sessions = app::Screen::PiSessions;
}

#[test]
fn pi_is_in_pi_nav_group() {
    let group = app::App::group_of_screen(app::Screen::Pi).expect("Pi should be a nav item");
    assert_eq!(app::App::nav_groups()[group].label, "Pi");
    assert!(app::App::nav_groups()[group]
        .items
        .iter()
        .any(|(label, screen)| *label == "Sessions" && *screen == app::Screen::PiSessions));
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
    assert!(app.pi_sessions.is_empty());
    assert_eq!(app.pi_sessions_index, 0);
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
fn pi_sessions_key_routes_view_and_delete_actions() {
    let mut app = app::App::new(PathBuf::from("/tmp/test-home"));
    app.pi_sessions = vec![droidgear_core::pi_sessions::PiSessionSummary {
        id: "session".to_string(),
        title: "Session".to_string(),
        project: "/tmp/project".to_string(),
        model: "model".to_string(),
        model_provider: "provider".to_string(),
        modified_at: 0.0,
        message_count: 1,
        token_usage: droidgear_core::pi_sessions::PiTokenUsage::default(),
        path: "/tmp/session.jsonl".to_string(),
    }];

    let view = super::keys_pi_sessions::handle_pi_sessions_key(&mut app, KeyCode::Enter);
    assert!(matches!(view, Some(super::Action::ViewPiSession { .. })));

    super::keys_pi_sessions::handle_pi_sessions_key(&mut app, KeyCode::Char('d'));
    assert!(matches!(
        app.modal,
        Some(app::Modal::Confirm {
            action: app::ConfirmAction::PiSessionDelete { .. },
            ..
        })
    ));
}

#[test]
fn pi_confirm_action_variants_exist() {
    let _apply = app::ConfirmAction::PiApply {
        id: "test".to_string(),
    };
    let _session_delete = app::ConfirmAction::PiSessionDelete {
        path: "/tmp/session.jsonl".to_string(),
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
fn dsh_screen_variants_exist() {
    let _dsh_desktop = app::Screen::DshDesktop;
    let _dsh_web = app::Screen::DshWeb;
    let _dsh_provider = app::Screen::DshProvider;
    let _dsh_model = app::Screen::DshModel;
    let _dsh_sessions = app::Screen::DshSessions;
}

#[test]
fn dsh_is_in_dsh_nav_group() {
    for screen in [
        app::Screen::DshDesktop,
        app::Screen::DshWeb,
        app::Screen::DshSessions,
    ] {
        let group = app::App::group_of_screen(screen).expect("Dsh screens should be nav items");
        assert_eq!(app::App::nav_groups()[group].label, "Dsh");
    }
    let group = app::App::group_of_screen(app::Screen::DshSessions)
        .expect("DshSessions should be a nav item");
    assert!(app::App::nav_groups()[group]
        .items
        .iter()
        .any(|(label, screen)| *label == "Sessions" && *screen == app::Screen::DshSessions));
}

#[test]
fn dsh_app_state_initializes_correctly() {
    use std::path::PathBuf;
    let app = app::App::new(PathBuf::from("/tmp/test-home"));
    assert!(app.dsh_providers.is_empty());
    assert_eq!(app.dsh_index, 0);
    assert!(app.dsh_provider_id.is_none());
    assert_eq!(app.dsh_provider_field_index, 0);
    assert_eq!(app.dsh_model_index, 0);
    assert_eq!(app.dsh_model_field_index, 0);
    assert!(app.dsh_sessions.is_empty());
    assert_eq!(app.dsh_sessions_index, 0);
}

#[test]
fn dsh_provider_f_key_routes_through_fetch_action() {
    let mut app = app::App::new(PathBuf::from("/tmp/test-home"));
    app.dsh_provider_id = Some("test-provider".to_string());
    app.dsh_providers = vec![(
        "test-provider".to_string(),
        droidgear_core::dsh::DshProviderConfig {
            base_url: Some("https://example.com/v1".to_string()),
            api_key_env: Some("TEST_API_KEY".to_string()),
            ..Default::default()
        },
    )];

    let action = super::keys_dsh::handle_dsh_provider_key(&mut app, KeyCode::Char('f'));

    match action {
        Some(super::Action::FetchDshModels { provider_id }) => {
            assert_eq!(provider_id, "test-provider");
        }
        other => panic!("expected FetchDshModels action, got {other:?}"),
    }
}

#[test]
fn dsh_sessions_key_routes_view_and_delete_actions() {
    let mut app = app::App::new(PathBuf::from("/tmp/test-home"));
    app.dsh_sessions = vec![droidgear_core::dsh_sessions::DshSessionSummary {
        id: "session".to_string(),
        title: "Session".to_string(),
        project: "/tmp/project".to_string(),
        model: "model".to_string(),
        model_provider: "provider".to_string(),
        modified_at: 0.0,
        token_usage: droidgear_core::dsh_sessions::DshTokenUsage::default(),
        path: "/tmp/session.v4.jsonl.zstd".to_string(),
    }];

    let view = super::keys_dsh_sessions::handle_dsh_sessions_key(&mut app, KeyCode::Enter);
    assert!(matches!(view, Some(super::Action::ViewDshSession { .. })));

    super::keys_dsh_sessions::handle_dsh_sessions_key(&mut app, KeyCode::Char('d'));
    assert!(matches!(
        app.modal,
        Some(app::Modal::Confirm {
            action: app::ConfirmAction::DshSessionDelete { .. },
            ..
        })
    ));
}

#[test]
fn dsh_action_variants_exist() {
    let _add = app::InputAction::DshAddProvider;
    let _add_channel = app::InputAction::DshAddProviderFromChannel;
    let _import_key = app::InputAction::DshImportSetApiKey {
        provider_id: "x".to_string(),
    };
    let _import_channel = app::SelectAction::DshImportFromChannel {
        provider_id: "x".to_string(),
    };
    let _import_toggle = app::SelectAction::DshImportToggleModel {
        provider_id: "x".to_string(),
    };
    let _add_fetched = app::SelectAction::DshAddFetchedModels {
        provider_id: "x".to_string(),
    };
    let _set_display = app::InputAction::DshSetProviderDisplayName {
        provider_id: "x".to_string(),
    };
    let _set_base = app::InputAction::DshSetProviderBaseUrl {
        provider_id: "x".to_string(),
    };
    let _set_env = app::InputAction::DshSetProviderApiKeyEnv {
        provider_id: "x".to_string(),
    };
    let _set_key = app::InputAction::DshSetProviderApiKey {
        provider_id: "x".to_string(),
    };
    let _add_model = app::InputAction::DshAddModel {
        provider_id: "x".to_string(),
    };
    let _set_model_id = app::InputAction::DshSetModelId {
        provider_id: "x".to_string(),
        model_index: 0,
    };
    let _set_model_name = app::InputAction::DshSetModelName {
        provider_id: "x".to_string(),
        model_index: 0,
    };
    let _set_ctx = app::InputAction::DshSetModelContextWindow {
        provider_id: "x".to_string(),
        model_index: 0,
    };
    let _set_max = app::InputAction::DshSetModelMaxTokens {
        provider_id: "x".to_string(),
        model_index: 0,
    };
    let _sel_api = app::SelectAction::DshSetProviderApi {
        provider_id: "x".to_string(),
    };
    let _del_provider = app::ConfirmAction::DshDeleteProvider {
        provider_id: "x".to_string(),
    };
    let _del_model = app::ConfirmAction::DshDeleteModel {
        provider_id: "x".to_string(),
        model_index: 0,
    };
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
fn preview_droid_run_uses_selected_settings_path_without_dumping_contents() {
    let temp = TempDir::new().unwrap();
    let settings_path = temp.path().join(".droidgear/droid-settings/profile-a.json");
    write_file(
        &settings_path,
        r#"{"apiKey":"sk-droid-secret","model":"demo"}"#,
    );

    let preview = preview_droid_run(temp.path(), &settings_path).unwrap();

    assert!(preview.contains("Droid run preview"));
    assert!(preview.contains(settings_path.to_string_lossy().as_ref()));
    assert!(preview.contains("FACTORY_DROID_AUTO_UPDATE_ENABLED=0"));
    assert!(preview.contains("ANTHROPIC_AUTH_TOKEN"));
    assert!(!preview.contains("sk-droid-secret"));
}

#[test]
fn list_droid_run_targets_lists_global_and_custom_names() {
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

    let output = list_droid_run_targets(temp.path()).unwrap();

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
            api_key_model_discovery: false,
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
            api_key_model_discovery: false,
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
            api_key_model_discovery: false,
            auth_profile_name: None,
        },
    )
    .unwrap();

    let output = preview_codex_temporary_run(temp.path(), "profile-a").unwrap();

    assert!(output.contains("Codex temporary run preview"));
    assert!(output.contains("Runtime CODEX_HOME:"));
    assert!(output.contains("Secret environment keys:"));
    assert!(output.contains("(none)"));
    assert!(!output.contains("sk-secret"));
}

#[test]
fn codex_set_provider_context_window_links_auto_compact_preset() {
    let temp = TempDir::new().unwrap();
    let mut providers = HashMap::new();
    providers.insert(
        "custom".to_string(),
        droidgear_core::codex::CodexProviderConfig {
            name: Some("Custom".to_string()),
            base_url: None,
            wire_api: Some("responses".to_string()),
            requires_openai_auth: Some(false),
            env_key: None,
            env_key_instructions: None,
            http_headers: None,
            query_params: None,
            model: Some("gpt-5.6-sol".to_string()),
            model_reasoning_effort: None,
            model_context_window: None,
            model_auto_compact_token_limit: None,
            api_key: None,
        },
    );
    droidgear_core::codex::save_codex_profile_for_home(
        temp.path(),
        droidgear_core::codex::CodexProfile {
            id: "profile-a".to_string(),
            name: "Alpha".to_string(),
            description: None,
            created_at: "2026-01-01T00:00:00Z".to_string(),
            updated_at: "2026-01-01T00:00:00Z".to_string(),
            providers,
            model_provider: "custom".to_string(),
            model: "gpt-5.6-sol".to_string(),
            model_reasoning_effort: None,
            api_key: None,
            api_key_model_discovery: false,
            auth_profile_name: None,
        },
    )
    .unwrap();

    let set_context_window = |app: &mut app::App, value: &str| {
        modal::run_input_action(
            app,
            app::InputAction::CodexSetProviderContextWindow {
                profile_id: "profile-a".to_string(),
                provider_id: "custom".to_string(),
            },
            value.to_string(),
        )
        .unwrap();
    };
    let get_provider = |home: &Path| {
        droidgear_core::codex::get_codex_profile_for_home(home, "profile-a")
            .unwrap()
            .providers
            .get("custom")
            .cloned()
            .unwrap()
    };

    let mut app = app::App::new(temp.path().to_path_buf());

    // 272K tier links the 250K auto-compact preset.
    set_context_window(&mut app, "272000");
    let provider = get_provider(temp.path());
    assert_eq!(provider.model_context_window, Some(272_000));
    assert_eq!(provider.model_auto_compact_token_limit, Some(250_000));

    // 1M tier links the 900K auto-compact preset.
    set_context_window(&mut app, "1000000");
    let provider = get_provider(temp.path());
    assert_eq!(provider.model_context_window, Some(1_000_000));
    assert_eq!(provider.model_auto_compact_token_limit, Some(900_000));

    // A non-preset window keeps the previously picked compact limit.
    set_context_window(&mut app, "300000");
    let provider = get_provider(temp.path());
    assert_eq!(provider.model_context_window, Some(300_000));
    assert_eq!(provider.model_auto_compact_token_limit, Some(900_000));
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
        app::Screen::DshDesktop,
        app::Screen::DshWeb,
        app::Screen::DshSessions,
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

#[test]
fn droid_settings_link_key_opens_input_and_links_local_json_file() {
    let temp = TempDir::new().unwrap();
    write_file(&temp.path().join(".factory/settings.json"), "{}");
    let external = temp.path().join("team-settings.json");
    write_file(&external, r#"{"customModels":[]}"#);

    let mut app = app::App::new(temp.path().to_path_buf());
    app.screen = app::Screen::DroidSettingsFiles;

    super::keys_droid_settings::handle_droid_settings_files_key(&mut app, KeyCode::Char('i'));
    assert!(matches!(
        app.modal.as_ref(),
        Some(app::Modal::Input {
            action: app::InputAction::DroidSettingsLink,
            ..
        })
    ));

    modal::run_input_action(
        &mut app,
        app::InputAction::DroidSettingsLink,
        external.to_string_lossy().to_string(),
    )
    .unwrap();

    let files =
        droidgear_core::droid_settings_files::list_settings_files_for_home(temp.path()).unwrap();
    assert!(files
        .iter()
        .any(|f| f.is_external && f.is_active && f.name == "team-settings"));

    // The source file stays untouched at its original location.
    assert!(external.exists());
    assert_eq!(
        std::fs::read_to_string(&external).unwrap(),
        r#"{"customModels":[]}"#
    );
}

#[test]
fn baizhi_preset_routes_through_confirmation_and_roundtrips_symbolic_config() {
    let temp = TempDir::new().unwrap();
    let mut app = app::App::new(temp.path().to_path_buf());
    app.screen = app::Screen::Mcp;
    handle_key(&mut app, KeyCode::Char('b'));
    assert!(matches!(
        app.modal,
        Some(app::Modal::Confirm {
            action: app::ConfirmAction::McpAddBaizhi,
            ..
        })
    ));
    assert!(!temp.path().join(".factory/mcp.json").exists());
    handle_key(&mut app, KeyCode::Enter);
    assert!(app.modal.is_none());
    let loaded = droidgear_core::mcp::load_mcp_servers_for_home(temp.path()).unwrap();
    assert_eq!(loaded.len(), 1);
    let saved = &loaded[0];
    assert_eq!(saved.name, "baizhi");
    assert_eq!(
        saved.config.server_type,
        droidgear_core::mcp::McpServerType::Http
    );
    assert!(saved.config.disabled);
    assert_eq!(
        saved.config.url.as_deref(),
        Some("https://agent-toolkit.app.baizhi.cloud/mcp")
    );
    assert_eq!(
        saved
            .config
            .headers
            .as_ref()
            .unwrap()
            .get("Authorization")
            .map(String::as_str),
        Some("Bearer ${BAIZHI_API_KEY}")
    );
    assert!(saved.config.env.is_none());
    let disk: serde_json::Value = serde_json::from_str(
        &std::fs::read_to_string(temp.path().join(".factory/mcp.json")).unwrap(),
    )
    .unwrap();
    assert_eq!(
        disk["mcpServers"]["baizhi"],
        serde_json::json!({
            "type": "http", "disabled": true, "oauth": false,
            "url": "https://agent-toolkit.app.baizhi.cloud/mcp",
            "headers": { "Authorization": "Bearer ${BAIZHI_API_KEY}" }
        })
    );
}

#[test]
fn baizhi_preset_cancel_does_not_write_config() {
    let temp = TempDir::new().unwrap();
    let mut app = app::App::new(temp.path().to_path_buf());
    app.screen = app::Screen::Mcp;
    handle_key(&mut app, KeyCode::Char('b'));
    handle_key(&mut app, KeyCode::Esc);
    assert!(app.modal.is_none());
    assert!(!temp.path().join(".factory/mcp.json").exists());
}

#[test]
fn baizhi_preset_preserves_existing_entry_at_confirmation() {
    let temp = TempDir::new().unwrap();
    let mut app = app::App::new(temp.path().to_path_buf());
    app.screen = app::Screen::Mcp;
    handle_key(&mut app, KeyCode::Char('b'));
    let path = temp.path().join(".factory/mcp.json");
    let contents = "{\"mcpServers\":{\"baizhi\":{\"type\":\"http\",\"disabled\":false,\"url\":\"https://example.test/custom\",\"oauth\":{\"clientId\":\"synthetic-client\"}}}}";
    write_file(&path, contents);
    handle_key(&mut app, KeyCode::Enter);
    assert_eq!(std::fs::read_to_string(path).unwrap(), contents);
    assert!(app.toast_message().contains("already exists"));
}

#[test]
fn baizhi_confirmation_renders_endpoint_header_and_controls_at_80x24() {
    let temp = TempDir::new().unwrap();
    let mut app = app::App::new(temp.path().to_path_buf());
    app.screen = app::Screen::Mcp;
    handle_key(&mut app, KeyCode::Char('b'));
    let backend = ratatui::backend::TestBackend::new(80, 24);
    let mut terminal = ratatui::Terminal::new(backend).unwrap();
    terminal.draw(|frame| crate::ui::draw(frame, &app)).unwrap();
    let buffer = terminal.backend().buffer();
    let rendered = (0..24)
        .map(|y| (0..80).map(|x| buffer[(x, y)].symbol()).collect::<String>())
        .collect::<Vec<_>>()
        .join("\n");
    for visible in [
        "https://agent-toolkit.app.baizhi.cloud/mcp",
        "Authorization: Bearer ${BAIZHI_API_KEY}",
        "oauth:false",
        "y/Enter: yes  n/Esc: no",
    ] {
        assert!(
            rendered.contains(visible),
            "Missing {visible} in:\n{rendered}"
        );
    }
    assert!(!temp.path().join(".factory/mcp.json").exists());
}

// --- DSH configuration screen (AGENTS.md rule 9) ---

fn dsh_config_write_profile(home: &Path, patch: &str) {
    let dir = home.join(".dsh/profiles/desktop");
    std::fs::create_dir_all(&dir).unwrap();
    write_file(
        &dir.join("package.json"),
        r#"{"name":"dsh-profile-desktop","private":true,"dsh":{"profile":{"bundles":["droidgear-test-bundle"]}}}"#,
    );
    write_file(&dir.join("cordis.yml"), "[]\n");
    write_file(&dir.join("cordis.patch.yml"), patch);
}

fn wait_dsh_config(app: &mut app::App) {
    for _ in 0..400 {
        super::refresh::poll_dsh_config_refresh(app);
        if !app.dsh_config_loading {
            return;
        }
        std::thread::sleep(std::time::Duration::from_millis(5));
    }
    panic!(
        "DSH configuration refresh did not finish (toast: {})",
        app.toast_message()
    );
}

fn dsh_config_entry(
    id: &str,
    tier: &str,
    layer: &str,
) -> droidgear_core::dsh_config::DshConfigEntryView {
    droidgear_core::dsh_config::DshConfigEntryView {
        id: id.to_string(),
        package_name: format!("@deepseek-ai/dsh-{id}"),
        layer: layer.to_string(),
        tier: tier.to_string(),
        description: String::new(),
        disabled: false,
        has_js: false,
        config: None,
        overridden: false,
        shadowed_by_home: false,
        fields: Vec::new(),
    }
}

fn dsh_config_field(
    name: &str,
    kind: &str,
    value: serde_json::Value,
    default: serde_json::Value,
) -> droidgear_core::dsh_config::DshConfigFieldView {
    droidgear_core::dsh_config::DshConfigFieldView {
        name: name.to_string(),
        kind: kind.to_string(),
        value: Some(value),
        default: Some(default),
        enum_values: vec!["light".to_string(), "dark".to_string()],
        volatile: true,
        secret: false,
        description: String::new(),
        overridden: true,
        tier: "A".to_string(),
    }
}

fn dsh_config_warning(
    severity: &str,
    code: &str,
    message: &str,
    hint: &str,
) -> droidgear_core::dsh_config::DshMigrationWarning {
    droidgear_core::dsh_config::DshMigrationWarning {
        code: code.to_string(),
        severity: severity.to_string(),
        entry_id: None,
        message: message.to_string(),
        hint: hint.to_string(),
    }
}

fn dsh_config_snapshot(
    entries: Vec<droidgear_core::dsh_config::DshConfigEntryView>,
    warnings: Vec<droidgear_core::dsh_config::DshMigrationWarning>,
) -> droidgear_core::dsh_config::DshConfigSnapshot {
    droidgear_core::dsh_config::DshConfigSnapshot {
        schema_version: "0.2.1-rc.2".to_string(),
        installed_version: Some("0.2.1".to_string()),
        profile: Some("desktop".to_string()),
        profile_patch_path: "/tmp/home/.dsh/profiles/desktop/cordis.patch.yml".to_string(),
        home_patch_path: "/tmp/home/.dsh/cordis.patch.yml".to_string(),
        home_patch_exists: false,
        legacy_settings_path: "/tmp/home/.dsh/settings.yaml".to_string(),
        legacy_settings_exists: false,
        patch_hash: "0123456789abcdef".to_string(),
        entries,
        warnings,
    }
}

fn render_visible(app: &app::App, width: u16, height: u16) -> String {
    let backend = ratatui::backend::TestBackend::new(width, height);
    let mut terminal = ratatui::Terminal::new(backend).unwrap();
    terminal.draw(|frame| crate::ui::draw(frame, app)).unwrap();
    let buffer = terminal.backend().buffer();
    (0..height)
        .map(|y| {
            (0..width)
                .map(|x| buffer[(x, y)].symbol())
                .collect::<String>()
        })
        .collect::<Vec<_>>()
        .join("\n")
}

#[test]
fn dsh_config_screen_initializes_correctly() {
    let app = app::App::new(PathBuf::from("/tmp/test-home"));
    assert!(app.dsh_config_snapshot.is_none());
    assert_eq!(app.dsh_config_index, 0);
    assert!(app.dsh_config_expanded.is_empty());
    assert_eq!(app.dsh_config_focus, app::DshConfigFocus::Entries);
    assert_eq!(app.dsh_config_field_index, 0);
    assert!(!app.dsh_config_loading);
    assert!(app.dsh_config_ordered_entries().is_empty());
    let _screen = app::Screen::DshConfig;
}

#[test]
fn dsh_c_key_opens_config_screen_from_dsh_list() {
    let temp = TempDir::new().unwrap();
    let mut app = app::App::new(temp.path().to_path_buf());
    app.screen = app::Screen::DshDesktop;

    super::keys_dsh::handle_dsh_key(&mut app, KeyCode::Char('c'));

    assert_eq!(app.screen, app::Screen::DshConfig);
    assert!(app.dsh_config_loading);
    wait_dsh_config(&mut app);
    assert!(!app.dsh_config_loading);
    // No ~/.dsh/profiles in this home: the snapshot explains that instead of
    // silently showing an empty list.
    let snapshot = app.dsh_config_snapshot.as_ref().expect("snapshot");
    assert!(snapshot.profile.is_none());
    assert!(snapshot
        .warnings
        .iter()
        .any(|warning| warning.code == "DSH_CONFIG_NO_PROFILE" && warning.severity == "error"));
    assert!(snapshot.entries.is_empty());
}

#[test]
fn dsh_config_empty_profile_renders_error_warning_and_no_entries() {
    let temp = TempDir::new().unwrap();
    let mut app = app::App::new(temp.path().to_path_buf());
    app.screen = app::Screen::DshConfig;
    super::refresh::request_dsh_config_refresh(&mut app);
    wait_dsh_config(&mut app);

    let rendered = render_visible(&app, 100, 30);
    for visible in [
        "Dsh Config",
        "Profile: (none)",
        "[E]",
        "No DSH runtime profile",
        "Entries (0)",
        "No configuration entries",
    ] {
        assert!(
            rendered.contains(visible),
            "Missing {visible} in:\n{rendered}"
        );
    }
}

#[test]
fn dsh_config_renders_header_tiered_entries_and_warnings() {
    let mut app = app::App::new(PathBuf::from("/tmp/test-home"));
    app.screen = app::Screen::DshConfig;

    let mut theme = dsh_config_entry("ui-theme", "A", "profile");
    theme.overridden = true;
    theme.has_js = true;
    theme.fields = vec![
        {
            let mut field = dsh_config_field(
                "preference",
                "enum",
                serde_json::json!("dark"),
                serde_json::json!("system"),
            );
            field.enum_values = vec!["light".to_string(), "dark".to_string()];
            field
        },
        dsh_config_field(
            "fontSize",
            "number",
            serde_json::json!(14),
            serde_json::json!(13),
        ),
        {
            let mut field = dsh_config_field(
                "textFontFamily",
                "string",
                serde_json::json!("Inter"),
                serde_json::json!(""),
            );
            field.secret = true;
            field
        },
    ];
    let mut log = dsh_config_entry("session-log-deepseek", "B", "bundle:@deepseek-ai/dsh-base");
    log.disabled = true;
    let mut search = dsh_config_entry("web-search-deepseek", "B", "home");
    search.shadowed_by_home = true;
    app.dsh_config_snapshot = Some(dsh_config_snapshot(
        vec![search, theme, log],
        vec![
            dsh_config_warning("error", "ERR", "tools.mode is invalid", "Use native or ptc"),
            dsh_config_warning("warning", "WARN", "field was removed", "Remove the field"),
            dsh_config_warning(
                "info",
                "INFO",
                "legacy settings present",
                "No action needed",
            ),
        ],
    ));
    app.dsh_config_expanded.insert("ui-theme".to_string());
    app.dsh_config_index = 0; // first entry in display order is tier A: ui-theme

    let rendered = render_visible(&app, 120, 40);

    for visible in [
        "Dsh Config",
        "Profile: desktop",
        "Schema: 0.2.1-rc.2",
        "Installed: 0.2.1",
        "Profile patch: /tmp/home/.dsh/profiles/desktop/cordis.patch.yml",
        "Patch hash: 0123456789abcdef",
        "Warnings (3)",
        "[E]",
        "[W]",
        "[i]",
        "tools.mode is invalid",
        "Use native or ptc",
        "legacy settings present",
        "-- tier A --",
        "-- tier B --",
        "ui-theme",
        "session-log-deepseek",
        "web-search-deepseek",
        "[disabled]",
        "[override]",
        "[home]",
        "[js]",
        "preference",
        "fontSize",
        "textFontFamily",
        "<secret>",
        "[volatile]",
        "[secret]",
        "default:",
        "Entries (3)",
    ] {
        assert!(
            rendered.contains(visible),
            "Missing {visible} in:\n{rendered}"
        );
    }

    // Tiers are grouped in order A before B, and the flag-only home entry
    // stays out of the writable profile layer.
    let tier_a = rendered.find("-- tier A --").expect("tier A header");
    let tier_b = rendered.find("-- tier B --").expect("tier B header");
    let first_entry = rendered.find("ui-theme").expect("ui-theme");
    assert!(tier_a < first_entry && first_entry < tier_b);
}

#[test]
fn dsh_config_renders_no_warnings_state() {
    let mut app = app::App::new(PathBuf::from("/tmp/test-home"));
    app.screen = app::Screen::DshConfig;
    app.dsh_config_snapshot = Some(dsh_config_snapshot(Vec::new(), Vec::new()));

    let rendered = render_visible(&app, 100, 30);
    assert!(rendered.contains("Warnings (0)"), "{rendered}");
    assert!(rendered.contains("No warnings"), "{rendered}");
}

#[test]
fn dsh_config_navigation_expands_entries_and_moves_fields() {
    let mut app = app::App::new(PathBuf::from("/tmp/test-home"));
    app.screen = app::Screen::DshConfig;

    let mut first = dsh_config_entry("alpha", "A", "profile");
    first.fields = vec![
        dsh_config_field("one", "number", serde_json::json!(1), serde_json::json!(1)),
        dsh_config_field(
            "two",
            "boolean",
            serde_json::json!(true),
            serde_json::json!(true),
        ),
    ];
    let second = dsh_config_entry("beta", "A", "profile");
    let third = dsh_config_entry("gamma", "C", "bundle:@deepseek-ai/dsh-base");
    app.dsh_config_snapshot = Some(dsh_config_snapshot(vec![first, second, third], Vec::new()));

    let press = |app: &mut app::App, code: KeyCode| {
        super::keys_dsh_config::handle_dsh_config_key(app, code);
    };

    press(&mut app, KeyCode::Down);
    assert_eq!(app.dsh_config_index, 1);
    press(&mut app, KeyCode::PageDown);
    assert_eq!(app.dsh_config_index, 2);
    press(&mut app, KeyCode::Down);
    assert_eq!(app.dsh_config_index, 2, "clamped at the last entry");
    press(&mut app, KeyCode::Up);
    assert_eq!(app.dsh_config_index, 1);
    press(&mut app, KeyCode::PageUp);
    assert_eq!(app.dsh_config_index, 0);

    // Enter toggles the field list, Tab moves focus into it.
    press(&mut app, KeyCode::Enter);
    assert!(app.dsh_config_expanded.contains("alpha"));
    assert_eq!(app.dsh_config_focus, app::DshConfigFocus::Entries);
    press(&mut app, KeyCode::Tab);
    assert_eq!(app.dsh_config_focus, app::DshConfigFocus::Fields);
    press(&mut app, KeyCode::Down);
    assert_eq!(app.dsh_config_field_index, 1);
    press(&mut app, KeyCode::Down);
    assert_eq!(app.dsh_config_field_index, 1, "clamped at the last field");
    press(&mut app, KeyCode::PageUp);
    assert_eq!(app.dsh_config_field_index, 0);
    press(&mut app, KeyCode::Tab);
    assert_eq!(app.dsh_config_focus, app::DshConfigFocus::Entries);
    press(&mut app, KeyCode::Enter);
    assert!(!app.dsh_config_expanded.contains("alpha"));
}

#[test]
fn dsh_config_d_toggles_disabled_through_core() {
    let temp = TempDir::new().unwrap();
    dsh_config_write_profile(
        temp.path(),
        "- id: session-log-deepseek\n  name: '@deepseek-ai/dsh-session-log-deepseek'\n  config:\n    enabled: true\n",
    );
    let mut app = app::App::new(temp.path().to_path_buf());
    app.dsh_active_profile = Some("desktop".to_string());
    app.screen = app::Screen::DshConfig;
    super::refresh::request_dsh_config_refresh(&mut app);
    wait_dsh_config(&mut app);

    let snapshot = app
        .dsh_config_snapshot
        .as_ref()
        .unwrap_or_else(|| panic!("snapshot missing; toast={}", app.toast_message()));
    assert_eq!(snapshot.entries.len(), 1);
    assert_eq!(snapshot.entries[0].id, "session-log-deepseek");
    assert!(!snapshot.entries[0].disabled);

    super::keys_dsh_config::handle_dsh_config_key(&mut app, KeyCode::Char('d'));
    assert!(
        app.toast_message().contains("Disabled"),
        "{}",
        app.toast_message()
    );
    wait_dsh_config(&mut app);

    let patch = std::fs::read_to_string(temp.path().join(".dsh/profiles/desktop/cordis.patch.yml"))
        .unwrap();
    assert!(patch.contains("disabled: true"), "{patch}");
    let snapshot = app.dsh_config_snapshot.as_ref().expect("snapshot");
    assert!(snapshot.entries[0].disabled);

    super::keys_dsh_config::handle_dsh_config_key(&mut app, KeyCode::Char('d'));
    wait_dsh_config(&mut app);
    assert!(!app.dsh_config_snapshot.as_ref().unwrap().entries[0].disabled);
}

#[test]
fn dsh_config_edits_boolean_field_through_select_modal() {
    let temp = TempDir::new().unwrap();
    dsh_config_write_profile(
        temp.path(),
        "- id: session-log-deepseek\n  name: '@deepseek-ai/dsh-session-log-deepseek'\n  config:\n    enabled: true\n",
    );
    let mut app = app::App::new(temp.path().to_path_buf());
    app.dsh_active_profile = Some("desktop".to_string());
    app.screen = app::Screen::DshConfig;
    super::refresh::request_dsh_config_refresh(&mut app);
    wait_dsh_config(&mut app);

    super::keys_dsh_config::handle_dsh_config_key(&mut app, KeyCode::Enter);
    assert!(app.dsh_config_expanded.contains("session-log-deepseek"));
    super::keys_dsh_config::handle_dsh_config_key(&mut app, KeyCode::Tab);
    assert_eq!(app.dsh_config_focus, app::DshConfigFocus::Fields);

    // The first schema field is the `enabled` boolean.
    super::keys_dsh_config::handle_dsh_config_key(&mut app, KeyCode::Enter);
    let Some(app::Modal::Select {
        options,
        index,
        action,
        ..
    }) = app.modal.take()
    else {
        panic!("expected a select modal, got {:?}", app.modal);
    };
    assert_eq!(options, vec!["true".to_string(), "false".to_string()]);
    assert_eq!(index, 0);

    modal::run_select_action(&mut app, action, 1, Some("false".to_string())).unwrap();
    wait_dsh_config(&mut app);

    let patch = std::fs::read_to_string(temp.path().join(".dsh/profiles/desktop/cordis.patch.yml"))
        .unwrap();
    assert!(patch.contains("enabled: false"), "{patch}");
    let snapshot = app.dsh_config_snapshot.as_ref().expect("snapshot");
    assert_eq!(
        snapshot.entries[0].config,
        Some(serde_json::json!({"enabled": false}))
    );
    assert!(
        app.toast_message().contains("Saved"),
        "{}",
        app.toast_message()
    );
}

// --- DSH provider default reasoning + agent default model (task-10) ---

fn dsh_provider_with_models() -> droidgear_core::dsh::DshProviderConfig {
    droidgear_core::dsh::DshProviderConfig {
        base_url: Some("https://example.com/v1".to_string()),
        api_key_env: Some("TEST_API_KEY".to_string()),
        ..Default::default()
    }
}

fn dsh_provider_app(home: &Path) -> app::App {
    let mut app = app::App::new(home.to_path_buf());
    app.dsh_active_profile = Some("desktop".to_string());
    app.screen = app::Screen::DshProvider;
    app.dsh_provider_id = Some("test-provider".to_string());
    super::refresh::refresh_dsh(&mut app);
    app.dsh_provider_field_index = 6;
    app
}

#[test]
fn dsh_provider_reasoning_field_edit_persists_and_clears() {
    let temp = TempDir::new().unwrap();
    dsh_config_write_profile(temp.path(), "[]\n");
    droidgear_core::dsh::save_dsh_provider_for_profile(
        temp.path(),
        Some("desktop"),
        "test-provider",
        &dsh_provider_with_models(),
    )
    .unwrap();

    let mut app = dsh_provider_app(temp.path());
    super::keys_dsh::handle_dsh_provider_key(&mut app, KeyCode::Enter);
    let Some(app::Modal::Input { action, value, .. }) = app.modal.take() else {
        panic!("expected reasoning input, got {:?}", app.modal);
    };
    assert!(value.is_empty());
    super::modal::run_input_action(&mut app, action, "HIGH".to_string()).unwrap();

    let patch = std::fs::read_to_string(temp.path().join(".dsh/profiles/desktop/cordis.patch.yml"))
        .unwrap();
    assert!(patch.contains("reasoning: high"), "{patch}");
    let current =
        droidgear_core::dsh::read_dsh_current_config_for_profile(temp.path(), Some("desktop"))
            .unwrap();
    assert_eq!(
        current
            .providers
            .get("test-provider")
            .unwrap()
            .reasoning
            .as_deref(),
        Some("high")
    );

    let rendered = render_visible(&app, 100, 30);
    assert!(rendered.contains("Default Reasoning"), "{rendered}");
    assert!(rendered.contains("high"), "{rendered}");

    // An empty submission clears the route default again.
    app.dsh_provider_field_index = 6;
    super::keys_dsh::handle_dsh_provider_key(&mut app, KeyCode::Enter);
    let Some(app::Modal::Input { action, value, .. }) = app.modal.take() else {
        panic!("expected reasoning input, got {:?}", app.modal);
    };
    assert_eq!(value, "high");
    super::modal::run_input_action(&mut app, action, String::new()).unwrap();
    let current =
        droidgear_core::dsh::read_dsh_current_config_for_profile(temp.path(), Some("desktop"))
            .unwrap();
    assert!(current
        .providers
        .get("test-provider")
        .unwrap()
        .reasoning
        .is_none());
}

#[test]
fn dsh_provider_reasoning_invalid_value_keeps_dialog_and_does_not_write() {
    let temp = TempDir::new().unwrap();
    dsh_config_write_profile(temp.path(), "[]\n");
    droidgear_core::dsh::save_dsh_provider_for_profile(
        temp.path(),
        Some("desktop"),
        "test-provider",
        &dsh_provider_with_models(),
    )
    .unwrap();
    let mut app = dsh_provider_app(temp.path());

    app.modal = Some(app::Modal::Input {
        title: "Default reasoning".to_string(),
        value: "turbo".to_string(),
        cursor: usize::MAX,
        is_secret: false,
        action: app::InputAction::DshSetProviderReasoning {
            provider_id: "test-provider".to_string(),
        },
    });
    handle_key(&mut app, KeyCode::Enter);

    assert!(
        matches!(app.modal, Some(app::Modal::Input { .. })),
        "invalid value must keep the dialog open"
    );
    match app.modal.as_ref() {
        Some(app::Modal::Input { value, .. }) => assert_eq!(value, "turbo"),
        other => panic!("expected input dialog, got {other:?}"),
    }
    assert!(
        app.toast_message().contains("Invalid reasoning level"),
        "{}",
        app.toast_message()
    );
    let rendered = render_visible(&app, 110, 30);
    assert!(rendered.contains("Invalid reasoning level"), "{rendered}");

    let current =
        droidgear_core::dsh::read_dsh_current_config_for_profile(temp.path(), Some("desktop"))
            .unwrap();
    assert!(current
        .providers
        .get("test-provider")
        .unwrap()
        .reasoning
        .is_none());
}

#[test]
fn dsh_model_g_writes_agent_default_model() {
    let temp = TempDir::new().unwrap();
    dsh_config_write_profile(
        temp.path(),
        "- id: agent-default-model\n  name: '@deepseek-ai/dsh-agent-default-model'\n  config:\n    provider: old-provider\n    model: old-model\n",
    );
    let mut provider = dsh_provider_with_models();
    provider.reasoning = Some("high".to_string());
    provider.models = vec![droidgear_core::dsh::DshModel {
        id: "test-model".to_string(),
        ..Default::default()
    }];
    droidgear_core::dsh::save_dsh_provider_for_profile(
        temp.path(),
        Some("desktop"),
        "test-provider",
        &provider,
    )
    .unwrap();

    let mut app = app::App::new(temp.path().to_path_buf());
    app.dsh_active_profile = Some("desktop".to_string());
    app.screen = app::Screen::DshModel;
    app.dsh_provider_id = Some("test-provider".to_string());
    app.dsh_model_index = 0;
    app.dsh_model_field_index = 0;
    super::refresh::refresh_dsh(&mut app);

    super::keys_dsh::handle_dsh_model_key(&mut app, KeyCode::Char('g'));
    assert!(
        app.toast_message().contains("Default model"),
        "{}",
        app.toast_message()
    );

    let snapshot = droidgear_core::dsh_config::read_dsh_config_snapshot_for_home(
        temp.path(),
        Some("desktop".to_string()),
    )
    .unwrap();
    let entry = snapshot
        .entries
        .iter()
        .find(|entry| entry.id == "agent-default-model")
        .expect("agent-default-model entry");
    assert_eq!(
        entry.config,
        Some(serde_json::json!({
            "provider": "test-provider",
            "model": "test-model",
            "reasoningEffort": "high",
        }))
    );
    let patch = std::fs::read_to_string(temp.path().join(".dsh/profiles/desktop/cordis.patch.yml"))
        .unwrap();
    assert!(patch.contains("reasoningEffort: high"), "{patch}");

    // The model screen advertises the shortcut and flags the current default.
    app.dsh_config_snapshot = Some(snapshot);
    let rendered = render_visible(&app, 110, 30);
    assert!(rendered.contains("g: set default model"), "{rendered}");
    assert!(rendered.contains("current default"), "{rendered}");

    // Without a provider-level reasoning default the entry omits it.
    provider.reasoning = None;
    droidgear_core::dsh::save_dsh_provider_for_profile(
        temp.path(),
        Some("desktop"),
        "test-provider",
        &provider,
    )
    .unwrap();
    super::refresh::refresh_dsh(&mut app);
    super::keys_dsh::handle_dsh_model_key(&mut app, KeyCode::Char('g'));
    let snapshot = droidgear_core::dsh_config::read_dsh_config_snapshot_for_home(
        temp.path(),
        Some("desktop".to_string()),
    )
    .unwrap();
    let entry = snapshot
        .entries
        .iter()
        .find(|entry| entry.id == "agent-default-model")
        .expect("agent-default-model entry");
    assert_eq!(
        entry.config,
        Some(serde_json::json!({
            "provider": "test-provider",
            "model": "test-model",
        }))
    );
}

// --- DSH model input modalities (task-12) ---

fn dsh_model_app(home: &Path) -> app::App {
    let mut provider = dsh_provider_with_models();
    provider.models = vec![droidgear_core::dsh::DshModel {
        id: "test-model".to_string(),
        ..Default::default()
    }];
    droidgear_core::dsh::save_dsh_provider_for_profile(
        home,
        Some("desktop"),
        "test-provider",
        &provider,
    )
    .unwrap();
    let mut app = app::App::new(home.to_path_buf());
    app.dsh_active_profile = Some("desktop".to_string());
    app.screen = app::Screen::DshModel;
    app.dsh_provider_id = Some("test-provider".to_string());
    app.dsh_model_index = 0;
    app.dsh_model_field_index = 5;
    super::refresh::refresh_dsh(&mut app);
    app
}

fn dsh_model_input(home: &Path) -> Option<Vec<String>> {
    let config =
        droidgear_core::dsh::read_dsh_current_config_for_profile(home, Some("desktop")).unwrap();
    config
        .providers
        .get("test-provider")
        .unwrap()
        .models
        .first()
        .unwrap()
        .input
        .clone()
}

fn select_model_input(app: &mut app::App) -> (app::SelectAction, usize) {
    super::keys_dsh::handle_dsh_model_key(app, KeyCode::Enter);
    let Some(app::Modal::Select { action, index, .. }) = app.modal.take() else {
        panic!("expected input modality select, got {:?}", app.modal);
    };
    (action, index)
}

#[test]
fn dsh_model_input_field_selects_text_image_and_auto() {
    let temp = TempDir::new().unwrap();
    dsh_config_write_profile(temp.path(), "[]\n");
    let mut app = dsh_model_app(temp.path());

    let (action, index) = select_model_input(&mut app);
    assert_eq!(index, 2, "unset input shows the auto option");
    super::modal::run_select_action(&mut app, action, 1, Some("text+image".to_string())).unwrap();
    assert_eq!(
        dsh_model_input(temp.path()),
        Some(vec!["text".to_string(), "image".to_string()])
    );
    let patch = std::fs::read_to_string(temp.path().join(".dsh/profiles/desktop/cordis.patch.yml"))
        .unwrap();
    assert!(patch.contains("input:"), "{patch}");
    assert!(patch.contains("- image"), "{patch}");

    super::refresh::refresh_dsh(&mut app);
    let rendered = render_visible(&app, 110, 30);
    assert!(rendered.contains("Input"), "{rendered}");
    assert!(rendered.contains("text, image"), "{rendered}");
    assert!(
        rendered.contains("Input field: text / text+image"),
        "{rendered}"
    );

    // The saved value is preselected next time round.
    let (action, index) = select_model_input(&mut app);
    assert_eq!(index, 1);
    super::modal::run_select_action(&mut app, action, 0, Some("text".to_string())).unwrap();
    assert_eq!(dsh_model_input(temp.path()), Some(vec!["text".to_string()]));

    // Auto clears the explicit list again.
    let (action, _) = select_model_input(&mut app);
    super::modal::run_select_action(&mut app, action, 2, Some("(auto: registry)".to_string()))
        .unwrap();
    assert_eq!(dsh_model_input(temp.path()), None);
    let patch = std::fs::read_to_string(temp.path().join(".dsh/profiles/desktop/cordis.patch.yml"))
        .unwrap();
    assert!(!patch.contains("input:"), "{patch}");
}

#[test]
fn dsh_model_input_cancel_does_not_write() {
    let temp = TempDir::new().unwrap();
    dsh_config_write_profile(temp.path(), "[]\n");
    let mut app = dsh_model_app(temp.path());
    let (action, _) = select_model_input(&mut app);
    // Esc/cancel arrives as `selected == None`.
    super::modal::run_select_action(&mut app, action, 0, None).unwrap();
    assert_eq!(dsh_model_input(temp.path()), None);
}

// --- DSH provider apiKeyEnv auto-derivation (task-14) ---

fn dsh_provider_create_app(home: &Path) -> app::App {
    let mut app = app::App::new(home.to_path_buf());
    app.dsh_active_profile = Some("desktop".to_string());
    app.screen = app::Screen::DshDesktop;
    super::refresh::refresh_dsh(&mut app);
    app
}

fn dsh_patch_text(home: &Path) -> String {
    std::fs::read_to_string(home.join(".dsh/profiles/desktop/cordis.patch.yml")).unwrap()
}

fn dsh_provider_env(home: &Path, provider_id: &str) -> Option<String> {
    let config =
        droidgear_core::dsh::read_dsh_current_config_for_profile(home, Some("desktop")).unwrap();
    config
        .providers
        .get(provider_id)
        .unwrap()
        .api_key_env
        .clone()
}

#[test]
fn dsh_provider_save_derives_api_key_env_when_empty() {
    let temp = TempDir::new().unwrap();
    dsh_config_write_profile(temp.path(), "[]\n");
    let mut app = dsh_provider_create_app(temp.path());

    super::modal::run_input_action(
        &mut app,
        app::InputAction::DshAddProvider,
        "openrouter".to_string(),
    )
    .unwrap();
    assert!(
        dsh_patch_text(temp.path()).contains("apiKeyEnv: OPENROUTER_API_KEY"),
        "{}",
        dsh_patch_text(temp.path())
    );
    assert_eq!(
        dsh_provider_env(temp.path(), "openrouter").as_deref(),
        Some("OPENROUTER_API_KEY")
    );

    // Clearing the field keeps the derived reference instead of writing None.
    super::modal::run_input_action(
        &mut app,
        app::InputAction::DshSetProviderApiKeyEnv {
            provider_id: "openrouter".to_string(),
        },
        String::new(),
    )
    .unwrap();
    assert_eq!(
        dsh_provider_env(temp.path(), "openrouter").as_deref(),
        Some("OPENROUTER_API_KEY")
    );
}

#[test]
fn dsh_provider_explicit_api_key_env_is_preserved() {
    let temp = TempDir::new().unwrap();
    dsh_config_write_profile(temp.path(), "[]\n");
    let mut envless = dsh_provider_with_models();
    envless.api_key_env = None;
    droidgear_core::dsh::save_dsh_provider_for_profile(
        temp.path(),
        Some("desktop"),
        "test-provider",
        &envless,
    )
    .unwrap();

    let mut app = dsh_provider_create_app(temp.path());
    super::modal::run_input_action(
        &mut app,
        app::InputAction::DshSetProviderApiKeyEnv {
            provider_id: "test-provider".to_string(),
        },
        "MY_KEY".to_string(),
    )
    .unwrap();

    assert!(
        dsh_patch_text(temp.path()).contains("apiKeyEnv: MY_KEY"),
        "{}",
        dsh_patch_text(temp.path())
    );
    assert_eq!(
        dsh_provider_env(temp.path(), "test-provider").as_deref(),
        Some("MY_KEY")
    );
}

#[test]
fn dsh_provider_derived_api_key_env_sanitizes_hyphens() {
    assert_eq!(
        super::keys_dsh::dsh_default_api_key_env("my-route"),
        "MY_ROUTE_API_KEY"
    );
    let temp = TempDir::new().unwrap();
    dsh_config_write_profile(temp.path(), "[]\n");
    let mut app = dsh_provider_create_app(temp.path());
    super::modal::run_input_action(
        &mut app,
        app::InputAction::DshAddProvider,
        "my-route-2".to_string(),
    )
    .unwrap();
    assert_eq!(
        dsh_provider_env(temp.path(), "my-route-2").as_deref(),
        Some("MY_ROUTE_2_API_KEY")
    );
    assert!(dsh_patch_text(temp.path()).contains("apiKeyEnv: MY_ROUTE_2_API_KEY"));
}

// --- DSH official DeepSeek API key (task-16) ---

fn dsh_official_app(home: &Path) -> app::App {
    let mut app = app::App::new(home.to_path_buf());
    app.dsh_active_profile = Some("desktop".to_string());
    app.screen = app::Screen::DshDesktop;
    super::refresh::refresh_dsh(&mut app);
    app
}

fn dsh_credentials_refs(home: &Path) -> HashMap<String, String> {
    droidgear_core::dsh::read_dsh_credentials_for_home(home)
        .unwrap()
        .refs
}

fn open_official_key_dialog(app: &mut app::App) -> (String, app::InputAction) {
    super::keys_dsh::handle_dsh_key(app, KeyCode::Char('o'));
    let Some(app::Modal::Input {
        is_secret,
        action,
        title,
        value,
        ..
    }) = app.modal.take()
    else {
        panic!("expected secret input dialog, got {:?}", app.modal);
    };
    assert!(is_secret);
    assert!(value.is_empty());
    (title, action)
}

#[test]
fn dsh_official_key_default_ref_saves_and_shows_status() {
    let temp = TempDir::new().unwrap();
    dsh_config_write_profile(temp.path(), "[]\n");
    let mut app = dsh_official_app(temp.path());
    assert_eq!(app.dsh_official_api_key_env, "DEEPSEEK_API_KEY");
    assert!(!app.dsh_official_api_key_configured);

    let (title, action) = open_official_key_dialog(&mut app);
    assert!(title.contains("DEEPSEEK_API_KEY"), "{title}");
    assert!(matches!(action, app::InputAction::DshSetOfficialApiKey));
    super::modal::run_input_action(&mut app, action, "sk-official".to_string()).unwrap();

    assert_eq!(
        dsh_credentials_refs(temp.path())
            .get("DEEPSEEK_API_KEY")
            .map(String::as_str),
        Some("sk-official")
    );
    assert!(
        app.toast_message().contains("Saved DEEPSEEK_API_KEY"),
        "{}",
        app.toast_message()
    );
    assert!(app.dsh_official_api_key_configured);

    let rendered = render_visible(&app, 100, 30);
    assert!(rendered.contains("o: DeepSeek official key"), "{rendered}");
    assert!(
        rendered.contains("DeepSeek official key: configured"),
        "{rendered}"
    );
    assert!(rendered.contains("(DEEPSEEK_API_KEY)"), "{rendered}");
}

#[test]
fn dsh_official_key_uses_configured_api_key_env_ref() {
    let temp = TempDir::new().unwrap();
    dsh_config_write_profile(
        temp.path(),
        "- id: llm-deepseek\n  name: '@deepseek-ai/dsh-llm-deepseek-api-key'\n  config:\n    apiKeyEnv: MY_DS_KEY\n",
    );
    let mut app = dsh_official_app(temp.path());
    assert_eq!(app.dsh_official_api_key_env, "MY_DS_KEY");

    let (title, action) = open_official_key_dialog(&mut app);
    assert!(title.contains("MY_DS_KEY"), "{title}");
    super::modal::run_input_action(&mut app, action, "sk-custom".to_string()).unwrap();

    let refs = dsh_credentials_refs(temp.path());
    assert_eq!(refs.get("MY_DS_KEY").map(String::as_str), Some("sk-custom"));
    assert!(!refs.contains_key("DEEPSEEK_API_KEY"));
    assert!(app.dsh_official_api_key_configured);
}

#[test]
fn dsh_official_key_empty_value_clears_ref() {
    let temp = TempDir::new().unwrap();
    dsh_config_write_profile(temp.path(), "[]\n");
    droidgear_core::dsh::save_dsh_credential_ref_for_home(
        temp.path(),
        "DEEPSEEK_API_KEY",
        "sk-old",
    )
    .unwrap();
    let mut app = dsh_official_app(temp.path());
    assert!(app.dsh_official_api_key_configured);

    let (_, action) = open_official_key_dialog(&mut app);
    super::modal::run_input_action(&mut app, action, String::new()).unwrap();

    let refs = dsh_credentials_refs(temp.path());
    assert!(!refs.contains_key("DEEPSEEK_API_KEY"), "{refs:?}");
    assert!(
        app.toast_message().contains("Cleared DEEPSEEK_API_KEY"),
        "{}",
        app.toast_message()
    );
    assert!(!app.dsh_official_api_key_configured);
}
