//! Dsh (DeepSeek Harness) session history, read directly from the on-disk
//! JSONL format (versions 0–4, optionally Zstandard-compressed).
//!
//! Dsh stores one directory per session under `<dsh home>/sessions`:
//!
//! ```text
//! ~/.dsh/sessions/
//!   --<normalized-cwd>--/          # readable project directory (or _no-cwd/)
//!     <session-id>/                # session-owned directory
//!       session.jsonl.zstd         # released v0, compressed
//!       session.v1.jsonl.zstd      # released v1, compressed
//!       session.v2.jsonl.zstd      # ...
//!       session.v3.jsonl.zstd
//!       session.v4.jsonl.zstd      # current version, compressed
//!       session.jsonl              # same generations, uncompressed
//! ```
//!
//! The numeric generation is the format version; the newest generation wins.
//! Compressed generations are a concatenation of independent Zstandard frames
//! (a header frame followed by one frame per durable append batch), so the
//! decoder streams all frames instead of assuming a single one.
//!
//! Parsing relies on the stable event envelope types only:
//!
//! - `session` — header: id / cwd / createdAt / version
//! - `session/title` — latest accepted title (`data.title`)
//! - `request/header` — model route (`data.header.config.provider/model`)
//! - `user/message` — a message; only `data.source.kind == "user"` is a real
//!   human turn (the runtime also injects instructions / context this way)
//! - `assistant/message` — `data.message.content` blocks (`reasoning`, `text`,
//!   `tool-call`) plus per-request `data.usage`
//!
//! Title, model and token totals are also mirrored into Dsh's projection cache
//! (`<dsh home>/storages/session_projcache`), which lets the list view avoid
//! decompressing every log. The log remains the fallback (and source of truth).

use crate::sessions::ContentBlock;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use specta::Type;
use std::fs;
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};

/// How many decompressed bytes of a log to scan for header / title / model.
const HEAD_SCAN_BYTES: u64 = 1024 * 1024;
/// Display title fallback when no title and no human message exist.
const UNTITLED: &str = "New Session";
/// Display model fallback.
const UNKNOWN_MODEL: &str = "unknown";

#[derive(Debug, Clone, Default, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct DshTokenUsage {
    pub input_tokens: f64,
    pub output_tokens: f64,
    pub cache_read_tokens: f64,
    pub cache_creation_tokens: f64,
    pub total_tokens: f64,
}

impl DshTokenUsage {
    fn add_usage(&mut self, usage: &Value) {
        let input = number(usage, "inputTokens").or_else(|| number(usage, "uncachedInputTokens"));
        self.input_tokens += input.unwrap_or(0.0);
        self.output_tokens += number(usage, "outputTokens").unwrap_or(0.0);
        self.cache_read_tokens += number(usage, "cacheReadTokens").unwrap_or(0.0);
        self.cache_creation_tokens += number(usage, "cacheWriteTokens").unwrap_or(0.0);
        self.total_tokens = self.input_tokens
            + self.output_tokens
            + self.cache_read_tokens
            + self.cache_creation_tokens;
    }

    fn from_projcache_totals(totals: &Value) -> Self {
        let input = number(totals, "uncachedInputTokens")
            .or_else(|| number(totals, "inputTokens"))
            .unwrap_or(0.0);
        let output = number(totals, "outputTokens").unwrap_or(0.0);
        let cache_read = number(totals, "cacheReadTokens").unwrap_or(0.0);
        let cache_creation = number(totals, "cacheWriteTokens").unwrap_or(0.0);
        Self {
            input_tokens: input,
            output_tokens: output,
            cache_read_tokens: cache_read,
            cache_creation_tokens: cache_creation,
            total_tokens: input + output + cache_read + cache_creation,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct DshSessionSummary {
    pub id: String,
    pub title: String,
    /// The header's cwd, not the lossy directory encoding.
    pub project: String,
    pub model: String,
    pub model_provider: String,
    pub modified_at: f64,
    pub token_usage: DshTokenUsage,
    pub path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct DshSessionMessage {
    pub id: String,
    pub role: String,
    pub content: Vec<ContentBlock>,
    pub timestamp: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct DshSessionDetail {
    pub summary: DshSessionSummary,
    pub messages: Vec<DshSessionMessage>,
}

// ============================================================================
// Path helpers
// ============================================================================

/// `<dsh home>/sessions`, e.g. `~/.dsh/sessions`.
pub fn dsh_sessions_dir_for_home(home_dir: &Path) -> PathBuf {
    crate::dsh::dsh_config_dir_for_home(home_dir).join("sessions")
}

/// `<dsh home>/sessions` for the current user.
pub fn dsh_sessions_dir() -> Result<PathBuf, String> {
    Ok(dsh_sessions_dir_for_home(&system_home_dir()?))
}

/// `<dsh home>/storages`, e.g. `~/.dsh/storages`.
fn dsh_storages_dir_for_home(home_dir: &Path) -> PathBuf {
    crate::dsh::dsh_config_dir_for_home(home_dir).join("storages")
}

fn system_home_dir() -> Result<PathBuf, String> {
    dirs::home_dir().ok_or_else(|| "Failed to get home directory".to_string())
}

fn modified_at_ms(path: &Path) -> f64 {
    fs::metadata(path)
        .ok()
        .and_then(|m| m.modified().ok())
        .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
        .map(|d| d.as_millis() as f64)
        .unwrap_or(0.0)
}

// ============================================================================
// Log discovery / decoding
// ============================================================================

/// Parse a canonical session log filename into `(format version, compressed)`.
///
/// Canonical names are `session.jsonl[.zstd]` for v0 and
/// `session.v<N>.jsonl[.zstd]` for later versions.
fn parse_log_name(name: &str) -> Option<(u32, bool)> {
    let (stem, compressed) = match name.strip_suffix(".zstd") {
        Some(stem) => (stem, true),
        None => (name, false),
    };
    let stem = stem.strip_suffix(".jsonl")?;
    if stem == "session" {
        return Some((0, compressed));
    }
    let version = stem.strip_prefix("session.v")?.parse::<u32>().ok()?;
    Some((version, compressed))
}

/// Select the newest canonical generation in a session directory.
fn newest_log_in_session_dir(dir: &Path) -> Option<PathBuf> {
    let entries = fs::read_dir(dir).ok()?;
    let mut best: Option<(u32, bool, PathBuf)> = None;
    for entry in entries.flatten() {
        let path = entry.path();
        if !path.is_file() {
            continue;
        }
        let Some(name) = path.file_name().and_then(|s| s.to_str()) else {
            continue;
        };
        let Some((version, compressed)) = parse_log_name(name) else {
            continue;
        };
        let replace = match &best {
            None => true,
            Some((best_version, best_compressed, _)) => {
                version > *best_version
                    || (version == *best_version && compressed && !*best_compressed)
            }
        };
        if replace {
            best = Some((version, compressed, path));
        }
    }
    best.map(|(_, _, path)| path)
}

/// Open a session log as a buffered line reader, transparently decompressing
/// Zstandard generations.
fn open_log_reader(path: &Path) -> Result<Box<dyn BufRead>, String> {
    let file = fs::File::open(path).map_err(|e| format!("Failed to open Dsh session: {e}"))?;
    if path.extension().and_then(|s| s.to_str()) == Some("zstd") {
        let decoder =
            zstd::stream::read::Decoder::new(file).map_err(|e| format!("zstd error: {e}"))?;
        Ok(Box::new(BufReader::new(decoder)))
    } else {
        Ok(Box::new(BufReader::new(file)))
    }
}

fn number(value: &Value, key: &str) -> Option<f64> {
    value.get(key).and_then(Value::as_f64)
}

fn non_empty_str<'a>(value: &'a Value, key: &str) -> Option<&'a str> {
    value
        .get(key)
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|s| !s.is_empty())
}

fn timestamp_from_ms(ms: Option<i64>) -> String {
    ms.and_then(chrono::DateTime::<chrono::Utc>::from_timestamp_millis)
        .map(|dt| dt.to_rfc3339_opts(chrono::SecondsFormat::Secs, true))
        .unwrap_or_default()
}

// ============================================================================
// Projection-cache metadata (optional fast path for the list view)
// ============================================================================

#[derive(Debug, Default)]
struct ProjcacheEntry {
    title: Option<String>,
    first_prompt: Option<String>,
    cwd: Option<String>,
    model: Option<String>,
    model_provider: Option<String>,
    token_usage: DshTokenUsage,
    has_usage: bool,
}

fn row_value<'a>(rows: &'a Value, key: &str) -> Option<&'a Value> {
    rows.get(key)?.get("val")
}

fn projcache_entry_from_record(record: &Value) -> Option<ProjcacheEntry> {
    let rows = record.get("rows")?;
    if !rows.is_object() {
        return None;
    }
    let identity = record.get("identity");
    let title = row_value(rows, "title")
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(str::to_string);
    let first_prompt = row_value(rows, "titleInput")
        .and_then(|input| input.get("first"))
        .and_then(|first| non_empty_str(first, "text").map(str::to_string));
    let cwd = identity
        .and_then(|identity| non_empty_str(identity, "cwd"))
        .map(str::to_string);
    let last_used = row_value(rows, "modelSelection").and_then(|value| value.get("lastUsed"));
    let model = last_used
        .and_then(|last_used| non_empty_str(last_used, "model"))
        .map(str::to_string);
    let model_provider = last_used
        .and_then(|last_used| non_empty_str(last_used, "provider"))
        .map(str::to_string);
    let totals = row_value(rows, "tokenUsage").and_then(|usage| usage.get("totals"));
    let (token_usage, has_usage) = match totals {
        Some(totals) => (DshTokenUsage::from_projcache_totals(totals), true),
        None => (DshTokenUsage::default(), false),
    };
    Some(ProjcacheEntry {
        title,
        first_prompt,
        cwd,
        model,
        model_provider,
        token_usage,
        has_usage,
    })
}

fn read_projcache_entry(home_dir: &Path, session_id: &str) -> Option<ProjcacheEntry> {
    let storages_dir = dsh_storages_dir_for_home(home_dir).join("session_projcache");
    let per_session = storages_dir
        .join("sessions")
        .join(format!("{session_id}.json"));
    if let Ok(content) = fs::read_to_string(&per_session) {
        if let Ok(json) = serde_json::from_str::<Value>(&content) {
            let record = json.get("record").unwrap_or(&json);
            if let Some(entry) = projcache_entry_from_record(record) {
                return Some(entry);
            }
        }
    }

    // Older layouts keep every session in one table instead of per-session
    // files; only consult it when the per-session file is missing.
    let content = fs::read_to_string(storages_dir.join("session_projcache.json")).ok()?;
    let json = serde_json::from_str::<Value>(&content).ok()?;
    let record = json.get("tables")?.get("sessions")?.get(session_id)?;
    projcache_entry_from_record(record)
}

// ============================================================================
// Log scanning
// ============================================================================

#[derive(Debug, Default)]
struct LogHead {
    id: Option<String>,
    cwd: Option<String>,
    created_at: Option<i64>,
    title: Option<String>,
    model: Option<String>,
    model_provider: Option<String>,
    first_prompt: Option<String>,
}

impl LogHead {
    fn apply(&mut self, json: &Value) {
        match json.get("type").and_then(Value::as_str) {
            Some("session") => {
                if let Some(id) = non_empty_str(json, "id") {
                    self.id = Some(id.to_string());
                }
                if let Some(cwd) = non_empty_str(json, "cwd") {
                    self.cwd = Some(cwd.to_string());
                }
                if self.created_at.is_none() {
                    self.created_at = json.get("createdAt").and_then(Value::as_i64);
                }
            }
            Some("session/title") => {
                if let Some(title) = json.pointer("/data/title").and_then(Value::as_str) {
                    let title = title.trim();
                    if !title.is_empty() {
                        self.title = Some(title.to_string());
                    }
                }
            }
            Some("request/header") => {
                if let Some(config) = json.pointer("/data/header/config") {
                    if let Some(model) = non_empty_str(config, "model") {
                        self.model = Some(model.to_string());
                    }
                    if let Some(provider) = non_empty_str(config, "provider") {
                        self.model_provider = Some(provider.to_string());
                    }
                }
            }
            Some("user/message") => {
                if json.pointer("/data/source/kind").and_then(Value::as_str) != Some("user") {
                    return;
                }
                if self.first_prompt.is_none() {
                    self.first_prompt = first_text_of_content(json.pointer("/data/content"));
                }
            }
            _ => {}
        }
    }
}

fn first_text_of_content(content: Option<&Value>) -> Option<String> {
    let blocks = content?.as_array()?;
    for block in blocks {
        if block.get("type").and_then(Value::as_str) == Some("text") {
            if let Some(text) = block.get("text").and_then(Value::as_str) {
                let text = text.trim();
                if !text.is_empty() {
                    return Some(text.to_string());
                }
            }
        }
    }
    None
}

/// Scan the head of a log (bounded by [`HEAD_SCAN_BYTES`] of decoded output).
fn scan_log_head(path: &Path) -> Result<LogHead, String> {
    let mut reader = open_log_reader(path)?;
    let limited = std::io::Read::take(reader.as_mut(), HEAD_SCAN_BYTES);
    let mut head = LogHead::default();
    for line in limited.lines() {
        let Ok(line) = line else {
            // A torn trailing frame only loses the suffix; keep what we have.
            break;
        };
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        let Ok(json) = serde_json::from_str::<Value>(line) else {
            continue;
        };
        head.apply(&json);
    }
    if head.id.is_none() {
        return Err("Dsh session header not found".to_string());
    }
    Ok(head)
}

fn title_from_text(text: &str) -> String {
    let mut title = text.split_whitespace().collect::<Vec<_>>().join(" ");
    if title.chars().count() > 80 {
        if let Some((end, _)) = title.char_indices().nth(80) {
            title.truncate(end);
            title.push('…');
        }
    }
    title
}

fn display_title(title: Option<&str>, first_prompt: Option<&str>) -> String {
    if let Some(title) = title.map(str::trim).filter(|t| !t.is_empty()) {
        return title.to_string();
    }
    if let Some(prompt) = first_prompt.map(str::trim).filter(|p| !p.is_empty()) {
        let derived = title_from_text(prompt);
        if !derived.is_empty() {
            return derived;
        }
    }
    UNTITLED.to_string()
}

// ============================================================================
// Listing
// ============================================================================

fn build_summary(home_dir: &Path, log_path: &Path) -> Result<DshSessionSummary, String> {
    let head = scan_log_head(log_path)?;
    let session_dir = log_path
        .parent()
        .ok_or_else(|| "Dsh session log has no parent directory".to_string())?;
    let id = head
        .id
        .clone()
        .or_else(|| {
            session_dir
                .file_name()
                .and_then(|s| s.to_str())
                .map(str::to_string)
        })
        .unwrap_or_default();

    let cache = read_projcache_entry(home_dir, &id);

    let title = display_title(
        cache
            .as_ref()
            .and_then(|entry| entry.title.as_deref())
            .or(head.title.as_deref()),
        cache
            .as_ref()
            .and_then(|entry| entry.first_prompt.as_deref())
            .or(head.first_prompt.as_deref()),
    );

    let project = cache
        .as_ref()
        .and_then(|entry| entry.cwd.clone())
        .or_else(|| head.cwd.clone())
        .unwrap_or_default();

    let model = cache
        .as_ref()
        .and_then(|entry| entry.model.clone())
        .or_else(|| head.model.clone())
        .unwrap_or_else(|| UNKNOWN_MODEL.to_string());

    let model_provider = cache
        .as_ref()
        .and_then(|entry| entry.model_provider.clone())
        .or_else(|| head.model_provider.clone())
        .unwrap_or_default();

    let token_usage = cache
        .as_ref()
        .filter(|entry| entry.has_usage)
        .map(|entry| entry.token_usage.clone())
        .unwrap_or_default();

    Ok(DshSessionSummary {
        id,
        title,
        project,
        model,
        model_provider,
        modified_at: modified_at_ms(log_path),
        token_usage,
        path: log_path.to_string_lossy().to_string(),
    })
}

pub fn list_dsh_sessions_for_home(home_dir: &Path) -> Result<Vec<DshSessionSummary>, String> {
    let sessions_dir = dsh_sessions_dir_for_home(home_dir);
    if !sessions_dir.exists() {
        return Ok(Vec::new());
    }

    let mut sessions: Vec<DshSessionSummary> = Vec::new();
    let project_dirs = fs::read_dir(&sessions_dir)
        .map_err(|e| format!("Failed to read Dsh sessions directory: {e}"))?;
    for project_dir in project_dirs.flatten() {
        let project_path = project_dir.path();
        if !project_path.is_dir() {
            continue;
        }
        let Ok(session_dirs) = fs::read_dir(&project_path) else {
            continue;
        };
        for session_dir in session_dirs.flatten() {
            let session_path = session_dir.path();
            if !session_path.is_dir() {
                continue;
            }
            let Some(log_path) = newest_log_in_session_dir(&session_path) else {
                continue;
            };
            match build_summary(home_dir, &log_path) {
                Ok(summary) => sessions.push(summary),
                Err(error) => log::warn!(
                    "Skipping unreadable Dsh session {}: {error}",
                    log_path.display()
                ),
            }
        }
    }

    sessions.sort_by(|a, b| {
        b.modified_at
            .partial_cmp(&a.modified_at)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    Ok(sessions)
}

pub fn list_dsh_sessions() -> Result<Vec<DshSessionSummary>, String> {
    list_dsh_sessions_for_home(&system_home_dir()?)
}

// ============================================================================
// Detail
// ============================================================================

fn content_blocks(content: Option<&Value>, role: &str) -> Vec<ContentBlock> {
    let Some(blocks) = content.and_then(Value::as_array) else {
        return Vec::new();
    };
    let mut out = Vec::new();
    for block in blocks {
        let block_type = block.get("type").and_then(Value::as_str).unwrap_or("");
        match block_type {
            "text" => {
                if let Some(text) = block.get("text").and_then(Value::as_str) {
                    let text = text.trim();
                    if !text.is_empty() {
                        out.push(ContentBlock {
                            content_type: "text".to_string(),
                            text: Some(text.to_string()),
                            thinking: None,
                        });
                    }
                }
            }
            "reasoning" if role == "assistant" => {
                if let Some(text) = block.get("text").and_then(Value::as_str) {
                    let text = text.trim();
                    if !text.is_empty() {
                        out.push(ContentBlock {
                            content_type: "thinking".to_string(),
                            text: None,
                            thinking: Some(text.to_string()),
                        });
                    }
                }
            }
            // Tool calls / results are not rendered in the transcript; they
            // stay raw in the log (same display contract as Codex/Droid/Pi).
            _ => {}
        }
    }
    out
}

pub fn get_dsh_session_detail_for_home(
    _home_dir: &Path,
    session_path: &str,
) -> Result<DshSessionDetail, String> {
    let log_path = PathBuf::from(session_path);
    if !log_path.is_file() {
        return Err("Dsh session file not found".to_string());
    }

    let modified_at = modified_at_ms(&log_path);
    let reader = open_log_reader(&log_path)?;

    let mut id = log_path
        .parent()
        .and_then(Path::file_name)
        .and_then(|s| s.to_str())
        .unwrap_or("")
        .to_string();
    let mut cwd = String::new();
    let mut title: Option<String> = None;
    let mut first_prompt: Option<String> = None;
    let mut model: Option<String> = None;
    let mut model_provider: Option<String> = None;
    let mut token_usage = DshTokenUsage::default();
    let mut messages: Vec<DshSessionMessage> = Vec::new();
    let mut message_counter: usize = 0;

    for line in reader.lines() {
        let Ok(line) = line else {
            // Torn final frame: keep the complete decoded prefix.
            break;
        };
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        let Ok(json) = serde_json::from_str::<Value>(line) else {
            continue;
        };

        match json.get("type").and_then(Value::as_str) {
            Some("session") => {
                if let Some(header_id) = non_empty_str(&json, "id") {
                    id = header_id.to_string();
                }
                if let Some(header_cwd) = non_empty_str(&json, "cwd") {
                    cwd = header_cwd.to_string();
                }
            }
            Some("session/title") => {
                if let Some(next) = json.pointer("/data/title").and_then(Value::as_str) {
                    let next = next.trim();
                    if !next.is_empty() {
                        title = Some(next.to_string());
                    }
                }
            }
            Some("request/header") => {
                if let Some(config) = json.pointer("/data/header/config") {
                    if let Some(next) = non_empty_str(config, "model") {
                        model = Some(next.to_string());
                    }
                    if let Some(next) = non_empty_str(config, "provider") {
                        model_provider = Some(next.to_string());
                    }
                }
            }
            Some("user/message") => {
                if json.pointer("/data/source/kind").and_then(Value::as_str) != Some("user") {
                    continue;
                }
                let blocks = content_blocks(json.pointer("/data/content"), "user");
                if blocks.is_empty() {
                    continue;
                }
                if first_prompt.is_none() {
                    first_prompt = blocks
                        .iter()
                        .find(|block| block.content_type == "text")
                        .and_then(|block| block.text.clone());
                }
                message_counter += 1;
                let message_id = non_empty_str(&json, "id")
                    .map(str::to_string)
                    .unwrap_or_else(|| format!("user-{message_counter}"));
                messages.push(DshSessionMessage {
                    id: message_id,
                    role: "user".to_string(),
                    content: blocks,
                    timestamp: timestamp_from_ms(json.get("time").and_then(Value::as_i64)),
                });
            }
            Some("assistant/message") => {
                let data = json.get("data").unwrap_or(&Value::Null);
                if let Some(usage) = data.get("usage") {
                    token_usage.add_usage(usage);
                }
                let blocks = content_blocks(data.pointer("/message/content"), "assistant");
                if blocks.is_empty() {
                    continue;
                }
                message_counter += 1;
                let message_id = data
                    .pointer("/message/id")
                    .and_then(Value::as_str)
                    .map(str::to_string)
                    .unwrap_or_else(|| format!("assistant-{message_counter}"));
                messages.push(DshSessionMessage {
                    id: message_id,
                    role: "assistant".to_string(),
                    content: blocks,
                    timestamp: timestamp_from_ms(json.get("time").and_then(Value::as_i64)),
                });
            }
            _ => {}
        }
    }

    let summary = DshSessionSummary {
        id,
        title: display_title(title.as_deref(), first_prompt.as_deref()),
        project: cwd,
        model: model.unwrap_or_else(|| UNKNOWN_MODEL.to_string()),
        model_provider: model_provider.unwrap_or_default(),
        modified_at,
        token_usage,
        path: log_path.to_string_lossy().to_string(),
    };

    Ok(DshSessionDetail { summary, messages })
}

pub fn get_dsh_session_detail(session_path: &str) -> Result<DshSessionDetail, String> {
    get_dsh_session_detail_for_home(&system_home_dir()?, session_path)
}

// ============================================================================
// Delete
// ============================================================================

pub fn delete_dsh_session_for_home(home_dir: &Path, session_path: &str) -> Result<(), String> {
    let sessions_dir = dsh_sessions_dir_for_home(home_dir);
    let log_path = PathBuf::from(session_path);

    // Safety guard: only delete logs inside the Dsh sessions directory.
    if !log_path.starts_with(&sessions_dir) || !log_path.is_file() {
        return Err("Dsh session file not found".to_string());
    }

    let session_dir = log_path
        .parent()
        .ok_or_else(|| "Dsh session file not found".to_string())?;
    let project_dir = session_dir
        .parent()
        .ok_or_else(|| "Dsh session file not found".to_string())?;
    if project_dir.parent() != Some(sessions_dir.as_path()) {
        return Err("Dsh session file not found".to_string());
    }

    fs::remove_dir_all(session_dir).map_err(|e| format!("Failed to delete Dsh session: {e}"))?;

    // Prune the project directory when it becomes empty.
    if fs::read_dir(project_dir)
        .map(|mut entries| entries.next().is_none())
        .unwrap_or(false)
    {
        let _ = fs::remove_dir(project_dir);
    }

    Ok(())
}

pub fn delete_dsh_session(session_path: &str) -> Result<(), String> {
    delete_dsh_session_for_home(&system_home_dir()?, session_path)
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::{
        delete_dsh_session_for_home, get_dsh_session_detail_for_home, list_dsh_sessions_for_home,
    };
    use std::fs;
    use std::io::Write;
    use std::path::PathBuf;
    use tempfile::TempDir;

    struct Fixture {
        _temp: TempDir,
        home: PathBuf,
        session_dir: PathBuf,
    }

    impl Fixture {
        /// Create `<home>/.dsh/sessions/--work-repo--/session-test/session.jsonl`
        /// with a raw (uncompressed) log.
        fn raw(session_lines: &[&str]) -> Self {
            Self::with_file("session-test", "session.jsonl", |file| {
                for line in session_lines {
                    writeln!(file, "{line}").unwrap();
                }
            })
        }

        /// Create a compressed log made of concatenated Zstandard frames, like
        /// Dsh's own `session[.vN].jsonl.zstd` generation files.
        fn zstd_frames(session_lines: &[&str]) -> Self {
            Self::with_file("session-test", "session.v4.jsonl.zstd", |file| {
                for line in session_lines {
                    // One frame per record: Dsh's persistence writes complete
                    // JSONL rows (including the newline) inside each frame.
                    let record = format!("{line}\n");
                    let frame = zstd::stream::encode_all(record.as_bytes(), 0).unwrap();
                    file.write_all(&frame).unwrap();
                }
            })
        }

        fn with_file<F>(session_id: &str, file_name: &str, write: F) -> Self
        where
            F: FnOnce(&mut fs::File),
        {
            let temp = TempDir::new().unwrap();
            let home = temp.path().to_path_buf();
            let session_dir = home.join(".dsh/sessions/--work-repo--").join(session_id);
            fs::create_dir_all(&session_dir).unwrap();
            let path = session_dir.join(file_name);
            let mut file = fs::File::create(path).unwrap();
            write(&mut file);
            Self {
                _temp: temp,
                home,
                session_dir,
            }
        }

        fn log_path(&self) -> String {
            let path = self
                .session_dir
                .read_dir()
                .unwrap()
                .flatten()
                .next()
                .unwrap()
                .path();
            // The fixture builds directories with literal "/" segments in
            // `join(".dsh/sessions/...")`, while the parser stores paths with
            // native separators. Normalize so the comparison is stable on
            // every platform.
            path.to_string_lossy()
                .replace('/', &std::path::MAIN_SEPARATOR.to_string())
        }

        fn write_projcache(&self, id: &str, title: Option<&str>, first_prompt: &str) {
            let dir = self.home.join(".dsh/storages/session_projcache/sessions");
            fs::create_dir_all(&dir).unwrap();
            let title_json = match title {
                Some(title) => format!("\"{title}\""),
                None => "null".to_string(),
            };
            let content = format!(
                r#"{{"version":7,"record":{{"identity":{{"formatVersion":4,"createdAt":1700000000000,"cwd":"/work/repo"}},"rows":{{"title":{{"ver":1,"seq":1,"val":{title_json}}},"titleInput":{{"ver":3,"seq":1,"val":{{"first":{{"seq":0,"text":"{first_prompt}"}},"count":1,"lastSeq":0}}}},"modelSelection":{{"ver":2,"seq":1,"val":{{"lastUsed":{{"provider":"deepseek-official","model":"deepseek-flash"}},"pending":null}}}},"tokenUsage":{{"ver":2,"seq":1,"val":{{"totals":{{"uncachedInputTokens":100,"outputTokens":20,"cacheReadTokens":30,"cacheWriteTokens":5}},"last":null}}}}}}}}}}"#
            );
            fs::write(dir.join(format!("{id}.json")), content).unwrap();
        }
    }

    const HEADER: &str = r#"{"type":"session","version":4,"id":"session-test","createdAt":1700000000000,"cwd":"/work/repo","isSeeded":false,"delegationDepth":0,"agentPreset":"standard"}"#;
    const RUNTIME_CONTEXT: &str = r#"{"type":"user/message","seq":1,"time":1700000001000,"data":{"content":[{"type":"text","text":"Injected runtime context"}],"source":{"kind":"runtime-context"},"role":"user","id":"ctx-1"}}"#;
    const USER_REAL: &str = r#"{"type":"user/message","seq":2,"time":1700000002000,"data":{"content":[{"type":"text","text":"Fix the Dsh session parser"}],"source":{"kind":"user"},"role":"user","id":"user-1"}}"#;
    const TITLE: &str = r#"{"type":"session/title","seq":3,"time":1700000003000,"data":{"title":"Fix the Dsh session parser","messageSeqs":[2],"source":{"kind":"fallback"}}}"#;
    const REQUEST_HEADER: &str = r#"{"type":"request/header","seq":4,"time":1700000004000,"data":{"header":{"config":{"provider":"deepseek-official","model":"deepseek-flash"}}}}"#;
    const ASSISTANT: &str = r#"{"type":"assistant/message","seq":5,"time":1700000005000,"data":{"turn":1,"step":1,"message":{"role":"assistant","content":[{"type":"reasoning","text":"Let me inspect the files."},{"type":"text","text":"Done."},{"type":"tool-call","id":"call-1","name":"bash","arguments":"{}"}],"source":{"kind":"model","provider":"deepseek-official","model":"deepseek-flash"},"id":"asst-1"},"usage":{"inputTokens":100,"outputTokens":20,"cacheReadTokens":30,"cacheWriteTokens":5,"totalTokens":155}}}"#;
    const FULL_SESSION: &[&str] = &[
        HEADER,
        RUNTIME_CONTEXT,
        REQUEST_HEADER,
        USER_REAL,
        TITLE,
        ASSISTANT,
    ];

    #[test]
    fn list_reads_summary_from_raw_log() {
        let fixture = Fixture::raw(FULL_SESSION);

        let sessions = list_dsh_sessions_for_home(&fixture.home).unwrap();
        assert_eq!(sessions.len(), 1);

        let session = &sessions[0];
        assert_eq!(session.id, "session-test");
        assert_eq!(session.title, "Fix the Dsh session parser");
        assert_eq!(session.project, "/work/repo");
        assert_eq!(session.model, "deepseek-flash");
        assert_eq!(session.model_provider, "deepseek-official");
        assert_eq!(session.token_usage.input_tokens, 0.0);
        assert_eq!(session.path, fixture.log_path());
        assert!(session.modified_at > 0.0);
    }

    #[test]
    fn list_reads_projcache_title_usage_and_model() {
        let fixture = Fixture::raw(&[HEADER, USER_REAL]);

        let sessions = list_dsh_sessions_for_home(&fixture.home).unwrap();
        assert_eq!(sessions[0].title, "Fix the Dsh session parser");
        assert_eq!(sessions[0].model, "unknown");
        assert_eq!(sessions[0].model_provider, "");

        fixture.write_projcache("session-test", Some("Renamed session"), "first prompt");
        let sessions = list_dsh_sessions_for_home(&fixture.home).unwrap();
        assert_eq!(sessions[0].title, "Renamed session");
        assert_eq!(sessions[0].model, "deepseek-flash");
        assert_eq!(sessions[0].model_provider, "deepseek-official");
        assert_eq!(sessions[0].token_usage.input_tokens, 100.0);
        assert_eq!(sessions[0].token_usage.output_tokens, 20.0);
        assert_eq!(sessions[0].token_usage.cache_read_tokens, 30.0);
        assert_eq!(sessions[0].token_usage.cache_creation_tokens, 5.0);
        assert_eq!(sessions[0].token_usage.total_tokens, 155.0);
    }

    #[test]
    fn list_falls_back_to_first_human_prompt_title() {
        let fixture = Fixture::raw(&[HEADER, RUNTIME_CONTEXT, USER_REAL]);

        let sessions = list_dsh_sessions_for_home(&fixture.home).unwrap();
        assert_eq!(sessions[0].title, "Fix the Dsh session parser");
    }

    #[test]
    fn list_prefers_newest_format_generation() {
        let fixture = Fixture::raw(&[HEADER, USER_REAL, TITLE]);
        let session_dir = fixture.session_dir.clone();
        let v4 = session_dir.join("session.v4.jsonl");
        let mut file = fs::File::create(&v4).unwrap();
        writeln!(
            file,
            r#"{{"type":"session","version":4,"id":"session-test","createdAt":1700000000000,"cwd":"/work/repo"}}"#
        )
        .unwrap();
        writeln!(
            file,
            r#"{{"type":"session/title","seq":1,"time":1700000000000,"data":{{"title":"Newest generation"}}}}"#
        )
        .unwrap();

        let sessions = list_dsh_sessions_for_home(&fixture.home).unwrap();
        assert_eq!(sessions.len(), 1);
        assert_eq!(sessions[0].title, "Newest generation");
        assert!(sessions[0].path.ends_with("session.v4.jsonl"));
    }

    #[test]
    fn list_decodes_concatenated_zstd_frames() {
        let fixture = Fixture::zstd_frames(FULL_SESSION);

        let sessions = list_dsh_sessions_for_home(&fixture.home).unwrap();
        assert_eq!(sessions.len(), 1);
        assert_eq!(sessions[0].title, "Fix the Dsh session parser");
        assert_eq!(sessions[0].model, "deepseek-flash");
    }

    #[test]
    fn detail_filters_injected_messages_and_maps_reasoning() {
        let fixture = Fixture::raw(FULL_SESSION);

        let detail = get_dsh_session_detail_for_home(&fixture.home, &fixture.log_path()).unwrap();
        let summary = &detail.summary;
        assert_eq!(summary.id, "session-test");
        assert_eq!(summary.project, "/work/repo");
        assert_eq!(summary.title, "Fix the Dsh session parser");
        assert_eq!(summary.model, "deepseek-flash");
        assert_eq!(summary.model_provider, "deepseek-official");
        assert_eq!(summary.token_usage.input_tokens, 100.0);
        assert_eq!(summary.token_usage.output_tokens, 20.0);
        assert_eq!(summary.token_usage.cache_read_tokens, 30.0);
        assert_eq!(summary.token_usage.cache_creation_tokens, 5.0);

        // Injected runtime-context message is filtered out; tool-call block is
        // skipped; reasoning becomes a thinking block.
        assert_eq!(detail.messages.len(), 2);
        assert_eq!(detail.messages[0].role, "user");
        assert_eq!(
            detail.messages[0].content[0].text.as_deref(),
            Some("Fix the Dsh session parser")
        );
        assert_eq!(detail.messages[1].role, "assistant");
        assert_eq!(detail.messages[1].content.len(), 2);
        assert_eq!(detail.messages[1].content[0].content_type, "thinking");
        assert_eq!(
            detail.messages[1].content[0].thinking.as_deref(),
            Some("Let me inspect the files.")
        );
        assert_eq!(detail.messages[1].content[1].text.as_deref(), Some("Done."));
    }

    #[test]
    fn detail_decodes_concatenated_zstd_frames() {
        let fixture = Fixture::zstd_frames(FULL_SESSION);

        let detail = get_dsh_session_detail_for_home(&fixture.home, &fixture.log_path()).unwrap();
        assert_eq!(detail.messages.len(), 2);
        assert_eq!(detail.messages[1].content[0].content_type, "thinking");
    }

    #[test]
    fn detail_accumulates_usage_across_assistant_messages() {
        let second_usage = r#"{"type":"assistant/message","seq":6,"time":1700000006000,"data":{"message":{"role":"assistant","content":[{"type":"text","text":"More."}],"id":"asst-2"},"usage":{"inputTokens":10,"outputTokens":2,"cacheReadTokens":3,"cacheWriteTokens":0}}}"#;
        let fixture = Fixture::raw(&[HEADER, USER_REAL, TITLE, ASSISTANT, second_usage]);

        let detail = get_dsh_session_detail_for_home(&fixture.home, &fixture.log_path()).unwrap();
        assert_eq!(detail.messages.len(), 3);
        assert_eq!(detail.summary.token_usage.input_tokens, 110.0);
        assert_eq!(detail.summary.token_usage.output_tokens, 22.0);
        assert_eq!(detail.summary.token_usage.cache_read_tokens, 33.0);
    }

    #[test]
    fn list_is_empty_when_sessions_dir_missing() {
        let temp = TempDir::new().unwrap();
        let sessions = list_dsh_sessions_for_home(temp.path()).unwrap();
        assert!(sessions.is_empty());
    }

    #[test]
    fn delete_removes_session_dir_and_prunes_project_dir() {
        let fixture = Fixture::raw(FULL_SESSION);
        let project_dir = fixture.home.join(".dsh/sessions/--work-repo--");

        delete_dsh_session_for_home(&fixture.home, &fixture.log_path()).unwrap();

        assert!(!fixture.session_dir.exists());
        assert!(!project_dir.exists());
        assert!(fixture.home.join(".dsh/sessions").exists());
    }

    #[test]
    fn delete_rejects_paths_outside_sessions_dir() {
        let fixture = Fixture::raw(FULL_SESSION);
        let outside = fixture.home.join("settings.yaml");
        fs::write(&outside, "x").unwrap();

        let err =
            delete_dsh_session_for_home(&fixture.home, &outside.to_string_lossy()).unwrap_err();
        assert!(err.contains("not found"));
        assert!(outside.exists());
    }
}
