//! MCP server configuration management (Tauri wrappers).
//!
//! Core logic lives in `droidgear-core`.

pub use droidgear_core::mcp::McpServer;

/// Loads all MCP servers from ~/.factory/mcp.json
#[tauri::command]
#[specta::specta]
pub async fn load_mcp_servers() -> Result<Vec<McpServer>, String> {
    droidgear_core::mcp::load_mcp_servers()
}

/// Saves an MCP server (creates or updates)
#[tauri::command]
#[specta::specta]
pub async fn save_mcp_server(server: McpServer) -> Result<(), String> {
    droidgear_core::mcp::save_mcp_server(server)
}

/// Saves the fixed, disabled Baizhi preset without accepting a key or replacing a name.
#[tauri::command]
#[specta::specta]
pub async fn save_baizhi_preset() -> Result<(), String> {
    droidgear_core::mcp::save_baizhi_preset()
}

/// Deletes an MCP server by name
#[tauri::command]
#[specta::specta]
pub async fn delete_mcp_server(name: String) -> Result<(), String> {
    droidgear_core::mcp::delete_mcp_server(&name)
}

/// Toggles an MCP server's disabled state
#[tauri::command]
#[specta::specta]
pub async fn toggle_mcp_server(name: String, disabled: bool) -> Result<(), String> {
    droidgear_core::mcp::toggle_mcp_server(&name, disabled)
}
