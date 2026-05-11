use super::*;

pub(super) fn handle_key(app: &mut app::App, code: KeyCode) -> Option<Action> {
    if let Some(modal) = app.modal.clone() {
        handle_modal_key(app, code, modal);
        return None;
    }

    match app.screen {
        app::Screen::Main => handle_main_key(app, code),
        app::Screen::Paths => handle_paths_key(app, code),
        app::Screen::DroidSettingsFiles => handle_droid_settings_files_key(app, code),
        app::Screen::Factory => handle_factory_key(app, code),
        app::Screen::FactoryModel => handle_factory_model_key(app, code),
        app::Screen::Mcp => handle_mcp_key(app, code),
        app::Screen::McpServer => handle_mcp_server_key(app, code),
        app::Screen::McpArgs => handle_mcp_args_key(app, code),
        app::Screen::McpKeyValues => handle_mcp_key_values_key(app, code),
        app::Screen::Claude => handle_claude_key(app, code),
        app::Screen::ClaudeProfile => handle_claude_profile_key(app, code),
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
        app::Screen::Hermes => handle_hermes_key(app, code),
        app::Screen::HermesProfile => handle_hermes_profile_key(app, code),
        app::Screen::HermesProvider => handle_hermes_provider_key(app, code),
        app::Screen::Sessions => handle_sessions_key(app, code),
        app::Screen::Specs => handle_specs_key(app, code),
        app::Screen::Channels => handle_channels_key(app, code),
        app::Screen::ChannelsEdit => handle_channels_edit_key(app, code),
        app::Screen::Missions => handle_missions_key(app, code),
        app::Screen::FactoryAuth => keys_factory_auth::handle_factory_auth_key(app, code),
    }
}

pub(super) fn handle_main_key(app: &mut app::App, code: KeyCode) -> Option<Action> {
    match code {
        KeyCode::Char('q') => {
            app.modal = Some(app::Modal::Confirm {
                message: "Quit DroidGear TUI?".to_string(),
                action: app::ConfirmAction::Quit,
            })
        }
        KeyCode::Char('s') => {
            let options: Vec<String> = app::App::nav_items()
                .iter()
                .map(|(label, _)| (*label).to_string())
                .collect();
            let index = app.nav_index.min(options.len().saturating_sub(1));
            app.modal = Some(app::Modal::Select {
                title: "Open module".to_string(),
                options,
                index,
                action: app::SelectAction::GoToNav,
            });
        }
        KeyCode::Down => app.nav_index = app.nav_index.saturating_add(1),
        KeyCode::Up => app.nav_index = app.nav_index.saturating_sub(1),
        KeyCode::Enter => {
            if let Some((_, screen)) = app::App::nav_items().get(app.nav_index) {
                app.screen = *screen;
                app.clear_toast();
                refresh_screen_data(app);
            }
        }
        _ => {}
    }
    None
}
