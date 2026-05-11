use super::*;

pub(super) fn run_action(app: &mut app::App, action: Action) -> anyhow::Result<()> {
    match action {
        Action::EditFactoryModels => edit_factory_models(app),
        Action::EditCodexProfile { id } => {
            let profile = droidgear_core::codex::get_codex_profile_for_home(&app.home_dir, &id)
                .map_err(anyhow::Error::msg)?;
            let edited = edit_json_in_editor(&profile)?;
            droidgear_core::codex::save_codex_profile_for_home(&app.home_dir, edited)
                .map_err(anyhow::Error::msg)?;
            app.set_toast("Saved", false);
            Ok(())
        }
        Action::EditOpenCodeProfile { id } => {
            let profile =
                droidgear_core::opencode::get_opencode_profile_for_home(&app.home_dir, &id)
                    .map_err(anyhow::Error::msg)?;
            let edited = edit_json_in_editor(&profile)?;
            droidgear_core::opencode::save_opencode_profile_for_home(&app.home_dir, edited)
                .map_err(anyhow::Error::msg)?;
            app.set_toast("Saved", false);
            Ok(())
        }
        Action::EditOpenClawProfile { id } => {
            let profile =
                droidgear_core::openclaw::get_openclaw_profile_for_home(&app.home_dir, &id)
                    .map_err(anyhow::Error::msg)?;
            let edited = edit_json_in_editor(&profile)?;
            droidgear_core::openclaw::save_openclaw_profile_for_home(&app.home_dir, edited)
                .map_err(anyhow::Error::msg)?;
            app.set_toast("Saved", false);
            Ok(())
        }
        Action::PreviewDroidRun { settings_path } => {
            let preview = preview_droid_temporary_run(&app.home_dir, Path::new(&settings_path))?;
            open_text_in_pager(&preview)?;
            Ok(())
        }
        Action::RunDroidRun { settings_path } => {
            run_droid_temporary_run(&app.home_dir, Path::new(&settings_path))?;
            app.should_quit = true;
            Ok(())
        }
        Action::PreviewClaudeRun { id } => {
            let preview = preview_claude_temporary_run(&app.home_dir, &id)?;
            open_text_in_pager(&preview)?;
            Ok(())
        }
        Action::RunClaudeRun { id } => {
            run_claude_temporary_run(&app.home_dir, &id)?;
            app.should_quit = true;
            Ok(())
        }
        Action::PreviewCodexApply { id } => {
            let diff = preview_codex_apply(&app.home_dir, &id)?;
            open_text_in_pager(&diff)?;
            Ok(())
        }
        Action::PreviewCodexRun { id } => {
            let preview = preview_codex_temporary_run(&app.home_dir, &id)?;
            open_text_in_pager(&preview)?;
            Ok(())
        }
        Action::RunCodexRun { id } => {
            run_codex_temporary_run(&app.home_dir, &id)?;
            app.should_quit = true;
            Ok(())
        }
        Action::PreviewOpenCodeApply { id } => {
            let diff = preview_opencode_apply(&app.home_dir, &id)?;
            open_text_in_pager(&diff)?;
            Ok(())
        }
        Action::PreviewOpenClawApply { id } => {
            let diff = preview_openclaw_apply(&app.home_dir, &id)?;
            open_text_in_pager(&diff)?;
            Ok(())
        }
        Action::ViewSession { path } => {
            let detail =
                droidgear_core::sessions::get_session_detail_for_home(&app.home_dir, &path)
                    .map_err(anyhow::Error::msg)?;
            let text = format_session_detail(&detail);

            let mut temp = NamedTempFile::new().context("create temp file")?;
            temp.write_all(text.as_bytes()).context("write temp file")?;
            temp.flush().context("flush temp file")?;
            editor::open_in_pager(temp.path())?;
            Ok(())
        }
        Action::EditSpec { path } => {
            let path = PathBuf::from(path);
            editor::open_in_editor(&path)?;
            Ok(())
        }
        Action::EditChannels => {
            let channels = droidgear_core::channel::load_channels_for_home(&app.home_dir)
                .map_err(anyhow::Error::msg)?;
            let edited: Vec<droidgear_core::channel::Channel> = edit_json_in_editor(&channels)?;
            droidgear_core::channel::save_channels_for_home(&app.home_dir, edited)
                .map_err(anyhow::Error::msg)?;
            app.set_toast("Saved", false);
            Ok(())
        }
        Action::EditChannelAuth { id } => {
            let auth_dir = app.home_dir.join(".droidgear").join("auth");
            std::fs::create_dir_all(&auth_dir).context("create auth dir")?;
            let auth_path = auth_dir.join(format!("{id}.json"));
            if !auth_path.exists() {
                std::fs::write(
                    &auth_path,
                    "{\n  \"type\": \"api_key\",\n  \"api_key\": \"\"\n}\n",
                )
                .context("write auth template")?;
            }
            editor::open_in_editor(&auth_path)?;
            Ok(())
        }
        Action::SetActiveSettingsFile { name } => {
            droidgear_core::droid_settings_files::set_active_settings_file(name)
                .map_err(|e| anyhow::anyhow!("{e}"))?;
            refresh_droid_settings_files(app);
            Ok(())
        }
    }
}

pub(super) fn format_session_detail(detail: &droidgear_core::sessions::SessionDetail) -> String {
    let mut out = String::new();
    out.push_str(&format!("Title: {}\n", detail.title));
    out.push_str(&format!("Project: {}\n", detail.project));
    out.push_str(&format!("Model: {}\n", detail.model));
    out.push_str(&format!("CWD: {}\n", detail.cwd));
    out.push('\n');

    for m in &detail.messages {
        out.push_str(&format!("[{}] {}\n", m.role, m.timestamp));
        for block in &m.content {
            if let Some(text) = block.text.as_deref() {
                out.push_str(text);
                if !text.ends_with('\n') {
                    out.push('\n');
                }
            }
            if let Some(thinking) = block.thinking.as_deref() {
                out.push_str("(thinking)\n");
                out.push_str(thinking);
                if !thinking.ends_with('\n') {
                    out.push('\n');
                }
            }
        }
        out.push('\n');
    }

    out
}

pub(super) fn edit_factory_models(app: &mut app::App) -> anyhow::Result<()> {
    let models = droidgear_core::factory_settings::load_custom_models_for_home(&app.home_dir)
        .map_err(anyhow::Error::msg)?;
    let edited: Vec<droidgear_core::factory_settings::CustomModel> = edit_json_in_editor(&models)?;
    droidgear_core::factory_settings::save_custom_models_for_home(&app.home_dir, edited)
        .map_err(anyhow::Error::msg)?;
    app.set_toast("Saved", false);
    Ok(())
}

pub(super) fn edit_json_in_editor<T>(value: &T) -> anyhow::Result<T>
where
    T: Serialize + DeserializeOwned,
{
    let mut temp = NamedTempFile::new().context("create temp file")?;
    let content = serde_json::to_string_pretty(value).context("serialize JSON")?;
    temp.write_all(content.as_bytes())
        .context("write temp file")?;
    temp.flush().context("flush temp file")?;

    editor::open_in_editor(temp.path())?;

    let edited = std::fs::read_to_string(temp.path()).context("read edited file")?;
    let parsed = serde_json::from_str(&edited).context("parse edited JSON")?;
    Ok(parsed)
}

pub(super) fn open_text_in_pager(text: &str) -> anyhow::Result<()> {
    let mut temp = NamedTempFile::new().context("create temp file")?;
    temp.write_all(text.as_bytes()).context("write temp file")?;
    temp.flush().context("flush temp file")?;
    editor::open_in_pager(temp.path())?;
    Ok(())
}

pub(super) fn read_to_string_if_exists(path: &Path) -> anyhow::Result<Option<String>> {
    if path.exists() {
        Ok(Some(
            std::fs::read_to_string(path).with_context(|| format!("read {}", path.display()))?,
        ))
    } else {
        Ok(None)
    }
}
