/** TypeScript types matching the Rust backend structures */

/** Matches Rust Config struct from config.rs */
export interface Config {
  // Audio settings
  audio_device: string;

  // Pipeline binaries
  whisper_bin: string;
  claude_bin: string;

  // Home Assistant settings
  ha_enabled: boolean;
  ha_entity: string;
  ha_url: string;
  ha_tts_entity: string;
  hass_token: string;

  // Recording settings
  record_duration: number;
  silence_threshold: string;
  silence_duration: number;

  // Whisper settings
  whisper_model: string;
  whisper_language: string;

  // Directories
  tmp_dir: string;
  control_fifo: string;

  // App-specific settings
  hotkey_send: string;
  hotkey_dictate: string;
  launch_at_login: boolean;
}

/** Matches Rust Mode enum from state.rs */
export type Mode = "VoiceToClaude" | "Dictation" | "Combined";

/** Matches Rust AppState enum from state.rs */
export type AppState =
  | { type: "Idle" }
  | { type: "Active"; mode: Mode }
  | { type: "Recording"; mode: Mode };
