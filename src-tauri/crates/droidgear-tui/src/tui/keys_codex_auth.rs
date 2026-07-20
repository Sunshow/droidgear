use super::*;

pub(super) fn refresh_codex_auth(app: &mut app::App) {
    match droidgear_core::codex_auth_profiles::list_profiles_for_home(&app.home_dir) {
        Ok(state) => {
            app.codex_auth_profiles = state.profiles;
            app.codex_auth_active = state.active;
            app.codex_auth_is_current_official = state.is_current_official;
        }
        Err(e) => {
            app.set_toast(format!("Failed to load auth profiles: {e}"), true);
        }
    }
}

pub(super) fn handle_codex_auth_key(app: &mut app::App, code: KeyCode) -> Option<Action> {
    match code {
        KeyCode::Esc | KeyCode::Char('q') => {
            app.screen = app::Screen::Main;
        }
        KeyCode::Up | KeyCode::Char('k') => {
            app.codex_auth_index = app.codex_auth_index.saturating_sub(1);
        }
        KeyCode::Down | KeyCode::Char('j') if !app.codex_auth_profiles.is_empty() => {
            app.codex_auth_index =
                (app.codex_auth_index + 1).min(app.codex_auth_profiles.len().saturating_sub(1));
        }
        KeyCode::Enter => {
            if let Some(profile) = app.codex_auth_profiles.get(app.codex_auth_index) {
                let name = profile.name.clone();
                if app.codex_auth_active.as_deref() != Some(&name) {
                    // Check for conflict
                    match droidgear_core::codex_auth_profiles::detect_auth_conflict_for_home(
                        &app.home_dir,
                        &name,
                    ) {
                        Ok(conflict) if conflict.has_conflict => {
                            app.modal = Some(app::Modal::Confirm {
                                message: format!(
                                    "Auth mode conflict! Save current auth before switching to '{}'?",
                                    profile.label
                                ),
                                action: app::ConfirmAction::CodexAuthConflictSwitch { name },
                            });
                        }
                        _ => {
                            app.modal = Some(app::Modal::Confirm {
                                message: format!("Switch to auth profile '{}'?", profile.label),
                                action: app::ConfirmAction::CodexAuthSwitch { name },
                            });
                        }
                    }
                }
            }
        }
        KeyCode::Char('s') => {
            if !app.codex_auth_is_current_official {
                app.set_toast(
                    "Save Current only available for official login configs".to_string(),
                    true,
                );
                return None;
            }
            app.modal = Some(app::Modal::Input {
                title: "Save current auth as profile (ID)".to_string(),
                value: String::new(),
                cursor: 0,
                is_secret: false,
                action: app::InputAction::CodexAuthSaveProfile,
            });
        }
        KeyCode::Char('r') => {
            if let Some(profile) = app.codex_auth_profiles.get(app.codex_auth_index) {
                let name = profile.name.clone();
                app.modal = Some(app::Modal::Input {
                    title: format!("Rename '{}' — new label", profile.label),
                    value: profile.label.clone(),
                    cursor: profile.label.chars().count(),
                    is_secret: false,
                    action: app::InputAction::CodexAuthRename { name },
                });
            }
        }
        KeyCode::Char('w') => {
            if !app.codex_auth_is_current_official {
                app.set_toast(
                    "Overwrite only available for official login configs".to_string(),
                    true,
                );
                return None;
            }
            if let Some(profile) = app.codex_auth_profiles.get(app.codex_auth_index) {
                app.modal = Some(app::Modal::Confirm {
                    message: format!(
                        "Overwrite auth profile '{}' with current login (auth + model)?",
                        profile.label
                    ),
                    action: app::ConfirmAction::CodexAuthOverwrite {
                        name: profile.name.clone(),
                        label: profile.label.clone(),
                    },
                });
            }
        }
        KeyCode::Char('d') | KeyCode::Delete => {
            if let Some(profile) = app.codex_auth_profiles.get(app.codex_auth_index) {
                if app.codex_auth_active.as_deref() != Some(&profile.name) {
                    app.modal = Some(app::Modal::Confirm {
                        message: format!("Delete auth profile '{}'?", profile.label),
                        action: app::ConfirmAction::CodexAuthDelete {
                            name: profile.name.clone(),
                        },
                    });
                } else {
                    app.set_toast("Cannot delete the active profile".to_string(), true);
                }
            }
        }
        _ => {}
    }
    None
}
