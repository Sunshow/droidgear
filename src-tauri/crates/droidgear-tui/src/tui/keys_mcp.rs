use super::*;

pub(super) fn handle_mcp_key(app: &mut app::App, code: KeyCode) -> Option<Action> {
    match code {
        KeyCode::Esc | KeyCode::Char('q') => app.screen = app::Screen::Main,
        KeyCode::Down => app.mcp_index = app.mcp_index.saturating_add(1),
        KeyCode::Up => app.mcp_index = app.mcp_index.saturating_sub(1),
        KeyCode::Char('r') => refresh_mcp(app),
        KeyCode::Char('n') => {
            app.modal = Some(app::Modal::Input {
                title: "New MCP server name".to_string(),
                value: String::new(),
                cursor: usize::MAX,
                is_secret: false,
                action: app::InputAction::McpCreateServer,
            });
        }
        KeyCode::Enter | KeyCode::Char('e') => {
            if let Some(server) = app.mcp_servers.get(app.mcp_index) {
                app.mcp_edit_original_name = Some(server.name.clone());
                app.mcp_edit_draft = Some(server.clone());
                app.mcp_edit_field_index = 0;
                app.mcp_args_index = 0;
                app.mcp_kv_index = 0;
                app.screen = app::Screen::McpServer;
            }
        }
        KeyCode::Char('t') => {
            if let Some(server) = app.mcp_servers.get(app.mcp_index) {
                app.modal = Some(app::Modal::Confirm {
                    message: format!(
                        "Toggle MCP server '{}' to {}?",
                        server.name,
                        if server.config.disabled {
                            "enabled"
                        } else {
                            "disabled"
                        }
                    ),
                    action: app::ConfirmAction::McpToggle {
                        name: server.name.clone(),
                        disabled: !server.config.disabled,
                    },
                });
            }
        }
        KeyCode::Char('d') => {
            if let Some(server) = app.mcp_servers.get(app.mcp_index) {
                app.modal = Some(app::Modal::Confirm {
                    message: format!("Delete MCP server '{}'?", server.name),
                    action: app::ConfirmAction::McpDelete {
                        name: server.name.clone(),
                    },
                });
            }
        }
        _ => {}
    }
    None
}

pub(super) fn handle_mcp_server_key(app: &mut app::App, code: KeyCode) -> Option<Action> {
    let Some(draft) = app.mcp_edit_draft.as_ref() else {
        app.screen = app::Screen::Mcp;
        return None;
    };
    let server_type = draft.config.server_type.clone();

    match code {
        KeyCode::Esc | KeyCode::Char('q') => {
            app.mcp_edit_draft = None;
            app.mcp_edit_original_name = None;
            app.screen = app::Screen::Mcp;
        }
        KeyCode::Down => app.mcp_edit_field_index = app.mcp_edit_field_index.saturating_add(1),
        KeyCode::Up => app.mcp_edit_field_index = app.mcp_edit_field_index.saturating_sub(1),
        KeyCode::Char('s') => {
            let Some(mut server) = app.mcp_edit_draft.clone() else {
                return None;
            };
            server.name = server.name.trim().to_string();
            if server.name.is_empty() {
                app.set_toast("Name is required", true);
                return None;
            }

            match server.config.server_type {
                droidgear_core::mcp::McpServerType::Stdio => {
                    server.config.url = None;
                    server.config.headers = None;

                    server.config.command = server
                        .config
                        .command
                        .map(|s| s.trim().to_string())
                        .filter(|s| !s.is_empty());
                    if server.config.command.is_none() {
                        app.set_toast("Command is required", true);
                        return None;
                    }

                    if let Some(mut args) = server.config.args.take() {
                        for a in args.iter_mut() {
                            *a = a.trim().to_string();
                        }
                        args.retain(|a| !a.is_empty());
                        server.config.args = (!args.is_empty()).then_some(args);
                    }

                    if let Some(env) = server.config.env.as_mut() {
                        let mut cleaned = std::collections::HashMap::new();
                        for (k, v) in env.drain() {
                            let key = k.trim().to_string();
                            if key.is_empty() {
                                continue;
                            }
                            cleaned.insert(key, v.trim().to_string());
                        }
                        server.config.env = (!cleaned.is_empty()).then_some(cleaned);
                    }
                }
                droidgear_core::mcp::McpServerType::Http => {
                    server.config.command = None;
                    server.config.args = None;
                    server.config.env = None;

                    server.config.url = server
                        .config
                        .url
                        .map(|s| s.trim().to_string())
                        .filter(|s| !s.is_empty());
                    if server.config.url.is_none() {
                        app.set_toast("URL is required", true);
                        return None;
                    }

                    if let Some(headers) = server.config.headers.as_mut() {
                        let mut cleaned = std::collections::HashMap::new();
                        for (k, v) in headers.drain() {
                            let key = k.trim().to_string();
                            if key.is_empty() {
                                continue;
                            }
                            cleaned.insert(key, v.trim().to_string());
                        }
                        server.config.headers = (!cleaned.is_empty()).then_some(cleaned);
                    }
                }
            }

            if let Some(original) = app.mcp_edit_original_name.as_deref() {
                if original != server.name {
                    if let Err(e) =
                        droidgear_core::mcp::delete_mcp_server_for_home(&app.home_dir, original)
                    {
                        app.set_toast(e, true);
                        return None;
                    }
                }
            }

            if let Err(e) =
                droidgear_core::mcp::save_mcp_server_for_home(&app.home_dir, server.clone())
            {
                app.set_toast(e, true);
                return None;
            }

            app.mcp_edit_draft = None;
            app.mcp_edit_original_name = None;
            app.screen = app::Screen::Mcp;
            refresh_mcp(app);
            if let Some(idx) = app.mcp_servers.iter().position(|s| s.name == server.name) {
                app.mcp_index = idx;
            }
            app.set_toast("Saved", false);
        }
        KeyCode::Enter | KeyCode::Char('e') => match server_type {
            droidgear_core::mcp::McpServerType::Stdio => match app.mcp_edit_field_index {
                0 => {
                    app.modal = Some(app::Modal::Input {
                        title: "Server name".to_string(),
                        value: draft.name.clone(),
                        cursor: usize::MAX,
                        is_secret: false,
                        action: app::InputAction::McpDraftSetName,
                    });
                }
                1 => {
                    let options = vec!["stdio".to_string(), "http".to_string()];
                    let index = match draft.config.server_type {
                        droidgear_core::mcp::McpServerType::Stdio => 0,
                        droidgear_core::mcp::McpServerType::Http => 1,
                    };
                    app.modal = Some(app::Modal::Select {
                        title: "Server type".to_string(),
                        options,
                        index,
                        action: app::SelectAction::McpDraftSetType,
                    });
                }
                2 => {
                    if let Some(server) = app.mcp_edit_draft.as_mut() {
                        server.config.disabled = !server.config.disabled;
                    }
                }
                3 => {
                    app.modal = Some(app::Modal::Input {
                        title: "Command".to_string(),
                        value: draft.config.command.clone().unwrap_or_default(),
                        cursor: usize::MAX,
                        is_secret: false,
                        action: app::InputAction::McpDraftSetCommand,
                    });
                }
                4 => {
                    app.mcp_args_index = 0;
                    app.screen = app::Screen::McpArgs;
                }
                5 => {
                    app.mcp_kv_mode = app::McpKeyValuesMode::Env;
                    app.mcp_kv_index = 0;
                    app.screen = app::Screen::McpKeyValues;
                }
                _ => {}
            },
            droidgear_core::mcp::McpServerType::Http => match app.mcp_edit_field_index {
                0 => {
                    app.modal = Some(app::Modal::Input {
                        title: "Server name".to_string(),
                        value: draft.name.clone(),
                        cursor: usize::MAX,
                        is_secret: false,
                        action: app::InputAction::McpDraftSetName,
                    });
                }
                1 => {
                    let options = vec!["stdio".to_string(), "http".to_string()];
                    let index = match draft.config.server_type {
                        droidgear_core::mcp::McpServerType::Stdio => 0,
                        droidgear_core::mcp::McpServerType::Http => 1,
                    };
                    app.modal = Some(app::Modal::Select {
                        title: "Server type".to_string(),
                        options,
                        index,
                        action: app::SelectAction::McpDraftSetType,
                    });
                }
                2 => {
                    if let Some(server) = app.mcp_edit_draft.as_mut() {
                        server.config.disabled = !server.config.disabled;
                    }
                }
                3 => {
                    app.modal = Some(app::Modal::Input {
                        title: "URL".to_string(),
                        value: draft.config.url.clone().unwrap_or_default(),
                        cursor: usize::MAX,
                        is_secret: false,
                        action: app::InputAction::McpDraftSetUrl,
                    });
                }
                4 => {
                    app.mcp_kv_mode = app::McpKeyValuesMode::Headers;
                    app.mcp_kv_index = 0;
                    app.screen = app::Screen::McpKeyValues;
                }
                _ => {}
            },
        },
        _ => {}
    }

    None
}

pub(super) fn handle_mcp_args_key(app: &mut app::App, code: KeyCode) -> Option<Action> {
    match code {
        KeyCode::Esc | KeyCode::Char('q') => app.screen = app::Screen::McpServer,
        KeyCode::Down => app.mcp_args_index = app.mcp_args_index.saturating_add(1),
        KeyCode::Up => app.mcp_args_index = app.mcp_args_index.saturating_sub(1),
        KeyCode::Char('n') => {
            app.modal = Some(app::Modal::Input {
                title: "New arg".to_string(),
                value: String::new(),
                cursor: usize::MAX,
                is_secret: false,
                action: app::InputAction::McpArgsAdd,
            });
        }
        KeyCode::Enter | KeyCode::Char('e') => {
            let current = app
                .mcp_edit_draft
                .as_ref()
                .and_then(|s| s.config.args.as_ref())
                .and_then(|v| v.get(app.mcp_args_index))
                .cloned();
            if let Some(current) = current {
                app.modal = Some(app::Modal::Input {
                    title: "Arg".to_string(),
                    value: current,
                    cursor: usize::MAX,
                    is_secret: false,
                    action: app::InputAction::McpArgsEdit {
                        index: app.mcp_args_index,
                    },
                });
            }
        }
        KeyCode::Char('x') => {
            if let Some(server) = app.mcp_edit_draft.as_mut() {
                if let Some(args) = server.config.args.as_mut() {
                    if app.mcp_args_index < args.len() {
                        args.remove(app.mcp_args_index);
                    }
                    if args.is_empty() {
                        server.config.args = None;
                        app.mcp_args_index = 0;
                    }
                }
            }
        }
        _ => {}
    }
    None
}

pub(super) fn handle_mcp_key_values_key(app: &mut app::App, code: KeyCode) -> Option<Action> {
    let Some(server) = app.mcp_edit_draft.as_ref() else {
        app.screen = app::Screen::Mcp;
        return None;
    };
    let mode = app.mcp_kv_mode;

    let mut keys: Vec<String> = match mode {
        app::McpKeyValuesMode::Env => server
            .config
            .env
            .as_ref()
            .map(|m| m.keys().cloned().collect())
            .unwrap_or_default(),
        app::McpKeyValuesMode::Headers => server
            .config
            .headers
            .as_ref()
            .map(|m| m.keys().cloned().collect())
            .unwrap_or_default(),
    };
    keys.sort_by_key(|a| a.to_lowercase());

    match code {
        KeyCode::Esc | KeyCode::Char('q') => app.screen = app::Screen::McpServer,
        KeyCode::Down => app.mcp_kv_index = app.mcp_kv_index.saturating_add(1),
        KeyCode::Up => app.mcp_kv_index = app.mcp_kv_index.saturating_sub(1),
        KeyCode::Char('n') => {
            app.modal = Some(app::Modal::Input {
                title: "key=value".to_string(),
                value: String::new(),
                cursor: usize::MAX,
                is_secret: false,
                action: app::InputAction::McpKeyValueAdd { mode },
            });
        }
        KeyCode::Enter | KeyCode::Char('e') => {
            if let Some(key) = keys.get(app.mcp_kv_index).cloned() {
                let value = match mode {
                    app::McpKeyValuesMode::Env => server
                        .config
                        .env
                        .as_ref()
                        .and_then(|m| m.get(&key))
                        .cloned()
                        .unwrap_or_default(),
                    app::McpKeyValuesMode::Headers => server
                        .config
                        .headers
                        .as_ref()
                        .and_then(|m| m.get(&key))
                        .cloned()
                        .unwrap_or_default(),
                };
                app.modal = Some(app::Modal::Input {
                    title: "key=value".to_string(),
                    value: format!("{key}={value}"),
                    cursor: usize::MAX,
                    is_secret: false,
                    action: app::InputAction::McpKeyValueEdit {
                        mode,
                        index: app.mcp_kv_index,
                    },
                });
            }
        }
        KeyCode::Char('x') => {
            if let Some(key) = keys.get(app.mcp_kv_index).cloned() {
                if let Some(server) = app.mcp_edit_draft.as_mut() {
                    match mode {
                        app::McpKeyValuesMode::Env => {
                            if let Some(env) = server.config.env.as_mut() {
                                env.remove(&key);
                                if env.is_empty() {
                                    server.config.env = None;
                                    app.mcp_kv_index = 0;
                                }
                            }
                        }
                        app::McpKeyValuesMode::Headers => {
                            if let Some(headers) = server.config.headers.as_mut() {
                                headers.remove(&key);
                                if headers.is_empty() {
                                    server.config.headers = None;
                                    app.mcp_kv_index = 0;
                                }
                            }
                        }
                    }
                }
            }
        }
        _ => {}
    }

    None
}
