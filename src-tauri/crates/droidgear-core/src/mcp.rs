//! MCP (Model Context Protocol) server configuration management (core).
//!
//! Handles reading and writing MCP server configurations in `~/.factory/mcp.json`.

use serde::{Deserialize, Serialize};
use serde_json::Value;
use specta::Type;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

use crate::paths;

// ============================================================================
// Types
// ============================================================================

/// MCP server type
#[derive(Debug, Clone, Serialize, Deserialize, Type, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum McpServerType {
    Stdio,
    Http,
}

/// MCP server configuration
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct McpServerConfig {
    /// Server type (stdio or http)
    #[serde(rename = "type")]
    pub server_type: McpServerType,
    /// Whether the server is disabled
    #[serde(default)]
    pub disabled: bool,
    /// Command to run (stdio only)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub command: Option<String>,
    /// Command arguments (stdio only)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub args: Option<Vec<String>>,
    /// Environment variables (stdio only)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub env: Option<HashMap<String, String>>,
    /// HTTP URL (http only)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
    /// HTTP headers (http only)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub headers: Option<HashMap<String, String>>,
}

/// MCP server entry with name
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct McpServer {
    /// Server name (unique identifier)
    pub name: String,
    /// Server configuration
    pub config: McpServerConfig,
}

// ============================================================================
// Helpers
// ============================================================================

fn mcp_config_path_for_home(home_dir: &Path) -> Result<PathBuf, String> {
    let config_paths = paths::load_config_paths_for_home(home_dir);
    let factory_dir = paths::get_factory_home_for_home(home_dir, &config_paths)?;

    if !factory_dir.exists() {
        std::fs::create_dir_all(&factory_dir)
            .map_err(|e| format!("Failed to create .factory directory: {e}"))?;
    }

    Ok(factory_dir.join("mcp.json"))
}

fn read_mcp_file_for_home(home_dir: &Path) -> Result<Value, String> {
    let config_path = mcp_config_path_for_home(home_dir)?;

    if !config_path.exists() {
        return Ok(serde_json::json!({ "mcpServers": {} }));
    }

    let contents = std::fs::read_to_string(&config_path)
        .map_err(|e| format!("Failed to read MCP config file: {e}"))?;

    if contents.trim().is_empty() {
        return Ok(serde_json::json!({ "mcpServers": {} }));
    }

    serde_json::from_str(&contents).map_err(|e| format!("Failed to parse MCP config JSON: {e}"))
}

fn write_mcp_file_for_home(home_dir: &Path, config: &Value) -> Result<(), String> {
    let config_path = mcp_config_path_for_home(home_dir)?;

    let actual_path = if config_path.is_symlink() {
        std::fs::canonicalize(&config_path)
            .map_err(|e| format!("Failed to resolve symlink: {e}"))?
    } else {
        config_path
    };

    let temp_path = actual_path.with_extension("tmp");
    let json_content = serde_json::to_string_pretty(config)
        .map_err(|e| format!("Failed to serialize MCP config: {e}"))?;

    std::fs::write(&temp_path, json_content)
        .map_err(|e| format!("Failed to write MCP config file: {e}"))?;

    std::fs::rename(&temp_path, &actual_path).map_err(|e| {
        let _ = std::fs::remove_file(&temp_path);
        format!("Failed to finalize MCP config file: {e}")
    })?;

    Ok(())
}

fn system_home_dir() -> Result<PathBuf, String> {
    dirs::home_dir().ok_or_else(|| "Failed to get home directory".to_string())
}

// ============================================================================
// Public API
// ============================================================================

pub fn load_mcp_servers_for_home(home_dir: &Path) -> Result<Vec<McpServer>, String> {
    let config = read_mcp_file_for_home(home_dir)?;

    let servers: Vec<McpServer> = config
        .get("mcpServers")
        .and_then(|v| v.as_object())
        .map(|obj| {
            obj.iter()
                .filter_map(|(name, value)| {
                    let config: McpServerConfig = serde_json::from_value(value.clone()).ok()?;
                    Some(McpServer {
                        name: name.clone(),
                        config,
                    })
                })
                .collect()
        })
        .unwrap_or_default();

    Ok(servers)
}

pub fn load_mcp_servers() -> Result<Vec<McpServer>, String> {
    load_mcp_servers_for_home(&system_home_dir()?)
}

/// Fixed Baizhi preset: stores a nonsecret reference, never an expanded API key.
pub fn baizhi_mcp_preset() -> McpServer {
    McpServer {
        name: "baizhi".to_string(),
        config: McpServerConfig {
            server_type: McpServerType::Http,
            disabled: true,
            command: None,
            args: None,
            env: None,
            url: Some("https://agent-toolkit.app.baizhi.cloud/mcp".to_string()),
            headers: Some(HashMap::from([(
                "Authorization".to_string(),
                "Bearer ${BAIZHI_API_KEY}".to_string(),
            )])),
        },
    }
}

/// Add the brand preset without replacing any existing raw record of that name.
/// This checks the observed file contents; atomic rename is not a cross-process lock.
pub fn save_baizhi_preset_for_home(home_dir: &Path) -> Result<(), String> {
    let mut config = read_mcp_file_for_home(home_dir)?;
    let obj = config
        .as_object_mut()
        .ok_or_else(|| "MCP config must be a JSON object".to_string())?;
    let servers = obj
        .entry("mcpServers")
        .or_insert_with(|| serde_json::json!({}))
        .as_object_mut()
        .ok_or_else(|| "mcpServers must be a JSON object".to_string())?;
    if servers.contains_key("baizhi") {
        return Err("MCP server 'baizhi' already exists; not replaced".to_string());
    }
    let preset = baizhi_mcp_preset();
    let mut value = serde_json::to_value(preset.config)
        .map_err(|e| format!("Failed to serialize Baizhi preset: {e}"))?;
    // This brand-only raw option does not change the generic configuration type.
    value["oauth"] = serde_json::Value::Bool(false);
    servers.insert(preset.name, value);
    write_mcp_file_for_home(home_dir, &config)
}

pub fn save_baizhi_preset() -> Result<(), String> {
    save_baizhi_preset_for_home(&system_home_dir()?)
}

pub fn save_mcp_server_for_home(home_dir: &Path, server: McpServer) -> Result<(), String> {
    let mut config = read_mcp_file_for_home(home_dir)?;

    let server_value = serde_json::to_value(&server.config)
        .map_err(|e| format!("Failed to serialize server config: {e}"))?;

    if let Some(obj) = config.as_object_mut() {
        let mcp_servers = obj
            .entry("mcpServers")
            .or_insert_with(|| serde_json::json!({}));

        if let Some(servers_obj) = mcp_servers.as_object_mut() {
            servers_obj.insert(server.name.clone(), server_value);
        }
    }

    write_mcp_file_for_home(home_dir, &config)
}

pub fn save_mcp_server(server: McpServer) -> Result<(), String> {
    save_mcp_server_for_home(&system_home_dir()?, server)
}

pub fn delete_mcp_server_for_home(home_dir: &Path, name: &str) -> Result<(), String> {
    let mut config = read_mcp_file_for_home(home_dir)?;

    if let Some(obj) = config.as_object_mut() {
        if let Some(mcp_servers) = obj.get_mut("mcpServers") {
            if let Some(servers_obj) = mcp_servers.as_object_mut() {
                servers_obj.remove(name);
            }
        }
    }

    write_mcp_file_for_home(home_dir, &config)
}

pub fn delete_mcp_server(name: &str) -> Result<(), String> {
    delete_mcp_server_for_home(&system_home_dir()?, name)
}

pub fn toggle_mcp_server_for_home(
    home_dir: &Path,
    name: &str,
    disabled: bool,
) -> Result<(), String> {
    let mut config = read_mcp_file_for_home(home_dir)?;

    if let Some(obj) = config.as_object_mut() {
        if let Some(mcp_servers) = obj.get_mut("mcpServers") {
            if let Some(servers_obj) = mcp_servers.as_object_mut() {
                if let Some(server) = servers_obj.get_mut(name) {
                    if let Some(server_obj) = server.as_object_mut() {
                        server_obj.insert("disabled".to_string(), serde_json::json!(disabled));
                    }
                } else {
                    return Err(format!("Server not found: {name}"));
                }
            }
        }
    }

    write_mcp_file_for_home(home_dir, &config)
}

pub fn toggle_mcp_server(name: &str, disabled: bool) -> Result<(), String> {
    toggle_mcp_server_for_home(&system_home_dir()?, name, disabled)
}
