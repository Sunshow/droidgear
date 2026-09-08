//! Pi session history, read directly from the documented JSONL format (v1–v3).
//! See https://github.com/earendil-works/pi/blob/main/packages/coding-agent/docs/sessions.md
//! and https://github.com/earendil-works/pi/blob/main/packages/coding-agent/docs/session-format.md.
//!
//! Discovery never opens sessions through Pi's SDK, which can migrate/rewrite files.
//! Usage covers the entire file; the saved branch is the ancestry of the last entry.

use crate::{paths, sessions::ContentBlock};
use serde::{Deserialize, Serialize};
use specta::Type;
use std::collections::{HashMap, HashSet};
use std::fs::{self, File};
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Default, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct PiTokenUsage {
    pub input_tokens: f64,
    pub output_tokens: f64,
    pub cache_read_tokens: f64,
    pub cache_creation_tokens: f64,
    pub reasoning_tokens: f64,
    pub total_tokens: f64,
    pub cost: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct PiSessionSummary {
    pub id: String,
    pub title: String,
    // The header's cwd, not the lossy directory encoding.
    pub project: String,
    pub model: String,
    pub model_provider: String,
    pub modified_at: f64,
    // All `message` entries in the session, including tool results.
    pub message_count: u32,
    pub token_usage: PiTokenUsage,
    pub path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct PiSessionMessage {
    pub id: String,
    pub role: String,
    pub content: Vec<ContentBlock>,
    pub timestamp: String,
    pub is_active_branch: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct PiSessionDetail {
    pub summary: PiSessionSummary,
    // Displayable history in append order, including pre-compaction messages.
    pub messages: Vec<PiSessionMessage>,
}

// Deserialize only the fields needed for history. Tool arguments, images and
// provider signatures can be large and must not be copied across the UI bridge.
#[derive(Deserialize, Default)]
#[serde(default, rename_all = "camelCase")]
struct Entry {
    #[serde(rename = "type")]
    kind: String,
    id: String,
    parent_id: Option<String>,
    version: Option<u32>,
    timestamp: String,
    cwd: String,
    name: Option<String>,
    model_id: String,
    provider: String,
    message: Option<Message>,
    content: Option<Content>,
    display: bool,
    summary: Option<String>,
    usage: Option<Usage>,
}

#[derive(Deserialize, Default)]
#[serde(default, rename_all = "camelCase")]
struct Message {
    role: String,
    content: Option<Content>,
    model: String,
    provider: String,
    timestamp: Option<f64>,
    display: bool,
    error_message: Option<String>,
    usage: Option<Usage>,
}

#[derive(Deserialize)]
#[serde(untagged)]
enum Content {
    Text(String),
    Blocks(Vec<ContentBlock>),
}

#[derive(Deserialize, Default)]
#[serde(default, rename_all = "camelCase")]
struct Usage {
    input: f64,
    output: f64,
    cache_read: f64,
    cache_write: f64,
    reasoning: f64,
    total_tokens: Option<f64>,
    cost: Cost,
}

#[derive(Deserialize, Default)]
#[serde(default)]
struct Cost {
    total: f64,
}

impl PiTokenUsage {
    fn add(&mut self, usage: &Usage) {
        self.input_tokens += usage.input;
        self.output_tokens += usage.output;
        self.cache_read_tokens += usage.cache_read;
        self.cache_creation_tokens += usage.cache_write;
        self.reasoning_tokens += usage.reasoning;
        self.total_tokens += usage
            .total_tokens
            .unwrap_or(usage.input + usage.output + usage.cache_read + usage.cache_write);
        self.cost += usage.cost.total;
    }
}

fn expand_path(home: &Path, value: &str) -> PathBuf {
    if value == "~" {
        home.to_path_buf()
    } else if let Some(rest) = value
        .strip_prefix("~/")
        .or_else(|| value.strip_prefix("~\\"))
    {
        home.join(rest)
    } else {
        PathBuf::from(value)
    }
}

fn resolve_sessions_dir(
    home: &Path,
    config: &paths::ConfigPaths,
    agent_env: Option<&str>,
    sessions_env: Option<&str>,
) -> Result<PathBuf, String> {
    let agent_dir = config
        .pi
        .as_deref()
        .or(agent_env)
        .filter(|value| !value.is_empty())
        .map(|value| expand_path(home, value))
        .unwrap_or_else(|| home.join(".pi/agent"));
    if let Some(value) = sessions_env.filter(|value| !value.is_empty()) {
        return Ok(expand_path(home, value));
    }
    Ok(agent_dir.join("sessions"))
}

pub fn pi_sessions_dir_for_home(home: &Path) -> Result<PathBuf, String> {
    resolve_sessions_dir(
        home,
        &paths::load_config_paths_for_home(home),
        std::env::var("PI_CODING_AGENT_DIR").ok().as_deref(),
        std::env::var("PI_CODING_AGENT_SESSION_DIR").ok().as_deref(),
    )
}

pub fn pi_sessions_dir() -> Result<PathBuf, String> {
    pi_sessions_dir_for_home(&paths::get_home_dir()?)
}

fn collect_files(dir: &Path, files: &mut Vec<PathBuf>) -> Result<(), String> {
    let entries = fs::read_dir(dir).map_err(|e| format!("Failed to read Pi sessions: {e}"))?;
    for entry in entries.flatten() {
        let Ok(kind) = entry.file_type() else {
            continue;
        };
        let path = entry.path();
        // Do not recurse through symlinks (cycles or files outside the root).
        if kind.is_dir() {
            if let Err(error) = collect_files(&path, files) {
                log::warn!("{error}");
            }
        } else if kind.is_file() && path.extension().and_then(|s| s.to_str()) == Some("jsonl") {
            files.push(path);
        }
    }
    Ok(())
}

fn timestamp_ms(value: &str) -> Option<f64> {
    chrono::DateTime::parse_from_rfc3339(value)
        .ok()
        .map(|t| t.timestamp_millis() as f64)
}

fn title_from_text(text: &str) -> String {
    let normalized = text.split_whitespace().collect::<Vec<_>>().join(" ");
    let mut title: String = normalized.chars().take(60).collect();
    if normalized.chars().count() > 60 {
        title.push('…');
    }
    title
}

fn text_block(kind: &str, text: String) -> ContentBlock {
    ContentBlock {
        content_type: kind.to_string(),
        text: Some(text),
        thinking: None,
    }
}

fn display_blocks(content: Option<Content>) -> Vec<ContentBlock> {
    match content {
        Some(Content::Text(text)) if !text.is_empty() => vec![text_block("text", text)],
        Some(Content::Blocks(blocks)) => blocks
            .into_iter()
            .filter_map(|block| match block.content_type.as_str() {
                "text" if block.text.as_ref().is_some_and(|s| !s.is_empty()) => Some(block),
                "thinking" if block.thinking.as_ref().is_some_and(|s| !s.is_empty()) => Some(block),
                "image" => Some(text_block("image", String::new())),
                _ => None,
            })
            .collect(),
        _ => Vec::new(),
    }
}

struct HistoryEntry {
    id: String,
    parent_id: Option<String>,
    model: Option<(String, String)>,
    message: Option<PiSessionMessage>,
}

fn read_session(path: &Path, include_messages: bool) -> Result<PiSessionDetail, String> {
    let file = File::open(path).map_err(|e| format!("Failed to open Pi session: {e}"))?;
    let modified = file
        .metadata()
        .ok()
        .and_then(|m| m.modified().ok())
        .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
        .map(|d| d.as_millis() as f64)
        .unwrap_or_default();
    let mut header: Option<Entry> = None;
    let mut entries: Vec<HistoryEntry> = Vec::new();
    let mut name = None;
    let mut first_message = String::new();
    let mut usage = PiTokenUsage::default();
    let mut message_count = 0;
    let mut activity: Option<f64> = None;

    // LF is the only record delimiter; malformed/incomplete trailing records
    // are ignored without repairing or rewriting the file.
    for (line_number, line) in BufReader::new(file).split(b'\n').enumerate() {
        let line = line.map_err(|e| format!("Failed to read Pi session: {e}"))?;
        let Ok(mut entry) = serde_json::from_slice::<Entry>(&line) else {
            continue;
        };
        let Some(session_header) = header.as_ref() else {
            if entry.kind != "session" || entry.id.is_empty() {
                return Err("Not a Pi session file".to_string());
            }
            header = Some(entry);
            continue;
        };
        if entry.kind == "session_info" {
            name = entry
                .name
                .take()
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty());
        }
        if entry.id.is_empty() {
            entry.id = format!("line-{line_number}");
        }
        if session_header.version.unwrap_or(1) < 2 {
            entry.parent_id = entries.last().map(|e| e.id.clone());
        }
        let mut model = None;
        let mut role = String::new();
        let mut blocks = Vec::new();
        match entry.kind.as_str() {
            "model_change" => model = Some((entry.provider, entry.model_id)),
            "message" => {
                // Pi's session picker counts every message entry, while the
                // detail view renders the conversational roles it can show.
                message_count += 1;
                if let Some(message) = entry.message {
                    if matches!(message.role.as_str(), "assistant" | "toolResult") {
                        if let Some(value) = message.usage.as_ref() {
                            usage.add(value);
                        }
                    }
                    if message.role == "assistant" && !message.model.is_empty() {
                        model = Some((message.provider.clone(), message.model.clone()));
                    }
                    if matches!(message.role.as_str(), "user" | "assistant") {
                        if let Some(time) =
                            message.timestamp.or_else(|| timestamp_ms(&entry.timestamp))
                        {
                            activity = Some(activity.unwrap_or(0.0).max(time));
                        }
                        blocks = display_blocks(message.content);
                        if message.role == "user" && first_message.is_empty() {
                            first_message = blocks
                                .iter()
                                .filter(|b| b.content_type == "text")
                                .filter_map(|b| b.text.as_deref())
                                .collect::<Vec<_>>()
                                .join(" ");
                        }
                        if let Some(error) = message.error_message.filter(|s| !s.is_empty()) {
                            blocks.push(text_block("error", error));
                        }
                        role = message.role;
                    } else if matches!(message.role.as_str(), "custom" | "hookMessage")
                        && message.display
                    {
                        blocks = display_blocks(message.content);
                        role = "system".to_string();
                    }
                }
            }
            "compaction" | "branch_summary" => {
                if let Some(value) = entry.usage.as_ref() {
                    usage.add(value);
                }
                if let Some(summary) = entry.summary {
                    blocks.push(text_block(&entry.kind, summary));
                    role = "system".to_string();
                }
                // retainedTail repeats existing messages; don't count it again.
            }
            "custom_message" if entry.display => {
                blocks = display_blocks(entry.content);
                role = "system".to_string();
            }
            _ => {}
        }
        let message = (include_messages && !blocks.is_empty()).then(|| PiSessionMessage {
            id: entry.id.clone(),
            role,
            content: blocks,
            timestamp: entry.timestamp,
            is_active_branch: false,
        });
        entries.push(HistoryEntry {
            id: entry.id,
            parent_id: entry.parent_id,
            model,
            message,
        });
    }
    let header = header.ok_or_else(|| "Not a Pi session file".to_string())?;
    let by_id: HashMap<&str, usize> = entries
        .iter()
        .enumerate()
        .map(|(i, e)| (e.id.as_str(), i))
        .collect();
    let mut active = HashSet::new();
    let mut current = entries.len().checked_sub(1);
    let mut model = None;
    while let Some(index) = current {
        if !active.insert(index) {
            break;
        }
        let entry = &entries[index];
        if model.is_none() {
            model = entry.model.clone();
        }
        current = entry
            .parent_id
            .as_deref()
            .and_then(|id| by_id.get(id).copied());
    }
    let (model_provider, model) = model.unwrap_or_else(|| (String::new(), "unknown".to_string()));
    let title = name.unwrap_or_else(|| {
        let title = title_from_text(&first_message);
        if title.is_empty() {
            "New Session".to_string()
        } else {
            title
        }
    });
    let modified_at = activity
        .filter(|time| *time > 0.0)
        .or_else(|| timestamp_ms(&header.timestamp))
        .unwrap_or(modified);
    let messages = entries
        .into_iter()
        .enumerate()
        .filter_map(|(i, entry)| {
            entry.message.map(|mut message| {
                message.is_active_branch = active.contains(&i);
                message
            })
        })
        .collect();
    Ok(PiSessionDetail {
        summary: PiSessionSummary {
            id: header.id,
            title,
            project: header.cwd,
            model,
            model_provider,
            modified_at,
            message_count,
            token_usage: usage,
            path: path.to_string_lossy().to_string(),
        },
        messages,
    })
}

fn list_in_dir(dir: &Path) -> Result<Vec<PiSessionSummary>, String> {
    if !dir.exists() {
        return Ok(Vec::new());
    }
    let mut files = Vec::new();
    collect_files(dir, &mut files)?;
    let mut sessions: Vec<_> = files
        .iter()
        .filter_map(|path| match read_session(path, false) {
            Ok(detail) => Some(detail.summary),
            Err(error) => {
                log::warn!("Skipping Pi session {}: {error}", path.display());
                None
            }
        })
        .collect();
    sessions.sort_by(|a, b| {
        b.modified_at
            .total_cmp(&a.modified_at)
            .then_with(|| a.path.cmp(&b.path))
    });
    Ok(sessions)
}

pub fn list_pi_sessions_for_home(home: &Path) -> Result<Vec<PiSessionSummary>, String> {
    list_in_dir(&pi_sessions_dir_for_home(home)?)
}

pub fn list_pi_sessions() -> Result<Vec<PiSessionSummary>, String> {
    list_pi_sessions_for_home(&paths::get_home_dir()?)
}

fn validate_session_path(root: &Path, session_path: &str) -> Result<PathBuf, String> {
    let root = root
        .canonicalize()
        .map_err(|e| format!("Pi sessions directory unavailable: {e}"))?;
    let path = Path::new(session_path)
        .canonicalize()
        .map_err(|e| format!("Pi session unavailable: {e}"))?;
    if !path.starts_with(root)
        || !path.is_file()
        || path.extension().and_then(|s| s.to_str()) != Some("jsonl")
    {
        return Err("Session must be a JSONL file inside the Pi sessions directory".to_string());
    }
    Ok(path)
}

pub fn get_pi_session_detail_for_home(
    home: &Path,
    session_path: &str,
) -> Result<PiSessionDetail, String> {
    let path = validate_session_path(&pi_sessions_dir_for_home(home)?, session_path)?;
    read_session(&path, true)
}

pub fn get_pi_session_detail(session_path: &str) -> Result<PiSessionDetail, String> {
    get_pi_session_detail_for_home(&paths::get_home_dir()?, session_path)
}

pub fn delete_pi_session_for_home(home: &Path, session_path: &str) -> Result<(), String> {
    let path = validate_session_path(&pi_sessions_dir_for_home(home)?, session_path)?;
    // Validate the header as well, so unrelated JSONL files cannot be removed.
    read_session(&path, false)?;
    fs::remove_file(&path).map_err(|e| format!("Failed to delete Pi session: {e}"))
}

pub fn delete_pi_session(session_path: &str) -> Result<(), String> {
    delete_pi_session_for_home(&paths::get_home_dir()?, session_path)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn write_session(home: &Path) -> PathBuf {
        let path = home.join(".pi/agent/sessions/project/session.jsonl");
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(
            &path,
            r#"{"type":"session","version":3,"id":"session-1","timestamp":"2026-01-01T00:00:00.000Z","cwd":"/work/demo"}
{"type":"model_change","id":"model-1","parentId":null,"timestamp":"2026-01-01T00:00:00.100Z","provider":"openai","modelId":"gpt-5"}
{"type":"message","id":"user-1","parentId":"model-1","timestamp":"2026-01-01T00:00:01.000Z","message":{"role":"user","content":"Investigate the parser","timestamp":1767225601000}}
{"type":"message","id":"assistant-1","parentId":"user-1","timestamp":"2026-01-01T00:00:02.000Z","message":{"role":"assistant","provider":"openai","model":"gpt-5","content":[{"type":"thinking","thinking":"Read the file"},{"type":"text","text":"Done"}],"usage":{"input":10,"output":5,"cacheRead":2,"cacheWrite":1,"totalTokens":18,"cost":{"total":0.03}},"timestamp":1767225602000}}
{"type":"session_info","id":"name-1","parentId":"assistant-1","timestamp":"2026-01-01T00:00:03.000Z","name":"Parser review"}
{"type":"session_info","id":"name-2","parentId":"name-1","timestamp":"2026-01-01T00:00:04.000Z","name":"  "}
{"type":"message","id":"branch-1","parentId":"user-1","timestamp":"2026-01-01T00:00:05.000Z","message":{"role":"assistant","provider":"openai","model":"gpt-5","content":"Alternate answer","usage":{"input":1,"output":1,"totalTokens":2}}}
{"type":"compaction","id":"compact-1","parentId":"branch-1","timestamp":"2026-01-01T00:00:06.000Z","summary":"Older context summarized","tokensBefore":100,"usage":{"input":3,"output":1,"totalTokens":4,"cost":{"total":0.01}}}
"#,
        )
        .unwrap();
        path
    }

    #[test]
    fn lists_pi_sessions_with_name_fallback_and_accumulated_usage() {
        let home = TempDir::new().unwrap();
        let path = write_session(home.path());
        let sessions = list_pi_sessions_for_home(home.path()).unwrap();
        assert_eq!(sessions.len(), 1);
        let session = &sessions[0];
        assert_eq!(session.id, "session-1");
        // An explicitly cleared session_info falls back to the first prompt.
        assert_eq!(session.title, "Investigate the parser");
        assert_eq!(session.project, "/work/demo");
        assert_eq!(session.model, "gpt-5");
        assert_eq!(session.message_count, 3);
        assert_eq!(session.token_usage.input_tokens, 14.0);
        assert_eq!(session.token_usage.output_tokens, 7.0);
        assert_eq!(session.token_usage.total_tokens, 24.0);
        assert_eq!(session.token_usage.cost, 0.04);
        assert_eq!(session.path, path.to_string_lossy());
    }

    #[test]
    fn detail_marks_only_the_last_entry_ancestry_as_active() {
        let home = TempDir::new().unwrap();
        let path = write_session(home.path());
        let detail = get_pi_session_detail_for_home(home.path(), &path.to_string_lossy()).unwrap();
        assert_eq!(detail.messages.len(), 4);
        assert_eq!(detail.messages[0].role, "user");
        assert!(detail.messages[0].is_active_branch);
        assert!(!detail
            .messages
            .iter()
            .any(|m| m.id == "assistant-1" && m.is_active_branch));
        assert!(detail
            .messages
            .iter()
            .any(|m| m.id == "compact-1" && m.is_active_branch));
    }

    #[test]
    fn delete_rejects_files_outside_pi_sessions() {
        let home = TempDir::new().unwrap();
        fs::create_dir_all(home.path().join(".pi/agent/sessions")).unwrap();
        let outside = home.path().join("outside.jsonl");
        fs::write(&outside, "{}").unwrap();
        let error =
            delete_pi_session_for_home(home.path(), &outside.to_string_lossy()).unwrap_err();
        assert!(error.contains("inside the Pi sessions directory"));
        assert!(outside.exists());
    }
}
