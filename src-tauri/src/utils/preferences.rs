use crate::types::AppPreferences;
use tauri::Manager;

pub fn load_preferences_from_path(prefs_path: &std::path::Path) -> Result<AppPreferences, String> {
    if !prefs_path.exists() {
        return Ok(AppPreferences::default());
    }

    let contents = std::fs::read_to_string(prefs_path)
        .map_err(|e| format!("Failed to read preferences: {e}"))?;
    serde_json::from_str(&contents).map_err(|e| format!("Failed to parse preferences: {e}"))
}

pub fn load_preferences(app: &tauri::AppHandle) -> Result<AppPreferences, String> {
    let prefs_path = {
        let app_data_dir = app
            .path()
            .app_data_dir()
            .map_err(|e| format!("Failed to get app data dir: {e}"))?;
        app_data_dir.join("preferences.json")
    };

    load_preferences_from_path(&prefs_path)
}

#[cfg(test)]
mod tests {
    use super::load_preferences_from_path;

    #[test]
    fn load_preferences_returns_default_when_file_is_missing() {
        let path = std::env::temp_dir().join(format!(
            "droidgear-missing-prefs-{}.json",
            std::process::id()
        ));
        let _ = std::fs::remove_file(&path);
        let prefs = load_preferences_from_path(&path).unwrap();

        assert!(prefs.preferred_terminal.is_none());
        assert_eq!(prefs.theme, "system");
    }

    #[test]
    fn load_preferences_reads_existing_json_payload() {
        let path = std::env::temp_dir().join(format!(
            "droidgear-test-preferences-{}.json",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        std::fs::write(
            &path,
            r#"{
              "theme": "system",
              "preferred_terminal": "terminal"
            }"#,
        )
        .unwrap();

        let prefs = load_preferences_from_path(&path).unwrap();
        let _ = std::fs::remove_file(&path);

        assert_eq!(prefs.preferred_terminal.as_deref(), Some("terminal"));
        assert_eq!(prefs.theme, "system");
    }
}
