use super::*;

pub(super) fn refresh_factory_auth(app: &mut app::App) {
    match droidgear_core::factory_auth_profiles::list_profiles_for_home(&app.home_dir) {
        Ok(state) => {
            app.factory_auth_profiles = state.profiles;
            app.factory_auth_active = state.active;
        }
        Err(e) => {
            app.set_toast(format!("Failed to load auth profiles: {e}"), true);
        }
    }
}

pub(super) fn handle_factory_auth_key(app: &mut app::App, code: KeyCode) -> Option<Action> {
    match code {
        KeyCode::Esc | KeyCode::Char('q') => {
            app.screen = app::Screen::Main;
        }
        KeyCode::Up | KeyCode::Char('k') => {
            app.factory_auth_index = app.factory_auth_index.saturating_sub(1);
        }
        KeyCode::Down | KeyCode::Char('j') => {
            if !app.factory_auth_profiles.is_empty() {
                app.factory_auth_index = (app.factory_auth_index + 1)
                    .min(app.factory_auth_profiles.len().saturating_sub(1));
            }
        }
        KeyCode::Enter => {
            if let Some(profile) = app.factory_auth_profiles.get(app.factory_auth_index) {
                let name = profile.name.clone();
                if app.factory_auth_active.as_deref() != Some(&name) {
                    app.modal = Some(app::Modal::Confirm {
                        message: format!("Switch to profile '{}'?", profile.label),
                        action: app::ConfirmAction::FactoryAuthSwitch { name },
                    });
                }
            }
        }
        KeyCode::Char('s') => {
            app.modal = Some(app::Modal::Input {
                title: "Save current auth as profile (ID)".to_string(),
                value: String::new(),
                cursor: 0,
                is_secret: false,
                action: app::InputAction::FactoryAuthSaveProfile,
            });
        }
        KeyCode::Char('r') => {
            if let Some(profile) = app.factory_auth_profiles.get(app.factory_auth_index) {
                let name = profile.name.clone();
                app.modal = Some(app::Modal::Input {
                    title: format!("Rename '{}' — new label", profile.label),
                    value: profile.label.clone(),
                    cursor: profile.label.chars().count(),
                    is_secret: false,
                    action: app::InputAction::FactoryAuthRename { name },
                });
            }
        }
        KeyCode::Char('d') | KeyCode::Delete => {
            if let Some(profile) = app.factory_auth_profiles.get(app.factory_auth_index) {
                if app.factory_auth_active.as_deref() != Some(&profile.name) {
                    app.modal = Some(app::Modal::Confirm {
                        message: format!("Delete profile '{}'?", profile.label),
                        action: app::ConfirmAction::FactoryAuthDelete {
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
