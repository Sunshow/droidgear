//! Dsh generic profile configuration commands.
//!
//! Unlike [`super::dsh`] (provider/credential-specific), these commands expose
//! the whole `cordis.patch.yml` surface: a composed read-only snapshot plus
//! targeted writes for one entry at a time.

pub use droidgear_core::dsh_config::{DshConfigSnapshot, DshConfigWriteResult};

/// Read the composed DSH configuration snapshot for a profile.
///
/// `profile` selects an official runtime profile (`desktop` / `web`); when
/// omitted the default resolution is used (`desktop`, then `web`).
#[tauri::command]
#[specta::specta]
pub async fn read_dsh_config_snapshot(
    profile: Option<String>,
) -> Result<DshConfigSnapshot, String> {
    droidgear_core::dsh_config::read_dsh_config_snapshot(profile)
}

/// Replace one entry's complete `config` in the profile patch.
///
/// `config: null` removes the override so the bundle layer applies again.
/// `expectedHash` is the `patchHash` from the last snapshot; a mismatch is
/// refused with a `DSH_CONFIG_CONFLICT` error.
#[tauri::command]
#[specta::specta]
pub async fn save_dsh_config_entry(
    profile: Option<String>,
    entry_id: String,
    config: Option<serde_json::Value>,
    expected_hash: Option<String>,
) -> Result<DshConfigWriteResult, String> {
    droidgear_core::dsh_config::save_dsh_config_entry(profile, &entry_id, config, expected_hash)
}

/// Enable or disable one entry in the profile patch layer.
#[tauri::command]
#[specta::specta]
pub async fn set_dsh_config_entry_disabled(
    profile: Option<String>,
    entry_id: String,
    disabled: bool,
    expected_hash: Option<String>,
) -> Result<DshConfigWriteResult, String> {
    droidgear_core::dsh_config::set_dsh_config_entry_disabled(
        profile,
        &entry_id,
        disabled,
        expected_hash,
    )
}
