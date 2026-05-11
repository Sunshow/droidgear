use super::*;

pub(super) fn handle_claude_key(app: &mut app::App, code: KeyCode) -> Option<Action> {
    match code {
        KeyCode::Esc | KeyCode::Char('q') => app.screen = app::Screen::Main,
        KeyCode::Down => app.claude_index = app.claude_index.saturating_add(1),
        KeyCode::Up => app.claude_index = app.claude_index.saturating_sub(1),
        KeyCode::Char('r') => refresh_claude(app),
        KeyCode::Char('n') => {
            app.modal = Some(app::Modal::Input {
                title: "New Claude profile name".to_string(),
                value: String::new(),
                cursor: usize::MAX,
                is_secret: false,
                action: app::InputAction::ClaudeCreateProfile,
            });
        }
        KeyCode::Enter | KeyCode::Char('e') => {
            if let Some(profile) = app.claude_profiles.get(app.claude_index) {
                app.claude_detail_id = Some(profile.id.clone());
                app.claude_detail_field_index = 0;
                app.screen = app::Screen::ClaudeProfile;
                refresh_claude_detail(app);
            }
        }
        KeyCode::Char('a') => {
            if let Some(profile) = app.claude_profiles.get(app.claude_index) {
                app.modal = Some(app::Modal::Confirm {
                    message: format!("Apply Claude profile '{}'?", profile.name),
                    action: app::ConfirmAction::ClaudeApply {
                        id: profile.id.clone(),
                    },
                });
            }
        }
        KeyCode::Char('d') => {
            if let Some(profile) = app.claude_profiles.get(app.claude_index) {
                app.modal = Some(app::Modal::Confirm {
                    message: format!("Delete Claude profile '{}'?", profile.name),
                    action: app::ConfirmAction::ClaudeDelete {
                        id: profile.id.clone(),
                    },
                });
            }
        }
        KeyCode::Char('c') => {
            if let Some(profile) = app.claude_profiles.get(app.claude_index) {
                app.modal = Some(app::Modal::Input {
                    title: "Duplicate profile name".to_string(),
                    value: format!("{} (copy)", profile.name),
                    cursor: usize::MAX,
                    is_secret: false,
                    action: app::InputAction::ClaudeDuplicate {
                        id: profile.id.clone(),
                    },
                });
            }
        }
        KeyCode::Char('t') => {
            if let Some(profile) = app.claude_profiles.get(app.claude_index) {
                return Some(Action::PreviewClaudeRun {
                    id: profile.id.clone(),
                });
            }
        }
        KeyCode::Char('x') => {
            if let Some(profile) = app.claude_profiles.get(app.claude_index) {
                return Some(Action::RunClaudeRun {
                    id: profile.id.clone(),
                });
            }
        }
        _ => {}
    }
    None
}

pub(super) fn handle_claude_profile_key(app: &mut app::App, code: KeyCode) -> Option<Action> {
    let Some(profile_id) = app.claude_detail_id.clone() else {
        app.screen = app::Screen::Claude;
        return None;
    };
    let Some(profile) = app.claude_detail.as_ref() else {
        return None;
    };

    match code {
        KeyCode::Esc | KeyCode::Char('q') => app.screen = app::Screen::Claude,
        KeyCode::Down => {
            app.claude_detail_field_index = app.claude_detail_field_index.saturating_add(1)
        }
        KeyCode::Up => {
            app.claude_detail_field_index = app.claude_detail_field_index.saturating_sub(1)
        }
        KeyCode::Char('r') => refresh_claude_detail(app),
        KeyCode::Char('a') => {
            app.modal = Some(app::Modal::Confirm {
                message: format!("Apply Claude profile '{}'?", profile.name),
                action: app::ConfirmAction::ClaudeApply {
                    id: profile_id.clone(),
                },
            });
        }
        KeyCode::Char('l') => {
            if let Err(e) = claude_load_from_live_config(app, &profile_id) {
                app.set_toast(e.to_string(), true);
            } else {
                app.set_toast("Loaded from live config", false);
                refresh_claude_detail(app);
            }
        }
        KeyCode::Char('t') => return Some(Action::PreviewClaudeRun { id: profile_id }),
        KeyCode::Char('x') => return Some(Action::RunClaudeRun { id: profile_id }),
        KeyCode::Enter | KeyCode::Char('e') => match app.claude_detail_field_index {
            0 => {
                app.modal = Some(app::Modal::Input {
                    title: "Profile name".to_string(),
                    value: profile.name.clone(),
                    cursor: usize::MAX,
                    is_secret: false,
                    action: app::InputAction::ClaudeSetProfileName {
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
                    action: app::InputAction::ClaudeSetProfileDescription {
                        id: profile_id.clone(),
                    },
                });
            }
            2 => {
                app.modal = Some(app::Modal::Input {
                    title: "Base URL".to_string(),
                    value: profile.base_url.clone().unwrap_or_default(),
                    cursor: usize::MAX,
                    is_secret: false,
                    action: app::InputAction::ClaudeSetProfileBaseUrl {
                        id: profile_id.clone(),
                    },
                });
            }
            3 => {
                app.modal = Some(app::Modal::Input {
                    title: "Bearer token".to_string(),
                    value: profile.bearer_token.clone().unwrap_or_default(),
                    cursor: usize::MAX,
                    is_secret: true,
                    action: app::InputAction::ClaudeSetProfileBearerToken {
                        id: profile_id.clone(),
                    },
                });
            }
            4 => {
                app.modal = Some(app::Modal::Input {
                    title: "Main model".to_string(),
                    value: profile.model.clone().unwrap_or_default(),
                    cursor: usize::MAX,
                    is_secret: false,
                    action: app::InputAction::ClaudeSetProfileModel {
                        id: profile_id.clone(),
                    },
                });
            }
            5 => {
                let mut next = profile.clone();
                next.small_model_uses_main_model = !next.small_model_uses_main_model;
                if let Err(e) =
                    droidgear_core::claude::save_claude_profile_for_home(&app.home_dir, next)
                {
                    app.set_toast(e, true);
                } else {
                    app.set_toast("Saved", false);
                    refresh_claude_detail(app);
                }
            }
            6 => {
                if profile.small_model_uses_main_model {
                    app.set_toast("Disable 'Use main model' first", true);
                    return None;
                }
                app.modal = Some(app::Modal::Input {
                    title: "Small model".to_string(),
                    value: profile.small_model.clone().unwrap_or_default(),
                    cursor: usize::MAX,
                    is_secret: false,
                    action: app::InputAction::ClaudeSetProfileSmallModel {
                        id: profile_id.clone(),
                    },
                });
            }
            7 => {
                let options = vec![
                    "(inherit)".to_string(),
                    "low".to_string(),
                    "medium".to_string(),
                    "high".to_string(),
                    "max".to_string(),
                ];
                let current = profile
                    .reasoning_effort
                    .map(|value| match value {
                        droidgear_core::claude::ClaudeReasoningEffort::Low => "low".to_string(),
                        droidgear_core::claude::ClaudeReasoningEffort::Medium => {
                            "medium".to_string()
                        }
                        droidgear_core::claude::ClaudeReasoningEffort::High => "high".to_string(),
                        droidgear_core::claude::ClaudeReasoningEffort::Max => "max".to_string(),
                    })
                    .unwrap_or_else(|| "(inherit)".to_string());
                let index = options
                    .iter()
                    .position(|option| option == &current)
                    .unwrap_or(0);
                app.modal = Some(app::Modal::Select {
                    title: "Reasoning effort".to_string(),
                    options,
                    index,
                    action: app::SelectAction::ClaudeSetProfileReasoningEffort {
                        id: profile_id.clone(),
                    },
                });
            }
            8 => {
                let options = vec!["inherit".to_string(), "on".to_string(), "off".to_string()];
                let current = match profile.thinking_mode {
                    droidgear_core::claude::ClaudeThinkingMode::Inherit => "inherit",
                    droidgear_core::claude::ClaudeThinkingMode::On => "on",
                    droidgear_core::claude::ClaudeThinkingMode::Off => "off",
                };
                let index = options
                    .iter()
                    .position(|option| option == current)
                    .unwrap_or(0);
                app.modal = Some(app::Modal::Select {
                    title: "Thinking mode".to_string(),
                    options,
                    index,
                    action: app::SelectAction::ClaudeSetProfileThinkingMode { id: profile_id },
                });
            }
            _ => {}
        },
        _ => {}
    }

    None
}
