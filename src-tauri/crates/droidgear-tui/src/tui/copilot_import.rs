use crate::app::{App, InputAction, Modal, SelectAction};
use droidgear_core::{
    channel::{self, Channel, ChannelToken},
    copilot::{self, CopilotChannelSelection},
};

pub(super) fn start(app: &mut App, profile_id: String) -> anyhow::Result<()> {
    let channels = channel::load_channels_for_home(&app.home_dir)
        .map_err(anyhow::Error::msg)?
        .into_iter()
        .filter(|channel| channel.enabled)
        .collect::<Vec<_>>();
    anyhow::ensure!(!channels.is_empty(), "No enabled channels available");
    app.modal = Some(Modal::Select {
        title: "Import Copilot model from channel".to_string(),
        options: channels
            .iter()
            .map(|channel| format!("{} ({})", channel.name, channel.base_url))
            .collect(),
        index: 0,
        action: SelectAction::CopilotImportChannel {
            profile_id,
            channels,
        },
    });
    Ok(())
}

pub(super) fn select_channel(
    app: &mut App,
    profile_id: String,
    channel: Channel,
) -> anyhow::Result<()> {
    let auth = if super::keys_channels::channel_type_uses_api_key(&channel.channel_type) {
        channel::get_channel_api_key_for_home(&app.home_dir, &channel.id)
            .map_err(anyhow::Error::msg)?
            .map(|key| (String::new(), key))
    } else {
        channel::get_channel_credentials_for_home(&app.home_dir, &channel.id)
            .map_err(anyhow::Error::msg)?
    };
    if let Some((username, password)) = auth.filter(|(_, password)| !password.trim().is_empty()) {
        let tokens = channel::fetch_channel_tokens_blocking(
            channel.channel_type.clone(),
            &channel.base_url,
            &username,
            &password,
        )
        .map_err(anyhow::Error::msg)?;
        show_tokens(app, profile_id, channel, tokens)
    } else {
        app.modal = Some(Modal::Input {
            title: format!("API key for '{}'", channel.name),
            value: String::new(),
            cursor: usize::MAX,
            is_secret: true,
            action: InputAction::CopilotImportApiKey {
                profile_id,
                channel,
            },
        });
        Ok(())
    }
}

fn show_tokens(
    app: &mut App,
    profile_id: String,
    channel: Channel,
    tokens: Vec<ChannelToken>,
) -> anyhow::Result<()> {
    let tokens = tokens
        .into_iter()
        .filter(|token| {
            !token.key.trim().is_empty()
                && copilot::supports_channel_platform(token.platform.as_deref())
        })
        .collect::<Vec<_>>();
    anyhow::ensure!(
        !tokens.is_empty(),
        "No OpenAI or Anthropic API keys available on this channel"
    );
    app.modal = Some(Modal::Select {
        title: format!("Select API key ({})", channel.name),
        // Selection uses the index, so duplicate names are safe and keys stay hidden.
        options: tokens
            .iter()
            .map(|token| {
                format!(
                    "{} ({})",
                    token.name,
                    token.platform.as_deref().unwrap_or("auto")
                )
            })
            .collect(),
        index: 0,
        action: SelectAction::CopilotImportToken {
            profile_id,
            channel,
            tokens,
        },
    });
    Ok(())
}

pub(super) fn select_token(
    app: &mut App,
    profile_id: String,
    channel: Channel,
    token: ChannelToken,
) -> anyhow::Result<()> {
    let selection = CopilotChannelSelection {
        channel_type: channel.channel_type,
        base_url: channel.base_url,
        api_key: token.key,
        platform: token.platform,
        provider: None,
        model: String::new(),
        max_output_tokens: None,
    };
    if selection
        .platform
        .as_deref()
        .is_some_and(|platform| platform.eq_ignore_ascii_case("deepseek"))
    {
        app.modal = Some(Modal::Select {
            title: "Select Copilot API protocol".to_string(),
            options: vec!["OpenAI".to_string(), "Anthropic".to_string()],
            index: 0,
            action: SelectAction::CopilotImportProtocol {
                profile_id,
                selection,
            },
        });
        Ok(())
    } else {
        fetch_models(app, profile_id, selection)
    }
}

pub(super) fn enter_api_key(
    app: &mut App,
    profile_id: String,
    channel: Channel,
    api_key: String,
) -> anyhow::Result<()> {
    anyhow::ensure!(!api_key.trim().is_empty(), "API key is required");
    fetch_models(
        app,
        profile_id,
        CopilotChannelSelection {
            channel_type: channel.channel_type,
            base_url: channel.base_url,
            api_key: api_key.trim().to_string(),
            platform: None,
            provider: None,
            model: String::new(),
            max_output_tokens: None,
        },
    )
}

pub(super) fn fetch_models(
    app: &mut App,
    profile_id: String,
    selection: CopilotChannelSelection,
) -> anyhow::Result<()> {
    let models = channel::fetch_models_by_api_key_blocking(
        &selection.base_url,
        &selection.api_key,
        selection.platform.as_deref(),
    )
    .map_err(anyhow::Error::msg)?;
    anyhow::ensure!(!models.is_empty(), "No models available for this API key");
    app.modal = Some(Modal::Select {
        title: "Select Copilot model (saves profile; does not apply)".to_string(),
        options: models.iter().map(|model| model.id.clone()).collect(),
        index: 0,
        action: SelectAction::CopilotImportModel {
            profile_id,
            selection,
            models,
        },
    });
    Ok(())
}

pub(super) fn save_import(
    app: &mut App,
    profile_id: &str,
    selection: CopilotChannelSelection,
) -> anyhow::Result<()> {
    let profile = copilot::get_copilot_profile_for_home(&app.home_dir, profile_id)
        .map_err(anyhow::Error::msg)?;
    let profile =
        copilot::prepare_channel_import(profile, selection).map_err(anyhow::Error::msg)?;
    copilot::save_copilot_profile_for_home(&app.home_dir, profile).map_err(anyhow::Error::msg)?;
    super::refresh_copilot(app);
    app.set_toast(
        "Imported and saved profile; press Enter to run or a to apply",
        false,
    );
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::Screen;
    use crate::tui::{keys_copilot::handle_copilot_key, modal::handle_modal_key};
    use crossterm::event::KeyCode;
    use droidgear_core::channel::ChannelType;
    use std::io::{BufRead, BufReader, Write};
    use std::net::TcpListener;
    use std::time::{Duration, Instant};
    use tempfile::TempDir;

    fn channel(base_url: String) -> Channel {
        Channel {
            id: "channel".to_string(),
            name: "Test channel".to_string(),
            channel_type: ChannelType::General,
            base_url,
            enabled: true,
            created_at: 0.0,
        }
    }

    fn token(id: f64, platform: &str) -> ChannelToken {
        ChannelToken {
            id,
            name: "Same name".to_string(),
            key: format!("private-key-{id}"),
            status: 1,
            remain_quota: 0.0,
            used_quota: 0.0,
            unlimited_quota: true,
            platform: Some(platform.to_string()),
            group_name: None,
        }
    }

    fn models_server() -> (String, std::thread::JoinHandle<String>) {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let address = listener.local_addr().unwrap();
        listener.set_nonblocking(true).unwrap();
        let handle = std::thread::spawn(move || {
            let deadline = Instant::now() + Duration::from_secs(5);
            let mut stream = loop {
                match listener.accept() {
                    Ok((stream, _)) => break stream,
                    Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => {
                        assert!(Instant::now() < deadline, "Model request timed out");
                        std::thread::sleep(Duration::from_millis(10));
                    }
                    Err(error) => panic!("{error}"),
                }
            };
            stream
                .set_read_timeout(Some(Duration::from_secs(5)))
                .unwrap();
            let mut reader = BufReader::new(&stream);
            let mut request = String::new();
            loop {
                let mut line = String::new();
                reader.read_line(&mut line).unwrap();
                if line.trim().is_empty() {
                    break;
                }
                request.push_str(&line);
            }
            let body = r#"{"data":[{"id":"first-model"},{"id":"selected-model"}]}"#;
            write!(stream, "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}", body.len()).unwrap();
            request
        });
        (format!("http://{address}"), handle)
    }

    fn select(app: &mut App, index: usize) {
        let Some(Modal::Select {
            title,
            options,
            action,
            ..
        }) = app.modal.take()
        else {
            panic!("Expected selection; {}", app.toast_message());
        };
        handle_modal_key(
            app,
            KeyCode::Enter,
            Modal::Select {
                title,
                options,
                index,
                action,
            },
        );
    }

    #[test]
    fn channel_import_selects_a_model_and_saves_without_applying() {
        let temp = TempDir::new().unwrap();
        let mut app = App::new(temp.path().to_path_buf());
        app.screen = Screen::Copilot;
        super::super::refresh_copilot(&mut app);
        let mut original = app.copilot_profiles[0].clone();
        original.use_official_auth = true;
        copilot::save_copilot_profile_for_home(temp.path(), original.clone()).unwrap();
        copilot::apply_copilot_profile_for_home(temp.path(), &original.id).unwrap();
        let config_path = temp.path().join(".droidgear/copilot/config.env");
        let before = std::fs::read(&config_path).unwrap();
        let (url, server) = models_server();
        let channel = channel(url.clone());
        channel::save_channels_for_home(temp.path(), vec![channel.clone()]).unwrap();
        channel::save_channel_api_key_for_home(temp.path(), &channel.id, "channel-secret").unwrap();

        handle_copilot_key(&mut app, KeyCode::Char('i'));
        select(&mut app, 0); // channel
        let Some(Modal::Select { options, .. }) = app.modal.as_ref() else {
            panic!("Expected token picker");
        };
        assert!(!options.join(" ").contains("channel-secret"));
        select(&mut app, 0); // API key, fetch models
        select(&mut app, 1); // selected-model

        let request = server.join().unwrap();
        assert!(request.starts_with("GET /v1/models "));
        assert!(request
            .to_lowercase()
            .contains("authorization: bearer channel-secret"));
        let imported = copilot::get_copilot_profile_for_home(temp.path(), &original.id).unwrap();
        assert_eq!(imported.name, original.name);
        assert!(!imported.use_official_auth);
        assert_eq!(imported.model.as_deref(), Some("selected-model"));
        assert_eq!(imported.base_url, Some(format!("{url}/v1")));
        assert_eq!(std::fs::read(config_path).unwrap(), before);
        assert_eq!(
            copilot::get_active_copilot_profile_id_for_home(temp.path())
                .unwrap()
                .as_deref(),
            Some(original.id.as_str())
        );
    }

    #[test]
    fn duplicate_token_names_keep_selection_and_protocol_while_cancel_does_not_save() {
        let temp = TempDir::new().unwrap();
        let mut app = App::new(temp.path().to_path_buf());
        app.screen = Screen::Copilot;
        super::super::refresh_copilot(&mut app);
        let original = app.copilot_profiles[0].clone();
        let (url, server) = models_server();
        let mut channel = channel(format!("{url}/v1"));
        channel.channel_type = ChannelType::Sub2Api;
        show_tokens(
            &mut app,
            original.id.clone(),
            channel,
            vec![
                token(0.0, "gemini"),
                token(1.0, "deepseek"),
                token(2.0, "deepseek"),
            ],
        )
        .unwrap();
        let Some(Modal::Select { options, .. }) = app.modal.as_ref() else {
            panic!("Expected token picker");
        };
        assert_eq!(options.len(), 2);
        assert_eq!(options[0], options[1]);
        assert!(!options.join(" ").contains("private-key"));
        select(&mut app, 1); // the second supported token
        select(&mut app, 1); // Anthropic
        assert!(server
            .join()
            .unwrap()
            .to_lowercase()
            .contains("authorization: bearer private-key-2"));
        let modal = app.modal.take().unwrap();
        let retry = modal.clone();
        handle_modal_key(&mut app, KeyCode::Esc, modal);
        assert!(app.modal.is_none());
        assert!(
            copilot::get_copilot_profile_for_home(temp.path(), &original.id)
                .unwrap()
                .model
                .is_none()
        );
        assert!(
            !copilot::get_copilot_config_status_for_home(temp.path())
                .unwrap()
                .config_exists
        );

        app.modal = Some(retry);
        select(&mut app, 1);
        let imported = copilot::get_copilot_profile_for_home(temp.path(), &original.id).unwrap();
        assert_eq!(imported.provider_type.as_deref(), Some("anthropic"));
        assert_eq!(imported.base_url, Some(url));
        assert_eq!(imported.api_key.as_deref(), Some("private-key-2"));
        assert!(copilot::get_active_copilot_profile_id_for_home(temp.path())
            .unwrap()
            .is_none());
    }
}
