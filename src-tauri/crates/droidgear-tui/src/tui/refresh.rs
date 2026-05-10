use super::*;

pub(super) fn refresh_paths(app: &mut app::App) {
    match droidgear_core::paths::get_effective_paths_for_home(&app.home_dir) {
        Ok(p) => app.paths = Some(p),
        Err(e) => app.set_toast(e, true),
    }
}

pub(super) fn refresh_droid_settings_files(app: &mut app::App) {
    match droidgear_core::droid_settings_files::list_settings_files() {
        Ok(files) => {
            app.droid_settings_files = files;
            if app.droid_settings_files_index >= app.droid_settings_files.len() {
                app.droid_settings_files_index = app.droid_settings_files.len().saturating_sub(1);
            }
        }
        Err(e) => app.set_toast(e, true),
    }
}

pub(super) fn refresh_factory(app: &mut app::App) {
    match droidgear_core::factory_settings::load_custom_models_for_home(&app.home_dir) {
        Ok(models) => app.custom_models = models,
        Err(e) => app.set_toast(e, true),
    }
    match droidgear_core::factory_settings::get_default_model_for_home(&app.home_dir) {
        Ok(id) => app.factory_default_model_id = id,
        Err(e) => app.set_toast(e, true),
    }
}

pub(super) fn refresh_mcp(app: &mut app::App) {
    match droidgear_core::mcp::load_mcp_servers_for_home(&app.home_dir) {
        Ok(servers) => app.mcp_servers = servers,
        Err(e) => app.set_toast(e, true),
    }
}

pub(super) fn refresh_claude(app: &mut app::App) {
    match droidgear_core::claude::list_claude_profiles_for_home(&app.home_dir) {
        Ok(list) => app.claude_profiles = list,
        Err(e) => app.set_toast(e, true),
    }

    if app.claude_profiles.is_empty() {
        match droidgear_core::claude::create_default_claude_profile_for_home(&app.home_dir) {
            Ok(profile) => app.claude_profiles = vec![profile],
            Err(error) => app.set_toast(error, true),
        }
    }

    match droidgear_core::claude::get_active_claude_profile_id_for_home(&app.home_dir) {
        Ok(id) => app.claude_active_id = id,
        Err(e) => app.set_toast(e, true),
    }
}

pub(super) fn refresh_claude_detail(app: &mut app::App) {
    let Some(id) = app.claude_detail_id.clone() else {
        app.claude_detail = None;
        return;
    };
    match droidgear_core::claude::get_claude_profile_for_home(&app.home_dir, &id) {
        Ok(profile) => app.claude_detail = Some(profile),
        Err(e) => {
            app.claude_detail = None;
            app.set_toast(e, true);
        }
    }
}

pub(super) fn claude_load_from_live_config(
    app: &mut app::App,
    profile_id: &str,
) -> anyhow::Result<()> {
    let live = droidgear_core::claude::read_claude_current_config_for_home(&app.home_dir)
        .map_err(anyhow::Error::msg)?;
    let mut profile =
        droidgear_core::claude::get_claude_profile_for_home(&app.home_dir, profile_id)
            .map_err(anyhow::Error::msg)?;
    profile.base_url = live.base_url;
    profile.bearer_token = live.bearer_token;
    profile.model = live.model;
    profile.small_model_uses_main_model = live.small_model_uses_main_model;
    profile.small_model = live.small_model;
    profile.reasoning_effort = live.reasoning_effort;
    profile.thinking_mode = live.thinking_mode;
    droidgear_core::claude::save_claude_profile_for_home(&app.home_dir, profile)
        .map_err(anyhow::Error::msg)?;
    Ok(())
}

pub(super) fn refresh_codex(app: &mut app::App) {
    match droidgear_core::codex::list_codex_profiles_for_home(&app.home_dir) {
        Ok(list) => app.codex_profiles = list,
        Err(e) => app.set_toast(e, true),
    }

    let has_user_profiles = app.codex_profiles.iter().any(|p| p.id != "official");
    if !has_user_profiles
        && droidgear_core::codex::create_default_codex_profile_for_home(&app.home_dir).is_ok()
    {
        if let Ok(list) = droidgear_core::codex::list_codex_profiles_for_home(&app.home_dir) {
            app.codex_profiles = list;
        }
    }

    match droidgear_core::codex::get_active_codex_profile_id_for_home(&app.home_dir) {
        Ok(id) => app.codex_active_id = id,
        Err(e) => app.set_toast(e, true),
    }
}

pub(super) fn refresh_codex_detail(app: &mut app::App) {
    let Some(id) = app.codex_detail_id.clone() else {
        app.codex_detail = None;
        app.codex_detail_provider_ids.clear();
        return;
    };
    match droidgear_core::codex::get_codex_profile_for_home(&app.home_dir, &id) {
        Ok(profile) => {
            app.codex_detail_provider_ids =
                profile.providers.keys().cloned().collect::<Vec<String>>();
            app.codex_detail_provider_ids
                .sort_by_key(|a| a.to_lowercase());
            app.codex_detail = Some(profile);
        }
        Err(e) => {
            app.codex_detail = None;
            app.codex_detail_provider_ids.clear();
            app.set_toast(e, true);
        }
    }
}

pub(super) fn codex_set_active_provider(
    app: &mut app::App,
    profile_id: &str,
    provider_id: &str,
) -> anyhow::Result<()> {
    let mut profile = droidgear_core::codex::get_codex_profile_for_home(&app.home_dir, profile_id)
        .map_err(anyhow::Error::msg)?;
    profile.model_provider = provider_id.to_string();
    droidgear_core::codex::save_codex_profile_for_home(&app.home_dir, profile)
        .map_err(anyhow::Error::msg)?;
    Ok(())
}

pub(super) fn codex_load_from_live_config(
    app: &mut app::App,
    profile_id: &str,
) -> anyhow::Result<()> {
    let live = droidgear_core::codex::read_codex_current_config_for_home(&app.home_dir)
        .map_err(anyhow::Error::msg)?;
    let mut profile = droidgear_core::codex::get_codex_profile_for_home(&app.home_dir, profile_id)
        .map_err(anyhow::Error::msg)?;
    profile.providers = live.providers;
    profile.model_provider = live.model_provider;
    profile.model = live.model;
    profile.model_reasoning_effort = live.model_reasoning_effort;
    profile.api_key = live.api_key;
    droidgear_core::codex::save_codex_profile_for_home(&app.home_dir, profile)
        .map_err(anyhow::Error::msg)?;
    Ok(())
}

pub(super) fn refresh_opencode(app: &mut app::App) {
    match droidgear_core::opencode::list_opencode_profiles_for_home(&app.home_dir) {
        Ok(list) => app.opencode_profiles = list,
        Err(e) => app.set_toast(e, true),
    }

    if app.opencode_profiles.is_empty() {
        if let Ok(p) = droidgear_core::opencode::create_default_profile_for_home(&app.home_dir) {
            app.opencode_profiles = vec![p]
        }
    }

    match droidgear_core::opencode::get_active_opencode_profile_id_for_home(&app.home_dir) {
        Ok(id) => app.opencode_active_id = id,
        Err(e) => app.set_toast(e, true),
    }
}

pub(super) fn refresh_opencode_detail(app: &mut app::App) {
    let Some(id) = app.opencode_detail_id.clone() else {
        app.opencode_detail = None;
        app.opencode_detail_provider_ids.clear();
        app.opencode_provider_model_ids.clear();
        return;
    };

    match droidgear_core::opencode::get_opencode_profile_for_home(&app.home_dir, &id) {
        Ok(profile) => {
            app.opencode_detail_provider_ids =
                profile.providers.keys().cloned().collect::<Vec<String>>();
            app.opencode_detail_provider_ids
                .sort_by_key(|a| a.to_lowercase());

            if let Some(provider_id) = app.opencode_provider_id.as_deref() {
                app.opencode_provider_model_ids = profile
                    .providers
                    .get(provider_id)
                    .and_then(|p| p.models.as_ref())
                    .map(|m| {
                        let mut ids = m.keys().cloned().collect::<Vec<String>>();
                        ids.sort_by_key(|a| a.to_lowercase());
                        ids
                    })
                    .unwrap_or_default();
            } else {
                app.opencode_provider_model_ids.clear();
            }

            app.opencode_detail = Some(profile);
        }
        Err(e) => {
            app.opencode_detail = None;
            app.opencode_detail_provider_ids.clear();
            app.opencode_provider_model_ids.clear();
            app.set_toast(e, true);
        }
    }
}

pub(super) fn refresh_openclaw(app: &mut app::App) {
    match droidgear_core::openclaw::list_openclaw_profiles_for_home(&app.home_dir) {
        Ok(list) => app.openclaw_profiles = list,
        Err(e) => app.set_toast(e, true),
    }

    if app.openclaw_profiles.is_empty() {
        if let Ok(p) =
            droidgear_core::openclaw::create_default_openclaw_profile_for_home(&app.home_dir)
        {
            app.openclaw_profiles = vec![p]
        }
    }

    match droidgear_core::openclaw::get_active_openclaw_profile_id_for_home(&app.home_dir) {
        Ok(id) => app.openclaw_active_id = id,
        Err(e) => app.set_toast(e, true),
    }
}

pub(super) fn refresh_openclaw_detail(app: &mut app::App) {
    let Some(id) = app.openclaw_detail_id.clone() else {
        app.openclaw_detail = None;
        app.openclaw_detail_provider_ids.clear();
        return;
    };
    match droidgear_core::openclaw::get_openclaw_profile_for_home(&app.home_dir, &id) {
        Ok(profile) => {
            app.openclaw_detail_provider_ids =
                profile.providers.keys().cloned().collect::<Vec<String>>();
            app.openclaw_detail_provider_ids
                .sort_by_key(|a| a.to_lowercase());
            app.openclaw_detail = Some(profile);
        }
        Err(e) => {
            app.openclaw_detail = None;
            app.openclaw_detail_provider_ids.clear();
            app.set_toast(e, true);
        }
    }
}

pub(super) fn refresh_openclaw_subagents(app: &mut app::App) {
    match droidgear_core::openclaw::read_openclaw_subagents_for_home(&app.home_dir) {
        Ok(list) => app.openclaw_subagents = list,
        Err(e) => app.set_toast(e, true),
    }
}

pub(super) fn refresh_pi(app: &mut app::App) {
    match droidgear_core::pi::list_pi_profiles_for_home(&app.home_dir) {
        Ok(list) => app.pi_profiles = list,
        Err(e) => app.set_toast(e, true),
    }

    if app.pi_profiles.is_empty() {
        if let Ok(p) = droidgear_core::pi::create_default_pi_profile_for_home(&app.home_dir) {
            app.pi_profiles = vec![p];
        }
    }

    match droidgear_core::pi::get_active_pi_profile_id_for_home(&app.home_dir) {
        Ok(id) => app.pi_active_id = id,
        Err(e) => app.set_toast(e, true),
    }
}

pub(super) fn refresh_pi_detail(app: &mut app::App) {
    let Some(id) = app.pi_detail_id.clone() else {
        app.pi_detail = None;
        return;
    };
    match droidgear_core::pi::get_pi_profile_for_home(&app.home_dir, &id) {
        Ok(profile) => {
            app.pi_detail = Some(profile);
        }
        Err(e) => {
            app.pi_detail = None;
            app.set_toast(e, true);
        }
    }
}

pub(super) fn pi_load_from_live_config(app: &mut app::App, profile_id: &str) -> anyhow::Result<()> {
    let live = droidgear_core::pi::read_pi_current_config_for_home(&app.home_dir)
        .map_err(anyhow::Error::msg)?;
    let mut profile = droidgear_core::pi::get_pi_profile_for_home(&app.home_dir, profile_id)
        .map_err(anyhow::Error::msg)?;
    profile.providers = live.providers;
    droidgear_core::pi::save_pi_profile_for_home(&app.home_dir, profile)
        .map_err(anyhow::Error::msg)?;
    Ok(())
}

pub(super) fn refresh_sessions(app: &mut app::App) {
    match droidgear_core::sessions::list_sessions_for_home(&app.home_dir, None) {
        Ok(list) => app.sessions = list,
        Err(e) => app.set_toast(e, true),
    }
}

pub(super) fn refresh_specs(app: &mut app::App) {
    match droidgear_core::specs::list_specs_for_home(&app.home_dir) {
        Ok(list) => app.specs = list,
        Err(e) => app.set_toast(e, true),
    }
}

pub(super) fn refresh_channels(app: &mut app::App) {
    match droidgear_core::channel::load_channels_for_home(&app.home_dir) {
        Ok(list) => app.channels = list,
        Err(e) => app.set_toast(e, true),
    }
}

pub(super) fn refresh_missions(app: &mut app::App) {
    match droidgear_core::factory_settings::get_mission_model_settings_for_home(&app.home_dir) {
        Ok(settings) => app.mission_settings = settings,
        Err(e) => app.set_toast(e, true),
    }
    // Also load custom models for model selection
    match droidgear_core::factory_settings::load_custom_models_for_home(&app.home_dir) {
        Ok(models) => app.custom_models = models,
        Err(e) => app.set_toast(e, true),
    }
}

pub(super) fn refresh_hermes(app: &mut app::App) {
    match droidgear_core::hermes::list_hermes_profiles_for_home(&app.home_dir) {
        Ok(list) => app.hermes_profiles = list,
        Err(e) => app.set_toast(e, true),
    }

    if app.hermes_profiles.is_empty() {
        if let Ok(p) = droidgear_core::hermes::create_default_hermes_profile_for_home(&app.home_dir)
        {
            app.hermes_profiles = vec![p];
        }
    }

    match droidgear_core::hermes::get_active_hermes_profile_id_for_home(&app.home_dir) {
        Ok(id) => app.hermes_active_id = id,
        Err(e) => app.set_toast(e, true),
    }
}

pub(super) fn refresh_hermes_detail(app: &mut app::App) {
    let Some(id) = app.hermes_detail_id.clone() else {
        app.hermes_detail = None;
        return;
    };
    match droidgear_core::hermes::get_hermes_profile_for_home(&app.home_dir, &id) {
        Ok(profile) => {
            app.hermes_detail = Some(profile);
        }
        Err(e) => {
            app.hermes_detail = None;
            app.set_toast(e, true);
        }
    }
}

pub(super) fn hermes_load_from_live_config(
    app: &mut app::App,
    profile_id: &str,
) -> anyhow::Result<()> {
    let live = droidgear_core::hermes::read_hermes_current_config_for_home(&app.home_dir)
        .map_err(anyhow::Error::msg)?;
    let mut profile =
        droidgear_core::hermes::get_hermes_profile_for_home(&app.home_dir, profile_id)
            .map_err(anyhow::Error::msg)?;
    profile.model = live.model;
    droidgear_core::hermes::save_hermes_profile_for_home(&app.home_dir, profile)
        .map_err(anyhow::Error::msg)?;
    Ok(())
}
