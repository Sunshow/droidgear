//! OMP session history commands (Tauri wrappers + watcher).

use notify::{Config, RecommendedWatcher, RecursiveMode, Watcher};
use std::sync::Mutex;
use tauri::{AppHandle, Emitter, Manager};

pub use droidgear_core::omp_sessions::{OmpSessionDetail, OmpSessionSummary};

#[tauri::command]
#[specta::specta]
pub async fn list_omp_sessions() -> Result<Vec<OmpSessionSummary>, String> {
    tauri::async_runtime::spawn_blocking(droidgear_core::omp_sessions::list_omp_sessions)
        .await
        .map_err(|e| format!("OMP session listing task failed: {e}"))?
}

#[tauri::command]
#[specta::specta]
pub async fn get_omp_session_detail(session_path: String) -> Result<OmpSessionDetail, String> {
    tauri::async_runtime::spawn_blocking(move || {
        droidgear_core::omp_sessions::get_omp_session_detail(&session_path)
    })
    .await
    .map_err(|e| format!("OMP session detail task failed: {e}"))?
}

#[tauri::command]
#[specta::specta]
pub async fn delete_omp_session(session_path: String) -> Result<(), String> {
    tauri::async_runtime::spawn_blocking(move || {
        droidgear_core::omp_sessions::delete_omp_session(&session_path)
    })
    .await
    .map_err(|e| format!("OMP session deletion task failed: {e}"))?
}

pub struct OmpSessionsWatcherState(pub Mutex<Option<RecommendedWatcher>>);

#[tauri::command]
#[specta::specta]
pub async fn start_omp_sessions_watcher(app: AppHandle) -> Result<(), String> {
    let sessions_dir = droidgear_core::omp_sessions::omp_sessions_dir()?;
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
                    let _ = app_handle.emit("omp-sessions-changed", ());
                }
            }
        },
        Config::default(),
    )
    .map_err(|e| format!("Failed to create OMP session watcher: {e}"))?;

    let state = app.state::<OmpSessionsWatcherState>();
    let mut guard = state.0.lock().map_err(|e| format!("Lock error: {e}"))?;
    if let Some(mut old_watcher) = guard.take() {
        let _ = old_watcher.unwatch(&sessions_dir);
    }
    let mut watcher = watcher;
    watcher
        .watch(&sessions_dir, RecursiveMode::Recursive)
        .map_err(|e| format!("Failed to watch OMP sessions: {e}"))?;
    *guard = Some(watcher);
    Ok(())
}

#[tauri::command]
#[specta::specta]
pub async fn stop_omp_sessions_watcher(app: AppHandle) -> Result<(), String> {
    let sessions_dir = droidgear_core::omp_sessions::omp_sessions_dir()?;
    let state = app.state::<OmpSessionsWatcherState>();
    let mut guard = state.0.lock().map_err(|e| format!("Lock error: {e}"))?;
    if let Some(mut watcher) = guard.take() {
        let _ = watcher.unwatch(&sessions_dir);
    }
    Ok(())
}
