//! Generic DeepSeek Harness profile configuration (`cordis.patch.yml`).
//!
//! DSH 0.2.1 has a single user-writable configuration layer per profile:
//! `~/.dsh/profiles/<profile>/cordis.patch.yml`. It is a YAML sequence of
//! loader patch rows (`- id: <entry>`, `name: <package>`, `config: {...}`,
//! `disabled: true`). A row's `config` **replaces** the whole configuration
//! object of the targeted entry; layers compose in this order (lowest first):
//!
//! 1. the profile's bundle patches (declared by `package.json`
//!    `dsh.profile.bundles`, baked into `res/dsh-config-schema.json`),
//! 2. the profile's `cordis.patch.yml` (the only layer this module writes),
//! 3. the home overlay `~/.dsh/cordis.patch.yml` (read-only for us),
//! 4. command-line `--patch` overlays (not used by the desktop app).
//!
//! Writes mirror DSH's own `config-editor` semantics: a lock file next to
//! `package.json`, a comment- and `!!js`-preserving document rewrite, schema
//! validation, secret redaction, atomic replacement, and a content hash so the
//! UI can refuse to overwrite a configuration changed underneath it.
//!
//! The schema snapshot in `res/dsh-config-schema.json` is generated from the
//! DSH source tree by `scripts/gen-dsh-config-schema.mjs`; never edit it by
//! hand.

use serde::{Deserialize, Serialize};
use serde_json::{json, Map as JsonMap, Value as JsonValue};
use serde_yaml::Value as YamlValue;
use specta::Type;
use std::collections::{HashMap, HashSet};
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::OnceLock;
use std::time::{Duration, Instant};

use crate::dsh::{
    dsh_home_patch_path_for_home, dsh_profile_dir_for_home, dsh_profile_patch_path_for_home,
    dsh_settings_path_for_home, leading_comment_block, patch_entry_id, read_patch_document,
    resolve_dsh_profile_for_home, serialize_patch_entry, split_patch_document, system_home_dir,
};
use crate::storage;

/// Prefix of a write refused because the patch changed since it was read.
pub const DSH_CONFIG_CONFLICT: &str = "DSH_CONFIG_CONFLICT";

/// Prefix of a write refused because a higher layer overrides the entry.
pub const DSH_CONFIG_SHADOWED: &str = "DSH_CONFIG_SHADOWED";

/// Profile lock file name (same one DSH's config editor takes).
const PROFILE_LOCK_FILENAME: &str = "package.json.lock";

/// A lock older than this is treated as abandoned and taken over.
const STALE_LOCK_AGE: Duration = Duration::from_secs(30);

/// How long to wait for the profile lock before failing.
const LOCK_WAIT: Duration = Duration::from_secs(2);

/// Entries the web-app bundle disables because the agent plane owns them now.
///
/// Writing `config` for these at the host layer has no effect in the Web /
/// desktop composition, so the UI warns when the profile patch tries.
const PRESET_ONLY_ENTRIES: &[&str] = &[
    "agent-instructions",
    "command-compact",
    "command-goal",
    "compaction-basic",
    "plan-mode",
    "skill-filesystem",
    "tool-bash",
    "tool-fs",
    "tool-fs-search",
    "tool-goal",
    "tool-jobs",
    "tool-plugin-manager",
    "tool-pwsh",
    "tool-result-pruner",
    "tool-skill",
    "tool-subagent",
    "tool-subagent-control",
    "tool-subagent-fork",
    "tool-subagent-list-agents",
    "tool-todo",
    "tool-web",
    "tool-workflow",
    "workflow-ptc",
];

/// Embedded schema snapshot, generated from the DSH source tree.
const SCHEMA_JSON: &str = include_str!("../res/dsh-config-schema.json");

// ============================================================================
// Public types
// ============================================================================

/// Everything the Dsh configuration page needs for one profile.
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct DshConfigSnapshot {
    /// DSH version the generated schema describes.
    pub schema_version: String,
    /// Version of the installed desktop app, when it can be detected.
    pub installed_version: Option<String>,
    /// Resolved profile name (`desktop`, `web`, ...).
    pub profile: Option<String>,
    /// Effective profile patch file (`cordis.patch.yml`).
    pub profile_patch_path: String,
    /// Home overlay patch (`~/.dsh/cordis.patch.yml`), read-only.
    pub home_patch_path: String,
    pub home_patch_exists: bool,
    /// Legacy pre-0.2 settings document.
    pub legacy_settings_path: String,
    /// Whether the legacy document (or its `.imported` rename) exists.
    pub legacy_settings_exists: bool,
    /// Content hash of the profile patch (`"absent"` when missing).
    pub patch_hash: String,
    /// Composed entries, lowest layer first.
    pub entries: Vec<DshConfigEntryView>,
    /// Migration / safety warnings for the UI.
    pub warnings: Vec<DshMigrationWarning>,
}

/// One composed plugin entry.
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct DshConfigEntryView {
    pub id: String,
    pub package_name: String,
    /// `bundle:<package>` | `profile` | `home`.
    pub layer: String,
    pub tier: String,
    pub description: String,
    pub disabled: bool,
    /// Effective config contains at least one `!!js` expression.
    pub has_js: bool,
    /// Effective config; secret fields are replaced by `{"$secret": true}`.
    pub config: Option<JsonValue>,
    /// The profile patch carries a config override for this entry.
    pub overridden: bool,
    /// The home overlay carries a config override for this entry.
    pub shadowed_by_home: bool,
    pub fields: Vec<DshConfigFieldView>,
}

/// One top-level configuration field with its effective value.
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct DshConfigFieldView {
    pub name: String,
    /// `string` | `number` | `boolean` | `enum` | `json`.
    pub kind: String,
    pub value: Option<JsonValue>,
    pub default: Option<JsonValue>,
    pub enum_values: Vec<String>,
    pub volatile: bool,
    pub secret: bool,
    pub description: String,
    pub overridden: bool,
    pub tier: String,
}

/// Result of one successful patch write.
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct DshConfigWriteResult {
    pub patch_hash: String,
    pub changed: bool,
}

/// One migration or safety warning.
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct DshMigrationWarning {
    pub code: String,
    /// `info` | `warning` | `error`.
    pub severity: String,
    pub entry_id: Option<String>,
    pub message: String,
    pub hint: String,
}

// ============================================================================
// Schema snapshot
// ============================================================================

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct SchemaDoc {
    #[serde(default)]
    schema_version: u32,
    dsh_version: String,
    #[serde(default)]
    bundles: HashMap<String, SchemaBundle>,
    #[serde(default)]
    entries: HashMap<String, SchemaEntry>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
struct SchemaBundle {
    #[serde(default)]
    entries: Vec<SchemaRow>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct SchemaRow {
    id: String,
    #[serde(default)]
    name: String,
    #[serde(default)]
    config: Option<JsonValue>,
    /// Present only when the row declares `disabled`; profile -> os -> bool.
    #[serde(default)]
    disabled_by_profile_os: Option<HashMap<String, HashMap<String, bool>>>,
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
struct SchemaEntry {
    #[serde(default)]
    package: String,
    #[serde(default)]
    tier: String,
    #[serde(default)]
    description: String,
    #[serde(default)]
    fields: Vec<SchemaField>,
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
struct SchemaField {
    name: String,
    #[serde(default = "default_field_kind")]
    kind: String,
    #[serde(default)]
    default: Option<JsonValue>,
    #[serde(default)]
    nullable: bool,
    #[serde(default, deserialize_with = "deserialize_string_list")]
    enum_values: Vec<String>,
    #[serde(default)]
    volatile: bool,
    #[serde(default)]
    secret: bool,
    #[serde(default)]
    tier: String,
    #[serde(default)]
    description: String,
}

fn default_field_kind() -> String {
    "string".to_string()
}

/// Accept a string list, flattening nested arrays the generator may emit for a
/// schemastery union; non-string leaves are ignored.
fn deserialize_string_list<'de, D>(deserializer: D) -> Result<Vec<String>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let value = JsonValue::deserialize(deserializer)?;
    fn flatten(value: &JsonValue, output: &mut Vec<String>) {
        match value {
            JsonValue::String(value) => output.push(value.clone()),
            JsonValue::Array(items) => items.iter().for_each(|item| flatten(item, output)),
            _ => {}
        }
    }
    let mut output = Vec::new();
    flatten(&value, &mut output);
    Ok(output)
}

fn schema() -> &'static SchemaDoc {
    static SCHEMA: OnceLock<SchemaDoc> = OnceLock::new();
    SCHEMA.get_or_init(|| {
        let doc: SchemaDoc = serde_json::from_str(SCHEMA_JSON)
            .unwrap_or_else(|error| panic!("invalid embedded dsh-config-schema.json: {error}"));
        assert_eq!(
            doc.schema_version, 1,
            "unsupported dsh-config-schema.json version: {}",
            doc.schema_version
        );
        doc
    })
}

/// DSH version the embedded schema was generated from.
pub fn dsh_schema_version() -> &'static str {
    &schema().dsh_version
}

// ============================================================================
// Composition
// ============================================================================

#[derive(Debug, Clone)]
struct ComposedEntry {
    id: String,
    package_name: String,
    description: String,
    tier: String,
    config: Option<JsonValue>,
    disabled: bool,
    layer: String,
    profile_config: Option<JsonValue>,
    home_config: Option<JsonValue>,
}

/// serde_yaml resolves `!!js` (a standard secondary-handle tag) to the plain
/// scalar and drops the tag, while a local `!js` tag survives as
/// [`serde_yaml::Value::Tagged`]. The read-only composition therefore rewrites
/// tag-position `!!js` to `!js` before parsing so `$js` markers round-trip.
/// Only the tag position (whitespace, `:`, `{`, `[` before, whitespace/end
/// after) is rewritten, so ordinary text values are untouched.
fn normalize_js_tags(text: &str) -> String {
    let mut output = String::with_capacity(text.len());
    let mut chars = text.char_indices().peekable();
    let mut in_single = false;
    let mut in_double = false;
    let mut in_comment = false;
    let mut escaped = false;
    while let Some((index, ch)) = chars.next() {
        if in_comment {
            output.push(ch);
            if ch == '\n' {
                in_comment = false;
            }
            continue;
        }
        if in_single || in_double {
            output.push(ch);
            if escaped {
                escaped = false;
            } else if ch == '\\' {
                escaped = true;
            } else if in_single && ch == '\'' {
                in_single = false;
            } else if in_double && ch == '"' {
                in_double = false;
            }
            continue;
        }
        if ch == '\'' {
            in_single = true;
            output.push(ch);
            continue;
        }
        if ch == '"' {
            in_double = true;
            output.push(ch);
            continue;
        }
        if ch == '#' {
            in_comment = true;
            output.push(ch);
            continue;
        }
        if text[index..].starts_with("!!js") {
            let before_ok = index == 0
                || matches!(
                    text[..index].chars().next_back(),
                    Some(' ' | '\t' | ':' | '{' | '[' | ',' | '\n')
                );
            let after = index + 4;
            let after_ok = after >= text.len()
                || matches!(
                    text[after..].chars().next(),
                    Some(' ' | '\t' | '\n' | '}' | ']' | ',')
                );
            if before_ok && after_ok {
                output.push_str("!js");
                chars.next();
                chars.next();
                chars.next();
                continue;
            }
        }
        output.push(ch);
    }
    output
}

/// Read a patch document with `!!js` tags normalized so they survive parsing.
fn read_patch_document_normalized(path: &Path) -> Result<(Vec<YamlValue>, String), String> {
    if !path.exists() {
        return Ok((Vec::new(), String::new()));
    }
    let text = fs::read_to_string(path)
        .map_err(|error| format!("Failed to read DSH patch file: {error}"))?;
    if text.trim().is_empty() {
        return Ok((Vec::new(), text));
    }
    let normalized = normalize_js_tags(&text);
    let value: YamlValue = serde_yaml::from_str(&normalized)
        .map_err(|error| format!("Invalid DSH patch YAML: {error}"))?;
    match value {
        YamlValue::Sequence(entries) => Ok((entries, text)),
        YamlValue::Null => Ok((Vec::new(), text)),
        _ => Err("Dsh patch file root must be a sequence".to_string()),
    }
}

fn os_key() -> &'static str {
    match std::env::consts::OS {
        "macos" => "darwin",
        "windows" => "win32",
        other => other,
    }
}

fn row_disabled(row: &SchemaRow, profile: &str) -> Option<bool> {
    let by_profile = row.disabled_by_profile_os.as_ref()?;
    let map = by_profile
        .get(profile)
        .or_else(|| by_profile.get("desktop"))
        .or_else(|| by_profile.get("web"))?;
    map.get(os_key()).copied()
}

fn read_profile_bundles(home: &Path, profile: &str) -> Result<Vec<String>, String> {
    let path = dsh_profile_dir_for_home(home, profile).join("package.json");
    let text = fs::read_to_string(&path).map_err(|error| {
        format!(
            "Failed to read DSH profile manifest {}: {error}",
            path.display()
        )
    })?;
    let value: JsonValue = serde_json::from_str(&text)
        .map_err(|error| format!("Invalid DSH profile manifest: {error}"))?;
    let bundles = value
        .pointer("/dsh/profile/bundles")
        .and_then(|bundles| bundles.as_array())
        .map(|bundles| {
            bundles
                .iter()
                .filter_map(|bundle| bundle.as_str().map(str::to_string))
                .collect::<Vec<_>>()
        })
        .filter(|bundles| !bundles.is_empty())
        .ok_or_else(|| {
            format!(
                "DSH profile manifest {} does not declare dsh.profile.bundles",
                path.display()
            )
        })?;
    Ok(bundles)
}

fn compose_entries(
    home: &Path,
    profile: &str,
    schema: &SchemaDoc,
) -> Result<Vec<ComposedEntry>, String> {
    let bundles = read_profile_bundles(home, profile)?;
    let mut entries: Vec<ComposedEntry> = Vec::new();

    for bundle in &bundles {
        let Some(schema_bundle) = schema.bundles.get(bundle) else {
            continue;
        };
        let layer = format!("bundle:{bundle}");
        for row in &schema_bundle.entries {
            let disabled = row_disabled(row, profile);
            if let Some(entry) = entries.iter_mut().find(|entry| entry.id == row.id) {
                if !row.name.is_empty() {
                    entry.package_name = row.name.clone();
                }
                if row.config.is_some() {
                    entry.config = row.config.clone();
                }
                if let Some(disabled) = disabled {
                    entry.disabled = disabled;
                }
                entry.layer = layer.clone();
            } else {
                entries.push(ComposedEntry {
                    id: row.id.clone(),
                    package_name: row.name.clone(),
                    description: String::new(),
                    tier: String::new(),
                    config: row.config.clone(),
                    disabled: disabled.unwrap_or(false),
                    layer: layer.clone(),
                    profile_config: None,
                    home_config: None,
                });
            }
        }
    }

    apply_patch_layer(
        &mut entries,
        &dsh_profile_patch_path_for_home(home, profile),
        "profile",
        true,
    )?;
    apply_patch_layer(
        &mut entries,
        &dsh_home_patch_path_for_home(home),
        "home",
        false,
    )?;

    for entry in &mut entries {
        if let Some(meta) = schema.entries.get(&entry.id) {
            if !meta.package.is_empty() {
                entry.package_name = meta.package.clone();
            }
            entry.description = meta.description.clone();
            entry.tier = if meta.tier.is_empty() {
                "C".to_string()
            } else {
                meta.tier.clone()
            };
        } else if entry.tier.is_empty() {
            entry.tier = "C".to_string();
        }
        if entry.package_name.is_empty() {
            entry.package_name = entry.id.clone();
        }
    }

    Ok(entries)
}

fn apply_patch_layer(
    entries: &mut Vec<ComposedEntry>,
    path: &Path,
    layer: &str,
    is_profile: bool,
) -> Result<(), String> {
    let (rows, _) = read_patch_document_normalized(path)?;
    for row in &rows {
        apply_patch_row(entries, row, layer, is_profile);
        if let Some(insert) = row.as_mapping().and_then(|row| row.get("insert")) {
            for item in insert.as_sequence().map(Vec::as_slice).unwrap_or_default() {
                apply_patch_row(entries, item, layer, is_profile);
            }
        }
    }
    Ok(())
}

fn apply_patch_row(
    entries: &mut Vec<ComposedEntry>,
    row: &YamlValue,
    layer: &str,
    is_profile: bool,
) {
    let Some(mapping) = row.as_mapping() else {
        return;
    };
    let Some(id) = mapping.get("id").and_then(YamlValue::as_str) else {
        return;
    };
    let config = mapping.get("config").map(yaml_to_json);
    let disabled = mapping.get("disabled").and_then(YamlValue::as_bool);
    let name = mapping
        .get("name")
        .and_then(YamlValue::as_str)
        .map(str::to_string);

    match entries.iter_mut().find(|entry| entry.id == id) {
        Some(entry) => {
            if let Some(name) = name {
                entry.package_name = name;
            }
            if let Some(config) = config {
                entry.config = Some(config.clone());
                if is_profile {
                    entry.profile_config = Some(config);
                } else {
                    entry.home_config = Some(config);
                }
            }
            if let Some(disabled) = disabled {
                entry.disabled = disabled;
            }
            entry.layer = layer.to_string();
        }
        None => {
            let Some(disabled) = disabled else {
                // A row without config/disabled that targets an unknown entry
                // is a no-op for the view.
                if config.is_none() {
                    return;
                }
                let profile_config = if is_profile { config.clone() } else { None };
                let home_config = if is_profile { None } else { config.clone() };
                entries.push(ComposedEntry {
                    id: id.to_string(),
                    package_name: name.unwrap_or_else(|| id.to_string()),
                    description: String::new(),
                    tier: "C".to_string(),
                    config: config.clone(),
                    disabled: false,
                    layer: layer.to_string(),
                    profile_config,
                    home_config,
                });
                return;
            };
            let profile_config = if is_profile { config.clone() } else { None };
            let home_config = if is_profile { None } else { config.clone() };
            entries.push(ComposedEntry {
                id: id.to_string(),
                package_name: name.unwrap_or_else(|| id.to_string()),
                description: String::new(),
                tier: "C".to_string(),
                config: config.clone(),
                disabled,
                layer: layer.to_string(),
                profile_config,
                home_config,
            });
        }
    }
}

// ============================================================================
// YAML <-> JSON (with `!!js` marker round-trip)
// ============================================================================

fn yaml_to_json(value: &YamlValue) -> JsonValue {
    match value {
        YamlValue::Null => JsonValue::Null,
        YamlValue::Bool(value) => JsonValue::Bool(*value),
        YamlValue::Number(number) => {
            if let Some(value) = number.as_i64() {
                JsonValue::from(value)
            } else if let Some(value) = number.as_u64() {
                JsonValue::from(value)
            } else {
                JsonValue::from(number.as_f64().unwrap_or(0.0))
            }
        }
        YamlValue::String(value) => JsonValue::String(value.clone()),
        YamlValue::Sequence(items) => JsonValue::Array(items.iter().map(yaml_to_json).collect()),
        YamlValue::Mapping(mapping) => {
            let mut object = JsonMap::new();
            for (key, value) in mapping {
                let key = match key {
                    YamlValue::String(key) => key.clone(),
                    other => serde_yaml::to_string(other)
                        .unwrap_or_default()
                        .trim()
                        .to_string(),
                };
                object.insert(key, yaml_to_json(value));
            }
            JsonValue::Object(object)
        }
        YamlValue::Tagged(tagged) => {
            let expression = match &tagged.value {
                YamlValue::String(value) => value.clone(),
                YamlValue::Number(number) => number.to_string(),
                other => serde_yaml::to_string(other)
                    .unwrap_or_default()
                    .trim()
                    .to_string(),
            };
            json!({ "$js": expression })
        }
    }
}

fn json_to_yaml(value: &JsonValue) -> YamlValue {
    match value {
        JsonValue::Null => YamlValue::Null,
        JsonValue::Bool(value) => YamlValue::Bool(*value),
        JsonValue::Number(number) => {
            if let Some(value) = number.as_i64() {
                YamlValue::Number(value.into())
            } else if let Some(value) = number.as_u64() {
                YamlValue::Number(value.into())
            } else {
                YamlValue::Number(serde_yaml::Number::from(
                    number.as_f64().unwrap_or_default(),
                ))
            }
        }
        JsonValue::String(value) => YamlValue::String(value.clone()),
        JsonValue::Array(items) => YamlValue::Sequence(items.iter().map(json_to_yaml).collect()),
        JsonValue::Object(object) => {
            if let Some(JsonValue::String(expression)) = object.get("$js") {
                return YamlValue::Tagged(Box::new(serde_yaml::value::TaggedValue {
                    tag: serde_yaml::value::Tag::new("tag:yaml.org,2002:js"),
                    value: YamlValue::String(expression.clone()),
                }));
            }
            let mut mapping = serde_yaml::Mapping::new();
            for (key, value) in object {
                mapping.insert(YamlValue::String(key.clone()), json_to_yaml(value));
            }
            YamlValue::Mapping(mapping)
        }
    }
}

fn is_js_marker(value: &JsonValue) -> bool {
    value
        .as_object()
        .map(|object| object.len() == 1 && object.get("$js").is_some_and(JsonValue::is_string))
        .unwrap_or(false)
}

fn is_secret_marker(value: &JsonValue) -> bool {
    value
        .as_object()
        .map(|object| object.len() == 1 && object.get("$secret").is_some_and(JsonValue::is_boolean))
        .unwrap_or(false)
}

fn contains_js(value: &JsonValue) -> bool {
    match value {
        JsonValue::Object(object) => is_js_marker(value) || object.values().any(contains_js),
        JsonValue::Array(items) => items.iter().any(contains_js),
        _ => false,
    }
}

// ============================================================================
// Read snapshot
// ============================================================================

/// Read the configuration snapshot for the system user's home directory.
pub fn read_dsh_config_snapshot(profile: Option<String>) -> Result<DshConfigSnapshot, String> {
    read_dsh_config_snapshot_for_home(&system_home_dir()?, profile)
}

/// Read the configuration snapshot for an explicit home directory.
pub fn read_dsh_config_snapshot_for_home(
    home: &Path,
    profile: Option<String>,
) -> Result<DshConfigSnapshot, String> {
    read_dsh_config_snapshot_with_schema(home, profile, schema())
}

fn read_dsh_config_snapshot_with_schema(
    home: &Path,
    profile: Option<String>,
    schema: &SchemaDoc,
) -> Result<DshConfigSnapshot, String> {
    let resolved = resolve_dsh_profile_for_home(home, profile.as_deref())?;
    let legacy_settings_path = dsh_settings_path_for_home(home);
    let imported_settings_path = legacy_settings_path.with_extension("yaml.imported");
    let legacy_settings_exists = legacy_settings_path.exists() || imported_settings_path.exists();

    let Some(profile_name) = resolved else {
        let mut snapshot = DshConfigSnapshot {
            schema_version: schema.dsh_version.clone(),
            installed_version: detect_installed_dsh_version(),
            profile: None,
            profile_patch_path: String::new(),
            home_patch_path: dsh_home_patch_path_for_home(home)
                .to_string_lossy()
                .to_string(),
            home_patch_exists: dsh_home_patch_path_for_home(home).exists(),
            legacy_settings_path: legacy_settings_path.to_string_lossy().to_string(),
            legacy_settings_exists,
            patch_hash: "absent".to_string(),
            entries: Vec::new(),
            warnings: vec![DshMigrationWarning {
                code: "DSH_CONFIG_NO_PROFILE".to_string(),
                severity: "error".to_string(),
                entry_id: None,
                message: "No DSH runtime profile found under ~/.dsh/profiles".to_string(),
                hint: "Start DeepSeek Harness once so it creates ~/.dsh/profiles/desktop."
                    .to_string(),
            }],
        };
        if legacy_settings_exists {
            snapshot.warnings.push(DshMigrationWarning {
                code: "DSH_CONFIG_LEGACY_SETTINGS".to_string(),
                severity: "info".to_string(),
                entry_id: None,
                message: "A legacy settings.yaml document is present".to_string(),
                hint: "DSH 0.2 imports it once and renames it to settings.yaml.imported; no profile was created.".to_string(),
            });
        }
        return Ok(snapshot);
    };

    let entries = compose_entries(home, &profile_name, schema)?;
    let profile_patch = dsh_profile_patch_path_for_home(home, &profile_name);
    let home_patch = dsh_home_patch_path_for_home(home);
    let patch_hash = file_hash(&profile_patch);
    let home_patch_exists = home_patch.exists();
    let installed_version = detect_installed_dsh_version();

    let mut views = Vec::with_capacity(entries.len());
    for entry in &entries {
        views.push(entry_view(entry, schema));
    }

    let warnings = build_warnings(
        &entries,
        &home_patch,
        schema,
        installed_version.as_deref(),
        legacy_settings_exists,
    );

    Ok(DshConfigSnapshot {
        schema_version: schema.dsh_version.clone(),
        installed_version,
        profile: Some(profile_name),
        profile_patch_path: profile_patch.to_string_lossy().to_string(),
        home_patch_path: home_patch.to_string_lossy().to_string(),
        home_patch_exists,
        legacy_settings_path: legacy_settings_path.to_string_lossy().to_string(),
        legacy_settings_exists,
        patch_hash,
        entries: views,
        warnings,
    })
}

fn entry_view(entry: &ComposedEntry, schema: &SchemaDoc) -> DshConfigEntryView {
    let meta = schema.entries.get(&entry.id).cloned().unwrap_or_default();
    let effective = entry.config.clone();
    let mut redacted = effective.clone();
    let mut fields = Vec::new();

    if let Some(value) = redacted.as_mut() {
        if let Some(object) = value.as_object_mut() {
            for field in &meta.fields {
                if field.secret && object.contains_key(&field.name) {
                    object.insert(field.name.clone(), json!({ "$secret": true }));
                }
            }
        }
    }

    for field in &meta.fields {
        let value = effective
            .as_ref()
            .and_then(JsonValue::as_object)
            .and_then(|object| object.get(&field.name))
            .map(|value| {
                if field.secret {
                    json!({ "$secret": true })
                } else {
                    value.clone()
                }
            });
        let overridden = entry
            .profile_config
            .as_ref()
            .and_then(JsonValue::as_object)
            .map(|object| object.contains_key(&field.name))
            .unwrap_or(false);
        fields.push(DshConfigFieldView {
            name: field.name.clone(),
            kind: field.kind.clone(),
            value,
            default: field.default.clone(),
            enum_values: field.enum_values.clone(),
            volatile: field.volatile,
            secret: field.secret,
            description: field.description.clone(),
            overridden,
            tier: field.tier.clone(),
        });
    }

    DshConfigEntryView {
        id: entry.id.clone(),
        package_name: entry.package_name.clone(),
        layer: entry.layer.clone(),
        tier: entry.tier.clone(),
        description: entry.description.clone(),
        disabled: entry.disabled,
        has_js: effective.as_ref().map(contains_js).unwrap_or(false),
        config: redacted,
        overridden: entry.profile_config.is_some(),
        shadowed_by_home: entry.home_config.is_some(),
        fields,
    }
}

fn build_warnings(
    entries: &[ComposedEntry],
    home_patch: &Path,
    schema: &SchemaDoc,
    installed_version: Option<&str>,
    legacy_settings_exists: bool,
) -> Vec<DshMigrationWarning> {
    let mut warnings = Vec::new();
    let find = |id: &str| entries.iter().find(|entry| entry.id == id);

    if let Some(entry) = find("tools") {
        if entry
            .config
            .as_ref()
            .and_then(|config| config.get("mode"))
            .and_then(JsonValue::as_str)
            == Some("both")
        {
            warnings.push(DshMigrationWarning {
                code: "DSH_CONFIG_TOOLS_MODE_BOTH".to_string(),
                severity: "error".to_string(),
                entry_id: Some(entry.id.clone()),
                message: "tools.mode = \"both\" is no longer accepted by DSH 0.2".to_string(),
                hint: "Use `native` or `ptc`.".to_string(),
            });
        }
    }

    for entry in entries {
        if !entry.id.starts_with("subagent") {
            continue;
        }
        if entry
            .config
            .as_ref()
            .and_then(JsonValue::as_object)
            .map(|object| object.contains_key("backgroundMode"))
            .unwrap_or(false)
        {
            warnings.push(DshMigrationWarning {
                code: "DSH_CONFIG_SUBAGENT_BACKGROUND_MODE".to_string(),
                severity: "warning".to_string(),
                entry_id: Some(entry.id.clone()),
                message: "subagent backgroundMode was removed in DSH 0.2".to_string(),
                hint: "Remove the field; use the subagent tool's own background controls."
                    .to_string(),
            });
        }
    }

    let preset_only: HashSet<&str> = PRESET_ONLY_ENTRIES.iter().copied().collect();
    for entry in entries {
        if !preset_only.contains(entry.id.as_str()) || entry.profile_config.is_none() {
            continue;
        }
        if !entry.disabled {
            continue;
        }
        warnings.push(DshMigrationWarning {
            code: "DSH_CONFIG_PRESET_ONLY_ROW".to_string(),
            severity: "warning".to_string(),
            entry_id: Some(entry.id.clone()),
            message: format!(
                "The profile patch configures `{}`, but the Web/desktop composition disables that host row",
                entry.id
            ),
            hint: "Agent-plane rows moved into the agent presets; edit preset-standard/ptc/minimal/cordis instead."
                .to_string(),
        });
    }

    if let Some(entry) = find("llm-pi-ai") {
        let has_legacy_azure = entry
            .config
            .as_ref()
            .and_then(|config| config.get("providers"))
            .and_then(JsonValue::as_object)
            .map(|providers| providers.contains_key("azure-openai-responses"))
            .unwrap_or(false);
        if has_legacy_azure {
            warnings.push(DshMigrationWarning {
                code: "DSH_CONFIG_AZURE_ROUTE_KEY".to_string(),
                severity: "warning".to_string(),
                entry_id: Some(entry.id.clone()),
                message: "The Azure pi-ai route key was renamed to `azure` in DSH 0.2.1".to_string(),
                hint: "Rename providers.azure-openai-responses to providers.azure; the `api` identifier stays azure-openai-responses."
                    .to_string(),
            });
        }
    }

    for id in ["llm-deepseek", "llm-deepseek-account"] {
        let Some(entry) = find(id) else {
            continue;
        };
        let has_image_detail = entry
            .config
            .as_ref()
            .and_then(|config| config.get("models"))
            .and_then(JsonValue::as_array)
            .map(|models| {
                models.iter().any(|model| {
                    model
                        .as_object()
                        .map(|object| object.contains_key("imageDetail"))
                        .unwrap_or(false)
                })
            })
            .unwrap_or(false);
        if has_image_detail {
            warnings.push(DshMigrationWarning {
                code: "DSH_CONFIG_IMAGE_DETAIL".to_string(),
                severity: "warning".to_string(),
                entry_id: Some(entry.id.clone()),
                message: "Model field `imageDetail` was removed in DSH 0.2".to_string(),
                hint: "Replace it with `imagePixelBudget`.".to_string(),
            });
        }
    }

    if let Some(installed) = installed_version {
        if installed != schema.dsh_version {
            warnings.push(DshMigrationWarning {
                code: "DSH_CONFIG_VERSION_MISMATCH".to_string(),
                severity: "warning".to_string(),
                entry_id: None,
                message: format!(
                    "Installed DeepSeek Harness is {installed}, but the bundled schema was generated from {}",
                    schema.dsh_version
                ),
                hint: "Regenerate res/dsh-config-schema.json from the installed DSH source before editing unfamiliar fields."
                    .to_string(),
            });
        }
    }

    for entry in entries {
        if entry.home_config.is_some() && entry.profile_config.is_some() {
            warnings.push(DshMigrationWarning {
                code: "DSH_CONFIG_HOME_SHADOW".to_string(),
                severity: "warning".to_string(),
                entry_id: Some(entry.id.clone()),
                message: format!(
                    "{} is configured in both the profile patch and the home overlay; the home overlay wins",
                    entry.id
                ),
                hint: format!(
                    "Move the setting into the profile patch, or clear it from {} first; DroidGear refuses to write a shadowed entry.",
                    home_patch.display()
                ),
            });
        }
    }

    if legacy_settings_exists {
        warnings.push(DshMigrationWarning {
            code: "DSH_CONFIG_LEGACY_SETTINGS".to_string(),
            severity: "info".to_string(),
            entry_id: None,
            message: "Legacy settings.yaml (or .imported) is present".to_string(),
            hint: "DSH 0.2 imports it once into the profile patch; DroidGear only needs cordis.patch.yml."
                .to_string(),
        });
    }

    warnings
}

/// Content hash used by the UI for optimistic concurrency.
pub fn file_hash(path: &Path) -> String {
    match fs::read(path) {
        Ok(bytes) => hash_bytes(&bytes),
        Err(_) => "absent".to_string(),
    }
}

fn hash_bytes(bytes: &[u8]) -> String {
    let mut hash: u64 = 0xcbf2_9ce4_8422_2325;
    for byte in bytes {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x0100_0000_01b3);
    }
    format!("{hash:016x}")
}

/// Detect the installed desktop app version from its runtime manifest.
pub fn detect_installed_dsh_version() -> Option<String> {
    if let Ok(path) = std::env::var("DSH_RUNTIME_MANIFEST") {
        if let Some(version) = read_desktop_version(Path::new(&path)) {
            return Some(version);
        }
    }
    let mut candidates: Vec<PathBuf> = vec![
        PathBuf::from(
            "/Applications/DeepSeek Harness.app/Contents/Resources/runtime/primary-runtime/runtime.json",
        ),
        PathBuf::from(
            "/Applications/DeepSeek Harness.app/Contents/Resources/runtime/versions.json",
        ),
    ];
    if let Some(home) = dirs::home_dir() {
        candidates.push(
            home.join("Applications/DeepSeek Harness.app/Contents/Resources/runtime/primary-runtime/runtime.json"),
        );
    }
    candidates
        .iter()
        .find_map(|path| read_desktop_version(path))
}

fn read_desktop_version(path: &Path) -> Option<String> {
    let text = fs::read_to_string(path).ok()?;
    let value: JsonValue = serde_json::from_str(&text).ok()?;
    value
        .get("desktopVersion")
        .or_else(|| value.get("version"))
        .and_then(JsonValue::as_str)
        .map(str::to_string)
}

// ============================================================================
// Write
// ============================================================================

/// Save one entry's complete configuration (or remove the override).
pub fn save_dsh_config_entry(
    profile: Option<String>,
    entry_id: &str,
    config: Option<JsonValue>,
    expected_hash: Option<String>,
) -> Result<DshConfigWriteResult, String> {
    save_dsh_config_entry_for_home(
        &system_home_dir()?,
        profile,
        entry_id,
        config,
        expected_hash,
    )
}

/// Save one entry's complete configuration for an explicit home directory.
pub fn save_dsh_config_entry_for_home(
    home: &Path,
    profile: Option<String>,
    entry_id: &str,
    config: Option<JsonValue>,
    expected_hash: Option<String>,
) -> Result<DshConfigWriteResult, String> {
    save_dsh_config_entry_with_schema(home, profile, entry_id, config, expected_hash, schema())
}

fn save_dsh_config_entry_with_schema(
    home: &Path,
    profile: Option<String>,
    entry_id: &str,
    config: Option<JsonValue>,
    expected_hash: Option<String>,
    schema: &SchemaDoc,
) -> Result<DshConfigWriteResult, String> {
    let entry_id = entry_id.trim();
    if entry_id.is_empty() {
        return Err("Entry id is required".to_string());
    }
    let Some(profile_name) = resolve_dsh_profile_for_home(home, profile.as_deref())? else {
        return Err("No DSH profile found under ~/.dsh/profiles".to_string());
    };
    let entries = compose_entries(home, &profile_name, schema)?;
    if !entries.iter().any(|entry| entry.id == entry_id) {
        return Err(format!("Unknown DSH configuration entry: {entry_id}"));
    }

    let prepared = match config {
        Some(config) => Some(prepare_config_for_write(entry_id, config, schema)?),
        None => None,
    };

    let profile_dir = dsh_profile_dir_for_home(home, &profile_name);
    fs::create_dir_all(&profile_dir)
        .map_err(|error| format!("Failed to create DSH profile directory: {error}"))?;
    let _lock = ProfileLock::acquire(&profile_dir)?;

    let path = dsh_profile_patch_path_for_home(home, &profile_name);
    let original = fs::read_to_string(&path).unwrap_or_default();
    if let Some(expected) = expected_hash {
        let current = if original.is_empty() && !path.exists() {
            "absent".to_string()
        } else {
            hash_bytes(original.as_bytes())
        };
        if expected != current {
            return Err(format!(
                "{DSH_CONFIG_CONFLICT}: the DSH profile patch changed since it was read (expected {expected}, found {current})"
            ));
        }
    }

    ensure_home_patch_does_not_shadow(home, entry_id)?;

    let (mut rows, read_original) = read_patch_document_normalized(&path)?;
    let source = if read_original.is_empty() {
        original
    } else {
        read_original
    };
    let (raw_header, mut blocks) = split_patch_document(&source);
    let header = if blocks.is_empty() {
        leading_comment_block(&source)
    } else {
        raw_header
    };

    let index = rows
        .iter()
        .position(|row| patch_entry_id(row) == Some(entry_id));
    if let Some(index) = index {
        ensure_patch_row_rewritable(&blocks[index], entry_id)?;
    }
    let default_name = schema
        .entries
        .get(entry_id)
        .map(|entry| entry.package.clone())
        .filter(|package| !package.is_empty())
        .unwrap_or_else(|| entry_id.to_string());

    match (index, prepared) {
        (Some(index), Some(config_value)) => {
            let row = rows[index]
                .as_mapping_mut()
                .ok_or_else(|| format!("DSH patch entry '{entry_id}' is not a mapping"))?;
            row.insert(
                YamlValue::String("config".to_string()),
                json_to_yaml(&config_value),
            );
            blocks[index] = finalize_block(serialize_patch_entry(&rows[index])?);
        }
        (Some(index), None) => {
            let row = rows[index]
                .as_mapping_mut()
                .ok_or_else(|| format!("DSH patch entry '{entry_id}' is not a mapping"))?;
            row.remove(YamlValue::String("config".to_string()));
            if row.len() <= 2 {
                rows.remove(index);
                blocks.remove(index);
            } else {
                blocks[index] = finalize_block(serialize_patch_entry(&rows[index])?);
            }
        }
        (None, Some(config_value)) => {
            let row = new_patch_row(
                entry_id,
                &default_name,
                Some(json_to_yaml(&config_value)),
                None,
            );
            let block = finalize_block(serialize_patch_entry(&row)?);
            rows.push(row);
            blocks.push(block);
        }
        (None, None) => {
            return Ok(DshConfigWriteResult {
                patch_hash: hash_bytes(source.as_bytes()),
                changed: false,
            });
        }
    }

    let document = render_document(&header, &rows, &blocks);
    storage::atomic_write_private(&path, document.as_bytes())?;
    Ok(DshConfigWriteResult {
        patch_hash: hash_bytes(document.as_bytes()),
        changed: true,
    })
}

/// Enable or disable one entry in the profile patch layer.
pub fn set_dsh_config_entry_disabled(
    profile: Option<String>,
    entry_id: &str,
    disabled: bool,
    expected_hash: Option<String>,
) -> Result<DshConfigWriteResult, String> {
    set_dsh_config_entry_disabled_for_home(
        &system_home_dir()?,
        profile,
        entry_id,
        disabled,
        expected_hash,
    )
}

/// Enable or disable one entry for an explicit home directory.
pub fn set_dsh_config_entry_disabled_for_home(
    home: &Path,
    profile: Option<String>,
    entry_id: &str,
    disabled: bool,
    expected_hash: Option<String>,
) -> Result<DshConfigWriteResult, String> {
    set_dsh_config_entry_disabled_with_schema(
        home,
        profile,
        entry_id,
        disabled,
        expected_hash,
        schema(),
    )
}

fn set_dsh_config_entry_disabled_with_schema(
    home: &Path,
    profile: Option<String>,
    entry_id: &str,
    disabled: bool,
    expected_hash: Option<String>,
    schema: &SchemaDoc,
) -> Result<DshConfigWriteResult, String> {
    let entry_id = entry_id.trim();
    if entry_id.is_empty() {
        return Err("Entry id is required".to_string());
    }
    let Some(profile_name) = resolve_dsh_profile_for_home(home, profile.as_deref())? else {
        return Err("No DSH profile found under ~/.dsh/profiles".to_string());
    };
    let entries = compose_entries(home, &profile_name, schema)?;
    if !entries.iter().any(|entry| entry.id == entry_id) {
        return Err(format!("Unknown DSH configuration entry: {entry_id}"));
    }
    let profile_dir = dsh_profile_dir_for_home(home, &profile_name);
    fs::create_dir_all(&profile_dir)
        .map_err(|error| format!("Failed to create DSH profile directory: {error}"))?;
    let _lock = ProfileLock::acquire(&profile_dir)?;

    let path = dsh_profile_patch_path_for_home(home, &profile_name);
    let original = fs::read_to_string(&path).unwrap_or_default();
    if let Some(expected) = expected_hash {
        let current = if original.is_empty() && !path.exists() {
            "absent".to_string()
        } else {
            hash_bytes(original.as_bytes())
        };
        if expected != current {
            return Err(format!(
                "{DSH_CONFIG_CONFLICT}: the DSH profile patch changed since it was read (expected {expected}, found {current})"
            ));
        }
    }

    let (mut rows, read_original) = read_patch_document_normalized(&path)?;
    let source = if read_original.is_empty() {
        original
    } else {
        read_original
    };
    let (raw_header, mut blocks) = split_patch_document(&source);
    let header = if blocks.is_empty() {
        leading_comment_block(&source)
    } else {
        raw_header
    };

    let index = rows
        .iter()
        .position(|row| patch_entry_id(row) == Some(entry_id));
    if let Some(index) = index {
        ensure_patch_row_rewritable(&blocks[index], entry_id)?;
    }
    let default_name = schema
        .entries
        .get(entry_id)
        .map(|entry| entry.package.clone())
        .filter(|package| !package.is_empty())
        .unwrap_or_else(|| entry_id.to_string());

    match (index, disabled) {
        (Some(index), disabled) => {
            let row = rows[index]
                .as_mapping_mut()
                .ok_or_else(|| format!("DSH patch entry '{entry_id}' is not a mapping"))?;
            if disabled {
                row.insert(
                    YamlValue::String("disabled".to_string()),
                    YamlValue::Bool(true),
                );
            } else {
                row.remove(YamlValue::String("disabled".to_string()));
            }
            if disabled || row.len() > 2 {
                blocks[index] = finalize_block(serialize_patch_entry(&rows[index])?);
            } else {
                rows.remove(index);
                blocks.remove(index);
            }
        }
        (None, true) => {
            let row = new_patch_row(entry_id, &default_name, None, Some(YamlValue::Bool(true)));
            let block = finalize_block(serialize_patch_entry(&row)?);
            rows.push(row);
            blocks.push(block);
        }
        (None, false) => {
            return Ok(DshConfigWriteResult {
                patch_hash: hash_bytes(source.as_bytes()),
                changed: false,
            });
        }
    }

    let document = render_document(&header, &rows, &blocks);
    storage::atomic_write_private(&path, document.as_bytes())?;
    Ok(DshConfigWriteResult {
        patch_hash: hash_bytes(document.as_bytes()),
        changed: true,
    })
}

fn new_patch_row(
    id: &str,
    name: &str,
    config: Option<YamlValue>,
    disabled: Option<YamlValue>,
) -> YamlValue {
    let mut mapping = serde_yaml::Mapping::new();
    mapping.insert(
        YamlValue::String("id".to_string()),
        YamlValue::String(id.to_string()),
    );
    if !name.is_empty() && name != id {
        mapping.insert(
            YamlValue::String("name".to_string()),
            YamlValue::String(name.to_string()),
        );
    }
    if let Some(config) = config {
        mapping.insert(YamlValue::String("config".to_string()), config);
    }
    if let Some(disabled) = disabled {
        mapping.insert(YamlValue::String("disabled".to_string()), disabled);
    }
    YamlValue::Mapping(mapping)
}

/// Whether a rewritten patch row carries a YAML tag we cannot round-trip.
///
/// `!!js` / `!js` expressions survive the read-write cycle (the read path
/// normalizes them to `$js` markers and the write path renders them back), but
/// any other custom tag would be silently dropped, so such rows are refused
/// instead of corrupted.
fn has_unsupported_tags(block: &str) -> bool {
    let mut in_single = false;
    let mut in_double = false;
    let mut in_comment = false;
    let mut escaped = false;
    let mut previous_is_tag_position = true;
    for (index, ch) in block.char_indices() {
        if in_comment {
            if ch == '\n' {
                in_comment = false;
                previous_is_tag_position = true;
            }
            continue;
        }
        if in_single || in_double {
            if escaped {
                escaped = false;
            } else if ch == '\\' {
                escaped = true;
            } else if (in_single && ch == '\'') || (in_double && ch == '"') {
                in_single = false;
                in_double = false;
            }
            continue;
        }
        if ch == '\'' {
            in_single = true;
            previous_is_tag_position = false;
            continue;
        }
        if ch == '"' {
            in_double = true;
            previous_is_tag_position = false;
            continue;
        }
        if ch == '#' {
            in_comment = true;
            continue;
        }
        if ch == '!' && previous_is_tag_position {
            let rest = &block[index..];
            let allowed = rest.starts_with("!!js")
                || rest.starts_with("!js")
                || rest.starts_with("!<tag:yaml.org,2002:js>")
                || rest.starts_with("!tag:yaml.org,2002:js");
            if !allowed {
                return true;
            }
        }
        previous_is_tag_position = matches!(ch, ' ' | '\t' | '\n' | ':' | '{' | '[' | ',');
    }
    false
}

fn ensure_patch_row_rewritable(block: &str, entry_id: &str) -> Result<(), String> {
    if has_unsupported_tags(block) {
        return Err(format!(
            "DSH patch entry '{entry_id}' contains YAML tags that cannot be rewritten safely; edit the file manually"
        ));
    }
    Ok(())
}

/// serde_yaml renders a full-URI tag as `!tag:yaml.org,2002:js`; DSH writes
/// and reads the `!!js` shorthand, so normalize our own output.
fn finalize_block(block: String) -> String {
    block
        .replace("!<tag:yaml.org,2002:js>", "!!js")
        .replace("!tag:yaml.org,2002:js", "!!js")
}

fn render_document(header: &str, rows: &[YamlValue], blocks: &[String]) -> String {
    if rows.is_empty() {
        return format!("{header}[]\n");
    }
    format!("{header}{}", blocks.concat())
}

fn ensure_home_patch_does_not_shadow(home: &Path, entry_id: &str) -> Result<(), String> {
    let home_patch = dsh_home_patch_path_for_home(home);
    if !home_patch.exists() {
        return Ok(());
    }
    let (rows, _) = read_patch_document(&home_patch)?;
    let shadowed = rows.iter().any(|row| {
        patch_entry_id(row) == Some(entry_id)
            && row
                .as_mapping()
                .map(|mapping| mapping.contains_key(YamlValue::String("config".to_string())))
                .unwrap_or(false)
    });
    if shadowed {
        return Err(format!(
            "{DSH_CONFIG_SHADOWED}: '{entry_id}' is already configured by the home patch {}; edit that file or move the setting to the profile patch first",
            home_patch.display()
        ));
    }
    Ok(())
}

fn prepare_config_for_write(
    entry_id: &str,
    config: JsonValue,
    schema: &SchemaDoc,
) -> Result<JsonValue, String> {
    let JsonValue::Object(mut object) = config else {
        return Err("Configuration must be a JSON object".to_string());
    };
    let meta = schema.entries.get(entry_id).cloned().unwrap_or_default();
    let markers: Vec<String> = object
        .iter()
        .filter(|(_, value)| is_secret_marker(value))
        .map(|(key, _)| key.clone())
        .collect();
    for key in markers {
        object.remove(&key);
    }
    for field in &meta.fields {
        if field.secret {
            if let Some(value) = object.get(&field.name) {
                if is_secret_marker(value) {
                    // The UI only knows whether a secret is set; dropping the
                    // key keeps the inherited value instead of writing a
                    // placeholder into the file.
                    object.remove(&field.name);
                } else if !value.is_null() {
                    return Err(format!(
                        "Field '{}' is a secret; store it in ~/.dsh/.credentials.yaml (or the DSH Models page) instead of cordis.patch.yml",
                        field.name
                    ));
                }
            }
        }
    }
    let prepared = JsonValue::Object(object);
    validate_config(entry_id, &prepared, schema)?;
    Ok(prepared)
}

fn validate_config(entry_id: &str, config: &JsonValue, schema: &SchemaDoc) -> Result<(), String> {
    let Some(meta) = schema.entries.get(entry_id) else {
        return Ok(());
    };
    let Some(object) = config.as_object() else {
        return Err("Configuration must be a JSON object".to_string());
    };
    for field in &meta.fields {
        let Some(value) = object.get(&field.name) else {
            continue;
        };
        if field.secret || is_secret_marker(value) {
            continue;
        }
        if value.is_null() {
            if !field.nullable {
                return Err(format!("Field '{}' cannot be null", field.name));
            }
            continue;
        }
        if is_js_marker(value) {
            continue;
        }
        match field.kind.as_str() {
            "number" => {
                if !value.is_number() {
                    return Err(format!("Field '{}' must be a number", field.name));
                }
            }
            "boolean" => {
                if !value.is_boolean() {
                    return Err(format!("Field '{}' must be a boolean", field.name));
                }
            }
            "string" => {
                if !value.is_string() {
                    return Err(format!("Field '{}' must be a string", field.name));
                }
            }
            "enum" => {
                let Some(value) = value.as_str() else {
                    return Err(format!("Field '{}' must be a string", field.name));
                };
                if !field.enum_values.is_empty()
                    && !field.enum_values.iter().any(|allowed| allowed == value)
                {
                    return Err(format!(
                        "Field '{}' must be one of: {}",
                        field.name,
                        field.enum_values.join(", ")
                    ));
                }
            }
            _ => {}
        }
    }
    Ok(())
}

// ============================================================================
// Profile lock (shared with DSH's config editor)
// ============================================================================

struct ProfileLock {
    path: PathBuf,
}

impl ProfileLock {
    fn acquire(profile_dir: &Path) -> Result<Self, String> {
        let path = profile_dir.join(PROFILE_LOCK_FILENAME);
        let deadline = Instant::now() + LOCK_WAIT;
        loop {
            match fs::OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(&path)
            {
                Ok(mut file) => {
                    let _ = write!(file, "{}", std::process::id());
                    return Ok(Self { path });
                }
                Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {
                    if is_stale_lock(&path) {
                        let _ = fs::remove_file(&path);
                        continue;
                    }
                    if Instant::now() >= deadline {
                        return Err(format!(
                            "Another process is writing the DSH profile (lock {}); retry shortly",
                            path.display()
                        ));
                    }
                    std::thread::sleep(Duration::from_millis(50));
                }
                Err(error) => {
                    return Err(format!("Failed to lock DSH profile: {error}"));
                }
            }
        }
    }
}

impl Drop for ProfileLock {
    fn drop(&mut self) {
        let _ = fs::remove_file(&self.path);
    }
}

fn is_stale_lock(path: &Path) -> bool {
    fs::metadata(path)
        .and_then(|metadata| metadata.modified())
        .ok()
        .and_then(|modified| modified.elapsed().ok())
        .map(|age| age > STALE_LOCK_AGE)
        .unwrap_or(false)
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_support::slashes;
    use tempfile::TempDir;

    fn home(temp: &TempDir) -> &Path {
        temp.path()
    }

    fn write_profile(home: &Path, name: &str, bundles: &[&str]) {
        let dir = dsh_profile_dir_for_home(home, name);
        fs::create_dir_all(&dir).expect("profile dir");
        let manifest = json!({
            "name": format!("dsh-profile-{name}"),
            "private": true,
            "dsh": { "profile": { "bundles": bundles } }
        });
        fs::write(
            dir.join("package.json"),
            serde_json::to_string_pretty(&manifest).expect("manifest"),
        )
        .expect("write manifest");
        fs::write(dir.join("cordis.yml"), "[]\n").expect("cordis.yml");
    }

    fn row(id: &str, name: &str, config: Option<JsonValue>, disabled: Option<bool>) -> SchemaRow {
        SchemaRow {
            id: id.to_string(),
            name: name.to_string(),
            config,
            disabled_by_profile_os: disabled.map(|disabled| {
                let mut profiles = HashMap::new();
                for profile in ["desktop", "web"] {
                    let mut oses = HashMap::new();
                    for os in ["darwin", "win32", "linux"] {
                        oses.insert(os.to_string(), disabled);
                    }
                    profiles.insert(profile.to_string(), oses);
                }
                profiles
            }),
        }
    }

    fn field(
        name: &str,
        kind: &str,
        default: Option<JsonValue>,
        volatile: bool,
        secret: bool,
        enum_values: &[&str],
    ) -> SchemaField {
        SchemaField {
            name: name.to_string(),
            kind: kind.to_string(),
            default,
            nullable: true,
            enum_values: enum_values.iter().map(|value| value.to_string()).collect(),
            volatile,
            secret,
            tier: "A".to_string(),
            description: String::new(),
        }
    }

    fn test_schema() -> SchemaDoc {
        let mut entries = HashMap::new();
        entries.insert(
            "agent-default-model".to_string(),
            SchemaEntry {
                package: "@deepseek-ai/dsh-agent-default-model".to_string(),
                tier: "A".to_string(),
                description: "Default model".to_string(),
                fields: vec![
                    field("provider", "string", None, true, false, &[]),
                    field("model", "string", None, true, false, &[]),
                    field(
                        "reasoningEffort",
                        "enum",
                        None,
                        true,
                        false,
                        &["off", "high"],
                    ),
                ],
            },
        );
        entries.insert(
            "tools".to_string(),
            SchemaEntry {
                package: "@deepseek-ai/dsh-tools".to_string(),
                tier: "B".to_string(),
                description: "Tools".to_string(),
                fields: vec![field(
                    "mode",
                    "enum",
                    Some(json!("native")),
                    false,
                    false,
                    &["native", "ptc"],
                )],
            },
        );
        entries.insert(
            "web-search-deepseek".to_string(),
            SchemaEntry {
                package: "@deepseek-ai/dsh-web-search-deepseek".to_string(),
                tier: "B".to_string(),
                description: "Search".to_string(),
                fields: vec![
                    field("apiKey", "string", None, true, true, &[]),
                    field(
                        "apiKeyEnv",
                        "string",
                        Some(json!("DEEPSEEK_API_KEY")),
                        true,
                        false,
                        &[],
                    ),
                    field("maxUses", "number", Some(json!(5)), true, false, &[]),
                ],
            },
        );
        entries.insert(
            "llm-pi-ai".to_string(),
            SchemaEntry {
                package: "@deepseek-ai/dsh-llm-pi-ai".to_string(),
                tier: "A".to_string(),
                description: "Providers".to_string(),
                fields: vec![field("providers", "json", None, true, false, &[])],
            },
        );
        entries.insert(
            "subagent".to_string(),
            SchemaEntry {
                package: "@deepseek-ai/dsh-subagent".to_string(),
                tier: "A".to_string(),
                description: "Subagents".to_string(),
                fields: vec![
                    field("maxDepth", "number", Some(json!(1)), true, false, &[]),
                    field(
                        "maxActiveSubagents",
                        "number",
                        Some(json!(8)),
                        true,
                        false,
                        &[],
                    ),
                ],
            },
        );
        entries.insert(
            "compaction-basic".to_string(),
            SchemaEntry {
                package: "@deepseek-ai/dsh-compaction-basic".to_string(),
                tier: "B".to_string(),
                description: "Compaction".to_string(),
                fields: vec![field(
                    "thresholdRatio",
                    "number",
                    Some(json!(0.8)),
                    false,
                    false,
                    &[],
                )],
            },
        );

        let mut base = SchemaBundle {
            entries: vec![
                row(
                    "agent-default-model",
                    "@deepseek-ai/dsh-agent-default-model",
                    None,
                    None,
                ),
                row(
                    "tools",
                    "@deepseek-ai/dsh-tools",
                    Some(json!({"mode": "native"})),
                    None,
                ),
                row("llm-pi-ai", "@deepseek-ai/dsh-llm-pi-ai", None, None),
                row("subagent", "@deepseek-ai/dsh-subagent", None, None),
                row(
                    "compaction-basic",
                    "@deepseek-ai/dsh-compaction-basic",
                    None,
                    None,
                ),
                row("tool-bash", "@deepseek-ai/dsh-tool-bash", None, None),
            ],
        };
        base.entries[0].config =
            Some(json!({"provider": "deepseek-official", "model": "deepseek-flash"}));
        let mut web_app = SchemaBundle {
            entries: vec![
                row(
                    "web-search-deepseek",
                    "@deepseek-ai/dsh-web-search-deepseek",
                    Some(json!({"maxUses": 5, "apiKeyEnv": "DEEPSEEK_API_KEY"})),
                    None,
                ),
                row("tool-bash", "@deepseek-ai/dsh-tool-bash", None, Some(true)),
                row(
                    "compaction-basic",
                    "@deepseek-ai/dsh-compaction-basic",
                    Some(json!({"thresholdRatio": 0.8})),
                    Some(true),
                ),
            ],
        };
        web_app.entries[1].config = None;

        let mut bundles = HashMap::new();
        bundles.insert("@deepseek-ai/dsh-base".to_string(), base);
        bundles.insert("@deepseek-ai/dsh-web-app".to_string(), web_app);
        SchemaDoc {
            schema_version: 1,
            dsh_version: "0.2.1-rc.2".to_string(),
            bundles,
            entries,
        }
    }

    fn desktop_home(temp: &TempDir) -> &Path {
        let home = home(temp);
        write_profile(
            home,
            "desktop",
            &["@deepseek-ai/dsh-base", "@deepseek-ai/dsh-web-app"],
        );
        home
    }

    fn write_patch(home: &Path, text: &str) {
        let dir = dsh_profile_dir_for_home(home, "desktop");
        fs::write(dir.join("cordis.patch.yml"), text).expect("write patch");
    }

    fn entry<'a>(snapshot: &'a DshConfigSnapshot, id: &str) -> &'a DshConfigEntryView {
        snapshot
            .entries
            .iter()
            .find(|entry| entry.id == id)
            .unwrap_or_else(|| panic!("entry {id} missing"))
    }

    #[test]
    fn composes_bundle_profile_and_home_layers() {
        let temp = TempDir::new().expect("temp");
        let home = desktop_home(&temp);
        write_patch(
            home,
            "# header\n- id: agent-default-model\n  name: \"@deepseek-ai/dsh-agent-default-model\"\n  config:\n    reasoningEffort: high\n",
        );
        fs::write(
            dsh_home_patch_path_for_home(home),
            "- id: subagent\n  config:\n    maxDepth: 3\n",
        )
        .expect("home patch");

        let snapshot =
            read_dsh_config_snapshot_with_schema(home, None, &test_schema()).expect("snapshot");
        assert_eq!(snapshot.profile.as_deref(), Some("desktop"));
        assert_eq!(snapshot.patch_hash.len(), 16);

        let model = entry(&snapshot, "agent-default-model");
        assert!(model.overridden);
        assert_eq!(model.layer, "profile");
        assert_eq!(model.config, Some(json!({"reasoningEffort": "high"})));

        let subagent = entry(&snapshot, "subagent");
        assert_eq!(subagent.layer, "home");
        assert!(subagent.shadowed_by_home);
        assert_eq!(subagent.config, Some(json!({"maxDepth": 3})));

        let bash = entry(&snapshot, "tool-bash");
        assert!(bash.disabled);
    }

    #[test]
    fn field_views_expose_values_and_overrides() {
        let temp = TempDir::new().expect("temp");
        let home = desktop_home(&temp);
        write_patch(
            home,
            "- id: agent-default-model\n  config:\n    provider: deepseek-official\n    model: deepseek-flash\n    reasoningEffort: high\n",
        );
        let snapshot =
            read_dsh_config_snapshot_with_schema(home, None, &test_schema()).expect("snapshot");
        let model = entry(&snapshot, "agent-default-model");
        let provider = model
            .fields
            .iter()
            .find(|field| field.name == "provider")
            .expect("provider field");
        assert_eq!(provider.value, Some(json!("deepseek-official")));
        assert!(provider.overridden);
        let effort = model
            .fields
            .iter()
            .find(|field| field.name == "reasoningEffort")
            .expect("effort field");
        assert_eq!(effort.enum_values, vec!["off", "high"]);
    }

    #[test]
    fn saves_and_removes_profile_override() {
        let temp = TempDir::new().expect("temp");
        let home = desktop_home(&temp);
        write_patch(home, "# keep me\n- id: tools\n  name: \"@deepseek-ai/dsh-tools\"\n  config: {mode: native}\n");

        let snapshot =
            read_dsh_config_snapshot_with_schema(home, None, &test_schema()).expect("snapshot");
        let result = save_dsh_config_entry_with_schema(
            home,
            Some("desktop".to_string()),
            "tools",
            Some(json!({"mode": "ptc"})),
            Some(snapshot.patch_hash),
            &test_schema(),
        )
        .expect("save");
        assert!(result.changed);

        let text =
            fs::read_to_string(dsh_profile_patch_path_for_home(home, "desktop")).expect("patch");
        assert!(text.starts_with("# keep me\n"), "comment preserved: {text}");
        assert!(text.contains("mode: ptc"));

        let after =
            read_dsh_config_snapshot_with_schema(home, None, &test_schema()).expect("snapshot");
        assert_eq!(entry(&after, "tools").config, Some(json!({"mode": "ptc"})));

        let removed = save_dsh_config_entry_with_schema(
            home,
            Some("desktop".to_string()),
            "tools",
            None,
            Some(after.patch_hash),
            &test_schema(),
        )
        .expect("remove");
        assert!(removed.changed);
        let text =
            fs::read_to_string(dsh_profile_patch_path_for_home(home, "desktop")).expect("patch");
        assert!(!text.contains("tools"));
        assert!(text.starts_with("# keep me\n"));
    }

    #[test]
    fn refuses_stale_patch_hash() {
        let temp = TempDir::new().expect("temp");
        let home = desktop_home(&temp);
        write_patch(home, "[]\n");
        let error = save_dsh_config_entry_with_schema(
            home,
            Some("desktop".to_string()),
            "subagent",
            Some(json!({"maxDepth": 2})),
            Some("deadbeefdeadbeef".to_string()),
            &test_schema(),
        )
        .expect_err("stale hash must fail");
        assert!(error.starts_with(DSH_CONFIG_CONFLICT), "{error}");
    }

    #[test]
    fn refuses_home_patch_shadowed_entry() {
        let temp = TempDir::new().expect("temp");
        let home = desktop_home(&temp);
        fs::write(
            dsh_home_patch_path_for_home(home),
            "- id: subagent\n  config:\n    maxDepth: 2\n",
        )
        .expect("home patch");
        let error = save_dsh_config_entry_with_schema(
            home,
            Some("desktop".to_string()),
            "subagent",
            Some(json!({"maxDepth": 2})),
            None,
            &test_schema(),
        )
        .expect_err("shadowed entry must fail");
        assert!(error.starts_with(DSH_CONFIG_SHADOWED), "{error}");
    }

    #[test]
    fn writes_empty_document_as_array_and_keeps_comments() {
        let temp = TempDir::new().expect("temp");
        let home = desktop_home(&temp);
        // A comments-only file parses as an empty document; DSH would refuse
        // it at boot, so the writer always replaces it with `[]`.
        write_patch(home, "# only comments\n");
        let snapshot = read_dsh_config_snapshot_with_schema(home, None, &test_schema())
            .expect("comments-only snapshot");
        assert!(snapshot.entries.iter().any(|entry| entry.id == "tools"));
        write_patch(home, "[]\n");
        let snapshot =
            read_dsh_config_snapshot_with_schema(home, None, &test_schema()).expect("snapshot");
        assert!(snapshot.entries.iter().any(|entry| entry.id == "tools"));
    }

    #[test]
    fn round_trips_js_expressions() {
        let temp = TempDir::new().expect("temp");
        let home = desktop_home(&temp);
        write_patch(
            home,
            "- id: tools\n  config:\n    mode: !!js process.env.DSH_TOOLS_MODE ?? 'native'\n",
        );
        let snapshot =
            read_dsh_config_snapshot_with_schema(home, None, &test_schema()).expect("snapshot");
        let tools = entry(&snapshot, "tools");
        assert!(tools.has_js);
        assert_eq!(
            tools.config,
            Some(json!({"mode": {"$js": "process.env.DSH_TOOLS_MODE ?? 'native'"}}))
        );

        save_dsh_config_entry_with_schema(
            home,
            Some("desktop".to_string()),
            "tools",
            tools.config.clone(),
            Some(snapshot.patch_hash),
            &test_schema(),
        )
        .expect("save js");
        let text =
            fs::read_to_string(dsh_profile_patch_path_for_home(home, "desktop")).expect("patch");
        assert!(text.contains("!!js process.env.DSH_TOOLS_MODE"), "{text}");
    }

    #[test]
    fn hides_and_preserves_secrets() {
        let temp = TempDir::new().expect("temp");
        let home = desktop_home(&temp);
        write_patch(
            home,
            "- id: web-search-deepseek\n  config:\n    apiKey: sk-literal\n    maxUses: 7\n",
        );
        let snapshot =
            read_dsh_config_snapshot_with_schema(home, None, &test_schema()).expect("snapshot");
        let search = entry(&snapshot, "web-search-deepseek");
        assert_eq!(
            search.config,
            Some(json!({"apiKey": {"$secret": true}, "maxUses": 7}))
        );

        // Editing a sibling field keeps the secret untouched on disk.
        save_dsh_config_entry_with_schema(
            home,
            Some("desktop".to_string()),
            "web-search-deepseek",
            search.config.clone(),
            Some(snapshot.patch_hash),
            &test_schema(),
        )
        .expect("save");
        let text =
            fs::read_to_string(dsh_profile_patch_path_for_home(home, "desktop")).expect("patch");
        assert!(!text.contains("sk-literal"), "{text}");

        // Plaintext secrets are refused.
        let error = save_dsh_config_entry_with_schema(
            home,
            Some("desktop".to_string()),
            "web-search-deepseek",
            Some(json!({"apiKey": "sk-new", "maxUses": 8})),
            None,
            &test_schema(),
        )
        .expect_err("plaintext secret must fail");
        assert!(error.contains("secret"), "{error}");
    }

    #[test]
    fn toggles_entry_disabled() {
        let temp = TempDir::new().expect("temp");
        let home = desktop_home(&temp);
        write_patch(home, "[]\n");
        let snapshot =
            read_dsh_config_snapshot_with_schema(home, None, &test_schema()).expect("snapshot");
        let result = set_dsh_config_entry_disabled_with_schema(
            home,
            Some("desktop".to_string()),
            "subagent",
            true,
            Some(snapshot.patch_hash),
            &test_schema(),
        )
        .expect("disable");
        assert!(result.changed);
        let text =
            fs::read_to_string(dsh_profile_patch_path_for_home(home, "desktop")).expect("patch");
        assert!(text.contains("disabled: true"), "{text}");

        let after =
            read_dsh_config_snapshot_with_schema(home, None, &test_schema()).expect("snapshot");
        assert!(entry(&after, "subagent").disabled);

        set_dsh_config_entry_disabled_with_schema(
            home,
            Some("desktop".to_string()),
            "subagent",
            false,
            Some(after.patch_hash),
            &test_schema(),
        )
        .expect("enable");
        let after =
            read_dsh_config_snapshot_with_schema(home, None, &test_schema()).expect("snapshot");
        assert!(!entry(&after, "subagent").disabled);
    }

    #[test]
    fn validates_numeric_and_enum_fields() {
        let temp = TempDir::new().expect("temp");
        let home = desktop_home(&temp);
        write_patch(home, "[]\n");
        let schema = test_schema();
        let number_error = save_dsh_config_entry_with_schema(
            home,
            Some("desktop".to_string()),
            "subagent",
            Some(json!({"maxDepth": "two"})),
            None,
            &schema,
        )
        .expect_err("string number must fail");
        assert!(number_error.contains("number"), "{number_error}");

        let enum_error = save_dsh_config_entry_with_schema(
            home,
            Some("desktop".to_string()),
            "agent-default-model",
            Some(json!({"reasoningEffort": "turbo"})),
            None,
            &schema,
        )
        .expect_err("unknown enum must fail");
        assert!(enum_error.contains("off"), "{enum_error}");
    }

    #[test]
    fn reports_migration_warnings() {
        let temp = TempDir::new().expect("temp");
        let home = desktop_home(&temp);
        write_patch(
            home,
            "\
- id: tools
  config:
    mode: both
- id: compaction-basic
  config:
    thresholdRatio: 0.5
- id: llm-pi-ai
  config:
    providers:
      azure-openai-responses: {}
",
        );
        let snapshot =
            read_dsh_config_snapshot_with_schema(home, None, &test_schema()).expect("snapshot");
        let codes: Vec<_> = snapshot
            .warnings
            .iter()
            .map(|warning| warning.code.as_str())
            .collect();
        assert!(codes.contains(&"DSH_CONFIG_TOOLS_MODE_BOTH"), "{codes:?}");
        assert!(codes.contains(&"DSH_CONFIG_PRESET_ONLY_ROW"), "{codes:?}");
        assert!(codes.contains(&"DSH_CONFIG_AZURE_ROUTE_KEY"), "{codes:?}");
    }

    #[test]
    fn reports_no_profile() {
        let temp = TempDir::new().expect("temp");
        let snapshot = read_dsh_config_snapshot_with_schema(home(&temp), None, &test_schema())
            .expect("snapshot");
        assert!(snapshot.profile.is_none());
        assert_eq!(snapshot.warnings[0].code, "DSH_CONFIG_NO_PROFILE");
    }

    #[test]
    fn unknown_entry_is_rejected() {
        let temp = TempDir::new().expect("temp");
        let home = desktop_home(&temp);
        write_patch(home, "[]\n");
        let error = save_dsh_config_entry_with_schema(
            home,
            Some("desktop".to_string()),
            "not-a-real-entry",
            Some(json!({})),
            None,
            &test_schema(),
        )
        .expect_err("unknown entry must fail");
        assert!(error.contains("Unknown"), "{error}");
    }

    #[test]
    fn embedded_schema_covers_the_shipped_bundles() {
        let schema = schema();
        assert!(schema.bundles.contains_key("@deepseek-ai/dsh-base"));
        assert!(schema.bundles.contains_key("@deepseek-ai/dsh-web-app"));
        assert!(
            schema.entries.len() >= 60,
            "expected the full DSH config catalog, got {} entries",
            schema.entries.len()
        );
        let model = schema
            .entries
            .get("agent-default-model")
            .expect("agent-default-model schema");
        assert!(model
            .fields
            .iter()
            .any(|field| field.name == "provider" && field.volatile));
        let search = schema
            .entries
            .get("web-search-deepseek")
            .expect("web-search-deepseek schema");
        assert!(search.fields.iter().any(|field| field.name == "maxUses"));
    }

    #[test]
    fn normalizes_js_tags_only_at_tag_positions() {
        assert_eq!(normalize_js_tags("a: !!js x"), "a: !js x");
        assert_eq!(normalize_js_tags("a: '!!js x'"), "a: '!!js x'");
        assert_eq!(normalize_js_tags("- !!js y"), "- !js y");
        assert_eq!(
            normalize_js_tags("# !!js stays a comment"),
            "# !!js stays a comment"
        );
    }

    #[test]
    fn profile_patch_path_uses_native_separators() {
        let path = dsh_profile_patch_path_for_home(Path::new("/home/user"), "desktop");
        assert!(slashes(path.to_string_lossy())
            .ends_with("/home/user/.dsh/profiles/desktop/cordis.patch.yml"));
    }
}
