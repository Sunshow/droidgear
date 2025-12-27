//! Environment variable commands.

/// Gets the value of an environment variable.
/// Returns None if the variable is not set.
#[tauri::command]
#[specta::specta]
pub fn get_env_var(name: &str) -> Option<String> {
    std::env::var(name).ok()
}

/// Sets an environment variable for the current process.
/// Note: This only affects the current process, not the system or shell.
#[tauri::command]
#[specta::specta]
pub fn set_env_var(name: &str, value: &str) {
    std::env::set_var(name, value);
}

/// Removes an environment variable from the current process.
/// Note: This only affects the current process, not the system or shell.
#[tauri::command]
#[specta::specta]
pub fn remove_env_var(name: &str) {
    std::env::remove_var(name);
}
