use super::*;

pub(super) fn handle_channels_key(app: &mut app::App, code: KeyCode) -> Option<Action> {
    match code {
        KeyCode::Esc | KeyCode::Char('q') => app.screen = app::Screen::Main,
        KeyCode::Down => app.channels_index = app.channels_index.saturating_add(1),
        KeyCode::Up => app.channels_index = app.channels_index.saturating_sub(1),
        KeyCode::Char('r') => refresh_channels(app),
        KeyCode::Char('n') => {
            let id = uuid::Uuid::new_v4().to_string();
            let created_at = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis() as f64;

            app.channels_edit_draft = Some(droidgear_core::channel::Channel {
                id,
                name: String::new(),
                channel_type: droidgear_core::channel::ChannelType::General,
                base_url: String::new(),
                enabled: true,
                created_at,
            });
            app.channels_edit_field_index = 0;
            app.channels_edit_username.clear();
            app.channels_edit_password.clear();
            app.channels_edit_api_key.clear();
            app.screen = app::Screen::ChannelsEdit;
        }
        KeyCode::Enter | KeyCode::Char('e') => {
            if let Some(c) = app.channels.get(app.channels_index).cloned() {
                app.channels_edit_draft = Some(c.clone());
                app.channels_edit_field_index = 0;
                load_channel_auth_into_edit_state(app, &c);
                app.screen = app::Screen::ChannelsEdit;
            }
        }
        KeyCode::Char('t') => {
            if let Some(c) = app.channels.get(app.channels_index) {
                let mut channels = app.channels.clone();
                if let Some(found) = channels.iter_mut().find(|x| x.id == c.id) {
                    found.enabled = !found.enabled;
                }
                if let Err(e) =
                    droidgear_core::channel::save_channels_for_home(&app.home_dir, channels.clone())
                {
                    app.set_toast(e, true);
                } else {
                    app.channels = channels;
                    app.set_toast("Saved", false);
                }
            }
        }
        KeyCode::Char('d') => {
            if let Some(c) = app.channels.get(app.channels_index) {
                app.modal = Some(app::Modal::Confirm {
                    message: format!("Delete channel '{}'?", c.name),
                    action: app::ConfirmAction::ChannelDelete { id: c.id.clone() },
                });
            }
        }
        KeyCode::Char('E') => return Some(Action::EditChannels),
        KeyCode::Char('A') => {
            if let Some(c) = app.channels.get(app.channels_index) {
                return Some(Action::EditChannelAuth { id: c.id.clone() });
            }
        }
        _ => {}
    }
    None
}

pub(super) fn channel_type_uses_api_key(
    channel_type: &droidgear_core::channel::ChannelType,
) -> bool {
    matches!(
        channel_type,
        droidgear_core::channel::ChannelType::CliProxyApi
            | droidgear_core::channel::ChannelType::Ollama
            | droidgear_core::channel::ChannelType::General
    )
}

pub(super) fn load_channel_auth_into_edit_state(
    app: &mut app::App,
    channel: &droidgear_core::channel::Channel,
) {
    app.channels_edit_username.clear();
    app.channels_edit_password.clear();
    app.channels_edit_api_key.clear();

    if channel_type_uses_api_key(&channel.channel_type) {
        match droidgear_core::channel::get_channel_api_key_for_home(&app.home_dir, &channel.id) {
            Ok(Some(key)) => app.channels_edit_api_key = key,
            Ok(None) => {}
            Err(e) => app.set_toast(e, true),
        }
    } else {
        match droidgear_core::channel::get_channel_credentials_for_home(&app.home_dir, &channel.id)
        {
            Ok(Some((username, password))) => {
                app.channels_edit_username = username;
                app.channels_edit_password = password;
            }
            Ok(None) => {}
            Err(e) => app.set_toast(e, true),
        }
    }
}

pub(super) fn handle_channels_edit_key(app: &mut app::App, code: KeyCode) -> Option<Action> {
    let Some(draft) = app.channels_edit_draft.as_ref() else {
        app.screen = app::Screen::Channels;
        return None;
    };
    let uses_api_key = channel_type_uses_api_key(&draft.channel_type);

    match code {
        KeyCode::Esc | KeyCode::Char('q') => {
            app.channels_edit_draft = None;
            app.channels_edit_field_index = 0;
            app.channels_edit_username.clear();
            app.channels_edit_password.clear();
            app.channels_edit_api_key.clear();
            app.screen = app::Screen::Channels;
        }
        KeyCode::Down => {
            app.channels_edit_field_index = app.channels_edit_field_index.saturating_add(1)
        }
        KeyCode::Up => {
            app.channels_edit_field_index = app.channels_edit_field_index.saturating_sub(1)
        }
        KeyCode::Char('s') => {
            let Some(mut channel) = app.channels_edit_draft.clone() else {
                return None;
            };
            channel.name = channel.name.trim().to_string();
            channel.base_url = channel.base_url.trim().to_string();

            if channel.name.is_empty() {
                app.set_toast("Name is required", true);
                return None;
            }
            if channel.base_url.is_empty() {
                app.set_toast("Base URL is required", true);
                return None;
            }

            if channel_type_uses_api_key(&channel.channel_type) {
                let api_key = app.channels_edit_api_key.trim().to_string();
                if api_key.is_empty() {
                    app.set_toast("API key is required", true);
                    return None;
                }
                if let Err(e) = droidgear_core::channel::save_channel_api_key_for_home(
                    &app.home_dir,
                    &channel.id,
                    &api_key,
                ) {
                    app.set_toast(e, true);
                    return None;
                }
            } else {
                let username = app.channels_edit_username.trim().to_string();
                let password = app.channels_edit_password.trim().to_string();
                if username.is_empty() {
                    app.set_toast("Username is required", true);
                    return None;
                }
                if password.is_empty() {
                    app.set_toast("Password is required", true);
                    return None;
                }
                if let Err(e) = droidgear_core::channel::save_channel_credentials_for_home(
                    &app.home_dir,
                    &channel.id,
                    &username,
                    &password,
                ) {
                    app.set_toast(e, true);
                    return None;
                }
            }

            let mut channels = app.channels.clone();
            if let Some(idx) = channels.iter().position(|c| c.id == channel.id) {
                channels[idx] = channel.clone();
            } else {
                channels.push(channel.clone());
            }

            if let Err(e) =
                droidgear_core::channel::save_channels_for_home(&app.home_dir, channels.clone())
            {
                app.set_toast(e, true);
                return None;
            }

            app.channels_edit_draft = None;
            app.channels_edit_field_index = 0;
            app.channels_edit_username.clear();
            app.channels_edit_password.clear();
            app.channels_edit_api_key.clear();
            app.screen = app::Screen::Channels;
            refresh_channels(app);
            if let Some(idx) = app.channels.iter().position(|c| c.id == channel.id) {
                app.channels_index = idx;
            }
            app.set_toast("Saved", false);
        }
        KeyCode::Enter | KeyCode::Char('e') => match app.channels_edit_field_index {
            0 => {
                app.modal = Some(app::Modal::Input {
                    title: "Name".to_string(),
                    value: draft.name.clone(),
                    cursor: usize::MAX,
                    is_secret: false,
                    action: app::InputAction::ChannelsDraftSetName,
                });
            }
            1 => {
                let options = vec![
                    "new-api".to_string(),
                    "sub-2-api".to_string(),
                    "cli-proxy-api".to_string(),
                    "ollama".to_string(),
                    "general".to_string(),
                ];
                let index = match draft.channel_type {
                    droidgear_core::channel::ChannelType::NewApi => 0,
                    droidgear_core::channel::ChannelType::Sub2Api => 1,
                    droidgear_core::channel::ChannelType::CliProxyApi => 2,
                    droidgear_core::channel::ChannelType::Ollama => 3,
                    droidgear_core::channel::ChannelType::General => 4,
                };
                app.modal = Some(app::Modal::Select {
                    title: "Channel type".to_string(),
                    options,
                    index,
                    action: app::SelectAction::ChannelsDraftSetType,
                });
            }
            2 => {
                app.modal = Some(app::Modal::Input {
                    title: "Base URL".to_string(),
                    value: draft.base_url.clone(),
                    cursor: usize::MAX,
                    is_secret: false,
                    action: app::InputAction::ChannelsDraftSetBaseUrl,
                });
            }
            3 => {
                if let Some(channel) = app.channels_edit_draft.as_mut() {
                    channel.enabled = !channel.enabled;
                }
            }
            4 => {
                if uses_api_key {
                    app.modal = Some(app::Modal::Input {
                        title: "API key".to_string(),
                        value: app.channels_edit_api_key.clone(),
                        cursor: usize::MAX,
                        is_secret: true,
                        action: app::InputAction::ChannelsDraftSetApiKey,
                    });
                } else {
                    app.modal = Some(app::Modal::Input {
                        title: "Username".to_string(),
                        value: app.channels_edit_username.clone(),
                        cursor: usize::MAX,
                        is_secret: false,
                        action: app::InputAction::ChannelsDraftSetUsername,
                    });
                }
            }
            5 if !uses_api_key => {
                app.modal = Some(app::Modal::Input {
                    title: "Password".to_string(),
                    value: app.channels_edit_password.clone(),
                    cursor: usize::MAX,
                    is_secret: true,
                    action: app::InputAction::ChannelsDraftSetPassword,
                });
            }
            _ => {}
        },
        _ => {}
    }

    None
}
