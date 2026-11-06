use super::*;

pub(super) fn handle_pi_key(app: &mut app::App, code: KeyCode) -> Option<Action> {
    match code {
        KeyCode::Esc | KeyCode::Char('q') => app.go_back(),
        KeyCode::Down => app.pi_index = app.pi_index.saturating_add(1),
        KeyCode::Up => app.pi_index = app.pi_index.saturating_sub(1),
        KeyCode::Char('r') => refresh_pi(app),
        KeyCode::Char('n') => {
            app.modal = Some(app::Modal::Input {
                title: "New Pi profile name".to_string(),
                value: String::new(),
                cursor: usize::MAX,
                is_secret: false,
                action: app::InputAction::PiCreateProfile,
            });
        }
        KeyCode::Enter | KeyCode::Char('e') => {
            if let Some(p) = app.pi_profiles.get(app.pi_index) {
                app.pi_detail_id = Some(p.id.clone());
                app.pi_detail_field_index = 0;
                app.pi_provider_index = 0;
                app.pi_provider_field_index = 0;
                app.pi_model_index = 0;
                app.pi_model_field_index = 0;
                app.screen = app::Screen::PiProfile;
                refresh_pi_detail(app);
            }
        }
        KeyCode::Char('a') => {
            if let Some(p) = app.pi_profiles.get(app.pi_index) {
                app.modal = Some(app::Modal::Confirm {
                    message: format!("Apply Pi profile '{}'?", p.name),
                    action: app::ConfirmAction::PiApply { id: p.id.clone() },
                });
            }
        }
        KeyCode::Char('d') => {
            if let Some(p) = app.pi_profiles.get(app.pi_index) {
                app.modal = Some(app::Modal::Confirm {
                    message: format!("Delete Pi profile '{}'?", p.name),
                    action: app::ConfirmAction::PiDelete { id: p.id.clone() },
                });
            }
        }
        KeyCode::Char('c') => {
            if let Some(p) = app.pi_profiles.get(app.pi_index) {
                app.modal = Some(app::Modal::Input {
                    title: "Duplicate profile name".to_string(),
                    value: format!("{} (copy)", p.name),
                    cursor: usize::MAX,
                    is_secret: false,
                    action: app::InputAction::PiDuplicate { id: p.id.clone() },
                });
            }
        }
        // Startup model selection lives in ~/.pi/agent/settings.json, not in a
        // profile, so it gets its own keys on the profile list.
        KeyCode::Char('D') => {
            let current = droidgear_core::pi::read_pi_settings_for_home(&app.home_dir)
                .map(|status| status.default_provider.unwrap_or_default())
                .unwrap_or_default();
            app.modal = Some(app::Modal::Input {
                title: "Default provider (empty = automatic)".to_string(),
                value: current,
                cursor: usize::MAX,
                is_secret: false,
                action: app::InputAction::PiSetDefaultProvider,
            });
        }
        KeyCode::Char('M') => {
            let current = droidgear_core::pi::read_pi_settings_for_home(&app.home_dir)
                .map(|status| status.default_model.unwrap_or_default())
                .unwrap_or_default();
            app.modal = Some(app::Modal::Input {
                title: "Default model id (empty = automatic)".to_string(),
                value: current,
                cursor: usize::MAX,
                is_secret: false,
                action: app::InputAction::PiSetDefaultModel,
            });
        }
        KeyCode::Char('T') => {
            let current = droidgear_core::pi::read_pi_settings_for_home(&app.home_dir)
                .map(|status| status.default_thinking_level.unwrap_or_default())
                .unwrap_or_default();
            app.modal = Some(app::Modal::Input {
                title: "Default thinking level (off|minimal|low|medium|high|xhigh|max)".to_string(),
                value: current,
                cursor: usize::MAX,
                is_secret: false,
                action: app::InputAction::PiSetDefaultThinkingLevel,
            });
        }
        KeyCode::Char('x') => {
            app.modal = Some(app::Modal::Confirm {
                message: "Clear the Pi startup model selection?".to_string(),
                action: app::ConfirmAction::PiClearDefaults,
            });
        }
        _ => {}
    }
    None
}

pub(super) fn handle_pi_profile_key(app: &mut app::App, code: KeyCode) -> Option<Action> {
    let Some(profile_id) = app.pi_detail_id.clone() else {
        app.screen = app::Screen::Pi;
        return None;
    };
    let Some(profile) = app.pi_detail.as_ref() else {
        return None;
    };

    let fields_count = 2usize; // Name, Description
    let provider_count = profile.providers.len();

    match code {
        KeyCode::Esc | KeyCode::Char('q') => {
            app.go_back();
        }
        KeyCode::Down => {
            let total = fields_count + provider_count;
            if total > 0 {
                app.pi_detail_field_index = (app.pi_detail_field_index + 1).min(total - 1);
            }
        }
        KeyCode::Up => {
            app.pi_detail_field_index = app.pi_detail_field_index.saturating_sub(1);
        }
        KeyCode::Char('r') => refresh_pi_detail(app),
        KeyCode::Char('a') => {
            app.modal = Some(app::Modal::Confirm {
                message: format!("Apply Pi profile '{}'?", profile.name),
                action: app::ConfirmAction::PiApply {
                    id: profile_id.clone(),
                },
            });
        }
        KeyCode::Char('p') if provider_count > 0 => {
            // Navigate to provider detail for the currently selected provider
            app.pi_provider_index = app.pi_detail_field_index.saturating_sub(fields_count);
            app.pi_provider_field_index = 0;
            app.pi_model_index = 0;
            app.pi_model_field_index = 0;
            app.screen = app::Screen::PiProvider;
        }
        KeyCode::Char('l') => {
            if let Err(e) = pi_load_from_live_config(app, &profile_id) {
                app.set_toast(e.to_string(), true);
            } else {
                app.set_toast("Loaded from live config", false);
                refresh_pi_detail(app);
            }
        }
        KeyCode::Enter | KeyCode::Char('e') => {
            if app.pi_detail_field_index < fields_count {
                // Editing profile fields
                match app.pi_detail_field_index {
                    0 => {
                        app.modal = Some(app::Modal::Input {
                            title: "Profile name".to_string(),
                            value: profile.name.clone(),
                            cursor: usize::MAX,
                            is_secret: false,
                            action: app::InputAction::PiSetProfileName {
                                id: profile_id.clone(),
                            },
                        });
                    }
                    1 => {
                        app.modal = Some(app::Modal::Input {
                            title: "Profile description".to_string(),
                            value: profile.description.clone().unwrap_or_default(),
                            cursor: usize::MAX,
                            is_secret: false,
                            action: app::InputAction::PiSetProfileDescription {
                                id: profile_id.clone(),
                            },
                        });
                    }
                    _ => {}
                }
            } else {
                // Open the selected provider's detail screen
                let prov_idx = app.pi_detail_field_index - fields_count;
                app.pi_provider_index = prov_idx;
                app.pi_provider_field_index = 0;
                app.pi_model_index = 0;
                app.pi_model_field_index = 0;
                app.screen = app::Screen::PiProvider;
            }
        }
        KeyCode::Char('n') => {
            app.modal = Some(app::Modal::Input {
                title: "New provider id".to_string(),
                value: String::new(),
                cursor: usize::MAX,
                is_secret: false,
                action: app::InputAction::PiAddProvider {
                    profile_id: profile_id.clone(),
                },
            });
        }
        KeyCode::Char('i') => {
            refresh_channels(app);
            let options: Vec<String> = app
                .channels
                .iter()
                .filter(|c| c.enabled)
                .map(|c| format!("{} ({})", c.name, c.base_url))
                .collect();
            if options.is_empty() {
                app.set_toast("No channels configured", true);
            } else {
                app.modal = Some(app::Modal::Input {
                    title: "New provider id (create from channel)".to_string(),
                    value: String::new(),
                    cursor: usize::MAX,
                    is_secret: false,
                    action: app::InputAction::PiAddProviderFromChannel {
                        profile_id: profile_id.clone(),
                    },
                });
            }
        }
        KeyCode::Char('d') if app.pi_detail_field_index >= fields_count => {
            let prov_idx = app.pi_detail_field_index - fields_count;
            // Get the provider ID at this index
            let mut keys: Vec<String> = profile.providers.keys().cloned().collect();
            keys.sort_by_key(|a| a.to_lowercase());
            if let Some(provider_id) = keys.get(prov_idx).cloned() {
                app.modal = Some(app::Modal::Confirm {
                    message: format!("Delete provider '{provider_id}'?"),
                    action: app::ConfirmAction::PiDeleteProvider {
                        profile_id: profile_id.clone(),
                        provider_id,
                    },
                });
            }
        }
        _ => {}
    }

    None
}

pub(super) fn handle_pi_provider_key(app: &mut app::App, code: KeyCode) -> Option<Action> {
    let Some(profile_id) = app.pi_detail_id.clone() else {
        app.screen = app::Screen::Pi;
        return None;
    };
    let Some(provider_id) = app.pi_current_provider_id() else {
        app.screen = app::Screen::PiProfile;
        return None;
    };
    let Some(profile) = app.pi_detail.as_ref() else {
        return None;
    };
    let Some(config) = profile.providers.get(&provider_id) else {
        app.set_toast("Provider not found", true);
        app.screen = app::Screen::PiProfile;
        return None;
    };

    match code {
        KeyCode::Esc | KeyCode::Char('q') => app.go_back(),
        KeyCode::Down => {
            app.pi_provider_field_index = app.pi_provider_field_index.saturating_add(1)
        }
        KeyCode::Up => app.pi_provider_field_index = app.pi_provider_field_index.saturating_sub(1),
        KeyCode::Char('m') => {
            app.pi_model_index = 0;
            app.pi_model_field_index = 0;
            app.screen = app::Screen::PiModel;
        }
        KeyCode::Char('t') => {
            return Some(Action::TestPiProvider {
                provider_id,
                config: Box::new(config.clone()),
            });
        }
        KeyCode::Enter | KeyCode::Char('e') => match app.pi_provider_field_index {
            0 => {
                app.modal = Some(app::Modal::Input {
                    title: "Base URL".to_string(),
                    value: config.base_url.clone().unwrap_or_default(),
                    cursor: usize::MAX,
                    is_secret: false,
                    action: app::InputAction::PiSetProviderBaseUrl {
                        profile_id: profile_id.clone(),
                        provider_id: provider_id.clone(),
                    },
                });
            }
            1 => {
                let options = vec![
                    "openai-completions".to_string(),
                    "openai-responses".to_string(),
                    "anthropic-messages".to_string(),
                    "google-generative-ai".to_string(),
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
                    action: app::SelectAction::PiSetProviderApi {
                        profile_id: profile_id.clone(),
                        provider_id: provider_id.clone(),
                    },
                });
            }
            2 => {
                app.modal = Some(app::Modal::Input {
                    title: "API key".to_string(),
                    value: config.api_key.clone().unwrap_or_default(),
                    cursor: usize::MAX,
                    is_secret: true,
                    action: app::InputAction::PiSetProviderApiKey {
                        profile_id: profile_id.clone(),
                        provider_id: provider_id.clone(),
                    },
                });
            }
            3 => {
                app.modal = Some(app::Modal::Input {
                    title: "Provider name".to_string(),
                    value: config.name.clone().unwrap_or_default(),
                    cursor: usize::MAX,
                    is_secret: false,
                    action: app::InputAction::PiSetProviderName {
                        profile_id: profile_id.clone(),
                        provider_id: provider_id.clone(),
                    },
                });
            }
            _ => {}
        },
        KeyCode::Char('n') => {
            app.modal = Some(app::Modal::Input {
                title: "New model id".to_string(),
                value: String::new(),
                cursor: usize::MAX,
                is_secret: false,
                action: app::InputAction::PiAddModel {
                    profile_id: profile_id.clone(),
                    provider_id: provider_id.clone(),
                },
            });
        }
        KeyCode::Char('d')
            if !config.models.is_empty() && app.pi_model_index < config.models.len() =>
        {
            let model_id = config.models[app.pi_model_index].id.clone();
            app.modal = Some(app::Modal::Confirm {
                message: format!("Delete model '{model_id}'?"),
                action: app::ConfirmAction::PiDeleteModel {
                    profile_id: profile_id.clone(),
                    provider_id: provider_id.clone(),
                    model_index: app.pi_model_index,
                },
            });
        }
        KeyCode::Char('i') => {
            // Import from channel: refresh channels first
            refresh_channels(app);
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
                    action: app::SelectAction::PiImportFromChannel {
                        profile_id: profile_id.clone(),
                        provider_id: provider_id.clone(),
                    },
                });
            }
        }
        _ => {}
    }

    None
}

pub(super) fn handle_pi_model_key(app: &mut app::App, code: KeyCode) -> Option<Action> {
    let Some(profile_id) = app.pi_detail_id.clone() else {
        app.screen = app::Screen::Pi;
        return None;
    };
    let Some(provider_id) = app.pi_current_provider_id() else {
        app.screen = app::Screen::PiProfile;
        return None;
    };
    let Some(profile) = app.pi_detail.as_ref() else {
        return None;
    };
    let Some(provider) = profile.providers.get(&provider_id) else {
        app.set_toast("Provider not found", true);
        app.screen = app::Screen::PiProfile;
        return None;
    };
    let model_index = app.pi_model_index;
    let Some(model) = provider.models.get(model_index) else {
        app.set_toast("Model not found", true);
        app.screen = app::Screen::PiProvider;
        return None;
    };

    match code {
        KeyCode::Esc | KeyCode::Char('q') => app.go_back(),
        KeyCode::Down => app.pi_model_field_index = app.pi_model_field_index.saturating_add(1),
        KeyCode::Up => app.pi_model_field_index = app.pi_model_field_index.saturating_sub(1),
        KeyCode::Enter | KeyCode::Char('e') => match app.pi_model_field_index {
            0 => {
                app.modal = Some(app::Modal::Input {
                    title: "Model id".to_string(),
                    value: model.id.clone(),
                    cursor: usize::MAX,
                    is_secret: false,
                    action: app::InputAction::PiSetModelId {
                        profile_id: profile_id.clone(),
                        provider_id: provider_id.clone(),
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
                    action: app::InputAction::PiSetModelName {
                        profile_id: profile_id.clone(),
                        provider_id: provider_id.clone(),
                        model_index,
                    },
                });
            }
            2 => {
                // Toggle reasoning
                if let Err(e) =
                    pi_toggle_model_reasoning(app, &profile_id, &provider_id, model_index)
                {
                    app.set_toast(e.to_string(), true);
                } else {
                    refresh_pi_detail(app);
                }
            }
            3 => {
                // Toggle input type (text <-> text+image)
                if let Err(e) = pi_toggle_model_input(app, &profile_id, &provider_id, model_index) {
                    app.set_toast(e.to_string(), true);
                } else {
                    refresh_pi_detail(app);
                }
            }
            4 => {
                app.modal = Some(app::Modal::Input {
                    title: "Context window".to_string(),
                    value: model.context_window.to_string(),
                    cursor: usize::MAX,
                    is_secret: false,
                    action: app::InputAction::PiSetModelContextWindow {
                        profile_id: profile_id.clone(),
                        provider_id: provider_id.clone(),
                        model_index,
                    },
                });
            }
            5 => {
                app.modal = Some(app::Modal::Input {
                    title: "Max tokens".to_string(),
                    value: model.max_tokens.to_string(),
                    cursor: usize::MAX,
                    is_secret: false,
                    action: app::InputAction::PiSetModelMaxTokens {
                        profile_id: profile_id.clone(),
                        provider_id: provider_id.clone(),
                        model_index,
                    },
                });
            }
            6 => {
                // Edit cost as comma-separated: input,output,cacheRead,cacheWrite
                let cost_str = match &model.cost {
                    Some(c) => format!(
                        "{},{},{},{}",
                        c.input, c.output, c.cache_read, c.cache_write
                    ),
                    None => "0,0,0,0".to_string(),
                };
                app.modal = Some(app::Modal::Input {
                    title: "Cost (input,output,cacheRead,cacheWrite)".to_string(),
                    value: cost_str,
                    cursor: usize::MAX,
                    is_secret: false,
                    action: app::InputAction::PiSetModelCost {
                        profile_id: profile_id.clone(),
                        provider_id: provider_id.clone(),
                        model_index,
                    },
                });
            }
            7 => {
                app.modal = Some(app::Modal::Input {
                    title: "Model type (chat|image|classifier)".to_string(),
                    value: model.model_type.clone().unwrap_or_default(),
                    cursor: usize::MAX,
                    is_secret: false,
                    action: app::InputAction::PiSetModelType {
                        profile_id: profile_id.clone(),
                        provider_id: provider_id.clone(),
                        model_index,
                    },
                });
            }
            8 => {
                app.modal = Some(app::Modal::Input {
                    title: "Model API".to_string(),
                    value: model.api.clone().unwrap_or_default(),
                    cursor: usize::MAX,
                    is_secret: false,
                    action: app::InputAction::PiSetModelApi {
                        profile_id: profile_id.clone(),
                        provider_id: provider_id.clone(),
                        model_index,
                    },
                });
            }
            9 => {
                app.modal = Some(app::Modal::Input {
                    title: "Model base URL".to_string(),
                    value: model.base_url.clone().unwrap_or_default(),
                    cursor: usize::MAX,
                    is_secret: false,
                    action: app::InputAction::PiSetModelBaseUrl {
                        profile_id: profile_id.clone(),
                        provider_id: provider_id.clone(),
                        model_index,
                    },
                });
            }
            10 => {
                app.modal = Some(app::Modal::Input {
                    title: "inputLimits (JSON)".to_string(),
                    value: json_field_value(model.input_limits.as_ref()),
                    cursor: usize::MAX,
                    is_secret: false,
                    action: app::InputAction::PiSetModelInputLimits {
                        profile_id: profile_id.clone(),
                        provider_id: provider_id.clone(),
                        model_index,
                    },
                });
            }
            11 => {
                app.modal = Some(app::Modal::Input {
                    title: "promptCache (JSON)".to_string(),
                    value: json_field_value(model.prompt_cache.as_ref()),
                    cursor: usize::MAX,
                    is_secret: false,
                    action: app::InputAction::PiSetModelPromptCache {
                        profile_id: profile_id.clone(),
                        provider_id: provider_id.clone(),
                        model_index,
                    },
                });
            }
            12 => {
                app.modal = Some(app::Modal::Input {
                    title: "samplingParams (JSON)".to_string(),
                    value: sampling_params_value(model),
                    cursor: usize::MAX,
                    is_secret: false,
                    action: app::InputAction::PiSetModelSamplingParams {
                        profile_id: profile_id.clone(),
                        provider_id: provider_id.clone(),
                        model_index,
                    },
                });
            }
            _ => {}
        },
        _ => {}
    }

    None
}

/// Render an optional JSON field for the input modal ("" when unset).
pub(super) fn json_field_value(value: Option<&serde_json::Value>) -> String {
    match value {
        Some(value) => serde_json::to_string(value).unwrap_or_default(),
        None => String::new(),
    }
}

/// Render the model's sampling params (including the per-thinking-level map) as
/// one JSON object so a single input can cover both fields.
fn sampling_params_value(model: &droidgear_core::pi::PiModel) -> String {
    let mut object = serde_json::Map::new();
    if let Some(params) = model.sampling_params.as_ref() {
        object.insert(
            "samplingParams".to_string(),
            serde_json::Value::Object(params.clone().into_iter().collect()),
        );
    }
    if let Some(by_level) = model.sampling_params_by_thinking_level.as_ref() {
        object.insert(
            "samplingParamsByThinkingLevel".to_string(),
            serde_json::Value::Object(by_level.clone().into_iter().collect()),
        );
    }
    if object.is_empty() {
        String::new()
    } else {
        serde_json::to_string(&serde_json::Value::Object(object)).unwrap_or_default()
    }
}

pub(super) fn pi_toggle_model_reasoning(
    app: &mut app::App,
    profile_id: &str,
    provider_id: &str,
    model_index: usize,
) -> anyhow::Result<()> {
    let mut profile = droidgear_core::pi::get_pi_profile_for_home(&app.home_dir, profile_id)
        .map_err(anyhow::Error::msg)?;
    let Some(provider) = profile.providers.get_mut(provider_id) else {
        return Err(anyhow::Error::msg("Provider not found"));
    };
    let Some(model) = provider.models.get_mut(model_index) else {
        return Err(anyhow::Error::msg("Model not found"));
    };
    model.reasoning = !model.reasoning;
    droidgear_core::pi::save_pi_profile_for_home(&app.home_dir, profile)
        .map_err(anyhow::Error::msg)?;
    Ok(())
}

pub(super) fn pi_toggle_model_input(
    app: &mut app::App,
    profile_id: &str,
    provider_id: &str,
    model_index: usize,
) -> anyhow::Result<()> {
    let mut profile = droidgear_core::pi::get_pi_profile_for_home(&app.home_dir, profile_id)
        .map_err(anyhow::Error::msg)?;
    let Some(provider) = profile.providers.get_mut(provider_id) else {
        return Err(anyhow::Error::msg("Provider not found"));
    };
    let Some(model) = provider.models.get_mut(model_index) else {
        return Err(anyhow::Error::msg("Model not found"));
    };
    // Toggle between ["text"] and ["text", "image"]
    if model.input.iter().any(|t| t == "image") {
        model.input.retain(|t| t != "image");
    } else {
        model.input.push("image".to_string());
        model.input.sort();
        model.input.dedup();
    }
    droidgear_core::pi::save_pi_profile_for_home(&app.home_dir, profile)
        .map_err(anyhow::Error::msg)?;
    Ok(())
}
