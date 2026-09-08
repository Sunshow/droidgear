//! Codex sessions management commands (Tauri wrappers + watcher).
//!
//! Listing/parsing logic lives in `droidgear-core`. The watcher remains in the Tauri layer.

use notify::{Config, RecommendedWatcher, RecursiveMode, Watcher};
use std::path::PathBuf;
use std::sync::Mutex;
use tauri::{AppHandle, Emitter, Manager};

pub use droidgear_core::codex_sessions::{
    CodexSessionDetail, CodexSessionProvider, CodexSessionSummary,
};

fn codex_sessions_dir() -> Result<PathBuf, String> {
    Ok(droidgear_core::paths::get_codex_home()?.join("sessions"))
}

/// Lists all Codex sessions from `<codex home>/sessions`.
#[tauri::command]
#[specta::specta]
pub async fn list_codex_sessions() -> Result<Vec<CodexSessionSummary>, String> {
    droidgear_core::codex_sessions::list_codex_sessions()
}

/// Lists all Codex model providers aggregated from every configured profile.
#[tauri::command]
#[specta::specta]
pub async fn list_codex_session_providers() -> Result<Vec<CodexSessionProvider>, String> {
    droidgear_core::codex_sessions::list_codex_session_providers()
}

/// Rewrites the `model_provider` of a session file's `session_meta` line.
#[tauri::command]
#[specta::specta]
pub async fn set_codex_session_provider(
    session_path: String,
    provider_id: String,
) -> Result<(), String> {
    droidgear_core::codex_sessions::set_codex_session_provider(&session_path, &provider_id)
}

/// Gets detailed Codex session information including messages.
#[tauri::command]
#[specta::specta]
pub async fn get_codex_session_detail(session_path: String) -> Result<CodexSessionDetail, String> {
    droidgear_core::codex_sessions::get_codex_session_detail(&session_path)
}

/// Deletes a Codex session by removing its .jsonl file.
#[tauri::command]
#[specta::specta]
pub async fn delete_codex_session(session_path: String) -> Result<(), String> {
    droidgear_core::codex_sessions::delete_codex_session(&session_path)
}

/// State for the codex sessions file watcher
pub struct CodexSessionsWatcherState(pub Mutex<Option<RecommendedWatcher>>);

/// Starts watching the codex sessions directory for changes.
#[tauri::command]
#[specta::specta]
pub async fn start_codex_sessions_watcher(app: AppHandle) -> Result<(), String> {
    let sessions_dir = codex_sessions_dir()?;

    if !sessions_dir.exists() {
        return Ok(());
    }

    let app_handle = app.clone();

    let watcher = RecommendedWatcher::new(
        move |res: Result<notify::Event, notify::Error>| {
            if let Ok(event) = res {
                use notify::EventKind;
                match event.kind {
                    EventKind::Create(_) | EventKind::Modify(_) | EventKind::Remove(_) => {
                        let _ = app_handle.emit("codex-sessions-changed", ());
                    }
                    _ => {}
                }
            }
        },
        Config::default(),
    )
    .map_err(|e| format!("Failed to create watcher: {e}"))?;

    let state = app.state::<CodexSessionsWatcherState>();
    let mut guard = state.0.lock().map_err(|e| format!("Lock error: {e}"))?;

    if let Some(mut old_watcher) = guard.take() {
        let _ = old_watcher.unwatch(&sessions_dir);
    }

    let mut watcher = watcher;
    watcher
        .watch(&sessions_dir, RecursiveMode::Recursive)
        .map_err(|e| format!("Failed to watch directory: {e}"))?;

    *guard = Some(watcher);
    Ok(())
}

/// Stops watching the codex sessions directory.
#[tauri::command]
#[specta::specta]
pub async fn stop_codex_sessions_watcher(app: AppHandle) -> Result<(), String> {
    let sessions_dir = codex_sessions_dir()?;
    let state = app.state::<CodexSessionsWatcherState>();
    let mut guard = state.0.lock().map_err(|e| format!("Lock error: {e}"))?;

    if let Some(mut watcher) = guard.take() {
        let _ = watcher.unwatch(&sessions_dir);
    }

    Ok(())
}
