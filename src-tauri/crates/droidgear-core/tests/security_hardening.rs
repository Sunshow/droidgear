//! Regression tests for the 2026-11 security audit fixes.
//!
//! Each test pins one of the traversal/permission issues from
//! `docs/security/audit-2026-11-21.zh.md` so it cannot silently return.

use droidgear_core::{channel, claude_settings_files, droid_settings_files, specs, storage};
use std::fs;
use std::path::{Path, PathBuf};
use tempfile::TempDir;

fn write_file(path: &Path, contents: &str) {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).unwrap();
    }
    fs::write(path, contents).unwrap();
}

fn specs_dir(home: &Path) -> PathBuf {
    home.join(".factory").join("specs")
}

// ============================================================================
// F1 — spec paths must stay inside the specs directory
// ============================================================================

#[test]
fn spec_commands_reject_dot_dot_escape() {
    let home = TempDir::new().unwrap();
    fs::create_dir_all(specs_dir(home.path())).unwrap();
    let victim = home.path().join("victim.md");
    write_file(&victim, "original");

    let escape = specs_dir(home.path())
        .join("..")
        .join("..")
        .join("victim.md");
    let escape = escape.to_str().unwrap();

    assert!(specs::read_spec_for_home(home.path(), escape).is_err());
    assert!(specs::update_spec_for_home(home.path(), escape, "pwned").is_err());
    assert!(specs::delete_spec_for_home(home.path(), escape).is_err());
    assert!(specs::rename_spec_for_home(home.path(), escape, "moved").is_err());

    assert_eq!(fs::read_to_string(&victim).unwrap(), "original");
}

#[test]
fn spec_commands_reject_files_outside_the_specs_directory() {
    let home = TempDir::new().unwrap();
    fs::create_dir_all(specs_dir(home.path())).unwrap();
    let secret = home.path().join("id_rsa");
    write_file(&secret, "PRIVATE KEY");

    assert!(specs::read_spec_for_home(home.path(), secret.to_str().unwrap()).is_err());
    assert!(specs::delete_spec_for_home(home.path(), secret.to_str().unwrap()).is_err());
    assert_eq!(fs::read_to_string(&secret).unwrap(), "PRIVATE KEY");
}

#[test]
fn spec_commands_still_work_inside_the_specs_directory() {
    let home = TempDir::new().unwrap();
    let dir = specs_dir(home.path());
    fs::create_dir_all(&dir).unwrap();
    let first = dir.join("first.md");
    write_file(&first, "hello");

    let listed = specs::list_specs_for_home(home.path()).unwrap();
    assert_eq!(listed.len(), 1);

    let read = specs::read_spec_for_home(home.path(), first.to_str().unwrap()).unwrap();
    assert_eq!(read.content, "hello");

    let updated = specs::update_spec_for_home(home.path(), first.to_str().unwrap(), "updated");
    assert_eq!(updated.unwrap().content, "updated");

    let renamed = specs::rename_spec_for_home(home.path(), first.to_str().unwrap(), "second")
        .expect("rename inside the specs directory");
    assert_eq!(renamed.name, "second.md");

    specs::delete_spec_for_home(home.path(), &renamed.path).unwrap();
    assert!(!dir.join("second.md").exists());
}

// ============================================================================
// F2 — settings-file names must be a single path component
// ============================================================================

#[test]
fn claude_settings_read_rejects_traversal() {
    let home = TempDir::new().unwrap();
    write_file(
        &home.path().join(".claude/settings.json"),
        r#"{"env":{"ANTHROPIC_AUTH_TOKEN":"sk-live-secret"}}"#,
    );

    assert!(claude_settings_files::get_settings_path_by_name_for_home(
        home.path(),
        "../../.claude/settings"
    )
    .is_err());
    assert!(claude_settings_files::read_settings_file_for_home(
        home.path(),
        "../../.claude/settings"
    )
    .is_err());
    assert!(claude_settings_files::set_active_settings_file_for_home(
        home.path(),
        Some("../../.claude/settings".to_string())
    )
    .is_err());
}

#[test]
fn droid_settings_path_lookup_rejects_traversal() {
    let home = TempDir::new().unwrap();
    write_file(&home.path().join("outside.json"), "{}");

    assert!(
        droid_settings_files::get_settings_path_by_name_for_home(home.path(), "../../outside")
            .is_err()
    );
    assert!(droid_settings_files::set_active_settings_file_for_home(
        home.path(),
        Some("../../outside".to_string())
    )
    .is_err());
}

#[test]
fn droid_active_name_is_ignored_when_it_escapes() {
    let home = TempDir::new().unwrap();
    write_file(
        &home.path().join(".droidgear/settings.json"),
        r#"{"droidSettingsActiveFile":"../../.factory/settings"}"#,
    );
    write_file(&home.path().join(".factory/settings.json"), "{}");

    let active = droid_settings_files::get_active_settings_path_for_home(home.path()).unwrap();
    assert_eq!(
        active,
        droid_settings_files::global_settings_path_for_home(home.path())
    );
}

// ============================================================================
// F3 — channel ids must be a single path component
// ============================================================================

#[test]
fn channel_id_traversal_is_rejected() {
    let home = TempDir::new().unwrap();
    fs::create_dir_all(home.path().join(".droidgear")).unwrap();

    assert!(channel::save_channel_api_key_for_home(home.path(), "../../pwned", "sk-evil").is_err());
    assert!(channel::get_channel_api_key_for_home(home.path(), "../../pwned").is_err());
    assert!(channel::delete_channel_credentials_for_home(home.path(), "../../pwned").is_err());
    assert!(!home.path().join("pwned.json").exists());
}

#[test]
fn channel_auth_round_trips_for_a_generated_id() {
    let home = TempDir::new().unwrap();
    let id = "6f1b6a1c-2c4e-4b8f-9c3d-1a2b3c4d5e6f";

    channel::save_channel_api_key_for_home(home.path(), id, "sk-live").unwrap();
    let key = channel::get_channel_api_key_for_home(home.path(), id).unwrap();
    assert_eq!(key.as_deref(), Some("sk-live"));

    channel::delete_channel_credentials_for_home(home.path(), id).unwrap();
    assert!(channel::get_channel_api_key_for_home(home.path(), id)
        .unwrap()
        .is_none());
}

// ============================================================================
// F4 — secret documents stay owner-only
// ============================================================================

#[test]
fn validate_child_name_rejects_escapes() {
    for bad in ["", ".", "..", "../x", "a/b", "a\\b", "/abs", "C:\\abs"] {
        assert!(
            storage::validate_child_name(bad).is_err(),
            "{bad:?} should be rejected"
        );
    }
    for good in ["a", "profile-b", "my profile", "6f1b6a1c-2c4e"] {
        assert!(
            storage::validate_child_name(good).is_ok(),
            "{good:?} should be accepted"
        );
    }
}

#[cfg(unix)]
#[test]
fn atomic_write_preserves_an_existing_owner_only_mode() {
    use std::os::unix::fs::PermissionsExt;

    let dir = TempDir::new().unwrap();
    let path = dir.path().join("settings.json");
    write_file(&path, "{}");
    fs::set_permissions(&path, fs::Permissions::from_mode(0o600)).unwrap();

    storage::atomic_write(&path, b"{\"api_key\":\"sk-live\"}").unwrap();

    let mode = fs::metadata(&path).unwrap().permissions().mode() & 0o777;
    assert_eq!(mode, 0o600);
}

#[cfg(unix)]
#[test]
fn atomic_write_private_creates_an_owner_only_file() {
    use std::os::unix::fs::PermissionsExt;

    let dir = TempDir::new().unwrap();
    let path = dir.path().join("credentials.json");
    storage::atomic_write_private(&path, b"{}").unwrap();

    let mode = fs::metadata(&path).unwrap().permissions().mode() & 0o777;
    assert_eq!(mode, 0o600);
}
