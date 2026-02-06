// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod agent_runner;
mod audio;
mod config;
mod docker;
mod ha_client;
mod logging;
mod process_manager;
mod setup;
mod state;
mod transcription;
mod tray_icon;

use config::AgentConfig;
use state::AppStateManager;
use std::sync::Arc;
use tauri::menu::{CheckMenuItemBuilder, MenuBuilder};
use tauri::tray::TrayIconBuilder;
use tauri::{image::Image, Emitter, Listener, Manager};
use tokio::sync::{watch, Mutex as TokioMutex};

/// Determine the next agent ID in the cycle: Off → agents[0] → agents[1] → ... → Off
fn next_agent_in_cycle(current: Option<&str>, agents: &[AgentConfig]) -> Option<String> {
    if agents.is_empty() {
        return None;
    }
    match current {
        None => Some(agents[0].id.clone()),
        Some(id) => {
            if let Some(pos) = agents.iter().position(|a| a.id == id) {
                if pos + 1 < agents.len() {
                    Some(agents[pos + 1].id.clone())
                } else {
                    None // wrap back to Off
                }
            } else {
                Some(agents[0].id.clone())
            }
        }
    }
}

/// Build the tray menu with checkmarks based on the active agent
fn build_menu(
    app: &tauri::AppHandle,
    agents: &[AgentConfig],
    active_agent: Option<&str>,
) -> Result<tauri::menu::Menu<tauri::Wry>, Box<dyn std::error::Error>> {
    let mut builder = MenuBuilder::new(app);

    for agent in agents {
        let checked = active_agent == Some(agent.id.as_str());
        let label = if let Some(ref icon) = agent.icon {
            format!("{} {}", icon, agent.name)
        } else {
            agent.name.clone()
        };
        let item = CheckMenuItemBuilder::new(&label)
            .id(format!("agent-{}", agent.id))
            .checked(checked)
            .build(app)?;
        builder = builder.item(&item);
    }

    let menu = builder
        .separator()
        .text("preferences", "Preferences...")
        .text("show-logs", "Show Logs")
        .separator()
        .text("quit", "Quit")
        .build()?;

    Ok(menu)
}

/// Update tray icon based on app state
fn update_tray_icon_for_state(app: &tauri::AppHandle, app_state: &state::AppState) {
    let animator = app.state::<tray_icon::TrayAnimator>();
    match app_state {
        state::AppState::Idle => {
            animator.stop_animation();
            tray_icon::TrayAnimator::set_idle(app);
        }
        state::AppState::Active(_) => {
            animator.stop_animation();
            tray_icon::TrayAnimator::set_active(app);
        }
        state::AppState::Recording(_) => {
            animator.start_eq_animation(app.clone());
        }
        state::AppState::AgentSpeaking(_) => {
            animator.stop_animation();
            tray_icon::TrayAnimator::set_agent_speaking(app);
        }
    }
}

/// Tauri command to get the current app state
#[tauri::command]
fn get_app_state(state_manager: tauri::State<'_, AppStateManager>) -> state::AppState {
    state_manager.get_state()
}

/// Pipeline shutdown sender
struct PipelineShutdown(std::sync::Mutex<Option<watch::Sender<bool>>>);

/// Handle pipeline sub-state events (recording started, agent speaking, etc.)
fn handle_pipeline_event(app: &tauri::AppHandle, event: &str) {
    let state_manager = app.state::<AppStateManager>();
    let new_state = match event {
        "recording" => state_manager.set_recording(),
        "agent-speaking" => state_manager.set_agent_speaking(),
        "active" => state_manager.set_active(),
        _ => return,
    };
    tracing::debug!("Pipeline event '{}' -> state {:?}", event, new_state);
    update_tray_icon_for_state(app, &new_state);
    let _ = app.emit("state-changed", &new_state);
}

/// Handle agent toggle — update state, icon, menu, and start/stop pipelines
fn handle_agent_toggle(app: &tauri::AppHandle, agent_id: &str) {
    let state_manager = app.state::<AppStateManager>();
    let new_state = state_manager.toggle_agent(agent_id);

    tracing::info!("State changed to: {:?}", new_state);

    // Stop any running pipeline
    stop_current_pipeline(app);

    // Start pipeline if an agent is now active
    if let Some(active_id) = new_state.active_agent() {
        start_pipeline(app, active_id);
    }

    update_tray_and_emit(app, &new_state);
}

/// Set a specific agent (or None for Off)
fn handle_agent_set(app: &tauri::AppHandle, target: Option<&str>) {
    stop_current_pipeline(app);

    let state_manager = app.state::<AppStateManager>();
    let new_state = state_manager.set_agent(target);
    tracing::info!("Agent set to: {:?}", new_state);

    if let Some(active_id) = new_state.active_agent() {
        start_pipeline(app, active_id);
    }

    update_tray_and_emit(app, &new_state);
}

/// Stop the current pipeline if one is running
fn stop_current_pipeline(app: &tauri::AppHandle) {
    let shutdown = app.state::<PipelineShutdown>();
    let mut guard = shutdown.0.lock().unwrap();
    if let Some(tx) = guard.take() {
        let _ = tx.send(true);
    }
}

/// Update tray menu + emit events
fn update_tray_and_emit(app: &tauri::AppHandle, new_state: &state::AppState) {
    update_tray_icon_for_state(app, new_state);

    let config_state = app.state::<Arc<TokioMutex<config::Config>>>();
    // We need agents list for the menu — clone config synchronously via try_lock
    let agents = config_state
        .try_lock()
        .map(|c| c.agents.clone())
        .unwrap_or_default();

    if let Some(tray) = app.tray_by_id("main-tray") {
        if let Ok(new_menu) = build_menu(app, &agents, new_state.active_agent()) {
            if let Err(e) = tray.set_menu(Some(new_menu)) {
                tracing::error!("Failed to set tray menu: {}", e);
            }
        }
    }

    let _ = app.emit("state-changed", new_state);
    let _ = app.emit(
        "agent-changed",
        new_state.active_agent().unwrap_or(""),
    );
}

/// Spawn pipeline task for the given agent ID
fn start_pipeline(app: &tauri::AppHandle, agent_id: &str) {
    let (shutdown_tx, shutdown_rx) = watch::channel(false);
    {
        let shutdown = app.state::<PipelineShutdown>();
        let mut guard = shutdown.0.lock().unwrap();
        *guard = Some(shutdown_tx);
    }

    let config = app.state::<Arc<TokioMutex<config::Config>>>();
    let config = config.inner().clone();
    let app_handle = app.clone();

    let agent_id_owned = agent_id.to_string();
    let app_handle_recovery = app.clone();

    // Look up the agent config
    let agent_config = {
        let cfg = config.try_lock().unwrap();
        cfg.agents.iter().find(|a| a.id == agent_id_owned).cloned()
    };

    let agent_config = match agent_config {
        Some(c) => c,
        None => {
            tracing::error!("Agent '{}' not found in config", agent_id_owned);
            return;
        }
    };

    let agent_name = agent_config.name.clone();
    let agent_id_for_recovery = agent_id_owned.clone();

    let task = tauri::async_runtime::spawn(async move {
        let mut runner =
            agent_runner::AgentRunner::new(agent_config, config, shutdown_rx)
                .with_app_handle(app_handle);
        if let Err(e) = runner.run().await {
            tracing::error!("Agent '{}' error: {}", agent_name, e);
        }
    });

    // Monitor for panics and reset state on exit
    tauri::async_runtime::spawn(async move {
        match task.await {
            Ok(()) => {
                tracing::info!("Agent '{}' task finished", agent_id_for_recovery);
            }
            Err(e) => {
                tracing::error!("Agent '{}' panicked: {}", agent_id_for_recovery, e);
                let _ = app_handle_recovery.emit(
                    "pipeline-error",
                    format!("Agent '{}' panicked: {}", agent_id_for_recovery, e),
                );
            }
        }
        // Reset state to idle when pipeline exits
        let state_manager = app_handle_recovery.state::<AppStateManager>();
        let current = state_manager.get_state();
        if current.active_agent().is_some() {
            let idle = state_manager.set_agent(None);
            update_tray_icon_for_state(&app_handle_recovery, &idle);

            let config_state = app_handle_recovery.state::<Arc<TokioMutex<config::Config>>>();
            let agents = config_state
                .try_lock()
                .map(|c| c.agents.clone())
                .unwrap_or_default();
            if let Some(tray) = app_handle_recovery.tray_by_id("main-tray") {
                if let Ok(menu) = build_menu(&app_handle_recovery, &agents, None) {
                    let _ = tray.set_menu(Some(menu));
                }
            }
            let _ = app_handle_recovery.emit("state-changed", &idle);
            let _ = app_handle_recovery.emit("agent-changed", "");
        }
    });
}

fn main() {
    let _log_guard = logging::init_logging();

    tauri::Builder::default()
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .invoke_handler(tauri::generate_handler![
            logging::show_logs,
            logging::get_recent_logs,
            get_app_state,
            config::get_config,
            config::update_config,
            config::get_agents,
            config::update_agents,
            ha_client::test_ha_connection,
            docker::check_docker,
            docker::check_ha_container_status,
            docker::start_ha,
            docker::setup_ha,
            setup::check_dependencies,
            setup::is_first_run_check,
        ])
        .manage(AppStateManager::new())
        .manage(Arc::new(TokioMutex::new(config::Config::load())))
        .manage(Arc::new(process_manager::ProcessManager::new()))
        .manage(PipelineShutdown(std::sync::Mutex::new(None)))
        .manage(tray_icon::TrayAnimator::new())
        .setup(|app| {
            let window = app.get_webview_window("main").unwrap();
            window.hide().unwrap();

            // Listen for pipeline state events
            let app_handle_for_events = app.handle().clone();
            app.listen("pipeline-event", move |event| {
                if let Some(payload) = event
                    .payload()
                    .strip_prefix('"')
                    .and_then(|s| s.strip_suffix('"'))
                {
                    handle_pipeline_event(&app_handle_for_events, payload);
                }
            });

            let icon_idle = Image::from_bytes(include_bytes!("../icons/icon-idle.png"))?;

            // Build the initial menu from config agents
            let handle = app.handle();
            let config_state = handle.state::<Arc<TokioMutex<config::Config>>>();
            let agents = config_state
                .try_lock()
                .map(|c| c.agents.clone())
                .unwrap_or_default();
            let menu = build_menu(handle, &agents, None)?;

            let _tray = TrayIconBuilder::with_id("main-tray")
                .icon(icon_idle)
                .icon_as_template(true)
                .menu(&menu)
                .show_menu_on_left_click(true)
                .on_menu_event(|app: &tauri::AppHandle, event| {
                    let id = event.id().as_ref();
                    match id {
                        "quit" => {
                            app.exit(0);
                        }
                        "preferences" => {
                            if let Some(win) = app.get_webview_window("preferences") {
                                let _ = win.show();
                                let _ = win.set_focus();
                            }
                        }
                        "show-logs" => {
                            logging::open_log_dir();
                        }
                        _ => {
                            if let Some(agent_id) = id.strip_prefix("agent-") {
                                handle_agent_toggle(app, agent_id);
                            } else {
                                tracing::debug!("menu item clicked: {}", id);
                            }
                        }
                    }
                })
                .build(app)?;

            // Start real-time log streaming
            logging::start_log_stream(app.handle().clone());

            // Register global shortcut for agent cycling (F19)
            {
                use tauri_plugin_global_shortcut::{GlobalShortcutExt, Shortcut};

                let shortcut: Shortcut = "F19".parse().expect("Failed to parse F19 shortcut");

                app.global_shortcut()
                    .on_shortcut(shortcut, move |app_handle, _shortcut, event| {
                        if event.state
                            == tauri_plugin_global_shortcut::ShortcutState::Pressed
                        {
                            let state_manager = app_handle.state::<AppStateManager>();
                            let current_id = state_manager
                                .get_state()
                                .active_agent()
                                .map(|s| s.to_string());

                            let config_state =
                                app_handle.state::<Arc<TokioMutex<config::Config>>>();
                            let agents = config_state
                                .try_lock()
                                .map(|c| c.agents.clone())
                                .unwrap_or_default();

                            let next = next_agent_in_cycle(
                                current_id.as_deref(),
                                &agents,
                            );
                            tracing::info!(
                                "Hotkey pressed: {:?} -> {:?}",
                                current_id,
                                next
                            );
                            handle_agent_set(app_handle, next.as_deref());
                        }
                    })
                    .expect("Failed to register global shortcut");

                tracing::info!("Global shortcut F19 registered for agent cycling");
            }

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
