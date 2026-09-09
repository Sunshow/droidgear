//! Pi session history commands (Tauri wrappers + watcher).

use notify::{Config, RecommendedWatcher, RecursiveMode, Watcher};
use std::sync::Mutex;
use tauri::{AppHandle, Emitter, Manager};

pub use droidgear_core::pi_sessions::{PiSessionDetail, PiSessionSummary};

#[tauri::command]
#[specta::specta]
pub async fn list_pi_sessions() -> Result<Vec<PiSessionSummary>, String> {
    tauri::async_runtime::spawn_blocking(droidgear_core::pi_sessions::list_pi_sessions)
        .await
        .map_err(|e| format!("Pi session listing task failed: {e}"))?
}

#[tauri::command]
#[specta::specta]
pub async fn get_pi_session_detail(session_path: String) -> Result<PiSessionDetail, String> {
    tauri::async_runtime::spawn_blocking(move || {
        droidgear_core::pi_sessions::get_pi_session_detail(&session_path)
    })
    .await
    .map_err(|e| format!("Pi session detail task failed: {e}"))?
}

#[tauri::command]
#[specta::specta]
pub async fn delete_pi_session(session_path: String) -> Result<(), String> {
    tauri::async_runtime::spawn_blocking(move || {
        droidgear_core::pi_sessions::delete_pi_session(&session_path)
    })
    .await
    .map_err(|e| format!("Pi session deletion task failed: {e}"))?
}

pub struct PiSessionsWatcherState(pub Mutex<Option<RecommendedWatcher>>);

#[tauri::command]
#[specta::specta]
pub async fn start_pi_sessions_watcher(app: AppHandle) -> Result<(), String> {
    let sessions_dir = droidgear_core::pi_sessions::pi_sessions_dir()?;
    if !sessions_dir.exists() {
        return Ok(());
    }

    let app_handle = app.clone();
    let watcher = RecommendedWatcher::new(
        move |res: Result<notify::Event, notify::Error>| {
            if let Ok(event) = res {
                use notify::EventKind;
                if matches!(
                    event.kind,
                    EventKind::Create(_) | EventKind::Modify(_) | EventKind::Remove(_)
                ) {
                    let _ = app_handle.emit("pi-sessions-changed", ());
                }
            }
        },
        Config::default(),
    )
    .map_err(|e| format!("Failed to create Pi session watcher: {e}"))?;

    let state = app.state::<PiSessionsWatcherState>();
    let mut guard = state.0.lock().map_err(|e| format!("Lock error: {e}"))?;
    if let Some(mut old_watcher) = guard.take() {
        let _ = old_watcher.unwatch(&sessions_dir);
    }
    let mut watcher = watcher;
    watcher
        .watch(&sessions_dir, RecursiveMode::Recursive)
        .map_err(|e| format!("Failed to watch Pi sessions: {e}"))?;
    *guard = Some(watcher);
    Ok(())
}

#[tauri::command]
#[specta::specta]
pub async fn stop_pi_sessions_watcher(app: AppHandle) -> Result<(), String> {
    let sessions_dir = droidgear_core::pi_sessions::pi_sessions_dir()?;
    let state = app.state::<PiSessionsWatcherState>();
    let mut guard = state.0.lock().map_err(|e| format!("Lock error: {e}"))?;
    if let Some(mut watcher) = guard.take() {
        let _ = watcher.unwatch(&sessions_dir);
    }
    Ok(())
}
