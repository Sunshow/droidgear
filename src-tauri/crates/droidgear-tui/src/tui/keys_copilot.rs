use super::*;

pub(super) fn handle_copilot_key(app: &mut app::App, code: KeyCode) -> Option<Action> {
    match code {
        KeyCode::Esc | KeyCode::Char('q') => app.go_back(),
        KeyCode::Down => app.copilot_index = app.copilot_index.saturating_add(1),
        KeyCode::Up => app.copilot_index = app.copilot_index.saturating_sub(1),
        KeyCode::Char('r') => refresh_copilot(app),
        KeyCode::Char('i') => {
            if let Some(profile) = app.copilot_profiles.get(app.copilot_index) {
                if let Err(error) = super::copilot_import::start(app, profile.id.clone()) {
                    app.set_toast(error.to_string(), true);
                }
            }
        }
        KeyCode::Char('n') => {
            app.modal = Some(app::Modal::Input {
                title: "New Copilot profile name".to_string(),
                value: String::new(),
                cursor: usize::MAX,
                is_secret: false,
                action: app::InputAction::CopilotCreateProfile,
            });
        }
        KeyCode::Char('c') => {
            if let Some(profile) = app.copilot_profiles.get(app.copilot_index) {
                app.modal = Some(app::Modal::Input {
                    title: "Duplicate profile name".to_string(),
                    value: format!("{} (copy)", profile.name),
                    cursor: usize::MAX,
                    is_secret: false,
                    action: app::InputAction::CopilotDuplicate {
                        id: profile.id.clone(),
                    },
                });
            }
        }
        KeyCode::Char('d') => {
            if let Some(profile) = app.copilot_profiles.get(app.copilot_index) {
                app.modal = Some(app::Modal::Confirm {
                    message: format!("Delete Copilot profile '{}'?", profile.name),
                    action: app::ConfirmAction::CopilotDelete {
                        id: profile.id.clone(),
                    },
                });
            }
        }
        KeyCode::Enter | KeyCode::Char('t') => {
            if let Some(profile) = app.copilot_profiles.get(app.copilot_index) {
                return Some(Action::RunCopilotRun {
                    id: profile.id.clone(),
                });
            }
        }
        KeyCode::Char('e') => {
            if let Some(profile) = app.copilot_profiles.get(app.copilot_index) {
                return Some(Action::EditCopilotProfile {
                    id: profile.id.clone(),
                });
            }
        }
        KeyCode::Char('a') => {
            if let Some(profile) = app.copilot_profiles.get(app.copilot_index) {
                app.modal = Some(app::Modal::Confirm {
                    message: format!("Apply Copilot profile '{}' to config.env?", profile.name),
                    action: app::ConfirmAction::CopilotApply {
                        id: profile.id.clone(),
                    },
                });
            }
        }
        KeyCode::Char('l') => {
            if !droidgear_core::copilot::get_copilot_config_status_for_home(&app.home_dir)
                .is_ok_and(|status| status.config_exists)
            {
                app.set_toast("No applied Copilot config.env to load", true);
                return None;
            }
            if let Some(profile) = app.copilot_profiles.get(app.copilot_index) {
                match droidgear_core::copilot::read_copilot_current_config_for_home(&app.home_dir) {
                    Ok(live) => {
                        let mut updated = profile.clone();
                        updated.use_official_auth = !live.is_byok;
                        updated.base_url = live.base_url;
                        updated.provider_type = live.provider_type;
                        updated.api_key = live.api_key;
                        updated.model = live.model;
                        updated.max_prompt_tokens = live.max_prompt_tokens;
                        updated.max_output_tokens = live.max_output_tokens;
                        match droidgear_core::copilot::save_copilot_profile_for_home(
                            &app.home_dir,
                            updated,
                        ) {
                            Ok(()) => {
                                app.set_toast("Loaded current config", false);
                                refresh_copilot(app);
                            }
                            Err(error) => app.set_toast(error, true),
                        }
                    }
                    Err(error) => app.set_toast(error, true),
                }
            }
        }
        _ => {}
    }
    app.clamp_indices();
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tui::modal::{run_confirm_action, run_input_action};
    use tempfile::TempDir;

    #[test]
    fn copilot_profiles_can_be_created_copied_applied_and_deleted() {
        let temp = TempDir::new().unwrap();
        let mut app = app::App::new(temp.path().to_path_buf());
        app.screen = app::Screen::Copilot;
        handle_copilot_key(&mut app, KeyCode::Char('n'));
        let Some(app::Modal::Input { action, .. }) = app.modal.take() else {
            panic!("expected profile name input");
        };
        run_input_action(&mut app, action, "First".to_string()).unwrap();
        let id = app.copilot_profiles[app.copilot_index].id.clone();
        let mut profile =
            droidgear_core::copilot::get_copilot_profile_for_home(temp.path(), &id).unwrap();
        profile.use_official_auth = true;
        droidgear_core::copilot::save_copilot_profile_for_home(temp.path(), profile).unwrap();
        refresh_copilot(&mut app);

        handle_copilot_key(&mut app, KeyCode::Char('c'));
        let Some(app::Modal::Input { action, .. }) = app.modal.take() else {
            panic!("expected duplicate name input");
        };
        run_input_action(&mut app, action, "Second".to_string()).unwrap();
        assert_eq!(app.copilot_profiles.len(), 2);
        let copy_id = app.copilot_profiles[app.copilot_index].id.clone();
        assert_ne!(copy_id, id);
        handle_copilot_key(&mut app, KeyCode::Char('a'));
        let Some(app::Modal::Confirm { action, .. }) = app.modal.take() else {
            panic!("expected apply confirmation");
        };
        run_confirm_action(&mut app, action).unwrap();
        refresh_copilot(&mut app);
        assert_eq!(app.copilot_active_id.as_deref(), Some(copy_id.as_str()));
        let listing = crate::tui::utils::list_copilot_temporary_run_targets(temp.path()).unwrap();
        assert!(listing.contains("* 2. Second (official)"));
        assert!(!listing.contains('$'));
        assert!(
            matches!(handle_copilot_key(&mut app, KeyCode::Enter), Some(Action::RunCopilotRun { id }) if id == copy_id)
        );

        handle_copilot_key(&mut app, KeyCode::Char('d'));
        let Some(app::Modal::Confirm { action, .. }) = app.modal.take() else {
            panic!("expected delete confirmation");
        };
        run_confirm_action(&mut app, action).unwrap();
        refresh_copilot(&mut app);
        app.clamp_indices();
        assert_eq!(app.copilot_profiles.len(), 1);
        assert!(app.copilot_active_id.is_none());
        assert_eq!(app.copilot_index, 0);
    }

    #[test]
    fn loading_a_missing_config_does_not_overwrite_the_profile() {
        let temp = TempDir::new().unwrap();
        let mut app = app::App::new(temp.path().to_path_buf());
        refresh_copilot(&mut app);
        let id = app.copilot_profiles[0].id.clone();
        handle_copilot_key(&mut app, KeyCode::Char('l'));
        assert!(app.toast_message().contains("No applied"));
        assert!(
            !droidgear_core::copilot::get_copilot_profile_for_home(temp.path(), &id)
                .unwrap()
                .use_official_auth
        );
    }
}
