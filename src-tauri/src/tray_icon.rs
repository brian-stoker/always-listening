use tauri::{image::Image, AppHandle};
use std::sync::atomic::{AtomicBool, AtomicU8, Ordering};
use std::sync::Arc;

/// EQ animation frames embedded at compile time
const EQ_FRAMES: &[&[u8]] = &[
    include_bytes!("../icons/icon-eq-1.png"),
    include_bytes!("../icons/icon-eq-2.png"),
    include_bytes!("../icons/icon-eq-3.png"),
    include_bytes!("../icons/icon-eq-4.png"),
];

/// Helper to set icon + template mode on the tray
fn set_tray_icon(app: &AppHandle, icon_bytes: &[u8], template: bool) {
    if let Some(tray) = app.tray_by_id("main-tray") {
        if let Ok(icon) = Image::from_bytes(icon_bytes) {
            let _ = tray.set_icon(Some(icon));
            let _ = tray.set_icon_as_template(template);
        }
    }
}

/// Manages the tray icon, including EQ animation during recording
pub struct TrayAnimator {
    animating: Arc<AtomicBool>,
    frame_index: Arc<AtomicU8>,
}

impl TrayAnimator {
    pub fn new() -> Self {
        Self {
            animating: Arc::new(AtomicBool::new(false)),
            frame_index: Arc::new(AtomicU8::new(0)),
        }
    }

    /// Start the EQ animation loop
    pub fn start_eq_animation(&self, app: AppHandle) {
        // If already animating, do nothing
        if self.animating.swap(true, Ordering::SeqCst) {
            return;
        }

        let animating = Arc::clone(&self.animating);
        let frame_index = Arc::clone(&self.frame_index);

        // Set template mode once at start
        if let Some(tray) = app.tray_by_id("main-tray") {
            let _ = tray.set_icon_as_template(true);
        }

        tauri::async_runtime::spawn(async move {
            while animating.load(Ordering::SeqCst) {
                let idx = frame_index.fetch_add(1, Ordering::SeqCst) as usize % EQ_FRAMES.len();

                if let Some(tray) = app.tray_by_id("main-tray") {
                    if let Ok(icon) = Image::from_bytes(EQ_FRAMES[idx]) {
                        let _ = tray.set_icon(Some(icon));
                    }
                }

                tokio::time::sleep(std::time::Duration::from_millis(150)).await;
            }
        });
    }

    /// Stop the EQ animation
    pub fn stop_animation(&self) {
        self.animating.store(false, Ordering::SeqCst);
        self.frame_index.store(0, Ordering::SeqCst);
    }

    /// Set the idle icon (horizontal line) — template mode for light/dark
    pub fn set_idle(app: &AppHandle) {
        set_tray_icon(app, include_bytes!("../icons/icon-idle.png"), true);
    }

    /// Set the active icon (line with dot) — template mode for light/dark
    pub fn set_active(app: &AppHandle) {
        set_tray_icon(app, include_bytes!("../icons/icon-active.png"), true);
    }

    /// Set the agent speaking icon — NOT template (colored icon)
    pub fn set_agent_speaking(app: &AppHandle) {
        set_tray_icon(app, include_bytes!("../icons/icon.png"), false);
    }
}
