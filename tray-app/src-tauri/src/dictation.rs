use crate::config::Config;
use std::sync::Arc;
use tokio::sync::{Mutex, watch};

/// Runs the Dictation pipeline
/// Waits for hotkey trigger, records, transcribes, types at cursor
pub struct DictationPipeline {
    config: Arc<Mutex<Config>>,
    shutdown_rx: watch::Receiver<bool>,
}

impl DictationPipeline {
    pub fn new(
        config: Arc<Mutex<Config>>,
        shutdown_rx: watch::Receiver<bool>,
    ) -> Self {
        Self { config, shutdown_rx }
    }

    /// Run a single dictation cycle (record → transcribe → type)
    pub async fn run_once(&self) -> Result<(), String> {
        tracing::info!("Dictation: recording...");

        // 1. Record audio
        let audio_file = self.record_audio().await?;

        // 2. Transcribe
        let text = self.transcribe(&audio_file).await?;

        if text.trim().is_empty() {
            tracing::debug!("Dictation: empty transcription");
            return Ok(());
        }

        tracing::info!("Dictation: '{}'", text);

        // 3. Type at cursor
        self.type_at_cursor(&text).await?;

        // 4. Submit (press Enter)
        self.press_enter().await?;

        Ok(())
    }

    /// Record audio using ffmpeg
    async fn record_audio(&self) -> Result<String, String> {
        let config = self.config.lock().await;
        let tmp_dir = &config.tmp_dir;
        let audio_device = &config.audio_device;
        let silence_threshold = &config.silence_threshold;
        let silence_duration = config.silence_duration;
        let record_duration = config.record_duration;
        let audio_file = format!("{}/dictation_audio.wav", tmp_dir);

        tokio::fs::create_dir_all(tmp_dir).await.map_err(|e| e.to_string())?;

        let output = tokio::process::Command::new("ffmpeg")
            .args(&[
                "-y",
                "-f", "avfoundation",
                "-i", audio_device,
                "-ar", "16000",
                "-ac", "1",
                "-sample_fmt", "s16",
                "-t", &record_duration.to_string(),
                "-af", &format!("silencedetect=noise={}:d={}", silence_threshold, silence_duration),
                &audio_file,
            ])
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .output()
            .await
            .map_err(|e| format!("ffmpeg failed: {}", e))?;

        if tokio::fs::metadata(&audio_file).await.is_err() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(format!("Recording failed: {}", stderr));
        }

        Ok(audio_file)
    }

    /// Transcribe audio using Whisper CLI
    async fn transcribe(&self, audio_file: &str) -> Result<String, String> {
        let config = self.config.lock().await;
        let whisper_bin = config.whisper_bin.clone();
        let model = config.whisper_model.clone();
        let language = config.whisper_language.clone();
        let tmp_dir = config.tmp_dir.clone();
        drop(config);

        let output = tokio::process::Command::new(&whisper_bin)
            .args(&[
                audio_file,
                "--model", &model,
                "--language", &language,
                "--output_format", "txt",
                "--output_dir", &tmp_dir,
            ])
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .output()
            .await
            .map_err(|e| format!("Whisper failed: {}", e))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(format!("Whisper error: {}", stderr));
        }

        let txt_file = audio_file.replace(".wav", ".txt");
        let text = tokio::fs::read_to_string(&txt_file)
            .await
            .map_err(|e| format!("Failed to read transcription: {}", e))?;

        Ok(text.trim().to_string())
    }

    /// Type text at the current cursor position
    async fn type_at_cursor(&self, text: &str) -> Result<(), String> {
        #[cfg(target_os = "macos")]
        {
            // Escape text for AppleScript
            let escaped = text
                .replace("\\", "\\\\")
                .replace("\"", "\\\"");

            let script = format!(
                "tell application \"System Events\" to keystroke \"{}\"",
                escaped
            );

            let output = tokio::process::Command::new("osascript")
                .args(&["-e", &script])
                .output()
                .await
                .map_err(|e| format!("osascript failed: {}", e))?;

            if !output.status.success() {
                let stderr = String::from_utf8_lossy(&output.stderr);
                return Err(format!("osascript error: {}", stderr));
            }
        }

        #[cfg(target_os = "windows")]
        {
            // Use PowerShell with SendKeys for Windows
            let escaped = text.replace("'", "''");
            let script = format!(
                "$wsh = New-Object -ComObject WScript.Shell; $wsh.SendKeys('{}')",
                escaped
            );

            let _ = tokio::process::Command::new("powershell")
                .args(&["-Command", &script])
                .output()
                .await;
        }

        Ok(())
    }

    /// Press Enter key to submit
    async fn press_enter(&self) -> Result<(), String> {
        #[cfg(target_os = "macos")]
        {
            let _ = tokio::process::Command::new("osascript")
                .args(&["-e", "tell application \"System Events\" to key code 36"])
                .output()
                .await;
        }

        #[cfg(target_os = "windows")]
        {
            let _ = tokio::process::Command::new("powershell")
                .args(&["-Command", "$wsh = New-Object -ComObject WScript.Shell; $wsh.SendKeys('{ENTER}')"])
                .output()
                .await;
        }

        Ok(())
    }
}
