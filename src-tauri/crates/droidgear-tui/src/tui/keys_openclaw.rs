use super::*;

pub(super) fn handle_openclaw_key(app: &mut app::App, code: KeyCode) -> Option<Action> {
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

pub(super) fn openclaw_available_model_refs(
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

pub(super) fn openclaw_load_from_live_config(
    app: &mut app::App,
    profile_id: &str,
) -> anyhow::Result<()> {
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

pub(super) fn handle_openclaw_profile_key(app: &mut app::App, code: KeyCode) -> Option<Action> {
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

pub(super) fn handle_openclaw_provider_key(app: &mut app::App, code: KeyCode) -> Option<Action> {
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

pub(super) fn handle_openclaw_model_key(app: &mut app::App, code: KeyCode) -> Option<Action> {
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
                    title: "Max output tokens".to_string(),
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

pub(super) fn openclaw_toggle_model_reasoning(
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

pub(super) fn openclaw_toggle_model_input(
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

pub(super) fn handle_openclaw_helpers_key(app: &mut app::App, code: KeyCode) -> Option<Action> {
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

pub(super) fn openclaw_toggle_telegram_block_streaming(
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

pub(super) fn openclaw_reset_helpers(app: &mut app::App, profile_id: &str) -> anyhow::Result<()> {
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

pub(super) fn openclaw_subagent_allowed_ids(
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

pub(super) fn handle_openclaw_subagents_key(app: &mut app::App, code: KeyCode) -> Option<Action> {
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

pub(super) fn handle_openclaw_subagent_detail_key(
    app: &mut app::App,
    code: KeyCode,
) -> Option<Action> {
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

pub(super) fn openclaw_update_subagent(
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
