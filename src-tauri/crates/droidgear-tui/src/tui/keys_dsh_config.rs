//! Keys and write helpers for the DSH configuration screen
//! (`Screen::DshConfig`), which renders the composed `cordis.patch.yml`
//! snapshot from `droidgear_core::dsh_config`.

use super::*;
use serde_json::Value as JsonValue;

/// Entries are cheap to walk one row at a time; pages move by this many.
const ENTRY_PAGE: usize = 10;
/// Expanded field lists are short; pages move by this many rows.
const FIELD_PAGE: usize = 5;

pub(super) fn handle_dsh_config_key(app: &mut app::App, code: KeyCode) -> Option<Action> {
    match code {
        KeyCode::Esc | KeyCode::Char('q') => app.go_back(),
        KeyCode::Down => move_selection(app, 1),
        KeyCode::Up => move_selection(app, -1),
        KeyCode::PageDown => move_selection(app, page_size(app) as isize),
        KeyCode::PageUp => move_selection(app, -(page_size(app) as isize)),
        KeyCode::Enter => match app.dsh_config_focus {
            app::DshConfigFocus::Entries => toggle_expanded(app),
            app::DshConfigFocus::Fields => open_field_editor(app),
        },
        KeyCode::Tab => toggle_focus(app),
        KeyCode::Left => app.dsh_config_focus = app::DshConfigFocus::Entries,
        KeyCode::Right => focus_fields(app),
        KeyCode::Char('d') => toggle_disabled(app),
        KeyCode::Char('r') => request_dsh_config_refresh(app),
        _ => {}
    }
    None
}

fn page_size(app: &app::App) -> usize {
    match app.dsh_config_focus {
        app::DshConfigFocus::Entries => ENTRY_PAGE,
        app::DshConfigFocus::Fields => FIELD_PAGE,
    }
}

fn move_selection(app: &mut app::App, delta: isize) {
    match app.dsh_config_focus {
        app::DshConfigFocus::Entries => {
            let count = app.dsh_config_ordered_entries().len();
            app.dsh_config_index = shift_index(app.dsh_config_index, delta, count);
        }
        app::DshConfigFocus::Fields => {
            let count = app
                .dsh_config_selected_entry()
                .map(|entry| entry.fields.len())
                .unwrap_or(0);
            app.dsh_config_field_index = shift_index(app.dsh_config_field_index, delta, count);
        }
    }
}

fn shift_index(current: usize, delta: isize, count: usize) -> usize {
    if count == 0 {
        return 0;
    }
    if delta >= 0 {
        current.saturating_add(delta as usize).min(count - 1)
    } else {
        current.saturating_sub(delta.unsigned_abs())
    }
}

fn toggle_expanded(app: &mut app::App) {
    let Some(entry) = app.dsh_config_selected_entry().cloned() else {
        return;
    };
    if app.dsh_config_expanded.contains(&entry.id) {
        app.dsh_config_expanded.remove(&entry.id);
        app.dsh_config_focus = app::DshConfigFocus::Entries;
    } else {
        app.dsh_config_expanded.insert(entry.id);
        app.dsh_config_field_index = 0;
    }
}

fn toggle_focus(app: &mut app::App) {
    match app.dsh_config_focus {
        app::DshConfigFocus::Entries => focus_fields(app),
        app::DshConfigFocus::Fields => app.dsh_config_focus = app::DshConfigFocus::Entries,
    }
}

fn focus_fields(app: &mut app::App) {
    let Some(entry) = app.dsh_config_selected_entry() else {
        return;
    };
    if !app.dsh_config_expanded.contains(&entry.id) {
        return;
    }
    let field_count = entry.fields.len();
    app.dsh_config_focus = app::DshConfigFocus::Fields;
    app.dsh_config_field_index = app
        .dsh_config_field_index
        .min(field_count.saturating_sub(1));
}

fn open_field_editor(app: &mut app::App) {
    let Some(entry) = app.dsh_config_selected_entry().cloned() else {
        return;
    };
    let Some(field) = entry.fields.get(app.dsh_config_field_index).cloned() else {
        app.set_toast("This entry has no editable config fields", false);
        return;
    };
    if entry.shadowed_by_home {
        let home_patch = app
            .dsh_config_snapshot
            .as_ref()
            .map(|snapshot| snapshot.home_patch_path.clone())
            .unwrap_or_default();
        app.set_toast(
            format!(
                "{} is overridden by the home overlay; edit {home_patch} instead",
                entry.id
            ),
            true,
        );
        return;
    }
    if field.secret {
        app.set_toast(
            "Secret fields are not editable here; use the DSH Models page",
            true,
        );
        return;
    }
    if field.kind == "json" {
        app.set_toast("JSON fields are not editable in the TUI", true);
        return;
    }

    let title = format!("{}.{}", entry.id, field.name);
    let action = app::InputAction::DshConfigSetField {
        entry_id: entry.id.clone(),
        field_name: field.name.clone(),
        kind: field.kind.clone(),
    };
    match field.kind.as_str() {
        "boolean" => {
            let current = field
                .value
                .as_ref()
                .and_then(JsonValue::as_bool)
                .or_else(|| field.default.as_ref().and_then(JsonValue::as_bool))
                .unwrap_or(false);
            app.modal = Some(app::Modal::Select {
                title,
                options: vec!["true".to_string(), "false".to_string()],
                index: if current { 0 } else { 1 },
                action: app::SelectAction::DshConfigSetField {
                    entry_id: entry.id,
                    field_name: field.name,
                    kind: field.kind,
                },
            });
        }
        "enum" => {
            if field.enum_values.is_empty() {
                app.set_toast(format!("{title} has no allowed values"), true);
                return;
            }
            let current = field
                .value
                .as_ref()
                .and_then(JsonValue::as_str)
                .or_else(|| field.default.as_ref().and_then(JsonValue::as_str))
                .unwrap_or_default();
            let index = field
                .enum_values
                .iter()
                .position(|value| value == current)
                .unwrap_or(0);
            app.modal = Some(app::Modal::Select {
                title,
                options: field.enum_values.clone(),
                index,
                action: app::SelectAction::DshConfigSetField {
                    entry_id: entry.id,
                    field_name: field.name,
                    kind: field.kind,
                },
            });
        }
        "number" => {
            let value = field
                .value
                .as_ref()
                .or(field.default.as_ref())
                .map(json_value_text)
                .unwrap_or_default();
            app.modal = Some(app::Modal::Input {
                title,
                value,
                cursor: usize::MAX,
                is_secret: false,
                action,
            });
        }
        "string" => {
            let value = field
                .value
                .as_ref()
                .and_then(JsonValue::as_str)
                .unwrap_or_default()
                .to_string();
            app.modal = Some(app::Modal::Input {
                title,
                value,
                cursor: usize::MAX,
                is_secret: false,
                action,
            });
        }
        other => app.set_toast(format!("Field kind '{other}' cannot be edited"), true),
    }
}

fn toggle_disabled(app: &mut app::App) {
    let Some(snapshot) = app.dsh_config_snapshot.clone() else {
        return;
    };
    let Some(entry) = app.dsh_config_selected_entry().cloned() else {
        return;
    };
    if entry.shadowed_by_home {
        app.set_toast(
            format!(
                "{} is overridden by the home overlay; edit {} instead",
                entry.id, snapshot.home_patch_path
            ),
            true,
        );
        return;
    }
    let disabled = !entry.disabled;
    match droidgear_core::dsh_config::set_dsh_config_entry_disabled_for_home(
        &app.home_dir,
        app.dsh_active_profile.clone(),
        &entry.id,
        disabled,
        Some(snapshot.patch_hash),
    ) {
        Ok(result) => {
            let verb = if disabled { "Disabled" } else { "Enabled" };
            let suffix = if result.changed { "" } else { " (unchanged)" };
            app.set_toast(format!("{verb} {}{suffix}", entry.id), false);
            request_dsh_config_refresh(app);
        }
        Err(error) => app.set_toast(error, true),
    }
}

/// Persist one top-level field of an entry: read the effective config, replace
/// the field, write the whole config back (DSH replaces `config` wholesale),
/// then refresh the snapshot. Used by both `Modal::Input` and `Modal::Select`.
pub(super) fn dsh_config_save_field_value(
    app: &mut app::App,
    entry_id: &str,
    field_name: &str,
    kind: &str,
    raw_value: &str,
) -> anyhow::Result<()> {
    let Some(snapshot) = app.dsh_config_snapshot.clone() else {
        return Err(anyhow::anyhow!(
            "Reload the DSH configuration before saving"
        ));
    };
    let entry = snapshot
        .entries
        .iter()
        .find(|entry| entry.id == entry_id)
        .ok_or_else(|| anyhow::anyhow!("DSH config entry '{entry_id}' not found"))?;
    let volatile = entry
        .fields
        .iter()
        .find(|field| field.name == field_name)
        .map(|field| field.volatile)
        .unwrap_or(true);
    let mut object = match entry.config.clone() {
        None | Some(JsonValue::Null) => serde_json::Map::new(),
        Some(JsonValue::Object(object)) => object,
        Some(_) => {
            return Err(anyhow::anyhow!(
                "DSH config entry '{entry_id}' is not an object"
            ))
        }
    };

    let trimmed = raw_value.trim();
    if trimmed.is_empty() {
        object.remove(field_name);
    } else {
        let value = match kind {
            "boolean" => JsonValue::Bool(parse_boolean(trimmed)?),
            "number" => parse_number(trimmed)?,
            "string" | "enum" => JsonValue::String(trimmed.to_string()),
            other => return Err(anyhow::anyhow!("Field kind '{other}' cannot be saved")),
        };
        object.insert(field_name.to_string(), value);
    }

    let config = if object.is_empty() {
        // DSH treats a missing `config` as "inherit the layer below", which is
        // what an emptied override should restore.
        None
    } else {
        Some(JsonValue::Object(object))
    };
    let result = droidgear_core::dsh_config::save_dsh_config_entry_for_home(
        &app.home_dir,
        app.dsh_active_profile.clone(),
        entry_id,
        config,
        Some(snapshot.patch_hash),
    )
    .map_err(anyhow::Error::msg)?;
    let suffix = if result.changed {
        if volatile {
            ""
        } else {
            " (restart DSH to apply)"
        }
    } else {
        " (unchanged)"
    };
    app.set_toast(format!("Saved {entry_id}.{field_name}{suffix}"), false);
    request_dsh_config_refresh(app);
    Ok(())
}

fn parse_boolean(raw: &str) -> anyhow::Result<bool> {
    match raw.to_ascii_lowercase().as_str() {
        "true" | "1" | "yes" | "on" => Ok(true),
        "false" | "0" | "no" | "off" => Ok(false),
        _ => Err(anyhow::anyhow!("'{raw}' is not a boolean")),
    }
}

fn parse_number(raw: &str) -> anyhow::Result<JsonValue> {
    if let Ok(value) = raw.parse::<i64>() {
        return Ok(JsonValue::from(value));
    }
    if let Ok(value) = raw.parse::<f64>() {
        return Ok(serde_json::json!(value));
    }
    Err(anyhow::anyhow!("'{raw}' is not a number"))
}

/// Short text for a JSON scalar in input prefill and field rows.
pub(super) fn json_value_text(value: &JsonValue) -> String {
    match value {
        JsonValue::String(text) => text.clone(),
        other => other.to_string(),
    }
}
