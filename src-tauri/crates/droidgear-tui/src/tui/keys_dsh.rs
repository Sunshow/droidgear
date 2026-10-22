use super::*;

pub(super) fn handle_dsh_key(app: &mut app::App, code: KeyCode) -> Option<Action> {
    match code {
        KeyCode::Esc | KeyCode::Char('q') => app.go_back(),
        KeyCode::Down => app.dsh_index = app.dsh_index.saturating_add(1),
        KeyCode::Up => app.dsh_index = app.dsh_index.saturating_sub(1),
        KeyCode::Char('r') => refresh_dsh(app),
        KeyCode::Char('c') => {
            // AGENTS.md rule 9: the DSH config surfaces have a TUI peer view.
            app.dsh_config_index = 0;
            app.dsh_config_field_index = 0;
            app.dsh_config_focus = app::DshConfigFocus::Entries;
            app.dsh_config_expanded.clear();
            app.screen = app::Screen::DshConfig;
            request_dsh_config_refresh(app);
        }
        KeyCode::Char('o') => {
            // The official DeepSeek route keeps its key in `.credentials.yaml`
            // under a configurable ref name (default DEEPSEEK_API_KEY).
            let env_name = resolve_dsh_official_api_key_env(app);
            app.modal = Some(app::Modal::Input {
                title: format!("DeepSeek official API key [{env_name}] (empty = clear)"),
                value: String::new(),
                cursor: usize::MAX,
                is_secret: true,
                action: app::InputAction::DshSetOfficialApiKey,
            });
        }
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
        app.screen = app.dsh_list_screen();
        return None;
    };
    let Some(config) = app.dsh_current_provider().cloned() else {
        app.set_toast("Provider not found", true);
        app.screen = app.dsh_list_screen();
        return None;
    };

    // DisplayName, BaseURL, API, APIKeyEnv, API Key value, SupportsDevRole,
    // DefaultReasoning (pi-ai route default thinking level)
    let fields_count = 7usize;
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
                            title: "API key env var (empty = auto)".to_string(),
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
                    6 => {
                        app.modal = Some(app::Modal::Input {
                            title: "Default reasoning (off/minimal/low/medium/high/xhigh/max; empty = inherit)"
                                .to_string(),
                            value: config.reasoning.clone().unwrap_or_default(),
                            cursor: usize::MAX,
                            is_secret: false,
                            action: app::InputAction::DshSetProviderReasoning {
                                provider_id: provider_id.clone(),
                            },
                        });
                    }
                    _ => {}
                }
            } else {
                // Open the selected model
                app.dsh_model_index = app.dsh_provider_field_index - fields_count;
                app.dsh_model_field_index = 0;
                app.screen = app::Screen::DshModel;
                // The title shows the current `agent-default-model`; load the
                // config snapshot in the background.
                request_dsh_config_refresh(app);
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
    // `g` writes the `agent-default-model` entry and needs `&mut App`; handle
    // it before borrowing the current provider/model below.
    if code == KeyCode::Char('g') {
        dsh_set_default_model(app);
        return None;
    }
    let Some(provider_id) = app.dsh_provider_id.clone() else {
        app.screen = app.dsh_list_screen();
        return None;
    };
    let Some(config) = app.dsh_current_provider() else {
        app.set_toast("Provider not found", true);
        app.screen = app.dsh_list_screen();
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
            5 => {
                let options: Vec<String> = DSH_MODEL_INPUT_OPTIONS
                    .iter()
                    .map(|option| option.to_string())
                    .collect();
                let index = dsh_model_input_index(model.input.as_deref());
                app.modal = Some(app::Modal::Select {
                    title: format!("{} input modalities", model.id),
                    options,
                    index,
                    action: app::SelectAction::DshSetModelInput {
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
    let mut config = droidgear_core::dsh::read_dsh_current_config_for_profile(
        &app.home_dir,
        app.dsh_active_profile.as_deref(),
    )
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
    dsh_save_provider(app, provider_id, provider)?;
    Ok(())
}

/// Validation shared by the pre-submit modal check (which keeps the dialog
/// open) and the action handler (which must not write an invalid level).
pub(super) fn dsh_provider_reasoning_error(value: &str) -> Option<String> {
    let normalized = value.trim().to_ascii_lowercase();
    if normalized.is_empty()
        || droidgear_core::dsh::DSH_REASONING_LEVELS.contains(&normalized.as_str())
    {
        return None;
    }
    Some(format!(
        "Invalid reasoning level '{value}'; expected one of: {}",
        droidgear_core::dsh::DSH_REASONING_LEVELS.join(", ")
    ))
}

/// Validation hook for `Modal::Input` submissions that must keep the dialog
/// open on invalid values.
pub(super) fn dsh_input_action_error(action: &app::InputAction, value: &str) -> Option<String> {
    match action {
        app::InputAction::DshSetProviderReasoning { .. } => dsh_provider_reasoning_error(value),
        _ => None,
    }
}

/// Write the current provider/model as the profile's `agent-default-model`
/// entry, using the composed config snapshot hash as an optimistic lock.
pub(super) fn dsh_set_default_model(app: &mut app::App) {
    let Some(provider_id) = app.dsh_provider_id.clone() else {
        app.set_toast("Provider not found", true);
        return;
    };
    let Some(provider) = app.dsh_current_provider().cloned() else {
        app.set_toast("Provider not found", true);
        return;
    };
    let Some(model) = provider.models.get(app.dsh_model_index).cloned() else {
        app.set_toast("Model not found", true);
        return;
    };

    let snapshot = match droidgear_core::dsh_config::read_dsh_config_snapshot_for_home(
        &app.home_dir,
        app.dsh_active_profile.clone(),
    ) {
        Ok(snapshot) => snapshot,
        Err(error) => {
            app.set_toast(error, true);
            return;
        }
    };
    let Some(profile) = snapshot.profile.clone() else {
        app.set_toast("No DSH profile found under ~/.dsh/profiles", true);
        return;
    };

    let mut config = serde_json::Map::new();
    config.insert(
        "provider".to_string(),
        serde_json::Value::String(provider_id.clone()),
    );
    config.insert(
        "model".to_string(),
        serde_json::Value::String(model.id.clone()),
    );
    // DSH resolves the route default from `providers.<id>.reasoning`; only
    // write the matching `reasoningEffort` when the provider declares one.
    if let Some(reasoning) = provider
        .reasoning
        .as_deref()
        .map(str::trim)
        .filter(|level| !level.is_empty())
    {
        config.insert(
            "reasoningEffort".to_string(),
            serde_json::Value::String(reasoning.to_ascii_lowercase()),
        );
    }

    match droidgear_core::dsh_config::save_dsh_config_entry_for_home(
        &app.home_dir,
        Some(profile),
        "agent-default-model",
        Some(serde_json::Value::Object(config)),
        Some(snapshot.patch_hash),
    ) {
        Ok(result) => {
            let suffix = if result.changed { "" } else { " (unchanged)" };
            app.set_toast(
                format!("Default model: {provider_id} / {}{suffix}", model.id),
                false,
            );
            request_dsh_config_refresh(app);
        }
        Err(error) => app.set_toast(error, true),
    }
}

/// Select labels for one model's pi-ai input modalities. The last entry
/// clears the explicit list so core re-enriches it from the model registry.
pub(super) const DSH_MODEL_INPUT_OPTIONS: [&str; 3] = ["text", "text+image", "(auto: registry)"];

/// Index of the current modalities in [`DSH_MODEL_INPUT_OPTIONS`].
pub(super) fn dsh_model_input_index(input: Option<&[String]>) -> usize {
    match input {
        Some(values)
            if values
                .iter()
                .any(|value| value.eq_ignore_ascii_case("image")) =>
        {
            1
        }
        Some(values) if !values.is_empty() => 0,
        _ => 2,
    }
}

/// Map a [`DSH_MODEL_INPUT_OPTIONS`] label back to `DshModel.input`
/// (`None` = leave it unset so core fills it from the registry).
pub(super) fn dsh_model_input_from_label(label: &str) -> Option<Vec<String>> {
    match label {
        "text" => Some(vec!["text".to_string()]),
        "text+image" => Some(vec!["text".to_string(), "image".to_string()]),
        _ => None,
    }
}

/// Credential env var name the Models page derives for a provider id:
/// uppercase, non-alphanumerics (except `_`) become `_`, then `_API_KEY`.
pub(super) fn dsh_default_api_key_env(provider_id: &str) -> String {
    let upper = provider_id.to_uppercase();
    let sanitized: String = upper
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || c == '_' {
                c
            } else {
                '_'
            }
        })
        .collect();
    format!("{sanitized}_API_KEY")
}

/// Save a provider from the TUI. An empty `apiKeyEnv` falls back to the
/// derived `<PROVIDER_ID>_API_KEY` reference, so saving an API key value never
/// produces a provider that cannot reference it.
pub(super) fn dsh_save_provider(
    app: &app::App,
    provider_id: &str,
    config: &droidgear_core::dsh::DshProviderConfig,
) -> anyhow::Result<()> {
    let mut config = config.clone();
    if !provider_id.is_empty()
        && config
            .api_key_env
            .as_deref()
            .is_none_or(|env| env.trim().is_empty())
    {
        config.api_key_env = Some(dsh_default_api_key_env(provider_id));
    }
    droidgear_core::dsh::save_dsh_provider_for_profile(
        &app.home_dir,
        app.dsh_active_profile.as_deref(),
        provider_id,
        &config,
    )
    .map_err(anyhow::Error::msg)
}
