//! Sessions management (core).
//!
//! Handles reading session files from Factory sessions directory.

use serde::{Deserialize, Serialize};
use serde_json::Value;
use specta::Type;
use std::fs;
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};

use crate::paths;

/// Session project (directory containing sessions)
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct SessionProject {
    /// Directory name (e.g., "-Users-sunshow-GIT-sunshow-quickcast-api")
    pub name: String,
    /// Full path to the directory
    pub path: String,
    /// Number of sessions in this project
    pub session_count: u32,
    /// Last modified timestamp in milliseconds
    pub modified_at: f64,
}

/// Token usage statistics
#[derive(Debug, Clone, Serialize, Deserialize, Type, Default)]
#[serde(rename_all = "camelCase")]
pub struct TokenUsage {
    pub input_tokens: f64,
    pub output_tokens: f64,
    pub cache_creation_tokens: f64,
    pub cache_read_tokens: f64,
    pub thinking_tokens: f64,
}

/// Session summary for list view
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct SessionSummary {
    /// Session UUID
    pub id: String,
    /// Session title
    pub title: String,
    /// Project directory name
    pub project: String,
    /// Model used
    pub model: String,
    /// Last modified timestamp in milliseconds
    pub modified_at: f64,
    /// Token usage
    pub token_usage: TokenUsage,
    /// Full path to the session files (without extension)
    pub path: String,
}

/// Message content block
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct ContentBlock {
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
pub struct SessionMessage {
    pub id: String,
    pub role: String,
    pub content: Vec<ContentBlock>,
    pub timestamp: String,
}

/// Session detail with messages
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct SessionDetail {
    pub id: String,
    pub title: String,
    pub project: String,
    pub model: String,
    pub cwd: String,
    pub modified_at: f64,
    pub token_usage: TokenUsage,
    pub messages: Vec<SessionMessage>,
}

fn sessions_dir_for_home(home_dir: &Path) -> Result<PathBuf, String> {
    let config_paths = paths::load_config_paths_for_home(home_dir);
    let factory_dir = paths::get_factory_home_for_home(home_dir, &config_paths)?;
    Ok(factory_dir.join("sessions"))
}

fn system_home_dir() -> Result<PathBuf, String> {
    dirs::home_dir().ok_or_else(|| "Failed to get home directory".to_string())
}

pub fn list_session_projects_for_home(home_dir: &Path) -> Result<Vec<SessionProject>, String> {
    let sessions_dir = sessions_dir_for_home(home_dir)?;

    if !sessions_dir.exists() {
        return Ok(Vec::new());
    }

    let mut projects: Vec<SessionProject> = Vec::new();

    let entries = fs::read_dir(&sessions_dir)
        .map_err(|e| format!("Failed to read sessions directory: {e}"))?;

    for entry in entries.flatten() {
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }

        let name = path
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or("")
            .to_string();

        // Count .jsonl files (sessions)
        let session_count = fs::read_dir(&path)
            .map(|entries| {
                entries
                    .flatten()
                    .filter(|e| {
                        e.path()
                            .extension()
                            .and_then(|s| s.to_str())
                            .is_some_and(|s| s == "jsonl")
                    })
                    .count() as u32
            })
            .unwrap_or(0);

        if session_count == 0 {
            continue;
        }

        let modified_at = fs::metadata(&path)
            .ok()
            .and_then(|m| m.modified().ok())
            .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
            .map(|d| d.as_millis() as f64)
            .unwrap_or(0.0);

        projects.push(SessionProject {
            name,
            path: path.to_string_lossy().to_string(),
            session_count,
            modified_at,
        });
    }

    projects.sort_by(|a, b| {
        b.modified_at
            .partial_cmp(&a.modified_at)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    Ok(projects)
}

pub fn list_session_projects() -> Result<Vec<SessionProject>, String> {
    list_session_projects_for_home(&system_home_dir()?)
}

pub fn list_sessions_for_home(
    home_dir: &Path,
    project: Option<&str>,
) -> Result<Vec<SessionSummary>, String> {
    let sessions_dir = sessions_dir_for_home(home_dir)?;

    if !sessions_dir.exists() {
        return Ok(Vec::new());
    }

    let mut sessions: Vec<SessionSummary> = Vec::new();

    let project_dirs: Vec<PathBuf> = if let Some(proj) = project {
        vec![sessions_dir.join(proj)]
    } else {
        fs::read_dir(&sessions_dir)
            .map_err(|e| format!("Failed to read sessions directory: {e}"))?
            .flatten()
            .map(|e| e.path())
            .filter(|p| p.is_dir())
            .collect()
    };

    for project_dir in project_dirs {
        if !project_dir.exists() || !project_dir.is_dir() {
            continue;
        }

        let project_name = project_dir
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or("")
            .to_string();

        let entries = match fs::read_dir(&project_dir) {
            Ok(e) => e,
            Err(_) => continue,
        };

        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) != Some("jsonl") {
                continue;
            }

            let session_id = path
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("")
                .to_string();

            let settings_path = project_dir.join(format!("{session_id}.settings.json"));

            let (model, token_usage) = if settings_path.exists() {
                match fs::read_to_string(&settings_path) {
                    Ok(content) => {
                        let json: Value = serde_json::from_str(&content).unwrap_or_default();
                        let model = json["model"].as_str().unwrap_or("unknown").to_string();
                        let tu = TokenUsage {
                            input_tokens: json["tokenUsage"]["inputTokens"].as_f64().unwrap_or(0.0),
                            output_tokens: json["tokenUsage"]["outputTokens"]
                                .as_f64()
                                .unwrap_or(0.0),
                            cache_creation_tokens: json["tokenUsage"]["cacheCreationTokens"]
                                .as_f64()
                                .unwrap_or(0.0),
                            cache_read_tokens: json["tokenUsage"]["cacheReadTokens"]
                                .as_f64()
                                .unwrap_or(0.0),
                            thinking_tokens: json["tokenUsage"]["thinkingTokens"]
                                .as_f64()
                                .unwrap_or(0.0),
                        };
                        (model, tu)
                    }
                    Err(_) => ("unknown".to_string(), TokenUsage::default()),
                }
            } else {
                ("unknown".to_string(), TokenUsage::default())
            };

            // Read first line of jsonl for session title
            let title = match fs::File::open(&path) {
                Ok(file) => {
                    let reader = BufReader::new(file);
                    if let Some(Ok(line)) = reader.lines().next() {
                        let json: Value = serde_json::from_str(&line).unwrap_or_default();
                        json["sessionTitle"]
                            .as_str()
                            .or_else(|| json["title"].as_str())
                            .unwrap_or("Untitled")
                            .to_string()
                    } else {
                        "Untitled".to_string()
                    }
                }
                Err(_) => "Untitled".to_string(),
            };

            let modified_at = fs::metadata(&path)
                .ok()
                .and_then(|m| m.modified().ok())
                .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
                .map(|d| d.as_millis() as f64)
                .unwrap_or(0.0);

            sessions.push(SessionSummary {
                id: session_id.clone(),
                title,
                project: project_name.clone(),
                model,
                modified_at,
                token_usage,
                path: path.with_extension("").to_string_lossy().to_string(),
            });
        }
    }

    sessions.sort_by(|a, b| {
        b.modified_at
            .partial_cmp(&a.modified_at)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    Ok(sessions)
}

pub fn list_sessions(project: Option<&str>) -> Result<Vec<SessionSummary>, String> {
    list_sessions_for_home(&system_home_dir()?, project)
}

pub fn get_session_detail_for_home(
    _home_dir: &Path,
    session_path: &str,
) -> Result<SessionDetail, String> {
    let jsonl_path = PathBuf::from(format!("{session_path}.jsonl"));
    let settings_path = PathBuf::from(format!("{session_path}.settings.json"));

    if !jsonl_path.exists() {
        return Err("Session file not found".to_string());
    }

    let project = jsonl_path
        .parent()
        .and_then(|p| p.file_name())
        .and_then(|s| s.to_str())
        .unwrap_or("")
        .to_string();

    let (model, token_usage) = if settings_path.exists() {
        match fs::read_to_string(&settings_path) {
            Ok(content) => {
                let json: Value = serde_json::from_str(&content).unwrap_or_default();
                let model = json["model"].as_str().unwrap_or("unknown").to_string();
                let tu = TokenUsage {
                    input_tokens: json["tokenUsage"]["inputTokens"].as_f64().unwrap_or(0.0),
                    output_tokens: json["tokenUsage"]["outputTokens"].as_f64().unwrap_or(0.0),
                    cache_creation_tokens: json["tokenUsage"]["cacheCreationTokens"]
                        .as_f64()
                        .unwrap_or(0.0),
                    cache_read_tokens: json["tokenUsage"]["cacheReadTokens"]
                        .as_f64()
                        .unwrap_or(0.0),
                    thinking_tokens: json["tokenUsage"]["thinkingTokens"].as_f64().unwrap_or(0.0),
                };
                (model, tu)
            }
            Err(_) => ("unknown".to_string(), TokenUsage::default()),
        }
    } else {
        ("unknown".to_string(), TokenUsage::default())
    };

    let modified_at = fs::metadata(&jsonl_path)
        .ok()
        .and_then(|m| m.modified().ok())
        .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
        .map(|d| d.as_millis() as f64)
        .unwrap_or(0.0);

    let file =
        fs::File::open(&jsonl_path).map_err(|e| format!("Failed to open session file: {e}"))?;
    let reader = BufReader::new(file);

    let mut id = String::new();
    let mut title = String::from("Untitled");
    let mut cwd = String::new();
    let mut messages: Vec<SessionMessage> = Vec::new();

    for line in reader.lines() {
        let line = match line {
            Ok(l) => l,
            Err(_) => continue,
        };

        let json: Value = match serde_json::from_str(&line) {
            Ok(j) => j,
            Err(_) => continue,
        };

        let msg_type = json["type"].as_str().unwrap_or("");

        match msg_type {
            "session_start" => {
                id = json["id"].as_str().unwrap_or("").to_string();
                title = json["sessionTitle"]
                    .as_str()
                    .or_else(|| json["title"].as_str())
                    .unwrap_or("Untitled")
                    .to_string();
                cwd = json["cwd"].as_str().unwrap_or("").to_string();
            }
            "message" => {
                let msg_id = json["id"].as_str().unwrap_or("").to_string();
                let timestamp = json["timestamp"].as_str().unwrap_or("").to_string();
                let role = json["message"]["role"].as_str().unwrap_or("").to_string();

                let content_arr = json["message"]["content"].as_array();
                let mut content_blocks: Vec<ContentBlock> = Vec::new();

                if let Some(arr) = content_arr {
                    for item in arr {
                        let content_type = item["type"].as_str().unwrap_or("text").to_string();
                        let text = item["text"].as_str().map(|s| s.to_string());
                        let thinking = item["thinking"].as_str().map(|s| s.to_string());

                        // Skip tool_use and tool_result for cleaner display
                        if content_type == "tool_use" || content_type == "tool_result" {
                            continue;
                        }

                        content_blocks.push(ContentBlock {
                            content_type,
                            text,
                            thinking,
                        });
                    }
                }

                if !content_blocks.is_empty() {
                    messages.push(SessionMessage {
                        id: msg_id,
                        role,
                        content: content_blocks,
                        timestamp,
                    });
                }
            }
            _ => {}
        }
    }

    Ok(SessionDetail {
        id,
        title,
        project,
        model,
        cwd,
        modified_at,
        token_usage,
        messages,
    })
}

pub fn get_session_detail(session_path: &str) -> Result<SessionDetail, String> {
    let _home_dir = system_home_dir()?;
    get_session_detail_for_home(&_home_dir, session_path)
}

pub fn delete_session_for_home(home_dir: &Path, session_path: &str) -> Result<(), String> {
    let sessions_dir = sessions_dir_for_home(home_dir)?;
    // Safety guard: only delete files inside the Droid sessions directory.
    let jsonl_path = crate::storage::canonicalize_within(
        &sessions_dir,
        Path::new(&format!("{session_path}.jsonl")),
    )
    .map_err(|_| "Session file not found".to_string())?;
    if !jsonl_path.is_file() {
        return Err("Session file not found".to_string());
    }
    let settings_path = jsonl_path.with_extension("settings.json");

    fs::remove_file(&jsonl_path).map_err(|e| format!("Failed to delete session: {e}"))?;
    if settings_path.exists() {
        let _ = fs::remove_file(&settings_path);
    }

    Ok(())
}

pub fn delete_session(session_path: &str) -> Result<(), String> {
    delete_session_for_home(&system_home_dir()?, session_path)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn write_session(home: &Path, id: &str) -> PathBuf {
        let dir = home.join(".factory/sessions/-work-repo");
        fs::create_dir_all(&dir).unwrap();
        fs::write(dir.join(format!("{id}.jsonl")), "{}\n").unwrap();
        fs::write(dir.join(format!("{id}.settings.json")), "{}").unwrap();
        dir.join(id)
    }

    #[test]
    fn delete_removes_session_and_settings_files() {
        let temp = TempDir::new().unwrap();
        let base = write_session(temp.path(), "abc");

        delete_session_for_home(temp.path(), &base.to_string_lossy()).unwrap();

        assert!(!base.with_extension("jsonl").exists());
        assert!(!base.with_extension("settings.json").exists());
    }

    #[test]
    fn delete_rejects_paths_outside_sessions_dir() {
        let temp = TempDir::new().unwrap();
        write_session(temp.path(), "abc");
        let outside = temp.path().join("notes.jsonl");
        fs::write(&outside, "x").unwrap();

        let direct = temp.path().join("notes");
        let traversal = temp.path().join(".factory/sessions/../../notes");
        for path in [direct, traversal] {
            let err = delete_session_for_home(temp.path(), &path.to_string_lossy()).unwrap_err();
            assert!(err.contains("not found"));
        }
        assert!(outside.exists());
    }
}
