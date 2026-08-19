use super::*;

pub(super) fn handle_trusted_folders_key(app: &mut app::App, code: KeyCode) -> Option<Action> {
    match code {
        KeyCode::Esc | KeyCode::Char('q') => app.go_back(),
        KeyCode::Down => app.trusted_folders_index = app.trusted_folders_index.saturating_add(1),
        KeyCode::Up => app.trusted_folders_index = app.trusted_folders_index.saturating_sub(1),
        KeyCode::Char('r') => refresh_trusted_folders(app),
        KeyCode::Char(' ') => {
            if let Some(folder) = app.trusted_folders.get(app.trusted_folders_index) {
                if !app.trusted_folders_selected.insert(folder.path.clone()) {
                    app.trusted_folders_selected.remove(&folder.path);
                }
            }
        }
        KeyCode::Char('A') => {
            let all_selected = !app.trusted_folders.is_empty()
                && app
                    .trusted_folders
                    .iter()
                    .all(|folder| app.trusted_folders_selected.contains(&folder.path));
            if all_selected {
                app.trusted_folders_selected.clear();
            } else {
                app.trusted_folders_selected = app
                    .trusted_folders
                    .iter()
                    .map(|folder| folder.path.clone())
                    .collect();
            }
        }
        KeyCode::Char('a') => {
            app.modal = Some(app::Modal::Input {
                title: "Add trusted folder (absolute path)".to_string(),
                value: String::new(),
                cursor: 0,
                is_secret: false,
                action: app::InputAction::TrustedFolderAdd,
            });
        }
        KeyCode::Char('d') | KeyCode::Char('x') => {
            let selected_paths: Vec<String> = if app.trusted_folders_selected.is_empty() {
                app.trusted_folders
                    .get(app.trusted_folders_index)
                    .map(|folder| vec![folder.path.clone()])
                    .unwrap_or_default()
            } else {
                app.trusted_folders
                    .iter()
                    .filter(|folder| app.trusted_folders_selected.contains(&folder.path))
                    .map(|folder| folder.path.clone())
                    .collect()
            };

            if selected_paths.len() > 1 {
                app.modal = Some(app::Modal::Confirm {
                    message: format!("Remove {} selected trusted folders?", selected_paths.len()),
                    action: app::ConfirmAction::TrustedFoldersDelete {
                        paths: selected_paths,
                    },
                });
            } else if let Some(path) = selected_paths.into_iter().next() {
                app.modal = Some(app::Modal::Confirm {
                    message: format!("Remove trusted folder '{}' ?", path),
                    action: app::ConfirmAction::TrustedFolderDelete { path },
                });
            }
        }
        _ => {}
    }
    None
}
