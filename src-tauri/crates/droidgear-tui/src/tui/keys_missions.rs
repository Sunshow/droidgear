use super::*;

pub(super) fn handle_missions_key(app: &mut app::App, code: KeyCode) -> Option<Action> {
    let effort_options = || {
        vec![
            "(not set)".to_string(),
            "none".to_string(),
            "low".to_string(),
            "medium".to_string(),
            "high".to_string(),
        ]
    };

    let model_options = |app: &app::App| -> Vec<String> {
        let mut opts = vec!["(not set)".to_string()];
        for m in &app.custom_models {
            let id = m.id.as_deref().unwrap_or(&m.model);
            opts.push(id.to_string());
        }
        opts
    };

    match code {
        KeyCode::Esc | KeyCode::Char('q') => app.screen = app::Screen::Main,
        KeyCode::Down => app.mission_field_index = app.mission_field_index.saturating_add(1),
        KeyCode::Up => app.mission_field_index = app.mission_field_index.saturating_sub(1),
        KeyCode::Char('r') => refresh_missions(app),
        KeyCode::Enter | KeyCode::Char('e') => match app.mission_field_index {
            0 => {
                let options = model_options(app);
                let current = app
                    .mission_settings
                    .worker_model
                    .as_deref()
                    .unwrap_or("(not set)");
                let index = options.iter().position(|o| o == current).unwrap_or(0);
                app.modal = Some(app::Modal::Select {
                    title: "Worker Model".to_string(),
                    options,
                    index,
                    action: app::SelectAction::MissionsSetWorkerModel,
                });
            }
            1 => {
                let options = effort_options();
                let current = app
                    .mission_settings
                    .worker_reasoning_effort
                    .as_deref()
                    .unwrap_or("(not set)");
                let index = options.iter().position(|o| o == current).unwrap_or(0);
                app.modal = Some(app::Modal::Select {
                    title: "Worker Reasoning Effort".to_string(),
                    options,
                    index,
                    action: app::SelectAction::MissionsSetWorkerReasoningEffort,
                });
            }
            2 => {
                let options = model_options(app);
                let current = app
                    .mission_settings
                    .validation_worker_model
                    .as_deref()
                    .unwrap_or("(not set)");
                let index = options.iter().position(|o| o == current).unwrap_or(0);
                app.modal = Some(app::Modal::Select {
                    title: "Validation Worker Model".to_string(),
                    options,
                    index,
                    action: app::SelectAction::MissionsSetValidationWorkerModel,
                });
            }
            3 => {
                let options = effort_options();
                let current = app
                    .mission_settings
                    .validation_worker_reasoning_effort
                    .as_deref()
                    .unwrap_or("(not set)");
                let index = options.iter().position(|o| o == current).unwrap_or(0);
                app.modal = Some(app::Modal::Select {
                    title: "Validation Worker Reasoning Effort".to_string(),
                    options,
                    index,
                    action: app::SelectAction::MissionsSetValidationWorkerReasoningEffort,
                });
            }
            _ => {}
        },
        _ => {}
    }

    None
}
