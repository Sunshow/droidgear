//! Dsh (DeepSeek Harness) configuration management (core).
//!
//! Modern Dsh (>= 0.1.x) stores its configuration as *profiles*: each profile
//! lives in `~/.dsh/profiles/<name>/` and its user-editable layer is the
//! `cordis.patch.yml` file — a YAML sequence of loader patch entries
//! (`- id: <entry>`, `name: <package>`, `config: {...}`). DroidGear manages
//! the `llm-pi-ai` entry's `config.providers` map: provider id → provider
//! config (displayName, baseURL, apiKeyEnv, api, compat, models).
//!
//! Reading merges the layers Dsh itself composes (lowest first):
//! legacy `~/.dsh/settings.yaml` (still present only until a modern Dsh
//! imports it), the selected profile's `cordis.patch.yml`, and the
//! home-level `~/.dsh/cordis.patch.yml` overlay. Writing targets the selected
//! profile's patch file; when no profile exists yet, it falls back to the
//! legacy `~/.dsh/settings.yaml` so older Dsh installs keep working.
//!
//! The whole patch document is round-tripped as YAML, so all other entries
//! (`ui-settings-general`, `agent-default-model`, `locale`, `permission`,
//! `ui-conversation`, ...) and the template header comments are preserved.
//! Unknown fields inside provider and model entries are kept via
//! `#[serde(flatten)]` extras.

use serde::{Deserialize, Serialize};
use serde_yaml::Value as YamlValue;
use specta::Type;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::time::Duration;

use crate::storage;

/// The patch entry id (and legacy settings.yaml section) DroidGear manages.
pub const DSH_LLM_SECTION: &str = "llm-pi-ai";

/// The package name recorded on the `llm-pi-ai` patch entry.
pub const DSH_LLM_ENTRY_NAME: &str = "@deepseek-ai/dsh-llm-pi-ai";

/// The subsection inside the `llm-pi-ai` entry config that holds the provider map.
pub const DSH_PROVIDERS_KEY: &str = "providers";

/// Directory under the Dsh home that holds all profiles.
pub const DSH_PROFILES_DIR_NAME: &str = "profiles";

/// The official Dsh runtime profiles, in default preference order. Only
/// these are managed; user-created plugin profiles are out of scope.
pub const DSH_OFFICIAL_PROFILES: [&str; 2] = ["desktop", "web"];

/// User patch layer filename inside a profile dir (and at the Dsh home root).
pub const DSH_PATCH_FILENAME: &str = "cordis.patch.yml";

/// Legacy flat settings document, read by Dsh only for its one-time import.
pub const DSH_LEGACY_SETTINGS_FILENAME: &str = "settings.yaml";

// ============================================================================
// Types
// ============================================================================

/// Dsh provider compatibility flags. Unknown fields are retained.
#[derive(Debug, Clone, Default, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct DshCompatConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub supports_developer_role: Option<bool>,
    #[serde(flatten)]
    #[specta(skip)]
    pub extra: HashMap<String, serde_json::Value>,
}

/// Dsh model definition inside a provider.
#[derive(Debug, Clone, Default, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct DshModel {
    pub id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub context_window: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_tokens: Option<u32>,
    /// Reasoning effort mapping, e.g. `{"off": null, "high": "high"}`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reasoning_efforts: Option<HashMap<String, Option<String>>>,
    #[serde(flatten)]
    #[specta(skip)]
    pub extra: HashMap<String, serde_json::Value>,
}

/// Dsh provider configuration (one entry of `llm-pi-ai.providers`).
#[derive(Debug, Clone, Default, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct DshProviderConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub display_name: Option<String>,
    /// The file uses `baseURL` (not `baseUrl`).
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "baseURL")]
    pub base_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub api_key_env: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub api: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub compat: Option<DshCompatConfig>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub models: Vec<DshModel>,
    #[serde(flatten)]
    #[specta(skip)]
    pub extra: HashMap<String, serde_json::Value>,
}

/// Current Dsh configuration (the merged provider map across config layers).
#[derive(Debug, Clone, Default, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct DshCurrentConfig {
    #[serde(default)]
    pub providers: HashMap<String, DshProviderConfig>,
}

/// One Dsh profile directory under `~/.dsh/profiles/`.
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct DshProfile {
    /// Profile name (directory name under `~/.dsh/profiles/`).
    pub name: String,
    /// Absolute profile directory.
    pub dir: String,
    /// Absolute path of the profile's user patch layer (`cordis.patch.yml`).
    pub patch_path: String,
}

/// Dsh configuration file status.
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct DshConfigStatus {
    /// Resolved profile name (`None` when no profile exists and the legacy
    /// `settings.yaml` layout is used).
    pub profile_name: Option<String>,
    pub config_exists: bool,
    /// Effective file DroidGear writes providers into: the profile's
    /// `cordis.patch.yml`, or the legacy `settings.yaml` without a profile.
    pub config_path: String,
    /// Whether the legacy `~/.dsh/settings.yaml` still exists (modern Dsh
    /// imports it once and renames it to `settings.yaml.imported`).
    pub legacy_exists: bool,
    pub legacy_path: String,
    pub credentials_exists: bool,
    pub credentials_path: String,
}

/// Credentials read from `~/.dsh/.credentials.yaml` (version + env refs).
#[derive(Debug, Clone, Default, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct DshCredentials {
    #[serde(default = "default_credentials_version")]
    pub version: u32,
    /// Environment variable name → API key value.
    #[serde(default)]
    pub refs: HashMap<String, String>,
    #[serde(flatten)]
    #[specta(skip)]
    pub extra: HashMap<String, serde_json::Value>,
}

fn default_credentials_version() -> u32 {
    1
}

// ============================================================================
// Path Helpers
// ============================================================================

/// `~/.dsh/`
pub fn dsh_config_dir_for_home(home_dir: &Path) -> PathBuf {
    home_dir.join(".dsh")
}

/// `~/.dsh/settings.yaml` (legacy flat layout, pre-0.1.x).
pub fn dsh_settings_path_for_home(home_dir: &Path) -> PathBuf {
    dsh_config_dir_for_home(home_dir).join(DSH_LEGACY_SETTINGS_FILENAME)
}

/// `~/.dsh/profiles/`
pub fn dsh_profiles_dir_for_home(home_dir: &Path) -> PathBuf {
    dsh_config_dir_for_home(home_dir).join(DSH_PROFILES_DIR_NAME)
}

/// `~/.dsh/profiles/<name>/`
pub fn dsh_profile_dir_for_home(home_dir: &Path, name: &str) -> PathBuf {
    dsh_profiles_dir_for_home(home_dir).join(name)
}

/// `~/.dsh/profiles/<name>/cordis.patch.yml`
pub fn dsh_profile_patch_path_for_home(home_dir: &Path, name: &str) -> PathBuf {
    dsh_profile_dir_for_home(home_dir, name).join(DSH_PATCH_FILENAME)
}

/// `~/.dsh/cordis.patch.yml` (home-level patch overlay applied after every
/// profile patch layer).
pub fn dsh_home_patch_path_for_home(home_dir: &Path) -> PathBuf {
    dsh_config_dir_for_home(home_dir).join(DSH_PATCH_FILENAME)
}

fn system_home_dir() -> Result<PathBuf, String> {
    dirs::home_dir().ok_or_else(|| "Failed to get home directory".to_string())
}

pub fn dsh_settings_path() -> Result<PathBuf, String> {
    Ok(dsh_settings_path_for_home(&system_home_dir()?))
}

fn validate_provider_id(id: &str) -> Result<(), String> {
    let ok = id
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_');
    if ok && !id.is_empty() {
        Ok(())
    } else {
        Err("Invalid provider id".to_string())
    }
}

// ============================================================================
// Profile Discovery / Resolution
// ============================================================================

/// List the official Dsh runtime profiles (`desktop`, `web`) under
/// `~/.dsh/profiles/`, in that order.
///
/// A directory counts as a profile when it carries the profile manifest
/// (`package.json`) or a user patch layer (`cordis.patch.yml`). Directories
/// with other names (user-created plugin profiles) are ignored; an
/// unreadable or missing profiles directory yields an empty list.
pub fn list_dsh_profiles_for_home(home_dir: &Path) -> Result<Vec<DshProfile>, String> {
    let dir = dsh_profiles_dir_for_home(home_dir);
    if !dir.exists() {
        return Ok(Vec::new());
    }
    let mut profiles = Vec::new();
    for name in DSH_OFFICIAL_PROFILES {
        let profile_dir = dsh_profile_dir_for_home(home_dir, name);
        let patch_path = dsh_profile_patch_path_for_home(home_dir, name);
        let is_profile = profile_dir.join("package.json").exists()
            || profile_dir.join(DSH_PATCH_FILENAME).exists();
        if is_profile {
            profiles.push(DshProfile {
                name: name.to_string(),
                dir: profile_dir.to_string_lossy().to_string(),
                patch_path: patch_path.to_string_lossy().to_string(),
            });
        }
    }
    Ok(profiles)
}

/// Resolve the official profile DroidGear manages.
///
/// Only `desktop` and `web` are accepted; an explicit `requested` name must
/// be one of them and must exist, otherwise an error is returned. Without a
/// request the default is `desktop`, then `web`. Returns `None` when neither
/// official profile exists (legacy `settings.yaml` layout).
pub fn resolve_dsh_profile_for_home(
    home_dir: &Path,
    requested: Option<&str>,
) -> Result<Option<String>, String> {
    if let Some(name) = requested {
        if !DSH_OFFICIAL_PROFILES.contains(&name) {
            return Err(format!(
                "Dsh profile '{name}' is not an official runtime profile ({})",
                DSH_OFFICIAL_PROFILES.join(" / ")
            ));
        }
        let profile_dir = dsh_profile_dir_for_home(home_dir, name);
        if !profile_dir.join("package.json").exists()
            && !profile_dir.join(DSH_PATCH_FILENAME).exists()
        {
            return Err(format!("Dsh profile '{name}' does not exist"));
        }
        return Ok(Some(name.to_string()));
    }
    let profiles = list_dsh_profiles_for_home(home_dir)?;
    Ok(profiles.first().map(|p| p.name.clone()))
}

// ============================================================================
// DroidGear Model Registry (reasoningEfforts auto-adaptation)
// ============================================================================

const MODEL_REGISTRY_JSON: &str = include_str!("../../../../src/lib/model-registry-data.json");

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct DshRegistryReasoningConfig {
    #[serde(default)]
    efforts: Vec<String>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct DshRegistryModel {
    id: String,
    name: String,
    #[serde(default)]
    aliases: Vec<String>,
    #[serde(default)]
    thinking_level_map: Option<HashMap<String, Option<String>>>,
    #[serde(default)]
    reasoning_config: Option<DshRegistryReasoningConfig>,
    context_window: u32,
    max_output_tokens: Option<u32>,
}

fn registry_models() -> &'static [DshRegistryModel] {
    static MODELS: std::sync::OnceLock<Vec<DshRegistryModel>> = std::sync::OnceLock::new();
    MODELS.get_or_init(|| serde_json::from_str(MODEL_REGISTRY_JSON).unwrap_or_default())
}

/// Build Dsh `reasoningEfforts` for a registry entry: always starts with
/// `off: null` (reasoning disabled), then maps each whitelisted effort from
/// `reasoningConfig.efforts` to itself (`high: high`, `max: max`).
///
/// The registry's `thinkingLevelMap` uses Pi-style level names (minimal/low/
/// medium/...) which do not belong in Dsh's `reasoningEfforts`; it is only a
/// fallback for entries without a whitelist, with `off` forced in.
fn registry_reasoning_efforts(
    metadata: &DshRegistryModel,
) -> Option<HashMap<String, Option<String>>> {
    if let Some(config) = &metadata.reasoning_config {
        if !config.efforts.is_empty() {
            let mut efforts = HashMap::new();
            efforts.insert("off".to_string(), None);
            for effort in &config.efforts {
                if effort != "none" && !effort.is_empty() {
                    efforts.insert(effort.clone(), Some(effort.clone()));
                }
            }
            return Some(efforts);
        }
    }

    let mut efforts = metadata.thinking_level_map.clone()?;
    efforts.entry("off".to_string()).or_insert(None);
    Some(efforts)
}

/// Auto-adapt a Dsh model from the DroidGear registry by model id/alias:
/// fills `reasoningEfforts`, `name`, `contextWindow` and `maxTokens` from the
/// registry entry when the user has not set them explicitly.
///
/// Returns true when a registry entry matched.
pub fn enrich_dsh_model_from_registry(model: &mut DshModel) -> bool {
    let Some(metadata) = registry_models()
        .iter()
        .find(|entry| entry.id == model.id || entry.aliases.iter().any(|alias| alias == &model.id))
    else {
        return false;
    };

    if model.name.is_none() {
        model.name = Some(metadata.name.clone());
    }
    if model.context_window.is_none() {
        model.context_window = Some(metadata.context_window);
    }
    if model.max_tokens.is_none() {
        if let Some(max_tokens) = metadata.max_output_tokens {
            model.max_tokens = Some(max_tokens);
        }
    }
    if model.reasoning_efforts.is_none()
        || model.reasoning_efforts.as_ref() == metadata.thinking_level_map.as_ref()
    {
        model.reasoning_efforts = registry_reasoning_efforts(metadata);
    }
    true
}

// ============================================================================
// YAML Read/Write
// ============================================================================

fn empty_mapping() -> YamlValue {
    YamlValue::Mapping(serde_yaml::Mapping::new())
}

fn read_settings_yaml_for_home(home_dir: &Path) -> Result<YamlValue, String> {
    let path = dsh_settings_path_for_home(home_dir);
    if !path.exists() {
        return Ok(empty_mapping());
    }
    let s = std::fs::read_to_string(&path)
        .map_err(|e| format!("Failed to read Dsh settings file: {e}"))?;
    if s.trim().is_empty() {
        return Ok(empty_mapping());
    }
    let value: YamlValue =
        serde_yaml::from_str(&s).map_err(|e| format!("Invalid Dsh settings YAML: {e}"))?;
    if value.is_mapping() {
        Ok(value)
    } else {
        Err("Dsh settings.yaml root must be a mapping".to_string())
    }
}

fn write_settings_yaml_for_home(home_dir: &Path, value: &YamlValue) -> Result<(), String> {
    let path = dsh_settings_path_for_home(home_dir);
    let dir = dsh_config_dir_for_home(home_dir);
    if !dir.exists() {
        std::fs::create_dir_all(&dir)
            .map_err(|e| format!("Failed to create Dsh directory: {e}"))?;
    }
    let yaml_str = serde_yaml::to_string(value)
        .map_err(|e| format!("Failed to serialize Dsh settings: {e}"))?;
    storage::atomic_write(&path, yaml_str.as_bytes())
}

// ============================================================================
// Patch Document (`cordis.patch.yml`) Read/Write
// ============================================================================

/// Leading block-comment lines of a document (template header comments), or
/// an empty string when the document starts with content.
fn leading_comment_block(original: &str) -> String {
    let mut block = String::new();
    for line in original.lines() {
        let trimmed = line.trim_start();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            block.push_str(line);
            block.push('\n');
        } else {
            break;
        }
    }
    block
}

/// Read a patch document: a YAML sequence of loader patch entry mappings.
///
/// Returns the parsed entries plus the raw text (for comment preservation).
/// A missing or empty file yields an empty document; `null` and `~` roots
/// (empty patch layers) are also accepted as empty.
fn read_patch_document(path: &Path) -> Result<(Vec<YamlValue>, String), String> {
    if !path.exists() {
        return Ok((Vec::new(), String::new()));
    }
    let s =
        std::fs::read_to_string(path).map_err(|e| format!("Failed to read Dsh patch file: {e}"))?;
    if s.trim().is_empty() {
        return Ok((Vec::new(), String::new()));
    }
    let value: YamlValue =
        serde_yaml::from_str(&s).map_err(|e| format!("Invalid Dsh patch YAML: {e}"))?;
    match value {
        YamlValue::Sequence(entries) => Ok((entries, s)),
        YamlValue::Null => Ok((Vec::new(), s)),
        _ => Err("Dsh patch file root must be a sequence".to_string()),
    }
}

/// Write a patch document while preserving untouched entry blocks verbatim.
///
/// The document is split into its header and per-entry text blocks. Entries
/// other than the `llm-pi-ai` one keep their original text, so `!!js`
/// expressions (which serde_yaml cannot round-trip) survive. The `llm-pi-ai`
/// entry itself is re-serialized from its parsed value; a raw block that
/// contains a `!!` tag token is refused instead of being corrupted. Empty
/// documents (`[]`, `~`, missing file) keep only their leading comments.
fn write_patch_document(path: &Path, original: &str, entries: &[YamlValue]) -> Result<(), String> {
    let (raw_header, mut blocks) = split_patch_document(original);
    // A save may append a fresh llm-pi-ai entry after reading: the entry
    // list is then one longer than the preserved blocks.
    let appended_llm = entries.len() == blocks.len() + 1
        && entries.last().and_then(patch_entry_id) == Some(DSH_LLM_SECTION);
    if !appended_llm && entries.len() != blocks.len() {
        return Err(
            "Dsh patch layout could not be matched for safe rewriting; edit the file manually"
                .to_string(),
        );
    }
    let header = if blocks.is_empty() {
        // Missing, empty, `[]` or `~` document: nothing but comments to keep.
        leading_comment_block(original)
    } else {
        raw_header
    };
    if appended_llm {
        blocks.push(String::new());
    }

    let Some(index) = find_llm_entry(entries) else {
        return Ok(());
    };
    if !blocks[index].is_empty() && blocks[index].contains("!!") {
        return Err(format!(
            "Dsh patch entry '{DSH_LLM_SECTION}' contains tagged values (`!!...`) that cannot be rewritten safely; edit the file manually"
        ));
    }
    blocks[index] = serialize_patch_entry(&entries[index])?;

    if let Some(parent) = path.parent() {
        if !parent.exists() {
            std::fs::create_dir_all(parent)
                .map_err(|e| format!("Failed to create Dsh profile directory: {e}"))?;
        }
    }
    storage::atomic_write(path, format!("{header}{}", blocks.concat()).as_bytes())
}

/// Serialize a patch entry mapping as a sequence item: first line prefixed
/// with `- `, continuation lines indented two spaces.
fn serialize_patch_entry(entry: &YamlValue) -> Result<String, String> {
    let body = serde_yaml::to_string(entry)
        .map_err(|e| format!("Failed to serialize Dsh patch entry: {e}"))?;
    let mut out = String::from("- ");
    for (i, line) in body.lines().enumerate() {
        if i > 0 {
            out.push('\n');
            out.push_str("  ");
        }
        out.push_str(line);
    }
    out.push('\n');
    Ok(out)
}

/// Split a patch document into its header (everything before the first
/// column-0 sequence item, comments and blank lines included) and one raw
/// text block per top-level sequence item.
fn split_patch_document(raw: &str) -> (String, Vec<String>) {
    let mut header = String::new();
    let mut blocks = Vec::new();
    let mut current = String::new();
    let mut in_entries = false;
    for line in raw.lines() {
        let is_item = line == "-" || line.starts_with("- ") || line.starts_with("-\t");
        if is_item {
            in_entries = true;
            if !current.is_empty() {
                blocks.push(std::mem::take(&mut current));
            }
            current.push_str(line);
            current.push('\n');
        } else if in_entries {
            current.push_str(line);
            current.push('\n');
        } else {
            header.push_str(line);
            header.push('\n');
        }
    }
    if !current.is_empty() {
        blocks.push(current);
    }
    (header, blocks)
}

/// The `id` of a patch entry, if it is a mapping with a string id.
fn patch_entry_id(entry: &YamlValue) -> Option<&str> {
    entry.as_mapping()?.get("id")?.as_str()
}

/// Index of the `llm-pi-ai` entry in a patch document, if present.
fn find_llm_entry(entries: &[YamlValue]) -> Option<usize> {
    entries
        .iter()
        .position(|entry| patch_entry_id(entry) == Some(DSH_LLM_SECTION))
}

/// A fresh `llm-pi-ai` patch entry with an empty config mapping.
fn new_llm_entry() -> YamlValue {
    let mut entry = serde_yaml::Mapping::new();
    entry.insert(
        YamlValue::String("id".to_string()),
        YamlValue::String(DSH_LLM_SECTION.to_string()),
    );
    entry.insert(
        YamlValue::String("name".to_string()),
        YamlValue::String(DSH_LLM_ENTRY_NAME.to_string()),
    );
    entry.insert(YamlValue::String("config".to_string()), empty_mapping());
    YamlValue::Mapping(entry)
}

/// Refuse writes that would re-serialize an `llm-pi-ai` entry whose raw
/// block contains `!!` tag tokens (serde_yaml cannot round-trip them).
fn ensure_llm_entry_rewritable(original: &str, entries: &[YamlValue]) -> Result<(), String> {
    let Some(index) = find_llm_entry(entries) else {
        return Ok(());
    };
    let (_, blocks) = split_patch_document(original);
    if blocks.get(index).is_some_and(|block| block.contains("!!")) {
        return Err(format!(
            "Dsh patch entry '{DSH_LLM_SECTION}' contains tagged values (`!!...`) that cannot be rewritten safely; edit the file manually"
        ));
    }
    Ok(())
}

/// Extract `llm-pi-ai.config.providers` from a patch document.
///
/// A missing file or entry yields an empty map. Errors when the providers
/// value is not a mapping or a provider entry cannot be deserialized.
fn read_providers_from_patch(path: &Path) -> Result<HashMap<String, DshProviderConfig>, String> {
    let (entries, _) = read_patch_document(path)?;
    let Some(index) = find_llm_entry(&entries) else {
        return Ok(HashMap::new());
    };
    let entry = &entries[index];
    let Some(config) = entry.as_mapping().and_then(|m| m.get("config")) else {
        return Ok(HashMap::new());
    };
    let Some(providers) = config.as_mapping().and_then(|m| m.get(DSH_PROVIDERS_KEY)) else {
        return Ok(HashMap::new());
    };
    let Some(map) = providers.as_mapping() else {
        return Err(format!(
            "{DSH_LLM_SECTION}.config.{DSH_PROVIDERS_KEY} must be a mapping"
        ));
    };
    parse_provider_map(map)
}

/// Extract `llm-pi-ai.providers` from the legacy `settings.yaml` document.
fn read_providers_from_legacy(
    home_dir: &Path,
) -> Result<HashMap<String, DshProviderConfig>, String> {
    let path = dsh_settings_path_for_home(home_dir);
    if !path.exists() {
        return Ok(HashMap::new());
    }
    let root = read_settings_yaml_for_home(home_dir)?;
    let Some(section) = root.get(DSH_LLM_SECTION) else {
        return Ok(HashMap::new());
    };
    let Some(providers) = section.get(DSH_PROVIDERS_KEY) else {
        return Ok(HashMap::new());
    };
    let Some(map) = providers.as_mapping() else {
        return Err(format!(
            "{DSH_LLM_SECTION}.{DSH_PROVIDERS_KEY} must be a mapping"
        ));
    };
    parse_provider_map(map)
}

/// Deserialize a provider id → config YAML mapping.
fn parse_provider_map(
    map: &serde_yaml::Mapping,
) -> Result<HashMap<String, DshProviderConfig>, String> {
    let mut result = HashMap::new();
    for (key, value) in map {
        let Some(provider_id) = key.as_str() else {
            continue;
        };
        let config: DshProviderConfig = serde_yaml::from_value(value.clone())
            .map_err(|e| format!("Invalid Dsh provider '{provider_id}': {e}"))?;
        result.insert(provider_id.to_string(), config);
    }
    Ok(result)
}

/// Overlay `over` onto `under` (per-provider insert/update).
fn merge_providers(
    under: &mut HashMap<String, DshProviderConfig>,
    over: HashMap<String, DshProviderConfig>,
) {
    for (id, config) in over {
        under.insert(id, config);
    }
}

/// Read the effective `llm-pi-ai.providers` with the default profile
/// resolution (see [`resolve_dsh_profile_for_home`]).
pub fn read_dsh_current_config_for_home(home_dir: &Path) -> Result<DshCurrentConfig, String> {
    let profile = resolve_dsh_profile_for_home(home_dir, None)?;
    read_dsh_current_config_for_profile(home_dir, profile.as_deref())
}

/// Read the effective `llm-pi-ai.providers` for an explicit profile.
///
/// Layers are merged lowest-first, mirroring Dsh's own composition: legacy
/// `settings.yaml`, the profile's `cordis.patch.yml`, then the home-level
/// `~/.dsh/cordis.patch.yml` overlay.
///
/// Returns an empty config when no layer defines the section. Errors when a
/// document is malformed or a provider entry is not a mapping.
pub fn read_dsh_current_config_for_profile(
    home_dir: &Path,
    profile: Option<&str>,
) -> Result<DshCurrentConfig, String> {
    let mut providers = read_providers_from_legacy(home_dir)?;
    if let Some(name) = profile {
        let patch = dsh_profile_patch_path_for_home(home_dir, name);
        merge_providers(&mut providers, read_providers_from_patch(&patch)?);
    }
    let home_patch = dsh_home_patch_path_for_home(home_dir);
    if home_patch.exists() {
        merge_providers(&mut providers, read_providers_from_patch(&home_patch)?);
    }
    Ok(DshCurrentConfig { providers })
}

/// Prepare a provider config for writing: auto-adapt model metadata from the
/// DroidGear registry and strip `compat.supportsDeveloperRole` on protocols
/// whose routes reject it.
fn prepare_provider_config(config: &DshProviderConfig) -> Result<DshProviderConfig, String> {
    let mut config = config.clone();
    for model in &mut config.models {
        enrich_dsh_model_from_registry(model);
    }

    // `compat.supportsDeveloperRole` only exists on OpenAI-style routes;
    // writing it under other protocols makes DSH reject the provider.
    if !supports_developer_role_protocol(config.api.as_deref()) {
        if let Some(compat) = config.compat.as_mut() {
            compat.supports_developer_role = None;
            if compat.extra.is_empty() {
                config.compat = None;
            }
        }
    }
    Ok(config)
}

/// Insert or update one provider with the default profile resolution (see
/// [`resolve_dsh_profile_for_home`]).
pub fn save_dsh_provider_for_home(
    home_dir: &Path,
    provider_id: &str,
    config: &DshProviderConfig,
) -> Result<(), String> {
    let profile = resolve_dsh_profile_for_home(home_dir, None)?;
    save_dsh_provider_for_profile(home_dir, profile.as_deref(), provider_id, config)
}

/// Insert or update one provider for an explicit profile.
///
/// With a profile, the provider is written into the profile's
/// `cordis.patch.yml` (`llm-pi-ai` entry), preserving every other patch
/// entry and the document's header comments. Without a profile the legacy
/// `~/.dsh/settings.yaml` layout is used, preserving the rest of that
/// document.
pub fn save_dsh_provider_for_profile(
    home_dir: &Path,
    profile: Option<&str>,
    provider_id: &str,
    config: &DshProviderConfig,
) -> Result<(), String> {
    let provider_id = provider_id.trim();
    validate_provider_id(provider_id)?;
    let config = prepare_provider_config(config)?;

    let Some(name) = profile else {
        return save_dsh_provider_legacy(home_dir, provider_id, &config);
    };

    let patch = dsh_profile_patch_path_for_home(home_dir, name);
    let (mut entries, original) = read_patch_document(&patch)?;
    ensure_llm_entry_rewritable(&original, &entries)?;
    let index = match find_llm_entry(&entries) {
        Some(index) => index,
        None => {
            entries.push(new_llm_entry());
            entries.len() - 1
        }
    };

    let entry_map = entries[index]
        .as_mapping_mut()
        .ok_or_else(|| format!("{DSH_LLM_SECTION} patch entry must be a mapping"))?;
    let config_value = entry_map
        .entry(YamlValue::String("config".to_string()))
        .or_insert_with(empty_mapping);
    let config_map = config_value
        .as_mapping_mut()
        .ok_or_else(|| format!("{DSH_LLM_SECTION} config must be a mapping"))?;
    let providers_value = config_map
        .entry(YamlValue::String(DSH_PROVIDERS_KEY.to_string()))
        .or_insert_with(empty_mapping);
    let providers_map = providers_value
        .as_mapping_mut()
        .ok_or_else(|| format!("{DSH_LLM_SECTION}.config.{DSH_PROVIDERS_KEY} must be a mapping"))?;

    let value = serde_yaml::to_value(&config)
        .map_err(|e| format!("Failed to serialize Dsh provider: {e}"))?;
    providers_map.insert(YamlValue::String(provider_id.to_string()), value);

    write_patch_document(&patch, &original, &entries)
}

/// Legacy `settings.yaml` write path used when no profile exists.
fn save_dsh_provider_legacy(
    home_dir: &Path,
    provider_id: &str,
    config: &DshProviderConfig,
) -> Result<(), String> {
    let mut root = read_settings_yaml_for_home(home_dir)?;
    let root_map = root
        .as_mapping_mut()
        .ok_or_else(|| "Dsh settings.yaml root must be a mapping".to_string())?;

    let section_key = YamlValue::String(DSH_LLM_SECTION.to_string());
    let section = root_map.entry(section_key).or_insert_with(empty_mapping);
    let section_map = section
        .as_mapping_mut()
        .ok_or_else(|| format!("{DSH_LLM_SECTION} must be a mapping"))?;

    let providers_key = YamlValue::String(DSH_PROVIDERS_KEY.to_string());
    let providers = section_map
        .entry(providers_key)
        .or_insert_with(empty_mapping);
    let providers_map = providers
        .as_mapping_mut()
        .ok_or_else(|| format!("{DSH_LLM_SECTION}.{DSH_PROVIDERS_KEY} must be a mapping"))?;

    let value = serde_yaml::to_value(config)
        .map_err(|e| format!("Failed to serialize Dsh provider: {e}"))?;
    providers_map.insert(YamlValue::String(provider_id.to_string()), value);

    write_settings_yaml_for_home(home_dir, &root)
}

/// Protocols whose routes accept `compat.supportsDeveloperRole`.
const SUPPORTS_DEV_ROLE_PROTOCOLS: &[&str] = &[
    "openai-completions",
    "openai-responses",
    "azure-openai-responses",
    "openai-codex-responses",
];

/// Whether the given provider `api` protocol accepts `compat.supportsDeveloperRole`.
pub fn supports_developer_role_protocol(api: Option<&str>) -> bool {
    match api {
        Some(api) => SUPPORTS_DEV_ROLE_PROTOCOLS.contains(&api),
        None => false,
    }
}

/// Remove one provider with the default profile resolution (see
/// [`resolve_dsh_profile_for_home`]).
pub fn delete_dsh_provider_for_home(home_dir: &Path, provider_id: &str) -> Result<(), String> {
    let profile = resolve_dsh_profile_for_home(home_dir, None)?;
    delete_dsh_provider_for_profile(home_dir, profile.as_deref(), provider_id)
}

/// Remove one provider for an explicit profile, preserving the rest of the
/// target document (profile `cordis.patch.yml` or legacy `settings.yaml`).
pub fn delete_dsh_provider_for_profile(
    home_dir: &Path,
    profile: Option<&str>,
    provider_id: &str,
) -> Result<(), String> {
    let provider_id = provider_id.trim();
    validate_provider_id(provider_id)?;

    let Some(name) = profile else {
        return delete_dsh_provider_legacy(home_dir, provider_id);
    };

    let patch = dsh_profile_patch_path_for_home(home_dir, name);
    let (mut entries, original) = read_patch_document(&patch)?;
    let Some(index) = find_llm_entry(&entries) else {
        return Ok(());
    };
    ensure_llm_entry_rewritable(&original, &entries)?;
    let entry_map = entries[index]
        .as_mapping_mut()
        .ok_or_else(|| format!("{DSH_LLM_SECTION} patch entry must be a mapping"))?;
    if let Some(config) = entry_map.get_mut(YamlValue::String("config".to_string())) {
        let Some(config_map) = config.as_mapping_mut() else {
            return Err(format!("{DSH_LLM_SECTION} config must be a mapping"));
        };
        if let Some(providers) =
            config_map.get_mut(YamlValue::String(DSH_PROVIDERS_KEY.to_string()))
        {
            let Some(providers_map) = providers.as_mapping_mut() else {
                return Err(format!(
                    "{DSH_LLM_SECTION}.config.{DSH_PROVIDERS_KEY} must be a mapping"
                ));
            };
            providers_map.remove(YamlValue::String(provider_id.to_string()));
        }
    }

    write_patch_document(&patch, &original, &entries)
}

/// Legacy `settings.yaml` removal path used when no profile exists.
fn delete_dsh_provider_legacy(home_dir: &Path, provider_id: &str) -> Result<(), String> {
    let mut root = read_settings_yaml_for_home(home_dir)?;
    let root_map = root
        .as_mapping_mut()
        .ok_or_else(|| "Dsh settings.yaml root must be a mapping".to_string())?;

    if let Some(section) = root_map.get_mut(DSH_LLM_SECTION) {
        let Some(section_map) = section.as_mapping_mut() else {
            return Err(format!("{DSH_LLM_SECTION} must be a mapping"));
        };
        if let Some(providers) = section_map.get_mut(DSH_PROVIDERS_KEY) {
            let Some(providers_map) = providers.as_mapping_mut() else {
                return Err(format!(
                    "{DSH_LLM_SECTION}.{DSH_PROVIDERS_KEY} must be a mapping"
                ));
            };
            providers_map.remove(YamlValue::String(provider_id.to_string()));
        }
    }

    write_settings_yaml_for_home(home_dir, &root)
}

/// Get the Dsh configuration file status with the default profile resolution
/// (see [`resolve_dsh_profile_for_home`]).
pub fn get_dsh_config_status_for_home(home_dir: &Path) -> Result<DshConfigStatus, String> {
    let profile = resolve_dsh_profile_for_home(home_dir, None)?;
    get_dsh_config_status_for_profile(home_dir, profile.as_deref())
}

/// Get the Dsh configuration file status for an explicit profile.
pub fn get_dsh_config_status_for_profile(
    home_dir: &Path,
    profile: Option<&str>,
) -> Result<DshConfigStatus, String> {
    let config_path = match profile {
        Some(name) => dsh_profile_patch_path_for_home(home_dir, name),
        None => dsh_settings_path_for_home(home_dir),
    };
    let legacy_path = dsh_settings_path_for_home(home_dir);
    let credentials_path = dsh_credentials_path_for_home(home_dir);
    Ok(DshConfigStatus {
        profile_name: profile.map(str::to_string),
        config_exists: config_path.exists(),
        config_path: config_path.to_string_lossy().to_string(),
        legacy_exists: legacy_path.exists(),
        legacy_path: legacy_path.to_string_lossy().to_string(),
        credentials_exists: credentials_path.exists(),
        credentials_path: credentials_path.to_string_lossy().to_string(),
    })
}

// ============================================================================
// Credentials (~/.dsh/.credentials.yaml)
// ============================================================================

/// `~/.dsh/.credentials.yaml`
pub fn dsh_credentials_path_for_home(home_dir: &Path) -> PathBuf {
    dsh_config_dir_for_home(home_dir).join(".credentials.yaml")
}

pub fn dsh_credentials_path() -> Result<PathBuf, String> {
    Ok(dsh_credentials_path_for_home(&system_home_dir()?))
}

/// The credentials document layout understood by this build (mirrors DSH).
const CREDENTIALS_VERSION: u32 = 1;

fn validate_credential_name(name: &str) -> Result<(), String> {
    let ok = name
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-');
    if ok && !name.is_empty() {
        Ok(())
    } else {
        Err("Invalid credential env var name".to_string())
    }
}

fn read_credentials_yaml_for_home(home_dir: &Path) -> Result<YamlValue, String> {
    let path = dsh_credentials_path_for_home(home_dir);
    if !path.exists() {
        return Ok(empty_mapping());
    }
    let s = std::fs::read_to_string(&path)
        .map_err(|e| format!("Failed to read Dsh credentials file: {e}"))?;
    if s.trim().is_empty() {
        return Ok(empty_mapping());
    }
    let value: YamlValue =
        serde_yaml::from_str(&s).map_err(|e| format!("Invalid Dsh credentials YAML: {e}"))?;
    let Some(map) = value.as_mapping() else {
        return Err("Dsh credentials root must be a mapping".to_string());
    };

    // Mirror DSH's validation: a non-empty document must carry version: 1 and
    // only known top-level keys (version/refs/records), so DroidGear never
    // silently rewrites a layout the Harness itself would reject.
    if !map.contains_key(YamlValue::String("version".to_string())) {
        return Err(
            "Dsh credentials file uses the pre-release flat layout; add `version: 1` and nest entries under `refs:`"
                .to_string(),
        );
    }
    for key in map.keys() {
        let known = matches!(key.as_str(), Some("version" | "refs" | "records"));
        if !known {
            return Err(format!(
                "Dsh credentials file has unknown top-level key: {key:?}"
            ));
        }
    }
    Ok(value)
}

/// Write the credentials document with owner-only permissions (0600) and an
/// atomic temp-file + rename, so the format DSH expects is never violated.
fn write_credentials_yaml_for_home(home_dir: &Path, value: &YamlValue) -> Result<(), String> {
    let path = dsh_credentials_path_for_home(home_dir);
    let dir = dsh_config_dir_for_home(home_dir);
    if !dir.exists() {
        std::fs::create_dir_all(&dir)
            .map_err(|e| format!("Failed to create Dsh directory: {e}"))?;
    }
    let yaml_str = serde_yaml::to_string(value)
        .map_err(|e| format!("Failed to serialize Dsh credentials: {e}"))?;

    let temp_path = path.with_extension("tmp");
    let mut options = std::fs::OpenOptions::new();
    options.write(true).create(true).truncate(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(0o600);
    }
    std::io::Write::write_all(
        &mut options
            .open(&temp_path)
            .map_err(|e| format!("Failed to create credentials temp file: {e}"))?,
        yaml_str.as_bytes(),
    )
    .map_err(|e| format!("Failed to write credentials temp file: {e}"))?;
    std::fs::rename(&temp_path, &path).map_err(|e| {
        let _ = std::fs::remove_file(&temp_path);
        format!("Failed to finalize credentials file: {e}")
    })
}

/// Read `~/.dsh/.credentials.yaml` (version + refs map).
///
/// Returns an empty document (version 1, no refs) when the file is missing.
/// Errors when the document is malformed, uses an unsupported version, or
/// `refs` is not a string→string mapping.
pub fn read_dsh_credentials_for_home(home_dir: &Path) -> Result<DshCredentials, String> {
    let root = read_credentials_yaml_for_home(home_dir)?;
    let Some(map) = root.as_mapping() else {
        return Err("Dsh credentials root must be a mapping".to_string());
    };

    if let Some(version) = map.get(YamlValue::String("version".to_string())) {
        match version.as_u64() {
            Some(v) if v == u64::from(CREDENTIALS_VERSION) => {}
            _ => return Err(format!("Unsupported Dsh credentials version: {version:?}")),
        }
    }

    let mut refs = HashMap::new();
    if let Some(refs_value) = map.get(YamlValue::String("refs".to_string())) {
        let Some(refs_map) = refs_value.as_mapping() else {
            return Err("Dsh credentials 'refs' must be a mapping".to_string());
        };
        for (key, value) in refs_map {
            let Some(name) = key.as_str() else {
                continue;
            };
            let Some(secret) = value.as_str() else {
                return Err(format!("Dsh credentials ref '{name}' must be a string"));
            };
            refs.insert(name.to_string(), secret.to_string());
        }
    }

    Ok(DshCredentials {
        version: CREDENTIALS_VERSION,
        refs,
        extra: HashMap::new(),
    })
}

/// Insert or update one `refs` entry (env var name → API key value) in
/// `~/.dsh/.credentials.yaml`, preserving the rest of the document (e.g.
/// `records`). An empty value removes the entry.
pub fn save_dsh_credential_ref_for_home(
    home_dir: &Path,
    name: &str,
    value: &str,
) -> Result<(), String> {
    let name = name.trim();
    validate_credential_name(name)?;

    let mut root = read_credentials_yaml_for_home(home_dir)?;
    let root_map = root
        .as_mapping_mut()
        .ok_or_else(|| "Dsh credentials root must be a mapping".to_string())?;

    // Validate version before mutating anything.
    if let Some(version) = root_map.get(YamlValue::String("version".to_string())) {
        match version.as_u64() {
            Some(v) if v == u64::from(CREDENTIALS_VERSION) => {}
            _ => return Err(format!("Unsupported Dsh credentials version: {version:?}")),
        }
    }

    root_map.insert(
        YamlValue::String("version".to_string()),
        YamlValue::Number(CREDENTIALS_VERSION.into()),
    );

    let refs_key = YamlValue::String("refs".to_string());
    let refs = root_map.entry(refs_key).or_insert_with(empty_mapping);
    let refs_map = refs
        .as_mapping_mut()
        .ok_or_else(|| "Dsh credentials 'refs' must be a mapping".to_string())?;

    let name_key = YamlValue::String(name.to_string());
    if value.trim().is_empty() {
        refs_map.remove(name_key);
    } else {
        refs_map.insert(name_key, YamlValue::String(value.to_string()));
    }

    write_credentials_yaml_for_home(home_dir, &root)
}

/// Remove one `refs` entry from `~/.dsh/.credentials.yaml`.
pub fn delete_dsh_credential_ref_for_home(home_dir: &Path, name: &str) -> Result<(), String> {
    let name = name.trim();
    validate_credential_name(name)?;

    let mut root = read_credentials_yaml_for_home(home_dir)?;
    let root_map = root
        .as_mapping_mut()
        .ok_or_else(|| "Dsh credentials root must be a mapping".to_string())?;

    if let Some(refs) = root_map.get_mut(YamlValue::String("refs".to_string())) {
        let Some(refs_map) = refs.as_mapping_mut() else {
            return Err("Dsh credentials 'refs' must be a mapping".to_string());
        };
        refs_map.remove(YamlValue::String(name.to_string()));
    }

    write_credentials_yaml_for_home(home_dir, &root)
}

// ============================================================================
// System wrappers
// ============================================================================

pub fn read_dsh_current_config() -> Result<DshCurrentConfig, String> {
    read_dsh_current_config_for_home(&system_home_dir()?)
}

pub fn read_dsh_current_config_with_profile(
    profile: Option<String>,
) -> Result<DshCurrentConfig, String> {
    read_dsh_current_config_for_profile(&system_home_dir()?, profile.as_deref())
}

pub fn list_dsh_profiles() -> Result<Vec<DshProfile>, String> {
    list_dsh_profiles_for_home(&system_home_dir()?)
}

pub fn save_dsh_provider(provider_id: &str, config: &DshProviderConfig) -> Result<(), String> {
    save_dsh_provider_for_home(&system_home_dir()?, provider_id, config)
}

pub fn save_dsh_provider_with_profile(
    profile: Option<String>,
    provider_id: &str,
    config: &DshProviderConfig,
) -> Result<(), String> {
    save_dsh_provider_for_profile(&system_home_dir()?, profile.as_deref(), provider_id, config)
}

pub fn delete_dsh_provider(provider_id: &str) -> Result<(), String> {
    delete_dsh_provider_for_home(&system_home_dir()?, provider_id)
}

pub fn delete_dsh_provider_with_profile(
    profile: Option<String>,
    provider_id: &str,
) -> Result<(), String> {
    delete_dsh_provider_for_profile(&system_home_dir()?, profile.as_deref(), provider_id)
}

pub fn get_dsh_config_status() -> Result<DshConfigStatus, String> {
    get_dsh_config_status_for_home(&system_home_dir()?)
}

pub fn get_dsh_config_status_with_profile(
    profile: Option<String>,
) -> Result<DshConfigStatus, String> {
    get_dsh_config_status_for_profile(&system_home_dir()?, profile.as_deref())
}

pub fn read_dsh_credentials() -> Result<DshCredentials, String> {
    read_dsh_credentials_for_home(&system_home_dir()?)
}

pub fn save_dsh_credential_ref(name: &str, value: &str) -> Result<(), String> {
    save_dsh_credential_ref_for_home(&system_home_dir()?, name, value)
}

pub fn delete_dsh_credential_ref(name: &str) -> Result<(), String> {
    delete_dsh_credential_ref_for_home(&system_home_dir()?, name)
}

// ============================================================================
// Model Fetching (/{baseURL}/models)
// ============================================================================

const FETCH_MODELS_TIMEOUT: Duration = Duration::from_secs(30);

fn parse_openai_models_list(data: &serde_json::Value) -> Vec<String> {
    data.get("data")
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|m| m.get("id")?.as_str().map(|s| s.to_string()))
                .collect()
        })
        .unwrap_or_default()
}

fn parse_gemini_models_list(data: &serde_json::Value) -> Vec<String> {
    data.get("models")
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|m| {
                    let name = m.get("name")?.as_str()?;
                    Some(name.strip_prefix("models/").unwrap_or(name).to_string())
                })
                .collect()
        })
        .unwrap_or_default()
}

/// Fetch the model list from a provider's `/{baseURL}/models` endpoint using
/// the given API key, and enrich each returned id with registry metadata
/// (name, contextWindow, maxTokens, reasoningEfforts).
pub async fn fetch_dsh_models(
    base_url: &str,
    api_key: &str,
    api: Option<&str>,
) -> Result<Vec<DshModel>, String> {
    let trimmed_base = base_url.trim().trim_end_matches('/');
    if trimmed_base.is_empty() {
        return Err("Base URL is required to fetch models".to_string());
    }
    let key = api_key.trim();
    if key.is_empty() {
        return Err("API key is required to fetch models".to_string());
    }

    let client = reqwest::Client::builder()
        .timeout(FETCH_MODELS_TIMEOUT)
        .build()
        .map_err(|e| format!("Failed to build HTTP client: {e}"))?;

    let api = api.unwrap_or("openai-completions");
    let (url, headers_bearer) = match api {
        "anthropic-messages" => (format!("{trimmed_base}/models"), false),
        "google-generative-ai" => (format!("{trimmed_base}/models?key={key}"), false),
        _ => (format!("{trimmed_base}/models"), true),
    };

    let mut request = client.get(&url);
    if headers_bearer {
        request = request.header("Authorization", format!("Bearer {key}"));
    } else if api == "anthropic-messages" {
        request = request
            .header("x-api-key", key)
            .header("anthropic-version", "2023-06-01");
    }
    let response = request
        .send()
        .await
        .map_err(|e| format!("Failed to fetch models from {url}: {e}"))?;

    let status = response.status();
    if !status.is_success() {
        let body = response.text().await.unwrap_or_default();
        let truncated = if body.len() > 500 {
            format!("{}...", &body[..500])
        } else {
            body
        };
        return Err(format!("API error {status}: {truncated}"));
    }

    let body = response
        .text()
        .await
        .map_err(|e| format!("Failed to read response body: {e}"))?;
    let data: serde_json::Value =
        serde_json::from_str(&body).map_err(|e| format!("Failed to parse models response: {e}"))?;

    let ids = if api == "google-generative-ai" {
        parse_gemini_models_list(&data)
    } else {
        parse_openai_models_list(&data)
    };
    if ids.is_empty() {
        return Err("Provider returned no models".to_string());
    }

    let mut models = Vec::with_capacity(ids.len());
    for id in ids {
        let mut model = DshModel {
            id,
            ..Default::default()
        };
        enrich_dsh_model_from_registry(&mut model);
        models.push(model);
    }
    Ok(models)
}

/// Blocking wrapper around [`fetch_dsh_models`] for the TUI.
pub fn fetch_dsh_models_blocking(
    base_url: &str,
    api_key: &str,
    api: Option<&str>,
) -> Result<Vec<DshModel>, String> {
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .map_err(|e| format!("Failed to create tokio runtime: {e}"))?;
    let base_url = base_url.to_string();
    let api_key = api_key.to_string();
    let api = api.map(|s| s.to_string());
    runtime.block_on(async { fetch_dsh_models(&base_url, &api_key, api.as_deref()).await })
}

// ============================================================================
// Unit Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn home(temp: &TempDir) -> &Path {
        temp.path()
    }

    fn sample_provider() -> DshProviderConfig {
        DshProviderConfig {
            display_name: Some("wududu-deepseek-chat".to_string()),
            base_url: Some("https://sub.wududu.com/v1".to_string()),
            api_key_env: Some("WUDUDU_DEEPSEEK_CHAT_API_KEY".to_string()),
            api: Some("openai-completions".to_string()),
            compat: Some(DshCompatConfig {
                supports_developer_role: Some(false),
                ..Default::default()
            }),
            models: vec![DshModel {
                id: "deepseek-v4-pro".to_string(),
                name: Some("deepseek-v4-pro".to_string()),
                context_window: Some(1000000),
                max_tokens: Some(384000),
                reasoning_efforts: Some(HashMap::from([
                    ("off".to_string(), None),
                    ("high".to_string(), Some("high".to_string())),
                    ("max".to_string(), Some("max".to_string())),
                ])),
                extra: HashMap::new(),
            }],
            ..Default::default()
        }
    }

    #[test]
    fn test_read_current_config_missing_file() {
        let temp = TempDir::new().unwrap();
        let config = read_dsh_current_config_for_home(home(&temp)).unwrap();
        assert!(config.providers.is_empty());
    }

    #[test]
    fn test_save_provider_preserves_other_sections() {
        let temp = TempDir::new().unwrap();
        let existing = r#"ui-onboarding:
  welcomeNoticeVersion: 2026-08-13.1
locale:
  preference: zh
agent-default-model:
  provider: wududu-deepseek-chat
  model: deepseek-v4-pro
  reasoningEffort: max
"#;
        std::fs::create_dir_all(dsh_config_dir_for_home(home(&temp))).unwrap();
        std::fs::write(dsh_settings_path_for_home(home(&temp)), existing).unwrap();

        save_dsh_provider_for_home(home(&temp), "wududu-deepseek-chat", &sample_provider())
            .unwrap();

        let raw = std::fs::read_to_string(dsh_settings_path_for_home(home(&temp))).unwrap();
        assert!(raw.contains("ui-onboarding:"));
        assert!(raw.contains("welcomeNoticeVersion: 2026-08-13.1"));
        assert!(raw.contains("preference: zh"));
        assert!(raw.contains("llm-pi-ai:"));
        assert!(raw.contains("wududu-deepseek-chat:"));

        let config = read_dsh_current_config_for_home(home(&temp)).unwrap();
        assert_eq!(config.providers.len(), 1);
        let provider = &config.providers["wududu-deepseek-chat"];
        assert_eq!(
            provider.display_name.as_deref(),
            Some("wududu-deepseek-chat")
        );
        assert_eq!(
            provider.base_url.as_deref(),
            Some("https://sub.wududu.com/v1")
        );
        assert_eq!(
            provider.api_key_env.as_deref(),
            Some("WUDUDU_DEEPSEEK_CHAT_API_KEY")
        );
        assert_eq!(provider.models.len(), 1);
        assert_eq!(
            provider.models[0].reasoning_efforts.as_ref().unwrap()["off"],
            None
        );
        assert_eq!(
            provider.models[0].reasoning_efforts.as_ref().unwrap()["high"],
            Some("high".to_string())
        );
    }

    #[test]
    fn test_save_provider_creates_file_and_section() {
        let temp = TempDir::new().unwrap();
        save_dsh_provider_for_home(home(&temp), "openai", &sample_provider()).unwrap();

        let config = read_dsh_current_config_for_home(home(&temp)).unwrap();
        assert!(config.providers.contains_key("openai"));
    }

    #[test]
    fn test_delete_provider_preserves_other_providers() {
        let temp = TempDir::new().unwrap();
        save_dsh_provider_for_home(home(&temp), "openai", &sample_provider()).unwrap();
        save_dsh_provider_for_home(home(&temp), "other", &sample_provider()).unwrap();

        delete_dsh_provider_for_home(home(&temp), "openai").unwrap();

        let config = read_dsh_current_config_for_home(home(&temp)).unwrap();
        assert!(!config.providers.contains_key("openai"));
        assert!(config.providers.contains_key("other"));
    }

    #[test]
    fn test_update_existing_provider() {
        let temp = TempDir::new().unwrap();
        save_dsh_provider_for_home(home(&temp), "openai", &sample_provider()).unwrap();

        let mut updated = sample_provider();
        updated.display_name = Some("renamed".to_string());
        save_dsh_provider_for_home(home(&temp), "openai", &updated).unwrap();

        let config = read_dsh_current_config_for_home(home(&temp)).unwrap();
        assert_eq!(config.providers.len(), 1);
        assert_eq!(
            config.providers["openai"].display_name.as_deref(),
            Some("renamed")
        );
    }

    #[test]
    fn test_validate_provider_id() {
        assert!(validate_provider_id("openai").is_ok());
        assert!(validate_provider_id("wududu-deepseek-chat").is_ok());
        assert!(validate_provider_id("").is_err());
        assert!(validate_provider_id("has spaces").is_err());
        assert!(validate_provider_id("has/slash").is_err());
    }

    #[test]
    fn test_parse_openai_models_list() {
        let data = serde_json::json!({
            "data": [
                {"id": "gpt-5.2", "object": "model"},
                {"id": "deepseek-v4-pro"},
                {"object": "model"}
            ]
        });
        let ids = parse_openai_models_list(&data);
        assert_eq!(ids, vec!["gpt-5.2", "deepseek-v4-pro"]);
    }

    #[test]
    fn test_parse_gemini_models_list() {
        let data = serde_json::json!({
            "models": [
                {"name": "models/gemini-2.5-pro"},
                {"name": "gemini-2.5-flash"}
            ]
        });
        let ids = parse_gemini_models_list(&data);
        assert_eq!(ids, vec!["gemini-2.5-pro", "gemini-2.5-flash"]);
    }

    #[test]
    fn test_registry_enriches_reasoning_efforts_and_metadata() {
        let mut model = DshModel {
            id: "gpt-5.2".to_string(),
            ..Default::default()
        };
        assert!(enrich_dsh_model_from_registry(&mut model));
        assert_eq!(model.name.as_deref(), Some("GPT-5.2"));
        assert_eq!(model.context_window, Some(400000));
        assert_eq!(model.max_tokens, Some(128000));
        let efforts = model.reasoning_efforts.unwrap();
        // "off" must always be present and disabled.
        assert_eq!(efforts.get("off"), Some(&None));
        // Whitelist efforts map to themselves; Pi-style level names must not
        // leak into reasoningEfforts.
        assert_eq!(efforts.get("low"), Some(&Some("low".to_string())));
        assert_eq!(efforts.get("medium"), Some(&Some("medium".to_string())));
        assert_eq!(efforts.get("high"), Some(&Some("high".to_string())));
        assert_eq!(efforts.get("xhigh"), Some(&Some("xhigh".to_string())));
        assert!(!efforts.contains_key("minimal"));
    }

    #[test]
    fn test_registry_enrich_uses_whitelist_for_deepseek_v4_flash() {
        let mut model = DshModel {
            id: "deepseek-v4-flash".to_string(),
            ..Default::default()
        };
        assert!(enrich_dsh_model_from_registry(&mut model));
        let efforts = model.reasoning_efforts.unwrap();
        // Registry thinkingLevelMap {minimal: null, medium: null, max: "max"}
        // must NOT leak minimal/medium into reasoningEfforts.
        assert!(!efforts.contains_key("minimal"));
        assert!(!efforts.contains_key("medium"));
        // Whitelist efforts = [none, low, high, max] → off + low/high/max.
        assert_eq!(efforts.get("off"), Some(&None));
        assert_eq!(efforts.get("low"), Some(&Some("low".to_string())));
        assert_eq!(efforts.get("high"), Some(&Some("high".to_string())));
        assert_eq!(efforts.get("max"), Some(&Some("max".to_string())));
    }

    #[test]
    fn test_registry_enrich_replaces_stale_thinking_level_map_copy() {
        // Values written by older versions copied thinkingLevelMap verbatim;
        // when the stored value still equals the registry's map, regenerate
        // it from the whitelist instead of preserving it.
        let stale: HashMap<String, Option<String>> = HashMap::from([
            ("minimal".to_string(), None),
            ("medium".to_string(), None),
            ("max".to_string(), Some("max".to_string())),
        ]);
        let mut model = DshModel {
            id: "deepseek-v4-flash".to_string(),
            reasoning_efforts: Some(stale),
            ..Default::default()
        };
        assert!(enrich_dsh_model_from_registry(&mut model));
        let efforts = model.reasoning_efforts.unwrap();
        assert!(!efforts.contains_key("minimal"));
        assert!(!efforts.contains_key("medium"));
        assert_eq!(efforts.get("off"), Some(&None));
        assert_eq!(efforts.get("high"), Some(&Some("high".to_string())));
    }

    #[test]
    fn test_registry_enrich_does_not_clobber_user_values() {
        let mut model = DshModel {
            id: "gpt-5.2".to_string(),
            name: Some("Custom Name".to_string()),
            reasoning_efforts: Some(HashMap::from([("off".to_string(), None)])),
            ..Default::default()
        };
        assert!(enrich_dsh_model_from_registry(&mut model));
        assert_eq!(model.name.as_deref(), Some("Custom Name"));
        assert_eq!(model.reasoning_efforts.unwrap().len(), 1);
    }

    #[test]
    fn test_registry_enrich_unknown_model_returns_false() {
        let mut model = DshModel {
            id: "totally-unknown-model".to_string(),
            ..Default::default()
        };
        assert!(!enrich_dsh_model_from_registry(&mut model));
        assert!(model.name.is_none());
        assert!(model.reasoning_efforts.is_none());
    }

    #[test]
    fn test_save_provider_auto_enriches_models() {
        let temp = TempDir::new().unwrap();
        let mut provider = sample_provider();
        provider.models = vec![DshModel {
            id: "deepseek-v4-pro".to_string(),
            ..Default::default()
        }];
        save_dsh_provider_for_home(home(&temp), "test", &provider).unwrap();

        let config = read_dsh_current_config_for_home(home(&temp)).unwrap();
        let model = &config.providers["test"].models[0];
        assert_eq!(model.name.as_deref(), Some("DeepSeek V4 Pro"));
        assert_eq!(model.context_window, Some(1000000));
        assert_eq!(model.max_tokens, Some(384000));
        let efforts = model.reasoning_efforts.as_ref().unwrap();
        assert_eq!(efforts.get("off"), Some(&None));
        assert_eq!(efforts.get("high"), Some(&Some("high".to_string())));
        assert_eq!(efforts.get("max"), Some(&Some("max".to_string())));
    }

    #[test]
    fn test_provider_serialization_uses_file_field_names() {
        let value = serde_yaml::to_value(sample_provider()).unwrap();
        assert!(value.get("displayName").is_some());
        assert!(value.get("baseURL").is_some());
        assert!(value.get("apiKeyEnv").is_some());
        assert!(value.get("models").is_some());
        let model = &value["models"][0];
        assert!(model.get("contextWindow").is_some());
        assert!(model.get("maxTokens").is_some());
        assert!(model.get("reasoningEfforts").is_some());
    }

    #[test]
    fn test_unknown_provider_fields_roundtrip() {
        let temp = TempDir::new().unwrap();
        let existing = r#"llm-pi-ai:
  providers:
    custom:
      displayName: custom
      baseURL: https://example.com/v1
      futureProviderField: preserved
      models:
        - id: m1
          futureModelField: also-preserved
"#;
        std::fs::create_dir_all(dsh_config_dir_for_home(home(&temp))).unwrap();
        std::fs::write(dsh_settings_path_for_home(home(&temp)), existing).unwrap();

        let config = read_dsh_current_config_for_home(home(&temp)).unwrap();
        let provider = &config.providers["custom"];
        assert_eq!(
            provider
                .extra
                .get("futureProviderField")
                .and_then(|v| v.as_str()),
            Some("preserved")
        );
        assert_eq!(
            provider.models[0]
                .extra
                .get("futureModelField")
                .and_then(|v| v.as_str()),
            Some("also-preserved")
        );

        // Saving the read provider must keep the unknown fields in the file.
        save_dsh_provider_for_home(home(&temp), "custom", provider).unwrap();
        let raw = std::fs::read_to_string(dsh_settings_path_for_home(home(&temp))).unwrap();
        assert!(raw.contains("futureProviderField: preserved"));
        assert!(raw.contains("futureModelField: also-preserved"));
    }

    #[test]
    fn test_config_status() {
        let temp = TempDir::new().unwrap();
        let status = get_dsh_config_status_for_home(home(&temp)).unwrap();
        assert!(!status.config_exists);
        assert!(!status.credentials_exists);
        assert!(status.config_path.ends_with(".dsh/settings.yaml"));
        assert!(status.credentials_path.ends_with(".dsh/.credentials.yaml"));
        assert_eq!(status.profile_name, None);
        assert!(!status.legacy_exists);

        save_dsh_provider_for_home(home(&temp), "openai", &sample_provider()).unwrap();
        let status = get_dsh_config_status_for_home(home(&temp)).unwrap();
        assert!(status.config_exists);
        assert!(status.legacy_exists);
        assert!(!status.credentials_exists);

        save_dsh_credential_ref_for_home(home(&temp), "OPENAI_API_KEY", "sk-test").unwrap();
        let status = get_dsh_config_status_for_home(home(&temp)).unwrap();
        assert!(status.credentials_exists);
    }

    // =========================================================================
    // Profile / cordis.patch.yml Tests
    // =========================================================================

    /// Create `~/.dsh/profiles/<name>/package.json` so the directory counts
    /// as a Dsh profile.
    fn make_profile(temp: &TempDir, name: &str) -> std::path::PathBuf {
        let dir = dsh_profile_dir_for_home(home(temp), name);
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join("package.json"), "{}").unwrap();
        dir
    }

    const PROFILE_TEMPLATE: &str = "# Your patch layer for this dsh profile, applied after every bundle layer:\n# a top-level YAML array of loader patch entries.\n- id: locale\n  name: \"@deepseek-ai/dsh-client-locale\"\n  config:\n    preference: zh\n";

    #[test]
    fn test_list_and_resolve_profiles() {
        let temp = TempDir::new().unwrap();

        // No profiles directory at all → empty list, no resolution.
        assert!(list_dsh_profiles_for_home(home(&temp)).unwrap().is_empty());
        assert_eq!(
            resolve_dsh_profile_for_home(home(&temp), None).unwrap(),
            None
        );
        assert!(resolve_dsh_profile_for_home(home(&temp), Some("desktop")).is_err());

        make_profile(&temp, "web");
        make_profile(&temp, "desktop");
        // User-created plugin profiles are out of scope and must be ignored.
        make_profile(&temp, "headless");
        make_profile(&temp, "acp");

        let profiles = list_dsh_profiles_for_home(home(&temp)).unwrap();
        let names: Vec<_> = profiles.iter().map(|p| p.name.as_str()).collect();
        assert_eq!(names, vec!["desktop", "web"]);
        assert!(profiles[0].patch_path.ends_with("desktop/cordis.patch.yml"));

        // Default preference: desktop, then web.
        assert_eq!(
            resolve_dsh_profile_for_home(home(&temp), None)
                .unwrap()
                .as_deref(),
            Some("desktop")
        );
        std::fs::remove_dir_all(dsh_profile_dir_for_home(home(&temp), "desktop")).unwrap();
        assert_eq!(
            resolve_dsh_profile_for_home(home(&temp), None)
                .unwrap()
                .as_deref(),
            Some("web")
        );
        assert_eq!(
            resolve_dsh_profile_for_home(home(&temp), Some("web"))
                .unwrap()
                .as_deref(),
            Some("web")
        );
        // Non-official and missing names are rejected.
        assert!(resolve_dsh_profile_for_home(home(&temp), Some("headless")).is_err());
        assert!(resolve_dsh_profile_for_home(home(&temp), Some("missing")).is_err());
        assert!(resolve_dsh_profile_for_home(home(&temp), Some("desktop")).is_err());
    }

    #[test]
    fn test_save_provider_writes_profile_patch_and_preserves_entries() {
        let temp = TempDir::new().unwrap();
        make_profile(&temp, "desktop");
        std::fs::write(
            dsh_profile_patch_path_for_home(home(&temp), "desktop"),
            PROFILE_TEMPLATE,
        )
        .unwrap();

        save_dsh_provider_for_profile(
            home(&temp),
            Some("desktop"),
            "wududu-deepseek-chat",
            &sample_provider(),
        )
        .unwrap();

        let raw = std::fs::read_to_string(dsh_profile_patch_path_for_home(home(&temp), "desktop"))
            .unwrap();
        // Header comments and unrelated entries survive the rewrite.
        assert!(raw.contains("# Your patch layer for this dsh profile"));
        assert!(raw.contains("id: locale"));
        assert!(raw.contains("preference: zh"));
        // The llm-pi-ai entry was appended with the package name.
        assert!(raw.contains("id: llm-pi-ai"));
        assert!(raw.contains("dsh-llm-pi-ai"));
        assert!(raw.contains("wududu-deepseek-chat:"));

        // Legacy settings.yaml must not be created in profile mode.
        assert!(!dsh_settings_path_for_home(home(&temp)).exists());

        let config = read_dsh_current_config_for_profile(home(&temp), Some("desktop")).unwrap();
        assert_eq!(config.providers.len(), 1);
        assert_eq!(
            config.providers["wududu-deepseek-chat"].base_url.as_deref(),
            Some("https://sub.wududu.com/v1")
        );
    }

    #[test]
    fn test_save_provider_appends_to_existing_llm_entry() {
        let temp = TempDir::new().unwrap();
        make_profile(&temp, "desktop");
        let existing = "- id: llm-pi-ai\n  name: \"@deepseek-ai/dsh-llm-pi-ai\"\n  config:\n    providers:\n      existing:\n        displayName: existing\n        baseURL: https://example.com/v1\n";
        std::fs::write(
            dsh_profile_patch_path_for_home(home(&temp), "desktop"),
            existing,
        )
        .unwrap();

        save_dsh_provider_for_profile(home(&temp), Some("desktop"), "new-one", &sample_provider())
            .unwrap();

        let config = read_dsh_current_config_for_profile(home(&temp), Some("desktop")).unwrap();
        assert_eq!(config.providers.len(), 2);
        assert!(config.providers.contains_key("existing"));
        assert!(config.providers.contains_key("new-one"));
    }

    #[test]
    fn test_read_merges_legacy_profile_and_home_layers() {
        let temp = TempDir::new().unwrap();
        make_profile(&temp, "desktop");

        // Legacy layer (lowest).
        let legacy = "llm-pi-ai:\n  providers:\n    legacy-only:\n      displayName: legacy\n      baseURL: https://legacy/v1\n    shared:\n      displayName: from-legacy\n      baseURL: https://legacy/v1\n";
        std::fs::create_dir_all(dsh_config_dir_for_home(home(&temp))).unwrap();
        std::fs::write(dsh_settings_path_for_home(home(&temp)), legacy).unwrap();

        // Profile patch layer.
        let profile_patch = "- id: llm-pi-ai\n  name: \"@deepseek-ai/dsh-llm-pi-ai\"\n  config:\n    providers:\n      profile-only:\n        displayName: profile\n        baseURL: https://profile/v1\n      shared:\n        displayName: from-profile\n        baseURL: https://profile/v1\n";
        std::fs::write(
            dsh_profile_patch_path_for_home(home(&temp), "desktop"),
            profile_patch,
        )
        .unwrap();

        // Home patch layer (highest).
        let home_patch = "- id: llm-pi-ai\n  name: \"@deepseek-ai/dsh-llm-pi-ai\"\n  config:\n    providers:\n      shared:\n        displayName: from-home\n        baseURL: https://home/v1\n";
        std::fs::write(dsh_home_patch_path_for_home(home(&temp)), home_patch).unwrap();

        let config = read_dsh_current_config_for_profile(home(&temp), Some("desktop")).unwrap();
        assert_eq!(config.providers.len(), 3);
        assert!(config.providers.contains_key("legacy-only"));
        assert!(config.providers.contains_key("profile-only"));
        assert_eq!(
            config.providers["shared"].display_name.as_deref(),
            Some("from-home")
        );
    }

    #[test]
    fn test_delete_provider_from_patch() {
        let temp = TempDir::new().unwrap();
        make_profile(&temp, "desktop");
        save_dsh_provider_for_profile(home(&temp), Some("desktop"), "keep", &sample_provider())
            .unwrap();
        let mut other = sample_provider();
        other.display_name = Some("other".to_string());
        save_dsh_provider_for_profile(home(&temp), Some("desktop"), "other", &other).unwrap();

        delete_dsh_provider_for_profile(home(&temp), Some("desktop"), "other").unwrap();

        let config = read_dsh_current_config_for_profile(home(&temp), Some("desktop")).unwrap();
        assert_eq!(config.providers.len(), 1);
        assert!(config.providers.contains_key("keep"));
    }

    #[test]
    fn test_save_provider_without_profile_uses_legacy_layout() {
        let temp = TempDir::new().unwrap();
        save_dsh_provider_for_profile(home(&temp), None, "legacy-provider", &sample_provider())
            .unwrap();

        let raw = std::fs::read_to_string(dsh_settings_path_for_home(home(&temp))).unwrap();
        assert!(raw.contains("llm-pi-ai:"));
        assert!(raw.contains("legacy-provider:"));

        let config = read_dsh_current_config_for_profile(home(&temp), None).unwrap();
        assert!(config.providers.contains_key("legacy-provider"));
    }

    #[test]
    fn test_status_with_profile() {
        let temp = TempDir::new().unwrap();
        make_profile(&temp, "desktop");
        let status = get_dsh_config_status_for_profile(home(&temp), Some("desktop")).unwrap();
        assert_eq!(status.profile_name.as_deref(), Some("desktop"));
        assert!(!status.config_exists);
        assert!(status.config_path.ends_with("desktop/cordis.patch.yml"));
        assert!(!status.legacy_exists);
        assert!(status.legacy_path.ends_with(".dsh/settings.yaml"));

        save_dsh_provider_for_profile(home(&temp), Some("desktop"), "openai", &sample_provider())
            .unwrap();
        let status = get_dsh_config_status_for_profile(home(&temp), Some("desktop")).unwrap();
        assert!(status.config_exists);
    }

    #[test]
    fn test_patch_roundtrip_preserves_tagged_values_in_other_entries() {
        // `!!js` expressions are allowed in patch entries (see the file
        // template); rewriting the document must not corrupt them. Entries
        // DroidGear does not touch are kept verbatim.
        let temp = TempDir::new().unwrap();
        make_profile(&temp, "desktop");
        let with_tag =
            "- id: custom\n  name: \"@deepseek-ai/custom\"\n  config:\n    expr: !!js \"1 + 1\"\n";
        std::fs::write(
            dsh_profile_patch_path_for_home(home(&temp), "desktop"),
            with_tag,
        )
        .unwrap();

        save_dsh_provider_for_profile(home(&temp), Some("desktop"), "openai", &sample_provider())
            .unwrap();

        let raw = std::fs::read_to_string(dsh_profile_patch_path_for_home(home(&temp), "desktop"))
            .unwrap();
        assert!(raw.contains("!!js"), "tagged value lost: {raw}");
        assert!(raw.contains("expr: !!js"));

        // The saved provider must still be readable.
        let config = read_dsh_current_config_for_profile(home(&temp), Some("desktop")).unwrap();
        assert!(config.providers.contains_key("openai"));
    }

    #[test]
    fn test_save_refuses_llm_entry_with_tagged_values() {
        // The llm-pi-ai entry itself must be rewritten on save; when it
        // contains `!!js` tags (which cannot be round-tripped), refuse
        // instead of corrupting the document.
        let temp = TempDir::new().unwrap();
        make_profile(&temp, "desktop");
        let with_tag = "- id: llm-pi-ai\n  name: \"@deepseek-ai/dsh-llm-pi-ai\"\n  config:\n    providers: !!js \"{}\"\n";
        std::fs::write(
            dsh_profile_patch_path_for_home(home(&temp), "desktop"),
            with_tag,
        )
        .unwrap();

        let err = save_dsh_provider_for_profile(
            home(&temp),
            Some("desktop"),
            "openai",
            &sample_provider(),
        )
        .unwrap_err();
        assert!(err.contains("cannot be rewritten safely"));

        // The document is untouched by the refused write.
        let raw = std::fs::read_to_string(dsh_profile_patch_path_for_home(home(&temp), "desktop"))
            .unwrap();
        assert!(raw.contains("!!js"));
    }

    #[test]
    fn test_read_rejects_malformed_yaml() {
        let temp = TempDir::new().unwrap();
        std::fs::create_dir_all(dsh_config_dir_for_home(home(&temp))).unwrap();
        std::fs::write(dsh_settings_path_for_home(home(&temp)), "not: [valid").unwrap();

        let err = read_dsh_current_config_for_home(home(&temp)).unwrap_err();
        assert!(err.contains("Invalid Dsh settings YAML"));
    }

    // =========================================================================
    // Credentials Tests
    // =========================================================================

    #[test]
    fn test_read_credentials_missing_file() {
        let temp = TempDir::new().unwrap();
        let credentials = read_dsh_credentials_for_home(home(&temp)).unwrap();
        assert_eq!(credentials.version, 1);
        assert!(credentials.refs.is_empty());
    }

    #[test]
    fn test_save_credential_ref_creates_version_and_refs() {
        let temp = TempDir::new().unwrap();
        save_dsh_credential_ref_for_home(home(&temp), "OPENAI_API_KEY", "sk-abc").unwrap();

        let raw = std::fs::read_to_string(dsh_credentials_path_for_home(home(&temp))).unwrap();
        assert!(raw.contains("version: 1"));
        assert!(raw.contains("OPENAI_API_KEY: sk-abc"));

        let credentials = read_dsh_credentials_for_home(home(&temp)).unwrap();
        assert_eq!(credentials.refs["OPENAI_API_KEY"], "sk-abc");
    }

    #[test]
    fn test_supports_developer_role_protocol() {
        for api in [
            "openai-completions",
            "openai-responses",
            "azure-openai-responses",
            "openai-codex-responses",
        ] {
            assert!(supports_developer_role_protocol(Some(api)));
        }
        for api in [
            "anthropic-messages",
            "google-generative-ai",
            "openai-chat",
            "unknown",
        ] {
            assert!(!supports_developer_role_protocol(Some(api)));
        }
        assert!(!supports_developer_role_protocol(None));
    }

    #[test]
    fn test_save_strips_supports_developer_role_for_other_protocols() {
        let temp = TempDir::new().unwrap();
        let mut provider = sample_provider();
        provider.api = Some("anthropic-messages".to_string());
        provider.compat = Some(DshCompatConfig {
            supports_developer_role: Some(true),
            extra: HashMap::from([("customFlag".to_string(), serde_json::Value::Bool(true))]),
        });
        save_dsh_provider_for_home(home(&temp), "test", &provider).unwrap();

        let config = read_dsh_current_config_for_home(home(&temp)).unwrap();
        let compat = config.providers["test"].compat.as_ref().unwrap();
        assert!(compat.supports_developer_role.is_none());
        assert_eq!(
            compat.extra.get("customFlag"),
            Some(&serde_json::Value::Bool(true))
        );
    }

    #[test]
    fn test_save_drops_empty_compat_for_other_protocols() {
        let temp = TempDir::new().unwrap();
        let mut provider = sample_provider();
        provider.api = Some("google-generative-ai".to_string());
        provider.compat = Some(DshCompatConfig {
            supports_developer_role: Some(true),
            ..Default::default()
        });
        save_dsh_provider_for_home(home(&temp), "test", &provider).unwrap();

        let config = read_dsh_current_config_for_home(home(&temp)).unwrap();
        assert!(config.providers["test"].compat.is_none());
    }

    #[test]
    fn test_save_keeps_supports_developer_role_for_openai_protocols() {
        let temp = TempDir::new().unwrap();
        let mut provider = sample_provider();
        provider.api = Some("openai-completions".to_string());
        provider.compat = Some(DshCompatConfig {
            supports_developer_role: Some(false),
            ..Default::default()
        });
        save_dsh_provider_for_home(home(&temp), "test", &provider).unwrap();

        let config = read_dsh_current_config_for_home(home(&temp)).unwrap();
        assert_eq!(
            config.providers["test"]
                .compat
                .as_ref()
                .unwrap()
                .supports_developer_role,
            Some(false)
        );
    }

    #[test]
    fn test_save_credential_ref_updates_existing_entry() {
        let temp = TempDir::new().unwrap();
        save_dsh_credential_ref_for_home(home(&temp), "OPENAI_API_KEY", "sk-old").unwrap();
        save_dsh_credential_ref_for_home(home(&temp), "OPENAI_API_KEY", "sk-new").unwrap();

        let credentials = read_dsh_credentials_for_home(home(&temp)).unwrap();
        assert_eq!(credentials.refs.len(), 1);
        assert_eq!(credentials.refs["OPENAI_API_KEY"], "sk-new");
    }

    #[test]
    fn test_save_credential_ref_preserves_records_and_other_refs() {
        let temp = TempDir::new().unwrap();
        let existing = r#"version: 1
refs:
  DEEPSEEK_API_KEY: sk-keep
records:
  some-scope/some-id:
    type: oauth
    value: whatever
"#;
        std::fs::create_dir_all(dsh_config_dir_for_home(home(&temp))).unwrap();
        std::fs::write(dsh_credentials_path_for_home(home(&temp)), existing).unwrap();

        save_dsh_credential_ref_for_home(home(&temp), "OPENAI_API_KEY", "sk-new").unwrap();

        let raw = std::fs::read_to_string(dsh_credentials_path_for_home(home(&temp))).unwrap();
        assert!(raw.contains("DEEPSEEK_API_KEY: sk-keep"));
        assert!(raw.contains("some-scope/some-id:"));
        assert!(raw.contains("OPENAI_API_KEY: sk-new"));

        let credentials = read_dsh_credentials_for_home(home(&temp)).unwrap();
        assert_eq!(credentials.refs.len(), 2);
    }

    #[test]
    fn test_save_credential_ref_with_empty_value_removes_entry() {
        let temp = TempDir::new().unwrap();
        save_dsh_credential_ref_for_home(home(&temp), "OPENAI_API_KEY", "sk-abc").unwrap();
        save_dsh_credential_ref_for_home(home(&temp), "OPENAI_API_KEY", "").unwrap();

        let credentials = read_dsh_credentials_for_home(home(&temp)).unwrap();
        assert!(!credentials.refs.contains_key("OPENAI_API_KEY"));
    }

    #[test]
    fn test_delete_credential_ref() {
        let temp = TempDir::new().unwrap();
        save_dsh_credential_ref_for_home(home(&temp), "OPENAI_API_KEY", "sk-abc").unwrap();
        save_dsh_credential_ref_for_home(home(&temp), "DEEPSEEK_API_KEY", "sk-def").unwrap();

        delete_dsh_credential_ref_for_home(home(&temp), "OPENAI_API_KEY").unwrap();

        let credentials = read_dsh_credentials_for_home(home(&temp)).unwrap();
        assert!(!credentials.refs.contains_key("OPENAI_API_KEY"));
        assert_eq!(credentials.refs["DEEPSEEK_API_KEY"], "sk-def");
    }

    #[test]
    fn test_validate_credential_name() {
        assert!(validate_credential_name("OPENAI_API_KEY").is_ok());
        assert!(validate_credential_name("DEEPSEEK-API-KEY").is_ok());
        assert!(validate_credential_name("").is_err());
        assert!(validate_credential_name("has space").is_err());
    }

    #[test]
    fn test_read_credentials_rejects_unsupported_version() {
        let temp = TempDir::new().unwrap();
        std::fs::create_dir_all(dsh_config_dir_for_home(home(&temp))).unwrap();
        std::fs::write(
            dsh_credentials_path_for_home(home(&temp)),
            "version: 2\nrefs:\n  OPENAI_API_KEY: sk-abc\n",
        )
        .unwrap();

        let err = read_dsh_credentials_for_home(home(&temp)).unwrap_err();
        assert!(err.contains("Unsupported Dsh credentials version"));
    }

    #[cfg(unix)]
    #[test]
    fn test_credentials_file_is_owner_only() {
        use std::os::unix::fs::PermissionsExt;

        let temp = TempDir::new().unwrap();
        save_dsh_credential_ref_for_home(home(&temp), "OPENAI_API_KEY", "sk-abc").unwrap();

        let mode = std::fs::metadata(dsh_credentials_path_for_home(home(&temp)))
            .unwrap()
            .permissions()
            .mode();
        assert_eq!(mode & 0o777, 0o600);
    }
}
