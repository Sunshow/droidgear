use super::*;

use droidgear_core::omp_roles::OMP_ROLE_NAMES;

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
            index if index >= 2 && index < 2 + OMP_ROLE_NAMES.len() => {
                let role = OMP_ROLE_NAMES[index - 2];
                let current = app
                    .omp_detail
                    .as_ref()
                    .and_then(|p| droidgear_core::omp_roles::role_value(&p.model_roles, role))
                    .unwrap_or_default();
                app.modal = Some(app::Modal::Input {
                    title: format!("{role} model (provider/model)"),
                    value: current,
                    cursor: 0,
                    is_secret: false,
                    action: app::InputAction::OmpUpdateModelRole {
                        role: role.to_string(),
                    },
                });
            }
            17 => {
                let current = app
                    .omp_detail
                    .as_ref()
                    .and_then(|p| p.default_thinking_level.clone())
                    .unwrap_or_default();
                app.modal = Some(app::Modal::Input {
                    title: "Default thinking level (auto|off|minimal|low|medium|high|xhigh|max)"
                        .to_string(),
                    value: current,
                    cursor: 0,
                    is_secret: false,
                    action: app::InputAction::OmpUpdateDefaultThinkingLevel,
                });
            }
            18 => {
                let current = app
                    .omp_detail
                    .as_ref()
                    .and_then(|p| p.model_presets.as_ref())
                    .and_then(|presets| serde_json::to_string_pretty(presets).ok())
                    .unwrap_or_default();
                app.modal = Some(app::Modal::Input {
                    title: "Model presets (JSON)".to_string(),
                    value: current,
                    cursor: 0,
                    is_secret: false,
                    action: app::InputAction::OmpUpdateModelPresets,
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
        KeyCode::Down => app.omp_provider_field_index = (app.omp_provider_field_index + 1).min(4),
        KeyCode::Char('r') => refresh_omp_custom_providers(app),
        KeyCode::Char('m') => {
            // Models are edited as one JSON array so every models.yml field
            // (including the ones this build has no dedicated input for) stays
            // reachable from the terminal.
            let value = serde_json::to_string(&config.models).unwrap_or_else(|_| "[]".to_string());
            app.modal = Some(app::Modal::Input {
                title: "Models (JSON array)".to_string(),
                value,
                cursor: usize::MAX,
                is_secret: false,
                action: app::InputAction::OmpSetCustomProviderModels { provider_id },
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
            3 => {
                app.modal = Some(app::Modal::Input {
                    title: "Transport (pi-native)".to_string(),
                    value: config.transport.clone().unwrap_or_default(),
                    cursor: usize::MAX,
                    is_secret: false,
                    action: app::InputAction::OmpSetCustomProviderTransport { provider_id },
                })
            }
            _ => {}
        },
        _ => {}
    }
    None
}
