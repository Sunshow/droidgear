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
