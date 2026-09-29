//! GitHub Copilot CLI profile and local configuration management.
//!
//! Copilot's CLI currently reads the provider settings from its process
//! environment.  DroidGear keeps those settings in profiles below
//! `~/.droidgear/copilot/` and can materialize the selected profile as a
//! local `config.env` file without touching the user's Copilot installation.

use chrono::Utc;
use serde::{Deserialize, Serialize};
use specta::Type;
use std::collections::HashMap;
use std::io::Write;
use std::path::{Path, PathBuf};
use uuid::Uuid;

use crate::channel::ChannelType;
use crate::factory_settings::Provider;
use crate::{paths, storage};

const COPILOT_DIR: &str = "copilot";
const CONFIG_FILE: &str = "config.env";
const ACTIVE_PROFILE_FILE: &str = "active-profile.txt";

pub(crate) const COPILOT_PROVIDER_BASE_URL: &str = "COPILOT_PROVIDER_BASE_URL";
pub(crate) const COPILOT_PROVIDER_TYPE: &str = "COPILOT_PROVIDER_TYPE";
pub(crate) const COPILOT_PROVIDER_API_KEY: &str = "COPILOT_PROVIDER_API_KEY";
pub(crate) const COPILOT_MODEL: &str = "COPILOT_MODEL";
pub(crate) const COPILOT_PROVIDER_MAX_PROMPT_TOKENS: &str = "COPILOT_PROVIDER_MAX_PROMPT_TOKENS";
pub(crate) const COPILOT_PROVIDER_MAX_OUTPUT_TOKENS: &str = "COPILOT_PROVIDER_MAX_OUTPUT_TOKENS";

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct CopilotProfile {
    pub id: String,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    pub created_at: String,
    pub updated_at: String,
    #[serde(default, skip_serializing_if = "is_false")]
    pub use_official_auth: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub base_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub provider_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub api_key: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_prompt_tokens: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_output_tokens: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct CopilotConfigStatus {
    pub config_exists: bool,
    pub config_path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct CopilotCurrentConfig {
    pub is_byok: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub base_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub provider_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub api_key: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_prompt_tokens: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_output_tokens: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct CopilotChannelSelection {
    pub channel_type: ChannelType,
    pub base_url: String,
    pub api_key: String,
    pub platform: Option<String>,
    pub provider: Option<Provider>,
    pub model: String,
    pub max_output_tokens: Option<u64>,
}

pub fn supports_channel_platform(platform: Option<&str>) -> bool {
    matches!(
        platform.unwrap_or("").to_lowercase().as_str(),
        "" | "openai" | "anthropic" | "claude" | "grok" | "deepseek"
    )
}

/// Fill an editable profile using a channel selection without applying it.
pub fn prepare_channel_import(
    mut profile: CopilotProfile,
    selection: CopilotChannelSelection,
) -> Result<CopilotProfile, String> {
    if !supports_channel_platform(selection.platform.as_deref()) {
        return Err(
            "This channel platform does not expose a supported OpenAI or Anthropic API".to_string(),
        );
    }
    let platform = selection.platform.as_deref().unwrap_or("").to_lowercase();
    let uses_model_protocol = matches!(
        selection.channel_type,
        ChannelType::NewApi | ChannelType::CliProxyApi | ChannelType::General
    );
    let anthropic = match selection.provider {
        Some(Provider::Anthropic) => true,
        Some(_) => false,
        None if !uses_model_protocol
            && matches!(platform.as_str(), "anthropic" | "claude" | "grok") =>
        {
            true
        }
        None if !uses_model_protocol && matches!(platform.as_str(), "openai" | "deepseek") => false,
        None => selection.model.to_lowercase().starts_with("claude-"),
    };
    profile.use_official_auth = false;
    profile.provider_type = Some(if anthropic { "anthropic" } else { "openai" }.to_string());
    profile.base_url = Some(selection.base_url);
    profile.api_key = Some(selection.api_key);
    profile.model = Some(selection.model);
    profile.max_prompt_tokens = None;
    profile.max_output_tokens = selection.max_output_tokens;
    normalize_profile(profile)
}

pub(crate) fn normalize_profile(mut profile: CopilotProfile) -> Result<CopilotProfile, String> {
    validate_profile(&profile)?;
    if profile.use_official_auth {
        return Ok(profile);
    }
    let provider = trim_non_empty(profile.provider_type.as_deref())
        .unwrap_or("openai")
        .to_lowercase();
    if !matches!(provider.as_str(), "openai" | "anthropic") {
        return Err("Copilot supports OpenAI and Anthropic provider types".to_string());
    }
    if let Some(base_url) = trim_non_empty(profile.base_url.as_deref()) {
        let mut url = reqwest::Url::parse(base_url)
            .map_err(|_| "Base URL must be an absolute HTTP(S) URL".to_string())?;
        if !matches!(url.scheme(), "http" | "https") || url.host_str().is_none() {
            return Err("Base URL must be an absolute HTTP(S) URL".to_string());
        }
        let path = url.path().trim_end_matches('/');
        let path = if provider == "openai" && !path.ends_with("/v1") {
            format!("{path}/v1")
        } else if provider == "anthropic" {
            path.strip_suffix("/v1").unwrap_or(path).to_string()
        } else {
            path.to_string()
        };
        url.set_path(&path);
        profile.base_url = Some(
            if url.path() == "/" && url.query().is_none() && url.fragment().is_none() {
                url.to_string().trim_end_matches('/').to_string()
            } else {
                url.to_string()
            },
        );
    }
    profile.provider_type = Some(provider);
    Ok(profile)
}

fn is_false(value: &bool) -> bool {
    !*value
}

fn now_rfc3339() -> String {
    Utc::now().to_rfc3339()
}

fn system_home_dir() -> Result<PathBuf, String> {
    dirs::home_dir().ok_or_else(|| "Failed to get home directory".to_string())
}

fn droidgear_copilot_dir_for_home(home_dir: &Path) -> PathBuf {
    paths::droidgear_dir_from_home(home_dir).join(COPILOT_DIR)
}

pub fn profiles_dir_for_home(home_dir: &Path) -> Result<PathBuf, String> {
    let path = droidgear_copilot_dir_for_home(home_dir).join("profiles");
    std::fs::create_dir_all(&path)
        .map_err(|e| format!("Failed to create copilot profiles directory: {e}"))?;
    Ok(path)
}

fn live_config_path_for_home(home_dir: &Path) -> Result<PathBuf, String> {
    let dir = droidgear_copilot_dir_for_home(home_dir);
    std::fs::create_dir_all(&dir)
        .map_err(|e| format!("Failed to create copilot directory: {e}"))?;
    Ok(dir.join(CONFIG_FILE))
}

fn active_profile_path_for_home(home_dir: &Path) -> Result<PathBuf, String> {
    let dir = droidgear_copilot_dir_for_home(home_dir);
    std::fs::create_dir_all(&dir)
        .map_err(|e| format!("Failed to create copilot directory: {e}"))?;
    Ok(dir.join(ACTIVE_PROFILE_FILE))
}

fn validate_profile_id(id: &str) -> Result<(), String> {
    if !id.is_empty()
        && id
            .chars()
            .all(|value| value.is_ascii_alphanumeric() || value == '-' || value == '_')
    {
        Ok(())
    } else {
        Err("Invalid profile id".to_string())
    }
}

fn profile_path_for_home(home_dir: &Path, id: &str) -> Result<PathBuf, String> {
    validate_profile_id(id)?;
    Ok(profiles_dir_for_home(home_dir)?.join(format!("{id}.json")))
}

fn profile_env_path_for_home(home_dir: &Path, id: &str) -> Result<PathBuf, String> {
    validate_profile_id(id)?;
    Ok(profiles_dir_for_home(home_dir)?.join(format!("{id}.env")))
}

fn read_profile_file(path: &Path) -> Result<CopilotProfile, String> {
    let content =
        std::fs::read_to_string(path).map_err(|e| format!("Failed to read profile: {e}"))?;
    serde_json::from_str(&content).map_err(|e| format!("Invalid profile JSON: {e}"))
}

fn write_profile_file(home_dir: &Path, profile: &CopilotProfile) -> Result<(), String> {
    let path = profile_path_for_home(home_dir, &profile.id)?;
    let content = serde_json::to_string_pretty(profile)
        .map_err(|e| format!("Failed to serialize profile JSON: {e}"))?;
    write_private_file(&path, &content)
}

fn load_profile_by_id(home_dir: &Path, id: &str) -> Result<CopilotProfile, String> {
    let profile = read_profile_file(&profile_path_for_home(home_dir, id)?)?;
    if profile.id != id {
        return Err("Profile id does not match its file name".to_string());
    }
    Ok(profile)
}

// NamedTempFile creates files with owner-only permissions on Unix and uses a
// unique temporary name so concurrent profile writes cannot collide.
fn write_private_file(path: &Path, contents: &str) -> Result<(), String> {
    let parent = path.parent().ok_or("Missing config directory")?;
    let mut file = tempfile::NamedTempFile::new_in(parent)
        .map_err(|e| format!("Failed to create Copilot config file: {e}"))?;
    file.write_all(contents.as_bytes())
        .map_err(|e| format!("Failed to write Copilot config file: {e}"))?;
    file.persist(path)
        .map_err(|e| format!("Failed to finalize Copilot config file: {e}"))?;
    Ok(())
}

pub(crate) fn validate_profile(profile: &CopilotProfile) -> Result<(), String> {
    if profile.name.trim().is_empty() {
        return Err("Profile name is required".to_string());
    }
    for (field, value) in [
        ("name", Some(profile.name.as_str())),
        ("baseUrl", profile.base_url.as_deref()),
        ("providerType", profile.provider_type.as_deref()),
        ("apiKey", profile.api_key.as_deref()),
        ("model", profile.model.as_deref()),
    ] {
        if value.is_some_and(|value| value.contains(['\n', '\r', '\0'])) {
            return Err(format!(
                "{field} must be a single line without NUL characters"
            ));
        }
    }
    if profile.max_prompt_tokens == Some(0) || profile.max_output_tokens == Some(0) {
        return Err("Token limits must be positive integers".to_string());
    }
    Ok(())
}

fn resolve_profile_by_name<'a>(
    profiles: &'a [CopilotProfile],
    selector: &str,
) -> Result<Option<&'a CopilotProfile>, String> {
    let exact_matches = profiles
        .iter()
        .filter(|profile| profile.name == selector)
        .collect::<Vec<_>>();
    match exact_matches.as_slice() {
        [] => {}
        [profile] => return Ok(Some(profile)),
        _ => {
            return Err(format!(
                "Multiple Copilot profiles share the name '{selector}'. Use the profile index or id instead."
            ));
        }
    }

    let folded_selector = selector.to_lowercase();
    let folded_matches = profiles
        .iter()
        .filter(|profile| profile.name.to_lowercase() == folded_selector)
        .collect::<Vec<_>>();
    match folded_matches.as_slice() {
        [] => Ok(None),
        [profile] => Ok(Some(profile)),
        _ => Err(format!(
            "Multiple Copilot profiles share the name '{selector}'. Use the profile index or id instead."
        )),
    }
}

fn trim_non_empty(value: Option<&str>) -> Option<&str> {
    value.map(str::trim).filter(|value| !value.is_empty())
}

/// Render the profile's BYOK values as a human-editable env file.
///
/// The file deliberately comments out missing values so users can see the
/// supported knobs without accidentally inheriting a stale value.
pub fn render_env_file(profile: &CopilotProfile) -> String {
    if profile.use_official_auth {
        return "# Official GitHub Copilot subscription mode.\nCOPILOT_OFFLINE=false\n".to_string();
    }
    fn line(key: &str, value: Option<&str>) -> String {
        match trim_non_empty(value) {
            Some(value) => format!("{key}={value}"),
            None => format!("# {key}="),
        }
    }

    let profile_line = format!("#  Profile: {} ({})", profile.name, profile.id);

    format!(
        "# ============================================================\n\
#  DroidGear-managed config for the GitHub Copilot BYOK profile\n\
{profile_line}\n\
# ============================================================\n\n\
COPILOT_OFFLINE=true\n\n\
# ---- 模型提供商 API 地址 ----\n\
{}\n\n\
# ---- 提供商类型（openai / anthropic）----\n\
{}\n\n\
# ---- API Key ----\n\
{}\n\n\
# ---- 默认模型名称 ----\n\
{}\n\n\
# ---- 最大 Prompt Token 数 ----\n\
{}\n\n\
# ---- 最大输出 Token 数 ----\n\
{}\n",
        line(COPILOT_PROVIDER_BASE_URL, profile.base_url.as_deref()),
        line(
            COPILOT_PROVIDER_TYPE,
            Some(trim_non_empty(profile.provider_type.as_deref()).unwrap_or("openai"))
        ),
        line(COPILOT_PROVIDER_API_KEY, profile.api_key.as_deref()),
        line(COPILOT_MODEL, profile.model.as_deref()),
        profile
            .max_prompt_tokens
            .map(|value| format!("{COPILOT_PROVIDER_MAX_PROMPT_TOKENS}={value}"))
            .unwrap_or_else(|| format!("# {COPILOT_PROVIDER_MAX_PROMPT_TOKENS}=")),
        profile
            .max_output_tokens
            .map(|value| format!("{COPILOT_PROVIDER_MAX_OUTPUT_TOKENS}={value}"))
            .unwrap_or_else(|| format!("# {COPILOT_PROVIDER_MAX_OUTPUT_TOKENS}=")),
    )
}

fn write_profile_env(home_dir: &Path, profile: &CopilotProfile) -> Result<(), String> {
    let path = profile_env_path_for_home(home_dir, &profile.id)?;
    write_private_file(&path, &render_env_file(profile))
}

fn write_profile_and_env(home_dir: &Path, profile: &CopilotProfile) -> Result<(), String> {
    validate_profile(profile)?;
    write_profile_file(home_dir, profile)?;
    write_profile_env(home_dir, profile)
}

pub fn list_copilot_profiles_for_home(home_dir: &Path) -> Result<Vec<CopilotProfile>, String> {
    let dir = profiles_dir_for_home(home_dir)?;
    let mut profiles = Vec::new();
    for entry in std::fs::read_dir(&dir).map_err(|e| format!("Failed to read profiles dir: {e}"))? {
        let entry = match entry {
            Ok(entry) => entry,
            Err(_) => continue,
        };
        let path = entry.path();
        if path.extension().and_then(|value| value.to_str()) != Some("json") {
            continue;
        }
        if let Ok(profile) = read_profile_file(&path) {
            if path.file_stem().and_then(|name| name.to_str()) != Some(&profile.id) {
                continue;
            }
            profiles.push(profile);
        }
    }
    profiles.sort_by_key(|profile| (profile.name.to_lowercase(), profile.id.clone()));
    Ok(profiles)
}

pub fn get_copilot_profile_for_home(home_dir: &Path, id: &str) -> Result<CopilotProfile, String> {
    load_profile_by_id(home_dir, id)
}

pub fn resolve_copilot_profile_selector_for_home(
    home_dir: &Path,
    selector: &str,
) -> Result<CopilotProfile, String> {
    let selector = selector.trim();
    if selector.is_empty() {
        return Err("Copilot profile selector cannot be empty".to_string());
    }

    let profiles = list_copilot_profiles_for_home(home_dir)?;
    if let Some(profile) = profiles.iter().find(|profile| profile.id == selector) {
        return Ok(profile.clone());
    }
    if let Some(profile) = resolve_profile_by_name(&profiles, selector)? {
        return Ok(profile.clone());
    }
    if let Ok(index) = selector.parse::<usize>() {
        if let Some(profile) = index
            .checked_sub(1)
            .and_then(|zero_based_index| profiles.get(zero_based_index))
        {
            return Ok(profile.clone());
        }
    }

    Err(format!(
        "No Copilot profile matches '{selector}'. Use `droidgear-tui run copilot --list` to inspect available profiles."
    ))
}

pub fn save_copilot_profile_for_home(
    home_dir: &Path,
    profile: CopilotProfile,
) -> Result<(), String> {
    let mut profile = normalize_profile(profile)?;
    if profile.id.trim().is_empty() {
        profile.id = Uuid::new_v4().to_string();
        profile.created_at = now_rfc3339();
    } else if profile_path_for_home(home_dir, &profile.id)?.exists() {
        if let Ok(old) = load_profile_by_id(home_dir, &profile.id) {
            profile.created_at = old.created_at;
        }
    } else if profile.created_at.trim().is_empty() {
        profile.created_at = now_rfc3339();
    }

    profile.updated_at = now_rfc3339();
    write_profile_and_env(home_dir, &profile)
}

pub fn delete_copilot_profile_for_home(home_dir: &Path, id: &str) -> Result<(), String> {
    let path = profile_path_for_home(home_dir, id)?;
    if path.exists() {
        std::fs::remove_file(&path).map_err(|e| format!("Failed to delete profile: {e}"))?;
    }

    let env_path = profile_env_path_for_home(home_dir, id)?;
    if env_path.exists() {
        std::fs::remove_file(&env_path)
            .map_err(|e| format!("Failed to remove profile config.env: {e}"))?;
    }

    if get_active_copilot_profile_id_for_home(home_dir)?.as_deref() == Some(id) {
        let active_path = active_profile_path_for_home(home_dir)?;
        if active_path.exists() {
            std::fs::remove_file(active_path)
                .map_err(|e| format!("Failed to remove active profile id: {e}"))?;
        }
    }
    Ok(())
}

pub fn duplicate_copilot_profile_for_home(
    home_dir: &Path,
    id: &str,
    new_name: &str,
) -> Result<CopilotProfile, String> {
    let mut profile = normalize_profile(load_profile_by_id(home_dir, id)?)?;
    profile.id = Uuid::new_v4().to_string();
    profile.name = new_name.to_string();
    profile.created_at = now_rfc3339();
    profile.updated_at = profile.created_at.clone();
    write_profile_and_env(home_dir, &profile)?;
    Ok(profile)
}

pub fn create_default_copilot_profile_for_home(home_dir: &Path) -> Result<CopilotProfile, String> {
    if !list_copilot_profiles_for_home(home_dir)?.is_empty() {
        return Err("Profiles already exist".to_string());
    }

    let now = now_rfc3339();
    let profile = CopilotProfile {
        id: Uuid::new_v4().to_string(),
        name: "默认".to_string(),
        description: None,
        created_at: now.clone(),
        updated_at: now,
        use_official_auth: false,
        base_url: None,
        provider_type: Some("openai".to_string()),
        api_key: None,
        model: None,
        max_prompt_tokens: None,
        max_output_tokens: None,
    };
    write_profile_and_env(home_dir, &profile)?;
    Ok(profile)
}

pub fn get_active_copilot_profile_id_for_home(home_dir: &Path) -> Result<Option<String>, String> {
    let path = active_profile_path_for_home(home_dir)?;
    if !path.exists() {
        return Ok(None);
    }
    let content = std::fs::read_to_string(&path)
        .map_err(|e| format!("Failed to read active profile id: {e}"))?;
    let id = content.trim().to_string();
    if id.is_empty() {
        Ok(None)
    } else {
        Ok(Some(id))
    }
}

fn set_active_copilot_profile_id_for_home(home_dir: &Path, id: &str) -> Result<(), String> {
    let path = active_profile_path_for_home(home_dir)?;
    storage::atomic_write(&path, id.trim().as_bytes())
}

pub fn apply_copilot_profile_for_home(home_dir: &Path, id: &str) -> Result<(), String> {
    let profile = normalize_profile(load_profile_by_id(home_dir, id)?)?;
    crate::copilot_runtime::build_run_plan(&profile)?;
    let config_path = live_config_path_for_home(home_dir)?;
    write_private_file(&config_path, &render_env_file(&profile))?;
    set_active_copilot_profile_id_for_home(home_dir, id)
}

pub fn get_copilot_config_status_for_home(home_dir: &Path) -> Result<CopilotConfigStatus, String> {
    let path = live_config_path_for_home(home_dir)?;
    Ok(CopilotConfigStatus {
        config_exists: path.exists(),
        config_path: path.to_string_lossy().to_string(),
    })
}

fn get_value(map: &HashMap<String, String>, key: &str) -> Option<String> {
    map.get(key)
        .map(String::as_str)
        .and_then(|value| trim_non_empty(Some(value)))
        .map(ToOwned::to_owned)
}

pub fn read_env_config_from_path(path: &Path) -> Result<CopilotCurrentConfig, String> {
    if !path.exists() {
        return Ok(CopilotCurrentConfig {
            is_byok: false,
            base_url: None,
            provider_type: None,
            api_key: None,
            model: None,
            max_prompt_tokens: None,
            max_output_tokens: None,
        });
    }

    let content =
        std::fs::read_to_string(path).map_err(|e| format!("Failed to read config.env: {e}"))?;
    let mut values = HashMap::new();
    for line in content.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let Some((key, value)) = line.split_once('=') else {
            continue;
        };
        let key = key.trim();
        values.insert(key.to_string(), value.trim().to_string());
    }

    let base_url = get_value(&values, COPILOT_PROVIDER_BASE_URL);
    let provider_type = get_value(&values, COPILOT_PROVIDER_TYPE);
    let api_key = get_value(&values, COPILOT_PROVIDER_API_KEY);
    let model = get_value(&values, COPILOT_MODEL);
    let token_limit = |key| -> Result<Option<u64>, String> {
        get_value(&values, key)
            .map(|value| {
                value
                    .parse::<u64>()
                    .ok()
                    .filter(|value| *value > 0)
                    .ok_or_else(|| format!("{key} must be a positive integer"))
            })
            .transpose()
    };
    let max_prompt_tokens = token_limit(COPILOT_PROVIDER_MAX_PROMPT_TOKENS)?;
    let max_output_tokens = token_limit(COPILOT_PROVIDER_MAX_OUTPUT_TOKENS)?;

    Ok(CopilotCurrentConfig {
        is_byok: get_value(&values, "COPILOT_OFFLINE").as_deref() != Some("false")
            && (get_value(&values, "COPILOT_OFFLINE").as_deref() == Some("true")
                || base_url.is_some()
                || provider_type.is_some()
                || api_key.is_some()
                || model.is_some()),
        base_url,
        provider_type,
        api_key,
        model,
        max_prompt_tokens,
        max_output_tokens,
    })
}

pub fn read_copilot_current_config_for_home(
    home_dir: &Path,
) -> Result<CopilotCurrentConfig, String> {
    let path = live_config_path_for_home(home_dir)?;
    read_env_config_from_path(&path)
}

pub fn profiles_dir() -> Result<PathBuf, String> {
    profiles_dir_for_home(&system_home_dir()?)
}

pub fn list_copilot_profiles() -> Result<Vec<CopilotProfile>, String> {
    list_copilot_profiles_for_home(&system_home_dir()?)
}

pub fn get_copilot_profile(id: &str) -> Result<CopilotProfile, String> {
    get_copilot_profile_for_home(&system_home_dir()?, id)
}

pub fn resolve_copilot_profile_selector(selector: &str) -> Result<CopilotProfile, String> {
    resolve_copilot_profile_selector_for_home(&system_home_dir()?, selector)
}

pub fn save_copilot_profile(profile: CopilotProfile) -> Result<(), String> {
    save_copilot_profile_for_home(&system_home_dir()?, profile)
}

pub fn delete_copilot_profile(id: &str) -> Result<(), String> {
    delete_copilot_profile_for_home(&system_home_dir()?, id)
}

pub fn duplicate_copilot_profile(id: &str, new_name: &str) -> Result<CopilotProfile, String> {
    duplicate_copilot_profile_for_home(&system_home_dir()?, id, new_name)
}

pub fn create_default_copilot_profile() -> Result<CopilotProfile, String> {
    create_default_copilot_profile_for_home(&system_home_dir()?)
}

pub fn get_active_copilot_profile_id() -> Result<Option<String>, String> {
    get_active_copilot_profile_id_for_home(&system_home_dir()?)
}

pub fn apply_copilot_profile(id: &str) -> Result<(), String> {
    apply_copilot_profile_for_home(&system_home_dir()?, id)
}

pub fn get_copilot_config_status() -> Result<CopilotConfigStatus, String> {
    get_copilot_config_status_for_home(&system_home_dir()?)
}

pub fn read_copilot_current_config() -> Result<CopilotCurrentConfig, String> {
    read_copilot_current_config_for_home(&system_home_dir()?)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn profile(id: &str, name: &str) -> CopilotProfile {
        CopilotProfile {
            id: id.to_string(),
            name: name.to_string(),
            description: None,
            created_at: "2026-01-01T00:00:00Z".to_string(),
            updated_at: "2026-01-01T00:00:00Z".to_string(),
            use_official_auth: false,
            base_url: Some("https://example.test/v1".to_string()),
            provider_type: Some("openai".to_string()),
            api_key: Some("sk-test".to_string()),
            model: Some("model".to_string()),
            max_prompt_tokens: Some(100),
            max_output_tokens: Some(50),
        }
    }

    #[test]
    fn save_and_apply_normalize_endpoints_by_protocol() {
        for (provider, input, expected) in [
            ("openai", "https://example.test", "https://example.test/v1"),
            (
                "openai",
                "https://example.test/v1/",
                "https://example.test/v1",
            ),
            (
                "openai",
                "https://example.test/gateway/",
                "https://example.test/gateway/v1",
            ),
            (
                "anthropic",
                "https://example.test/v1/",
                "https://example.test",
            ),
            (
                "anthropic",
                "https://example.test/gateway/v1",
                "https://example.test/gateway",
            ),
            (
                "anthropic",
                "https://example.test/gateway",
                "https://example.test/gateway",
            ),
            (
                "openai",
                "https://example.test/gateway?route=/",
                "https://example.test/gateway/v1?route=/",
            ),
        ] {
            let temp = TempDir::new().unwrap();
            let mut value = profile("one", "One");
            value.provider_type = Some(provider.to_string());
            value.base_url = Some(input.to_string());
            save_copilot_profile_for_home(temp.path(), value).unwrap();
            assert_eq!(
                get_copilot_profile_for_home(temp.path(), "one")
                    .unwrap()
                    .base_url
                    .as_deref(),
                Some(expected)
            );
            apply_copilot_profile_for_home(temp.path(), "one").unwrap();
            assert_eq!(
                read_copilot_current_config_for_home(temp.path())
                    .unwrap()
                    .base_url
                    .as_deref(),
                Some(expected)
            );
        }
    }

    #[test]
    fn channel_import_maps_protocol_and_preserves_identity_without_applying() {
        let temp = TempDir::new().unwrap();
        let mut original = profile("one", "Keep name");
        original.description = Some("Keep description".to_string());
        save_copilot_profile_for_home(temp.path(), original.clone()).unwrap();
        apply_copilot_profile_for_home(temp.path(), "one").unwrap();
        let live_path = live_config_path_for_home(temp.path()).unwrap();
        let before = std::fs::read(&live_path).unwrap();
        for (channel_type, platform, provider, model, expected_type) in [
            (
                ChannelType::Sub2Api,
                Some("openai"),
                None,
                "gpt-custom",
                "openai",
            ),
            (
                ChannelType::Sub2Api,
                Some("anthropic"),
                None,
                "custom-name",
                "anthropic",
            ),
            (
                ChannelType::NewApi,
                None,
                None,
                "claude-custom",
                "anthropic",
            ),
            (
                ChannelType::General,
                None,
                Some(Provider::GenericChatCompletionApi),
                "claude-custom",
                "openai",
            ),
            (
                ChannelType::Sub2Api,
                Some("deepseek"),
                Some(Provider::Anthropic),
                "deepseek-chat",
                "anthropic",
            ),
        ] {
            let mut value = original.clone();
            value.use_official_auth = true;
            let imported = prepare_channel_import(
                value,
                CopilotChannelSelection {
                    channel_type,
                    base_url: "https://channel.test/v1".to_string(),
                    api_key: "imported-key".to_string(),
                    platform: platform.map(str::to_string),
                    provider,
                    model: model.to_string(),
                    max_output_tokens: Some(8192),
                },
            )
            .unwrap();
            assert_eq!(imported.id, original.id);
            assert_eq!(imported.name, original.name);
            assert_eq!(imported.description, original.description);
            assert_eq!(imported.created_at, original.created_at);
            assert!(!imported.use_official_auth);
            assert_eq!(imported.provider_type.as_deref(), Some(expected_type));
            assert_eq!(
                imported.base_url.as_deref(),
                Some(if expected_type == "openai" {
                    "https://channel.test/v1"
                } else {
                    "https://channel.test"
                })
            );
            assert_eq!(imported.model.as_deref(), Some(model));
            assert_eq!(imported.api_key.as_deref(), Some("imported-key"));
            assert_eq!(imported.max_prompt_tokens, None);
            assert_eq!(imported.max_output_tokens, Some(8192));
            assert_eq!(
                get_copilot_profile_for_home(temp.path(), "one")
                    .unwrap()
                    .model,
                original.model
            );
            assert_eq!(std::fs::read(&live_path).unwrap(), before);
        }
    }

    #[test]
    fn invalid_provider_and_url_are_rejected_before_writing() {
        let temp = TempDir::new().unwrap();
        let mut value = profile("one", "One");
        value.provider_type = Some("azure".to_string());
        assert!(save_copilot_profile_for_home(temp.path(), value.clone())
            .unwrap_err()
            .contains("OpenAI and Anthropic"));
        value.provider_type = Some("openai".to_string());
        value.base_url = Some("file:///tmp/provider".to_string());
        assert!(save_copilot_profile_for_home(temp.path(), value)
            .unwrap_err()
            .contains("HTTP(S)"));
        assert!(list_copilot_profiles_for_home(temp.path())
            .unwrap()
            .is_empty());
        assert!(!supports_channel_platform(Some("gemini")));
    }

    #[test]
    fn render_env_file_comments_missing_values() {
        let mut value = profile("id", "name");
        value.base_url = None;
        let text = render_env_file(&value);
        assert!(text.contains("# COPILOT_PROVIDER_BASE_URL="));
        assert!(text.contains("COPILOT_MODEL=model"));
    }

    #[test]
    fn profile_crud_and_apply_round_trip() {
        let temp = TempDir::new().unwrap();
        let home = temp.path();
        save_copilot_profile_for_home(home, profile("one", "One")).unwrap();
        apply_copilot_profile_for_home(home, "one").unwrap();
        let current = read_copilot_current_config_for_home(home).unwrap();
        assert!(current.is_byok);
        assert_eq!(current.model.as_deref(), Some("model"));
        assert_eq!(
            get_active_copilot_profile_id_for_home(home)
                .unwrap()
                .as_deref(),
            Some("one")
        );
        let copy = duplicate_copilot_profile_for_home(home, "one", "Two").unwrap();
        assert_ne!(copy.id, "one");
        assert_eq!(copy.api_key.as_deref(), Some("sk-test"));
        assert_eq!(
            resolve_copilot_profile_selector_for_home(home, "two")
                .unwrap()
                .id,
            copy.id
        );
        assert_eq!(
            resolve_copilot_profile_selector_for_home(home, "1")
                .unwrap()
                .id,
            "one"
        );
        delete_copilot_profile_for_home(home, "one").unwrap();
        assert!(get_active_copilot_profile_id_for_home(home)
            .unwrap()
            .is_none());
        assert_eq!(list_copilot_profiles_for_home(home).unwrap().len(), 1);
        assert!(!profile_env_path_for_home(home, "one").unwrap().exists());
    }

    #[test]
    fn saving_and_launching_an_applied_profile_do_not_change_live_config() {
        let temp = TempDir::new().unwrap();
        let home = temp.path();
        let mut value = profile("one", "One");
        save_copilot_profile_for_home(home, value.clone()).unwrap();
        apply_copilot_profile_for_home(home, "one").unwrap();
        let live_path = live_config_path_for_home(home).unwrap();
        let before = std::fs::read(&live_path).unwrap();
        value.model = Some("another-model".to_string());
        save_copilot_profile_for_home(home, value.clone()).unwrap();
        let plan = crate::copilot_runtime::build_run_plan(&value).unwrap();
        assert!(plan
            .env
            .contains(&(COPILOT_MODEL.to_string(), "another-model".to_string())));
        assert_eq!(std::fs::read(live_path).unwrap(), before);
        assert_eq!(
            read_env_config_from_path(&profile_env_path_for_home(home, "one").unwrap())
                .unwrap()
                .model,
            value.model
        );
    }

    #[test]
    fn official_config_removes_stored_byok_credentials_from_export() {
        let temp = TempDir::new().unwrap();
        let home = temp.path();
        let mut value = profile("one", "One");
        save_copilot_profile_for_home(home, value.clone()).unwrap();
        apply_copilot_profile_for_home(home, "one").unwrap();
        value.use_official_auth = true;
        save_copilot_profile_for_home(home, value).unwrap();
        apply_copilot_profile_for_home(home, "one").unwrap();
        let current = read_copilot_current_config_for_home(home).unwrap();
        assert!(!current.is_byok);
        assert!(current.api_key.is_none());
        assert!(current.model.is_none());
        let contents = std::fs::read_to_string(live_config_path_for_home(home).unwrap()).unwrap();
        assert!(!contents.contains("sk-test"));
    }

    #[test]
    fn env_values_are_literal_and_newlines_cannot_inject_settings() {
        let temp = TempDir::new().unwrap();
        let mut value = profile("one", "One");
        value.api_key = Some("literal=$value!#\"'&=tail".to_string());
        save_copilot_profile_for_home(temp.path(), value.clone()).unwrap();
        apply_copilot_profile_for_home(temp.path(), "one").unwrap();
        assert_eq!(
            read_copilot_current_config_for_home(temp.path())
                .unwrap()
                .api_key,
            value.api_key
        );
        value.api_key = Some("secret\nCOPILOT_MODEL=injected".to_string());
        let error = save_copilot_profile_for_home(temp.path(), value).unwrap_err();
        assert!(error.contains("apiKey"));
        assert!(!error.contains("secret"));
        assert!(get_copilot_profile_for_home(temp.path(), "../config").is_err());
    }

    #[cfg(unix)]
    #[test]
    fn profile_and_env_files_are_private() {
        use std::os::unix::fs::PermissionsExt;
        let temp = TempDir::new().unwrap();
        save_copilot_profile_for_home(temp.path(), profile("one", "One")).unwrap();
        apply_copilot_profile_for_home(temp.path(), "one").unwrap();
        for path in [
            profile_path_for_home(temp.path(), "one").unwrap(),
            profile_env_path_for_home(temp.path(), "one").unwrap(),
            live_config_path_for_home(temp.path()).unwrap(),
        ] {
            assert_eq!(
                std::fs::metadata(path).unwrap().permissions().mode() & 0o777,
                0o600
            );
        }
    }
}
