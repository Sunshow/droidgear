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

    // Window state plugin - saves/restores window position and size.
    // Pre-check the on-disk state file before the plugin reads it: corrupted
    // or absurdly oversized files (e.g. saved with a now-disconnected monitor
    // or after an OS DPI glitch) get quarantined so we fall back to defaults.
    // Exclude VISIBLE so the frontend can show the window after loading.
    // Exclude DECORATIONS because we always force `decorations(false)` in
    // code (custom titlebar); persisting it lets a corrupted file confuse
    // the restore path.
    // Exclude FULLSCREEN because a stale fullscreen flag combined with our
    // custom titlebar can leave users unable to exit fullscreen.
    #[cfg(desktop)]
    {
        use tauri_plugin_window_state::StateFlags;
        commands::window::precheck_state_file();
        let flags = StateFlags::all()
            & !StateFlags::VISIBLE
            & !StateFlags::DECORATIONS
            & !StateFlags::FULLSCREEN;
        app_builder = app_builder.plugin(
            tauri_plugin_window_state::Builder::new()
                .with_state_flags(flags)
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
        .manage(commands::codex_sessions::CodexSessionsWatcherState(
            Mutex::new(None),
        ))
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
                // Check if a window state file exists. The plugin uses
                // `app_config_dir`/.window-state.json (note the leading dot);
                // matching that exactly avoids a stale "first launch" branch
                // when the plugin has, in fact, restored saved geometry.
                let config_dir = app.path().app_config_dir()?;
                let state_file = config_dir.join(".window-state.json");
                let has_saved_state = state_file.exists();

                // Create window with visible: false initially
                // Window state plugin will restore position automatically after build()
                let window =
                    WebviewWindowBuilder::new(app, "main", tauri::WebviewUrl::App("/".into()))
                        .title("DroidGear")
                        .inner_size(1150.0, 700.0)
                        .min_inner_size(1150.0, 700.0)
                        .resizable(true)
                        .fullscreen(false)
                        .maximized(false)
                        .visible(false); // Start hidden, frontend will show it

                // Platform-specific settings
                #[cfg(target_os = "windows")]
                let window = window.decorations(false).transparent(false);

                #[cfg(target_os = "macos")]
                let window = window.decorations(false).transparent(true);

                #[cfg(target_os = "linux")]
                let window = window.decorations(false).transparent(false);

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
                } else if let Some(window) = app.get_webview_window("main") {
                    // Saved state restored - validate the resulting geometry
                    // is still on a visible monitor. If a secondary display
                    // was unplugged or the file lied about size, snap back.
                    let _ = commands::window::validate_and_clamp_window(&window);
                    log::debug!("Window state restored by plugin");
                }
            }

            // Set up global shortcut plugin and register Cmd/Ctrl+Shift+0 as
            // a system-wide escape hatch: even when the window is offscreen
            // and the menubar is unreachable, this resets it to defaults.
            #[cfg(desktop)]
            {
                use tauri_plugin_global_shortcut::{
                    Builder, Code, GlobalShortcutExt, Modifiers, Shortcut, ShortcutState,
                };

                #[cfg(target_os = "macos")]
                let primary_mod = Modifiers::SUPER;
                #[cfg(not(target_os = "macos"))]
                let primary_mod = Modifiers::CONTROL;

                let reset_shortcut =
                    Shortcut::new(Some(primary_mod | Modifiers::SHIFT), Code::Digit0);
                let reset_shortcut_for_handler = reset_shortcut;

                app.handle().plugin(
                    Builder::new()
                        .with_handler(move |app, shortcut, event| {
                            if event.state != ShortcutState::Pressed {
                                return;
                            }
                            if *shortcut == reset_shortcut_for_handler {
                                let app_handle = app.clone();
                                tauri::async_runtime::spawn(async move {
                                    if let Err(err) =
                                        commands::window::reset_window_state(app_handle).await
                                    {
                                        log::warn!(
                                            "Global shortcut reset_window_state failed: {err}"
                                        );
                                    }
                                });
                            }
                        })
                        .build(),
                )?;

                if let Err(err) = app.handle().global_shortcut().register(reset_shortcut) {
                    log::warn!("Failed to register window reset shortcut: {err}");
                }
            }

            // NOTE: Application menu is built from JavaScript for i18n support
            // See src/lib/menu.ts for the menu implementation

            Ok(())
        })
        .invoke_handler(builder.invoke_handler())
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
