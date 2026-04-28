pub mod channel;
pub mod codex;
pub mod connectivity;
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
