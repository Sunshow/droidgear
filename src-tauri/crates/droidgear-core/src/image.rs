//! Playground image models and protocol clients.
//!
//! Supports three upstream protocols:
//!
//! - `chat`: OpenAI ChatCompletions (`/v1/chat/completions`). Image models
//!   return their pictures as `message.images` entries (OpenRouter style) or
//!   as markdown data URLs inside `message.content`, which is what New API's
//!   Gemini channel produces.
//! - `responses`: OpenAI Responses (`/v1/responses`) with the server-side
//!   `image_generation` tool. Results arrive as `image_generation_call` output
//!   items carrying a base64 `result`.
//! - `google`: Gemini's native wire (`/v1beta/models/{model}:generateContent`)
//!   with `responseModalities: ["TEXT", "IMAGE"]`, or `:predict` for `imagen*`
//!   models.
//!
//! Decoded images are written to `~/.droidgear/playground/media/images/` so
//! task records stay small and a restart can still show the pictures. Models,
//! task history and custom-endpoint secrets live next to the video files under
//! `~/.droidgear/playground/`.

use base64::Engine as _;
use serde::{Deserialize, Serialize};
use serde_json::{json, Map, Value};
use specta::Type;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use crate::playground::{self, authorized, format_api_error, send_json, snippet, ResolvedSource};

const IMAGE_MODELS_FILE: &str = "image_models.json";
const IMAGE_TASKS_FILE: &str = "image_tasks.json";
const IMAGE_SECRETS_FILE: &str = "image_secrets.json";
const IMAGE_MEDIA_DIR: &str = "images";
const GENERATION_TIMEOUT: Duration = Duration::from_secs(300);
const DOWNLOAD_TIMEOUT: Duration = Duration::from_secs(60);
const REFERENCE_TIMEOUT: Duration = Duration::from_secs(30);
const MAX_IMAGE_BYTES: usize = 64 * 1024 * 1024;
const MAX_REFERENCE_BYTES: usize = 20 * 1024 * 1024;
const MAX_IMAGES_PER_TASK: usize = 8;

// ============================================================================
// Types
// ============================================================================

/// Image wire protocol used by a playground model.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, Type, PartialEq, Eq, Default)]
#[serde(rename_all = "kebab-case")]
pub enum ImageProtocol {
    /// OpenAI ChatCompletions (`/v1/chat/completions`).
    #[default]
    Chat,
    /// OpenAI Responses with the `image_generation` tool (`/v1/responses`).
    Responses,
    /// Gemini native `:generateContent` / `:predict`.
    Google,
}

/// Where an image model gets its endpoint and credentials.
#[derive(Debug, Clone, Serialize, Deserialize, Type, PartialEq)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum ImageSource {
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

/// An image model configured for the playground.
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct ImageModel {
    pub id: String,
    pub name: String,
    pub model_id: String,
    pub source: ImageSource,
    pub protocol: ImageProtocol,
    /// Suggested size values. The meaning follows the protocol: `WxH` for
    /// Chat/Responses, `WxH`, `A:B` or `1K`/`2K`/`4K` for Google.
    pub sizes: Vec<String>,
    /// Suggested quality values (Responses: low/medium/high/auto, Google:
    /// 1K/2K/4K).
    pub qualities: Vec<String>,
    /// Whether the model accepts reference images.
    pub supports_image_input: bool,
}

/// Lifecycle of one playground image task. Generation is a single request, so
/// a task only stays `running` while the request is in flight.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, Type, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ImageTaskStatus {
    Running,
    Completed,
    Failed,
}

/// One generated image, surfaced to the UI and stored with the task.
#[derive(Debug, Clone, Serialize, Deserialize, Type, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ImageAsset {
    /// Absolute path of the decoded file under the playground media directory.
    /// Absent when the upstream only returned a link that could not be saved.
    pub local_path: Option<String>,
    /// Remote URL, kept so the image can still be opened or re-downloaded.
    pub url: Option<String>,
    pub mime_type: String,
}

/// Form values captured when an image task was submitted.
#[derive(Debug, Clone, Serialize, Deserialize, Type, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ImageTaskParams {
    pub prompt: String,
    pub size: String,
    pub quality: String,
    pub count: u32,
    pub image_url: String,
    pub extra_json: String,
}

/// A playground image task persisted under
/// `~/.droidgear/playground/image_tasks.json`.
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct ImageTaskRecord {
    pub id: String,
    pub model_id: String,
    pub model_name: String,
    /// Snapshot of the model at submit time. Custom API keys are stripped on
    /// disk and merged back from `image_secrets.json` on load.
    pub model: ImageModel,
    pub params: ImageTaskParams,
    pub status: ImageTaskStatus,
    pub images: Vec<ImageAsset>,
    pub revised_prompt: Option<String>,
    /// Text the model returned together with the images (Gemini usually adds a
    /// short explanation).
    pub response_text: Option<String>,
    pub created_at: i64,
    pub completed_at: Option<i64>,
    pub error: Option<String>,
}

/// One protocol result, normalized for the playground UI.
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct ImageTaskSnapshot {
    pub images: Vec<ImageAsset>,
    pub revised_prompt: Option<String>,
    pub response_text: Option<String>,
    /// Milliseconds since the Unix epoch.
    pub completed_at: i64,
}

// ============================================================================
// Paths and persistence
// ============================================================================

fn playground_dir(home: &Path) -> PathBuf {
    playground::playground_dir(home)
}

pub fn image_models_path_for_home(home: &Path) -> PathBuf {
    playground_dir(home).join(IMAGE_MODELS_FILE)
}

pub fn image_tasks_path_for_home(home: &Path) -> PathBuf {
    playground_dir(home).join(IMAGE_TASKS_FILE)
}

pub fn image_secrets_path_for_home(home: &Path) -> PathBuf {
    playground_dir(home).join(IMAGE_SECRETS_FILE)
}

pub fn image_media_dir_for_home(home: &Path) -> PathBuf {
    playground::media_dir(home).join(IMAGE_MEDIA_DIR)
}

/// Loads playground image models, merging custom-endpoint API keys back in.
pub fn load_image_models() -> Result<Vec<ImageModel>, String> {
    load_image_models_for_home(&crate::paths::get_home_dir()?)
}

pub fn load_image_models_for_home(home: &Path) -> Result<Vec<ImageModel>, String> {
    let path = image_models_path_for_home(home);
    if !path.exists() {
        return Ok(Vec::new());
    }
    let content =
        fs::read_to_string(&path).map_err(|e| format!("Failed to read image models file: {e}"))?;
    if content.trim().is_empty() {
        return Ok(Vec::new());
    }
    let mut models: Vec<ImageModel> = serde_json::from_str(&content)
        .map_err(|e| format!("Failed to parse image models file: {e}"))?;
    let secrets = load_secrets_for_home(home);
    for model in &mut models {
        if let ImageSource::Custom { api_key, .. } = &mut model.source {
            if let Some(secret) = secrets.get(&model.id) {
                *api_key = secret.clone();
            }
        }
    }
    Ok(models)
}

/// Saves playground image models. Custom-endpoint API keys are stripped from
/// the model file and written to an owner-only secrets file instead.
pub fn save_image_models(models: &[ImageModel]) -> Result<(), String> {
    save_image_models_for_home(&crate::paths::get_home_dir()?, models)
}

pub fn save_image_models_for_home(home: &Path, models: &[ImageModel]) -> Result<(), String> {
    let mut sanitized = models.to_vec();
    let mut secrets = HashMap::new();
    for model in &mut sanitized {
        if let ImageSource::Custom { api_key, .. } = &mut model.source {
            let trimmed = api_key.trim();
            if !trimmed.is_empty() {
                secrets.insert(model.id.clone(), trimmed.to_string());
            }
            api_key.clear();
        }
    }

    let dir = playground_dir(home);
    fs::create_dir_all(&dir).map_err(|e| format!("Failed to create playground directory: {e}"))?;

    playground::write_secrets(&image_secrets_path_for_home(home), &secrets, "image")?;
    playground::write_json(
        &image_models_path_for_home(home),
        &sanitized,
        "image models",
    )
}

/// Loads persisted image tasks and reattaches custom-endpoint API keys.
pub fn load_image_tasks() -> Result<Vec<ImageTaskRecord>, String> {
    load_image_tasks_for_home(&crate::paths::get_home_dir()?)
}

pub fn load_image_tasks_for_home(home: &Path) -> Result<Vec<ImageTaskRecord>, String> {
    let path = image_tasks_path_for_home(home);
    if !path.exists() {
        return Ok(Vec::new());
    }
    let content = fs::read_to_string(&path)
        .map_err(|error| format!("Failed to read image tasks file: {error}"))?;
    if content.trim().is_empty() {
        return Ok(Vec::new());
    }
    let mut tasks: Vec<ImageTaskRecord> = serde_json::from_str(&content)
        .map_err(|error| format!("Failed to parse image tasks file: {error}"))?;
    // A task id becomes a media file name; drop corrupt records rather than
    // letting them reach the file writer.
    tasks.retain(|task| validate_task_id(&task.id).is_ok());

    let secrets = load_secrets_for_home(home);
    for task in &mut tasks {
        if let ImageSource::Custom { api_key, .. } = &mut task.model.source {
            if let Some(secret) = secrets.get(&task.model.id) {
                *api_key = secret.clone();
            }
        }
    }
    Ok(tasks)
}

/// Persists playground image tasks. Custom-endpoint API keys are stripped from
/// the task file; they are read back from `image_secrets.json` by model id.
pub fn save_image_tasks(tasks: &[ImageTaskRecord]) -> Result<(), String> {
    save_image_tasks_for_home(&crate::paths::get_home_dir()?, tasks)
}

pub fn save_image_tasks_for_home(home: &Path, tasks: &[ImageTaskRecord]) -> Result<(), String> {
    let mut sanitized = tasks.to_vec();
    for task in &mut sanitized {
        if let ImageSource::Custom { api_key, .. } = &mut task.model.source {
            api_key.clear();
        }
    }

    let dir = playground_dir(home);
    fs::create_dir_all(&dir)
        .map_err(|error| format!("Failed to create playground directory: {error}"))?;
    playground::write_json(&image_tasks_path_for_home(home), &sanitized, "image tasks")
}

fn load_secrets_for_home(home: &Path) -> HashMap<String, String> {
    playground::read_secrets(&image_secrets_path_for_home(home), "Image")
}

fn validate_task_id(task_id: &str) -> Result<(), String> {
    if task_id.is_empty() {
        return Err("Task ID is empty.".to_string());
    }
    if task_id.len() > 120
        || !task_id
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, '-' | '_' | '.'))
    {
        return Err("Task ID contains unsupported characters.".to_string());
    }
    Ok(())
}

// ============================================================================
// Source resolution
// ============================================================================

async fn resolve_source(model: &ImageModel) -> Result<ResolvedSource, String> {
    match &model.source {
        ImageSource::Custom { base_url, api_key } => {
            playground::resolve_custom_source(base_url, api_key)
        }
        ImageSource::Channel {
            channel_id,
            token_id,
        } => playground::resolve_channel_source(channel_id, *token_id).await,
    }
}

// ============================================================================
// URLs
// ============================================================================

fn trimmed(base: &str) -> &str {
    base.trim().trim_end_matches('/')
}

/// Base URL ending at `/v1`. Accepts a root, a `/v1` base, or an endpoint URL
/// copied from the docs (chat/completions, responses, images/generations).
fn openai_api_base(base: &str) -> String {
    let base = trimmed(base);
    for suffix in [
        "/v1/chat/completions",
        "/chat/completions",
        "/v1/responses",
        "/responses",
        "/v1/images/generations",
        "/images/generations",
    ] {
        if let Some(stripped) = base.strip_suffix(suffix) {
            return openai_api_base(stripped);
        }
    }
    // `/v1` is the usual OpenAI base. `/openai` is Google's OpenAI-compatible
    // base (`/v1beta/openai`), which already carries its version.
    if base.ends_with("/v1") || base.ends_with("/openai") {
        base.to_string()
    } else {
        format!("{base}/v1")
    }
}

fn chat_completions_url(base: &str) -> String {
    format!("{}/chat/completions", openai_api_base(base))
}

fn responses_url(base: &str) -> String {
    format!("{}/responses", openai_api_base(base))
}

/// Base URL ending at the Google API version (`/v1beta` by default).
///
/// Accepts a root, a `/v1beta` base, or a full model URL copied from the docs
/// (`/v1beta/models/...`).
fn google_api_base(base: &str) -> String {
    let base = trimmed(base);
    let base = if let Some(index) = base.find("/models/") {
        &base[..index]
    } else {
        base.strip_suffix("/models").unwrap_or(base)
    };
    if base.ends_with("/v1beta") || base.ends_with("/v1") {
        base.to_string()
    } else {
        format!("{base}/v1beta")
    }
}

fn google_model_segment(model: &str) -> Result<String, String> {
    let model = model.trim().trim_start_matches("models/");
    if model.is_empty() {
        return Err("This model has no model ID. Edit it in Manage models.".to_string());
    }
    if model.contains(['?', '#', ' ']) {
        return Err(
            "The model ID contains characters that cannot be used in a Google URL.".to_string(),
        );
    }
    if model.contains(':') {
        return Err(
            "Use the bare model ID for Google models (without :generateContent).".to_string(),
        );
    }
    Ok(model.to_string())
}

fn google_generate_url(base: &str, model: &str) -> Result<String, String> {
    Ok(format!(
        "{}/models/{}:generateContent",
        google_api_base(base),
        google_model_segment(model)?
    ))
}

fn google_predict_url(base: &str, model: &str) -> Result<String, String> {
    Ok(format!(
        "{}/models/{}:predict",
        google_api_base(base),
        google_model_segment(model)?
    ))
}

/// Gemini expects `x-goog-api-key`; New API style gateways usually accept the
/// bearer token too. Send both, skipping empty values.
fn google_authorized(request: reqwest::RequestBuilder, api_key: &str) -> reqwest::RequestBuilder {
    let request = authorized(request, api_key);
    if api_key.trim().is_empty() {
        request
    } else {
        request.header("x-goog-api-key", api_key.trim())
    }
}

fn is_imagen_model(model_id: &str) -> bool {
    model_id
        .trim()
        .trim_start_matches("models/")
        .to_ascii_lowercase()
        .starts_with("imagen")
}

// ============================================================================
// Sizes and request bodies
// ============================================================================

/// Splits the Google `size` field into `(aspect_ratio, image_size)`.
///
/// Tokens may be combined with spaces or commas: `16:9 2K`. `WxH` is reduced to
/// an aspect ratio; a bare `1K`/`2K`/`4K`/`512` is an image size.
fn split_google_size(size: &str) -> (Option<String>, Option<String>) {
    let mut aspect_ratio = None;
    let mut image_size = None;
    for token in size.split([' ', ',', ';']).filter(|t| !t.trim().is_empty()) {
        let token = token.trim();
        if let Some((width, height)) = parse_dimensions(token) {
            aspect_ratio = Some(reduced_ratio(width, height));
            continue;
        }
        if token.contains(':') {
            let valid = token.split_once(':').is_some_and(|(w, h)| {
                w.trim().parse::<u32>().is_ok() && h.trim().parse::<u32>().is_ok()
            });
            if valid {
                aspect_ratio = Some(token.to_string());
            }
            continue;
        }
        let upper = token.to_ascii_uppercase();
        if matches!(upper.as_str(), "512" | "1K" | "2K" | "4K") {
            image_size = Some(upper);
        }
    }
    (aspect_ratio, image_size)
}

fn parse_dimensions(value: &str) -> Option<(u32, u32)> {
    let (width, height) = value.split_once(['x', 'X', '*'])?;
    let width = width.trim().parse::<u32>().ok()?.max(1);
    let height = height.trim().parse::<u32>().ok()?.max(1);
    Some((width, height))
}

fn reduced_ratio(width: u32, height: u32) -> String {
    let gcd = gcd(width, height);
    format!("{}:{}", width / gcd, height / gcd)
}

fn gcd(mut a: u32, mut b: u32) -> u32 {
    while b != 0 {
        let remainder = a % b;
        a = b;
        b = remainder;
    }
    a.max(1)
}

fn parse_extra(extra_json: &str) -> Result<Map<String, Value>, String> {
    let extra_json = extra_json.trim();
    if extra_json.is_empty() {
        return Ok(Map::new());
    }
    let value: Value = serde_json::from_str(extra_json)
        .map_err(|error| format!("Advanced parameters must be valid JSON: {error}"))?;
    match value {
        Value::Object(map) => Ok(map),
        _ => Err("Advanced parameters must be a JSON object.".to_string()),
    }
}

fn merge_object(base: &mut Value, extra: &Map<String, Value>, protected: &[&str]) {
    let Some(object) = base.as_object_mut() else {
        return;
    };
    for (key, value) in extra {
        if protected.contains(&key.as_str()) {
            continue;
        }
        object.insert(key.clone(), value.clone());
    }
}

struct ImageParams<'a> {
    model: &'a str,
    prompt: &'a str,
    size: &'a str,
    quality: &'a str,
    count: u32,
    image_urls: Vec<&'a str>,
    extra: Map<String, Value>,
}

/// ChatCompletions body. `modalities` is what image-capable chat gateways
/// (OpenRouter and friends) use to ask for an image; New API's Gemini channel
/// adds Gemini's `responseModalities` on its own.
fn build_chat_body(params: &ImageParams<'_>) -> Value {
    let mut content = vec![json!({ "type": "text", "text": params.prompt })];
    for url in &params.image_urls {
        content.push(json!({
            "type": "image_url",
            "image_url": { "url": url }
        }));
    }

    let mut body = json!({
        "model": params.model,
        "messages": [{ "role": "user", "content": content }],
        "modalities": ["image", "text"],
    });
    // Only the message and modality shape is forced; `n`, `size` and
    // `image_config` from the advanced JSON survive when the form leaves them
    // empty, and the form wins when it sets them.
    merge_object(
        &mut body,
        &params.extra,
        &["model", "messages", "modalities"],
    );

    if params.count > 1 {
        body["n"] = json!(params.count);
    }
    if !params.size.is_empty() {
        if params.size.contains(':') {
            body["image_config"] = json!({ "aspect_ratio": params.size });
        } else {
            body["size"] = json!(params.size);
        }
    }
    body
}

/// Responses body with the server-side image generation tool.
fn build_responses_body(params: &ImageParams<'_>) -> Value {
    let mut content = vec![json!({ "type": "input_text", "text": params.prompt })];
    for url in &params.image_urls {
        content.push(json!({ "type": "input_image", "image_url": url }));
    }

    let mut tool = json!({ "type": "image_generation" });
    if !params.size.is_empty() {
        tool["size"] = json!(params.size);
    }
    if !params.quality.is_empty() {
        tool["quality"] = json!(params.quality);
    }

    let mut body = json!({
        "model": params.model,
        "input": [{ "role": "user", "content": content }],
        "tools": [tool],
    });
    merge_object(&mut body, &params.extra, &["model", "input"]);

    // Advanced parameters may declare their own tools. Keep them, but make sure
    // an image_generation tool is present and carries the form's options.
    match body.get_mut("tools").and_then(Value::as_array_mut) {
        Some(tools) => {
            let existing = tools
                .iter_mut()
                .find(|item| item.get("type").and_then(Value::as_str) == Some("image_generation"));
            match existing {
                Some(item) => {
                    if !params.size.is_empty() {
                        item["size"] = json!(params.size);
                    }
                    if !params.quality.is_empty() {
                        item["quality"] = json!(params.quality);
                    }
                }
                None => tools.push(tool),
            }
        }
        None => body["tools"] = json!([tool]),
    }
    body
}

/// Gemini `generateContent` body. Reference images must be inlined as base64,
/// so the caller resolves them first.
fn build_google_body(params: &ImageParams<'_>, inline_images: &[(String, String)]) -> Value {
    let mut parts = vec![json!({ "text": params.prompt })];
    for (mime_type, data) in inline_images {
        parts.push(json!({
            "inlineData": { "mimeType": mime_type, "data": data }
        }));
    }

    let mut generation_config = json!({
        "responseModalities": ["TEXT", "IMAGE"]
    });
    let (aspect_ratio, image_size) = split_google_size(params.size);
    let mut image_config = Map::new();
    if let Some(aspect_ratio) = aspect_ratio {
        image_config.insert("aspectRatio".to_string(), json!(aspect_ratio));
    }
    if let Some(image_size) = image_size {
        image_config.insert("imageSize".to_string(), json!(image_size));
    }
    let image_config = (!image_config.is_empty()).then(|| Value::Object(image_config));
    if let Some(config) = &image_config {
        generation_config["imageConfig"] = config.clone();
    }

    let mut body = json!({
        "contents": [{ "role": "user", "parts": parts }],
        "generationConfig": generation_config,
    });
    // `generationConfig` is merged below key by key so the user can add
    // thinkingConfig/seed without dropping the image options.
    merge_object(&mut body, &params.extra, &["contents", "generationConfig"]);

    if let Some(extra_config) = params
        .extra
        .get("generationConfig")
        .and_then(Value::as_object)
    {
        if let Some(config) = body
            .get_mut("generationConfig")
            .and_then(Value::as_object_mut)
        {
            for (key, value) in extra_config {
                // The form's responseModalities/imageConfig already exist and
                // take precedence; everything else is additive.
                if key == "imageConfig" {
                    continue;
                }
                config.entry(key.clone()).or_insert_with(|| value.clone());
            }
            if let Some(extra_image_config) =
                extra_config.get("imageConfig").and_then(Value::as_object)
            {
                match config.get_mut("imageConfig").and_then(Value::as_object_mut) {
                    Some(target) => {
                        for (key, value) in extra_image_config {
                            target.entry(key.clone()).or_insert_with(|| value.clone());
                        }
                    }
                    None => {
                        config.insert(
                            "imageConfig".to_string(),
                            Value::Object(extra_image_config.clone()),
                        );
                    }
                }
            }
        }
    }
    body
}

/// Imagen `:predict` body.
fn build_imagen_body(params: &ImageParams<'_>) -> Value {
    let mut parameters = Map::new();
    parameters.insert("sampleCount".to_string(), json!(params.count.clamp(1, 4)));
    parameters.insert("personGeneration".to_string(), json!("allow_adult"));
    let (aspect_ratio, image_size) = split_google_size(params.size);
    if let Some(aspect_ratio) = aspect_ratio {
        parameters.insert("aspectRatio".to_string(), json!(aspect_ratio));
    }
    if let Some(image_size) = image_size {
        parameters.insert("imageSize".to_string(), json!(image_size));
    }

    let mut body = json!({
        "instances": [{ "prompt": params.prompt }],
        "parameters": Value::Object(parameters),
    });
    merge_object(&mut body, &params.extra, &["instances"]);

    // Merge the user's parameters map instead of replacing the defaults.
    if let Some(user_parameters) = params.extra.get("parameters").and_then(Value::as_object) {
        if let Some(target) = body.get_mut("parameters").and_then(Value::as_object_mut) {
            for (key, value) in user_parameters {
                target.entry(key.clone()).or_insert_with(|| value.clone());
            }
        }
    }
    body
}

// ============================================================================
// Reference images
// ============================================================================

fn parse_data_url(url: &str) -> Option<(String, String)> {
    let rest = url.strip_prefix("data:")?;
    let (meta, data) = rest.split_once(',')?;
    if !meta.to_ascii_lowercase().contains(";base64") {
        return None;
    }
    let mime_type = meta
        .split(';')
        .next()
        .map(str::trim)
        .filter(|mime| !mime.is_empty())
        .unwrap_or("image/png");
    Some((mime_type.to_string(), data.to_string()))
}

fn strip_base64_whitespace(value: &str) -> String {
    value.chars().filter(|c| !c.is_whitespace()).collect()
}

fn sniff_image_mime(bytes: &[u8]) -> Option<&'static str> {
    match bytes {
        [0x89, b'P', b'N', b'G', ..] => Some("image/png"),
        [0xFF, 0xD8, 0xFF, ..] => Some("image/jpeg"),
        [b'G', b'I', b'F', b'8', ..] => Some("image/gif"),
        [b'R', b'I', b'F', b'F', _, _, _, _, b'W', b'E', b'B', b'P', ..] => Some("image/webp"),
        [b'B', b'M', ..] => Some("image/bmp"),
        _ => None,
    }
}

fn sniff_mime(bytes: &[u8]) -> &'static str {
    sniff_image_mime(bytes).unwrap_or("image/png")
}

fn normalize_mime(mime_type: &str) -> String {
    let mime_type = mime_type.trim();
    if mime_type.is_empty() {
        return "image/png".to_string();
    }
    let mime_type = mime_type.strip_prefix("image/").unwrap_or(mime_type);
    match mime_type.to_ascii_lowercase().as_str() {
        "png" | "jpeg" | "jpg" | "gif" | "webp" => format!(
            "image/{}",
            if mime_type.eq_ignore_ascii_case("jpg") {
                "jpeg"
            } else {
                mime_type
            }
        ),
        other => format!("image/{other}"),
    }
}

fn extension_for_mime(mime_type: &str) -> &'static str {
    match mime_type {
        "image/jpeg" => "jpg",
        "image/gif" => "gif",
        "image/webp" => "webp",
        "image/bmp" => "bmp",
        _ => "png",
    }
}

/// Loads a reference image as `(mime_type, base64)` for the native Google
/// wire, which only accepts inline data. Local files arrive as `data:` URLs;
/// remote URLs are fetched and inlined here.
async fn load_reference_image(
    client: &reqwest::Client,
    url: &str,
) -> Result<(String, String), String> {
    if let Some((mime_type, data)) = parse_data_url(url) {
        return Ok((normalize_mime(&mime_type), strip_base64_whitespace(&data)));
    }
    if !url.starts_with("http://") && !url.starts_with("https://") {
        return Err("Reference images must be http(s) URLs or data URLs.".to_string());
    }

    let response = client
        .get(url)
        .timeout(REFERENCE_TIMEOUT)
        .send()
        .await
        .map_err(|error| format!("Failed to download reference image: {error}"))?;
    let status = response.status();
    if !status.is_success() {
        let body = response.text().await.unwrap_or_default();
        return Err(format_api_error(status, &body));
    }
    let header_type = response
        .headers()
        .get(reqwest::header::CONTENT_TYPE)
        .and_then(|value| value.to_str().ok())
        .unwrap_or_default()
        .to_ascii_lowercase();
    let bytes = response
        .bytes()
        .await
        .map_err(|error| format!("Failed to read reference image: {error}"))?;
    if bytes.is_empty() {
        return Err("The reference image URL returned an empty body.".to_string());
    }
    if bytes.len() > MAX_REFERENCE_BYTES {
        return Err(format!(
            "Reference images must be at most {} MB.",
            MAX_REFERENCE_BYTES / 1024 / 1024
        ));
    }
    let mime_type = if header_type.starts_with("image/") {
        normalize_mime(&header_type)
    } else if let Some(mime_type) = sniff_image_mime(&bytes) {
        mime_type.to_string()
    } else {
        return Err("The reference image URL returned a body that is not an image.".to_string());
    };
    Ok((
        mime_type,
        base64::engine::general_purpose::STANDARD.encode(bytes),
    ))
}

// ============================================================================
// Response parsing
// ============================================================================

#[derive(Debug, PartialEq, Eq)]
enum RawImage {
    Data { mime_type: String, base64: String },
    Url(String),
}

fn classify_image(value: &str) -> Option<RawImage> {
    let value = value.trim();
    if value.is_empty() {
        return None;
    }
    if let Some((mime_type, data)) = parse_data_url(value) {
        return Some(RawImage::Data {
            mime_type,
            base64: strip_base64_whitespace(&data),
        });
    }
    if value.starts_with("http://") || value.starts_with("https://") {
        return Some(RawImage::Url(value.to_string()));
    }
    // A bare base64 payload; mime is sniffed from the decoded bytes.
    Some(RawImage::Data {
        mime_type: String::new(),
        base64: strip_base64_whitespace(value),
    })
}

/// Extracts markdown image targets (`![alt](url)`), which is how New API's
/// Gemini channel embeds images in ChatCompletions content.
fn markdown_image_targets(text: &str) -> Vec<String> {
    let mut targets = Vec::new();
    let mut rest = text;
    while let Some(start) = rest.find("![") {
        let after = &rest[start + 2..];
        let Some(label_end) = after.find("](") else {
            rest = after;
            continue;
        };
        let target = &after[label_end + 2..];
        let Some(end) = target.find(')') else {
            break;
        };
        let url = target[..end]
            .trim()
            .trim_start_matches('<')
            .trim_end_matches('>')
            .split_whitespace()
            .next()
            .unwrap_or_default();
        if !url.is_empty() {
            targets.push(url.to_string());
        }
        rest = &target[end + 1..];
    }
    targets
}

fn push_image(images: &mut Vec<RawImage>, value: &str) {
    if let Some(image) = classify_image(value) {
        if !images.contains(&image) {
            images.push(image);
        }
    }
}

fn collect_text(value: &Value, target: &mut Vec<String>, images: &mut Vec<RawImage>) {
    match value {
        Value::String(text) => {
            let text = text.trim();
            if !text.is_empty() {
                for url in markdown_image_targets(text) {
                    push_image(images, &url);
                }
                target.push(text.to_string());
            }
        }
        Value::Array(items) => {
            for item in items {
                collect_text(item, target, images);
            }
        }
        Value::Object(object) => {
            if let Some(text) = object.get("text").and_then(Value::as_str) {
                for url in markdown_image_targets(text) {
                    push_image(images, &url);
                }
                if !text.trim().is_empty() {
                    target.push(text.trim().to_string());
                }
            }
            for key in ["image_url", "url", "data"] {
                if let Some(image) = object.get(key) {
                    collect_image_field(image, images);
                }
            }
            if let Some(data) = object.get("b64_json").and_then(Value::as_str) {
                push_image(images, data);
            }
        }
        _ => {}
    }
}

fn collect_image_field(value: &Value, images: &mut Vec<RawImage>) {
    match value {
        Value::String(text) => push_image(images, text),
        Value::Object(object) => {
            if let Some(url) = object.get("url").and_then(Value::as_str) {
                push_image(images, url);
            }
            if let Some(data) = object.get("b64_json").and_then(Value::as_str) {
                push_image(images, data);
            }
        }
        _ => {}
    }
}

/// Normalized pieces of one protocol answer.
#[derive(Debug)]
struct ParsedResponse {
    images: Vec<RawImage>,
    revised_prompt: Option<String>,
    response_text: Option<String>,
}

/// Parses a ChatCompletions response: `message.images`, markdown images in
/// `message.content`, and content-part arrays are all accepted.
fn parse_chat_response(value: &Value) -> ParsedResponse {
    let mut images = Vec::new();
    let mut texts = Vec::new();

    if let Some(choices) = value.get("choices").and_then(Value::as_array) {
        for choice in choices {
            let message = choice.get("message").unwrap_or(choice);
            if let Some(content) = message.get("content") {
                collect_text(content, &mut texts, &mut images);
            }
            if let Some(items) = message.get("images").and_then(Value::as_array) {
                for item in items {
                    match item {
                        Value::String(text) => push_image(&mut images, text),
                        Value::Object(object) => {
                            collect_image_field(
                                object.get("image_url").unwrap_or(item),
                                &mut images,
                            );
                            if let Some(data) = object.get("b64_json").and_then(Value::as_str) {
                                push_image(&mut images, data);
                            }
                        }
                        _ => {}
                    }
                }
            }
        }
    }

    // Some gateways answer a chat request with the Images API shape.
    if images.is_empty() {
        if let Some(data) = value.get("data") {
            collect_text(data, &mut texts, &mut images);
        }
    }

    ParsedResponse {
        images,
        revised_prompt: None,
        response_text: join_text(texts),
    }
}

/// Parses a Responses response: image_generation_call items carry base64
/// results, and output text is kept as a short explanation.
fn parse_responses_response(value: &Value) -> Result<ParsedResponse, String> {
    let mut images = Vec::new();
    let mut texts = Vec::new();
    let mut revised_prompt = None;
    let mut failure: Option<String> = None;

    if let Some(output) = value.get("output").and_then(Value::as_array) {
        for item in output {
            let item_type = item.get("type").and_then(Value::as_str).unwrap_or_default();
            if item_type == "image_generation_call" {
                if item.get("status").and_then(Value::as_str) == Some("failed") {
                    // A later call may still have produced an image, so only
                    // surface this failure when nothing else decoded.
                    if failure.is_none() {
                        failure = Some(
                            item.get("error")
                                .and_then(playground::value_message)
                                .unwrap_or_else(|| "The image generation call failed.".to_string()),
                        );
                    }
                    continue;
                }
                if revised_prompt.is_none() {
                    revised_prompt = item
                        .get("revised_prompt")
                        .and_then(Value::as_str)
                        .map(str::trim)
                        .filter(|text| !text.is_empty())
                        .map(str::to_string);
                }
                if let Some(result) = item.get("result").and_then(Value::as_str) {
                    push_image(&mut images, result);
                }
                continue;
            }
            if let Some(content) = item.get("content") {
                collect_text(content, &mut texts, &mut images);
            }
        }
    }

    let status = value
        .get("status")
        .and_then(Value::as_str)
        .unwrap_or_default();
    if images.is_empty() && status == "failed" {
        let reason = value
            .get("error")
            .and_then(playground::value_message)
            .or_else(|| {
                value
                    .get("incomplete_details")
                    .and_then(playground::value_message)
            })
            .unwrap_or_else(|| "The Responses request failed.".to_string());
        return Err(reason);
    }
    if images.is_empty() {
        if let Some(reason) = failure {
            return Err(reason);
        }
    }

    Ok(ParsedResponse {
        images,
        revised_prompt,
        response_text: join_text(texts),
    })
}

/// Parses a Gemini `generateContent` response or an Imagen `:predict`
/// response.
fn parse_google_response(value: &Value) -> Result<ParsedResponse, String> {
    let mut images = Vec::new();
    let mut texts = Vec::new();
    let mut finish_reason: Option<String> = None;

    if let Some(candidates) = value.get("candidates").and_then(Value::as_array) {
        for candidate in candidates {
            if let Some(reason) = candidate.get("finishReason").and_then(Value::as_str) {
                if !reason.is_empty() && reason != "STOP" {
                    finish_reason = Some(reason.to_string());
                }
            }
            let Some(parts) = candidate
                .get("content")
                .and_then(|content| content.get("parts"))
                .and_then(Value::as_array)
            else {
                continue;
            };
            for part in parts {
                if let Some(inline) = part.get("inlineData").or_else(|| part.get("inline_data")) {
                    let mime_type = inline
                        .get("mimeType")
                        .or_else(|| inline.get("mime_type"))
                        .and_then(Value::as_str);
                    if let Some(data) = inline.get("data").and_then(Value::as_str) {
                        push_image_with_mime(&mut images, mime_type, data);
                    }
                }
                if let Some(text) = part.get("text").and_then(Value::as_str) {
                    for url in markdown_image_targets(text) {
                        push_image(&mut images, &url);
                    }
                    if !text.trim().is_empty() {
                        texts.push(text.trim().to_string());
                    }
                }
            }
        }
    }

    let mut filtered_reason: Option<String> = None;
    if let Some(predictions) = value.get("predictions").and_then(Value::as_array) {
        for prediction in predictions {
            if let Some(reason) = prediction
                .get("raiFilteredReason")
                .and_then(Value::as_str)
                .map(str::trim)
                .filter(|reason| !reason.is_empty())
            {
                filtered_reason.get_or_insert_with(|| reason.to_string());
                continue;
            }
            let data = prediction
                .get("bytesBase64Encoded")
                .or_else(|| prediction.get("bytes_base64_encoded"))
                .and_then(Value::as_str);
            if let Some(data) = data {
                let mime_type = prediction
                    .get("mimeType")
                    .or_else(|| prediction.get("mime_type"))
                    .and_then(Value::as_str);
                push_image_with_mime(&mut images, mime_type, data);
            }
        }
    }

    let blocked = value
        .get("promptFeedback")
        .or_else(|| value.get("prompt_feedback"))
        .and_then(|feedback| {
            feedback
                .get("blockReason")
                .or_else(|| feedback.get("block_reason"))
        })
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|reason| !reason.is_empty());
    if images.is_empty() {
        if let Some(reason) = filtered_reason {
            return Err(format!("The image was filtered by Google: {reason}"));
        }
        if let Some(reason) = blocked {
            return Err(format!("Google blocked the prompt: {reason}"));
        }
        if let Some(reason) = finish_reason.filter(|reason| is_blocking_finish_reason(reason)) {
            return Err(format!("Google refused to generate the image: {reason}"));
        }
    }

    Ok(ParsedResponse {
        images,
        revised_prompt: None,
        response_text: join_text(texts),
    })
}

fn is_blocking_finish_reason(reason: &str) -> bool {
    matches!(
        reason,
        "SAFETY"
            | "RECITATION"
            | "BLOCKLIST"
            | "PROHIBITED_CONTENT"
            | "SPII"
            | "IMAGE_SAFETY"
            | "IMAGE_PROHIBITED_CONTENT"
            | "IMAGE_RECITATION"
    )
}

fn push_image_with_mime(images: &mut Vec<RawImage>, mime_type: Option<&str>, data: &str) {
    // An absent MIME type stays empty so the bytes are sniffed on save.
    let image = RawImage::Data {
        mime_type: mime_type.map(normalize_mime).unwrap_or_default(),
        base64: strip_base64_whitespace(data),
    };
    if !image_is_empty(&image) && !images.contains(&image) {
        images.push(image);
    }
}

fn image_is_empty(image: &RawImage) -> bool {
    match image {
        RawImage::Data { base64, .. } => base64.is_empty(),
        RawImage::Url(url) => url.trim().is_empty(),
    }
}

fn join_text(parts: Vec<String>) -> Option<String> {
    let joined = parts
        .into_iter()
        .filter(|part| !part.trim().is_empty())
        .collect::<Vec<_>>()
        .join("\n\n");
    let joined = joined.trim();
    (!joined.is_empty()).then(|| joined.to_string())
}

fn body_error(value: &Value) -> Option<String> {
    value.get("error").and_then(playground::value_message)
}

// ============================================================================
// Writing images
// ============================================================================

fn now_millis() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_millis() as i64)
        .unwrap_or_default()
}

fn write_image(
    dir: &Path,
    task_id: &str,
    index: usize,
    bytes: &[u8],
    mime_type: &str,
) -> Result<String, String> {
    if bytes.is_empty() {
        return Err("The upstream returned an empty image.".to_string());
    }
    if bytes.len() > MAX_IMAGE_BYTES {
        return Err(format!(
            "The generated image is larger than {} MB.",
            MAX_IMAGE_BYTES / 1024 / 1024
        ));
    }
    let path = dir.join(format!(
        "{task_id}-{index}.{}",
        extension_for_mime(mime_type)
    ));
    if fs::metadata(&path)
        .map(|meta| meta.len() > 0)
        .unwrap_or(false)
    {
        return Ok(path.to_string_lossy().to_string());
    }
    crate::storage::atomic_write(&path, bytes)
        .map_err(|error| format!("Failed to save the generated image: {error}"))?;
    Ok(path.to_string_lossy().to_string())
}

async fn download_image(client: &reqwest::Client, url: &str) -> Result<(Vec<u8>, String), String> {
    let response = client
        .get(url)
        .timeout(DOWNLOAD_TIMEOUT)
        .send()
        .await
        .map_err(|error| format!("Failed to download the generated image: {error}"))?;
    let status = response.status();
    if !status.is_success() {
        let body = response.text().await.unwrap_or_default();
        return Err(format_api_error(status, &body));
    }
    let header_type = response
        .headers()
        .get(reqwest::header::CONTENT_TYPE)
        .and_then(|value| value.to_str().ok())
        .unwrap_or_default()
        .to_string();
    let bytes = response
        .bytes()
        .await
        .map_err(|error| format!("Failed to read the generated image: {error}"))?
        .to_vec();
    if bytes.is_empty() {
        return Err("The image URL returned an empty body.".to_string());
    }
    if bytes.len() > MAX_IMAGE_BYTES {
        return Err("The generated image is larger than the supported size.".to_string());
    }
    let mime_type = if header_type.to_ascii_lowercase().starts_with("image/") {
        normalize_mime(&header_type)
    } else if let Some(mime_type) = sniff_image_mime(&bytes) {
        mime_type.to_string()
    } else {
        return Err("The image URL returned a body that is not an image.".to_string());
    };
    Ok((bytes, mime_type))
}

/// Decodes every returned image into the media directory. Remote URLs are
/// downloaded for durability; if that fails the URL is kept so the UI can
/// still render it.
async fn persist_images(
    client: &reqwest::Client,
    task_id: &str,
    images: Vec<RawImage>,
) -> Result<Vec<ImageAsset>, String> {
    let home = crate::paths::get_home_dir()?;
    let dir = image_media_dir_for_home(&home);
    fs::create_dir_all(&dir)
        .map_err(|error| format!("Failed to create image media directory: {error}"))?;

    let mut assets = Vec::new();
    for (index, image) in images.into_iter().enumerate() {
        match image {
            RawImage::Data { mime_type, base64 } => {
                let bytes =
                    match base64::engine::general_purpose::STANDARD.decode(base64.as_bytes()) {
                        Ok(bytes) => bytes,
                        Err(error) => {
                            // A malformed entry must not discard the images that
                            // did decode.
                            log::warn!("Image: skipping undecodable image data: {error}");
                            continue;
                        }
                    };
                let mime_type = if mime_type.trim().is_empty() {
                    sniff_mime(&bytes).to_string()
                } else {
                    normalize_mime(&mime_type)
                };
                let local_path = write_image(&dir, task_id, index, &bytes, &mime_type)?;
                assets.push(ImageAsset {
                    local_path: Some(local_path),
                    url: None,
                    mime_type,
                });
            }
            RawImage::Url(url) => match download_image(client, &url).await {
                Ok((bytes, mime_type)) => {
                    let local_path = write_image(&dir, task_id, index, &bytes, &mime_type)?;
                    assets.push(ImageAsset {
                        local_path: Some(local_path),
                        url: Some(url),
                        mime_type,
                    });
                }
                Err(error) => {
                    log::warn!("Image: failed to cache generated image: {error}");
                    assets.push(ImageAsset {
                        local_path: None,
                        url: Some(url),
                        mime_type: String::new(),
                    });
                }
            },
        }
    }
    Ok(assets)
}

// ============================================================================
// Public protocol operation
// ============================================================================

/// Generates images with the model's protocol and stores them locally.
#[allow(clippy::too_many_arguments)]
pub async fn create_image_task(
    model: &ImageModel,
    task_id: &str,
    prompt: &str,
    size: &str,
    quality: &str,
    count: u32,
    image_urls: Vec<String>,
    extra_json: &str,
) -> Result<ImageTaskSnapshot, String> {
    validate_task_id(task_id)?;
    let prompt = prompt.trim();
    if prompt.is_empty() {
        return Err("Prompt is required.".to_string());
    }
    if count == 0 || count as usize > MAX_IMAGES_PER_TASK {
        return Err(format!(
            "Image count must be between 1 and {MAX_IMAGES_PER_TASK}."
        ));
    }
    let image_urls: Vec<&str> = image_urls
        .iter()
        .map(|url| url.trim())
        .filter(|url| !url.is_empty())
        .collect();
    if !model.supports_image_input && !image_urls.is_empty() {
        return Err(
            "This model is configured without reference-image support. Remove the images or enable the option in Manage models."
                .to_string(),
        );
    }
    let extra = parse_extra(extra_json)?;
    let params = ImageParams {
        model: model.model_id.trim(),
        prompt,
        size: size.trim(),
        quality: quality.trim(),
        count,
        image_urls,
        extra,
    };
    if params.model.is_empty() {
        return Err("This model has no model ID. Edit it in Manage models.".to_string());
    }

    let source = resolve_source(model).await?;
    let client = playground::build_client(GENERATION_TIMEOUT)?;

    let parsed = match model.protocol {
        ImageProtocol::Chat => {
            let body = build_chat_body(&params);
            let request = authorized(
                client
                    .post(chat_completions_url(&source.base_url))
                    .json(&body),
                &source.api_key,
            )
            .timeout(GENERATION_TIMEOUT);
            let value = send_json(request).await?;
            if let Some(error) = body_error(&value) {
                return Err(error);
            }
            parse_chat_response(&value)
        }
        ImageProtocol::Responses => {
            let body = build_responses_body(&params);
            let request = authorized(
                client.post(responses_url(&source.base_url)).json(&body),
                &source.api_key,
            )
            .timeout(GENERATION_TIMEOUT);
            let value = send_json(request).await?;
            if let Some(error) = body_error(&value) {
                return Err(error);
            }
            parse_responses_response(&value)?
        }
        ImageProtocol::Google => {
            if is_imagen_model(params.model) {
                if !params.image_urls.is_empty() {
                    return Err(
                        "Imagen models do not accept reference images. Use a Gemini image model for image editing."
                            .to_string(),
                    );
                }
                let body = build_imagen_body(&params);
                let url = google_predict_url(&source.base_url, params.model)?;
                let request = google_authorized(client.post(url).json(&body), &source.api_key)
                    .timeout(GENERATION_TIMEOUT);
                let value = send_json(request).await?;
                if let Some(error) = body_error(&value) {
                    return Err(error);
                }
                parse_google_response(&value)?
            } else {
                let mut inline_images = Vec::new();
                for url in &params.image_urls {
                    inline_images.push(load_reference_image(&client, url).await?);
                }
                let body = build_google_body(&params, &inline_images);
                let url = google_generate_url(&source.base_url, params.model)?;
                let request = google_authorized(client.post(url).json(&body), &source.api_key)
                    .timeout(GENERATION_TIMEOUT);
                let value = send_json(request).await?;
                if let Some(error) = body_error(&value) {
                    return Err(error);
                }
                parse_google_response(&value)?
            }
        }
    };

    if parsed.images.is_empty() {
        return Err(no_image_error(parsed.response_text.as_deref()));
    }
    let images = persist_images(&client, task_id, parsed.images).await?;
    if images.is_empty() {
        return Err("The upstream returned image data that could not be decoded.".to_string());
    }
    Ok(ImageTaskSnapshot {
        images,
        revised_prompt: parsed.revised_prompt,
        response_text: parsed.response_text,
        completed_at: now_millis(),
    })
}

fn no_image_error(response_text: Option<&str>) -> String {
    match response_text.map(str::trim).filter(|text| !text.is_empty()) {
        Some(text) => format!(
            "The upstream returned no images. Its answer was: {}",
            snippet(text)
        ),
        None => "The upstream returned no images.".to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn params<'a>(model: &'a str, size: &'a str) -> ImageParams<'a> {
        ImageParams {
            model,
            prompt: "a cat",
            size,
            quality: "",
            count: 1,
            image_urls: Vec::new(),
            extra: Map::new(),
        }
    }

    #[test]
    fn openai_urls_normalize_roots_versions_and_endpoints() {
        assert_eq!(
            chat_completions_url("https://gateway.example"),
            "https://gateway.example/v1/chat/completions"
        );
        assert_eq!(
            chat_completions_url("https://gateway.example/v1/"),
            "https://gateway.example/v1/chat/completions"
        );
        assert_eq!(
            chat_completions_url("https://gateway.example/v1/chat/completions"),
            "https://gateway.example/v1/chat/completions"
        );
        assert_eq!(
            responses_url("https://gateway.example/v1/chat/completions"),
            "https://gateway.example/v1/responses"
        );
        assert_eq!(
            responses_url("https://openrouter.ai/api/v1"),
            "https://openrouter.ai/api/v1/responses"
        );
        assert_eq!(
            chat_completions_url("https://generativelanguage.googleapis.com/v1beta/openai"),
            "https://generativelanguage.googleapis.com/v1beta/openai/chat/completions"
        );
    }

    #[test]
    fn google_urls_use_the_v1beta_version_and_native_actions() {
        assert_eq!(
            google_generate_url("https://generativelanguage.googleapis.com", "gemini-3-pro-image")
                .unwrap(),
            "https://generativelanguage.googleapis.com/v1beta/models/gemini-3-pro-image:generateContent"
        );
        assert_eq!(
            google_predict_url("https://generativelanguage.googleapis.com/v1beta", "imagen-4.0-generate-001")
                .unwrap(),
            "https://generativelanguage.googleapis.com/v1beta/models/imagen-4.0-generate-001:predict"
        );
        assert_eq!(
            google_generate_url("https://gw.example/v1beta/models", "gemini-2.5-flash-image")
                .unwrap(),
            "https://gw.example/v1beta/models/gemini-2.5-flash-image:generateContent"
        );
    }

    #[test]
    fn google_model_ids_reject_url_control_characters() {
        assert!(google_generate_url("https://gw.example", "").is_err());
        assert!(google_generate_url("https://gw.example", "gemini x").is_err());
        assert!(google_generate_url("https://gw.example", "gemini:generateContent").is_err());
        assert_eq!(
            google_generate_url("https://gw.example", "models/gemini-2.5-flash-image").unwrap(),
            "https://gw.example/v1beta/models/gemini-2.5-flash-image:generateContent"
        );
    }

    #[test]
    fn google_sizes_split_ratios_from_image_sizes() {
        assert_eq!(split_google_size("16:9"), (Some("16:9".to_string()), None));
        assert_eq!(
            split_google_size("1536x1024"),
            (Some("3:2".to_string()), None)
        );
        assert_eq!(
            split_google_size("1024x1024 2K"),
            (Some("1:1".to_string()), Some("2K".to_string()))
        );
        assert_eq!(split_google_size("4k"), (None, Some("4K".to_string())));
        assert_eq!(split_google_size(""), (None, None));
        assert_eq!(split_google_size("weird"), (None, None));
    }

    #[test]
    fn chat_body_carries_text_images_and_modalities() {
        let mut params = params("google/gemini-3-pro-image", "16:9");
        params.count = 2;
        params.image_urls = vec!["https://example.com/a.png"];
        let body = build_chat_body(&params);
        assert_eq!(body["model"], "google/gemini-3-pro-image");
        assert_eq!(body["modalities"], json!(["image", "text"]));
        assert_eq!(body["n"], 2);
        assert_eq!(body["image_config"]["aspect_ratio"], "16:9");
        assert_eq!(body["messages"][0]["content"][0]["text"], "a cat");
        assert_eq!(
            body["messages"][0]["content"][1]["image_url"]["url"],
            "https://example.com/a.png"
        );
    }

    #[test]
    fn chat_body_keeps_advanced_size_fields_when_the_form_is_empty() {
        let mut params = params("google/gemini-3-pro-image", "");
        params.extra.insert(
            "image_config".to_string(),
            json!({ "aspect_ratio": "16:9", "image_size": "2K" }),
        );
        params.extra.insert("n".to_string(), json!(3));
        let body = build_chat_body(&params);
        assert_eq!(body["image_config"]["aspect_ratio"], "16:9");
        assert_eq!(body["image_config"]["image_size"], "2K");
        assert_eq!(body["n"], 3);
    }

    #[test]
    fn chat_body_sends_wxh_sizes_and_keeps_advanced_fields() {
        let mut params = params("gpt-image-1", "1024x1024");
        params
            .extra
            .insert("modalities".to_string(), json!(["text"]));
        params.extra.insert("seed".to_string(), json!(7));
        params.extra.insert("model".to_string(), json!("ignored"));
        let body = build_chat_body(&params);
        assert_eq!(body["size"], "1024x1024");
        assert_eq!(body["seed"], 7);
        assert_eq!(body["model"], "gpt-image-1");
        // The form's modalities win over the advanced JSON.
        assert_eq!(body["modalities"], json!(["image", "text"]));
    }

    #[test]
    fn responses_body_declares_the_image_generation_tool() {
        let mut params = params("gpt-5.4-mini", "1024x1536");
        params.quality = "high";
        params.image_urls = vec!["data:image/png;base64,AAA"];
        let body = build_responses_body(&params);
        assert_eq!(body["tools"][0]["type"], "image_generation");
        assert_eq!(body["tools"][0]["size"], "1024x1536");
        assert_eq!(body["tools"][0]["quality"], "high");
        assert_eq!(body["input"][0]["content"][0]["type"], "input_text");
        assert_eq!(body["input"][0]["content"][1]["type"], "input_image");
        assert_eq!(
            body["input"][0]["content"][1]["image_url"],
            "data:image/png;base64,AAA"
        );
    }

    #[test]
    fn responses_body_merges_user_tools_instead_of_dropping_the_form_tool() {
        let mut params = params("gpt-5.4-mini", "auto");
        params
            .extra
            .insert("tools".to_string(), json!([{ "type": "web_search" }]));
        let body = build_responses_body(&params);
        assert_eq!(body["tools"][0]["type"], "web_search");
        assert_eq!(body["tools"][1]["type"], "image_generation");
        assert_eq!(body["tools"][1]["size"], "auto");
    }

    #[test]
    fn google_body_uses_response_modalities_and_inline_images() {
        let mut params = params("gemini-3-pro-image", "16:9 2K");
        params.image_urls = vec!["https://example.com/a.png"];
        let body = build_google_body(&params, &[("image/png".to_string(), "AAA".to_string())]);
        assert_eq!(
            body["generationConfig"]["responseModalities"],
            json!(["TEXT", "IMAGE"])
        );
        assert_eq!(
            body["generationConfig"]["imageConfig"]["aspectRatio"],
            "16:9"
        );
        assert_eq!(body["generationConfig"]["imageConfig"]["imageSize"], "2K");
        assert_eq!(body["contents"][0]["parts"][0]["text"], "a cat");
        assert_eq!(body["contents"][0]["parts"][1]["inlineData"]["data"], "AAA");
    }

    #[test]
    fn google_body_keeps_user_generation_config_keys() {
        let mut base = params("gemini-3-pro-image", "");
        base.extra.insert(
            "generationConfig".to_string(),
            json!({ "seed": 3, "responseModalities": ["TEXT"] }),
        );
        let body = build_google_body(&base, &[]);
        assert_eq!(body["generationConfig"]["seed"], 3);
        assert_eq!(
            body["generationConfig"]["responseModalities"],
            json!(["TEXT", "IMAGE"])
        );
        // Additional user keys survive without replacing the image options.
        let mut sized_params = params("gemini-3-pro-image", "16:9");
        sized_params.extra.insert(
            "generationConfig".to_string(),
            json!({ "seed": 3, "imageConfig": { "imageSize": "4K" } }),
        );
        let body = build_google_body(&sized_params, &[]);
        assert_eq!(body["generationConfig"]["seed"], 3);
        assert_eq!(
            body["generationConfig"]["imageConfig"]["aspectRatio"],
            "16:9"
        );
        // The form left quality empty, so the advanced imageSize survives.
        assert_eq!(body["generationConfig"]["imageConfig"]["imageSize"], "4K");
    }

    #[test]
    fn imagen_body_maps_count_and_aspect_ratio() {
        let mut params = params("imagen-4.0-generate-001", "3:2");
        params.count = 9;
        let body = build_imagen_body(&params);
        assert_eq!(body["instances"][0]["prompt"], "a cat");
        assert_eq!(body["parameters"]["sampleCount"], 4);
        assert_eq!(body["parameters"]["aspectRatio"], "3:2");
        assert_eq!(body["parameters"]["personGeneration"], "allow_adult");
    }

    #[test]
    fn chat_parser_reads_images_and_markdown_content() {
        let value = json!({
            "choices": [{
                "message": {
                    "role": "assistant",
                    "content": "Here you go:\n![image](data:image/png;base64,QUJD)",
                    "images": [{
                        "type": "image_url",
                        "image_url": { "url": "https://cdn.example/result.png" }
                    }]
                }
            }]
        });
        let parsed = parse_chat_response(&value);
        assert_eq!(parsed.images.len(), 2);
        assert!(
            matches!(&parsed.images[0], RawImage::Data { mime_type, .. } if mime_type == "image/png")
        );
        assert_eq!(
            parsed.images[1],
            RawImage::Url("https://cdn.example/result.png".to_string())
        );
        assert!(parsed.revised_prompt.is_none());
        assert!(parsed.response_text.unwrap().contains("Here you go"));
    }

    #[test]
    fn chat_parser_accepts_content_parts_and_bare_base64() {
        let value = json!({
            "choices": [{
                "message": {
                    "content": [
                        { "type": "text", "text": "done" },
                        { "type": "image_url", "image_url": { "url": "data:image/jpeg;base64,QUJD" } }
                    ],
                    "images": ["QUJD"]
                }
            }]
        });
        let parsed = parse_chat_response(&value);
        assert_eq!(parsed.images.len(), 2);
        assert_eq!(parsed.response_text.as_deref(), Some("done"));
        assert!(
            matches!(&parsed.images[0], RawImage::Data { mime_type, .. } if mime_type == "image/jpeg")
        );
        assert!(
            matches!(&parsed.images[1], RawImage::Data { mime_type, .. } if mime_type.is_empty())
        );
    }

    #[test]
    fn chat_parser_falls_back_to_the_images_api_shape() {
        let value = json!({
            "data": [
                { "b64_json": "QUJD" },
                { "url": "https://cdn.example/b.png" }
            ]
        });
        assert_eq!(parse_chat_response(&value).images.len(), 2);
    }

    #[test]
    fn responses_parser_reads_image_generation_calls() {
        let value = json!({
            "status": "completed",
            "output": [
                { "type": "message", "content": [{ "type": "output_text", "text": "Here it is" }] },
                {
                    "type": "image_generation_call",
                    "status": "completed",
                    "result": "QUJD",
                    "revised_prompt": "a cat on a sofa"
                }
            ]
        });
        let parsed = parse_responses_response(&value).unwrap();
        assert_eq!(parsed.images.len(), 1);
        assert_eq!(parsed.revised_prompt.as_deref(), Some("a cat on a sofa"));
        assert_eq!(parsed.response_text.as_deref(), Some("Here it is"));
    }

    #[test]
    fn responses_parser_reports_failed_calls() {
        let value = json!({
            "status": "failed",
            "error": { "message": "content policy" },
            "output": []
        });
        let error = parse_responses_response(&value).unwrap_err();
        assert!(error.contains("content policy"));
    }

    #[test]
    fn google_parser_reads_inline_data_and_predictions() {
        let value = json!({
            "candidates": [{
                "content": {
                    "parts": [
                        { "text": "Here is the image" },
                        { "inlineData": { "mimeType": "image/png", "data": "QUJD" } }
                    ]
                }
            }]
        });
        let parsed = parse_google_response(&value).unwrap();
        assert_eq!(parsed.images.len(), 1);
        assert_eq!(parsed.response_text.as_deref(), Some("Here is the image"));

        let imagen = json!({
            "predictions": [
                { "bytesBase64Encoded": "QUJD", "mimeType": "image/jpeg" }
            ]
        });
        let parsed = parse_google_response(&imagen).unwrap();
        assert_eq!(
            parsed.images,
            vec![RawImage::Data {
                mime_type: "image/jpeg".to_string(),
                base64: "QUJD".to_string()
            }]
        );

        // A prediction without a MIME type keeps it empty so the bytes are
        // sniffed when the file is written.
        let untyped = json!({ "predictions": [{ "bytesBase64Encoded": "QUJD" }] });
        let parsed = parse_google_response(&untyped).unwrap();
        assert!(
            matches!(&parsed.images[0], RawImage::Data { mime_type, .. } if mime_type.is_empty())
        );
    }

    #[test]
    fn responses_parser_keeps_images_when_one_call_fails() {
        let value = json!({
            "status": "completed",
            "output": [
                { "type": "image_generation_call", "status": "failed", "error": "boom" },
                { "type": "image_generation_call", "status": "completed", "result": "QUJD" }
            ]
        });
        let parsed = parse_responses_response(&value).unwrap();
        assert_eq!(parsed.images.len(), 1);

        let failed = json!({
            "status": "completed",
            "output": [
                { "type": "image_generation_call", "status": "failed", "error": "boom" }
            ]
        });
        assert_eq!(parse_responses_response(&failed).unwrap_err(), "boom");
    }

    #[test]
    fn google_parser_keeps_images_when_another_prediction_is_filtered() {
        let value = json!({
            "predictions": [
                { "raiFilteredReason": "derogatory" },
                { "bytesBase64Encoded": "QUJD", "mimeType": "image/png" }
            ]
        });
        let parsed = parse_google_response(&value).unwrap();
        assert_eq!(parsed.images.len(), 1);

        let filtered = json!({
            "predictions": [{ "raiFilteredReason": "derogatory" }]
        });
        let error = parse_google_response(&filtered).unwrap_err();
        assert!(error.contains("derogatory"));
    }

    #[test]
    fn google_parser_reports_blocked_prompts_and_filters() {
        let blocked = json!({
            "promptFeedback": { "blockReason": "SAFETY" },
            "candidates": []
        });
        let error = parse_google_response(&blocked).unwrap_err();
        assert!(error.contains("SAFETY"));

        let filtered = json!({
            "predictions": [{ "raiFilteredReason": "derogatory" }]
        });
        let error = parse_google_response(&filtered).unwrap_err();
        assert!(error.contains("derogatory"));
    }

    #[test]
    fn markdown_targets_ignore_unbalanced_or_plain_text() {
        assert_eq!(
            markdown_image_targets(
                "a ![x](https://a.example/1.png) b ![y](<data:image/png;base64,AAA>)"
            ),
            vec![
                "https://a.example/1.png".to_string(),
                "data:image/png;base64,AAA".to_string()
            ]
        );
        assert!(markdown_image_targets("no images here ![broken").is_empty());
    }

    #[test]
    fn data_urls_require_base64_payloads() {
        assert_eq!(
            parse_data_url("data:image/png;base64,QUJD"),
            Some(("image/png".to_string(), "QUJD".to_string()))
        );
        assert_eq!(
            parse_data_url("data:;base64,QUJD"),
            Some(("image/png".to_string(), "QUJD".to_string()))
        );
        assert!(parse_data_url("data:image/svg+xml,<svg/>").is_none());
        assert!(parse_data_url("https://example.com/a.png").is_none());
    }

    #[test]
    fn mime_sniffing_and_extensions_cover_common_formats() {
        assert_eq!(sniff_mime(&[0x89, b'P', b'N', b'G', 0x0D]), "image/png");
        assert_eq!(sniff_mime(&[0xFF, 0xD8, 0xFF, 0x00]), "image/jpeg");
        assert_eq!(sniff_image_mime(b"<html>nope</html>"), None);
        assert_eq!(extension_for_mime("image/jpeg"), "jpg");
        assert_eq!(extension_for_mime("image/webp"), "webp");
    }

    #[test]
    fn reference_images_accept_data_urls_only_with_base64() {
        // Data URLs are handled synchronously; anything else needs a network.
        let data_url = "data:image/png;base64,QUJD";
        assert_eq!(
            parse_data_url(data_url),
            Some(("image/png".to_string(), "QUJD".to_string()))
        );
    }

    #[test]
    fn task_ids_are_confined_to_safe_characters() {
        assert!(validate_task_id("img_abc-123.png").is_ok());
        assert!(validate_task_id("../etc/passwd").is_err());
        assert!(validate_task_id("").is_err());
    }

    #[test]
    fn persistence_round_trips_models_and_hides_api_keys() {
        let home = tempfile::tempdir().unwrap();
        let model = ImageModel {
            id: "im_1".to_string(),
            name: "Nano Banana".to_string(),
            model_id: "gemini-3-pro-image".to_string(),
            source: ImageSource::Custom {
                base_url: "https://gw.example".to_string(),
                api_key: "sk-secret".to_string(),
            },
            protocol: ImageProtocol::Google,
            sizes: vec!["16:9".to_string()],
            qualities: vec!["2K".to_string()],
            supports_image_input: true,
        };
        save_image_models_for_home(home.path(), &[model]).unwrap();

        let raw = fs::read_to_string(image_models_path_for_home(home.path())).unwrap();
        assert!(!raw.contains("sk-secret"));
        let loaded = load_image_models_for_home(home.path()).unwrap();
        assert_eq!(loaded.len(), 1);
        match &loaded[0].source {
            ImageSource::Custom { api_key, .. } => assert_eq!(api_key, "sk-secret"),
            _ => panic!("expected a custom source"),
        }
        let secrets = fs::read_to_string(image_secrets_path_for_home(home.path())).unwrap();
        assert!(secrets.contains("sk-secret"));
    }

    #[test]
    fn persistence_round_trips_tasks_and_drops_bad_ids() {
        let home = tempfile::tempdir().unwrap();
        let model = ImageModel {
            id: "im_1".to_string(),
            name: "Nano Banana".to_string(),
            model_id: "gemini-3-pro-image".to_string(),
            source: ImageSource::Channel {
                channel_id: "ch_1".to_string(),
                token_id: None,
            },
            protocol: ImageProtocol::Google,
            sizes: Vec::new(),
            qualities: Vec::new(),
            supports_image_input: true,
        };
        let task = ImageTaskRecord {
            id: "img_1".to_string(),
            model_id: model.id.clone(),
            model_name: model.name.clone(),
            model,
            params: ImageTaskParams {
                prompt: "a cat".to_string(),
                size: "16:9".to_string(),
                quality: "2K".to_string(),
                count: 1,
                image_url: String::new(),
                extra_json: String::new(),
            },
            status: ImageTaskStatus::Completed,
            images: vec![ImageAsset {
                local_path: Some("/tmp/a.png".to_string()),
                url: None,
                mime_type: "image/png".to_string(),
            }],
            revised_prompt: None,
            response_text: None,
            created_at: 1,
            completed_at: Some(2),
            error: None,
        };
        let mut corrupt = task.clone();
        corrupt.id = "bad/id".to_string();
        save_image_tasks_for_home(home.path(), &[task, corrupt]).unwrap();

        let loaded = load_image_tasks_for_home(home.path()).unwrap();
        assert_eq!(loaded.len(), 1);
        assert_eq!(loaded[0].id, "img_1");
    }
}
