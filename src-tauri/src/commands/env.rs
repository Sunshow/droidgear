//! Environment variable commands.
//!
//! Only `get_shell_env` remains: the GUI needs the login-shell environment to
//! seed the embedded terminal. Process-wide environment mutation
//! (`set_env_var` / `remove_env_var`) and arbitrary reads (`get_env_var`) were
//! removed — they had no callers and let a compromised webview plant
//! `LD_PRELOAD` / `DYLD_INSERT_LIBRARIES` / `PATH` for the CLIs launched later.

use std::collections::HashMap;

/// Gets environment variables from a login shell.
/// This is useful for GUI apps that don't inherit shell environment.
/// Uses non-interactive mode for faster execution.
#[tauri::command]
#[specta::specta]
pub async fn get_shell_env() -> Result<HashMap<String, String>, String> {
    #[cfg(target_os = "windows")]
    {
        // Windows doesn't have this issue, return current env
        Ok(std::env::vars().collect())
    }

    #[cfg(not(target_os = "windows"))]
    {
        use std::process::Command;

        // Run in blocking thread pool to avoid blocking the async runtime
        tauri::async_runtime::spawn_blocking(|| {
            let shell = std::env::var("SHELL").unwrap_or_else(|_| "/bin/zsh".to_string());

            // Run login shell (non-interactive) to get environment, then print it
            // Using -l (login) without -i (interactive) is much faster as it skips
            // interactive-only initialization like prompt setup
            let output = Command::new(&shell)
                .args(["-l", "-c", "env"])
                .output()
                .map_err(|e| format!("Failed to run shell: {e}"))?;

            if !output.status.success() {
                return Err("Shell command failed".to_string());
            }

            let env_str = String::from_utf8_lossy(&output.stdout);
            let env_map: HashMap<String, String> = env_str
                .lines()
                .filter_map(|line| {
                    let (key, value) = line.split_once('=')?;
                    Some((key.to_string(), value.to_string()))
                })
                .collect();

            Ok(env_map)
        })
        .await
        .map_err(|e| format!("Task join error: {e}"))?
    }
}
