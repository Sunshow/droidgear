use super::*;

pub(super) fn handle_hermes_key(app: &mut app::App, code: KeyCode) -> Option<Action> {
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

pub(super) fn handle_hermes_profile_key(app: &mut app::App, code: KeyCode) -> Option<Action> {
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

pub(super) fn handle_hermes_provider_key(app: &mut app::App, code: KeyCode) -> Option<Action> {
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
