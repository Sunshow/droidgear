use super::*;

pub(super) fn handle_pi_sessions_key(app: &mut app::App, code: KeyCode) -> Option<Action> {
    match code {
        KeyCode::Esc | KeyCode::Char('q') => app.go_back(),
        KeyCode::Down => app.pi_sessions_index = app.pi_sessions_index.saturating_add(1),
        KeyCode::Up => app.pi_sessions_index = app.pi_sessions_index.saturating_sub(1),
        KeyCode::Char('r') => refresh_pi_sessions(app),
        KeyCode::Enter | KeyCode::Char('v') => {
            if let Some(session) = app.pi_sessions.get(app.pi_sessions_index) {
                return Some(Action::ViewPiSession {
                    path: session.path.clone(),
                });
            }
        }
        KeyCode::Char('d') => {
            if let Some(session) = app.pi_sessions.get(app.pi_sessions_index) {
                app.modal = Some(app::Modal::Confirm {
                    message: format!("Delete Pi session '{}'?", session.title),
                    action: app::ConfirmAction::PiSessionDelete {
                        path: session.path.clone(),
                    },
                });
            }
        }
        _ => {}
    }
    None
}
