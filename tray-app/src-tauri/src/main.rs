// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod logging;
mod state;

use tauri::menu::{MenuBuilder, CheckMenuItemBuilder};
use tauri::tray::TrayIconBuilder;
use tauri::{image::Image, Emitter, Manager};
use state::{AppStateManager, Mode};

/// Build the tray menu with checkmarks based on the active mode
fn build_menu(app: &tauri::AppHandle, active_mode: Option<Mode>) -> Result<tauri::menu::Menu<tauri::Wry>, Box<dyn std::error::Error>> {
    let voice_checked = active_mode == Some(Mode::VoiceToClaude);
    let dict_checked = active_mode == Some(Mode::Dictation);
    let combined_checked = active_mode == Some(Mode::Combined);

    let mode_voice = CheckMenuItemBuilder::new("Voice-to-Claude (Mode 1)")
        .id("mode-voice")
        .checked(voice_checked)
        .build(app)?;

    let mode_dictation = CheckMenuItemBuilder::new("Dictation (Mode 2)")
        .id("mode-dictation")
        .checked(dict_checked)
        .build(app)?;

    let mode_combined = CheckMenuItemBuilder::new("Combined (Mode 3)")
        .id("mode-combined")
        .checked(combined_checked)
        .build(app)?;

    let menu = MenuBuilder::new(app)
        .item(&mode_voice)
        .item(&mode_dictation)
        .item(&mode_combined)
        .separator()
        .text("preferences", "Preferences...")
        .text("show-logs", "Show Logs")
        .separator()
        .text("quit", "Quit")
        .build()?;

    Ok(menu)
}

/// Get the icon for a given app state
fn get_icon_for_state(app_state: state::AppState) -> Result<Image<'static>, Box<dyn std::error::Error>> {
    let icon = match app_state {
        state::AppState::Idle => Image::from_bytes(include_bytes!("../icons/icon-idle.png"))?,
        state::AppState::Active(_) => Image::from_bytes(include_bytes!("../icons/icon-active.png"))?,
        state::AppState::Recording(_) => Image::from_bytes(include_bytes!("../icons/icon-recording.png"))?,
    };
    Ok(icon)
}

/// Tauri command to get the current app state
#[tauri::command]
fn get_app_state(state_manager: tauri::State<'_, AppStateManager>) -> state::AppState {
    state_manager.get_state()
}

/// Handle mode toggle - update state, icon, and menu
fn handle_mode_toggle(app: &tauri::AppHandle, mode: Mode) {
    // Get the state manager
    let state_manager = app.state::<AppStateManager>();

    // Toggle the mode
    let new_state = state_manager.toggle_mode(mode);

    // Log the state change
    tracing::info!("State changed to: {:?}", new_state);

    // Update the tray icon
    if let Some(tray) = app.tray_by_id("main-tray") {
        // Update icon based on new state
        if let Ok(new_icon) = get_icon_for_state(new_state) {
            if let Err(e) = tray.set_icon(Some(new_icon)) {
                tracing::error!("Failed to set tray icon: {}", e);
            }
        }

        // Rebuild menu with updated checkmarks
        if let Ok(new_menu) = build_menu(app, new_state.active_mode()) {
            if let Err(e) = tray.set_menu(Some(new_menu)) {
                tracing::error!("Failed to set tray menu: {}", e);
            }
        }
    }

    // Emit events for the frontend
    let _ = app.emit("state-changed", &new_state);
    let _ = app.emit("mode-changed", &new_state.active_mode());
}

fn main() {
    // Initialize logging - guard must be kept alive for app lifetime
    let _log_guard = logging::init_logging();

    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![logging::show_logs, get_app_state])
        .manage(AppStateManager::new())
        .setup(|app| {
            // Get the main window
            let window = app.get_webview_window("main").unwrap();

            // Hide the window initially since we're a tray app
            window.hide().unwrap();

            // Load the initial idle icon
            let icon_idle = Image::from_bytes(include_bytes!("../icons/icon-idle.png"))?;

            // Build the initial menu (no mode active)
            let menu = build_menu(app, None)?;

            // Create the tray icon with an ID so we can update it later
            let _tray = TrayIconBuilder::new()
                .id("main-tray")
                .icon(icon_idle)
                .menu(&menu)
                .show_menu_on_left_click(true)
                .on_menu_event(|app, event| {
                    match event.id().as_ref() {
                        "quit" => {
                            app.exit(0);
                        }
                        "mode-voice" => {
                            handle_mode_toggle(app, Mode::VoiceToClaude);
                        }
                        "mode-dictation" => {
                            handle_mode_toggle(app, Mode::Dictation);
                        }
                        "mode-combined" => {
                            handle_mode_toggle(app, Mode::Combined);
                        }
                        "preferences" => {
                            tracing::info!("Preferences selected");
                        }
                        "show-logs" => {
                            logging::open_log_dir();
                        }
                        id => {
                            tracing::debug!("menu item clicked: {}", id);
                        }
                    }
                })
                .build(app)?;

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
