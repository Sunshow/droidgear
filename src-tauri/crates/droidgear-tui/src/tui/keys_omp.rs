use super::*;

pub(super) fn handle_omp_key(app: &mut app::App, code: KeyCode) -> Option<Action> {
    match code {
        KeyCode::Esc | KeyCode::Char('q') => app.go_back(),
        KeyCode::Down => app.omp_index = app.omp_index.saturating_add(1),
        KeyCode::Up => app.omp_index = app.omp_index.saturating_sub(1),
        KeyCode::Char('r') => refresh_omp(app),
        KeyCode::Enter | KeyCode::Char('e') => {
            if let Some(p) = app.omp_profiles.get(app.omp_index) {
                app.omp_detail_id = Some(p.id.clone());
                app.omp_detail = Some(p.clone());
                app.omp_detail_field_index = 0;
                app.screen = app::Screen::OmpProfile;
            }
        }
        KeyCode::Char('a') => {
            if let Some(p) = app.omp_profiles.get(app.omp_index) {
                app.modal = Some(app::Modal::Confirm {
                    message: format!("Apply OMP profile '{}'?", p.name),
                    action: app::ConfirmAction::OmpApply { id: p.id.clone() },
                });
            }
        }
        KeyCode::Char('n') => {
            app.modal = Some(app::Modal::Input {
                title: "New OMP profile name".to_string(),
                value: String::new(),
                cursor: usize::MAX,
                is_secret: false,
                action: app::InputAction::OmpCreateProfile,
            });
        }
        KeyCode::Char('d') => {
            if let Some(p) = app.omp_profiles.get(app.omp_index) {
                app.modal = Some(app::Modal::Confirm {
                    message: format!("Delete OMP profile '{}'?", p.name),
                    action: app::ConfirmAction::OmpDelete { id: p.id.clone() },
                });
            }
        }
        KeyCode::Char('c') => {
            if let Some(p) = app.omp_profiles.get(app.omp_index) {
                app.modal = Some(app::Modal::Input {
                    title: format!("Duplicate '{}' as:", p.name),
                    value: format!("{} (copy)", p.name),
                    cursor: usize::MAX,
                    is_secret: false,
                    action: app::InputAction::OmpDuplicate { id: p.id.clone() },
                });
            }
        }
        KeyCode::Char('l') => {
            if let Some(p) = app.omp_profiles.get(app.omp_index) {
                match droidgear_core::omp::read_omp_current_config_for_home(&app.home_dir) {
                    Ok(config) => {
                        let mut updated = p.clone();
                        updated.model_roles = config.agent_config.model_roles.unwrap_or_default();
                        if let Err(e) =
                            droidgear_core::omp::save_omp_profile_for_home(&app.home_dir, updated)
                        {
                            app.set_toast(e, true);
                        } else {
                            app.set_toast("Loaded from live OMP config", false);
                            refresh_omp(app);
                        }
                    }
                    Err(e) => app.set_toast(e, true),
                }
            }
        }
        KeyCode::Char('t') => {
            app.modal = Some(app::Modal::Confirm {
                message: "Test all OMP providers?".to_string(),
                action: app::ConfirmAction::OmpTestAll,
            });
        }
        KeyCode::Char('p') => {
            refresh_omp_custom_providers(app);
            app.omp_provider_index = 0;
            app.screen = app::Screen::OmpProviders;
        }
        _ => {}
    }
    None
}

pub(super) fn handle_omp_profile_key(app: &mut app::App, code: KeyCode) -> Option<Action> {
    match code {
        KeyCode::Esc | KeyCode::Char('q') => app.go_back(),
        KeyCode::Down => app.omp_detail_field_index = app.omp_detail_field_index.saturating_add(1),
        KeyCode::Up => app.omp_detail_field_index = app.omp_detail_field_index.saturating_sub(1),
        KeyCode::Char('a') => {
            if let Some(ref p) = app.omp_detail {
                app.modal = Some(app::Modal::Confirm {
                    message: format!("Apply OMP profile '{}'?", p.name),
                    action: app::ConfirmAction::OmpApply { id: p.id.clone() },
                });
            }
        }
        KeyCode::Char('e') => match app.omp_detail_field_index {
            0 => {
                if let Some(ref p) = app.omp_detail {
                    app.modal = Some(app::Modal::Input {
                        title: "Profile name".to_string(),
                        value: p.name.clone(),
                        cursor: p.name.len(),
                        is_secret: false,
                        action: app::InputAction::OmpUpdateProfileName,
                    });
                }
            }
            1 => {
                if let Some(ref p) = app.omp_detail {
                    app.modal = Some(app::Modal::Input {
                        title: "Description".to_string(),
                        value: p.description.clone().unwrap_or_default(),
                        cursor: 0,
                        is_secret: false,
                        action: app::InputAction::OmpUpdateProfileDescription,
                    });
                }
            }
            2 => {
                let current = app
                    .omp_detail
                    .as_ref()
                    .and_then(|p| p.model_roles.default.clone())
                    .unwrap_or_default();
                app.modal = Some(app::Modal::Input {
                    title: "Default model (provider/model)".to_string(),
                    value: current,
                    cursor: 0,
                    is_secret: false,
                    action: app::InputAction::OmpUpdateModelRole {
                        role: "default".to_string(),
                    },
                });
            }
            3 => {
                let current = app
                    .omp_detail
                    .as_ref()
                    .and_then(|p| p.model_roles.smol.clone())
                    .unwrap_or_default();
                app.modal = Some(app::Modal::Input {
                    title: "Smol model (provider/model)".to_string(),
                    value: current,
                    cursor: 0,
                    is_secret: false,
                    action: app::InputAction::OmpUpdateModelRole {
                        role: "smol".to_string(),
                    },
                });
            }
            4 => {
                let current = app
                    .omp_detail
                    .as_ref()
                    .and_then(|p| p.model_roles.slow.clone())
                    .unwrap_or_default();
                app.modal = Some(app::Modal::Input {
                    title: "Slow model (provider/model)".to_string(),
                    value: current,
                    cursor: 0,
                    is_secret: false,
                    action: app::InputAction::OmpUpdateModelRole {
                        role: "slow".to_string(),
                    },
                });
            }
            5 => {
                let current = app
                    .omp_detail
                    .as_ref()
                    .and_then(|p| p.model_roles.plan.clone())
                    .unwrap_or_default();
                app.modal = Some(app::Modal::Input {
                    title: "Plan model (provider/model)".to_string(),
                    value: current,
                    cursor: 0,
                    is_secret: false,
                    action: app::InputAction::OmpUpdateModelRole {
                        role: "plan".to_string(),
                    },
                });
            }
            6 => {
                let current = app
                    .omp_detail
                    .as_ref()
                    .and_then(|p| p.model_roles.commit.clone())
                    .unwrap_or_default();
                app.modal = Some(app::Modal::Input {
                    title: "Commit model (provider/model)".to_string(),
                    value: current,
                    cursor: 0,
                    is_secret: false,
                    action: app::InputAction::OmpUpdateModelRole {
                        role: "commit".to_string(),
                    },
                });
            }
            _ => {}
        },
        _ => {}
    }
    None
}

pub(super) fn handle_omp_providers_key(app: &mut app::App, code: KeyCode) -> Option<Action> {
    match code {
        KeyCode::Esc | KeyCode::Char('q') => app.go_back(),
        KeyCode::Up => app.omp_provider_index = app.omp_provider_index.saturating_sub(1),
        KeyCode::Down => app.omp_provider_index = app.omp_provider_index.saturating_add(1),
        KeyCode::Char('r') => refresh_omp_custom_providers(app),
        KeyCode::Char('n') => {
            app.modal = Some(app::Modal::Input {
                title: "New OMP provider id".to_string(),
                value: String::new(),
                cursor: usize::MAX,
                is_secret: false,
                action: app::InputAction::OmpCreateCustomProvider,
            })
        }
        KeyCode::Enter | KeyCode::Char('e') => {
            if let Some(p) = app.omp_custom_providers.get(app.omp_provider_index) {
                app.omp_provider_id = Some(p.provider_id.clone());
                app.omp_provider_field_index = 0;
                app.screen = app::Screen::OmpProvider;
            }
        }
        KeyCode::Char('d') => {
            if let Some(p) = app.omp_custom_providers.get(app.omp_provider_index) {
                app.modal = Some(app::Modal::Confirm {
                    message: format!("Delete OMP provider '{}' ?", p.provider_id),
                    action: app::ConfirmAction::OmpDeleteCustomProvider {
                        provider_id: p.provider_id.clone(),
                    },
                });
            }
        }
        _ => {}
    }
    None
}

pub(super) fn handle_omp_provider_key(app: &mut app::App, code: KeyCode) -> Option<Action> {
    let Some(provider_id) = app.omp_provider_id.clone() else {
        app.screen = app::Screen::OmpProviders;
        return None;
    };
    let Some(entry) = app
        .omp_custom_providers
        .iter()
        .find(|p| p.provider_id == provider_id)
    else {
        app.screen = app::Screen::OmpProviders;
        return None;
    };
    let config = &entry.config;
    match code {
        KeyCode::Esc | KeyCode::Char('q') => app.go_back(),
        KeyCode::Up => {
            app.omp_provider_field_index = app.omp_provider_field_index.saturating_sub(1)
        }
        KeyCode::Down => app.omp_provider_field_index = (app.omp_provider_field_index + 1).min(3),
        KeyCode::Char('r') => refresh_omp_custom_providers(app),
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
                app.modal = Some(app::Modal::Select {
                    title: "Import/update from channel".to_string(),
                    options,
                    index: 0,
                    action: app::SelectAction::OmpImportCustomProviderFromChannel { provider_id },
                });
            }
        }
        KeyCode::Enter | KeyCode::Char('e') => match app.omp_provider_field_index {
            0 => {
                app.modal = Some(app::Modal::Input {
                    title: "Base URL".to_string(),
                    value: config.base_url.clone().unwrap_or_default(),
                    cursor: usize::MAX,
                    is_secret: false,
                    action: app::InputAction::OmpSetCustomProviderBaseUrl { provider_id },
                })
            }
            1 => {
                app.modal = Some(app::Modal::Input {
                    title: "API type".to_string(),
                    value: config.api.clone().unwrap_or_default(),
                    cursor: usize::MAX,
                    is_secret: false,
                    action: app::InputAction::OmpSetCustomProviderApi { provider_id },
                })
            }
            2 => {
                app.modal = Some(app::Modal::Input {
                    title: "API key".to_string(),
                    value: config.api_key.clone().unwrap_or_default(),
                    cursor: usize::MAX,
                    is_secret: true,
                    action: app::InputAction::OmpSetCustomProviderApiKey { provider_id },
                })
            }
            _ => {}
        },
        _ => {}
    }
    None
}
