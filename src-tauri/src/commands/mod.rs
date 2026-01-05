//! Tauri command handlers organized by domain.
//!
//! Each submodule contains related commands and their helper functions.
//! Import specific commands via their submodule (e.g., `commands::preferences::greet`).

pub mod channel;
pub mod config;
pub mod env;
pub mod mcp;
pub mod notifications;
pub mod opencode;
pub mod preferences;
pub mod quick_pane;
pub mod recovery;
pub mod specs;
