//! Playground video models and protocol clients.
//!
//! Supports three upstream protocols:
//!
//! - `sora`: New API's OpenAI-compatible Videos host protocol at
//!   `/v1/videos`, `/v1/videos/{id}` and `/v1/videos/{id}/content`.
//! - `new-api`: New API's native video wire at `/v1/video/generations`.
//! - `doubao`: Ark's native task wire at
//!   `/api/v3/contents/generations/tasks`. New API relays the same wire under
//!   `/doubao`, so a New API channel gets that prefix automatically.
//!
//! The playground only reads channels: credentials are resolved from the
//! channel store at request time (API-key channels directly, username/password
//! channels through their token list). Custom endpoints keep their API key in
//! a separate owner-only file. Models and task history live under
//! `~/.droidgear/playground/`.

use serde::{Deserialize, Serialize};
use serde_json::{json, Map, Value};
use specta::Type;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use crate::channel::ChannelType;
use crate::playground::{self, authorized, format_api_error, send_json, snippet, ResolvedSource};

const VIDEO_MODELS_FILE: &str = "video_models.json";
const VIDEO_TASKS_FILE: &str = "video_tasks.json";
const VIDEO_SECRETS_FILE: &str = "video_secrets.json";
const CREATE_TIMEOUT: Duration = Duration::from_secs(120);
const QUERY_TIMEOUT: Duration = Duration::from_secs(30);
const CONTENT_TIMEOUT: Duration = Duration::from_secs(300);

// ============================================================================
// Types
// ============================================================================

/// Video wire protocol used by a playground model.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, Type, PartialEq, Eq, Default)]
#[serde(rename_all = "kebab-case")]
pub enum VideoProtocol {
    /// New API's OpenAI-compatible Videos wire (`/v1/videos`).
    #[serde(alias = "new-api-videos")]
    #[default]
    Sora,
    /// New API's native video generation wire (`/v1/video/generations`).
    NewApi,
    /// Doubao / Ark task API (`/api/v3/contents/generations/tasks`).
    Doubao,
}

/// Where a video model gets its endpoint and credentials.
#[derive(Debug, Clone, Serialize, Deserialize, Type, PartialEq)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum VideoSource {
    /// Reuse an existing channel. `token_id` is the token selected when the
    /// model was imported; `None` lets the resolver pick the first active one.
    Channel {
        #[serde(rename = "channelId")]
        channel_id: String,
        #[serde(rename = "tokenId")]
        token_id: Option<f64>,
    },
    /// A standalone endpoint with its own API key.
    Custom {
        #[serde(rename = "baseUrl")]
        base_url: String,
        #[serde(rename = "apiKey")]
        api_key: String,
    },
}

/// A video model configured for the playground.
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct VideoModel {
    pub id: String,
    pub name: String,
    pub model_id: String,
    pub source: VideoSource,
    pub protocol: VideoProtocol,
    /// Editable suggestion lists. Older files may predate them; defaulting
    /// keeps the rest of the file (and its other models) loadable instead of
    /// failing the whole model list over two optional presets.
    #[serde(default)]
    pub durations: Vec<u32>,
    #[serde(default)]
    pub resolutions: Vec<String>,
    pub supports_image_input: bool,
}

/// Normalized task states shared by both upstream protocols.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, Type, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum VideoTaskStatus {
    Queued,
    InProgress,
    Completed,
    Failed,
    Cancelled,
}

/// One protocol query result, normalized for the playground UI.
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct VideoTaskSnapshot {
    pub id: String,
    pub status: VideoTaskStatus,
    pub progress: u32,
    pub video_url: Option<String>,
    pub local_path: Option<String>,
    pub error: Option<String>,
    /// Milliseconds since the Unix epoch, when the upstream reports one.
    pub completed_at: Option<i64>,
}

/// Form values captured when a video task was submitted.
#[derive(Debug, Clone, Serialize, Deserialize, Type, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct VideoTaskParams {
    pub prompt: String,
    pub duration: String,
    pub resolution: String,
    pub image_url: String,
    pub extra_json: String,
}

/// A playground task persisted under `~/.droidgear/playground/video_tasks.json`.
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct VideoTaskRecord {
    pub id: String,
    pub model_id: String,
    pub model_name: String,
    /// Snapshot of the model at submit time. Custom API keys are stripped on
    /// disk and merged back from `video_secrets.json` on load.
    pub model: VideoModel,
    pub params: VideoTaskParams,
    pub status: VideoTaskStatus,
    pub progress: u32,
    pub created_at: i64,
    pub completed_at: Option<i64>,
    pub video_url: Option<String>,
    pub local_path: Option<String>,
    pub error: Option<String>,
}

// ============================================================================
// Paths and persistence
// ============================================================================

pub fn video_models_path_for_home(home: &Path) -> PathBuf {
    playground::playground_dir(home).join(VIDEO_MODELS_FILE)
}

pub fn video_tasks_path_for_home(home: &Path) -> PathBuf {
    playground::playground_dir(home).join(VIDEO_TASKS_FILE)
}

pub fn video_secrets_path_for_home(home: &Path) -> PathBuf {
    playground::playground_dir(home).join(VIDEO_SECRETS_FILE)
}

pub fn video_media_dir_for_home(home: &Path) -> PathBuf {
    playground::media_dir(home)
}

/// Loads playground video models, merging custom-endpoint API keys back in.
pub fn load_video_models() -> Result<Vec<VideoModel>, String> {
    load_video_models_for_home(&crate::paths::get_home_dir()?)
}

pub fn load_video_models_for_home(home: &Path) -> Result<Vec<VideoModel>, String> {
    let path = video_models_path_for_home(home);
    if !path.exists() {
        return Ok(Vec::new());
    }
    let content =
        fs::read_to_string(&path).map_err(|e| format!("Failed to read video models file: {e}"))?;
    if content.trim().is_empty() {
        return Ok(Vec::new());
    }
    let mut models: Vec<VideoModel> = serde_json::from_str(&content)
        .map_err(|e| format!("Failed to parse video models file: {e}"))?;
    let secrets = load_secrets_for_home(home);
    for model in &mut models {
        if let VideoSource::Custom { api_key, .. } = &mut model.source {
            if let Some(secret) = secrets.get(&model.id) {
                *api_key = secret.clone();
            }
        }
    }
    Ok(models)
}

/// Saves playground video models. Custom-endpoint API keys are stripped from
/// the model file and written to an owner-only secrets file instead.
pub fn save_video_models(models: &[VideoModel]) -> Result<(), String> {
    save_video_models_for_home(&crate::paths::get_home_dir()?, models)
}

pub fn save_video_models_for_home(home: &Path, models: &[VideoModel]) -> Result<(), String> {
    let mut sanitized = models.to_vec();
    let mut secrets = HashMap::new();
    for model in &mut sanitized {
        if let VideoSource::Custom { api_key, .. } = &mut model.source {
            let trimmed = api_key.trim();
            if !trimmed.is_empty() {
                secrets.insert(model.id.clone(), trimmed.to_string());
            }
            api_key.clear();
        }
    }

    let dir = playground::playground_dir(home);
    fs::create_dir_all(&dir).map_err(|e| format!("Failed to create playground directory: {e}"))?;

    playground::write_secrets(&video_secrets_path_for_home(home), &secrets, "video")?;
    playground::write_json(
        &video_models_path_for_home(home),
        &sanitized,
        "video models",
    )
}

/// Loads persisted playground tasks and reattaches custom-endpoint API keys.
pub fn load_video_tasks() -> Result<Vec<VideoTaskRecord>, String> {
    load_video_tasks_for_home(&crate::paths::get_home_dir()?)
}

pub fn load_video_tasks_for_home(home: &Path) -> Result<Vec<VideoTaskRecord>, String> {
    let path = video_tasks_path_for_home(home);
    if !path.exists() {
        return Ok(Vec::new());
    }
    let content = fs::read_to_string(&path)
        .map_err(|error| format!("Failed to read video tasks file: {error}"))?;
    if content.trim().is_empty() {
        return Ok(Vec::new());
    }
    let mut tasks: Vec<VideoTaskRecord> = serde_json::from_str(&content)
        .map_err(|error| format!("Failed to parse video tasks file: {error}"))?;
    // A task id becomes a media file name; drop corrupt records rather than
    // letting them reach the download code.
    tasks.retain(|task| validate_task_id(&task.id).is_ok());

    let secrets = load_secrets_for_home(home);
    for task in &mut tasks {
        if let VideoSource::Custom { api_key, .. } = &mut task.model.source {
            if let Some(secret) = secrets.get(&task.model.id) {
                *api_key = secret.clone();
            }
        }
    }
    Ok(tasks)
}

/// Persists playground tasks. Custom-endpoint API keys are stripped from the
/// task file; they are read back from `video_secrets.json` by model id.
pub fn save_video_tasks(tasks: &[VideoTaskRecord]) -> Result<(), String> {
    save_video_tasks_for_home(&crate::paths::get_home_dir()?, tasks)
}

pub fn save_video_tasks_for_home(home: &Path, tasks: &[VideoTaskRecord]) -> Result<(), String> {
    let mut sanitized = tasks.to_vec();
    for task in &mut sanitized {
        if let VideoSource::Custom { api_key, .. } = &mut task.model.source {
            api_key.clear();
        }
    }

    let dir = playground::playground_dir(home);
    fs::create_dir_all(&dir)
        .map_err(|error| format!("Failed to create playground directory: {error}"))?;
    playground::write_json(&video_tasks_path_for_home(home), &sanitized, "video tasks")
}

fn load_secrets_for_home(home: &Path) -> HashMap<String, String> {
    playground::read_secrets(&video_secrets_path_for_home(home), "Video")
}

// ============================================================================
// Source resolution
// ============================================================================

async fn resolve_source(model: &VideoModel) -> Result<ResolvedSource, String> {
    match &model.source {
        VideoSource::Custom { base_url, api_key } => {
            playground::resolve_custom_source(base_url, api_key)
        }
        VideoSource::Channel {
            channel_id,
            token_id,
        } => playground::resolve_channel_source(channel_id, *token_id).await,
    }
}

// ============================================================================
// URLs and request bodies
// ============================================================================

fn trimmed(base: &str) -> &str {
    base.trim().trim_end_matches('/')
}

/// Base URL that ends at the collection of videos, e.g.
/// `https://gateway.example/v1/videos`.
fn sora_videos_base(base: &str) -> String {
    let base = trimmed(base);
    if base.ends_with("/v1/videos") {
        base.to_string()
    } else if base.ends_with("/v1") {
        format!("{base}/videos")
    } else {
        format!("{base}/v1/videos")
    }
}

/// Base URL for New API's native generation endpoint.
fn new_api_generations_base(base: &str) -> String {
    let base = trimmed(base);
    if base.ends_with("/v1/video/generations") {
        base.to_string()
    } else if base.ends_with("/v1") {
        format!("{base}/video/generations")
    } else {
        format!("{base}/v1/video/generations")
    }
}

/// Base URL that ends at Ark's task collection.
///
/// New API serves the doubao plugin's native wire under `/doubao`, so a New
/// API channel gets that prefix unless its base URL already carries it. A
/// custom endpoint is used as-is, which lets the user include `/doubao`
/// themselves when they point at a New API-compatible gateway.
fn doubao_tasks_base(base: &str, channel_type: Option<&ChannelType>) -> String {
    let mut base = trimmed(base).to_string();
    for suffix in [
        "/api/v3/contents/generations/tasks",
        "/api/v3/contents/generations",
    ] {
        if let Some(stripped) = base.strip_suffix(suffix) {
            base = stripped.trim_end_matches('/').to_string();
            break;
        }
    }
    if matches!(channel_type, Some(ChannelType::NewApi)) && !base.ends_with("/doubao") {
        base.push_str("/doubao");
    }
    format!(
        "{}/api/v3/contents/generations/tasks",
        base.trim_end_matches('/')
    )
}

fn validate_task_id(task_id: &str) -> Result<(), String> {
    if task_id.is_empty() {
        return Err("Task ID is empty.".to_string());
    }
    if task_id.len() > 200
        || !task_id
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, '-' | '_' | '.'))
    {
        return Err("Task ID contains unsupported characters.".to_string());
    }
    Ok(())
}

struct CreateParams<'a> {
    model: &'a str,
    prompt: &'a str,
    duration: Option<u32>,
    resolution: Option<&'a str>,
    image_urls: Vec<&'a str>,
    extra: Map<String, Value>,
}

/// Resolution split into the shapes the three wires need.
#[derive(Debug, Clone, Copy)]
struct ResolutionInfo {
    width: Option<u32>,
    height: Option<u32>,
    tier: Option<&'static str>,
}

fn parse_resolution(value: &str) -> ResolutionInfo {
    let raw = value.trim();
    let lower = raw.to_ascii_lowercase();
    let tier = match lower.as_str() {
        "480p" => Some(("480p", 854, 480)),
        "720p" => Some(("720p", 1280, 720)),
        "1080p" => Some(("1080p", 1920, 1080)),
        "4k" | "2160p" => Some(("4k", 3840, 2160)),
        _ => None,
    };
    if let Some((tier, width, height)) = tier {
        return ResolutionInfo {
            width: Some(width),
            height: Some(height),
            tier: Some(tier),
        };
    }

    let normalized = raw.to_ascii_lowercase().replace('*', "x");
    let mut parts = normalized.split('x');
    if let (Some(width), Some(height), None) = (parts.next(), parts.next(), parts.next()) {
        if let (Ok(width), Ok(height)) = (width.trim().parse::<u32>(), height.trim().parse::<u32>())
        {
            if width > 0 && height > 0 {
                let longest = width.max(height);
                let tier = if longest >= 3840 {
                    Some("4k")
                } else if longest >= 1920 {
                    Some("1080p")
                } else if longest >= 1280 {
                    Some("720p")
                } else {
                    Some("480p")
                };
                return ResolutionInfo {
                    width: Some(width),
                    height: Some(height),
                    tier,
                };
            }
        }
    }
    ResolutionInfo {
        width: None,
        height: None,
        tier: None,
    }
}

fn parse_extra(extra_json: &str) -> Result<Map<String, Value>, String> {
    if extra_json.trim().is_empty() {
        return Ok(Map::new());
    }
    match serde_json::from_str::<Value>(extra_json) {
        Ok(Value::Object(map)) => Ok(map),
        Ok(_) => Err("Advanced parameters must be a JSON object.".to_string()),
        Err(error) => Err(format!("Advanced parameters are not valid JSON: {error}")),
    }
}

/// Doubao/Seedance video models need explicit roles on image contents when a
/// request carries more than one image. Other vendors read the plain `images`
/// array, so the role mapping is limited to this model family.
fn is_doubao_video_model(model: &str) -> bool {
    let model = model.to_ascii_lowercase();
    model.contains("doubao") || model.contains("seedance")
}

fn image_content(url: &str, role: Option<&str>) -> Value {
    let mut item = json!({
        "type": "image_url",
        "image_url": { "url": url }
    });
    if let Some(role) = role {
        item["role"] = json!(role);
    }
    item
}

fn image_content_array(images: &[&str], role: &str) -> Value {
    Value::Array(
        images
            .iter()
            .map(|url| image_content(url, Some(role)))
            .collect(),
    )
}

fn build_sora_create_body(params: &CreateParams) -> Value {
    let mut body = params.extra.clone();
    body.remove("model");
    body.remove("prompt");
    body.insert("model".to_string(), json!(params.model));
    body.insert("prompt".to_string(), json!(params.prompt));
    if let Some(duration) = params.duration {
        // OpenAI's VideoSeconds type is a string enum ("4", "8", "12").
        body.insert("seconds".to_string(), json!(duration.to_string()));
    }
    if let Some(resolution) = params.resolution {
        let info = parse_resolution(resolution);
        let size = match (info.width, info.height) {
            (Some(width), Some(height)) => format!("{width}x{height}"),
            _ => resolution.to_string(),
        };
        body.insert("size".to_string(), json!(size));
    }
    if !params.image_urls.is_empty() {
        // New API's /v1/videos is a generic host protocol now that Sora is
        // gone: vendor plugins read `image` for one frame and `images` for
        // multiple frames.
        body.remove("input_reference");
        body.remove("image");
        body.remove("images");
        let doubao = is_doubao_video_model(params.model);
        if doubao && params.image_urls.len() > 1 {
            // Doubao's openai_video bridge drops roles when it turns `images`
            // into Ark content, so send the role-bearing content directly.
            let mut metadata = body
                .get("metadata")
                .and_then(Value::as_object)
                .cloned()
                .unwrap_or_default();
            metadata.remove("content");
            metadata.insert(
                "content".to_string(),
                image_content_array(&params.image_urls, "reference_image"),
            );
            body.insert("metadata".to_string(), Value::Object(metadata));
        } else if doubao {
            // The Doubao bridge reads images (not image) for this host.
            body.insert("images".to_string(), json!(params.image_urls));
        } else if let [image_url] = params.image_urls.as_slice() {
            body.insert("image".to_string(), json!(image_url));
        } else {
            body.insert("images".to_string(), json!(params.image_urls));
        }
    }
    Value::Object(body)
}

fn build_new_api_generation_body(params: &CreateParams) -> Value {
    let mut body = params.extra.clone();
    body.remove("model");
    body.remove("prompt");
    body.insert("model".to_string(), json!(params.model));
    body.insert("prompt".to_string(), json!(params.prompt));

    let mut metadata = body
        .get("metadata")
        .and_then(Value::as_object)
        .cloned()
        .unwrap_or_default();

    if !params.image_urls.is_empty() {
        body.remove("image");
        body.remove("images");
        if is_doubao_video_model(params.model) && params.image_urls.len() > 1 {
            // The native New API path reaches the same Doubao bridge; Ark
            // requires a role on every image when there are several.
            metadata.remove("content");
            metadata.insert(
                "content".to_string(),
                image_content_array(&params.image_urls, "reference_image"),
            );
        } else if let [image_url] = params.image_urls.as_slice() {
            body.insert("image".to_string(), json!(image_url));
        } else {
            body.insert("images".to_string(), json!(params.image_urls));
        }
    }

    if let Some(duration) = params.duration {
        body.insert("duration".to_string(), json!(duration));
        // TaskSubmitReq and the plugin hooks read `seconds`; the native
        // documentation calls the same value `duration`.
        body.insert("seconds".to_string(), json!(duration.to_string()));
        metadata.insert("duration".to_string(), json!(duration));
    }
    if let Some(resolution) = params.resolution {
        let info = parse_resolution(resolution);
        if let (Some(width), Some(height)) = (info.width, info.height) {
            body.insert("width".to_string(), json!(width));
            body.insert("height".to_string(), json!(height));
            body.insert("size".to_string(), json!(format!("{width}x{height}")));
        }
        metadata.insert(
            "resolution".to_string(),
            json!(info.tier.unwrap_or(resolution)),
        );
    }
    if !metadata.is_empty() {
        body.insert("metadata".to_string(), Value::Object(metadata));
    }
    Value::Object(body)
}

fn build_doubao_create_body(params: &CreateParams) -> Value {
    let mut body = params.extra.clone();
    body.remove("model");
    body.remove("content");

    let mut content = Vec::new();
    let role = (params.image_urls.len() > 1).then_some("reference_image");
    for image_url in &params.image_urls {
        content.push(image_content(image_url, role));
    }
    content.push(json!({ "type": "text", "text": params.prompt }));
    body.insert("content".to_string(), Value::Array(content));
    body.insert("model".to_string(), json!(params.model));
    if let Some(duration) = params.duration {
        body.insert("duration".to_string(), json!(duration));
    }
    if let Some(resolution) = params.resolution {
        let info = parse_resolution(resolution);
        body.insert(
            "resolution".to_string(),
            json!(info.tier.unwrap_or(resolution)),
        );
    }
    Value::Object(body)
}

// ============================================================================
// Response parsing
// ============================================================================

fn task_object(body: &Value) -> Result<&Map<String, Value>, String> {
    let object = body
        .as_object()
        .ok_or_else(|| format!("Unexpected response: {}", snippet(&body.to_string())))?;
    if object.contains_key("id") || object.contains_key("task_id") || object.contains_key("status")
    {
        return Ok(object);
    }
    if let Some(data) = object.get("data").and_then(Value::as_object) {
        return Ok(data);
    }
    Ok(object)
}

fn string_field(object: &Map<String, Value>, names: &[&str]) -> Option<String> {
    names.iter().find_map(|name| {
        object
            .get(*name)
            .and_then(Value::as_str)
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(str::to_string)
    })
}

fn value_to_string(value: &Value) -> Option<String> {
    match value {
        Value::String(text) => {
            let text = text.trim();
            (!text.is_empty()).then(|| text.to_string())
        }
        Value::Number(number) => Some(number.to_string()),
        _ => None,
    }
}

fn value_message(value: &Value) -> Option<String> {
    match value {
        Value::String(text) => {
            let text = text.trim();
            (!text.is_empty()).then(|| text.to_string())
        }
        Value::Object(object) => {
            let message = object
                .get("message")
                .and_then(Value::as_str)
                .map(str::trim)
                .filter(|text| !text.is_empty());
            let code = object
                .get("code")
                .and_then(value_to_string)
                .filter(|code| code != "0");
            match (message, code) {
                (Some(message), Some(code)) => Some(format!("{message} ({code})")),
                (Some(message), None) => Some(message.to_string()),
                (None, Some(code)) => Some(code),
                (None, None) => None,
            }
        }
        _ => None,
    }
}

fn extract_error(object: &Map<String, Value>) -> Option<String> {
    for key in ["error", "fail_reason", "error_message"] {
        if let Some(message) = object.get(key).and_then(value_message) {
            return Some(message);
        }
    }
    None
}

fn find_video_url(object: &Map<String, Value>) -> Option<String> {
    // New API's native task fetch returns a TaskDto under `data`; its result is
    // `result_url`, not the OpenAI `url` or the doubao `content.video_url`.
    for key in ["video_url", "url", "result_url"] {
        if let Some(url) = object
            .get(key)
            .and_then(Value::as_str)
            .map(str::trim)
            .filter(|url| !url.is_empty())
        {
            return Some(url.to_string());
        }
    }
    object
        .get("content")
        .and_then(Value::as_object)
        .and_then(|content| content.get("video_url"))
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|url| !url.is_empty())
        .map(str::to_string)
}

fn parse_progress(value: Option<&Value>) -> u32 {
    let number = match value {
        Some(Value::Number(number)) => number.as_f64(),
        Some(Value::String(text)) => text.trim().trim_end_matches('%').parse::<f64>().ok(),
        _ => None,
    };
    number.unwrap_or(0.0).clamp(0.0, 100.0) as u32
}

fn map_status(raw: &str) -> Option<VideoTaskStatus> {
    match raw.trim().to_ascii_lowercase().as_str() {
        "queued" | "pending" | "not_start" | "not_started" | "submitted" | "uploading" => {
            Some(VideoTaskStatus::Queued)
        }
        "in_progress" | "in-progress" | "running" | "processing" | "generating" => {
            Some(VideoTaskStatus::InProgress)
        }
        "completed" | "succeeded" | "success" | "done" => Some(VideoTaskStatus::Completed),
        "failed" | "failure" | "expired" | "error" => Some(VideoTaskStatus::Failed),
        "cancelled" | "canceled" => Some(VideoTaskStatus::Cancelled),
        _ => None,
    }
}

fn progress_for(status: VideoTaskStatus, raw: u32) -> u32 {
    match status {
        VideoTaskStatus::Queued => 0,
        VideoTaskStatus::InProgress => raw,
        VideoTaskStatus::Completed | VideoTaskStatus::Failed | VideoTaskStatus::Cancelled => 100,
    }
}

fn parse_completed_at(object: &Map<String, Value>) -> Option<i64> {
    object
        .get("completed_at")
        .or_else(|| object.get("updated_at"))
        .and_then(Value::as_i64)
        .filter(|value| *value > 0)
        .map(|seconds| seconds.saturating_mul(1000))
}

fn parse_sora_snapshot(
    body: &Value,
    expected_id: Option<&str>,
    require_status: bool,
) -> Result<VideoTaskSnapshot, String> {
    let object = task_object(body)?;
    let id = expected_id
        .map(str::to_string)
        .or_else(|| string_field(object, &["id", "task_id"]))
        .ok_or_else(|| {
            format!(
                "The Sora video response did not include a task id: {}",
                snippet(&body.to_string())
            )
        })?;
    validate_task_id(&id)?;

    let error = extract_error(object);
    let status = match string_field(object, &["status"]) {
        Some(raw) => {
            map_status(&raw).ok_or_else(|| format!("Unrecognized video task status '{raw}'."))?
        }
        None if require_status => {
            return Err(format!(
                "The Sora video response did not include a status: {}",
                snippet(&body.to_string())
            ))
        }
        None if error.is_some() => VideoTaskStatus::Failed,
        None => VideoTaskStatus::Queued,
    };
    let progress = progress_for(status, parse_progress(object.get("progress")));
    let error = error.or_else(|| {
        (status == VideoTaskStatus::Failed)
            .then(|| "The video task failed without an error message.".to_string())
    });

    Ok(VideoTaskSnapshot {
        id,
        status,
        progress,
        video_url: find_video_url(object),
        local_path: None,
        error,
        completed_at: parse_completed_at(object),
    })
}

fn parse_new_api_generation_snapshot(
    body: &Value,
    expected_id: Option<&str>,
    require_status: bool,
) -> Result<VideoTaskSnapshot, String> {
    let object = task_object(body)?;
    let id = expected_id
        .map(str::to_string)
        .or_else(|| string_field(object, &["task_id", "id"]))
        .ok_or_else(|| {
            format!(
                "The New API video response did not include a task id: {}",
                snippet(&body.to_string())
            )
        })?;
    validate_task_id(&id)?;

    let error = extract_error(object);
    let status = match string_field(object, &["status"]) {
        Some(raw) => {
            map_status(&raw).ok_or_else(|| format!("Unrecognized video task status '{raw}'."))?
        }
        None if require_status => {
            return Err(format!(
                "The New API video response did not include a status: {}",
                snippet(&body.to_string())
            ))
        }
        None if error.is_some() => VideoTaskStatus::Failed,
        None => VideoTaskStatus::Queued,
    };
    let raw_progress = parse_progress(object.get("progress"));
    let progress = if status == VideoTaskStatus::InProgress && raw_progress == 0 {
        50
    } else {
        progress_for(status, raw_progress)
    };
    let video_url = find_video_url(object);
    let error = error.or_else(|| {
        if status == VideoTaskStatus::Failed {
            Some("The video task failed without an error message.".to_string())
        } else if status == VideoTaskStatus::Completed && video_url.is_none() {
            Some("The video task completed but no video URL was returned.".to_string())
        } else {
            None
        }
    });

    Ok(VideoTaskSnapshot {
        id,
        status,
        progress,
        video_url,
        local_path: None,
        error,
        completed_at: parse_completed_at(object),
    })
}

fn parse_doubao_snapshot(
    body: &Value,
    expected_id: Option<&str>,
    require_status: bool,
) -> Result<VideoTaskSnapshot, String> {
    let object = task_object(body)?;
    let id = expected_id
        .map(str::to_string)
        .or_else(|| string_field(object, &["id", "task_id"]))
        .ok_or_else(|| {
            format!(
                "The Doubao task response did not include a task id: {}",
                snippet(&body.to_string())
            )
        })?;
    validate_task_id(&id)?;

    let error = extract_error(object);
    let status = match string_field(object, &["status"]) {
        Some(raw) => {
            map_status(&raw).ok_or_else(|| format!("Unrecognized video task status '{raw}'."))?
        }
        None if require_status => {
            return Err(format!(
                "The Doubao task response did not include a status: {}",
                snippet(&body.to_string())
            ))
        }
        None if error.is_some() => VideoTaskStatus::Failed,
        None => VideoTaskStatus::Queued,
    };
    let progress = progress_for(status, parse_progress(object.get("progress")));
    let video_url = find_video_url(object);
    let error = error.or_else(|| {
        if status == VideoTaskStatus::Failed {
            Some("The video task failed without an error message.".to_string())
        } else if status == VideoTaskStatus::Completed && video_url.is_none() {
            Some("The video task completed but no video URL was returned.".to_string())
        } else {
            None
        }
    });

    Ok(VideoTaskSnapshot {
        id,
        status,
        progress,
        video_url,
        local_path: None,
        error,
        completed_at: parse_completed_at(object),
    })
}

// ============================================================================
// Public protocol operations
// ============================================================================

/// Submits a video task and returns the upstream task in normalized form.
pub async fn create_video_task(
    model: &VideoModel,
    prompt: &str,
    duration: Option<u32>,
    resolution: Option<&str>,
    image_urls: Vec<String>,
    extra_json: &str,
) -> Result<VideoTaskSnapshot, String> {
    let prompt = prompt.trim();
    if prompt.is_empty() {
        return Err("Prompt is required.".to_string());
    }
    if let Some(seconds) = duration {
        if seconds == 0 {
            return Err("Duration must be greater than zero.".to_string());
        }
        if seconds > 3600 {
            return Err("Duration must be at most 3600 seconds.".to_string());
        }
    }

    let image_urls: Vec<&str> = image_urls
        .iter()
        .map(|url| url.trim())
        .filter(|url| !url.is_empty())
        .collect();
    let extra = parse_extra(extra_json)?;
    let params = CreateParams {
        model: &model.model_id,
        prompt,
        duration,
        resolution: resolution.map(str::trim).filter(|value| !value.is_empty()),
        image_urls,
        extra,
    };
    let source = resolve_source(model).await?;
    let client = playground::build_client(CREATE_TIMEOUT)?;

    match model.protocol {
        VideoProtocol::Sora => {
            let url = sora_videos_base(&source.base_url);
            let body = build_sora_create_body(&params);
            let request =
                authorized(client.post(url).json(&body), &source.api_key).timeout(CREATE_TIMEOUT);
            let value = send_json(request).await?;
            parse_sora_snapshot(&value, None, false)
        }
        VideoProtocol::NewApi => {
            let url = new_api_generations_base(&source.base_url);
            let body = build_new_api_generation_body(&params);
            let request =
                authorized(client.post(url).json(&body), &source.api_key).timeout(CREATE_TIMEOUT);
            let value = send_json(request).await?;
            parse_new_api_generation_snapshot(&value, None, false)
        }
        VideoProtocol::Doubao => {
            let url = doubao_tasks_base(&source.base_url, source.channel_type.as_ref());
            let body = build_doubao_create_body(&params);
            let request =
                authorized(client.post(url).json(&body), &source.api_key).timeout(CREATE_TIMEOUT);
            let value = send_json(request).await?;
            parse_doubao_snapshot(&value, None, false)
        }
    }
}

/// Queries one video task. Sora content is downloaded to the playground media
/// directory because its content endpoint requires the channel's API key. The
/// native New API wire returns a direct `url`.
pub async fn query_video_task(
    model: &VideoModel,
    task_id: &str,
) -> Result<VideoTaskSnapshot, String> {
    validate_task_id(task_id)?;
    let source = resolve_source(model).await?;
    let client = playground::build_client(CREATE_TIMEOUT)?;

    match model.protocol {
        VideoProtocol::Sora => {
            let base = sora_videos_base(&source.base_url);
            let url = format!("{base}/{task_id}");
            let request = authorized(client.get(url), &source.api_key).timeout(QUERY_TIMEOUT);
            let value = send_json(request).await?;
            let mut snapshot = parse_sora_snapshot(&value, Some(task_id), true)?;
            if snapshot.status == VideoTaskStatus::Completed {
                match download_new_api_content(&client, &base, task_id, &source.api_key).await {
                    Ok(path) => snapshot.local_path = Some(path),
                    Err(error) => {
                        log::warn!("Video: failed to download content for task {task_id}: {error}");
                        if snapshot.video_url.is_none() {
                            snapshot.error = Some(format!(
                                "The video finished, but its content could not be downloaded: {error}"
                            ));
                        }
                    }
                }
            }
            Ok(snapshot)
        }
        VideoProtocol::NewApi => {
            let base = new_api_generations_base(&source.base_url);
            let url = format!("{base}/{task_id}");
            let request = authorized(client.get(url), &source.api_key).timeout(QUERY_TIMEOUT);
            let value = send_json(request).await?;
            parse_new_api_generation_snapshot(&value, Some(task_id), true)
        }
        VideoProtocol::Doubao => {
            let base = doubao_tasks_base(&source.base_url, source.channel_type.as_ref());
            let url = format!("{base}/{task_id}");
            let request = authorized(client.get(url), &source.api_key).timeout(QUERY_TIMEOUT);
            let value = send_json(request).await?;
            parse_doubao_snapshot(&value, Some(task_id), true)
        }
    }
}

async fn download_new_api_content(
    client: &reqwest::Client,
    videos_base: &str,
    task_id: &str,
    api_key: &str,
) -> Result<String, String> {
    use tokio::io::AsyncWriteExt;

    let home = crate::paths::get_home_dir()?;
    let media_dir = video_media_dir_for_home(&home);
    for extension in ["mp4", "webm", "mov"] {
        let path = media_dir.join(format!("{task_id}.{extension}"));
        if let Ok(metadata) = fs::metadata(&path) {
            if metadata.len() > 0 {
                return Ok(path.to_string_lossy().to_string());
            }
        }
    }

    fs::create_dir_all(&media_dir)
        .map_err(|error| format!("Failed to create video media directory: {error}"))?;
    let url = format!("{videos_base}/{task_id}/content");
    let response = authorized(client.get(url), api_key)
        .timeout(CONTENT_TIMEOUT)
        .send()
        .await
        .map_err(|error| format!("Video content request failed: {error}"))?;
    let status = response.status();
    let content_type = response
        .headers()
        .get(reqwest::header::CONTENT_TYPE)
        .and_then(|value| value.to_str().ok())
        .unwrap_or_default()
        .to_ascii_lowercase();
    if !status.is_success() {
        let body = response.text().await.unwrap_or_default();
        return Err(format_api_error(status, &body));
    }
    if content_type.starts_with("application/json") || content_type.starts_with("text/") {
        let body = response.text().await.unwrap_or_default();
        return Err(format!(
            "The content endpoint returned {content_type} instead of a video: {}",
            snippet(&body)
        ));
    }

    let extension = if content_type.contains("webm") {
        "webm"
    } else if content_type.contains("quicktime") {
        "mov"
    } else {
        "mp4"
    };
    let final_path = media_dir.join(format!("{task_id}.{extension}"));
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_nanos())
        .unwrap_or_default();
    let temp_path = media_dir.join(format!(".{task_id}.{nanos}.part"));

    let mut file = tokio::fs::File::create(&temp_path)
        .await
        .map_err(|error| format!("Failed to create video file: {error}"))?;
    let mut response = response;
    let mut bytes_written: u64 = 0;
    loop {
        match response.chunk().await {
            Ok(Some(chunk)) => {
                if let Err(error) = file.write_all(&chunk).await {
                    drop(file);
                    let _ = tokio::fs::remove_file(&temp_path).await;
                    return Err(format!("Failed to write video file: {error}"));
                }
                bytes_written += chunk.len() as u64;
            }
            Ok(None) => break,
            Err(error) => {
                drop(file);
                let _ = tokio::fs::remove_file(&temp_path).await;
                return Err(format!("Failed while downloading video content: {error}"));
            }
        }
    }
    if let Err(error) = file.flush().await {
        drop(file);
        let _ = tokio::fs::remove_file(&temp_path).await;
        return Err(format!("Failed to flush video file: {error}"));
    }
    drop(file);

    if bytes_written == 0 {
        let _ = tokio::fs::remove_file(&temp_path).await;
        return Err("The content endpoint returned an empty body.".to_string());
    }
    if let Err(error) = tokio::fs::rename(&temp_path, &final_path).await {
        let _ = tokio::fs::remove_file(&temp_path).await;
        return Err(format!("Failed to move video file into place: {error}"));
    }
    Ok(final_path.to_string_lossy().to_string())
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_model(source: VideoSource, protocol: VideoProtocol) -> VideoModel {
        VideoModel {
            id: "vm_test".to_string(),
            name: "Seedance".to_string(),
            model_id: "doubao-seedance-2-0-mini-260615".to_string(),
            source,
            protocol,
            durations: vec![5, 10],
            resolutions: vec!["720p".to_string()],
            supports_image_input: true,
        }
    }

    fn sample_params<'a>(extra: Map<String, Value>) -> CreateParams<'a> {
        CreateParams {
            model: "doubao-seedance-2-0-mini-260615",
            prompt: "a cat",
            duration: Some(5),
            resolution: Some("720p"),
            image_urls: vec!["https://example.com/cat.png"],
            extra,
        }
    }

    #[test]
    fn sora_videos_base_normalizes_version_suffixes() {
        assert_eq!(
            sora_videos_base("https://gateway.example"),
            "https://gateway.example/v1/videos"
        );
        assert_eq!(
            sora_videos_base("https://gateway.example/"),
            "https://gateway.example/v1/videos"
        );
        assert_eq!(
            sora_videos_base("https://gateway.example/v1"),
            "https://gateway.example/v1/videos"
        );
        assert_eq!(
            sora_videos_base("https://gateway.example/v1/videos"),
            "https://gateway.example/v1/videos"
        );
    }

    #[test]
    fn new_api_generations_base_normalizes_version_suffixes() {
        assert_eq!(
            new_api_generations_base("https://gateway.example"),
            "https://gateway.example/v1/video/generations"
        );
        assert_eq!(
            new_api_generations_base("https://gateway.example/v1"),
            "https://gateway.example/v1/video/generations"
        );
        assert_eq!(
            new_api_generations_base("https://gateway.example/v1/video/generations"),
            "https://gateway.example/v1/video/generations"
        );
    }

    #[test]
    fn doubao_tasks_base_uses_the_new_api_prefix_only_for_new_api_channels() {
        assert_eq!(
            doubao_tasks_base("https://gateway.example", Some(&ChannelType::NewApi)),
            "https://gateway.example/doubao/api/v3/contents/generations/tasks"
        );
        assert_eq!(
            doubao_tasks_base("https://gateway.example/doubao", Some(&ChannelType::NewApi)),
            "https://gateway.example/doubao/api/v3/contents/generations/tasks"
        );
        assert_eq!(
            doubao_tasks_base("https://ark.example", Some(&ChannelType::General)),
            "https://ark.example/api/v3/contents/generations/tasks"
        );
        assert_eq!(
            doubao_tasks_base(
                "https://ark.example/api/v3/contents/generations/tasks",
                None
            ),
            "https://ark.example/api/v3/contents/generations/tasks"
        );
    }

    #[test]
    fn sora_body_maps_single_doubao_image_as_images() {
        let mut extra = Map::new();
        extra.insert("seed".to_string(), json!(42));
        extra.insert("seconds".to_string(), json!(99));
        extra.insert("prompt".to_string(), json!("ignored"));
        let body = build_sora_create_body(&sample_params(extra));
        assert_eq!(body["model"], "doubao-seedance-2-0-mini-260615");
        assert_eq!(body["prompt"], "a cat");
        assert_eq!(body["seconds"], "5");
        assert_eq!(body["size"], "1280x720");
        assert_eq!(body["images"], json!(["https://example.com/cat.png"]));
        assert!(body.get("image").is_none());
        assert!(body.get("input_reference").is_none());
        assert_eq!(body["seed"], 42);
    }

    #[test]
    fn sora_body_maps_single_non_doubao_image() {
        let mut params = sample_params(Map::new());
        params.model = "sora-2";
        let body = build_sora_create_body(&params);
        assert_eq!(body["image"], "https://example.com/cat.png");
        assert!(body.get("images").is_none());
        assert!(body.get("input_reference").is_none());
    }

    #[test]
    fn sora_body_maps_doubao_multiple_images_with_roles() {
        let mut params = sample_params(Map::new());
        params.image_urls = vec![
            "https://example.com/frame-a.png",
            "https://example.com/frame-b.png",
        ];
        let body = build_sora_create_body(&params);
        assert!(body.get("image").is_none());
        assert!(body.get("images").is_none());
        assert!(body.get("input_reference").is_none());
        let content = body["metadata"]["content"].as_array().expect("content");
        assert_eq!(content.len(), 2);
        assert_eq!(content[0]["role"], "reference_image");
        assert_eq!(content[1]["role"], "reference_image");
        assert_eq!(
            content[0]["image_url"]["url"],
            "https://example.com/frame-a.png"
        );
    }

    #[test]
    fn sora_body_maps_non_doubao_multiple_images() {
        let mut params = sample_params(Map::new());
        params.model = "sora-2";
        params.image_urls = vec![
            "https://example.com/frame-a.png",
            "https://example.com/frame-b.png",
        ];
        let body = build_sora_create_body(&params);
        assert!(body.get("image").is_none());
        assert_eq!(
            body["images"],
            json!([
                "https://example.com/frame-a.png",
                "https://example.com/frame-b.png"
            ])
        );
        assert!(body.get("metadata").is_none() || body["metadata"].get("content").is_none());
    }

    #[test]
    fn new_api_body_maps_single_image_and_resolution() {
        let mut extra = Map::new();
        extra.insert("seed".to_string(), json!(42));
        extra.insert("seconds".to_string(), json!(99));
        let body = build_new_api_generation_body(&sample_params(extra));
        assert_eq!(body["model"], "doubao-seedance-2-0-mini-260615");
        assert_eq!(body["prompt"], "a cat");
        assert_eq!(body["image"], "https://example.com/cat.png");
        assert!(body.get("images").is_none());
        assert_eq!(body["duration"], 5);
        assert_eq!(body["seconds"], "5");
        assert_eq!(body["width"], 1280);
        assert_eq!(body["height"], 720);
        assert_eq!(body["size"], "1280x720");
        assert_eq!(body["metadata"]["resolution"], "720p");
        assert_eq!(body["metadata"]["duration"], 5);
        assert_eq!(body["seed"], 42);
    }

    #[test]
    fn new_api_body_maps_doubao_multiple_images_with_roles() {
        let mut params = sample_params(Map::new());
        params.image_urls = vec![
            "https://example.com/frame-a.png",
            "https://example.com/frame-b.png",
        ];
        let body = build_new_api_generation_body(&params);
        assert!(body.get("image").is_none());
        assert!(body.get("images").is_none());
        let content = body["metadata"]["content"].as_array().expect("content");
        assert_eq!(content.len(), 2);
        assert_eq!(content[0]["role"], "reference_image");
        assert_eq!(content[1]["role"], "reference_image");
        assert_eq!(body["metadata"]["duration"], 5);
        assert_eq!(body["metadata"]["resolution"], "720p");
    }

    #[test]
    fn new_api_body_maps_non_doubao_multiple_images() {
        let mut params = sample_params(Map::new());
        params.model = "kling-v3";
        params.image_urls = vec![
            "https://example.com/frame-a.png",
            "https://example.com/frame-b.png",
        ];
        let body = build_new_api_generation_body(&params);
        assert!(body.get("image").is_none());
        assert_eq!(
            body["images"],
            json!([
                "https://example.com/frame-a.png",
                "https://example.com/frame-b.png"
            ])
        );
        assert!(body.get("metadata").is_none() || body["metadata"].get("content").is_none());
    }

    #[test]
    fn doubao_body_puts_all_images_before_text() {
        let mut extra = Map::new();
        extra.insert("seed".to_string(), json!(7));
        extra.insert(
            "content".to_string(),
            json!([{ "type": "text", "text": "ignored" }]),
        );
        let mut params = sample_params(extra);
        params.image_urls = vec![
            "https://example.com/frame-a.png",
            "https://example.com/frame-b.png",
        ];
        let body = build_doubao_create_body(&params);
        assert_eq!(body["model"], "doubao-seedance-2-0-mini-260615");
        assert_eq!(body["duration"], 5);
        assert_eq!(body["resolution"], "720p");
        assert_eq!(body["seed"], 7);
        let content = body["content"].as_array().expect("content array");
        assert_eq!(content.len(), 3);
        assert_eq!(content[0]["type"], "image_url");
        assert_eq!(
            content[0]["image_url"]["url"],
            "https://example.com/frame-a.png"
        );
        assert_eq!(content[0]["role"], "reference_image");
        assert_eq!(content[1]["type"], "image_url");
        assert_eq!(
            content[1]["image_url"]["url"],
            "https://example.com/frame-b.png"
        );
        assert_eq!(content[1]["role"], "reference_image");
        assert_eq!(content[2]["type"], "text");
        assert_eq!(content[2]["text"], "a cat");
    }

    #[test]
    fn resolution_parser_handles_tiers_and_sizes() {
        let tier = parse_resolution("720p");
        assert_eq!(
            (tier.width, tier.height, tier.tier),
            (Some(1280), Some(720), Some("720p"))
        );
        let size = parse_resolution("1920x1080");
        assert_eq!(
            (size.width, size.height, size.tier),
            (Some(1920), Some(1080), Some("1080p"))
        );
        let uppercase = parse_resolution("1280X720");
        assert_eq!((uppercase.width, uppercase.height), (Some(1280), Some(720)));
        let custom = parse_resolution("1024x1792");
        assert_eq!(
            (custom.width, custom.height, custom.tier),
            (Some(1024), Some(1792), Some("720p"))
        );
        let unknown = parse_resolution("portrait");
        assert_eq!((unknown.width, unknown.tier), (None, None));
    }

    #[test]
    fn parses_sora_task_objects() {
        let body = json!({
            "id": "task_123",
            "object": "video",
            "status": "in_progress",
            "progress": 40,
            "created_at": 1_700_000_000
        });
        let snapshot = parse_sora_snapshot(&body, None, true).expect("snapshot");
        assert_eq!(snapshot.id, "task_123");
        assert_eq!(snapshot.status, VideoTaskStatus::InProgress);
        assert_eq!(snapshot.progress, 40);
        assert!(snapshot.error.is_none());
    }

    #[test]
    fn parses_sora_errors_and_completion_timestamps() {
        let body = json!({
            "id": "task_123",
            "status": "failed",
            "progress": "90%",
            "error": { "code": "content_policy", "message": "prompt rejected" },
            "completed_at": 1_700_000_100
        });
        let snapshot = parse_sora_snapshot(&body, Some("task_123"), true).expect("snapshot");
        assert_eq!(snapshot.status, VideoTaskStatus::Failed);
        assert_eq!(snapshot.progress, 100);
        assert_eq!(
            snapshot.error.as_deref(),
            Some("prompt rejected (content_policy)")
        );
        assert_eq!(snapshot.completed_at, Some(1_700_000_100_000));
    }

    #[test]
    fn parses_new_api_generation_responses() {
        let created = json!({ "task_id": "task_native", "status": "queued" });
        let snapshot =
            parse_new_api_generation_snapshot(&created, None, false).expect("create snapshot");
        assert_eq!(snapshot.id, "task_native");
        assert_eq!(snapshot.status, VideoTaskStatus::Queued);
        assert_eq!(snapshot.progress, 0);

        let completed = json!({
            "task_id": "task_native",
            "status": "completed",
            "url": "https://cdn.example/native.mp4",
            "format": "mp4",
            "metadata": { "duration": 5, "width": 1280, "height": 720, "seed": 1 },
            "error": { "code": 0, "message": "" }
        });
        let snapshot = parse_new_api_generation_snapshot(&completed, Some("task_native"), true)
            .expect("query snapshot");
        assert_eq!(snapshot.status, VideoTaskStatus::Completed);
        assert_eq!(snapshot.progress, 100);
        assert_eq!(
            snapshot.video_url.as_deref(),
            Some("https://cdn.example/native.mp4")
        );
        assert!(snapshot.error.is_none());

        let wrapped = json!({
            "code": "success",
            "data": {
                "task_id": "task_native",
                "status": "SUCCESS",
                "result_url": "https://cdn.example/wrapped.mp4",
                "progress": "100%"
            }
        });
        let snapshot = parse_new_api_generation_snapshot(&wrapped, Some("task_native"), true)
            .expect("wrapped snapshot");
        assert_eq!(snapshot.status, VideoTaskStatus::Completed);
        assert_eq!(
            snapshot.video_url.as_deref(),
            Some("https://cdn.example/wrapped.mp4")
        );
        assert!(snapshot.error.is_none());

        let running = json!({ "task_id": "task_native", "status": "in_progress" });
        let snapshot = parse_new_api_generation_snapshot(&running, Some("task_native"), true)
            .expect("running snapshot");
        assert_eq!(snapshot.status, VideoTaskStatus::InProgress);
        assert_eq!(snapshot.progress, 50);

        let failed = json!({
            "task_id": "task_native",
            "status": "failed",
            "error": { "code": 100, "message": "bad prompt" }
        });
        let snapshot = parse_new_api_generation_snapshot(&failed, Some("task_native"), true)
            .expect("failed snapshot");
        assert_eq!(snapshot.status, VideoTaskStatus::Failed);
        assert_eq!(snapshot.error.as_deref(), Some("bad prompt (100)"));
    }

    #[test]
    fn parses_doubao_task_objects() {
        let body = json!({
            "id": "cgt-123",
            "status": "succeeded",
            "content": { "video_url": "https://cdn.example/video.mp4" },
            "usage": { "completion_tokens": 1000 }
        });
        let snapshot = parse_doubao_snapshot(&body, Some("cgt-123"), true).expect("snapshot");
        assert_eq!(snapshot.id, "cgt-123");
        assert_eq!(snapshot.status, VideoTaskStatus::Completed);
        assert_eq!(snapshot.progress, 100);
        assert_eq!(
            snapshot.video_url.as_deref(),
            Some("https://cdn.example/video.mp4")
        );
        assert!(snapshot.error.is_none());
    }

    #[test]
    fn completed_doubao_task_without_a_url_reports_an_error() {
        let body = json!({ "id": "cgt-123", "status": "succeeded" });
        let snapshot = parse_doubao_snapshot(&body, Some("cgt-123"), true).expect("snapshot");
        assert_eq!(snapshot.status, VideoTaskStatus::Completed);
        assert_eq!(
            snapshot.error.as_deref(),
            Some("The video task completed but no video URL was returned.")
        );
    }

    #[test]
    fn parses_doubao_failures() {
        let body = json!({
            "id": "cgt-123",
            "status": "failed",
            "error": { "code": "InvalidParameter", "message": "bad duration" }
        });
        let snapshot = parse_doubao_snapshot(&body, None, true).expect("snapshot");
        assert_eq!(snapshot.status, VideoTaskStatus::Failed);
        assert_eq!(
            snapshot.error.as_deref(),
            Some("bad duration (InvalidParameter)")
        );
    }

    #[test]
    fn status_aliases_cover_both_protocols() {
        assert_eq!(map_status("QUEUED"), Some(VideoTaskStatus::Queued));
        assert_eq!(map_status("running"), Some(VideoTaskStatus::InProgress));
        assert_eq!(map_status("succeeded"), Some(VideoTaskStatus::Completed));
        assert_eq!(map_status("expired"), Some(VideoTaskStatus::Failed));
        assert_eq!(map_status("canceled"), Some(VideoTaskStatus::Cancelled));
        assert_eq!(map_status("mystery"), None);
    }

    #[test]
    fn progress_accepts_numbers_and_percent_strings() {
        assert_eq!(parse_progress(Some(&json!(42))), 42);
        assert_eq!(parse_progress(Some(&json!("42%"))), 42);
        assert_eq!(parse_progress(Some(&json!(150))), 100);
        assert_eq!(parse_progress(None), 0);
    }

    #[test]
    fn task_ids_are_confined_to_safe_characters() {
        assert!(validate_task_id("task_abc-123.mp4").is_ok());
        assert!(validate_task_id("../etc/passwd").is_err());
        assert!(validate_task_id("").is_err());
    }

    #[test]
    fn persistence_round_trips_and_keeps_api_keys_out_of_the_model_file() {
        let dir = tempfile::TempDir::new().expect("temp home");
        let home = dir.path();
        let model = sample_model(
            VideoSource::Custom {
                base_url: "https://ark.example".to_string(),
                api_key: "sk-secret".to_string(),
            },
            VideoProtocol::Doubao,
        );

        save_video_models_for_home(home, std::slice::from_ref(&model)).expect("save");
        let raw = fs::read_to_string(video_models_path_for_home(home)).expect("models file");
        assert!(!raw.contains("sk-secret"));

        let loaded = load_video_models_for_home(home).expect("load");
        assert_eq!(loaded.len(), 1);
        match &loaded[0].source {
            VideoSource::Custom { api_key, .. } => assert_eq!(api_key, "sk-secret"),
            other => panic!("unexpected source: {other:?}"),
        }

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mode = fs::metadata(video_secrets_path_for_home(home))
                .expect("secrets file")
                .permissions()
                .mode();
            assert_eq!(mode & 0o777, 0o600);
        }
    }

    #[test]
    fn persistence_round_trips_channel_models() {
        let dir = tempfile::TempDir::new().expect("temp home");
        let home = dir.path();
        let model = sample_model(
            VideoSource::Channel {
                channel_id: "channel-1".to_string(),
                token_id: Some(7.0),
            },
            VideoProtocol::Sora,
        );

        save_video_models_for_home(home, std::slice::from_ref(&model)).expect("save");
        let loaded = load_video_models_for_home(home).expect("load");
        assert_eq!(loaded.len(), 1);
        assert_eq!(loaded[0].protocol, VideoProtocol::Sora);
        match &loaded[0].source {
            VideoSource::Channel {
                channel_id,
                token_id,
            } => {
                assert_eq!(channel_id, "channel-1");
                assert_eq!(*token_id, Some(7.0));
            }
            other => panic!("unexpected source: {other:?}"),
        }
    }

    #[test]
    fn loads_models_saved_without_the_preset_lists() {
        let dir = tempfile::TempDir::new().expect("temp home");
        let home = dir.path();
        let path = video_models_path_for_home(home);
        fs::create_dir_all(path.parent().expect("models dir")).expect("models dir");
        fs::write(
            &path,
            r#"[{"id":"vm_1","name":"Seedance","modelId":"doubao-seedance",
                "source":{"kind":"channel","channelId":"channel-1","tokenId":null},
                "protocol":"doubao","supportsImageInput":true}]"#,
        )
        .expect("models file");

        let loaded = load_video_models_for_home(home).expect("load");
        assert_eq!(loaded.len(), 1);
        assert!(loaded[0].durations.is_empty());
        assert!(loaded[0].resolutions.is_empty());
    }

    #[test]
    fn creates_a_sora_task_over_http() {
        use std::io::{Read, Write};
        use std::net::TcpListener;
        use std::sync::mpsc;

        fn request_end(buffer: &[u8]) -> Option<(usize, usize)> {
            let index = buffer.windows(4).position(|window| window == b"\r\n\r\n")?;
            let headers = String::from_utf8_lossy(&buffer[..index]).to_string();
            let content_length = headers.lines().find_map(|line| {
                let (name, value) = line.split_once(':')?;
                if name.eq_ignore_ascii_case("content-length") {
                    value.trim().parse::<usize>().ok()
                } else {
                    None
                }
            })?;
            Some((index + 4, content_length))
        }

        let listener = TcpListener::bind("127.0.0.1:0").expect("bind mock server");
        let port = listener.local_addr().expect("local addr").port();
        let (sender, receiver) = mpsc::channel();
        let server = std::thread::spawn(move || {
            let (mut stream, _) = listener.accept().expect("accept");
            let mut buffer = Vec::new();
            let mut chunk = [0u8; 2048];
            loop {
                let read = stream.read(&mut chunk).expect("read request");
                assert!(read > 0, "client closed before sending headers");
                buffer.extend_from_slice(&chunk[..read]);
                if let Some((body_start, content_length)) = request_end(&buffer) {
                    while buffer.len() < body_start + content_length {
                        let read = stream.read(&mut chunk).expect("read body");
                        assert!(read > 0, "client closed before sending body");
                        buffer.extend_from_slice(&chunk[..read]);
                    }
                    break;
                }
            }
            sender
                .send(String::from_utf8_lossy(&buffer).to_string())
                .expect("send captured request");

            let body = r#"{"id":"task_http","status":"queued","progress":0}"#;
            let response = format!(
                "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                body.len(),
                body
            );
            stream
                .write_all(response.as_bytes())
                .expect("write response");
        });

        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .expect("runtime");
        let mut model = sample_model(
            VideoSource::Custom {
                base_url: format!("http://127.0.0.1:{port}"),
                api_key: "sk-test".to_string(),
            },
            VideoProtocol::Sora,
        );
        model.model_id = "sora-2".to_string();
        let snapshot = runtime
            .block_on(create_video_task(
                &model,
                "a cat",
                Some(5),
                Some("1280x720"),
                vec!["https://example.com/cat.png".to_string()],
                "",
            ))
            .expect("create task");

        assert_eq!(snapshot.id, "task_http");
        assert_eq!(snapshot.status, VideoTaskStatus::Queued);
        let request = receiver
            .recv_timeout(Duration::from_secs(5))
            .expect("request");
        server.join().expect("server");
        assert!(request.starts_with("POST /v1/videos HTTP/1.1\r\n"));
        assert!(request
            .to_ascii_lowercase()
            .contains("authorization: bearer sk-test"));
        assert!(request.contains(r#""model":"sora-2""#));
        assert!(request.contains(r#""prompt":"a cat""#));
        assert!(request.contains(r#""seconds":"5""#));
        assert!(request.contains(r#""size":"1280x720""#));
        assert!(request.contains(r#""image":"https://example.com/cat.png""#));
        assert!(!request.contains("input_reference"));
    }

    #[test]
    fn protocol_alias_accepts_the_old_protocol_name() {
        let model = sample_model(
            VideoSource::Custom {
                base_url: "https://gateway.example".to_string(),
                api_key: String::new(),
            },
            VideoProtocol::Sora,
        );
        let mut value = serde_json::to_value(model).expect("serialize model");
        value["protocol"] = json!("new-api-videos");
        let loaded: VideoModel = serde_json::from_value(value).expect("deserialize model");
        assert_eq!(loaded.protocol, VideoProtocol::Sora);
    }

    fn sample_task(model: VideoModel, status: VideoTaskStatus) -> VideoTaskRecord {
        VideoTaskRecord {
            id: "task_1".to_string(),
            model_id: model.id.clone(),
            model_name: model.name.clone(),
            model,
            params: VideoTaskParams {
                prompt: "a cat".to_string(),
                duration: "5".to_string(),
                resolution: "720p".to_string(),
                image_url: String::new(),
                extra_json: String::new(),
            },
            status,
            progress: if status == VideoTaskStatus::Completed {
                100
            } else {
                0
            },
            created_at: 1_700_000_000_000,
            completed_at: Some(1_700_000_030_000),
            video_url: Some("https://cdn.example/video.mp4".to_string()),
            local_path: None,
            error: None,
        }
    }

    #[test]
    fn persistence_round_trips_tasks_without_leaking_api_keys() {
        let dir = tempfile::TempDir::new().expect("temp home");
        let home = dir.path();
        let model = sample_model(
            VideoSource::Custom {
                base_url: "https://ark.example".to_string(),
                api_key: "sk-task-secret".to_string(),
            },
            VideoProtocol::Doubao,
        );
        save_video_models_for_home(home, std::slice::from_ref(&model)).expect("save models");
        let task = sample_task(model, VideoTaskStatus::Completed);

        save_video_tasks_for_home(home, std::slice::from_ref(&task)).expect("save tasks");
        let raw = fs::read_to_string(video_tasks_path_for_home(home)).expect("tasks file");
        assert!(!raw.contains("sk-task-secret"));

        let loaded = load_video_tasks_for_home(home).expect("load tasks");
        assert_eq!(loaded.len(), 1);
        assert_eq!(loaded[0].id, "task_1");
        assert_eq!(loaded[0].status, VideoTaskStatus::Completed);
        assert_eq!(loaded[0].params.prompt, "a cat");
        assert_eq!(
            loaded[0].video_url.as_deref(),
            Some("https://cdn.example/video.mp4")
        );
        match &loaded[0].model.source {
            VideoSource::Custom { api_key, .. } => assert_eq!(api_key, "sk-task-secret"),
            other => panic!("unexpected source: {other:?}"),
        }
    }

    #[test]
    fn persistence_round_trips_channel_tasks() {
        let dir = tempfile::TempDir::new().expect("temp home");
        let home = dir.path();
        let model = sample_model(
            VideoSource::Channel {
                channel_id: "channel-1".to_string(),
                token_id: Some(9.0),
            },
            VideoProtocol::NewApi,
        );
        let mut task = sample_task(model, VideoTaskStatus::InProgress);
        task.completed_at = None;
        task.video_url = None;
        task.progress = 40;

        save_video_tasks_for_home(home, std::slice::from_ref(&task)).expect("save tasks");
        let loaded = load_video_tasks_for_home(home).expect("load tasks");
        assert_eq!(loaded.len(), 1);
        assert_eq!(loaded[0].status, VideoTaskStatus::InProgress);
        assert_eq!(loaded[0].progress, 40);
        assert!(loaded[0].completed_at.is_none());
        match &loaded[0].model.source {
            VideoSource::Channel {
                channel_id,
                token_id,
            } => {
                assert_eq!(channel_id, "channel-1");
                assert_eq!(*token_id, Some(9.0));
            }
            other => panic!("unexpected source: {other:?}"),
        }
    }

    #[test]
    fn extra_parameters_must_be_an_object() {
        assert!(parse_extra("[]").is_err());
        assert!(parse_extra("not json").is_err());
        assert!(parse_extra("{\"seed\": 1}").is_ok());
    }
}
