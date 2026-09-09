//! Codex CLI sessions management (core).
//!
//! Reads Codex session rollout files from `<codex home>/sessions/YYYY/MM/DD/rollout-*.jsonl`.
//! The Codex home defaults to `~/.codex` and honors the path overrides managed
//! by [`crate::paths`]. Session titles come from `<codex home>/session_index.jsonl`
//! (written by `codex` itself, newest entry wins) with a fallback to the first
//! real user message.
//!
//! JSONL line types understood here:
//! - `session_meta` — session id / cwd (first line of the file)
//! - `turn_context` — model in use
//! - `response_item` — `message` (user/assistant chat), `reasoning` (thinking),
//!   tool calls (skipped for display)
//! - `event_msg` — `agent_reasoning` (plain thinking stream), `token_count`
//!   (cumulative token usage)

use serde::{Deserialize, Serialize};
use serde_json::Value;
use specta::Type;
use std::collections::HashMap;
use std::fs::{self, File};
use std::io::{BufRead, BufReader, Read, Seek, SeekFrom};
use std::path::{Path, PathBuf};

use crate::paths;

/// How many bytes of the file head to scan for session meta / title / model.
const HEAD_BYTES: u64 = 256 * 1024;
/// How many bytes of the file tail to scan for the final token usage.
const TAIL_BYTES: u64 = 128 * 1024;
/// Display title fallback when neither the index nor a user message exists.
const UNTITLED: &str = "New Session";

/// Token usage statistics (from the last `event_msg/token_count` event).
#[derive(Debug, Clone, Serialize, Deserialize, Type, Default)]
#[serde(rename_all = "camelCase")]
pub struct CodexTokenUsage {
    pub input_tokens: f64,
    pub output_tokens: f64,
    pub cache_creation_tokens: f64,
    pub cache_read_tokens: f64,
    pub reasoning_tokens: f64,
    pub total_tokens: f64,
}

/// A Codex model provider aggregated from all configured profiles.
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct CodexSessionProvider {
    /// Provider id as used in config.toml (`model_provider`) and session files
    pub id: String,
    /// Display name (provider config name, or the id itself)
    pub name: String,
}

/// Session summary for the list view.
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct CodexSessionSummary {
    /// Session UUID
    pub id: String,
    /// Session title
    pub title: String,
    /// Working directory the session ran in (used as its "project")
    pub project: String,
    /// Model used
    pub model: String,
    /// Model provider owning this session (from session_meta)
    pub model_provider: String,
    /// Last modified timestamp in milliseconds
    pub modified_at: f64,
    /// Token usage
    pub token_usage: CodexTokenUsage,
    /// Full path to the session .jsonl file
    pub path: String,
}

/// Message content block
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct CodexContentBlock {
    #[serde(rename = "type")]
    pub content_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub thinking: Option<String>,
}

/// Session message
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct CodexSessionMessage {
    pub id: String,
    pub role: String,
    pub content: Vec<CodexContentBlock>,
    pub timestamp: String,
}

/// Session detail with messages
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct CodexSessionDetail {
    pub id: String,
    pub title: String,
    pub cwd: String,
    pub model: String,
    pub model_provider: String,
    pub modified_at: f64,
    pub token_usage: CodexTokenUsage,
    pub messages: Vec<CodexSessionMessage>,
}

// ============================================================================
// Path helpers
// ============================================================================

fn codex_sessions_dir_for_home(home_dir: &Path) -> Result<PathBuf, String> {
    let config_paths = paths::load_config_paths_for_home(home_dir);
    let codex_dir = paths::get_codex_home_for_home(home_dir, &config_paths)?;
    Ok(codex_dir.join("sessions"))
}

fn system_home_dir() -> Result<PathBuf, String> {
    dirs::home_dir().ok_or_else(|| "Failed to get home directory".to_string())
}

/// Collects all `*.jsonl` session files below the sessions dir (recursive).
fn collect_jsonl_files(dir: &Path, out: &mut Vec<PathBuf>) {
    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            collect_jsonl_files(&path, out);
        } else if path.extension().and_then(|s| s.to_str()) == Some("jsonl") {
            out.push(path);
        }
    }
}

/// Reads `session_index.jsonl` and returns the newest `thread_name` per id.
fn read_session_index_titles(codex_home: &Path) -> HashMap<String, String> {
    let mut titles: HashMap<String, String> = HashMap::new();
    let index_path = codex_home.join("session_index.jsonl");
    let Ok(file) = File::open(index_path) else {
        return titles;
    };
    for line in BufReader::new(file).lines() {
        let Ok(line) = line else { continue };
        let Ok(json) = serde_json::from_str::<Value>(&line) else {
            continue;
        };
        let Some(id) = json["id"].as_str() else {
            continue;
        };
        if let Some(name) = json["thread_name"].as_str() {
            let name = name.trim();
            if !name.is_empty() {
                titles.insert(id.to_string(), name.to_string());
            }
        }
    }
    titles
}

/// Reads up to `max_bytes` from the start of the file, keeping whole lines.
/// When the read is truncated at the byte limit, the trailing partial line
/// is dropped. Codex session files may contain raw tool output that is not
/// valid UTF-8, so decoding is lossy and never fails on the content.
fn read_prefix_lines(path: &Path, max_bytes: u64) -> Result<Vec<String>, String> {
    let file = File::open(path).map_err(|e| format!("Failed to open {path:?}: {e}"))?;
    let mut buf = Vec::new();
    file.take(max_bytes)
        .read_to_end(&mut buf)
        .map_err(|e| format!("Failed to read {path:?}: {e}"))?;
    if buf.len() == max_bytes as usize {
        if let Some(pos) = buf.iter().rposition(|&b| b == b'\n') {
            buf.truncate(pos + 1);
        }
    }
    Ok(String::from_utf8_lossy(&buf)
        .lines()
        .map(str::to_string)
        .collect())
}

/// Reads up to `max_bytes` from the end of the file, keeping whole lines.
/// Returns `None` when the whole file already fits in the head window.
fn read_tail_lines(path: &Path, max_bytes: u64) -> Result<Option<Vec<String>>, String> {
    let mut file = File::open(path).map_err(|e| format!("Failed to open {path:?}: {e}"))?;
    let len = file
        .metadata()
        .map_err(|e| format!("Failed to stat {path:?}: {e}"))?
        .len();
    if len <= max_bytes {
        return Ok(None);
    }
    file.seek(SeekFrom::Start(len - max_bytes))
        .map_err(|e| format!("Failed to seek {path:?}: {e}"))?;
    let mut buf = Vec::new();
    file.take(max_bytes)
        .read_to_end(&mut buf)
        .map_err(|e| format!("Failed to read {path:?}: {e}"))?;
    let text = String::from_utf8_lossy(&buf);
    // The first segment starts mid-line; drop it.
    let mut lines: Vec<String> = text.lines().map(str::to_string).collect();
    if !lines.is_empty() {
        lines.remove(0);
    }
    Ok(Some(lines))
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
// JSONL line helpers
// ============================================================================

/// Injected user-message blocks that should not show up as chat content.
fn is_hidden_user_block(text: &str) -> bool {
    let trimmed = text.trim_start();
    trimmed.starts_with("<user_instructions>") || trimmed.starts_with("<environment_context>")
}

/// Extracts the displayable text from a message payload `content` array.
/// Injected instructions / environment-context blocks are skipped for user
/// messages; tool and other block types are ignored entirely.
fn extract_message_blocks(content: &Value, role: &str) -> Vec<CodexContentBlock> {
    let mut blocks: Vec<CodexContentBlock> = Vec::new();
    let Some(items) = content.as_array() else {
        return blocks;
    };
    for item in items {
        let block_type = item["type"].as_str().unwrap_or("");
        match block_type {
            "input_text" | "output_text" => {
                let Some(text) = item["text"].as_str() else {
                    continue;
                };
                if role == "user" && is_hidden_user_block(text) {
                    continue;
                }
                blocks.push(CodexContentBlock {
                    content_type: "text".to_string(),
                    text: Some(text.to_string()),
                    thinking: None,
                });
            }
            _ => {}
        }
    }
    blocks
}

/// First displayable user-message text, used as the title fallback.
fn first_user_text(lines: &[String]) -> Option<String> {
    for line in lines {
        let Ok(json) = serde_json::from_str::<Value>(line) else {
            continue;
        };
        if json["type"].as_str() != Some("response_item") {
            continue;
        }
        let payload = &json["payload"];
        if payload["type"].as_str() != Some("message") || payload["role"].as_str() != Some("user") {
            continue;
        }
        let blocks = extract_message_blocks(&payload["content"], "user");
        if !blocks.is_empty() {
            return blocks[0].text.clone();
        }
    }
    None
}

/// Truncates a long message to a single-line display title.
fn title_from_text(text: &str) -> String {
    let mut title = text.split_whitespace().collect::<Vec<_>>().join(" ");
    if title.chars().count() > 60 {
        if let Some((end, _)) = title.char_indices().nth(60) {
            title.truncate(end);
            title.push('…');
        }
    }
    title
}

fn parse_token_usage(info: &Value) -> CodexTokenUsage {
    CodexTokenUsage {
        input_tokens: info["input_tokens"].as_f64().unwrap_or(0.0),
        output_tokens: info["output_tokens"].as_f64().unwrap_or(0.0),
        cache_creation_tokens: info["cache_write_input_tokens"].as_f64().unwrap_or(0.0),
        cache_read_tokens: info["cached_input_tokens"].as_f64().unwrap_or(0.0),
        reasoning_tokens: info["reasoning_output_tokens"].as_f64().unwrap_or(0.0),
        total_tokens: info["total_tokens"].as_f64().unwrap_or(0.0),
    }
}

/// Last cumulative token usage found in `lines`.
fn last_token_usage(lines: &[String]) -> CodexTokenUsage {
    let mut usage = CodexTokenUsage::default();
    for line in lines {
        let Ok(json) = serde_json::from_str::<Value>(line) else {
            continue;
        };
        if json["type"].as_str() != Some("event_msg") {
            continue;
        }
        if json["payload"]["type"].as_str() != Some("token_count") {
            continue;
        }
        if let Some(info) = json["payload"]["info"]["total_token_usage"].as_object() {
            if !info.is_empty() {
                usage = parse_token_usage(&json["payload"]["info"]["total_token_usage"]);
            }
        }
    }
    usage
}

/// First session id / cwd / model / model provider seen in `lines`.
fn scan_head_meta(lines: &[String]) -> (String, String, String, String) {
    let mut id = String::new();
    let mut cwd = String::new();
    let mut model = String::new();
    let mut model_provider = String::new();
    for line in lines {
        let Ok(json) = serde_json::from_str::<Value>(line) else {
            continue;
        };
        match json["type"].as_str() {
            Some("session_meta") => {
                if id.is_empty() {
                    id = json["payload"]["id"].as_str().unwrap_or("").to_string();
                }
                if cwd.is_empty() {
                    cwd = json["payload"]["cwd"].as_str().unwrap_or("").to_string();
                }
                if model_provider.is_empty() {
                    model_provider = json["payload"]["model_provider"]
                        .as_str()
                        .unwrap_or("")
                        .to_string();
                }
            }
            Some("turn_context") if model.is_empty() => {
                model = json["payload"]["model"].as_str().unwrap_or("").to_string();
            }
            _ => {}
        }
    }
    (id, cwd, model, model_provider)
}

/// Collects plain-text thinking blocks from a `reasoning` payload.
fn reasoning_summary_text(payload: &Value) -> Vec<String> {
    let mut texts: Vec<String> = Vec::new();
    let summaries = payload["summary"]
        .as_array()
        .or_else(|| payload["content"].as_array());
    let Some(items) = summaries else {
        return texts;
    };
    for item in items {
        if item["type"].as_str() == Some("summary_text") {
            if let Some(text) = item["text"].as_str() {
                texts.push(text.to_string());
            }
        }
    }
    texts
}

// ============================================================================
// Listing
// ============================================================================

pub fn list_codex_sessions_for_home(home_dir: &Path) -> Result<Vec<CodexSessionSummary>, String> {
    let sessions_dir = codex_sessions_dir_for_home(home_dir)?;
    if !sessions_dir.exists() {
        return Ok(Vec::new());
    }

    let codex_home = sessions_dir
        .parent()
        .map(Path::to_path_buf)
        .unwrap_or_default();
    let titles = read_session_index_titles(&codex_home);

    let mut files: Vec<PathBuf> = Vec::new();
    collect_jsonl_files(&sessions_dir, &mut files);

    let mut sessions: Vec<CodexSessionSummary> = Vec::new();
    for path in files {
        let modified_at = modified_at_ms(&path);

        let head = read_prefix_lines(&path, HEAD_BYTES)?;
        let (mut id, cwd, model, model_provider) = scan_head_meta(&head);
        if id.is_empty() {
            id = path
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("")
                .to_string();
        }

        let title = match titles.get(&id) {
            Some(index_title) => index_title.clone(),
            None => first_user_text(&head)
                .as_deref()
                .map(title_from_text)
                .unwrap_or_else(|| UNTITLED.to_string()),
        };

        let tail = read_tail_lines(&path, TAIL_BYTES)?;
        let token_usage = match tail {
            Some(tail_lines) => last_token_usage(&tail_lines),
            None => last_token_usage(&head),
        };

        sessions.push(CodexSessionSummary {
            id,
            title,
            project: cwd.clone(),
            model: if model.is_empty() {
                "unknown".to_string()
            } else {
                model
            },
            model_provider,
            modified_at,
            token_usage,
            path: path.to_string_lossy().to_string(),
        });
    }

    sessions.sort_by(|a, b| {
        b.modified_at
            .partial_cmp(&a.modified_at)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    Ok(sessions)
}

pub fn list_codex_sessions() -> Result<Vec<CodexSessionSummary>, String> {
    list_codex_sessions_for_home(&system_home_dir()?)
}

// ============================================================================
// Providers
// ============================================================================

/// Display name for a provider id aggregated from a profile.
fn provider_display_name(id: &str, name: Option<&str>) -> String {
    if id == "openai" {
        return "OpenAI".to_string();
    }
    name.map(str::trim)
        .filter(|n| !n.is_empty())
        .unwrap_or(id)
        .to_string()
}

/// All Codex model providers aggregated from every configured profile.
///
/// A profile contributes its `model_provider` plus every provider id in its
/// `providers` map. Entries are deduplicated by id, with `openai` listed
/// first, then everything else sorted by name.
pub fn list_codex_session_providers_for_home(
    home_dir: &Path,
) -> Result<Vec<CodexSessionProvider>, String> {
    let profiles = crate::codex::list_codex_profiles_for_home(home_dir)?;

    // Aggregate (id → best display name) across every profile. A profile
    // contributes its `model_provider` plus every provider id in its
    // `providers` map; a named entry upgrades a bare id-only entry.
    let mut names: HashMap<String, Option<String>> = HashMap::new();
    let add = |names: &mut HashMap<String, Option<String>>, id: &str, name: Option<&str>| {
        let id = id.trim();
        if id.is_empty() {
            return;
        }
        let name = name
            .map(str::trim)
            .filter(|n| !n.is_empty())
            .map(str::to_string);
        names
            .entry(id.to_string())
            .and_modify(|existing| {
                if existing.is_none() {
                    *existing = name.clone();
                }
            })
            .or_insert(name);
    };

    for profile in &profiles {
        add(&mut names, &profile.model_provider, None);
        for (id, config) in &profile.providers {
            add(&mut names, id, config.name.as_deref());
        }
    }

    let mut providers: Vec<CodexSessionProvider> = names
        .into_iter()
        .map(|(id, name)| CodexSessionProvider {
            name: provider_display_name(&id, name.as_deref()),
            id,
        })
        .collect();

    providers.sort_by(|a, b| {
        if a.id == "openai" {
            return std::cmp::Ordering::Less;
        }
        if b.id == "openai" {
            return std::cmp::Ordering::Greater;
        }
        a.name.to_lowercase().cmp(&b.name.to_lowercase())
    });

    Ok(providers)
}

pub fn list_codex_session_providers() -> Result<Vec<CodexSessionProvider>, String> {
    list_codex_session_providers_for_home(&system_home_dir()?)
}

/// The newest `state_*.sqlite` database in the codex home, if any. Codex
/// indexes threads there and `codex resume` lists sessions from the
/// `threads.model_provider` column, so provider switches must update it too.
fn codex_state_db(codex_home: &Path) -> Option<PathBuf> {
    let mut candidates: Vec<(u64, PathBuf)> = fs::read_dir(codex_home)
        .ok()?
        .flatten()
        .map(|entry| entry.path())
        .filter_map(|path| {
            let name = path.file_name()?.to_str()?;
            let number = name
                .strip_prefix("state_")?
                .strip_suffix(".sqlite")?
                .parse::<u64>()
                .ok()?;
            Some((number, path))
        })
        .collect();
    candidates.sort_by_key(|(number, _)| std::cmp::Reverse(*number));
    candidates.into_iter().next().map(|(_, path)| path)
}

/// Updates `threads.model_provider` for a session in the codex state
/// database. Best-effort: sessions that are not indexed (or older Codex
/// versions without the database) are left alone.
fn update_thread_index_provider(
    state_db: &Path,
    session_id: &str,
    provider_id: &str,
) -> Result<(), String> {
    let conn = rusqlite::Connection::open(state_db)
        .map_err(|e| format!("Failed to open codex state database: {e}"))?;
    conn.busy_timeout(std::time::Duration::from_millis(2000))
        .map_err(|e| format!("Failed to set database busy timeout: {e}"))?;

    let has_threads: bool = conn
        .query_row(
            "SELECT 1 FROM sqlite_master WHERE type = 'table' AND name = 'threads'",
            [],
            |_| Ok(true),
        )
        .unwrap_or(false);
    if !has_threads {
        return Ok(());
    }

    conn.execute(
        "UPDATE threads SET model_provider = ?1 WHERE id = ?2",
        rusqlite::params![provider_id, session_id],
    )
    .map_err(|e| format!("Failed to update thread provider: {e}"))?;
    Ok(())
}

/// Rewrites the `model_provider` field of a session file's `session_meta`
/// line and keeps the codex thread index (`state_*.sqlite` `threads` table)
/// in sync, so the session moves to the new provider's `codex resume` list.
/// The rest of the file is preserved byte-for-byte.
pub fn set_codex_session_provider_for_home(
    home_dir: &Path,
    session_path: &str,
    provider_id: &str,
) -> Result<(), String> {
    let sessions_dir = codex_sessions_dir_for_home(home_dir)?;
    let jsonl_path = PathBuf::from(session_path);

    // Safety guard: only rewrite files inside the codex sessions directory.
    if !jsonl_path.starts_with(&sessions_dir) || !jsonl_path.exists() {
        return Err("Session file not found".to_string());
    }

    let raw = fs::read(&jsonl_path).map_err(|e| format!("Failed to read session file: {e}"))?;
    let newline = raw
        .iter()
        .position(|&b| b == b'\n')
        .ok_or_else(|| "Session file is empty".to_string())?;
    let (first_line, rest) = raw.split_at(newline);
    let rest = &rest[1..]; // skip the newline itself

    let mut json: Value = serde_json::from_slice(first_line)
        .map_err(|e| format!("Failed to parse session_meta: {e}"))?;
    if json["type"].as_str() != Some("session_meta") {
        return Err("First line of session file is not session_meta".to_string());
    }
    let session_id = json["payload"]["id"].as_str().unwrap_or("").to_string();

    // Update the authoritative thread index first; if it fails, the session
    // file stays untouched so the change is all-or-nothing.
    let codex_home = sessions_dir
        .parent()
        .map(Path::to_path_buf)
        .unwrap_or_default();
    if !session_id.is_empty() {
        if let Some(state_db) = codex_state_db(&codex_home) {
            update_thread_index_provider(&state_db, &session_id, provider_id)?;
        }
    }

    json["payload"]["model_provider"] = Value::String(provider_id.to_string());

    let mut updated =
        serde_json::to_vec(&json).map_err(|e| format!("Failed to serialize session_meta: {e}"))?;
    updated.push(b'\n');
    updated.extend_from_slice(rest);

    crate::storage::atomic_write(&jsonl_path, &updated)
        .map_err(|e| format!("Failed to write session file: {e}"))
}

pub fn set_codex_session_provider(session_path: &str, provider_id: &str) -> Result<(), String> {
    let home_dir = system_home_dir()?;
    set_codex_session_provider_for_home(&home_dir, session_path, provider_id)
}

// ============================================================================
// Detail
// ============================================================================

/// Takes the accumulated thinking stream, preferring the plain
/// `agent_reasoning` events over `reasoning` response-item summaries, and
/// joins it into a single string when non-empty.
fn take_pending_thinking(
    pending_thinking: &mut Vec<String>,
    pending_thinking_summary: &mut Vec<String>,
) -> Option<String> {
    let chosen = if !pending_thinking.is_empty() {
        std::mem::take(pending_thinking)
    } else {
        std::mem::take(pending_thinking_summary)
    };
    pending_thinking.clear();
    pending_thinking_summary.clear();
    if chosen.is_empty() {
        None
    } else {
        Some(chosen.join("\n\n"))
    }
}

pub fn get_codex_session_detail_for_home(
    _home_dir: &Path,
    session_path: &str,
) -> Result<CodexSessionDetail, String> {
    let jsonl_path = PathBuf::from(session_path);
    if !jsonl_path.exists() {
        return Err("Session file not found".to_string());
    }

    let modified_at = modified_at_ms(&jsonl_path);

    // Read the whole file as bytes: Codex session files can contain raw tool
    // output that is not valid UTF-8, so each line is decoded lossily and
    // unparseable lines are skipped.
    let mut raw = Vec::new();
    File::open(&jsonl_path)
        .map_err(|e| format!("Failed to open session file: {e}"))?
        .read_to_end(&mut raw)
        .map_err(|e| format!("Failed to read session file: {e}"))?;

    let mut id = jsonl_path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("")
        .to_string();
    let mut cwd = String::new();
    let mut model = String::new();
    let mut model_provider = String::new();
    let mut token_usage = CodexTokenUsage::default();

    let mut messages: Vec<CodexSessionMessage> = Vec::new();
    let mut pending_thinking: Vec<String> = Vec::new();
    let mut pending_thinking_summary: Vec<String> = Vec::new();
    let mut first_user_message: Option<String> = None;
    let mut msg_counter: usize = 0;

    for line in raw.split(|&b| b == b'\n') {
        let line = String::from_utf8_lossy(line);
        let json: Value = match serde_json::from_str(&line) {
            Ok(j) => j,
            Err(_) => continue,
        };
        let timestamp = json["timestamp"].as_str().unwrap_or("").to_string();
        let payload = &json["payload"];

        match json["type"].as_str() {
            Some("session_meta") => {
                if let Some(meta_id) = payload["id"].as_str() {
                    id = meta_id.to_string();
                }
                if let Some(meta_cwd) = payload["cwd"].as_str() {
                    cwd = meta_cwd.to_string();
                }
                if model_provider.is_empty() {
                    if let Some(meta_provider) = payload["model_provider"].as_str() {
                        model_provider = meta_provider.to_string();
                    }
                }
            }
            Some("turn_context") => {
                if model.is_empty() {
                    model = payload["model"].as_str().unwrap_or("").to_string();
                }
            }
            Some("response_item") => match payload["type"].as_str() {
                Some("message") => {
                    let role = payload["role"].as_str().unwrap_or("").to_string();
                    // System-ish developer messages are not part of the chat.
                    if role != "user" && role != "assistant" {
                        continue;
                    }
                    let mut blocks = extract_message_blocks(&payload["content"], &role);
                    if role == "user" {
                        if first_user_message.is_none() {
                            first_user_message = blocks.first().and_then(|b| b.text.clone());
                        }
                        if blocks.is_empty() {
                            continue;
                        }
                    }
                    if role == "assistant" {
                        // Attach the accumulated thinking stream as the
                        // leading block of the assistant message.
                        if let Some(thinking) = take_pending_thinking(
                            &mut pending_thinking,
                            &mut pending_thinking_summary,
                        ) {
                            blocks.insert(
                                0,
                                CodexContentBlock {
                                    content_type: "thinking".to_string(),
                                    text: None,
                                    thinking: Some(thinking),
                                },
                            );
                        }
                    }
                    msg_counter += 1;
                    messages.push(CodexSessionMessage {
                        id: payload["id"]
                            .as_str()
                            .map(str::to_string)
                            .unwrap_or_else(|| format!("msg-{msg_counter}")),
                        role,
                        content: blocks,
                        timestamp,
                    });
                }
                Some("reasoning") => {
                    for text in reasoning_summary_text(payload) {
                        pending_thinking_summary.push(text);
                    }
                }
                _ => {}
            },
            Some("event_msg") => match payload["type"].as_str() {
                Some("agent_reasoning") => {
                    if let Some(text) = payload["text"].as_str() {
                        let text = text.trim();
                        if !text.is_empty() {
                            pending_thinking.push(text.to_string());
                        }
                    }
                }
                Some("token_count") => {
                    if let Some(info) = payload["info"]["total_token_usage"].as_object() {
                        if !info.is_empty() {
                            token_usage = parse_token_usage(&payload["info"]["total_token_usage"]);
                        }
                    }
                }
                _ => {}
            },
            _ => {}
        }
    }

    // A session that ended mid-turn keeps its trailing thinking stream.
    if let Some(thinking) =
        take_pending_thinking(&mut pending_thinking, &mut pending_thinking_summary)
    {
        msg_counter += 1;
        messages.push(CodexSessionMessage {
            id: format!("thinking-{msg_counter}"),
            role: "assistant".to_string(),
            content: vec![CodexContentBlock {
                content_type: "thinking".to_string(),
                text: None,
                thinking: Some(thinking),
            }],
            timestamp: String::new(),
        });
    }

    let title = first_user_message
        .as_deref()
        .map(title_from_text)
        .unwrap_or_else(|| UNTITLED.to_string());

    Ok(CodexSessionDetail {
        id,
        title,
        cwd,
        model: if model.is_empty() {
            "unknown".to_string()
        } else {
            model
        },
        model_provider,
        modified_at,
        token_usage,
        messages,
    })
}

pub fn get_codex_session_detail(session_path: &str) -> Result<CodexSessionDetail, String> {
    let home_dir = system_home_dir()?;
    get_codex_session_detail_for_home(&home_dir, session_path)
}

// ============================================================================
// Delete
// ============================================================================

pub fn delete_codex_session_for_home(home_dir: &Path, session_path: &str) -> Result<(), String> {
    let sessions_dir = codex_sessions_dir_for_home(home_dir)?;
    let jsonl_path = PathBuf::from(session_path);

    // Safety guard: only delete files inside the codex sessions directory.
    if !jsonl_path.starts_with(&sessions_dir) || !jsonl_path.exists() {
        return Err("Session file not found".to_string());
    }

    fs::remove_file(&jsonl_path).map_err(|e| format!("Failed to delete session: {e}"))?;

    // Prune empty day/month directories left behind, up to the sessions dir.
    let mut parent = jsonl_path.parent().map(Path::to_path_buf);
    while let Some(dir) = parent {
        if dir == sessions_dir || !dir.starts_with(&sessions_dir) {
            break;
        }
        match fs::remove_dir(&dir) {
            Ok(()) => parent = dir.parent().map(Path::to_path_buf),
            Err(_) => break,
        }
    }

    Ok(())
}

pub fn delete_codex_session(session_path: &str) -> Result<(), String> {
    let home_dir = system_home_dir()?;
    delete_codex_session_for_home(&home_dir, session_path)
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::{
        delete_codex_session_for_home, get_codex_session_detail_for_home,
        list_codex_session_providers_for_home, list_codex_sessions_for_home,
        set_codex_session_provider_for_home,
    };
    use std::fs;
    use std::io::Write;
    use tempfile::TempDir;

    struct Fixture {
        _temp: TempDir,
        home: std::path::PathBuf,
    }

    impl Fixture {
        fn new(session_lines: &[&str]) -> Self {
            let temp = TempDir::new().unwrap();
            let home = temp.path().to_path_buf();
            let session_dir = home.join(".codex/sessions/2026/09/08");
            fs::create_dir_all(&session_dir).unwrap();
            let session_path = session_dir.join("rollout-test.jsonl");
            let mut file = fs::File::create(&session_path).unwrap();
            for line in session_lines {
                writeln!(file, "{line}").unwrap();
            }
            Self { _temp: temp, home }
        }

        fn with_index(home: &std::path::Path, id: &str, thread_name: &str) {
            let index_path = home.join(".codex/session_index.jsonl");
            let mut file = fs::OpenOptions::new()
                .create(true)
                .append(true)
                .open(index_path)
                .unwrap();
            writeln!(
                file,
                "{{\"id\":\"{id}\",\"thread_name\":\"{thread_name}\",\"updated_at\":\"2026-09-08T00:00:00Z\"}}"
            )
            .unwrap();
        }

        fn session_path(&self) -> String {
            self.home
                .join(".codex/sessions/2026/09/08/rollout-test.jsonl")
                .to_string_lossy()
                .to_string()
        }
    }

    const META: &str = r#"{"timestamp":"2026-09-08T00:00:00Z","type":"session_meta","payload":{"id":"sess-1","cwd":"/work/repo","model_provider":"openai"}}"#;
    const TURN: &str = r#"{"timestamp":"2026-09-08T00:00:01Z","type":"turn_context","payload":{"cwd":"/work/repo","model":"gpt-5"}}"#;
    const USER_INSTRUCTIONS: &str = r#"{"timestamp":"2026-09-08T00:00:02Z","type":"response_item","payload":{"type":"message","role":"user","content":[{"type":"input_text","text":"<user_instructions>\n# Guidelines\n"}]}}"#;
    const USER_ENV: &str = r#"{"timestamp":"2026-09-08T00:00:03Z","type":"response_item","payload":{"type":"message","role":"user","content":[{"type":"input_text","text":"<environment_context>\n  <cwd>/work/repo</cwd>\n</environment_context>"}]}}"#;
    const USER_REAL: &str = r#"{"timestamp":"2026-09-08T00:00:04Z","type":"response_item","payload":{"type":"message","role":"user","content":[{"type":"input_text","text":"Fix the bug"}]}}"#;
    const THINKING: &str = r#"{"timestamp":"2026-09-08T00:00:05Z","type":"event_msg","payload":{"type":"agent_reasoning","text":"I should check the parser."}}"#;
    const REASONING_SUMMARY: &str = r#"{"timestamp":"2026-09-08T00:00:06Z","type":"response_item","payload":{"type":"reasoning","summary":[{"type":"summary_text","text":"summary text"}],"encrypted_content":"enc"}}"#;
    const ASSISTANT: &str = r#"{"timestamp":"2026-09-08T00:00:07Z","type":"response_item","payload":{"type":"message","role":"assistant","content":[{"type":"output_text","text":"Done"}]}}"#;
    const TOKENS: &str = r#"{"timestamp":"2026-09-08T00:00:08Z","type":"event_msg","payload":{"type":"token_count","info":{"total_token_usage":{"input_tokens":100,"cached_input_tokens":10,"cache_write_input_tokens":5,"output_tokens":20,"reasoning_output_tokens":30,"total_tokens":165}}}}"#;

    const FULL_SESSION: &[&str] = &[
        META,
        TURN,
        USER_INSTRUCTIONS,
        USER_ENV,
        USER_REAL,
        THINKING,
        REASONING_SUMMARY,
        ASSISTANT,
        TOKENS,
    ];

    #[test]
    fn list_reads_summary_fields_and_index_title() {
        let fixture = Fixture::new(FULL_SESSION);
        Fixture::with_index(&fixture.home, "sess-1", "Fix the bug");

        let sessions = list_codex_sessions_for_home(&fixture.home).unwrap();
        assert_eq!(sessions.len(), 1);

        let s = &sessions[0];
        assert_eq!(s.id, "sess-1");
        assert_eq!(s.title, "Fix the bug");
        assert_eq!(s.project, "/work/repo");
        assert_eq!(s.model, "gpt-5");
        assert_eq!(s.model_provider, "openai");
        assert_eq!(s.token_usage.input_tokens, 100.0);
        assert_eq!(s.token_usage.output_tokens, 20.0);
        assert_eq!(s.token_usage.cache_read_tokens, 10.0);
        assert_eq!(s.token_usage.cache_creation_tokens, 5.0);
        assert_eq!(s.token_usage.reasoning_tokens, 30.0);
        assert_eq!(s.token_usage.total_tokens, 165.0);
        assert_eq!(s.path, fixture.session_path());
        assert!(s.modified_at > 0.0);
    }

    #[test]
    fn list_falls_back_to_first_user_message_title() {
        let fixture = Fixture::new(FULL_SESSION);

        let sessions = list_codex_sessions_for_home(&fixture.home).unwrap();
        assert_eq!(sessions[0].title, "Fix the bug");
    }

    #[test]
    fn list_uses_untitled_when_no_title_source() {
        let fixture = Fixture::new(&[META, TURN]);

        let sessions = list_codex_sessions_for_home(&fixture.home).unwrap();
        assert_eq!(sessions[0].title, "New Session");
        assert_eq!(sessions[0].model, "gpt-5");
        assert_eq!(sessions[0].token_usage.total_tokens, 0.0);
    }

    #[test]
    fn detail_filters_injected_blocks_and_attaches_thinking() {
        let fixture = Fixture::new(FULL_SESSION);

        let detail =
            get_codex_session_detail_for_home(&fixture.home, &fixture.session_path()).unwrap();
        assert_eq!(detail.id, "sess-1");
        assert_eq!(detail.cwd, "/work/repo");
        assert_eq!(detail.model, "gpt-5");
        assert_eq!(detail.model_provider, "openai");
        assert_eq!(detail.token_usage.total_tokens, 165.0);

        // 1 user message (injected instruction/env messages filtered out)
        // + 1 assistant message carrying thinking + text.
        assert_eq!(detail.messages.len(), 2);

        let user_messages: Vec<_> = detail
            .messages
            .iter()
            .filter(|m| m.role == "user")
            .collect();
        assert_eq!(user_messages.len(), 1);
        assert_eq!(
            user_messages[0].content[0].text.as_deref(),
            Some("Fix the bug")
        );

        let assistant_messages: Vec<_> = detail
            .messages
            .iter()
            .filter(|m| m.role == "assistant")
            .collect();
        assert_eq!(assistant_messages.len(), 1);
        assert_eq!(assistant_messages[0].content.len(), 2);
        assert_eq!(assistant_messages[0].content[0].content_type, "thinking");
        assert_eq!(
            assistant_messages[0].content[0].thinking.as_deref(),
            Some("I should check the parser."),
            "agent_reasoning wins over reasoning summaries"
        );
        assert_eq!(
            assistant_messages[0].content[1].text.as_deref(),
            Some("Done")
        );
    }

    #[test]
    fn detail_uses_reasoning_summary_when_agent_reasoning_absent() {
        let fixture = Fixture::new(&[META, USER_REAL, REASONING_SUMMARY, ASSISTANT]);

        let detail =
            get_codex_session_detail_for_home(&fixture.home, &fixture.session_path()).unwrap();
        let thinking = detail
            .messages
            .iter()
            .flat_map(|m| m.content.iter())
            .find_map(|b| b.thinking.as_deref());
        assert_eq!(thinking, Some("summary text"));
    }

    #[test]
    fn delete_removes_file_and_prunes_empty_date_dirs() {
        let fixture = Fixture::new(FULL_SESSION);
        let day_dir = fixture.home.join(".codex/sessions/2026/09/08");

        delete_codex_session_for_home(&fixture.home, &fixture.session_path()).unwrap();

        assert!(!fixture.home.join(".codex/sessions/2026/09/08").exists());
        assert!(!fixture.home.join(".codex/sessions/2026/09").exists());
        assert!(!fixture.home.join(".codex/sessions/2026").exists());
        assert!(!day_dir.exists());
    }

    #[test]
    fn delete_rejects_paths_outside_sessions_dir() {
        let fixture = Fixture::new(FULL_SESSION);
        let outside = fixture.home.join("config.toml");
        fs::write(&outside, "x").unwrap();

        let err =
            delete_codex_session_for_home(&fixture.home, &outside.to_string_lossy()).unwrap_err();
        assert!(err.contains("not found"));
        assert!(outside.exists());
    }

    #[test]
    fn list_reads_token_usage_from_tail_window_for_large_files() {
        let fixture = Fixture::new(&[META]);
        let session_path = fixture.session_path();
        // Pad the file well beyond HEAD_BYTES + TAIL_BYTES so the final
        // token_count event only lives in the tail window.
        let mut file = fs::OpenOptions::new()
            .append(true)
            .open(&session_path)
            .unwrap();
        let padding = "{\"timestamp\":\"2026-09-08T00:00:00Z\",\"type\":\"event_msg\",\"payload\":{\"type\":\"agent_reasoning\",\"text\":\"xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx\"}}\n";
        while file.metadata().unwrap().len() < 400 * 1024 {
            file.write_all(padding.as_bytes()).unwrap();
        }
        writeln!(file, "{TOKENS}").unwrap();

        let sessions = list_codex_sessions_for_home(&fixture.home).unwrap();
        assert_eq!(sessions[0].token_usage.total_tokens, 165.0);
    }

    #[test]
    fn list_and_detail_tolerate_invalid_utf8_lines() {
        let temp = TempDir::new().unwrap();
        let home = temp.path().to_path_buf();
        let session_dir = home.join(".codex/sessions/2026/02/11");
        fs::create_dir_all(&session_dir).unwrap();
        let session_path = session_dir.join("rollout-binary.jsonl");
        let mut file = fs::File::create(&session_path).unwrap();
        writeln!(file, "{META}").unwrap();
        writeln!(file, "{USER_REAL}").unwrap();
        // Raw tool output containing invalid UTF-8 bytes.
        file.write_all(b"{\"timestamp\":\"2026-09-08T00:00:05Z\",\"type\":\"response_item\",\"payload\":{\"type\":\"function_call_output\",\"output\":\"\xFF\xFE\"}}\n")
            .unwrap();
        writeln!(file, "{THINKING}").unwrap();
        writeln!(file, "{ASSISTANT}").unwrap();
        writeln!(file, "{TOKENS}").unwrap();

        let sessions = list_codex_sessions_for_home(&home).unwrap();
        assert_eq!(sessions.len(), 1);
        assert_eq!(sessions[0].title, "Fix the bug");
        assert_eq!(sessions[0].token_usage.total_tokens, 165.0);

        let detail = get_codex_session_detail_for_home(&home, &sessions[0].path).unwrap();
        assert_eq!(
            detail.messages.len(),
            2,
            "valid lines around the binary line are kept"
        );
        assert_eq!(detail.token_usage.total_tokens, 165.0);
    }

    #[test]
    fn list_tolerates_invalid_utf8_in_tail_window() {
        let fixture = Fixture::new(&[META]);
        let session_path = fixture.session_path();
        // Pad past the tail window, then append a binary line and the final
        // token usage — both only visible to the tail read.
        let mut file = fs::OpenOptions::new()
            .append(true)
            .open(&session_path)
            .unwrap();
        let padding = "{\"timestamp\":\"2026-09-08T00:00:00Z\",\"type\":\"event_msg\",\"payload\":{\"type\":\"agent_reasoning\",\"text\":\"xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx\"}}\n";
        while file.metadata().unwrap().len() < 400 * 1024 {
            file.write_all(padding.as_bytes()).unwrap();
        }
        file.write_all(
            b"{\"timestamp\":\"2026-09-08T00:00:00Z\",\"type\":\"response_item\",\"payload\":{\"type\":\"function_call_output\",\"output\":\"\xF0\x28\x8C\x28\"}}\n",
        )
        .unwrap();
        writeln!(file, "{TOKENS}").unwrap();

        let sessions = list_codex_sessions_for_home(&fixture.home).unwrap();
        assert_eq!(sessions[0].token_usage.total_tokens, 165.0);
    }

    #[test]
    fn providers_aggregate_across_all_profiles() {
        let temp = TempDir::new().unwrap();
        let home = temp.path().to_path_buf();

        let openai_profile = crate::codex::CodexProfile {
            id: "p-openai".to_string(),
            name: "Official".to_string(),
            description: None,
            created_at: String::new(),
            updated_at: String::new(),
            providers: Default::default(),
            model_provider: "openai".to_string(),
            model: "gpt-5".to_string(),
            model_reasoning_effort: None,
            api_key: None,
            auth_profile_name: None,
        };
        crate::codex::save_codex_profile_for_home(&home, openai_profile).unwrap();

        let mut custom_providers = std::collections::HashMap::new();
        custom_providers.insert(
            "deepseek".to_string(),
            crate::codex::CodexProviderConfig {
                name: Some("DeepSeek".to_string()),
                base_url: None,
                wire_api: None,
                requires_openai_auth: None,
                env_key: None,
                env_key_instructions: None,
                http_headers: None,
                query_params: None,
                model: None,
                model_reasoning_effort: None,
                api_key: None,
            },
        );
        custom_providers.insert(
            "mimo".to_string(),
            crate::codex::CodexProviderConfig {
                name: None,
                base_url: None,
                wire_api: None,
                requires_openai_auth: None,
                env_key: None,
                env_key_instructions: None,
                http_headers: None,
                query_params: None,
                model: None,
                model_reasoning_effort: None,
                api_key: None,
            },
        );
        let custom_profile = crate::codex::CodexProfile {
            id: "p-custom".to_string(),
            name: "Custom".to_string(),
            description: None,
            created_at: String::new(),
            updated_at: String::new(),
            providers: custom_providers,
            model_provider: "deepseek".to_string(),
            model: "deepseek-v4-pro".to_string(),
            model_reasoning_effort: None,
            api_key: None,
            auth_profile_name: None,
        };
        crate::codex::save_codex_profile_for_home(&home, custom_profile).unwrap();

        let providers = list_codex_session_providers_for_home(&home).unwrap();
        assert_eq!(providers.len(), 3);
        assert_eq!(providers[0].id, "openai");
        assert_eq!(providers[0].name, "OpenAI");
        let by_id = |id: &str| providers.iter().find(|p| p.id == id).unwrap();
        assert_eq!(by_id("deepseek").name, "DeepSeek");
        // Provider without a name falls back to its id.
        assert_eq!(by_id("mimo").name, "mimo");
    }

    #[test]
    fn set_provider_rewrites_session_meta_and_preserves_rest() {
        let fixture = Fixture::new(FULL_SESSION);
        let original = fs::read(fixture.session_path()).unwrap();

        set_codex_session_provider_for_home(&fixture.home, &fixture.session_path(), "deepseek")
            .unwrap();

        let updated = fs::read(fixture.session_path()).unwrap();
        let first_line =
            String::from_utf8_lossy(updated.split(|&b| b == b'\n').next().expect("first line"));
        assert!(first_line.contains(r#""model_provider":"deepseek""#));
        // Rest of the file is preserved byte-for-byte.
        let original_rest = &original[original.iter().position(|&b| b == b'\n').unwrap() + 1..];
        let updated_rest = &updated[updated.iter().position(|&b| b == b'\n').unwrap() + 1..];
        assert_eq!(original_rest, updated_rest);

        let sessions = list_codex_sessions_for_home(&fixture.home).unwrap();
        assert_eq!(sessions[0].model_provider, "deepseek");

        let detail =
            get_codex_session_detail_for_home(&fixture.home, &fixture.session_path()).unwrap();
        assert_eq!(detail.model_provider, "deepseek");
        assert_eq!(detail.id, "sess-1");
        assert_eq!(detail.messages.len(), 2);
    }

    #[test]
    fn set_provider_rejects_paths_outside_sessions_dir() {
        let fixture = Fixture::new(FULL_SESSION);
        let outside = fixture.home.join("config.toml");
        fs::write(&outside, "x").unwrap();

        let err = set_codex_session_provider_for_home(
            &fixture.home,
            &outside.to_string_lossy(),
            "deepseek",
        )
        .unwrap_err();
        assert!(err.contains("not found"));
        assert_eq!(fs::read_to_string(&outside).unwrap(), "x");
    }

    #[test]
    fn set_provider_rejects_files_without_session_meta_header() {
        let fixture = Fixture::new(&[TURN]);

        let err =
            set_codex_session_provider_for_home(&fixture.home, &fixture.session_path(), "deepseek")
                .unwrap_err();
        assert!(err.contains("session_meta"));
    }

    #[test]
    fn set_provider_updates_thread_index_db() {
        let fixture = Fixture::new(FULL_SESSION);
        let state_db = fixture.home.join(".codex/state_5.sqlite");
        let conn = rusqlite::Connection::open(&state_db).unwrap();
        conn.execute(
            "CREATE TABLE threads (id TEXT PRIMARY KEY, model_provider TEXT NOT NULL)",
            [],
        )
        .unwrap();
        conn.execute(
            "INSERT INTO threads (id, model_provider) VALUES ('sess-1', 'openai')",
            [],
        )
        .unwrap();
        drop(conn);

        set_codex_session_provider_for_home(&fixture.home, &fixture.session_path(), "deepseek")
            .unwrap();

        let conn = rusqlite::Connection::open(&state_db).unwrap();
        let provider: String = conn
            .query_row(
                "SELECT model_provider FROM threads WHERE id = 'sess-1'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(provider, "deepseek");

        let sessions = list_codex_sessions_for_home(&fixture.home).unwrap();
        assert_eq!(sessions[0].model_provider, "deepseek");
    }

    #[test]
    fn set_provider_ignores_state_db_without_threads_table() {
        let fixture = Fixture::new(FULL_SESSION);
        let state_db = fixture.home.join(".codex/state_5.sqlite");
        let conn = rusqlite::Connection::open(&state_db).unwrap();
        conn.execute("CREATE TABLE other (id TEXT)", []).unwrap();
        drop(conn);

        set_codex_session_provider_for_home(&fixture.home, &fixture.session_path(), "deepseek")
            .unwrap();

        let sessions = list_codex_sessions_for_home(&fixture.home).unwrap();
        assert_eq!(sessions[0].model_provider, "deepseek");
    }
}
