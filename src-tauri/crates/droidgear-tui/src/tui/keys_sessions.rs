use super::*;

pub(super) fn handle_sessions_key(app: &mut app::App, code: KeyCode) -> Option<Action> {
    match code {
        KeyCode::Esc | KeyCode::Char('q') => app.screen = app::Screen::Main,
        KeyCode::Down => app.sessions_index = app.sessions_index.saturating_add(1),
        KeyCode::Up => app.sessions_index = app.sessions_index.saturating_sub(1),
        KeyCode::Char('r') => refresh_sessions(app),
        KeyCode::Enter | KeyCode::Char('v') => {
            if let Some(s) = app.sessions.get(app.sessions_index) {
                return Some(Action::ViewSession {
                    path: s.path.clone(),
                });
            }
        }
        KeyCode::Char('d') => {
            if let Some(s) = app.sessions.get(app.sessions_index) {
                app.modal = Some(app::Modal::Confirm {
                    message: format!("Delete session '{}'?", s.title),
                    action: app::ConfirmAction::SessionDelete {
                        path: s.path.clone(),
                    },
                });
            }
        }
        _ => {}
    }
    None
}
