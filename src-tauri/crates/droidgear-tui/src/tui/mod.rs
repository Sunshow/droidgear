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
use std::process::Command;
use std::time::Duration;
use tempfile::{NamedTempFile, TempDir};

mod actions;
mod keys_channels;
mod keys_claude;
mod keys_codex;
mod keys_droid_settings;
mod keys_factory;
mod keys_factory_auth;
mod keys_hermes;
mod keys_main;
mod keys_mcp;
mod keys_missions;
mod keys_openclaw;
mod keys_opencode;
mod keys_paths;
mod keys_pi;
mod keys_sessions;
mod keys_specs;
mod modal;
mod refresh;
mod utils;

#[cfg(test)]
mod tests;

pub use utils::list_claude_temporary_run_targets;
pub use utils::list_codex_temporary_run_targets;
pub use utils::list_droid_temporary_run_targets;
pub use utils::preview_claude_temporary_run_for_selector;
pub use utils::run_claude_temporary_run_for_selector;
pub use utils::run_codex_temporary_run_for_selector;
pub use utils::run_droid_temporary_run_for_settings_name;

use actions::{read_to_string_if_exists, run_action};
use keys_channels::{handle_channels_edit_key, handle_channels_key};
use keys_claude::{handle_claude_key, handle_claude_profile_key};
use keys_codex::{handle_codex_key, handle_codex_profile_key, handle_codex_provider_key};
use keys_droid_settings::handle_droid_settings_files_key;
use keys_factory::{handle_factory_key, handle_factory_model_key, normalize_factory_models};
use keys_hermes::{handle_hermes_key, handle_hermes_profile_key, handle_hermes_provider_key};
use keys_main::handle_key;
use keys_mcp::{
    handle_mcp_args_key, handle_mcp_key, handle_mcp_key_values_key, handle_mcp_server_key,
};
use keys_missions::handle_missions_key;
use keys_openclaw::{
    handle_openclaw_helpers_key, handle_openclaw_key, handle_openclaw_model_key,
    handle_openclaw_profile_key, handle_openclaw_provider_key, handle_openclaw_subagent_detail_key,
    handle_openclaw_subagents_key, openclaw_update_subagent,
};
use keys_opencode::{
    handle_opencode_key, handle_opencode_model_key, handle_opencode_profile_key,
    handle_opencode_provider_key,
};
use keys_paths::handle_paths_key;
use keys_pi::{handle_pi_key, handle_pi_model_key, handle_pi_profile_key, handle_pi_provider_key};
use keys_sessions::handle_sessions_key;
use keys_specs::handle_specs_key;
use modal::handle_modal_key;
use refresh::*;
use utils::{
    factory_model_id, insert_char_at, preview_claude_temporary_run, preview_codex_apply,
    preview_codex_temporary_run, preview_droid_temporary_run, preview_openclaw_apply,
    preview_opencode_apply, remove_char_at, run_claude_temporary_run, run_codex_temporary_run,
    run_droid_temporary_run,
};

type UiTerminal = Terminal<CrosstermBackend<io::Stdout>>;
const APP_IDENTIFIER: &str = "com.droidgear.app";

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
    PreviewDroidRun { settings_path: String },
    RunDroidRun { settings_path: String },
    PreviewClaudeRun { id: String },
    RunClaudeRun { id: String },
    PreviewCodexApply { id: String },
    PreviewCodexRun { id: String },
    RunCodexRun { id: String },
    PreviewOpenCodeApply { id: String },
    PreviewOpenClawApply { id: String },
    ViewSession { path: String },
    EditSpec { path: String },
    EditChannels,
    EditChannelAuth { id: String },
    SetActiveSettingsFile { name: Option<String> },
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
        app::Screen::Claude => refresh_claude(app),
        app::Screen::ClaudeProfile => {
            refresh_claude(app);
            refresh_claude_detail(app);
        }
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
        app::Screen::Pi => refresh_pi(app),
        app::Screen::PiProfile | app::Screen::PiProvider | app::Screen::PiModel => {
            refresh_pi(app);
            refresh_pi_detail(app);
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
        app::Screen::DroidSettingsFiles => refresh_droid_settings_files(app),
        app::Screen::FactoryAuth => keys_factory_auth::refresh_factory_auth(app),
    }
}
