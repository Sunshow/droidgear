use super::*;

pub(super) fn handle_paths_key(app: &mut app::App, code: KeyCode) -> Option<Action> {
    match code {
        KeyCode::Esc | KeyCode::Char('q') => app.screen = app::Screen::Main,
        KeyCode::Down => app.paths_index = app.paths_index.saturating_add(1),
        KeyCode::Up => app.paths_index = app.paths_index.saturating_sub(1),
        KeyCode::Char('r') => refresh_paths(app),
        KeyCode::Char('e') | KeyCode::Enter => {
            let Some(key) = app.current_paths_key() else {
                return None;
            };
            let current = app
                .current_paths_entry()
                .map(|p| p.path.clone())
                .unwrap_or_default();
            app.modal = Some(app::Modal::Input {
                title: format!("Set path for {key}"),
                value: current,
                cursor: usize::MAX,
                is_secret: false,
                action: app::InputAction::PathsSetKey { key },
            });
        }
        KeyCode::Char('x') => {
            let Some(key) = app.current_paths_key() else {
                return None;
            };
            app.modal = Some(app::Modal::Confirm {
                message: format!("Reset path override for {key}?"),
                action: app::ConfirmAction::PathsResetKey { key },
            });
        }
        _ => {}
    }
    None
}
