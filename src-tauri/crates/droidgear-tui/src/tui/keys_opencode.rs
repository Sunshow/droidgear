use super::*;

pub(super) fn handle_opencode_key(app: &mut app::App, code: KeyCode) -> Option<Action> {
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

pub(super) fn handle_opencode_profile_key(app: &mut app::App, code: KeyCode) -> Option<Action> {
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

pub(super) fn handle_opencode_provider_key(app: &mut app::App, code: KeyCode) -> Option<Action> {
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

pub(super) fn handle_opencode_model_key(app: &mut app::App, code: KeyCode) -> Option<Action> {
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
