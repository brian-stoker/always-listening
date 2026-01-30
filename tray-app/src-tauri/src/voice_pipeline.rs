use crate::config::Config;
use crate::process_manager::ProcessManager;
use std::sync::Arc;
use tokio::sync::{Mutex, watch};

/// Runs the Voice-to-Claude pipeline loop
/// This is the Rust implementation of voice-pipeline.sh
pub struct VoicePipeline {
    process_manager: Arc<ProcessManager>,
    config: Arc<Mutex<Config>>,
    shutdown_rx: watch::Receiver<bool>,
}

impl VoicePipeline {
    pub fn new(
        process_manager: Arc<ProcessManager>,
        config: Arc<Mutex<Config>>,
        shutdown_rx: watch::Receiver<bool>,
    ) -> Self {
        Self { process_manager, config, shutdown_rx }
    }

    /// Start the voice pipeline loop
    pub async fn run(&mut self) -> Result<(), String> {
        tracing::info!("Voice-to-Claude pipeline starting...");

        loop {
            // Check for shutdown signal
            if *self.shutdown_rx.borrow() {
                tracing::info!("Voice pipeline received shutdown signal");
                break;
            }

            // 1. Record audio
            let audio_file = match self.record_audio().await {
                Ok(path) => path,
                Err(e) => {
                    tracing::debug!("Recording failed or no speech: {}", e);
                    continue;
                }
            };

            // Check shutdown after recording
            if *self.shutdown_rx.borrow() { break; }

            // 2. Transcribe with Whisper
            let text = match self.transcribe(&audio_file).await {
                Ok(t) if t.trim().is_empty() => {
                    tracing::debug!("Transcription empty, listening again...");
                    continue;
                }
                Ok(t) => t,
                Err(e) => {
                    tracing::warn!("Transcription failed: {}", e);
                    continue;
                }
            };

            tracing::info!("You said: {}", text);

            // 3. Check for exit command
            let text_lower = text.to_lowercase();
            if text_lower.contains("goodbye clawdbot")
                || text_lower.contains("goodbye claudbot")
                || text_lower.contains("goodbye claude bot") {
                tracing::info!("Exit command detected. Goodbye!");
                self.speak("Goodbye!").await;
                break;
            }

            // 4. Send to Claude/Clawdbot
            let response = match self.ask_claude(&text).await {
                Ok(r) if r.trim().is_empty() => {
                    tracing::warn!("Claude returned empty response");
                    self.speak("Sorry, I didn't get a response.").await;
                    continue;
                }
                Ok(r) => r,
                Err(e) => {
                    tracing::error!("Claude error: {}", e);
                    self.speak("Sorry, I encountered an error.").await;
                    continue;
                }
            };

            tracing::info!("Claude: {}", response);

            // 5. Speak the response
            self.speak(&response).await;
        }

        tracing::info!("Voice pipeline stopped");
        Ok(())
    }

    /// Record audio using ffmpeg with silence detection
    async fn record_audio(&self) -> Result<String, String> {
        let config = self.config.lock().await;
        let tmp_dir = &config.tmp_dir;
        let audio_device = &config.audio_device;
        let silence_threshold = &config.silence_threshold;
        let silence_duration = config.silence_duration;
        let record_duration = config.record_duration;
        let audio_file = format!("{}/audio_input.wav", tmp_dir);

        // Create tmp dir
        tokio::fs::create_dir_all(tmp_dir).await.map_err(|e| e.to_string())?;

        // Record with ffmpeg - silence detection stops recording
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

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            // ffmpeg often exits non-zero with silence detection, check if file exists
            if tokio::fs::metadata(&audio_file).await.is_err() {
                return Err(format!("Recording failed: {}", stderr));
            }
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

        // Read the output text file
        let txt_file = audio_file.replace(".wav", ".txt");
        let text = tokio::fs::read_to_string(&txt_file)
            .await
            .map_err(|e| format!("Failed to read transcription: {}", e))?;

        Ok(text.trim().to_string())
    }

    /// Send text to Claude/Clawdbot CLI and get response
    async fn ask_claude(&self, text: &str) -> Result<String, String> {
        let config = self.config.lock().await;
        let claude_bin = config.claude_bin.clone();
        drop(config);

        let output = tokio::process::Command::new(&claude_bin)
            .args(&[
                "agent",
                "--agent", "main",
                "--session-id", "voice-pipeline",
                "--message", text,
                "--json",
            ])
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .output()
            .await
            .map_err(|e| format!("Claude CLI failed: {}", e))?;

        let stdout = String::from_utf8_lossy(&output.stdout);

        // Try to parse JSON response
        if let Ok(json) = serde_json::from_str::<serde_json::Value>(&stdout) {
            // Extract text from result.payloads[0].text
            if let Some(text) = json.get("result")
                .and_then(|r| r.get("payloads"))
                .and_then(|p| p.get(0))
                .and_then(|p| p.get("text"))
                .and_then(|t| t.as_str()) {
                return Ok(text.to_string());
            }
        }

        // Fallback: use raw stdout if JSON parsing fails
        if !stdout.trim().is_empty() {
            Ok(stdout.trim().to_string())
        } else {
            Err("Empty response from Claude".to_string())
        }
    }

    /// Speak text using TTS (Home Assistant or local fallback)
    async fn speak(&self, text: &str) {
        let config = self.config.lock().await;
        let ha_enabled = config.ha_enabled;
        let ha_url = config.ha_url.clone();
        let ha_tts_entity = config.ha_tts_entity.clone();
        let ha_entity = config.ha_entity.clone();
        let hass_token = config.hass_token.clone();
        drop(config);

        if ha_enabled && !hass_token.is_empty() {
            // Use Home Assistant TTS
            if let Err(e) = self.speak_ha(text, &ha_url, &ha_tts_entity, &ha_entity, &hass_token).await {
                tracing::warn!("HA TTS failed, falling back to local: {}", e);
                self.speak_local(text).await;
            }
        } else {
            self.speak_local(text).await;
        }
    }

    /// TTS via Home Assistant API
    async fn speak_ha(&self, text: &str, ha_url: &str, tts_entity: &str, media_entity: &str, token: &str) -> Result<(), String> {
        let url = format!("{}/api/services/tts/speak", ha_url);
        let body = serde_json::json!({
            "entity_id": tts_entity,
            "media_player_entity_id": media_entity,
            "message": text
        });

        let client = reqwest::Client::new();
        client.post(&url)
            .header("Authorization", format!("Bearer {}", token))
            .header("Content-Type", "application/json")
            .json(&body)
            .send()
            .await
            .map_err(|e| format!("HA API error: {}", e))?;

        Ok(())
    }

    /// Local TTS fallback (macOS: say, Windows: SAPI)
    async fn speak_local(&self, text: &str) {
        #[cfg(target_os = "macos")]
        {
            let _ = tokio::process::Command::new("say")
                .arg(text)
                .output()
                .await;
        }
        #[cfg(target_os = "windows")]
        {
            // Use PowerShell for SAPI TTS on Windows
            let script = format!(
                "Add-Type -AssemblyName System.Speech; $synth = New-Object System.Speech.Synthesis.SpeechSynthesizer; $synth.Speak('{}')",
                text.replace("'", "''")
            );
            let _ = tokio::process::Command::new("powershell")
                .args(&["-Command", &script])
                .output()
                .await;
        }
    }
}
