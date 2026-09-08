use super::*;

pub(super) fn handle_codex_sessions_key(app: &mut app::App, code: KeyCode) -> Option<Action> {
    match code {
        KeyCode::Esc | KeyCode::Char('q') => app.go_back(),
        KeyCode::Down => app.codex_sessions_index = app.codex_sessions_index.saturating_add(1),
        KeyCode::Up => app.codex_sessions_index = app.codex_sessions_index.saturating_sub(1),
        KeyCode::Char('r') => refresh_codex_sessions(app),
        KeyCode::Enter | KeyCode::Char('v') => {
            if let Some(s) = app.codex_sessions.get(app.codex_sessions_index) {
                return Some(Action::ViewCodexSession {
                    path: s.path.clone(),
                });
            }
        }
        KeyCode::Char('d') => {
            if let Some(s) = app.codex_sessions.get(app.codex_sessions_index) {
                app.modal = Some(app::Modal::Confirm {
                    message: format!("Delete session '{}'?", s.title),
                    action: app::ConfirmAction::CodexSessionDelete {
                        path: s.path.clone(),
                    },
                });
            }
        }
        KeyCode::Char('p') => {
            if let Some(s) = app.codex_sessions.get(app.codex_sessions_index) {
                let providers =
                    match droidgear_core::codex_sessions::list_codex_session_providers_for_home(
                        &app.home_dir,
                    ) {
                        Ok(providers) => providers,
                        Err(e) => {
                            app.set_toast(e, true);
                            return None;
                        }
                    };
                if providers.is_empty() {
                    app.set_toast("No Codex profiles configured", true);
                    return None;
                }
                let options: Vec<String> = providers.iter().map(|p| p.id.clone()).collect();
                let index = options
                    .iter()
                    .position(|id| id == &s.model_provider)
                    .unwrap_or(0);
                app.modal = Some(app::Modal::Select {
                    title: "Switch session provider".to_string(),
                    options,
                    index,
                    action: app::SelectAction::CodexSessionSetProvider {
                        path: s.path.clone(),
                    },
                });
            }
        }
        _ => {}
    }
    None
}
