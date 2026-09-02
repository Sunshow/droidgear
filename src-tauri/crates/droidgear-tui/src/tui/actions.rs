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
        Action::PreviewClaudeRun { name } => {
            let preview = preview_claude_temporary_run_from_file(&app.home_dir, &name)?;
            open_text_in_pager(&preview)?;
            Ok(())
        }
        Action::RunClaudeRun {
            name,
            skip_dangerous,
        } => {
            run_claude_temporary_run_from_file(&app.home_dir, &name, skip_dangerous)?;
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
        Action::TestPiProvider {
            provider_id,
            config,
        } => {
            let result = droidgear_core::pi::test_pi_provider_connection(&provider_id, *config)
                .map_err(anyhow::Error::msg)?;
            if result.success {
                app.set_toast(
                    format!(
                        "Pi provider connected: {} / {} ({}ms)",
                        result.provider_id, result.model_id, result.latency_ms
                    ),
                    false,
                );
            } else {
                app.set_toast(
                    result
                        .error
                        .unwrap_or_else(|| "Pi provider test failed".to_string()),
                    true,
                );
            }
            Ok(())
        }
        Action::FetchDshModels { provider_id } => {
            let config = droidgear_core::dsh::read_dsh_current_config_for_home(&app.home_dir)
                .map_err(anyhow::Error::msg)?;
            let Some(provider) = config.providers.get(&provider_id) else {
                return Err(anyhow::anyhow!("Provider not found"));
            };
            let base_url = provider
                .base_url
                .clone()
                .ok_or_else(|| anyhow::anyhow!("Base URL not set; configure it before fetching"))?;
            let env_name = provider
                .api_key_env
                .clone()
                .filter(|v| !v.trim().is_empty())
                .ok_or_else(|| {
                    anyhow::anyhow!("API key env var not set; configure it before fetching")
                })?;
            let credentials = droidgear_core::dsh::read_dsh_credentials_for_home(&app.home_dir)
                .map_err(anyhow::Error::msg)?;
            let api_key = credentials
                .refs
                .get(&env_name)
                .cloned()
                .ok_or_else(|| anyhow::anyhow!("No API key value for '{env_name}'"))?;

            let fetched = droidgear_core::dsh::fetch_dsh_models_blocking(
                &base_url,
                &api_key,
                provider.api.as_deref(),
            )
            .map_err(anyhow::Error::msg)?;

            let existing_ids: std::collections::HashSet<String> =
                provider.models.iter().map(|m| m.id.clone()).collect();
            let new_models: Vec<droidgear_core::dsh::DshModel> = fetched
                .into_iter()
                .filter(|m| !existing_ids.contains(&m.id))
                .collect();

            if new_models.is_empty() {
                app.set_toast("No new models from provider", false);
                return Ok(());
            }

            app.dsh_fetch_pending_models = Some(new_models);
            let selected = vec![true; app.dsh_fetch_pending_models.as_ref().map_or(0, |v| v.len())];
            app.pending_multi_selected = Some(selected.clone());
            let options: Vec<String> = app
                .dsh_fetch_pending_models
                .as_ref()
                .map(|models| models.iter().map(|m| m.id.clone()).collect())
                .unwrap_or_default();
            app.modal = Some(app::Modal::MultiSelect {
                title: "Select models to add (Tab/c: confirm)".to_string(),
                options,
                selected,
                index: 0,
                action: app::SelectAction::DshAddFetchedModels { provider_id },
            });
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
        Action::SetActiveClaudeSettingsFile { name } => {
            let active_label = name.clone().unwrap_or_else(|| "Global".to_string());
            droidgear_core::claude_settings_files::set_active_settings_file_for_home(
                &app.home_dir,
                name,
            )
            .map_err(|e| anyhow::anyhow!("{e}"))?;
            refresh_claude(app);
            app.set_toast(format!("Active: {active_label}"), false);
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
