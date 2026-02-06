use crate::config::{AgentConfig, Config, TtsMethod};
use crate::transcription::Transcriber;
use std::sync::Arc;
use std::time::Instant;
use tauri::Emitter;
use tokio::sync::{mpsc, watch, Mutex};

/// How long after speaking we still consider transcriptions as potential echoes.
const ECHO_WINDOW_SECS: f64 = 15.0;
/// Minimum word-overlap ratio to consider a transcription an echo of spoken text.
const ECHO_SIMILARITY_THRESHOLD: f64 = 0.5;

/// Generic agent runner — executes any AgentConfig against a shared Transcriber.
pub struct AgentRunner {
    agent: AgentConfig,
    config: Arc<Mutex<Config>>,
    shutdown_rx: watch::Receiver<bool>,
    app_handle: Option<tauri::AppHandle>,
    /// Recently spoken texts with timestamps, used for echo detection.
    recent_spoken: Vec<(String, Instant)>,
}

impl AgentRunner {
    pub fn new(
        agent: AgentConfig,
        config: Arc<Mutex<Config>>,
        shutdown_rx: watch::Receiver<bool>,
    ) -> Self {
        Self {
            agent,
            config,
            shutdown_rx,
            app_handle: None,
            recent_spoken: Vec::new(),
        }
    }

    pub fn with_app_handle(mut self, handle: tauri::AppHandle) -> Self {
        self.app_handle = Some(handle);
        self
    }

    fn emit_event(&self, event: &str) {
        if let Some(ref handle) = self.app_handle {
            let _ = handle.emit("pipeline-event", event);
        }
    }

    /// Normalize text for echo comparison: lowercase, strip punctuation, collapse whitespace.
    fn normalize_for_echo(text: &str) -> String {
        text.to_lowercase()
            .chars()
            .map(|c| if c.is_alphanumeric() || c.is_whitespace() { c } else { ' ' })
            .collect::<String>()
            .split_whitespace()
            .collect::<Vec<_>>()
            .join(" ")
    }

    /// Check if a transcription is an echo of recently spoken TTS output.
    fn is_echo(&mut self, transcription: &str) -> bool {
        let now = Instant::now();

        // Prune old entries
        self.recent_spoken
            .retain(|(_, t)| now.duration_since(*t).as_secs_f64() < ECHO_WINDOW_SECS);

        let norm_heard = Self::normalize_for_echo(transcription);
        let heard_words: Vec<&str> = norm_heard.split_whitespace().collect();

        if heard_words.is_empty() {
            return false;
        }

        for (spoken, _) in &self.recent_spoken {
            let norm_spoken = Self::normalize_for_echo(spoken);
            let spoken_words: Vec<&str> = norm_spoken.split_whitespace().collect();

            if spoken_words.is_empty() {
                continue;
            }

            // Count how many words from the transcription appear in the spoken text
            let matching = heard_words
                .iter()
                .filter(|w| spoken_words.contains(w))
                .count();

            let overlap = matching as f64 / heard_words.len() as f64;

            if overlap >= ECHO_SIMILARITY_THRESHOLD {
                tracing::info!(
                    "Echo detected (overlap={:.0}%): heard={:?} matches spoken={:?}",
                    overlap * 100.0,
                    transcription,
                    spoken,
                );
                return true;
            }
        }

        false
    }

    /// Record text that was spoken via TTS for echo detection.
    fn record_spoken(&mut self, text: &str) {
        self.recent_spoken.push((text.to_string(), Instant::now()));
    }

    /// Run the agent loop.
    /// Spawns the Transcriber on a dedicated thread (it is !Send) and receives
    /// transcription results via a channel.
    pub async fn run(&mut self) -> Result<(), String> {
        tracing::info!("Agent '{}' starting...", self.agent.name);

        let (model_path, model_name, device, language) = {
            let cfg = self.config.lock().await;
            (
                cfg.whisper_model_path.clone(),
                cfg.whisper_model.clone(),
                cfg.audio_device.clone(),
                cfg.whisper_language.clone(),
            )
        };

        let device_opt = if device == "default" || device == ":1" {
            None
        } else {
            Some(device)
        };

        // Channel for transcription results from the dedicated thread
        let (text_tx, mut text_rx) = mpsc::channel::<Result<Option<String>, String>>(4);
        let mut shutdown_rx_for_thread = self.shutdown_rx.clone();

        // Spawn a dedicated thread for the Transcriber (!Send)
        std::thread::spawn(move || {
            let rt = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .expect("Failed to build tokio runtime for transcriber");

            rt.block_on(async move {
                let mut transcriber = match Transcriber::new(
                    &model_path,
                    &model_name,
                    language,
                    device_opt,
                ).await {
                    Ok(t) => t,
                    Err(e) => {
                        let _ = text_tx.send(Err(e)).await;
                        return;
                    }
                };

                loop {
                    if *shutdown_rx_for_thread.borrow() {
                        break;
                    }

                    let result = tokio::select! {
                        r = transcriber.listen_once() => r,
                        _ = shutdown_rx_for_thread.changed() => break,
                    };

                    if text_tx.send(result).await.is_err() {
                        break; // receiver dropped
                    }
                }
            });
        });

        tracing::info!("Agent '{}' listening...", self.agent.name);

        loop {
            if *self.shutdown_rx.borrow() {
                tracing::info!("Agent '{}' received shutdown signal", self.agent.name);
                break;
            }

            self.emit_event("recording");

            // Wait for transcription result from the dedicated thread
            let text = tokio::select! {
                result = text_rx.recv() => {
                    match result {
                        Some(Ok(Some(t))) => t,
                        Some(Ok(None)) => continue,
                        Some(Err(e)) => {
                            tracing::error!("Transcription error in agent '{}': {}", self.agent.name, e);
                            return Err(e);
                        }
                        None => {
                            tracing::info!("Transcriber channel closed for agent '{}'", self.agent.name);
                            break;
                        }
                    }
                }
                _ = self.shutdown_rx.changed() => {
                    tracing::info!("Shutdown during listen for agent '{}'", self.agent.name);
                    break;
                }
            };

            self.emit_event("active");
            tracing::info!("[{}] Heard: {}", self.agent.name, text);

            // Filter out echoes of our own TTS output
            if self.is_echo(&text) {
                tracing::info!("[{}] Skipping echo of TTS output", self.agent.name);
                continue;
            }

            // Check exit phrases
            let text_lower = text.to_lowercase();
            if self
                .agent
                .exit_phrases
                .iter()
                .any(|phrase| text_lower.contains(phrase))
            {
                tracing::info!("[{}] Exit phrase detected. Stopping.", self.agent.name);
                self.speak("Goodbye!").await;
                break;
            }

            // If agent has a command, execute it and optionally speak the response
            if let Some(ref cmd_template) = self.agent.command {
                let input_text = if let Some(ref prefix) = self.agent.prompt_prefix {
                    format!("{}{}", prefix, text)
                } else {
                    text.clone()
                };

                let response = match self.execute_command(cmd_template, &input_text).await {
                    Ok(r) if r.trim().is_empty() => {
                        tracing::warn!("[{}] Command returned empty response", self.agent.name);
                        None
                    }
                    Ok(r) => {
                        tracing::info!("[{}] Response: {}", self.agent.name, r);
                        Some(r)
                    }
                    Err(e) => {
                        tracing::error!("[{}] Command error: {}", self.agent.name, e);
                        let err_msg = "Sorry, I encountered an error.";
                        self.record_spoken(err_msg);
                        self.speak(err_msg).await;
                        None
                    }
                };

                // Speak the response if TTS is configured
                if let Some(ref resp) = response {
                    if self.agent.tts.method != TtsMethod::None {
                        self.emit_event("agent-speaking");
                        self.record_spoken(resp);
                        self.speak(resp).await;
                        self.emit_event("active");
                    }
                }
            }

            // Type at cursor if configured
            if self.agent.type_at_cursor {
                if let Err(e) = Self::type_at_cursor(&text).await {
                    tracing::error!("[{}] type_at_cursor failed: {}", self.agent.name, e);
                }
                if self.agent.auto_submit {
                    if let Err(e) = Self::press_enter().await {
                        tracing::error!("[{}] press_enter failed: {}", self.agent.name, e);
                    }
                }
            }
        }

        tracing::info!("Agent '{}' stopped", self.agent.name);
        Ok(())
    }

    /// Execute the agent command template, substituting `{text}`.
    async fn execute_command(&self, template: &str, text: &str) -> Result<String, String> {
        // Shell-escape the text to prevent injection and handle special chars
        let escaped_text = text.replace('\\', "\\\\").replace('"', "\\\"");
        let full_command = template.replace("{text}", &escaped_text);

        // Use a login shell so that ~/.zshrc / ~/.bashrc / NVM paths are available.
        // macOS Tauri apps launched from Finder/Dock get a minimal PATH that misses
        // tools installed via NVM, Homebrew in user paths, etc.
        let output = tokio::process::Command::new("bash")
            .args(&["-l", "-c", &full_command])
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .output()
            .await
            .map_err(|e| format!("Command failed: {}", e))?;

        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();

        if !output.status.success() {
            tracing::error!("Command exited with status {}: stderr={}", output.status, stderr.trim());
        }
        if !stderr.trim().is_empty() {
            tracing::warn!("Command stderr: {}", stderr.trim());
        }

        // If a JSON path is configured, extract from the response
        if let Some(ref json_path) = self.agent.response_json_path {
            if let Ok(json) = serde_json::from_str::<serde_json::Value>(&stdout) {
                if let Some(val) = resolve_json_path(&json, json_path) {
                    if let Some(s) = val.as_str() {
                        return Ok(s.to_string());
                    }
                    return Ok(val.to_string());
                }
            }
        }

        if !stdout.trim().is_empty() {
            Ok(stdout.trim().to_string())
        } else {
            Err("Empty response from command".to_string())
        }
    }

    /// Speak text using TTS (HA or local say)
    async fn speak(&self, text: &str) {
        match self.agent.tts.method {
            TtsMethod::Ha => {
                let config = self.config.lock().await;
                let ha_url = config.ha_url.clone();
                let hass_token = config.hass_token.clone();
                drop(config);

                let tts_entity = self
                    .agent
                    .tts
                    .ha_tts_entity
                    .as_deref()
                    .unwrap_or("tts.google_en_com");
                let media_entity = self
                    .agent
                    .tts
                    .ha_media_entity
                    .as_deref()
                    .unwrap_or("media_player.office_speaker");

                if !hass_token.is_empty() {
                    if let Err(e) =
                        Self::speak_ha(text, &ha_url, tts_entity, media_entity, &hass_token).await
                    {
                        tracing::warn!("HA TTS failed, falling back to local: {}", e);
                        Self::speak_local(text, self.agent.tts.local_voice.as_deref()).await;
                    }
                } else {
                    Self::speak_local(text, self.agent.tts.local_voice.as_deref()).await;
                }
            }
            TtsMethod::LocalSay => {
                Self::speak_local(text, self.agent.tts.local_voice.as_deref()).await;
            }
            TtsMethod::None => {}
        }
    }

    /// TTS via Home Assistant API
    async fn speak_ha(
        text: &str,
        ha_url: &str,
        tts_entity: &str,
        media_entity: &str,
        token: &str,
    ) -> Result<(), String> {
        let url = format!("{}/api/services/tts/speak", ha_url);
        let body = serde_json::json!({
            "entity_id": tts_entity,
            "media_player_entity_id": media_entity,
            "message": text
        });

        let client = reqwest::Client::new();
        client
            .post(&url)
            .header("Authorization", format!("Bearer {}", token))
            .header("Content-Type", "application/json")
            .json(&body)
            .send()
            .await
            .map_err(|e| format!("HA API error: {}", e))?;

        Ok(())
    }

    /// Local TTS fallback (macOS: say)
    async fn speak_local(text: &str, voice: Option<&str>) {
        #[cfg(target_os = "macos")]
        {
            let mut cmd = tokio::process::Command::new("say");
            if let Some(v) = voice {
                cmd.args(&["-v", v]);
            }
            cmd.arg(text);
            let _ = cmd.output().await;
        }
        #[cfg(target_os = "windows")]
        {
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

    /// Type text at the current cursor position via AppleScript
    pub async fn type_at_cursor(text: &str) -> Result<(), String> {
        #[cfg(target_os = "macos")]
        {
            let escaped = text.replace("\\", "\\\\").replace("\"", "\\\"");
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
    pub async fn press_enter() -> Result<(), String> {
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
                .args(&[
                    "-Command",
                    "$wsh = New-Object -ComObject WScript.Shell; $wsh.SendKeys('{ENTER}')",
                ])
                .output()
                .await;
        }
        Ok(())
    }
}

/// Resolve a dot-path like "result.payloads.0.text" into a JSON value.
fn resolve_json_path<'a>(value: &'a serde_json::Value, path: &str) -> Option<&'a serde_json::Value> {
    let mut current = value;
    for key in path.split('.') {
        if let Ok(idx) = key.parse::<usize>() {
            current = current.get(idx)?;
        } else {
            current = current.get(key)?;
        }
    }
    Some(current)
}
