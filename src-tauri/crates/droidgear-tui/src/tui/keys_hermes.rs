use super::*;

pub(super) fn handle_hermes_key(app: &mut app::App, code: KeyCode) -> Option<Action> {
    match code {
        KeyCode::Esc | KeyCode::Char('q') => app.go_back(),
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
                app.hermes_model_index = 0;
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

pub(super) fn handle_hermes_profile_key(app: &mut app::App, code: KeyCode) -> Option<Action> {
    let Some(profile_id) = app.hermes_detail_id.clone() else {
        app.screen = app::Screen::Hermes;
        return None;
    };
    let Some(profile) = app.hermes_detail.as_ref() else {
        return None;
    };
    let models_count = profile.models.len();

    match code {
        KeyCode::Esc | KeyCode::Char('q') => {
            app.go_back();
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
            if models_count > 0 {
                app.hermes_model_index = app.hermes_model_index.min(models_count - 1);
                app.hermes_provider_field_index = 0;
                app.screen = app::Screen::HermesProvider;
            }
        }
        KeyCode::Char('n') => {
            if let Err(e) = hermes_add_model(app, &profile_id) {
                app.set_toast(e.to_string(), true);
            }
        }
        KeyCode::Char('s') => {
            let index = if app.hermes_detail_field_index >= 3 {
                app.hermes_detail_field_index - 3
            } else {
                app.hermes_model_index.min(models_count.saturating_sub(1))
            };
            if models_count > 0 {
                if let Err(e) = hermes_set_default_model(app, &profile_id, index) {
                    app.set_toast(e.to_string(), true);
                } else {
                    app.set_toast("Set as default", false);
                }
            }
        }
        KeyCode::Char('d') => {
            let index = if app.hermes_detail_field_index >= 3 {
                app.hermes_detail_field_index - 3
            } else {
                app.hermes_model_index.min(models_count.saturating_sub(1))
            };
            if models_count > 0 {
                let name = profile
                    .models
                    .get(index)
                    .and_then(|m| m.name.clone())
                    .unwrap_or_else(|| format!("#{}", index + 1));
                app.modal = Some(app::Modal::Confirm {
                    message: format!("Delete model config '{}'?", name),
                    action: app::ConfirmAction::HermesDeleteModel {
                        id: profile_id.clone(),
                        model_index: index,
                    },
                });
            }
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
                    let options = vec![
                        "(none)".to_string(),
                        "none".to_string(),
                        "minimal".to_string(),
                        "low".to_string(),
                        "medium".to_string(),
                        "high".to_string(),
                        "xhigh".to_string(),
                        "max".to_string(),
                        "ultra".to_string(),
                    ];
                    let index = profile
                        .reasoning_effort
                        .as_deref()
                        .and_then(|v| options.iter().position(|o| o == v))
                        .unwrap_or(0);
                    app.modal = Some(app::Modal::Select {
                        title: "Reasoning effort".to_string(),
                        options,
                        index,
                        action: app::SelectAction::HermesSetProfileReasoningEffort {
                            id: profile_id.clone(),
                        },
                    });
                }
                idx if idx >= 3 => {
                    // Open the selected model entry
                    let model_index = idx - 3;
                    if model_index < models_count {
                        app.hermes_model_index = model_index;
                        app.hermes_provider_field_index = 0;
                        app.screen = app::Screen::HermesProvider;
                    }
                }
                _ => {}
            }
        }
        _ => {}
    }

    // 在条目行上移动时同步 hermes_model_index，便于 s/d 快捷键操作
    if app.hermes_detail_field_index >= 3 {
        app.hermes_model_index = app
            .hermes_detail_field_index
            .saturating_sub(3)
            .min(models_count.saturating_sub(1));
    }

    None
}

pub(super) fn handle_hermes_provider_key(app: &mut app::App, code: KeyCode) -> Option<Action> {
    let Some(profile_id) = app.hermes_detail_id.clone() else {
        app.screen = app::Screen::Hermes;
        return None;
    };
    let Some(profile) = app.hermes_detail.as_ref() else {
        return None;
    };
    let Some(entry) = profile.models.get(app.hermes_model_index) else {
        app.go_back();
        return None;
    };
    let entry = entry.clone();
    let model_index = app.hermes_model_index;

    match code {
        KeyCode::Esc | KeyCode::Char('q') => {
            app.go_back();
        }
        KeyCode::Down => {
            app.hermes_provider_field_index = app.hermes_provider_field_index.saturating_add(1)
        }
        KeyCode::Up => {
            app.hermes_provider_field_index = app.hermes_provider_field_index.saturating_sub(1)
        }
        KeyCode::Char('r') => refresh_hermes_detail(app),
        KeyCode::Char('s') => {
            if let Err(e) = hermes_set_default_model(app, &profile_id, model_index) {
                app.set_toast(e.to_string(), true);
            } else {
                app.set_toast("Set as default", false);
            }
        }
        KeyCode::Char('d') => {
            let name = entry
                .name
                .clone()
                .unwrap_or_else(|| format!("#{}", model_index + 1));
            app.modal = Some(app::Modal::Confirm {
                message: format!("Delete model config '{}'?", name),
                action: app::ConfirmAction::HermesDeleteModel {
                    id: profile_id.clone(),
                    model_index,
                },
            });
        }
        KeyCode::Char('i') => {
            // Import from channel: refresh and present channel list
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
                    action: app::SelectAction::HermesImportFromChannel {
                        profile_id,
                        model_index,
                    },
                });
            }
        }
        KeyCode::Enter | KeyCode::Char('e') => match app.hermes_provider_field_index {
            0 => {
                app.modal = Some(app::Modal::Input {
                    title: "Name (provider reference name)".to_string(),
                    value: entry.name.unwrap_or_default(),
                    cursor: usize::MAX,
                    is_secret: false,
                    action: app::InputAction::HermesSetModelName {
                        id: profile_id,
                        model_index,
                    },
                });
            }
            1 => {
                app.modal = Some(app::Modal::Input {
                    title: "Default model".to_string(),
                    value: entry.default.unwrap_or_default(),
                    cursor: usize::MAX,
                    is_secret: false,
                    action: app::InputAction::HermesSetModelDefault {
                        id: profile_id,
                        model_index,
                    },
                });
            }
            2 => {
                app.modal = Some(app::Modal::Input {
                    title: "Provider".to_string(),
                    value: entry.provider.unwrap_or_default(),
                    cursor: usize::MAX,
                    is_secret: false,
                    action: app::InputAction::HermesSetModelProvider {
                        id: profile_id,
                        model_index,
                    },
                });
            }
            3 => {
                app.modal = Some(app::Modal::Input {
                    title: "Base URL".to_string(),
                    value: entry.base_url.unwrap_or_default(),
                    cursor: usize::MAX,
                    is_secret: false,
                    action: app::InputAction::HermesSetModelBaseUrl {
                        id: profile_id,
                        model_index,
                    },
                });
            }
            4 => {
                app.modal = Some(app::Modal::Input {
                    title: "API key".to_string(),
                    value: entry.api_key.unwrap_or_default(),
                    cursor: usize::MAX,
                    is_secret: true,
                    action: app::InputAction::HermesSetModelApiKey {
                        id: profile_id,
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
