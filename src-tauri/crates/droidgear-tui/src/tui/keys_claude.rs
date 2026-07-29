use super::*;

pub(super) fn handle_claude_key(app: &mut app::App, code: KeyCode) -> Option<Action> {
    match code {
        KeyCode::Esc | KeyCode::Char('q') => app.screen = app::Screen::Main,
        KeyCode::Down => app.claude_index = app.claude_index.saturating_add(1),
        KeyCode::Up => app.claude_index = app.claude_index.saturating_sub(1),
        KeyCode::Char('r') => refresh_claude(app),
        KeyCode::Char('n') => {
            app.modal = Some(app::Modal::Input {
                title: "New Claude settings file name".to_string(),
                value: String::new(),
                cursor: usize::MAX,
                is_secret: false,
                action: app::InputAction::ClaudeSettingsCreateFile,
            });
        }
        KeyCode::Enter | KeyCode::Char('e') => {
            if let Some(file) = app.claude_files.get(app.claude_index) {
                app.claude_detail_name = Some(file.name.clone());
                app.claude_detail_field_index = 0;
                app.screen = app::Screen::ClaudeSettingsDetail;
                refresh_claude_detail(app);
            }
        }
        KeyCode::Char('a') => {
            if let Some(file) = app.claude_files.get(app.claude_index) {
                app.modal = Some(app::Modal::Confirm {
                    message: format!("Merge Claude settings '{}' into global?", file.name),
                    action: app::ConfirmAction::ClaudeSettingsApply {
                        name: file.name.clone(),
                    },
                });
            }
        }
        KeyCode::Char('d') => {
            if let Some(file) = app.claude_files.get(app.claude_index) {
                if file.is_global {
                    app.set_toast("Cannot delete the global settings file", true);
                    return None;
                }
                app.modal = Some(app::Modal::Confirm {
                    message: format!("Delete Claude settings file '{}'?", file.name),
                    action: app::ConfirmAction::ClaudeSettingsDelete {
                        name: file.name.clone(),
                    },
                });
            }
        }
        KeyCode::Char('c') => {
            if let Some(file) = app.claude_files.get(app.claude_index) {
                app.modal = Some(app::Modal::Input {
                    title: format!("Copy '{}' as:", file.name),
                    value: String::new(),
                    cursor: usize::MAX,
                    is_secret: false,
                    action: app::InputAction::ClaudeSettingsDuplicate {
                        name: file.name.clone(),
                    },
                });
            }
        }
        KeyCode::Char('l') => {
            if let Some(file) = app.claude_files.get(app.claude_index) {
                let file_name = file.name.clone();
                match claude_load_from_live_config(app, &file_name) {
                    Ok(()) => {
                        app.set_toast(format!("Loaded live config into '{}'", file_name), false);
                        refresh_claude_detail(app);
                    }
                    Err(e) => app.set_toast(e.to_string(), true),
                }
            }
        }
        KeyCode::Char('t') => {
            if let Some(file) = app.claude_files.get(app.claude_index) {
                return Some(Action::RunClaudeRun {
                    name: file.name.clone(),
                });
            }
        }
        KeyCode::Char('p') => {
            if let Some(file) = app.claude_files.get(app.claude_index) {
                return Some(Action::PreviewClaudeRun {
                    name: file.name.clone(),
                });
            }
        }
        _ => {}
    }
    app.clamp_indices();
    None
}

pub(super) fn handle_claude_settings_detail_key(
    app: &mut app::App,
    code: KeyCode,
) -> Option<Action> {
    let Some(name) = app.claude_detail_name.clone() else {
        return None;
    };

    match code {
        KeyCode::Esc | KeyCode::Char('q') => {
            app.screen = app::Screen::ClaudeSettings;
            app.claude_detail_name = None;
            app.claude_detail_json = None;
        }
        KeyCode::Down => {
            app.claude_detail_field_index = app.claude_detail_field_index.saturating_add(1)
        }
        KeyCode::Up => {
            app.claude_detail_field_index = app.claude_detail_field_index.saturating_sub(1)
        }
        KeyCode::Char('s') => {
            if let Some(ref json) = app.claude_detail_json {
                match droidgear_core::claude_settings_files::save_settings_file_for_home(
                    &app.home_dir,
                    &name,
                    json,
                ) {
                    Ok(()) => app.set_toast(format!("Saved '{}'", name), false),
                    Err(e) => app.set_toast(e, true),
                }
            }
        }
        KeyCode::Enter => {
            // Edit the selected field
            let field_idx = app.claude_detail_field_index;
            match field_idx {
                0 | 1 | 2 | 4 | 10 => {
                    // Text fields: base_url, bearer_token, model, small_model, cleanup_period
                    let current_value = get_claude_field_value(app, field_idx);
                    app.modal = Some(app::Modal::Input {
                        title: claude_field_label(field_idx),
                        value: current_value,
                        cursor: usize::MAX,
                        is_secret: field_idx == 1, // bearer_token is secret
                        action: app::InputAction::ClaudeSettingsEditField {
                            field_index: field_idx,
                        },
                    });
                }
                3 | 5 | 8 | 9 | 13 => {
                    // Toggle fields: small_model_mirror, 1M_context, autoUpdate, includeCoAuthoredBy, skipDangerous
                    toggle_claude_field(app, field_idx);
                }
                6 => {
                    // Reasoning effort select
                    app.modal = Some(app::Modal::Select {
                        title: "Reasoning Effort".to_string(),
                        options: vec![
                            "inherit".to_string(),
                            "low".to_string(),
                            "medium".to_string(),
                            "high".to_string(),
                            "max".to_string(),
                        ],
                        index: claude_reasoning_index(app),
                        action: app::SelectAction::ClaudeSettingsSetReasoningEffort,
                    });
                }
                7 => {
                    // Thinking mode select
                    app.modal = Some(app::Modal::Select {
                        title: "Thinking Mode".to_string(),
                        options: vec!["inherit".to_string(), "on".to_string(), "off".to_string()],
                        index: claude_thinking_index(app),
                        action: app::SelectAction::ClaudeSettingsSetThinkingMode,
                    });
                }
                11 => {
                    // Permissions defaultMode select
                    app.modal = Some(app::Modal::Select {
                        title: "Permissions Default Mode".to_string(),
                        options: vec![
                            "(unset)".to_string(),
                            "default".to_string(),
                            "acceptEdits".to_string(),
                            "plan".to_string(),
                            "auto".to_string(),
                            "dontAsk".to_string(),
                            "bypassPermissions".to_string(),
                        ],
                        index: claude_permissions_default_mode_index(app),
                        action: app::SelectAction::ClaudeSettingsSetPermissionsDefaultMode,
                    });
                }
                12 => {
                    // disableBypass select
                    let current =
                        get_claude_permissions_string(app, "disableBypassPermissionsMode");
                    let options = vec!["(unset)".to_string(), "disable".to_string()];
                    let idx = if current == Some("disable".to_string()) {
                        1
                    } else {
                        0
                    };
                    app.modal = Some(app::Modal::Select {
                        title: "disableBypassPermissionsMode".to_string(),
                        options,
                        index: idx,
                        action: app::SelectAction::ClaudeSettingsSetDisableBypass,
                    });
                }
                _ => {}
            }
        }
        _ => {}
    }
    app.clamp_indices();
    None
}

// ---------------------------------------------------------------------------
// Field helpers
// ---------------------------------------------------------------------------

/// Returns the label for a field index in the detail view.
pub fn claude_field_label(idx: usize) -> String {
    match idx {
        0 => "Base URL".to_string(),
        1 => "Bearer Token".to_string(),
        2 => "Model".to_string(),
        3 => "Use main for small".to_string(),
        4 => "Small Model".to_string(),
        5 => "1M Context".to_string(),
        6 => "Reasoning Effort".to_string(),
        7 => "Thinking Mode".to_string(),
        8 => "autoUpdate".to_string(),
        9 => "includeCoAuthoredBy".to_string(),
        10 => "cleanupPeriodDays".to_string(),
        11 => "Permissions defaultMode".to_string(),
        12 => "disableBypass".to_string(),
        13 => "skipDangerousPrompt".to_string(),
        _ => "Unknown".to_string(),
    }
}

fn get_env_string(json: &serde_json::Value, key: &str) -> String {
    json.get("env")
        .and_then(|env| env.get(key))
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string()
}

fn get_claude_field_value(app: &app::App, idx: usize) -> String {
    let Some(ref json) = app.claude_detail_json else {
        return String::new();
    };
    match idx {
        0 => get_env_string(json, "ANTHROPIC_BASE_URL"),
        1 => get_env_string(json, "ANTHROPIC_AUTH_TOKEN"),
        2 => get_env_string(json, "ANTHROPIC_MODEL"),
        4 => get_env_string(json, "ANTHROPIC_DEFAULT_HAIKU_MODEL"),
        10 => json
            .get("cleanupPeriodDays")
            .and_then(|v| v.as_u64())
            .map(|v| v.to_string())
            .unwrap_or_else(|| "30".to_string()),
        _ => String::new(),
    }
}

fn get_claude_permissions_string(app: &app::App, key: &str) -> Option<String> {
    let json = app.claude_detail_json.as_ref()?;
    json.get(key)
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
}

fn toggle_claude_field(app: &mut app::App, idx: usize) {
    let Some(ref mut json) = app.claude_detail_json else {
        return;
    };
    let obj = json.as_object_mut().unwrap();

    match idx {
        3 => {
            // Small model mirror toggle: flip between "mirror" (no explicit
            // small model) and "explicit copy of main model" (consistent with
            // the GUI checkbox behaviour).  The user can edit the small model
            // via field 4 afterwards.
            let small_set = obj
                .get("env")
                .and_then(|e| e.get("ANTHROPIC_DEFAULT_HAIKU_MODEL"))
                .and_then(|v| v.as_str())
                .is_some_and(|s| !s.is_empty());
            let env = obj
                .entry("env".to_string())
                .or_insert_with(|| serde_json::Value::Object(serde_json::Map::new()));
            if let Some(env_obj) = env.as_object_mut() {
                if small_set {
                    // Currently explicit → switch to mirror.
                    env_obj.remove("ANTHROPIC_DEFAULT_HAIKU_MODEL");
                } else {
                    // Currently mirroring → switch to explicit copy of main model.
                    let main = env_obj
                        .get("ANTHROPIC_MODEL")
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .to_string();
                    if !main.is_empty() {
                        env_obj.insert(
                            "ANTHROPIC_DEFAULT_HAIKU_MODEL".to_string(),
                            serde_json::Value::String(main),
                        );
                    }
                }
            }
        }
        5 => {
            // 1M context: toggle [1m] suffix on model
            let env = obj
                .entry("env".to_string())
                .or_insert_with(|| serde_json::Value::Object(serde_json::Map::new()));
            if let Some(env_obj) = env.as_object_mut() {
                let current = env_obj
                    .get("ANTHROPIC_MODEL")
                    .and_then(|v| v.as_str())
                    .unwrap_or("");
                if current.ends_with("[1m]") {
                    let stripped = current.trim_end_matches("[1m]").to_string();
                    env_obj.insert(
                        "ANTHROPIC_MODEL".to_string(),
                        serde_json::Value::String(stripped),
                    );
                } else if !current.is_empty() {
                    env_obj.insert(
                        "ANTHROPIC_MODEL".to_string(),
                        serde_json::Value::String(format!("{current}[1m]")),
                    );
                }
            }
        }
        8 => {
            // autoUpdate toggle
            let current = obj
                .get("autoUpdate")
                .and_then(|v| v.as_bool())
                .unwrap_or(true);
            if current {
                obj.insert("autoUpdate".to_string(), serde_json::Value::Bool(false));
            } else {
                obj.remove("autoUpdate");
            }
        }
        9 => {
            // includeCoAuthoredBy toggle
            let current = obj
                .get("includeCoAuthoredBy")
                .and_then(|v| v.as_bool())
                .unwrap_or(true);
            if current {
                obj.insert(
                    "includeCoAuthoredBy".to_string(),
                    serde_json::Value::Bool(false),
                );
            } else {
                obj.remove("includeCoAuthoredBy");
            }
        }
        13 => {
            // skipDangerousPrompt toggle
            let current = obj
                .get("skipDangerousModePermissionPrompt")
                .and_then(|v| v.as_bool())
                .unwrap_or(false);
            if current {
                obj.remove("skipDangerousModePermissionPrompt");
            } else {
                obj.insert(
                    "skipDangerousModePermissionPrompt".to_string(),
                    serde_json::Value::Bool(true),
                );
            }
        }
        _ => {}
    }
}

fn claude_reasoning_index(app: &app::App) -> usize {
    let val = app
        .claude_detail_json
        .as_ref()
        .and_then(|j| j.get("env"))
        .and_then(|e| e.get("CLAUDE_CODE_EFFORT_LEVEL"))
        .and_then(|v| v.as_str())
        .unwrap_or("inherit");
    match val {
        "low" => 1,
        "medium" => 2,
        "high" => 3,
        "max" => 4,
        _ => 0,
    }
}

fn claude_thinking_index(app: &app::App) -> usize {
    let val = app
        .claude_detail_json
        .as_ref()
        .and_then(|j| j.get("env"))
        .and_then(|e| e.get("CLAUDE_CODE_DISABLE_THINKING"))
        .and_then(|v| v.as_str());
    let always = app
        .claude_detail_json
        .as_ref()
        .and_then(|j| j.get("alwaysThinkingEnabled"))
        .and_then(|v| v.as_bool());
    match (val, always) {
        (Some("1"), _) => 2,  // off
        (_, Some(true)) => 1, // on
        _ => 0,               // inherit
    }
}

fn claude_permissions_default_mode_index(app: &app::App) -> usize {
    let val = app
        .claude_detail_json
        .as_ref()
        .and_then(|j| j.get("permissions"))
        .and_then(|p| p.get("defaultMode"))
        .and_then(|v| v.as_str());
    match val {
        Some("default") => 1,
        Some("acceptEdits") => 2,
        Some("plan") => 3,
        Some("auto") => 4,
        Some("dontAsk") => 5,
        Some("bypassPermissions") => 6,
        _ => 0,
    }
}
