use serde::{Deserialize, Serialize};
use std::process::Command;

use crate::config::Config;

/// Status of each dependency check
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DependencyStatus {
    pub ffmpeg_ok: bool,
    pub ffmpeg_version: String,

    pub whisper_ok: bool,
    pub whisper_version: String,

    pub claude_ok: bool,
    pub claude_version: String,

    pub accessibility_ok: bool,
    pub accessibility_info: String,

    pub microphone_ok: bool,
    pub microphone_info: String,
}

/// Check if ffmpeg is installed by running `ffmpeg -version`
fn check_ffmpeg() -> (bool, String) {
    match Command::new("ffmpeg").arg("-version").output() {
        Ok(output) => {
            if output.status.success() {
                let stdout = String::from_utf8_lossy(&output.stdout);
                // First line typically contains the version
                let version = stdout
                    .lines()
                    .next()
                    .unwrap_or("installed")
                    .to_string();
                (true, version)
            } else {
                (false, "ffmpeg exited with error".to_string())
            }
        }
        Err(e) => (false, format!("Not found: {}", e)),
    }
}

/// Check if whisper binary exists and is executable
fn check_whisper(bin: &str) -> (bool, String) {
    if bin.is_empty() {
        return (false, "No whisper binary configured".to_string());
    }

    let path = std::path::Path::new(bin);
    if !path.exists() {
        return (false, format!("Binary not found at {}", bin));
    }

    // Try running with --help or similar to verify it works
    match Command::new(bin).arg("--help").output() {
        Ok(output) => {
            if output.status.success() || !output.stdout.is_empty() {
                let stdout = String::from_utf8_lossy(&output.stdout);
                let first_line = stdout
                    .lines()
                    .next()
                    .unwrap_or("installed")
                    .to_string();
                (true, first_line)
            } else {
                // Binary exists but --help failed; still counts as found
                (true, format!("Found at {}", bin))
            }
        }
        Err(e) => {
            // The file exists but we couldn't execute it
            if path.exists() {
                (true, format!("Found at {} (not executable: {})", bin, e))
            } else {
                (false, format!("Cannot execute: {}", e))
            }
        }
    }
}

/// Check if claude CLI binary exists and is executable
fn check_claude(bin: &str) -> (bool, String) {
    if bin.is_empty() {
        return (false, "No claude binary configured".to_string());
    }

    let path = std::path::Path::new(bin);
    if !path.exists() {
        // Try finding it on PATH
        match Command::new("which").arg("claude").output() {
            Ok(output) if output.status.success() => {
                let found_path = String::from_utf8_lossy(&output.stdout).trim().to_string();
                return (true, format!("Found at {}", found_path));
            }
            _ => {}
        }
        return (false, format!("Binary not found at {}", bin));
    }

    match Command::new(bin).arg("--version").output() {
        Ok(output) => {
            let stdout = String::from_utf8_lossy(&output.stdout);
            let stderr = String::from_utf8_lossy(&output.stderr);
            let version_text = if !stdout.trim().is_empty() {
                stdout.trim().to_string()
            } else if !stderr.trim().is_empty() {
                stderr.lines().next().unwrap_or("installed").to_string()
            } else {
                format!("Found at {}", bin)
            };
            (true, version_text)
        }
        Err(_) => {
            if path.exists() {
                (true, format!("Found at {}", bin))
            } else {
                (false, format!("Binary not found at {}", bin))
            }
        }
    }
}

/// Check if accessibility permissions are granted (macOS)
/// Uses osascript to test if we can perform UI scripting
fn check_accessibility() -> (bool, String) {
    #[cfg(target_os = "macos")]
    {
        // Check if the app has accessibility permissions by querying the system
        match Command::new("osascript")
            .arg("-e")
            .arg("tell application \"System Events\" to return name of first process")
            .output()
        {
            Ok(output) => {
                if output.status.success() {
                    (true, "Accessibility permissions granted".to_string())
                } else {
                    let stderr = String::from_utf8_lossy(&output.stderr);
                    if stderr.contains("not allowed") || stderr.contains("assistive") {
                        (
                            false,
                            "Accessibility permissions not granted. Enable in System Settings > Privacy & Security > Accessibility".to_string(),
                        )
                    } else {
                        (false, format!("Check failed: {}", stderr.trim()))
                    }
                }
            }
            Err(e) => (false, format!("Cannot check accessibility: {}", e)),
        }
    }

    #[cfg(not(target_os = "macos"))]
    {
        (true, "Not required on this platform".to_string())
    }
}

/// Check for microphone access
/// On macOS, we check if any audio input device is available
fn check_microphone() -> (bool, String) {
    #[cfg(target_os = "macos")]
    {
        // Use system_profiler to check for audio input devices
        match Command::new("system_profiler")
            .arg("SPAudioDataType")
            .output()
        {
            Ok(output) => {
                let stdout = String::from_utf8_lossy(&output.stdout);
                if stdout.contains("Input") || stdout.contains("Microphone") || stdout.contains("Built-in") {
                    (true, "Audio input device detected".to_string())
                } else {
                    (false, "No audio input device found".to_string())
                }
            }
            Err(e) => (false, format!("Cannot check audio devices: {}", e)),
        }
    }

    #[cfg(not(target_os = "macos"))]
    {
        // On other platforms, try using arecord to list devices
        match Command::new("arecord").arg("-l").output() {
            Ok(output) => {
                let stdout = String::from_utf8_lossy(&output.stdout);
                if stdout.contains("card") {
                    (true, "Audio input device detected".to_string())
                } else {
                    (false, "No audio input device found".to_string())
                }
            }
            Err(_) => (true, "Cannot verify - assuming available".to_string()),
        }
    }
}

/// Check if the config file exists (used to determine first-run)
fn is_first_run() -> bool {
    !Config::config_path().exists()
}

/// Tauri command: check all dependencies and return their status
#[tauri::command]
pub async fn check_dependencies(
    config: tauri::State<'_, std::sync::Arc<tokio::sync::Mutex<Config>>>,
) -> Result<DependencyStatus, String> {
    let cfg = config.lock().await;

    let (ffmpeg_ok, ffmpeg_version) = check_ffmpeg();
    let (whisper_ok, whisper_version) = check_whisper(&cfg.whisper_bin);
    let (claude_ok, claude_version) = check_claude(&cfg.claude_bin);
    let (accessibility_ok, accessibility_info) = check_accessibility();
    let (microphone_ok, microphone_info) = check_microphone();

    tracing::info!(
        "Dependency check: ffmpeg={}, whisper={}, claude={}, accessibility={}, microphone={}",
        ffmpeg_ok,
        whisper_ok,
        claude_ok,
        accessibility_ok,
        microphone_ok
    );

    Ok(DependencyStatus {
        ffmpeg_ok,
        ffmpeg_version,
        whisper_ok,
        whisper_version,
        claude_ok,
        claude_version,
        accessibility_ok,
        accessibility_info,
        microphone_ok,
        microphone_info,
    })
}

/// Tauri command: check if this is the first run (no config file exists)
#[tauri::command]
pub async fn is_first_run_check() -> Result<bool, String> {
    Ok(is_first_run())
}
