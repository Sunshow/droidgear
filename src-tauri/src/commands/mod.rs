//! Tauri command handlers organized by domain.
//!
//! Each submodule contains related commands and their helper functions.
//! Import specific commands via their submodule (e.g., `commands::preferences::greet`).

pub mod channel;
pub mod codex;
pub mod config;
pub mod connectivity;
pub mod env;
pub mod hermes;
pub mod mcp;
pub mod notifications;
pub mod openclaw;
pub mod opencode;
pub mod paths;
pub mod pi;
pub mod preferences;
pub mod recovery;
pub mod sessions;
pub mod specs;
pub mod updater;
