//! Specs management (core).
//!
//! Handles reading spec files from Factory specs directory.

use serde::{Deserialize, Serialize};
use specta::Type;
use std::fs;
use std::path::{Path, PathBuf};

use crate::paths;

/// Spec file metadata
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
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

fn specs_dir_for_home(home_dir: &Path) -> Result<PathBuf, String> {
    let config_paths = paths::load_config_paths_for_home(home_dir);
    let factory_dir = paths::get_factory_home_for_home(home_dir, &config_paths)?;
    Ok(factory_dir.join("specs"))
}

fn system_home_dir() -> Result<PathBuf, String> {
    dirs::home_dir().ok_or_else(|| "Failed to get home directory".to_string())
}

pub fn list_specs_for_home(home_dir: &Path) -> Result<Vec<SpecFile>, String> {
    let specs_dir = specs_dir_for_home(home_dir)?;

    if !specs_dir.exists() {
        return Ok(Vec::new());
    }

    let mut specs: Vec<SpecFile> = Vec::new();
    let entries =
        fs::read_dir(&specs_dir).map_err(|e| format!("Failed to read specs directory: {e}"))?;

    for entry in entries {
        let entry = match entry {
            Ok(e) => e,
            Err(_) => continue,
        };

        let path = entry.path();
        if path.extension().and_then(|s| s.to_str()) != Some("md") {
            continue;
        }

        let metadata = match fs::metadata(&path) {
            Ok(m) => m,
            Err(_) => continue,
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
            Err(_) => continue,
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

    specs.sort_by(|a, b| {
        b.modified_at
            .partial_cmp(&a.modified_at)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    Ok(specs)
}

pub fn list_specs() -> Result<Vec<SpecFile>, String> {
    list_specs_for_home(&system_home_dir()?)
}

pub fn read_spec(path: &str) -> Result<SpecFile, String> {
    let path_buf = PathBuf::from(path);
    if !path_buf.exists() {
        return Err("Spec file not found".to_string());
    }

    let metadata =
        fs::metadata(&path_buf).map_err(|e| format!("Failed to read file metadata: {e}"))?;
    let modified_at = metadata
        .modified()
        .ok()
        .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
        .map(|d| d.as_millis() as f64)
        .unwrap_or(0.0);

    let content =
        fs::read_to_string(&path_buf).map_err(|e| format!("Failed to read file content: {e}"))?;

    let name = path_buf
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("")
        .to_string();

    Ok(SpecFile {
        name,
        path: path.to_string(),
        content,
        modified_at,
    })
}

pub fn rename_spec_for_home(
    home_dir: &Path,
    old_path: &str,
    new_name: &str,
) -> Result<SpecFile, String> {
    let specs_dir = specs_dir_for_home(home_dir)?;
    let old_path_buf = PathBuf::from(old_path);

    if !old_path_buf.starts_with(&specs_dir) {
        return Err("Invalid file path".to_string());
    }

    if !old_path_buf.exists() {
        return Err("Spec file not found".to_string());
    }

    let new_name = new_name.trim();
    if new_name.is_empty() {
        return Err("File name cannot be empty".to_string());
    }

    let new_name = if new_name.ends_with(".md") {
        new_name.to_string()
    } else {
        format!("{new_name}.md")
    };

    if new_name.contains('/') || new_name.contains('\\') {
        return Err("Invalid file name".to_string());
    }

    let new_path = specs_dir.join(&new_name);

    if new_path.exists() && new_path != old_path_buf {
        return Err("A file with this name already exists".to_string());
    }

    fs::rename(&old_path_buf, &new_path).map_err(|e| format!("Failed to rename file: {e}"))?;

    read_spec(&new_path.to_string_lossy())
}

pub fn rename_spec(old_path: &str, new_name: &str) -> Result<SpecFile, String> {
    rename_spec_for_home(&system_home_dir()?, old_path, new_name)
}

pub fn delete_spec_for_home(home_dir: &Path, path: &str) -> Result<(), String> {
    let specs_dir = specs_dir_for_home(home_dir)?;
    let path_buf = PathBuf::from(path);

    if !path_buf.starts_with(&specs_dir) {
        return Err("Invalid file path".to_string());
    }
    if !path_buf.exists() {
        return Err("Spec file not found".to_string());
    }

    fs::remove_file(&path_buf).map_err(|e| format!("Failed to delete file: {e}"))?;
    Ok(())
}

pub fn delete_spec(path: &str) -> Result<(), String> {
    delete_spec_for_home(&system_home_dir()?, path)
}

pub fn update_spec_for_home(
    home_dir: &Path,
    path: &str,
    content: &str,
) -> Result<SpecFile, String> {
    let specs_dir = specs_dir_for_home(home_dir)?;
    let path_buf = PathBuf::from(path);

    if !path_buf.starts_with(&specs_dir) {
        return Err("Invalid file path".to_string());
    }
    if !path_buf.exists() {
        return Err("Spec file not found".to_string());
    }

    crate::storage::atomic_write(&path_buf, content.as_bytes())?;
    read_spec(path)
}

pub fn update_spec(path: &str, content: &str) -> Result<SpecFile, String> {
    update_spec_for_home(&system_home_dir()?, path, content)
}
