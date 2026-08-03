//! Channel model info export system.
//!
//! Users define export templates in `~/.droidgear/export-templates.yaml`.
//! Each template is a "form" specifying:
//! - Which channels/tokens/models to include (filters)
//! - Which fields to output (field selectors + renames)
//! - Output format (json/yaml/toml)
//! - Output path
//!
//! The engine collects data, applies filters, flattens into records,
//! renders in the target format, and writes to disk — all at runtime
//! without recompilation.

use crate::channel::{
    fetch_channel_tokens, get_channel_api_key, get_channel_credentials, load_channels, Channel,
    ChannelToken, ChannelType,
};
use crate::factory_settings::ModelInfo;
use serde::{Deserialize, Serialize};
use specta::Type;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

// ============================================================================
// Types — the "form" schema
// ============================================================================

/// Output format
#[derive(Debug, Clone, Serialize, Deserialize, Type, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum ExportFormat {
    Json,
    Yaml,
    Toml,
}

/// Output structure
#[derive(Debug, Clone, Serialize, Deserialize, Type, PartialEq, Default)]
#[serde(rename_all = "lowercase")]
pub enum OutputStructure {
    #[default]
    Flat,
    /// Nested: channels → tokens → models
    Nested,
}

/// Channel filter conditions
#[derive(Debug, Clone, Serialize, Deserialize, Type, Default)]
#[serde(rename_all = "camelCase")]
pub struct ChannelFilter {
    /// Only include these channel types (empty = all)
    #[serde(default)]
    pub types: Vec<ChannelType>,
    /// Only include enabled channels
    #[serde(default)]
    pub enabled_only: bool,
    /// Only include channels with these IDs (empty = all)
    #[serde(default)]
    pub ids: Vec<String>,
}

/// Token filter conditions
#[derive(Debug, Clone, Serialize, Deserialize, Type, Default)]
#[serde(rename_all = "camelCase")]
pub struct TokenFilter {
    /// Token status (1=enabled, empty = all)
    pub status: Option<i32>,
    /// Only include tokens matching these platforms (empty = all)
    #[serde(default)]
    pub platforms: Vec<String>,
}

/// A single export template ("the form")
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct ExportTemplate {
    /// Template name (unique identifier)
    pub name: String,
    /// Human-readable description
    #[serde(default)]
    pub description: String,

    // ——— Filters ———
    #[serde(default)]
    pub channels: ChannelFilter,
    #[serde(default)]
    pub tokens: TokenFilter,

    /// Whether to fetch the model list from the API
    #[serde(default = "default_true")]
    pub fetch_models: bool,

    /// Optional protocol overrides by model ID prefix (glob-like)
    /// e.g. {"claude-*": "anthropic"}
    #[serde(default)]
    pub model_protocol_overrides: HashMap<String, String>,

    // ——— Field selectors ("填表" core) ———
    /// Map of source field path → output field name
    /// e.g. {"channel.name": "channel", "model.id": "model", "token.key": "apiKey"}
    /// Empty = all fields with original names
    #[serde(default)]
    pub fields: HashMap<String, String>,

    // ——— Output ———
    pub format: ExportFormat,
    #[serde(default)]
    pub output_structure: OutputStructure,
    /// Output file path (supports ~ and {timestamp})
    pub output_path: String,
}

fn default_true() -> bool {
    true
}

/// Top-level config file structure
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct ExportConfig {
    #[serde(default)]
    pub templates: Vec<ExportTemplate>,
}

// ============================================================================
// Intermediate data
// ============================================================================

/// A single flat record ready for serialization
pub type ExportRecord = HashMap<String, serde_json::Value>;

// ============================================================================
// Config path helpers
// ============================================================================

fn droidgear_dir() -> Result<PathBuf, String> {
    let home = crate::paths::get_home_dir()?;
    Ok(home.join(".droidgear"))
}

fn config_path() -> Result<PathBuf, String> {
    Ok(droidgear_dir()?.join("export-templates.yaml"))
}

/// Load export templates from config file.
/// Returns empty vec if file doesn't exist.
pub fn load_export_templates() -> Result<Vec<ExportTemplate>, String> {
    let path = config_path()?;
    if !path.exists() {
        return Ok(vec![]);
    }
    let content =
        fs::read_to_string(&path).map_err(|e| format!("Failed to read export templates: {e}"))?;
    let config: ExportConfig = serde_yaml::from_str(&content)
        .map_err(|e| format!("Failed to parse export templates: {e}"))?;
    Ok(config.templates)
}

/// Save all export templates to config file.
pub fn save_export_templates(templates: &[ExportTemplate]) -> Result<(), String> {
    let dir = droidgear_dir()?;
    fs::create_dir_all(&dir).map_err(|e| format!("Failed to create .droidgear directory: {e}"))?;

    let config = ExportConfig {
        templates: templates.to_vec(),
    };
    let content = serde_yaml::to_string(&config)
        .map_err(|e| format!("Failed to serialize export templates: {e}"))?;

    let path = config_path()?;
    let temp_path = path.with_extension("tmp");
    fs::write(&temp_path, &content)
        .map_err(|e| format!("Failed to write export templates: {e}"))?;
    fs::rename(&temp_path, &path)
        .map_err(|e| format!("Failed to finalize export templates: {e}"))?;

    Ok(())
}

// ============================================================================
// Protocol inference (mirrors frontend logic)
// ============================================================================

/// Infer model protocol from model ID.
fn infer_protocol_from_model_id(model_id: &str) -> &'static str {
    let lower = model_id.to_lowercase();

    if lower.starts_with("claude-") {
        "anthropic"
    } else if lower.starts_with("gpt-") || lower.starts_with("o1-") || lower.starts_with("o3-") {
        "openai"
    } else if lower.starts_with("gemini-") {
        "google-ai"
    } else if lower.starts_with("deepseek-") {
        "openai"
    } else {
        "openai-compatible"
    }
}

/// Infer protocol with user overrides applied.
fn resolve_protocol(model_id: &str, overrides: &HashMap<String, String>) -> String {
    // Check overrides first (simple prefix matching)
    for (pattern, protocol) in overrides {
        if pattern.ends_with('*') {
            let prefix = pattern.trim_end_matches('*');
            if model_id.starts_with(prefix) {
                return protocol.clone();
            }
        } else if model_id == pattern {
            return protocol.clone();
        }
    }
    infer_protocol_from_model_id(model_id).to_string()
}

// ============================================================================
// Field extraction helpers
// ============================================================================

/// Build a record from channel + token + model data, selecting only the fields specified.
fn build_flat_record(
    channel: &Channel,
    token: &ChannelToken,
    model: Option<&ModelInfo>,
    protocol: Option<&str>,
    fields: &HashMap<String, String>,
) -> ExportRecord {
    let all_fields: HashMap<&str, serde_json::Value> = HashMap::from([
        // Channel fields
        ("channel.id", serde_json::Value::String(channel.id.clone())),
        (
            "channel.name",
            serde_json::Value::String(channel.name.clone()),
        ),
        (
            "channel.type",
            serde_json::Value::String(format!("{:?}", channel.channel_type)),
        ),
        (
            "channel.baseUrl",
            serde_json::Value::String(channel.base_url.clone()),
        ),
        ("channel.enabled", serde_json::Value::Bool(channel.enabled)),
        // Token fields
        ("token.name", serde_json::Value::String(token.name.clone())),
        ("token.key", serde_json::Value::String(token.key.clone())),
        (
            "token.status",
            serde_json::Value::Number(serde_json::Number::from(token.status)),
        ),
        (
            "token.platform",
            token
                .platform
                .as_ref()
                .map_or(serde_json::Value::Null, |p| {
                    serde_json::Value::String(p.clone())
                }),
        ),
        (
            "token.groupName",
            token
                .group_name
                .as_ref()
                .map_or(serde_json::Value::Null, |g| {
                    serde_json::Value::String(g.clone())
                }),
        ),
        (
            "token.remainQuota",
            serde_json::Value::Number(
                serde_json::Number::from_f64(token.remain_quota)
                    .unwrap_or(serde_json::Number::from(0)),
            ),
        ),
        (
            "token.usedQuota",
            serde_json::Value::Number(
                serde_json::Number::from_f64(token.used_quota)
                    .unwrap_or(serde_json::Number::from(0)),
            ),
        ),
        (
            "token.unlimitedQuota",
            serde_json::Value::Bool(token.unlimited_quota),
        ),
    ]);

    let record: ExportRecord = if fields.is_empty() {
        // No field selection = include all
        // Convert all_fields from HashMap<&str, Value> to ExportRecord (HashMap<String, Value>)
        let mut r = ExportRecord::new();
        for (k, v) in &all_fields {
            r.insert(k.to_string(), v.clone());
        }
        if let Some(m) = model {
            r.insert(
                "model.id".to_string(),
                serde_json::Value::String(m.id.clone()),
            );
            r.insert(
                "model.name".to_string(),
                m.name.as_ref().map_or(serde_json::Value::Null, |n| {
                    serde_json::Value::String(n.clone())
                }),
            );
            r.insert(
                "model.protocol".to_string(),
                serde_json::Value::String(protocol.unwrap_or("openai-compatible").to_string()),
            );
        }
        r
    } else {
        let mut r = ExportRecord::new();
        for (source, output_name) in fields {
            let value = if let Some(v) = all_fields.get(source.as_str()) {
                v.clone()
            } else if source == "model.id" {
                model.map_or(serde_json::Value::Null, |m| {
                    serde_json::Value::String(m.id.clone())
                })
            } else if source == "model.name" {
                model
                    .and_then(|m| m.name.as_ref())
                    .map_or(serde_json::Value::Null, |n| {
                        serde_json::Value::String(n.clone())
                    })
            } else if source == "model.protocol" {
                serde_json::Value::String(protocol.unwrap_or("openai-compatible").to_string())
            } else {
                continue;
            };
            r.insert(output_name.clone(), value);
        }
        r
    };

    record
}

// ============================================================================
// Token auth resolution
// ============================================================================

/// Resolve auth and fetch tokens for a channel.
fn fetch_tokens_for_channel(channel: &Channel) -> Result<Vec<ChannelToken>, String> {
    match channel.channel_type {
        ChannelType::NewApi | ChannelType::Sub2Api => {
            let creds = get_channel_credentials(&channel.id)?;
            match creds {
                Some((username, password)) => {
                    fetch_channel_tokens_blocking(channel, &username, &password)
                }
                None => Err(format!("No credentials for channel '{}'", channel.name)),
            }
        }
        ChannelType::CliProxyApi
        | ChannelType::Ollama
        | ChannelType::General
        | ChannelType::DeepSeek => {
            let api_key = get_channel_api_key(&channel.id)?;
            match api_key {
                Some(key) => {
                    // For API-key-based channels, return a single synthetic token
                    Ok(vec![ChannelToken {
                        id: 0.0,
                        name: "API Key".to_string(),
                        key,
                        status: 1,
                        remain_quota: 0.0,
                        used_quota: 0.0,
                        unlimited_quota: true,
                        platform: None,
                        group_name: None,
                    }])
                }
                None => Err(format!("No API key for channel '{}'", channel.name)),
            }
        }
    }
}

fn fetch_channel_tokens_blocking(
    _channel: &Channel,
    username: &str,
    password: &str,
) -> Result<Vec<ChannelToken>, String> {
    // Use the existing async function via blocking wrapper
    let channel_type = _channel.channel_type.clone();
    let base_url = _channel.base_url.clone();
    let username = username.to_string();
    let password = password.to_string();

    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .map_err(|e| format!("Failed to create tokio runtime: {e}"))?;
    runtime.block_on(fetch_channel_tokens(
        channel_type,
        &base_url,
        &username,
        &password,
    ))
}

// ============================================================================
// Model fetching
// ============================================================================

fn fetch_models_blocking(
    base_url: &str,
    api_key: &str,
    platform: Option<&str>,
) -> Result<Vec<ModelInfo>, String> {
    crate::channel::fetch_models_by_api_key_blocking(base_url, api_key, platform)
}

// ============================================================================
// Export engine — the main entry point
// ============================================================================

/// Result of an export run.
#[derive(Debug, Clone, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct ExportResult {
    /// Template name
    pub template: String,
    /// Number of channels processed
    pub channels_count: usize,
    /// Number of tokens processed
    pub tokens_count: usize,
    /// Number of models exported
    pub models_count: usize,
    /// Output file path
    pub output_path: String,
    /// Record count (rows written)
    pub record_count: usize,
    /// Any warnings
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub warnings: Vec<String>,
}

/// Run a single export template: collect data, format, write.
pub fn run_export(template: &ExportTemplate) -> Result<ExportResult, String> {
    let mut warnings: Vec<String> = vec![];
    let mut records: Vec<ExportRecord> = vec![];
    let mut token_count = 0;
    let mut model_count = 0;

    // 1. Load channels
    let all_channels = load_channels()?;

    // 2. Filter channels
    let channels: Vec<&Channel> = all_channels
        .iter()
        .filter(|ch| {
            if template.channels.enabled_only && !ch.enabled {
                return false;
            }
            if !template.channels.types.is_empty()
                && !template.channels.types.contains(&ch.channel_type)
            {
                return false;
            }
            if !template.channels.ids.is_empty() && !template.channels.ids.contains(&ch.id) {
                return false;
            }
            true
        })
        .collect();

    // 3. For each channel, fetch tokens
    for channel in &channels {
        let tokens = match fetch_tokens_for_channel(channel) {
            Ok(t) => t,
            Err(e) => {
                warnings.push(format!("Channel '{}': {e}", channel.name));
                continue;
            }
        };

        // 4. Filter tokens
        let filtered_tokens: Vec<&ChannelToken> = tokens
            .iter()
            .filter(|t| {
                if let Some(status) = template.tokens.status {
                    if t.status != status {
                        return false;
                    }
                }
                if !template.tokens.platforms.is_empty() {
                    let platform = t.platform.as_deref().unwrap_or("");
                    if !template.tokens.platforms.iter().any(|p| p == platform) {
                        return false;
                    }
                }
                true
            })
            .collect();

        for token in &filtered_tokens {
            token_count += 1;

            if template.fetch_models {
                // Fetch models from the API
                let platform = token.platform.as_deref();
                let models = match fetch_models_blocking(&channel.base_url, &token.key, platform) {
                    Ok(m) => m,
                    Err(e) => {
                        warnings.push(format!(
                            "Channel '{}', token '{}': {e}",
                            channel.name, token.name
                        ));
                        vec![]
                    }
                };

                if models.is_empty() {
                    // Still emit a record with channel + token info, no model
                    let record = build_flat_record(channel, token, None, None, &template.fields);
                    records.push(record);
                } else {
                    for m in &models {
                        let protocol = resolve_protocol(&m.id, &template.model_protocol_overrides);
                        let record = build_flat_record(
                            channel,
                            token,
                            Some(m),
                            Some(&protocol),
                            &template.fields,
                        );
                        records.push(record);
                        model_count += 1;
                    }
                }
            } else {
                // No model fetching — just channel + token info
                let record = build_flat_record(channel, token, None, None, &template.fields);
                records.push(record);
            }
        }
    }

    // 5. Resolve output path (expand ~ and {timestamp})
    let mut output_path = resolve_output_path(&template.output_path)?;

    // 6. Ensure file extension matches the selected format
    ensure_extension(&mut output_path, &template.format);

    // 7. Render and write
    match template.format {
        ExportFormat::Json => render_json(&records, &output_path)?,
        ExportFormat::Yaml => render_yaml(&records, &output_path)?,
        ExportFormat::Toml => render_toml(&records, &output_path)?,
    }

    Ok(ExportResult {
        template: template.name.clone(),
        channels_count: channels.len(),
        tokens_count: token_count,
        models_count: model_count,
        output_path: output_path.to_string_lossy().to_string(),
        record_count: records.len(),
        warnings,
    })
}

// ============================================================================
// Output path resolution
/// Ensure the file extension matches the output format.
/// Replaces the extension if it doesn't match; appends one if there's no extension.
fn ensure_extension(path: &mut PathBuf, format: &ExportFormat) {
    let expected = match format {
        ExportFormat::Json => "json",
        ExportFormat::Yaml => "yaml",
        ExportFormat::Toml => "toml",
    };

    if let Some(ext) = path.extension() {
        let current = ext.to_string_lossy().to_lowercase();
        if current != expected {
            path.set_extension(expected);
        }
    } else {
        path.set_extension(expected);
    }
}

fn resolve_output_path(raw: &str) -> Result<PathBuf, String> {
    let expanded = if raw.starts_with('~') {
        let home = crate::paths::get_home_dir()?;
        let rest = raw.strip_prefix('~').unwrap_or("");
        if rest.starts_with('/') || rest.starts_with('\\') {
            home.join(&rest[1..])
        } else {
            home.join(rest)
        }
    } else {
        PathBuf::from(raw)
    };

    // Replace {timestamp} placeholder
    let expanded_str = expanded.to_string_lossy().to_string();
    let timestamp = chrono::Local::now().format("%Y%m%d-%H%M%S").to_string();
    let final_path = expanded_str.replace("{timestamp}", &timestamp);

    // Create parent directory
    let path = PathBuf::from(&final_path);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| {
            format!(
                "Failed to create output directory '{}': {e}",
                parent.display()
            )
        })?;
    }

    Ok(path)
}

// ============================================================================
// Formatters
// ============================================================================

/// Render records as JSON and write to file.
fn render_json(records: &[ExportRecord], path: &Path) -> Result<(), String> {
    let value: Vec<&ExportRecord> = records.iter().collect();
    let content = serde_json::to_string_pretty(&value)
        .map_err(|e| format!("JSON serialization failed: {e}"))?;
    write_atomic(path, &content)
}

/// Render records as YAML and write to file.
fn render_yaml(records: &[ExportRecord], path: &Path) -> Result<(), String> {
    let content =
        serde_yaml::to_string(&records).map_err(|e| format!("YAML serialization failed: {e}"))?;
    write_atomic(path, &content)
}

/// Render records as TOML and write to file.
/// TOML requires an array of tables — we serialize as `[[record]]` array.
fn render_toml(records: &[ExportRecord], path: &Path) -> Result<(), String> {
    // TOML doesn't natively support top-level arrays of tables in the spec
    // as a standalone document well. We wrap in a `records` key.
    let wrapper = serde_json::json!({ "records": records });
    // Convert through serde_value to toml
    let toml_value: toml::Value =
        toml::Value::try_from(&wrapper).map_err(|e| format!("TOML conversion failed: {e}"))?;
    let content = toml::to_string_pretty(&toml_value)
        .map_err(|e| format!("TOML serialization failed: {e}"))?;
    write_atomic(path, &content)
}

/// Atomic file write: temp file + rename.
fn write_atomic(path: &Path, content: &str) -> Result<(), String> {
    let temp_path = path.with_extension("tmp");
    fs::write(&temp_path, content)
        .map_err(|e| format!("Failed to write to '{}': {e}", temp_path.display()))?;
    fs::rename(&temp_path, path)
        .map_err(|e| format!("Failed to rename temp file to '{}': {e}", path.display()))?;
    Ok(())
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_infer_protocol() {
        assert_eq!(
            infer_protocol_from_model_id("claude-sonnet-4-20250514"),
            "anthropic"
        );
        assert_eq!(infer_protocol_from_model_id("gpt-4o"), "openai");
        assert_eq!(
            infer_protocol_from_model_id("gemini-2.0-flash"),
            "google-ai"
        );
        assert_eq!(infer_protocol_from_model_id("deepseek-chat"), "openai");
        assert_eq!(
            infer_protocol_from_model_id("unknown-model"),
            "openai-compatible"
        );
    }

    #[test]
    fn test_resolve_protocol_with_overrides() {
        let mut overrides = HashMap::new();
        overrides.insert("custom-*".to_string(), "anthropic".to_string());
        assert_eq!(resolve_protocol("custom-v1", &overrides), "anthropic");
        assert_eq!(resolve_protocol("gpt-4o", &overrides), "openai"); // fallback to built-in
    }

    #[test]
    fn test_resolve_output_path_with_tilde() {
        let path = resolve_output_path("~/output/models.yaml").unwrap();
        assert!(path.to_string_lossy().ends_with("output/models.yaml"));
    }

    #[test]
    fn test_resolve_output_path_with_timestamp() {
        let path = resolve_output_path("/tmp/export-{timestamp}.json").unwrap();
        let name = path.to_string_lossy();
        assert!(name.starts_with("/tmp/export-"));
        assert!(name.ends_with(".json"));
    }

    #[test]
    fn test_build_flat_record_all_fields() {
        let channel = Channel {
            id: "ch-1".to_string(),
            name: "Test Channel".to_string(),
            channel_type: ChannelType::NewApi,
            base_url: "https://api.example.com".to_string(),
            enabled: true,
            created_at: 1000.0,
        };
        let token = ChannelToken {
            id: 1.0,
            name: "Default Key".to_string(),
            key: "sk-test".to_string(),
            status: 1,
            remain_quota: 1000.0,
            used_quota: 500.0,
            unlimited_quota: false,
            platform: Some("anthropic".to_string()),
            group_name: Some("group-1".to_string()),
        };
        let model = ModelInfo {
            id: "claude-sonnet-4".to_string(),
            name: Some("Claude Sonnet 4".to_string()),
        };

        let fields = HashMap::new(); // empty = all fields
        let record = build_flat_record(&channel, &token, Some(&model), Some("anthropic"), &fields);

        assert_eq!(
            record.get("channel.name"),
            Some(&serde_json::Value::String("Test Channel".to_string()))
        );
        assert_eq!(
            record.get("model.id"),
            Some(&serde_json::Value::String("claude-sonnet-4".to_string()))
        );
        assert_eq!(
            record.get("model.protocol"),
            Some(&serde_json::Value::String("anthropic".to_string()))
        );
        assert_eq!(
            record.get("token.key"),
            Some(&serde_json::Value::String("sk-test".to_string()))
        );
    }

    #[test]
    fn test_build_flat_record_selected_fields_with_rename() {
        let channel = Channel {
            id: "ch-1".to_string(),
            name: "Test Channel".to_string(),
            channel_type: ChannelType::NewApi,
            base_url: "https://api.example.com".to_string(),
            enabled: true,
            created_at: 1000.0,
        };
        let token = ChannelToken {
            id: 1.0,
            name: "Default Key".to_string(),
            key: "sk-test".to_string(),
            status: 1,
            remain_quota: 1000.0,
            used_quota: 500.0,
            unlimited_quota: false,
            platform: Some("anthropic".to_string()),
            group_name: None,
        };
        let model = ModelInfo {
            id: "claude-sonnet-4".to_string(),
            name: None,
        };

        let mut fields = HashMap::new();
        fields.insert("channel.name".to_string(), "channel".to_string());
        fields.insert("model.id".to_string(), "model".to_string());
        fields.insert("model.protocol".to_string(), "protocol".to_string());
        fields.insert("token.key".to_string(), "apiKey".to_string());

        let record = build_flat_record(&channel, &token, Some(&model), Some("anthropic"), &fields);

        assert_eq!(
            record.get("channel"),
            Some(&serde_json::Value::String("Test Channel".to_string()))
        );
        assert_eq!(
            record.get("model"),
            Some(&serde_json::Value::String("claude-sonnet-4".to_string()))
        );
        assert_eq!(
            record.get("protocol"),
            Some(&serde_json::Value::String("anthropic".to_string()))
        );
        assert_eq!(
            record.get("apiKey"),
            Some(&serde_json::Value::String("sk-test".to_string()))
        );
        // Unselected fields should NOT be present
        assert!(!record.contains_key("token.name"));
        assert!(!record.contains_key("channel.baseUrl"));
    }
}
