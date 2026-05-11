use super::*;

pub(super) fn handle_modal_key(app: &mut app::App, code: KeyCode, modal: app::Modal) {
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

pub(super) fn run_select_action(
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
        app::SelectAction::ClaudeSetProfileReasoningEffort { id } => {
            let mut profile =
                droidgear_core::claude::get_claude_profile_for_home(&app.home_dir, &id)
                    .map_err(anyhow::Error::msg)?;
            profile.reasoning_effort = match selected.as_deref() {
                Some("(inherit)") | None => None,
                Some("low") => Some(droidgear_core::claude::ClaudeReasoningEffort::Low),
                Some("medium") => Some(droidgear_core::claude::ClaudeReasoningEffort::Medium),
                Some("high") => Some(droidgear_core::claude::ClaudeReasoningEffort::High),
                Some("max") => Some(droidgear_core::claude::ClaudeReasoningEffort::Max),
                Some(_) => return Err(anyhow::Error::msg("Invalid reasoning effort")),
            };
            droidgear_core::claude::save_claude_profile_for_home(&app.home_dir, profile)
                .map_err(anyhow::Error::msg)?;
            app.set_toast("Saved", false);
            Ok(())
        }
        app::SelectAction::ClaudeSetProfileThinkingMode { id } => {
            let mut profile =
                droidgear_core::claude::get_claude_profile_for_home(&app.home_dir, &id)
                    .map_err(anyhow::Error::msg)?;
            profile.thinking_mode = match selected.as_deref() {
                Some("inherit") | None => droidgear_core::claude::ClaudeThinkingMode::Inherit,
                Some("on") => droidgear_core::claude::ClaudeThinkingMode::On,
                Some("off") => droidgear_core::claude::ClaudeThinkingMode::Off,
                Some(_) => return Err(anyhow::Error::msg("Invalid thinking mode")),
            };
            droidgear_core::claude::save_claude_profile_for_home(&app.home_dir, profile)
                .map_err(anyhow::Error::msg)?;
            app.set_toast("Saved", false);
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
        app::SelectAction::PiSetProviderApi {
            profile_id,
            provider_id,
        } => {
            let Some(selected) = selected else {
                return Ok(());
            };
            let mut profile =
                droidgear_core::pi::get_pi_profile_for_home(&app.home_dir, &profile_id)
                    .map_err(anyhow::Error::msg)?;
            let Some(provider) = profile.providers.get_mut(&provider_id) else {
                return Err(anyhow::Error::msg("Provider not found"));
            };
            provider.api = Some(selected);
            droidgear_core::pi::save_pi_profile_for_home(&app.home_dir, profile)
                .map_err(anyhow::Error::msg)?;
            app.set_toast("Saved", false);
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

pub(super) fn run_confirm_action(
    app: &mut app::App,
    action: app::ConfirmAction,
) -> anyhow::Result<()> {
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
        app::ConfirmAction::ClaudeApply { id } => {
            droidgear_core::claude::apply_claude_profile_for_home(&app.home_dir, &id)
                .map_err(anyhow::Error::msg)?;
            app.set_toast("Applied", false);
            Ok(())
        }
        app::ConfirmAction::ClaudeDelete { id } => {
            droidgear_core::claude::delete_claude_profile_for_home(&app.home_dir, &id)
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
        app::ConfirmAction::PiApply { id } => {
            droidgear_core::pi::apply_pi_profile_for_home(&app.home_dir, &id)
                .map_err(anyhow::Error::msg)?;
            app.set_toast("Applied", false);
            Ok(())
        }
        app::ConfirmAction::PiDelete { id } => {
            droidgear_core::pi::delete_pi_profile_for_home(&app.home_dir, &id)
                .map_err(anyhow::Error::msg)?;
            Ok(())
        }
        app::ConfirmAction::PiDeleteProvider {
            profile_id,
            provider_id,
        } => {
            let mut profile =
                droidgear_core::pi::get_pi_profile_for_home(&app.home_dir, &profile_id)
                    .map_err(anyhow::Error::msg)?;
            profile.providers.remove(&provider_id);
            droidgear_core::pi::save_pi_profile_for_home(&app.home_dir, profile)
                .map_err(anyhow::Error::msg)?;
            Ok(())
        }
        app::ConfirmAction::PiDeleteModel {
            profile_id,
            provider_id,
            model_index,
        } => {
            let mut profile =
                droidgear_core::pi::get_pi_profile_for_home(&app.home_dir, &profile_id)
                    .map_err(anyhow::Error::msg)?;
            let Some(provider) = profile.providers.get_mut(&provider_id) else {
                return Err(anyhow::Error::msg("Provider not found"));
            };
            if model_index < provider.models.len() {
                provider.models.remove(model_index);
            }
            droidgear_core::pi::save_pi_profile_for_home(&app.home_dir, profile)
                .map_err(anyhow::Error::msg)?;
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
        app::ConfirmAction::FactoryAuthDelete { name } => {
            droidgear_core::factory_auth_profiles::delete_profile_for_home(&app.home_dir, &name)
                .map_err(anyhow::Error::msg)?;
            Ok(())
        }
        app::ConfirmAction::FactoryAuthSwitch { name } => {
            droidgear_core::factory_auth_profiles::switch_profile_for_home(&app.home_dir, &name)
                .map_err(anyhow::Error::msg)?;
            Ok(())
        }
    }
}

pub(super) fn run_input_action(
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
        app::InputAction::ClaudeCreateProfile => {
            if trimmed.is_empty() {
                return Err(anyhow::Error::msg("Profile name is required"));
            }

            let before = droidgear_core::claude::list_claude_profiles_for_home(&app.home_dir)
                .map_err(anyhow::Error::msg)?;
            let before_ids = before
                .iter()
                .map(|profile| profile.id.clone())
                .collect::<std::collections::HashSet<String>>();

            let profile = droidgear_core::claude::ClaudeCodeProfile {
                id: String::new(),
                name: trimmed.to_string(),
                description: None,
                base_url: None,
                bearer_token: None,
                model: None,
                small_model_uses_main_model: true,
                small_model: None,
                reasoning_effort: None,
                thinking_mode: droidgear_core::claude::ClaudeThinkingMode::Inherit,
                created_at: String::new(),
                updated_at: String::new(),
            };

            droidgear_core::claude::save_claude_profile_for_home(&app.home_dir, profile)
                .map_err(anyhow::Error::msg)?;

            refresh_claude(app);
            if let Some((idx, profile)) = app
                .claude_profiles
                .iter()
                .enumerate()
                .find(|(_, profile)| !before_ids.contains(&profile.id))
            {
                app.claude_index = idx;
                app.claude_detail_id = Some(profile.id.clone());
                app.claude_detail_field_index = 0;
                app.screen = app::Screen::ClaudeProfile;
                refresh_claude_detail(app);
            }

            Ok(())
        }
        app::InputAction::ClaudeDuplicate { id } => {
            if trimmed.is_empty() {
                return Err(anyhow::Error::msg("Profile name is required"));
            }
            let new_profile = droidgear_core::claude::duplicate_claude_profile_for_home(
                &app.home_dir,
                &id,
                trimmed,
            )
            .map_err(anyhow::Error::msg)?;
            refresh_claude(app);
            if let Some(idx) = app
                .claude_profiles
                .iter()
                .position(|profile| profile.id == new_profile.id)
            {
                app.claude_index = idx;
            }
            Ok(())
        }
        app::InputAction::ClaudeSetProfileName { id } => {
            if trimmed.is_empty() {
                return Err(anyhow::Error::msg("Profile name is required"));
            }
            let mut profile =
                droidgear_core::claude::get_claude_profile_for_home(&app.home_dir, &id)
                    .map_err(anyhow::Error::msg)?;
            profile.name = trimmed.to_string();
            droidgear_core::claude::save_claude_profile_for_home(&app.home_dir, profile)
                .map_err(anyhow::Error::msg)?;
            app.set_toast("Saved", false);
            Ok(())
        }
        app::InputAction::ClaudeSetProfileDescription { id } => {
            let mut profile =
                droidgear_core::claude::get_claude_profile_for_home(&app.home_dir, &id)
                    .map_err(anyhow::Error::msg)?;
            profile.description = (!trimmed.is_empty()).then(|| trimmed.to_string());
            droidgear_core::claude::save_claude_profile_for_home(&app.home_dir, profile)
                .map_err(anyhow::Error::msg)?;
            app.set_toast("Saved", false);
            Ok(())
        }
        app::InputAction::ClaudeSetProfileBaseUrl { id } => {
            let mut profile =
                droidgear_core::claude::get_claude_profile_for_home(&app.home_dir, &id)
                    .map_err(anyhow::Error::msg)?;
            profile.base_url = (!trimmed.is_empty()).then(|| trimmed.to_string());
            droidgear_core::claude::save_claude_profile_for_home(&app.home_dir, profile)
                .map_err(anyhow::Error::msg)?;
            app.set_toast("Saved", false);
            Ok(())
        }
        app::InputAction::ClaudeSetProfileBearerToken { id } => {
            let mut profile =
                droidgear_core::claude::get_claude_profile_for_home(&app.home_dir, &id)
                    .map_err(anyhow::Error::msg)?;
            profile.bearer_token = (!trimmed.is_empty()).then(|| trimmed.to_string());
            droidgear_core::claude::save_claude_profile_for_home(&app.home_dir, profile)
                .map_err(anyhow::Error::msg)?;
            app.set_toast("Saved", false);
            Ok(())
        }
        app::InputAction::ClaudeSetProfileModel { id } => {
            let mut profile =
                droidgear_core::claude::get_claude_profile_for_home(&app.home_dir, &id)
                    .map_err(anyhow::Error::msg)?;
            profile.model = (!trimmed.is_empty()).then(|| trimmed.to_string());
            droidgear_core::claude::save_claude_profile_for_home(&app.home_dir, profile)
                .map_err(anyhow::Error::msg)?;
            app.set_toast("Saved", false);
            Ok(())
        }
        app::InputAction::ClaudeSetProfileSmallModel { id } => {
            let mut profile =
                droidgear_core::claude::get_claude_profile_for_home(&app.home_dir, &id)
                    .map_err(anyhow::Error::msg)?;
            profile.small_model = (!trimmed.is_empty()).then(|| trimmed.to_string());
            droidgear_core::claude::save_claude_profile_for_home(&app.home_dir, profile)
                .map_err(anyhow::Error::msg)?;
            app.set_toast("Saved", false);
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
        app::InputAction::PiCreateProfile => {
            if trimmed.is_empty() {
                return Err(anyhow::Error::msg("Profile name is required"));
            }

            let before = droidgear_core::pi::list_pi_profiles_for_home(&app.home_dir)
                .map_err(anyhow::Error::msg)?;
            let before_ids = before
                .iter()
                .map(|p| p.id.clone())
                .collect::<std::collections::HashSet<String>>();

            let profile = droidgear_core::pi::PiProfile {
                id: String::new(),
                name: trimmed.to_string(),
                description: None,
                created_at: String::new(),
                updated_at: String::new(),
                providers: std::collections::HashMap::new(),
            };
            droidgear_core::pi::save_pi_profile_for_home(&app.home_dir, profile)
                .map_err(anyhow::Error::msg)?;

            refresh_pi(app);
            if let Some((idx, p)) = app
                .pi_profiles
                .iter()
                .enumerate()
                .find(|(_, p)| !before_ids.contains(&p.id))
            {
                app.pi_index = idx;
                app.pi_detail_id = Some(p.id.clone());
                app.pi_detail_field_index = 0;
                app.pi_provider_index = 0;
                app.pi_provider_field_index = 0;
                app.pi_model_index = 0;
                app.pi_model_field_index = 0;
                app.screen = app::Screen::PiProfile;
                refresh_pi_detail(app);
            }

            Ok(())
        }
        app::InputAction::PiDuplicate { id } => {
            if trimmed.is_empty() {
                return Err(anyhow::Error::msg("Profile name is required"));
            }
            let new_profile =
                droidgear_core::pi::duplicate_pi_profile_for_home(&app.home_dir, &id, trimmed)
                    .map_err(anyhow::Error::msg)?;
            refresh_pi(app);
            if let Some(idx) = app.pi_profiles.iter().position(|p| p.id == new_profile.id) {
                app.pi_index = idx;
            }
            Ok(())
        }
        app::InputAction::PiSetProfileName { id } => {
            if trimmed.is_empty() {
                return Err(anyhow::Error::msg("Profile name is required"));
            }
            let mut profile = droidgear_core::pi::get_pi_profile_for_home(&app.home_dir, &id)
                .map_err(anyhow::Error::msg)?;
            profile.name = trimmed.to_string();
            droidgear_core::pi::save_pi_profile_for_home(&app.home_dir, profile)
                .map_err(anyhow::Error::msg)?;
            app.set_toast("Saved", false);
            Ok(())
        }
        app::InputAction::PiSetProfileDescription { id } => {
            let mut profile = droidgear_core::pi::get_pi_profile_for_home(&app.home_dir, &id)
                .map_err(anyhow::Error::msg)?;
            profile.description = (!trimmed.is_empty()).then(|| trimmed.to_string());
            droidgear_core::pi::save_pi_profile_for_home(&app.home_dir, profile)
                .map_err(anyhow::Error::msg)?;
            app.set_toast("Saved", false);
            Ok(())
        }
        app::InputAction::PiAddProvider { profile_id } => {
            if trimmed.is_empty() {
                return Err(anyhow::Error::msg("Provider id is required"));
            }
            let mut profile =
                droidgear_core::pi::get_pi_profile_for_home(&app.home_dir, &profile_id)
                    .map_err(anyhow::Error::msg)?;
            if profile.providers.contains_key(trimmed) {
                return Err(anyhow::Error::msg("Provider already exists"));
            }
            profile.providers.insert(
                trimmed.to_string(),
                droidgear_core::pi::PiProviderConfig::default(),
            );
            droidgear_core::pi::save_pi_profile_for_home(&app.home_dir, profile)
                .map_err(anyhow::Error::msg)?;

            app.pi_provider_index = 0;
            app.pi_provider_field_index = 0;
            app.pi_model_index = 0;
            app.pi_model_field_index = 0;
            refresh_pi_detail(app);
            Ok(())
        }
        app::InputAction::PiSetProviderBaseUrl {
            profile_id,
            provider_id,
        } => {
            let mut profile =
                droidgear_core::pi::get_pi_profile_for_home(&app.home_dir, &profile_id)
                    .map_err(anyhow::Error::msg)?;
            let Some(provider) = profile.providers.get_mut(&provider_id) else {
                return Err(anyhow::Error::msg("Provider not found"));
            };
            provider.base_url = (!trimmed.is_empty()).then(|| trimmed.to_string());
            droidgear_core::pi::save_pi_profile_for_home(&app.home_dir, profile)
                .map_err(anyhow::Error::msg)?;
            app.set_toast("Saved", false);
            Ok(())
        }
        app::InputAction::PiSetProviderApiKey {
            profile_id,
            provider_id,
        } => {
            let mut profile =
                droidgear_core::pi::get_pi_profile_for_home(&app.home_dir, &profile_id)
                    .map_err(anyhow::Error::msg)?;
            let Some(provider) = profile.providers.get_mut(&provider_id) else {
                return Err(anyhow::Error::msg("Provider not found"));
            };
            provider.api_key = (!trimmed.is_empty()).then(|| trimmed.to_string());
            droidgear_core::pi::save_pi_profile_for_home(&app.home_dir, profile)
                .map_err(anyhow::Error::msg)?;
            app.set_toast("Saved", false);
            Ok(())
        }
        app::InputAction::PiAddModel {
            profile_id,
            provider_id,
        } => {
            if trimmed.is_empty() {
                return Err(anyhow::Error::msg("Model id is required"));
            }
            let mut profile =
                droidgear_core::pi::get_pi_profile_for_home(&app.home_dir, &profile_id)
                    .map_err(anyhow::Error::msg)?;
            let Some(provider) = profile.providers.get_mut(&provider_id) else {
                return Err(anyhow::Error::msg("Provider not found"));
            };
            let new_index = provider.models.len();
            provider.models.push(droidgear_core::pi::PiModel {
                id: trimmed.to_string(),
                name: None,
                api: None,
                reasoning: false,
                input: vec!["text".to_string()],
                context_window: 128000,
                max_tokens: 16384,
                cost: None,
                compat: None,
            });
            droidgear_core::pi::save_pi_profile_for_home(&app.home_dir, profile)
                .map_err(anyhow::Error::msg)?;
            app.pi_model_index = new_index;
            app.pi_model_field_index = 0;
            app.screen = app::Screen::PiModel;
            refresh_pi_detail(app);
            Ok(())
        }
        app::InputAction::PiSetModelId {
            profile_id,
            provider_id,
            model_index,
        } => {
            if trimmed.is_empty() {
                return Err(anyhow::Error::msg("Model id is required"));
            }
            let mut profile =
                droidgear_core::pi::get_pi_profile_for_home(&app.home_dir, &profile_id)
                    .map_err(anyhow::Error::msg)?;
            let Some(provider) = profile.providers.get_mut(&provider_id) else {
                return Err(anyhow::Error::msg("Provider not found"));
            };
            let Some(model) = provider.models.get_mut(model_index) else {
                return Err(anyhow::Error::msg("Model not found"));
            };
            model.id = trimmed.to_string();
            droidgear_core::pi::save_pi_profile_for_home(&app.home_dir, profile)
                .map_err(anyhow::Error::msg)?;
            app.set_toast("Saved", false);
            Ok(())
        }
        app::InputAction::PiSetModelName {
            profile_id,
            provider_id,
            model_index,
        } => {
            let mut profile =
                droidgear_core::pi::get_pi_profile_for_home(&app.home_dir, &profile_id)
                    .map_err(anyhow::Error::msg)?;
            let Some(provider) = profile.providers.get_mut(&provider_id) else {
                return Err(anyhow::Error::msg("Provider not found"));
            };
            let Some(model) = provider.models.get_mut(model_index) else {
                return Err(anyhow::Error::msg("Model not found"));
            };
            model.name = (!trimmed.is_empty()).then(|| trimmed.to_string());
            droidgear_core::pi::save_pi_profile_for_home(&app.home_dir, profile)
                .map_err(anyhow::Error::msg)?;
            app.set_toast("Saved", false);
            Ok(())
        }
        app::InputAction::PiSetModelContextWindow {
            profile_id,
            provider_id,
            model_index,
        } => {
            let context_window = trimmed
                .parse::<u32>()
                .map_err(|_| anyhow::Error::msg("Invalid context window"))?;
            let mut profile =
                droidgear_core::pi::get_pi_profile_for_home(&app.home_dir, &profile_id)
                    .map_err(anyhow::Error::msg)?;
            let Some(provider) = profile.providers.get_mut(&provider_id) else {
                return Err(anyhow::Error::msg("Provider not found"));
            };
            let Some(model) = provider.models.get_mut(model_index) else {
                return Err(anyhow::Error::msg("Model not found"));
            };
            model.context_window = context_window;
            droidgear_core::pi::save_pi_profile_for_home(&app.home_dir, profile)
                .map_err(anyhow::Error::msg)?;
            app.set_toast("Saved", false);
            Ok(())
        }
        app::InputAction::PiSetModelMaxTokens {
            profile_id,
            provider_id,
            model_index,
        } => {
            let max_tokens = trimmed
                .parse::<u32>()
                .map_err(|_| anyhow::Error::msg("Invalid max tokens"))?;
            let mut profile =
                droidgear_core::pi::get_pi_profile_for_home(&app.home_dir, &profile_id)
                    .map_err(anyhow::Error::msg)?;
            let Some(provider) = profile.providers.get_mut(&provider_id) else {
                return Err(anyhow::Error::msg("Provider not found"));
            };
            let Some(model) = provider.models.get_mut(model_index) else {
                return Err(anyhow::Error::msg("Model not found"));
            };
            model.max_tokens = max_tokens;
            droidgear_core::pi::save_pi_profile_for_home(&app.home_dir, profile)
                .map_err(anyhow::Error::msg)?;
            app.set_toast("Saved", false);
            Ok(())
        }
        app::InputAction::PiSetModelCost {
            profile_id,
            provider_id,
            model_index,
        } => {
            // Parse comma-separated: input,output,cacheRead,cacheWrite
            let parts: Vec<&str> = trimmed.split(',').map(|s| s.trim()).collect();
            if parts.len() != 4 {
                return Err(anyhow::Error::msg(
                    "Expected 4 comma-separated values: input,output,cacheRead,cacheWrite",
                ));
            }
            let input = parts[0]
                .parse::<f64>()
                .map_err(|_| anyhow::Error::msg("Invalid cost input value"))?;
            let output = parts[1]
                .parse::<f64>()
                .map_err(|_| anyhow::Error::msg("Invalid cost output value"))?;
            let cache_read = parts[2]
                .parse::<f64>()
                .map_err(|_| anyhow::Error::msg("Invalid cost cacheRead value"))?;
            let cache_write = parts[3]
                .parse::<f64>()
                .map_err(|_| anyhow::Error::msg("Invalid cost cacheWrite value"))?;
            let mut profile =
                droidgear_core::pi::get_pi_profile_for_home(&app.home_dir, &profile_id)
                    .map_err(anyhow::Error::msg)?;
            let Some(provider) = profile.providers.get_mut(&provider_id) else {
                return Err(anyhow::Error::msg("Provider not found"));
            };
            let Some(model) = provider.models.get_mut(model_index) else {
                return Err(anyhow::Error::msg("Model not found"));
            };
            model.cost = Some(droidgear_core::pi::PiModelCost {
                input,
                output,
                cache_read,
                cache_write,
            });
            droidgear_core::pi::save_pi_profile_for_home(&app.home_dir, profile)
                .map_err(anyhow::Error::msg)?;
            app.set_toast("Saved", false);
            Ok(())
        }
        app::InputAction::FactoryAuthSaveProfile => {
            if trimmed.is_empty() {
                return Err(anyhow::Error::msg("Profile ID is required"));
            }
            droidgear_core::factory_auth_profiles::save_current_as_profile_for_home(
                &app.home_dir,
                trimmed,
                trimmed,
            )
            .map_err(anyhow::Error::msg)?;
            app.set_toast(format!("Saved as '{trimmed}'"), false);
            Ok(())
        }
        app::InputAction::FactoryAuthRename { name } => {
            if trimmed.is_empty() {
                return Err(anyhow::Error::msg("Label is required"));
            }
            droidgear_core::factory_auth_profiles::rename_profile_for_home(
                &app.home_dir,
                &name,
                trimmed,
            )
            .map_err(anyhow::Error::msg)?;
            app.set_toast("Renamed", false);
            Ok(())
        }
    }
}
