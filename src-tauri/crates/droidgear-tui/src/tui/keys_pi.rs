use super::*;

pub(super) fn handle_pi_key(app: &mut app::App, code: KeyCode) -> Option<Action> {
    match code {
        KeyCode::Esc | KeyCode::Char('q') => app.screen = app::Screen::Main,
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
            app.screen = app::Screen::Pi;
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
        KeyCode::Esc | KeyCode::Char('q') => app.screen = app::Screen::PiProfile,
        KeyCode::Down => {
            app.pi_provider_field_index = app.pi_provider_field_index.saturating_add(1)
        }
        KeyCode::Up => app.pi_provider_field_index = app.pi_provider_field_index.saturating_sub(1),
        KeyCode::Char('m') => {
            app.pi_model_index = 0;
            app.pi_model_field_index = 0;
            app.screen = app::Screen::PiModel;
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
        KeyCode::Esc | KeyCode::Char('q') => app.screen = app::Screen::PiProvider,
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
            _ => {}
        },
        _ => {}
    }

    None
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
