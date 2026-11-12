use droidgear_core::channel::{self, Channel, ChannelType};
use tempfile::TempDir;

fn mk_channel(id: &str, channel_type: ChannelType) -> Channel {
    Channel {
        id: id.to_string(),
        name: format!("Channel {id}"),
        channel_type,
        base_url: "http://example.test".to_string(),
        enabled: true,
        created_at: 123.0,
    }
}

#[test]
fn channels_save_load_roundtrip() {
    let temp = TempDir::new().unwrap();
    let home = temp.path();

    let channels = vec![mk_channel("c1", ChannelType::General)];
    channel::save_channels_for_home(home, channels.clone()).unwrap();

    let loaded = channel::load_channels_for_home(home).unwrap();
    assert_eq!(loaded.len(), 1);
    assert_eq!(loaded[0].id, channels[0].id);
    assert_eq!(loaded[0].name, channels[0].name);
    assert_eq!(loaded[0].channel_type, channels[0].channel_type);
    assert_eq!(loaded[0].base_url, channels[0].base_url);
    assert_eq!(loaded[0].enabled, channels[0].enabled);
    assert_eq!(loaded[0].created_at, channels[0].created_at);
}

#[test]
fn channels_api_key_auth_roundtrip_and_delete() {
    let temp = TempDir::new().unwrap();
    let home = temp.path();

    let ch = mk_channel("c_api", ChannelType::General);
    channel::save_channels_for_home(home, vec![ch.clone()]).unwrap();

    channel::save_channel_api_key_for_home(home, &ch.id, "sk-test").unwrap();
    let key = channel::get_channel_api_key_for_home(home, &ch.id).unwrap();
    assert_eq!(key.as_deref(), Some("sk-test"));

    channel::delete_channel_credentials_for_home(home, &ch.id).unwrap();
    let key = channel::get_channel_api_key_for_home(home, &ch.id).unwrap();
    assert_eq!(key, None);
}

#[cfg(unix)]
#[test]
fn channels_auth_file_is_owner_only_and_leaves_no_temp_files() {
    use std::os::unix::fs::PermissionsExt;

    let temp = TempDir::new().unwrap();
    let home = temp.path();
    channel::save_channel_api_key_for_home(home, "c1", "sk-test").unwrap();

    let auth_dir = home.join(".droidgear/auth");
    let auth_file = auth_dir.join("c1.json");
    let mode = std::fs::metadata(&auth_file).unwrap().permissions().mode();
    assert_eq!(mode & 0o777, 0o600);

    let names: Vec<String> = std::fs::read_dir(&auth_dir)
        .unwrap()
        .map(|entry| entry.unwrap().file_name().to_string_lossy().to_string())
        .collect();
    assert_eq!(names, vec!["c1.json".to_string()]);
}

#[test]
fn channels_credentials_roundtrip_and_delete() {
    let temp = TempDir::new().unwrap();
    let home = temp.path();

    let ch = mk_channel("c_creds", ChannelType::NewApi);
    channel::save_channels_for_home(home, vec![ch.clone()]).unwrap();

    channel::save_channel_credentials_for_home(home, &ch.id, "u", "p").unwrap();
    let creds = channel::get_channel_credentials_for_home(home, &ch.id).unwrap();
    assert_eq!(creds, Some(("u".to_string(), "p".to_string())));

    channel::delete_channel_credentials_for_home(home, &ch.id).unwrap();
    let creds = channel::get_channel_credentials_for_home(home, &ch.id).unwrap();
    assert_eq!(creds, None);
}

#[test]
fn channels_load_migrates_from_factory_settings_when_missing() {
    let temp = TempDir::new().unwrap();
    let home = temp.path();

    // Seed ~/.factory/settings.json with channels array (old location).
    let settings_path = home.join(".factory").join("settings.json");
    std::fs::create_dir_all(settings_path.parent().unwrap()).unwrap();
    std::fs::write(
        &settings_path,
        r#"{
  "channels": [
    {
      "id": "m1",
      "name": "Migrated",
      "type": "general",
      "baseUrl": "http://migrated.test",
      "enabled": true,
      "createdAt": 1
    }
  ]
}"#,
    )
    .unwrap();

    let loaded = channel::load_channels_for_home(home).unwrap();
    assert_eq!(loaded.len(), 1);
    assert_eq!(loaded[0].id, "m1");
    assert_eq!(loaded[0].name, "Migrated");
    assert_eq!(loaded[0].base_url, "http://migrated.test");

    // Migration should also write ~/.droidgear/channels.json.
    let migrated_path = home.join(".droidgear").join("channels.json");
    assert!(migrated_path.exists());
}
