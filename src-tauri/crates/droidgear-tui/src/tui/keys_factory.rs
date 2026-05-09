use super::*;

pub(super) fn handle_factory_key(app: &mut app::App, code: KeyCode) -> Option<Action> {
    match code {
        KeyCode::Esc | KeyCode::Char('q') => app.screen = app::Screen::Main,
        KeyCode::Down => app.factory_models_index = app.factory_models_index.saturating_add(1),
        KeyCode::Up => app.factory_models_index = app.factory_models_index.saturating_sub(1),
        KeyCode::Char('r') => refresh_factory(app),
        KeyCode::Char('E') => return Some(Action::EditFactoryModels),
        KeyCode::Char('n') => {
            app.factory_edit_index = None;
            app.factory_model_field_index = 0;
            app.factory_draft = Some(droidgear_core::factory_settings::CustomModel {
                model: String::new(),
                id: None,
                index: None,
                display_name: None,
                base_url: String::new(),
                api_key: String::new(),
                provider: droidgear_core::factory_settings::Provider::Openai,
                max_output_tokens: None,
                no_image_support: None,
                extra_args: None,
                extra_headers: None,
            });
            app.screen = app::Screen::FactoryModel;
        }
        KeyCode::Char('c') => {
            if let Some(m) = app.custom_models.get(app.factory_models_index) {
                let mut copy = m.clone();
                copy.id = None;
                copy.index = None;
                app.factory_edit_index = None;
                app.factory_model_field_index = 0;
                app.factory_draft = Some(copy);
                app.screen = app::Screen::FactoryModel;
            }
        }
        KeyCode::Char('x') if !app.custom_models.is_empty() => {
            app.modal = Some(app::Modal::Confirm {
                message: "Delete selected custom model?".to_string(),
                action: app::ConfirmAction::FactoryDeleteModel {
                    index: app.factory_models_index,
                },
            });
        }
        KeyCode::Enter | KeyCode::Char('e') => {
            if let Some(m) = app.custom_models.get(app.factory_models_index) {
                app.factory_edit_index = Some(app.factory_models_index);
                app.factory_model_field_index = 0;
                app.factory_draft = Some(m.clone());
                app.screen = app::Screen::FactoryModel;
            }
        }
        KeyCode::Char('d') => {
            if let Some(model_id) = factory_model_id(
                app.custom_models.get(app.factory_models_index),
                app.factory_models_index,
            ) {
                app.modal = Some(app::Modal::Confirm {
                    message: format!("Set Factory default model to {model_id}?"),
                    action: app::ConfirmAction::FactorySetDefaultModel { model_id },
                });
            }
        }
        _ => {}
    }
    None
}

pub(super) fn normalize_factory_models(
    models: &mut [droidgear_core::factory_settings::CustomModel],
) {
    for (idx, m) in models.iter_mut().enumerate() {
        m.index = Some(idx as u32);
        let display = m
            .display_name
            .clone()
            .filter(|s| !s.trim().is_empty())
            .unwrap_or_else(|| m.model.clone());
        m.id = Some(format!("custom:{display}-{idx}"));
    }
}

pub(super) fn handle_factory_model_key(app: &mut app::App, code: KeyCode) -> Option<Action> {
    let Some(draft) = app.factory_draft.as_ref() else {
        app.screen = app::Screen::Factory;
        return None;
    };

    match code {
        KeyCode::Esc | KeyCode::Char('q') => {
            app.factory_draft = None;
            app.factory_edit_index = None;
            app.screen = app::Screen::Factory;
        }
        KeyCode::Down => {
            app.factory_model_field_index = app.factory_model_field_index.saturating_add(1)
        }
        KeyCode::Up => {
            app.factory_model_field_index = app.factory_model_field_index.saturating_sub(1)
        }
        KeyCode::Char('s') => {
            let Some(mut draft) = app.factory_draft.clone() else {
                return None;
            };
            draft.model = draft.model.trim().to_string();
            draft.base_url = draft.base_url.trim().to_string();
            draft.api_key = draft.api_key.trim().to_string();

            if draft.model.is_empty() {
                app.set_toast("Model id is required", true);
                return None;
            }
            if draft.base_url.is_empty() {
                app.set_toast("Base URL is required", true);
                return None;
            }
            if draft.api_key.is_empty() {
                app.set_toast("API key is required", true);
                return None;
            }

            let mut models = app.custom_models.clone();
            let saved_index = if let Some(edit_index) = app.factory_edit_index {
                if edit_index < models.len() {
                    models[edit_index] = draft;
                    edit_index
                } else {
                    models.push(draft);
                    models.len().saturating_sub(1)
                }
            } else {
                models.push(draft);
                models.len().saturating_sub(1)
            };

            normalize_factory_models(&mut models);

            if let Err(e) =
                droidgear_core::factory_settings::save_custom_models_for_home(&app.home_dir, models)
                    .map_err(anyhow::Error::msg)
            {
                app.set_toast(e.to_string(), true);
                return None;
            }

            app.factory_models_index = saved_index;
            app.factory_draft = None;
            app.factory_edit_index = None;
            app.screen = app::Screen::Factory;
            refresh_factory(app);
            app.set_toast("Saved", false);
        }
        KeyCode::Enter | KeyCode::Char('e') => match app.factory_model_field_index {
            0 => {
                let options = vec![
                    "anthropic".to_string(),
                    "openai".to_string(),
                    "generic-chat-completion-api".to_string(),
                ];
                let current = match draft.provider {
                    droidgear_core::factory_settings::Provider::Anthropic => "anthropic",
                    droidgear_core::factory_settings::Provider::Openai => "openai",
                    droidgear_core::factory_settings::Provider::GenericChatCompletionApi => {
                        "generic-chat-completion-api"
                    }
                };
                let index = options.iter().position(|o| o == current).unwrap_or(0);
                app.modal = Some(app::Modal::Select {
                    title: "Provider".to_string(),
                    options,
                    index,
                    action: app::SelectAction::FactoryDraftSetProvider,
                });
            }
            1 => {
                app.modal = Some(app::Modal::Input {
                    title: "Base URL".to_string(),
                    value: draft.base_url.clone(),
                    cursor: usize::MAX,
                    is_secret: false,
                    action: app::InputAction::FactoryDraftSetBaseUrl,
                });
            }
            2 => {
                app.modal = Some(app::Modal::Input {
                    title: "API key".to_string(),
                    value: draft.api_key.clone(),
                    cursor: usize::MAX,
                    is_secret: true,
                    action: app::InputAction::FactoryDraftSetApiKey,
                });
            }
            3 => {
                app.modal = Some(app::Modal::Input {
                    title: "Model id".to_string(),
                    value: draft.model.clone(),
                    cursor: usize::MAX,
                    is_secret: false,
                    action: app::InputAction::FactoryDraftSetModel,
                });
            }
            4 => {
                app.modal = Some(app::Modal::Input {
                    title: "Display name".to_string(),
                    value: draft.display_name.clone().unwrap_or_default(),
                    cursor: usize::MAX,
                    is_secret: false,
                    action: app::InputAction::FactoryDraftSetDisplayName,
                });
            }
            5 => {
                app.modal = Some(app::Modal::Input {
                    title: "Max output tokens".to_string(),
                    value: draft
                        .max_output_tokens
                        .map(|v| v.to_string())
                        .unwrap_or_default(),
                    cursor: usize::MAX,
                    is_secret: false,
                    action: app::InputAction::FactoryDraftSetMaxOutputTokens,
                });
            }
            6 => {
                if let Some(draft) = app.factory_draft.as_mut() {
                    let current = draft.no_image_support.unwrap_or(false);
                    draft.no_image_support = if !current { Some(true) } else { None };
                }
            }
            7 => {
                let current = app
                    .factory_draft
                    .as_ref()
                    .and_then(|d| d.extra_args.as_ref())
                    .and_then(|m| m.get("reasoning"))
                    .and_then(|v| v.as_object())
                    .and_then(|obj| obj.get("effort"))
                    .and_then(|v| v.as_str())
                    .unwrap_or("none")
                    .to_string();
                let options = vec![
                    "none".to_string(),
                    "low".to_string(),
                    "medium".to_string(),
                    "high".to_string(),
                    "xhigh".to_string(),
                ];
                let index = options.iter().position(|o| o == &current).unwrap_or(0);
                app.modal = Some(app::Modal::Select {
                    title: "Reasoning Effort".to_string(),
                    options,
                    index,
                    action: app::SelectAction::FactoryDraftSetReasoningEffort,
                });
            }
            8 => {
                let current = app
                    .factory_draft
                    .as_ref()
                    .and_then(|d| d.extra_args.as_ref())
                    .map(|m| serde_json::to_string_pretty(m).unwrap_or_default())
                    .unwrap_or_default();
                app.modal = Some(app::Modal::Input {
                    title: "Extra Args (JSON object)".to_string(),
                    value: current,
                    cursor: usize::MAX,
                    is_secret: false,
                    action: app::InputAction::FactoryDraftSetExtraArgs,
                });
            }
            9 => {
                let current = app
                    .factory_draft
                    .as_ref()
                    .and_then(|d| d.extra_headers.as_ref())
                    .map(|m| serde_json::to_string_pretty(m).unwrap_or_default())
                    .unwrap_or_default();
                app.modal = Some(app::Modal::Input {
                    title: "Extra Headers (JSON object)".to_string(),
                    value: current,
                    cursor: usize::MAX,
                    is_secret: false,
                    action: app::InputAction::FactoryDraftSetExtraHeaders,
                });
            }
            _ => {}
        },
        _ => {}
    }

    None
}
