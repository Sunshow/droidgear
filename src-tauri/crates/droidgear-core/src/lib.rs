pub mod channel;
pub mod channel_export;
pub mod claude;
pub mod claude_runtime;
pub mod claude_settings_files;
pub mod codex;
pub mod codex_auth_profiles;
pub mod codex_runtime;
pub mod codex_sessions;
pub mod connectivity;
pub mod copilot;
pub mod copilot_runtime;
pub mod droid_runtime;
pub mod droid_settings_files;
pub mod dsh;
pub mod factory_auth_profiles;
pub mod factory_settings;
pub mod hermes;
pub mod json;
pub mod mcp;
pub mod omp;
pub mod openclaw;
pub mod opencode;
pub mod paths;
pub mod pi;
pub mod pi_sessions;
pub mod sessions;
pub mod specs;
pub mod storage;
pub mod trusted_folders;

pub fn core_version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}

/// Helpers shared by this crate's unit tests.
#[cfg(test)]
pub(crate) mod test_support {
    /// Rewrites `\` separators to `/` so path assertions comparing
    /// production-built paths (native separators) with fixtures that embed
    /// literal `/` in `join` arguments behave identically on every platform.
    pub(crate) fn slashes(path: impl AsRef<str>) -> String {
        path.as_ref().replace('\\', "/")
    }
}
