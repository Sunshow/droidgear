#![allow(clippy::question_mark)]

use crate::{app, editor, ui};
use anyhow::Context;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};
use serde::{de::DeserializeOwned, Serialize};
use similar::TextDiff;
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::time::Duration;
use tempfile::{NamedTempFile, TempDir};

type UiTerminal = Terminal<CrosstermBackend<io::Stdout>>;

struct TerminalGuard;

impl TerminalGuard {
    fn enter() -> anyhow::Result<Self> {
        enable_raw_mode().context("enable raw mode")?;
        let mut stdout = io::stdout();
        execute!(stdout, EnterAlternateScreen, EnableMouseCapture).context("enter alt screen")?;
        Ok(Self)
    }
}

impl Drop for TerminalGuard {
    fn drop(&mut self) {
        let _ = disable_raw_mode();
        let mut stdout = io::stdout();
        let _ = execute!(stdout, LeaveAlternateScreen, DisableMouseCapture);
    }
}

#[derive(Debug, Clone)]
enum Action {
    EditFactoryModels,
    EditCodexProfile { id: String },
    EditOpenCodeProfile { id: String },
    EditOpenClawProfile { id: String },
    PreviewCodexApply { id: String },
    PreviewOpenCodeApply { id: String },
    PreviewOpenClawApply { id: String },
    ViewSession { path: String },
    EditSpec { path: String },
    EditChannels,
    EditChannelAuth { id: String },
}

pub fn run(app: &mut app::App) -> anyhow::Result<()> {
    let _guard = TerminalGuard::enter()?;
    let backend = CrosstermBackend::new(io::stdout());
    let mut terminal = Terminal::new(backend).context("create terminal")?;

    refresh_screen_data(app);

    while !app.should_quit {
        app.clamp_indices();
        terminal.draw(|f| ui::draw(f, app)).context("draw")?;

        if event::poll(Duration::from_millis(200)).context("poll event")? {
            if let Event::Key(key) = event::read().context("read event")? {
                if key.kind == KeyEventKind::Press {
                    if let Some(action) = handle_key(app, key.code) {
                        if let Err(e) = run_action_with_terminal(&mut terminal, app, action) {
                            app.set_toast(e.to_string(), true);
                        }
                        refresh_screen_data(app);
                    }
                }
            }
        }
    }

    Ok(())
}

fn run_action_with_terminal(
    terminal: &mut UiTerminal,
    app: &mut app::App,
    action: Action,
) -> anyhow::Result<()> {
    suspend_terminal(terminal)?;
    let result = run_action(app, action);
    let resume_result = resume_terminal(terminal);

    resume_result?;
    result
}

fn suspend_terminal(terminal: &mut UiTerminal) -> anyhow::Result<()> {
    disable_raw_mode().context("disable raw mode")?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )
    .context("leave alt screen")?;
    Ok(())
}

fn resume_terminal(terminal: &mut UiTerminal) -> anyhow::Result<()> {
    execute!(
        terminal.backend_mut(),
        EnterAlternateScreen,
        EnableMouseCapture
    )
    .context("enter alt screen")?;
    enable_raw_mode().context("enable raw mode")?;
    terminal.clear().context("clear terminal")?;
    Ok(())
}

fn refresh_screen_data(app: &mut app::App) {
    match app.screen {
        app::Screen::Main => {}
        app::Screen::Paths => refresh_paths(app),
        app::Screen::Factory => refresh_factory(app),
        app::Screen::FactoryModel => {}
        app::Screen::Mcp => refresh_mcp(app),
        app::Screen::McpServer | app::Screen::McpArgs | app::Screen::McpKeyValues => {}
        app::Screen::Codex => refresh_codex(app),
        app::Screen::CodexProfile | app::Screen::CodexProvider => {
            refresh_codex(app);
            refresh_codex_detail(app);
        }
        app::Screen::OpenCode => refresh_opencode(app),
        app::Screen::OpenCodeProfile
        | app::Screen::OpenCodeProvider
        | app::Screen::OpenCodeModel => {
            refresh_opencode(app);
            refresh_opencode_detail(app);
        }
        app::Screen::OpenClaw => refresh_openclaw(app),
        app::Screen::OpenClawProfile
        | app::Screen::OpenClawProvider
        | app::Screen::OpenClawModel
        | app::Screen::OpenClawHelpers => {
            refresh_openclaw(app);
            refresh_openclaw_detail(app);
        }
        app::Screen::OpenClawSubagents | app::Screen::OpenClawSubagentDetail => {
            refresh_openclaw_subagents(app);
        }
        app::Screen::Hermes => refresh_hermes(app),
        app::Screen::HermesProfile | app::Screen::HermesProvider => {
            refresh_hermes(app);
            refresh_hermes_detail(app);
        }
        app::Screen::Sessions => refresh_sessions(app),
        app::Screen::Specs => refresh_specs(app),
        app::Screen::Channels => refresh_channels(app),
        app::Screen::ChannelsEdit => {}
        app::Screen::Missions => refresh_missions(app),
    }
}

fn refresh_paths(app: &mut app::App) {
    match droidgear_core::paths::get_effective_paths_for_home(&app.home_dir) {
        Ok(p) => app.paths = Some(p),
        Err(e) => app.set_toast(e, true),
    }
}

fn refresh_factory(app: &mut app::App) {
    match droidgear_core::factory_settings::load_custom_models_for_home(&app.home_dir) {
        Ok(models) => app.custom_models = models,
        Err(e) => app.set_toast(e, true),
    }
    match droidgear_core::factory_settings::get_default_model_for_home(&app.home_dir) {
        Ok(id) => app.factory_default_model_id = id,
        Err(e) => app.set_toast(e, true),
    }
}

fn refresh_mcp(app: &mut app::App) {
    match droidgear_core::mcp::load_mcp_servers_for_home(&app.home_dir) {
        Ok(servers) => app.mcp_servers = servers,
        Err(e) => app.set_toast(e, true),
    }
}

fn refresh_codex(app: &mut app::App) {
    match droidgear_core::codex::list_codex_profiles_for_home(&app.home_dir) {
        Ok(list) => app.codex_profiles = list,
        Err(e) => app.set_toast(e, true),
    }

    let has_user_profiles = app.codex_profiles.iter().any(|p| p.id != "official");
    if !has_user_profiles
        && droidgear_core::codex::create_default_codex_profile_for_home(&app.home_dir).is_ok()
    {
        if let Ok(list) = droidgear_core::codex::list_codex_profiles_for_home(&app.home_dir) {
            app.codex_profiles = list;
        }
    }

    match droidgear_core::codex::get_active_codex_profile_id_for_home(&app.home_dir) {
        Ok(id) => app.codex_active_id = id,
        Err(e) => app.set_toast(e, true),
    }
}

fn refresh_codex_detail(app: &mut app::App) {
    let Some(id) = app.codex_detail_id.clone() else {
        app.codex_detail = None;
        app.codex_detail_provider_ids.clear();
        return;
    };
    match droidgear_core::codex::get_codex_profile_for_home(&app.home_dir, &id) {
        Ok(profile) => {
            app.codex_detail_provider_ids =
                profile.providers.keys().cloned().collect::<Vec<String>>();
            app.codex_detail_provider_ids
                .sort_by_key(|a| a.to_lowercase());
            app.codex_detail = Some(profile);
        }
        Err(e) => {
            app.codex_detail = None;
            app.codex_detail_provider_ids.clear();
            app.set_toast(e, true);
        }
    }
}

fn codex_set_active_provider(
    app: &mut app::App,
    profile_id: &str,
    provider_id: &str,
) -> anyhow::Result<()> {
    let mut profile = droidgear_core::codex::get_codex_profile_for_home(&app.home_dir, profile_id)
        .map_err(anyhow::Error::msg)?;
    profile.model_provider = provider_id.to_string();
    droidgear_core::codex::save_codex_profile_for_home(&app.home_dir, profile)
        .map_err(anyhow::Error::msg)?;
    Ok(())
}

fn codex_load_from_live_config(app: &mut app::App, profile_id: &str) -> anyhow::Result<()> {
    let live = droidgear_core::codex::read_codex_current_config_for_home(&app.home_dir)
        .map_err(anyhow::Error::msg)?;
    let mut profile = droidgear_core::codex::get_codex_profile_for_home(&app.home_dir, profile_id)
        .map_err(anyhow::Error::msg)?;
    profile.providers = live.providers;
    profile.model_provider = live.model_provider;
    profile.model = live.model;
    profile.model_reasoning_effort = live.model_reasoning_effort;
    profile.api_key = live.api_key;
    droidgear_core::codex::save_codex_profile_for_home(&app.home_dir, profile)
        .map_err(anyhow::Error::msg)?;
    Ok(())
}

fn refresh_opencode(app: &mut app::App) {
    match droidgear_core::opencode::list_opencode_profiles_for_home(&app.home_dir) {
        Ok(list) => app.opencode_profiles = list,
        Err(e) => app.set_toast(e, true),
    }

    if app.opencode_profiles.is_empty() {
        if let Ok(p) = droidgear_core::opencode::create_default_profile_for_home(&app.home_dir) {
            app.opencode_profiles = vec![p]
        }
    }

    match droidgear_core::opencode::get_active_opencode_profile_id_for_home(&app.home_dir) {
        Ok(id) => app.opencode_active_id = id,
        Err(e) => app.set_toast(e, true),
    }
}

fn refresh_opencode_detail(app: &mut app::App) {
    let Some(id) = app.opencode_detail_id.clone() else {
        app.opencode_detail = None;
        app.opencode_detail_provider_ids.clear();
        app.opencode_provider_model_ids.clear();
        return;
    };

    match droidgear_core::opencode::get_opencode_profile_for_home(&app.home_dir, &id) {
        Ok(profile) => {
            app.opencode_detail_provider_ids =
                profile.providers.keys().cloned().collect::<Vec<String>>();
            app.opencode_detail_provider_ids
                .sort_by_key(|a| a.to_lowercase());

            if let Some(provider_id) = app.opencode_provider_id.as_deref() {
                app.opencode_provider_model_ids = profile
                    .providers
                    .get(provider_id)
                    .and_then(|p| p.models.as_ref())
                    .map(|m| {
                        let mut ids = m.keys().cloned().collect::<Vec<String>>();
                        ids.sort_by_key(|a| a.to_lowercase());
                        ids
                    })
                    .unwrap_or_default();
            } else {
                app.opencode_provider_model_ids.clear();
            }

            app.opencode_detail = Some(profile);
        }
        Err(e) => {
            app.opencode_detail = None;
            app.opencode_detail_provider_ids.clear();
            app.opencode_provider_model_ids.clear();
            app.set_toast(e, true);
        }
    }
}

fn refresh_openclaw(app: &mut app::App) {
    match droidgear_core::openclaw::list_openclaw_profiles_for_home(&app.home_dir) {
        Ok(list) => app.openclaw_profiles = list,
        Err(e) => app.set_toast(e, true),
    }

    if app.openclaw_profiles.is_empty() {
        if let Ok(p) =
            droidgear_core::openclaw::create_default_openclaw_profile_for_home(&app.home_dir)
        {
            app.openclaw_profiles = vec![p]
        }
    }

    match droidgear_core::openclaw::get_active_openclaw_profile_id_for_home(&app.home_dir) {
        Ok(id) => app.openclaw_active_id = id,
        Err(e) => app.set_toast(e, true),
    }
}

fn refresh_openclaw_detail(app: &mut app::App) {
    let Some(id) = app.openclaw_detail_id.clone() else {
        app.openclaw_detail = None;
        app.openclaw_detail_provider_ids.clear();
        return;
    };
    match droidgear_core::openclaw::get_openclaw_profile_for_home(&app.home_dir, &id) {
        Ok(profile) => {
            app.openclaw_detail_provider_ids =
                profile.providers.keys().cloned().collect::<Vec<String>>();
            app.openclaw_detail_provider_ids
                .sort_by_key(|a| a.to_lowercase());
            app.openclaw_detail = Some(profile);
        }
        Err(e) => {
            app.openclaw_detail = None;
            app.openclaw_detail_provider_ids.clear();
            app.set_toast(e, true);
        }
    }
}

fn refresh_openclaw_subagents(app: &mut app::App) {
    match droidgear_core::openclaw::read_openclaw_subagents_for_home(&app.home_dir) {
        Ok(list) => app.openclaw_subagents = list,
        Err(e) => app.set_toast(e, true),
    }
}

fn refresh_sessions(app: &mut app::App) {
    match droidgear_core::sessions::list_sessions_for_home(&app.home_dir, None) {
        Ok(list) => app.sessions = list,
        Err(e) => app.set_toast(e, true),
    }
}

fn refresh_specs(app: &mut app::App) {
    match droidgear_core::specs::list_specs_for_home(&app.home_dir) {
        Ok(list) => app.specs = list,
        Err(e) => app.set_toast(e, true),
    }
}

fn refresh_channels(app: &mut app::App) {
    match droidgear_core::channel::load_channels_for_home(&app.home_dir) {
        Ok(list) => app.channels = list,
        Err(e) => app.set_toast(e, true),
    }
}

fn refresh_missions(app: &mut app::App) {
    match droidgear_core::factory_settings::get_mission_model_settings_for_home(&app.home_dir) {
        Ok(settings) => app.mission_settings = settings,
        Err(e) => app.set_toast(e, true),
    }
    // Also load custom models for model selection
    match droidgear_core::factory_settings::load_custom_models_for_home(&app.home_dir) {
        Ok(models) => app.custom_models = models,
        Err(e) => app.set_toast(e, true),
    }
}

fn refresh_hermes(app: &mut app::App) {
    match droidgear_core::hermes::list_hermes_profiles_for_home(&app.home_dir) {
        Ok(list) => app.hermes_profiles = list,
        Err(e) => app.set_toast(e, true),
    }

    if app.hermes_profiles.is_empty() {
        if let Ok(p) = droidgear_core::hermes::create_default_hermes_profile_for_home(&app.home_dir)
        {
            app.hermes_profiles = vec![p];
        }
    }

    match droidgear_core::hermes::get_active_hermes_profile_id_for_home(&app.home_dir) {
        Ok(id) => app.hermes_active_id = id,
        Err(e) => app.set_toast(e, true),
    }
}

fn refresh_hermes_detail(app: &mut app::App) {
    let Some(id) = app.hermes_detail_id.clone() else {
        app.hermes_detail = None;
        return;
    };
    match droidgear_core::hermes::get_hermes_profile_for_home(&app.home_dir, &id) {
        Ok(profile) => {
            app.hermes_detail = Some(profile);
        }
        Err(e) => {
            app.hermes_detail = None;
            app.set_toast(e, true);
        }
    }
}

fn hermes_load_from_live_config(app: &mut app::App, profile_id: &str) -> anyhow::Result<()> {
    let live = droidgear_core::hermes::read_hermes_current_config_for_home(&app.home_dir)
        .map_err(anyhow::Error::msg)?;
    let mut profile =
        droidgear_core::hermes::get_hermes_profile_for_home(&app.home_dir, profile_id)
            .map_err(anyhow::Error::msg)?;
    profile.model = live.model;
    droidgear_core::hermes::save_hermes_profile_for_home(&app.home_dir, profile)
        .map_err(anyhow::Error::msg)?;
    Ok(())
}

fn handle_key(app: &mut app::App, code: KeyCode) -> Option<Action> {
    if let Some(modal) = app.modal.clone() {
        handle_modal_key(app, code, modal);
        return None;
    }

    match app.screen {
        app::Screen::Main => handle_main_key(app, code),
        app::Screen::Paths => handle_paths_key(app, code),
        app::Screen::Factory => handle_factory_key(app, code),
        app::Screen::FactoryModel => handle_factory_model_key(app, code),
        app::Screen::Mcp => handle_mcp_key(app, code),
        app::Screen::McpServer => handle_mcp_server_key(app, code),
        app::Screen::McpArgs => handle_mcp_args_key(app, code),
        app::Screen::McpKeyValues => handle_mcp_key_values_key(app, code),
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
        app::Screen::Hermes => handle_hermes_key(app, code),
        app::Screen::HermesProfile => handle_hermes_profile_key(app, code),
        app::Screen::HermesProvider => handle_hermes_provider_key(app, code),
        app::Screen::Sessions => handle_sessions_key(app, code),
        app::Screen::Specs => handle_specs_key(app, code),
        app::Screen::Channels => handle_channels_key(app, code),
        app::Screen::ChannelsEdit => handle_channels_edit_key(app, code),
        app::Screen::Missions => handle_missions_key(app, code),
    }
}

fn handle_main_key(app: &mut app::App, code: KeyCode) -> Option<Action> {
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

fn handle_paths_key(app: &mut app::App, code: KeyCode) -> Option<Action> {
    match code {
        KeyCode::Esc | KeyCode::Char('q') => app.screen = app::Screen::Main,
        KeyCode::Down => app.paths_index = app.paths_index.saturating_add(1),
        KeyCode::Up => app.paths_index = app.paths_index.saturating_sub(1),
        KeyCode::Char('r') => refresh_paths(app),
        KeyCode::Char('e') | KeyCode::Enter => {
            let Some(key) = app.current_paths_key() else {
                return None;
            };
            let current = app
                .current_paths_entry()
                .map(|p| p.path.clone())
                .unwrap_or_default();
            app.modal = Some(app::Modal::Input {
                title: format!("Set path for {key}"),
                value: current,
                cursor: usize::MAX,
                is_secret: false,
                action: app::InputAction::PathsSetKey { key },
            });
        }
        KeyCode::Char('x') => {
            let Some(key) = app.current_paths_key() else {
                return None;
            };
            app.modal = Some(app::Modal::Confirm {
                message: format!("Reset path override for {key}?"),
                action: app::ConfirmAction::PathsResetKey { key },
            });
        }
        _ => {}
    }
    None
}

fn handle_factory_key(app: &mut app::App, code: KeyCode) -> Option<Action> {
    match code {
        KeyCode::Esc | KeyCode::Char('q') => app.screen = app::Screen::Main,
        KeyCode::Down => app.factory_models_index = app.factory_models_index.saturating_add(1),
        KeyCode::Up => app.factory_models_index = app.factory_models_index.saturating_sub(1),
        KeyCode::Char('r') => refresh_factory(app),
        KeyCode::Char('E') => return Some(Action::EditFactoryModels),
        KeyCode::Char('n') => {
            app.factory_edit_index = None;
            app.factory_model_field_index = 0;
            app.factory_draft = Some(droidgear_core::factory_settings::CustomModel {
                model: String::new(),
                id: None,
                index: None,
                display_name: None,
                base_url: String::new(),
                api_key: String::new(),
                provider: droidgear_core::factory_settings::Provider::Openai,
                max_output_tokens: None,
                no_image_support: None,
                extra_args: None,
                extra_headers: None,
            });
            app.screen = app::Screen::FactoryModel;
        }
        KeyCode::Char('c') => {
            if let Some(m) = app.custom_models.get(app.factory_models_index) {
                let mut copy = m.clone();
                copy.id = None;
                copy.index = None;
                app.factory_edit_index = None;
                app.factory_model_field_index = 0;
                app.factory_draft = Some(copy);
                app.screen = app::Screen::FactoryModel;
            }
        }
        KeyCode::Char('x') if !app.custom_models.is_empty() => {
            app.modal = Some(app::Modal::Confirm {
                message: "Delete selected custom model?".to_string(),
                action: app::ConfirmAction::FactoryDeleteModel {
                    index: app.factory_models_index,
                },
            });
        }
        KeyCode::Enter | KeyCode::Char('e') => {
            if let Some(m) = app.custom_models.get(app.factory_models_index) {
                app.factory_edit_index = Some(app.factory_models_index);
                app.factory_model_field_index = 0;
                app.factory_draft = Some(m.clone());
                app.screen = app::Screen::FactoryModel;
            }
        }
        KeyCode::Char('d') => {
            if let Some(model_id) = factory_model_id(
                app.custom_models.get(app.factory_models_index),
                app.factory_models_index,
            ) {
                app.modal = Some(app::Modal::Confirm {
                    message: format!("Set Factory default model to {model_id}?"),
                    action: app::ConfirmAction::FactorySetDefaultModel { model_id },
                });
            }
        }
        _ => {}
    }
    None
}

fn normalize_factory_models(models: &mut [droidgear_core::factory_settings::CustomModel]) {
    for (idx, m) in models.iter_mut().enumerate() {
        m.index = Some(idx as u32);
        let display = m
            .display_name
            .clone()
            .filter(|s| !s.trim().is_empty())
            .unwrap_or_else(|| m.model.clone());
        m.id = Some(format!("custom:{display}-{idx}"));
    }
}

fn handle_factory_model_key(app: &mut app::App, code: KeyCode) -> Option<Action> {
    let Some(draft) = app.factory_draft.as_ref() else {
        app.screen = app::Screen::Factory;
        return None;
    };

    match code {
        KeyCode::Esc | KeyCode::Char('q') => {
            app.factory_draft = None;
            app.factory_edit_index = None;
            app.screen = app::Screen::Factory;
        }
        KeyCode::Down => {
            app.factory_model_field_index = app.factory_model_field_index.saturating_add(1)
        }
        KeyCode::Up => {
            app.factory_model_field_index = app.factory_model_field_index.saturating_sub(1)
        }
        KeyCode::Char('s') => {
            let Some(mut draft) = app.factory_draft.clone() else {
                return None;
            };
            draft.model = draft.model.trim().to_string();
            draft.base_url = draft.base_url.trim().to_string();
            draft.api_key = draft.api_key.trim().to_string();

            if draft.model.is_empty() {
                app.set_toast("Model id is required", true);
                return None;
            }
            if draft.base_url.is_empty() {
                app.set_toast("Base URL is required", true);
                return None;
            }
            if draft.api_key.is_empty() {
                app.set_toast("API key is required", true);
                return None;
            }

            let mut models = app.custom_models.clone();
            let saved_index = if let Some(edit_index) = app.factory_edit_index {
                if edit_index < models.len() {
                    models[edit_index] = draft;
                    edit_index
                } else {
                    models.push(draft);
                    models.len().saturating_sub(1)
                }
            } else {
                models.push(draft);
                models.len().saturating_sub(1)
            };

            normalize_factory_models(&mut models);

            if let Err(e) =
                droidgear_core::factory_settings::save_custom_models_for_home(&app.home_dir, models)
                    .map_err(anyhow::Error::msg)
            {
                app.set_toast(e.to_string(), true);
                return None;
            }

            app.factory_models_index = saved_index;
            app.factory_draft = None;
            app.factory_edit_index = None;
            app.screen = app::Screen::Factory;
            refresh_factory(app);
            app.set_toast("Saved", false);
        }
        KeyCode::Enter | KeyCode::Char('e') => match app.factory_model_field_index {
            0 => {
                let options = vec![
                    "anthropic".to_string(),
                    "openai".to_string(),
                    "generic-chat-completion-api".to_string(),
                ];
                let current = match draft.provider {
                    droidgear_core::factory_settings::Provider::Anthropic => "anthropic",
                    droidgear_core::factory_settings::Provider::Openai => "openai",
                    droidgear_core::factory_settings::Provider::GenericChatCompletionApi => {
                        "generic-chat-completion-api"
                    }
                };
                let index = options.iter().position(|o| o == current).unwrap_or(0);
                app.modal = Some(app::Modal::Select {
                    title: "Provider".to_string(),
                    options,
                    index,
                    action: app::SelectAction::FactoryDraftSetProvider,
                });
            }
            1 => {
                app.modal = Some(app::Modal::Input {
                    title: "Base URL".to_string(),
                    value: draft.base_url.clone(),
                    cursor: usize::MAX,
                    is_secret: false,
                    action: app::InputAction::FactoryDraftSetBaseUrl,
                });
            }
            2 => {
                app.modal = Some(app::Modal::Input {
                    title: "API key".to_string(),
                    value: draft.api_key.clone(),
                    cursor: usize::MAX,
                    is_secret: true,
                    action: app::InputAction::FactoryDraftSetApiKey,
                });
            }
            3 => {
                app.modal = Some(app::Modal::Input {
                    title: "Model id".to_string(),
                    value: draft.model.clone(),
                    cursor: usize::MAX,
                    is_secret: false,
                    action: app::InputAction::FactoryDraftSetModel,
                });
            }
            4 => {
                app.modal = Some(app::Modal::Input {
                    title: "Display name".to_string(),
                    value: draft.display_name.clone().unwrap_or_default(),
                    cursor: usize::MAX,
                    is_secret: false,
                    action: app::InputAction::FactoryDraftSetDisplayName,
                });
            }
            5 => {
                app.modal = Some(app::Modal::Input {
                    title: "Max output tokens".to_string(),
                    value: draft
                        .max_output_tokens
                        .map(|v| v.to_string())
                        .unwrap_or_default(),
                    cursor: usize::MAX,
                    is_secret: false,
                    action: app::InputAction::FactoryDraftSetMaxOutputTokens,
                });
            }
            6 => {
                if let Some(draft) = app.factory_draft.as_mut() {
                    let current = draft.no_image_support.unwrap_or(false);
                    draft.no_image_support = if !current { Some(true) } else { None };
                }
            }
            7 => {
                let current = app
                    .factory_draft
                    .as_ref()
                    .and_then(|d| d.extra_args.as_ref())
                    .and_then(|m| m.get("reasoning"))
                    .and_then(|v| v.as_object())
                    .and_then(|obj| obj.get("effort"))
                    .and_then(|v| v.as_str())
                    .unwrap_or("none")
                    .to_string();
                let options = vec![
                    "none".to_string(),
                    "low".to_string(),
                    "medium".to_string(),
                    "high".to_string(),
                    "xhigh".to_string(),
                ];
                let index = options.iter().position(|o| o == &current).unwrap_or(0);
                app.modal = Some(app::Modal::Select {
                    title: "Reasoning Effort".to_string(),
                    options,
                    index,
                    action: app::SelectAction::FactoryDraftSetReasoningEffort,
                });
            }
            8 => {
                let current = app
                    .factory_draft
                    .as_ref()
                    .and_then(|d| d.extra_args.as_ref())
                    .map(|m| serde_json::to_string_pretty(m).unwrap_or_default())
                    .unwrap_or_default();
                app.modal = Some(app::Modal::Input {
                    title: "Extra Args (JSON object)".to_string(),
                    value: current,
                    cursor: usize::MAX,
                    is_secret: false,
                    action: app::InputAction::FactoryDraftSetExtraArgs,
                });
            }
            9 => {
                let current = app
                    .factory_draft
                    .as_ref()
                    .and_then(|d| d.extra_headers.as_ref())
                    .map(|m| serde_json::to_string_pretty(m).unwrap_or_default())
                    .unwrap_or_default();
                app.modal = Some(app::Modal::Input {
                    title: "Extra Headers (JSON object)".to_string(),
                    value: current,
                    cursor: usize::MAX,
                    is_secret: false,
                    action: app::InputAction::FactoryDraftSetExtraHeaders,
                });
            }
            _ => {}
        },
        _ => {}
    }

    None
}

fn handle_mcp_key(app: &mut app::App, code: KeyCode) -> Option<Action> {
    match code {
        KeyCode::Esc | KeyCode::Char('q') => app.screen = app::Screen::Main,
        KeyCode::Down => app.mcp_index = app.mcp_index.saturating_add(1),
        KeyCode::Up => app.mcp_index = app.mcp_index.saturating_sub(1),
        KeyCode::Char('r') => refresh_mcp(app),
        KeyCode::Char('n') => {
            app.modal = Some(app::Modal::Input {
                title: "New MCP server name".to_string(),
                value: String::new(),
                cursor: usize::MAX,
                is_secret: false,
                action: app::InputAction::McpCreateServer,
            });
        }
        KeyCode::Enter | KeyCode::Char('e') => {
            if let Some(server) = app.mcp_servers.get(app.mcp_index) {
                app.mcp_edit_original_name = Some(server.name.clone());
                app.mcp_edit_draft = Some(server.clone());
                app.mcp_edit_field_index = 0;
                app.mcp_args_index = 0;
                app.mcp_kv_index = 0;
                app.screen = app::Screen::McpServer;
            }
        }
        KeyCode::Char('t') => {
            if let Some(server) = app.mcp_servers.get(app.mcp_index) {
                app.modal = Some(app::Modal::Confirm {
                    message: format!(
                        "Toggle MCP server '{}' to {}?",
                        server.name,
                        if server.config.disabled {
                            "enabled"
                        } else {
                            "disabled"
                        }
                    ),
                    action: app::ConfirmAction::McpToggle {
                        name: server.name.clone(),
                        disabled: !server.config.disabled,
                    },
                });
            }
        }
        KeyCode::Char('d') => {
            if let Some(server) = app.mcp_servers.get(app.mcp_index) {
                app.modal = Some(app::Modal::Confirm {
                    message: format!("Delete MCP server '{}'?", server.name),
                    action: app::ConfirmAction::McpDelete {
                        name: server.name.clone(),
                    },
                });
            }
        }
        _ => {}
    }
    None
}

fn handle_mcp_server_key(app: &mut app::App, code: KeyCode) -> Option<Action> {
    let Some(draft) = app.mcp_edit_draft.as_ref() else {
        app.screen = app::Screen::Mcp;
        return None;
    };
    let server_type = draft.config.server_type.clone();

    match code {
        KeyCode::Esc | KeyCode::Char('q') => {
            app.mcp_edit_draft = None;
            app.mcp_edit_original_name = None;
            app.screen = app::Screen::Mcp;
        }
        KeyCode::Down => app.mcp_edit_field_index = app.mcp_edit_field_index.saturating_add(1),
        KeyCode::Up => app.mcp_edit_field_index = app.mcp_edit_field_index.saturating_sub(1),
        KeyCode::Char('s') => {
            let Some(mut server) = app.mcp_edit_draft.clone() else {
                return None;
            };
            server.name = server.name.trim().to_string();
            if server.name.is_empty() {
                app.set_toast("Name is required", true);
                return None;
            }

            match server.config.server_type {
                droidgear_core::mcp::McpServerType::Stdio => {
                    server.config.url = None;
                    server.config.headers = None;

                    server.config.command = server
                        .config
                        .command
                        .map(|s| s.trim().to_string())
                        .filter(|s| !s.is_empty());
                    if server.config.command.is_none() {
                        app.set_toast("Command is required", true);
                        return None;
                    }

                    if let Some(mut args) = server.config.args.take() {
                        for a in args.iter_mut() {
                            *a = a.trim().to_string();
                        }
                        args.retain(|a| !a.is_empty());
                        server.config.args = (!args.is_empty()).then_some(args);
                    }

                    if let Some(env) = server.config.env.as_mut() {
                        let mut cleaned = std::collections::HashMap::new();
                        for (k, v) in env.drain() {
                            let key = k.trim().to_string();
                            if key.is_empty() {
                                continue;
                            }
                            cleaned.insert(key, v.trim().to_string());
                        }
                        server.config.env = (!cleaned.is_empty()).then_some(cleaned);
                    }
                }
                droidgear_core::mcp::McpServerType::Http => {
                    server.config.command = None;
                    server.config.args = None;
                    server.config.env = None;

                    server.config.url = server
                        .config
                        .url
                        .map(|s| s.trim().to_string())
                        .filter(|s| !s.is_empty());
                    if server.config.url.is_none() {
                        app.set_toast("URL is required", true);
                        return None;
                    }

                    if let Some(headers) = server.config.headers.as_mut() {
                        let mut cleaned = std::collections::HashMap::new();
                        for (k, v) in headers.drain() {
                            let key = k.trim().to_string();
                            if key.is_empty() {
                                continue;
                            }
                            cleaned.insert(key, v.trim().to_string());
                        }
                        server.config.headers = (!cleaned.is_empty()).then_some(cleaned);
                    }
                }
            }

            if let Some(original) = app.mcp_edit_original_name.as_deref() {
                if original != server.name {
                    if let Err(e) =
                        droidgear_core::mcp::delete_mcp_server_for_home(&app.home_dir, original)
                    {
                        app.set_toast(e, true);
                        return None;
                    }
                }
            }

            if let Err(e) =
                droidgear_core::mcp::save_mcp_server_for_home(&app.home_dir, server.clone())
            {
                app.set_toast(e, true);
                return None;
            }

            app.mcp_edit_draft = None;
            app.mcp_edit_original_name = None;
            app.screen = app::Screen::Mcp;
            refresh_mcp(app);
            if let Some(idx) = app.mcp_servers.iter().position(|s| s.name == server.name) {
                app.mcp_index = idx;
            }
            app.set_toast("Saved", false);
        }
        KeyCode::Enter | KeyCode::Char('e') => match server_type {
            droidgear_core::mcp::McpServerType::Stdio => match app.mcp_edit_field_index {
                0 => {
                    app.modal = Some(app::Modal::Input {
                        title: "Server name".to_string(),
                        value: draft.name.clone(),
                        cursor: usize::MAX,
                        is_secret: false,
                        action: app::InputAction::McpDraftSetName,
                    });
                }
                1 => {
                    let options = vec!["stdio".to_string(), "http".to_string()];
                    let index = match draft.config.server_type {
                        droidgear_core::mcp::McpServerType::Stdio => 0,
                        droidgear_core::mcp::McpServerType::Http => 1,
                    };
                    app.modal = Some(app::Modal::Select {
                        title: "Server type".to_string(),
                        options,
                        index,
                        action: app::SelectAction::McpDraftSetType,
                    });
                }
                2 => {
                    if let Some(server) = app.mcp_edit_draft.as_mut() {
                        server.config.disabled = !server.config.disabled;
                    }
                }
                3 => {
                    app.modal = Some(app::Modal::Input {
                        title: "Command".to_string(),
                        value: draft.config.command.clone().unwrap_or_default(),
                        cursor: usize::MAX,
                        is_secret: false,
                        action: app::InputAction::McpDraftSetCommand,
                    });
                }
                4 => {
                    app.mcp_args_index = 0;
                    app.screen = app::Screen::McpArgs;
                }
                5 => {
                    app.mcp_kv_mode = app::McpKeyValuesMode::Env;
                    app.mcp_kv_index = 0;
                    app.screen = app::Screen::McpKeyValues;
                }
                _ => {}
            },
            droidgear_core::mcp::McpServerType::Http => match app.mcp_edit_field_index {
                0 => {
                    app.modal = Some(app::Modal::Input {
                        title: "Server name".to_string(),
                        value: draft.name.clone(),
                        cursor: usize::MAX,
                        is_secret: false,
                        action: app::InputAction::McpDraftSetName,
                    });
                }
                1 => {
                    let options = vec!["stdio".to_string(), "http".to_string()];
                    let index = match draft.config.server_type {
                        droidgear_core::mcp::McpServerType::Stdio => 0,
                        droidgear_core::mcp::McpServerType::Http => 1,
                    };
                    app.modal = Some(app::Modal::Select {
                        title: "Server type".to_string(),
                        options,
                        index,
                        action: app::SelectAction::McpDraftSetType,
                    });
                }
                2 => {
                    if let Some(server) = app.mcp_edit_draft.as_mut() {
                        server.config.disabled = !server.config.disabled;
                    }
                }
                3 => {
                    app.modal = Some(app::Modal::Input {
                        title: "URL".to_string(),
                        value: draft.config.url.clone().unwrap_or_default(),
                        cursor: usize::MAX,
                        is_secret: false,
                        action: app::InputAction::McpDraftSetUrl,
                    });
                }
                4 => {
                    app.mcp_kv_mode = app::McpKeyValuesMode::Headers;
                    app.mcp_kv_index = 0;
                    app.screen = app::Screen::McpKeyValues;
                }
                _ => {}
            },
        },
        _ => {}
    }

    None
}

fn handle_mcp_args_key(app: &mut app::App, code: KeyCode) -> Option<Action> {
    match code {
        KeyCode::Esc | KeyCode::Char('q') => app.screen = app::Screen::McpServer,
        KeyCode::Down => app.mcp_args_index = app.mcp_args_index.saturating_add(1),
        KeyCode::Up => app.mcp_args_index = app.mcp_args_index.saturating_sub(1),
        KeyCode::Char('n') => {
            app.modal = Some(app::Modal::Input {
                title: "New arg".to_string(),
                value: String::new(),
                cursor: usize::MAX,
                is_secret: false,
                action: app::InputAction::McpArgsAdd,
            });
        }
        KeyCode::Enter | KeyCode::Char('e') => {
            let current = app
                .mcp_edit_draft
                .as_ref()
                .and_then(|s| s.config.args.as_ref())
                .and_then(|v| v.get(app.mcp_args_index))
                .cloned();
            if let Some(current) = current {
                app.modal = Some(app::Modal::Input {
                    title: "Arg".to_string(),
                    value: current,
                    cursor: usize::MAX,
                    is_secret: false,
                    action: app::InputAction::McpArgsEdit {
                        index: app.mcp_args_index,
                    },
                });
            }
        }
        KeyCode::Char('x') => {
            if let Some(server) = app.mcp_edit_draft.as_mut() {
                if let Some(args) = server.config.args.as_mut() {
                    if app.mcp_args_index < args.len() {
                        args.remove(app.mcp_args_index);
                    }
                    if args.is_empty() {
                        server.config.args = None;
                        app.mcp_args_index = 0;
                    }
                }
            }
        }
        _ => {}
    }
    None
}

fn handle_mcp_key_values_key(app: &mut app::App, code: KeyCode) -> Option<Action> {
    let Some(server) = app.mcp_edit_draft.as_ref() else {
        app.screen = app::Screen::Mcp;
        return None;
    };
    let mode = app.mcp_kv_mode;

    let mut keys: Vec<String> = match mode {
        app::McpKeyValuesMode::Env => server
            .config
            .env
            .as_ref()
            .map(|m| m.keys().cloned().collect())
            .unwrap_or_default(),
        app::McpKeyValuesMode::Headers => server
            .config
            .headers
            .as_ref()
            .map(|m| m.keys().cloned().collect())
            .unwrap_or_default(),
    };
    keys.sort_by_key(|a| a.to_lowercase());

    match code {
        KeyCode::Esc | KeyCode::Char('q') => app.screen = app::Screen::McpServer,
        KeyCode::Down => app.mcp_kv_index = app.mcp_kv_index.saturating_add(1),
        KeyCode::Up => app.mcp_kv_index = app.mcp_kv_index.saturating_sub(1),
        KeyCode::Char('n') => {
            app.modal = Some(app::Modal::Input {
                title: "key=value".to_string(),
                value: String::new(),
                cursor: usize::MAX,
                is_secret: false,
                action: app::InputAction::McpKeyValueAdd { mode },
            });
        }
        KeyCode::Enter | KeyCode::Char('e') => {
            if let Some(key) = keys.get(app.mcp_kv_index).cloned() {
                let value = match mode {
                    app::McpKeyValuesMode::Env => server
                        .config
                        .env
                        .as_ref()
                        .and_then(|m| m.get(&key))
                        .cloned()
                        .unwrap_or_default(),
                    app::McpKeyValuesMode::Headers => server
                        .config
                        .headers
                        .as_ref()
                        .and_then(|m| m.get(&key))
                        .cloned()
                        .unwrap_or_default(),
                };
                app.modal = Some(app::Modal::Input {
                    title: "key=value".to_string(),
                    value: format!("{key}={value}"),
                    cursor: usize::MAX,
                    is_secret: false,
                    action: app::InputAction::McpKeyValueEdit {
                        mode,
                        index: app.mcp_kv_index,
                    },
                });
            }
        }
        KeyCode::Char('x') => {
            if let Some(key) = keys.get(app.mcp_kv_index).cloned() {
                if let Some(server) = app.mcp_edit_draft.as_mut() {
                    match mode {
                        app::McpKeyValuesMode::Env => {
                            if let Some(env) = server.config.env.as_mut() {
                                env.remove(&key);
                                if env.is_empty() {
                                    server.config.env = None;
                                    app.mcp_kv_index = 0;
                                }
                            }
                        }
                        app::McpKeyValuesMode::Headers => {
                            if let Some(headers) = server.config.headers.as_mut() {
                                headers.remove(&key);
                                if headers.is_empty() {
                                    server.config.headers = None;
                                    app.mcp_kv_index = 0;
                                }
                            }
                        }
                    }
                }
            }
        }
        _ => {}
    }

    None
}

fn handle_codex_key(app: &mut app::App, code: KeyCode) -> Option<Action> {
    match code {
        KeyCode::Esc | KeyCode::Char('q') => app.screen = app::Screen::Main,
        KeyCode::Down => app.codex_index = app.codex_index.saturating_add(1),
        KeyCode::Up => app.codex_index = app.codex_index.saturating_sub(1),
        KeyCode::Char('r') => refresh_codex(app),
        KeyCode::Char('p') => {
            if let Some(p) = app.codex_profiles.get(app.codex_index) {
                return Some(Action::PreviewCodexApply { id: p.id.clone() });
            }
        }
        KeyCode::Char('E') => {
            if let Some(p) = app.codex_profiles.get(app.codex_index) {
                if p.id == "official" {
                    app.set_toast("Official profile is read-only", true);
                    return None;
                }
                return Some(Action::EditCodexProfile { id: p.id.clone() });
            }
        }
        KeyCode::Char('n') => {
            app.modal = Some(app::Modal::Input {
                title: "New Codex profile name".to_string(),
                value: String::new(),
                cursor: usize::MAX,
                is_secret: false,
                action: app::InputAction::CodexCreateProfile,
            });
        }
        KeyCode::Enter | KeyCode::Char('e') => {
            if let Some(p) = app.codex_profiles.get(app.codex_index) {
                app.codex_detail_id = Some(p.id.clone());
                app.codex_detail_focus = app::CodexDetailFocus::Fields;
                app.codex_detail_field_index = 0;
                app.codex_detail_provider_index = 0;
                app.codex_provider_id = None;
                app.codex_provider_field_index = 0;
                app.screen = app::Screen::CodexProfile;
                refresh_codex_detail(app);
            }
        }
        KeyCode::Char('a') => {
            if let Some(p) = app.codex_profiles.get(app.codex_index) {
                app.modal = Some(app::Modal::Confirm {
                    message: format!("Apply Codex profile '{}'?", p.name),
                    action: app::ConfirmAction::CodexApply { id: p.id.clone() },
                });
            }
        }
        KeyCode::Char('d') => {
            if let Some(p) = app.codex_profiles.get(app.codex_index) {
                if p.id == "official" {
                    app.set_toast("Cannot delete official profile", true);
                    return None;
                }
                app.modal = Some(app::Modal::Confirm {
                    message: format!("Delete Codex profile '{}'?", p.name),
                    action: app::ConfirmAction::CodexDelete { id: p.id.clone() },
                });
            }
        }
        KeyCode::Char('c') => {
            if let Some(p) = app.codex_profiles.get(app.codex_index) {
                if p.id == "official" {
                    app.set_toast("Official profile is read-only", true);
                    return None;
                }
                app.modal = Some(app::Modal::Input {
                    title: "Duplicate profile name".to_string(),
                    value: format!("{} (copy)", p.name),
                    cursor: usize::MAX,
                    is_secret: false,
                    action: app::InputAction::CodexDuplicate { id: p.id.clone() },
                });
            }
        }
        _ => {}
    }
    None
}

fn handle_codex_profile_key(app: &mut app::App, code: KeyCode) -> Option<Action> {
    let Some(profile_id) = app.codex_detail_id.clone() else {
        app.screen = app::Screen::Codex;
        return None;
    };
    let Some(profile) = app.codex_detail.as_ref() else {
        return None;
    };
    let is_official = profile_id == "official";

    match code {
        KeyCode::Esc | KeyCode::Char('q') => {
            app.screen = app::Screen::Codex;
            app.codex_provider_id = None;
        }
        KeyCode::Tab => {
            app.codex_detail_focus = match app.codex_detail_focus {
                app::CodexDetailFocus::Fields => app::CodexDetailFocus::Providers,
                app::CodexDetailFocus::Providers => app::CodexDetailFocus::Fields,
            };
        }
        KeyCode::Down => match app.codex_detail_focus {
            app::CodexDetailFocus::Fields => {
                app.codex_detail_field_index = app.codex_detail_field_index.saturating_add(1)
            }
            app::CodexDetailFocus::Providers => {
                app.codex_detail_provider_index = app.codex_detail_provider_index.saturating_add(1)
            }
        },
        KeyCode::Up => match app.codex_detail_focus {
            app::CodexDetailFocus::Fields => {
                app.codex_detail_field_index = app.codex_detail_field_index.saturating_sub(1)
            }
            app::CodexDetailFocus::Providers => {
                app.codex_detail_provider_index = app.codex_detail_provider_index.saturating_sub(1)
            }
        },
        KeyCode::Char('r') => refresh_codex_detail(app),
        KeyCode::Char('p') => return Some(Action::PreviewCodexApply { id: profile_id }),
        KeyCode::Char('E') => {
            if is_official {
                app.set_toast("Official profile is read-only", true);
                return None;
            }
            return Some(Action::EditCodexProfile { id: profile_id });
        }
        KeyCode::Char('a') => {
            app.modal = Some(app::Modal::Confirm {
                message: format!("Apply Codex profile '{}'?", profile.name),
                action: app::ConfirmAction::CodexApply { id: profile_id },
            });
        }
        KeyCode::Char('l') => {
            if is_official {
                app.set_toast("Official profile is read-only", true);
                return None;
            }
            if let Err(e) = codex_load_from_live_config(app, &profile_id) {
                app.set_toast(e.to_string(), true);
            } else {
                app.set_toast("Loaded from live config", false);
                refresh_codex_detail(app);
            }
        }
        KeyCode::Char('n') => {
            if is_official {
                app.set_toast("Official profile is read-only", true);
                return None;
            }
            if app.codex_detail_focus == app::CodexDetailFocus::Providers {
                app.modal = Some(app::Modal::Input {
                    title: "New provider id".to_string(),
                    value: String::new(),
                    cursor: usize::MAX,
                    is_secret: false,
                    action: app::InputAction::CodexAddProvider { id: profile_id },
                });
            }
        }
        KeyCode::Char('s') => {
            if is_official {
                app.set_toast("Official profile is read-only", true);
                return None;
            }
            if app.codex_detail_focus == app::CodexDetailFocus::Providers {
                if let Some(provider_id) = app
                    .codex_detail_provider_ids
                    .get(app.codex_detail_provider_index)
                    .cloned()
                {
                    if let Err(e) = codex_set_active_provider(app, &profile_id, &provider_id) {
                        app.set_toast(e.to_string(), true);
                    } else {
                        app.set_toast("Active provider set", false);
                        refresh_codex_detail(app);
                    }
                }
            }
        }
        KeyCode::Char('d') => {
            if is_official {
                app.set_toast("Official profile is read-only", true);
                return None;
            }
            if app.codex_detail_focus == app::CodexDetailFocus::Providers {
                if let Some(provider_id) = app
                    .codex_detail_provider_ids
                    .get(app.codex_detail_provider_index)
                {
                    app.modal = Some(app::Modal::Confirm {
                        message: format!("Delete provider '{provider_id}'?"),
                        action: app::ConfirmAction::CodexDeleteProvider {
                            profile_id,
                            provider_id: provider_id.clone(),
                        },
                    });
                }
            }
        }
        KeyCode::Enter | KeyCode::Char('e') => match app.codex_detail_focus {
            app::CodexDetailFocus::Fields => match app.codex_detail_field_index {
                0 => {
                    if is_official {
                        app.set_toast("Official profile is read-only", true);
                        return None;
                    }
                    app.modal = Some(app::Modal::Input {
                        title: "Profile name".to_string(),
                        value: profile.name.clone(),
                        cursor: usize::MAX,
                        is_secret: false,
                        action: app::InputAction::CodexSetProfileName { id: profile_id },
                    });
                }
                1 => {
                    if is_official {
                        app.set_toast("Official profile is read-only", true);
                        return None;
                    }
                    app.modal = Some(app::Modal::Input {
                        title: "Profile description".to_string(),
                        value: profile.description.clone().unwrap_or_default(),
                        cursor: usize::MAX,
                        is_secret: false,
                        action: app::InputAction::CodexSetProfileDescription { id: profile_id },
                    });
                }
                2 => {
                    if is_official {
                        app.set_toast("Official profile is read-only", true);
                        return None;
                    }
                    if app.codex_detail_provider_ids.is_empty() {
                        app.set_toast("No providers configured", true);
                        return None;
                    }
                    let options = app.codex_detail_provider_ids.clone();
                    let index = options
                        .iter()
                        .position(|p| p == &profile.model_provider)
                        .unwrap_or(0);
                    app.modal = Some(app::Modal::Select {
                        title: "Model provider".to_string(),
                        options,
                        index,
                        action: app::SelectAction::CodexSetProfileModelProvider { id: profile_id },
                    });
                }
                3 => {
                    if is_official {
                        app.set_toast("Official profile is read-only", true);
                        return None;
                    }
                    app.modal = Some(app::Modal::Input {
                        title: "Model".to_string(),
                        value: profile.model.clone(),
                        cursor: usize::MAX,
                        is_secret: false,
                        action: app::InputAction::CodexSetProfileModel { id: profile_id },
                    });
                }
                4 => {
                    if is_official {
                        app.set_toast("Official profile is read-only", true);
                        return None;
                    }
                    let options = vec![
                        "(none)".to_string(),
                        "xhigh".to_string(),
                        "high".to_string(),
                        "medium".to_string(),
                        "low".to_string(),
                        "minimal".to_string(),
                    ];
                    let index = profile
                        .model_reasoning_effort
                        .as_deref()
                        .and_then(|v| options.iter().position(|o| o == v))
                        .unwrap_or(0);
                    app.modal = Some(app::Modal::Select {
                        title: "Reasoning effort".to_string(),
                        options,
                        index,
                        action: app::SelectAction::CodexSetProfileReasoningEffort {
                            id: profile_id,
                        },
                    });
                }
                5 => {
                    if is_official {
                        app.set_toast("Official profile is read-only", true);
                        return None;
                    }
                    app.modal = Some(app::Modal::Input {
                        title: "API key".to_string(),
                        value: profile.api_key.clone().unwrap_or_default(),
                        cursor: usize::MAX,
                        is_secret: true,
                        action: app::InputAction::CodexSetProfileApiKey { id: profile_id },
                    });
                }
                _ => {}
            },
            app::CodexDetailFocus::Providers => {
                if let Some(provider_id) = app
                    .codex_detail_provider_ids
                    .get(app.codex_detail_provider_index)
                {
                    app.codex_provider_id = Some(provider_id.clone());
                    app.codex_provider_field_index = 0;
                    app.screen = app::Screen::CodexProvider;
                }
            }
        },
        _ => {}
    }

    None
}

fn handle_codex_provider_key(app: &mut app::App, code: KeyCode) -> Option<Action> {
    let Some(profile_id) = app.codex_detail_id.clone() else {
        app.screen = app::Screen::Codex;
        return None;
    };
    let Some(provider_id) = app.codex_provider_id.clone() else {
        app.screen = app::Screen::CodexProfile;
        return None;
    };
    let Some(profile) = app.codex_detail.as_ref() else {
        return None;
    };
    let Some(config) = profile.providers.get(&provider_id) else {
        app.set_toast("Provider not found", true);
        app.screen = app::Screen::CodexProfile;
        return None;
    };
    let is_official = profile_id == "official";

    match code {
        KeyCode::Esc | KeyCode::Char('q') => {
            app.screen = app::Screen::CodexProfile;
        }
        KeyCode::Down => {
            app.codex_provider_field_index = app.codex_provider_field_index.saturating_add(1)
        }
        KeyCode::Up => {
            app.codex_provider_field_index = app.codex_provider_field_index.saturating_sub(1)
        }
        KeyCode::Enter | KeyCode::Char('e') => match app.codex_provider_field_index {
            0 => {
                if is_official {
                    app.set_toast("Official profile is read-only", true);
                    return None;
                }
                app.modal = Some(app::Modal::Input {
                    title: "Provider name".to_string(),
                    value: config.name.clone().unwrap_or_default(),
                    cursor: usize::MAX,
                    is_secret: false,
                    action: app::InputAction::CodexSetProviderName {
                        profile_id,
                        provider_id,
                    },
                });
            }
            1 => {
                if is_official {
                    app.set_toast("Official profile is read-only", true);
                    return None;
                }
                app.modal = Some(app::Modal::Input {
                    title: "Provider base URL".to_string(),
                    value: config.base_url.clone().unwrap_or_default(),
                    cursor: usize::MAX,
                    is_secret: false,
                    action: app::InputAction::CodexSetProviderBaseUrl {
                        profile_id,
                        provider_id,
                    },
                });
            }
            2 => {
                if is_official {
                    app.set_toast("Official profile is read-only", true);
                    return None;
                }
                let options = vec!["responses".to_string(), "chat".to_string()];
                let index = config
                    .wire_api
                    .as_deref()
                    .and_then(|v| options.iter().position(|o| o == v))
                    .unwrap_or(0);
                app.modal = Some(app::Modal::Select {
                    title: "Wire API".to_string(),
                    options,
                    index,
                    action: app::SelectAction::CodexSetProviderWireApi {
                        profile_id,
                        provider_id,
                    },
                });
            }
            3 => {
                if is_official {
                    app.set_toast("Official profile is read-only", true);
                    return None;
                }
                app.modal = Some(app::Modal::Input {
                    title: "Provider model".to_string(),
                    value: config.model.clone().unwrap_or_default(),
                    cursor: usize::MAX,
                    is_secret: false,
                    action: app::InputAction::CodexSetProviderModel {
                        profile_id,
                        provider_id,
                    },
                });
            }
            4 => {
                if is_official {
                    app.set_toast("Official profile is read-only", true);
                    return None;
                }
                let options = vec![
                    "(none)".to_string(),
                    "xhigh".to_string(),
                    "high".to_string(),
                    "medium".to_string(),
                    "low".to_string(),
                    "minimal".to_string(),
                ];
                let index = config
                    .model_reasoning_effort
                    .as_deref()
                    .and_then(|v| options.iter().position(|o| o == v))
                    .unwrap_or(0);
                app.modal = Some(app::Modal::Select {
                    title: "Reasoning effort".to_string(),
                    options,
                    index,
                    action: app::SelectAction::CodexSetProviderReasoningEffort {
                        profile_id,
                        provider_id,
                    },
                });
            }
            5 => {
                if is_official {
                    app.set_toast("Official profile is read-only", true);
                    return None;
                }
                app.modal = Some(app::Modal::Input {
                    title: "Provider API key".to_string(),
                    value: config.api_key.clone().unwrap_or_default(),
                    cursor: usize::MAX,
                    is_secret: true,
                    action: app::InputAction::CodexSetProviderApiKey {
                        profile_id,
                        provider_id,
                    },
                });
            }
            _ => {}
        },
        _ => {}
    }

    None
}

fn handle_opencode_key(app: &mut app::App, code: KeyCode) -> Option<Action> {
    match code {
        KeyCode::Esc | KeyCode::Char('q') => app.screen = app::Screen::Main,
        KeyCode::Down => app.opencode_index = app.opencode_index.saturating_add(1),
        KeyCode::Up => app.opencode_index = app.opencode_index.saturating_sub(1),
        KeyCode::Char('r') => refresh_opencode(app),
        KeyCode::Char('p') => {
            if let Some(p) = app.opencode_profiles.get(app.opencode_index) {
                return Some(Action::PreviewOpenCodeApply { id: p.id.clone() });
            }
        }
        KeyCode::Char('E') => {
            if let Some(p) = app.opencode_profiles.get(app.opencode_index) {
                return Some(Action::EditOpenCodeProfile { id: p.id.clone() });
            }
        }
        KeyCode::Char('n') => {
            app.modal = Some(app::Modal::Input {
                title: "New OpenCode profile name".to_string(),
                value: String::new(),
                cursor: usize::MAX,
                is_secret: false,
                action: app::InputAction::OpenCodeCreateProfile,
            });
        }
        KeyCode::Enter | KeyCode::Char('e') => {
            if let Some(p) = app.opencode_profiles.get(app.opencode_index) {
                app.opencode_detail_id = Some(p.id.clone());
                app.opencode_detail_focus = app::CodexDetailFocus::Fields;
                app.opencode_detail_field_index = 0;
                app.opencode_detail_provider_index = 0;
                app.opencode_provider_id = None;
                app.opencode_provider_focus = app::CodexDetailFocus::Fields;
                app.opencode_provider_field_index = 0;
                app.opencode_provider_model_index = 0;
                app.opencode_model_id = None;
                app.opencode_model_field_index = 0;
                app.screen = app::Screen::OpenCodeProfile;
                refresh_opencode_detail(app);
            }
        }
        KeyCode::Char('a') => {
            if let Some(p) = app.opencode_profiles.get(app.opencode_index) {
                app.modal = Some(app::Modal::Confirm {
                    message: format!("Apply OpenCode profile '{}'?", p.name),
                    action: app::ConfirmAction::OpenCodeApply { id: p.id.clone() },
                });
            }
        }
        KeyCode::Char('d') => {
            if let Some(p) = app.opencode_profiles.get(app.opencode_index) {
                app.modal = Some(app::Modal::Confirm {
                    message: format!("Delete OpenCode profile '{}'?", p.name),
                    action: app::ConfirmAction::OpenCodeDelete { id: p.id.clone() },
                });
            }
        }
        KeyCode::Char('c') => {
            if let Some(p) = app.opencode_profiles.get(app.opencode_index) {
                app.modal = Some(app::Modal::Input {
                    title: "Duplicate profile name".to_string(),
                    value: format!("{} (copy)", p.name),
                    cursor: usize::MAX,
                    is_secret: false,
                    action: app::InputAction::OpenCodeDuplicate { id: p.id.clone() },
                });
            }
        }
        _ => {}
    }
    None
}

fn handle_opencode_profile_key(app: &mut app::App, code: KeyCode) -> Option<Action> {
    let Some(profile_id) = app.opencode_detail_id.clone() else {
        app.screen = app::Screen::OpenCode;
        return None;
    };
    let Some(profile) = app.opencode_detail.as_ref() else {
        return None;
    };

    match code {
        KeyCode::Esc | KeyCode::Char('q') => {
            app.screen = app::Screen::OpenCode;
            app.opencode_provider_id = None;
            app.opencode_model_id = None;
        }
        KeyCode::Tab => {
            app.opencode_detail_focus = match app.opencode_detail_focus {
                app::CodexDetailFocus::Fields => app::CodexDetailFocus::Providers,
                app::CodexDetailFocus::Providers => app::CodexDetailFocus::Fields,
            };
        }
        KeyCode::Down => match app.opencode_detail_focus {
            app::CodexDetailFocus::Fields => {
                app.opencode_detail_field_index = app.opencode_detail_field_index.saturating_add(1)
            }
            app::CodexDetailFocus::Providers => {
                app.opencode_detail_provider_index =
                    app.opencode_detail_provider_index.saturating_add(1)
            }
        },
        KeyCode::Up => match app.opencode_detail_focus {
            app::CodexDetailFocus::Fields => {
                app.opencode_detail_field_index = app.opencode_detail_field_index.saturating_sub(1)
            }
            app::CodexDetailFocus::Providers => {
                app.opencode_detail_provider_index =
                    app.opencode_detail_provider_index.saturating_sub(1)
            }
        },
        KeyCode::Char('r') => refresh_opencode_detail(app),
        KeyCode::Char('p') => return Some(Action::PreviewOpenCodeApply { id: profile_id }),
        KeyCode::Char('E') => return Some(Action::EditOpenCodeProfile { id: profile_id }),
        KeyCode::Char('a') => {
            app.modal = Some(app::Modal::Confirm {
                message: format!("Apply OpenCode profile '{}'?", profile.name),
                action: app::ConfirmAction::OpenCodeApply { id: profile_id },
            });
        }
        KeyCode::Char('i') => {
            let options = vec!["skip".to_string(), "replace".to_string()];
            app.modal = Some(app::Modal::Select {
                title: "Import providers from live config".to_string(),
                options,
                index: 0,
                action: app::SelectAction::OpenCodeImportProviders { id: profile_id },
            });
        }
        KeyCode::Char('n') if app.opencode_detail_focus == app::CodexDetailFocus::Providers => {
            app.modal = Some(app::Modal::Input {
                title: "New provider id".to_string(),
                value: String::new(),
                cursor: usize::MAX,
                is_secret: false,
                action: app::InputAction::OpenCodeAddProvider { profile_id },
            });
        }
        KeyCode::Char('d') if app.opencode_detail_focus == app::CodexDetailFocus::Providers => {
            if let Some(provider_id) = app
                .opencode_detail_provider_ids
                .get(app.opencode_detail_provider_index)
            {
                app.modal = Some(app::Modal::Confirm {
                    message: format!("Delete provider '{provider_id}'?"),
                    action: app::ConfirmAction::OpenCodeDeleteProvider {
                        profile_id,
                        provider_id: provider_id.clone(),
                    },
                });
            }
        }
        KeyCode::Enter | KeyCode::Char('e') => match app.opencode_detail_focus {
            app::CodexDetailFocus::Fields => match app.opencode_detail_field_index {
                0 => {
                    app.modal = Some(app::Modal::Input {
                        title: "Profile name".to_string(),
                        value: profile.name.clone(),
                        cursor: usize::MAX,
                        is_secret: false,
                        action: app::InputAction::OpenCodeSetProfileName { id: profile_id },
                    });
                }
                1 => {
                    app.modal = Some(app::Modal::Input {
                        title: "Profile description".to_string(),
                        value: profile.description.clone().unwrap_or_default(),
                        cursor: usize::MAX,
                        is_secret: false,
                        action: app::InputAction::OpenCodeSetProfileDescription { id: profile_id },
                    });
                }
                _ => {}
            },
            app::CodexDetailFocus::Providers => {
                if let Some(provider_id) = app
                    .opencode_detail_provider_ids
                    .get(app.opencode_detail_provider_index)
                {
                    app.opencode_provider_id = Some(provider_id.clone());
                    app.opencode_provider_focus = app::CodexDetailFocus::Fields;
                    app.opencode_provider_field_index = 0;
                    app.opencode_provider_model_index = 0;
                    app.opencode_model_id = None;
                    app.opencode_model_field_index = 0;
                    app.screen = app::Screen::OpenCodeProvider;
                    refresh_opencode_detail(app);
                }
            }
        },
        _ => {}
    }

    None
}

fn handle_opencode_provider_key(app: &mut app::App, code: KeyCode) -> Option<Action> {
    let Some(profile_id) = app.opencode_detail_id.clone() else {
        app.screen = app::Screen::OpenCode;
        return None;
    };
    let Some(provider_id) = app.opencode_provider_id.clone() else {
        app.screen = app::Screen::OpenCodeProfile;
        return None;
    };
    let Some(profile) = app.opencode_detail.as_ref() else {
        return None;
    };
    let Some(config) = profile.providers.get(&provider_id) else {
        app.set_toast("Provider not found", true);
        app.screen = app::Screen::OpenCodeProfile;
        return None;
    };

    match code {
        KeyCode::Esc | KeyCode::Char('q') => {
            app.screen = app::Screen::OpenCodeProfile;
            app.opencode_model_id = None;
        }
        KeyCode::Tab => {
            app.opencode_provider_focus = match app.opencode_provider_focus {
                app::CodexDetailFocus::Fields => app::CodexDetailFocus::Providers,
                app::CodexDetailFocus::Providers => app::CodexDetailFocus::Fields,
            };
        }
        KeyCode::Down => match app.opencode_provider_focus {
            app::CodexDetailFocus::Fields => {
                app.opencode_provider_field_index =
                    app.opencode_provider_field_index.saturating_add(1)
            }
            app::CodexDetailFocus::Providers => {
                app.opencode_provider_model_index =
                    app.opencode_provider_model_index.saturating_add(1)
            }
        },
        KeyCode::Up => match app.opencode_provider_focus {
            app::CodexDetailFocus::Fields => {
                app.opencode_provider_field_index =
                    app.opencode_provider_field_index.saturating_sub(1)
            }
            app::CodexDetailFocus::Providers => {
                app.opencode_provider_model_index =
                    app.opencode_provider_model_index.saturating_sub(1)
            }
        },
        KeyCode::Char('n') if app.opencode_provider_focus == app::CodexDetailFocus::Providers => {
            app.modal = Some(app::Modal::Input {
                title: "New model id".to_string(),
                value: String::new(),
                cursor: usize::MAX,
                is_secret: false,
                action: app::InputAction::OpenCodeAddModel {
                    profile_id,
                    provider_id,
                },
            });
        }
        KeyCode::Char('d') if app.opencode_provider_focus == app::CodexDetailFocus::Providers => {
            if let Some(model_id) = app
                .opencode_provider_model_ids
                .get(app.opencode_provider_model_index)
            {
                app.modal = Some(app::Modal::Confirm {
                    message: format!("Delete model '{model_id}'?"),
                    action: app::ConfirmAction::OpenCodeDeleteModel {
                        profile_id,
                        provider_id,
                        model_id: model_id.clone(),
                    },
                });
            }
        }
        KeyCode::Enter | KeyCode::Char('e') => match app.opencode_provider_focus {
            app::CodexDetailFocus::Fields => match app.opencode_provider_field_index {
                0 => {
                    app.modal = Some(app::Modal::Input {
                        title: "Provider display name".to_string(),
                        value: config.name.clone().unwrap_or_default(),
                        cursor: usize::MAX,
                        is_secret: false,
                        action: app::InputAction::OpenCodeSetProviderName {
                            profile_id,
                            provider_id,
                        },
                    });
                }
                1 => {
                    app.modal = Some(app::Modal::Input {
                        title: "Provider NPM package".to_string(),
                        value: config.npm.clone().unwrap_or_default(),
                        cursor: usize::MAX,
                        is_secret: false,
                        action: app::InputAction::OpenCodeSetProviderNpm {
                            profile_id,
                            provider_id,
                        },
                    });
                }
                2 => {
                    let base_url = config
                        .options
                        .as_ref()
                        .and_then(|o| o.base_url.clone())
                        .unwrap_or_default();
                    app.modal = Some(app::Modal::Input {
                        title: "Provider base URL".to_string(),
                        value: base_url,
                        cursor: usize::MAX,
                        is_secret: false,
                        action: app::InputAction::OpenCodeSetProviderBaseUrl {
                            profile_id,
                            provider_id,
                        },
                    });
                }
                3 => {
                    let api_key = profile
                        .auth
                        .get(&provider_id)
                        .and_then(|v| v.get("key"))
                        .and_then(|k| k.as_str())
                        .unwrap_or("")
                        .to_string();
                    app.modal = Some(app::Modal::Input {
                        title: "Provider API key".to_string(),
                        value: api_key,
                        cursor: usize::MAX,
                        is_secret: true,
                        action: app::InputAction::OpenCodeSetProviderApiKey {
                            profile_id,
                            provider_id,
                        },
                    });
                }
                4 => {
                    let timeout = config
                        .options
                        .as_ref()
                        .and_then(|o| o.timeout)
                        .map(|t| t.to_string())
                        .unwrap_or_default();
                    app.modal = Some(app::Modal::Input {
                        title: "Timeout (ms)".to_string(),
                        value: timeout,
                        cursor: usize::MAX,
                        is_secret: false,
                        action: app::InputAction::OpenCodeSetProviderTimeout {
                            profile_id,
                            provider_id,
                        },
                    });
                }
                _ => {}
            },
            app::CodexDetailFocus::Providers => {
                if let Some(model_id) = app
                    .opencode_provider_model_ids
                    .get(app.opencode_provider_model_index)
                {
                    app.opencode_model_id = Some(model_id.clone());
                    app.opencode_model_field_index = 0;
                    app.screen = app::Screen::OpenCodeModel;
                }
            }
        },
        _ => {}
    }

    None
}

fn handle_opencode_model_key(app: &mut app::App, code: KeyCode) -> Option<Action> {
    let Some(profile_id) = app.opencode_detail_id.clone() else {
        app.screen = app::Screen::OpenCode;
        return None;
    };
    let Some(provider_id) = app.opencode_provider_id.clone() else {
        app.screen = app::Screen::OpenCodeProfile;
        return None;
    };
    let Some(model_id) = app.opencode_model_id.clone() else {
        app.screen = app::Screen::OpenCodeProvider;
        return None;
    };
    let Some(profile) = app.opencode_detail.as_ref() else {
        return None;
    };
    let model = profile
        .providers
        .get(&provider_id)
        .and_then(|p| p.models.as_ref())
        .and_then(|m| m.get(&model_id));
    let Some(model) = model else {
        app.set_toast("Model not found", true);
        app.screen = app::Screen::OpenCodeProvider;
        return None;
    };

    match code {
        KeyCode::Esc | KeyCode::Char('q') => app.screen = app::Screen::OpenCodeProvider,
        KeyCode::Down => {
            app.opencode_model_field_index = app.opencode_model_field_index.saturating_add(1)
        }
        KeyCode::Up => {
            app.opencode_model_field_index = app.opencode_model_field_index.saturating_sub(1)
        }
        KeyCode::Enter | KeyCode::Char('e') => match app.opencode_model_field_index {
            0 => {
                app.modal = Some(app::Modal::Input {
                    title: "Model display name".to_string(),
                    value: model.name.clone().unwrap_or_default(),
                    cursor: usize::MAX,
                    is_secret: false,
                    action: app::InputAction::OpenCodeSetModelName {
                        profile_id,
                        provider_id,
                        model_id,
                    },
                });
            }
            1 => {
                let current = model
                    .limit
                    .as_ref()
                    .and_then(|l| l.context)
                    .map(|v| v.to_string())
                    .unwrap_or_default();
                app.modal = Some(app::Modal::Input {
                    title: "Context limit".to_string(),
                    value: current,
                    cursor: usize::MAX,
                    is_secret: false,
                    action: app::InputAction::OpenCodeSetModelContextLimit {
                        profile_id,
                        provider_id,
                        model_id,
                    },
                });
            }
            2 => {
                let current = model
                    .limit
                    .as_ref()
                    .and_then(|l| l.output)
                    .map(|v| v.to_string())
                    .unwrap_or_default();
                app.modal = Some(app::Modal::Input {
                    title: "Output limit".to_string(),
                    value: current,
                    cursor: usize::MAX,
                    is_secret: false,
                    action: app::InputAction::OpenCodeSetModelOutputLimit {
                        profile_id,
                        provider_id,
                        model_id,
                    },
                });
            }
            _ => {}
        },
        _ => {}
    }

    None
}

fn handle_openclaw_key(app: &mut app::App, code: KeyCode) -> Option<Action> {
    match code {
        KeyCode::Esc | KeyCode::Char('q') => app.screen = app::Screen::Main,
        KeyCode::Down => app.openclaw_index = app.openclaw_index.saturating_add(1),
        KeyCode::Up => app.openclaw_index = app.openclaw_index.saturating_sub(1),
        KeyCode::Char('r') => refresh_openclaw(app),
        KeyCode::Char('p') => {
            if let Some(p) = app.openclaw_profiles.get(app.openclaw_index) {
                return Some(Action::PreviewOpenClawApply { id: p.id.clone() });
            }
        }
        KeyCode::Char('E') => {
            if let Some(p) = app.openclaw_profiles.get(app.openclaw_index) {
                return Some(Action::EditOpenClawProfile { id: p.id.clone() });
            }
        }
        KeyCode::Char('n') => {
            app.modal = Some(app::Modal::Input {
                title: "New OpenClaw profile name".to_string(),
                value: String::new(),
                cursor: usize::MAX,
                is_secret: false,
                action: app::InputAction::OpenClawCreateProfile,
            });
        }
        KeyCode::Enter | KeyCode::Char('e') => {
            if let Some(p) = app.openclaw_profiles.get(app.openclaw_index) {
                app.openclaw_detail_id = Some(p.id.clone());
                app.openclaw_detail_focus = app::OpenClawProfileFocus::Fields;
                app.openclaw_detail_field_index = 0;
                app.openclaw_detail_failover_index = 0;
                app.openclaw_detail_provider_index = 0;
                app.openclaw_provider_id = None;
                app.openclaw_provider_focus = app::CodexDetailFocus::Fields;
                app.openclaw_provider_field_index = 0;
                app.openclaw_provider_model_index = 0;
                app.openclaw_model_field_index = 0;
                app.openclaw_helpers_field_index = 0;
                app.screen = app::Screen::OpenClawProfile;
                refresh_openclaw_detail(app);
            }
        }
        KeyCode::Char('a') => {
            if let Some(p) = app.openclaw_profiles.get(app.openclaw_index) {
                app.modal = Some(app::Modal::Confirm {
                    message: format!("Apply OpenClaw profile '{}'?", p.name),
                    action: app::ConfirmAction::OpenClawApply { id: p.id.clone() },
                });
            }
        }
        KeyCode::Char('d') => {
            if let Some(p) = app.openclaw_profiles.get(app.openclaw_index) {
                app.modal = Some(app::Modal::Confirm {
                    message: format!("Delete OpenClaw profile '{}'?", p.name),
                    action: app::ConfirmAction::OpenClawDelete { id: p.id.clone() },
                });
            }
        }
        KeyCode::Char('c') => {
            if let Some(p) = app.openclaw_profiles.get(app.openclaw_index) {
                app.modal = Some(app::Modal::Input {
                    title: "Duplicate profile name".to_string(),
                    value: format!("{} (copy)", p.name),
                    cursor: usize::MAX,
                    is_secret: false,
                    action: app::InputAction::OpenClawDuplicate { id: p.id.clone() },
                });
            }
        }
        _ => {}
    }
    None
}

fn openclaw_available_model_refs(
    profile: &droidgear_core::openclaw::OpenClawProfile,
) -> Vec<String> {
    let mut refs: Vec<String> = Vec::new();
    for (provider_id, cfg) in &profile.providers {
        for m in &cfg.models {
            let mid = m.id.trim();
            if mid.is_empty() {
                continue;
            }
            refs.push(format!("{provider_id}/{mid}"));
        }
    }
    refs.sort_by_key(|a| a.to_lowercase());
    refs
}

fn openclaw_load_from_live_config(app: &mut app::App, profile_id: &str) -> anyhow::Result<()> {
    let live = droidgear_core::openclaw::read_openclaw_current_config_for_home(&app.home_dir)
        .map_err(anyhow::Error::msg)?;
    let mut profile =
        droidgear_core::openclaw::get_openclaw_profile_for_home(&app.home_dir, profile_id)
            .map_err(anyhow::Error::msg)?;
    profile.providers = live.providers;
    profile.default_model = live.default_model;
    droidgear_core::openclaw::save_openclaw_profile_for_home(&app.home_dir, profile)
        .map_err(anyhow::Error::msg)?;
    Ok(())
}

fn handle_openclaw_profile_key(app: &mut app::App, code: KeyCode) -> Option<Action> {
    let Some(profile_id) = app.openclaw_detail_id.clone() else {
        app.screen = app::Screen::OpenClaw;
        return None;
    };
    let Some(profile) = app.openclaw_detail.as_ref() else {
        return None;
    };

    match code {
        KeyCode::Esc | KeyCode::Char('q') => {
            app.screen = app::Screen::OpenClaw;
            app.openclaw_provider_id = None;
        }
        KeyCode::Tab => {
            app.openclaw_detail_focus = match app.openclaw_detail_focus {
                app::OpenClawProfileFocus::Fields => app::OpenClawProfileFocus::Failover,
                app::OpenClawProfileFocus::Failover => app::OpenClawProfileFocus::Providers,
                app::OpenClawProfileFocus::Providers => app::OpenClawProfileFocus::Fields,
            };
        }
        KeyCode::Down => match app.openclaw_detail_focus {
            app::OpenClawProfileFocus::Fields => {
                app.openclaw_detail_field_index = app.openclaw_detail_field_index.saturating_add(1)
            }
            app::OpenClawProfileFocus::Failover => {
                app.openclaw_detail_failover_index =
                    app.openclaw_detail_failover_index.saturating_add(1)
            }
            app::OpenClawProfileFocus::Providers => {
                app.openclaw_detail_provider_index =
                    app.openclaw_detail_provider_index.saturating_add(1)
            }
        },
        KeyCode::Up => match app.openclaw_detail_focus {
            app::OpenClawProfileFocus::Fields => {
                app.openclaw_detail_field_index = app.openclaw_detail_field_index.saturating_sub(1)
            }
            app::OpenClawProfileFocus::Failover => {
                app.openclaw_detail_failover_index =
                    app.openclaw_detail_failover_index.saturating_sub(1)
            }
            app::OpenClawProfileFocus::Providers => {
                app.openclaw_detail_provider_index =
                    app.openclaw_detail_provider_index.saturating_sub(1)
            }
        },
        KeyCode::Char('r') => refresh_openclaw_detail(app),
        KeyCode::Char('p') => return Some(Action::PreviewOpenClawApply { id: profile_id }),
        KeyCode::Char('E') => return Some(Action::EditOpenClawProfile { id: profile_id }),
        KeyCode::Char('a') => {
            app.modal = Some(app::Modal::Confirm {
                message: format!("Apply OpenClaw profile '{}'?", profile.name),
                action: app::ConfirmAction::OpenClawApply { id: profile_id },
            });
        }
        KeyCode::Char('l') => {
            if let Err(e) = openclaw_load_from_live_config(app, &profile_id) {
                app.set_toast(e.to_string(), true);
            } else {
                app.set_toast("Loaded from live config", false);
                refresh_openclaw_detail(app);
            }
        }
        KeyCode::Char('h') => {
            app.openclaw_helpers_field_index = 0;
            app.screen = app::Screen::OpenClawHelpers;
        }
        KeyCode::Char('s') => {
            app.openclaw_subagents_index = 0;
            app.openclaw_subagent_detail = None;
            app.openclaw_subagent_field_index = 0;
            app.screen = app::Screen::OpenClawSubagents;
            refresh_openclaw_subagents(app);
        }
        KeyCode::Char('n') => match app.openclaw_detail_focus {
            app::OpenClawProfileFocus::Failover => {
                let refs = openclaw_available_model_refs(profile);
                let current = profile.failover_models.as_deref().unwrap_or(&[]);
                let options = refs
                    .into_iter()
                    .filter(|r| !current.contains(r))
                    .collect::<Vec<String>>();
                if options.is_empty() {
                    app.set_toast("No models available to add", true);
                    return None;
                }
                app.modal = Some(app::Modal::Select {
                    title: "Add failover model".to_string(),
                    options,
                    index: 0,
                    action: app::SelectAction::OpenClawAddFailoverModel { id: profile_id },
                });
            }
            app::OpenClawProfileFocus::Providers => {
                app.modal = Some(app::Modal::Input {
                    title: "New provider id".to_string(),
                    value: String::new(),
                    cursor: usize::MAX,
                    is_secret: false,
                    action: app::InputAction::OpenClawAddProvider { profile_id },
                });
            }
            _ => {}
        },
        KeyCode::Char('d') => match app.openclaw_detail_focus {
            app::OpenClawProfileFocus::Failover => {
                let idx = app.openclaw_detail_failover_index;
                let mut profile = droidgear_core::openclaw::get_openclaw_profile_for_home(
                    &app.home_dir,
                    &profile_id,
                )
                .map_err(anyhow::Error::msg)
                .ok()?;
                let mut list = profile.failover_models.take().unwrap_or_default();
                if idx < list.len() {
                    list.remove(idx);
                }
                profile.failover_models = (!list.is_empty()).then_some(list);
                droidgear_core::openclaw::save_openclaw_profile_for_home(&app.home_dir, profile)
                    .map_err(anyhow::Error::msg)
                    .ok()?;
                refresh_openclaw_detail(app);
            }
            app::OpenClawProfileFocus::Providers => {
                if let Some(provider_id) = app
                    .openclaw_detail_provider_ids
                    .get(app.openclaw_detail_provider_index)
                {
                    app.modal = Some(app::Modal::Confirm {
                        message: format!("Delete provider '{provider_id}'?"),
                        action: app::ConfirmAction::OpenClawDeleteProvider {
                            profile_id,
                            provider_id: provider_id.clone(),
                        },
                    });
                }
            }
            _ => {}
        },
        KeyCode::Enter | KeyCode::Char('e') => match app.openclaw_detail_focus {
            app::OpenClawProfileFocus::Fields => match app.openclaw_detail_field_index {
                0 => {
                    app.modal = Some(app::Modal::Input {
                        title: "Profile name".to_string(),
                        value: profile.name.clone(),
                        cursor: usize::MAX,
                        is_secret: false,
                        action: app::InputAction::OpenClawSetProfileName { id: profile_id },
                    });
                }
                1 => {
                    app.modal = Some(app::Modal::Input {
                        title: "Profile description".to_string(),
                        value: profile.description.clone().unwrap_or_default(),
                        cursor: usize::MAX,
                        is_secret: false,
                        action: app::InputAction::OpenClawSetProfileDescription { id: profile_id },
                    });
                }
                2 => {
                    let mut options = vec!["(none)".to_string()];
                    options.extend(openclaw_available_model_refs(profile));
                    let index = profile
                        .default_model
                        .as_deref()
                        .and_then(|v| options.iter().position(|o| o == v))
                        .unwrap_or(0);
                    app.modal = Some(app::Modal::Select {
                        title: "Default model".to_string(),
                        options,
                        index,
                        action: app::SelectAction::OpenClawSetDefaultModel { id: profile_id },
                    });
                }
                _ => {}
            },
            app::OpenClawProfileFocus::Providers => {
                if let Some(provider_id) = app
                    .openclaw_detail_provider_ids
                    .get(app.openclaw_detail_provider_index)
                {
                    app.openclaw_provider_id = Some(provider_id.clone());
                    app.openclaw_provider_focus = app::CodexDetailFocus::Fields;
                    app.openclaw_provider_field_index = 0;
                    app.openclaw_provider_model_index = 0;
                    app.openclaw_model_field_index = 0;
                    app.screen = app::Screen::OpenClawProvider;
                }
            }
            _ => {}
        },
        _ => {}
    }

    None
}

fn handle_openclaw_provider_key(app: &mut app::App, code: KeyCode) -> Option<Action> {
    let Some(profile_id) = app.openclaw_detail_id.clone() else {
        app.screen = app::Screen::OpenClaw;
        return None;
    };
    let Some(provider_id) = app.openclaw_provider_id.clone() else {
        app.screen = app::Screen::OpenClawProfile;
        return None;
    };
    let Some(profile) = app.openclaw_detail.as_ref() else {
        return None;
    };
    let Some(config) = profile.providers.get(&provider_id) else {
        app.set_toast("Provider not found", true);
        app.screen = app::Screen::OpenClawProfile;
        return None;
    };

    match code {
        KeyCode::Esc | KeyCode::Char('q') => app.screen = app::Screen::OpenClawProfile,
        KeyCode::Tab => {
            app.openclaw_provider_focus = match app.openclaw_provider_focus {
                app::CodexDetailFocus::Fields => app::CodexDetailFocus::Providers,
                app::CodexDetailFocus::Providers => app::CodexDetailFocus::Fields,
            };
        }
        KeyCode::Down => match app.openclaw_provider_focus {
            app::CodexDetailFocus::Fields => {
                app.openclaw_provider_field_index =
                    app.openclaw_provider_field_index.saturating_add(1)
            }
            app::CodexDetailFocus::Providers => {
                app.openclaw_provider_model_index =
                    app.openclaw_provider_model_index.saturating_add(1)
            }
        },
        KeyCode::Up => match app.openclaw_provider_focus {
            app::CodexDetailFocus::Fields => {
                app.openclaw_provider_field_index =
                    app.openclaw_provider_field_index.saturating_sub(1)
            }
            app::CodexDetailFocus::Providers => {
                app.openclaw_provider_model_index =
                    app.openclaw_provider_model_index.saturating_sub(1)
            }
        },
        KeyCode::Char('n') if app.openclaw_provider_focus == app::CodexDetailFocus::Providers => {
            app.modal = Some(app::Modal::Input {
                title: "New model id".to_string(),
                value: String::new(),
                cursor: usize::MAX,
                is_secret: false,
                action: app::InputAction::OpenClawAddModel {
                    profile_id,
                    provider_id,
                },
            });
        }
        KeyCode::Char('d') if app.openclaw_provider_focus == app::CodexDetailFocus::Providers => {
            app.modal = Some(app::Modal::Confirm {
                message: "Delete selected model?".to_string(),
                action: app::ConfirmAction::OpenClawDeleteModel {
                    profile_id,
                    provider_id,
                    model_index: app.openclaw_provider_model_index,
                },
            });
        }
        KeyCode::Enter | KeyCode::Char('e') => match app.openclaw_provider_focus {
            app::CodexDetailFocus::Fields => match app.openclaw_provider_field_index {
                0 => {
                    app.modal = Some(app::Modal::Input {
                        title: "Provider base URL".to_string(),
                        value: config.base_url.clone().unwrap_or_default(),
                        cursor: usize::MAX,
                        is_secret: false,
                        action: app::InputAction::OpenClawSetProviderBaseUrl {
                            profile_id,
                            provider_id,
                        },
                    });
                }
                1 => {
                    app.modal = Some(app::Modal::Input {
                        title: "Provider API key".to_string(),
                        value: config.api_key.clone().unwrap_or_default(),
                        cursor: usize::MAX,
                        is_secret: true,
                        action: app::InputAction::OpenClawSetProviderApiKey {
                            profile_id,
                            provider_id,
                        },
                    });
                }
                2 => {
                    let options = vec![
                        "openai-completions".to_string(),
                        "openai-responses".to_string(),
                        "anthropic-messages".to_string(),
                    ];
                    let index = config
                        .api
                        .as_deref()
                        .and_then(|v| options.iter().position(|o| o == v))
                        .unwrap_or(0);
                    app.modal = Some(app::Modal::Select {
                        title: "API type".to_string(),
                        options,
                        index,
                        action: app::SelectAction::OpenClawSetProviderApiType {
                            profile_id,
                            provider_id,
                        },
                    });
                }
                _ => {}
            },
            app::CodexDetailFocus::Providers => {
                if !config.models.is_empty() {
                    app.openclaw_model_field_index = 0;
                    app.screen = app::Screen::OpenClawModel;
                }
            }
        },
        _ => {}
    }

    None
}

fn handle_openclaw_model_key(app: &mut app::App, code: KeyCode) -> Option<Action> {
    let Some(profile_id) = app.openclaw_detail_id.clone() else {
        app.screen = app::Screen::OpenClaw;
        return None;
    };
    let Some(provider_id) = app.openclaw_provider_id.clone() else {
        app.screen = app::Screen::OpenClawProfile;
        return None;
    };
    let Some(profile) = app.openclaw_detail.as_ref() else {
        return None;
    };
    let Some(provider) = profile.providers.get(&provider_id) else {
        app.set_toast("Provider not found", true);
        app.screen = app::Screen::OpenClawProfile;
        return None;
    };
    let model_index = app.openclaw_provider_model_index;
    let Some(model) = provider.models.get(model_index) else {
        app.set_toast("Model not found", true);
        app.screen = app::Screen::OpenClawProvider;
        return None;
    };

    match code {
        KeyCode::Esc | KeyCode::Char('q') => app.screen = app::Screen::OpenClawProvider,
        KeyCode::Down => {
            app.openclaw_model_field_index = app.openclaw_model_field_index.saturating_add(1)
        }
        KeyCode::Up => {
            app.openclaw_model_field_index = app.openclaw_model_field_index.saturating_sub(1)
        }
        KeyCode::Enter | KeyCode::Char('e') => match app.openclaw_model_field_index {
            0 => {
                app.modal = Some(app::Modal::Input {
                    title: "Model id".to_string(),
                    value: model.id.clone(),
                    cursor: usize::MAX,
                    is_secret: false,
                    action: app::InputAction::OpenClawSetModelId {
                        profile_id,
                        provider_id,
                        model_index,
                    },
                });
            }
            1 => {
                app.modal = Some(app::Modal::Input {
                    title: "Model name".to_string(),
                    value: model.name.clone().unwrap_or_default(),
                    cursor: usize::MAX,
                    is_secret: false,
                    action: app::InputAction::OpenClawSetModelName {
                        profile_id,
                        provider_id,
                        model_index,
                    },
                });
            }
            2 => {
                app.modal = Some(app::Modal::Input {
                    title: "Context window".to_string(),
                    value: model
                        .context_window
                        .map(|v| v.to_string())
                        .unwrap_or_default(),
                    cursor: usize::MAX,
                    is_secret: false,
                    action: app::InputAction::OpenClawSetModelContextWindow {
                        profile_id,
                        provider_id,
                        model_index,
                    },
                });
            }
            3 => {
                app.modal = Some(app::Modal::Input {
                    title: "Max tokens".to_string(),
                    value: model.max_tokens.map(|v| v.to_string()).unwrap_or_default(),
                    cursor: usize::MAX,
                    is_secret: false,
                    action: app::InputAction::OpenClawSetModelMaxTokens {
                        profile_id,
                        provider_id,
                        model_index,
                    },
                });
            }
            4 => {
                if let Err(e) =
                    openclaw_toggle_model_reasoning(app, &profile_id, &provider_id, model_index)
                {
                    app.set_toast(e.to_string(), true);
                } else {
                    refresh_openclaw_detail(app);
                }
            }
            5 => {
                if let Err(e) =
                    openclaw_toggle_model_input(app, &profile_id, &provider_id, model_index, "text")
                {
                    app.set_toast(e.to_string(), true);
                } else {
                    refresh_openclaw_detail(app);
                }
            }
            6 => {
                if let Err(e) = openclaw_toggle_model_input(
                    app,
                    &profile_id,
                    &provider_id,
                    model_index,
                    "image",
                ) {
                    app.set_toast(e.to_string(), true);
                } else {
                    refresh_openclaw_detail(app);
                }
            }
            _ => {}
        },
        _ => {}
    }

    None
}

fn openclaw_toggle_model_reasoning(
    app: &mut app::App,
    profile_id: &str,
    provider_id: &str,
    model_index: usize,
) -> anyhow::Result<()> {
    let mut profile =
        droidgear_core::openclaw::get_openclaw_profile_for_home(&app.home_dir, profile_id)
            .map_err(anyhow::Error::msg)?;
    let Some(provider) = profile.providers.get_mut(provider_id) else {
        return Err(anyhow::Error::msg("Provider not found"));
    };
    let Some(model) = provider.models.get_mut(model_index) else {
        return Err(anyhow::Error::msg("Model not found"));
    };
    model.reasoning = !model.reasoning;
    droidgear_core::openclaw::save_openclaw_profile_for_home(&app.home_dir, profile)
        .map_err(anyhow::Error::msg)?;
    Ok(())
}

fn openclaw_toggle_model_input(
    app: &mut app::App,
    profile_id: &str,
    provider_id: &str,
    model_index: usize,
    input_type: &str,
) -> anyhow::Result<()> {
    let mut profile =
        droidgear_core::openclaw::get_openclaw_profile_for_home(&app.home_dir, profile_id)
            .map_err(anyhow::Error::msg)?;
    let Some(provider) = profile.providers.get_mut(provider_id) else {
        return Err(anyhow::Error::msg("Provider not found"));
    };
    let Some(model) = provider.models.get_mut(model_index) else {
        return Err(anyhow::Error::msg("Model not found"));
    };
    if model.input.iter().any(|t| t == input_type) {
        model.input.retain(|t| t != input_type);
    } else {
        model.input.push(input_type.to_string());
        model.input.sort();
        model.input.dedup();
    }
    droidgear_core::openclaw::save_openclaw_profile_for_home(&app.home_dir, profile)
        .map_err(anyhow::Error::msg)?;
    Ok(())
}

fn handle_openclaw_helpers_key(app: &mut app::App, code: KeyCode) -> Option<Action> {
    let Some(profile_id) = app.openclaw_detail_id.clone() else {
        app.screen = app::Screen::OpenClaw;
        return None;
    };
    let Some(profile) = app.openclaw_detail.as_ref() else {
        return None;
    };

    match code {
        KeyCode::Esc | KeyCode::Char('q') => app.screen = app::Screen::OpenClawProfile,
        KeyCode::Down => {
            app.openclaw_helpers_field_index = app.openclaw_helpers_field_index.saturating_add(1)
        }
        KeyCode::Up => {
            app.openclaw_helpers_field_index = app.openclaw_helpers_field_index.saturating_sub(1)
        }
        KeyCode::Char('x') => {
            if let Err(e) = openclaw_reset_helpers(app, &profile_id) {
                app.set_toast(e.to_string(), true);
            } else {
                app.set_toast("Reset", false);
                refresh_openclaw_detail(app);
            }
        }
        KeyCode::Enter | KeyCode::Char('e') => match app.openclaw_helpers_field_index {
            0 => {
                let options = vec!["on".to_string(), "off".to_string()];
                let current = profile
                    .block_streaming_config
                    .as_ref()
                    .and_then(|c| c.block_streaming_default.as_deref())
                    .unwrap_or("on");
                let index = options.iter().position(|o| o == current).unwrap_or(0);
                app.modal = Some(app::Modal::Select {
                    title: "Block streaming default".to_string(),
                    options,
                    index,
                    action: app::SelectAction::OpenClawSetBlockStreamingDefault { id: profile_id },
                });
            }
            1 => {
                let options = vec!["text_end".to_string(), "message_end".to_string()];
                let current = profile
                    .block_streaming_config
                    .as_ref()
                    .and_then(|c| c.block_streaming_break.as_deref())
                    .unwrap_or("text_end");
                let index = options.iter().position(|o| o == current).unwrap_or(0);
                app.modal = Some(app::Modal::Select {
                    title: "Block streaming break".to_string(),
                    options,
                    index,
                    action: app::SelectAction::OpenClawSetBlockStreamingBreak { id: profile_id },
                });
            }
            2 => {
                let current = profile
                    .block_streaming_config
                    .as_ref()
                    .and_then(|c| c.block_streaming_chunk.as_ref())
                    .and_then(|c| c.min_chars)
                    .map(|v| v.to_string())
                    .unwrap_or_default();
                app.modal = Some(app::Modal::Input {
                    title: "Min chars".to_string(),
                    value: current,
                    cursor: usize::MAX,
                    is_secret: false,
                    action: app::InputAction::OpenClawSetBlockStreamingMinChars { profile_id },
                });
            }
            3 => {
                let current = profile
                    .block_streaming_config
                    .as_ref()
                    .and_then(|c| c.block_streaming_chunk.as_ref())
                    .and_then(|c| c.max_chars)
                    .map(|v| v.to_string())
                    .unwrap_or_default();
                app.modal = Some(app::Modal::Input {
                    title: "Max chars".to_string(),
                    value: current,
                    cursor: usize::MAX,
                    is_secret: false,
                    action: app::InputAction::OpenClawSetBlockStreamingMaxChars { profile_id },
                });
            }
            4 => {
                let current = profile
                    .block_streaming_config
                    .as_ref()
                    .and_then(|c| c.block_streaming_coalesce.as_ref())
                    .and_then(|c| c.idle_ms)
                    .map(|v| v.to_string())
                    .unwrap_or_default();
                app.modal = Some(app::Modal::Input {
                    title: "Idle ms".to_string(),
                    value: current,
                    cursor: usize::MAX,
                    is_secret: false,
                    action: app::InputAction::OpenClawSetBlockStreamingIdleMs { profile_id },
                });
            }
            5 => {
                if let Err(e) = openclaw_toggle_telegram_block_streaming(app, &profile_id) {
                    app.set_toast(e.to_string(), true);
                } else {
                    refresh_openclaw_detail(app);
                }
            }
            6 => {
                let options = vec!["newline".to_string(), "chars".to_string()];
                let current = profile
                    .block_streaming_config
                    .as_ref()
                    .and_then(|c| c.telegram_channel.as_ref())
                    .and_then(|t| t.chunk_mode.as_deref())
                    .unwrap_or("newline");
                let index = options.iter().position(|o| o == current).unwrap_or(0);
                app.modal = Some(app::Modal::Select {
                    title: "Telegram chunk mode".to_string(),
                    options,
                    index,
                    action: app::SelectAction::OpenClawSetTelegramChunkMode { id: profile_id },
                });
            }
            _ => {}
        },
        _ => {}
    }

    None
}

fn openclaw_toggle_telegram_block_streaming(
    app: &mut app::App,
    profile_id: &str,
) -> anyhow::Result<()> {
    let mut profile =
        droidgear_core::openclaw::get_openclaw_profile_for_home(&app.home_dir, profile_id)
            .map_err(anyhow::Error::msg)?;
    let cfg = profile.block_streaming_config.get_or_insert_with(|| {
        droidgear_core::openclaw::BlockStreamingConfig {
            block_streaming_default: Some("on".to_string()),
            block_streaming_break: Some("text_end".to_string()),
            block_streaming_chunk: Some(droidgear_core::openclaw::BlockStreamingChunk {
                min_chars: Some(200),
                max_chars: Some(600),
            }),
            block_streaming_coalesce: Some(droidgear_core::openclaw::BlockStreamingCoalesce {
                idle_ms: Some(200),
            }),
            telegram_channel: Some(droidgear_core::openclaw::TelegramChannelConfig {
                block_streaming: Some(true),
                chunk_mode: Some("newline".to_string()),
            }),
        }
    });
    let telegram = cfg.telegram_channel.get_or_insert_with(|| {
        droidgear_core::openclaw::TelegramChannelConfig {
            block_streaming: Some(true),
            chunk_mode: Some("newline".to_string()),
        }
    });
    let current = telegram.block_streaming.unwrap_or(true);
    telegram.block_streaming = Some(!current);
    droidgear_core::openclaw::save_openclaw_profile_for_home(&app.home_dir, profile)
        .map_err(anyhow::Error::msg)?;
    Ok(())
}

fn openclaw_reset_helpers(app: &mut app::App, profile_id: &str) -> anyhow::Result<()> {
    let mut profile =
        droidgear_core::openclaw::get_openclaw_profile_for_home(&app.home_dir, profile_id)
            .map_err(anyhow::Error::msg)?;
    profile.block_streaming_config = Some(droidgear_core::openclaw::BlockStreamingConfig {
        block_streaming_default: Some("on".to_string()),
        block_streaming_break: Some("text_end".to_string()),
        block_streaming_chunk: Some(droidgear_core::openclaw::BlockStreamingChunk {
            min_chars: Some(200),
            max_chars: Some(600),
        }),
        block_streaming_coalesce: Some(droidgear_core::openclaw::BlockStreamingCoalesce {
            idle_ms: Some(200),
        }),
        telegram_channel: Some(droidgear_core::openclaw::TelegramChannelConfig {
            block_streaming: Some(true),
            chunk_mode: Some("newline".to_string()),
        }),
    });
    droidgear_core::openclaw::save_openclaw_profile_for_home(&app.home_dir, profile)
        .map_err(anyhow::Error::msg)?;
    Ok(())
}

fn openclaw_subagent_allowed_ids(
    subagents: &[droidgear_core::openclaw::OpenClawSubAgent],
) -> std::collections::HashSet<String> {
    subagents
        .iter()
        .find(|a| a.id == "main")
        .and_then(|main| main.subagents.as_ref())
        .and_then(|sa| sa.allow_agents.as_ref())
        .map(|list| list.iter().cloned().collect())
        .unwrap_or_default()
}

fn handle_openclaw_subagents_key(app: &mut app::App, code: KeyCode) -> Option<Action> {
    // Filter non-main subagents for navigation
    let non_main: Vec<_> = app
        .openclaw_subagents
        .iter()
        .filter(|a| a.id != "main")
        .collect();
    let allowed = openclaw_subagent_allowed_ids(&app.openclaw_subagents);

    match code {
        KeyCode::Esc | KeyCode::Char('q') => app.screen = app::Screen::OpenClawProfile,
        KeyCode::Down => {
            app.openclaw_subagents_index = app.openclaw_subagents_index.saturating_add(1)
        }
        KeyCode::Up => {
            app.openclaw_subagents_index = app.openclaw_subagents_index.saturating_sub(1)
        }
        KeyCode::Char('r') => refresh_openclaw_subagents(app),
        KeyCode::Char('n') => {
            app.modal = Some(app::Modal::Input {
                title: "New subagent id".to_string(),
                value: String::new(),
                cursor: usize::MAX,
                is_secret: false,
                action: app::InputAction::OpenClawSubagentCreate,
            });
        }
        KeyCode::Char('d') => {
            if let Some(agent) = non_main.get(app.openclaw_subagents_index) {
                app.modal = Some(app::Modal::Confirm {
                    message: format!("Delete subagent '{}'?", agent.id),
                    action: app::ConfirmAction::OpenClawSubagentDelete {
                        id: agent.id.clone(),
                    },
                });
            }
        }
        KeyCode::Char('t') => {
            if let Some(agent) = non_main.get(app.openclaw_subagents_index) {
                let status = if allowed.contains(&agent.id) {
                    "disallow"
                } else {
                    "allow"
                };
                app.modal = Some(app::Modal::Confirm {
                    message: format!("{} subagent '{}'?", status, agent.id),
                    action: app::ConfirmAction::OpenClawSubagentToggleAllow {
                        id: agent.id.clone(),
                    },
                });
            }
        }
        KeyCode::Enter | KeyCode::Char('e') => {
            if let Some(agent) = non_main.get(app.openclaw_subagents_index) {
                app.openclaw_subagent_detail = Some((*agent).clone());
                app.openclaw_subagent_field_index = 0;
                app.screen = app::Screen::OpenClawSubagentDetail;
            }
        }
        _ => {}
    }
    None
}

fn handle_openclaw_subagent_detail_key(app: &mut app::App, code: KeyCode) -> Option<Action> {
    let Some(agent) = app.openclaw_subagent_detail.as_ref() else {
        app.screen = app::Screen::OpenClawSubagents;
        return None;
    };
    let agent_id = agent.id.clone();

    match code {
        KeyCode::Esc | KeyCode::Char('q') => {
            app.openclaw_subagent_detail = None;
            app.screen = app::Screen::OpenClawSubagents;
        }
        KeyCode::Down => {
            app.openclaw_subagent_field_index = app.openclaw_subagent_field_index.saturating_add(1)
        }
        KeyCode::Up => {
            app.openclaw_subagent_field_index = app.openclaw_subagent_field_index.saturating_sub(1)
        }
        KeyCode::Enter | KeyCode::Char('e') => match app.openclaw_subagent_field_index {
            0 => {
                app.modal = Some(app::Modal::Input {
                    title: "Name".to_string(),
                    value: agent.name.clone().unwrap_or_default(),
                    cursor: usize::MAX,
                    is_secret: false,
                    action: app::InputAction::OpenClawSubagentSetName { id: agent_id },
                });
            }
            1 => {
                app.modal = Some(app::Modal::Input {
                    title: "Emoji".to_string(),
                    value: agent
                        .identity
                        .as_ref()
                        .and_then(|i| i.emoji.clone())
                        .unwrap_or_default(),
                    cursor: usize::MAX,
                    is_secret: false,
                    action: app::InputAction::OpenClawSubagentSetEmoji { id: agent_id },
                });
            }
            2 => {
                app.modal = Some(app::Modal::Input {
                    title: "Primary model".to_string(),
                    value: agent
                        .model
                        .as_ref()
                        .and_then(|m| m.primary.clone())
                        .unwrap_or_default(),
                    cursor: usize::MAX,
                    is_secret: false,
                    action: app::InputAction::OpenClawSubagentSetPrimaryModel { id: agent_id },
                });
            }
            3 => {
                let options = vec!["full".to_string(), "read".to_string(), "none".to_string()];
                let current = agent.tools.as_ref().and_then(|t| t.profile.as_deref());
                let index = current
                    .and_then(|v| options.iter().position(|o| o == v))
                    .unwrap_or(0);
                app.modal = Some(app::Modal::Select {
                    title: "Tools profile".to_string(),
                    options,
                    index,
                    action: app::SelectAction::OpenClawSubagentSetToolsProfile { id: agent_id },
                });
            }
            4 => {
                app.modal = Some(app::Modal::Input {
                    title: "Workspace".to_string(),
                    value: agent.workspace.clone().unwrap_or_default(),
                    cursor: usize::MAX,
                    is_secret: false,
                    action: app::InputAction::OpenClawSubagentSetWorkspace { id: agent_id },
                });
            }
            _ => {}
        },
        _ => {}
    }
    None
}

fn openclaw_update_subagent(
    app: &mut app::App,
    id: &str,
    updater: impl FnOnce(&mut droidgear_core::openclaw::OpenClawSubAgent),
) -> anyhow::Result<()> {
    let mut subagents = droidgear_core::openclaw::read_openclaw_subagents_for_home(&app.home_dir)
        .map_err(anyhow::Error::msg)?;
    if let Some(agent) = subagents.iter_mut().find(|a| a.id == id) {
        updater(agent);
    }
    droidgear_core::openclaw::save_openclaw_subagents_for_home(&app.home_dir, subagents)
        .map_err(anyhow::Error::msg)?;
    refresh_openclaw_subagents(app);
    // Update detail if viewing
    if let Some(detail) = app.openclaw_subagent_detail.as_ref() {
        if detail.id == id {
            app.openclaw_subagent_detail =
                app.openclaw_subagents.iter().find(|a| a.id == id).cloned();
        }
    }
    Ok(())
}

fn handle_hermes_key(app: &mut app::App, code: KeyCode) -> Option<Action> {
    match code {
        KeyCode::Esc | KeyCode::Char('q') => app.screen = app::Screen::Main,
        KeyCode::Down => app.hermes_index = app.hermes_index.saturating_add(1),
        KeyCode::Up => app.hermes_index = app.hermes_index.saturating_sub(1),
        KeyCode::Char('r') => refresh_hermes(app),
        KeyCode::Char('n') => {
            app.modal = Some(app::Modal::Input {
                title: "New Hermes profile name".to_string(),
                value: String::new(),
                cursor: usize::MAX,
                is_secret: false,
                action: app::InputAction::HermesCreateProfile,
            });
        }
        KeyCode::Enter | KeyCode::Char('e') => {
            if let Some(p) = app.hermes_profiles.get(app.hermes_index) {
                app.hermes_detail_id = Some(p.id.clone());
                app.hermes_detail_field_index = 0;
                app.hermes_provider_field_index = 0;
                app.screen = app::Screen::HermesProfile;
                refresh_hermes_detail(app);
            }
        }
        KeyCode::Char('a') => {
            if let Some(p) = app.hermes_profiles.get(app.hermes_index) {
                app.modal = Some(app::Modal::Confirm {
                    message: format!("Apply Hermes profile '{}'?", p.name),
                    action: app::ConfirmAction::HermesApply { id: p.id.clone() },
                });
            }
        }
        KeyCode::Char('d') => {
            if let Some(p) = app.hermes_profiles.get(app.hermes_index) {
                app.modal = Some(app::Modal::Confirm {
                    message: format!("Delete Hermes profile '{}'?", p.name),
                    action: app::ConfirmAction::HermesDelete { id: p.id.clone() },
                });
            }
        }
        KeyCode::Char('c') => {
            if let Some(p) = app.hermes_profiles.get(app.hermes_index) {
                app.modal = Some(app::Modal::Input {
                    title: "Duplicate profile name".to_string(),
                    value: format!("{} (copy)", p.name),
                    cursor: usize::MAX,
                    is_secret: false,
                    action: app::InputAction::HermesDuplicate { id: p.id.clone() },
                });
            }
        }
        _ => {}
    }
    None
}

fn handle_hermes_profile_key(app: &mut app::App, code: KeyCode) -> Option<Action> {
    let Some(profile_id) = app.hermes_detail_id.clone() else {
        app.screen = app::Screen::Hermes;
        return None;
    };
    let Some(profile) = app.hermes_detail.as_ref() else {
        return None;
    };

    match code {
        KeyCode::Esc | KeyCode::Char('q') => {
            app.screen = app::Screen::Hermes;
        }
        KeyCode::Down => {
            app.hermes_detail_field_index = app.hermes_detail_field_index.saturating_add(1)
        }
        KeyCode::Up => {
            app.hermes_detail_field_index = app.hermes_detail_field_index.saturating_sub(1)
        }
        KeyCode::Char('r') => refresh_hermes_detail(app),
        KeyCode::Char('a') => {
            app.modal = Some(app::Modal::Confirm {
                message: format!("Apply Hermes profile '{}'?", profile.name),
                action: app::ConfirmAction::HermesApply {
                    id: profile_id.clone(),
                },
            });
        }
        KeyCode::Char('m') => {
            // Navigate to the model config (HermesProvider) screen
            app.hermes_provider_field_index = 0;
            app.screen = app::Screen::HermesProvider;
        }
        KeyCode::Char('l') => {
            if let Err(e) = hermes_load_from_live_config(app, &profile_id) {
                app.set_toast(e.to_string(), true);
            } else {
                app.set_toast("Loaded from live config", false);
                refresh_hermes_detail(app);
            }
        }
        KeyCode::Enter | KeyCode::Char('e') => {
            let profile_name = profile.name.clone();
            let model = profile.model.clone();
            match app.hermes_detail_field_index {
                0 => {
                    app.modal = Some(app::Modal::Input {
                        title: "Profile name".to_string(),
                        value: profile_name,
                        cursor: usize::MAX,
                        is_secret: false,
                        action: app::InputAction::HermesSetProfileName {
                            id: profile_id.clone(),
                        },
                    });
                }
                1 => {
                    app.modal = Some(app::Modal::Input {
                        title: "Profile description".to_string(),
                        value: app
                            .hermes_detail
                            .as_ref()
                            .and_then(|p| p.description.clone())
                            .unwrap_or_default(),
                        cursor: usize::MAX,
                        is_secret: false,
                        action: app::InputAction::HermesSetProfileDescription {
                            id: profile_id.clone(),
                        },
                    });
                }
                2 => {
                    app.modal = Some(app::Modal::Input {
                        title: "Default model".to_string(),
                        value: model.default.unwrap_or_default(),
                        cursor: usize::MAX,
                        is_secret: false,
                        action: app::InputAction::HermesSetProfileDefaultModel {
                            id: profile_id.clone(),
                        },
                    });
                }
                3 => {
                    app.modal = Some(app::Modal::Input {
                        title: "Provider".to_string(),
                        value: model.provider.unwrap_or_default(),
                        cursor: usize::MAX,
                        is_secret: false,
                        action: app::InputAction::HermesSetProfileProvider {
                            id: profile_id.clone(),
                        },
                    });
                }
                4 => {
                    app.modal = Some(app::Modal::Input {
                        title: "Base URL".to_string(),
                        value: model.base_url.unwrap_or_default(),
                        cursor: usize::MAX,
                        is_secret: false,
                        action: app::InputAction::HermesSetProfileBaseUrl {
                            id: profile_id.clone(),
                        },
                    });
                }
                5 => {
                    app.modal = Some(app::Modal::Input {
                        title: "API key".to_string(),
                        value: model.api_key.unwrap_or_default(),
                        cursor: usize::MAX,
                        is_secret: true,
                        action: app::InputAction::HermesSetProfileApiKey {
                            id: profile_id.clone(),
                        },
                    });
                }
                _ => {}
            }
        }
        _ => {}
    }

    None
}

fn handle_hermes_provider_key(app: &mut app::App, code: KeyCode) -> Option<Action> {
    let Some(_profile_id) = app.hermes_detail_id.clone() else {
        app.screen = app::Screen::Hermes;
        return None;
    };
    let Some(profile) = app.hermes_detail.as_ref() else {
        return None;
    };
    let model = profile.model.clone();
    let profile_id = app.hermes_detail_id.clone().unwrap_or_default();

    match code {
        KeyCode::Esc | KeyCode::Char('q') => {
            app.screen = app::Screen::HermesProfile;
        }
        KeyCode::Down => {
            app.hermes_provider_field_index = app.hermes_provider_field_index.saturating_add(1)
        }
        KeyCode::Up => {
            app.hermes_provider_field_index = app.hermes_provider_field_index.saturating_sub(1)
        }
        KeyCode::Enter | KeyCode::Char('e') => match app.hermes_provider_field_index {
            0 => {
                app.modal = Some(app::Modal::Input {
                    title: "Default model".to_string(),
                    value: model.default.unwrap_or_default(),
                    cursor: usize::MAX,
                    is_secret: false,
                    action: app::InputAction::HermesSetProfileDefaultModel { id: profile_id },
                });
            }
            1 => {
                app.modal = Some(app::Modal::Input {
                    title: "Provider".to_string(),
                    value: model.provider.unwrap_or_default(),
                    cursor: usize::MAX,
                    is_secret: false,
                    action: app::InputAction::HermesSetProfileProvider { id: profile_id },
                });
            }
            2 => {
                app.modal = Some(app::Modal::Input {
                    title: "Base URL".to_string(),
                    value: model.base_url.unwrap_or_default(),
                    cursor: usize::MAX,
                    is_secret: false,
                    action: app::InputAction::HermesSetProfileBaseUrl { id: profile_id },
                });
            }
            3 => {
                app.modal = Some(app::Modal::Input {
                    title: "API key".to_string(),
                    value: model.api_key.unwrap_or_default(),
                    cursor: usize::MAX,
                    is_secret: true,
                    action: app::InputAction::HermesSetProfileApiKey { id: profile_id },
                });
            }
            _ => {}
        },
        KeyCode::Char('i') => {
            // Import from channel: present channel list as a Select modal
            let options: Vec<String> = app
                .channels
                .iter()
                .filter(|c| c.enabled)
                .map(|c| format!("{} ({})", c.name, c.base_url))
                .collect();
            if options.is_empty() {
                app.set_toast("No channels configured", true);
            } else {
                app.modal = Some(app::Modal::Select {
                    title: "Import from channel".to_string(),
                    options,
                    index: 0,
                    action: app::SelectAction::HermesImportFromChannel { profile_id },
                });
            }
        }
        _ => {}
    }

    None
}

fn handle_sessions_key(app: &mut app::App, code: KeyCode) -> Option<Action> {
    match code {
        KeyCode::Esc | KeyCode::Char('q') => app.screen = app::Screen::Main,
        KeyCode::Down => app.sessions_index = app.sessions_index.saturating_add(1),
        KeyCode::Up => app.sessions_index = app.sessions_index.saturating_sub(1),
        KeyCode::Char('r') => refresh_sessions(app),
        KeyCode::Enter | KeyCode::Char('v') => {
            if let Some(s) = app.sessions.get(app.sessions_index) {
                return Some(Action::ViewSession {
                    path: s.path.clone(),
                });
            }
        }
        KeyCode::Char('d') => {
            if let Some(s) = app.sessions.get(app.sessions_index) {
                app.modal = Some(app::Modal::Confirm {
                    message: format!("Delete session '{}'?", s.title),
                    action: app::ConfirmAction::SessionDelete {
                        path: s.path.clone(),
                    },
                });
            }
        }
        _ => {}
    }
    None
}

fn handle_specs_key(app: &mut app::App, code: KeyCode) -> Option<Action> {
    match code {
        KeyCode::Esc | KeyCode::Char('q') => app.screen = app::Screen::Main,
        KeyCode::Down => app.specs_index = app.specs_index.saturating_add(1),
        KeyCode::Up => app.specs_index = app.specs_index.saturating_sub(1),
        KeyCode::Char('r') => refresh_specs(app),
        KeyCode::Enter | KeyCode::Char('e') => {
            if let Some(s) = app.specs.get(app.specs_index) {
                return Some(Action::EditSpec {
                    path: s.path.clone(),
                });
            }
        }
        KeyCode::Char('d') => {
            if let Some(s) = app.specs.get(app.specs_index) {
                app.modal = Some(app::Modal::Confirm {
                    message: format!("Delete spec '{}'?", s.name),
                    action: app::ConfirmAction::SpecDelete {
                        path: s.path.clone(),
                    },
                });
            }
        }
        _ => {}
    }
    None
}

fn handle_channels_key(app: &mut app::App, code: KeyCode) -> Option<Action> {
    match code {
        KeyCode::Esc | KeyCode::Char('q') => app.screen = app::Screen::Main,
        KeyCode::Down => app.channels_index = app.channels_index.saturating_add(1),
        KeyCode::Up => app.channels_index = app.channels_index.saturating_sub(1),
        KeyCode::Char('r') => refresh_channels(app),
        KeyCode::Char('n') => {
            let id = uuid::Uuid::new_v4().to_string();
            let created_at = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis() as f64;

            app.channels_edit_draft = Some(droidgear_core::channel::Channel {
                id,
                name: String::new(),
                channel_type: droidgear_core::channel::ChannelType::General,
                base_url: String::new(),
                enabled: true,
                created_at,
            });
            app.channels_edit_field_index = 0;
            app.channels_edit_username.clear();
            app.channels_edit_password.clear();
            app.channels_edit_api_key.clear();
            app.screen = app::Screen::ChannelsEdit;
        }
        KeyCode::Enter | KeyCode::Char('e') => {
            if let Some(c) = app.channels.get(app.channels_index).cloned() {
                app.channels_edit_draft = Some(c.clone());
                app.channels_edit_field_index = 0;
                load_channel_auth_into_edit_state(app, &c);
                app.screen = app::Screen::ChannelsEdit;
            }
        }
        KeyCode::Char('t') => {
            if let Some(c) = app.channels.get(app.channels_index) {
                let mut channels = app.channels.clone();
                if let Some(found) = channels.iter_mut().find(|x| x.id == c.id) {
                    found.enabled = !found.enabled;
                }
                if let Err(e) =
                    droidgear_core::channel::save_channels_for_home(&app.home_dir, channels.clone())
                {
                    app.set_toast(e, true);
                } else {
                    app.channels = channels;
                    app.set_toast("Saved", false);
                }
            }
        }
        KeyCode::Char('d') => {
            if let Some(c) = app.channels.get(app.channels_index) {
                app.modal = Some(app::Modal::Confirm {
                    message: format!("Delete channel '{}'?", c.name),
                    action: app::ConfirmAction::ChannelDelete { id: c.id.clone() },
                });
            }
        }
        KeyCode::Char('E') => return Some(Action::EditChannels),
        KeyCode::Char('A') => {
            if let Some(c) = app.channels.get(app.channels_index) {
                return Some(Action::EditChannelAuth { id: c.id.clone() });
            }
        }
        _ => {}
    }
    None
}

fn channel_type_uses_api_key(channel_type: &droidgear_core::channel::ChannelType) -> bool {
    matches!(
        channel_type,
        droidgear_core::channel::ChannelType::CliProxyApi
            | droidgear_core::channel::ChannelType::Ollama
            | droidgear_core::channel::ChannelType::General
    )
}

fn load_channel_auth_into_edit_state(
    app: &mut app::App,
    channel: &droidgear_core::channel::Channel,
) {
    app.channels_edit_username.clear();
    app.channels_edit_password.clear();
    app.channels_edit_api_key.clear();

    if channel_type_uses_api_key(&channel.channel_type) {
        match droidgear_core::channel::get_channel_api_key_for_home(&app.home_dir, &channel.id) {
            Ok(Some(key)) => app.channels_edit_api_key = key,
            Ok(None) => {}
            Err(e) => app.set_toast(e, true),
        }
    } else {
        match droidgear_core::channel::get_channel_credentials_for_home(&app.home_dir, &channel.id)
        {
            Ok(Some((username, password))) => {
                app.channels_edit_username = username;
                app.channels_edit_password = password;
            }
            Ok(None) => {}
            Err(e) => app.set_toast(e, true),
        }
    }
}

fn handle_channels_edit_key(app: &mut app::App, code: KeyCode) -> Option<Action> {
    let Some(draft) = app.channels_edit_draft.as_ref() else {
        app.screen = app::Screen::Channels;
        return None;
    };
    let uses_api_key = channel_type_uses_api_key(&draft.channel_type);

    match code {
        KeyCode::Esc | KeyCode::Char('q') => {
            app.channels_edit_draft = None;
            app.channels_edit_field_index = 0;
            app.channels_edit_username.clear();
            app.channels_edit_password.clear();
            app.channels_edit_api_key.clear();
            app.screen = app::Screen::Channels;
        }
        KeyCode::Down => {
            app.channels_edit_field_index = app.channels_edit_field_index.saturating_add(1)
        }
        KeyCode::Up => {
            app.channels_edit_field_index = app.channels_edit_field_index.saturating_sub(1)
        }
        KeyCode::Char('s') => {
            let Some(mut channel) = app.channels_edit_draft.clone() else {
                return None;
            };
            channel.name = channel.name.trim().to_string();
            channel.base_url = channel.base_url.trim().to_string();

            if channel.name.is_empty() {
                app.set_toast("Name is required", true);
                return None;
            }
            if channel.base_url.is_empty() {
                app.set_toast("Base URL is required", true);
                return None;
            }

            if channel_type_uses_api_key(&channel.channel_type) {
                let api_key = app.channels_edit_api_key.trim().to_string();
                if api_key.is_empty() {
                    app.set_toast("API key is required", true);
                    return None;
                }
                if let Err(e) = droidgear_core::channel::save_channel_api_key_for_home(
                    &app.home_dir,
                    &channel.id,
                    &api_key,
                ) {
                    app.set_toast(e, true);
                    return None;
                }
            } else {
                let username = app.channels_edit_username.trim().to_string();
                let password = app.channels_edit_password.trim().to_string();
                if username.is_empty() {
                    app.set_toast("Username is required", true);
                    return None;
                }
                if password.is_empty() {
                    app.set_toast("Password is required", true);
                    return None;
                }
                if let Err(e) = droidgear_core::channel::save_channel_credentials_for_home(
                    &app.home_dir,
                    &channel.id,
                    &username,
                    &password,
                ) {
                    app.set_toast(e, true);
                    return None;
                }
            }

            let mut channels = app.channels.clone();
            if let Some(idx) = channels.iter().position(|c| c.id == channel.id) {
                channels[idx] = channel.clone();
            } else {
                channels.push(channel.clone());
            }

            if let Err(e) =
                droidgear_core::channel::save_channels_for_home(&app.home_dir, channels.clone())
            {
                app.set_toast(e, true);
                return None;
            }

            app.channels_edit_draft = None;
            app.channels_edit_field_index = 0;
            app.channels_edit_username.clear();
            app.channels_edit_password.clear();
            app.channels_edit_api_key.clear();
            app.screen = app::Screen::Channels;
            refresh_channels(app);
            if let Some(idx) = app.channels.iter().position(|c| c.id == channel.id) {
                app.channels_index = idx;
            }
            app.set_toast("Saved", false);
        }
        KeyCode::Enter | KeyCode::Char('e') => match app.channels_edit_field_index {
            0 => {
                app.modal = Some(app::Modal::Input {
                    title: "Name".to_string(),
                    value: draft.name.clone(),
                    cursor: usize::MAX,
                    is_secret: false,
                    action: app::InputAction::ChannelsDraftSetName,
                });
            }
            1 => {
                let options = vec![
                    "new-api".to_string(),
                    "sub-2-api".to_string(),
                    "cli-proxy-api".to_string(),
                    "ollama".to_string(),
                    "general".to_string(),
                ];
                let index = match draft.channel_type {
                    droidgear_core::channel::ChannelType::NewApi => 0,
                    droidgear_core::channel::ChannelType::Sub2Api => 1,
                    droidgear_core::channel::ChannelType::CliProxyApi => 2,
                    droidgear_core::channel::ChannelType::Ollama => 3,
                    droidgear_core::channel::ChannelType::General => 4,
                };
                app.modal = Some(app::Modal::Select {
                    title: "Channel type".to_string(),
                    options,
                    index,
                    action: app::SelectAction::ChannelsDraftSetType,
                });
            }
            2 => {
                app.modal = Some(app::Modal::Input {
                    title: "Base URL".to_string(),
                    value: draft.base_url.clone(),
                    cursor: usize::MAX,
                    is_secret: false,
                    action: app::InputAction::ChannelsDraftSetBaseUrl,
                });
            }
            3 => {
                if let Some(channel) = app.channels_edit_draft.as_mut() {
                    channel.enabled = !channel.enabled;
                }
            }
            4 => {
                if uses_api_key {
                    app.modal = Some(app::Modal::Input {
                        title: "API key".to_string(),
                        value: app.channels_edit_api_key.clone(),
                        cursor: usize::MAX,
                        is_secret: true,
                        action: app::InputAction::ChannelsDraftSetApiKey,
                    });
                } else {
                    app.modal = Some(app::Modal::Input {
                        title: "Username".to_string(),
                        value: app.channels_edit_username.clone(),
                        cursor: usize::MAX,
                        is_secret: false,
                        action: app::InputAction::ChannelsDraftSetUsername,
                    });
                }
            }
            5 if !uses_api_key => {
                app.modal = Some(app::Modal::Input {
                    title: "Password".to_string(),
                    value: app.channels_edit_password.clone(),
                    cursor: usize::MAX,
                    is_secret: true,
                    action: app::InputAction::ChannelsDraftSetPassword,
                });
            }
            _ => {}
        },
        _ => {}
    }

    None
}

fn handle_missions_key(app: &mut app::App, code: KeyCode) -> Option<Action> {
    let effort_options = || {
        vec![
            "(not set)".to_string(),
            "none".to_string(),
            "low".to_string(),
            "medium".to_string(),
            "high".to_string(),
        ]
    };

    let model_options = |app: &app::App| -> Vec<String> {
        let mut opts = vec!["(not set)".to_string()];
        for m in &app.custom_models {
            let id = m.id.as_deref().unwrap_or(&m.model);
            opts.push(id.to_string());
        }
        opts
    };

    match code {
        KeyCode::Esc | KeyCode::Char('q') => app.screen = app::Screen::Main,
        KeyCode::Down => app.mission_field_index = app.mission_field_index.saturating_add(1),
        KeyCode::Up => app.mission_field_index = app.mission_field_index.saturating_sub(1),
        KeyCode::Char('r') => refresh_missions(app),
        KeyCode::Enter | KeyCode::Char('e') => match app.mission_field_index {
            0 => {
                let options = model_options(app);
                let current = app
                    .mission_settings
                    .worker_model
                    .as_deref()
                    .unwrap_or("(not set)");
                let index = options.iter().position(|o| o == current).unwrap_or(0);
                app.modal = Some(app::Modal::Select {
                    title: "Worker Model".to_string(),
                    options,
                    index,
                    action: app::SelectAction::MissionsSetWorkerModel,
                });
            }
            1 => {
                let options = effort_options();
                let current = app
                    .mission_settings
                    .worker_reasoning_effort
                    .as_deref()
                    .unwrap_or("(not set)");
                let index = options.iter().position(|o| o == current).unwrap_or(0);
                app.modal = Some(app::Modal::Select {
                    title: "Worker Reasoning Effort".to_string(),
                    options,
                    index,
                    action: app::SelectAction::MissionsSetWorkerReasoningEffort,
                });
            }
            2 => {
                let options = model_options(app);
                let current = app
                    .mission_settings
                    .validation_worker_model
                    .as_deref()
                    .unwrap_or("(not set)");
                let index = options.iter().position(|o| o == current).unwrap_or(0);
                app.modal = Some(app::Modal::Select {
                    title: "Validation Worker Model".to_string(),
                    options,
                    index,
                    action: app::SelectAction::MissionsSetValidationWorkerModel,
                });
            }
            3 => {
                let options = effort_options();
                let current = app
                    .mission_settings
                    .validation_worker_reasoning_effort
                    .as_deref()
                    .unwrap_or("(not set)");
                let index = options.iter().position(|o| o == current).unwrap_or(0);
                app.modal = Some(app::Modal::Select {
                    title: "Validation Worker Reasoning Effort".to_string(),
                    options,
                    index,
                    action: app::SelectAction::MissionsSetValidationWorkerReasoningEffort,
                });
            }
            _ => {}
        },
        _ => {}
    }

    None
}

fn run_action(app: &mut app::App, action: Action) -> anyhow::Result<()> {
    match action {
        Action::EditFactoryModels => edit_factory_models(app),
        Action::EditCodexProfile { id } => {
            let profile = droidgear_core::codex::get_codex_profile_for_home(&app.home_dir, &id)
                .map_err(anyhow::Error::msg)?;
            let edited = edit_json_in_editor(&profile)?;
            droidgear_core::codex::save_codex_profile_for_home(&app.home_dir, edited)
                .map_err(anyhow::Error::msg)?;
            app.set_toast("Saved", false);
            Ok(())
        }
        Action::EditOpenCodeProfile { id } => {
            let profile =
                droidgear_core::opencode::get_opencode_profile_for_home(&app.home_dir, &id)
                    .map_err(anyhow::Error::msg)?;
            let edited = edit_json_in_editor(&profile)?;
            droidgear_core::opencode::save_opencode_profile_for_home(&app.home_dir, edited)
                .map_err(anyhow::Error::msg)?;
            app.set_toast("Saved", false);
            Ok(())
        }
        Action::EditOpenClawProfile { id } => {
            let profile =
                droidgear_core::openclaw::get_openclaw_profile_for_home(&app.home_dir, &id)
                    .map_err(anyhow::Error::msg)?;
            let edited = edit_json_in_editor(&profile)?;
            droidgear_core::openclaw::save_openclaw_profile_for_home(&app.home_dir, edited)
                .map_err(anyhow::Error::msg)?;
            app.set_toast("Saved", false);
            Ok(())
        }
        Action::PreviewCodexApply { id } => {
            let diff = preview_codex_apply(&app.home_dir, &id)?;
            open_text_in_pager(&diff)?;
            Ok(())
        }
        Action::PreviewOpenCodeApply { id } => {
            let diff = preview_opencode_apply(&app.home_dir, &id)?;
            open_text_in_pager(&diff)?;
            Ok(())
        }
        Action::PreviewOpenClawApply { id } => {
            let diff = preview_openclaw_apply(&app.home_dir, &id)?;
            open_text_in_pager(&diff)?;
            Ok(())
        }
        Action::ViewSession { path } => {
            let detail =
                droidgear_core::sessions::get_session_detail_for_home(&app.home_dir, &path)
                    .map_err(anyhow::Error::msg)?;
            let text = format_session_detail(&detail);

            let mut temp = NamedTempFile::new().context("create temp file")?;
            temp.write_all(text.as_bytes()).context("write temp file")?;
            temp.flush().context("flush temp file")?;
            editor::open_in_pager(temp.path())?;
            Ok(())
        }
        Action::EditSpec { path } => {
            let path = PathBuf::from(path);
            editor::open_in_editor(&path)?;
            Ok(())
        }
        Action::EditChannels => {
            let channels = droidgear_core::channel::load_channels_for_home(&app.home_dir)
                .map_err(anyhow::Error::msg)?;
            let edited: Vec<droidgear_core::channel::Channel> = edit_json_in_editor(&channels)?;
            droidgear_core::channel::save_channels_for_home(&app.home_dir, edited)
                .map_err(anyhow::Error::msg)?;
            app.set_toast("Saved", false);
            Ok(())
        }
        Action::EditChannelAuth { id } => {
            let auth_dir = app.home_dir.join(".droidgear").join("auth");
            std::fs::create_dir_all(&auth_dir).context("create auth dir")?;
            let auth_path = auth_dir.join(format!("{id}.json"));
            if !auth_path.exists() {
                std::fs::write(
                    &auth_path,
                    "{\n  \"type\": \"api_key\",\n  \"api_key\": \"\"\n}\n",
                )
                .context("write auth template")?;
            }
            editor::open_in_editor(&auth_path)?;
            Ok(())
        }
    }
}

fn format_session_detail(detail: &droidgear_core::sessions::SessionDetail) -> String {
    let mut out = String::new();
    out.push_str(&format!("Title: {}\n", detail.title));
    out.push_str(&format!("Project: {}\n", detail.project));
    out.push_str(&format!("Model: {}\n", detail.model));
    out.push_str(&format!("CWD: {}\n", detail.cwd));
    out.push('\n');

    for m in &detail.messages {
        out.push_str(&format!("[{}] {}\n", m.role, m.timestamp));
        for block in &m.content {
            if let Some(text) = block.text.as_deref() {
                out.push_str(text);
                if !text.ends_with('\n') {
                    out.push('\n');
                }
            }
            if let Some(thinking) = block.thinking.as_deref() {
                out.push_str("(thinking)\n");
                out.push_str(thinking);
                if !thinking.ends_with('\n') {
                    out.push('\n');
                }
            }
        }
        out.push('\n');
    }

    out
}

fn edit_factory_models(app: &mut app::App) -> anyhow::Result<()> {
    let models = droidgear_core::factory_settings::load_custom_models_for_home(&app.home_dir)
        .map_err(anyhow::Error::msg)?;
    let edited: Vec<droidgear_core::factory_settings::CustomModel> = edit_json_in_editor(&models)?;
    droidgear_core::factory_settings::save_custom_models_for_home(&app.home_dir, edited)
        .map_err(anyhow::Error::msg)?;
    app.set_toast("Saved", false);
    Ok(())
}

fn edit_json_in_editor<T>(value: &T) -> anyhow::Result<T>
where
    T: Serialize + DeserializeOwned,
{
    let mut temp = NamedTempFile::new().context("create temp file")?;
    let content = serde_json::to_string_pretty(value).context("serialize JSON")?;
    temp.write_all(content.as_bytes())
        .context("write temp file")?;
    temp.flush().context("flush temp file")?;

    editor::open_in_editor(temp.path())?;

    let edited = std::fs::read_to_string(temp.path()).context("read edited file")?;
    let parsed = serde_json::from_str(&edited).context("parse edited JSON")?;
    Ok(parsed)
}

fn open_text_in_pager(text: &str) -> anyhow::Result<()> {
    let mut temp = NamedTempFile::new().context("create temp file")?;
    temp.write_all(text.as_bytes()).context("write temp file")?;
    temp.flush().context("flush temp file")?;
    editor::open_in_pager(temp.path())?;
    Ok(())
}

fn read_to_string_if_exists(path: &Path) -> anyhow::Result<Option<String>> {
    if path.exists() {
        Ok(Some(
            std::fs::read_to_string(path).with_context(|| format!("read {}", path.display()))?,
        ))
    } else {
        Ok(None)
    }
}

fn write_string(path: &Path, content: &str) -> anyhow::Result<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).with_context(|| format!("create {}", parent.display()))?;
    }
    std::fs::write(path, content).with_context(|| format!("write {}", path.display()))?;
    Ok(())
}

fn format_diff_report(title: &str, files: Vec<(String, Option<String>, Option<String>)>) -> String {
    let mut out = String::new();
    out.push_str(title);
    out.push_str("\n\n");

    let mut any = false;
    for (label, before, after) in files {
        if before.as_deref() == after.as_deref() {
            continue;
        }
        any = true;
        out.push_str(&format!("=== {label} ===\n"));

        let before_s = before.as_deref().unwrap_or("");
        let after_s = after.as_deref().unwrap_or("");
        let diff = TextDiff::from_lines(before_s, after_s);
        out.push_str(
            &diff
                .unified_diff()
                .header(&format!("{label} (before)"), &format!("{label} (after)"))
                .to_string(),
        );
        if !out.ends_with('\n') {
            out.push('\n');
        }
        out.push('\n');
    }

    if !any {
        out.push_str("No changes.\n");
    }

    out
}

fn preview_codex_apply(home_dir: &Path, profile_id: &str) -> anyhow::Result<String> {
    let status = droidgear_core::codex::get_codex_config_status_for_home(home_dir)
        .map_err(anyhow::Error::msg)?;
    let real_config_path = PathBuf::from(status.config_path);
    let real_auth_path = PathBuf::from(status.auth_path);
    let real_active_path = home_dir
        .join(".droidgear")
        .join("codex")
        .join("active-profile.txt");

    let before_config = read_to_string_if_exists(&real_config_path)?;
    let before_auth = read_to_string_if_exists(&real_auth_path)?;
    let before_active = read_to_string_if_exists(&real_active_path)?;

    let temp = TempDir::new().context("create temp home")?;
    let temp_home = temp.path();

    let temp_config_path = temp_home.join(".codex").join("config.toml");
    let temp_auth_path = temp_home.join(".codex").join("auth.json");
    if let Some(ref s) = before_config {
        write_string(&temp_config_path, s)?;
    }
    if let Some(ref s) = before_auth {
        write_string(&temp_auth_path, s)?;
    }

    let profile = droidgear_core::codex::get_codex_profile_for_home(home_dir, profile_id)
        .map_err(anyhow::Error::msg)?;
    droidgear_core::codex::save_codex_profile_for_home(temp_home, profile)
        .map_err(anyhow::Error::msg)?;
    droidgear_core::codex::apply_codex_profile_for_home(temp_home, profile_id)
        .map_err(anyhow::Error::msg)?;

    let after_config = read_to_string_if_exists(&temp_config_path)?;
    let after_auth = read_to_string_if_exists(&temp_auth_path)?;
    let temp_active_path = temp_home
        .join(".droidgear")
        .join("codex")
        .join("active-profile.txt");
    let after_active = read_to_string_if_exists(&temp_active_path)?;

    Ok(format_diff_report(
        "Codex apply preview",
        vec![
            (
                real_config_path.to_string_lossy().to_string(),
                before_config,
                after_config,
            ),
            (
                real_auth_path.to_string_lossy().to_string(),
                before_auth,
                after_auth,
            ),
            (
                real_active_path.to_string_lossy().to_string(),
                before_active,
                after_active,
            ),
        ],
    ))
}

fn preview_opencode_apply(home_dir: &Path, profile_id: &str) -> anyhow::Result<String> {
    let status = droidgear_core::opencode::get_opencode_config_status_for_home(home_dir)
        .map_err(anyhow::Error::msg)?;
    let real_config_path = PathBuf::from(status.config_path);
    let real_auth_path = PathBuf::from(status.auth_path);
    let real_active_path = home_dir
        .join(".droidgear")
        .join("opencode")
        .join("active-profile.txt");

    let before_config = read_to_string_if_exists(&real_config_path)?;
    let before_auth = read_to_string_if_exists(&real_auth_path)?;
    let before_active = read_to_string_if_exists(&real_active_path)?;

    let temp = TempDir::new().context("create temp home")?;
    let temp_home = temp.path();

    let config_file_name = real_config_path
        .file_name()
        .unwrap_or(std::ffi::OsStr::new("opencode.json"));
    let auth_file_name = real_auth_path
        .file_name()
        .unwrap_or(std::ffi::OsStr::new("auth.json"));

    let temp_config_path = temp_home
        .join(".config")
        .join("opencode")
        .join(config_file_name);
    let temp_auth_path = temp_home
        .join(".local")
        .join("share")
        .join("opencode")
        .join(auth_file_name);

    if let Some(ref s) = before_config {
        write_string(&temp_config_path, s)?;
    }
    if let Some(ref s) = before_auth {
        write_string(&temp_auth_path, s)?;
    }

    let profile = droidgear_core::opencode::get_opencode_profile_for_home(home_dir, profile_id)
        .map_err(anyhow::Error::msg)?;
    droidgear_core::opencode::save_opencode_profile_for_home(temp_home, profile)
        .map_err(anyhow::Error::msg)?;
    droidgear_core::opencode::apply_opencode_profile_for_home(temp_home, profile_id)
        .map_err(anyhow::Error::msg)?;

    let after_config = read_to_string_if_exists(&temp_config_path)?;
    let after_auth = read_to_string_if_exists(&temp_auth_path)?;
    let temp_active_path = temp_home
        .join(".droidgear")
        .join("opencode")
        .join("active-profile.txt");
    let after_active = read_to_string_if_exists(&temp_active_path)?;

    Ok(format_diff_report(
        "OpenCode apply preview",
        vec![
            (
                real_config_path.to_string_lossy().to_string(),
                before_config,
                after_config,
            ),
            (
                real_auth_path.to_string_lossy().to_string(),
                before_auth,
                after_auth,
            ),
            (
                real_active_path.to_string_lossy().to_string(),
                before_active,
                after_active,
            ),
        ],
    ))
}

fn preview_openclaw_apply(home_dir: &Path, profile_id: &str) -> anyhow::Result<String> {
    let status = droidgear_core::openclaw::get_openclaw_config_status_for_home(home_dir)
        .map_err(anyhow::Error::msg)?;
    let real_config_path = PathBuf::from(status.config_path);
    let real_active_path = home_dir
        .join(".droidgear")
        .join("openclaw")
        .join("active-profile.txt");

    let before_config = read_to_string_if_exists(&real_config_path)?;
    let before_active = read_to_string_if_exists(&real_active_path)?;

    let temp = TempDir::new().context("create temp home")?;
    let temp_home = temp.path();

    let temp_config_path = temp_home.join(".openclaw").join("openclaw.json");
    if let Some(ref s) = before_config {
        write_string(&temp_config_path, s)?;
    }

    let profile = droidgear_core::openclaw::get_openclaw_profile_for_home(home_dir, profile_id)
        .map_err(anyhow::Error::msg)?;
    droidgear_core::openclaw::save_openclaw_profile_for_home(temp_home, profile)
        .map_err(anyhow::Error::msg)?;
    droidgear_core::openclaw::apply_openclaw_profile_for_home(temp_home, profile_id)
        .map_err(anyhow::Error::msg)?;

    let after_config = read_to_string_if_exists(&temp_config_path)?;
    let temp_active_path = temp_home
        .join(".droidgear")
        .join("openclaw")
        .join("active-profile.txt");
    let after_active = read_to_string_if_exists(&temp_active_path)?;

    Ok(format_diff_report(
        "OpenClaw apply preview",
        vec![
            (
                real_config_path.to_string_lossy().to_string(),
                before_config,
                after_config,
            ),
            (
                real_active_path.to_string_lossy().to_string(),
                before_active,
                after_active,
            ),
        ],
    ))
}

fn byte_index_for_char(value: &str, char_idx: usize) -> usize {
    value
        .char_indices()
        .nth(char_idx)
        .map(|(i, _)| i)
        .unwrap_or(value.len())
}

fn remove_char_at(value: &mut String, char_idx: usize) {
    let byte_idx = byte_index_for_char(value, char_idx);
    if byte_idx < value.len() {
        value.remove(byte_idx);
    }
}

fn insert_char_at(value: &mut String, char_idx: usize, c: char) {
    let byte_idx = byte_index_for_char(value, char_idx);
    value.insert(byte_idx, c);
}

fn handle_modal_key(app: &mut app::App, code: KeyCode, modal: app::Modal) {
    match modal {
        app::Modal::Confirm { action, .. } => match code {
            KeyCode::Char('y') | KeyCode::Enter => {
                app.modal = None;
                if let Err(e) = run_confirm_action(app, action) {
                    app.set_toast(e.to_string(), true);
                } else {
                    refresh_screen_data(app);
                }
            }
            KeyCode::Char('n') | KeyCode::Esc => {
                app.modal = None;
            }
            _ => {}
        },
        app::Modal::Input {
            title,
            mut value,
            mut cursor,
            is_secret,
            action,
        } => {
            let value_len = value.chars().count();
            cursor = cursor.min(value_len);

            match code {
                KeyCode::Esc => app.modal = None,
                KeyCode::Enter => {
                    app.modal = None;
                    if let Err(e) = run_input_action(app, action, value) {
                        app.set_toast(e.to_string(), true);
                    } else {
                        refresh_screen_data(app);
                    }
                }

                // Cursor movement
                KeyCode::Left => {
                    cursor = cursor.saturating_sub(1);
                    app.modal = Some(app::Modal::Input {
                        title,
                        value,
                        cursor,
                        is_secret,
                        action,
                    });
                }
                KeyCode::Right => {
                    cursor = cursor.saturating_add(1).min(value_len);
                    app.modal = Some(app::Modal::Input {
                        title,
                        value,
                        cursor,
                        is_secret,
                        action,
                    });
                }
                KeyCode::Home => {
                    cursor = 0;
                    app.modal = Some(app::Modal::Input {
                        title,
                        value,
                        cursor,
                        is_secret,
                        action,
                    });
                }
                KeyCode::End => {
                    cursor = value_len;
                    app.modal = Some(app::Modal::Input {
                        title,
                        value,
                        cursor,
                        is_secret,
                        action,
                    });
                }

                // Editing
                KeyCode::Backspace if cursor > 0 => {
                    remove_char_at(&mut value, cursor.saturating_sub(1));
                    cursor = cursor.saturating_sub(1);
                    app.modal = Some(app::Modal::Input {
                        title,
                        value,
                        cursor,
                        is_secret,
                        action,
                    });
                }
                KeyCode::Delete if cursor < value_len => {
                    remove_char_at(&mut value, cursor);
                    app.modal = Some(app::Modal::Input {
                        title,
                        value,
                        cursor,
                        is_secret,
                        action,
                    });
                }
                KeyCode::Char(c) if !c.is_control() => {
                    // Default: insert mode (non-destructive).
                    insert_char_at(&mut value, cursor, c);
                    cursor = cursor.saturating_add(1);
                    app.modal = Some(app::Modal::Input {
                        title,
                        value,
                        cursor,
                        is_secret,
                        action,
                    });
                }

                _ => {}
            }
        }
        app::Modal::Select {
            title,
            options,
            mut index,
            action,
        } => match code {
            KeyCode::Esc => app.modal = None,
            KeyCode::Enter => {
                let selected = options.get(index).cloned();
                app.modal = None;
                if let Err(e) = run_select_action(app, action, index, selected) {
                    app.set_toast(e.to_string(), true);
                } else {
                    refresh_screen_data(app);
                }
            }
            KeyCode::Up => {
                index = index.saturating_sub(1);
                app.modal = Some(app::Modal::Select {
                    title,
                    options,
                    index,
                    action,
                });
            }
            KeyCode::Down => {
                index = index.saturating_add(1);
                if !options.is_empty() {
                    index = index.min(options.len().saturating_sub(1));
                } else {
                    index = 0;
                }
                app.modal = Some(app::Modal::Select {
                    title,
                    options,
                    index,
                    action,
                });
            }
            _ => {}
        },
    }
}

fn run_select_action(
    app: &mut app::App,
    action: app::SelectAction,
    index: usize,
    selected: Option<String>,
) -> anyhow::Result<()> {
    match action {
        app::SelectAction::GoToNav => {
            app.nav_index = index.min(app::App::nav_items().len().saturating_sub(1));
            if let Some((_, screen)) = app::App::nav_items().get(app.nav_index) {
                app.screen = *screen;
                app.clear_toast();
                refresh_screen_data(app);
            }
            Ok(())
        }
        app::SelectAction::CodexSetProfileModelProvider { id } => {
            let Some(selected) = selected else {
                return Ok(());
            };
            let mut profile = droidgear_core::codex::get_codex_profile_for_home(&app.home_dir, &id)
                .map_err(anyhow::Error::msg)?;
            profile.model_provider = selected;
            droidgear_core::codex::save_codex_profile_for_home(&app.home_dir, profile)
                .map_err(anyhow::Error::msg)?;
            app.set_toast("Saved", false);
            Ok(())
        }
        app::SelectAction::CodexSetProfileReasoningEffort { id } => {
            let mut profile = droidgear_core::codex::get_codex_profile_for_home(&app.home_dir, &id)
                .map_err(anyhow::Error::msg)?;
            profile.model_reasoning_effort = match selected.as_deref() {
                Some("(none)") | None => None,
                Some(v) => Some(v.to_string()),
            };
            droidgear_core::codex::save_codex_profile_for_home(&app.home_dir, profile)
                .map_err(anyhow::Error::msg)?;
            app.set_toast("Saved", false);
            Ok(())
        }
        app::SelectAction::CodexSetProviderWireApi {
            profile_id,
            provider_id,
        } => {
            let Some(selected) = selected else {
                return Ok(());
            };
            let mut profile =
                droidgear_core::codex::get_codex_profile_for_home(&app.home_dir, &profile_id)
                    .map_err(anyhow::Error::msg)?;
            if let Some(provider) = profile.providers.get_mut(&provider_id) {
                provider.wire_api = Some(selected);
            } else {
                return Err(anyhow::Error::msg("Provider not found"));
            }
            droidgear_core::codex::save_codex_profile_for_home(&app.home_dir, profile)
                .map_err(anyhow::Error::msg)?;
            app.set_toast("Saved", false);
            Ok(())
        }
        app::SelectAction::CodexSetProviderReasoningEffort {
            profile_id,
            provider_id,
        } => {
            let mut profile =
                droidgear_core::codex::get_codex_profile_for_home(&app.home_dir, &profile_id)
                    .map_err(anyhow::Error::msg)?;
            if let Some(provider) = profile.providers.get_mut(&provider_id) {
                provider.model_reasoning_effort = match selected.as_deref() {
                    Some("(none)") | None => None,
                    Some(v) => Some(v.to_string()),
                };
            } else {
                return Err(anyhow::Error::msg("Provider not found"));
            }
            droidgear_core::codex::save_codex_profile_for_home(&app.home_dir, profile)
                .map_err(anyhow::Error::msg)?;
            app.set_toast("Saved", false);
            Ok(())
        }
        app::SelectAction::OpenCodeImportProviders { id } => {
            let strategy = match selected.as_deref() {
                Some("replace") => "replace",
                _ => "skip",
            };
            let live =
                droidgear_core::opencode::read_opencode_current_config_for_home(&app.home_dir)
                    .map_err(anyhow::Error::msg)?;

            if live.providers.is_empty() {
                return Err(anyhow::Error::msg("No providers found in live config"));
            }

            let mut profile =
                droidgear_core::opencode::get_opencode_profile_for_home(&app.home_dir, &id)
                    .map_err(anyhow::Error::msg)?;

            for (provider_id, cfg) in live.providers {
                let exists = profile.providers.contains_key(&provider_id);
                if exists && strategy == "skip" {
                    continue;
                }
                profile.providers.insert(provider_id, cfg);
            }
            for (provider_id, auth) in live.auth {
                let exists = profile.auth.contains_key(&provider_id);
                if exists && strategy == "skip" {
                    continue;
                }
                profile.auth.insert(provider_id, auth);
            }

            droidgear_core::opencode::save_opencode_profile_for_home(&app.home_dir, profile)
                .map_err(anyhow::Error::msg)?;
            app.set_toast("Imported", false);
            Ok(())
        }
        app::SelectAction::OpenClawSetDefaultModel { id } => {
            let mut profile =
                droidgear_core::openclaw::get_openclaw_profile_for_home(&app.home_dir, &id)
                    .map_err(anyhow::Error::msg)?;
            profile.default_model = match selected.as_deref() {
                Some("(none)") | None => None,
                Some(v) => Some(v.to_string()),
            };
            droidgear_core::openclaw::save_openclaw_profile_for_home(&app.home_dir, profile)
                .map_err(anyhow::Error::msg)?;
            app.set_toast("Saved", false);
            Ok(())
        }
        app::SelectAction::OpenClawAddFailoverModel { id } => {
            let Some(selected) = selected else {
                return Ok(());
            };
            let mut profile =
                droidgear_core::openclaw::get_openclaw_profile_for_home(&app.home_dir, &id)
                    .map_err(anyhow::Error::msg)?;
            let mut list = profile.failover_models.take().unwrap_or_default();
            if !list.iter().any(|r| r == &selected) {
                list.push(selected);
            }
            profile.failover_models = (!list.is_empty()).then_some(list);
            droidgear_core::openclaw::save_openclaw_profile_for_home(&app.home_dir, profile)
                .map_err(anyhow::Error::msg)?;
            app.set_toast("Saved", false);
            Ok(())
        }
        app::SelectAction::OpenClawSetProviderApiType {
            profile_id,
            provider_id,
        } => {
            let Some(selected) = selected else {
                return Ok(());
            };
            let mut profile =
                droidgear_core::openclaw::get_openclaw_profile_for_home(&app.home_dir, &profile_id)
                    .map_err(anyhow::Error::msg)?;
            let Some(provider) = profile.providers.get_mut(&provider_id) else {
                return Err(anyhow::Error::msg("Provider not found"));
            };
            provider.api = Some(selected);
            droidgear_core::openclaw::save_openclaw_profile_for_home(&app.home_dir, profile)
                .map_err(anyhow::Error::msg)?;
            app.set_toast("Saved", false);
            Ok(())
        }
        app::SelectAction::OpenClawSetBlockStreamingDefault { id } => {
            let Some(selected) = selected else {
                return Ok(());
            };
            let mut profile =
                droidgear_core::openclaw::get_openclaw_profile_for_home(&app.home_dir, &id)
                    .map_err(anyhow::Error::msg)?;
            let cfg = profile.block_streaming_config.get_or_insert({
                droidgear_core::openclaw::BlockStreamingConfig {
                    block_streaming_default: None,
                    block_streaming_break: None,
                    block_streaming_chunk: None,
                    block_streaming_coalesce: None,
                    telegram_channel: None,
                }
            });
            cfg.block_streaming_default = Some(selected);
            droidgear_core::openclaw::save_openclaw_profile_for_home(&app.home_dir, profile)
                .map_err(anyhow::Error::msg)?;
            app.set_toast("Saved", false);
            Ok(())
        }
        app::SelectAction::OpenClawSetBlockStreamingBreak { id } => {
            let Some(selected) = selected else {
                return Ok(());
            };
            let mut profile =
                droidgear_core::openclaw::get_openclaw_profile_for_home(&app.home_dir, &id)
                    .map_err(anyhow::Error::msg)?;
            let cfg = profile.block_streaming_config.get_or_insert({
                droidgear_core::openclaw::BlockStreamingConfig {
                    block_streaming_default: None,
                    block_streaming_break: None,
                    block_streaming_chunk: None,
                    block_streaming_coalesce: None,
                    telegram_channel: None,
                }
            });
            cfg.block_streaming_break = Some(selected);
            droidgear_core::openclaw::save_openclaw_profile_for_home(&app.home_dir, profile)
                .map_err(anyhow::Error::msg)?;
            app.set_toast("Saved", false);
            Ok(())
        }
        app::SelectAction::OpenClawSetTelegramChunkMode { id } => {
            let Some(selected) = selected else {
                return Ok(());
            };
            let mut profile =
                droidgear_core::openclaw::get_openclaw_profile_for_home(&app.home_dir, &id)
                    .map_err(anyhow::Error::msg)?;
            let cfg = profile.block_streaming_config.get_or_insert({
                droidgear_core::openclaw::BlockStreamingConfig {
                    block_streaming_default: None,
                    block_streaming_break: None,
                    block_streaming_chunk: None,
                    block_streaming_coalesce: None,
                    telegram_channel: None,
                }
            });
            let telegram = cfg.telegram_channel.get_or_insert({
                droidgear_core::openclaw::TelegramChannelConfig {
                    block_streaming: None,
                    chunk_mode: None,
                }
            });
            telegram.chunk_mode = Some(selected);
            droidgear_core::openclaw::save_openclaw_profile_for_home(&app.home_dir, profile)
                .map_err(anyhow::Error::msg)?;
            app.set_toast("Saved", false);
            Ok(())
        }
        app::SelectAction::FactoryDraftSetProvider => {
            let Some(selected) = selected else {
                return Ok(());
            };
            let Some(draft) = app.factory_draft.as_mut() else {
                return Ok(());
            };

            let (provider, default_base_url) = match selected.as_str() {
                "anthropic" => (
                    droidgear_core::factory_settings::Provider::Anthropic,
                    "https://api.anthropic.com",
                ),
                "generic-chat-completion-api" => (
                    droidgear_core::factory_settings::Provider::GenericChatCompletionApi,
                    "",
                ),
                _ => (
                    droidgear_core::factory_settings::Provider::Openai,
                    "https://api.openai.com",
                ),
            };

            draft.provider = provider;
            if draft.base_url.trim().is_empty() {
                draft.base_url = default_base_url.to_string();
            }
            Ok(())
        }
        app::SelectAction::FactoryDraftSetReasoningEffort => {
            let Some(selected) = selected else {
                return Ok(());
            };
            let Some(draft) = app.factory_draft.as_mut() else {
                return Ok(());
            };

            if selected == "none" {
                // Remove reasoning from extra_args
                if let Some(args) = draft.extra_args.as_mut() {
                    args.remove("reasoning");
                    if args.is_empty() {
                        draft.extra_args = None;
                    }
                }
            } else {
                let args = draft
                    .extra_args
                    .get_or_insert_with(std::collections::HashMap::new);
                args.insert(
                    "reasoning".to_string(),
                    serde_json::json!({ "effort": selected }),
                );
            }
            Ok(())
        }
        app::SelectAction::McpDraftSetType => {
            let Some(selected) = selected else {
                return Ok(());
            };
            let Some(server) = app.mcp_edit_draft.as_mut() else {
                return Ok(());
            };

            let server_type = match selected.as_str() {
                "http" => droidgear_core::mcp::McpServerType::Http,
                _ => droidgear_core::mcp::McpServerType::Stdio,
            };
            server.config.server_type = server_type.clone();

            match server_type {
                droidgear_core::mcp::McpServerType::Stdio => {
                    server.config.url = None;
                    server.config.headers = None;
                }
                droidgear_core::mcp::McpServerType::Http => {
                    server.config.command = None;
                    server.config.args = None;
                    server.config.env = None;
                }
            }

            Ok(())
        }
        app::SelectAction::ChannelsDraftSetType => {
            let Some(selected) = selected else {
                return Ok(());
            };
            let Some(channel) = app.channels_edit_draft.as_mut() else {
                return Ok(());
            };

            let default_base_url = |t: &droidgear_core::channel::ChannelType| match t {
                droidgear_core::channel::ChannelType::NewApi => "https://api.newapi.ai",
                droidgear_core::channel::ChannelType::Sub2Api => "",
                droidgear_core::channel::ChannelType::CliProxyApi => "",
                droidgear_core::channel::ChannelType::Ollama => "http://localhost:11434",
                droidgear_core::channel::ChannelType::General => "",
            };

            let old_default = default_base_url(&channel.channel_type);
            let is_existing = app.channels.iter().any(|c| c.id == channel.id);
            let should_set_default = !is_existing
                && (channel.base_url.trim().is_empty() || channel.base_url.trim() == old_default);

            let new_type = match selected.as_str() {
                "new-api" => droidgear_core::channel::ChannelType::NewApi,
                "sub-2-api" => droidgear_core::channel::ChannelType::Sub2Api,
                "cli-proxy-api" => droidgear_core::channel::ChannelType::CliProxyApi,
                "ollama" => droidgear_core::channel::ChannelType::Ollama,
                _ => droidgear_core::channel::ChannelType::General,
            };

            channel.channel_type = new_type.clone();
            if should_set_default {
                channel.base_url = default_base_url(&new_type).to_string();
            }

            Ok(())
        }
        app::SelectAction::OpenClawSubagentSetToolsProfile { id } => {
            let Some(selected) = selected else {
                return Ok(());
            };
            let mut subagents =
                droidgear_core::openclaw::read_openclaw_subagents_for_home(&app.home_dir)
                    .map_err(anyhow::Error::msg)?;
            if let Some(agent) = subagents.iter_mut().find(|a| a.id == id) {
                let profile = if selected.is_empty() || selected == "(none)" {
                    None
                } else {
                    Some(selected)
                };
                agent.tools = Some(droidgear_core::openclaw::OpenClawSubAgentTools { profile });
            }
            droidgear_core::openclaw::save_openclaw_subagents_for_home(&app.home_dir, subagents)
                .map_err(anyhow::Error::msg)?;
            refresh_openclaw_subagents(app);
            // Update detail if viewing
            if let Some(detail) = app.openclaw_subagent_detail.as_ref() {
                if detail.id == id {
                    app.openclaw_subagent_detail =
                        app.openclaw_subagents.iter().find(|a| a.id == id).cloned();
                }
            }
            Ok(())
        }
        app::SelectAction::MissionsSetWorkerModel => {
            let Some(selected) = selected else {
                return Ok(());
            };
            app.mission_settings.worker_model = if selected == "(not set)" {
                None
            } else {
                Some(selected)
            };
            droidgear_core::factory_settings::save_mission_model_settings_for_home(
                &app.home_dir,
                app.mission_settings.clone(),
            )
            .map_err(anyhow::Error::msg)?;
            Ok(())
        }
        app::SelectAction::MissionsSetWorkerReasoningEffort => {
            let Some(selected) = selected else {
                return Ok(());
            };
            app.mission_settings.worker_reasoning_effort = if selected == "(not set)" {
                None
            } else {
                Some(selected)
            };
            droidgear_core::factory_settings::save_mission_model_settings_for_home(
                &app.home_dir,
                app.mission_settings.clone(),
            )
            .map_err(anyhow::Error::msg)?;
            Ok(())
        }
        app::SelectAction::MissionsSetValidationWorkerModel => {
            let Some(selected) = selected else {
                return Ok(());
            };
            app.mission_settings.validation_worker_model = if selected == "(not set)" {
                None
            } else {
                Some(selected)
            };
            droidgear_core::factory_settings::save_mission_model_settings_for_home(
                &app.home_dir,
                app.mission_settings.clone(),
            )
            .map_err(anyhow::Error::msg)?;
            Ok(())
        }
        app::SelectAction::MissionsSetValidationWorkerReasoningEffort => {
            let Some(selected) = selected else {
                return Ok(());
            };
            app.mission_settings.validation_worker_reasoning_effort = if selected == "(not set)" {
                None
            } else {
                Some(selected)
            };
            droidgear_core::factory_settings::save_mission_model_settings_for_home(
                &app.home_dir,
                app.mission_settings.clone(),
            )
            .map_err(anyhow::Error::msg)?;
            Ok(())
        }
        app::SelectAction::HermesImportFromChannel { profile_id } => {
            let Some(selected) = selected else {
                return Ok(());
            };
            // Find the matching channel by reconstructing the display string
            let channel = app
                .channels
                .iter()
                .find(|c| c.enabled && format!("{} ({})", c.name, c.base_url) == selected);
            let Some(channel) = channel else {
                return Err(anyhow::anyhow!("Channel not found"));
            };
            // Store channel info as pending import state; prompt for API key next
            app.hermes_import_pending_base_url = Some(channel.base_url.clone());
            app.hermes_import_pending_provider = Some("custom".to_string());
            app.modal = Some(app::Modal::Input {
                title: "API key for import".to_string(),
                value: String::new(),
                cursor: usize::MAX,
                is_secret: true,
                action: app::InputAction::HermesImportSetApiKey { id: profile_id },
            });
            Ok(())
        }
    }
}

fn run_confirm_action(app: &mut app::App, action: app::ConfirmAction) -> anyhow::Result<()> {
    match action {
        app::ConfirmAction::Quit => {
            app.should_quit = true;
            Ok(())
        }
        app::ConfirmAction::PathsResetKey { key } => {
            droidgear_core::paths::reset_config_path_for_home(&app.home_dir, &key)
                .map_err(anyhow::Error::msg)?;
            Ok(())
        }
        app::ConfirmAction::CodexApply { id } => {
            droidgear_core::codex::apply_codex_profile_for_home(&app.home_dir, &id)
                .map_err(anyhow::Error::msg)?;
            app.set_toast("Applied", false);
            Ok(())
        }
        app::ConfirmAction::CodexDelete { id } => {
            droidgear_core::codex::delete_codex_profile_for_home(&app.home_dir, &id)
                .map_err(anyhow::Error::msg)?;
            Ok(())
        }
        app::ConfirmAction::CodexDeleteProvider {
            profile_id,
            provider_id,
        } => {
            let mut profile =
                droidgear_core::codex::get_codex_profile_for_home(&app.home_dir, &profile_id)
                    .map_err(anyhow::Error::msg)?;
            profile.providers.remove(&provider_id);
            if profile.model_provider == provider_id {
                let mut provider_ids = profile.providers.keys().cloned().collect::<Vec<String>>();
                provider_ids.sort_by_key(|a| a.to_lowercase());
                profile.model_provider = provider_ids
                    .first()
                    .cloned()
                    .unwrap_or_else(|| "custom".to_string());
            }
            droidgear_core::codex::save_codex_profile_for_home(&app.home_dir, profile)
                .map_err(anyhow::Error::msg)?;
            Ok(())
        }
        app::ConfirmAction::OpenCodeApply { id } => {
            droidgear_core::opencode::apply_opencode_profile_for_home(&app.home_dir, &id)
                .map_err(anyhow::Error::msg)?;
            app.set_toast("Applied", false);
            Ok(())
        }
        app::ConfirmAction::OpenCodeDelete { id } => {
            droidgear_core::opencode::delete_opencode_profile_for_home(&app.home_dir, &id)
                .map_err(anyhow::Error::msg)?;
            Ok(())
        }
        app::ConfirmAction::OpenCodeDeleteProvider {
            profile_id,
            provider_id,
        } => {
            let mut profile =
                droidgear_core::opencode::get_opencode_profile_for_home(&app.home_dir, &profile_id)
                    .map_err(anyhow::Error::msg)?;
            profile.providers.remove(&provider_id);
            profile.auth.remove(&provider_id);
            droidgear_core::opencode::save_opencode_profile_for_home(&app.home_dir, profile)
                .map_err(anyhow::Error::msg)?;
            Ok(())
        }
        app::ConfirmAction::OpenCodeDeleteModel {
            profile_id,
            provider_id,
            model_id,
        } => {
            let mut profile =
                droidgear_core::opencode::get_opencode_profile_for_home(&app.home_dir, &profile_id)
                    .map_err(anyhow::Error::msg)?;
            let Some(provider) = profile.providers.get_mut(&provider_id) else {
                return Err(anyhow::Error::msg("Provider not found"));
            };
            if let Some(models) = provider.models.as_mut() {
                models.remove(&model_id);
                if models.is_empty() {
                    provider.models = None;
                }
            }
            droidgear_core::opencode::save_opencode_profile_for_home(&app.home_dir, profile)
                .map_err(anyhow::Error::msg)?;
            Ok(())
        }
        app::ConfirmAction::OpenClawApply { id } => {
            droidgear_core::openclaw::apply_openclaw_profile_for_home(&app.home_dir, &id)
                .map_err(anyhow::Error::msg)?;
            app.set_toast("Applied", false);
            Ok(())
        }
        app::ConfirmAction::OpenClawDelete { id } => {
            droidgear_core::openclaw::delete_openclaw_profile_for_home(&app.home_dir, &id)
                .map_err(anyhow::Error::msg)?;
            Ok(())
        }
        app::ConfirmAction::OpenClawDeleteProvider {
            profile_id,
            provider_id,
        } => {
            let mut profile =
                droidgear_core::openclaw::get_openclaw_profile_for_home(&app.home_dir, &profile_id)
                    .map_err(anyhow::Error::msg)?;
            profile.providers.remove(&provider_id);
            if let Some(ref default_model) = profile.default_model {
                if default_model.starts_with(&format!("{provider_id}/")) {
                    profile.default_model = None;
                }
            }
            if let Some(failovers) = profile.failover_models.as_mut() {
                failovers.retain(|r| !r.starts_with(&format!("{provider_id}/")));
                if failovers.is_empty() {
                    profile.failover_models = None;
                }
            }
            droidgear_core::openclaw::save_openclaw_profile_for_home(&app.home_dir, profile)
                .map_err(anyhow::Error::msg)?;
            Ok(())
        }
        app::ConfirmAction::OpenClawDeleteModel {
            profile_id,
            provider_id,
            model_index,
        } => {
            let mut profile =
                droidgear_core::openclaw::get_openclaw_profile_for_home(&app.home_dir, &profile_id)
                    .map_err(anyhow::Error::msg)?;
            let Some(provider) = profile.providers.get_mut(&provider_id) else {
                return Err(anyhow::Error::msg("Provider not found"));
            };
            let removed_id = provider
                .models
                .get(model_index)
                .map(|m| m.id.clone())
                .unwrap_or_default();
            if model_index < provider.models.len() {
                provider.models.remove(model_index);
            }
            let model_ref = format!("{provider_id}/{removed_id}");
            if profile.default_model.as_deref() == Some(model_ref.as_str()) {
                profile.default_model = None;
            }
            if let Some(failovers) = profile.failover_models.as_mut() {
                failovers.retain(|r| r != &model_ref);
                if failovers.is_empty() {
                    profile.failover_models = None;
                }
            }
            droidgear_core::openclaw::save_openclaw_profile_for_home(&app.home_dir, profile)
                .map_err(anyhow::Error::msg)?;
            Ok(())
        }
        app::ConfirmAction::McpToggle { name, disabled } => {
            droidgear_core::mcp::toggle_mcp_server_for_home(&app.home_dir, &name, disabled)
                .map_err(anyhow::Error::msg)?;
            Ok(())
        }
        app::ConfirmAction::McpDelete { name } => {
            droidgear_core::mcp::delete_mcp_server_for_home(&app.home_dir, &name)
                .map_err(anyhow::Error::msg)?;
            Ok(())
        }
        app::ConfirmAction::FactorySetDefaultModel { model_id } => {
            droidgear_core::factory_settings::save_default_model_for_home(&app.home_dir, &model_id)
                .map_err(anyhow::Error::msg)?;
            Ok(())
        }
        app::ConfirmAction::FactoryDeleteModel { index } => {
            let mut models =
                droidgear_core::factory_settings::load_custom_models_for_home(&app.home_dir)
                    .map_err(anyhow::Error::msg)?;
            if index < models.len() {
                models.remove(index);
            }
            normalize_factory_models(&mut models);
            droidgear_core::factory_settings::save_custom_models_for_home(&app.home_dir, models)
                .map_err(anyhow::Error::msg)?;
            Ok(())
        }
        app::ConfirmAction::SessionDelete { path } => {
            droidgear_core::sessions::delete_session(&path).map_err(anyhow::Error::msg)?;
            Ok(())
        }
        app::ConfirmAction::SpecDelete { path } => {
            droidgear_core::specs::delete_spec_for_home(&app.home_dir, &path)
                .map_err(anyhow::Error::msg)?;
            Ok(())
        }
        app::ConfirmAction::ChannelDelete { id } => {
            let mut channels = droidgear_core::channel::load_channels_for_home(&app.home_dir)
                .map_err(anyhow::Error::msg)?;
            channels.retain(|c| c.id != id);
            droidgear_core::channel::save_channels_for_home(&app.home_dir, channels)
                .map_err(anyhow::Error::msg)?;
            droidgear_core::channel::delete_channel_credentials_for_home(&app.home_dir, &id)
                .map_err(anyhow::Error::msg)?;
            Ok(())
        }
        app::ConfirmAction::OpenClawSubagentDelete { id } => {
            let mut subagents =
                droidgear_core::openclaw::read_openclaw_subagents_for_home(&app.home_dir)
                    .map_err(anyhow::Error::msg)?;
            subagents.retain(|a| a.id != id);
            // Also remove from main's allowAgents
            if let Some(main) = subagents.iter_mut().find(|a| a.id == "main") {
                if let Some(ref mut sa) = main.subagents {
                    if let Some(ref mut allows) = sa.allow_agents {
                        allows.retain(|a| a != &id);
                    }
                }
            }
            droidgear_core::openclaw::save_openclaw_subagents_for_home(&app.home_dir, subagents)
                .map_err(anyhow::Error::msg)?;
            refresh_openclaw_subagents(app);
            Ok(())
        }
        app::ConfirmAction::OpenClawSubagentToggleAllow { id } => {
            let mut subagents =
                droidgear_core::openclaw::read_openclaw_subagents_for_home(&app.home_dir)
                    .map_err(anyhow::Error::msg)?;
            let has_main = subagents.iter().any(|a| a.id == "main");
            if has_main {
                if let Some(main) = subagents.iter_mut().find(|a| a.id == "main") {
                    let sa = main.subagents.get_or_insert(
                        droidgear_core::openclaw::OpenClawSubAgentSubagentsConfig {
                            allow_agents: None,
                            max_concurrent: None,
                        },
                    );
                    let allows = sa.allow_agents.get_or_insert_with(Vec::new);
                    if allows.contains(&id) {
                        allows.retain(|a| a != &id);
                    } else {
                        allows.push(id);
                    }
                }
            } else {
                subagents.insert(
                    0,
                    droidgear_core::openclaw::OpenClawSubAgent {
                        id: "main".to_string(),
                        name: None,
                        identity: None,
                        model: None,
                        tools: None,
                        workspace: None,
                        subagents: Some(
                            droidgear_core::openclaw::OpenClawSubAgentSubagentsConfig {
                                allow_agents: Some(vec![id]),
                                max_concurrent: None,
                            },
                        ),
                    },
                );
            }
            droidgear_core::openclaw::save_openclaw_subagents_for_home(&app.home_dir, subagents)
                .map_err(anyhow::Error::msg)?;
            refresh_openclaw_subagents(app);
            Ok(())
        }
        app::ConfirmAction::HermesApply { id } => {
            droidgear_core::hermes::apply_hermes_profile_for_home(&app.home_dir, &id)
                .map_err(anyhow::Error::msg)?;
            app.set_toast("Applied", false);
            Ok(())
        }
        app::ConfirmAction::HermesDelete { id } => {
            droidgear_core::hermes::delete_hermes_profile_for_home(&app.home_dir, &id)
                .map_err(anyhow::Error::msg)?;
            Ok(())
        }
    }
}

fn run_input_action(
    app: &mut app::App,
    action: app::InputAction,
    value: String,
) -> anyhow::Result<()> {
    let trimmed = value.trim();
    match action {
        app::InputAction::PathsSetKey { key } => {
            droidgear_core::paths::save_config_path_for_home(&app.home_dir, &key, trimmed)
                .map_err(anyhow::Error::msg)?;
            Ok(())
        }
        app::InputAction::CodexCreateProfile => {
            if trimmed.is_empty() {
                return Err(anyhow::Error::msg("Profile name is required"));
            }

            let before = droidgear_core::codex::list_codex_profiles_for_home(&app.home_dir)
                .map_err(anyhow::Error::msg)?;
            let before_ids = before
                .iter()
                .map(|p| p.id.clone())
                .collect::<std::collections::HashSet<String>>();

            let mut providers = std::collections::HashMap::new();
            providers.insert(
                "custom".to_string(),
                droidgear_core::codex::CodexProviderConfig {
                    name: Some("Custom Provider".to_string()),
                    base_url: None,
                    wire_api: Some("responses".to_string()),
                    requires_openai_auth: Some(true),
                    env_key: None,
                    env_key_instructions: None,
                    http_headers: None,
                    query_params: None,
                    model: Some("gpt-5.2".to_string()),
                    model_reasoning_effort: Some("high".to_string()),
                    api_key: Some(String::new()),
                },
            );

            let profile = droidgear_core::codex::CodexProfile {
                id: String::new(),
                name: trimmed.to_string(),
                description: None,
                created_at: String::new(),
                updated_at: String::new(),
                providers,
                model_provider: "custom".to_string(),
                model: "gpt-5.2".to_string(),
                model_reasoning_effort: Some("high".to_string()),
                api_key: Some(String::new()),
            };

            droidgear_core::codex::save_codex_profile_for_home(&app.home_dir, profile)
                .map_err(anyhow::Error::msg)?;

            refresh_codex(app);
            if let Some((idx, p)) = app
                .codex_profiles
                .iter()
                .enumerate()
                .find(|(_, p)| !before_ids.contains(&p.id))
            {
                app.codex_index = idx;
                app.codex_detail_id = Some(p.id.clone());
                app.codex_detail_focus = app::CodexDetailFocus::Fields;
                app.codex_detail_field_index = 0;
                app.codex_detail_provider_index = 0;
                app.screen = app::Screen::CodexProfile;
                refresh_codex_detail(app);
            }

            Ok(())
        }
        app::InputAction::CodexDuplicate { id } => {
            if trimmed.is_empty() {
                return Err(anyhow::Error::msg("Profile name is required"));
            }
            let new_profile = droidgear_core::codex::duplicate_codex_profile_for_home(
                &app.home_dir,
                &id,
                trimmed,
            )
            .map_err(anyhow::Error::msg)?;
            refresh_codex(app);
            if let Some(idx) = app
                .codex_profiles
                .iter()
                .position(|p| p.id == new_profile.id)
            {
                app.codex_index = idx;
            }
            Ok(())
        }
        app::InputAction::CodexSetProfileName { id } => {
            if trimmed.is_empty() {
                return Err(anyhow::Error::msg("Profile name is required"));
            }
            let mut profile = droidgear_core::codex::get_codex_profile_for_home(&app.home_dir, &id)
                .map_err(anyhow::Error::msg)?;
            profile.name = trimmed.to_string();
            droidgear_core::codex::save_codex_profile_for_home(&app.home_dir, profile)
                .map_err(anyhow::Error::msg)?;
            app.set_toast("Saved", false);
            Ok(())
        }
        app::InputAction::CodexSetProfileDescription { id } => {
            let mut profile = droidgear_core::codex::get_codex_profile_for_home(&app.home_dir, &id)
                .map_err(anyhow::Error::msg)?;
            profile.description = (!trimmed.is_empty()).then(|| trimmed.to_string());
            droidgear_core::codex::save_codex_profile_for_home(&app.home_dir, profile)
                .map_err(anyhow::Error::msg)?;
            app.set_toast("Saved", false);
            Ok(())
        }
        app::InputAction::CodexSetProfileModel { id } => {
            let mut profile = droidgear_core::codex::get_codex_profile_for_home(&app.home_dir, &id)
                .map_err(anyhow::Error::msg)?;
            profile.model = trimmed.to_string();
            droidgear_core::codex::save_codex_profile_for_home(&app.home_dir, profile)
                .map_err(anyhow::Error::msg)?;
            app.set_toast("Saved", false);
            Ok(())
        }
        app::InputAction::CodexSetProfileApiKey { id } => {
            let mut profile = droidgear_core::codex::get_codex_profile_for_home(&app.home_dir, &id)
                .map_err(anyhow::Error::msg)?;
            profile.api_key = (!trimmed.is_empty()).then(|| trimmed.to_string());
            droidgear_core::codex::save_codex_profile_for_home(&app.home_dir, profile)
                .map_err(anyhow::Error::msg)?;
            app.set_toast("Saved", false);
            Ok(())
        }
        app::InputAction::CodexAddProvider { id } => {
            if trimmed.is_empty() {
                return Err(anyhow::Error::msg("Provider id is required"));
            }
            let mut profile = droidgear_core::codex::get_codex_profile_for_home(&app.home_dir, &id)
                .map_err(anyhow::Error::msg)?;
            if profile.providers.contains_key(trimmed) {
                return Err(anyhow::Error::msg("Provider already exists"));
            }

            profile.providers.insert(
                trimmed.to_string(),
                droidgear_core::codex::CodexProviderConfig {
                    name: None,
                    base_url: None,
                    wire_api: Some("responses".to_string()),
                    requires_openai_auth: None,
                    env_key: None,
                    env_key_instructions: None,
                    http_headers: None,
                    query_params: None,
                    model: None,
                    model_reasoning_effort: Some("high".to_string()),
                    api_key: None,
                },
            );

            droidgear_core::codex::save_codex_profile_for_home(&app.home_dir, profile)
                .map_err(anyhow::Error::msg)?;

            app.codex_provider_id = Some(trimmed.to_string());
            app.codex_provider_field_index = 0;
            app.screen = app::Screen::CodexProvider;
            refresh_codex_detail(app);
            Ok(())
        }
        app::InputAction::CodexSetProviderName {
            profile_id,
            provider_id,
        } => {
            let mut profile =
                droidgear_core::codex::get_codex_profile_for_home(&app.home_dir, &profile_id)
                    .map_err(anyhow::Error::msg)?;
            let Some(provider) = profile.providers.get_mut(&provider_id) else {
                return Err(anyhow::Error::msg("Provider not found"));
            };
            provider.name = (!trimmed.is_empty()).then(|| trimmed.to_string());
            droidgear_core::codex::save_codex_profile_for_home(&app.home_dir, profile)
                .map_err(anyhow::Error::msg)?;
            app.set_toast("Saved", false);
            Ok(())
        }
        app::InputAction::CodexSetProviderBaseUrl {
            profile_id,
            provider_id,
        } => {
            let mut profile =
                droidgear_core::codex::get_codex_profile_for_home(&app.home_dir, &profile_id)
                    .map_err(anyhow::Error::msg)?;
            let Some(provider) = profile.providers.get_mut(&provider_id) else {
                return Err(anyhow::Error::msg("Provider not found"));
            };
            provider.base_url = (!trimmed.is_empty()).then(|| trimmed.to_string());
            droidgear_core::codex::save_codex_profile_for_home(&app.home_dir, profile)
                .map_err(anyhow::Error::msg)?;
            app.set_toast("Saved", false);
            Ok(())
        }
        app::InputAction::CodexSetProviderApiKey {
            profile_id,
            provider_id,
        } => {
            let mut profile =
                droidgear_core::codex::get_codex_profile_for_home(&app.home_dir, &profile_id)
                    .map_err(anyhow::Error::msg)?;
            let Some(provider) = profile.providers.get_mut(&provider_id) else {
                return Err(anyhow::Error::msg("Provider not found"));
            };
            provider.api_key = (!trimmed.is_empty()).then(|| trimmed.to_string());
            droidgear_core::codex::save_codex_profile_for_home(&app.home_dir, profile)
                .map_err(anyhow::Error::msg)?;
            app.set_toast("Saved", false);
            Ok(())
        }
        app::InputAction::CodexSetProviderModel {
            profile_id,
            provider_id,
        } => {
            let mut profile =
                droidgear_core::codex::get_codex_profile_for_home(&app.home_dir, &profile_id)
                    .map_err(anyhow::Error::msg)?;
            let Some(provider) = profile.providers.get_mut(&provider_id) else {
                return Err(anyhow::Error::msg("Provider not found"));
            };
            provider.model = (!trimmed.is_empty()).then(|| trimmed.to_string());
            droidgear_core::codex::save_codex_profile_for_home(&app.home_dir, profile)
                .map_err(anyhow::Error::msg)?;
            app.set_toast("Saved", false);
            Ok(())
        }
        app::InputAction::OpenCodeCreateProfile => {
            if trimmed.is_empty() {
                return Err(anyhow::Error::msg("Profile name is required"));
            }

            let before = droidgear_core::opencode::list_opencode_profiles_for_home(&app.home_dir)
                .map_err(anyhow::Error::msg)?;
            let before_ids = before
                .iter()
                .map(|p| p.id.clone())
                .collect::<std::collections::HashSet<String>>();

            let profile = droidgear_core::opencode::OpenCodeProfile {
                id: String::new(),
                name: trimmed.to_string(),
                description: None,
                created_at: String::new(),
                updated_at: String::new(),
                providers: std::collections::HashMap::new(),
                auth: std::collections::HashMap::new(),
            };
            droidgear_core::opencode::save_opencode_profile_for_home(&app.home_dir, profile)
                .map_err(anyhow::Error::msg)?;

            refresh_opencode(app);
            if let Some((idx, p)) = app
                .opencode_profiles
                .iter()
                .enumerate()
                .find(|(_, p)| !before_ids.contains(&p.id))
            {
                app.opencode_index = idx;
                app.opencode_detail_id = Some(p.id.clone());
                app.opencode_detail_focus = app::CodexDetailFocus::Fields;
                app.opencode_detail_field_index = 0;
                app.opencode_detail_provider_index = 0;
                app.opencode_provider_id = None;
                app.opencode_provider_focus = app::CodexDetailFocus::Fields;
                app.opencode_provider_field_index = 0;
                app.opencode_provider_model_index = 0;
                app.opencode_model_id = None;
                app.opencode_model_field_index = 0;
                app.screen = app::Screen::OpenCodeProfile;
                refresh_opencode_detail(app);
            }

            Ok(())
        }
        app::InputAction::OpenCodeDuplicate { id } => {
            if trimmed.is_empty() {
                return Err(anyhow::Error::msg("Profile name is required"));
            }
            let new_profile = droidgear_core::opencode::duplicate_opencode_profile_for_home(
                &app.home_dir,
                &id,
                trimmed,
            )
            .map_err(anyhow::Error::msg)?;
            refresh_opencode(app);
            if let Some(idx) = app
                .opencode_profiles
                .iter()
                .position(|p| p.id == new_profile.id)
            {
                app.opencode_index = idx;
            }
            Ok(())
        }
        app::InputAction::OpenCodeSetProfileName { id } => {
            if trimmed.is_empty() {
                return Err(anyhow::Error::msg("Profile name is required"));
            }
            let mut profile =
                droidgear_core::opencode::get_opencode_profile_for_home(&app.home_dir, &id)
                    .map_err(anyhow::Error::msg)?;
            profile.name = trimmed.to_string();
            droidgear_core::opencode::save_opencode_profile_for_home(&app.home_dir, profile)
                .map_err(anyhow::Error::msg)?;
            app.set_toast("Saved", false);
            Ok(())
        }
        app::InputAction::OpenCodeSetProfileDescription { id } => {
            let mut profile =
                droidgear_core::opencode::get_opencode_profile_for_home(&app.home_dir, &id)
                    .map_err(anyhow::Error::msg)?;
            profile.description = (!trimmed.is_empty()).then(|| trimmed.to_string());
            droidgear_core::opencode::save_opencode_profile_for_home(&app.home_dir, profile)
                .map_err(anyhow::Error::msg)?;
            app.set_toast("Saved", false);
            Ok(())
        }
        app::InputAction::OpenCodeAddProvider { profile_id } => {
            if trimmed.is_empty() {
                return Err(anyhow::Error::msg("Provider id is required"));
            }
            let mut profile =
                droidgear_core::opencode::get_opencode_profile_for_home(&app.home_dir, &profile_id)
                    .map_err(anyhow::Error::msg)?;
            if profile.providers.contains_key(trimmed) {
                return Err(anyhow::Error::msg("Provider already exists"));
            }
            profile.providers.insert(
                trimmed.to_string(),
                droidgear_core::opencode::OpenCodeProviderConfig {
                    npm: None,
                    name: None,
                    options: Some(droidgear_core::opencode::OpenCodeProviderOptions::default()),
                    models: None,
                },
            );
            droidgear_core::opencode::save_opencode_profile_for_home(&app.home_dir, profile)
                .map_err(anyhow::Error::msg)?;

            app.opencode_provider_id = Some(trimmed.to_string());
            app.opencode_provider_focus = app::CodexDetailFocus::Fields;
            app.opencode_provider_field_index = 0;
            app.opencode_provider_model_index = 0;
            app.opencode_model_id = None;
            app.opencode_model_field_index = 0;
            app.screen = app::Screen::OpenCodeProvider;
            refresh_opencode_detail(app);
            Ok(())
        }
        app::InputAction::OpenCodeSetProviderName {
            profile_id,
            provider_id,
        } => {
            let mut profile =
                droidgear_core::opencode::get_opencode_profile_for_home(&app.home_dir, &profile_id)
                    .map_err(anyhow::Error::msg)?;
            let Some(provider) = profile.providers.get_mut(&provider_id) else {
                return Err(anyhow::Error::msg("Provider not found"));
            };
            provider.name = (!trimmed.is_empty()).then(|| trimmed.to_string());
            droidgear_core::opencode::save_opencode_profile_for_home(&app.home_dir, profile)
                .map_err(anyhow::Error::msg)?;
            app.set_toast("Saved", false);
            Ok(())
        }
        app::InputAction::OpenCodeSetProviderNpm {
            profile_id,
            provider_id,
        } => {
            let mut profile =
                droidgear_core::opencode::get_opencode_profile_for_home(&app.home_dir, &profile_id)
                    .map_err(anyhow::Error::msg)?;
            let Some(provider) = profile.providers.get_mut(&provider_id) else {
                return Err(anyhow::Error::msg("Provider not found"));
            };
            provider.npm = (!trimmed.is_empty()).then(|| trimmed.to_string());
            droidgear_core::opencode::save_opencode_profile_for_home(&app.home_dir, profile)
                .map_err(anyhow::Error::msg)?;
            app.set_toast("Saved", false);
            Ok(())
        }
        app::InputAction::OpenCodeSetProviderBaseUrl {
            profile_id,
            provider_id,
        } => {
            let mut profile =
                droidgear_core::opencode::get_opencode_profile_for_home(&app.home_dir, &profile_id)
                    .map_err(anyhow::Error::msg)?;
            let Some(provider) = profile.providers.get_mut(&provider_id) else {
                return Err(anyhow::Error::msg("Provider not found"));
            };
            let options = provider
                .options
                .get_or_insert_with(droidgear_core::opencode::OpenCodeProviderOptions::default);
            options.base_url = (!trimmed.is_empty()).then(|| trimmed.to_string());
            droidgear_core::opencode::save_opencode_profile_for_home(&app.home_dir, profile)
                .map_err(anyhow::Error::msg)?;
            app.set_toast("Saved", false);
            Ok(())
        }
        app::InputAction::OpenCodeSetProviderApiKey {
            profile_id,
            provider_id,
        } => {
            let mut profile =
                droidgear_core::opencode::get_opencode_profile_for_home(&app.home_dir, &profile_id)
                    .map_err(anyhow::Error::msg)?;
            if trimmed.is_empty() {
                profile.auth.remove(&provider_id);
            } else {
                profile.auth.insert(
                    provider_id,
                    serde_json::json!({
                        "type": "api",
                        "key": trimmed,
                    }),
                );
            }
            droidgear_core::opencode::save_opencode_profile_for_home(&app.home_dir, profile)
                .map_err(anyhow::Error::msg)?;
            app.set_toast("Saved", false);
            Ok(())
        }
        app::InputAction::OpenCodeSetProviderTimeout {
            profile_id,
            provider_id,
        } => {
            let timeout = if trimmed.is_empty() {
                None
            } else {
                Some(
                    trimmed
                        .parse::<u32>()
                        .map_err(|_| anyhow::Error::msg("Invalid timeout"))?,
                )
            };

            let mut profile =
                droidgear_core::opencode::get_opencode_profile_for_home(&app.home_dir, &profile_id)
                    .map_err(anyhow::Error::msg)?;
            let Some(provider) = profile.providers.get_mut(&provider_id) else {
                return Err(anyhow::Error::msg("Provider not found"));
            };
            let options = provider
                .options
                .get_or_insert_with(droidgear_core::opencode::OpenCodeProviderOptions::default);
            options.timeout = timeout;
            droidgear_core::opencode::save_opencode_profile_for_home(&app.home_dir, profile)
                .map_err(anyhow::Error::msg)?;
            app.set_toast("Saved", false);
            Ok(())
        }
        app::InputAction::OpenCodeAddModel {
            profile_id,
            provider_id,
        } => {
            if trimmed.is_empty() {
                return Err(anyhow::Error::msg("Model id is required"));
            }
            let mut profile =
                droidgear_core::opencode::get_opencode_profile_for_home(&app.home_dir, &profile_id)
                    .map_err(anyhow::Error::msg)?;
            let Some(provider) = profile.providers.get_mut(&provider_id) else {
                return Err(anyhow::Error::msg("Provider not found"));
            };
            let models = provider
                .models
                .get_or_insert_with(std::collections::HashMap::new);
            if models.contains_key(trimmed) {
                return Err(anyhow::Error::msg("Model already exists"));
            }
            models.insert(
                trimmed.to_string(),
                droidgear_core::opencode::OpenCodeModelConfig::default(),
            );
            droidgear_core::opencode::save_opencode_profile_for_home(&app.home_dir, profile)
                .map_err(anyhow::Error::msg)?;

            app.opencode_model_id = Some(trimmed.to_string());
            app.opencode_model_field_index = 0;
            app.screen = app::Screen::OpenCodeModel;
            refresh_opencode_detail(app);
            Ok(())
        }
        app::InputAction::OpenCodeSetModelName {
            profile_id,
            provider_id,
            model_id,
        } => {
            let mut profile =
                droidgear_core::opencode::get_opencode_profile_for_home(&app.home_dir, &profile_id)
                    .map_err(anyhow::Error::msg)?;
            let Some(provider) = profile.providers.get_mut(&provider_id) else {
                return Err(anyhow::Error::msg("Provider not found"));
            };
            let Some(models) = provider.models.as_mut() else {
                return Err(anyhow::Error::msg("No models configured"));
            };
            let Some(model) = models.get_mut(&model_id) else {
                return Err(anyhow::Error::msg("Model not found"));
            };
            model.name = (!trimmed.is_empty()).then(|| trimmed.to_string());
            droidgear_core::opencode::save_opencode_profile_for_home(&app.home_dir, profile)
                .map_err(anyhow::Error::msg)?;
            app.set_toast("Saved", false);
            Ok(())
        }
        app::InputAction::OpenCodeSetModelContextLimit {
            profile_id,
            provider_id,
            model_id,
        } => {
            let context = if trimmed.is_empty() {
                None
            } else {
                Some(
                    trimmed
                        .parse::<u32>()
                        .map_err(|_| anyhow::Error::msg("Invalid context limit"))?,
                )
            };
            let mut profile =
                droidgear_core::opencode::get_opencode_profile_for_home(&app.home_dir, &profile_id)
                    .map_err(anyhow::Error::msg)?;
            let Some(provider) = profile.providers.get_mut(&provider_id) else {
                return Err(anyhow::Error::msg("Provider not found"));
            };
            let Some(models) = provider.models.as_mut() else {
                return Err(anyhow::Error::msg("No models configured"));
            };
            let Some(model) = models.get_mut(&model_id) else {
                return Err(anyhow::Error::msg("Model not found"));
            };
            let limit = model
                .limit
                .get_or_insert_with(droidgear_core::opencode::OpenCodeModelLimit::default);
            limit.context = context;
            if limit.context.is_none() && limit.output.is_none() {
                model.limit = None;
            }
            droidgear_core::opencode::save_opencode_profile_for_home(&app.home_dir, profile)
                .map_err(anyhow::Error::msg)?;
            app.set_toast("Saved", false);
            Ok(())
        }
        app::InputAction::OpenCodeSetModelOutputLimit {
            profile_id,
            provider_id,
            model_id,
        } => {
            let output = if trimmed.is_empty() {
                None
            } else {
                Some(
                    trimmed
                        .parse::<u32>()
                        .map_err(|_| anyhow::Error::msg("Invalid output limit"))?,
                )
            };
            let mut profile =
                droidgear_core::opencode::get_opencode_profile_for_home(&app.home_dir, &profile_id)
                    .map_err(anyhow::Error::msg)?;
            let Some(provider) = profile.providers.get_mut(&provider_id) else {
                return Err(anyhow::Error::msg("Provider not found"));
            };
            let Some(models) = provider.models.as_mut() else {
                return Err(anyhow::Error::msg("No models configured"));
            };
            let Some(model) = models.get_mut(&model_id) else {
                return Err(anyhow::Error::msg("Model not found"));
            };
            let limit = model
                .limit
                .get_or_insert_with(droidgear_core::opencode::OpenCodeModelLimit::default);
            limit.output = output;
            if limit.context.is_none() && limit.output.is_none() {
                model.limit = None;
            }
            droidgear_core::opencode::save_opencode_profile_for_home(&app.home_dir, profile)
                .map_err(anyhow::Error::msg)?;
            app.set_toast("Saved", false);
            Ok(())
        }
        app::InputAction::OpenClawCreateProfile => {
            if trimmed.is_empty() {
                return Err(anyhow::Error::msg("Profile name is required"));
            }

            let before = droidgear_core::openclaw::list_openclaw_profiles_for_home(&app.home_dir)
                .map_err(anyhow::Error::msg)?;
            let before_ids = before
                .iter()
                .map(|p| p.id.clone())
                .collect::<std::collections::HashSet<String>>();

            let profile = droidgear_core::openclaw::OpenClawProfile {
                id: String::new(),
                name: trimmed.to_string(),
                description: None,
                created_at: String::new(),
                updated_at: String::new(),
                default_model: None,
                failover_models: None,
                providers: std::collections::HashMap::new(),
                block_streaming_config: None,
            };
            droidgear_core::openclaw::save_openclaw_profile_for_home(&app.home_dir, profile)
                .map_err(anyhow::Error::msg)?;

            refresh_openclaw(app);
            if let Some((idx, p)) = app
                .openclaw_profiles
                .iter()
                .enumerate()
                .find(|(_, p)| !before_ids.contains(&p.id))
            {
                app.openclaw_index = idx;
                app.openclaw_detail_id = Some(p.id.clone());
                app.openclaw_detail_focus = app::OpenClawProfileFocus::Fields;
                app.openclaw_detail_field_index = 0;
                app.openclaw_detail_failover_index = 0;
                app.openclaw_detail_provider_index = 0;
                app.openclaw_provider_id = None;
                app.openclaw_provider_focus = app::CodexDetailFocus::Fields;
                app.openclaw_provider_field_index = 0;
                app.openclaw_provider_model_index = 0;
                app.openclaw_model_field_index = 0;
                app.openclaw_helpers_field_index = 0;
                app.screen = app::Screen::OpenClawProfile;
                refresh_openclaw_detail(app);
            }

            Ok(())
        }
        app::InputAction::OpenClawDuplicate { id } => {
            if trimmed.is_empty() {
                return Err(anyhow::Error::msg("Profile name is required"));
            }
            let new_profile = droidgear_core::openclaw::duplicate_openclaw_profile_for_home(
                &app.home_dir,
                &id,
                trimmed,
            )
            .map_err(anyhow::Error::msg)?;
            refresh_openclaw(app);
            if let Some(idx) = app
                .openclaw_profiles
                .iter()
                .position(|p| p.id == new_profile.id)
            {
                app.openclaw_index = idx;
            }
            Ok(())
        }
        app::InputAction::OpenClawSetProfileName { id } => {
            if trimmed.is_empty() {
                return Err(anyhow::Error::msg("Profile name is required"));
            }
            let mut profile =
                droidgear_core::openclaw::get_openclaw_profile_for_home(&app.home_dir, &id)
                    .map_err(anyhow::Error::msg)?;
            profile.name = trimmed.to_string();
            droidgear_core::openclaw::save_openclaw_profile_for_home(&app.home_dir, profile)
                .map_err(anyhow::Error::msg)?;
            app.set_toast("Saved", false);
            Ok(())
        }
        app::InputAction::OpenClawSetProfileDescription { id } => {
            let mut profile =
                droidgear_core::openclaw::get_openclaw_profile_for_home(&app.home_dir, &id)
                    .map_err(anyhow::Error::msg)?;
            profile.description = (!trimmed.is_empty()).then(|| trimmed.to_string());
            droidgear_core::openclaw::save_openclaw_profile_for_home(&app.home_dir, profile)
                .map_err(anyhow::Error::msg)?;
            app.set_toast("Saved", false);
            Ok(())
        }
        app::InputAction::OpenClawAddProvider { profile_id } => {
            if trimmed.is_empty() {
                return Err(anyhow::Error::msg("Provider id is required"));
            }
            let mut profile =
                droidgear_core::openclaw::get_openclaw_profile_for_home(&app.home_dir, &profile_id)
                    .map_err(anyhow::Error::msg)?;
            if profile.providers.contains_key(trimmed) {
                return Err(anyhow::Error::msg("Provider already exists"));
            }
            profile.providers.insert(
                trimmed.to_string(),
                droidgear_core::openclaw::OpenClawProviderConfig {
                    base_url: None,
                    api_key: None,
                    api: Some("openai-completions".to_string()),
                    models: Vec::new(),
                },
            );
            droidgear_core::openclaw::save_openclaw_profile_for_home(&app.home_dir, profile)
                .map_err(anyhow::Error::msg)?;

            app.openclaw_provider_id = Some(trimmed.to_string());
            app.openclaw_provider_focus = app::CodexDetailFocus::Fields;
            app.openclaw_provider_field_index = 0;
            app.openclaw_provider_model_index = 0;
            app.openclaw_model_field_index = 0;
            app.screen = app::Screen::OpenClawProvider;
            refresh_openclaw_detail(app);
            Ok(())
        }
        app::InputAction::OpenClawSetProviderBaseUrl {
            profile_id,
            provider_id,
        } => {
            let mut profile =
                droidgear_core::openclaw::get_openclaw_profile_for_home(&app.home_dir, &profile_id)
                    .map_err(anyhow::Error::msg)?;
            let Some(provider) = profile.providers.get_mut(&provider_id) else {
                return Err(anyhow::Error::msg("Provider not found"));
            };
            provider.base_url = (!trimmed.is_empty()).then(|| trimmed.to_string());
            droidgear_core::openclaw::save_openclaw_profile_for_home(&app.home_dir, profile)
                .map_err(anyhow::Error::msg)?;
            app.set_toast("Saved", false);
            Ok(())
        }
        app::InputAction::OpenClawSetProviderApiKey {
            profile_id,
            provider_id,
        } => {
            let mut profile =
                droidgear_core::openclaw::get_openclaw_profile_for_home(&app.home_dir, &profile_id)
                    .map_err(anyhow::Error::msg)?;
            let Some(provider) = profile.providers.get_mut(&provider_id) else {
                return Err(anyhow::Error::msg("Provider not found"));
            };
            provider.api_key = (!trimmed.is_empty()).then(|| trimmed.to_string());
            droidgear_core::openclaw::save_openclaw_profile_for_home(&app.home_dir, profile)
                .map_err(anyhow::Error::msg)?;
            app.set_toast("Saved", false);
            Ok(())
        }
        app::InputAction::OpenClawAddModel {
            profile_id,
            provider_id,
        } => {
            if trimmed.is_empty() {
                return Err(anyhow::Error::msg("Model id is required"));
            }
            let mut profile =
                droidgear_core::openclaw::get_openclaw_profile_for_home(&app.home_dir, &profile_id)
                    .map_err(anyhow::Error::msg)?;
            let Some(provider) = profile.providers.get_mut(&provider_id) else {
                return Err(anyhow::Error::msg("Provider not found"));
            };
            let new_index = provider.models.len();
            provider
                .models
                .push(droidgear_core::openclaw::OpenClawModel {
                    id: trimmed.to_string(),
                    name: None,
                    reasoning: true,
                    input: vec!["text".to_string(), "image".to_string()],
                    context_window: Some(200000),
                    max_tokens: Some(8192),
                });
            droidgear_core::openclaw::save_openclaw_profile_for_home(&app.home_dir, profile)
                .map_err(anyhow::Error::msg)?;
            app.openclaw_provider_id = Some(provider_id);
            app.openclaw_provider_model_index = new_index;
            app.openclaw_model_field_index = 0;
            app.screen = app::Screen::OpenClawModel;
            refresh_openclaw_detail(app);
            Ok(())
        }
        app::InputAction::OpenClawSetModelId {
            profile_id,
            provider_id,
            model_index,
        } => {
            if trimmed.is_empty() {
                return Err(anyhow::Error::msg("Model id is required"));
            }
            let mut profile =
                droidgear_core::openclaw::get_openclaw_profile_for_home(&app.home_dir, &profile_id)
                    .map_err(anyhow::Error::msg)?;
            let Some(provider) = profile.providers.get_mut(&provider_id) else {
                return Err(anyhow::Error::msg("Provider not found"));
            };
            let Some(model) = provider.models.get_mut(model_index) else {
                return Err(anyhow::Error::msg("Model not found"));
            };
            model.id = trimmed.to_string();
            droidgear_core::openclaw::save_openclaw_profile_for_home(&app.home_dir, profile)
                .map_err(anyhow::Error::msg)?;
            app.set_toast("Saved", false);
            Ok(())
        }
        app::InputAction::OpenClawSetModelName {
            profile_id,
            provider_id,
            model_index,
        } => {
            let mut profile =
                droidgear_core::openclaw::get_openclaw_profile_for_home(&app.home_dir, &profile_id)
                    .map_err(anyhow::Error::msg)?;
            let Some(provider) = profile.providers.get_mut(&provider_id) else {
                return Err(anyhow::Error::msg("Provider not found"));
            };
            let Some(model) = provider.models.get_mut(model_index) else {
                return Err(anyhow::Error::msg("Model not found"));
            };
            model.name = (!trimmed.is_empty()).then(|| trimmed.to_string());
            droidgear_core::openclaw::save_openclaw_profile_for_home(&app.home_dir, profile)
                .map_err(anyhow::Error::msg)?;
            app.set_toast("Saved", false);
            Ok(())
        }
        app::InputAction::OpenClawSetModelContextWindow {
            profile_id,
            provider_id,
            model_index,
        } => {
            let context_window = if trimmed.is_empty() {
                None
            } else {
                Some(
                    trimmed
                        .parse::<u32>()
                        .map_err(|_| anyhow::Error::msg("Invalid context window"))?,
                )
            };
            let mut profile =
                droidgear_core::openclaw::get_openclaw_profile_for_home(&app.home_dir, &profile_id)
                    .map_err(anyhow::Error::msg)?;
            let Some(provider) = profile.providers.get_mut(&provider_id) else {
                return Err(anyhow::Error::msg("Provider not found"));
            };
            let Some(model) = provider.models.get_mut(model_index) else {
                return Err(anyhow::Error::msg("Model not found"));
            };
            model.context_window = context_window;
            droidgear_core::openclaw::save_openclaw_profile_for_home(&app.home_dir, profile)
                .map_err(anyhow::Error::msg)?;
            app.set_toast("Saved", false);
            Ok(())
        }
        app::InputAction::OpenClawSetModelMaxTokens {
            profile_id,
            provider_id,
            model_index,
        } => {
            let max_tokens = if trimmed.is_empty() {
                None
            } else {
                Some(
                    trimmed
                        .parse::<u32>()
                        .map_err(|_| anyhow::Error::msg("Invalid max tokens"))?,
                )
            };
            let mut profile =
                droidgear_core::openclaw::get_openclaw_profile_for_home(&app.home_dir, &profile_id)
                    .map_err(anyhow::Error::msg)?;
            let Some(provider) = profile.providers.get_mut(&provider_id) else {
                return Err(anyhow::Error::msg("Provider not found"));
            };
            let Some(model) = provider.models.get_mut(model_index) else {
                return Err(anyhow::Error::msg("Model not found"));
            };
            model.max_tokens = max_tokens;
            droidgear_core::openclaw::save_openclaw_profile_for_home(&app.home_dir, profile)
                .map_err(anyhow::Error::msg)?;
            app.set_toast("Saved", false);
            Ok(())
        }
        app::InputAction::OpenClawSetBlockStreamingMinChars { profile_id } => {
            let min_chars = if trimmed.is_empty() {
                None
            } else {
                Some(
                    trimmed
                        .parse::<u32>()
                        .map_err(|_| anyhow::Error::msg("Invalid min chars"))?,
                )
            };
            let mut profile =
                droidgear_core::openclaw::get_openclaw_profile_for_home(&app.home_dir, &profile_id)
                    .map_err(anyhow::Error::msg)?;
            let cfg = profile.block_streaming_config.get_or_insert({
                droidgear_core::openclaw::BlockStreamingConfig {
                    block_streaming_default: None,
                    block_streaming_break: None,
                    block_streaming_chunk: None,
                    block_streaming_coalesce: None,
                    telegram_channel: None,
                }
            });
            let chunk = cfg.block_streaming_chunk.get_or_insert({
                droidgear_core::openclaw::BlockStreamingChunk {
                    min_chars: None,
                    max_chars: None,
                }
            });
            chunk.min_chars = min_chars;
            if chunk.min_chars.is_none() && chunk.max_chars.is_none() {
                cfg.block_streaming_chunk = None;
            }
            droidgear_core::openclaw::save_openclaw_profile_for_home(&app.home_dir, profile)
                .map_err(anyhow::Error::msg)?;
            app.set_toast("Saved", false);
            Ok(())
        }
        app::InputAction::OpenClawSetBlockStreamingMaxChars { profile_id } => {
            let max_chars = if trimmed.is_empty() {
                None
            } else {
                Some(
                    trimmed
                        .parse::<u32>()
                        .map_err(|_| anyhow::Error::msg("Invalid max chars"))?,
                )
            };
            let mut profile =
                droidgear_core::openclaw::get_openclaw_profile_for_home(&app.home_dir, &profile_id)
                    .map_err(anyhow::Error::msg)?;
            let cfg = profile.block_streaming_config.get_or_insert({
                droidgear_core::openclaw::BlockStreamingConfig {
                    block_streaming_default: None,
                    block_streaming_break: None,
                    block_streaming_chunk: None,
                    block_streaming_coalesce: None,
                    telegram_channel: None,
                }
            });
            let chunk = cfg.block_streaming_chunk.get_or_insert({
                droidgear_core::openclaw::BlockStreamingChunk {
                    min_chars: None,
                    max_chars: None,
                }
            });
            chunk.max_chars = max_chars;
            if chunk.min_chars.is_none() && chunk.max_chars.is_none() {
                cfg.block_streaming_chunk = None;
            }
            droidgear_core::openclaw::save_openclaw_profile_for_home(&app.home_dir, profile)
                .map_err(anyhow::Error::msg)?;
            app.set_toast("Saved", false);
            Ok(())
        }
        app::InputAction::OpenClawSetBlockStreamingIdleMs { profile_id } => {
            let idle_ms = if trimmed.is_empty() {
                None
            } else {
                Some(
                    trimmed
                        .parse::<u32>()
                        .map_err(|_| anyhow::Error::msg("Invalid idle ms"))?,
                )
            };
            let mut profile =
                droidgear_core::openclaw::get_openclaw_profile_for_home(&app.home_dir, &profile_id)
                    .map_err(anyhow::Error::msg)?;
            let cfg = profile.block_streaming_config.get_or_insert({
                droidgear_core::openclaw::BlockStreamingConfig {
                    block_streaming_default: None,
                    block_streaming_break: None,
                    block_streaming_chunk: None,
                    block_streaming_coalesce: None,
                    telegram_channel: None,
                }
            });
            let coalesce = cfg.block_streaming_coalesce.get_or_insert({
                droidgear_core::openclaw::BlockStreamingCoalesce { idle_ms: None }
            });
            coalesce.idle_ms = idle_ms;
            if coalesce.idle_ms.is_none() {
                cfg.block_streaming_coalesce = None;
            }
            droidgear_core::openclaw::save_openclaw_profile_for_home(&app.home_dir, profile)
                .map_err(anyhow::Error::msg)?;
            app.set_toast("Saved", false);
            Ok(())
        }
        app::InputAction::FactoryDraftSetBaseUrl => {
            if let Some(draft) = app.factory_draft.as_mut() {
                draft.base_url = value;
            }
            Ok(())
        }
        app::InputAction::FactoryDraftSetApiKey => {
            if let Some(draft) = app.factory_draft.as_mut() {
                draft.api_key = value;
            }
            Ok(())
        }
        app::InputAction::FactoryDraftSetModel => {
            if let Some(draft) = app.factory_draft.as_mut() {
                draft.model = value;
            }
            Ok(())
        }
        app::InputAction::FactoryDraftSetDisplayName => {
            if let Some(draft) = app.factory_draft.as_mut() {
                draft.display_name = (!trimmed.is_empty()).then(|| trimmed.to_string());
            }
            Ok(())
        }
        app::InputAction::FactoryDraftSetMaxOutputTokens => {
            let tokens = if trimmed.is_empty() {
                None
            } else {
                Some(
                    trimmed
                        .parse::<u32>()
                        .map_err(|_| anyhow::Error::msg("Invalid max output tokens"))?,
                )
            };
            if let Some(draft) = app.factory_draft.as_mut() {
                draft.max_output_tokens = tokens;
            }
            Ok(())
        }
        app::InputAction::FactoryDraftSetExtraArgs => {
            if let Some(draft) = app.factory_draft.as_mut() {
                if trimmed.is_empty() {
                    draft.extra_args = None;
                } else {
                    let parsed: std::collections::HashMap<String, serde_json::Value> =
                        serde_json::from_str(trimmed)
                            .map_err(|e| anyhow::Error::msg(format!("Invalid JSON: {e}")))?;
                    draft.extra_args = Some(parsed);
                }
            }
            Ok(())
        }
        app::InputAction::FactoryDraftSetExtraHeaders => {
            if let Some(draft) = app.factory_draft.as_mut() {
                if trimmed.is_empty() {
                    draft.extra_headers = None;
                } else {
                    let parsed: std::collections::HashMap<String, String> =
                        serde_json::from_str(trimmed)
                            .map_err(|e| anyhow::Error::msg(format!("Invalid JSON: {e}")))?;
                    draft.extra_headers = Some(parsed);
                }
            }
            Ok(())
        }
        app::InputAction::McpCreateServer => {
            if trimmed.is_empty() {
                return Err(anyhow::Error::msg("Server name is required"));
            }
            app.mcp_edit_original_name = None;
            app.mcp_edit_draft = Some(droidgear_core::mcp::McpServer {
                name: trimmed.to_string(),
                config: droidgear_core::mcp::McpServerConfig {
                    server_type: droidgear_core::mcp::McpServerType::Stdio,
                    disabled: false,
                    command: None,
                    args: None,
                    env: None,
                    url: None,
                    headers: None,
                },
            });
            app.mcp_edit_field_index = 0;
            app.mcp_args_index = 0;
            app.mcp_kv_index = 0;
            app.screen = app::Screen::McpServer;
            Ok(())
        }
        app::InputAction::McpDraftSetName => {
            if trimmed.is_empty() {
                return Err(anyhow::Error::msg("Server name is required"));
            }
            if let Some(server) = app.mcp_edit_draft.as_mut() {
                server.name = trimmed.to_string();
            }
            Ok(())
        }
        app::InputAction::McpDraftSetCommand => {
            if let Some(server) = app.mcp_edit_draft.as_mut() {
                server.config.command = (!trimmed.is_empty()).then(|| trimmed.to_string());
            }
            Ok(())
        }
        app::InputAction::McpDraftSetUrl => {
            if let Some(server) = app.mcp_edit_draft.as_mut() {
                server.config.url = (!trimmed.is_empty()).then(|| trimmed.to_string());
            }
            Ok(())
        }
        app::InputAction::McpArgsAdd => {
            if trimmed.is_empty() {
                return Ok(());
            }
            if let Some(server) = app.mcp_edit_draft.as_mut() {
                let args = server.config.args.get_or_insert_with(Vec::new);
                args.push(trimmed.to_string());
            }
            Ok(())
        }
        app::InputAction::McpArgsEdit { index } => {
            if let Some(server) = app.mcp_edit_draft.as_mut() {
                if let Some(args) = server.config.args.as_mut() {
                    if index < args.len() {
                        if trimmed.is_empty() {
                            args.remove(index);
                        } else {
                            args[index] = trimmed.to_string();
                        }
                    }
                    if args.is_empty() {
                        server.config.args = None;
                    }
                }
            }
            Ok(())
        }
        app::InputAction::McpKeyValueAdd { mode } => {
            if trimmed.is_empty() {
                return Err(anyhow::Error::msg("key=value is required"));
            }
            let (k, v) = trimmed.split_once('=').unwrap_or((trimmed, ""));
            let key = k.trim();
            if key.is_empty() {
                return Err(anyhow::Error::msg("Key is required"));
            }
            let value = v.trim().to_string();

            if let Some(server) = app.mcp_edit_draft.as_mut() {
                match mode {
                    app::McpKeyValuesMode::Env => {
                        let env = server
                            .config
                            .env
                            .get_or_insert_with(std::collections::HashMap::new);
                        env.insert(key.to_string(), value);
                    }
                    app::McpKeyValuesMode::Headers => {
                        let headers = server
                            .config
                            .headers
                            .get_or_insert_with(std::collections::HashMap::new);
                        headers.insert(key.to_string(), value);
                    }
                }
            }
            Ok(())
        }
        app::InputAction::McpKeyValueEdit { mode, index } => {
            if trimmed.is_empty() {
                return Err(anyhow::Error::msg("key=value is required"));
            }
            let (k, v) = trimmed.split_once('=').unwrap_or((trimmed, ""));
            let key = k.trim();
            if key.is_empty() {
                return Err(anyhow::Error::msg("Key is required"));
            }
            let value = v.trim().to_string();

            if let Some(server) = app.mcp_edit_draft.as_mut() {
                match mode {
                    app::McpKeyValuesMode::Env => {
                        let Some(env) = server.config.env.as_mut() else {
                            return Ok(());
                        };
                        let mut keys: Vec<String> = env.keys().cloned().collect();
                        keys.sort_by_key(|a| a.to_lowercase());
                        let Some(old_key) = keys.get(index).cloned() else {
                            return Ok(());
                        };
                        env.remove(&old_key);
                        env.insert(key.to_string(), value);
                        if env.is_empty() {
                            server.config.env = None;
                        }
                    }
                    app::McpKeyValuesMode::Headers => {
                        let Some(headers) = server.config.headers.as_mut() else {
                            return Ok(());
                        };
                        let mut keys: Vec<String> = headers.keys().cloned().collect();
                        keys.sort_by_key(|a| a.to_lowercase());
                        let Some(old_key) = keys.get(index).cloned() else {
                            return Ok(());
                        };
                        headers.remove(&old_key);
                        headers.insert(key.to_string(), value);
                        if headers.is_empty() {
                            server.config.headers = None;
                        }
                    }
                }
            }
            Ok(())
        }
        app::InputAction::ChannelsDraftSetName => {
            if trimmed.is_empty() {
                return Err(anyhow::Error::msg("Name is required"));
            }
            if let Some(channel) = app.channels_edit_draft.as_mut() {
                channel.name = value;
            }
            Ok(())
        }
        app::InputAction::ChannelsDraftSetBaseUrl => {
            if let Some(channel) = app.channels_edit_draft.as_mut() {
                channel.base_url = value;
            }
            Ok(())
        }
        app::InputAction::ChannelsDraftSetUsername => {
            app.channels_edit_username = value;
            Ok(())
        }
        app::InputAction::ChannelsDraftSetPassword => {
            app.channels_edit_password = value;
            Ok(())
        }
        app::InputAction::ChannelsDraftSetApiKey => {
            app.channels_edit_api_key = value;
            Ok(())
        }
        app::InputAction::OpenClawSubagentCreate => {
            let id = value.trim().to_string();
            if id.is_empty() {
                return Ok(());
            }
            let mut subagents =
                droidgear_core::openclaw::read_openclaw_subagents_for_home(&app.home_dir)
                    .map_err(anyhow::Error::msg)?;
            if subagents.iter().any(|a| a.id == id) {
                app.set_toast(format!("Subagent '{}' already exists", id), true);
                return Ok(());
            }
            let agent = droidgear_core::openclaw::OpenClawSubAgent {
                id: id.clone(),
                name: None,
                identity: Some(droidgear_core::openclaw::OpenClawSubAgentIdentity {
                    emoji: Some("💻".to_string()),
                    name: None,
                }),
                model: None,
                tools: Some(droidgear_core::openclaw::OpenClawSubAgentTools {
                    profile: Some("full".to_string()),
                }),
                workspace: None,
                subagents: None,
            };
            subagents.push(agent);
            // Auto-add to main's allowAgents
            if let Some(main) = subagents.iter_mut().find(|a| a.id == "main") {
                let sa = main.subagents.get_or_insert(
                    droidgear_core::openclaw::OpenClawSubAgentSubagentsConfig {
                        allow_agents: None,
                        max_concurrent: None,
                    },
                );
                let allows = sa.allow_agents.get_or_insert_with(Vec::new);
                if !allows.contains(&id) {
                    allows.push(id);
                }
            }
            droidgear_core::openclaw::save_openclaw_subagents_for_home(&app.home_dir, subagents)
                .map_err(anyhow::Error::msg)?;
            refresh_openclaw_subagents(app);
            Ok(())
        }
        app::InputAction::OpenClawSubagentSetName { id } => {
            openclaw_update_subagent(app, &id, |agent| {
                agent.name = if value.trim().is_empty() {
                    None
                } else {
                    Some(value.trim().to_string())
                };
            })
        }
        app::InputAction::OpenClawSubagentSetEmoji { id } => {
            openclaw_update_subagent(app, &id, |agent| {
                let emoji = if value.trim().is_empty() {
                    None
                } else {
                    Some(value.trim().to_string())
                };
                let identity = agent.identity.get_or_insert(
                    droidgear_core::openclaw::OpenClawSubAgentIdentity {
                        emoji: None,
                        name: None,
                    },
                );
                identity.emoji = emoji;
            })
        }
        app::InputAction::OpenClawSubagentSetPrimaryModel { id } => {
            openclaw_update_subagent(app, &id, |agent| {
                let primary = if value.trim().is_empty() {
                    None
                } else {
                    Some(value.trim().to_string())
                };
                let model =
                    agent
                        .model
                        .get_or_insert(droidgear_core::openclaw::OpenClawSubAgentModel {
                            primary: None,
                            fallbacks: None,
                        });
                model.primary = primary;
            })
        }
        app::InputAction::OpenClawSubagentSetWorkspace { id } => {
            openclaw_update_subagent(app, &id, |agent| {
                agent.workspace = if value.trim().is_empty() {
                    None
                } else {
                    Some(value.trim().to_string())
                };
            })
        }
        app::InputAction::HermesCreateProfile => {
            if trimmed.is_empty() {
                return Err(anyhow::Error::msg("Profile name is required"));
            }

            let before = droidgear_core::hermes::list_hermes_profiles_for_home(&app.home_dir)
                .map_err(anyhow::Error::msg)?;
            let before_ids = before
                .iter()
                .map(|p| p.id.clone())
                .collect::<std::collections::HashSet<String>>();

            let profile = droidgear_core::hermes::HermesProfile {
                id: String::new(),
                name: trimmed.to_string(),
                description: None,
                created_at: String::new(),
                updated_at: String::new(),
                model: droidgear_core::hermes::HermesModelConfig {
                    default: Some(String::new()),
                    provider: Some(String::new()),
                    base_url: Some(String::new()),
                    api_key: Some(String::new()),
                },
            };

            droidgear_core::hermes::save_hermes_profile_for_home(&app.home_dir, profile)
                .map_err(anyhow::Error::msg)?;

            refresh_hermes(app);
            if let Some((idx, p)) = app
                .hermes_profiles
                .iter()
                .enumerate()
                .find(|(_, p)| !before_ids.contains(&p.id))
            {
                app.hermes_index = idx;
                app.hermes_detail_id = Some(p.id.clone());
                app.hermes_detail_field_index = 0;
                app.screen = app::Screen::HermesProfile;
                refresh_hermes_detail(app);
            }

            Ok(())
        }
        app::InputAction::HermesDuplicate { id } => {
            if trimmed.is_empty() {
                return Err(anyhow::Error::msg("Profile name is required"));
            }
            let new_profile = droidgear_core::hermes::duplicate_hermes_profile_for_home(
                &app.home_dir,
                &id,
                trimmed,
            )
            .map_err(anyhow::Error::msg)?;
            refresh_hermes(app);
            if let Some(idx) = app
                .hermes_profiles
                .iter()
                .position(|p| p.id == new_profile.id)
            {
                app.hermes_index = idx;
            }
            Ok(())
        }
        app::InputAction::HermesSetProfileName { id } => {
            if trimmed.is_empty() {
                return Err(anyhow::Error::msg("Profile name is required"));
            }
            let mut profile =
                droidgear_core::hermes::get_hermes_profile_for_home(&app.home_dir, &id)
                    .map_err(anyhow::Error::msg)?;
            profile.name = trimmed.to_string();
            droidgear_core::hermes::save_hermes_profile_for_home(&app.home_dir, profile)
                .map_err(anyhow::Error::msg)?;
            app.set_toast("Saved", false);
            Ok(())
        }
        app::InputAction::HermesSetProfileDescription { id } => {
            let mut profile =
                droidgear_core::hermes::get_hermes_profile_for_home(&app.home_dir, &id)
                    .map_err(anyhow::Error::msg)?;
            profile.description = (!trimmed.is_empty()).then(|| trimmed.to_string());
            droidgear_core::hermes::save_hermes_profile_for_home(&app.home_dir, profile)
                .map_err(anyhow::Error::msg)?;
            app.set_toast("Saved", false);
            Ok(())
        }
        app::InputAction::HermesSetProfileDefaultModel { id } => {
            let mut profile =
                droidgear_core::hermes::get_hermes_profile_for_home(&app.home_dir, &id)
                    .map_err(anyhow::Error::msg)?;
            profile.model.default = (!trimmed.is_empty()).then(|| trimmed.to_string());
            droidgear_core::hermes::save_hermes_profile_for_home(&app.home_dir, profile)
                .map_err(anyhow::Error::msg)?;
            app.set_toast("Saved", false);
            Ok(())
        }
        app::InputAction::HermesSetProfileProvider { id } => {
            let mut profile =
                droidgear_core::hermes::get_hermes_profile_for_home(&app.home_dir, &id)
                    .map_err(anyhow::Error::msg)?;
            profile.model.provider = (!trimmed.is_empty()).then(|| trimmed.to_string());
            droidgear_core::hermes::save_hermes_profile_for_home(&app.home_dir, profile)
                .map_err(anyhow::Error::msg)?;
            app.set_toast("Saved", false);
            Ok(())
        }
        app::InputAction::HermesSetProfileBaseUrl { id } => {
            let mut profile =
                droidgear_core::hermes::get_hermes_profile_for_home(&app.home_dir, &id)
                    .map_err(anyhow::Error::msg)?;
            profile.model.base_url = (!trimmed.is_empty()).then(|| trimmed.to_string());
            droidgear_core::hermes::save_hermes_profile_for_home(&app.home_dir, profile)
                .map_err(anyhow::Error::msg)?;
            app.set_toast("Saved", false);
            Ok(())
        }
        app::InputAction::HermesSetProfileApiKey { id } => {
            let mut profile =
                droidgear_core::hermes::get_hermes_profile_for_home(&app.home_dir, &id)
                    .map_err(anyhow::Error::msg)?;
            profile.model.api_key = (!trimmed.is_empty()).then(|| trimmed.to_string());
            droidgear_core::hermes::save_hermes_profile_for_home(&app.home_dir, profile)
                .map_err(anyhow::Error::msg)?;
            app.set_toast("Saved", false);
            Ok(())
        }
        app::InputAction::HermesImportSetApiKey { id } => {
            // Complete the "import from channel" flow: apply stored base_url/provider + entered api_key
            let base_url = app.hermes_import_pending_base_url.take();
            let provider = app.hermes_import_pending_provider.take();
            let mut profile =
                droidgear_core::hermes::get_hermes_profile_for_home(&app.home_dir, &id)
                    .map_err(anyhow::Error::msg)?;
            profile.model.base_url = base_url;
            profile.model.provider = provider;
            profile.model.api_key = (!trimmed.is_empty()).then(|| trimmed.to_string());
            droidgear_core::hermes::save_hermes_profile_for_home(&app.home_dir, profile.clone())
                .map_err(anyhow::Error::msg)?;
            app.hermes_detail = Some(profile);
            app.set_toast("Imported from channel", false);
            Ok(())
        }
    }
}

fn factory_model_id(
    model: Option<&droidgear_core::factory_settings::CustomModel>,
    index: usize,
) -> Option<String> {
    let model = model?;
    if let Some(id) = model.id.clone().filter(|s| !s.trim().is_empty()) {
        return Some(id);
    }
    let display = model
        .display_name
        .clone()
        .filter(|s| !s.trim().is_empty())
        .unwrap_or_else(|| model.model.clone());
    Some(format!("custom:{display}-{index}"))
}

#[cfg(test)]
mod tests {
    use super::*;

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
}
