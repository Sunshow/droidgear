use super::*;

pub(super) fn handle_key(app: &mut app::App, code: KeyCode) -> Option<Action> {
    if let Some(modal) = app.modal.clone() {
        handle_modal_key(app, code, modal);
        return None;
    }

    match app.screen {
        app::Screen::Main => handle_main_key(app, code),
        app::Screen::FeatureList => handle_feature_list_key(app, code),
        app::Screen::Paths => handle_paths_key(app, code),
        app::Screen::DroidSettingsFiles => handle_droid_settings_files_key(app, code),
        app::Screen::TrustedFolders => handle_trusted_folders_key(app, code),
        app::Screen::Factory => handle_factory_key(app, code),
        app::Screen::FactoryModel => handle_factory_model_key(app, code),
        app::Screen::Mcp => handle_mcp_key(app, code),
        app::Screen::McpServer => handle_mcp_server_key(app, code),
        app::Screen::McpArgs => handle_mcp_args_key(app, code),
        app::Screen::McpKeyValues => handle_mcp_key_values_key(app, code),
        app::Screen::ClaudeSettings => handle_claude_key(app, code),
        app::Screen::ClaudeSettingsDetail => handle_claude_settings_detail_key(app, code),
        app::Screen::Codex => handle_codex_key(app, code),
        app::Screen::CodexProfile => handle_codex_profile_key(app, code),
        app::Screen::CodexProvider => handle_codex_provider_key(app, code),
        app::Screen::OpenCode => handle_opencode_key(app, code),
        app::Screen::OpenCodeProfile => handle_opencode_profile_key(app, code),
        app::Screen::OpenCodeProvider => handle_opencode_provider_key(app, code),
        app::Screen::OpenCodeModel => handle_opencode_model_key(app, code),
        app::Screen::OpenClaw => handle_openclaw_key(app, code),
        app::Screen::OpenClawProfile => handle_openclaw_profile_key(app, code),
        app::Screen::OpenClawProvider => handle_openclaw_provider_key(app, code),
        app::Screen::OpenClawModel => handle_openclaw_model_key(app, code),
        app::Screen::OpenClawHelpers => handle_openclaw_helpers_key(app, code),
        app::Screen::OpenClawSubagents => handle_openclaw_subagents_key(app, code),
        app::Screen::OpenClawSubagentDetail => handle_openclaw_subagent_detail_key(app, code),
        app::Screen::Pi => handle_pi_key(app, code),
        app::Screen::PiProfile => handle_pi_profile_key(app, code),
        app::Screen::PiProvider => handle_pi_provider_key(app, code),
        app::Screen::PiModel => handle_pi_model_key(app, code),
        app::Screen::Omp => handle_omp_key(app, code),
        app::Screen::OmpProfile => handle_omp_profile_key(app, code),
        app::Screen::Hermes => handle_hermes_key(app, code),
        app::Screen::HermesProfile => handle_hermes_profile_key(app, code),
        app::Screen::HermesProvider => handle_hermes_provider_key(app, code),
        app::Screen::Sessions => handle_sessions_key(app, code),
        app::Screen::Specs => handle_specs_key(app, code),
        app::Screen::Channels => handle_channels_key(app, code),
        app::Screen::ChannelsEdit => handle_channels_edit_key(app, code),
        app::Screen::Missions => handle_missions_key(app, code),
        app::Screen::FactoryAuth => keys_factory_auth::handle_factory_auth_key(app, code),
        app::Screen::CodexAuth => keys_codex_auth::handle_codex_auth_key(app, code),
    }
}

/// Open the module picker for the current screen's position.
fn open_nav_picker(app: &mut app::App) {
    let targets = app::App::nav_targets();
    let options: Vec<String> = targets.iter().map(|(label, _)| label.clone()).collect();
    let index = targets
        .iter()
        .position(|(_, screen)| *screen == app.screen)
        .unwrap_or(0)
        .min(options.len().saturating_sub(1));
    app.modal_filter.clear();
    app.modal = Some(app::Modal::Select {
        title: "Open module".to_string(),
        options,
        index,
        action: app::SelectAction::GoToNav,
    });
}

/// Enter on the selected nav group: multi-item groups open the feature
/// list, single-item groups open their screen directly.
fn open_selected_group(app: &mut app::App) {
    let Some(group) = app::App::nav_groups().get(app.nav_index) else {
        return;
    };
    if group.items.len() > 1 {
        app.screen = app::Screen::FeatureList;
        app.feature_index = 0;
    } else {
        app.screen = group.items[0].1;
        app.clear_toast();
        refresh_screen_data(app);
    }
}

/// Enter on the selected feature inside the current group.
fn open_selected_feature(app: &mut app::App) {
    let Some(group) = app::App::nav_groups().get(app.nav_index) else {
        return;
    };
    let Some((_, screen)) = group.items.get(app.feature_index) else {
        return;
    };
    app.screen = *screen;
    app.clear_toast();
    refresh_screen_data(app);
}

pub(super) fn handle_main_key(app: &mut app::App, code: KeyCode) -> Option<Action> {
    match code {
        KeyCode::Char('q') => {
            app.modal = Some(app::Modal::Confirm {
                message: "Quit DroidGear TUI?".to_string(),
                action: app::ConfirmAction::Quit,
            })
        }
        KeyCode::Char('s') => open_nav_picker(app),
        KeyCode::Down => app.nav_index = app.nav_index.saturating_add(1),
        KeyCode::Up => app.nav_index = app.nav_index.saturating_sub(1),
        KeyCode::Enter => open_selected_group(app),
        _ => {}
    }
    None
}

pub(super) fn handle_feature_list_key(app: &mut app::App, code: KeyCode) -> Option<Action> {
    match code {
        KeyCode::Esc | KeyCode::Char('q') => app.go_back(),
        KeyCode::Char('s') => open_nav_picker(app),
        KeyCode::Down => app.feature_index = app.feature_index.saturating_add(1),
        KeyCode::Up => app.feature_index = app.feature_index.saturating_sub(1),
        KeyCode::Enter => open_selected_feature(app),
        _ => {}
    }
    None
}
