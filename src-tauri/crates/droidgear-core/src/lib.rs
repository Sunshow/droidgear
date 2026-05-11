pub mod channel;
pub mod claude;
pub mod claude_runtime;
pub mod codex;
pub mod codex_runtime;
pub mod connectivity;
pub mod droid_runtime;
pub mod droid_settings_files;
pub mod factory_auth_profiles;
pub mod factory_settings;
pub mod hermes;
pub mod json;
pub mod mcp;
pub mod openclaw;
pub mod opencode;
pub mod paths;
pub mod pi;
pub mod sessions;
pub mod specs;
pub mod storage;

pub fn core_version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}
