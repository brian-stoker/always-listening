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

/// Find the most recent log file in the log directory
fn find_current_log_file() -> Option<PathBuf> {
    let log_dir = log_dir();
    let entries = fs::read_dir(&log_dir).ok()?;

    let mut newest: Option<(PathBuf, SystemTime)> = None;

    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().and_then(|s| s.to_str()) == Some("log") {
            if let Ok(meta) = entry.metadata() {
                if let Ok(modified) = meta.modified() {
                    if newest.as_ref().map_or(true, |(_, t)| modified > *t) {
                        newest = Some((path, modified));
                    }
                }
            }
        }
    }

    newest.map(|(p, _)| p)
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

/// Tauri command: return the last N lines from the current log file
#[tauri::command]
pub fn get_recent_logs(lines: Option<usize>) -> Result<Vec<String>, String> {
    let max_lines = lines.unwrap_or(200);
    let log_file = find_current_log_file()
        .ok_or_else(|| "No log file found".to_string())?;

    let contents = fs::read_to_string(&log_file)
        .map_err(|e| format!("Failed to read log file: {}", e))?;

    let all_lines: Vec<String> = contents.lines().map(|l| l.to_string()).collect();
    let start = all_lines.len().saturating_sub(max_lines);
    Ok(all_lines[start..].to_vec())
}

/// Start a background task that tails the log file and emits `log-line` events
pub fn start_log_stream(app_handle: tauri::AppHandle) {
    use tauri::Emitter;

    tauri::async_runtime::spawn(async move {
        // Find current log file
        let log_file = match find_current_log_file() {
            Some(p) => p,
            None => {
                tracing::warn!("Log stream: no log file found yet, will retry");
                tokio::time::sleep(std::time::Duration::from_secs(2)).await;
                match find_current_log_file() {
                    Some(p) => p,
                    None => return,
                }
            }
        };

        // Read initial size
        let mut last_size = fs::metadata(&log_file)
            .map(|m| m.len())
            .unwrap_or(0);

        loop {
            tokio::time::sleep(std::time::Duration::from_millis(500)).await;

            let current_size = match fs::metadata(&log_file) {
                Ok(m) => m.len(),
                Err(_) => continue,
            };

            if current_size > last_size {
                // Read only the new bytes
                if let Ok(contents) = fs::read_to_string(&log_file) {
                    let bytes = contents.as_bytes();
                    if (last_size as usize) < bytes.len() {
                        let new_text = String::from_utf8_lossy(&bytes[last_size as usize..]);
                        for line in new_text.lines() {
                            if !line.is_empty() {
                                let _ = app_handle.emit("log-line", line);
                            }
                        }
                    }
                }
                last_size = current_size;
            } else if current_size < last_size {
                // File was truncated/rotated — reset
                last_size = 0;
            }
        }
    });
}
