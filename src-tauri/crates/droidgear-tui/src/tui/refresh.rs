use super::*;

pub(super) fn refresh_paths(app: &mut app::App) {
    match droidgear_core::paths::get_effective_paths_for_home(&app.home_dir) {
        Ok(p) => app.paths = Some(p),
        Err(e) => app.set_toast(e, true),
    }
}

pub(super) fn refresh_droid_settings_files(app: &mut app::App) {
    match droidgear_core::droid_settings_files::list_settings_files_for_home(&app.home_dir) {
        Ok(files) => {
            app.droid_settings_files = files;
            if app.droid_settings_files_index >= app.droid_settings_files.len() {
                app.droid_settings_files_index = app.droid_settings_files.len().saturating_sub(1);
            }
        }
        Err(e) => app.set_toast(e, true),
    }
}

pub(super) fn refresh_trusted_folders(app: &mut app::App) {
    match droidgear_core::trusted_folders::list_trusted_folders_for_home(&app.home_dir) {
        Ok(folders) => {
            app.trusted_folders = folders;
            let folder_paths: Vec<String> = app
                .trusted_folders
                .iter()
                .map(|folder| folder.path.clone())
                .collect();
            app.trusted_folders_selected
                .retain(|path| folder_paths.iter().any(|folder_path| folder_path == path));
            if app.trusted_folders_index >= app.trusted_folders.len() {
                app.trusted_folders_index = app.trusted_folders.len().saturating_sub(1);
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
    match droidgear_core::factory_settings::get_model_favorites_for_home(&app.home_dir) {
        Ok(favorites) => app.model_favorites = favorites,
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
    match droidgear_core::claude_settings_files::list_settings_files_for_home(&app.home_dir) {
        Ok(list) => app.claude_files = list,
        Err(e) => app.set_toast(e, true),
    }
}

pub(super) fn refresh_claude_detail(app: &mut app::App) {
    let Some(name) = app.claude_detail_name.clone() else {
        app.claude_detail_json = None;
        return;
    };
    match droidgear_core::claude_settings_files::read_settings_file_for_home(&app.home_dir, &name) {
        Ok(json) => app.claude_detail_json = Some(json),
        Err(e) => {
            app.claude_detail_json = None;
            app.set_toast(e, true);
        }
    }
}

pub(super) fn claude_load_from_live_config(app: &mut app::App, name: &str) -> anyhow::Result<()> {
    droidgear_core::claude_settings_files::load_settings_file_from_live_for_home(
        &app.home_dir,
        name,
    )
    .map_err(anyhow::Error::msg)?;
    Ok(())
}

pub(super) fn refresh_codex(app: &mut app::App) {
    match droidgear_core::codex::list_codex_profiles_for_home(&app.home_dir) {
        Ok(list) => app.codex_profiles = list,
        Err(e) => app.set_toast(e, true),
    }

    let has_profiles = !app.codex_profiles.is_empty();
    if !has_profiles
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

pub(super) fn refresh_copilot(app: &mut app::App) {
    match droidgear_core::copilot::list_copilot_profiles_for_home(&app.home_dir) {
        Ok(list) => app.copilot_profiles = list,
        Err(e) => app.set_toast(e, true),
    }

    if app.copilot_profiles.is_empty() {
        if let Ok(profile) =
            droidgear_core::copilot::create_default_copilot_profile_for_home(&app.home_dir)
        {
            app.copilot_profiles = vec![profile];
        }
    }

    match droidgear_core::copilot::get_active_copilot_profile_id_for_home(&app.home_dir) {
        Ok(id) => app.copilot_active_id = id,
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
    droidgear_core::codex::save_codex_profile_for_home_and_apply_if_active(&app.home_dir, profile)
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
    profile.api_key_model_discovery = live.api_key_model_discovery;
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

pub(super) fn refresh_omp(app: &mut app::App) {
    match droidgear_core::omp::list_omp_profiles_for_home(&app.home_dir) {
        Ok(list) => app.omp_profiles = list,
        Err(e) => app.set_toast(e, true),
    }

    if app.omp_profiles.is_empty() {
        if let Ok(p) = droidgear_core::omp::create_default_omp_profile_for_home(&app.home_dir) {
            app.omp_profiles = vec![p];
        }
    }

    match droidgear_core::omp::get_active_omp_profile_id_for_home(&app.home_dir) {
        Ok(id) => app.omp_active_id = id,
        Err(e) => app.set_toast(e, true),
    }
}

pub(super) fn refresh_omp_detail(app: &mut app::App) {
    let Some(id) = app.omp_detail_id.clone() else {
        app.omp_detail = None;
        return;
    };
    match droidgear_core::omp::get_omp_profile_for_home(&app.home_dir, &id) {
        Ok(profile) => {
            app.omp_detail = Some(profile);
        }
        Err(e) => {
            app.omp_detail = None;
            app.set_toast(e, true);
        }
    }
}

pub(super) fn refresh_omp_custom_providers(app: &mut app::App) {
    match droidgear_core::omp::read_omp_custom_providers_for_home(&app.home_dir) {
        Ok(config) => {
            app.omp_custom_providers = config.providers;
            if app.omp_provider_index >= app.omp_custom_providers.len() {
                app.omp_provider_index = app.omp_custom_providers.len().saturating_sub(1);
            }
        }
        Err(e) => app.set_toast(e, true),
    }
}
pub(super) fn refresh_dsh(app: &mut app::App) {
    // The active profile follows the desktop/web screen the user opened; the
    // profile is set when the screen is entered from the feature list.
    match droidgear_core::dsh::read_dsh_current_config_for_profile(
        &app.home_dir,
        app.dsh_active_profile.as_deref(),
    ) {
        Ok(config) => {
            let mut providers: Vec<(String, droidgear_core::dsh::DshProviderConfig)> =
                config.providers.into_iter().collect();
            providers.sort_by_key(|(id, _)| id.to_lowercase());
            app.dsh_providers = providers;
        }
        Err(e) => {
            app.dsh_providers.clear();
            app.set_toast(e, true);
        }
    }

    match droidgear_core::dsh::read_dsh_credentials_for_home(&app.home_dir) {
        Ok(credentials) => app.dsh_credentials = credentials.refs,
        Err(e) => {
            app.dsh_credentials.clear();
            app.set_toast(e, true);
        }
    }

    // The official DeepSeek route references its key through
    // `llm-deepseek.config.apiKeyEnv`; show whether that ref is configured.
    let official_env = resolve_dsh_official_api_key_env(app);
    app.dsh_official_api_key_configured = app
        .dsh_credentials
        .get(&official_env)
        .is_some_and(|value| !value.trim().is_empty());
    app.dsh_official_api_key_env = official_env;

    // Drop the open provider if it was removed on disk.
    if let Some(provider_id) = app.dsh_provider_id.clone() {
        if !app.dsh_providers.iter().any(|(id, _)| id == &provider_id) {
            app.dsh_provider_id = None;
        }
    }
}

pub(super) fn refresh_sessions(app: &mut app::App) {
    match droidgear_core::sessions::list_sessions_for_home(&app.home_dir, None) {
        Ok(list) => app.sessions = list,
        Err(e) => app.set_toast(e, true),
    }
}

pub(super) fn refresh_codex_sessions(app: &mut app::App) {
    match droidgear_core::codex_sessions::list_codex_sessions_for_home(&app.home_dir) {
        Ok(list) => app.codex_sessions = list,
        Err(e) => app.set_toast(e, true),
    }
}

pub(super) fn refresh_pi_sessions(app: &mut app::App) {
    match droidgear_core::pi_sessions::list_pi_sessions_for_home(&app.home_dir) {
        Ok(list) => app.pi_sessions = list,
        Err(e) => app.set_toast(e, true),
    }
}

/// Fallback credential ref for the official DeepSeek route when the composed
/// config does not expose one.
pub(super) const DSH_OFFICIAL_API_KEY_ENV_FALLBACK: &str = "DEEPSEEK_API_KEY";

/// Resolve the credential reference of the official DeepSeek route from the
/// composed snapshot: `llm-deepseek.config.apiKeyEnv`, then the schema field
/// value/default, then [`DSH_OFFICIAL_API_KEY_ENV_FALLBACK`]. Never reads the
/// secret value itself.
pub(super) fn resolve_dsh_official_api_key_env(app: &app::App) -> String {
    let Ok(snapshot) = droidgear_core::dsh_config::read_dsh_config_snapshot_for_home(
        &app.home_dir,
        app.dsh_active_profile.clone(),
    ) else {
        return DSH_OFFICIAL_API_KEY_ENV_FALLBACK.to_string();
    };
    let Some(entry) = snapshot
        .entries
        .iter()
        .find(|entry| entry.id == "llm-deepseek")
    else {
        return DSH_OFFICIAL_API_KEY_ENV_FALLBACK.to_string();
    };
    if let Some(value) = entry
        .config
        .as_ref()
        .and_then(|config| config.get("apiKeyEnv"))
        .and_then(serde_json::Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
    {
        return value.to_string();
    }
    entry
        .fields
        .iter()
        .find(|field| field.name == "apiKeyEnv")
        .and_then(|field| field.value.as_ref().or(field.default.as_ref()))
        .and_then(serde_json::Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .unwrap_or(DSH_OFFICIAL_API_KEY_ENV_FALLBACK)
        .to_string()
}

pub(super) fn refresh_dsh_sessions(app: &mut app::App) {
    match droidgear_core::dsh_sessions::list_dsh_sessions_for_home(&app.home_dir) {
        Ok(list) => app.dsh_sessions = list,
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
    profile.models = live.models;
    droidgear_core::hermes::save_hermes_profile_for_home(&app.home_dir, profile)
        .map_err(anyhow::Error::msg)?;
    Ok(())
}

/// 在 profile 末尾追加一条空模型配置（列表为空时自动设为默认），
/// 并跳到 HermesProvider 屏幕编辑它。
pub(super) fn hermes_add_model(app: &mut app::App, profile_id: &str) -> anyhow::Result<()> {
    let mut profile =
        droidgear_core::hermes::get_hermes_profile_for_home(&app.home_dir, profile_id)
            .map_err(anyhow::Error::msg)?;
    let is_default = profile.models.is_empty();
    profile
        .models
        .push(droidgear_core::hermes::HermesModelConfig {
            name: None,
            default: None,
            provider: None,
            base_url: None,
            api_key: None,
            is_default,
        });
    droidgear_core::hermes::save_hermes_profile_for_home(&app.home_dir, profile)
        .map_err(anyhow::Error::msg)?;
    refresh_hermes_detail(app);
    app.hermes_model_index = app
        .hermes_detail
        .as_ref()
        .map(|p| p.models.len().saturating_sub(1))
        .unwrap_or(0);
    app.hermes_provider_field_index = 0;
    app.screen = app::Screen::HermesProvider;
    Ok(())
}

/// 把指定下标条目设为默认（其余清除）。
pub(super) fn hermes_set_default_model(
    app: &mut app::App,
    profile_id: &str,
    model_index: usize,
) -> anyhow::Result<()> {
    let mut profile =
        droidgear_core::hermes::get_hermes_profile_for_home(&app.home_dir, profile_id)
            .map_err(anyhow::Error::msg)?;
    if model_index >= profile.models.len() {
        return Err(anyhow::anyhow!("Model config not found"));
    }
    for (i, entry) in profile.models.iter_mut().enumerate() {
        entry.is_default = i == model_index;
    }
    droidgear_core::hermes::save_hermes_profile_for_home(&app.home_dir, profile)
        .map_err(anyhow::Error::msg)?;
    refresh_hermes_detail(app);
    Ok(())
}

// ============================================================================
// DSH configuration screen (`cordis.patch.yml`)
// ============================================================================
//
// Composing a snapshot reads the profile manifest plus every bundle patch, so
// it runs on a worker thread instead of the draw/event loop. The result is
// dropped into `app.dsh_config_refresh_slot` and applied by
// `poll_dsh_config_refresh` on the next UI tick.

/// Start a background DSH configuration read for the active profile.
pub(super) fn request_dsh_config_refresh(app: &mut app::App) {
    if app.dsh_config_loading {
        return;
    }
    let slot: app::DshConfigRefreshSlot = std::sync::Arc::new(std::sync::Mutex::new(None));
    let thread_slot = std::sync::Arc::clone(&slot);
    let home_dir = app.home_dir.clone();
    let profile = app.dsh_active_profile.clone();
    let spawned = std::thread::Builder::new()
        .name("dsh-config-refresh".to_string())
        .spawn(move || {
            let result =
                droidgear_core::dsh_config::read_dsh_config_snapshot_for_home(&home_dir, profile);
            let mut guard = thread_slot
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner());
            *guard = Some(result);
        });
    match spawned {
        Ok(_) => {
            app.dsh_config_refresh_slot = Some(slot);
            app.dsh_config_loading = true;
        }
        Err(error) => app.set_toast(format!("Failed to start DSH config refresh: {error}"), true),
    }
}

/// Apply a background read that has completed; no-op while it is running.
pub(super) fn poll_dsh_config_refresh(app: &mut app::App) {
    let Some(slot) = app.dsh_config_refresh_slot.as_ref() else {
        return;
    };
    let result = slot
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
        .take();
    let Some(result) = result else {
        return;
    };
    app.dsh_config_refresh_slot = None;
    app.dsh_config_loading = false;
    match result {
        Ok(snapshot) => apply_dsh_config_snapshot(app, snapshot),
        Err(error) => app.set_toast(error, true),
    }
}

fn apply_dsh_config_snapshot(
    app: &mut app::App,
    snapshot: droidgear_core::dsh_config::DshConfigSnapshot,
) {
    app.dsh_config_index = app
        .dsh_config_index
        .min(snapshot.entries.len().saturating_sub(1));
    app.dsh_config_expanded
        .retain(|id| snapshot.entries.iter().any(|entry| &entry.id == id));
    app.dsh_config_snapshot = Some(snapshot);
    let field_count = app
        .dsh_config_selected_entry()
        .map(|entry| entry.fields.len())
        .unwrap_or(0);
    if app.dsh_config_field_index >= field_count {
        app.dsh_config_field_index = field_count.saturating_sub(1);
    }
}
