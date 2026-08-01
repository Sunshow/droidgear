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
                app.claude_detail_dirty = false;
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
        KeyCode::Char('s') => {
            if let Some(file) = app.claude_files.get(app.claude_index) {
                let name = if file.is_global {
                    None
                } else {
                    Some(file.name.clone())
                };
                return Some(Action::SetActiveClaudeSettingsFile { name });
            }
        }
        KeyCode::Char('t') => {
            if let Some(file) = app.claude_files.get(app.claude_index) {
                return Some(Action::RunClaudeRun {
                    name: file.name.clone(),
                    skip_dangerous: false,
                });
            }
        }
        KeyCode::Char('T') => {
            if let Some(file) = app.claude_files.get(app.claude_index) {
                if claude_skip_permissions_disabled(app, &file.name) {
                    app.set_toast(
                        "Running with --dangerously-skip-permissions is disabled by this settings file",
                        true,
                    );
                    return None;
                }
                return Some(Action::RunClaudeRun {
                    name: file.name.clone(),
                    skip_dangerous: true,
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

/// Returns true when the settings file disables `--dangerously-skip-permissions`
/// via `disableBypassPermissionsMode`. Mirrors the GUI RunSkip button guard.
fn claude_skip_permissions_disabled(app: &app::App, name: &str) -> bool {
    droidgear_core::claude_settings_files::read_settings_file_for_home(&app.home_dir, name)
        .ok()
        .map(|json| {
            json.get("disableBypassPermissionsMode")
                .and_then(serde_json::Value::as_str)
                .map(|s| s == "disable")
                .unwrap_or(false)
        })
        .unwrap_or(false)
}

/// Saves the detail JSON to disk when it has unsaved edits. Returns true when
/// nothing was pending or the save succeeded. Mirrors the GUI launch flow,
/// which auto-saves before running.
fn claude_save_detail_if_dirty(app: &mut app::App) -> bool {
    if !app.claude_detail_dirty {
        return true;
    }
    let Some(name) = app.claude_detail_name.clone() else {
        return true;
    };
    let Some(json) = app.claude_detail_json.clone() else {
        return true;
    };
    match droidgear_core::claude_settings_files::save_settings_file_for_home(
        &app.home_dir,
        &name,
        json,
    ) {
        Ok(()) => {
            app.claude_detail_dirty = false;
            app.set_toast(format!("Saved '{}'", name), false);
            true
        }
        Err(e) => {
            app.set_toast(e, true);
            false
        }
    }
}

pub(super) fn exit_claude_detail(app: &mut app::App) {
    app.screen = app::Screen::ClaudeSettings;
    app.claude_detail_name = None;
    app.claude_detail_json = None;
    app.claude_detail_dirty = false;
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
            if app.claude_detail_dirty {
                app.modal = Some(app::Modal::Confirm {
                    message: format!("Discard unsaved changes to '{name}'?"),
                    action: app::ConfirmAction::ClaudeSettingsDiscardDetail,
                });
            } else {
                exit_claude_detail(app);
            }
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
                    json.clone(),
                ) {
                    Ok(()) => {
                        app.claude_detail_dirty = false;
                        app.set_toast(format!("Saved '{}'", name), false);
                    }
                    Err(e) => app.set_toast(e, true),
                }
            }
        }
        KeyCode::Char('t') => {
            if !claude_save_detail_if_dirty(app) {
                return None;
            }
            return Some(Action::RunClaudeRun {
                name,
                skip_dangerous: false,
            });
        }
        KeyCode::Char('T') => {
            if claude_skip_permissions_disabled(app, &name) {
                app.set_toast(
                    "Running with --dangerously-skip-permissions is disabled by this settings file",
                    true,
                );
                return None;
            }
            if !claude_save_detail_if_dirty(app) {
                return None;
            }
            return Some(Action::RunClaudeRun {
                name,
                skip_dangerous: true,
            });
        }
        KeyCode::Char('p') => {
            if !claude_save_detail_if_dirty(app) {
                return None;
            }
            return Some(Action::PreviewClaudeRun { name });
        }
        KeyCode::Char('a') => {
            app.modal = Some(app::Modal::Confirm {
                message: format!("Merge Claude settings '{name}' into global?"),
                action: app::ConfirmAction::ClaudeSettingsApply { name },
            });
        }
        KeyCode::Char('l') => match claude_load_from_live_config(app, &name) {
            Ok(()) => {
                app.set_toast(format!("Loaded live config into '{name}'"), false);
                app.claude_detail_dirty = false;
                refresh_claude_detail(app);
            }
            Err(e) => app.set_toast(e.to_string(), true),
        },
        KeyCode::Char('i') => {
            refresh_channels(app);
            let enabled: Vec<_> = app.channels.iter().filter(|c| c.enabled).collect();
            if enabled.is_empty() {
                app.set_toast("No enabled channels available for import", true);
                return None;
            }
            app.modal = Some(app::Modal::Select {
                title: "Import from Channel".to_string(),
                options: enabled
                    .iter()
                    .map(|c| format!("{} ({})", c.name, c.base_url))
                    .collect(),
                index: 0,
                action: app::SelectAction::ClaudeSettingsImportChannel,
            });
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
                    app.claude_detail_dirty = true;
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
            // 1M context: toggle [1m] suffix on the model. Reads the resolved
            // model (env first, then top-level `model`) and writes the result
            // into env, removing a stale top-level `model` — same semantics as
            // the GUI `toggleModel1MContext` + `syncTopLevelModel`.
            let resolved = obj
                .get("env")
                .and_then(|env| env.get("ANTHROPIC_MODEL"))
                .and_then(|v| v.as_str())
                .map(|s| s.to_string())
                .or_else(|| {
                    obj.get("model")
                        .and_then(|v| v.as_str())
                        .map(|s| s.to_string())
                })
                .unwrap_or_default();
            if resolved.is_empty() {
                return;
            }
            let stripped = resolved.trim_end_matches("[1m]").to_string();
            if stripped.is_empty() {
                return;
            }
            let next = if resolved.ends_with("[1m]") {
                stripped
            } else {
                format!("{stripped}[1m]")
            };
            let env = obj
                .entry("env".to_string())
                .or_insert_with(|| serde_json::Value::Object(serde_json::Map::new()));
            if let Some(env_obj) = env.as_object_mut() {
                env_obj.insert(
                    "ANTHROPIC_MODEL".to_string(),
                    serde_json::Value::String(next),
                );
            }
            obj.remove("model");
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
