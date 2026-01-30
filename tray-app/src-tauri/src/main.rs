// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod logging;

use tauri::menu::MenuBuilder;
use tauri::tray::TrayIconBuilder;
use tauri::{image::Image, Manager};

fn main() {
    // Initialize logging - guard must be kept alive for app lifetime
    let _log_guard = logging::init_logging();

    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![logging::show_logs])
        .setup(|app| {
            // Get the main window
            let window = app.get_webview_window("main").unwrap();

            // Hide the window initially since we're a tray app
            window.hide().unwrap();

            // Load the idle icon
            let icon_idle = Image::from_bytes(include_bytes!("../icons/icon-idle.png"))?;

            // Build the context menu
            let menu = MenuBuilder::new(app)
                .text("mode-voice", "Voice-to-Claude (Mode 1)")
                .text("mode-dictation", "Dictation (Mode 2)")
                .text("mode-combined", "Combined (Mode 3)")
                .separator()
                .text("preferences", "Preferences...")
                .text("show-logs", "Show Logs")
                .separator()
                .text("quit", "Quit")
                .build()?;

            // Create the tray icon
            let _tray = TrayIconBuilder::new()
                .icon(icon_idle)
                .menu(&menu)
                .show_menu_on_left_click(true)
                .on_menu_event(|app, event| {
                    match event.id().as_ref() {
                        "quit" => {
                            app.exit(0);
                        }
                        "mode-voice" => {
                            println!("Voice-to-Claude mode selected");
                        }
                        "mode-dictation" => {
                            println!("Dictation mode selected");
                        }
                        "mode-combined" => {
                            println!("Combined mode selected");
                        }
                        "preferences" => {
                            println!("Preferences selected");
                        }
                        "show-logs" => {
                            logging::open_log_dir();
                        }
                        id => {
                            println!("menu item clicked: {}", id);
                        }
                    }
                })
                .build(app)?;

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
