use crate::app;
use ratatui::{
    layout::{Constraint, Direction, Layout, Position, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, ListState, Padding, Paragraph, Wrap},
    Frame,
};
use std::sync::OnceLock;

#[derive(Debug, Clone, Copy)]
struct Theme {
    border: Color,
    title: Color,
    key: Color,
    dim: Color,
    selection_bg: Color,
    selection_fg: Color,
    selection_mod: Modifier,
    success: Color,
    error: Color,
    warning: Color,
    modal_bg: Color,
}

impl Theme {
    fn plain() -> Self {
        Self {
            border: Color::Reset,
            title: Color::Reset,
            key: Color::Reset,
            dim: Color::Reset,
            selection_bg: Color::Reset,
            selection_fg: Color::Reset,
            selection_mod: Modifier::REVERSED,
            success: Color::Reset,
            error: Color::Reset,
            warning: Color::Reset,
            modal_bg: Color::Reset,
        }
    }

    fn nord() -> Self {
        // Inspired by the Nord palette (similar vibe to many “modern” TUIs like lazygit).
        let nord0 = Color::Rgb(0x2e, 0x34, 0x40);
        let nord3 = Color::Rgb(0x4c, 0x56, 0x6a);
        let nord6 = Color::Rgb(0xec, 0xef, 0xf4);
        let nord8 = Color::Rgb(0x88, 0xc0, 0xd0);
        let nord9 = Color::Rgb(0x81, 0xa1, 0xc1);
        let nord10 = Color::Rgb(0x5e, 0x81, 0xac);
        let nord11 = Color::Rgb(0xbf, 0x61, 0x6a);
        let nord13 = Color::Rgb(0xeb, 0xcb, 0x8b);
        let nord14 = Color::Rgb(0xa3, 0xbe, 0x8c);

        Self {
            border: nord3,
            title: nord8,
            key: nord9,
            dim: nord3,
            selection_bg: nord10,
            selection_fg: nord6,
            selection_mod: Modifier::BOLD,
            success: nord14,
            error: nord11,
            warning: nord13,
            modal_bg: nord0,
        }
    }

    fn opencode() -> Self {
        // A more vibrant, high-contrast palette inspired by modern TUIs.
        // Designed to make keys, selections, and “needs attention” values pop.
        let bg = Color::Rgb(0x0b, 0x0f, 0x19); // deep navy
        let slate = Color::Rgb(0x3b, 0x42, 0x5a);
        let cyan = Color::Rgb(0x22, 0xd3, 0xee);
        let violet = Color::Rgb(0x8b, 0x5c, 0xf6);
        let fuchsia = Color::Rgb(0xe8, 0x79, 0xf9);
        let amber = Color::Rgb(0xf5, 0x9e, 0x0b);
        let fg = Color::Rgb(0xf8, 0xfa, 0xfc);

        let green = Color::Rgb(0x22, 0xc5, 0x5e);
        let rose = Color::Rgb(0xfb, 0x71, 0x85);

        Self {
            border: slate,
            title: cyan,
            key: fuchsia,
            dim: Color::Rgb(0x6b, 0x72, 0x80),
            selection_bg: violet,
            selection_fg: fg,
            selection_mod: Modifier::BOLD,
            success: green,
            error: rose,
            warning: amber,
            modal_bg: bg,
        }
    }

    fn border_style(&self) -> Style {
        Style::default().fg(self.border)
    }

    fn title_style(&self) -> Style {
        Style::default().fg(self.title).add_modifier(Modifier::BOLD)
    }

    fn inactive_title_style(&self) -> Style {
        Style::default().fg(self.dim).add_modifier(Modifier::BOLD)
    }

    fn key_style(&self) -> Style {
        Style::default().fg(self.key).add_modifier(Modifier::BOLD)
    }

    fn dim_style(&self) -> Style {
        Style::default().fg(self.dim)
    }

    fn placeholder_style(&self) -> Style {
        self.dim_style().add_modifier(Modifier::DIM)
    }

    fn selected_style(&self) -> Style {
        Style::default()
            .fg(self.selection_fg)
            .bg(self.selection_bg)
            .add_modifier(self.selection_mod)
    }

    fn selected_row_style(&self) -> Style {
        Style::default()
            .bg(self.selection_bg)
            .add_modifier(self.selection_mod)
    }

    fn focused_border_style(&self) -> Style {
        Style::default().fg(self.title)
    }

    fn success_fg_style(&self) -> Style {
        Style::default()
            .fg(self.success)
            .add_modifier(Modifier::BOLD)
    }

    fn success_style(&self) -> Style {
        Style::default()
            .fg(self.success)
            .add_modifier(Modifier::BOLD)
    }

    fn error_style(&self) -> Style {
        Style::default().fg(self.error).add_modifier(Modifier::BOLD)
    }

    fn warning_fg_style(&self) -> Style {
        Style::default()
            .fg(self.warning)
            .add_modifier(Modifier::BOLD)
    }

    fn warning_style(&self) -> Style {
        Style::default()
            .fg(self.warning)
            .add_modifier(Modifier::BOLD)
    }

    fn modal_style(&self) -> Style {
        Style::default().bg(self.modal_bg)
    }
}

fn theme() -> &'static Theme {
    static THEME: OnceLock<Theme> = OnceLock::new();
    THEME.get_or_init(|| {
        let term = std::env::var("TERM").unwrap_or_default();
        let no_color = std::env::var_os("NO_COLOR").is_some() || term == "dumb";
        if no_color {
            return Theme::plain();
        }

        match std::env::var("DROIDGEAR_TUI_THEME").ok().as_deref() {
            Some("plain") | Some("none") | Some("no-color") => Theme::plain(),
            Some("nord") => Theme::nord(),
            Some("opencode") | Some("default") | None => Theme::opencode(),
            Some(_) => Theme::opencode(),
        }
    })
}

fn block<'a>(title: impl Into<ratatui::widgets::block::Title<'a>>) -> Block<'a> {
    let t = theme();
    Block::default()
        .borders(Borders::ALL)
        .padding(Padding::horizontal(1))
        .title(title)
        .border_style(t.border_style())
        .title_style(t.title_style())
}

fn block_focus<'a>(
    title: impl Into<ratatui::widgets::block::Title<'a>>,
    focused: bool,
) -> Block<'a> {
    let t = theme();
    if focused {
        block(title)
            .border_style(t.focused_border_style())
            .title_style(t.title_style())
    } else {
        block(title).title_style(t.inactive_title_style())
    }
}

fn help_block() -> Block<'static> {
    Block::default()
        .borders(Borders::NONE)
        .padding(Padding::horizontal(1))
}

fn help_line(text: &str) -> Line<'static> {
    let t = theme();
    let mut spans: Vec<Span<'static>> = Vec::new();
    for (i, seg) in text
        .split("  ")
        .filter(|s| !s.trim().is_empty())
        .enumerate()
    {
        if i > 0 {
            spans.push(Span::raw("  "));
        }
        if let Some((keys, desc)) = seg.split_once(':') {
            spans.push(Span::styled(keys.trim().to_string(), t.key_style()));
            spans.push(Span::raw(": "));
            spans.push(Span::raw(desc.trim().to_string()));
        } else {
            spans.push(Span::raw(seg.to_string()));
        }
    }
    Line::from(spans)
}

fn hint_line(text: &str) -> Line<'static> {
    let t = theme();
    let mut spans: Vec<Span<'static>> = Vec::new();
    for (i, seg) in text
        .split("  ")
        .filter(|s| !s.trim().is_empty())
        .enumerate()
    {
        if i > 0 {
            spans.push(Span::styled("  ".to_string(), t.dim_style()));
        }
        if let Some((keys, desc)) = seg.split_once(':') {
            spans.push(Span::styled(keys.trim().to_string(), t.key_style()));
            spans.push(Span::styled(": ".to_string(), t.dim_style()));
            spans.push(Span::styled(desc.trim().to_string(), t.dim_style()));
        } else {
            spans.push(Span::styled(seg.to_string(), t.dim_style()));
        }
    }
    Line::from(spans)
}

fn help_paragraph(text: &str) -> Paragraph<'static> {
    let t = theme();
    Paragraph::new(vec![help_line(text)])
        .style(t.dim_style())
        .block(help_block())
        .wrap(Wrap { trim: true })
}

fn byte_index_for_char(value: &str, char_idx: usize) -> usize {
    value
        .char_indices()
        .nth(char_idx)
        .map(|(i, _)| i)
        .unwrap_or(value.len())
}

fn not_set_if_blank(value: &str) -> String {
    if value.trim().is_empty() {
        "(not set)".to_string()
    } else {
        value.to_string()
    }
}

fn is_placeholder_value(value: &str) -> bool {
    let v = value.trim();
    v.is_empty() || v == "-" || (v.starts_with('(') && v.ends_with(')'))
}

fn value_style_for(value: &str) -> Style {
    let t = theme();
    let v_raw = value.trim();
    let v = v_raw.to_ascii_lowercase();
    match v.as_str() {
        "yes" | "on" | "enabled" => t.success_fg_style(),
        "no" | "off" | "disabled" => t.warning_fg_style(),
        "(not set)" | "(unset)" | "(missing)" => t.warning_style(),
        _ if is_placeholder_value(value) => t.placeholder_style(),
        _ => Style::default(),
    }
}

fn field_line_custom(
    label: &str,
    value: &str,
    label_width: usize,
    custom_value_style: Option<Style>,
) -> Line<'static> {
    let t = theme();
    Line::from(vec![
        Span::styled(
            format!("{label:>label_width$}:", label_width = label_width),
            t.dim_style(),
        ),
        Span::raw(" "),
        Span::styled(
            value.to_string(),
            custom_value_style.unwrap_or_else(|| value_style_for(value)),
        ),
    ])
}

fn field_line(label: &str, value: &str, label_width: usize) -> Line<'static> {
    field_line_custom(label, value, label_width, None)
}

fn render_list<'a>(frame: &mut Frame, list: List<'a>, area: Rect, selected: Option<usize>) {
    let mut state = ListState::default();
    state.select(selected);
    frame.render_stateful_widget(list, area, &mut state);
}

pub fn draw(frame: &mut Frame, app: &app::App) {
    let area = frame.area();
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Length(18), Constraint::Min(0)].as_ref())
        .split(area);

    draw_nav(frame, app, chunks[0]);
    draw_main(frame, app, chunks[1]);

    if let Some(modal) = app.modal.as_ref() {
        draw_modal(frame, modal);
    } else if let Some(toast) = app.toast.as_ref() {
        draw_toast(frame, toast);
    }
}

fn draw_nav(frame: &mut Frame, app: &app::App, area: Rect) {
    let t = theme();
    let items: Vec<ListItem> = app::App::nav_items()
        .iter()
        .map(|(label, _)| ListItem::new(Line::from(*label)))
        .collect();

    let selected = (app.screen == app::Screen::Main).then_some(app.nav_index);
    let list = List::new(items)
        .block(block("DroidGear"))
        .highlight_style(t.selected_row_style());
    render_list(frame, list, area, selected);
}

fn draw_main(frame: &mut Frame, app: &app::App, area: Rect) {
    match app.screen {
        app::Screen::Main => draw_home(frame, area),
        app::Screen::Paths => draw_paths(frame, app, area),
        app::Screen::Factory => draw_factory(frame, app, area),
        app::Screen::FactoryModel => draw_factory_model(frame, app, area),
        app::Screen::Mcp => draw_mcp(frame, app, area),
        app::Screen::McpServer => draw_mcp_server(frame, app, area),
        app::Screen::McpArgs => draw_mcp_args(frame, app, area),
        app::Screen::McpKeyValues => draw_mcp_key_values(frame, app, area),
        app::Screen::Codex => draw_codex_profiles(frame, app, area),
        app::Screen::CodexProfile => draw_codex_profile(frame, app, area),
        app::Screen::CodexProvider => draw_codex_provider(frame, app, area),
        app::Screen::OpenCode => draw_opencode_profiles(frame, app, area),
        app::Screen::OpenCodeProfile => draw_opencode_profile(frame, app, area),
        app::Screen::OpenCodeProvider => draw_opencode_provider(frame, app, area),
        app::Screen::OpenCodeModel => draw_opencode_model(frame, app, area),
        app::Screen::OpenClaw => draw_openclaw_profiles(frame, app, area),
        app::Screen::OpenClawProfile => draw_openclaw_profile(frame, app, area),
        app::Screen::OpenClawProvider => draw_openclaw_provider(frame, app, area),
        app::Screen::OpenClawModel => draw_openclaw_model(frame, app, area),
        app::Screen::OpenClawHelpers => draw_openclaw_helpers(frame, app, area),
        app::Screen::OpenClawSubagents => draw_openclaw_subagents(frame, app, area),
        app::Screen::OpenClawSubagentDetail => draw_openclaw_subagent_detail(frame, app, area),
        app::Screen::Hermes => draw_hermes_profiles(frame, app, area),
        app::Screen::HermesProfile => draw_hermes_profile(frame, app, area),
        app::Screen::HermesProvider => draw_hermes_provider(frame, app, area),
        app::Screen::Sessions => draw_sessions(frame, app, area),
        app::Screen::Specs => draw_specs(frame, app, area),
        app::Screen::Channels => draw_channels(frame, app, area),
        app::Screen::ChannelsEdit => draw_channels_edit(frame, app, area),
        app::Screen::Missions => draw_missions(frame, app, area),
    }
}

fn draw_home(frame: &mut Frame, area: Rect) {
    let text = vec![
        help_line("Enter: open module"),
        help_line("s: module picker"),
        help_line("Up/Down: navigate"),
        help_line("q: quit"),
    ];
    let p = Paragraph::new(text)
        .block(block("Home"))
        .wrap(Wrap { trim: true });
    frame.render_widget(p, area);
}

fn draw_paths(frame: &mut Frame, app: &app::App, area: Rect) {
    let t = theme();
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(0), Constraint::Length(2)].as_ref())
        .split(area);

    let mut lines: Vec<Line> = Vec::new();
    if let Some(paths) = app.paths.as_ref() {
        let entries = [
            &paths.factory,
            &paths.opencode,
            &paths.opencode_auth,
            &paths.codex,
            &paths.openclaw,
            &paths.hermes,
        ];
        for (i, p) in entries.iter().enumerate() {
            let selected = i == app.paths_index;
            let default_tag = if p.is_default { "default" } else { "custom" };
            let key_style = if selected {
                t.selected_style()
            } else {
                t.dim_style()
            };
            let path_style = if selected {
                t.selected_style()
            } else {
                Style::default()
            };
            let tag_style = if selected {
                t.selected_style()
            } else if p.is_default {
                t.placeholder_style()
            } else {
                t.key_style()
            };
            lines.push(Line::from(vec![
                Span::styled(format!("{:>10}: ", p.key), key_style),
                Span::styled(p.path.clone(), path_style),
                Span::styled(format!("  [{default_tag}]"), tag_style),
            ]));
        }
    } else {
        lines.push(Line::from(Span::styled(
            "Failed to load paths",
            t.error_style(),
        )));
    }

    let p = Paragraph::new(lines)
        .block(block("Paths"))
        .wrap(Wrap { trim: false });
    frame.render_widget(p, chunks[0]);

    let help = help_paragraph("Enter/e: edit  x: reset  r: refresh  q/Esc: back");
    frame.render_widget(help, chunks[1]);
}

fn draw_factory(frame: &mut Frame, app: &app::App, area: Rect) {
    let t = theme();
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(0), Constraint::Length(2)].as_ref())
        .split(area);

    let mut items: Vec<ListItem> = Vec::new();
    for (i, m) in app.custom_models.iter().enumerate() {
        let selected = i == app.factory_models_index;
        let name = m
            .display_name
            .as_deref()
            .filter(|s| !s.trim().is_empty())
            .unwrap_or(&m.model);
        let id =
            m.id.as_deref()
                .filter(|s| !s.trim().is_empty())
                .unwrap_or("-");
        let is_default = app
            .factory_default_model_id
            .as_deref()
            .is_some_and(|d| d == id);
        if selected {
            let default_tag = if is_default { " *" } else { "" };
            items.push(ListItem::new(Line::from(format!(
                "{name}  ({id}){default_tag}"
            ))));
        } else {
            let mut spans = Vec::new();
            spans.push(Span::raw(name.to_string()));
            spans.push(Span::raw("  "));
            spans.push(Span::styled(format!("({id})"), t.dim_style()));
            if is_default {
                spans.push(Span::styled(" *".to_string(), t.success_style()));
            }
            items.push(ListItem::new(Line::from(spans)));
        }
    }

    if items.is_empty() {
        items.push(ListItem::new(Line::from(Span::styled(
            "No custom models",
            t.placeholder_style(),
        ))));
    }

    let title = match app.factory_default_model_id.as_deref() {
        Some(id) => format!("Factory (default model: {id})"),
        None => "Factory (default model: -)".to_string(),
    };

    let selected = (!app.custom_models.is_empty()).then_some(app.factory_models_index);
    let list = List::new(items)
        .block(block(title))
        .highlight_style(t.selected_row_style());
    render_list(frame, list, chunks[0], selected);

    let help = help_paragraph(
        "Up/Down: select  Enter/e: open  n: new  c: copy  x: delete  d: set default  E: raw edit  r: refresh  q/Esc: back",
    );
    frame.render_widget(help, chunks[1]);
}

fn draw_factory_model(frame: &mut Frame, app: &app::App, area: Rect) {
    let t = theme();
    let Some(draft) = app.factory_draft.as_ref() else {
        let p = Paragraph::new(vec![Line::from(Span::styled(
            "No model loaded",
            t.warning_style(),
        ))])
        .block(block("Factory Model"))
        .wrap(Wrap { trim: true });
        frame.render_widget(p, area);
        return;
    };

    let provider = match draft.provider {
        droidgear_core::factory_settings::Provider::Anthropic => "anthropic",
        droidgear_core::factory_settings::Provider::Openai => "openai",
        droidgear_core::factory_settings::Provider::GenericChatCompletionApi => {
            "generic-chat-completion-api"
        }
    };
    let api_key_set = !draft.api_key.trim().is_empty();
    let max_tokens = draft
        .max_output_tokens
        .map(|v| v.to_string())
        .unwrap_or_else(|| "(default)".to_string());
    let no_image_support = draft.no_image_support.unwrap_or(false);
    let display_name = draft.display_name.clone().unwrap_or_default();

    let title = if draft.model.trim().is_empty() {
        "Factory Model: (new)".to_string()
    } else {
        format!("Factory Model: {}", draft.model)
    };

    let fields: Vec<(&str, String)> = vec![
        ("Provider", provider.to_string()),
        ("Base URL", draft.base_url.clone()),
        (
            "API Key",
            if api_key_set {
                "********".to_string()
            } else {
                "(not set)".to_string()
            },
        ),
        ("Model", not_set_if_blank(&draft.model)),
        (
            "Display Name",
            if display_name.trim().is_empty() {
                "(none)".to_string()
            } else {
                display_name
            },
        ),
        ("Max Tokens", max_tokens),
        (
            "No Image Support",
            if no_image_support {
                "yes".to_string()
            } else {
                "no".to_string()
            },
        ),
        (
            "Reasoning Effort",
            draft
                .extra_args
                .as_ref()
                .and_then(|m| m.get("reasoning"))
                .and_then(|v| v.as_object())
                .and_then(|obj| obj.get("effort"))
                .and_then(|v| v.as_str())
                .map(|s| s.to_string())
                .unwrap_or_else(|| "(none)".to_string()),
        ),
        (
            "Extra Args",
            draft
                .extra_args
                .as_ref()
                .map(|m| {
                    if m.is_empty() {
                        "(none)".to_string()
                    } else {
                        format!("{} entries", m.len())
                    }
                })
                .unwrap_or_else(|| "(none)".to_string()),
        ),
        (
            "Extra Headers",
            draft
                .extra_headers
                .as_ref()
                .map(|m| {
                    if m.is_empty() {
                        "(none)".to_string()
                    } else {
                        format!("{} entries", m.len())
                    }
                })
                .unwrap_or_else(|| "(none)".to_string()),
        ),
    ];

    let mut items: Vec<ListItem> = Vec::new();
    for (i, (label, value)) in fields.into_iter().enumerate() {
        let selected = i == app.factory_model_field_index;
        let line = if selected {
            Line::from(format!("{label:>16}: {value}"))
        } else {
            let custom_style = match label {
                "API Key" if value == "********" => Some(t.success_fg_style()),
                "No Image Support" => Some(if value.trim().eq_ignore_ascii_case("yes") {
                    t.warning_fg_style()
                } else {
                    t.success_fg_style()
                }),
                _ => None,
            };
            field_line_custom(label, &value, 16, custom_style)
        };
        items.push(ListItem::new(line));
    }

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(0), Constraint::Length(2)].as_ref())
        .split(area);

    let list = List::new(items)
        .block(block(title))
        .highlight_style(t.selected_row_style());
    render_list(frame, list, chunks[0], Some(app.factory_model_field_index));

    let help = help_paragraph("Up/Down: select  Enter/e: edit/toggle  s: save  q/Esc: back");
    frame.render_widget(help, chunks[1]);
}

fn draw_mcp(frame: &mut Frame, app: &app::App, area: Rect) {
    let t = theme();
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(0), Constraint::Length(2)].as_ref())
        .split(area);

    let mut items: Vec<ListItem> = Vec::new();
    for (i, s) in app.mcp_servers.iter().enumerate() {
        let selected = i == app.mcp_index;
        let status = if s.config.disabled {
            "disabled"
        } else {
            "enabled"
        };
        if selected {
            items.push(ListItem::new(Line::from(format!("{}  [{status}]", s.name))));
        } else {
            let status_style = if s.config.disabled {
                t.warning_fg_style()
            } else {
                t.success_fg_style()
            };
            items.push(ListItem::new(Line::from(vec![
                Span::raw(s.name.clone()),
                Span::raw("  "),
                Span::styled(format!("[{status}]"), status_style),
            ])));
        }
    }
    if items.is_empty() {
        items.push(ListItem::new(Line::from(Span::styled(
            "No MCP servers",
            t.placeholder_style(),
        ))));
    }

    let selected = (!app.mcp_servers.is_empty()).then_some(app.mcp_index);
    let list = List::new(items)
        .block(block("MCP"))
        .highlight_style(t.selected_row_style());
    render_list(frame, list, chunks[0], selected);

    let help = help_paragraph(
        "Up/Down: select  Enter/e: open  n: new  t: toggle  d: delete  r: refresh  q/Esc: back",
    );
    frame.render_widget(help, chunks[1]);
}

fn draw_mcp_server(frame: &mut Frame, app: &app::App, area: Rect) {
    let t = theme();
    let Some(server) = app.mcp_edit_draft.as_ref() else {
        let p = Paragraph::new(vec![Line::from(Span::styled(
            "No server loaded",
            t.warning_style(),
        ))])
        .block(block("MCP Server"))
        .wrap(Wrap { trim: true });
        frame.render_widget(p, area);
        return;
    };

    let disabled = if server.config.disabled { "yes" } else { "no" };
    let server_type = match server.config.server_type {
        droidgear_core::mcp::McpServerType::Stdio => "stdio",
        droidgear_core::mcp::McpServerType::Http => "http",
    };

    let mut fields: Vec<(&str, String)> = vec![
        ("Name", not_set_if_blank(&server.name)),
        ("Type", server_type.to_string()),
        ("Disabled", disabled.to_string()),
    ];

    match server.config.server_type {
        droidgear_core::mcp::McpServerType::Stdio => {
            let args_count = server.config.args.as_ref().map(|v| v.len()).unwrap_or(0);
            let env_count = server.config.env.as_ref().map(|m| m.len()).unwrap_or(0);
            fields.push((
                "Command",
                not_set_if_blank(server.config.command.as_deref().unwrap_or("")),
            ));
            fields.push(("Args", format!("{args_count}")));
            fields.push(("Env", format!("{env_count}")));
        }
        droidgear_core::mcp::McpServerType::Http => {
            let headers_count = server.config.headers.as_ref().map(|m| m.len()).unwrap_or(0);
            fields.push((
                "URL",
                not_set_if_blank(server.config.url.as_deref().unwrap_or("")),
            ));
            fields.push(("Headers", format!("{headers_count}")));
        }
    }

    let mut items: Vec<ListItem> = Vec::new();
    for (i, (label, value)) in fields.into_iter().enumerate() {
        let selected = i == app.mcp_edit_field_index;
        let line = if selected {
            Line::from(format!("{label:>16}: {value}"))
        } else {
            let custom_style = match label {
                "Disabled" => Some(if value.trim() == "yes" {
                    t.warning_fg_style()
                } else {
                    t.success_fg_style()
                }),
                _ => None,
            };
            field_line_custom(label, &value, 16, custom_style)
        };
        items.push(ListItem::new(line));
    }

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(0), Constraint::Length(2)].as_ref())
        .split(area);

    let title = if server.name.trim().is_empty() {
        "MCP Server: (new)".to_string()
    } else {
        format!("MCP Server: {}", server.name)
    };
    let list = List::new(items)
        .block(block(title))
        .highlight_style(t.selected_row_style());
    render_list(frame, list, chunks[0], Some(app.mcp_edit_field_index));

    let help = help_paragraph("Up/Down: select  Enter/e: edit/open/toggle  s: save  q/Esc: back");
    frame.render_widget(help, chunks[1]);
}

fn draw_mcp_args(frame: &mut Frame, app: &app::App, area: Rect) {
    let t = theme();
    let args = app
        .mcp_edit_draft
        .as_ref()
        .and_then(|s| s.config.args.as_ref())
        .cloned()
        .unwrap_or_default();

    let mut items: Vec<ListItem> = Vec::new();
    for a in args.iter() {
        items.push(ListItem::new(Line::from(Span::raw(a.clone()))));
    }
    if items.is_empty() {
        items.push(ListItem::new(Line::from(Span::styled(
            "No args",
            t.placeholder_style(),
        ))));
    }

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(0), Constraint::Length(2)].as_ref())
        .split(area);

    let selected = (!args.is_empty()).then_some(app.mcp_args_index);
    let list = List::new(items)
        .block(block("MCP Args"))
        .highlight_style(t.selected_row_style());
    render_list(frame, list, chunks[0], selected);

    let help = help_paragraph("Up/Down: select  n: add  Enter/e: edit  x: delete  q/Esc: back");
    frame.render_widget(help, chunks[1]);
}

fn draw_mcp_key_values(frame: &mut Frame, app: &app::App, area: Rect) {
    let t = theme();
    let Some(server) = app.mcp_edit_draft.as_ref() else {
        let p = Paragraph::new(vec![Line::from(Span::styled(
            "No server loaded",
            t.warning_style(),
        ))])
        .block(block("MCP"))
        .wrap(Wrap { trim: true });
        frame.render_widget(p, area);
        return;
    };

    let (title, map) = match app.mcp_kv_mode {
        app::McpKeyValuesMode::Env => ("MCP Env", server.config.env.as_ref()),
        app::McpKeyValuesMode::Headers => ("MCP Headers", server.config.headers.as_ref()),
    };
    let mut entries: Vec<(String, String)> = map
        .map(|m| m.iter().map(|(k, v)| (k.clone(), v.clone())).collect())
        .unwrap_or_default();
    entries.sort_by_key(|a| a.0.to_lowercase());

    let mut items: Vec<ListItem> = Vec::new();
    for (i, (k, v)) in entries.iter().enumerate() {
        let selected = i == app.mcp_kv_index;
        if selected {
            items.push(ListItem::new(Line::from(format!("{k}={v}"))));
        } else {
            items.push(ListItem::new(Line::from(vec![
                Span::styled(k.clone(), t.dim_style()),
                Span::raw("="),
                Span::styled(v.clone(), value_style_for(v)),
            ])));
        }
    }
    if items.is_empty() {
        items.push(ListItem::new(Line::from(Span::styled(
            "No entries",
            t.placeholder_style(),
        ))));
    }

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(0), Constraint::Length(2)].as_ref())
        .split(area);

    let selected = (!entries.is_empty()).then_some(app.mcp_kv_index);
    let list = List::new(items)
        .block(block(title))
        .highlight_style(t.selected_row_style());
    render_list(frame, list, chunks[0], selected);

    let help = help_paragraph("Up/Down: select  n: add  Enter/e: edit  x: delete  q/Esc: back");
    frame.render_widget(help, chunks[1]);
}

fn draw_codex_profiles(frame: &mut Frame, app: &app::App, area: Rect) {
    let active = app.codex_active_id.as_deref();
    let selected_index = app.codex_index;
    draw_profile_list(
        frame,
        area,
        "Codex Profiles",
        app.codex_profiles
            .iter()
            .map(|p| (p.name.as_str(), p.id.as_str())),
        active,
        selected_index,
        "Up/Down: select  Enter/e: open  E: raw edit  p: preview  a: apply  n: new  c: copy  d: delete  r: refresh  q/Esc: back",
    );
}

fn draw_opencode_profiles(frame: &mut Frame, app: &app::App, area: Rect) {
    let active = app.opencode_active_id.as_deref();
    let selected_index = app.opencode_index;
    draw_profile_list(
        frame,
        area,
        "OpenCode Profiles",
        app.opencode_profiles
            .iter()
            .map(|p| (p.name.as_str(), p.id.as_str())),
        active,
        selected_index,
        "Up/Down: select  Enter/e: open  E: raw edit  p: preview  a: apply  n: new  c: copy  d: delete  r: refresh  q/Esc: back",
    );
}

fn draw_openclaw_profiles(frame: &mut Frame, app: &app::App, area: Rect) {
    let active = app.openclaw_active_id.as_deref();
    let selected_index = app.openclaw_index;
    draw_profile_list(
        frame,
        area,
        "OpenClaw Profiles",
        app.openclaw_profiles
            .iter()
            .map(|p| (p.name.as_str(), p.id.as_str())),
        active,
        selected_index,
        "Up/Down: select  Enter/e: open  E: raw edit  p: preview  a: apply  n: new  c: copy  d: delete  r: refresh  q/Esc: back",
    );
}

fn draw_openclaw_profile(frame: &mut Frame, app: &app::App, area: Rect) {
    let t = theme();
    let Some(profile) = app.openclaw_detail.as_ref() else {
        let p = Paragraph::new(vec![Line::from(Span::styled(
            "Failed to load profile",
            t.error_style(),
        ))])
        .block(block("OpenClaw Profile"))
        .wrap(Wrap { trim: true });
        frame.render_widget(p, area);
        return;
    };

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints(
            [
                Constraint::Percentage(28),
                Constraint::Percentage(26),
                Constraint::Min(0),
                Constraint::Length(2),
            ]
            .as_ref(),
        )
        .split(area);

    let fields: Vec<(&str, String)> = vec![
        ("Name", profile.name.clone()),
        (
            "Description",
            profile
                .description
                .clone()
                .unwrap_or_else(|| "".to_string()),
        ),
        (
            "Default Model",
            profile
                .default_model
                .clone()
                .unwrap_or_else(|| "(none)".to_string()),
        ),
    ];

    let mut field_items: Vec<ListItem> = Vec::new();
    for (i, (label, value)) in fields.into_iter().enumerate() {
        let selected = app.openclaw_detail_focus == app::OpenClawProfileFocus::Fields
            && i == app.openclaw_detail_field_index;
        let line = if selected {
            Line::from(format!("{label:>14}: {value}"))
        } else {
            field_line(label, &value, 14)
        };
        field_items.push(ListItem::new(line));
    }
    let field_selected = (app.openclaw_detail_focus == app::OpenClawProfileFocus::Fields)
        .then_some(app.openclaw_detail_field_index);
    let field_list = List::new(field_items)
        .block(block_focus(
            format!("OpenClaw Profile: {}", profile.name),
            app.openclaw_detail_focus == app::OpenClawProfileFocus::Fields,
        ))
        .highlight_style(t.selected_row_style());
    render_list(frame, field_list, chunks[0], field_selected);

    let failovers = profile.failover_models.as_deref().unwrap_or(&[]);
    let mut failover_items: Vec<ListItem> = Vec::new();
    for r in failovers.iter() {
        failover_items.push(ListItem::new(Line::from(Span::raw(r.clone()))));
    }
    if failover_items.is_empty() {
        failover_items.push(ListItem::new(Line::from(Span::styled(
            "No failover models",
            t.placeholder_style(),
        ))));
    }
    let failover_list = List::new(failover_items).block(block_focus(
        "Failover Models (Tab to focus)",
        app.openclaw_detail_focus == app::OpenClawProfileFocus::Failover,
    ));
    let failover_selected = (app.openclaw_detail_focus == app::OpenClawProfileFocus::Failover
        && !failovers.is_empty())
    .then_some(app.openclaw_detail_failover_index);
    let failover_list = failover_list.highlight_style(t.selected_row_style());
    render_list(frame, failover_list, chunks[1], failover_selected);

    let mut provider_items: Vec<ListItem> = Vec::new();
    for pid in app.openclaw_detail_provider_ids.iter() {
        provider_items.push(ListItem::new(Line::from(Span::raw(pid.clone()))));
    }
    if provider_items.is_empty() {
        provider_items.push(ListItem::new(Line::from(Span::styled(
            "No providers",
            t.placeholder_style(),
        ))));
    }
    let provider_list = List::new(provider_items).block(block_focus(
        "Providers (Tab to focus)",
        app.openclaw_detail_focus == app::OpenClawProfileFocus::Providers,
    ));
    let provider_selected = (app.openclaw_detail_focus == app::OpenClawProfileFocus::Providers
        && !app.openclaw_detail_provider_ids.is_empty())
    .then_some(app.openclaw_detail_provider_index);
    let provider_list = provider_list.highlight_style(t.selected_row_style());
    render_list(frame, provider_list, chunks[2], provider_selected);

    let help = help_paragraph(
        "Tab: switch  Up/Down: move  Enter/e: edit/open  n: add  d: delete  l: load live  h: helpers  p: preview  a: apply  q/Esc: back",
    );
    frame.render_widget(help, chunks[3]);
}

fn draw_openclaw_provider(frame: &mut Frame, app: &app::App, area: Rect) {
    let t = theme();
    let Some(profile) = app.openclaw_detail.as_ref() else {
        let p = Paragraph::new(vec![Line::from(Span::styled(
            "Failed to load profile",
            t.error_style(),
        ))])
        .block(block("OpenClaw Provider"))
        .wrap(Wrap { trim: true });
        frame.render_widget(p, area);
        return;
    };
    let Some(provider_id) = app.openclaw_provider_id.as_deref() else {
        let p = Paragraph::new(vec![Line::from(Span::styled(
            "No provider selected",
            t.warning_style(),
        ))])
        .block(block("OpenClaw Provider"))
        .wrap(Wrap { trim: true });
        frame.render_widget(p, area);
        return;
    };
    let Some(config) = profile.providers.get(provider_id) else {
        let p = Paragraph::new(vec![Line::from(Span::styled(
            "Provider not found",
            t.error_style(),
        ))])
        .block(block("OpenClaw Provider"))
        .wrap(Wrap { trim: true });
        frame.render_widget(p, area);
        return;
    };

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints(
            [
                Constraint::Percentage(45),
                Constraint::Min(0),
                Constraint::Length(2),
            ]
            .as_ref(),
        )
        .split(area);

    let api_key_set = config
        .api_key
        .as_deref()
        .is_some_and(|k| !k.trim().is_empty());
    let api_type = config.api.as_deref().unwrap_or("openai-completions");

    let provider_fields: Vec<(&str, String)> = vec![
        (
            "Base URL",
            config.base_url.clone().unwrap_or_else(|| "".to_string()),
        ),
        (
            "API Key",
            if api_key_set {
                "********".to_string()
            } else {
                "(not set)".to_string()
            },
        ),
        ("API Type", api_type.to_string()),
    ];

    let mut field_items: Vec<ListItem> = Vec::new();
    for (i, (label, value)) in provider_fields.into_iter().enumerate() {
        let selected = app.openclaw_provider_focus == app::CodexDetailFocus::Fields
            && i == app.openclaw_provider_field_index;
        let line = if selected {
            Line::from(format!("{label:>14}: {value}"))
        } else {
            let custom_style = match label {
                "API Key" if value == "********" => Some(t.success_fg_style()),
                _ => None,
            };
            field_line_custom(label, &value, 14, custom_style)
        };
        field_items.push(ListItem::new(line));
    }
    let fields_list = List::new(field_items).block(block_focus(
        format!("OpenClaw Provider: {provider_id}"),
        app.openclaw_provider_focus == app::CodexDetailFocus::Fields,
    ));
    let fields_selected = (app.openclaw_provider_focus == app::CodexDetailFocus::Fields)
        .then_some(app.openclaw_provider_field_index);
    let fields_list = fields_list.highlight_style(t.selected_row_style());
    render_list(frame, fields_list, chunks[0], fields_selected);

    let mut model_items: Vec<ListItem> = Vec::new();
    for m in config.models.iter() {
        model_items.push(ListItem::new(Line::from(Span::raw(m.id.clone()))));
    }
    if model_items.is_empty() {
        model_items.push(ListItem::new(Line::from(Span::styled(
            "No models",
            t.placeholder_style(),
        ))));
    }
    let models_list = List::new(model_items).block(block_focus(
        "Models (Tab to focus)",
        app.openclaw_provider_focus == app::CodexDetailFocus::Providers,
    ));
    let models_selected = (app.openclaw_provider_focus == app::CodexDetailFocus::Providers
        && !config.models.is_empty())
    .then_some(app.openclaw_provider_model_index);
    let models_list = models_list.highlight_style(t.selected_row_style());
    render_list(frame, models_list, chunks[1], models_selected);

    let help = help_paragraph(
        "Tab: switch  Up/Down: move  Enter/e: edit/open  n: add model  d: delete model  q/Esc: back",
    );
    frame.render_widget(help, chunks[2]);
}

fn draw_openclaw_model(frame: &mut Frame, app: &app::App, area: Rect) {
    let t = theme();
    let Some(profile) = app.openclaw_detail.as_ref() else {
        let p = Paragraph::new(vec![Line::from(Span::styled(
            "Failed to load profile",
            t.error_style(),
        ))])
        .block(block("OpenClaw Model"))
        .wrap(Wrap { trim: true });
        frame.render_widget(p, area);
        return;
    };
    let Some(provider_id) = app.openclaw_provider_id.as_deref() else {
        let p = Paragraph::new(vec![Line::from(Span::styled(
            "No provider selected",
            t.warning_style(),
        ))])
        .block(block("OpenClaw Model"))
        .wrap(Wrap { trim: true });
        frame.render_widget(p, area);
        return;
    };
    let Some(provider) = profile.providers.get(provider_id) else {
        let p = Paragraph::new(vec![Line::from(Span::styled(
            "Provider not found",
            t.error_style(),
        ))])
        .block(block("OpenClaw Model"))
        .wrap(Wrap { trim: true });
        frame.render_widget(p, area);
        return;
    };
    let Some(model) = provider.models.get(app.openclaw_provider_model_index) else {
        let p = Paragraph::new(vec![Line::from(Span::styled(
            "Model not found",
            t.error_style(),
        ))])
        .block(block("OpenClaw Model"))
        .wrap(Wrap { trim: true });
        frame.render_widget(p, area);
        return;
    };

    let input_text = model.input.iter().any(|t| t == "text");
    let input_image = model.input.iter().any(|t| t == "image");

    let fields: Vec<(&str, String)> = vec![
        ("Model ID", model.id.clone()),
        ("Name", model.name.clone().unwrap_or_else(|| "".to_string())),
        (
            "Context Window",
            model
                .context_window
                .map(|v| v.to_string())
                .unwrap_or_default(),
        ),
        (
            "Max Tokens",
            model.max_tokens.map(|v| v.to_string()).unwrap_or_default(),
        ),
        (
            "Reasoning",
            if model.reasoning { "on" } else { "off" }.to_string(),
        ),
        (
            "Input Text",
            if input_text { "on" } else { "off" }.to_string(),
        ),
        (
            "Input Image",
            if input_image { "on" } else { "off" }.to_string(),
        ),
    ];

    let mut items: Vec<ListItem> = Vec::new();
    for (i, (label, value)) in fields.into_iter().enumerate() {
        let selected = i == app.openclaw_model_field_index;
        let line = if selected {
            Line::from(format!("{label:>14}: {value}"))
        } else {
            field_line(label, &value, 14)
        };
        items.push(ListItem::new(line));
    }

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(0), Constraint::Length(2)].as_ref())
        .split(area);

    let list = List::new(items)
        .block(block("OpenClaw Model"))
        .highlight_style(t.selected_row_style());
    render_list(frame, list, chunks[0], Some(app.openclaw_model_field_index));

    let help = help_paragraph("Up/Down: select  Enter/e: edit/toggle  q/Esc: back");
    frame.render_widget(help, chunks[1]);
}

fn draw_openclaw_helpers(frame: &mut Frame, app: &app::App, area: Rect) {
    let t = theme();
    let Some(profile) = app.openclaw_detail.as_ref() else {
        let p = Paragraph::new(vec![Line::from(Span::styled(
            "Failed to load profile",
            t.error_style(),
        ))])
        .block(block("OpenClaw Helpers"))
        .wrap(Wrap { trim: true });
        frame.render_widget(p, area);
        return;
    };

    let cfg = profile.block_streaming_config.as_ref();
    let default_mode = cfg
        .and_then(|c| c.block_streaming_default.as_deref())
        .unwrap_or("on");
    let break_mode = cfg
        .and_then(|c| c.block_streaming_break.as_deref())
        .unwrap_or("text_end");
    let min_chars = cfg
        .and_then(|c| c.block_streaming_chunk.as_ref())
        .and_then(|c| c.min_chars)
        .unwrap_or(200);
    let max_chars = cfg
        .and_then(|c| c.block_streaming_chunk.as_ref())
        .and_then(|c| c.max_chars)
        .unwrap_or(600);
    let idle_ms = cfg
        .and_then(|c| c.block_streaming_coalesce.as_ref())
        .and_then(|c| c.idle_ms)
        .unwrap_or(200);
    let telegram_block = cfg
        .and_then(|c| c.telegram_channel.as_ref())
        .and_then(|t| t.block_streaming)
        .unwrap_or(true);
    let telegram_chunk = cfg
        .and_then(|c| c.telegram_channel.as_ref())
        .and_then(|t| t.chunk_mode.as_deref())
        .unwrap_or("newline");

    let fields: Vec<(&str, String)> = vec![
        ("Default", default_mode.to_string()),
        ("Break", break_mode.to_string()),
        ("Min Chars", min_chars.to_string()),
        ("Max Chars", max_chars.to_string()),
        ("Idle Ms", idle_ms.to_string()),
        (
            "Telegram Block",
            if telegram_block { "on" } else { "off" }.to_string(),
        ),
        ("Telegram Mode", telegram_chunk.to_string()),
    ];

    let mut items: Vec<ListItem> = Vec::new();
    for (i, (label, value)) in fields.into_iter().enumerate() {
        let selected = i == app.openclaw_helpers_field_index;
        let line = if selected {
            Line::from(format!("{label:>14}: {value}"))
        } else {
            field_line(label, &value, 14)
        };
        items.push(ListItem::new(line));
    }

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(0), Constraint::Length(2)].as_ref())
        .split(area);

    let list = List::new(items)
        .block(block("OpenClaw Helpers"))
        .highlight_style(t.selected_row_style());
    render_list(
        frame,
        list,
        chunks[0],
        Some(app.openclaw_helpers_field_index),
    );

    let help =
        help_paragraph("Up/Down: select  Enter/e: edit/toggle  x: reset defaults  q/Esc: back");
    frame.render_widget(help, chunks[1]);
}

fn draw_openclaw_subagents(frame: &mut Frame, app: &app::App, area: Rect) {
    let t = theme();
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(0), Constraint::Length(2)].as_ref())
        .split(area);

    // Derive allowed set from main agent
    let allowed: std::collections::HashSet<String> = app
        .openclaw_subagents
        .iter()
        .find(|a| a.id == "main")
        .and_then(|main| main.subagents.as_ref())
        .and_then(|sa| sa.allow_agents.as_ref())
        .map(|list| list.iter().cloned().collect())
        .unwrap_or_default();

    let non_main: Vec<_> = app
        .openclaw_subagents
        .iter()
        .filter(|a| a.id != "main")
        .collect();

    let mut items: Vec<ListItem> = Vec::new();
    for (i, agent) in non_main.iter().enumerate() {
        let selected = i == app.openclaw_subagents_index;
        let emoji = agent
            .identity
            .as_ref()
            .and_then(|id| id.emoji.as_deref())
            .unwrap_or("");
        let name = agent.name.as_deref().unwrap_or(&agent.id);
        let model = agent
            .model
            .as_ref()
            .and_then(|m| m.primary.as_deref())
            .unwrap_or("(no model)");
        let is_allowed = allowed.contains(&agent.id);
        let status = if is_allowed { "allowed" } else { "disallowed" };

        if selected {
            items.push(ListItem::new(Line::from(format!(
                "{emoji} {name}  ({model})  [{status}]"
            ))));
        } else {
            let status_style = if is_allowed {
                t.success_fg_style()
            } else {
                t.warning_fg_style()
            };
            items.push(ListItem::new(Line::from(vec![
                Span::raw(format!("{emoji} {name}")),
                Span::styled(format!("  ({model})"), t.dim_style()),
                Span::raw("  "),
                Span::styled(format!("[{status}]"), status_style),
            ])));
        }
    }

    if items.is_empty() {
        items.push(ListItem::new(Line::from(Span::styled(
            "No subagents",
            t.placeholder_style(),
        ))));
    }

    let selected = (!non_main.is_empty()).then_some(app.openclaw_subagents_index);
    let list = List::new(items)
        .block(block("OpenClaw Subagents"))
        .highlight_style(t.selected_row_style());
    render_list(frame, list, chunks[0], selected);

    let help = help_paragraph(
        "Up/Down: select  Enter/e: edit  n: new  d: delete  t: toggle allow  r: refresh  q/Esc: back",
    );
    frame.render_widget(help, chunks[1]);
}

fn draw_openclaw_subagent_detail(frame: &mut Frame, app: &app::App, area: Rect) {
    let t = theme();
    let Some(agent) = app.openclaw_subagent_detail.as_ref() else {
        let p = Paragraph::new(vec![Line::from(Span::styled(
            "No subagent loaded",
            t.warning_style(),
        ))])
        .block(block("Subagent"))
        .wrap(Wrap { trim: true });
        frame.render_widget(p, area);
        return;
    };

    let name = agent.name.as_deref().unwrap_or("(none)").to_string();
    let emoji = agent
        .identity
        .as_ref()
        .and_then(|id| id.emoji.as_deref())
        .unwrap_or("(none)")
        .to_string();
    let primary_model = agent
        .model
        .as_ref()
        .and_then(|m| m.primary.as_deref())
        .unwrap_or("(none)")
        .to_string();
    let tools_profile = agent
        .tools
        .as_ref()
        .and_then(|t| t.profile.as_deref())
        .unwrap_or("(none)")
        .to_string();
    let workspace = agent.workspace.as_deref().unwrap_or("(none)").to_string();

    let fields: Vec<(&str, String)> = vec![
        ("Name", name),
        ("Emoji", emoji),
        ("Primary Model", primary_model),
        ("Tools Profile", tools_profile),
        ("Workspace", workspace),
    ];

    let mut items: Vec<ListItem> = Vec::new();
    for (i, (label, value)) in fields.into_iter().enumerate() {
        let selected = i == app.openclaw_subagent_field_index;
        let line = if selected {
            Line::from(format!("{label:>14}: {value}"))
        } else {
            field_line(label, &value, 14)
        };
        items.push(ListItem::new(line));
    }

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(0), Constraint::Length(2)].as_ref())
        .split(area);

    let title = format!("Subagent: {}", agent.id);
    let list = List::new(items)
        .block(block(title))
        .highlight_style(t.selected_row_style());
    render_list(
        frame,
        list,
        chunks[0],
        Some(app.openclaw_subagent_field_index),
    );

    let help = help_paragraph("Up/Down: select  Enter/e: edit  q/Esc: back");
    frame.render_widget(help, chunks[1]);
}

fn draw_opencode_profile(frame: &mut Frame, app: &app::App, area: Rect) {
    let t = theme();
    let Some(profile) = app.opencode_detail.as_ref() else {
        let p = Paragraph::new(vec![Line::from(Span::styled(
            "Failed to load profile",
            t.error_style(),
        ))])
        .block(block("OpenCode Profile"))
        .wrap(Wrap { trim: true });
        frame.render_widget(p, area);
        return;
    };

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints(
            [
                Constraint::Percentage(45),
                Constraint::Min(0),
                Constraint::Length(2),
            ]
            .as_ref(),
        )
        .split(area);

    let fields: Vec<(&str, String)> = vec![
        ("Name", profile.name.clone()),
        (
            "Description",
            profile
                .description
                .clone()
                .unwrap_or_else(|| "".to_string()),
        ),
    ];

    let mut lines: Vec<Line> = Vec::new();
    for (i, (label, value)) in fields.into_iter().enumerate() {
        let selected = app.opencode_detail_focus == app::CodexDetailFocus::Fields
            && i == app.opencode_detail_field_index;
        let label_style = if selected {
            t.selected_style()
        } else {
            t.dim_style()
        };
        let value_style = if selected {
            t.selected_style()
        } else {
            value_style_for(&value)
        };
        lines.push(Line::from(vec![
            Span::styled(format!("{label:>16}: "), label_style),
            Span::styled(value, value_style),
        ]));
    }

    let fields_block = Paragraph::new(lines)
        .block(block_focus(
            format!("OpenCode Profile: {}", profile.name),
            app.opencode_detail_focus == app::CodexDetailFocus::Fields,
        ))
        .wrap(Wrap { trim: false });
    frame.render_widget(fields_block, chunks[0]);

    let mut provider_items: Vec<ListItem> = Vec::new();
    for pid in app.opencode_detail_provider_ids.iter() {
        provider_items.push(ListItem::new(Line::from(Span::raw(pid.clone()))));
    }
    if provider_items.is_empty() {
        provider_items.push(ListItem::new(Line::from(Span::styled(
            "No providers",
            t.placeholder_style(),
        ))));
    }

    let providers_selected = (app.opencode_detail_focus == app::CodexDetailFocus::Providers
        && !app.opencode_detail_provider_ids.is_empty())
    .then_some(app.opencode_detail_provider_index);
    let providers_list = List::new(provider_items)
        .block(block_focus(
            "Providers (Tab to focus)",
            app.opencode_detail_focus == app::CodexDetailFocus::Providers,
        ))
        .highlight_style(t.selected_row_style());
    render_list(frame, providers_list, chunks[1], providers_selected);

    let help = help_paragraph(
        "Tab: switch  Up/Down: move  Enter/e: edit/open  n: add provider  d: delete provider  i: import live  p: preview  a: apply  q/Esc: back",
    );
    frame.render_widget(help, chunks[2]);
}

fn draw_opencode_provider(frame: &mut Frame, app: &app::App, area: Rect) {
    let t = theme();
    let Some(profile) = app.opencode_detail.as_ref() else {
        let p = Paragraph::new(vec![Line::from(Span::styled(
            "Failed to load profile",
            t.error_style(),
        ))])
        .block(block("OpenCode Provider"))
        .wrap(Wrap { trim: true });
        frame.render_widget(p, area);
        return;
    };
    let Some(provider_id) = app.opencode_provider_id.as_deref() else {
        let p = Paragraph::new(vec![Line::from(Span::styled(
            "No provider selected",
            t.warning_style(),
        ))])
        .block(block("OpenCode Provider"))
        .wrap(Wrap { trim: true });
        frame.render_widget(p, area);
        return;
    };
    let Some(config) = profile.providers.get(provider_id) else {
        let p = Paragraph::new(vec![Line::from(Span::styled(
            "Provider not found",
            t.error_style(),
        ))])
        .block(block("OpenCode Provider"))
        .wrap(Wrap { trim: true });
        frame.render_widget(p, area);
        return;
    };

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints(
            [
                Constraint::Percentage(45),
                Constraint::Min(0),
                Constraint::Length(2),
            ]
            .as_ref(),
        )
        .split(area);

    let base_url = config
        .options
        .as_ref()
        .and_then(|o| o.base_url.as_deref())
        .unwrap_or("");
    let timeout = config
        .options
        .as_ref()
        .and_then(|o| o.timeout)
        .map(|t| t.to_string())
        .unwrap_or_else(|| "".to_string());
    let api_key_set = profile
        .auth
        .get(provider_id)
        .and_then(|v| v.get("key"))
        .and_then(|k| k.as_str())
        .is_some_and(|k| !k.trim().is_empty());

    let provider_fields: Vec<(&str, String)> = vec![
        (
            "Display Name",
            config.name.clone().unwrap_or_else(|| "".to_string()),
        ),
        ("NPM", not_set_if_blank(config.npm.as_deref().unwrap_or(""))),
        ("Base URL", base_url.to_string()),
        (
            "API Key",
            if api_key_set {
                "********".to_string()
            } else {
                "(not set)".to_string()
            },
        ),
        ("Timeout", timeout),
    ];

    let mut field_items: Vec<ListItem> = Vec::new();
    for (i, (label, value)) in provider_fields.into_iter().enumerate() {
        let selected = app.opencode_provider_focus == app::CodexDetailFocus::Fields
            && i == app.opencode_provider_field_index;
        let line = if selected {
            Line::from(format!("{label:>16}: {value}"))
        } else {
            let custom_style = match label {
                "API Key" if value == "********" => Some(t.success_fg_style()),
                _ => None,
            };
            field_line_custom(label, &value, 16, custom_style)
        };
        field_items.push(ListItem::new(line));
    }
    let fields_list = List::new(field_items).block(block_focus(
        format!("OpenCode Provider: {provider_id}"),
        app.opencode_provider_focus == app::CodexDetailFocus::Fields,
    ));
    let fields_selected = (app.opencode_provider_focus == app::CodexDetailFocus::Fields)
        .then_some(app.opencode_provider_field_index);
    let fields_list = fields_list.highlight_style(t.selected_row_style());
    render_list(frame, fields_list, chunks[0], fields_selected);

    let mut model_items: Vec<ListItem> = Vec::new();
    for mid in app.opencode_provider_model_ids.iter() {
        model_items.push(ListItem::new(Line::from(Span::raw(mid.clone()))));
    }
    if model_items.is_empty() {
        model_items.push(ListItem::new(Line::from(Span::styled(
            "No models",
            t.placeholder_style(),
        ))));
    }
    let models_list = List::new(model_items).block(block_focus(
        "Models (Tab to focus)",
        app.opencode_provider_focus == app::CodexDetailFocus::Providers,
    ));
    let models_selected = (app.opencode_provider_focus == app::CodexDetailFocus::Providers
        && !app.opencode_provider_model_ids.is_empty())
    .then_some(app.opencode_provider_model_index);
    let models_list = models_list.highlight_style(t.selected_row_style());
    render_list(frame, models_list, chunks[1], models_selected);

    let help = help_paragraph(
        "Tab: switch  Up/Down: move  Enter/e: edit/open  n: add model  d: delete model  q/Esc: back",
    );
    frame.render_widget(help, chunks[2]);
}

fn draw_opencode_model(frame: &mut Frame, app: &app::App, area: Rect) {
    let t = theme();
    let Some(profile) = app.opencode_detail.as_ref() else {
        let p = Paragraph::new(vec![Line::from(Span::styled(
            "Failed to load profile",
            t.error_style(),
        ))])
        .block(block("OpenCode Model"))
        .wrap(Wrap { trim: true });
        frame.render_widget(p, area);
        return;
    };
    let Some(provider_id) = app.opencode_provider_id.as_deref() else {
        let p = Paragraph::new(vec![Line::from(Span::styled(
            "No provider selected",
            t.warning_style(),
        ))])
        .block(block("OpenCode Model"))
        .wrap(Wrap { trim: true });
        frame.render_widget(p, area);
        return;
    };
    let Some(model_id) = app.opencode_model_id.as_deref() else {
        let p = Paragraph::new(vec![Line::from(Span::styled(
            "No model selected",
            t.warning_style(),
        ))])
        .block(block("OpenCode Model"))
        .wrap(Wrap { trim: true });
        frame.render_widget(p, area);
        return;
    };
    let model = profile
        .providers
        .get(provider_id)
        .and_then(|p| p.models.as_ref())
        .and_then(|m| m.get(model_id));
    let Some(model) = model else {
        let p = Paragraph::new(vec![Line::from(Span::styled(
            "Model not found",
            t.error_style(),
        ))])
        .block(block("OpenCode Model"))
        .wrap(Wrap { trim: true });
        frame.render_widget(p, area);
        return;
    };

    let context = model
        .limit
        .as_ref()
        .and_then(|l| l.context)
        .map(|v| v.to_string())
        .unwrap_or_else(|| "".to_string());
    let output = model
        .limit
        .as_ref()
        .and_then(|l| l.output)
        .map(|v| v.to_string())
        .unwrap_or_else(|| "".to_string());

    let fields: Vec<(&str, String)> = vec![
        (
            "Display Name",
            model.name.clone().unwrap_or_else(|| "".to_string()),
        ),
        ("Context Limit", context),
        ("Output Limit", output),
    ];

    let mut items: Vec<ListItem> = Vec::new();
    for (i, (label, value)) in fields.into_iter().enumerate() {
        let selected = i == app.opencode_model_field_index;
        let line = if selected {
            Line::from(format!("{label:>16}: {value}"))
        } else {
            field_line(label, &value, 16)
        };
        items.push(ListItem::new(line));
    }

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(0), Constraint::Length(2)].as_ref())
        .split(area);

    let list = List::new(items)
        .block(block(format!("OpenCode Model: {model_id}")))
        .highlight_style(t.selected_row_style());
    render_list(frame, list, chunks[0], Some(app.opencode_model_field_index));

    let help = help_paragraph("Up/Down: select  Enter/e: edit  q/Esc: back");
    frame.render_widget(help, chunks[1]);
}

fn draw_codex_profile(frame: &mut Frame, app: &app::App, area: Rect) {
    let t = theme();
    let Some(profile) = app.codex_detail.as_ref() else {
        let p = Paragraph::new(vec![Line::from(Span::styled(
            "Failed to load profile",
            t.error_style(),
        ))])
        .block(block("Codex Profile"))
        .wrap(Wrap { trim: true });
        frame.render_widget(p, area);
        return;
    };

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints(
            [
                Constraint::Percentage(45),
                Constraint::Min(0),
                Constraint::Length(2),
            ]
            .as_ref(),
        )
        .split(area);

    let mut lines: Vec<Line> = Vec::new();
    let effort = profile
        .model_reasoning_effort
        .as_deref()
        .unwrap_or("(none)");
    let api_key_set = profile
        .api_key
        .as_deref()
        .is_some_and(|k| !k.trim().is_empty());

    let fields: Vec<(&str, String)> = vec![
        ("Name", profile.name.clone()),
        (
            "Description",
            profile
                .description
                .clone()
                .unwrap_or_else(|| "".to_string()),
        ),
        ("Model Provider", profile.model_provider.clone()),
        ("Model", profile.model.clone()),
        ("Reasoning Effort", effort.to_string()),
        (
            "API Key",
            if api_key_set {
                "********".to_string()
            } else {
                "(not set)".to_string()
            },
        ),
    ];

    for (i, (label, value)) in fields.into_iter().enumerate() {
        let selected = app.codex_detail_focus == app::CodexDetailFocus::Fields
            && i == app.codex_detail_field_index;
        let label_style = if selected {
            t.selected_style()
        } else {
            t.dim_style()
        };
        let value_style = if selected {
            t.selected_style()
        } else if label == "API Key" && value == "********" {
            t.success_fg_style()
        } else {
            value_style_for(&value)
        };
        lines.push(Line::from(vec![
            Span::styled(format!("{label:>16}: "), label_style),
            Span::styled(value, value_style),
        ]));
    }

    let fields_block = Paragraph::new(lines)
        .block(block_focus(
            format!("Codex Profile: {}", profile.name),
            app.codex_detail_focus == app::CodexDetailFocus::Fields,
        ))
        .wrap(Wrap { trim: false });
    frame.render_widget(fields_block, chunks[0]);

    let mut provider_items: Vec<ListItem> = Vec::new();
    for pid in app.codex_detail_provider_ids.iter() {
        let active_tag = if pid == &profile.model_provider {
            " *"
        } else {
            ""
        };
        if active_tag.is_empty() {
            provider_items.push(ListItem::new(Line::from(Span::raw(pid.clone()))));
        } else {
            provider_items.push(ListItem::new(Line::from(vec![
                Span::raw(pid.clone()),
                Span::styled(active_tag.to_string(), t.success_style()),
            ])));
        }
    }
    if provider_items.is_empty() {
        provider_items.push(ListItem::new(Line::from(Span::styled(
            "No providers",
            t.placeholder_style(),
        ))));
    }
    let providers_selected = (app.codex_detail_focus == app::CodexDetailFocus::Providers
        && !app.codex_detail_provider_ids.is_empty())
    .then_some(app.codex_detail_provider_index);
    let providers_list = List::new(provider_items)
        .block(block_focus(
            "Providers (Tab to focus)",
            app.codex_detail_focus == app::CodexDetailFocus::Providers,
        ))
        .highlight_style(t.selected_row_style());
    render_list(frame, providers_list, chunks[1], providers_selected);

    let help = help_paragraph(
        "Tab: switch  Up/Down: move  Enter/e: edit/open  n: add provider  s: set active  d: delete provider  l: load live  p: preview  a: apply  q/Esc: back",
    );
    frame.render_widget(help, chunks[2]);
}

fn draw_codex_provider(frame: &mut Frame, app: &app::App, area: Rect) {
    let t = theme();
    let Some(profile) = app.codex_detail.as_ref() else {
        let p = Paragraph::new(vec![Line::from(Span::styled(
            "Failed to load profile",
            t.error_style(),
        ))])
        .block(block("Codex Provider"))
        .wrap(Wrap { trim: true });
        frame.render_widget(p, area);
        return;
    };
    let Some(provider_id) = app.codex_provider_id.as_deref() else {
        let p = Paragraph::new(vec![Line::from(Span::styled(
            "No provider selected",
            t.warning_style(),
        ))])
        .block(block("Codex Provider"))
        .wrap(Wrap { trim: true });
        frame.render_widget(p, area);
        return;
    };
    let Some(config) = profile.providers.get(provider_id) else {
        let p = Paragraph::new(vec![Line::from(Span::styled(
            "Provider not found",
            t.error_style(),
        ))])
        .block(block("Codex Provider"))
        .wrap(Wrap { trim: true });
        frame.render_widget(p, area);
        return;
    };

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(0), Constraint::Length(2)].as_ref())
        .split(area);

    let wire_api = config.wire_api.as_deref().unwrap_or("responses");
    let effort = config.model_reasoning_effort.as_deref().unwrap_or("(none)");
    let api_key_set = config
        .api_key
        .as_deref()
        .is_some_and(|k| !k.trim().is_empty());

    let fields: Vec<(&str, String)> = vec![
        (
            "Name",
            config.name.clone().unwrap_or_else(|| "".to_string()),
        ),
        (
            "Base URL",
            config.base_url.clone().unwrap_or_else(|| "".to_string()),
        ),
        ("Wire API", wire_api.to_string()),
        (
            "Model",
            not_set_if_blank(config.model.as_deref().unwrap_or("")),
        ),
        ("Reasoning Effort", effort.to_string()),
        (
            "API Key",
            if api_key_set {
                "********".to_string()
            } else {
                "(not set)".to_string()
            },
        ),
    ];

    let mut items: Vec<ListItem> = Vec::new();
    for (i, (label, value)) in fields.into_iter().enumerate() {
        let selected = i == app.codex_provider_field_index;
        let line = if selected {
            Line::from(format!("{label:>16}: {value}"))
        } else {
            let custom_style = match label {
                "API Key" if value == "********" => Some(t.success_fg_style()),
                _ => None,
            };
            field_line_custom(label, &value, 16, custom_style)
        };
        items.push(ListItem::new(line));
    }

    let list = List::new(items)
        .block(block(format!("Codex Provider: {provider_id}")))
        .highlight_style(t.selected_row_style());
    render_list(frame, list, chunks[0], Some(app.codex_provider_field_index));

    let help = help_paragraph("Up/Down: select  Enter/e: edit  q/Esc: back");
    frame.render_widget(help, chunks[1]);
}

fn draw_sessions(frame: &mut Frame, app: &app::App, area: Rect) {
    let t = theme();
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(0), Constraint::Length(2)].as_ref())
        .split(area);

    let mut items: Vec<ListItem> = Vec::new();
    for (i, s) in app.sessions.iter().enumerate() {
        let selected = i == app.sessions_index;
        if selected {
            items.push(ListItem::new(Line::from(vec![
                Span::raw(s.title.clone()),
                Span::raw("  "),
                Span::raw(format!("[{}]", s.project)),
                Span::raw("  "),
                Span::styled(s.model.clone(), t.key_style()),
            ])));
        } else {
            items.push(ListItem::new(Line::from(vec![
                Span::raw(s.title.clone()),
                Span::raw("  "),
                Span::styled(format!("[{}]", s.project), t.dim_style()),
                Span::raw("  "),
                Span::styled(s.model.clone(), t.key_style()),
            ])));
        }
    }
    if items.is_empty() {
        items.push(ListItem::new(Line::from(Span::styled(
            "No sessions",
            t.placeholder_style(),
        ))));
    }

    let selected = (!app.sessions.is_empty()).then_some(app.sessions_index);
    let list = List::new(items)
        .block(block("Sessions"))
        .highlight_style(t.selected_row_style());
    render_list(frame, list, chunks[0], selected);

    let help = help_paragraph("Up/Down: select  Enter/v: view  d: delete  r: refresh  q/Esc: back");
    frame.render_widget(help, chunks[1]);
}

fn draw_specs(frame: &mut Frame, app: &app::App, area: Rect) {
    let t = theme();
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(0), Constraint::Length(2)].as_ref())
        .split(area);

    let mut items: Vec<ListItem> = Vec::new();
    for s in app.specs.iter() {
        items.push(ListItem::new(Line::from(Span::raw(s.name.clone()))));
    }
    if items.is_empty() {
        items.push(ListItem::new(Line::from(Span::styled(
            "No specs",
            t.placeholder_style(),
        ))));
    }

    let selected = (!app.specs.is_empty()).then_some(app.specs_index);
    let list = List::new(items)
        .block(block("Specs"))
        .highlight_style(t.selected_row_style());
    render_list(frame, list, chunks[0], selected);

    let help = help_paragraph("Up/Down: select  Enter/e: edit  d: delete  r: refresh  q/Esc: back");
    frame.render_widget(help, chunks[1]);
}

fn draw_channels(frame: &mut Frame, app: &app::App, area: Rect) {
    let t = theme();
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(0), Constraint::Length(2)].as_ref())
        .split(area);

    let mut items: Vec<ListItem> = Vec::new();
    for (i, c) in app.channels.iter().enumerate() {
        let selected = i == app.channels_index;
        let enabled = if c.enabled { "on" } else { "off" };
        if selected {
            let enabled_style = if c.enabled {
                t.success_fg_style()
            } else {
                t.warning_fg_style()
            };
            items.push(ListItem::new(Line::from(vec![
                Span::raw(c.name.clone()),
                Span::raw("  "),
                Span::styled(format!("[{enabled}]"), enabled_style),
                Span::raw("  "),
                Span::raw(c.base_url.clone()),
            ])));
        } else {
            let enabled_style = if c.enabled {
                t.success_fg_style()
            } else {
                t.warning_fg_style()
            };
            items.push(ListItem::new(Line::from(vec![
                Span::raw(c.name.clone()),
                Span::raw("  "),
                Span::styled(format!("[{enabled}]"), enabled_style),
                Span::raw("  "),
                Span::styled(c.base_url.clone(), t.dim_style()),
            ])));
        }
    }
    if items.is_empty() {
        items.push(ListItem::new(Line::from(Span::styled(
            "No channels",
            t.placeholder_style(),
        ))));
    }

    let selected = (!app.channels.is_empty()).then_some(app.channels_index);
    let list = List::new(items)
        .block(block("Channels"))
        .highlight_style(t.selected_row_style());
    render_list(frame, list, chunks[0], selected);

    let help = help_paragraph(
        "Up/Down: select  Enter/e: open  n: new  t: toggle  d: delete  E: raw list  A: raw auth  r: refresh  q/Esc: back",
    );
    frame.render_widget(help, chunks[1]);
}

fn draw_channels_edit(frame: &mut Frame, app: &app::App, area: Rect) {
    let t = theme();
    let Some(channel) = app.channels_edit_draft.as_ref() else {
        let p = Paragraph::new(vec![Line::from(Span::styled(
            "No channel loaded",
            t.warning_style(),
        ))])
        .block(block("Channel"))
        .wrap(Wrap { trim: true });
        frame.render_widget(p, area);
        return;
    };

    let channel_type = match channel.channel_type {
        droidgear_core::channel::ChannelType::NewApi => "new-api",
        droidgear_core::channel::ChannelType::Sub2Api => "sub-2-api",
        droidgear_core::channel::ChannelType::CliProxyApi => "cli-proxy-api",
        droidgear_core::channel::ChannelType::Ollama => "ollama",
        droidgear_core::channel::ChannelType::General => "general",
    };

    let uses_api_key = matches!(
        channel.channel_type,
        droidgear_core::channel::ChannelType::CliProxyApi
            | droidgear_core::channel::ChannelType::Ollama
            | droidgear_core::channel::ChannelType::General
    );
    let enabled = if channel.enabled { "yes" } else { "no" };
    let api_key_set = !app.channels_edit_api_key.trim().is_empty();
    let password_set = !app.channels_edit_password.trim().is_empty();

    let mut fields: Vec<(&str, String)> = vec![
        ("Name", not_set_if_blank(&channel.name)),
        ("Type", channel_type.to_string()),
        ("Base URL", not_set_if_blank(&channel.base_url)),
        ("Enabled", enabled.to_string()),
    ];

    if uses_api_key {
        fields.push((
            "API Key",
            if api_key_set {
                "********".to_string()
            } else {
                "(not set)".to_string()
            },
        ));
    } else {
        fields.push(("Username", app.channels_edit_username.clone()));
        fields.push((
            "Password",
            if password_set {
                "********".to_string()
            } else {
                "(not set)".to_string()
            },
        ));
    }

    let mut items: Vec<ListItem> = Vec::new();
    for (i, (label, value)) in fields.into_iter().enumerate() {
        let selected = i == app.channels_edit_field_index;
        let line = if selected {
            Line::from(format!("{label:>16}: {value}"))
        } else {
            let custom_style = match label {
                "API Key" | "Password" if value == "********" => Some(t.success_fg_style()),
                _ => None,
            };
            field_line_custom(label, &value, 16, custom_style)
        };
        items.push(ListItem::new(line));
    }

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(0), Constraint::Length(2)].as_ref())
        .split(area);

    let title = if channel.name.trim().is_empty() {
        "Channel: (new)".to_string()
    } else {
        format!("Channel: {}", channel.name)
    };
    let list = List::new(items)
        .block(block(title))
        .highlight_style(t.selected_row_style());
    render_list(frame, list, chunks[0], Some(app.channels_edit_field_index));

    let help = help_paragraph("Up/Down: select  Enter/e: edit/toggle  s: save  q/Esc: back");
    frame.render_widget(help, chunks[1]);
}

fn draw_profile_list<'a>(
    frame: &mut Frame,
    area: Rect,
    title: &str,
    profiles: impl Iterator<Item = (&'a str, &'a str)>,
    active_id: Option<&str>,
    selected_index: usize,
    help_text: &str,
) {
    let t = theme();
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(0), Constraint::Length(2)].as_ref())
        .split(area);

    let mut has_profiles = false;
    let mut items: Vec<ListItem> = Vec::new();
    for (name, id) in profiles {
        has_profiles = true;
        let active_tag = if active_id.is_some_and(|a| a == id) {
            " *"
        } else {
            ""
        };
        if active_tag.is_empty() {
            items.push(ListItem::new(Line::from(Span::raw(name.to_string()))));
        } else {
            items.push(ListItem::new(Line::from(vec![
                Span::styled(name.to_string(), t.success_fg_style()),
                Span::styled(active_tag.to_string(), t.success_style()),
            ])));
        }
    }
    if items.is_empty() {
        items.push(ListItem::new(Line::from(Span::styled(
            "No profiles",
            t.placeholder_style(),
        ))));
    }

    let selected = has_profiles.then_some(selected_index);
    let list = List::new(items)
        .block(block(title))
        .highlight_style(t.selected_row_style());
    render_list(frame, list, chunks[0], selected);

    let help = help_paragraph(help_text);
    frame.render_widget(help, chunks[1]);
}

fn draw_modal(frame: &mut Frame, modal: &app::Modal) {
    let t = theme();
    let area = centered_rect(70, 30, frame.area());
    frame.render_widget(Clear, area);
    match modal {
        app::Modal::Confirm { message, .. } => {
            let text = vec![
                Line::from(message.as_str()),
                Line::from(""),
                hint_line("y/Enter: yes  n/Esc: no"),
            ];
            let block = block("Confirm")
                .border_style(t.warning_style())
                .title_style(t.warning_style());
            let p = Paragraph::new(text)
                .block(block)
                .wrap(Wrap { trim: false })
                .style(t.modal_style());
            frame.render_widget(p, area);
        }
        app::Modal::Input {
            title,
            value,
            cursor,
            is_secret,
            ..
        } => {
            let body = if *is_secret {
                "*".repeat(value.chars().count())
            } else {
                value.clone()
            };

            let block = block(title.as_str())
                .border_style(t.focused_border_style())
                .style(t.modal_style());
            let inner = block.inner(area);
            frame.render_widget(block, area);

            let input_area = Rect {
                x: inner.x,
                y: inner.y,
                width: inner.width,
                height: 1,
            };
            let hint_area = Rect {
                x: inner.x,
                y: inner.y.saturating_add(2),
                width: inner.width,
                height: 1,
            };

            let body_len = body.chars().count();
            let cursor = (*cursor).min(body_len);
            let cursor_byte = byte_index_for_char(&body, cursor);

            let view_width = input_area.width as usize;
            let total_width = Span::raw(body.as_str()).width();
            let cursor_col = Span::raw(&body[..cursor_byte]).width();
            let mut effective_cursor_col = cursor_col;
            if cursor == body_len
                && total_width >= view_width
                && total_width > 0
                && cursor_col == total_width
            {
                // When the cursor is at EOF and there's no extra room to render the cell after the
                // last character, place it on the last visible cell (like most terminal inputs).
                effective_cursor_col = total_width.saturating_sub(1);
            }

            // Keep the cursor roughly 1/3 into the viewport so you can still see some trailing text.
            let target_x = view_width / 3;
            let max_scroll = total_width.saturating_sub(view_width);
            let scroll_x = effective_cursor_col
                .saturating_sub(target_x)
                .min(max_scroll)
                .min(u16::MAX as usize) as u16;

            let input = Paragraph::new(Line::from(Span::raw(body)))
                .style(t.modal_style())
                .scroll((0, scroll_x));
            frame.render_widget(input, input_area);

            let hint = Paragraph::new(vec![hint_line(
                "Left/Right: move  Backspace: delete  Enter: confirm  Esc: cancel",
            )])
            .style(t.modal_style())
            .wrap(Wrap { trim: true });
            frame.render_widget(hint, hint_area);

            if input_area.width > 0 {
                let scroll_x_usize = scroll_x as usize;
                let cursor_x = effective_cursor_col
                    .saturating_sub(scroll_x_usize)
                    .min(view_width.saturating_sub(1));
                frame.set_cursor_position(Position {
                    x: input_area.x.saturating_add(cursor_x as u16),
                    y: input_area.y,
                });
            }
        }
        app::Modal::Select {
            title,
            options,
            index,
            ..
        } => {
            let mut lines: Vec<Line> = Vec::new();
            if options.is_empty() {
                lines.push(Line::from("(no options)"));
            } else {
                for (i, opt) in options.iter().enumerate() {
                    let mut style = Style::default();
                    if i == *index {
                        style = t.selected_style();
                    }
                    lines.push(Line::from(Span::styled(opt.clone(), style)));
                }
            }
            lines.push(Line::from(""));
            lines.push(hint_line("Up/Down: select  Enter: confirm  Esc: cancel"));
            let block = block(title.as_str()).border_style(t.focused_border_style());
            let p = Paragraph::new(lines)
                .block(block)
                .wrap(Wrap { trim: false })
                .style(t.modal_style());
            frame.render_widget(p, area);
        }
    }
}

fn draw_toast(frame: &mut Frame, toast: &app::Toast) {
    let t = theme();
    let area = Rect {
        x: 0,
        y: frame.area().height.saturating_sub(1),
        width: frame.area().width,
        height: 1,
    };
    let style = if toast.is_error {
        t.error_style()
    } else {
        t.success_style()
    };
    let p = Paragraph::new(Line::from(Span::styled(&toast.message, style)))
        .block(Block::default().borders(Borders::NONE));
    frame.render_widget(p, area);
}

fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints(
            [
                Constraint::Percentage((100 - percent_y) / 2),
                Constraint::Percentage(percent_y),
                Constraint::Percentage((100 - percent_y) / 2),
            ]
            .as_ref(),
        )
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints(
            [
                Constraint::Percentage((100 - percent_x) / 2),
                Constraint::Percentage(percent_x),
                Constraint::Percentage((100 - percent_x) / 2),
            ]
            .as_ref(),
        )
        .split(popup_layout[1])[1]
}

fn draw_missions(frame: &mut Frame, app: &app::App, area: Rect) {
    let t = theme();
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(0), Constraint::Length(2)].as_ref())
        .split(area);

    let not_set = "(not set)";
    let fields: Vec<(&str, String)> = vec![
        (
            "Worker Model",
            app.mission_settings
                .worker_model
                .clone()
                .unwrap_or_else(|| not_set.to_string()),
        ),
        (
            "Worker Reasoning",
            app.mission_settings
                .worker_reasoning_effort
                .clone()
                .unwrap_or_else(|| not_set.to_string()),
        ),
        (
            "Validation Model",
            app.mission_settings
                .validation_worker_model
                .clone()
                .unwrap_or_else(|| not_set.to_string()),
        ),
        (
            "Validation Reasoning",
            app.mission_settings
                .validation_worker_reasoning_effort
                .clone()
                .unwrap_or_else(|| not_set.to_string()),
        ),
    ];

    let mut items: Vec<ListItem> = Vec::new();
    for (i, (label, value)) in fields.into_iter().enumerate() {
        let selected = i == app.mission_field_index;
        let line = if selected {
            Line::from(format!("{label:>22}: {value}"))
        } else {
            field_line(label, &value, 22)
        };
        items.push(ListItem::new(line));
    }

    let list = List::new(items)
        .block(block("Missions"))
        .highlight_style(t.selected_row_style());
    render_list(frame, list, chunks[0], Some(app.mission_field_index));

    let help = help_paragraph("Up/Down: select  Enter/e: edit  r: refresh  q/Esc: back");
    frame.render_widget(help, chunks[1]);
}

fn draw_hermes_profiles(frame: &mut Frame, app: &app::App, area: Rect) {
    let active = app.hermes_active_id.as_deref();
    let selected_index = app.hermes_index;
    draw_profile_list(
        frame,
        area,
        "Hermes Profiles",
        app.hermes_profiles
            .iter()
            .map(|p| (p.name.as_str(), p.id.as_str())),
        active,
        selected_index,
        "Up/Down: select  Enter/e: open  a: apply  n: new  c: copy  d: delete  r: refresh  q/Esc: back",
    );
}

fn draw_hermes_profile(frame: &mut Frame, app: &app::App, area: Rect) {
    let t = theme();
    let Some(profile) = app.hermes_detail.as_ref() else {
        let p = Paragraph::new(vec![Line::from(Span::styled(
            "Failed to load profile",
            t.error_style(),
        ))])
        .block(block("Hermes Profile"))
        .wrap(Wrap { trim: true });
        frame.render_widget(p, area);
        return;
    };

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(0), Constraint::Length(2)].as_ref())
        .split(area);

    let api_key_set = profile
        .model
        .api_key
        .as_deref()
        .is_some_and(|k| !k.trim().is_empty());

    let fields: Vec<(&str, String)> = vec![
        ("Name", profile.name.clone()),
        (
            "Description",
            profile
                .description
                .clone()
                .unwrap_or_else(|| "".to_string()),
        ),
        (
            "Default Model",
            profile
                .model
                .default
                .clone()
                .unwrap_or_else(|| "".to_string()),
        ),
        (
            "Provider",
            profile
                .model
                .provider
                .clone()
                .unwrap_or_else(|| "".to_string()),
        ),
        (
            "Base URL",
            profile
                .model
                .base_url
                .clone()
                .unwrap_or_else(|| "".to_string()),
        ),
        (
            "API Key",
            if api_key_set {
                "********".to_string()
            } else {
                "(not set)".to_string()
            },
        ),
    ];

    let mut items: Vec<ListItem> = Vec::new();
    for (i, (label, value)) in fields.into_iter().enumerate() {
        let selected = i == app.hermes_detail_field_index;
        let line = if selected {
            Line::from(format!("{label:>16}: {value}"))
        } else {
            let custom_style = match label {
                "API Key" if value == "********" => Some(t.success_fg_style()),
                _ => None,
            };
            field_line_custom(label, &value, 16, custom_style)
        };
        items.push(ListItem::new(line));
    }

    let list = List::new(items)
        .block(block(format!("Hermes Profile: {}", profile.name)))
        .highlight_style(t.selected_row_style());
    render_list(frame, list, chunks[0], Some(app.hermes_detail_field_index));

    let help = help_paragraph(
        "Up/Down: move  Enter/e: edit  m: model config  l: load live  a: apply  q/Esc: back",
    );
    frame.render_widget(help, chunks[1]);
}

fn draw_hermes_provider(frame: &mut Frame, app: &app::App, area: Rect) {
    let t = theme();
    let Some(profile) = app.hermes_detail.as_ref() else {
        let p = Paragraph::new(vec![Line::from(Span::styled(
            "Failed to load profile",
            t.error_style(),
        ))])
        .block(block("Hermes Model Config"))
        .wrap(Wrap { trim: true });
        frame.render_widget(p, area);
        return;
    };

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(0), Constraint::Length(2)].as_ref())
        .split(area);

    let api_key_set = profile
        .model
        .api_key
        .as_deref()
        .is_some_and(|k| !k.trim().is_empty());

    let fields: Vec<(&str, String)> = vec![
        (
            "Default Model",
            profile
                .model
                .default
                .clone()
                .unwrap_or_else(|| "".to_string()),
        ),
        (
            "Provider",
            profile
                .model
                .provider
                .clone()
                .unwrap_or_else(|| "".to_string()),
        ),
        (
            "Base URL",
            profile
                .model
                .base_url
                .clone()
                .unwrap_or_else(|| "".to_string()),
        ),
        (
            "API Key",
            if api_key_set {
                "********".to_string()
            } else {
                "(not set)".to_string()
            },
        ),
    ];

    let mut items: Vec<ListItem> = Vec::new();
    for (i, (label, value)) in fields.into_iter().enumerate() {
        let selected = i == app.hermes_provider_field_index;
        let line = if selected {
            Line::from(format!("{label:>16}: {value}"))
        } else {
            let custom_style = match label {
                "API Key" if value == "********" => Some(t.success_fg_style()),
                _ => None,
            };
            field_line_custom(label, &value, 16, custom_style)
        };
        items.push(ListItem::new(line));
    }

    let list = List::new(items)
        .block(block(format!("Hermes Model Config: {}", profile.name)))
        .highlight_style(t.selected_row_style());
    render_list(
        frame,
        list,
        chunks[0],
        Some(app.hermes_provider_field_index),
    );

    let help =
        help_paragraph("Up/Down: select  Enter/e: edit  i: import from channel  q/Esc: back");
    frame.render_widget(help, chunks[1]);
}
