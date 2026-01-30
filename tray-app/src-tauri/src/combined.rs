use crate::config::Config;
use crate::dictation::DictationPipeline;
use crate::process_manager::ProcessManager;
use crate::voice_pipeline::VoicePipeline;
use std::sync::Arc;
use tokio::sync::{Mutex, watch};

/// Combined Mode (Mode 3) - runs both Voice-to-Claude and Dictation pipelines
/// simultaneously. Both pipelines share the same shutdown channel so they
/// stop together when the mode is deactivated.
pub struct CombinedPipeline {
    process_manager: Arc<ProcessManager>,
    config: Arc<Mutex<Config>>,
    shutdown_rx: watch::Receiver<bool>,
}

impl CombinedPipeline {
    pub fn new(
        process_manager: Arc<ProcessManager>,
        config: Arc<Mutex<Config>>,
        shutdown_rx: watch::Receiver<bool>,
    ) -> Self {
        Self {
            process_manager,
            config,
            shutdown_rx,
        }
    }

    /// Start both pipelines concurrently. Returns when both have stopped
    /// (either from shutdown signal or an error in the voice pipeline).
    pub async fn run(&self) -> Result<(), String> {
        tracing::info!("Combined mode starting: voice pipeline + dictation");

        // Clone the shared state for each task
        let voice_pm = Arc::clone(&self.process_manager);
        let voice_config = Arc::clone(&self.config);
        let voice_shutdown_rx = self.shutdown_rx.clone();

        let dict_config = Arc::clone(&self.config);
        let dict_shutdown_rx = self.shutdown_rx.clone();
        let dict_loop_shutdown_rx = self.shutdown_rx.clone();

        // Spawn the voice-to-Claude pipeline task
        let voice_task = tokio::spawn(async move {
            let mut pipeline = VoicePipeline::new(voice_pm, voice_config, voice_shutdown_rx);
            if let Err(e) = pipeline.run().await {
                tracing::error!("Voice pipeline error in combined mode: {}", e);
            }
            tracing::info!("Combined mode: voice pipeline task exited");
        });

        // Spawn the dictation pipeline task (loop until shutdown)
        let dictation_task = tokio::spawn(async move {
            let pipeline = DictationPipeline::new(dict_config, dict_shutdown_rx);
            loop {
                if *dict_loop_shutdown_rx.borrow() {
                    break;
                }
                if let Err(e) = pipeline.run_once().await {
                    tracing::error!("Dictation error in combined mode: {}", e);
                }
            }
            tracing::info!("Combined mode: dictation task exited");
        });

        // Wait for both tasks to complete. If either finishes early (e.g. voice
        // pipeline exits due to a goodbye command), the other continues until
        // the shared shutdown signal is sent by the orchestrator.
        let (voice_result, dictation_result) = tokio::join!(voice_task, dictation_task);

        if let Err(e) = voice_result {
            tracing::error!("Voice task panicked: {}", e);
        }
        if let Err(e) = dictation_result {
            tracing::error!("Dictation task panicked: {}", e);
        }

        tracing::info!("Combined mode stopped");
        Ok(())
    }
}
