use super::*;

pub(super) fn handle_droid_settings_files_key(app: &mut app::App, code: KeyCode) -> Option<Action> {
    match code {
        KeyCode::Esc | KeyCode::Char('q') => app.screen = app::Screen::Main,
        KeyCode::Down => {
            app.droid_settings_files_index = app.droid_settings_files_index.saturating_add(1)
        }
        KeyCode::Up => {
            app.droid_settings_files_index = app.droid_settings_files_index.saturating_sub(1)
        }
        KeyCode::Char('r') => refresh_droid_settings_files(app),
        KeyCode::Enter => {
            if let Some(file) = app.droid_settings_files.get(app.droid_settings_files_index) {
                let name = if file.is_global {
                    None
                } else {
                    Some(file.name.clone())
                };
                return Some(Action::SetActiveSettingsFile { name });
            }
        }
        _ => {}
    }
    None
}
