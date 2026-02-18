//! Tauri application library entry point.
//!
//! This module serves as the main entry point for the Tauri application.
//! Command implementations are organized in the `commands` module,
//! and shared types are in the `types` module.

mod bindings;
mod commands;
mod types;
mod utils;

use std::sync::Mutex;
use tauri::{Manager, WebviewWindowBuilder};

/// Application entry point. Sets up all plugins and initializes the app.
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let builder = bindings::generate_bindings();

    // Export TypeScript bindings in debug builds
    #[cfg(debug_assertions)]
    bindings::export_ts_bindings();

    // Build with common plugins
    let mut app_builder = tauri::Builder::default();

    // Single instance plugin must be registered FIRST
    // When user tries to open a second instance, focus the existing window instead
    #[cfg(desktop)]
    {
        app_builder = app_builder.plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
            if let Some(window) = app.get_webview_window("main") {
                let _ = window.set_focus();
                let _ = window.unminimize();
            }
        }));
    }

    // Window state plugin - saves/restores window position and size
    // Exclude VISIBLE flag to prevent white screen on startup - frontend will show window after loading
    #[cfg(desktop)]
    {
        use tauri_plugin_window_state::StateFlags;
        app_builder = app_builder.plugin(
            tauri_plugin_window_state::Builder::new()
                .with_state_flags(StateFlags::all() & !StateFlags::VISIBLE)
                .build(),
        );
    }

    // Updater plugin for in-app updates
    #[cfg(desktop)]
    {
        app_builder = app_builder.plugin(tauri_plugin_updater::Builder::new().build());
    }

    app_builder = app_builder
        .plugin(tauri_plugin_process::init())
        .plugin(tauri_plugin_notification::init())
        .plugin(
            tauri_plugin_log::Builder::new()
                // Use Debug level in development, Info in production
                .level(if cfg!(debug_assertions) {
                    log::LevelFilter::Debug
                } else {
                    log::LevelFilter::Info
                })
                .targets([
                    // Always log to stdout for development
                    tauri_plugin_log::Target::new(tauri_plugin_log::TargetKind::Stdout),
                    // Log to webview console for development
                    tauri_plugin_log::Target::new(tauri_plugin_log::TargetKind::Webview),
                    // Log to system logs on macOS (appears in Console.app)
                    #[cfg(target_os = "macos")]
                    tauri_plugin_log::Target::new(tauri_plugin_log::TargetKind::LogDir {
                        file_name: None,
                    }),
                ])
                .build(),
        );

    app_builder
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_persisted_scope::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_clipboard_manager::init())
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_os::init())
        .plugin(tauri_plugin_pty::init())
        .plugin(tauri_plugin_system_fonts::init())
        .manage(commands::specs::SpecsWatcherState(Mutex::new(None)))
        .manage(commands::sessions::SessionsWatcherState(Mutex::new(None)))
        .setup(|app| {
            log::info!("Application starting up");
            log::debug!(
                "App handle initialized for package: {}",
                app.package_info().name
            );

            // Create main window manually to control state restoration timing
            // Window state plugin will automatically restore position after build()
            // Since window is created with visible: false, no "jump" effect will occur
            #[cfg(desktop)]
            {
                // Check if window state file exists
                let app_dir = app.path().app_data_dir()?;
                let state_file = app_dir.join("window-state.json");
                let has_saved_state = state_file.exists();

                // Create window with visible: false initially
                // Window state plugin will restore position automatically after build()
                let window = WebviewWindowBuilder::new(app, "main", tauri::WebviewUrl::App("/".into()))
                    .title("DroidGear")
                    .inner_size(1150.0, 700.0)
                    .min_inner_size(1150.0, 700.0)
                    .resizable(true)
                    .fullscreen(false)
                    .maximized(false)
                    .visible(false);  // Start hidden, frontend will show it

                // Platform-specific settings
                #[cfg(target_os = "windows")]
                let window = window
                    .decorations(false)
                    .transparent(false);

                #[cfg(target_os = "macos")]
                let window = window
                    .decorations(false)
                    .transparent(true);

                #[cfg(target_os = "linux")]
                let window = window
                    .decorations(true)
                    .transparent(false);

                let _window = window.build()?;

                // Give window state plugin time to restore state
                // This happens while window is still invisible
                std::thread::sleep(std::time::Duration::from_millis(50));

                if !has_saved_state {
                    // First launch - center the window
                    if let Some(window) = app.get_webview_window("main") {
                        let _ = window.center();
                        log::debug!("First launch: window centered");
                    }
                } else {
                    log::debug!("Window state restored by plugin");
                }
            }

            // Set up global shortcut plugin (without any shortcuts - we register them separately)
            #[cfg(desktop)]
            {
                use tauri_plugin_global_shortcut::Builder;

                app.handle().plugin(Builder::new().build())?;
            }

            // NOTE: Application menu is built from JavaScript for i18n support
            // See src/lib/menu.ts for the menu implementation

            Ok(())
        })
        .invoke_handler(builder.invoke_handler())
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
