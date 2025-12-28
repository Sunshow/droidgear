//! Specs management commands.
//!
//! Handles reading spec files from ~/.factory/specs directory.

use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

/// Spec file metadata
#[derive(Debug, Clone, Serialize, Deserialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct SpecFile {
    /// File name (e.g., "2025-12-18-ui.md")
    pub name: String,
    /// Full path to the file
    pub path: String,
    /// File content
    pub content: String,
    /// Last modified timestamp in milliseconds
    pub modified_at: f64,
}

/// Gets the path to the specs directory (~/.factory/specs).
fn get_specs_dir() -> Result<PathBuf, String> {
    let home_dir = dirs::home_dir().ok_or_else(|| "Failed to get home directory".to_string())?;
    Ok(home_dir.join(".factory").join("specs"))
}

/// Lists all spec files from ~/.factory/specs directory.
/// Returns files sorted by modification time (newest first).
#[tauri::command]
#[specta::specta]
pub async fn list_specs() -> Result<Vec<SpecFile>, String> {
    log::debug!("Listing specs from ~/.factory/specs");
    let specs_dir = get_specs_dir()?;

    if !specs_dir.exists() {
        log::info!("Specs directory does not exist: {specs_dir:?}");
        return Ok(Vec::new());
    }

    let mut specs: Vec<SpecFile> = Vec::new();

    let entries = fs::read_dir(&specs_dir).map_err(|e| {
        log::error!("Failed to read specs directory: {e}");
        format!("Failed to read specs directory: {e}")
    })?;

    for entry in entries {
        let entry = match entry {
            Ok(e) => e,
            Err(e) => {
                log::warn!("Failed to read directory entry: {e}");
                continue;
            }
        };

        let path = entry.path();

        // Only process markdown files
        if path.extension().and_then(|s| s.to_str()) != Some("md") {
            continue;
        }

        let metadata = match fs::metadata(&path) {
            Ok(m) => m,
            Err(e) => {
                log::warn!("Failed to read file metadata for {path:?}: {e}");
                continue;
            }
        };

        if !metadata.is_file() {
            continue;
        }

        let modified_at = metadata
            .modified()
            .ok()
            .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
            .map(|d| d.as_millis() as f64)
            .unwrap_or(0.0);

        let content = match fs::read_to_string(&path) {
            Ok(c) => c,
            Err(e) => {
                log::warn!("Failed to read file content for {path:?}: {e}");
                continue;
            }
        };

        let name = path
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or("")
            .to_string();

        specs.push(SpecFile {
            name,
            path: path.to_string_lossy().to_string(),
            content,
            modified_at,
        });
    }

    // Sort by modified time, newest first
    specs.sort_by(|a, b| b.modified_at.partial_cmp(&a.modified_at).unwrap_or(std::cmp::Ordering::Equal));

    log::info!("Found {} spec files", specs.len());
    Ok(specs)
}

/// Reads a single spec file by path.
#[tauri::command]
#[specta::specta]
pub async fn read_spec(path: String) -> Result<SpecFile, String> {
    log::debug!("Reading spec file: {path}");

    let path_buf = PathBuf::from(&path);

    if !path_buf.exists() {
        return Err("Spec file not found".to_string());
    }

    let metadata = fs::metadata(&path_buf).map_err(|e| {
        log::error!("Failed to read file metadata: {e}");
        format!("Failed to read file metadata: {e}")
    })?;

    let modified_at = metadata
        .modified()
        .ok()
        .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
        .map(|d| d.as_millis() as f64)
        .unwrap_or(0.0);

    let content = fs::read_to_string(&path_buf).map_err(|e| {
        log::error!("Failed to read file content: {e}");
        format!("Failed to read file content: {e}")
    })?;

    let name = path_buf
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("")
        .to_string();

    Ok(SpecFile {
        name,
        path,
        content,
        modified_at,
    })
}
