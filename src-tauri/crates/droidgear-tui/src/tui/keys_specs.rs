use super::*;

pub(super) fn handle_specs_key(app: &mut app::App, code: KeyCode) -> Option<Action> {
    match code {
        KeyCode::Esc | KeyCode::Char('q') => app.screen = app::Screen::Main,
        KeyCode::Down => app.specs_index = app.specs_index.saturating_add(1),
        KeyCode::Up => app.specs_index = app.specs_index.saturating_sub(1),
        KeyCode::Char('r') => refresh_specs(app),
        KeyCode::Enter | KeyCode::Char('e') => {
            if let Some(s) = app.specs.get(app.specs_index) {
                return Some(Action::EditSpec {
                    path: s.path.clone(),
                });
            }
        }
        KeyCode::Char('d') => {
            if let Some(s) = app.specs.get(app.specs_index) {
                app.modal = Some(app::Modal::Confirm {
                    message: format!("Delete spec '{}'?", s.name),
                    action: app::ConfirmAction::SpecDelete {
                        path: s.path.clone(),
                    },
                });
            }
        }
        _ => {}
    }
    None
}
