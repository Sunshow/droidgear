use super::*;

pub(super) fn handle_dsh_sessions_key(app: &mut app::App, code: KeyCode) -> Option<Action> {
    match code {
        KeyCode::Esc | KeyCode::Char('q') => app.go_back(),
        KeyCode::Down => app.dsh_sessions_index = app.dsh_sessions_index.saturating_add(1),
        KeyCode::Up => app.dsh_sessions_index = app.dsh_sessions_index.saturating_sub(1),
        KeyCode::Char('r') => refresh_dsh_sessions(app),
        KeyCode::Enter | KeyCode::Char('v') => {
            if let Some(session) = app.dsh_sessions.get(app.dsh_sessions_index) {
                return Some(Action::ViewDshSession {
                    path: session.path.clone(),
                });
            }
        }
        KeyCode::Char('d') => {
            if let Some(session) = app.dsh_sessions.get(app.dsh_sessions_index) {
                app.modal = Some(app::Modal::Confirm {
                    message: format!("Delete Dsh session '{}'?", session.title),
                    action: app::ConfirmAction::DshSessionDelete {
                        path: session.path.clone(),
                    },
                });
            }
        }
        _ => {}
    }
    None
}
