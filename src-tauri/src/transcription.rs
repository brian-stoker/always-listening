use crate::audio::AudioEngine;
use std::io::Write;
use std::path::Path;
use whisper_rs::{FullParams, SamplingStrategy, WhisperContext, WhisperContextParameters, WhisperState};

/// Known Whisper hallucination phrases that appear when audio is mostly silent
pub const WHISPER_HALLUCINATIONS: &[&str] = &[
    "you",
    "thank you",
    "thanks for watching",
    "bye",
    "the end",
    "thanks",
    "thank you for watching",
    "okay",
    "so",
    "ugh",
    "hmm",
    "",
];

/// Wraps AudioEngine + whisper-rs for speech-to-text.
///
/// This struct is intentionally `!Send` because `WhisperState` holds a raw
/// pointer internally. The agent runner spawns it inside a `spawn_local`-style
/// context or uses `spawn_blocking` to keep it on a single thread.
pub struct Transcriber {
    audio: AudioEngine,
    whisper_state: WhisperState,
    _whisper_ctx: WhisperContext,
    language: String,
}

impl Transcriber {
    /// Create a new Transcriber. Downloads the model if it doesn't exist.
    pub async fn new(
        model_path: &str,
        model_name: &str,
        language: String,
        input_device: Option<String>,
    ) -> Result<Self, String> {
        Self::ensure_model_exists(model_path, model_name).await?;

        tracing::info!("Loading Whisper model from {}...", model_path);
        let ctx = WhisperContext::new_with_params(model_path, WhisperContextParameters::default())
            .map_err(|e| format!("Failed to load Whisper model: {}", e))?;

        let state = ctx
            .create_state()
            .map_err(|e| format!("Failed to create Whisper state: {}", e))?;

        tracing::info!("Initializing Audio Engine...");
        let audio =
            AudioEngine::new(input_device).map_err(|e| format!("Audio init failed: {}", e))?;

        Ok(Self {
            audio,
            whisper_state: state,
            _whisper_ctx: ctx,
            language,
        })
    }

    /// Wait for the next speech segment and transcribe it. Returns None if the
    /// audio channel closes or if the transcription was empty / a hallucination.
    pub async fn listen_once(&mut self) -> Result<Option<String>, String> {
        let segment = match self.audio.next_segment().await {
            Some(s) => s,
            None => return Ok(None),
        };

        tracing::debug!("Transcribing {} samples...", segment.samples.len());

        let mut params = FullParams::new(SamplingStrategy::Greedy { best_of: 1 });
        params.set_language(Some(&self.language));
        params.set_print_special(false);
        params.set_print_progress(false);
        params.set_print_realtime(false);
        params.set_print_timestamps(false);

        if let Err(e) = self.whisper_state.full(params, &segment.samples) {
            return Err(format!("Whisper transcription failed: {}", e));
        }

        let num_segments = self.whisper_state.full_n_segments()
            .map_err(|e| format!("Failed to get segments: {}", e))?;
        let mut text = String::new();
        for i in 0..num_segments {
            if let Ok(s) = self.whisper_state.full_get_segment_text(i) {
                text.push_str(&s);
                text.push(' ');
            }
        }
        let text = text.trim().to_string();

        if text.is_empty() {
            return Ok(None);
        }

        // Filter hallucinations
        let cleaned = text
            .to_lowercase()
            .trim_matches(|c: char| !c.is_alphanumeric())
            .to_string();
        if WHISPER_HALLUCINATIONS.contains(&cleaned.as_str()) {
            tracing::debug!("Filtered hallucination: '{}'", text);
            return Ok(None);
        }

        tracing::info!("Transcribed: {}", text);
        Ok(Some(text))
    }

    /// Ensure Whisper model exists, downloading if necessary
    async fn ensure_model_exists(model_path: &str, model_name: &str) -> Result<(), String> {
        let path = Path::new(model_path);
        if path.exists() {
            return Ok(());
        }

        tracing::info!("Whisper model not found at {:?}. Downloading...", path);

        if let Some(parent) = path.parent() {
            tokio::fs::create_dir_all(parent)
                .await
                .map_err(|e| e.to_string())?;
        }

        let url = format!(
            "https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-{}.bin",
            model_name
        );

        let response = reqwest::get(&url)
            .await
            .map_err(|e| format!("Failed to download model: {}", e))?;
        if !response.status().is_success() {
            return Err(format!(
                "Download failed with status: {}",
                response.status()
            ));
        }

        let content = response
            .bytes()
            .await
            .map_err(|e| format!("Failed to get model bytes: {}", e))?;

        let mut file = std::fs::File::create(path).map_err(|e| e.to_string())?;
        file.write_all(&content).map_err(|e| e.to_string())?;

        tracing::info!("Model downloaded successfully.");
        Ok(())
    }
}
