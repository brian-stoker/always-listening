use std::path::PathBuf;
use std::fs;
use std::time::SystemTime;
use tracing_appender::rolling::{RollingFileAppender, Rotation};
use tracing_subscriber::{fmt, EnvFilter, layer::SubscriberExt, util::SubscriberInitExt};

/// Get the platform-appropriate log directory
pub fn log_dir() -> PathBuf {
    #[cfg(target_os = "macos")]
    {
        dirs::home_dir()
            .unwrap_or_default()
            .join("Library/Logs/AlwaysListening")
    }
    #[cfg(target_os = "windows")]
    {
        dirs::data_dir()
            .unwrap_or_default()
            .join("AlwaysListening/logs")
    }
    #[cfg(not(any(target_os = "macos", target_os = "windows")))]
    {
        dirs::data_dir()
            .unwrap_or_default()
            .join("always-listening/logs")
    }
}

/// Clean up log files older than the specified number of days
fn cleanup_old_logs(log_dir: &PathBuf, max_age_days: u64) {
    let max_age_secs = max_age_days * 24 * 60 * 60;

    if let Ok(entries) = fs::read_dir(log_dir) {
        let now = SystemTime::now();

        for entry in entries.flatten() {
            let path = entry.path();

            // Only process .log files
            if path.extension().and_then(|s| s.to_str()) == Some("log") {
                if let Ok(metadata) = entry.metadata() {
                    if let Ok(modified) = metadata.modified() {
                        if let Ok(age) = now.duration_since(modified) {
                            if age.as_secs() > max_age_secs {
                                // Delete old log file
                                if let Err(e) = fs::remove_file(&path) {
                                    eprintln!("Failed to delete old log file {:?}: {}", path, e);
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

/// Initialize the logging system with daily rotation and 7-day retention
///
/// Returns a guard that must be kept alive for the lifetime of the application.
/// The guard ensures that all buffered log messages are flushed before the app exits.
pub fn init_logging() -> tracing_appender::non_blocking::WorkerGuard {
    let log_dir = log_dir();
    std::fs::create_dir_all(&log_dir).ok();

    // Clean up logs older than 7 days
    cleanup_old_logs(&log_dir, 7);

    // Daily rotating file appender
    let file_appender = RollingFileAppender::new(
        Rotation::DAILY,
        &log_dir,
        "always-listening.log",
    );
    let (non_blocking, guard) = tracing_appender::non_blocking(file_appender);

    // Environment filter - defaults to "info", override with ALWAYS_LISTENING_LOG
    let env_filter = EnvFilter::try_from_env("ALWAYS_LISTENING_LOG")
        .unwrap_or_else(|_| EnvFilter::new("info"));

    tracing_subscriber::registry()
        .with(env_filter)
        .with(fmt::layer().with_writer(non_blocking))
        .with(fmt::layer().with_writer(std::io::stdout))
        .init();

    tracing::info!("Logging initialized. Log directory: {:?}", log_dir);

    // Return the guard - caller must keep it alive
    guard
}

/// Open the log directory in the system file manager
pub fn open_log_dir() {
    let log_dir = log_dir();
    #[cfg(target_os = "macos")]
    {
        std::process::Command::new("open")
            .arg(&log_dir)
            .spawn()
            .ok();
    }
    #[cfg(target_os = "windows")]
    {
        std::process::Command::new("explorer")
            .arg(&log_dir)
            .spawn()
            .ok();
    }
    #[cfg(not(any(target_os = "macos", target_os = "windows")))]
    {
        // On Linux, try xdg-open
        std::process::Command::new("xdg-open")
            .arg(&log_dir)
            .spawn()
            .ok();
    }
}

/// Tauri command to show logs from the tray menu
#[tauri::command]
pub fn show_logs() {
    tracing::info!("Opening log directory");
    open_log_dir();
}
