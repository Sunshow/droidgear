use super::*;

pub(super) fn handle_codex_key(app: &mut app::App, code: KeyCode) -> Option<Action> {
    match code {
        KeyCode::Esc | KeyCode::Char('q') => app.screen = app::Screen::Main,
        KeyCode::Down => app.codex_index = app.codex_index.saturating_add(1),
        KeyCode::Up => app.codex_index = app.codex_index.saturating_sub(1),
        KeyCode::Char('r') => refresh_codex(app),
        KeyCode::Char('p') => {
            if let Some(p) = app.codex_profiles.get(app.codex_index) {
                return Some(Action::PreviewCodexApply { id: p.id.clone() });
            }
        }
        KeyCode::Char('t') => {
            if let Some(p) = app.codex_profiles.get(app.codex_index) {
                return Some(Action::PreviewCodexRun { id: p.id.clone() });
            }
        }
        KeyCode::Char('x') => {
            if let Some(p) = app.codex_profiles.get(app.codex_index) {
                return Some(Action::RunCodexRun { id: p.id.clone() });
            }
        }
        KeyCode::Char('E') => {
            if let Some(p) = app.codex_profiles.get(app.codex_index) {
                if p.id == "official" {
                    app.set_toast("Official profile is read-only", true);
                    return None;
                }
                return Some(Action::EditCodexProfile { id: p.id.clone() });
            }
        }
        KeyCode::Char('n') => {
            app.modal = Some(app::Modal::Input {
                title: "New Codex profile name".to_string(),
                value: String::new(),
                cursor: usize::MAX,
                is_secret: false,
                action: app::InputAction::CodexCreateProfile,
            });
        }
        KeyCode::Enter | KeyCode::Char('e') => {
            if let Some(p) = app.codex_profiles.get(app.codex_index) {
                app.codex_detail_id = Some(p.id.clone());
                app.codex_detail_focus = app::CodexDetailFocus::Fields;
                app.codex_detail_field_index = 0;
                app.codex_detail_provider_index = 0;
                app.codex_provider_id = None;
                app.codex_provider_field_index = 0;
                app.screen = app::Screen::CodexProfile;
                refresh_codex_detail(app);
            }
        }
        KeyCode::Char('a') => {
            if let Some(p) = app.codex_profiles.get(app.codex_index) {
                app.modal = Some(app::Modal::Confirm {
                    message: format!("Apply Codex profile '{}'?", p.name),
                    action: app::ConfirmAction::CodexApply { id: p.id.clone() },
                });
            }
        }
        KeyCode::Char('d') => {
            if let Some(p) = app.codex_profiles.get(app.codex_index) {
                if p.id == "official" {
                    app.set_toast("Cannot delete official profile", true);
                    return None;
                }
                app.modal = Some(app::Modal::Confirm {
                    message: format!("Delete Codex profile '{}'?", p.name),
                    action: app::ConfirmAction::CodexDelete { id: p.id.clone() },
                });
            }
        }
        KeyCode::Char('c') => {
            if let Some(p) = app.codex_profiles.get(app.codex_index) {
                if p.id == "official" {
                    app.set_toast("Official profile is read-only", true);
                    return None;
                }
                app.modal = Some(app::Modal::Input {
                    title: "Duplicate profile name".to_string(),
                    value: format!("{} (copy)", p.name),
                    cursor: usize::MAX,
                    is_secret: false,
                    action: app::InputAction::CodexDuplicate { id: p.id.clone() },
                });
            }
        }
        _ => {}
    }
    None
}

pub(super) fn handle_codex_profile_key(app: &mut app::App, code: KeyCode) -> Option<Action> {
    let Some(profile_id) = app.codex_detail_id.clone() else {
        app.screen = app::Screen::Codex;
        return None;
    };
    let Some(profile) = app.codex_detail.as_ref() else {
        return None;
    };
    let is_official = profile_id == "official";

    match code {
        KeyCode::Esc | KeyCode::Char('q') => {
            app.screen = app::Screen::Codex;
            app.codex_provider_id = None;
        }
        KeyCode::Tab => {
            app.codex_detail_focus = match app.codex_detail_focus {
                app::CodexDetailFocus::Fields => app::CodexDetailFocus::Providers,
                app::CodexDetailFocus::Providers => app::CodexDetailFocus::Fields,
            };
        }
        KeyCode::Down => match app.codex_detail_focus {
            app::CodexDetailFocus::Fields => {
                app.codex_detail_field_index = app.codex_detail_field_index.saturating_add(1)
            }
            app::CodexDetailFocus::Providers => {
                app.codex_detail_provider_index = app.codex_detail_provider_index.saturating_add(1)
            }
        },
        KeyCode::Up => match app.codex_detail_focus {
            app::CodexDetailFocus::Fields => {
                app.codex_detail_field_index = app.codex_detail_field_index.saturating_sub(1)
            }
            app::CodexDetailFocus::Providers => {
                app.codex_detail_provider_index = app.codex_detail_provider_index.saturating_sub(1)
            }
        },
        KeyCode::Char('r') => refresh_codex_detail(app),
        KeyCode::Char('p') => return Some(Action::PreviewCodexApply { id: profile_id }),
        KeyCode::Char('t') => {
            return Some(Action::PreviewCodexRun {
                id: profile_id.clone(),
            })
        }
        KeyCode::Char('x') => {
            return Some(Action::RunCodexRun {
                id: profile_id.clone(),
            })
        }
        KeyCode::Char('E') => {
            if is_official {
                app.set_toast("Official profile is read-only", true);
                return None;
            }
            return Some(Action::EditCodexProfile { id: profile_id });
        }
        KeyCode::Char('a') => {
            app.modal = Some(app::Modal::Confirm {
                message: format!("Apply Codex profile '{}'?", profile.name),
                action: app::ConfirmAction::CodexApply { id: profile_id },
            });
        }
        KeyCode::Char('l') => {
            if is_official {
                app.set_toast("Official profile is read-only", true);
                return None;
            }
            if let Err(e) = codex_load_from_live_config(app, &profile_id) {
                app.set_toast(e.to_string(), true);
            } else {
                app.set_toast("Loaded from live config", false);
                refresh_codex_detail(app);
            }
        }
        KeyCode::Char('n') => {
            if is_official {
                app.set_toast("Official profile is read-only", true);
                return None;
            }
            if app.codex_detail_focus == app::CodexDetailFocus::Providers {
                app.modal = Some(app::Modal::Input {
                    title: "New provider id".to_string(),
                    value: String::new(),
                    cursor: usize::MAX,
                    is_secret: false,
                    action: app::InputAction::CodexAddProvider { id: profile_id },
                });
            }
        }
        KeyCode::Char('s') => {
            if is_official {
                app.set_toast("Official profile is read-only", true);
                return None;
            }
            if app.codex_detail_focus == app::CodexDetailFocus::Providers {
                if let Some(provider_id) = app
                    .codex_detail_provider_ids
                    .get(app.codex_detail_provider_index)
                    .cloned()
                {
                    if let Err(e) = codex_set_active_provider(app, &profile_id, &provider_id) {
                        app.set_toast(e.to_string(), true);
                    } else {
                        app.set_toast("Active provider set", false);
                        refresh_codex_detail(app);
                    }
                }
            }
        }
        KeyCode::Char('d') => {
            if is_official {
                app.set_toast("Official profile is read-only", true);
                return None;
            }
            if app.codex_detail_focus == app::CodexDetailFocus::Providers {
                if let Some(provider_id) = app
                    .codex_detail_provider_ids
                    .get(app.codex_detail_provider_index)
                {
                    app.modal = Some(app::Modal::Confirm {
                        message: format!("Delete provider '{provider_id}'?"),
                        action: app::ConfirmAction::CodexDeleteProvider {
                            profile_id,
                            provider_id: provider_id.clone(),
                        },
                    });
                }
            }
        }
        KeyCode::Enter | KeyCode::Char('e') => match app.codex_detail_focus {
            app::CodexDetailFocus::Fields => match app.codex_detail_field_index {
                0 => {
                    if is_official {
                        app.set_toast("Official profile is read-only", true);
                        return None;
                    }
                    app.modal = Some(app::Modal::Input {
                        title: "Profile name".to_string(),
                        value: profile.name.clone(),
                        cursor: usize::MAX,
                        is_secret: false,
                        action: app::InputAction::CodexSetProfileName { id: profile_id },
                    });
                }
                1 => {
                    if is_official {
                        app.set_toast("Official profile is read-only", true);
                        return None;
                    }
                    app.modal = Some(app::Modal::Input {
                        title: "Profile description".to_string(),
                        value: profile.description.clone().unwrap_or_default(),
                        cursor: usize::MAX,
                        is_secret: false,
                        action: app::InputAction::CodexSetProfileDescription { id: profile_id },
                    });
                }
                2 => {
                    if is_official {
                        app.set_toast("Official profile is read-only", true);
                        return None;
                    }
                    if app.codex_detail_provider_ids.is_empty() {
                        app.set_toast("No providers configured", true);
                        return None;
                    }
                    let options = app.codex_detail_provider_ids.clone();
                    let index = options
                        .iter()
                        .position(|p| p == &profile.model_provider)
                        .unwrap_or(0);
                    app.modal = Some(app::Modal::Select {
                        title: "Model provider".to_string(),
                        options,
                        index,
                        action: app::SelectAction::CodexSetProfileModelProvider { id: profile_id },
                    });
                }
                3 => {
                    if is_official {
                        app.set_toast("Official profile is read-only", true);
                        return None;
                    }
                    app.modal = Some(app::Modal::Input {
                        title: "Model".to_string(),
                        value: profile.model.clone(),
                        cursor: usize::MAX,
                        is_secret: false,
                        action: app::InputAction::CodexSetProfileModel { id: profile_id },
                    });
                }
                4 => {
                    if is_official {
                        app.set_toast("Official profile is read-only", true);
                        return None;
                    }
                    let options = vec![
                        "(none)".to_string(),
                        "xhigh".to_string(),
                        "high".to_string(),
                        "medium".to_string(),
                        "low".to_string(),
                        "minimal".to_string(),
                    ];
                    let index = profile
                        .model_reasoning_effort
                        .as_deref()
                        .and_then(|v| options.iter().position(|o| o == v))
                        .unwrap_or(0);
                    app.modal = Some(app::Modal::Select {
                        title: "Reasoning effort".to_string(),
                        options,
                        index,
                        action: app::SelectAction::CodexSetProfileReasoningEffort {
                            id: profile_id,
                        },
                    });
                }
                5 => {
                    if is_official {
                        app.set_toast("Official profile is read-only", true);
                        return None;
                    }
                    app.modal = Some(app::Modal::Input {
                        title: "API key".to_string(),
                        value: profile.api_key.clone().unwrap_or_default(),
                        cursor: usize::MAX,
                        is_secret: true,
                        action: app::InputAction::CodexSetProfileApiKey { id: profile_id },
                    });
                }
                _ => {}
            },
            app::CodexDetailFocus::Providers => {
                if let Some(provider_id) = app
                    .codex_detail_provider_ids
                    .get(app.codex_detail_provider_index)
                {
                    app.codex_provider_id = Some(provider_id.clone());
                    app.codex_provider_field_index = 0;
                    app.screen = app::Screen::CodexProvider;
                }
            }
        },
        _ => {}
    }

    None
}

pub(super) fn handle_codex_provider_key(app: &mut app::App, code: KeyCode) -> Option<Action> {
    let Some(profile_id) = app.codex_detail_id.clone() else {
        app.screen = app::Screen::Codex;
        return None;
    };
    let Some(provider_id) = app.codex_provider_id.clone() else {
        app.screen = app::Screen::CodexProfile;
        return None;
    };
    let Some(profile) = app.codex_detail.as_ref() else {
        return None;
    };
    let Some(config) = profile.providers.get(&provider_id) else {
        app.set_toast("Provider not found", true);
        app.screen = app::Screen::CodexProfile;
        return None;
    };
    let is_official = profile_id == "official";

    match code {
        KeyCode::Esc | KeyCode::Char('q') => {
            app.screen = app::Screen::CodexProfile;
        }
        KeyCode::Down => {
            app.codex_provider_field_index = app.codex_provider_field_index.saturating_add(1)
        }
        KeyCode::Up => {
            app.codex_provider_field_index = app.codex_provider_field_index.saturating_sub(1)
        }
        KeyCode::Enter | KeyCode::Char('e') => match app.codex_provider_field_index {
            0 => {
                if is_official {
                    app.set_toast("Official profile is read-only", true);
                    return None;
                }
                app.modal = Some(app::Modal::Input {
                    title: "Provider name".to_string(),
                    value: config.name.clone().unwrap_or_default(),
                    cursor: usize::MAX,
                    is_secret: false,
                    action: app::InputAction::CodexSetProviderName {
                        profile_id,
                        provider_id,
                    },
                });
            }
            1 => {
                if is_official {
                    app.set_toast("Official profile is read-only", true);
                    return None;
                }
                app.modal = Some(app::Modal::Input {
                    title: "Provider base URL".to_string(),
                    value: config.base_url.clone().unwrap_or_default(),
                    cursor: usize::MAX,
                    is_secret: false,
                    action: app::InputAction::CodexSetProviderBaseUrl {
                        profile_id,
                        provider_id,
                    },
                });
            }
            2 => {
                if is_official {
                    app.set_toast("Official profile is read-only", true);
                    return None;
                }
                let options = vec!["responses".to_string(), "chat".to_string()];
                let index = config
                    .wire_api
                    .as_deref()
                    .and_then(|v| options.iter().position(|o| o == v))
                    .unwrap_or(0);
                app.modal = Some(app::Modal::Select {
                    title: "Wire API".to_string(),
                    options,
                    index,
                    action: app::SelectAction::CodexSetProviderWireApi {
                        profile_id,
                        provider_id,
                    },
                });
            }
            3 => {
                if is_official {
                    app.set_toast("Official profile is read-only", true);
                    return None;
                }
                app.modal = Some(app::Modal::Input {
                    title: "Provider model".to_string(),
                    value: config.model.clone().unwrap_or_default(),
                    cursor: usize::MAX,
                    is_secret: false,
                    action: app::InputAction::CodexSetProviderModel {
                        profile_id,
                        provider_id,
                    },
                });
            }
            4 => {
                if is_official {
                    app.set_toast("Official profile is read-only", true);
                    return None;
                }
                let options = vec![
                    "(none)".to_string(),
                    "xhigh".to_string(),
                    "high".to_string(),
                    "medium".to_string(),
                    "low".to_string(),
                    "minimal".to_string(),
                ];
                let index = config
                    .model_reasoning_effort
                    .as_deref()
                    .and_then(|v| options.iter().position(|o| o == v))
                    .unwrap_or(0);
                app.modal = Some(app::Modal::Select {
                    title: "Reasoning effort".to_string(),
                    options,
                    index,
                    action: app::SelectAction::CodexSetProviderReasoningEffort {
                        profile_id,
                        provider_id,
                    },
                });
            }
            5 => {
                if is_official {
                    app.set_toast("Official profile is read-only", true);
                    return None;
                }
                app.modal = Some(app::Modal::Input {
                    title: "Provider API key".to_string(),
                    value: config.api_key.clone().unwrap_or_default(),
                    cursor: usize::MAX,
                    is_secret: true,
                    action: app::InputAction::CodexSetProviderApiKey {
                        profile_id,
                        provider_id,
                    },
                });
            }
            _ => {}
        },
        _ => {}
    }

    None
}
