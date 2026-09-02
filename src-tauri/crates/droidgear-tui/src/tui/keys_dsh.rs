use super::*;

pub(super) fn handle_dsh_key(app: &mut app::App, code: KeyCode) -> Option<Action> {
    match code {
        KeyCode::Esc | KeyCode::Char('q') => app.go_back(),
        KeyCode::Down => app.dsh_index = app.dsh_index.saturating_add(1),
        KeyCode::Up => app.dsh_index = app.dsh_index.saturating_sub(1),
        KeyCode::Char('r') => refresh_dsh(app),
        KeyCode::Char('n') => {
            app.modal = Some(app::Modal::Input {
                title: "New Dsh provider id".to_string(),
                value: String::new(),
                cursor: usize::MAX,
                is_secret: false,
                action: app::InputAction::DshAddProvider,
            });
        }
        KeyCode::Char('i') => {
            app.modal = Some(app::Modal::Input {
                title: "New provider id (from channel)".to_string(),
                value: String::new(),
                cursor: usize::MAX,
                is_secret: false,
                action: app::InputAction::DshAddProviderFromChannel,
            });
        }
        KeyCode::Enter | KeyCode::Char('e') => {
            if let Some(provider_id) = app.dsh_current_provider_id() {
                app.dsh_provider_id = Some(provider_id);
                app.dsh_provider_field_index = 0;
                app.dsh_model_index = 0;
                app.dsh_model_field_index = 0;
                app.screen = app::Screen::DshProvider;
                refresh_dsh(app);
            }
        }
        KeyCode::Char('d') => {
            if let Some(provider_id) = app.dsh_current_provider_id() {
                app.modal = Some(app::Modal::Confirm {
                    message: format!("Delete Dsh provider '{provider_id}'?"),
                    action: app::ConfirmAction::DshDeleteProvider { provider_id },
                });
            }
        }
        _ => {}
    }
    None
}

pub(super) fn handle_dsh_provider_key(app: &mut app::App, code: KeyCode) -> Option<Action> {
    let Some(provider_id) = app.dsh_provider_id.clone() else {
        app.screen = app::Screen::Dsh;
        return None;
    };
    let Some(config) = app.dsh_current_provider().cloned() else {
        app.set_toast("Provider not found", true);
        app.screen = app::Screen::Dsh;
        return None;
    };

    let fields_count = 6usize; // DisplayName, BaseURL, API, APIKeyEnv, API Key value, SupportsDevRole
    let model_count = config.models.len();

    match code {
        KeyCode::Esc | KeyCode::Char('q') => app.go_back(),
        KeyCode::Down => {
            let total = fields_count + model_count;
            if total > 0 {
                app.dsh_provider_field_index = (app.dsh_provider_field_index + 1).min(total - 1);
            }
        }
        KeyCode::Up => {
            app.dsh_provider_field_index = app.dsh_provider_field_index.saturating_sub(1);
        }
        KeyCode::Char('r') => refresh_dsh(app),
        KeyCode::Enter | KeyCode::Char('e') => {
            if app.dsh_provider_field_index < fields_count {
                match app.dsh_provider_field_index {
                    0 => {
                        app.modal = Some(app::Modal::Input {
                            title: "Display name".to_string(),
                            value: config.display_name.clone().unwrap_or_default(),
                            cursor: usize::MAX,
                            is_secret: false,
                            action: app::InputAction::DshSetProviderDisplayName {
                                provider_id: provider_id.clone(),
                            },
                        });
                    }
                    1 => {
                        app.modal = Some(app::Modal::Input {
                            title: "Base URL".to_string(),
                            value: config.base_url.clone().unwrap_or_default(),
                            cursor: usize::MAX,
                            is_secret: false,
                            action: app::InputAction::DshSetProviderBaseUrl {
                                provider_id: provider_id.clone(),
                            },
                        });
                    }
                    2 => {
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
                            action: app::SelectAction::DshSetProviderApi {
                                provider_id: provider_id.clone(),
                            },
                        });
                    }
                    3 => {
                        app.modal = Some(app::Modal::Input {
                            title: "API key env var".to_string(),
                            value: config.api_key_env.clone().unwrap_or_default(),
                            cursor: usize::MAX,
                            is_secret: false,
                            action: app::InputAction::DshSetProviderApiKeyEnv {
                                provider_id: provider_id.clone(),
                            },
                        });
                    }
                    4 => {
                        let env_name = config.api_key_env.clone().unwrap_or_default();
                        if env_name.trim().is_empty() {
                            app.set_toast("API key env var not set", true);
                        } else {
                            let current = app
                                .dsh_credentials
                                .get(&env_name)
                                .cloned()
                                .unwrap_or_default();
                            app.modal = Some(app::Modal::Input {
                                title: format!("API key value ({env_name})"),
                                value: current,
                                cursor: usize::MAX,
                                is_secret: true,
                                action: app::InputAction::DshSetProviderApiKey {
                                    provider_id: provider_id.clone(),
                                },
                            });
                        }
                    }
                    5 => {
                        if !droidgear_core::dsh::supports_developer_role_protocol(
                            config.api.as_deref(),
                        ) {
                            app.set_toast(
                                "supportsDeveloperRole only applies to openai-completions/openai-responses/azure-openai-responses/openai-codex-responses",
                                true,
                            );
                        } else if let Err(e) = dsh_toggle_supports_developer_role(app, &provider_id)
                        {
                            app.set_toast(e.to_string(), true);
                        } else {
                            refresh_dsh(app);
                        }
                    }
                    _ => {}
                }
            } else {
                // Open the selected model
                app.dsh_model_index = app.dsh_provider_field_index - fields_count;
                app.dsh_model_field_index = 0;
                app.screen = app::Screen::DshModel;
            }
        }
        KeyCode::Char('n') => {
            app.modal = Some(app::Modal::Input {
                title: "New model id".to_string(),
                value: String::new(),
                cursor: usize::MAX,
                is_secret: false,
                action: app::InputAction::DshAddModel {
                    provider_id: provider_id.clone(),
                },
            });
        }
        KeyCode::Char('f') => {
            return Some(Action::FetchDshModels {
                provider_id: provider_id.clone(),
            });
        }
        KeyCode::Char('d') if app.dsh_provider_field_index >= fields_count => {
            let model_index = app.dsh_provider_field_index - fields_count;
            if let Some(model) = config.models.get(model_index) {
                app.modal = Some(app::Modal::Confirm {
                    message: format!("Delete model '{}'?", model.id),
                    action: app::ConfirmAction::DshDeleteModel {
                        provider_id: provider_id.clone(),
                        model_index,
                    },
                });
            }
        }
        _ => {}
    }

    None
}

pub(super) fn handle_dsh_model_key(app: &mut app::App, code: KeyCode) -> Option<Action> {
    let Some(provider_id) = app.dsh_provider_id.clone() else {
        app.screen = app::Screen::Dsh;
        return None;
    };
    let Some(config) = app.dsh_current_provider() else {
        app.set_toast("Provider not found", true);
        app.screen = app::Screen::Dsh;
        return None;
    };
    let model_index = app.dsh_model_index;
    let Some(model) = config.models.get(model_index) else {
        app.set_toast("Model not found", true);
        app.screen = app::Screen::DshProvider;
        return None;
    };

    match code {
        KeyCode::Esc | KeyCode::Char('q') => app.go_back(),
        KeyCode::Down => app.dsh_model_field_index = app.dsh_model_field_index.saturating_add(1),
        KeyCode::Up => app.dsh_model_field_index = app.dsh_model_field_index.saturating_sub(1),
        KeyCode::Enter | KeyCode::Char('e') => match app.dsh_model_field_index {
            0 => {
                app.modal = Some(app::Modal::Input {
                    title: "Model id".to_string(),
                    value: model.id.clone(),
                    cursor: usize::MAX,
                    is_secret: false,
                    action: app::InputAction::DshSetModelId {
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
                    action: app::InputAction::DshSetModelName {
                        provider_id: provider_id.clone(),
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
                    action: app::InputAction::DshSetModelContextWindow {
                        provider_id: provider_id.clone(),
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
                    action: app::InputAction::DshSetModelMaxTokens {
                        provider_id: provider_id.clone(),
                        model_index,
                    },
                });
            }
            4 => {
                app.set_toast(
                    "Reasoning efforts auto-adapt from the model registry",
                    false,
                );
            }
            _ => {}
        },
        _ => {}
    }

    None
}

pub(super) fn dsh_default_provider_config() -> droidgear_core::dsh::DshProviderConfig {
    droidgear_core::dsh::DshProviderConfig {
        compat: Some(droidgear_core::dsh::DshCompatConfig {
            supports_developer_role: Some(false),
            ..Default::default()
        }),
        ..Default::default()
    }
}

pub(super) fn dsh_toggle_supports_developer_role(
    app: &mut app::App,
    provider_id: &str,
) -> anyhow::Result<()> {
    let mut config = droidgear_core::dsh::read_dsh_current_config_for_home(&app.home_dir)
        .map_err(anyhow::Error::msg)?;
    let Some(provider) = config.providers.get_mut(provider_id) else {
        return Err(anyhow::Error::msg("Provider not found"));
    };
    if !droidgear_core::dsh::supports_developer_role_protocol(provider.api.as_deref()) {
        return Err(anyhow::Error::msg(
            "supportsDeveloperRole only applies to openai-completions/openai-responses/azure-openai-responses/openai-codex-responses",
        ));
    }
    let current = provider
        .compat
        .as_ref()
        .and_then(|c| c.supports_developer_role)
        .unwrap_or(false);
    let compat = provider.compat.get_or_insert_with(Default::default);
    compat.supports_developer_role = Some(!current);
    droidgear_core::dsh::save_dsh_provider_for_home(&app.home_dir, provider_id, provider)
        .map_err(anyhow::Error::msg)?;
    Ok(())
}
