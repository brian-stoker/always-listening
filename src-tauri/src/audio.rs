use anyhow::{anyhow, Result};
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use ringbuf::HeapRb;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tokio::sync::mpsc;
use tracing::{debug, error, info, warn};
use webrtc_vad::{Vad, VadMode};

// Whisper expects 16kHz audio
const WHISPER_SAMPLE_RATE: u32 = 16000;
// VAD processing chunk size (WebRTC VAD supports 10, 20, or 30ms)
const VAD_CHUNK_MS: usize = 30;

#[derive(Debug, Clone)]
pub struct AudioSegment {
    pub samples: Vec<f32>,
}

pub struct AudioEngine {
    #[allow(dead_code)]
    stream: cpal::Stream,
    is_running: Arc<AtomicBool>,
    segment_rx: mpsc::Receiver<AudioSegment>,
}

impl AudioEngine {
    pub fn new(input_device_name: Option<String>) -> Result<Self> {
        let host = cpal::default_host();

        let device = if let Some(name) = input_device_name {
            host.input_devices()?
                .find(|d| d.name().map(|n| n == name).unwrap_or(false))
                .ok_or_else(|| anyhow!("Input device not found: {}", name))?
        } else {
            host.default_input_device()
                .ok_or_else(|| anyhow!("No default input device found"))?
        };

        info!("Using input device: {}", device.name()?);

        let config = device.default_input_config()?;
        let sample_rate = config.sample_rate().0;
        let channels = config.channels();

        info!("Input config: Sample Rate: {}, Channels: {}", sample_rate, channels);

        // Ring buffer to bridge cpal callback (real-time) and processing thread
        let buffer_size = sample_rate as usize * channels as usize; 
        let (mut producer, mut consumer) = HeapRb::<f32>::new(buffer_size).split();

        let err_fn = |err| error!("an error occurred on stream: {}", err);

        let stream = match config.sample_format() {
            cpal::SampleFormat::F32 => device.build_input_stream(
                &config.into(),
                move |data: &[f32], _: &_| {
                    let _ = producer.push_slice(data);
                },
                err_fn,
                None,
            )?,
            cpal::SampleFormat::I16 => device.build_input_stream(
                &config.into(),
                move |data: &[i16], _: &_| {
                    // Convert i16 to f32
                    for &sample in data {
                        let _ = producer.push(sample as f32 / 32768.0);
                    }
                },
                err_fn,
                None,
            )?,
            _ => return Err(anyhow!("Unsupported sample format")),
        };

        stream.play()?;

        let is_running = Arc::new(AtomicBool::new(true));
        let is_running_clone = is_running.clone();
        let (segment_tx, segment_rx) = mpsc::channel(10);

        // Processing thread: Resampling + VAD + Segmentation
        std::thread::spawn(move || {
            // Initialize WebRTC VAD
            // Mode 0: Normal, Mode 3: Very Aggressive (filters more noise)
            let mut vad = Vad::new_with_rate_and_mode(
                webrtc_vad::SampleRate::Rate16kHz, 
                VadMode::Aggressive
            );

            // State for VAD
            let mut speech_buffer: Vec<f32> = Vec::new();
            let mut is_speaking = false;
            let mut silence_frames = 0;
            
            // Configurable thresholds
            let silence_limit_ms = 1000; // 1s of silence ends the utterance
            let min_speech_ms = 500; // Ignore detected "speech" shorter than 0.5s
            let pre_speech_buffer_frames = 10; // Keep ~300ms of audio before trigger
            
            // We process in 30ms chunks for VAD
            let chunk_size_in = (sample_rate as usize * VAD_CHUNK_MS) / 1000; 
            // 30ms at 16kHz = 480 samples
            let vad_chunk_size = (WHISPER_SAMPLE_RATE as usize * VAD_CHUNK_MS) / 1000;
            
            let mut buffer_in: Vec<f32> = Vec::with_capacity(chunk_size_in);
            let mut pre_buffer: std::collections::VecDeque<Vec<f32>> = std::collections::VecDeque::with_capacity(pre_speech_buffer_frames);

            while is_running_clone.load(Ordering::Relaxed) {
                // Read from ringbuffer
                if consumer.len() >= chunk_size_in {
                    for _ in 0..chunk_size_in {
                        if let Some(s) = consumer.pop() {
                            buffer_in.push(s);
                        }
                    }
                    
                    // Downmix to mono
                    let mono_chunk: Vec<f32> = if channels > 1 {
                        buffer_in.chunks(channels as usize)
                            .map(|frame| frame.iter().sum::<f32>() / channels as f32)
                            .collect()
                    } else {
                        buffer_in.clone()
                    };
                    buffer_in.clear();

                    // Resample to 16kHz
                    let resampled_chunk: Vec<f32> = if sample_rate != WHISPER_SAMPLE_RATE {
                        // Naive Linear interpolation (placeholder for rubato)
                        let ratio = sample_rate as f32 / WHISPER_SAMPLE_RATE as f32;
                        let output_len = (mono_chunk.len() as f32 / ratio).ceil() as usize;
                        let mut out = Vec::with_capacity(output_len);
                        for i in 0..output_len {
                            let src_idx = i as f32 * ratio;
                            let idx0 = src_idx.floor() as usize;
                            let idx1 = (idx0 + 1).min(mono_chunk.len() - 1);
                            let t = src_idx - idx0 as f32;
                            let val = mono_chunk[idx0] * (1.0 - t) + mono_chunk[idx1] * t;
                            out.push(val);
                        }
                        
                        // Ensure we strictly match VAD chunk size requirements (480 samples)
                        // This naive resample might result in +/- 1 sample diff, so we truncate or pad
                        if out.len() > vad_chunk_size {
                            out[..vad_chunk_size].to_vec()
                        } else if out.len() < vad_chunk_size {
                            // padding (bad for audio but prevents crash)
                            let mut padded = out;
                            padded.resize(vad_chunk_size, 0.0);
                            padded
                        } else {
                            out
                        }
                    } else {
                        mono_chunk
                    };
                    
                    // Convert to i16 for WebRTC VAD
                    let vad_samples: Vec<i16> = resampled_chunk.iter()
                        .map(|&s| (s.clamp(-1.0, 1.0) * 32767.0) as i16)
                        .collect();

                    // Check VAD
                    let is_voice = match vad.is_voice_segment(&vad_samples) {
                        Ok(v) => v,
                        Err(_) => {
                            // If VAD fails (usually wrong buffer size), treat as silence
                            warn!("VAD error (likely wrong buffer size)");
                            false
                        }
                    };
                    
                    // State Machine
                    if is_voice {
                        if !is_speaking {
                            debug!("Voice detected!");
                            is_speaking = true;
                            // Add pre-buffer to capture the start of the word
                            for chunk in pre_buffer.drain(..) {
                                speech_buffer.extend(chunk);
                            }
                        }
                        silence_frames = 0;
                        speech_buffer.extend(&resampled_chunk);
                    } else {
                        // Silence
                        if is_speaking {
                            silence_frames += 1;
                            // Keep recording silence for a bit (natural pauses)
                            speech_buffer.extend(&resampled_chunk);

                            let silence_ms = silence_frames * VAD_CHUNK_MS;
                            if silence_ms > silence_limit_ms {
                                debug!("End of utterance ({}ms silence)", silence_ms);
                                
                                // Calculate total duration
                                let total_duration_ms = (speech_buffer.len() as f32 / WHISPER_SAMPLE_RATE as f32) * 1000.0;
                                
                                // Trim the trailing silence (keep ~200ms)
                                let trim_amount = ((silence_limit_ms - 200) as f32 * WHISPER_SAMPLE_RATE as f32 / 1000.0) as usize;
                                let valid_len = speech_buffer.len().saturating_sub(trim_amount);
                                let final_buffer = speech_buffer[..valid_len].to_vec();

                                if total_duration_ms > min_speech_ms as f32 {
                                    info!("Sending speech segment: {:.2}s", total_duration_ms / 1000.0);
                                    if let Err(e) = segment_tx.blocking_send(AudioSegment { samples: final_buffer }) {
                                        error!("Failed to send audio segment: {}", e);
                                        break;
                                    }
                                } else {
                                    debug!("Ignored short noise ({:.2}ms)", total_duration_ms);
                                }
                                
                                speech_buffer.clear();
                                is_speaking = false;
                                silence_frames = 0;
                            }
                        } else {
                            // Not speaking, maintain circular pre-buffer
                            pre_buffer.push_back(resampled_chunk);
                            if pre_buffer.len() > pre_speech_buffer_frames {
                                pre_buffer.pop_front();
                            }
                        }
                    }
                    
                    // Max duration safety cut (e.g., 30 seconds)
                    if speech_buffer.len() > WHISPER_SAMPLE_RATE as usize * 30 {
                         warn!("Max duration reached, cutting.");
                         let final_buffer = speech_buffer.clone();
                         let _ = segment_tx.blocking_send(AudioSegment { samples: final_buffer });
                         speech_buffer.clear();
                         is_speaking = false;
                         silence_frames = 0;
                    }
                    
                } else {
                    // Sleep briefly to let buffer fill
                    std::thread::sleep(std::time::Duration::from_millis(5));
                }
            }
        });

        Ok(Self {
            stream,
            is_running,
            segment_rx,
        })
    }
    
    pub async fn next_segment(&mut self) -> Option<AudioSegment> {
        self.segment_rx.recv().await
    }
}

impl Drop for AudioEngine {
    fn drop(&mut self) {
        self.is_running.store(false, Ordering::Relaxed);
    }
}
